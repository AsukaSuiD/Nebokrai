//! Движущаяся форма световой стрелы CLightingArrowPhalanx (0xCB).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/lightingarrowphalanx.cpp.
//!
//! Конструктор получает общий clock→ID и сохраняет путь; конечная клетка
//! сначала (0,0). Истёкший срок или пустой путь завершают форму. Иначе AI
//! после первых часов каждый раз ищет
//! первый блок 2, записывает границу/назначение, сохраняет индекс и читает
//! вторые часы. За AI атакуется не более одной клетки; текущий индекс
//! увеличивается свежим чтением ПОСЛЕ Attack. Затем завершение вызывает общий
//! End либо выполняется единственный ForceMove; флаг пишется после его возврата.
//! При блоке 2 в первой клетке она всё равно атакуется до проверки завершения.
//!
//! Attack(cell) сохраняет исходный список клетки, но проверяет
//! живой CMoveShape, дедупликацию и player-допуск перед каждым контактом.
//! Attack(target) проверяет IsDied, повторно проверяет/публикует
//! дедупликацию и только затем создаёт AttackInformation. PK-снимок формы
//! остаётся исходным; для непользовательского master его дополнительные поля
//! нулевые. Общий яд DaubPoison применяется до свежего Calculate и сырого
//! OnBeenAttacked; RP не начисляется.
//!
//! Calculate ищет CPlayer по одному master ID, даже для другого
//! типа master. Сохранённый процент читается FIMUL dword как i32; произведение
//! weapon modifier и 0.01_f32 записывается в float лишь в конце. Общий оружейный
//! расчёт сохраняет MAX→MIN→RNG→MIN, ELEMENT/SOUL/CCH и второй RNG без ловкости.
//! Исчезнувший игрок при живой цели — native null-deref (0x005FAE11/5FAE31),
//! а не поддержанный non-player расчёт: безопасный адаптер оставляет исходный
//! пустой UNKNOWN/1, не изобретая боевые свойства или замену источника.
//!
//! Общий wire-префикс skill/level/master type/id/remained предшествует CShape.
//! Vec и региональная арена заменяют указатели; ключи дедупликации различают
//! региональных владельцев одинакового ID, но не привязывают CPlayer к региону.
//! Региональный runtime сохраняет публикацию формы во всех callbacks и общий
//! ForceMove/End. Неиспользуемый server decode ещё требует реконструкции; ниже сохранены адресные метаданные.

use super::heartlessarrow::apply_daub_poison;
use super::lightingarrow::LIGHTING_ARROW_SKILL_ID;
use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, RegionShapeResolver};
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ArrowTargetIdentity {
    region_id: i32,
    identity: ShapeIdentity,
}

impl ArrowTargetIdentity {
    pub(super) const fn new(region_id: i32, identity: ShapeIdentity) -> Self {
        Self { region_id: if identity.object_type == 400 { 0 } else { region_id }, identity }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CLightingArrowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    hit_modifier: i32,
    damage_factor_percent: u32,
    path: Vec<(i32, i32, u8)>,
    speed_ms: u32,
    current_cell: u32,
    attack_cell_count: u32,
    destination: (i32, i32),
    force_moved: bool,
    attacked: Vec<ArrowTargetIdentity>,
}

pub(crate) fn lighting_arrow_targets(
    game: &CGame, region_id: i32, tile_x: i32, tile_y: i32,
) -> Vec<ShapeIdentity> {
    let Some(owner) = game.find_region(region_id) else { return Vec::new(); };
    let resolver = RegionShapeResolver { game, owner };
    let (width, height) = game.area_dimensions();
    let mut shapes = Vec::new();
    if owner.base().get_shapes(tile_x, tile_y, width, height, &resolver, &mut shapes).is_err() {
        return Vec::new();
    }
    shapes.into_iter().map(|view| view.identity).collect()
}

impl CLightingArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля соответствуют аргументам конструктора EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, hit_modifier: i32, damage_factor_percent: u32,
        path: Vec<(i32, i32, u8)>, speed_ms: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level, hit_modifier,
            damage_factor_percent, path, speed_ms, current_cell: 0, attack_cell_count: 0,
            destination: (0, 0), force_moved: false, attacked: Vec::new(),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn was_attacked(&self, region_id: i32, identity: ShapeIdentity) -> bool {
        self.attacked.contains(&ArrowTargetIdentity::new(region_id, identity))
    }

    pub(crate) fn mark_attacked(&mut self, region_id: i32, identity: ShapeIdentity) -> bool {
        if self.was_attacked(region_id, identity) { return false; }
        self.attacked.push(ArrowTargetIdentity::new(region_id, identity));
        true
    }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms || self.path.is_empty()
    }

    pub(crate) fn prepare_attack_cells(&mut self) -> u32 {
        let count = self.path.iter().position(|cell| cell.2 == 2).unwrap_or(self.path.len());
        self.attack_cell_count = count as u32;
        if let Some(cell) = self.path.get(count).or_else(|| self.path.last()) {
            self.destination = (cell.0, cell.1);
        }
        self.current_cell
    }

    pub(crate) fn due_attack_cell(&self, cell_index: u32, now_ms: u32) -> Option<(i32, i32)> {
        if self.started_at_ms.wrapping_add(self.speed_ms.wrapping_mul(cell_index)) > now_ms {
            return None;
        }
        let cell = self.path.get(cell_index as usize)?;
        Some((cell.0, cell.1))
    }

    pub(crate) fn advance_attack_cell(&mut self) {
        self.current_cell = self.current_cell.wrapping_add(1);
    }

    pub(crate) const fn attack_cells_finished(&self) -> bool {
        self.attack_cell_count <= self.current_cell
    }

    pub(crate) fn force_move_destination(&self) -> Option<(i32, i32, u32)> {
        (!self.force_moved).then_some((
            self.destination.0, self.destination.1,
            self.speed_ms.wrapping_mul(self.attack_cell_count),
        ))
    }

    pub(crate) fn mark_force_moved(&mut self) { self.force_moved = true; }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, LIGHTING_ARROW_SKILL_ID as i32, self.skill_level,
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

fn calculate_lighting_arrow_attack(
    game: &mut CGame, phalanx: &CLightingArrowPhalanx,
    region_id: i32, target: ShapeIdentity, attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    attack.skill_id = LIGHTING_ARROW_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(region_id, target) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    attack.damage_factor = (f64::from(weapon_factor)
        * f64::from(phalanx.damage_factor_percent as i32) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = phalanx.hit_modifier;
    fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::AbsoluteRange, attack);
}

/// Вызывается после IsDied и публикации повторной дедупликации живой формы.
pub(crate) fn apply_lighting_arrow_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, phalanx: &CLightingArrowPhalanx,
    region_id: i32, target: ShapeIdentity, runtime: &mut Runtime,
) {
    let master = phalanx.attack_master();
    let mut attack = AttackInformation::for_master(master);
    apply_daub_poison(game, attack.attacker_id, region_id, target, &mut || runtime.now_milliseconds());
    calculate_lighting_arrow_attack(game, phalanx, region_id, target, &mut attack);
    game.apply_owned_skill_contact(master, target, region_id, attack, runtime);
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
