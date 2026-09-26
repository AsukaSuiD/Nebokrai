//! Данные живой области CPoisonFogPhalanx, её форма и тело Summon CPoisonFog.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/poisonfogphalanx.cpp/.h` и `appserver/skills/poisonfog.cpp`.
//! Конструктор VA 0x005FBD90, AI VA 0x005FC040. Композит `CPoisonFogPhalanx`
//! (CShape + область) следует старому адаптеру без новых машинных оснований.
//! Summon захватывает полный U, Master с country0 и уровень оружия Player
//! либо 0, затем свежую таблицу: порядок запросов `ER_COEFF(224)`→
//! `ER_LOSS(212)`→`DODGE_LOSS(210)`→`DEF_COEFF(223)`→`DEF_LOSS(209)`→
//! `PERSIST(10002)`→уровень→`LIFETIME(30001)` предшествует конструктору с
//! часами и ID. SetTile выполняется до свежего actual region U; перекрытие
//! прежних C9, AddShape и encode/BF502 остаются швами-фасадами
//! (`ZonalCastContact` в `skills/zonalcast.rs`): они обязательны даже при
//! отказе Add. Состояния области не принадлежат касту.

use nebokrai_shared::values::CGuid;
use crate::combat::MasterInfo;
use crate::effects::{PoisonFogState, POISON_FOG_STATE_ID};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::zonalcast::{ZonalCastContact, ZonalCastMoveShape};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoisonFogPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoisonFogPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    state_template: PoisonFogState,
    cell_active: bool,
}

impl PoisonFogPhalanx {
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        state_template: PoisonFogState,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, state_template, cell_active: true }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.state_template.skill_level() }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn cell_active(&self) -> bool { self.cell_active }

    pub fn clear_cell(&mut self) { self.cell_active = false; }

    pub const fn tick(&self, now_ms: u32) -> PoisonFogPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            PoisonFogPhalanxTick::Expired
        } else {
            PoisonFogPhalanxTick::Scan
        }
    }

    pub fn state(&self) -> PoisonFogState { self.state_template.clone() }
}

const STATE_TIME_PROPERTY: u32 = 10_002;
const DEF_LOSS_PROPERTY: u32 = 209;
const DODGE_LOSS_PROPERTY: u32 = 210;
const ELEMENT_LOSS_PROPERTY: u32 = 212;
const DEF_COEFFICIENT_PROPERTY: u32 = 223;
const ER_COEFFICIENT_PROPERTY: u32 = 224;
const LIFETIME_PROPERTY: u32 = 30_001;

/// Живая форма области ядовитого тумана: связка CShape и области правил.
/// Все маски RU-варианта равны одной активной клетке, поэтому вместо CScope
/// хранится один бит; перекрытие очищает маску, но не сокращает срок жизни.
/// Состав и порядок соответствуют адаптеру старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CPoisonFogPhalanx {
    shape: CShape,
    area: PoisonFogPhalanx,
}

impl CPoisonFogPhalanx {
    #[allow(clippy::too_many_arguments, reason = "независимые параметры области и ослабления цели")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        state_keep_time_ms: u32,
        defense_loss: u32,
        defense_loss_coefficient: u32,
        dodge_loss: u32,
        element_resistance_loss: u32,
        element_resistance_loss_coefficient: u32,
        weapon_damage_level: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        let state = PoisonFogState::new(
            skill_level, state_keep_time_ms, defense_loss, defense_loss_coefficient,
            dodge_loss, element_resistance_loss, element_resistance_loss_coefficient,
            weapon_damage_level,
        );
        Self { shape, area: PoisonFogPhalanx::new(master, started_at_ms, lifetime_ms, state) }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.area.master() }
    pub const fn skill_level(&self) -> i32 { self.area.skill_level() }
    pub const fn cell_active(&self) -> bool { self.area.cell_active() }

    pub fn set_center(&mut self, x: i32, y: i32) {
        let y = (f64::from(y) + 0.5) as f32;
        let x = (f64::from(x) + 0.5) as f32;
        self.shape.set_pos_xy_base(x, y);
    }

    pub fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) {
            self.area.clear_cell();
        }
    }

    /// Истечение отмечает удаление формы; ветвь AI выбирает считыватель.
    pub fn tick(&mut self, now_ms: u32) -> PoisonFogPhalanxTick {
        let tick = self.area.tick(now_ms);
        if tick == PoisonFogPhalanxTick::Expired {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
        }
        tick
    }

    pub fn state(&self) -> PoisonFogState { self.area.state() }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, POISON_FOG_STATE_ID as i32, self.area.skill_level(),
            self.area.master().master_type, self.area.master().master_id,
            self.area.started_at_ms(), self.area.lifetime_ms(), now,
        )
    }
}

/// Тело `CPoisonFog::Summon`: захват полного U, Master(country0), уровень
/// оружия Player либо 0 до свежей таблицы; запросы в прежнем порядке;
/// SetTile до свежего actual region U; Add и encode/BF502 обязательны даже
/// при отказе Add.
pub fn summon_poison_fog<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    destination: (i32, i32),
    runtime: &mut Runtime,
    now_milliseconds: &mut dyn FnMut(&mut Runtime) -> u32,
)
where
    Game: ZonalCastContact<Runtime>,
{
    let Some(mut master) = game.zonal_source_master(source) else { return; };
    master.master_country_id = 0;
    let weapon = if source.1.object_type == PLAYER_TYPE {
        match game.find_player(source.1.id) {
            Some(player) => game.player_weapon_damage_level(player),
            None => 0,
        }
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let er_coefficient = properties.query_property(ER_COEFFICIENT_PROPERTY);
    let element_loss = properties.query_property(ELEMENT_LOSS_PROPERTY);
    let dodge_loss = properties.query_property(DODGE_LOSS_PROPERTY);
    let def_coefficient = properties.query_property(DEF_COEFFICIENT_PROPERTY);
    let def_loss = properties.query_property(DEF_LOSS_PROPERTY);
    let state_time = properties.query_property(STATE_TIME_PROPERTY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(LIFETIME_PROPERTY);
    let started = now_milliseconds(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CPoisonFogPhalanx::new(
        id, master, started, lifetime, level, state_time, def_loss, def_coefficient,
        dodge_loss, element_loss, er_coefficient, weapon,
    );
    phalanx.set_center(destination.0, destination.1);
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    if let Some(Ok(id)) = game.add_poison_fog_phalanx(region_id, phalanx, destination.0, destination.1, started, runtime) {
        let _ = game.send_poison_fog_phalanx_entry(region_id, id, runtime);
    }
}
