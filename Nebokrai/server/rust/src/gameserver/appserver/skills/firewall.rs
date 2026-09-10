//! Огненная стена `CFireWall` (`0x134`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/firewall.cpp`. Владелец сохраняет три перегрузки цели
//! через типизированный dispatch, двойную проверку MP, строгие границы
//! восстановления и задержки, проверку всех клеток пути, направление и точные
//! пакеты визуального эффекта. После применения создаётся региональный
//! `CFireWallPhalanx`; `CGame` только разрешает независимых владельцев,
//! регистрирует область и выполняет фактическую доставку.
//! Стихийная прибавка усекается из расширенного x87-результата; коэффициент
//! времени жизни отдельно сохраняется в `f32` до умножения и усечения.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; задержка и
//! lifetime стены остаются elapsed.
//! End (0x005AE7A0) сначала возвращает движение, затем при успехе вызывает
//! оружейный AfterUseSkill. Пустой callback CPlayer +0x158 не заменён
//! пересчётом свойств; удаление региональной стены не является частью End.
//! AfterUse/reuse принадлежат общей границе экземпляра: свежие часы reuse
//! читаются после износа, а не перед ним, и не записываются в замену навыка.

use super::baseattack::time_reached;
use super::fightdefense::truncate_original;
use super::firewallphalanx::CFireWallPhalanx;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const FIRE_WALL_SKILL_ID: u32 = 0x134;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const DELAY_TIME: u32 = 10_001;
const REUSE_DELAY_TIME: u32 = 10_005;
const MIN_ATTACK: u32 = 20_001;
const MAX_ATTACK: u32 = 20_002;
const CONST: u32 = 20_010;
const EM_MODIFIER: u32 = 20_015;
const SUMMONED_LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn target_position(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: FIRE_WALL_SKILL_ID, x, y } => Some((x, y, None)),
        PlayerSkillDispatch::Object { skill_id: FIRE_WALL_SKILL_ID, target }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) =>
        {
            let view = game.base_magic_target_view(region_id, target)?;
            Some((view.tile_x, view.tile_y, Some(target)))
        }
        _ => None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    action: u8,
    destination: Option<(i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(FIRE_WALL_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        let Some((x, y)) = destination else { return };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
    successful: bool,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if successful {
        game.after_use_player_skill(player_id, FIRE_WALL_SKILL_ID, runtime);
    }
}

pub(crate) fn cancel_player_fire_wall<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    record_reuse: bool,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, FIRE_WALL_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish(game, player_id, runtime, record_reuse);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) const fn is_fire_wall_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: FIRE_WALL_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: FIRE_WALL_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

pub(crate) fn execute_player_fire_wall<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some((region_id, source_x, source_y, level, initial_mana, combat, master)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(FIRE_WALL_SKILL_ID, game.skill_factory()),
                player.mana(),
                player.combat_properties(),
                MasterInfo {
                    master_type: PLAYER_TYPE,
                    master_id: player_id,
                    master_guild_id: player.faction_id(),
                    master_team_id: player.team_id(),
                    master_union_id: player.union_id(),
                    master_country_id: 0,
                    permitted_to_kill_player: i32::from(player.pk_permissions().player),
                    permitted_to_kill_teammate: i32::from(player.pk_permissions().teammate),
                    permitted_to_kill_guild_member: i32::from(player.pk_permissions().guild_member),
                    permitted_to_kill_criminal: i32::from(player.pk_permissions().criminal),
                },
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(FIRE_WALL_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let delay_ms = properties.query_property(DELAY_TIME);
    let reuse_delay_ms = properties.query_property(REUSE_DELAY_TIME);
    let minimum_attack = properties.query_property(MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(MAX_ATTACK) as i32;
    let constant = properties.query_property(CONST);
    let em_modifier = properties.query_property(EM_MODIFIER);
    let summoned_lifetime = properties.query_property(SUMMONED_LIFETIME);

    if game.player_skill_execution(player_id, FIRE_WALL_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, FIRE_WALL_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((target_x, target_y, _)) = target_position(game, region_id, dispatch) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| matches!(cell.2, 1 | 2)) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info(player_id, b"GS0282");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(FIRE_WALL_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, FIRE_WALL_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target)) = target_position(game, region_id, dispatch) else {
        finish(game, player_id, runtime, false);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish(game, player_id, runtime, false);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, FIRE_WALL_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish(game, player_id, runtime, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, target_x, target_y,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, 1, None);
        if let Some(state) = game.player_skill_execution_mut(player_id, FIRE_WALL_SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_execution(player_id, FIRE_WALL_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение огненной стены создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_visual(game, player_id, level, 2, Some((target_x, target_y)));

    let element_bonus = truncate_original(
        f64::from(em_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let lifetime_scale = constant.wrapping_mul(element_bonus as u32).wrapping_add(100);
    let lifetime_factor = (f64::from(lifetime_scale) * f64::from(0.01_f32)) as f32;
    let lifetime_ms = truncate_original(
        f64::from(summoned_lifetime) * f64::from(lifetime_factor),
    ) as u32;
    let element_attack = (combat.add_element_attack as i32).wrapping_add(element_bonus);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CFireWallPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        level,
        frequency_ms,
        minimum_attack,
        maximum_attack,
        element_attack,
        i32::from(combat.cch),
    );
    phalanx.shape_mut().set_region_id(region_id);
    let summoned = game
        .add_fire_wall_phalanx(
            region_id, phalanx, target_x, target_y, summon_started_at_ms, runtime,
        )
        .is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_fire_wall_phalanx_entry(region_id, summon_id);
    }

    if let Some(state) = game.player_skill_execution_mut(player_id, FIRE_WALL_SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish(game, player_id, runtime, true);
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::Rejected
    })
}
