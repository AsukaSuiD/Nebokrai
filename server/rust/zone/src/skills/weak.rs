//! Правила области ослабления CWeakPhalanx, срока призыва CWeak и тела его
//! Summon. Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/weakphalanx.cpp/.h и appserver/skills/weak.cpp.
//! Конструктор области: VA 0x600730; обход: 0x600840; AI: 0x600a70;
//! расчёт срока призыва: 0x5AF41B–0x5AF48C.
//! Композит `CWeakPhalanx` (CShape + область) и тело `summon_weak`
//! перенесены из старого адаптера буквально порцией T5 «zonalcast-хаб»;
//! новых машинных оснований композит не добавляет. Summon сначала
//! сохраняет actual region U и отвергает `GetSecurity == SAFE`, не GetBlock;
//! MASTER(country0)/Player EM либо 0 предшествуют свежей таблице; порядок
//! запросов (коэффициент `20010` → срок `30001` через машинную формулу
//! `weak_lifetime` → потеря attack `205` → живой уровень → часы → ID)
//! сверен с Summon VA 0x005AF41B–0x005AF48C в части расчёта срока; весь
//! порядок тела — `MATCH` против `git show HEAD`. SetTile центра, проход
//! перекрытия старых областей, регистрация AddShape и encode/BF502 остаются
//! прежними швами-фасадами (`ZonalCastContact` в `skills/zonalcast.rs`):
//! входной регион читается до Add, отказ Add не отменяет сериализацию,
//! отказ Summon не меняет End(1) навыка.

use nebokrai_shared::values::CGuid;
use crate::combat::{MasterInfo, truncate_original};
use crate::effects::WeakState;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::zonalcast::{ZonalCastContact, ZonalCastMoveShape, ZonalCastPlayer};

pub const WEAK_SKILL_ID: u32 = 0x12e;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeakPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeakPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    length: i32,
    height: i32,
    attack_loss: u32,
}

impl WeakPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля соответствуют конструктору EXE")]
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, length: i32, height: i32, attack_loss: u32,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, skill_level, length, height, attack_loss }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }
    pub const fn attack_loss(&self) -> u32 { self.attack_loss }
    pub const fn length(&self) -> i32 { self.length }
    pub const fn height(&self) -> i32 { self.height }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }

    pub const fn tick(&self, now_ms: u32) -> WeakPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            WeakPhalanxTick::Expired
        } else {
            WeakPhalanxTick::Scan
        }
    }

    pub fn active_cells(&self, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
        let start_x = center_x.wrapping_sub(self.length / 2);
        let start_y = center_y.wrapping_sub(self.height / 2);
        let mut cells = Vec::new();
        for x in start_x..start_x.wrapping_add(self.length) {
            // В оригинале внутренний предел тоже использует length, не height.
            for y in start_y..start_y.wrapping_add(self.length) {
                cells.push((x, y));
            }
        }
        cells
    }

    pub const fn state(&self, center_x: i32, center_y: i32) -> WeakState {
        WeakState::new(self.attack_loss, center_x, center_y, self.length, self.height)
    }
}

pub fn weak_lifetime(lifetime_factor: u32, element_modify: u32, base_lifetime: u32) -> u32 {
    let factor = lifetime_factor.wrapping_mul(element_modify).wrapping_add(100);
    let factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    truncate_original(f64::from(base_lifetime) * f64::from(factor)) as u32
}

const ATTACK_LOSS_PROPERTY: u32 = 205;
const LIFETIME_FACTOR_PROPERTY: u32 = 20_010;
const LIFETIME_PROPERTY: u32 = 30_001;

/// Живая форма области ослабления: связка CShape и области правил.
/// Состав и порядок соответствуют адаптеру старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CWeakPhalanx {
    shape: CShape,
    area: WeakPhalanx,
}

impl CWeakPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, length: i32, height: i32, attack_loss: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, area: WeakPhalanx::new(master, started_at_ms, lifetime_ms, skill_level, length, height, attack_loss) }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.area.master() }
    pub const fn skill_level(&self) -> i32 { self.area.skill_level() }
    pub const fn attack_loss(&self) -> u32 { self.area.attack_loss() }
    pub const fn length(&self) -> i32 { self.area.length() }
    pub const fn height(&self) -> i32 { self.area.height() }
    pub const fn skill_id(&self) -> u32 { WEAK_SKILL_ID }

    pub fn set_center(&mut self, x: i32, y: i32) {
        let y = (f64::from(y) + 0.5) as f32;
        let x = (f64::from(x) + 0.5) as f32;
        self.shape.set_pos_xy_base(x, y);
    }

    pub fn state(&self) -> Option<WeakState> {
        let y = self.shape.get_tile_y().ok()?;
        let x = self.shape.get_tile_x().ok()?;
        Some(self.area.state(x, y))
    }

    pub fn mark_ended(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub const fn tick(&self, now_ms: u32) -> WeakPhalanxTick {
        self.area.tick(now_ms)
    }

    pub fn active_cells(&self) -> Vec<(i32, i32)> {
        let Ok(center_x) = self.shape.get_tile_x() else { return Vec::new() };
        let Ok(center_y) = self.shape.get_tile_y() else { return Vec::new() };
        self.area.active_cells(center_x, center_y)
    }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, WEAK_SKILL_ID as i32, self.area.skill_level(),
            self.area.master().master_type, self.area.master().master_id,
            self.area.started_at_ms(), self.area.lifetime_ms(), now,
        )
    }
}

/// Тело `CWeak::Summon` (VA 0x005AF41B–0x005AF48C для расчёта срока):
/// actual region U и SAFE-гейт предшествуют Master(country0)/Player EM и
/// свежей таблице; вслед за выбором чтений — ctor области 1×1 с живым
/// уровнем и потерей атаки.
pub fn summon_weak<Game, Runtime>(
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
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    let Some(safe) = game.zonal_area_safe(region_id, destination.0, destination.1) else { return; };
    if safe { return; }
    let Some(mut master) = game.zonal_source_master(source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == PLAYER_TYPE {
        game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify as u32)
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    let lifetime = weak_lifetime(
        properties.query_property(LIFETIME_FACTOR_PROPERTY), element, properties.query_property(LIFETIME_PROPERTY),
    );
    let attack_loss = properties.query_property(ATTACK_LOSS_PROPERTY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let started = now_milliseconds(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CWeakPhalanx::new(id, master, started, lifetime, level, 1, 1, attack_loss);
    phalanx.set_center(destination.0, destination.1);
    if let Some(Ok(id)) = game.add_weak_phalanx(region_id, phalanx, destination.0, destination.1, started, runtime) {
        let _ = game.send_weak_phalanx_entry(region_id, id, runtime);
    }
}
