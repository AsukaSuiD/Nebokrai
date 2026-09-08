//! Гром боевого духа `CThunder` (`0x21F`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunder.cpp`. Здесь находятся проверки цели и пути,
//! задержка повторного использования, расход MP, стадии
//! `SkillExecutionKernel`, визуальные пакеты и построение `CThunderPhalanx`.
//! `CGame` только разрешает владельцев,
//! регистрирует область в регионе и выполняет сетевую доставку.
//! Sprite сначала масштабируется через исходное усечение x87 в `i64` с
//! последующим чтением младших 32 бит; стихийный коэффициент усекается в `i32`.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`, а ожидание
//! стадии сравнивает unsigned now с wrapping(start + delay), cmp/jb 0x00521c25.

//! OnScheduleAboutWarSoul (0x00509861) вызывает объектный Begin даже для
//! координатного запроса, передавая null. CheckCastCondition (0x00521330,
//! для Leiming2 — 0x0051ff90) отклоняет null до проверки reuse и MP.
//! End(0) (0x005222a0) отправляет завершение эффекта; затем расписание
//! отправляет общий отказ. Область по переданным координатам не создаётся.
//! При остальных отказах Begin общий ответ `4,2` также следует после End(0);
//! ошибки уже начатого AI не повторяют ответ расписания.

use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
};
use super::battlefairytransfer::send_goods_update;
use super::fightdefense::truncate_original;
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use super::thunderphalanx::CThunderPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const THUNDER_SKILL_ID: u32 = 0x21f;
pub(crate) const THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;

pub(super) fn truncate_original_i64_low(value: f64) -> i32 {
    if !value.is_finite()
        || value < -9_223_372_036_854_775_808.0
        || value >= 9_223_372_036_854_775_808.0
    {
        i64::MIN as i32
    } else {
        (value as i64) as i32
    }
}

pub(super) fn scaled_battle_fairy_sprite(sprite: i32) -> i32 {
    truncate_original_i64_low(f64::from(sprite) * 0.0001)
}

pub(super) fn thunder_element_modifier(em_modifier: u32, scaled_sprite: i32) -> i32 {
    truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(scaled_sprite),
    )
}

pub(super) fn thunder_base_damage(target_damage_factor: u32, sprite: i32) -> i32 {
    truncate_original_i64_low(
        f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6,
    )
}
const PLAYER_TYPE: i32 = 400;
const VISUAL_OBJECT_TYPE: i32 = 700;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0xd2;
const DENIED_STATE_C: u32 = 0x67;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

pub(super) fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(super) fn dispatch_position(
    game: &CGame,
    region_id: i32,
    dispatch: BattleFairySkillDispatch,
) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        BattleFairySkillDispatch::SelfTarget { .. }
        | BattleFairySkillDispatch::Point { .. } => None,
        BattleFairySkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|shape| (shape.tile_x, shape.tile_y, Some(target))),
    }
}

pub(super) fn reject_thunder_null_target(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    skill_level: i32,
) -> QueuedSkillExecutionOutcome {
    send_thunder_family_cast(game, player_id, skill_id, skill_level, 3, None);
    game.send_battle_fairy_skill_failure(player_id, 2);
    terminal(QueuedSkillExecutionState::Rejected)
}

pub(super) fn send_thunder_family_cast(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    skill_level: i32,
    action: u8,
    target_position: Option<(i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(skill_id as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(VISUAL_OBJECT_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let Some((x, y)) = target_position else { return };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(super) fn reject_thunder_family(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    skill_level: i32,
    action: u8,
    string_id: &[u8],
) -> QueuedSkillExecutionOutcome {
    game.send_battle_fairy_skill_failure(player_id, action);
    if !string_id.is_empty() {
        game.send_skill_system_info(player_id, string_id);
    }
    send_thunder_family_cast(game, player_id, skill_id, skill_level, 3, None);
    terminal(QueuedSkillExecutionState::Rejected)
}

pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
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

pub(crate) fn execute_battle_fairy_thunder<Runtime: GameMainLoopRuntime>(
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
    if skill_id != THUNDER_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !matches!(dispatch, BattleFairySkillDispatch::Object { .. }) {
        return reject_thunder_null_target(game, player_id, THUNDER_SKILL_ID, skill_level);
    }
    let starting = game.battle_fairy_execution(player_id, THUNDER_SKILL_ID).is_none();
    let reject_before_ai = |game: &mut CGame, action: u8, text: &[u8]| {
        if action != 2 { game.send_battle_fairy_skill_failure(player_id, action); }
        if !text.is_empty() { game.send_skill_system_info(player_id, text); }
        send_thunder_family_cast(game, player_id, THUNDER_SKILL_ID, skill_level, 3, None);
        if starting { game.send_battle_fairy_skill_failure(player_id, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let Some(properties) = game.skill_base_properties(THUNDER_SKILL_ID, skill_level) else {
        return reject_before_ai(game, 2, b"");
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let target_count = properties.query_property(SKILL_USAGE_CONST);
    let em_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, THUNDER_SKILL_ID).is_none() {
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
        if !skill_is_restored(game.battle_fairy_skill_last_used_ms(player_id, THUNDER_SKILL_ID), cooldown_ms, cooldown_now_ms) {
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
        game.begin_battle_fairy_state(player_id, player_ai, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.battle_fairy_execution(player_id, THUNDER_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.battle_fairy_execution(player_id, THUNDER_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        if let BattleFairySkillDispatch::Object { target, .. } = dispatch
            && game.periodic_state_target_dead(region_id, target)
        {
            return reject_thunder_family(game, player_id, THUNDER_SKILL_ID, skill_level, 10, b"ZHGS0050");
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
                send_thunder_family_cast(game, player_id, THUNDER_SKILL_ID, skill_level, 3, None);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
        }
        send_thunder_family_cast(game, player_id, THUNDER_SKILL_ID, skill_level, 1, None);
        if let Some(execution) = game.battle_fairy_execution_mut(player_id, THUNDER_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game
        .battle_fairy_execution(player_id, THUNDER_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение грома создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some((target_x, target_y, target)) =
        dispatch_position(game, region_id, dispatch)
    else {
        return reject_thunder_family(game, player_id, THUNDER_SKILL_ID, skill_level, 10, b"ZHGS0050");
    };
    if target.is_some_and(|target| game.periodic_state_target_dead(region_id, target)) {
        return reject_thunder_family(game, player_id, THUNDER_SKILL_ID, skill_level, 10, b"ZHGS0050");
    }
    send_thunder_family_cast(game, player_id, THUNDER_SKILL_ID, skill_level, 2, Some((target_x, target_y)));
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(sprite) = player.war_soul_goods(game.goods_factory()).map(|goods| {
        goods.addon_property_value(
            game.goods_factory(),
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE,
            1,
        )
    }) else {
        return reject_thunder_family(game, player_id, THUNDER_SKILL_ID, skill_level, 2, b"");
    };
    let scaled_sprite = scaled_battle_fairy_sprite(sprite);
    let element_modifier = thunder_element_modifier(em_modifier, scaled_sprite);
    let master = master_info(player);
    let cch = i32::from(player.combat_properties().cch);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CThunderPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        skill_level,
        frequency_ms,
        minimum_attack,
        maximum_attack,
        element_modifier,
        target_count,
        cch,
    );
    phalanx.shape_mut().set_region_id(region_id);
    phalanx.initialize(target_x, target_y, &mut |maximum| game.skill_random_below(maximum));
    let summoned = game.add_thunder_phalanx(
        region_id,
        phalanx,
        target_x,
        target_y,
        summon_started_at_ms,
        runtime,
    )
    .is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_thunder_phalanx_entry(region_id, summon_id, runtime);
    }
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, THUNDER_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_thunder_family_cast(game, player_id, THUNDER_SKILL_ID, skill_level, 3, None);
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::Rejected
    })
}
