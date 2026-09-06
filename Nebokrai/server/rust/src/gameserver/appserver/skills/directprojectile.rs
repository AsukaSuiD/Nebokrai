//! Общий player-путь прямых снарядов `CChuckStone` и `CSkeletonArchery`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий жизненный
//! цикл: проверку лука или арбалета, задержку с запретом движения, повторный
//! расчёт пути перед выстрелом, остановку на первой непролётной либо занятой
//! клетке и удар по всем допустимым целям этой клетки в живом порядке региона.
//! Cell-impact делает RTTI-переход `CShape -> CMoveShape`: для каждого игрока,
//! NPC или монстра, кроме стрелка, сначала рассчитывается собственная атака и
//! только затем вызывается `Defense`. Поэтому даже отклонённая защита сохраняет
//! RNG-порядок; у NPC `GetHP == 0`, и `CFightDefense` завершает вызов без урона.
//! Конкретные владельцы задают только идентификатор навыка. `CGame` разрешает
//! владельцев, применяет рассчитанные удары и доставляет готовые пакеты.
//! Оба владельца вычисляют критический множитель в расширенной точности x87 и
//! усекают его к нулю при записи в `int`.
//! Reuse использует exact `CSkill::IsRestored`; cast и flight часы — elapsed.
//! Только ChuckStone выставляет prepared после fire (0x0053DE95).
//! SkeletonArchery::AI (0x005393A0) пишет fired в +0x58 (0x005396EF),
//! но не prepared в +0x44: его полёт остаётся в активном AI. Поэтому
//! одинаковая траектория не означает одинаковый переход в фон.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::flash::{cell_views, master_info};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::poisonmoth::{MONSTER_TYPE, PLAYER_TYPE};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const TARGET_MIN_DISTANCE: u32 = 5_004;
const MISSILE_FLYING_TIME: u32 = 10_008;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const NPC_TYPE: i32 = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerDirectProjectileExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    fired: bool,
    destination: (i32, i32),
    impact: (i32, i32),
    missile_flying_time_ms: u32,
}

impl PlayerDirectProjectileExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), condition_checked: false, fired: false, destination, impact: (0, 0), missile_flying_time_ms: 0 }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn skill_id(&self) -> u32 { skill_id(self.kernel.dispatch()) }
}

const fn skill_id(dispatch: PlayerSkillDispatch) -> u32 {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_direct_projectile_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(skill_id(dispatch), super::chuckstone::CHUCK_STONE_SKILL_ID | super::skeletonarchery::SKELETON_ARCHERY_SKILL_ID)
        && !matches!(dispatch, PlayerSkillDispatch::SelfTarget { .. })
}

fn destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn current_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch, fallback: (i32, i32)) -> (i32, i32) {
    destination(game, region_id, dispatch).unwrap_or(fallback)
}

fn live_object_target_dead(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> bool {
    object_target(dispatch).is_some_and(|target| {
        game.base_magic_target_view(region_id, target).is_some()
            && game.periodic_state_target_dead(region_id, target)
    })
}

fn object_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None }
}

fn send_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action); }

fn send_start(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, delay_ms: u32) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1); message.add_long(skill_id as i32); message.add_short(level as i16);
    message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(direction); message.add_ulong(delay_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, target: Option<ShapeIdentity>, impact: (i32, i32), flying_time_ms: u32) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2); message.add_long(skill_id as i32); message.add_short(level as i16);
    message.add_long(PLAYER_TYPE); message.add_long(player_id);
    message.add_long(target.map_or(0, |identity| identity.object_type)); message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(impact.0); message.add_long(impact.1); message.add_ulong(flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
}

fn finish<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, skill_id: u32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_movement(game, player_id);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(skill_id, now_ms));
}

fn abort(game: &mut CGame, player_id: i32) { restore_movement(game, player_id); }

pub(super) fn abort_player_direct_projectile_on_region_change(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    expected_skill_id: u32,
) -> bool {
    let Some((dispatch, skill_id)) = ai
        .player_skill_state::<PlayerDirectProjectileExecutionState>(expected_skill_id)
        .map(|state| (state.kernel().dispatch(), state.skill_id()))
    else {
        return false;
    };
    if skill_id != expected_skill_id {
        return false;
    }
    abort(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn cancel_player_direct_projectile<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, execution_skill_id: u32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some((dispatch, skill_id)) = ai.player_skill_state::<PlayerDirectProjectileExecutionState>(execution_skill_id).map(|state| (state.kernel().dispatch(), state.skill_id())) else { return false };
    finish(game, player_id, skill_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn calculate_attack(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, hit_modifier: i32) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master) = game.find_player(player_id).map(|player| (player.combat_properties(), master_info(player)))?;
    let minimum = combat.minimum_attack as i32;
    let span = (combat.maximum_attack as i32).wrapping_sub(minimum).wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let mut attack = AttackInformation {
        skill_id, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id, hit_modifier, damage_factor: 1.0,
        damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    Some((master, attack))
}

fn attack_impact<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, skill_id: u32, level: i32, hit_modifier: i32, impact: (i32, i32), runtime: &mut Runtime) {
    if impact == (0, 0) { return }
    for view in cell_views(game, region_id, impact.0, impact.1) {
        let target = view.identity;
        if (target.object_type == PLAYER_TYPE && target.id == player_id)
            || !matches!(target.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE)
        { continue }
        let Some((master, attack)) = calculate_attack(game, player_id, skill_id, level, hit_modifier) else { continue };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
            // Точный `CNpc::GetHP == 0` завершает `CFightDefense::Defense`
            // сразу после уже выполненного CalculateAttackPower.
            NPC_TYPE => {}
            _ => {}
        }
    }
}

pub(crate) fn execute_player_direct_projectile<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_direct_projectile_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let skill_id = skill_id(dispatch);
    let Some((region_id, source_x, source_y, level)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(skill_id)))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let minimum = properties.query_property(TARGET_MIN_DISTANCE).max(1);
    let missile_unit_ms = properties.query_property(MISSILE_FLYING_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();

    if ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).is_none() {
        if !skill_is_restored(ai.skill_last_used_ms(skill_id), reuse_ms, now_ms) {
            send_failure(game, player_id, 0x0d); return terminal(QueuedSkillExecutionState::Rejected)
        }
        let Some(target_position) = destination(game, region_id, dispatch) else { send_failure(game, player_id, 10); return terminal(QueuedSkillExecutionState::Rejected) };
        let path = game.base_magic_path(region_id, source_x, source_y, target_position.0, target_position.1, None);
        if maximum != 0 && path.len() > maximum.wrapping_add(1) as usize { send_failure(game, player_id, 0x0b); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        match player.equipment().get_goods(2).map(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1)) {
            None => { send_failure(game, player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0283"); return terminal(QueuedSkillExecutionState::Rejected) }
            Some(3 | 4) => {}
            Some(_) => { send_failure(game, player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0284"); return terminal(QueuedSkillExecutionState::Rejected) }
        }
        if live_object_target_dead(game, region_id, dispatch) { send_failure(game, player_id, 10); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); }
        ai.begin_player_skill_execution(PlayerDirectProjectileExecutionState::begin(dispatch, target_position, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).is_none_or(|state| state.kernel().dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if live_object_target_dead(game, region_id, dispatch) {
        send_failure(game, player_id, 10); abort(game, player_id); return terminal(QueuedSkillExecutionState::Rejected)
    }
    if ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).is_some_and(|state| !state.condition_checked) {
        let fallback = ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).map(|state| state.destination).unwrap_or((0, 0));
        let target_position = current_destination(game, region_id, dispatch, fallback);
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_position.0, target_position.1)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_start(game, player_id, skill_id, level, delay_ms);
        if let Some(state) = ai.player_skill_state_mut::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()) { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started_at_ms = ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).map(|state| state.kernel().started_at_ms()).unwrap_or_default();
    if ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).is_some_and(|state| !state.fired) {
        if !time_reached(now_ms, started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
        restore_movement(game, player_id);
        let fallback = ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).map(|state| state.destination).unwrap_or((0, 0));
        let target_position = current_destination(game, region_id, dispatch, fallback);
        let path = game.base_magic_path(region_id, source_x, source_y, target_position.0, target_position.1, None);
        if (maximum != 0 && path.len() > maximum.wrapping_add(1) as usize) || (minimum != 0 && path.len() < minimum as usize) {
            send_failure(game, player_id, 0x0b); abort(game, player_id); return terminal(QueuedSkillExecutionState::Rejected)
        }
        let Some(master) = game.find_player(player_id).map(master_info) else { abort(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        let mut path_index = path.len();
        let mut impact = target_position;
        let mut visual_target = object_target(dispatch);
        for (index, &(x, y, block)) in path.iter().enumerate() {
            let blocking_target = if block == BLOCK_SHAPE {
                cell_views(game, region_id, x, y).into_iter().map(|view| view.identity).find(|identity| {
                    !(identity.object_type == PLAYER_TYPE && identity.id == player_id)
                        && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                        && game.owned_player_skill_target_attackable(master, *identity, region_id)
                })
            } else { None };
            if block == BLOCK_UNFLY || blocking_target.is_some() {
                path_index = index; impact = (x, y); visual_target = blocking_target; break;
            }
        }
        let flying_time_ms = missile_unit_ms.wrapping_mul(path_index as u32);
        send_fire(game, player_id, skill_id, level, visual_target, impact, flying_time_ms);
        if let Some(state) = ai.player_skill_state_mut::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()) {
            state.fired = true; state.destination = target_position; state.impact = impact; state.missile_flying_time_ms = flying_time_ms;
            if skill_id == super::chuckstone::CHUCK_STONE_SKILL_ID {
                state.kernel_mut().mark_prepared();
            }
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let Some(state) = ai.player_skill_state::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()).copied() else { return terminal(QueuedSkillExecutionState::Rejected) };
    if !time_reached(now_ms, started_at_ms, delay_ms.wrapping_add(state.missile_flying_time_ms)) { return terminal(QueuedSkillExecutionState::Pending) }
    attack_impact(game, player_id, region_id, skill_id, level, hit_modifier, state.impact, runtime);
    if let Some(state) = ai.player_skill_state_mut::<PlayerDirectProjectileExecutionState>(dispatch.skill_id()) { let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish(game, player_id, skill_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
