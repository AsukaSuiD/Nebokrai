//! Ядовитая стрела боевого духа `CPoisonArrow` (`0x21E`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonarrow.cpp`. Модуль сохраняет проверки цели и пути,
//! расход MP боевого духа, задержку, пакеты и замену периодического состояния.
//! `CGame` предоставляет опубликованных владельцев; установка и периодические
//! удары принадлежат `poisonarrowstate.rs`. Координатный `Begin` не создаёт
//! клеточную атаку: без object-target он проходит отказ `10 → ZHGS0045 → 2 → End → 2`:
//! первый 4,2 принадлежит Begin, последний — OnScheduleAboutWarSoul.
//! В Rust внутренний 4,2 перед visual-End остаётся здесь; внешний 4,2
//! отправляет только координатор после общего End(0), не concrete Begin.
//! CPoisonArrowEffect: object Begin 0x005190D0 после общей базы выделяет 0xC,
//! вызывает CVisualEffect(0x005DC200), ставит vtable 0x00656B30 и loop=1
//! до CheckCast. Update 0x005191A0 требует точный тип навыка, !ended и GetUser;
//! режимы 0/1/3 дают action 1/2/3, причём mode 1 требует live GetSufferer.
//! ID/level читаются из базы, failure требует CPlayer. Общий publisher
//! сохраняет безусловный хвост 0x005DC1E0; mode 13 даёт 4,13, в отличие
//! от BloodLoss. End(int) 0x0051A700 при наличии source повторно начинает effect
//! через BeginVisualEffect(1) перед mode 3; здесь остаются режимы Begin/AI,
//! а visual состояния цели принадлежит poisonarrowstate.rs.
//! Reuse использует exact `CSkill::IsRestored`; ожидание навыка сравнивает
//! unsigned now с wrapping(start + delay), cmp/jb 0x00519fa6.
//! Периодические часы принадлежат владельцу состояния.
//! AI 0x00519D70 не делает RNG. После mode1 снимается MasterInfo с country=0,
//! затем PK OnFirstSkill читает security по координатам source в регионе цели
//! (0x0051A2AB..0x0051A2D3). CONST/frequency/keep и ctor 0x0051A32D идут
//! до первого старого ID21E: End → destructor свежего остатка той же позиции →
//! Begin(source,target) → прежний слот либо append. Отказ Begin не меняет
//! успешный End(1) навыка; внешняя публикация списка состояний здесь отсутствует.
//! Первый AI требует goods духа и при NULL остаётся Pending; MP0 также проходит
//! signed wrapping MP-gate, запись и BF918 даже при отказе SerializeForOldClient.
//! После пакета CAN_BE_BREAKED записывается в базовую available перед mode0,
//! отдельно от derived допуска AI.
//! Начальные часы задаёт общий Begin расписания; CheckCast 0x00519750
//! читает часы только для reuse и выбирает первый запрещающий state по позиции.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{
    SkillExecutionKernel, SkillStage, battle_fairy_mana_text_cost, skill_is_restored,
};
use super::poisonarrowstate::{PoisonArrowState, begin_primary_poison_arrow_state};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const POISON_ARROW_SKILL_ID: u32 = 0x21e;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0x67;
const DENIED_STATE_C: u32 = 0xd2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

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

pub(crate) fn execute_battle_fairy_poison_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_level, target) = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id: POISON_ARROW_SKILL_ID,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Point {
            skill_id: POISON_ARROW_SKILL_ID,
            skill_level,
            ..
        } => (skill_level, None),
        BattleFairySkillDispatch::Object {
            skill_id: POISON_ARROW_SKILL_ID,
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
    let starting = game.battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID).is_none();
    let reject_before_ai = |game: &mut CGame| {
        if starting { game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let Some(properties) = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level) else {
        return reject_before_ai(game);
    };
    let Some(target) = target else {
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
        game.send_skill_system_info(player_id, b"ZHGS0045");
        return reject_before_ai(game);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);

    if game.battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID).is_none() {
        if target.id == player_id && target.object_type == PLAYER_TYPE {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
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
        let started_at_ms = game.player_skill_lifecycle(player_id, POISON_ARROW_SKILL_ID)
            .expect("общий Begin расписания сохранил базу отравленной стрелы")
            .started_at_ms();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.battle_fairy_skill_last_used_ms(player_id, POISON_ARROW_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return reject_before_ai(game);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
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
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 0x0b);
            game.send_skill_system_info(player_id, b"ZHGS0049");
            return reject_before_ai(game);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 0x0f);
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
            return reject_before_ai(game);
        }
        let mp_loss = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level)
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
                game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 7);
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
    } else if game
        .battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if resolve_state_move_shape(game, region_id, target).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.periodic_state_target_dead(region_id, target) {
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game
        .battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID)
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(current) = game.find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else { return terminal(QueuedSkillExecutionState::Pending) };
        let mp_loss = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level)
            .expect("свойства исполняемого навыка сохраняются")
            .query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 7);
            game.send_skill_system_info_with_unsigned(
                player_id, b"ZHGS0052", battle_fairy_mana_text_cost(mp_loss),
            );
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let update = game.spend_war_soul_mana_record(player_id, mp_loss)
            .map(|(update, _encoded)| update);
        let Some(update) = update else { return terminal(QueuedSkillExecutionState::Rejected) };
        send_goods_update(game, &update);
        let can_be_breaked = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level)
            .expect("свойства исполняемого навыка сохраняются")
            .query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(execution) = game.battle_fairy_execution_mut(player_id, POISON_ARROW_SKILL_ID) {
            execution.lifecycle_mut().set_available(can_be_breaked != 0);
        }
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 0);
        if let Some(state) = game.battle_fairy_execution_mut(player_id, POISON_ARROW_SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game
        .battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение ядовитой стрелы создано или восстановлено");
    let delay_ms = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level)
        .expect("свойства исполняемого навыка сохраняются")
        .query_property(SKILL_USAGE_DELAY_TIME);
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source_view = game
        .find_player(player_id)
        .and_then(CPlayer::shape_view)
        .expect("владелец ядовитой стрелы сохранён");
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        None,
    );
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == 2)
    {
        let action = if maximum_distance != 0 && path.len() > maximum_distance as usize {
            0x0b
        } else {
            0x0f
        };
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, action);
        if action == 0x0b {
            game.send_skill_system_info(player_id, b"ZHGS0049");
        } else {
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 1);
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
        let properties = game.skill_base_properties(POISON_ARROW_SKILL_ID, skill_level)?;
        let hp_loss = properties.query_property(SKILL_USAGE_CONST);
        let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
        let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
        let state = PoisonArrowState::new(master, keep_time_ms, frequency_ms, hp_loss);
        let shape = resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
        let placement = if let Some((position, key)) = shape.find_state_position(|state| state.state_id() == POISON_ARROW_SKILL_ID) {
            let location = shape.applied_state_replacement_location(key)?;
            let _ = end_and_destroy_state_at(game, sufferer.0, sufferer.1, position);
            Some(location)
        } else { None };
        begin_primary_poison_arrow_state(
            game, sufferer.0, sufferer.1, Some(user), Some(sufferer), state, placement,
            &mut || runtime.now_milliseconds(),
        )
    });
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, POISON_ARROW_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

// Ниже сохранены точные координатные перегрузки как локальное доказательство
// отсутствующей object-target и обязательного failure-tail.
// ============================================================================
// FUNCTION: CPoisonArrow::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrow.cpp:197
// RVA: 0x00118F00
// ADDRESS: 00518f00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPoisonArrow::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrow.cpp:214
// RVA: 0x00118FD0
// ADDRESS: 00518fd0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
