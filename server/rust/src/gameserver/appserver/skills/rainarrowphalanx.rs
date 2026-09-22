//! Трёхлучевая форма дождя стрел CRainArrowPhalanx (0xCE).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/rainarrowphalanx.cpp.
//! Три независимых пути владеют своими клетками и активной длиной. Конструктор
//! сохраняет исходные длины, скорость и дальность; первый индекс равен единице.
//!
//! AI при истёкшем сроке вызывает общий End и возвращает управление.
//! Превышение дальности тоже вызывает End, но затем этот же AI продолжается.
//! После вторых часов обрабатываются центр, правый и левый луч; только центр
//! использует индекс, сохранённый до часов, остальные читают его после
//! предыдущих callbacks. Общий индекс увеличивается после всех трёх лучей.
//! Остановка одного луча не меняет остальные пути и не завершает всю форму.
//!
//! Attack(cell) сохраняет список GetShapes, проверяет полный CMoveShape,
//! исключает master по type/id и для player-master требует живого FindPlayer
//! и ненулевой допуск цели. Все допущенные цели обрабатываются без дедупликации.
//! Даже мёртвая цель, отклонённая внутри Attack(target), останавливает луч.
//! PK-поля формы применяются только к master типа 400; сырой контакт не даёт RP.
//!
//! Calculate ищет CPlayer по master ID независимо от master type. Сохранённый
//! знаковый процент умножается на weapon modifier и 0.01_f32 до записи float;
//! общий оружейный хвост читает MAX→MIN→RNG→MIN, ELEMENT/SOUL/CCH и второй RNG.
//! Живые боевые свойства не заменяются снимком конструктора. Исчезнувший игрок
//! при ненулевой цели даёт native null-deref (0x005FA2D1); безопасный адаптер
//! оставляет исходный UNKNOWN/1 без выдуманных свойств и другого источника.
//!
//! Wire содержит skill/level/master type/id/remained и CShape, но не пути.
//! Vec заменяет три массива; маленький Copy-снимок атаки не копирует пути.
//! Форма остаётся в региональной арене во время callbacks; End и публикация
//! используют общий summon-механизм. Неиспользуемый server decode сохранён ниже.

use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_shared::values::CGuid;

pub(crate) const RAIN_ARROW_SKILL_ID: u32 = 0xce;
pub(crate) type RainArrowCell = (i32, i32, u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RainArrowBeam { Center, Right, Left }

impl RainArrowBeam {
    const fn index(self) -> usize {
        match self { Self::Center => 0, Self::Right => 1, Self::Left => 2 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RainArrowPath {
    pub(super) cells: Vec<RainArrowCell>,
    pub(super) active_cells: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RainArrowAttack {
    master: MasterInfo,
    skill_level: i32,
    hit_modifier: i32,
    damage_factor_percent: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CRainArrowPhalanx {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack: RainArrowAttack,
    beams: [RainArrowPath; 3],
    current_step: u32,
    speed_ms: i32,
    maximum_distance: i32,
}

impl CRainArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "три независимых пути и снимки соответствуют конструктору EXE")]
    pub(super) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, hit_modifier: i32, damage_factor_percent: i32,
        left: RainArrowPath, center: RainArrowPath, right: RainArrowPath,
        speed_ms: i32, maximum_distance: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms, lifetime_ms,
            attack: RainArrowAttack { master, skill_level, hit_modifier, damage_factor_percent },
            beams: [center, right, left],
            current_step: 1, speed_ms, maximum_distance,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> RainArrowAttack { self.attack }
    pub(crate) const fn current_step(&self) -> u32 { self.current_step }

    pub(crate) const fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) const fn maximum_distance_exceeded(&self) -> bool {
        (self.maximum_distance as u32) < self.current_step
    }

    pub(crate) const fn attack_due_at(&self, step: u32, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add((self.speed_ms as u32).wrapping_mul(step)) <= now_ms
    }

    pub(crate) fn beam_cell(&self, beam: RainArrowBeam, step: u32) -> Option<(i32, i32)> {
        let path = &self.beams[beam.index()];
        if step >= path.active_cells.wrapping_add(1) { return None; }
        // Неверная native длина выходит за массив. Безопасная граница не
        // меняет опубликованную длину и не подставляет клетку другого луча.
        let cell = path.cells.get(step.wrapping_sub(1) as usize)?;
        Some((cell.0, cell.1))
    }

    pub(crate) fn stop_beam(&mut self, beam: RainArrowBeam) {
        self.beams[beam.index()].active_cells = 0;
    }

    pub(crate) fn advance_step(&mut self) {
        self.current_step = self.current_step.wrapping_add(1);
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, RAIN_ARROW_SKILL_ID as i32, self.attack.skill_level,
            self.attack.master.master_type, self.attack.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}

impl RainArrowAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

fn calculate_rain_arrow_attack(
    game: &mut CGame, snapshot: RainArrowAttack, region_id: i32,
    target: ShapeIdentity, attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    attack.skill_id = RAIN_ARROW_SKILL_ID;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(region_id, target) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    attack.damage_factor = (f64::from(weapon_factor)
        * f64::from(snapshot.damage_factor_percent) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = snapshot.hit_modifier;
    fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::AbsoluteRange, attack);
}

pub(crate) fn apply_rain_arrow_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: RainArrowAttack, region_id: i32,
    target: ShapeIdentity, runtime: &mut Runtime,
) {
    if game.move_shape_health(region_id, target).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate_rain_arrow_attack(game, snapshot, region_id, target, &mut attack);
    game.apply_owned_skill_contact(master, target, region_id, attack, runtime);
}

// Server decode не имеет достигнутого caller-а. Он читает только wire-префикс
// и базовую форму, но не создаёт пути; нельзя подменять его готовой runtime-формой.
// Восстановлены имена полей префикса по смещениям EXE.
//
// FUNCTION: CRainArrowPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: appserver/skills/rainarrowphalanx.cpp:269
// RVA: 0x001F9FA0
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
