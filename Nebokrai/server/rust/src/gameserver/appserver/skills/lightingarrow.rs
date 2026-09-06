//! Световая стрела `CLightingArrow` (`0xCB`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lightingarrow.cpp`. Модуль хранит execution pipeline,
//! проверки лука, MP, задержку и построение движущейся phalanx-формы. MP
//! списывается до повторной проверки оружия и не откатывается при позднем
//! отказе. `CGame` только разрешает владельцев, регистрирует форму и доставляет
//! точные сообщения `0xBFE01/0xBF502`. Cooldown использует абсолютный срок
//! `CSkill::IsRestored`; задержка и движение phalanx остаются elapsed.

use super::baseattack::time_reached;
use super::basemagic::{BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
    SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::lightingarrowphalanx::CLightingArrowPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase,
    QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LIGHTING_ARROW_SKILL_ID: u32 = 0xcb;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LightingArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>, destination: (i32, i32),
    target: Option<ShapeIdentity>, condition_checked: bool,
}
impl LightingArrowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), target: Option<ShapeIdentity>, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination, target, condition_checked: false }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}
fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
}
fn finish_player_lighting_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_skill_used(LIGHTING_ARROW_SKILL_ID, now_ms));
}
fn abort_player_lighting_arrow(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}
pub(crate) fn complete_player_lighting_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_lighting_arrow(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_lighting_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_lighting_arrow(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon|
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3)
}
fn master_info(player: &CPlayer) -> MasterInfo {
    let p = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(),
        master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: 0,
        permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate),
        permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) }
}
fn target_snapshot(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(i32, i32, bool)> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.is_dead()))),
        MONSTER_TYPE => game.find_region(region_id).and_then(|r| { let m = r.base().find_monster_by_id(target.id)?;
            Some((m.move_shape().shape().get_tile_x().ok()?, m.move_shape().shape().get_tile_y().ok()?, m.hit_points() == 0)) }),
        _ => None,
    }
}
fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE); message.add_byte(1);
    message.add_long(LIGHTING_ARROW_SKILL_ID as i32); message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_fire(game: &mut CGame, player_id: i32, level: i32, target: Option<ShapeIdentity>, x: i32, y: i32) {
    let target = target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: crate::public::guid::CGuid::GUID_INVALID });
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE); message.add_byte(2);
    message.add_long(LIGHTING_ARROW_SKILL_ID as i32); message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(target.object_type);
    message.add_long(target.id); message.add_long(x); message.add_long(y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(crate) const fn is_lighting_arrow_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::Point { skill_id: LIGHTING_ARROW_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: LIGHTING_ARROW_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } })
}

pub(crate) fn execute_player_lighting_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32,
    dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_lighting_arrow_dispatch(dispatch) { return outcome(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, level, source_x, source_y)) = game.find_player(player_id).and_then(|p|
        Some((p.server_region_id()?, p.learned_skill_level(LIGHTING_ARROW_SKILL_ID), p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?)))
        else { return outcome(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(LIGHTING_ARROW_SKILL_ID, level) else { if player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().is_some() { abort_player_lighting_arrow(game, player_id); } return outcome(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME); let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let hit = properties.query_property(crate::gameserver::appserver::skills::baseattack::SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().is_none() {
        let (destination, target) = match dispatch {
            PlayerSkillDispatch::Point { x, y, .. } => ((x, y), None),
            PlayerSkillDispatch::Object { target, .. } => match target_snapshot(game, region_id, target) {
                Some((x, y, _)) => ((x, y), Some(target)),
                _ => { game.send_base_magic_failure(player_id, 10); game.send_skill_system_info(player_id, b"GS0285"); return outcome(QueuedSkillExecutionState::Rejected); }
            }, _ => unreachable!(),
        };
        if !skill_is_restored(player_ai.skill_last_used_ms(LIGHTING_ARROW_SKILL_ID), reuse_ms, runtime.now_milliseconds()) {
            game.send_base_magic_failure(player_id, 0x0d); game.send_skill_system_info(player_id, b"GS0278"); return outcome(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None);
        if maximum != 0 && path.len() > maximum as usize { game.send_base_magic_failure(player_id, 0x0b); game.send_skill_system_info(player_id, b"GS0290"); return outcome(QueuedSkillExecutionState::Rejected); }
        let Some(player) = game.find_player(player_id) else { return outcome(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0297"); return outcome(QueuedSkillExecutionState::Rejected); }
        if mp_loss != 0 && (player.mana().wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); return outcome(QueuedSkillExecutionState::Rejected); }
        let now_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(LIGHTING_ARROW_SKILL_ID)); }
        player_ai.begin_player_skill_execution(LightingArrowExecutionState::begin(dispatch, destination, target, now_ms));
        return outcome(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().is_none_or(|s| s.kernel().dispatch() != dispatch) { return outcome(QueuedSkillExecutionState::Rejected); }

    let state = player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().expect("выполнение световой стрелы создано");
    let (mut destination, target) = (state.destination, state.target);
    if let Some(target) = target {
        match target_snapshot(game, region_id, target) {
            Some((x, y, false)) => destination = (x, y),
            _ => { game.send_base_magic_failure(player_id, 10); game.send_skill_system_info(player_id, b"GS0285"); abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected); }
        }
    }
    if !state.condition_checked {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|p| !weapon_is_valid(game, p)) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0297"); abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); }
        send_start(game, player_id, level);
        if let Some(state) = player_ai.player_skill_state_mut::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID) { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = player_ai.player_skill_state::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID).copied().expect("состояние сохранено").kernel().started_at_ms();
    if !time_reached(runtime.now_milliseconds(), started, delay_ms) { return outcome(QueuedSkillExecutionState::Pending); }
    restore_player_movement(game, player_id);
    let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None);
    if path.is_empty() { game.send_base_magic_failure(player_id, 2); abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected); }
    if maximum != 0 && path.len() > maximum as usize + 1 { game.send_base_magic_failure(player_id, 0x0b); game.send_skill_system_info(player_id, b"GS0290"); abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected); }
    send_fire(game, player_id, level, target, destination.0, destination.1);
    let Some(player) = game.find_player(player_id) else { abort_player_lighting_arrow(game, player_id); return outcome(QueuedSkillExecutionState::Rejected) };
    let master = master_info(player); let face = player.shape().get_face_position();
    let summon_id = game.allocate_summon_shape_id(); let now_ms = runtime.now_milliseconds();
    let mut phalanx = CLightingArrowPhalanx::new(summon_id, master, now_ms, lifetime, level, hit, factor, path, speed);
    phalanx.shape_mut().set_region_id(region_id);
    if let Ok(face) = face {
        let result = game.add_lighting_arrow_phalanx(region_id, phalanx, face.x, face.y, now_ms, runtime);
        if result.is_some_and(|result| result.is_ok()) { let _ = game.send_lighting_arrow_phalanx_entry(region_id, summon_id, runtime); }
    }
    if let Some(state) = player_ai.player_skill_state_mut::<LightingArrowExecutionState>(LIGHTING_ARROW_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_lighting_arrow(game, player_id, player_ai, runtime);
    outcome(QueuedSkillExecutionState::Completed)
}
