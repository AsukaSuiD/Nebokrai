//! Провокация питомцами `CGibe` (`0xD8`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/gibe.cpp`. Навык обходит девять клеток области в исходном
//! порядке, для каждого подходящего монстра строит список питомцев владельца и
//! выполняет ровно один вызов исходного генератора случайных чисел только для
//! непустого списка. Проверка
//! `CMonster::IsAttackAble`, строгая граница расстояния и возможная повторная
//! смена цели более поздним кандидатом сохранены. `CGame` предоставляет и
//! возвращает владельца региона; выбор питомца и правила назначения цели
//! принадлежат этому модулю. Восстановление использует абсолютный срок
//! `CSkill::IsRestored`; стадийная задержка остаётся elapsed.
//! End (0x005AFA40) использует пустой AfterUseSkill (0x00601A70), а общий
//! callback игрока +0x158 также пуст: ни износа, ни UpdateProperty при End нет.
//! Общая граница AfterUse/reuse выбирает этот пустой override из фабричного
//! каталога; время записывается в тот же экземпляр без отдельного хвоста CGibe.

use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::monsterattack::monster_attackable_by_monster;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::public::guid::CGuid;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const GIBE_SKILL_ID: u32 = 0xd8;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;

#[derive(Clone)]
struct MonsterSnapshot {
    identity: ShapeIdentity,
    view: ShapeView,
    property: MonsterProperties,
    tamed: bool,
    master: MasterInfo,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
    }
}

fn monster_snapshot(
    game: &CGame,
    region: &crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
) -> Option<MonsterSnapshot> {
    let monster = region.find_monster_by_id(monster_id)?;
    let property = game
        .find_monster_property_by_origin_name(monster.base_property_key()?)?
        .clone();
    Some(MonsterSnapshot {
        identity: ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: monster_id,
            ex_id: CGuid::GUID_INVALID,
        },
        view: monster.shape_view(&property)?,
        property,
        tamed: monster.is_tamed(),
        master: monster.master_info(),
    })
}

fn apply_gibe(
    game: &mut CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    player_id: i32,
    area_index: usize,
    maximum_distance: u32,
) {
    let pets: Vec<_> = region
        .owned_pet_ids(player_id)
        .into_iter()
        .filter_map(|pet_id| monster_snapshot(game, region, pet_id))
        .collect();
    if pets.is_empty() {
        return;
    }

    for monster_id in region.monster_ids_around_area(area_index) {
        let Some(candidate) = monster_snapshot(game, region, monster_id) else {
            continue;
        };
        if region
            .find_monster_by_id(monster_id)
            .and_then(|monster| monster.ai_target())
            .is_some_and(|target| target.object_type == PLAYER_TYPE && target.id != player_id)
        {
            continue;
        }

        let mut eligible = Vec::new();
        for pet in &pets {
            if monster_attackable_by_monster(
                game,
                &candidate.property,
                candidate.tamed,
                candidate.master,
                &pet.property,
                pet.tamed,
                pet.master,
                region.id,
            ) && (candidate.view.distance(pet.view) as u32) < maximum_distance
            {
                eligible.push(pet.identity);
            }
        }
        if eligible.is_empty() {
            continue;
        }

        let last = eligible.len().saturating_sub(1) as i32;
        let selected = game
            .skill_random_below(eligible.len() as i32)
            .clamp(0, last) as usize;
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_ai_target(eligible[selected]);
        }
    }
}

pub(crate) fn execute_player_gibe<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    if skill_id != GIBE_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, area_index, skill_level, dead)) =
        game.find_player(player_id).and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().area_index()?,
                player.learned_skill_level(GIBE_SKILL_ID, game.skill_factory()),
                player.is_dead(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if dead {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(mut region) = game.take_region_owner(region_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if region.base().owned_pet_ids(player_id).is_empty() {
        game.restore_region_owner(region);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(properties) = game.skill_base_properties(GIBE_SKILL_ID, skill_level) else {
        game.restore_region_owner(region);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let now_ms = runtime.now_milliseconds();
    if !skill_is_restored(game.player_skill_last_used_ms(player_id, GIBE_SKILL_ID), reuse_delay_ms, now_ms) {
        game.restore_region_owner(region);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_execution(player_id, GIBE_SKILL_ID).is_none() {
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(GIBE_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now_ms));
        game.restore_region_owner(region);
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, GIBE_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        game.restore_region_owner(region);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(execution) = game.player_skill_execution_mut(player_id, GIBE_SKILL_ID) {
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
    }
    apply_gibe(
        game,
        region.base_mut(),
        player_id,
        area_index,
        maximum_distance,
    );
    if let Some(execution) = game.player_skill_execution_mut(player_id, GIBE_SKILL_ID) {
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    game.restore_region_owner(region);
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    game.after_use_player_skill(player_id, GIBE_SKILL_ID, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

/// Общий `CSkill::End(bool)` для полностью материализованной провокации:
/// выбор игрока сохраняется, а ненулевой End фиксирует
/// тот же realtime cooldown, что и нормальное синхронное завершение.
pub(crate) fn cancel_player_gibe<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    record_reuse: bool,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, GIBE_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    if record_reuse {
        game.after_use_player_skill(player_id, GIBE_SKILL_ID, runtime);
    }
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}
