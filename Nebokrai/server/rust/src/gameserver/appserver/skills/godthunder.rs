//! Божественный гром `CGodThunder` (`0x140`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godthunder.cpp`. Владелец сохраняет проверки пути и его
//! непроходимых клеток, две проверки MP, блокировку движения, задержку,
//! cooldown, визуальные пакеты и построение `CGodThunderPhalanx`. `CGame`
//! только разрешает независимых владельцев, регистрирует форму и доставляет.
//! Общий для двух вариантов `End(1)` фиксирует только успешно созданную
//! область; `End(0)` очищает отказ или отмену без обновления cooldown.
//! Общая стихийная прибавка обоих вариантов сохраняет расширенное вычисление
//! x87 из целых свойств и усечение результата к нулю. Восстановление обоих
//! вариантов использует абсолютный срок `CSkill::IsRestored`; задержка формы
//! остаётся elapsed.

use super::baseattack::time_reached;
use super::fightdefense::truncate_original;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME};
use super::godthunderphalanx::CGodThunderPhalanx;
use super::godthunderphalanx2::CGodThunderPhalanx2;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const GOD_THUNDER_SKILL_ID: u32 = 0x140;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const MP_LOSE: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const FREQUENCY: u32 = 6_001;
const REUSE_TIME: u32 = 10_005;
const MIN_ATTACK: u32 = 20_001;
const MAX_ATTACK: u32 = 20_002;
const TARGET_COUNT: u32 = 20_010;
const ELEMENT_MODIFIER: u32 = 20_015;
const LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn position(game: &CGame, region: i32, player: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player).and_then(CPlayer::shape_view).map(|s| (s.tile_x, s.tile_y, None)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => game.base_magic_target_view(region, target).map(|s| (s.tile_x, s.tile_y, Some(target))),
        _ => None,
    }
}

fn fail(game: &CGame, player: i32, code: u8, mp: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player, b"GS0288", mp),
        10 => game.send_skill_system_info(player, b"GS0285"),
        0x0b => game.send_skill_system_info(player, b"GS0290"),
        0x0d => game.send_skill_system_info(player, b"GS0278"),
        0x0f => game.send_skill_system_info(player, b"GS0282"),
        _ => {}
    }
}

fn visual(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, action: u8, target: Option<(i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action); message.add_long(skill_id as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(player.shape().identity().object_type); message.add_long(player_id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else if let Some((x, y)) = target {
        message.add_long(0); message.add_long(0); message.add_long(x); message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(crate) const fn is_god_thunder_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: GOD_THUNDER_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: GOD_THUNDER_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: GOD_THUNDER_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

fn execution(ai: &CPlayerAI, second: bool) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
    if second { ai.player_skill_execution(crate::gameserver::appserver::skills::godthunder2::GOD_THUNDER_2_SKILL_ID) } else { ai.player_skill_execution(GOD_THUNDER_SKILL_ID) }
}
fn execution_mut(ai: &mut CPlayerAI, second: bool) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
    if second { ai.player_skill_execution_mut(crate::gameserver::appserver::skills::godthunder2::GOD_THUNDER_2_SKILL_ID) } else { ai.player_skill_execution_mut(GOD_THUNDER_SKILL_ID) }
}

fn last_used(ai: &CPlayerAI, second: bool) -> u32 {
    if second { ai.skill_last_used_ms(crate::gameserver::appserver::skills::godthunder2::GOD_THUNDER_2_SKILL_ID) } else { ai.skill_last_used_ms(GOD_THUNDER_SKILL_ID) }
}
fn mark_used(ai: &mut CPlayerAI, second: bool, now: u32) {
    if second { ai.mark_skill_used(crate::gameserver::appserver::skills::godthunder2::GOD_THUNDER_2_SKILL_ID, now); } else { ai.mark_skill_used(GOD_THUNDER_SKILL_ID, now); }
}

fn finish_player_god_thunder<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, second: bool, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| mark_used(ai, second, now_ms));
}

fn abort_player_god_thunder(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_player_god_thunder_family<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, second: bool, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = execution(ai, second).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_god_thunder(game, player_id, ai, second, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_god_thunder_family<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, second: bool, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = execution(ai, second).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_god_thunder(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_god_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_god_thunder_family(game, player_id, dispatch, ai, runtime, GOD_THUNDER_SKILL_ID, false)
}

pub(super) fn execute_player_god_thunder_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime, skill_id: u32, second: bool,
) -> QueuedSkillExecutionOutcome {
    let requested = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id };
    if requested != skill_id { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region, level, source_x, source_y, initial_mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.learned_skill_level(skill_id),
        player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(skill_id, level) else {
        if execution(ai, second).is_some() { abort_player_god_thunder(game, player_id); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse = properties.query_property(REUSE_TIME);
    let maximum_distance = properties.query_property(MAX_DISTANCE);
    let mp = properties.query_property(MP_LOSE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime = properties.query_property(LIFETIME);
    let frequency = properties.query_property(FREQUENCY);
    let minimum = properties.query_property(MIN_ATTACK) as i32;
    let maximum = properties.query_property(MAX_ATTACK) as i32;
    let target_count = properties.query_property(TARGET_COUNT);
    let element_property = properties.query_property(ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if execution(ai, second).is_none() {
        if !skill_is_restored(last_used(ai, second), reuse, runtime.now_milliseconds()) {
            fail(game, player_id, 0x0d, mp); return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((x, y, _)) = position(game, region, player_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) };
        let path = game.base_magic_path(region, source_x, source_y, x, y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            fail(game, player_id, 0x0b, mp); return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            fail(game, player_id, 0x0f, mp); return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp == 0 || (initial_mana.wrapping_sub(mp) as i32) < 0 {
            if mp != 0 { fail(game, player_id, 7, mp); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id));
        }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, runtime.now_milliseconds()));
    } else if execution(ai, second).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target)) = position(game, region, player_id, dispatch) else {
        abort_player_god_thunder(game, player_id); return terminal(QueuedSkillExecutionState::Rejected);
    };
    if execution(ai, second).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp) as i32) < 0 {
            fail(game, player_id, 7, mp); abort_player_god_thunder(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let _ = game.update_player_criminal_state(player_id, GamePlayerFightStatePhase::MoveShapeAi, runtime);
        visual(game, player_id, skill_id, level, 1, None);
        if let Some(state) = execution_mut(ai, second) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started = execution(ai, second).map(SkillExecutionKernel::started_at_ms).expect("божественный гром начат");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if target.is_some_and(|identity| game.periodic_state_target_dead(region, identity)) {
        fail(game, player_id, 10, mp); abort_player_god_thunder(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    visual(game, player_id, skill_id, level, 2, Some((target_x, target_y)));
    let Some(player) = game.find_player(player_id) else { abort_player_god_thunder(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    let combat = player.combat_properties();
    let element_bonus = truncate_original(
        f64::from(element_property)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let element = (combat.add_element_attack as i32).wrapping_add(element_bonus);
    let cch = i32::from(combat.cch);
    let master = master_info(player);
    let summon_id = game.allocate_summon_shape_id();
    let summon_time = runtime.now_milliseconds();
    let summoned = if second {
        let mut phalanx = CGodThunderPhalanx2::new(summon_id, master, summon_time, lifetime, level, frequency, minimum, maximum, element, target_count, cch);
        phalanx.shape_mut().set_region_id(region);
        phalanx.initialize(target_x, target_y, &mut |maximum| game.skill_random_below(maximum));
        let summoned = game.add_god_thunder_2_phalanx(region, phalanx, target_x, target_y, summon_time, runtime).is_some_and(|result| result.is_ok());
        if summoned { let _ = game.send_god_thunder_2_phalanx_entry(region, summon_id, runtime); }
        summoned
    } else {
        let mut phalanx = CGodThunderPhalanx::new(summon_id, master, summon_time, lifetime, level, frequency, minimum, maximum, element, target_count, cch);
        phalanx.shape_mut().set_region_id(region);
        phalanx.initialize(target_x, target_y, &mut |maximum| game.skill_random_below(maximum));
        let summoned = game.add_god_thunder_phalanx(region, phalanx, target_x, target_y, summon_time, runtime).is_some_and(|result| result.is_ok());
        if summoned { let _ = game.send_god_thunder_phalanx_entry(region, summon_id, runtime); }
        summoned
    };
    if let Some(state) = execution_mut(ai, second) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    if summoned {
        finish_player_god_thunder(game, player_id, ai, second, runtime);
    } else {
        abort_player_god_thunder(game, player_id);
    }
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 2);
    tracing::trace!(region, player_id, summon_id, summoned, "создана область божественного грома");
    terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
