//! Небесный огонь боевого духа `CTianhuo` (`0x21A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/tianhuo.cpp`. Конкретный `CheckCastCondition` требует
//! объектную `CMoveShape`-цель. OnScheduleAboutWarSoul даже для координатного
//! запроса вызывает объектный Begin (0x005222f0) с null. Проверка
//! (0x005228d0) отказывает до reuse и MP; End(0) (0x005222a0) отправляет
//! action 3, затем расписание отправляет общий отказ 4, 2. Координатные
//! перегрузки не образуют отдельного исполняемого пути. Здесь находятся проверки состояния и
//! длины пути, задержка повторного использования, необратимый расход MP,
//! повторная проверка пути после расхода, поворот игрока, стадии
//! `SkillExecutionKernel`, точные визуальные пакеты и построение
//! `CTianhuoPhalanx`. Пакет стадии применения намеренно содержит legacy ID
//! `0x13A`, хотя ID навыка равен `0x21A`. `CGame` только разрешает владельцев,
//! регистрирует область, применяет результат к независимым владельцам и
//! выполняет доставку. Восстановление использует абсолютный срок
//! `CSkill::IsRestored`; стадийная задержка сравнивает unsigned now с
//! wrapping(start + delay), cmp/jb 0x005231e1.
//! Отказ объектного Begin (0x005222f0) завершает эффект через End(0), затем
//! расписание отправляет `4,2`; ошибки начатого AI сохраняют отдельный путь.

use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_MAX_ATTACK,
    SKILL_USAGE_MIN_ATTACK,
};
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use super::thunder::{dispatch_position, master_info, terminal};
use super::tianhuophalanx::CTianhuoPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const TIANHUO_SKILL_ID: u32 = 0x21a;
pub(crate) const TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const APPLICATION_VISUAL_SKILL_ID: i32 = 0x13a;
const PLAYER_TYPE: i32 = 400;
const VISUAL_OBJECT_TYPE: i32 = 700;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0xd2;
const DENIED_STATE_C: u32 = 0x67;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    match action {
        1 | 3 => {
            message.add_long(TIANHUO_SKILL_ID as i32);
            message.base_mut().add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        2 => {
            let (target, x, y) = target.unwrap_or((
                ShapeIdentity {
                    object_type: 0,
                    id: 0,
                    ex_id: crate::public::guid::CGuid::GUID_INVALID,
                },
                0,
                0,
            ));
            message.add_long(APPLICATION_VISUAL_SKILL_ID);
            message.base_mut().add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(target.object_type);
            message.add_long(target.id);
            message.add_long(x);
            message.add_long(y);
        }
        _ => return,
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
    send_visual(game, player_id, skill_level, 3, None);
    terminal(QueuedSkillExecutionState::Rejected)
}

pub(crate) fn execute_battle_fairy_tianhuo<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_level, skill_id) = match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Object { skill_id, skill_level, .. } => (skill_level, skill_id),
    };
    if skill_id != TIANHUO_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !matches!(dispatch, BattleFairySkillDispatch::Object { .. }) {
        send_visual(game, player_id, skill_level, 3, None);
        game.send_battle_fairy_skill_failure(player_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let starting = player_ai.tianhuo().is_none();
    let reject_before_ai = |game: &mut CGame, action: u8, text: &[u8]| {
        if action != 2 { game.send_battle_fairy_skill_failure(player_id, action); }
        if !text.is_empty() { game.send_skill_system_info(player_id, text); }
        send_visual(game, player_id, skill_level, 3, None);
        if starting { game.send_battle_fairy_skill_failure(player_id, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let Some(properties) = game.skill_base_properties(TIANHUO_SKILL_ID, skill_level) else {
        return reject_before_ai(game, 2, b"");
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER) as i32;

    if player_ai.tianhuo().is_none() {
        if let BattleFairySkillDispatch::Object { target, .. } = dispatch {
            if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
                || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C)
            {
                game.send_skill_system_info(player_id, b"ZHGS0046");
                return reject_before_ai(game, 2, b"");
            }
            if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B) {
                game.send_skill_system_info(player_id, b"ZHGS0047");
                return reject_before_ai(game, 2, b"");
            }
        }
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            player_ai.tianhuo_last_used_ms(),
            cooldown_ms,
            cooldown_now_ms,
        ) {
            return reject_before_ai(game, 0x0d, b"ZHGS0048");
        }
        let Some((target_x, target_y, _)) = dispatch_position(game, region_id, dispatch)
        else {
            return reject_before_ai(game, 10, b"");
        };
        let Some(source) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source.tile_x,
            source.tile_y,
            target_x,
            target_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            return reject_before_ai(game, 0x0b, b"ZHGS0049");
        }
        let Some(war_soul_mana) = game
            .find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else {
            return reject_before_ai(game, 2, b"");
        };
        if mp_loss != 0 && i64::from(war_soul_mana) - i64::from(mp_loss) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(
                player_id,
                b"ZHGS0052",
                battle_fairy_mana_text_cost(mp_loss),
            );
            return reject_before_ai(game, 2, b"");
        }
        player_ai.begin_tianhuo(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .tianhuo()
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .tianhuo()
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
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
                send_visual(game, player_id, skill_level, 3, None);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            let mut message = CMessage::new(update.message_type as i32);
            message.add_long(update.player_id);
            message.base_mut().add_guid(update.goods.ex_id);
            message.add_ulong(update.old_client_payload.len() as u32);
            message.base_mut().add(&update.old_client_payload);
            let _ = message.send_to_player(game.net_server(), player_id);
        }
        let Some((target_x, target_y, _)) = dispatch_position(game, region_id, dispatch)
        else {
            return reject(game, player_id, skill_level, 10, b"");
        };
        if let Some(player) = game.find_player_mut(player_id) {
            let source = player.shape();
            let direction = get_line_direction(
                source.get_tile_x().unwrap_or(0),
                source.get_tile_y().unwrap_or(0),
                target_x,
                target_y,
            );
            player.movement_shape_mut().set_direction(direction);
        }
        let Some(source) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source.tile_x,
            source.tile_y,
            target_x,
            target_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            return reject(game, player_id, skill_level, 0x0b, b"ZHGS0049");
        }
        send_visual(game, player_id, skill_level, 1, None);
        if let Some(execution) = player_ai.tianhuo_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .tianhuo()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение небесного огня создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some((target_x, target_y, target)) = dispatch_position(game, region_id, dispatch)
    else {
        return reject(game, player_id, skill_level, 10, b"");
    };
    if target.is_some_and(|target| game.periodic_state_target_dead(region_id, target)) {
        return reject(game, player_id, skill_level, 10, b"");
    }
    send_visual(
        game,
        player_id,
        skill_level,
        2,
        Some((
            target.unwrap_or(ShapeIdentity {
                object_type: 0,
                id: 0,
                ex_id: crate::public::guid::CGuid::GUID_INVALID,
            }),
            target_x,
            target_y,
        )),
    );
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let master = master_info(player);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CTianhuoPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        skill_level,
        minimum_attack,
        maximum_attack,
        element_modifier,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let summoned = game
        .add_tianhuo_phalanx(
            region_id,
            phalanx,
            target_x,
            target_y,
            summon_started_at_ms,
            runtime,
        )
        .is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_tianhuo_phalanx_entry(region_id, summon_id);
    }
    if let Some(execution) = player_ai.tianhuo_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_visual(game, player_id, skill_level, 3, None);
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::Rejected
    })
}
