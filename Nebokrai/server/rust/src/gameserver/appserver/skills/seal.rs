//! Печать `CSeal` (`0x138`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/seal.cpp`. Достигнутый объектный путь сохраняет две
//! проверки MP, время восстановления, повторную проверку прямого пути после
//! задержки, запрет `BLOCK_UNFLY`, полёт снаряда и один исходный вызов RNG.
//! Формула, пакеты и жизненный цикл состояния принадлежат этому владельцу;
//! `CGame` только разрешает владельцев, применяет атаку и атомарно заменяет состояние
//! монстра. Координатный и пустой `Begin` доходят до владельца и сохраняют
//! исходный отказ `10 + GS0317`, затем общий отказ `2` и `End(0)`.
//! `End(1)` сбрасывает execution-состояние, возвращает движение и завершает
//! применение; отмена использует `End(0)` без обновления cooldown.
//! Стихийная прибавка сохраняет расширенное вычисление x87 из целых свойств и
//! сохранённой `f32`-константы, затем усекается к нулю. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; cast и полёт остаются elapsed.
//! Выпуск устанавливает общий prepared-флаг после эффекта 1
//! (0x005AAB8D). Последующий AI продолжает тот же
//! экземпляр в фоне; повторный Begin и отдельное хранилище не создаются.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::fightdefense::truncate_original;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::sealstate::SealState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const SEAL_SKILL_ID: u32 = 0x138;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const CURE_SKILL_ID: u32 = 0x131;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_CONST: u32 = 20_010;
const TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SealExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
    missile_flying_time_ms: u32,
}

impl SealExecutionState {
    const fn begin(dispatch: PlayerSkillDispatch, target: ShapeIdentity, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
            condition_checked: false,
            missile_flying_time_ms: 0,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> { self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn reject_begin(game: &CGame, player_id: i32) -> QueuedSkillExecutionOutcome {
    send_failure(game, player_id, 2);
    terminal(QueuedSkillExecutionState::Rejected)
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(SEAL_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "поля являются точным содержимым исходного пакета выстрела")]
fn send_fire(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    level: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(SEAL_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_seal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, SEAL_SKILL_ID, runtime);
}

fn abort_player_seal(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub(crate) fn complete_player_seal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_seal(game, player_id, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_seal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    abort_player_seal(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn target_snapshot(game: &CGame, region_id: i32, monster_id: i32) -> Option<(i32, i32, i32, Vec<u8>, bool)> {
    let owner = game.find_region(region_id)?;
    let monster = owner.base().find_monster_by_id(monster_id)?;
    let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
    Some((
        monster.move_shape().shape().get_tile_x().ok()?,
        monster.move_shape().shape().get_tile_y().ok()?,
        property.level as i32,
        monster.display_name().to_vec(),
        monster.hit_points() == 0,
    ))
}

#[allow(clippy::too_many_arguments, reason = "аргументы являются точными свойствами формулы навыка")]
fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
    damage_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let width_delta = maximum.wrapping_sub(minimum);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let element_bonus = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let element_damage = (combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(minimum)
        .wrapping_add(element_bonus)
        .max(0);
    Some((master, AttackInformation {
        skill_id: SEAL_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: 1.0,
        damage_modifier,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: element_damage, mp_damage: 0 }],
    }))
}

pub(crate) const fn is_seal_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget {
            skill_id: SEAL_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Point {
            skill_id: SEAL_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: SEAL_SKILL_ID,
            ..
        }
    )
}

pub(crate) fn execute_player_seal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let target = match dispatch {
        PlayerSkillDispatch::Object { skill_id: SEAL_SKILL_ID, target: target @ ShapeIdentity { object_type: MONSTER_TYPE, .. } } => target,
        _ => {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0317");
            return reject_begin(game, player_id);
        }
    };
    let Some((region_id, source_x, source_y, level, initial_mana, master)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?,
        player.shape().get_tile_x().ok()?,
        player.shape().get_tile_y().ok()?,
        player.learned_skill_level(SEAL_SKILL_ID, game.skill_factory()),
        player.mana(),
        master_info(player),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(SEAL_SKILL_ID, level) else {
        if game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().is_some() {
            abort_player_seal(game, player_id);
        }
        return reject_begin(game, player_id);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let missile_per_cell_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let state_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state_const = properties.query_property(SKILL_USAGE_CONST) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_modifier = properties.query_property(TARGET_FINAL_DAMAGE_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, SEAL_SKILL_ID), reuse_delay_ms, runtime.now_milliseconds()) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return reject_begin(game, player_id);
        }
        let Some((target_x, target_y, _, target_name, _)) = target_snapshot(game, region_id, target.id) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0317");
            return reject_begin(game, player_id);
        };
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return reject_begin(game, player_id);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0295", &target_name);
            return reject_begin(game, player_id);
        }
        if mp_loss == 0 { return reject_begin(game, player_id) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return reject_begin(game, player_id);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(SEAL_SKILL_ID));
            player.set_skill_moveable(false);
        }
        game.begin_player_skill_execution(player_id, player_ai, SealExecutionState::begin(dispatch, target, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().is_none_or(|state| state.kernel().dispatch() != dispatch || state.target != target) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target_level, target_name, target_dead)) =
        target_snapshot(game, region_id, target.id)
    else {
        abort_player_seal(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target_dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_seal(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if !game.owned_player_skill_target_attackable(master, target, region_id) {
        abort_player_seal(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().is_some_and(|state| !state.condition_checked) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_seal(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_start(game, player_id, level);
        if let Some(state) = game.player_skill_state_mut::<SealExecutionState>(player_id, SEAL_SKILL_ID) {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().map(|state| state.kernel().started_at_ms()).expect("выполнение печати создано или восстановлено");
    if game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().is_some_and(|state| !state.kernel().is_prepared()) {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
        let current_source_level = game.find_player(player_id).map_or(0, |player| i32::from(player.level()));
        if current_source_level.wrapping_add(10) < target_level {
            send_failure(game, player_id, 2);
            finish_player_seal(game, player_id, runtime);
            return terminal(QueuedSkillExecutionState::Completed);
        }
        let Some((current_x, current_y)) = game.find_player(player_id).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))) else {
            abort_player_seal(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(region_id, current_x, current_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            abort_player_seal(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0296", &target_name);
            abort_player_seal(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let missile_flying_time_ms = missile_per_cell_ms.wrapping_mul(path.len() as u32);
        send_fire(game, player_id, target, target_x, target_y, level, missile_flying_time_ms);
        if let Some(state) = game.player_skill_state_mut::<SealExecutionState>(player_id, SEAL_SKILL_ID) {
            state.kernel_mut().mark_prepared();
            state.missile_flying_time_ms = missile_flying_time_ms;
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let missile_flying_time_ms = game.player_skill_state::<SealExecutionState>(player_id, SEAL_SKILL_ID).copied().map_or(0, |state| state.missile_flying_time_ms);
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms.wrapping_add(missile_flying_time_ms)) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let has_cure = game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).is_some_and(|monster| monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID));
    if !has_cure {
        if let Some((master, attack)) = calculate_attack(game, player_id, level, minimum, maximum, element_modifier, hit_modifier, damage_modifier) {
            game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime);
            let current_source_level = game.find_player(player_id).map_or(0, |player| i32::from(player.level()));
            let multiplier = current_source_level
                .wrapping_sub(target_level)
                .wrapping_add(state_const)
                .max(1);
            let state_started_at_ms = runtime.now_milliseconds();
            let _ = runtime.now_milliseconds();
            let state_visual_at_ms = runtime.now_milliseconds();
            let state = SealState::new(
                state_started_at_ms,
                state_time_ms.wrapping_mul(multiplier as u32),
            );
            let _ = game.replace_owned_monster_seal_state(
                region_id,
                target.id,
                state,
                state_visual_at_ms,
            );
        }
    }
    if let Some(state) = game.player_skill_state_mut::<SealExecutionState>(player_id, SEAL_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_seal(game, player_id, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
