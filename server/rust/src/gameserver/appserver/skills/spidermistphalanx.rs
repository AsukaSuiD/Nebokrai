//! Область паучьего тумана `CSpiderMistPhalanx` (`0x198`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spidermistphalanx.cpp`. Владелец хранит подтверждённую
//! для всех трёх уровней полную маску 5×5, строгий срок с переполнением `u32` и параметры
//! `SpiderPoisonState`. Обход активных
//! клеток остаётся X→Y; разрешение форм и применение
//! состояния выполняет владелец исполнения, которому принадлежит раздельный
//! доступ к игрокам, монстрам и региону. Исходное вычитание перекрытия направлено
//! в отдельный `SKILL_POISON_FOG` (`0xC9`) и не подменяется самоперекрытием.
//! Уже существующий SpiderPoison не заменяется: Cure/яд запрещают
//! наложение. Источник заново разрешается перед ctor; Begin(source,target)
//! и visual исполняются до общего append в опубликованном регионе.
//! До IsAttackAble проверяются смерть цели, Cure и SpiderPoison. Права берутся
//! у живого player/monster-источника, а будущий яд сохраняет MasterInfo области.

use super::spidermist::SPIDER_MIST_SKILL_ID;
use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const SCOPE_SIDE: usize = 5;
const SCOPE: [[u8; SCOPE_SIDE]; SCOPE_SIDE] = [
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpiderMistPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSpiderMistPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    state_lifetime_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    scope: [[u8; SCOPE_SIDE]; SCOPE_SIDE],
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
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            state_lifetime_ms,
            frequency_ms,
            hp_loss,
            scope: SCOPE,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn state_lifetime_ms(&self) -> u32 { self.state_lifetime_ms }
    pub(crate) const fn frequency_ms(&self) -> u32 { self.frequency_ms }
    pub(crate) const fn hp_loss(&self) -> u32 { self.hp_loss }

    pub(crate) fn tick(&mut self, now_ms: u32) -> SpiderMistPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            SpiderMistPhalanxTick::Expired
        } else {
            SpiderMistPhalanxTick::Scan
        }
    }

    pub(crate) fn active_cells(&self) -> Vec<(i32, i32)> {
        let Ok(center_x) = self.shape.get_tile_x() else { return Vec::new() };
        let Ok(center_y) = self.shape.get_tile_y() else { return Vec::new() };
        let start_x = center_x.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let start_y = center_y.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let mut cells = Vec::new();
        for x in 0..SCOPE_SIDE {
            for y in 0..SCOPE_SIDE {
                if self.scope[y][x] != 0 {
                    cells.push((start_x.wrapping_add(x as i32), start_y.wrapping_add(y as i32)));
                }
            }
        }
        cells
    }

    /// Вычитает из этой области клетки новой области того же семейства.
    /// Во всех подтверждённых уровнях входная маска SpiderMist полная 5x5.
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
        let half = (SCOPE_SIDE / 2) as i32;
        let existing_left = center_x.wrapping_sub(half);
        let existing_top = center_y.wrapping_sub(half);
        let incoming_left = incoming_tile_x.wrapping_sub(half);
        let incoming_top = incoming_tile_y.wrapping_sub(half);
        let existing_right = existing_left.wrapping_add(SCOPE_SIDE as i32);
        let existing_bottom = existing_top.wrapping_add(SCOPE_SIDE as i32);
        let incoming_right = incoming_left.wrapping_add(SCOPE_SIDE as i32);
        let incoming_bottom = incoming_top.wrapping_add(SCOPE_SIDE as i32);

        let overlap_left = existing_left.max(incoming_left);
        let overlap_top = existing_top.max(incoming_top);
        let overlap_right = existing_right.min(incoming_right);
        let overlap_bottom = existing_bottom.min(incoming_bottom);
        if overlap_left >= overlap_right || overlap_top >= overlap_bottom {
            return;
        }

        for world_y in overlap_top..overlap_bottom {
            for world_x in overlap_left..overlap_right {
                let incoming_x = world_x.wrapping_sub(incoming_left) as usize;
                let incoming_y = world_y.wrapping_sub(incoming_top) as usize;
                if SCOPE[incoming_y][incoming_x] != 0 {
                    let existing_x = world_x.wrapping_sub(existing_left) as usize;
                    let existing_y = world_y.wrapping_sub(existing_top) as usize;
                    self.scope[existing_y][existing_x] = 0;
                }
            }
        }
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


// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermistphalanx.cpp

// ============================================================================
// FUNCTION: CSpiderMistPhalanx::ReplaceAffectRegion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermistphalanx.cpp:204
// RVA: 0x001EAC30
// ADDRESS: 005eac30
// PROTOTYPE: void __thiscall ReplaceAffectRegion(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMistPhalanx::CSpiderMistPhalanx
// STATUS: IMPLEMENTED
// Конструктор сохраняет владельца, жизненный цикл и параметры яда; данные PDB
// `0x006A47F4..0x006A485C` подтверждают три одинаковые полные маски 5×5.

// ============================================================================
// FUNCTION: CSpiderMistPhalanx::~CSpiderMistPhalanx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermistphalanx.cpp:62
// RVA: 0x001EB000
// ADDRESS: 005eb000
// PROTOTYPE: void __thiscall ~CSpiderMistPhalanx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMistPhalanx::AI
// STATUS: IMPLEMENTED
// Строгий срок с переполнением `u32`, X→Y-обход клеток, фильтры целей и установка
// `SpiderPoisonState` материализованы функциями владельца выше.

// COMPONENT_VARIANT_END: GameServer
