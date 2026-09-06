//! Трёхлучевой выстрел `CRainArrow` (`0xCE`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rainarrow.cpp`. Модуль владеет проверками, MP, углом,
//! построением и обрезкой трёх путей, сообщением выстрела и созданием phalanx. MP
//! списывается до поздней проверки лука без отката; `CGame` только разрешает
//! владельцев, регистрирует форму и доставляет построенные сообщения. Общий
//! `End` (0x0058DE90) очищает все три пути и возвращает движение, затем
//! передаёт исходный аргумент CStateSkill::End (0x005DFBD0). Только End(1)
//! вызывает AfterUseSkill (+0x8C, износ оружия) и фиксирует cooldown.
//! Обычный AI после Summon вызывает End(0) (0x0058EDD8): phalanx остаётся
//! в регионе, но выстрел не изнашивает оружие и не обновляет reuse.
//! m_bSkillPrepared выставлен лишь на синхронный Summon и сбрасывается End;
//! отдельного фонового исполнения этого навыка между AI-тактами нет.
//! Cooldown использует абсолютный
//! срок `CSkill::IsRestored`; задержка исполнения остаётся elapsed.

use super::baseattack::{real_distance, time_reached, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::stateskill::finish_state_skill;
use super::rainarrowphalanx::{CRainArrowPhalanx, RainArrowCell, RAIN_ARROW_SKILL_ID};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase,
    QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400; const MONSTER_TYPE: i32 = 600; const USER_MP_LOSE: u32 = 2;
const MINIMUM_ANGLE: u32 = 2_002; const MAXIMUM_ANGLE: u32 = 2_003;
const MISSILE_FLYING_TIME: u32 = 10_008; const DAMAGE_FACTOR: u32 = 20_003;
const SUMMONED_LIFETIME: u32 = 30_001; const SUMMONED_SPEED: u32 = 30_002;

#[derive(Clone, Debug, Eq, PartialEq)] pub(crate) struct RainArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>, destination: (i32, i32), target: Option<ShapeIdentity>,
    condition_checked: bool, angle_bits: u32, left: Vec<RainArrowCell>, center: Vec<RainArrowCell>, right: Vec<RainArrowCell>,
}
impl RainArrowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), target: Option<ShapeIdentity>, now: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now), destination, target, condition_checked: false,
            angle_bits: 0.0f32.to_bits(), left: Vec::new(), center: Vec::new(), right: Vec::new() }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}
fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn restore_player_movement(game: &mut CGame, id: i32) { if let Some(player) = game.find_player_mut(id) { player.set_skill_moveable(true); } }
fn finish_player_rain_arrow<R: GameMainLoopRuntime>(game: &mut CGame, id: i32, ai: &mut CPlayerAI, runtime: &mut R) {
    restore_player_movement(game, id);
    finish_state_skill(game, id, ai, runtime, |ai, now_ms| ai.mark_skill_used(crate::gameserver::appserver::skills::rainarrowphalanx::RAIN_ARROW_SKILL_ID, now_ms));
}
fn abort_player_rain_arrow(game: &mut CGame, id: i32) { restore_player_movement(game, id); abort_skill(game, id); }
pub(crate) fn complete_player_rain_arrow<R: GameMainLoopRuntime>(game: &mut CGame, id: i32, ai: &mut CPlayerAI, runtime: &mut R) -> bool {
    let Some(dispatch) = ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_rain_arrow(game, id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_rain_arrow<R: GameMainLoopRuntime>(game: &mut CGame, id: i32, ai: &mut CPlayerAI, _runtime: &mut R) -> bool {
    let Some(dispatch) = ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_rain_arrow(game, id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn weapon_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3) }
fn master(player: &CPlayer) -> MasterInfo { let p = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(),
    master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
    permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate),
    permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) } }
fn target_snapshot(game: &CGame, region: i32, target: ShapeIdentity) -> Option<(i32, i32, bool)> { match target.object_type {
    PLAYER_TYPE => game.find_player(target.id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.is_dead()))),
    MONSTER_TYPE => game.find_region(region).and_then(|r| { let m = r.base().find_monster_by_id(target.id)?; Some((m.move_shape().shape().get_tile_x().ok()?, m.move_shape().shape().get_tile_y().ok()?, m.hit_points() == 0)) }), _ => None } }
fn send_start(game: &mut CGame, id: i32, level: i32) { let Some(player) = game.find_player(id) else { return }; let mut m = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    m.add_byte(1); m.add_long(RAIN_ARROW_SKILL_ID as i32); m.base_mut().add_short(level as i16); m.add_long(PLAYER_TYPE); m.add_long(id); m.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(id, None, &m); }
fn impact(path: &[RainArrowCell], count: usize, fallback: (i32, i32)) -> (i32, i32) { count.checked_sub(1).and_then(|i| path.get(i)).map_or(fallback, |c| (c.0, c.1)) }
fn send_fire(game: &mut CGame, id: i32, level: i32, target: Option<ShapeIdentity>, destination: (i32, i32),
    right: (i32, i32), left: (i32, i32), flight: u32) { let t = target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: CGuid::GUID_INVALID });
    let mut m = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE); m.add_byte(2); m.add_long(RAIN_ARROW_SKILL_ID as i32); m.base_mut().add_short(level as i16);
    m.add_long(PLAYER_TYPE); m.add_long(id); m.add_long(t.object_type); m.add_long(t.id); m.add_long(destination.0); m.add_long(destination.1);
    m.add_long(right.0); m.add_long(right.1); m.add_long(left.0); m.add_long(left.1); m.add_ulong(flight); let _ = game.send_player_shape_around(id, None, &m); }
fn round_up(value: f32) -> i32 { let integer = value as i32; if value - integer as f32 > 0.5 { integer.wrapping_add(1) } else { integer } }
fn rotate(x: i32, y: i32, angle: f32) -> (i32, i32) { let radians = angle * 0.005_555_555_7 * core::f32::consts::PI;
    let (sin, cos) = radians.sin_cos(); (round_up(cos * x as f32 - sin * y as f32), round_up(cos * y as f32 + sin * x as f32)) }
fn build_paths(game: &CGame, region: i32, source: (i32, i32), destination: (i32, i32), maximum: u32, angle: f32)
    -> (Vec<RainArrowCell>, Vec<RainArrowCell>, Vec<RainArrowCell>, (i32, i32)) {
    let center = game.base_magic_path(region, source.0, source.1, destination.0, destination.1, (maximum != 0).then_some(maximum));
    let Some(first) = center.first().copied() else { return (Vec::new(), center, Vec::new(), destination) }; let last = center.last().copied().unwrap_or(first);
    let delta = (last.0.wrapping_sub(first.0), last.1.wrapping_sub(first.1)); let left_delta = rotate(delta.0, delta.1, angle); let right_delta = rotate(delta.0, delta.1, -angle);
    let side_length = if maximum == 0 { 0 } else { maximum - 1 }; let mut left = vec![first]; let mut right = vec![first];
    left.extend(game.base_magic_path(region, first.0, first.1, first.0.wrapping_add(left_delta.0), first.1.wrapping_add(left_delta.1), Some(side_length)).into_iter().skip(1));
    right.extend(game.base_magic_path(region, first.0, first.1, first.0.wrapping_add(right_delta.0), first.1.wrapping_add(right_delta.1), Some(side_length)).into_iter().skip(1));
    (left, center, right, (last.0, last.1))
}
fn usable(path: &[RainArrowCell]) -> (usize, Option<(i32, i32)>) { for (i, &(x, y, block)) in path.iter().enumerate() { if block == 2 { return (i, Some((x, y))) } } (path.len(), None) }
pub(crate) const fn is_rain_arrow_dispatch(d: PlayerSkillDispatch) -> bool { matches!(d,
    PlayerSkillDispatch::SelfTarget { skill_id: RAIN_ARROW_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: RAIN_ARROW_SKILL_ID, .. }
    | PlayerSkillDispatch::Object { skill_id: RAIN_ARROW_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }

pub(crate) fn execute_player_rain_arrow<R: GameMainLoopRuntime>(game: &mut CGame, id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut R) -> QueuedSkillExecutionOutcome {
    if !is_rain_arrow_dispatch(dispatch) { return outcome(QueuedSkillExecutionState::Rejected) }
    let Some((region, level, source, direction)) = game.find_player(id).and_then(|p| Some((p.server_region_id()?, p.learned_skill_level(RAIN_ARROW_SKILL_ID),
        (p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?), p.shape().get_direction()))) else { return outcome(QueuedSkillExecutionState::Rejected) };
    let Some(props) = game.skill_base_properties(RAIN_ARROW_SKILL_ID, level) else { if ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().is_some() { abort_player_rain_arrow(game, id); } return outcome(QueuedSkillExecutionState::Rejected) };
    let mp = props.query_property(USER_MP_LOSE); let reuse = props.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = props.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum = props.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let min_angle = props.query_property(MINIMUM_ANGLE); let max_angle = props.query_property(MAXIMUM_ANGLE);
    let flight = props.query_property(MISSILE_FLYING_TIME); let lifetime = props.query_property(SUMMONED_LIFETIME); let speed = props.query_property(SUMMONED_SPEED) as i32;
    let hit = props.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let factor = props.query_property(DAMAGE_FACTOR) as i32; let _breakable = props.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().is_none() { let (destination, target) = match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => ((x, y), None),
        PlayerSkillDispatch::SelfTarget { .. } => { let p = CShape::get_direction_position(direction, crate::gameserver::appserver::shape::ShapeAreaCoordinates { x: source.0, y: source.1 }).ok(); (p.map_or(source, |p| (p.x, p.y)), None) },
        PlayerSkillDispatch::Object { target, .. } if target.object_type == PLAYER_TYPE && target.id == id => { let p = CShape::get_direction_position(direction, crate::gameserver::appserver::shape::ShapeAreaCoordinates { x: source.0, y: source.1 }).ok(); (p.map_or(source, |p| (p.x, p.y)), None) },
        PlayerSkillDispatch::Object { target, .. } => match target_snapshot(game, region, target) { Some((x, y, false)) => ((x, y), Some(target)), _ => { game.send_base_magic_failure(id, 10); game.send_skill_system_info(id, b"GS0285"); return outcome(QueuedSkillExecutionState::Rejected) } } };
        if !skill_is_restored(ai.skill_last_used_ms(crate::gameserver::appserver::skills::rainarrowphalanx::RAIN_ARROW_SKILL_ID), reuse, runtime.now_milliseconds()) { game.send_base_magic_failure(id, 0x0d); game.send_skill_system_info(id, b"GS0278"); return outcome(QueuedSkillExecutionState::Rejected) }
        let path = game.base_magic_path(region, source.0, source.1, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { game.send_base_magic_failure(id, 0x0b); game.send_skill_system_info(id, b"GS0290"); return outcome(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(id) else { return outcome(QueuedSkillExecutionState::Rejected) }; if !weapon_valid(game, player) { game.send_base_magic_failure(id, 0x0e); game.send_skill_system_info(id, b"GS0297"); return outcome(QueuedSkillExecutionState::Rejected) }
        if mp != 0 && (player.mana().wrapping_sub(mp) as i32) < 0 { game.send_base_magic_failure(id, 7); game.send_skill_system_info_with_unsigned(id, b"GS0288", mp); return outcome(QueuedSkillExecutionState::Rejected) }
        if let Some(p) = game.find_player_mut(id) { p.set_skill_moveable(false); p.set_current_skill_id(Some(RAIN_ARROW_SKILL_ID)); }
        ai.begin_player_skill_execution(RainArrowExecutionState::begin(dispatch, destination, target, runtime.now_milliseconds()));
    } else if ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().is_none_or(|s| s.kernel().dispatch() != dispatch) { return outcome(QueuedSkillExecutionState::Rejected) }
    let mut state = ai.player_skill_state::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID).cloned().expect("выполнение дождя стрел создано"); if let Some(target) = state.target { match target_snapshot(game, region, target) { Some((x, y, false)) => state.destination = (x, y), _ => { game.send_base_magic_failure(id, 10); game.send_skill_system_info(id, b"GS0285"); abort_player_rain_arrow(game, id); return outcome(QueuedSkillExecutionState::Rejected) } } }
    if !state.condition_checked { let mana = game.find_player(id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp) as i32) < 0 { game.send_base_magic_failure(id, 7); game.send_skill_system_info_with_unsigned(id, b"GS0288", mp); abort_player_rain_arrow(game, id); return outcome(QueuedSkillExecutionState::Rejected) }
        if let Some(p) = game.find_player_mut(id) { p.set_mana(mana.wrapping_sub(mp)); } let _ = game.update_player_current_state(id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(id).is_none_or(|p| !weapon_valid(game, p)) { game.send_base_magic_failure(id, 0x0e); game.send_skill_system_info(id, b"GS0297"); abort_player_rain_arrow(game, id); return outcome(QueuedSkillExecutionState::Rejected) }
        if let Some(p) = game.find_player_mut(id) { p.movement_shape_mut().set_direction(get_line_direction(source.0, source.1, state.destination.0, state.destination.1)); }
        let distance = real_distance(source.0, source.1, state.destination.0, state.destination.1).max(1) as u32; let distance = if maximum == 0 { distance } else { distance.min(maximum) };
        let divisor = if maximum == 0 { 1.0 } else { maximum as f32 }; let angle = max_angle as f32 - distance as f32 / divisor * max_angle.wrapping_sub(min_angle) as f32; state.angle_bits = angle.to_bits();
        (state.left, state.center, state.right, state.destination) = build_paths(game, region, source, state.destination, maximum, angle);
        // `CalculateFinalPosition` очищает sufferer сразу после снимка точки:
        // Сообщение выстрела и задержанный полёт больше не следуют за живой целью.
        state.target = None;
        state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); send_start(game, id, level); if let Some(s) = ai.player_skill_state_mut::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID) { *s = state.clone(); }
    }
    if !time_reached(runtime.now_milliseconds(), state.kernel().started_at_ms(), delay) { return outcome(QueuedSkillExecutionState::Pending) }
    if let Some(player) = game.find_player_mut(id) { player.set_skill_moveable(true); }
    let (center_count, block) = usable(&state.center); if let Some(block) = block { state.destination = block } let (left_count, _) = usable(&state.left); let (right_count, _) = usable(&state.right);
    let right_impact = impact(&state.right, right_count, source); let left_impact = impact(&state.left, left_count, source);
    send_fire(game, id, level, state.target, state.destination, right_impact, left_impact, flight);
    let Some(player) = game.find_player(id) else { abort_player_rain_arrow(game, id); return outcome(QueuedSkillExecutionState::Rejected) }; let master = master(player); let face = player.shape().get_face_position().ok();
    let summon_id = game.allocate_summon_shape_id(); let now = runtime.now_milliseconds(); let mut phalanx = CRainArrowPhalanx::new(summon_id, master, now, lifetime, level, hit, factor,
        state.left, left_count, state.center, center_count, state.right, right_count, speed, maximum as i32); phalanx.shape_mut().set_region_id(region);
    if let Some(face) = face { let result = game.add_rain_arrow_phalanx(region, phalanx, face.x, face.y, now, runtime); if result.is_some_and(|r| r.is_ok()) { let _ = game.send_rain_arrow_phalanx_entry(region, summon_id, runtime); } }
    if let Some(s) = ai.player_skill_state_mut::<RainArrowExecutionState>(RAIN_ARROW_SKILL_ID) { let _ = s.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = s.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = s.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    abort_player_rain_arrow(game, id); outcome(QueuedSkillExecutionState::Completed)
}
