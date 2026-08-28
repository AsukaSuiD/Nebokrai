//! Приручение обычного монстра (`CMonsterTaming`, навык `0xd4`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/monstertaming.cpp`. Владелец навыка сохраняет проверки
//! пути, MP, безопасных клеток, уровней, лимита питомцев, единственный бросок
//! `random(10000)` и странную границу счётчика попыток. Успех переводит тот же
//! региональный объект в жизненный цикл питомца без параллельной копии и рассылает
//! `0xC0201`; `CGame` используется только для доступа к владельцам и доставки.

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_WEAPON_LEVEL_MODIFIER: u32 = 20_018;
const SKILL_USAGE_PET_AMOUNT_LIMIT: u32 = 31_001;
const SKILL_USAGE_BASE_PROBABILITY: u32 = 40_001;

#[derive(Clone, Debug)]
struct TamingTarget {
    property: MonsterProperties,
    tile_x: i32,
    tile_y: i32,
    display_name: Vec<u8>,
    hit_points: u32,
    tamable: bool,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn send_failure(game: &mut CGame, player_id: i32, reason: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, reason);
}

fn target_snapshot(game: &CGame, region_id: i32, monster_id: i32) -> Option<TamingTarget> {
    let monster = game.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
    let property = game
        .find_monster_property_by_origin_name(monster.base_property_key()?)?
        .clone();
    Some(TamingTarget {
        tile_x: monster.move_shape().shape().get_tile_x().ok()?,
        tile_y: monster.move_shape().shape().get_tile_y().ok()?,
        display_name: monster.display_name().to_vec(),
        hit_points: monster.hit_points(),
        tamable: monster.is_tamable(&property),
        property,
    })
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    monster_id: i32,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let source = player.shape().identity();
    let region_id = player.server_region_id();
    let direction = player.shape().get_direction();
    let target = region_id.and_then(|region_id| target_snapshot(game, region_id, monster_id));
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(MONSTER_TAMING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(source.object_type);
    message.add_long(source.id);
    if action == 1 {
        message.add_long(direction);
    } else if action == 2 {
        let Some(target) = target else { return; };
        message.add_long(MONSTER_TYPE);
        message.add_long(monster_id);
        message.add_long(target.tile_x);
        message.add_long(target.tile_y);
        message.add_long(0);
    } else {
        return;
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn add_legacy_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    let visible = value.iter().position(|byte| *byte == 0).map_or(value, |end| &value[..end]);
    message.add(visible);
    message.add_byte(0);
}

fn increase_attempt(game: &mut CGame, region_id: i32, monster_id: i32) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else { return false; };
    let increased = owner.base_mut().find_monster_by_id_mut(monster_id).is_some_and(|monster| {
        monster.increase_tame_attempt_count();
        true
    });
    game.restore_region_owner(owner);
    increased
}

fn apply_success(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    monster_id: i32,
    property: &MonsterProperties,
) -> bool {
    let Some((player_name, pet_mode)) = game.find_player(player_id).map(|player| {
        (player.player_name().to_vec(), player.current_pets_mode())
    }) else {
        return false;
    };
    let factors = game.globe_setup().pet_progression(0).map(|(_, factors)| factors);
    let Some(mut owner) = game.take_region_owner(region_id) else { return false; };
    let result = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        monster.try_become_tamed(
            property,
            MasterInfo {
                master_type: PLAYER_TYPE,
                master_id: player_id,
                ..MasterInfo::default()
            },
            pet_mode,
            factors,
        ).then(|| {
            let (level, experience) = monster.pet_progress();
            (
                monster.move_shape().shape().clone(),
                level,
                experience,
                monster.hit_points(),
                monster.pet_maximum_hp(property),
            )
        })
    });
    game.restore_region_owner(owner);
    let Some((shape, level, experience, hit_points, maximum_hp)) = result else {
        return false;
    };
    if let Some(player) = game.find_player_mut(player_id) {
        player.add_active_pet(MONSTER_TYPE, monster_id, property.figure as u8 as i32);
    }
    let mut message = CMessage::new(0x000c_0201);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    add_legacy_c_string(message.base_mut(), &player_name);
    message.add_ulong(level);
    message.add_ulong(experience);
    message.add_ulong(hit_points);
    message.add_ulong(maximum_hp);
    if let Some(region) = game.find_region(region_id) {
        let _ = game.send_game_shape_around(region.base(), &shape, None, &message);
    }
    if let Some(mut owner) = game.take_region_owner(region_id) {
        owner.base_mut().finish_owned_monster_taming(monster_id);
        game.restore_region_owner(owner);
    }
    true
}

pub(crate) fn execute_player_monster_taming<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let monster_id = match dispatch {
        PlayerSkillDispatch::Object {
            skill_id: MONSTER_TAMING_SKILL_ID,
            target: ShapeIdentity { object_type: MONSTER_TYPE, id, .. },
        } => id,
        _ => {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    };
    let Some((region_id, skill_level)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(MONSTER_TAMING_SKILL_ID),
        ))
    })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(MONSTER_TAMING_SKILL_ID, skill_level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let weapon_modifier = properties.query_property(SKILL_USAGE_WEAPON_LEVEL_MODIFIER) as i32;
    let pet_limit = properties.query_property(SKILL_USAGE_PET_AMOUNT_LIMIT);
    let probability = properties.query_property(SKILL_USAGE_BASE_PROBABILITY);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.monster_taming().is_none() {
        let Some((source_x, source_y, initial_mana)) = game
            .find_player(player_id)
            .and_then(|player| {
                Some((
                    player.shape().get_tile_x().ok()?,
                    player.shape().get_tile_y().ok()?,
                    player.mana(),
                ))
            })
        else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(initial_target) = target_snapshot(game, region_id, monster_id) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0294");
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if !initial_target.tamable {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0312");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            initial_target.tile_x,
            initial_target.tile_y,
            None,
        );
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if player_ai.monster_taming_last_used_ms() != 0
            && !time_reached(runtime.now_milliseconds(), player_ai.monster_taming_last_used_ms(), reuse_delay_ms)
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0295", &initial_target.display_name);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(MONSTER_TAMING_SKILL_ID));
        }
        player_ai.begin_monster_taming(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.monster_taming().is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(current_target) = target_snapshot(game, region_id, monster_id) else {
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if current_target.hit_points == 0 {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.monster_taming().is_some_and(|state| state.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_movement(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
            ))
        }) else {
            finish_movement(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                current_target.tile_x,
                current_target.tile_y,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_cast(game, player_id, monster_id, skill_level, 1);
        if let Some(state) = player_ai.monster_taming_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai
        .monster_taming()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение приручения создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    finish_movement(game, player_id);
    let Some(target) = target_snapshot(game, region_id, monster_id) else {
        player_ai.mark_monster_taming_used(runtime.now_milliseconds());
        return terminal(QueuedSkillExecutionState::Completed);
    };
    let Some((source_x, source_y, player_level, weapon_level, pet_count)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.level(),
                player.weapon_damage_level(game.goods_factory()),
                player.active_pets().len() as u32,
            ))
        })
    else {
        player_ai.mark_monster_taming_used(runtime.now_milliseconds());
        return terminal(QueuedSkillExecutionState::Completed);
    };
    let current_path = game.base_magic_path(region_id, source_x, source_y, target.tile_x, target.tile_y, None);
    if maximum_distance != 0 && current_path.len() > maximum_distance as usize {
        send_failure(game, player_id, 0x0b);
        game.send_skill_system_info(player_id, b"GS0290");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_cast(game, player_id, monster_id, skill_level, 2);
    if let Some(state) = player_ai.monster_taming_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    if !target.tamable || !increase_attempt(game, region_id, monster_id) {
        player_ai.mark_monster_taming_used(runtime.now_milliseconds());
        return terminal(QueuedSkillExecutionState::Completed);
    }
    let safe_cell = game.find_region(region_id).is_none_or(|region| {
        region.base().region.get_block(target.tile_x, target.tile_y).unwrap_or(2) == 2
            || region.base().region.get_block(source_x, source_y).unwrap_or(2) == 2
    });
    let target_level = target.property.level as u8;
    if safe_cell {
        game.send_skill_system_info(player_id, b"GS0315");
    } else if player_level < target_level {
        game.send_skill_system_info(player_id, b"GS0314");
    } else if weapon_level.wrapping_sub(weapon_modifier) < i32::from(target_level) {
        game.send_skill_system_info(player_id, b"GS0314");
    } else if pet_limit <= pet_count {
        game.send_skill_system_info(player_id, b"GS0313");
    } else if game.skill_random_below(10_000) as u32 <= probability {
        if !apply_success(game, player_id, region_id, monster_id, &target.property) {
            game.send_skill_system_info(player_id, b"GS0312");
        }
    }
    if let Some(state) = player_ai.monster_taming_mut() {
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_monster_taming_used(runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}
