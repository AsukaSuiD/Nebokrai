//! Стационарная форма громового рассечения CThunderSlashPhalanx (0x72).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/thunderslashphalanx.cpp.
//!
//! Снимок владельца и боевых свойств не обновляется после создания формы.
//! Ошибка конструктора сохранена: MIN и MAX получают переданный максимум,
//! но Calculate всё равно вызывает RNG диапазона перед RNG критического удара.
//! DWORD-коэффициент умножается на исходный 0.01_f32 без промежуточной записи
//! в float; критический множитель усекается к нулю через общий FISTP-адаптер.
//! Отсутствие таблицы свойств оставляет UNKNOWN/1 и пустой урон, но не отменяет
//! OnBeenAttacked и последующий IncreaseRp.
//!
//! AI сохраняет строгие unsigned границы срока и частоты. Третьи часы записываются
//! в живую форму до разрешения региона и единственного GetShape собственной клетки;
//! первая немобильная форма не заменяется следующей целью. End и публикацией
//! удаления управляет общий региональный runtime. Клиентский снимок использует
//! общий wire-префикс skill/level/master type/id/remained и затем CShape.
//! Клетка атаки хранится отдельно от базовой позиции: конструктор не переносит
//! её в float-координаты CShape. Add регистрирует форму из исходной клетки (0,0),
//! и именно базовая позиция определяет область видимости формы и её удаления.
//! Vec и региональная арена заменяют владение указателями без отдельного хранилища.

use super::fightdefense::truncate_original;
use super::thunderslash::THUNDER_SLASH_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, RegionShapeResolver};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CThunderSlashPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    tile_x: i32,
    tile_y: i32,
    frequency_ms: u32,
    last_attack_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_attack: i32,
    dexterity: i32,
    critical_chance: i32,
    soul_attack: i32,
}

pub(crate) fn thunder_slash_target(
    game: &CGame, region_id: i32, phalanx: &CThunderSlashPhalanx,
) -> Option<ShapeIdentity> {
    let owner = game.find_region(region_id)?;
    let resolver = RegionShapeResolver { game, owner };
    let (x, y) = phalanx.tile();
    let (width, height) = game.area_dimensions();
    let first = owner.base().get_shape(x, y, width, height, &resolver).ok()??;
    resolve_state_move_shape(game, region_id, first.identity)?;
    Some(first.identity)
}

impl CThunderSlashPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимки соответствуют аргументам конструктора EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, maximum_attack: i32,
        _minimum_attack: i32, element_attack: i32, dexterity: i32,
        critical_chance: i32, soul_attack: i32, tile_x: i32, tile_y: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, master, started_at_ms, lifetime_ms, tile_x, tile_y, frequency_ms,
            last_attack_ms: 0, skill_level, minimum_attack: maximum_attack, maximum_attack,
            element_attack, dexterity, critical_chance, soul_attack,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn tile(&self) -> (i32, i32) { (self.tile_x, self.tile_y) }

    pub(crate) const fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
            || self.tile_x == 0 || self.tile_y == 0
    }

    pub(crate) const fn attack_due_at(&self, now_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    pub(crate) fn mark_attack_at(&mut self, now_ms: u32) {
        self.last_attack_ms = now_ms;
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, THUNDER_SLASH_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }

    fn attack_master(&self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

pub(crate) fn calculate_owned_thunder_slash_attack(
    game: &mut CGame, phalanx: &CThunderSlashPhalanx,
) -> AttackInformation {
    let mut attack = AttackInformation::for_master(phalanx.attack_master());
    let Some(properties) = game.skill_base_properties(THUNDER_SLASH_SKILL_ID, phalanx.skill_level) else {
        return attack;
    };
    attack.skill_id = THUNDER_SLASH_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = (f64::from(properties.query_property(20_003))
        * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(20_001) as i32;

    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let mut physical = game.skill_random_below(width).wrapping_add(phalanx.minimum_attack);
    if attack.attacker_type == 400 {
        physical = physical.wrapping_add(phalanx.dexterity);
    }
    attack.damages = vec![
        AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Element, hp_damage: phalanx.element_attack.max(0), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Soul, hp_damage: phalanx.soul_attack.max(0), mp_damage: 0 },
    ];
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    attack
}

pub(crate) fn apply_thunder_slash_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, phalanx: &CThunderSlashPhalanx,
    region_id: i32, target: ShapeIdentity, runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, region_id, target).is_none()
        || game.base_magic_target_dead(region_id, target)
    {
        return;
    }
    // FindPlayer вызывается по одному ID даже для снимка иного типа.
    let source = game.find_player(phalanx.master.master_id)
        .map(|player| player.shape().identity());
    if let Some(source) = source
        && !game.live_skill_target_attackable(region_id, source, target)
    {
        return;
    }
    let attack = calculate_owned_thunder_slash_attack(game, phalanx);
    game.apply_owned_skill_contact(phalanx.attack_master(), target, region_id, attack, runtime);
    if let Some(source) = source {
        game.increase_owned_player_rp(source.id, true, 0);
    }
}
