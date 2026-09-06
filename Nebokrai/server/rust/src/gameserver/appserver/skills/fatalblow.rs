//! Смертельный удар боевого духа `CFatalBlow` (`0x21C`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fatalblow.cpp`. Владелец сохраняет проверки цели и пути,
//! задержку повторного использования, расход MP боевого духа, время подготовки,
//! визуальные пакеты и создание
//! `CFatalBlowPhalanx`. `CGame` предоставляет владельцев, регион, регистрацию
//! снаряда и доставку. Координатный и пустой `Begin` сохраняют исходный отказ
//! `10 + ZHGS0045` и завершающий visual action `3`; затем планировщик выдаёт
//! общий 4,2. Сам Begin (0x0051e3c0) не выдаёт ранний 4,2, в отличие от
//! BloodLoss/PoisonArrow. Отказ уже запущенного AI не получает ответ расписания.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; подготовка
//! сравнивает unsigned now с wrapping(start + delay), cmp/jb 0x0051f330.
//! Lifetime снаряда принадлежит отдельному владельцу.

use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::fatalblowphalanx::CFatalBlowPhalanx;
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const FATAL_BLOW_SKILL_ID: u32 = 0x21c;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const VISUAL_OBJECT_TYPE: i32 = 700;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0x67;
const DENIED_STATE_C: u32 = 0xd2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn send_end(game: &mut CGame, player_id: i32, skill_level: i32) {
    send_cast(game, player_id, skill_level, 3, None, 0);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
    missile_flying_time: u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(FATAL_BLOW_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(VISUAL_OBJECT_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let Some((target, x, y)) = target else { return };
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(x);
        message.add_long(y);
        message.add_ulong(missile_flying_time);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn reject(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    string_id: &[u8],
) -> QueuedSkillExecutionOutcome {
    game.send_battle_fairy_skill_failure(player_id, action);
    if !string_id.is_empty() {
        game.send_skill_system_info(player_id, string_id);
    }
    send_end(game, player_id, skill_level);
    terminal(QueuedSkillExecutionState::Rejected)
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

pub(crate) const fn is_fatal_blow_dispatch(dispatch: BattleFairySkillDispatch) -> bool {
    matches!(
        dispatch,
        BattleFairySkillDispatch::SelfTarget {
            skill_id: FATAL_BLOW_SKILL_ID,
            ..
        } | BattleFairySkillDispatch::Point {
            skill_id: FATAL_BLOW_SKILL_ID,
            ..
        } | BattleFairySkillDispatch::Object {
            skill_id: FATAL_BLOW_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
            ..
        }
    )
}

pub(crate) fn execute_battle_fairy_fatal_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let starting = player_ai.battle_fairy_execution(FATAL_BLOW_SKILL_ID).is_none();
    let reject_before_ai = |game: &mut CGame, level: i32, action: u8, text: &[u8]| {
        if action != 2 { game.send_battle_fairy_skill_failure(player_id, action); }
        if !text.is_empty() { game.send_skill_system_info(player_id, text); }
        send_end(game, player_id, level);
        if starting { game.send_battle_fairy_skill_failure(player_id, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let (skill_level, target) = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id: FATAL_BLOW_SKILL_ID,
            skill_level,
            ..
        } | BattleFairySkillDispatch::Point {
            skill_id: FATAL_BLOW_SKILL_ID,
            skill_level,
            ..
        } => return reject_before_ai(game, skill_level, 10, b"ZHGS0045"),
        BattleFairySkillDispatch::Object {
            skill_id: FATAL_BLOW_SKILL_ID,
            skill_level,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => (skill_level, target),
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(FATAL_BLOW_SKILL_ID, skill_level) else {
        return reject_before_ai(game, skill_level, 2, b"");
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let missile_step_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let damage_factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32;
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let skill_name = properties.skill_name().to_vec();
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.battle_fairy_execution(FATAL_BLOW_SKILL_ID).is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            return reject_before_ai(game, skill_level, 10, b"ZHGS0045");
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            game.send_skill_system_info(player_id, b"ZHGS0046");
            return reject_before_ai(game, skill_level, 2, b"");
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            game.send_skill_system_info(player_id, b"ZHGS0047");
            return reject_before_ai(game, skill_level, 2, b"");
        }
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(player_ai.battle_fairy_skill_last_used_ms(FATAL_BLOW_SKILL_ID), cooldown_ms, cooldown_now_ms) {
            return reject_before_ai(game, skill_level, 0x0d, b"ZHGS0048");
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            return reject_before_ai(game, skill_level, 10, b"ZHGS0045");
        };
        let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source_view.tile_x,
            source_view.tile_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            return reject_before_ai(game, skill_level, 0x0b, b"ZHGS0049");
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_battle_fairy_skill_failure(player_id, 0x0f);
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
            return reject_before_ai(game, skill_level, 2, b"");
        }
        let war_soul_mana = game
            .find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()));
        let Some(war_soul_mana) = war_soul_mana else {
            return reject_before_ai(game, skill_level, 2, b"");
        };
        if mp_loss != 0 {
            if i64::from(war_soul_mana) - i64::from(mp_loss) < 0 {
                game.send_battle_fairy_skill_failure(player_id, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                return reject_before_ai(game, skill_level, 2, b"");
            }
        }
        player_ai.begin_battle_fairy_state(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .battle_fairy_execution(FATAL_BLOW_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .battle_fairy_execution(FATAL_BLOW_SKILL_ID)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        if game.periodic_state_target_dead(region_id, target) {
            return reject(game, player_id, skill_level, 10, b"ZHGS0050");
        }
        if mp_loss != 0 {
            let goods_factory = game.goods_factory().clone();
            let da_kong_key = game.globe_setup().da_kong_key();
            let update = game.find_player_mut(player_id).and_then(|player| {
                player.spend_war_soul_mana(mp_loss, &goods_factory, da_kong_key)
            });
            let Some(update) = update else {
                game.send_battle_fairy_skill_failure(player_id, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                send_end(game, player_id, skill_level);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
        }
        send_cast(game, player_id, skill_level, 1, None, 0);
        if let Some(execution) = player_ai.battle_fairy_execution_mut(FATAL_BLOW_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .battle_fairy_execution(FATAL_BLOW_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение смертельного удара создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        return reject(game, player_id, skill_level, 10, b"ZHGS0050");
    };
    if game.periodic_state_target_dead(region_id, target) {
        return reject(game, player_id, skill_level, 10, b"ZHGS0050");
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        None,
    );
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        return reject(game, player_id, skill_level, 0x0b, b"ZHGS0049");
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.send_battle_fairy_skill_failure(player_id, 0x0f);
        game.send_skill_system_info_with_text(player_id, b"ZHGS0053", &skill_name);
        send_end(game, player_id, skill_level);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let missile_flying_time = missile_step_ms.wrapping_mul(path.len() as u32);
    send_cast(
        game,
        player_id,
        skill_level,
        2,
        Some((target, target_view.tile_x, target_view.tile_y)),
        missile_flying_time,
    );
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if player.war_soul_goods(game.goods_factory()).is_none() {
        return reject(game, player_id, skill_level, 2, b"");
    }
    let master = master_info(player);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CFatalBlowPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        skill_level,
        damage_factor,
        target,
        minimum_attack,
        maximum_attack,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let result = game.add_fatal_blow_phalanx(
        region_id,
        phalanx,
        target_view.tile_x,
        target_view.tile_y,
        summon_started_at_ms,
        runtime,
    );
    let summoned = result.is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_fatal_blow_phalanx_entry(region_id, summon_id, runtime);
    }
    if let Some(execution) = player_ai.battle_fairy_execution_mut(FATAL_BLOW_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_end(game, player_id, skill_level);
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::Rejected
    })
}
