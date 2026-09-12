//! Оглушающий снаряд CStrike (0xDD).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/strike.cpp.
//! Зарегистрированный навык хранит подготовку и фоновый полёт; прямой удар
//! принадлежит OnBeenAttacked цели, а последующее оглушение — Rush2State (0x7C).
//! Общий End завершает навык и единожды выполняет унаследованный AfterUseSkill.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::poisonmoth::master_info;
use super::rush::scaled_state_time;
use super::rushstate2::{RUSH_2_STATE_ID, Rush2State, begin_primary_rush_2_state};
use super::skillfactory::{SkillOwner, UNKNOWN_SKILL_ID};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const STRIKE_SKILL_ID: u32 = 0xDD;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const STATE_TIME: u32 = 10_002;
const MISSILE_TIME: u32 = 10_008;
const DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrikeExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    missile_flying_time_ms: u32,
}

impl StrikeExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), missile_flying_time_ms: 0 }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_strike_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == STRIKE_SKILL_ID
}

pub(crate) fn complete_player_strike<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, STRIKE_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_strike<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, STRIKE_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn target(game: &CGame, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, STRIKE_SKILL_ID)?;
    let (region, identity) = resolve_skill_sufferer(game, lifecycle)?;
    let shape = resolve_state_move_shape(game, region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

fn skill_level(game: &CGame, player_id: i32) -> Option<i32> {
    game.registered_player_skill(player_id, STRIKE_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
}

pub(crate) fn publish_strike_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CStrike
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Strike || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
        let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
        let shape = target.shape();
        let (Ok(x), Ok(y)) = (shape.get_tile_x(), shape.get_tile_y()) else { return; };
        message.add_long(shape.identity().object_type);
        message.add_long(shape.identity().id);
        message.add_long(x);
        message.add_long(y);
        message.add_ulong(skill.player_state::<StrikeExecutionState>()
            .map_or(0, |state| state.missile_flying_time_ms));
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn failure(game: &mut CGame, player_id: i32, mode: u32, text: &[u8], mp: Option<u32>) {
    game.update_player_skill_visual(player_id, STRIKE_SKILL_ID, mode);
    if let Some(mp) = mp {
        game.send_skill_system_info_with_unsigned(player_id, text, mp);
    } else {
        game.send_skill_system_info(player_id, text);
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, runtime: &mut Runtime) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    if target(game, player_id).is_none_or(|(_, target)| target.object_type == PLAYER_TYPE && target.id == player_id) {
        failure(game, player_id, 10, b"GS0286", None);
        return false;
    }
    let Some(skill) = game.registered_player_skill(player_id, STRIKE_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
    else { return false; };
    let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, skill.level()) else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(
        game.player_skill_last_used_ms(player_id, STRIKE_SKILL_ID), reuse, runtime.now_milliseconds(),
    ) {
        failure(game, player_id, 13, b"GS0278", None);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, player_id, 11, b"GS0290", None);
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        failure(game, player_id, 15, b"GS0282", None);
        return false;
    }
    // Нулевая стоимость в исходном CheckCast означает тихий отказ; общий
    // эффект 2 после любого отказа принадлежит Begin, а не этой проверке.
    if properties.query_property(MP_LOSS) == 0 { return false; }
    let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return false; };
    if (mana.wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0 {
        let amount = properties.query_property(MP_LOSS);
        failure(game, player_id, 7, b"GS0288", Some(amount));
        return false;
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    true
}

fn begin_strike<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    game.replace_player_skill_visual_effect(
        player_id, STRIKE_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Strike, 1),
    );
    if !check_cast(game, player_id, runtime) {
        game.update_player_skill_visual(player_id, STRIKE_SKILL_ID, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(started) = game.player_skill_lifecycle(player_id, STRIKE_SKILL_ID)
        .map(|lifecycle| lifecycle.started_at_ms())
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    game.begin_player_skill_execution(player_id, StrikeExecutionState::begin(dispatch, started));
    terminal(QueuedSkillExecutionState::Begun)
}

fn calculate_attack(game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
    let Some(level) = skill_level(game, player_id) else { return; };
    let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else { return; };
    attack.skill_id = STRIKE_SKILL_ID;
    attack.skill_level = level as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let Some(player) = game.find_player(player_id) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor);
    let factor = properties.query_property(DAMAGE_FACTOR);
    // x87 сохраняет только конечный коэффициент в float. Ширина RNG —
    // буквальный DWORD max-min+1, даже при инвертированных границах.
    attack.damage_factor = (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = (properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32).wrapping_neg();
    let minimum = player.combat_properties().minimum_attack as i32;
    let maximum = player.combat_properties().maximum_attack as i32;
    let random = game.skill_random_below(maximum.wrapping_sub(minimum).wrapping_add(1));
    let Some(player) = game.find_player(player_id) else { return; };
    let physical = random.wrapping_add(player.combat_properties().minimum_attack as i32).max(0);
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let critical = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    if resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let master = master_info(player);
    let mut attack = AttackInformation {
        skill_id: UNKNOWN_SKILL_ID,
        skill_level: 1,
        attacker_type: player.shape().identity().object_type,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 0,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: Vec::new(),
    };
    // Calculate с отсутствующей таблицей оставляет исходную пустую атаку;
    // OnBeenAttacked всё равно вызывается, без предварительного IsAttackAble.
    calculate_attack(game, player_id, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn add_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    if keep == 0 { return; }
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return; };
    if !game.live_skill_target_attackable(target.0, user, target.1) { return; }
    let Some(level) = skill_level(game, player_id) else { return; };
    if game.skill_base_properties(STRIKE_SKILL_ID, level).is_none() { return; }
    let Some(region) = game.find_player(player_id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then_some(player.shape().get_region_id())
    }) else { return; };
    if game.find_region(region).is_none() { return; }
    let state = Rush2State::new(keep);
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == RUSH_2_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let sufferer = resolve_state_move_shape(game, target.0, target.1)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let _ = begin_primary_rush_2_state(
        game, target.0, target.1, user, sufferer, state, &mut || runtime.now_milliseconds(),
    );
}

pub(crate) fn execute_player_strike<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_strike_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, STRIKE_SKILL_ID).is_none() {
        return begin_strike(game, player_id, dispatch, runtime);
    }
    let Some(level) = skill_level(game, player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.find_player(player_id).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(target) = target(game, player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if game.base_magic_target_dead(target.0, target.1) {
        failure(game, player_id, 10, b"GS0285", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if target.1.object_type == PLAYER_TYPE && target.1.id == player_id {
        failure(game, player_id, 10, b"GS0286", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, STRIKE_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(MP_LOSS);
            failure(game, player_id, 7, b"GS0288", Some(amount));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, STRIKE_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        let Some((target_x, target_y)) = resolve_state_move_shape(game, target.0, target.1).and_then(|target| {
            let y = target.shape().get_tile_y().ok()?;
            let x = target.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_player_skill_visual(player_id, STRIKE_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, STRIKE_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(kernel) = game.player_skill_execution(player_id, STRIKE_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !kernel.is_prepared() {
        let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        if runtime.now_milliseconds() < kernel.started_at_ms().wrapping_add(delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
        let Some(lifecycle) = game.player_skill_lifecycle(player_id, STRIKE_SKILL_ID) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.skill_target_path(lifecycle);
        let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if properties.query_property(MAX_DISTANCE) != 0
            && properties.query_property(MAX_DISTANCE) < path.len() as u32
        {
            failure(game, player_id, 11, b"GS0290", None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.update_player_skill_visual(player_id, STRIKE_SKILL_ID, 15);
            let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default();
            game.send_skill_system_info_with_text(player_id, b"GS0296", name);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let flight = properties.query_property(MISSILE_TIME).wrapping_mul(path.len() as u32);
        if let Some(state) = game.player_skill_state_mut::<StrikeExecutionState>(player_id, STRIKE_SKILL_ID) {
            state.missile_flying_time_ms = flight;
        }
        game.update_player_skill_visual(player_id, STRIKE_SKILL_ID, 1);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, STRIKE_SKILL_ID) {
            kernel.mark_prepared();
            let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        }
    }
    // Оба срока абсолютные с DWORD overflow. После visual 1 общий prepared
    // переносит тот же экземпляр в фон, а не создаёт отдельный снаряд.
    let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(state) = game.player_skill_state::<StrikeExecutionState>(player_id, STRIKE_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let deadline = state.kernel.started_at_ms().wrapping_add(delay).wrapping_add(state.missile_flying_time_ms);
    if runtime.now_milliseconds() < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    // S и таблица свойств сохраняются с начала этого AI; Calculate и AddState
    // отдельно читают зарегистрированный навык. Уровни для keep нужны после
    // всех damage callbacks, при опубликованном настоящем AI источника.
    game.with_published_player_ai(player_id, ai, |game| {
        attack(game, player_id, target, runtime);
        if !game.base_magic_target_dead(target.0, target.1) {
            let Some(source_level) = game.find_player(player_id).map(CPlayer::level) else { return; };
            let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
            let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else { return; };
            let keep = scaled_state_time(source_level, target_level, properties.query_property(STATE_TIME));
            if keep != 0 { add_state(game, player_id, target, keep, runtime); }
        }
    });
    if let Some(kernel) = game.player_skill_execution_mut(player_id, STRIKE_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
