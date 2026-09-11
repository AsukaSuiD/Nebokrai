//! Потеря крови боевого духа `CBloodLoss` (`0x21D`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bloodloss.cpp`. Модуль сохраняет проверки цели и пути,
//! расход MP боевого духа, задержку, пакеты и замену периодического состояния.
//! `CGame` предоставляет опубликованных владельцев; установка состояния
//! и последующие периодические удары принадлежат `bloodlossstate.rs`.
//! Координатный `Begin` не создаёт клеточную атаку: без
//! object-target он проходит отказ `10 → ZHGS0045 → 2 → End → 2`:
//! первый 4,2 принадлежит Begin, последний — OnScheduleAboutWarSoul.
//! В Rust внутренний 4,2 перед visual-End остаётся здесь; внешний 4,2
//! отправляет только координатор после общего End(0), не concrete Begin.
//! CBloodLossEffect: object Begin 0x0051A770 после общей базы выделяет 0xC,
//! вызывает CVisualEffect(0x005DC200), ставит vtable 0x00656BE8 и loop=1
//! до CheckCast. Update 0x0051A840 требует точный тип навыка, !ended и GetUser;
//! режимы 0/1/3 дают action 1/2/3, причём mode 1 требует live GetSufferer.
//! ID/level читаются из базы, failure требует CPlayer; mode 13 намеренно даёт
//! bytes 0,13 (0x0051ABC6/0x0051ABD9). Общий publisher сохраняет эти детали
//! и безусловный хвост 0x005DC1E0. End(int) 0x0051A700 при наличии source
//! повторно начинает effect через BeginVisualEffect(1) перед mode 3; здесь остаются
//! только режимы Begin/AI, а visual состояния цели принадлежит bloodlossstate.rs.
//! Коэффициент периодического урона вычисляется в x87 из полного unsigned
//! `u32` и `0.01_f32`, после чего единожды сохраняется как `f32`.
//! Модификатор урона сохраняет исходное усечение x87 через 64-битное целое,
//! младшие 32 бита которого затем переводятся в `float` как беззнаковое число.
//! Reuse использует exact `CSkill::IsRestored`; ожидание навыка сравнивает
//! unsigned now с wrapping(start + delay), cmp/jb 0x0051b649.
//! Периодические часы принадлежат владельцу состояния.
//! AI 0x0051B410: mode1 → MasterInfo с country=0 → PK по source XY в регионе
//! цели → goods духа → MAX/MIN/BF_ATTACK → factor/frequency/keep → ctor.
//! При NULL goods минимум, максимум и модификатор равны нулю
//! (0x0051B991..0x0051B9A2). RNG и critical принадлежат периодическому owner.
//! Первый старый ID21D проходит End → destructor свежего остатка той же позиции
//! → Begin → прежний слот либо append. Отказ Begin не отменяет End(1) навыка;
//! дополнительной публикации списка состояний нет.
//! Первый AI при NULL goods остаётся Pending; MP0 также проходит signed
//! wrapping-проверку, запись и BF918, включая отказ SerializeForOldClient.
//! После пакета CAN_BE_BREAKED записывается в базовую available перед mode0.
//! Начальные часы уже принадлежат общему Begin расписания; CheckCast повторно
//! читает часы только для reuse и выбирает первый запрещающий state в арене.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{
    SkillExecutionKernel, SkillStage, battle_fairy_mana_text_cost, skill_is_restored,
};
use super::bloodlossstate::{BloodLossState, begin_primary_blood_loss_state};
use super::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_ATTACK;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const BLOOD_LOSS_SKILL_ID: u32 = 0x21d;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0x67;
const DENIED_STATE_C: u32 = 0xd2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
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
        master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(crate) fn execute_battle_fairy_blood_loss<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_level, target) = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id: BLOOD_LOSS_SKILL_ID,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Point {
            skill_id: BLOOD_LOSS_SKILL_ID,
            skill_level,
            ..
        } => (skill_level, None),
        BattleFairySkillDispatch::Object {
            skill_id: BLOOD_LOSS_SKILL_ID,
            skill_level,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
            (skill_level, Some(target))
        }
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let starting = game.battle_fairy_execution(player_id, BLOOD_LOSS_SKILL_ID).is_none();
    let reject_before_ai = |game: &mut CGame| {
        if starting { game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    if game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level).is_none() {
        return reject_before_ai(game);
    }
    let Some(target) = target else {
        game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 10);
        game.send_skill_system_info(player_id, b"ZHGS0045");
        return reject_before_ai(game);
    };

    if game.battle_fairy_execution(player_id, BLOOD_LOSS_SKILL_ID).is_none() {
        if target.id == player_id && target.object_type == PLAYER_TYPE {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            return reject_before_ai(game);
        }
        let denied_state = resolve_state_move_shape(game, region_id, target).and_then(|shape| {
            let (_, key) = shape.find_state_position(|state| {
                matches!(state.state_id(), DENIED_STATE_A | DENIED_STATE_B | DENIED_STATE_C)
            })?;
            Some(shape.applied_state_data(key)?.state_id())
        });
        if let Some(state_id) = denied_state {
            game.send_skill_system_info(
                player_id, if state_id == DENIED_STATE_C { b"ZHGS0047" } else { b"ZHGS0046" },
            );
            return reject_before_ai(game);
        }
        let started_at_ms = game.player_skill_lifecycle(player_id, BLOOD_LOSS_SKILL_ID)
            .expect("общий Begin расписания сохранил базу потери крови")
            .started_at_ms();
        let reuse_delay_ms = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
            .expect("свойства допуска навыка сохраняются")
            .query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.battle_fairy_skill_last_used_ms(player_id, BLOOD_LOSS_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return reject_before_ai(game);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            return reject_before_ai(game);
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
        let maximum_distance = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
            .expect("свойства допуска навыка сохраняются")
            .query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 0x0b);
            game.send_skill_system_info(player_id, b"ZHGS0049");
            return reject_before_ai(game);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 0x0f);
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
            return reject_before_ai(game);
        }
        let mp_loss = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
            .expect("свойства допуска навыка сохраняются")
            .query_property(SKILL_USAGE_USER_MP_LOSE);
        if mp_loss != 0 {
            let Some(current) = game
                .find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else {
                return reject_before_ai(game);
            };
            if current.wrapping_sub(mp_loss as i32) < 0 {
                game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                return reject_before_ai(game);
            }
        }
        game.begin_battle_fairy_state(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.battle_fairy_execution(player_id, BLOOD_LOSS_SKILL_ID)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if resolve_state_move_shape(game, region_id, target).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.periodic_state_target_dead(region_id, target) {
        game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.battle_fairy_execution(player_id, BLOOD_LOSS_SKILL_ID)
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(current) = game.find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else { return terminal(QueuedSkillExecutionState::Pending) };
        let mp_loss = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
            .expect("свойства исполняемого навыка сохраняются")
            .query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 7);
            game.send_skill_system_info_with_unsigned(
                player_id, b"ZHGS0052", battle_fairy_mana_text_cost(mp_loss),
            );
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let goods_factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let update = game.find_player_mut(player_id).and_then(|player| {
            player.spend_war_soul_mana_record(mp_loss, &goods_factory, da_kong_key)
                .map(|(update, _encoded)| update)
        });
        let Some(update) = update else { return terminal(QueuedSkillExecutionState::Rejected) };
        send_goods_update(game, &update);
        let can_be_breaked = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
            .expect("свойства исполняемого навыка сохраняются")
            .query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(execution) = game.battle_fairy_execution_mut(player_id, BLOOD_LOSS_SKILL_ID) {
            execution.lifecycle_mut().set_available(can_be_breaked != 0);
        }
        game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 0);
        if let Some(state) = game.battle_fairy_execution_mut(player_id, BLOOD_LOSS_SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.battle_fairy_execution(player_id, BLOOD_LOSS_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение потери крови создано или восстановлено");
    let delay_ms = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
        .expect("свойства исполняемого навыка сохраняются")
        .query_property(SKILL_USAGE_DELAY_TIME);
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source_view = game
        .find_player(player_id)
        .and_then(CPlayer::shape_view)
        .expect("владелец потери крови сохранён");
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        None,
    );
    let maximum_distance = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
        .expect("свойства исполняемого навыка сохраняются")
        .query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == 2)
    {
        let action = if maximum_distance != 0 && path.len() > maximum_distance as usize {
            0x0b
        } else {
            0x0f
        };
        game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, action);
        if action == 0x0b {
            game.send_skill_system_info(player_id, b"ZHGS0049");
        } else {
            let skill_name = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)
                .expect("свойства исполняемого навыка сохраняются")
                .skill_name().to_vec();
            game.send_skill_system_info_with_text(player_id, b"ZHGS0053", &skill_name);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_player_skill_visual(player_id, BLOOD_LOSS_SKILL_ID, 1);
    let _ = game.with_published_player_ai(player_id, player_ai, |game| {
        let player = game.find_player(player_id)?;
        let master = master_info(player);
        let user = (player.shape().get_region_id(), player.shape().identity());
        let shape = resolve_state_move_shape(game, region_id, target)?;
        let sufferer = (shape.shape().get_region_id(), shape.shape().identity());
        if target.object_type == PLAYER_TYPE {
            let source_y = player.shape().get_tile_y().ok()?;
            let source_x = player.shape().get_tile_x().ok()?;
            let _ = game.player_on_first_skill_at_position(
                player_id, target.id, sufferer.0, source_x, source_y, runtime,
            );
        }
        let goods = game.find_player(player_id)?.war_soul_goods(game.goods_factory());
        let properties = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level)?;
        let (minimum_attack, maximum_attack, damage_modifier) = if let Some(goods) = goods {
            let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as u16;
            let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as u16;
            let value = goods.addon_property_value(game.goods_factory(), GAP_BF_ATTACK, 1);
            let scaled = truncate_original_i64_low(f64::from(value) * 0.0001);
            (minimum_attack, maximum_attack, f64::from(scaled as u32) as f32)
        } else { (0, 0, 0.0) };
        let damage_factor = (f64::from(
            properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR),
        ) * f64::from(0.01_f32)) as f32;
        let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
        let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
        let state = BloodLossState::new(
            master, keep_time_ms, frequency_ms, damage_factor, damage_modifier,
            minimum_attack, maximum_attack,
        );
        let shape = resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
        let placement = if let Some((position, key)) = shape.find_state_position(|state| state.state_id() == BLOOD_LOSS_SKILL_ID) {
            let location = shape.applied_state_replacement_location(key)?;
            let _ = end_and_destroy_state_at(game, sufferer.0, sufferer.1, position);
            Some(location)
        } else { None };
        begin_primary_blood_loss_state(
            game, sufferer.0, sufferer.1, Some(user), Some(sufferer), state, placement,
            &mut || runtime.now_milliseconds(),
        )
    });
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, BLOOD_LOSS_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

// Ниже сохранены точные координатные перегрузки как локальное доказательство
// отсутствующей object-target и обязательного failure-tail.
// ============================================================================
// FUNCTION: CBloodLoss::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodloss.cpp:197
// RVA: 0x0011A550
// ADDRESS: 0051a550
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLoss::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodloss.cpp:214
// RVA: 0x0011A620
// ADDRESS: 0051a620
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
