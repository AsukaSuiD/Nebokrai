//! Движущаяся форма световой стрелы `CLightingArrowPhalanx` (`0xCB`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lightingarrowphalanx.cpp`. Форма проходит путь по одной
//! клетке за проход ИИ, останавливается перед первым блоком `2` и запоминает
//! уже атакованные цели в порядке первого контакта. Яд оружия применяется до
//! расчёта урона. Формула сохраняет два вызова генератора MSVCRT; `CGame`
//! разрешает региональные identity и применяет рассчитанный результат.

use super::lightingarrow::LIGHTING_ARROW_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LightingArrowPhalanxTick {
    Pending,
    Active { force_move: Option<(i32, i32, u32)>, cell: Option<(i32, i32, u32)> },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CLightingArrowPhalanx {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    skill_level: i32, hit_modifier: i32, damage_factor_percent: u32,
    path: Vec<(i32, i32, u8)>, speed_ms: u32, current_cell: usize,
    attack_cell_count: usize, destination: (i32, i32), force_moved: bool,
    attacked: Vec<ShapeIdentity>,
}

pub(crate) fn lighting_arrow_targets(game: &CGame, region_id: i32, phalanx: &CLightingArrowPhalanx, tile_x: i32, tile_y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (width, height) = game.area_dimensions();
    let mut shapes = Vec::new();
    if region.get_shapes(tile_x, tile_y, width, height, game, &mut shapes).is_err() { return Vec::new() }
    shapes.into_iter().map(|view| view.identity).filter(|identity| {
        *identity != phalanx.shape().identity()
            && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
            && matches!(identity.object_type, 400 | 600)
            && !phalanx.was_attacked(*identity)
            && match identity.object_type {
                400 => game.find_player(identity.id).is_some_and(|player| !player.is_dead())
                    && (phalanx.master().master_type != 400 || game.player_base_attackable(phalanx.master().master_id, identity.id)),
                600 => game.lighting_arrow_monster_attackable(region_id, phalanx.master(), identity.id),
                _ => false,
            }
    }).collect()
}

impl CLightingArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, hit_modifier: i32, damage_factor_percent: u32,
        path: Vec<(i32, i32, u8)>, speed_ms: u32) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        let attack_cell_count = path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len());
        let destination = if attack_cell_count < path.len() { (path[attack_cell_count].0, path[attack_cell_count].1) }
            else { path.last().map_or((0, 0), |cell| (cell.0, cell.1)) };
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, hit_modifier,
            damage_factor_percent, path, speed_ms, current_cell: 0, attack_cell_count,
            destination, force_moved: false, attacked: Vec::new() }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn hit_modifier(&self) -> i32 { self.hit_modifier }
    pub(crate) const fn damage_factor_percent(&self) -> u32 { self.damage_factor_percent }
    pub(crate) fn was_attacked(&self, identity: ShapeIdentity) -> bool { self.attacked.contains(&identity) }
    pub(crate) fn mark_attacked(&mut self, identity: ShapeIdentity) -> bool {
        if self.was_attacked(identity) { false } else { self.attacked.push(identity); true }
    }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, attack_now_ms: impl FnOnce() -> u32) -> LightingArrowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms || self.path.is_empty() {
            self.finish(); return LightingArrowPhalanxTick::Expired;
        }
        let now_ms = attack_now_ms();
        // Поздний owner проверяет завершение только после `Attack`: если
        // первая клетка уже имеет блок `2`, она всё равно получает один проход.
        let may_attack = self.current_cell < self.attack_cell_count
            || (self.attack_cell_count == 0 && self.current_cell == 0);
        let cell = if may_attack && self.current_cell < self.path.len()
            && self.started_at_ms.wrapping_add(self.speed_ms.wrapping_mul(self.current_cell as u32)) <= now_ms {
            let cell = self.path[self.current_cell]; self.current_cell = self.current_cell.wrapping_add(1);
            Some((cell.0, cell.1, now_ms))
        } else { None };
        if self.current_cell >= self.attack_cell_count {
            self.finish();
            return LightingArrowPhalanxTick::Active { force_move: None, cell };
        }
        let force_move = if self.force_moved { None } else {
            self.force_moved = true;
            Some((self.destination.0, self.destination.1, self.speed_ms.wrapping_mul(self.attack_cell_count as u32)))
        };
        if force_move.is_some() || cell.is_some() { LightingArrowPhalanxTick::Active { force_move, cell } }
        else { LightingArrowPhalanxTick::Pending }
    }
    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 }
            else { self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms) };
        let mut payload = Vec::new();
        { let mut writer = LegacyWriter::new(&mut payload); writer.write_i32(LIGHTING_ARROW_SKILL_ID as i32);
          writer.write_i32(self.skill_level); writer.write_i32(self.master.master_type);
          writer.write_i32(self.master.master_id); writer.write_u32(remained); }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_lighting_arrow_attack(game: &mut CGame, phalanx: &CLightingArrowPhalanx,
    target_level: u8) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?;
    let mut combat = player.combat_properties(); let occupation = player.occupation(); let attacker_level = player.level();
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor);
    let damage_factor = weapon_factor * phalanx.damage_factor_percent() as f32 * 0.01;
    let minimum = combat.minimum_attack as i32; let maximum = combat.maximum_attack as i32;
    let delta = maximum.wrapping_sub(minimum); let width = if delta < 0 { delta.wrapping_neg() } else { delta }.wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(width)); let master = phalanx.master();
    let mut attack = AttackInformation { skill_id: LIGHTING_ARROW_SKILL_ID, skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type, attacker_id: master.master_id, attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: phalanx.hit_modifier(), damage_factor, damage_modifier: 0, critical: false,
        blast_attack: false, full_miss: 0, damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate).round_ties_even() as i32; } }
    let [ba, bd, eba, ebd, fm] = game.globe_setup().base_combat_scales();
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = ba.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = bd.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = eba.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = ebd.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = fm.max(0.01).to_bits(); }
    if combat.critical_rate() < 1.0 { combat.critical_rate_bits = game.globe_setup().critical_rate().max(1.0).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}

// Оставшиеся контракты: UNKNOWN; декомпилят хранится локально. Декодирование серверного снимка не имеет
// достигнутого caller-а; его нельзя объявлять заменённым одним клиентским encoder-ом.
//
// FUNCTION: CLightingArrowPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lightingarrowphalanx.cpp:368
// RVA: 0x001FA8C0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
