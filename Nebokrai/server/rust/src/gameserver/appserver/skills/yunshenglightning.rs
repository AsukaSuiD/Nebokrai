//! Владелец одноцелевой молнии `CYunShengLightning` (`0x19E`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/yunshenglightning.cpp`. Player-путь сохраняет повторную
//! проверку и необратимый расход MP, задержку, постоянное время полёта, один
//! RNG-вызов elemental-формулы и последующую проверку допустимости живой цели.
//! Нулевой `SKILL_USAGE_USER_MP_LOSE` отклоняет player-cast, как исходный
//! `CheckCastCondition`, а не превращает его в бесплатный навык.
//! Исчезнувшая объектная цель оставляет координату для визуального завершения,
//! но не получает удар. Monster/pet-путь использует те же стадии с собственной
//! формулой и attack interval. `CGame` только разрешает владельцев, применяет
//! рассчитанный удар и выполняет доставку.
//! Player-стихийная прибавка остаётся в x87 до усечения к нулю; property
//! загружается как беззнаковый DWORD, а modifier игрока — как знаковый. Player
//! и monster ветви используют абсолютный срок `CSkill::IsRestored`; cast и
//! время полёта остаются elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_ELEMENT_MODIFIER};
use super::fightdefense::truncate_original;
use super::flash::master_info;
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const YUNSHENG_LIGHTNING_SKILL_ID: u32 = 0x19e;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerYunShengLightningExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
    condition_checked: bool,
    fired: bool,
}

impl PlayerYunShengLightningExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination, condition_checked: false, fired: false } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct YunShengLightningProgress {
    destination_x: i32,
    destination_y: i32,
    flying_time_ms: u32,
    fired: bool,
}

impl YunShengLightningProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            flying_time_ms: 0,
            fired: false,
        }
    }

    pub(crate) const fn fired(self) -> bool {
        self.fired
    }

    pub(crate) const fn destination(self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    pub(crate) const fn flying_time_ms(self) -> u32 {
        self.flying_time_ms
    }

    pub(crate) fn fire(
        &mut self,
        destination_x: i32,
        destination_y: i32,
        flying_time_ms: u32,
    ) {
        self.destination_x = destination_x;
        self.destination_y = destination_y;
        self.flying_time_ms = flying_time_ms;
        self.fired = true;
    }
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    target: Option<ShapeIdentity>,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |target| target.object_type));
    message.add_long(target.map_or(0, |target| target.id));
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт полёта")]
pub(crate) fn execute_owned_yunsheng_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((
        source,
        property,
        master,
        tamed,
        attack_interval_ms,
        cast,
        progress,
        last_used_ms,
    )) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.master_info(),
                monster.is_tamed(),
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.yunsheng_lightning_progress(),
                monster.skill_last_used_ms(YUNSHENG_LIGHTNING_SKILL_ID),
            ))
        })
    else {
        return false;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let target_coordinates = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some((target_x, target_y)) = target_coordinates
        .or_else(|| progress.map(YunShengLightningProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            target.as_ref().map_or_else(
                || MonsterTraceTarget::point(target_x, target_y),
                |target| MonsterTraceTarget::Shape(target.view),
            ),
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return true;
            }
        }
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(
                target_identity, YUNSHENG_LIGHTNING_SKILL_ID, skill_level, now_ms,
            );
            monster.set_yunsheng_lightning_progress(YunShengLightningProgress::new(
                target_x, target_y,
            ));
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение молнии проверено выше");
    if cast.dispatch().skill_id != YUNSHENG_LIGHTNING_SKILL_ID {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        send_fire(
            game,
            region,
            &source,
            skill_level,
            target.as_ref().map(|_| target_identity),
            target_x,
            target_y,
        );
        let flying_time_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
        progress.fire(target_x, target_y, flying_time_ms);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_yunsheng_lightning_progress(progress);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.flying_time_ms()),
    ) {
        return true;
    }

    if let Some(target) = target
        && !target.dead
        && !target.god
        && !target.city_dead
        && owned_monster_attackable(
            game, region.id, &property, tamed, master, target_identity, &target,
        )
    {
        // Windows `CMonster::GetAddElementAtk` возвращает ноль даже для
        // приручённого монстра; поэтому здесь остаётся ровно один RNG-вызов.
        let base_element = 0_i32;
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let skill_span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
            .wrapping_sub(minimum)
            .unsigned_abs()
            .wrapping_add(1) as i32;
        let skill_element = minimum.wrapping_add(game.skill_random_below(skill_span));
        let attack = AttackInformation {
            skill_id: YUNSHENG_LIGHTNING_SKILL_ID,
            skill_level: skill_level as u8,
            attacker_type: MONSTER_TYPE,
            attacker_id: monster_id,
            attacker_team_id: 0,
            attacker_faction_id: 0,
            attacker_union_id: 0,
            hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
            damage_factor: 1.0,
            damage_modifier: properties
                .query_property(SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER) as i32,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: base_element.wrapping_add(skill_element).max(0),
                mp_damage: 0,
            }],
        };
        let attack = defend_owned_monster_attack(
            game, target_identity, target.mana, target.war_soul_mana,
            target.player_properties, target.monster_properties, attack,
        );
        apply_owned_monster_attack_hit(
            game, region, runtime, now_ms, monster_id, master, target_identity,
            &target.shape, target.health, target.mana, target.master,
            target.monster_property, target.tamed, target.carriage, attack, deaths,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn is_player_yunsheng_lightning_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Point { skill_id: YUNSHENG_LIGHTNING_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: YUNSHENG_LIGHTNING_SKILL_ID, .. }) }
fn player_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch, fallback: Option<(i32, i32)>) -> Option<(i32, i32)> { match dispatch { PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)).or(fallback), PlayerSkillDispatch::SelfTarget { .. } => None } }
fn player_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }
fn send_player_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(0x000b_fe01, player_id, action); }
fn send_player_start(game: &mut CGame, player_id: i32, level: i32) { let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return }; let mut message = CMessage::new(0x000b_fe01); message.add_byte(1); message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(direction); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_player_fire(game: &mut CGame, player_id: i32, level: i32, target: Option<ShapeIdentity>, destination: (i32, i32)) { let mut message = CMessage::new(0x000b_fe01); message.add_byte(2); message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(target.map_or(0, |target| target.object_type)); message.add_long(target.map_or(0, |target| target.id)); message.add_long(destination.0); message.add_long(destination.1); let _ = game.send_player_shape_around(player_id, None, &message); }
fn restore_player(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) { restore_player(game, player_id); finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(YUNSHENG_LIGHTNING_SKILL_ID, now_ms)); }
pub(crate) fn cancel_player_yunsheng_lightning<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.yunsheng_lightning().map(|state| state.kernel().dispatch()) else { return false }; finish_player(game, player_id, ai, runtime); ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }

fn calculate_player_attack(game: &mut CGame, player_id: i32, level: i32, properties: &CSkillBaseProperties) -> Option<(MasterInfo, AttackInformation)> { let (combat, master) = game.find_player(player_id).map(|player| (player.combat_properties(), master_info(player)))?; let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32; let span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32).wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32; let random = game.skill_random_below(span); let modifier = truncate_original(f64::from(properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER)) * f64::from(0.01_f32) * f64::from(combat.element_modify)); let element = (combat.add_element_attack as i32).wrapping_add(minimum).wrapping_add(random).wrapping_add(modifier).max(0); Some((master, AttackInformation { skill_id: YUNSHENG_LIGHTNING_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32, damage_factor: 1.0, damage_modifier: properties.query_property(SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER) as i32, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 }] })) }

pub(crate) fn execute_player_yunsheng_lightning<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_yunsheng_lightning_dispatch(dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(YUNSHENG_LIGHTNING_SKILL_ID), player.mana()))) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(YUNSHENG_LIGHTNING_SKILL_ID, level).cloned() else { if ai.yunsheng_lightning().is_some() { restore_player(game, player_id); abort_skill(game, player_id); } return player_terminal(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let flight = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED); let now = runtime.now_milliseconds();
    if ai.yunsheng_lightning().is_none() { if !skill_is_restored(ai.skill_last_used_ms(YUNSHENG_LIGHTNING_SKILL_ID), reuse, now) { send_player_failure(game, player_id, 0x0d); return player_terminal(QueuedSkillExecutionState::Rejected) } let Some(destination) = player_destination(game, region_id, dispatch, None) else { return player_terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); return player_terminal(QueuedSkillExecutionState::Rejected) } if mp_loss == 0 { return player_terminal(QueuedSkillExecutionState::Rejected) } if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(YUNSHENG_LIGHTNING_SKILL_ID)); } ai.begin_yunsheng_lightning(PlayerYunShengLightningExecutionState::begin(dispatch, destination, now)); }
    let fallback = ai.yunsheng_lightning().map(|state| state.destination); let Some(destination) = player_destination(game, region_id, dispatch, fallback) else { restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) };
    if ai.yunsheng_lightning().is_some_and(|state| !state.condition_checked) { let current = game.find_player(player_id).map_or(0, CPlayer::mana); if (current.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current.wrapping_sub(mp_loss)); player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_player_start(game, player_id, level); if let Some(state) = ai.yunsheng_lightning_mut() { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.yunsheng_lightning().map(|state| state.kernel().started_at_ms()).unwrap_or_default(); if ai.yunsheng_lightning().is_some_and(|state| !state.fired) { if !time_reached(now, started, delay) { return player_terminal(QueuedSkillExecutionState::Pending) } let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } let visual_target = player_target(dispatch).filter(|target| game.base_magic_target_view(region_id, *target).is_some()); send_player_fire(game, player_id, level, visual_target, destination); if let Some(state) = ai.yunsheng_lightning_mut() { state.destination = destination; state.fired = true; let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); } }
    if !time_reached(now, started, delay.wrapping_add(flight)) { return player_terminal(QueuedSkillExecutionState::Pending) }
    if let Some(target) = player_target(dispatch) && game.base_magic_target_view(region_id, target).is_some() && game.find_player(player_id).map(master_info).is_some_and(|master| game.owned_player_skill_target_attackable(master, target, region_id)) { if let Some((master, attack)) = calculate_player_attack(game, player_id, level, &properties) { match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} } } }
    if let Some(state) = ai.yunsheng_lightning_mut() { let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); } finish_player(game, player_id, ai, runtime); player_terminal(QueuedSkillExecutionState::Completed)
}
