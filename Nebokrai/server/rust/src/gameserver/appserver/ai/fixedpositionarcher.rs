//! Достигнутая часть ИИ неподвижного лучника `CFixedPositionArcher`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/fixedpositionarcher.cpp` подтверждают
//! порядок девяти соседних областей, игроков перед питомцами, дальность охраны
//! и необычное предпочтение цели вне минимальной дистанции текущего навыка.
//! Реальный путь `monsterbaseattack` выполняет выбор навыка и поиск цели, а
//! достигнутый владелец навыка не использует произвольный порядок хранилища
//! сущностей. `OnIdle` ставит строгую очередь
//! `ChangeSkill → Stand → SearchEnemy`, а завершённая атака сохраняет навык и
//! снова ставит поиск. PDB содержит единственное concrete-определение
//! `CFixedPositionArcher::OnChangeSkill`; стационарные guard-классы не
//! переопределяют его. Унаследованный метод проверяет `CSkill::IsRestored` и
//! только для ещё не восстановленного навыка дописывает полный
//! `GetRestoreTime` в хвост FIFO.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::ai::baseai::AiShapeAction;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::{
    SKILL_USAGE_REUSE_DELAY_TIME, real_distance,
};
use crate::gameserver::appserver::skills::kernel::skill_is_restored;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FixedArcherTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

/// Стационарные лучники и охранники завершают атаку новым поиском, не сбрасывая
/// текущий навык. Остальные monster-owner-ы сохраняют общий `ChangeSkill`.
pub(crate) const fn attack_completion_action(ai_type: u32) -> AiShapeAction {
    if matches!(ai_type, 5 | 11 | 103) {
        AiShapeAction::SearchEnemy
    } else {
        AiShapeAction::ChangeSkill
    }
}

pub(crate) const fn inherits_fixed_archer_change_skill(ai_type: u32) -> bool {
    matches!(ai_type, 5 | 8 | 11 | 13 | 17 | 100 | 101 | 103)
}

/// Ставит общую точную очередь стационарного `OnIdle`. Каждый исходный
/// `AddAIEvent` получает отдельный замер часов.
pub(crate) fn queue_stationary_guard_idle<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    monster.begin_active_ai_change_skill(runtime.now_milliseconds());
    monster.begin_active_ai_stand(stop_frame, runtime.now_milliseconds());
    monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
    true
}

/// Выполняет унаследованный хвост `OnChangeSkill` стационарных лучников и
/// guard-вариантов.
/// `false` означает, что выбранный concrete skill не разрешился и общий owner
/// обязан назначить default skill. Разрешённый выбор сохраняется; для ещё не
/// восстановленного навыка полный `GetRestoreTime` дописывается в FIFO.
/// Отдельный вызов часов повторяет точный `CSkill::IsRestored`.
pub(crate) fn queue_fixed_archer_skill_delay<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    selected_skill_id: u16,
    runtime: &mut Runtime,
) -> bool {
    if !inherits_fixed_archer_change_skill(property.ai) {
        return false;
    }
    let Some(skill) = property
        .skills
        .iter()
        .filter(|skill| skill.id == selected_skill_id)
        .max_by_key(|skill| skill.level)
    else {
        return false;
    };
    let Some(skill_properties) =
        game.skill_base_properties(u32::from(skill.id), i32::from(skill.level))
    else {
        return false;
    };
    let delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let last_used_ms = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.skill_last_used_ms(u32::from(skill.id)))
        .unwrap_or_default();
    let restored_at_ms = runtime.now_milliseconds();
    if skill_is_restored(last_used_ms, delay_ms, restored_at_ms) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_stand(delay_ms, runtime.now_milliseconds());
    }
    true
}

/// Повторяет необычное правило `OnSearchEnemy`: выбирается ближайшая цель не
/// ближе минимальной дистанции навыка; если уже выбранная цель слишком близка,
/// следующая допустимая по дальности охраны запись заменяет её даже при большей
/// дистанции. Поэтому порядок игроков, затем питомцев и порядок внутри индексов
/// являются частью результата.
pub(crate) fn consider_fixed_archer_target(
    selected: Option<FixedArcherTarget>,
    candidate: FixedArcherTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<FixedArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    let Some(current) = selected else {
        return Some(candidate);
    };
    if current.distance <= candidate.distance {
        if current.distance < minimum_skill_distance {
            Some(candidate)
        } else {
            Some(current)
        }
    } else if candidate.distance < minimum_skill_distance {
        Some(current)
    } else {
        Some(candidate)
    }
}

/// Выполняет достигнутый `OnSearchEnemy` AI5, сохраняя отдельные проходы
/// игроков и питомцев по упорядоченным индексам региона.
pub(crate) fn select_fixed_archer_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity> {
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    selected.map(|selected| selected.identity)
}
