//! Фронтальный рубящий удар `CJuCut` (`0x6C`).
//! Успешный Begin возвращает Begun до первого AI. Расход ресурсов,
//! перемещение и атака остаются у AI после постановки Attack в том же Run;
//! раннее время Begin сохраняется общим kernel.
//! Reuse проверяется exact `CSkill::IsRestored`; cast delay остаётся elapsed.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/jucut.cpp`. Навык сохраняет частичное изменение MP до
//! повторной проверки оружия, направление на исходную цель и первую
//! `CMoveShape` лицевой клетки в региональном порядке. Неатакуемая первая
//! фигура не пропускается ради следующей. Формула выполняет один бросок урона
//! и один бросок критического удара только для фактически атакуемой цели.
//! `CGame` используется только для разрешения независимого владельца цели,
//! применения рассчитанной атаки и доставки. `Attack` и `AI` не изнашивают
//! оружие в точке попадания: унаследованный `AfterUseSkill` делает это один
//! раз через общий `End(1)` фронтального семейства.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::frontcellsword::{
    FrontCellSwordDefinition, MONSTER_TYPE, PLAYER_TYPE, calculate_attack, destination,
    finish_front_cell_sword, front_shape, master_info, send_failure, send_visual, target_level,
    weapon_is_compatible,
};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const JU_CUT_SKILL_ID: u32 = 0x6c;
const USER_MP_LOSE: u32 = 2;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const DEFINITION: FrontCellSwordDefinition = FrontCellSwordDefinition {
    skill_id: JU_CUT_SKILL_ID,
    weapon_category: 2,
    weapon_failure_string: b"GS0292",
};

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_ju_cut_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == JU_CUT_SKILL_ID,
    }
}

fn finish_player_ju_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_front_cell_sword(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(JU_CUT_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_ju_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(JU_CUT_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_ju_cut(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_ju_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_ju_cut_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, source_x, source_y, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.learned_skill_level(JU_CUT_SKILL_ID, game.skill_factory()),
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(JU_CUT_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let target_damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(JU_CUT_SKILL_ID).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(player_ai.skill_last_used_ms(JU_CUT_SKILL_ID), cooldown_ms, now_ms) {
            send_failure(game, player_id, DEFINITION, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if !weapon_is_compatible(game, player, DEFINITION) {
            send_failure(game, player_id, DEFINITION, 0x0e, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, DEFINITION, 7, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(JU_CUT_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .player_skill_execution(JU_CUT_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .player_skill_execution(JU_CUT_SKILL_ID)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, DEFINITION, 7, mp_loss);
            finish_player_ju_cut(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game
            .find_player(player_id)
            .is_none_or(|player| !weapon_is_compatible(game, player, DEFINITION))
        {
            send_failure(game, player_id, DEFINITION, 0x0e, mp_loss);
            finish_player_ju_cut(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some((_, target_x, target_y)) = destination(game, region_id, player_id, dispatch)
            && let Some(player) = game.find_player_mut(player_id)
        {
            player
                .movement_shape_mut()
                .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        send_visual(game, player_id, DEFINITION, level, dispatch, 1);
        if let Some(execution) = player_ai.player_skill_execution_mut(JU_CUT_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .player_skill_execution(JU_CUT_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение рубящего удара создано");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_visual(game, player_id, DEFINITION, level, dispatch, 2);
    if let Some(execution) = player_ai.player_skill_execution_mut(JU_CUT_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let owner = game.find_player(player_id).map(master_info).unwrap_or_default();
    if let Some(target) = front_shape(game, region_id, player_id)
        && matches!(target.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
        && !(target.identity.object_type == PLAYER_TYPE && target.identity.id == player_id)
        && !game.periodic_state_target_dead(region_id, target.identity)
        && game.owned_player_skill_target_attackable(owner, target.identity, region_id)
        && let Some(level_of_target) = target_level(game, region_id, target.identity)
        && let Some((master, attack)) = calculate_attack(
            game,
            player_id,
            DEFINITION,
            level_of_target,
            level,
            hit_modifier,
            target_damage_factor,
        )
    {
        match target.identity.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
                master,
                target.identity.id,
                region_id,
                attack,
                runtime,
            ),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
                master,
                target.identity.id,
                region_id,
                attack,
                runtime,
            ),
            _ => unreachable!("тип цели проверен перед расчётом"),
        }
    }
    if let Some(execution) = player_ai.player_skill_execution_mut(JU_CUT_SKILL_ID) {
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_ju_cut(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
