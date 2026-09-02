//! Предметный навык громового огня `CItemSkill_2` (`0x322`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/itemskill2.cpp`. Владелец сохраняет whitelist цели,
//! две проверки MP, задержку, cooldown одновременно навыка и расходника,
//! точную позицию stack-а и порядок: расход предмета, container-пакет,
//! `0xBF709`, регистрация `CThunderFirePhalanx`. `CGame` только связывает
//! независимых владельцев региона, игрока и сетевой доставки. `End(1)` сначала
//! возвращает движение, затем отдельными отсчётами фиксирует cooldown расходника
//! и навыка; предмет удаляется только успешным `Summon`, но не отменой.

use super::baseattack::{real_distance, time_reached, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::soulcollectstate::send_soul_collect_state_visual;
use super::thunderfirephalanx::CThunderFirePhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::cs2ccontainerobjectamountchange::CS2CContainerObjectAmountChange;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{CS2CContainerObjectMove, ContainerObjectMoveOperation};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const ITEM_SKILL_2_ID: u32 = 0x322;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const ITEM_INDEX: u32 = 50_001;
const ITEM_AMOUNT: u32 = 50_002;
const ALLOW_PLAYER: u32 = 70_001;
const ALLOW_MONSTER: u32 = 70_004;
const MONSTER_GROUP: u32 = 70_005;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
const fn has_mana(mana: u32, loss: u32) -> bool { mana.wrapping_sub(loss) as i32 >= 0 }

fn master_info(player: &CPlayer) -> MasterInfo {
    let p = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: 0, permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate), permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) }
}

fn target_view(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(ShapeIdentity, i32, i32, bool, Vec<u8>)> {
    let PlayerSkillDispatch::Object { target, .. } = dispatch else { return None };
    let view = game.base_magic_target_view(region_id, target)?;
    let original_name = if target.object_type == MONSTER_TYPE { game.find_region(region_id)?.base().find_monster_by_id(target.id)?.original_name().to_vec() } else { Vec::new() };
    let dead = match target.object_type { PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead), MONSTER_TYPE => game.find_region(region_id)?.base().find_monster_by_id(target.id).is_none_or(|monster| monster.hit_points() == 0), _ => true };
    Some((target, view.tile_x, view.tile_y, dead, original_name))
}

fn send_notify(game: &CGame, player_id: i32, string_id: &[u8]) {
    let mut message = CMessage::new(0x000b_f806);
    message.add_ulong(0xffff_0000); message.add_ulong(0);
    message.base_mut().add(game.get_string_by_id(string_id)); message.base_mut().add_byte(0);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn send_failure(game: &CGame, player_id: i32, action: u8, string_id: &[u8], value: Option<u32>) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
    if let Some(value) = value { game.send_skill_system_info_with_unsigned(player_id, string_id, value); } else { game.send_skill_system_info(player_id, string_id); }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, target: Option<(ShapeIdentity, i32, i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if target.is_some() { 2 } else { 1 }); message.add_long(ITEM_SKILL_2_ID as i32); message.base_mut().add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if let Some((identity, x, y, attack_time)) = target { message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(x); message.add_long(y); message.add_long(attack_time); } else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_item_skill_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    item_index: Option<u32>,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if let Some(item_index) = item_index {
        let item_used_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) {
            player.mark_skill_item_used(item_index, item_used_at_ms);
        }
    }
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    player_ai.mark_item_skill_2_used(runtime.now_milliseconds());
}

pub(crate) fn cancel_player_item_skill_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.item_skill_2().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    let item_index = game.find_player(player_id).and_then(|player| {
        let level = player.item_skill_level(ITEM_SKILL_2_ID);
        game.skill_base_properties(ITEM_SKILL_2_ID, level)
            .map(|properties| properties.query_property(ITEM_INDEX))
    });
    finish_player_item_skill_2(game, player_id, player_ai, item_index, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn consume_item(game: &mut CGame, player_id: i32, position: u32, item_index: u32, amount: u32) -> bool {
    let Some(c) = game.find_player_mut(player_id).and_then(|player| player.consume_skill_item_at(position, item_index, amount)) else { return false };
    if c.remaining_amount == 0 {
        let mut deleted = CS2CContainerObjectMove::default(); deleted.set_operation(ContainerObjectMoveOperation::DeleteObject); deleted.set_source_container(PLAYER_TYPE, player_id, position); deleted.set_source_container_extend_id(1); deleted.set_source_object(c.goods.object_type, c.goods.ex_id, 0); let _ = deleted.send_to_player(game, player_id);
    } else {
        let mut changed = CS2CContainerObjectAmountChange::default(); changed.set_source_container(PLAYER_TYPE, player_id, position); changed.set_source_container_extend_id(1); changed.set_object(c.goods.object_type, c.goods.ex_id); changed.set_object_amount(c.remaining_amount); let _ = changed.send_to_player(game, player_id);
    }
    let mut used = CMessage::new(0x000b_f709); used.add_byte(b'3'); used.add_byte(0); used.add_long(player_id); used.add_ulong(item_index); let _ = used.send_to_player(game.net_server(), player_id); true
}

pub(crate) const fn is_item_skill_2_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Object { skill_id: ITEM_SKILL_2_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }

pub(crate) fn execute_player_item_skill_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_item_skill_2_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, level, mana, source_x, source_y, item_position)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.item_skill_level(ITEM_SKILL_2_ID), player.mana(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.item_skill_position(ITEM_SKILL_2_ID)?))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    if item_position < 0 { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(p) = game.skill_base_properties(ITEM_SKILL_2_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let item_index=p.query_property(ITEM_INDEX); let item_amount=p.query_property(ITEM_AMOUNT); let mp_loss=p.query_property(USER_MP_LOSE); let reuse=p.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay=p.query_property(SKILL_USAGE_DELAY_TIME); let maximum=p.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let speed=p.query_property(SKILL_USAGE_SUMMONED_SPEED); let lifetime=p.query_property(SKILL_USAGE_SUMMONED_LIFETIME); let minimum_attack=p.query_property(SKILL_USAGE_MIN_ATTACK) as i32; let maximum_attack=p.query_property(SKILL_USAGE_MAX_ATTACK) as i32; let element_modifier=p.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32; let group=p.query_property(MONSTER_GROUP); let allow_player=p.query_property(ALLOW_PLAYER)!=0; let allow_monster=p.query_property(ALLOW_MONSTER)!=0; let _breakable=p.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let Some((target, target_x, target_y, target_dead, original_name)) = target_view(game, region_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) };
    if player_ai.item_skill_2().is_none() {
        if game.find_player(player_id).is_none_or(|player| player.check_item_in_packet(item_index) < item_amount) { send_notify(game, player_id, b"GS1179"); return terminal(QueuedSkillExecutionState::Rejected); }
        if target.object_type==PLAYER_TYPE && !allow_player { send_notify(game, player_id, b"GS1180"); return terminal(QueuedSkillExecutionState::Rejected); }
        if target.object_type==MONSTER_TYPE && !allow_monster { send_notify(game, player_id, b"GS1181"); return terminal(QueuedSkillExecutionState::Rejected); }
        if target.object_type==MONSTER_TYPE && !game.new_skill_monster_conf().groups().get(&group).is_some_and(|names| names.iter().any(|name| name==&original_name)) { send_notify(game, player_id, b"GS1182"); return terminal(QueuedSkillExecutionState::Rejected); }
        if target.object_type==PLAYER_TYPE && target.id==player_id { send_failure(game, player_id, 10, b"GS1183", None); return terminal(QueuedSkillExecutionState::Rejected); }
        let item_last=game.find_player(player_id).map_or(0, |player| player.last_skill_item_use_ms(item_index)); let last=player_ai.item_skill_2_last_used_ms().max(item_last);
        if last!=0 && !time_reached(runtime.now_milliseconds(), last, reuse) { send_failure(game, player_id, 0x0d, b"GS1184", None); return terminal(QueuedSkillExecutionState::Rejected); }
        let path=game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None); if maximum!=0 && path.len()>maximum as usize { send_failure(game, player_id, 0x0b, b"GS1185", None); return terminal(QueuedSkillExecutionState::Rejected); } if path.iter().any(|cell| cell.2==2) { send_failure(game, player_id, 0x0f, b"GS1186", None); return terminal(QueuedSkillExecutionState::Rejected); } if mp_loss!=0 && !has_mana(mana, mp_loss) { send_failure(game, player_id, 7, b"GS1187", Some(mp_loss)); return terminal(QueuedSkillExecutionState::Rejected); }
        player_ai.begin_item_skill_2(SkillExecutionKernel::begin(dispatch, runtime.now_milliseconds()));
    } else if player_ai.item_skill_2().is_none_or(|execution| execution.dispatch()!=dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if player_ai.item_skill_2().is_some_and(|execution| execution.stage()==SkillStage::Begin) {
        let current=game.find_player(player_id).map_or(0,CPlayer::mana); if !has_mana(current,mp_loss) { send_failure(game,player_id,7,b"GS1187",Some(mp_loss)); finish_player_item_skill_2(game,player_id,player_ai,Some(item_index),runtime); return terminal(QueuedSkillExecutionState::Rejected); } if target_dead { send_failure(game,player_id,10,b"GS1188",None); finish_player_item_skill_2(game,player_id,player_ai,Some(item_index),runtime); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player)=game.find_player_mut(player_id) { player.set_mana(current.wrapping_sub(mp_loss)); player.movement_shape_mut().set_direction(get_line_direction(source_x,source_y,target_x,target_y)); player.set_skill_moveable(false); player.set_current_skill_id(Some(ITEM_SKILL_2_ID)); }
        let _=game.update_player_current_state(player_id,GamePlayerFightStatePhase::MoveShapeAi); send_visual(game,player_id,level,None); if let Some(execution)=player_ai.item_skill_2_mut(){let _=execution.advance(SkillStage::Begin,SkillStage::Check);}
    }
    let started=player_ai.item_skill_2().map(SkillExecutionKernel::started_at_ms).expect("громовой огонь начат"); if !time_reached(runtime.now_milliseconds(),started,delay){return terminal(QueuedSkillExecutionState::Pending);} if let Some(player)=game.find_player_mut(player_id){player.set_skill_moveable(true);}
    let Some((live_target,live_x,live_y,dead,_))=target_view(game,region_id,dispatch) else {send_failure(game,player_id,10,b"GS1188",None);finish_player_item_skill_2(game,player_id,player_ai,Some(item_index),runtime);return terminal(QueuedSkillExecutionState::Rejected)}; if dead {send_failure(game,player_id,10,b"GS1188",None);finish_player_item_skill_2(game,player_id,player_ai,Some(item_index),runtime);return terminal(QueuedSkillExecutionState::Rejected);}
    let distance=real_distance(source_x,source_y,live_x,live_y); send_visual(game,player_id,level,Some((live_target,live_x,live_y,distance.wrapping_mul(speed as i32)))); let path=game.base_magic_path(region_id,source_x,source_y,live_x,live_y,Some(distance.max(0) as u32));
    if !path.is_empty() && !path.iter().any(|cell|cell.2==2) { let path=path.into_iter().map(|(x,y,_)|(x,y)).collect::<Vec<_>>(); let soul=game.find_player_mut(player_id).and_then(CPlayer::take_soul_collect_state); let (souls,variable)=soul.map_or((0,0),|state|{send_soul_collect_state_visual(game,region_id,ShapeIdentity{object_type:PLAYER_TYPE,id:player_id,ex_id:Default::default()},source_x,source_y,state,false);(state.souls(),state.variable_percent())}); let master=game.find_player(player_id).map(master_info); if let Some(master)=master { let summon_id=game.allocate_summon_shape_id(); let now=runtime.now_milliseconds(); let mut phalanx=CThunderFirePhalanx::new(summon_id,master,now,lifetime,level,minimum_attack,maximum_attack,element_modifier,path.clone(),speed,souls,variable); phalanx.shape_mut().set_region_id(region_id); if consume_item(game,player_id,item_position as u32,item_index,item_amount) { if let Some((x,y))=path.first().copied(){let result=game.add_thunder_fire_phalanx(region_id,phalanx,x,y,now,runtime);if result.as_ref().is_some_and(|result|result.is_ok()){let _=game.send_thunder_fire_phalanx_entry(region_id,summon_id,runtime);}tracing::trace!(region_id,player_id,summon_id,?result,"создан громовой огонь");}}}}
    if let Some(execution)=player_ai.item_skill_2_mut(){let _=execution.advance(SkillStage::Check,SkillStage::Calculate);let _=execution.advance(SkillStage::Calculate,SkillStage::Attack);let _=execution.advance(SkillStage::Attack,SkillStage::Apply);} finish_player_item_skill_2(game,player_id,player_ai,Some(item_index),runtime); terminal(QueuedSkillExecutionState::Completed)
}
