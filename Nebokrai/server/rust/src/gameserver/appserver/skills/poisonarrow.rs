//! Ядовитая стрела боевого духа `CPoisonArrow` (`0x21E`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonarrow.cpp`. Модуль сохраняет проверки цели и пути,
//! расход MP боевого духа, задержку, пакеты и замену периодического состояния.
//! `CGame` предоставляет владельцев, регион и атомарную установку состояния
//! игрока или монстра; последующие периодические удары принадлежат
//! `poisonarrowstate.rs`. Координатный `Begin` не создаёт клеточную атаку: без
//! object-target он проходит отказ `10 → ZHGS0045 → 2 → End → 2`:
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

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{
    SkillExecutionKernel, SkillStage, battle_fairy_mana_text_cost, skill_is_restored,
};
use super::poisonarrowstate::{PoisonArrowState, send_poison_arrow_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
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
        master_country_id: i32::from(player.country()),
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
    _player_ai: &mut CPlayerAI,
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
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID).is_none() {
        if target.id == player_id && target.object_type == PLAYER_TYPE {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            return reject_before_ai(game);
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            game.send_skill_system_info(player_id, b"ZHGS0046");
            return reject_before_ai(game);
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            game.send_skill_system_info(player_id, b"ZHGS0047");
            return reject_before_ai(game);
        }
        let started_at_ms = runtime.now_milliseconds();
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
        if mp_loss != 0 {
            let Some(current) = game
                .find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else {
                return reject_before_ai(game);
            };
            if i64::from(current) - i64::from(mp_loss) < 0 {
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

    if game
        .battle_fairy_execution(player_id, POISON_ARROW_SKILL_ID)
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let target_alive = game
            .base_magic_target_view(region_id, target)
            .is_some_and(|_| !game.periodic_state_target_dead(region_id, target));
        if !target_alive {
            game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 {
            let goods_factory = game.goods_factory().clone();
            let da_kong_key = game.globe_setup().da_kong_key();
            let update = game.find_player_mut(player_id).and_then(|player| {
                player.spend_war_soul_mana(mp_loss, &goods_factory, da_kong_key)
            });
            let Some(update) = update else {
                game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
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
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.periodic_state_target_dead(region_id, target) {
        game.update_player_skill_visual(player_id, POISON_ARROW_SKILL_ID, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
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
    if target.object_type == PLAYER_TYPE {
        let _ = game.player_on_first_skill(player_id, target.id, Some(region_id), runtime);
    }
    let master = game
        .find_player(player_id)
        .map(master_info)
        .expect("владелец ядовитой стрелы сохранён");
    let state_now_ms = runtime.now_milliseconds();
    let state = PoisonArrowState::new(master, state_now_ms, keep_time_ms, frequency_ms, hp_loss);
    if let Some((previous, identity, x, y)) = game.replace_poison_arrow_state(region_id, target, state) {
        if let Some(previous) = previous {
            send_poison_arrow_state_visual(game, region_id, identity, x, y, previous, false, state_now_ms);
        }
        send_poison_arrow_state_visual(game, region_id, identity, x, y, state, true, state_now_ms);
        if target.object_type == PLAYER_TYPE {
            let _ = game.publish_player_states(target.id);
        }
    }
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
