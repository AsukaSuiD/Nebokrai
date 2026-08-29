//! Молния `CLightning` (`0x133`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lightning.cpp`. Владелец сохраняет две проверки MP,
//! время восстановления, повторную проверку расстояния после задержки,
//! точные пакеты начала и удара и непосредственное применение атаки в том же
//! такте. Формула сохраняет два исходных RNG-вызова: разброс элементального
//! урона, затем критический удар. `CGame` только разрешает владельцев цели и
//! применяет рассчитанную атаку через общую защиту. Координатные перегрузки
//! `Begin` остаются ниже недостигнутыми.
//! `End(1)` сбрасывает execution-состояние и завершает успешную атаку;
//! `End(0)` очищает отменённую команду без обновления свойств и cooldown.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LIGHTNING_SKILL_ID: u32 = 0x133;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LightningExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
    attacking_started: bool,
}

impl LightningExecutionState {
    const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
            condition_checked: false,
            attacking_started: false,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    const fn condition_checked(self) -> bool {
        self.condition_checked
    }

    fn mark_condition_checked(&mut self) {
        self.condition_checked = true;
    }

    const fn attacking_started(self) -> bool {
        self.attacking_started
    }

    fn mark_attacking_started(&mut self) {
        self.attacking_started = true;
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(LIGHTNING_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    level: i32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(LIGHTNING_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_cancel(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(3);
    message.add_long(LIGHTNING_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_lightning_used(now_ms);
    });
}

fn abort_player_lightning(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    abort_skill(game, player_id);
}

pub(crate) fn cancel_player_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.lightning().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    abort_player_lightning(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn target_dead(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => true,
    }
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
    let width = if width_delta < 0 {
        width_delta.wrapping_neg()
    } else {
        width_delta
    }
    .wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let element_bonus = ((element_modifier as f32)
        * 0.01
        * (combat.element_modify as f32))
        .round_ties_even() as i32;
    let element_damage = element_bonus
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(minimum)
        .max(0);
    let mut attack = AttackInformation {
        skill_id: LIGHTNING_SKILL_ID,
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
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: element_damage,
            mp_damage: 0,
        }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                ((power.hp_damage as f32) * critical_rate).round_ties_even() as i32;
        }
    }
    Some((master, attack))
}

pub(crate) const fn is_lightning_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Object {
            skill_id: LIGHTNING_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
        }
    )
}

pub(crate) fn execute_player_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let target = match dispatch {
        PlayerSkillDispatch::Object {
            skill_id: LIGHTNING_SKILL_ID,
            target,
        } => target,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, source_x, source_y, level, initial_mana)) =
        game.find_player(player_id).and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(LIGHTNING_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(LIGHTNING_SKILL_ID, level) else {
        if player_ai.lightning().is_some() {
            abort_player_lightning(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_modifier = properties.query_property(TARGET_FINAL_DAMAGE_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.lightning().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if player_ai.lightning_last_used_ms() != 0
            && !time_reached(
                runtime.now_milliseconds(),
                player_ai.lightning_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
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
            player.set_current_skill_id(Some(LIGHTNING_SKILL_ID));
        }
        player_ai.begin_lightning(LightningExecutionState::begin(
            dispatch,
            target,
            started_at_ms,
        ));
    } else if player_ai
        .lightning()
        .is_none_or(|state| state.kernel().dispatch() != dispatch || state.target != target)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .lightning()
        .is_some_and(|state| !state.condition_checked())
    {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target_view.tile_x,
                target_view.tile_y,
            ));
            player.set_skill_moveable(false);
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        send_start(game, player_id, level);
        if let Some(state) = player_ai.lightning_mut() {
            state.mark_condition_checked();
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .lightning()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение молнии создано или восстановлено");
    if player_ai
        .lightning()
        .is_some_and(|state| !state.attacking_started())
    {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10);
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if target_dead(game, region_id, target) {
            send_failure(game, player_id, 10);
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((current_source_x, current_source_y)) =
            game.find_player(player_id).and_then(|player| {
                Some((
                    player.shape().get_tile_x().ok()?,
                    player.shape().get_tile_y().ok()?,
                ))
            })
        else {
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            current_source_x,
            current_source_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            abort_player_lightning(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        send_fire(
            game,
            player_id,
            target,
            target_view.tile_x,
            target_view.tile_y,
            level,
        );
        if let Some(state) = player_ai.lightning_mut() {
            state.mark_attacking_started();
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(master) = game.find_player(player_id).map(master_info) else {
        abort_player_lightning(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target_dead(game, region_id, target)
        || !game.owned_player_skill_target_attackable(master, target, region_id)
    {
        send_cancel(game, player_id, level);
        abort_player_lightning(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((master, attack)) = calculate_attack(
        game,
        player_id,
        level,
        minimum,
        maximum,
        element_modifier,
        hit_modifier,
        damage_modifier,
    ) else {
        abort_player_lightning(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    match target.object_type {
        PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        _ => {}
    }
    if let Some(state) = player_ai.lightning_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_lightning(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

// Остаются недостигнутыми координатные перегрузки запуска навыка.
// ============================================================================
// FUNCTION: CLightning::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lightning.cpp:157
// RVA: 0x001AC130
// ADDRESS: 005ac130
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
// ============================================================================
// FUNCTION: CLightning::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lightning.cpp:176
// RVA: 0x001AC200
// ADDRESS: 005ac200
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
