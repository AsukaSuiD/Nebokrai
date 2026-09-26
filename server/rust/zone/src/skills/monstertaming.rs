//! Приручение обычного монстра `CMonsterTaming` (ID `0xd4`): player-путь
//! Check/AI и жизненный цикл питомца буквально. Успех переводит тот же
//! региональный объект в жизненный цикл питомца без параллельной копии и
//! рассылает `0xC0201`; регион и монстр остаются опубликованы через
//! синхронные End, пакет читает живую shape без копирования владельца.
//!
//! Машинные quirks: нулевая MP-цена в Check — молчаливый ret 0, а Begin
//! всех трёх форм после КАЖДОГО CheckCastCondition-отказа (включая
//! молчаливые) шлёт терминальный кадр `{0xBFE01, 0, 2}`; шанс — единственный
//! `random(10000) <= Query(40001)` после всей цепочки гейтов; успех шлёт
//! кадр `0xC0201` с legacy-C-строкой имени.
//!
//! Швы: hub-трейты `skills/monsterattack.rs`; часы — fn-параметр
//! `now_milliseconds` делегата старого main loop; сроки reuse/delay — общий
//! `CSkill::IsRestored` + wrapping-сложение.
//!
//! Исходный владелец PDB: `appserver/skills/monstertaming.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#monstertaming--cmonstertaming-0xd4

use nebokrai_shared::runtime::get_line_direction;
use nebokrai_shared::resources::MonsterProperties;

use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::baseattackruntime::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatGame, MonsterCombatOutcome, MonsterCombatPlayer, MonsterTamingTarget,
};

pub const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;

const CAST_VISUAL_MESSAGE: i32 = 0x000b_fe01;
const PET_TAMED_MESSAGE: i32 = 0x000c_0201;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_WEAPON_LEVEL_MODIFIER: u32 = 20_018;
const SKILL_USAGE_PET_AMOUNT_LIMIT: u32 = 31_001;
const SKILL_USAGE_BASE_PROBABILITY: u32 = 40_001;

/// Терминальный кадр `{0xBFE01, 0, 2}` после каждого CheckCastCondition-
/// отказа (DIFF-T2, message-owner mode 2 всех трёх Begin машины).
const TAMING_CHECK_REJECT_REASON: u8 = 2;

/// Кадр начала/огня приручения: action 1 — направление источника, action 2 —
/// тип/id цели, её клетка и нулевой хвост (поле `[skill+0x58]`, всегда 0 к
/// моменту fire — конструктор и AI семьи его не ставят).
fn taming_cast_message(
    skill_level: i32,
    player_id: i32,
    action: u8,
    direction: i32,
    monster_id: i32,
    target: Option<&MonsterTamingTarget>,
) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(action);
    message.add_long(MONSTER_TAMING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else {
        let target = target.expect("кадр огня приручения требует снимок цели");
        message.add_long(MONSTER_TYPE);
        message.add_long(monster_id);
        message.add_long(target.tile_x);
        message.add_long(target.tile_y);
        message.add_long(0);
    }
    message
}

fn add_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    let visible = value.iter().position(|byte| *byte == 0).map_or(value, |end| &value[..end]);
    let base = message.base_mut();
    base.add(visible);
    base.add_byte(0);
}

/// Отказ CheckCastCondition: собственный кадр причины/GS вызывающего, затем
/// терминальный кадр `{0xBFE01, 0, 2}` (машина посылает его из Begin после
/// ЛЮБОГО возврата 0 CheckCastCondition, включая молчаливые ветви — T2).
fn reject_taming_check<Game: MonsterCombatGame>(game: &Game, player_id: i32) {
    game.send_cast_failure(player_id, TAMING_CHECK_REJECT_REASON);
}

/// Абсолютный срок как у машины: `timeGetTime >= [+0x2C] + delay` (unsigned
/// wrapping-сложение DWORD владельца).
fn absolute_deadline_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    started_at_ms.wrapping_add(delay_ms) <= now_ms
}

fn finish_movement<Game: MonsterCombatGame>(game: &mut Game, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_cast<Game, Runtime>(game: &mut Game, player_id: i32, runtime: &mut Runtime)
where
    Game: MonsterCombatContact<Runtime>,
{
    game.monster_combat_after_use_player_skill(player_id, MONSTER_TAMING_SKILL_ID, runtime);
}

/// Внешнее завершение player-cast успехом: возврат движения (derived End)
/// и `AfterUseSkill` владельца.
pub fn complete_player_monster_taming<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    let Some(dispatch) = game
        .player_kernel(player_id, MONSTER_TAMING_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else {
        return false;
    };
    finish_movement(game, player_id);
    finish_player_cast(game, player_id, runtime);
    game.finish_monster_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

/// Внешняя отмена player-cast: возврат движения без AfterUseSkill
/// (`End(0)` не читает и не пишет reuse).
pub fn cancel_player_monster_taming<Game: MonsterCombatGame>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game
        .player_kernel(player_id, MONSTER_TAMING_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else {
        return false;
    };
    finish_movement(game, player_id);
    game.finish_monster_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

/// Увеличение счётчика попыток приручения живого монстра
/// (`IncreaseTameAttemptCount` владельца до safe-cell/гейтов).
fn increase_attempt<Game: MonsterCombatGame>(game: &mut Game, region_id: i32, monster_id: i32) -> bool {
    game
        .with_monster_combat_region(region_id, |game, region| {
            game.monster_increase_tame_attempt(region, monster_id)
        })
        .unwrap_or(false)
}

/// Успешная ветвь приручения (0x57C802..0x57C98E буквально): StopAllSkills →
/// назначение master/знака с auxiliary-эффектами → AddPet (фигура снимка
/// setup-строки, см. UNKNOWN#3 в шапке) → UpgradePetLevel → кадр 0xC0201 →
/// refresh-record decrement.
fn apply_success<Game: MonsterCombatGame>(
    game: &mut Game,
    player_id: i32,
    region_id: i32,
    monster_id: i32,
    property: &MonsterProperties,
) -> bool {
    let Some(holder) = game
        .with_monster_combat_region(region_id, |game, region| {
            game.monster_region_shape(region, monster_id).map(|shape| shape.identity())
        })
        .flatten()
    else {
        return false;
    };
    game.monster_combat_stop_all_skills(region_id, holder);
    let Some((player_name, pet_mode)) = game.find_player(player_id).map(|player| {
        (player.player_name().to_vec(), player.current_pets_mode())
    }) else {
        return false;
    };
    let succeeded = game
        .with_monster_combat_region(region_id, |game, region| {
            game.monster_try_become_tamed(
                region,
                monster_id,
                MasterInfo {
                    master_type: PLAYER_TYPE,
                    master_id: player_id,
                    ..MasterInfo::default()
                },
                pet_mode,
            )
        })
        .unwrap_or(false);
    if !succeeded {
        return false;
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.add_active_pet(MONSTER_TYPE, monster_id, property.figure as u8 as i32);
    }
    let Some(level) = game
        .with_monster_combat_region(region_id, |game, region| {
            game.monster_pet_progress(region, monster_id)
        })
        .flatten()
        .map(|progress| progress.0)
    else {
        return false;
    };
    let progression = game.globe_setup().pet_progression(level);
    let experience_factor = progression.map_or(0.0, |(factor, _)| factor);
    let current_factors = progression.map(|(_, factors)| factors);
    let next_factors = game.globe_setup().pet_progression(level.wrapping_add(1))
        .map(|(_, factors)| factors);
    let Some(snapshot) = game
        .with_monster_combat_region(region_id, |game, region| {
            game.monster_upgrade_pet_level(
                region, monster_id, property, experience_factor, current_factors, next_factors,
            );
            let progress = game.monster_pet_progress(region, monster_id);
            let hp = game.monster_hp_snapshot(region, monster_id, property);
            (progress, hp)
        })
    else {
        return false;
    };
    let (Some((level, experience)), Some((hit_points, maximum_hp))) = snapshot else {
        return false;
    };
    let mut message = CMessage::new(PET_TAMED_MESSAGE);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    add_legacy_c_string(&mut message, &player_name);
    message.add_ulong(level);
    message.add_ulong(experience);
    message.add_ulong(hit_points);
    message.add_ulong(maximum_hp);
    let _ = game.with_monster_combat_region(region_id, |game, region| {
        if let Some(shape) = game.monster_region_shape(region, monster_id) {
            game.send_visual_around(region, &shape, &message);
        }
    });
    let _ = game.with_monster_combat_region(region_id, |game, region| {
        game.monster_taming_decrease_refresh(region, monster_id);
    });
    true
}

/// Player-вход `0xd4`: приведение цели по форме объекта-600, Check с
/// гейтами reuse/пути/block/MP (T1/T2), фаза направления+старт, delay-
/// absolute, повторный path-поход с GS0290, IsTamable+attempt-гейты,
/// единственный `random(10000)` и успешная ветвь приручения.
pub fn execute_player_monster_taming<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> MonsterCombatOutcome
where
    Game: MonsterCombatContact<Runtime>,
{
    let monster_id = match dispatch {
        PlayerSkillDispatch::Object {
            skill_id: MONSTER_TAMING_SKILL_ID,
            target: ShapeIdentity { object_type: MONSTER_TYPE, id, .. },
        } => id,
        _ => {
            game.send_cast_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
    };
    let Some((region_id, skill_level)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            game.monster_combat_player_skill_level(player_id, MONSTER_TAMING_SKILL_ID)?,
        ))
    }) else {
        return MonsterCombatOutcome::Rejected;
    };
    let Some(properties) = game
        .skill_base_properties(MONSTER_TAMING_SKILL_ID, skill_level)
        .cloned()
    else {
        // Null-свойства: Check возвращает 0 без кадра, Begin — терминальный {0xBFE01,0,2}.
        reject_taming_check(game, player_id);
        return MonsterCombatOutcome::Rejected;
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let weapon_modifier = properties.query_property(SKILL_USAGE_WEAPON_LEVEL_MODIFIER) as i32;
    let pet_limit = properties.query_property(SKILL_USAGE_PET_AMOUNT_LIMIT);
    let probability = properties.query_property(SKILL_USAGE_BASE_PROBABILITY);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_kernel(player_id, MONSTER_TAMING_SKILL_ID).is_none() {
        let Some((source_x, source_y, initial_mana)) = game.find_player(player_id).and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.mana(),
            ))
        }) else {
            return MonsterCombatOutcome::Rejected;
        };
        let Some(initial_target) = game.monster_taming_target_snapshot(region_id, monster_id) else {
            game.send_cast_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0294");
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        };
        if !initial_target.tamable {
            game.send_cast_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0312");
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            initial_target.tile_x,
            initial_target.tile_y,
        );
        let started_at_ms = now_milliseconds();
        game.begin_player_combat_command(player_id, dispatch, started_at_ms);
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, MONSTER_TAMING_SKILL_ID),
            reuse_delay_ms,
            now_milliseconds(),
        ) {
            game.send_cast_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.send_cast_failure(player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_cast_failure(player_id, 0x0f);
            game.send_skill_system_info_text(player_id, b"GS0295", &initial_target.display_name);
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        // DIFF-T1: нулевая стоимость MP отклоняется машиной молча (jbe→ret0);
        // прежний Rust такой cast начинал. Терминальный кадр T2 отправляет
        // даже эту молчаливую ветвь (Begin после любого отказа Check).
        if mp_loss == 0 {
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_cast_failure(player_id, 7);
            game.send_skill_system_info_unsigned(player_id, b"GS0288", mp_loss);
            reject_taming_check(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            // SetMoveable(0) безусловен в success-ветви машинного Check; при
            // T1 нулевая стоимость сюда не доходит.
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(MONSTER_TAMING_SKILL_ID));
        }
        game.begin_player_kernel(
            player_id,
            SkillExecutionKernel::begin(dispatch, started_at_ms),
        );
        return MonsterCombatOutcome::Begun;
    }
    if game
        .player_kernel(player_id, MONSTER_TAMING_SKILL_ID)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return MonsterCombatOutcome::Rejected;
    }

    let Some(current_target) = game.monster_taming_target_snapshot(region_id, monster_id) else {
        finish_movement(game, player_id);
        return MonsterCombatOutcome::Rejected;
    };
    if current_target.hit_points == 0 {
        game.send_cast_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish_movement(game, player_id);
        return MonsterCombatOutcome::Rejected;
    }
    if game
        .player_kernel(player_id, MONSTER_TAMING_SKILL_ID)
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, |player| player.mana());
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_cast_failure(player_id, 7);
            game.send_skill_system_info_unsigned(player_id, b"GS0288", mp_loss);
            finish_movement(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
            ))
        }) else {
            finish_movement(game, player_id);
            return MonsterCombatOutcome::Rejected;
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player.face_cast_direction(get_line_direction(
                source_x,
                source_y,
                current_target.tile_x,
                current_target.tile_y,
            ));
        }
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(direction) = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction())
        {
            let message = taming_cast_message(skill_level, player_id, 1, direction, monster_id, None);
            game.send_player_visual(player_id, &message);
        }
        if let Some(state) = game.player_kernel_mut(player_id, MONSTER_TAMING_SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = game
        .player_kernel(player_id, MONSTER_TAMING_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение приручения создано или восстановлено");
    if !absolute_deadline_reached(now_milliseconds(), started_at_ms, delay_ms) {
        return MonsterCombatOutcome::Pending;
    }
    finish_movement(game, player_id);
    let Some(target) = game.monster_taming_target_snapshot(region_id, monster_id) else {
        finish_player_cast(game, player_id, runtime);
        return MonsterCombatOutcome::Completed;
    };
    let Some((source_x, source_y, player_level, weapon_level, pet_count)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.level(),
                game.monster_combat_weapon_damage_level(player_id)?,
                player.active_pets_count(),
            ))
        })
    else {
        finish_player_cast(game, player_id, runtime);
        return MonsterCombatOutcome::Completed;
    };
    let current_path = game.base_magic_path(
        region_id,
        source_x,
        source_y,
        target.tile_x,
        target.tile_y,
    );
    if maximum_distance != 0 && current_path.len() > maximum_distance as usize {
        game.send_cast_failure(player_id, 0x0b);
        game.send_skill_system_info(player_id, b"GS0290");
        return MonsterCombatOutcome::Rejected;
    }
    let message = taming_cast_message(
        skill_level, player_id, 2, 0, monster_id, Some(&target),
    );
    game.send_player_visual(player_id, &message);
    if let Some(state) = game.player_kernel_mut(player_id, MONSTER_TAMING_SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    if !target.tamable || !increase_attempt(game, region_id, monster_id) {
        finish_player_cast(game, player_id, runtime);
        return MonsterCombatOutcome::Completed;
    }
    let safe_cell = game.monster_combat_cell_blocked(region_id, target.tile_x, target.tile_y)
        || game.monster_combat_cell_blocked(region_id, source_x, source_y);
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
    if let Some(state) = game.player_kernel_mut(player_id, MONSTER_TAMING_SKILL_ID) {
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_cast(game, player_id, runtime);
    MonsterCombatOutcome::Completed
}
