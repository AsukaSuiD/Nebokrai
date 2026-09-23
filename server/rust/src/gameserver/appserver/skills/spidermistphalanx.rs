//! Адаптер `CSpiderMistPhalanx` (`0x198`) к CShape и живому региону.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/spidermistphalanx.cpp/.h.
//! Данные, маска и время находятся в zone/skills/spidermist.rs.
//! Поиск цели, проверки Cure/SpiderPoison, Begin и visual остаются здесь;
//! исходное перекрытие в текущем caller-е направлено на PoisonFog (`0xC9`).

use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::{SPIDER_MIST_SKILL_ID, SpiderMistPhalanx};

pub(crate) use nebokrai_zone::skills::SpiderMistPhalanxTick;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSpiderMistPhalanx {
    shape: CShape,
    rule: SpiderMistPhalanx,
}

pub(crate) fn spider_mist_targets(game: &CGame, region_id: i32, phalanx: &CSpiderMistPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (width, height) = game.area_dimensions();
    let mut targets = Vec::new();
    for (tile_x, tile_y) in phalanx.active_cells() {
        let mut shapes = Vec::new();
        if region.get_shapes(tile_x, tile_y, width, height, game, &mut shapes).is_err() { break }
        targets.extend(shapes.into_iter().map(|shape| shape.identity));
    }
    targets
}

impl CSpiderMistPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        state_lifetime_ms: u32,
        frequency_ms: u32,
        hp_loss: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self { shape, rule: SpiderMistPhalanx::new(
            master, started_at_ms, lifetime_ms, skill_level,
            state_lifetime_ms, frequency_ms, hp_loss,
        ) }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.rule.master() }
    pub(crate) const fn skill_level(&self) -> i32 { self.rule.skill_level() }
    pub(crate) const fn state_lifetime_ms(&self) -> u32 { self.rule.state_lifetime_ms() }
    pub(crate) const fn frequency_ms(&self) -> u32 { self.rule.frequency_ms() }
    pub(crate) const fn hp_loss(&self) -> u32 { self.rule.hp_loss() }

    pub(crate) fn tick(&mut self, now_ms: u32) -> SpiderMistPhalanxTick {
        let tick = self.rule.tick(now_ms);
        if tick == SpiderMistPhalanxTick::Expired {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
        }
        tick
    }

    pub(crate) fn active_cells(&self) -> Vec<(i32, i32)> {
        let Ok(center_x) = self.shape.get_tile_x() else { return Vec::new() };
        let Ok(center_y) = self.shape.get_tile_y() else { return Vec::new() };
        self.rule.active_cells(center_x, center_y)
    }

    pub(crate) fn replace_affect_region(
        &mut self,
        _incoming_level: i32,
        incoming_tile_x: i32,
        incoming_tile_y: i32,
    ) {
        let (Ok(center_x), Ok(center_y)) =
            (self.shape.get_tile_x(), self.shape.get_tile_y())
        else {
            return;
        };
        self.rule.replace_affect_region(center_x, center_y, incoming_tile_x, incoming_tile_y);
    }

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape
            .add_to_byte_array(&mut payload, true)
            .then_some(payload)
    }

    pub(crate) const fn skill_id(&self) -> u32 { SPIDER_MIST_SKILL_ID }
}

pub(crate) fn apply_spider_mist_targets(
    game: &mut CGame,
    region_id: i32,
    phalanx: &CSpiderMistPhalanx,
    candidates: Vec<ShapeIdentity>,
    mut now_milliseconds: impl FnMut() -> u32,
) -> usize {
    let master = phalanx.master();
    let source = ShapeIdentity {
        object_type: master.master_type, id: master.master_id, ex_id: CGuid::GUID_INVALID,
    };
    if game.find_shape_in_region(region_id, source).is_none()
        || resolve_state_move_shape(game, region_id, source).is_none()
    {
        return 0;
    }
    let mut applied = 0usize;
    for candidate in candidates {
        if candidate.object_type == master.master_type && candidate.id == master.master_id {
            continue;
        }
        if game.move_shape_health(region_id, candidate).is_none_or(|health| health == 0) {
            continue;
        }
        let Some(target) = resolve_state_move_shape(game, region_id, candidate) else { continue; };
        if target.has_state_by_skill_id(0x131)
            || target.has_state_by_skill_id(super::spiderpoison::SPIDER_POISON_SKILL_ID)
        {
            continue;
        }
        if !game.live_skill_target_attackable(region_id, source, candidate) { continue; }
        if game.find_shape_in_region(region_id, source).is_none() { continue; }
        let Some(source_region) = resolve_state_move_shape(game, region_id, source)
            .map(|shape| shape.shape().get_region_id()) else { continue; };
        let state = SpiderPoisonState::new(
            master, phalanx.state_lifetime_ms(), phalanx.frequency_ms(), phalanx.hp_loss(),
        );
        if begin_primary_spider_poison_state(
            game, region_id, candidate, Some((source_region, source)),
            Some((region_id, candidate)), state, None, &mut now_milliseconds,
        ).is_some() {
            applied = applied.wrapping_add(1);
        }
    }
    applied
}
