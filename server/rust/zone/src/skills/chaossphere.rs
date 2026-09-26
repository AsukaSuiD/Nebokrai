//! Движущаяся область CChaosSpherePhalanx и её живая форма.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! идентификаторы —
//! docs/reconstruction/gameserver-skills.md#идентификаторы-сборки.
//! Summon VA 0x005A8290, ctor VA 0x005FEE80, AddToByteArray VA 0x005FECB0,
//! AI VA 0x005FF270 (appserver/skills/chaossphere.cpp и chaosspherephalanx.cpp/.h).
//! Композит `CChaosSpherePhalanx` (CShape + область) перенесён буквально из
//! старого адаптера; новых машинных оснований он не добавляет. Тело
//! `summon_chaos_sphere` тоже перенесено буквально
//! (ветви и порядок запросов сверены ниже и в
//! `docs/gameplay/skills.md`): `chaos_sphere_path_length` после свежей
//! таблицы; исходный пустой путь прекращает Summon, обрезанный по BLOCK2
//! до пустого всё ещё допускает форму; затем Master(country0)/Player EM→
//! параметры `ChaosSphereSummonParameters::read`→ctor(clock→ID). SetTile
//! использует path[0] либо свежие captured U Y/X, не аргументы AI. После
//! свежего actual region U выполняются Add→encode/BF502 даже при отказе
//! (швы `ZonalCastContact` в `skills/zonalcast.rs`). Путь принадлежит Vec
//! формы; отдельного payload или condition рядом с kernel нет.

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::values::CGuid;
use crate::combat::{MasterInfo, truncate_original};
use crate::effects::timed_client_state_time;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;
use super::summonshape::SUMMON_SHAPE_TYPE;
use super::zonalcast::{ZonalCastContact, ZonalCastMoveShape, ZonalCastPlayer};
use super::{ElementPhalanxAttack, ElementSummonLiveField};

pub const CHAOS_SPHERE_SKILL_ID: u32 = 0x137;
const SPEED_PROPERTY: u32 = 30_002;
const LIFETIME_PROPERTY: u32 = 30_001;
const ELEMENT_SCALE_PROPERTY: u32 = 20_015;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const FREQUENCY_PROPERTY: u32 = 6_001;

/// Читает SPEED дважды только в ненулевой ветке, затем LIFETIME.
pub fn chaos_sphere_path_length(mut query_property: impl FnMut(u32) -> u32) -> u32 {
    if query_property(SPEED_PROPERTY) == 0 { return 0; }
    let speed = query_property(SPEED_PROPERTY);
    let lifetime = query_property(LIFETIME_PROPERTY);
    lifetime / speed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChaosSphereSummonParameters {
    pub skill_level: i32,
    pub critical_chance: i32,
    pub speed_ms: u32,
    pub element_attack: i32,
    pub maximum_attack: i32,
    pub minimum_attack: i32,
    pub frequency_ms: u32,
    pub lifetime_ms: u32,
}

impl ChaosSphereSummonParameters {
    /// Вызывается после построения и обрезки пути и чтения живого модификатора элемента.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(ElementSummonLiveField) -> Option<i32>,
        current_level: impl FnOnce() -> Option<i32>,
        element_modifier: i32,
    ) -> Option<Self> {
        let scale = query_property(ELEMENT_SCALE_PROPERTY);
        let scaled_element = truncate_original(
            f64::from(scale) * f64::from(0.01_f32) * f64::from(element_modifier),
        );
        let critical_chance = (read_live(ElementSummonLiveField::CriticalChance)? as u16) as i32;
        let speed_ms = query_property(SPEED_PROPERTY);
        let element_attack = read_live(ElementSummonLiveField::AddElementAttack)?
            .wrapping_add(scaled_element);
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let frequency_ms = query_property(FREQUENCY_PROPERTY);
        let skill_level = current_level()?;
        let lifetime_ms = query_property(LIFETIME_PROPERTY);
        Some(Self { skill_level, critical_chance, speed_ms, element_attack,
            maximum_attack, minimum_attack, frequency_ms, lifetime_ms })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChaosSpherePhalanx {
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    last_attack_ms: u32,
    last_move_ms: u32,
    current_cell: u32,
    force_moved: bool,
}

impl ChaosSpherePhalanx {
    pub fn new(
        attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        frequency_ms: u32, path: Vec<(i32, i32)>, speed_ms: u32,
    ) -> Self {
        Self { attack, started_at_ms, lifetime_ms, frequency_ms, path, speed_ms,
            last_attack_ms: 0, last_move_ms: 0, current_cell: 0, force_moved: false }
    }

    pub const fn master(&self) -> crate::combat::MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub fn has_path(&self) -> bool { !self.path.is_empty() }
    pub fn initial_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }
    pub fn mark_force_moved_at(&mut self, now: u32) {
        self.last_move_ms = now;
        self.force_moved = true;
    }
    pub const fn movement_due_at(&self, now: u32) -> bool {
        self.speed_ms.wrapping_add(self.last_move_ms) < now
    }
    pub fn advance_at(&mut self, now: u32) {
        self.last_move_ms = now;
        if self.current_cell < (self.path.len() as u32).wrapping_sub(1) {
            self.current_cell = self.current_cell.wrapping_add(1);
        }
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }
    pub fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub fn attack_origin(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_cell as usize)
            .map(|&(x, y)| (x.wrapping_sub(1), y.wrapping_sub(1)))
    }

    /// Пять полей перед базовым CShape; путь и часы периодов не сериализуются.
    pub fn write_client_snapshot_fields(&self, payload: &mut Vec<u8>, now: impl FnMut() -> u32) {
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(self.attack.skill_id);
        writer.write_i32(self.attack.skill_level);
        writer.write_i32(self.attack.master.master_type);
        writer.write_i32(self.attack.master.master_id);
        writer.write_u32(timed_client_state_time(self.started_at_ms, self.lifetime_ms, now));
    }
}

/// Живая форма сферы хаоса: связка CShape и движущейся области; скорость
/// кладётся в форму, остальное читают область и клиентский конверт.
/// Состав и порядок соответствуют адаптеру старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CChaosSpherePhalanx {
    shape: CShape,
    area: ChaosSpherePhalanx,
}

impl CChaosSpherePhalanx {
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32,
        parameters: ChaosSphereSummonParameters, path: Vec<(i32, i32)>,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        shape.set_speed((parameters.speed_ms as i32) as f32);
        let attack = ElementPhalanxAttack {
            master, skill_id: CHAOS_SPHERE_SKILL_ID, skill_level: parameters.skill_level,
            minimum: parameters.minimum_attack, maximum: parameters.maximum_attack,
            element: parameters.element_attack, critical_chance: parameters.critical_chance,
        };
        let area = ChaosSpherePhalanx::new(
            attack, started_at_ms, parameters.lifetime_ms,
            parameters.frequency_ms, path, parameters.speed_ms,
        );
        Self { shape, area }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.area.master() }
    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.area.attack_snapshot() }
    pub const fn expired_at(&self, now: u32) -> bool { self.area.expired_at(now) }
    pub fn has_path(&self) -> bool { self.area.has_path() }
    pub fn initial_force_move(&self) -> Option<(i32, i32, u32)> {
        self.area.initial_force_move()
    }
    pub fn mark_force_moved_at(&mut self, now: u32) { self.area.mark_force_moved_at(now); }
    pub const fn movement_due_at(&self, now: u32) -> bool { self.area.movement_due_at(now) }
    pub fn advance_at(&mut self, now: u32) { self.area.advance_at(now); }
    pub const fn attack_due_at(&self, now: u32) -> bool { self.area.attack_due_at(now) }
    pub fn mark_attack_at(&mut self, now: u32) { self.area.mark_attack_at(now); }
    pub fn attack_origin(&self) -> Option<(i32, i32)> { self.area.attack_origin() }
    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.area.write_client_snapshot_fields(&mut payload, now);
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

/// Тело `CChaosSphere::Summon` (ветви VA 0x005A82E8–0x005A84F3): пользователь
/// и свежая таблица, длина пути `chaos_sphere_path_length`, пустой путь
/// прекращает; identity S очищается до обрезки остатка по BLOCK2; затем
/// Master(country0)/Player EM (отсутствие игрока прерывает, как и раньше)
/// и параметры `ChaosSphereSummonParameters::read`; ctor с часами и ID;
/// SetTile — path[0] или свежие Y/X captured U; Add→encode/BF502 — швами.
/// Тело перенесено буквально.
pub fn summon_chaos_sphere<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    runtime: &mut Runtime,
    now_milliseconds: &mut dyn FnMut(&mut Runtime) -> u32,
)
where
    Game: ZonalCastContact<Runtime>,
{
    if game.resolve_state_move_shape(source.0, source.1).is_none() { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let length = chaos_sphere_path_length(|property| properties.query_property(property));
    let mut path = game.skill_target_path_with_length(skill.lifecycle(), length);
    if path.is_empty() { return; }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    if let Some(index) = path.iter().position(|cell| cell.2 == 2) { path.truncate(index); }
    let Some(mut master) = game.zonal_source_master(source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().element_modify
    } else { 0 };
    let Some(parameters) = ChaosSphereSummonParameters::read(
        |property| properties.query_property(property),
        |field| game.zonal_source_property(source, field).map(|value| value as i32),
        || game.registered_skill(instance).map(|skill| i32::from(skill.level())),
        element,
    ) else { return; };
    let started = now_milliseconds(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CChaosSpherePhalanx::new(
        id, master, started, parameters,
        path.iter().map(|&(x, y, _)| (x, y)).collect(),
    );
    let (x, y) = if let Some(&(x, y, _)) = path.first() { (x, y) } else {
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        (x, y)
    };
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
    );
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_chaos_sphere_phalanx(region, phalanx, started, runtime);
}
