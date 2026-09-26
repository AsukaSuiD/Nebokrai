//! Адаптер `CSpiderMistPhalanx` (`0x198`) к CShape и живому региону.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/spidermistphalanx.cpp/.h.
//! Данные, маска, AI обхода области и создание состояния находятся в
//! `nebokrai_zone::skills::spidermist` (статусы MATCH и состав ctor `0x5EAEB0`
//! см. там); исходное перекрытие в текущем caller-е направлено на PoisonFog
//! (`0xC9`). Здесь — форма с zone-правилом (конструктор `from_rule` — шов
//! порции №3 по конвенции), кодированный снимок wire-конверта `summonshape`
//! и делегации обхода/наложения с прежними сигнатурами.

use crate::gameserver::appserver::masterinfo::MasterInfo;
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
    let cells = phalanx.active_cells();
    nebokrai_zone::skills::spider_mist_cell_targets(game, region_id, &cells)
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
        Self::from_rule(
            id,
            SpiderMistPhalanx::new(
                master, started_at_ms, lifetime_ms, skill_level,
                state_lifetime_ms, frequency_ms, hp_loss,
            ),
        )
    }

    /// Шов порции №3: живая форма из готового zone-правила; CShape-часть
    /// сохраняет конструкторские default-ы прежнего `new`.
    pub(crate) fn from_rule(id: i32, rule: SpiderMistPhalanx) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self { shape, rule }
    }

    pub(crate) const fn rule(&self) -> &SpiderMistPhalanx { &self.rule }
    pub(crate) const fn skill_level(&self) -> i32 { self.rule.skill_level() }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.rule.master() }

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

    /// Машинное тело `?AddToByteArray@CSpiderMistPhalanx@@` = RVA `0x1E47A0`
    /// (ICF-группа): пятипольный префикс (skill id, уровень, master type/id,
    /// GetRemainedTime) перед CShape — досверка хвоста счётчика 0xBF502.
    pub(crate) fn encode_client_snapshot(&self, now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        nebokrai_zone::skills::encode_related_phalanx_snapshot(
            &self.shape,
            SPIDER_MIST_SKILL_ID as i32,
            self.rule.skill_level(),
            self.master().master_type,
            self.master().master_id,
            self.rule.started_at_ms(),
            self.rule.lifetime_ms(),
            now_milliseconds,
        )
    }
}

pub(crate) fn apply_spider_mist_targets(
    game: &mut CGame,
    region_id: i32,
    phalanx: &CSpiderMistPhalanx,
    candidates: Vec<ShapeIdentity>,
    mut now_milliseconds: impl FnMut() -> u32,
) -> usize {
    nebokrai_zone::skills::apply_spider_mist_targets(
        game, region_id, phalanx.rule(), &candidates, &mut now_milliseconds,
    )
}


