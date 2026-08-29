//! Владелец достигнутой семантики AI101: создание и связывание пары Цзюмай,
//! выбор цели с минимальным текущим HP и передача цели свободному близнецу.
//! `WhenBeenHurted`, `OnLoseTarget` и `OnSchedule` ниже сохранены как RAW:
//! их ветви движения и повторной постановки событий ещё не подключены целиком.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp

// ============================================================================
// FUNCTION: CJiuMai::WhenBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp:191
// RVA: 0x0020A750
// ADDRESS: 0060a750
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiuMai::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp:360
// RVA: 0x0020A990
// ADDRESS: 0060a990
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiuMai::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp:84
// RVA: 0x0020AB10
// ADDRESS: 0060ab10
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use super::archer::select_archer_enemy;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JiuMaiAiState {
    twins_id: i32,
}

impl JiuMaiAiState {
    pub(crate) const fn twins_id(&self) -> i32 {
        self.twins_id
    }

    pub(crate) const fn set_twins_id(&mut self, twins_id: i32) {
        self.twins_id = twins_id;
    }
}

/// Выполняет достигнутый префикс `OnIdle` AI101. Обычный монстр один раз
/// создаёт бессрочного близнеца того же свойства, а призванный близнец берёт
/// обратный ID из `master_id` и не создаёт следующую сущность.
pub(crate) fn ensure_jiumai_twin<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    runtime: &mut Runtime,
) -> bool {
    let Some((twins_id, summoned, master, owner)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.jiu_mai_ai()?.twins_id(),
                monster.is_summoned_creature(),
                monster.master_info(),
                monster.shape_view(property)?,
            ))
        })
    else {
        return false;
    };
    if twins_id != 0 {
        return true;
    }
    if summoned {
        if let Some(state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| monster.jiu_mai_ai_mut())
        {
            state.set_twins_id(master.master_id);
        }
        return true;
    }

    let position = region.region.get_random_pos_in_range(
        owner.tile_x.wrapping_sub(5),
        owner.tile_y.wrapping_sub(5),
        10,
        10,
        runtime,
    );
    let spawned = position.ok().and_then(|position| {
        let (area_width, area_height) = game.area_dimensions();
        region
            .add_summoned_creature(
                property,
                MasterInfo {
                    master_type: MONSTER_TYPE,
                    master_id: monster_id,
                    ..MasterInfo::default()
                },
                position.x,
                position.y,
                -1,
                u32::MAX,
                area_width,
                area_height,
                runtime,
                |runtime| runtime.now_milliseconds(),
            )
            .ok()
    });
    if let Some(state) = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| monster.jiu_mai_ai_mut())
    {
        state.set_twins_id(spawned.unwrap_or(-1));
    }
    true
}

/// Выбирает цель `OnSearchEnemy` AI101: среди игроков и питомцев внутри
/// дальности охраны остаётся первая цель с минимальным текущим HP.
pub(crate) fn select_jiumai_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    select_archer_enemy(game, region, owner, area_index, guard_range)
}

/// Повторяет `CJiuMai::SetTarget`: основной владелец получает цель всегда,
/// живой близнец — только когда ещё не ведёт собственный бой.
pub(crate) fn assign_jiumai_target(
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
) -> bool {
    let Some(twins_id) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.jiu_mai_ai())
        .map(JiuMaiAiState::twins_id)
    else {
        return false;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_ai_target(target);
    }
    if twins_id > 0
        && let Some(twin) = region.find_monster_by_id_mut(twins_id)
        && !CMoveShape::is_died(twin.hit_points())
        && twin.ai_target().is_none()
    {
        twin.set_ai_target(target);
    }
    true
}
