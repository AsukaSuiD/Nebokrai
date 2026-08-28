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

use super::spidermist::SPIDER_MIST_SKILL_ID;
use super::spiderpoison::install_spider_poison_state;
use super::spiderpoisonstate::SpiderPoisonState;
use super::monsterattack::{owned_monster_attackable, resolve_owned_monster_attack_target};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
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
        if now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
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

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape
            .encode_to_byte_array(&mut payload, true)
            .then_some(payload)
    }

    pub(crate) const fn skill_id(&self) -> u32 { SPIDER_MIST_SKILL_ID }
}

pub(crate) fn apply_spider_mist_targets(
    game: &mut CGame,
    region: &mut CServerRegion,
    phalanx: &CSpiderMistPhalanx,
    candidates: Vec<ShapeIdentity>,
    mut now_milliseconds: impl FnMut() -> u32,
) -> usize {
    let master = phalanx.master();
    let Some((attacker_property, attacker_tamed, attacker_master)) = region
        .find_monster_by_id(master.master_id)
        .and_then(|monster| {
            Some((
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.is_tamed(),
                monster.master_info(),
            ))
        })
    else {
        return 0;
    };
    let mut applied = 0usize;
    for candidate in candidates {
        if candidate.object_type == master.master_type && candidate.id == master.master_id {
            continue;
        }
        let Some(target) = resolve_owned_monster_attack_target(game, region, candidate) else {
            continue;
        };
        if target.dead
            || target.god
            || target.city_dead
            || !owned_monster_attackable(
                game,
                region.id,
                &attacker_property,
                attacker_tamed,
                attacker_master,
                candidate,
                &target,
            )
        {
            continue;
        }
        let blocked = match candidate.object_type {
            400 => game.find_player(candidate.id).is_none_or(|player| {
                player.has_state_by_skill_id(0x131)
                    || player.has_state_by_skill_id(super::spiderpoison::SPIDER_POISON_SKILL_ID)
            }),
            600 => region.find_monster_by_id(candidate.id).is_none_or(|monster| {
                monster.move_shape().has_state_by_skill_id(0x131)
                    || monster
                        .move_shape()
                        .has_state_by_skill_id(super::spiderpoison::SPIDER_POISON_SKILL_ID)
            }),
            _ => true,
        };
        if blocked {
            continue;
        }
        let now_ms = now_milliseconds();
        install_spider_poison_state(
            game,
            region,
            candidate,
            SpiderPoisonState::new(
                master,
                now_ms,
                phalanx.state_lifetime_ms(),
                phalanx.frequency_ms(),
                phalanx.hp_loss(),
            ),
            now_ms,
        );
        applied = applied.wrapping_add(1);
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
