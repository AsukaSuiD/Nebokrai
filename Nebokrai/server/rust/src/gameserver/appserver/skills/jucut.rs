//! Фронтальный рубящий удар `CJuCut` (`0x6C`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/jucut.cpp`. Навык сохраняет частичное изменение MP до
//! повторной проверки оружия, направление на исходную цель и первую
//! `CMoveShape` лицевой клетки в региональном порядке. Неатакуемая первая
//! фигура не пропускается ради следующей. Формула выполняет один бросок урона
//! и один бросок критического удара только для фактически атакуемой цели.
//! `CGame` используется только для разрешения независимого владельца цели,
//! применения рассчитанной атаки, износа оружия и доставки.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const JU_CUT_SKILL_ID: u32 = 0x6c;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const BATTLE_FAIRY_TYPE: i32 = 1200;
const USER_MP_LOSE: u32 = 2;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_ju_cut_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == JU_CUT_SKILL_ID,
    }
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
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

fn weapon_is_compatible(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0292"),
        _ => {}
    }
}

fn destination(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(Option<ShapeIdentity>, i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((None, x, y)),
        PlayerSkillDispatch::Object { target, .. } => {
            let view = game.base_magic_target_view(region_id, target)?;
            Some((Some(target), view.tile_x, view.tile_y))
        }
        PlayerSkillDispatch::SelfTarget { .. } => {
            let face = game.find_player(player_id)?.shape().get_face_position().ok()?;
            Some((None, face.x, face.y))
        }
    }
}

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    dispatch: PlayerSkillDispatch,
    action: u8,
) {
    let Some((region_id, direction)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().get_direction()))
    }) else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(JU_CUT_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else {
        let (target, x, y) =
            destination(game, region_id, player_id, dispatch).unwrap_or((None, 0, 0));
        message.add_long(target.map_or(0, |identity| identity.object_type));
        message.add_long(target.map_or(0, |identity| identity.id));
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn front_shape(game: &CGame, region_id: i32, player_id: i32) -> Option<ShapeView> {
    let face = game.find_player(player_id)?.shape().get_face_position().ok()?;
    let region = game.find_region(region_id)?.base();
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    region
        .get_shapes(face.x, face.y, area_width, area_height, game, &mut shapes)
        .ok()?;
    shapes.into_iter().find(|shape| {
        matches!(
            shape.identity.object_type,
            PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | BATTLE_FAIRY_TYPE
        )
    })
}

fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target.id)?;
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as u8)
        }),
        _ => None,
    }
}

fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    target_level: u8,
    level: i32,
    hit_modifier: i32,
    target_damage_factor: u32,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let weapon_level = player.weapon_damage_level(game.goods_factory());
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let level_delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let weapon_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        (level_delta as f32 / weapon_divisor)
            .min(1.0)
            .max(weapon_minimum)
    };
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let width = maximum
        .wrapping_sub(minimum)
        .wrapping_abs()
        .wrapping_add(1);
    let physical = minimum
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(combat.dexterity as i32)
        .max(0);
    let mut attack = AttackInformation {
        skill_id: JU_CUT_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: target_damage_factor as f32 * weapon_factor * 0.01,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: (combat.add_element_attack as i32).max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(combat.add_soul_attack),
                mp_damage: 0,
            },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32;
        }
    }
    Some((master, attack))
}

pub(crate) fn execute_player_ju_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_ju_cut_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, source_x, source_y, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.learned_skill_level(JU_CUT_SKILL_ID),
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(JU_CUT_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let target_damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.ju_cut().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.ju_cut_last_used_ms() != 0
            && !time_reached(now_ms, player_ai.ju_cut_last_used_ms(), cooldown_ms)
        {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if !weapon_is_compatible(game, player) {
            send_failure(game, player_id, 0x0e, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(JU_CUT_SKILL_ID));
        }
        player_ai.begin_ju_cut(SkillExecutionKernel::begin(dispatch, now_ms));
    } else if player_ai
        .ju_cut()
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .ju_cut()
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game
            .find_player(player_id)
            .is_none_or(|player| !weapon_is_compatible(game, player))
        {
            send_failure(game, player_id, 0x0e, mp_loss);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some((_, target_x, target_y)) = destination(game, region_id, player_id, dispatch)
            && let Some(player) = game.find_player_mut(player_id)
        {
            player
                .movement_shape_mut()
                .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        send_visual(game, player_id, level, dispatch, 1);
        if let Some(execution) = player_ai.ju_cut_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .ju_cut()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение рубящего удара создано");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_visual(game, player_id, level, dispatch, 2);
    if let Some(execution) = player_ai.ju_cut_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let owner = game.find_player(player_id).map(master_info).unwrap_or_default();
    if let Some(target) = front_shape(game, region_id, player_id)
        && matches!(target.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
        && !(target.identity.object_type == PLAYER_TYPE && target.identity.id == player_id)
        && !game.periodic_state_target_dead(region_id, target.identity)
        && game.owned_player_skill_target_attackable(owner, target.identity, region_id)
        && let Some(level_of_target) = target_level(game, region_id, target.identity)
        && let Some((master, attack)) = calculate_attack(
            game,
            player_id,
            level_of_target,
            level,
            hit_modifier,
            target_damage_factor,
        )
    {
        match target.identity.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
                master,
                target.identity.id,
                region_id,
                attack,
                runtime,
            ),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
                master,
                target.identity.id,
                region_id,
                attack,
                runtime,
            ),
            _ => unreachable!("тип цели проверен перед расчётом"),
        }
        game.damage_player_weapon(player_id, runtime);
    }
    if let Some(execution) = player_ai.ju_cut_mut() {
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_ju_cut_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(QueuedSkillExecutionState::Completed)
}
