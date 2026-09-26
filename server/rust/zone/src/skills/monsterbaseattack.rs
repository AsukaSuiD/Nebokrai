//! Базовая атака боевых монстров `CMonsterBaseAttack` (ID `0x2bd`):
//! player-путь (reuse/гейты/дистанция/failure 2) и машинная база общих
//! AI/Calculate/Attack/End семьи. Монстр-вход исполнения остаётся hub у
//! `execute_owned_monster_base_attack` прежнего пакета (hub диспетчера
//! расписания, реестр исполнителей и общий монстр-драйвер кадров —
//! `appserver/skills/monsterbaseattack.rs` старого пакета).
//!
//! Машинные quirks: мёртвая цель mid-cast завершается End(1) со штампом
//! reuse; type S == 500 пропускается Attack; разброс `max(max−min, 0) + 1`;
//! End без movement-restore и без пересчёта свойств; crit-roll — только у
//! dyn-CPlayer (`fmul` по `[player+0x414]` из `fild`). Coordinator игрока и
//! queue-финализация остаются у планировщика старого пакета; формула
//! player-удара — шов `monster_combat_calculate_attack` владельца lord.
//!
//! Швы: hub-трейты `skills/monsterattack.rs`; часы — fn-параметр
//! `now_milliseconds` делегата старого main loop.
//!
//! Исходный владелец PDB: `appserver/skills/monsterbaseattack.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#monsterbaseattack--cmonsterbaseattack-0x2bd

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::real_distance_between_points;

use super::baseattackruntime::{
    BaseAttackContact, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatOutcome, MonsterCombatPlayer,
};

pub const MONSTER_BASE_ATTACK_SKILL_ID: u32 = 0x2bd;

const CAST_VISUAL_MESSAGE: i32 = 0x000b_fe01;

/// Диспетчерская форма player-cast базовой атаки боевых монстров: любая
/// self/point форма `0x2bd` и объектная по подвижной цели, NPC постройке или
/// воротам (допуск целей машины AI выше).
pub const fn is_player_monster_base_attack(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: MONSTER_BASE_ATTACK_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE | 500 | 1100 | 1200, .. } })
}

/// Кадр начала `0x000BFE01` player-ветви: action 1, направление «как есть»
/// после поворота Begin-стадии (message-owner mode 0 по 0x513EB5).
fn cast_start_message(skill_level: i32, player_id: i32, direction: i32) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(1);
    message.add_long(MONSTER_BASE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    message
}

/// Кадр выпуска player-ветви: action 2, сторона источника, identity цели
/// (нулевая при исчезнувшей) и её клетка либо запрошенная точка (message-
/// owner mode 1 по 0x513F34; fallback x/y держится в навыке).
fn cast_fire_message(
    skill_level: i32,
    player_id: i32,
    target: Option<ShapeIdentity>,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(2);
    message.add_long(MONSTER_BASE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// Хвост исхода player-ветви: успех выполняет `AfterUseSkill` (износ оружия
/// только у игрока) и фиксирует reuse владельца; отказ ничего не ставит,
/// как `End(0)` производного End без movement-restore.
fn end_player_cast<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    runtime: &mut Runtime,
    success: bool,
) where
    Game: MonsterCombatContact<Runtime>,
{
    if success {
        game.monster_combat_after_use_player_skill(player_id, MONSTER_BASE_ATTACK_SKILL_ID, runtime);
    }
}

/// Внешнее завершение player-cast по dispatch регистра: end-политика
/// прежнего `m_pCurrentSkill` не меняет — выбранный навык игрока трогают
/// OnChangeSkill/OnLoseTarget, а не этот End.
pub fn finish_player_monster_base_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    success: bool,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    let Some(dispatch) = game
        .player_kernel(player_id, MONSTER_BASE_ATTACK_SKILL_ID)
        .map(|kernel| kernel.dispatch())
    else {
        return false;
    };
    end_player_cast(game, player_id, runtime, success);
    game.finish_monster_player_skill(
        player_id,
        player_ai,
        dispatch,
        if success { SkillTermination::Completed } else { SkillTermination::Cancelled },
    )
}

/// Player-вход `0x2bd`: gate формы → reuse с failure 13+GS1143 → регистрация
/// cast → цель тика (sufferer) и failure 2 с после-use-success при мёртвой
/// цели → дистанционный gate Begin-стадии с поворотом и start-кадром →
/// абсолютный срок delay → fire-кадр → допуск/расчёт/контакт с IncreaseRp
/// → хвост успеха с AfterUseSkill.
pub fn execute_player_monster_base_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> MonsterCombatOutcome
where
    Game: MonsterCombatContact<Runtime> + BaseAttackContact<Runtime>,
{
    if !is_player_monster_base_attack(dispatch) {
        return MonsterCombatOutcome::Rejected;
    }
    let Some((region_id, level, source)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            game.monster_combat_player_skill_level(player_id, MONSTER_BASE_ATTACK_SKILL_ID)?,
            player.shape_view()?,
        ))
    }) else {
        return MonsterCombatOutcome::Rejected;
    };
    let Some(properties) = game
        .skill_base_properties(MONSTER_BASE_ATTACK_SKILL_ID, level)
        .cloned()
    else {
        // Begin-отказ по отсутствию свойств: End(0) без кадра и эффектов.
        end_player_cast(game, player_id, runtime, false);
        return MonsterCombatOutcome::Rejected;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if game.player_kernel(player_id, MONSTER_BASE_ATTACK_SKILL_ID).is_none() {
        let now = now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, MONSTER_BASE_ATTACK_SKILL_ID),
            reuse,
            now,
        ) {
            game.send_cast_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS1143");
            end_player_cast(game, player_id, runtime, false);
            return MonsterCombatOutcome::Rejected;
        }
        game.begin_player_kernel(
            player_id,
            SkillExecutionKernel::begin(dispatch, now),
        );
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(MONSTER_BASE_ATTACK_SKILL_ID));
        }
        return MonsterCombatOutcome::Begun;
    }
    if game
        .player_kernel(player_id, MONSTER_BASE_ATTACK_SKILL_ID)
        .is_none_or(|kernel| kernel.dispatch() != dispatch)
    {
        return MonsterCombatOutcome::Rejected;
    }
    let requested = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => game.resolve_identity_sufferer(region_id, target),
        PlayerSkillDispatch::Point { x, y, .. } => game.resolve_coordinate_sufferer(region_id, x, y),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    };
    let target = requested.and_then(|identity| {
        game.base_magic_target_view(region_id, identity).map(|view| (identity, view))
    });
    // Мёртвая цель mid-cast: машина AI посылает failure 2 и завершает End(1)
    // со штампом reuse (0x114820) — player-ветвь соответствует изначально.
    if target.is_some_and(|(identity, _)| game.base_magic_target_dead(region_id, identity)) {
        game.send_cast_failure(player_id, 2);
        end_player_cast(game, player_id, runtime, true);
        return MonsterCombatOutcome::Completed;
    }
    let (fallback_x, fallback_y) = match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => (x, y),
        _ => (0, 0),
    };
    let (target_x, target_y) =
        target.map_or((fallback_x, fallback_y), |(_, view)| (view.tile_x, view.tile_y));
    if game
        .player_kernel(player_id, MONSTER_BASE_ATTACK_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let distance = target.map_or_else(
            || real_distance_between_points(source.tile_x, source.tile_y, target_x, target_y),
            |(_, view)| source.real_distance(Some(view)),
        );
        // Первая AI-фаза: беззнаковая дистанционная проверка до поворота.
        if maximum_distance != 0 && maximum_distance < distance as u32 {
            game.send_cast_failure(player_id, 0x0b);
            end_player_cast(game, player_id, runtime, false);
            return MonsterCombatOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.face_cast_direction(get_line_direction(
                source.tile_x, source.tile_y, target_x, target_y,
            ));
        }
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction());
        if let Some(direction) = direction {
            let message = cast_start_message(level, player_id, direction);
            game.send_player_visual(player_id, &message);
        }
        if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game
        .player_kernel(player_id, MONSTER_BASE_ATTACK_SKILL_ID)
        .map(|kernel| kernel.started_at_ms())
        .expect("базовая атака хранит начало");
    if !skill_is_restored(started, delay, now_milliseconds()) {
        return MonsterCombatOutcome::Pending;
    }
    let fire = cast_fire_message(level, player_id, target.map(|(identity, _)| identity), target_x, target_y);
    game.send_player_visual(player_id, &fire);
    if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    if let Some((identity, _)) = target
        && !(identity.object_type == PLAYER_TYPE && identity.id == player_id)
        && let Some(master) = game.find_player(player_id).map(|player| player.master_info())
        && (if matches!(identity.object_type, 1_100 | 1_200) {
            game.stationary_build_attackable_by_player(player_id, region_id, identity)
        } else {
            game.owned_player_skill_target_attackable(master, identity, region_id)
        })
        && let Some((master, mut attack)) =
            game.monster_combat_calculate_attack(player_id, MONSTER_BASE_ATTACK_SKILL_ID, level, hit_modifier)
    {
        if matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE | 1_100 | 1_200) {
            // Машинный Calculate не замещает конструкторские UNKNOWN/уровень 1
            // seed tagAttackInformation (формула — владелец lord, старый пакет).
            attack.skill_id = crate::combat::UNKNOWN_SKILL_ID;
            attack.skill_level = 1;
            game.with_published_player_ai(player_id, player_ai, |game| {
                game.apply_owned_skill_contact(master, identity, region_id, attack, runtime);
                game.monster_combat_increase_rp(player_id, true, 0);
            });
        }
    }
    if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    end_player_cast(game, player_id, runtime, true);
    MonsterCombatOutcome::Completed
}
