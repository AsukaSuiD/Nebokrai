//! Владелец достигнутой семантики AI101: создание и связывание пары Цзюмай,
//! выбор цели с минимальным текущим HP и передача цели свободному близнецу.
//! У `WhenBeenHurted` достигнуты прямые цели игрока и приручённого
//! монстра либо повозки; пространственное отступление при исчезнувшем игроке
//! остаётся RAW. У `OnSchedule` достигнуто сближение близнецов перед общим
//! боевым расписанием, а очередь боевых событий ещё не замкнута.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp

// ============================================================================
// FUNCTION: CJiuMai::WhenBeenHurted
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `retarget_jiumai_after_hurt` вызывается всеми достигнутыми
// владельцами нанесения урона и сохраняет базовую защиту и прямые допустимые цели.
// REMAINS: пространственное отступление при отсутствующем игроке остаётся RAW.
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
// FUNCTION: CJiuMai::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `maintain_jiumai_twin` сохраняет проверку дистанций, один
// `GetRandomPosInRange` и последующий `ForceMove` до общего боевого такта.
// REMAINS: точная очередь `ASA_SEARCH_ENEMY/ASA_ATTACK` остаётся RAW.
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
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JiuMaiAiState {
    twins_id: i32,
    linked_target: bool,
}

impl JiuMaiAiState {
    pub(crate) const fn twins_id(&self) -> i32 {
        self.twins_id
    }

    pub(crate) const fn set_twins_id(&mut self, twins_id: i32) {
        self.twins_id = twins_id;
    }

    const fn linked_target(&self) -> bool {
        self.linked_target
    }

    const fn set_linked_target(&mut self, linked_target: bool) {
        self.linked_target = linked_target;
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

/// Сохраняет достигнутый префикс `OnSchedule`: если живой близнец дальше
/// пяти клеток и текущая цель не ближе к владельцу, владелец получает точный
/// случайный пункт около близнеца и выполняет исходный `ForceMove` с нулевой
/// длительностью. Порядок RNG остаётся перед общим боевым расписанием.
pub(crate) fn maintain_jiumai_twin<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    runtime: &mut Runtime,
) -> bool {
    let Some((twins_id, owner, target)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.jiu_mai_ai()?.twins_id(),
                monster.shape_view(property)?,
                monster.ai_target(),
            ))
        })
    else {
        return false;
    };
    let Some(twin) = region.find_monster_by_id(twins_id).and_then(|monster| {
        (!CMoveShape::is_died(monster.hit_points())).then(|| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        })?
    }) else {
        return true;
    };
    if real_distance(owner.tile_x, owner.tile_y, twin.tile_x, twin.tile_y) <= 5 {
        return true;
    }

    let target = target.and_then(|identity| match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).and_then(|player| player.shape_view()),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        }),
        _ => None,
    });
    if target.is_some_and(|target| {
        real_distance(target.tile_x, target.tile_y, owner.tile_x, owner.tile_y)
            <= real_distance(target.tile_x, target.tile_y, twin.tile_x, twin.tile_y)
    }) {
        return true;
    }

    let Ok(destination) = region.region.get_random_pos_in_range(
        twin.tile_x.wrapping_sub(5),
        twin.tile_y.wrapping_sub(5),
        10,
        10,
        runtime,
    ) else {
        return true;
    };
    let _ = game.force_move_owned_shape(
        region,
        owner.identity,
        destination.x,
        destination.y,
        0,
        runtime,
    );
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
        if let Some(state) = monster.jiu_mai_ai_mut() {
            state.set_linked_target(true);
        }
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

/// Завершает достигнутый `OnLoseTarget` после общего боевого такта. Метка
/// отличает реальный переход ранее связанной цели от обычного бездействия:
/// независимый бой близнеца без предшествующего `SetTarget` не стирается.
pub(crate) fn synchronize_jiumai_target_loss(
    region: &mut CServerRegion,
    monster_id: i32,
) -> bool {
    let Some((twins_id, lost_linked_target)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let state = monster.jiu_mai_ai()?;
            Some((
                state.twins_id(),
                state.linked_target() && monster.ai_target().is_none(),
            ))
        })
    else {
        return false;
    };
    if !lost_linked_target {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id)
        && let Some(state) = monster.jiu_mai_ai_mut()
    {
        state.set_linked_target(false);
    }
    if twins_id > 0
        && let Some(twin) = region.find_monster_by_id_mut(twins_id)
        && !CMoveShape::is_died(twin.hit_points())
        && twin.ai_target().is_some()
    {
        twin.clear_ai_target();
    }
    true
}

/// Выполняет достигнутые прямые ветви `WhenBeenHurted` AI101 после
/// освобождения изменяемого заимствования цели. Событие защиты ставится всегда;
/// свободная пара принимает существующего игрока либо приручённого монстра или
/// повозку. Отступление при исчезнувшем игроке пока остаётся RAW.
pub(crate) fn retarget_jiumai_after_hurt(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker: ShapeIdentity,
    now_ms: u32,
) -> bool {
    let Some(fighting) = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        monster.jiu_mai_ai()?;
        monster.when_been_hurted(now_ms);
        Some(monster.ai_target().is_some())
    }) else {
        return false;
    };
    if fighting {
        return true;
    }
    let eligible = match attacker.object_type {
        PLAYER_TYPE => game
            .find_player(attacker.id)
            .is_some_and(|player| player.server_region_id() == Some(region.id)),
        MONSTER_TYPE => region.find_monster_by_id(attacker.id).is_some_and(|monster| {
            monster.is_tamed()
                || monster
                    .base_property_key()
                    .and_then(|key| game.find_monster_property_by_origin_name(key))
                    .is_some_and(|property| monster.is_carriage(property))
        }),
        _ => false,
    };
    if eligible {
        let _ = assign_jiumai_target(region, monster_id, attacker);
    }
    true
}
