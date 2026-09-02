//! Семь падающих звёзд `CSevenShootingStar` (`0x136`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/sevenshootingstar.cpp`. Навык один раз фиксирует
//! ограниченный прямой путь к цели и затем в порядке клеток периодически
//! атакует допустимые фигуры до первой `BLOCK_UNFLY`. Владелец сохраняет
//! двойную проверку MP, строгие границы времени, точные визуальные пакеты и
//! два вызова legacy RNG на рассчитанную атаку. `CGame` только разрешает
//! владельцев, применяет готовую атаку и выполняет доставку.
//! Element modifier вычисляется в расширенной точности x87 из целых свойств и
//! сохранённой `f32`-константы; он и критический множитель усекаются к нулю
//! перед `int`.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const SEVEN_SHOOTING_STAR_SKILL_ID: u32 = 0x136;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_PERSIST_TIME: u32 = 10_007;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SevenShootingStarExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Option<Vec<(i32, i32, u8)>>,
    destination: Option<(i32, i32)>,
    last_attack_ms: u32,
}

impl SevenShootingStarExecutionState {
    pub(crate) fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            path: None,
            destination: None,
            last_attack_ms: 0,
        }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    fn path(&self) -> Option<&[(i32, i32, u8)]> { self.path.as_deref() }
    fn destination(&self) -> Option<(i32, i32)> { self.destination }
    fn needs_path_refresh(&self) -> bool { self.last_attack_ms == 0 }
    fn set_path(&mut self, path: Vec<(i32, i32, u8)>, destination: (i32, i32)) {
        self.path = Some(path);
        self.destination = Some(destination);
    }
    fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }
    fn record_attack(&mut self, now_ms: u32) { self.last_attack_ms = now_ms; }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn target_position(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: SEVEN_SHOOTING_STAR_SKILL_ID, x, y } => Some((x, y)),
        PlayerSkillDispatch::Object { skill_id: SEVEN_SHOOTING_STAR_SKILL_ID, target }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) =>
        {
            game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y))
        }
        _ => None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, destination: Option<(i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(SEVEN_SHOOTING_STAR_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if matches!(action, 1 | 3) {
        message.add_long(player.shape().get_direction());
    } else {
        let Some((x, y)) = destination else { return };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
    successful: bool,
) {
    if successful {
        game.damage_player_weapon(player_id, runtime);
    }
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

pub(crate) fn cancel_player_seven_shooting_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    record_reuse: bool,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .seven_shooting_star()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish(game, player_id, runtime, record_reuse);
    if record_reuse {
        player_ai.mark_seven_shooting_star_used(runtime.now_milliseconds());
    }
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
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

fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .and_then(|monster| monster.base_property_key())
            .and_then(|key| game.find_monster_property_by_origin_name(key))
            .map(|property| property.level as u8),
        _ => None,
    }
}

fn path_targets(game: &CGame, region_id: i32, path: &[(i32, i32, u8)]) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    for &(x, y, block) in path {
        if block == BLOCK_UNFLY { break; }
        let mut shapes = Vec::new();
        if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { continue; }
        targets.extend(shapes.into_iter().map(|shape| shape.identity)
            .filter(|identity| matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)));
    }
    targets
}

#[allow(clippy::too_many_arguments, reason = "параметры соответствуют свойствам навыка EXE")]
fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = target_level(game, region_id, target)?;
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(
        game.goods_factory(),
        i32::from(target_level),
        weapon_divisor,
        weapon_minimum,
    );
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let element_bonus = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = (combat.add_element_attack as i32).wrapping_add(random_damage)
        .wrapping_add(minimum).wrapping_add(element_bonus).max(0);
    let mut attack = AttackInformation {
        skill_id: SEVEN_SHOOTING_STAR_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(critical_rate),
            );
        }
    }
    Some((master, attack))
}

pub(crate) const fn is_seven_shooting_star_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: SEVEN_SHOOTING_STAR_SKILL_ID, .. }
        | PlayerSkillDispatch::Object {
            skill_id: SEVEN_SHOOTING_STAR_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        })
}

pub(crate) fn execute_player_seven_shooting_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_seven_shooting_star_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(SEVEN_SHOOTING_STAR_SKILL_ID), player.mana(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(SEVEN_SHOOTING_STAR_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let persist_ms = properties.query_property(SKILL_PERSIST_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.seven_shooting_star().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.seven_shooting_star_last_used_ms() != 0
            && !time_reached(cooldown_now_ms, player_ai.seven_shooting_star_last_used_ms(), reuse_delay_ms)
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || target_position(game, region_id, dispatch).is_none() {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SEVEN_SHOOTING_STAR_SKILL_ID));
        }
        player_ai.begin_seven_shooting_star(SevenShootingStarExecutionState::begin(dispatch, started_at_ms));
    } else if player_ai.seven_shooting_star()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.seven_shooting_star()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let Some((target_x, target_y)) = target_position(game, region_id, dispatch) else {
            finish(game, player_id, runtime, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish(game, player_id, runtime, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, 1, None);
        if let Some(state) = player_ai.seven_shooting_star_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai.seven_shooting_star()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение семи падающих звёзд создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if player_ai.seven_shooting_star().is_some_and(SevenShootingStarExecutionState::needs_path_refresh) {
        let destination = player_ai.seven_shooting_star()
            .and_then(SevenShootingStarExecutionState::destination)
            .or_else(|| target_position(game, region_id, dispatch));
        let Some((target_x, target_y)) = destination else {
            finish(game, player_id, runtime, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, Some(maximum_distance));
        if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
        path.truncate(maximum_distance as usize);
        let endpoint = path.last().map(|cell| (cell.0, cell.1)).unwrap_or((target_x, target_y));
        send_visual(game, player_id, level, 2, Some(endpoint));
        if let Some(state) = player_ai.seven_shooting_star_mut() {
            state.set_path(path, endpoint);
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let attack_now_ms = runtime.now_milliseconds();
    if player_ai.seven_shooting_star().is_some_and(|state| state.attack_due(attack_now_ms, frequency_ms)) {
        let path = player_ai.seven_shooting_star().and_then(SevenShootingStarExecutionState::path)
            .unwrap_or_default().to_vec();
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        for target in path_targets(game, region_id, &path) {
            let Some(master) = game.find_player(player_id).map(master_info) else { break };
            let attackable = if target.object_type == PLAYER_TYPE {
                game.player_base_attackable(player_id, target.id)
            } else {
                game.owned_player_skill_target_attackable(master, target, region_id)
            };
            if !attackable { continue; }
            let Some((master, attack)) = calculate_attack(
                game, player_id, region_id, target, level, minimum, maximum, element_modifier, hit_modifier,
            ) else { continue };
            match target.object_type {
                PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                _ => {}
            }
        }
        let recorded_at_ms = runtime.now_milliseconds();
        if let Some(state) = player_ai.seven_shooting_star_mut() {
            state.record_attack(recorded_at_ms);
            if state.kernel().stage() == SkillStage::Calculate {
                let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
            }
        }
    }

    let expiration_now_ms = runtime.now_milliseconds();
    if started_at_ms.wrapping_add(delay_ms).wrapping_add(persist_ms) < expiration_now_ms {
        send_visual(game, player_id, level, 3, None);
        if let Some(state) = player_ai.seven_shooting_star_mut() {
            if state.kernel().stage() == SkillStage::Calculate {
                let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
            }
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        player_ai.mark_seven_shooting_star_used(expiration_now_ms);
        finish(game, player_id, runtime, true);
        terminal(QueuedSkillExecutionState::Completed)
    } else {
        terminal(QueuedSkillExecutionState::Pending)
    }
}
