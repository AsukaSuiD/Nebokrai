//! Рыцарский удар `CKnightCut` (`0x67`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knightcut.cpp`. Навык сохраняет частичную трату MP до
//! проверки RP и повторной проверки оружия, полный scope 5×5 вокруг клетки
//! перед игроком и порядок `x → y → региональный порядок`. `reank` атакующего
//! вычитается из длительности каждого состояния. `CGame` используется только
//! для разрешения независимых владельцев, PK-перехода, `ForceMove` и доставки.
//! Завершение использует подтверждённый общий хвост `CSummonSkill::End(1)`.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::cure::CURE_SKILL_ID;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::knightcutstate::{KnightCutState, replace_monster_knight_cut_state, replace_player_knight_cut_state};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const KNIGHT_CUT_SKILL_ID: u32 = 0x67;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;
const TIME_PERCENT: u32 = 15_005;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnightCutExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    direction: i32,
}

impl KnightCutExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), direction: -1 }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy)]
struct Target { identity: ShapeIdentity, x: i32, y: i32, level: u8, dead: bool, cured: bool }

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_knight_cut_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == KNIGHT_CUT_SKILL_ID,
    }
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(),
        master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn weapon_is_compatible(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        matches!(weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1), 1 | 2)
    })
}

fn send_failure(game: &CGame, player_id: i32, code: u8, amount: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0308"),
        _ => {}
    }
}

fn finish_player_knight_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_knight_cut_used(now_ms);
    });
}

pub(crate) fn cancel_player_knight_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.knight_cut().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_knight_cut(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn destination(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(|player| { let face = player.shape().get_face_position().ok()?; Some((face.x, face.y)) }),
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, direction: i32, apply: bool) {
    let Some((identity, source)) = game.find_player(player_id).map(|player| (player.shape().identity(), player.shape().clone())) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(if apply { 2 } else { 1 });
    message.add_long(KNIGHT_CUT_SKILL_ID as i32); message.add_short(level as i16);
    message.add_long(identity.object_type); message.add_long(identity.id);
    if apply {
        let front = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: source.get_tile_x().unwrap_or_default(), y: source.get_tile_y().unwrap_or_default() }).unwrap_or(ShapeAreaCoordinates { x: source.get_tile_x().unwrap_or_default(), y: source.get_tile_y().unwrap_or_default() });
        message.add_long(0); message.add_long(0); message.add_long(front.x); message.add_long(front.y);
    } else { message.add_long(direction); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<Target> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            (player.server_region_id() == Some(region_id)).then_some(Target { identity, x: player.shape().get_tile_x().ok()?, y: player.shape().get_tile_y().ok()?, level: player.level(), dead: player.is_dead(), cured: player.has_state_by_skill_id(CURE_SKILL_ID) })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(Target { identity, x: monster.move_shape().shape().get_tile_x().ok()?, y: monster.move_shape().shape().get_tile_y().ok()?, level: property.level as u8, dead: monster.hit_points() == 0, cured: monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID) })
        }
        _ => None,
    }
}

fn scope_targets(game: &CGame, region_id: i32, player_id: i32, direction: i32) -> Vec<ShapeIdentity> {
    let Some(source) = game.find_player(player_id).map(|player| player.shape().clone()) else { return Vec::new() };
    let position = ShapeAreaCoordinates { x: source.get_tile_x().unwrap_or_default(), y: source.get_tile_y().unwrap_or_default() };
    let Ok(front) = CShape::get_direction_position(direction, position) else { return Vec::new() };
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions(); let mut targets = Vec::new();
    for offset_x in 0..5 { for offset_y in 0..5 {
        let mut shapes = Vec::new();
        if region.get_shapes(front.x.wrapping_sub(2).wrapping_add(offset_x), front.y.wrapping_sub(2).wrapping_add(offset_y), area_width, area_height, game, &mut shapes).is_ok() {
            targets.extend(shapes.into_iter().map(|shape| shape.identity));
        }
    }}
    targets
}

fn safe_player_pair(game: &CGame, region_id: i32, source_x: i32, source_y: i32, target_x: i32, target_y: i32) -> bool {
    game.find_region(region_id).is_some_and(|region| {
        region.get_security(source_x, source_y).ok() == Some(RegionSecurity::SAFE)
            || region.get_security(target_x, target_y).ok() == Some(RegionSecurity::SAFE)
    })
}

fn knockback(game: &CGame, region_id: i32, source_x: i32, source_y: i32, target: Target, steps: u32) -> Option<(i32, i32, u32)> {
    let region = game.find_region(region_id)?.base(); let direction = get_line_direction(source_x, source_y, target.x, target.y);
    let mut position = ShapeAreaCoordinates { x: target.x, y: target.y }; let mut moved = 0_u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break };
        if region.block_at(next.x, next.y).is_none_or(|block| block & 7 != 0) { break }
        position = next; moved = moved.wrapping_add(1);
    }
    Some((position.x, position.y, moved))
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет состояние и ForceMove одной цели")]
fn apply_target<Runtime: GameMainLoopRuntime>(game: &mut CGame, runtime: &mut Runtime, region_id: i32, player_id: i32, target: Target, state: KnightCutState, destination: (i32, i32, u32), move_speed: u32, now_ms: u32) {
    let installed = match target.identity.object_type {
        PLAYER_TYPE => replace_player_knight_cut_state(game, target.identity.id, state, now_ms),
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id); let installed = owner.as_mut().is_some_and(|owner| replace_monster_knight_cut_state(game, owner.base_mut(), target.identity.id, state, now_ms));
            if let Some(owner) = owner { game.restore_region_owner(owner); } installed
        }
        _ => false,
    };
    if !installed { return }
    game.enter_player_combat_state(player_id);
    let Some(mut owner) = game.take_region_owner(region_id) else { return };
    let _ = game.force_move_owned_shape(owner.base_mut(), target.identity, destination.0, destination.1, move_speed.wrapping_mul(destination.2), runtime);
    game.restore_region_owner(owner);
}

pub(crate) fn execute_player_knight_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_knight_cut_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, level, source_level, source_x, source_y, initial_mana, initial_rp)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(KNIGHT_CUT_SKILL_ID), player.level(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(), player.rp()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(KNIGHT_CUT_SKILL_ID, level) else { if player_ai.knight_cut().is_some() { finish_player_knight_cut(game, player_id, player_ai, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let rp_loss = properties.query_property(USER_RP_LOSE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let persist = properties.query_property(STATE_PERSIST_TIME); let time_percent = properties.query_property(TIME_PERCENT);
    let back_steps = properties.query_property(TARGET_BACK_STEP); let move_speed = properties.query_property(TARGET_MOVE_SPEED);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.knight_cut().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.knight_cut_last_used_ms() != 0 && !time_reached(now_ms, player_ai.knight_cut_last_used_ms(), reuse) { send_failure(game, player_id, 0x0d, 0); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_compatible(game, player) { send_failure(game, player_id, 0x0e, 0); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if rp_loss != 0 && (u32::from(initial_rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(KNIGHT_CUT_SKILL_ID)); }
        player_ai.begin_knight_cut(KnightCutExecutionState::begin(dispatch, now_ms));
    } else if player_ai.knight_cut().is_none_or(|execution| execution.kernel().dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if player_ai.knight_cut().is_some_and(|execution| execution.kernel().stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); finish_player_knight_cut(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss); finish_player_knight_cut(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_rp(rp.wrapping_sub(rp_loss as u16)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_compatible(game, player)) { send_failure(game, player_id, 0x0e, 0); finish_player_knight_cut(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let (target_x, target_y) = destination(game, region_id, player_id, dispatch).unwrap_or((source_x, source_y));
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(direction); }
        if let Some(execution) = player_ai.knight_cut_mut() { execution.direction = direction; let _ = execution.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
        send_visual(game, player_id, level, direction, false);
    }
    let Some(execution) = player_ai.knight_cut() else { return terminal(QueuedSkillExecutionState::Rejected) };
    if !time_reached(runtime.now_milliseconds(), execution.kernel().started_at_ms(), delay) { return terminal(QueuedSkillExecutionState::Pending) }
    let direction = execution.direction; send_visual(game, player_id, level, direction, true);
    if let Some(execution) = player_ai.knight_cut_mut() { let _ = execution.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = execution.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); }
    let master = game.find_player(player_id).map(master_info).unwrap_or_default();
    let reank = game.find_player(player_id).map_or(0, |player| u32::from(player.combat_properties().reank));
    for identity in scope_targets(game, region_id, player_id, direction) {
        if identity == (ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: Default::default() }) { continue }
        let Some(target) = target_snapshot(game, region_id, identity) else { continue };
        if target.dead || target.cured || target.level >= source_level || !game.owned_player_skill_target_attackable(master, identity, region_id) { continue }
        if identity.object_type == PLAYER_TYPE {
            if safe_player_pair(game, region_id, source_x, source_y, target.x, target.y) { continue }
            let _ = game.player_on_first_skill(player_id, identity.id, Some(region_id), runtime);
        }
        let raw_duration = if identity.object_type == PLAYER_TYPE { persist } else { (persist as f32 * time_percent as f32 * 0.01).round_ties_even() as u32 };
        let reduced = raw_duration.wrapping_sub(reank); let duration = if (reduced as i32) < 0 { 0 } else { reduced };
        let Some(destination) = knockback(game, region_id, source_x, source_y, target, back_steps) else { continue };
        let now_ms = runtime.now_milliseconds();
        apply_target(game, runtime, region_id, player_id, target, KnightCutState::new(now_ms, duration), destination, move_speed, now_ms);
    }
    if let Some(execution) = player_ai.knight_cut_mut() { let _ = execution.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_knight_cut(game, player_id, player_ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
