//! Отложенный гром боевого духа `CLeiming2` (`0x21B`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunder2.cpp`. Владелец сохраняет проверки цели и пути,
//! задержку повторного использования, расход MP, ожидание, стадии
//! `SkillExecutionKernel`, визуальные
//! пакеты и построение однократной области `CLeimingPhalanx2`. Общие только
//! для пары громовых навыков построители пакета выполнения и отказа находятся
//! в `thunder.rs`; формула и жизненный цикл области остаются здесь и в
//! `thunder2phalanx.rs`. `CGame` разрешает владельцев, регистрирует область,
//! применяет атаку к целям и выполняет фактическую доставку.
//! Пара с `CThunder` использует то же усечение sprite через исходный `i64` и
//! отдельное усечение стихийного коэффициента в `i32`. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; ожидание сравнивает
//! unsigned now с wrapping(start + delay), cmp/jb 0x00520885.
//! Отказ объектного Begin завершает эффект через End(0), затем расписание
//! отправляет `4,2`; отказ уже начатого AI не повторяет этот общий ответ.
//! В Rust внешний 4,2 отправляет только координатор после общего End(0),
//! а concrete Begin и общий null-target helper сохраняют visual action 3.
//! AI после virtual Summon(+0x8C) в 0x005208AB сразу вызывает End(1).
//! Поэтому неудачная регистрация области после попытки Summon возвращает
//! RejectedAfterUse, сохраняя общий AfterUse и reuse без продления навыка.
//! Visual Update(mode=1, wire action 2) в AI (0x00520899) предшествует Summon. Отсутствие
//! GetWarSoulGoods внутри Summon (0x00520a02..0x00520a09 → 0x00520c99)
//! возвращает 0 без 4,2; вызывающий AI всё равно выполняет End(1).

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_MAX_ATTACK,
    SKILL_USAGE_MIN_ATTACK,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use super::thunder::{
    dispatch_position, master_info, reject_thunder_family, reject_thunder_null_target, scaled_battle_fairy_sprite,
    send_thunder_family_cast, terminal, thunder_element_modifier,
};
use super::thunder2phalanx::CLeimingPhalanx2;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const LEIMING2_SKILL_ID: u32 = 0x21b;
pub(crate) const LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0xd2;
const DENIED_STATE_C: u32 = 0x67;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

fn reject(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    string_id: &[u8],
) -> QueuedSkillExecutionOutcome {
    reject_thunder_family(
        game,
        player_id,
        LEIMING2_SKILL_ID,
        skill_level,
        action,
        string_id,
    )
}

pub(crate) fn execute_battle_fairy_leiming2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_level, skill_id) = match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Object { skill_id, skill_level, .. } => (skill_level, skill_id),
    };
    if skill_id != LEIMING2_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !matches!(dispatch, BattleFairySkillDispatch::Object { .. }) {
        return reject_thunder_null_target(game, player_id, LEIMING2_SKILL_ID, skill_level);
    }
    let reject_before_ai = |game: &mut CGame, action: u8, text: &[u8]| {
        if action != 2 { game.send_battle_fairy_skill_failure(player_id, action); }
        if !text.is_empty() { game.send_skill_system_info(player_id, text); }
        send_thunder_family_cast(game, player_id, LEIMING2_SKILL_ID, skill_level, 3, None);
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let Some(properties) = game.skill_base_properties(LEIMING2_SKILL_ID, skill_level) else {
        return reject_before_ai(game, 2, b"");
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let em_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, LEIMING2_SKILL_ID).is_none() {
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
        if !skill_is_restored(game.battle_fairy_skill_last_used_ms(player_id, LEIMING2_SKILL_ID), cooldown_ms, cooldown_now_ms) {
            return reject_before_ai(game, 0x0d, b"ZHGS0048");
        }
        let Some((target_x, target_y, _)) =
            dispatch_position(game, region_id, dispatch)
        else {
            return reject_before_ai(game, 10, b"ZHGS0050");
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
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_battle_fairy_skill_failure(player_id, 0x0f);
            game.send_skill_system_info(player_id, b"ZHGS0051");
            return reject_before_ai(game, 2, b"");
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
        game.begin_battle_fairy_state(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game
        .battle_fairy_execution(player_id, LEIMING2_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game
        .battle_fairy_execution(player_id, LEIMING2_SKILL_ID)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        if let BattleFairySkillDispatch::Object { target, .. } = dispatch
            && game.periodic_state_target_dead(region_id, target)
        {
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
                send_thunder_family_cast(
                    game, player_id, LEIMING2_SKILL_ID, skill_level, 3, None,
                );
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
        }
        send_thunder_family_cast(
            game, player_id, LEIMING2_SKILL_ID, skill_level, 1, None,
        );
        if let Some(execution) = game.battle_fairy_execution_mut(player_id, LEIMING2_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game
        .battle_fairy_execution(player_id, LEIMING2_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение отложенного грома создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some((target_x, target_y, target)) =
        dispatch_position(game, region_id, dispatch)
    else {
        return reject(game, player_id, skill_level, 10, b"ZHGS0050");
    };
    if target.is_some_and(|target| game.periodic_state_target_dead(region_id, target)) {
        return reject(game, player_id, skill_level, 10, b"ZHGS0050");
    }
    send_thunder_family_cast(
        game,
        player_id,
        LEIMING2_SKILL_ID,
        skill_level,
        2,
        Some((target_x, target_y)),
    );
    let Some(player) = game.find_player(player_id) else {
        send_thunder_family_cast(game, player_id, LEIMING2_SKILL_ID, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    };
    let Some(sprite) = player
        .war_soul_goods(game.goods_factory())
        .map(|goods| goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1))
    else {
        send_thunder_family_cast(game, player_id, LEIMING2_SKILL_ID, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    };
    let scaled_sprite = scaled_battle_fairy_sprite(sprite);
    let element_modifier = player.combat_properties().element_modify.wrapping_add(
        thunder_element_modifier(em_modifier, scaled_sprite),
    );
    let master = master_info(player);
    let cch = i32::from(player.combat_properties().cch);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CLeimingPhalanx2::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        skill_level,
        minimum_attack,
        maximum_attack,
        element_modifier,
        cch,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let summoned = game
        .add_leiming2_phalanx(
            region_id,
            phalanx,
            target_x,
            target_y,
            summon_started_at_ms,
            runtime,
        )
        .is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_leiming2_phalanx_entry(region_id, summon_id, runtime);
    }
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, LEIMING2_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_thunder_family_cast(
        game, player_id, LEIMING2_SKILL_ID, skill_level, 3, None,
    );
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::RejectedAfterUse
    })
}
