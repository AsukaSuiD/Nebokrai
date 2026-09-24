//! Владелец диспетчера сообщений организаций GameServer `OnOrgasysMessage`.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! исходный владелец `appserver/message/organsysmessage.cpp`. Реализованы
//! жизненный цикл фракций, городские и деревенские войны, FourNation, задания,
//! сценарии игрока, городские ворота, налоговые сеансы и region-control.
//!
//! Обработчик сохраняет точный разбор полей, корреляцию идентификатора и пароля
//! сеанса, порядок списания денег и предметов, мутации владельцев регионов и
//! маршрутизацию World/клиент. Для подвига FourNation сохранён порядок
//! `0xBF80C` → ограниченное состояние игрока → `UpdateProperty` →
//! `0xBF806(GS1177)`, а ответ результатов по-прежнему содержит ровно пять
//! счётчиков в `0x60319`.
//!
//! Налоговые prompt/result возникают в сетевом владельце и проходят через
//! общий типизированный `GameEffectJournal` в порядке FIFO. Остальные эффекты
//! выполняются синхронно и в журнал не копируются. Диагностические сведения о
//! корреляции и доставке публикуются через `tracing` в месте возникновения и
//! не возвращаются деревьями отчётов.
//! `0x7FE25` замыкает city guard snapshot через реальные `CGame/CMonster` и
//! region spawn owners без прежних monster/spawn callbacks runtime-а.
//! `0x7FE24` аналогично передаёт полный city player/gate проход `CGame`, не
//! оставляя возврат игроков внешнему callback-у.
//! OrganSys больше не получает process RNG: `CPlayer::ChangeRegion`, tax
//! password и FourNation relocation расходуют единый поток `CGame`. Runtime
//! остаётся только у достигнутых clock/container контрактов соседних ветвей;
//! Nation NPC/monster combat исполняют concrete owners.
//! Обновление списков городских и деревенских contender-ов также читает
//! faction, публикует `0xBFF29` и меняет `0xBFF28` через canonical `CGame`,
//! не делегируя эти четыре операции process runtime-у.
//! FourNation declare и refresh также принадлежат concrete Nation owner-у:
//! первый сбрасывает подтверждённое боевое состояние, второй восстанавливает
//! четыре исходных magic-stone NPC только при отсутствии каждого имени;
//! concrete clock/log/spatial spawn-effects выполняет canonical `CGame`.
//! Village timeout сохраняет `0x60136 → GS0240`: первый эффект отправляет
//! `CGame`, второй остаётся у достигнутого war-log sink.
//! Village end передаёт `goods × players` snapshot владельцу `CGame` и
//! возвращает region owner только для финального таймера/ownership reset.
//! City timeout аналогично сохраняет `ownership → 0x60138 → war-log`, причём
//! сетевой кадр больше не является runtime callback-ом.
//! Honor-list команды `0x9012A..0x9012D` читают тот же `CHonorRanks` snapshot
//! и публикуют исходные client frames `0xBFF32..0xBFF35` через `CGame`.
//! Quest-команды `0x90124..0x90128` сохраняют World forwarding и state-zero
//! запуск complete/abandon scripts из canonical `CQuestSystem`.
//! Остальная client relay-семья `0x90102`, `0x90107..0x90121` сохраняет исходное
//! соответствие World opcodes, payload и позицию дописанного player ID.
//! World responses `0x7FE02..05`, `0x7FE08..17`, `0x7FE1A..1C` сохраняют
//! исходный buffer, включая уже прочитанный address ID; `0x7FE09` использует
//! attached player context, а
//! `0x7FE16/17` сохраняют legacy-пропуск client opcode `0xBFF16`.
//! Region-control `0x7FE26/27/2B/2C/2D` сохраняет map-order суточного сбора,
//! приоритет local→proxy для ownership и exact virtual-аргументы, подтверждённые
//! vtable `CServerRegion` в исходном EXE/PDB.
//! City refresh `0x7FE2E` возвращает guard targets из owner-а; создание,
//! регистрация и публикация монстров выполняются canonical `CGame`, поэтому
//! organizing runtime больше не владеет monster/spatial effects.

use std::ffi::CString;
use thiserror::Error;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityPhaseContext, CAttackCitySys,
};
use super::super::organizingsystem::fournationwarsys::FourNationPhaseContext;
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarPhaseContext,
};
use super::super::region::{RegionCellAccessBlock, RegionRandomContext};
use super::super::serverregion::{
    RegionMembershipBlock, RegionTaxSessionKind, ServerRegionNpcSetup,
};
use super::super::serverwarregion::{WarRegionClearContext, WarRegionContext};
use super::super::shape::{ShapeCoordinateBlock, ShapeIdentity};
use nebokrai_shared::protocol::LegacyReader;
use crate::gameserver::gameserver::game::{
    CGame, GameClockContext, GameWarRegionHandle, LegacyFormatArgument,
    PlayerRegionChangeContext, ServerRegionOwner, colored_player_notice_message,
    format_legacy_mixed, format_legacy_text_fields, game_tick_milliseconds,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::netsessionmanager::NetSessionCallbackOutcome;
use crate::public::tools::{add_game_error_log_text, add_game_log_text, put_string_to_file};
use tracing::trace;

pub(crate) trait GameOrganizingWarRuntime: PlayerRegionChangeContext {}

impl<T: PlayerRegionChangeContext + ?Sized> GameOrganizingWarRuntime for T {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationReliveBlock {
    CountryOutsideRectangles { country: u8 },
    RandomPosition(RegionCellAccessBlock),
    Coordinate(crate::gameserver::appserver::shape::ShapeCoordinateBlock),
    Position(RegionMembershipBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationClearBlock {
    MonsterCoordinate {
        monster_id: i32,
        block: ShapeCoordinateBlock,
    },
    NpcRemoval {
        npc_id: i32,
        block: RegionMembershipBlock,
    },
    NpcTraversalMismatch {
        npc_id: i32,
    },
}

fn report_four_nation_relive_block(player_id: i32, block: FourNationReliveBlock) {
    let text = format!("FourNation: возврат игрока {player_id} остановлен: {block:?}");
    add_game_error_log_text(text.as_bytes());
}

fn report_four_nation_clear_block(block: FourNationClearBlock) {
    let text = format!("FourNation: очистка войны остановлена для объекта: {block:?}");
    add_game_error_log_text(text.as_bytes());
}

pub(crate) trait WarFactionUpdateContext {
    fn update_attack_city_contend_player(&mut self, region_id: i32, schedules: &CAttackCitySys);
    fn update_village_contend_player(&mut self, region_id: i32, schedules: &CVillageWarSys);
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum WarFactionUpdateDispatchError {
    #[error("AttackCity faction update: {0}")]
    AttackCity(#[source] AttackCityDecodeError),
    #[error("Village faction update: {0}")]
    Village(#[source] VillageWarDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerQuestCommandKind {
    Add,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum GamePlayerQuestCommandError {
    #[error("quest command не содержит player ID")]
    MissingPlayerId,
    #[error("quest command не содержит quest ID")]
    MissingQuestId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOrganizingMessageError {
    FactionLifecycle(FactionLifecycleDispatchError),
    FactionBillboard(FactionLifecycleDispatchError),
    CityGate(FactionLifecycleDispatchError),
    VillageApplication(FactionLifecycleDispatchError),
    CityApplication(FactionLifecycleDispatchError),
    FactionUpdate(WarFactionUpdateDispatchError),
    Phase(WarPhaseDispatchError),
    Control(OrganizingControlDispatchError),
    PlayerQuest(GamePlayerQuestCommandError),
    PlayerRunScript(FactionLifecycleDispatchError),
    RegionTax(FactionLifecycleDispatchError),
    RegionControl(FactionLifecycleDispatchError),
    HonorRanks(FactionLifecycleDispatchError),
    QuestActions(FactionLifecycleDispatchError),
    ClientRelay(FactionLifecycleDispatchError),
    ClientResponse(FactionLifecycleDispatchError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionLifecycleDispatchError {
    UnexpectedEnd { field: &'static str },
    MissingPlayer,
    InvalidPayload,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum OrganizingControlDispatchError {
    #[error("OrganSys control {field} с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("для country {country_id} не опубликован _max_country_treasury")]
    CountryTreasuryLimitMissing {
        country_id: u8,
    },
    #[error("для FourNation exploit игрока {player_id} не опубликован _max_exploit")]
    CountryExploitLimitMissing {
        player_id: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum WarPhaseDispatchError {
    #[error("war phase ID с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

/// Обрабатывает только доказанные faction-update opcodes `0x7FE35/0x7FE36`.
pub(crate) fn dispatch_war_faction_update<Context: WarFactionUpdateContext>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    context: &mut Context,
) -> Option<Result<(), WarFactionUpdateDispatchError>> {
    match opcode {
        0x7fe35 => {
            let region_id = match attack_city_sys.update_apply_war_factions(payload, cursor) {
                Ok(region_id) => region_id,
                Err(error) => {
                    return Some(Err(WarFactionUpdateDispatchError::AttackCity(error)));
                }
            };
            if let Some(region_id) = region_id {
                context.update_attack_city_contend_player(region_id, attack_city_sys);
            }
            trace!(opcode, schedule_found = region_id.is_some(), "Обновлены участники городской войны");
            Some(Ok(()))
        }
        0x7fe36 => {
            let region_id = match village_war_sys.update_apply_war_factions(payload, cursor) {
                Ok(region_id) => region_id,
                Err(error) => {
                    return Some(Err(WarFactionUpdateDispatchError::Village(error)));
                }
            };
            if let Some(region_id) = region_id {
                context.update_village_contend_player(region_id, village_war_sys);
            }
            trace!(opcode, schedule_found = region_id.is_some(), "Обновлены участники деревенской войны");
            Some(Ok(()))
        }
        _ => None,
    }
}

/// Обрабатывает только доказанные фазовые opcodes городских и деревенских войн.
pub(crate) fn dispatch_war_phase<Context>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    context: &mut Context,
) -> Option<Result<(), WarPhaseDispatchError>>
where
    Context: AttackCityPhaseContext + VillageWarPhaseContext,
{
    if !matches!(opcode, 0x7fe1f..=0x7fe25 | 0x7fe2f..=0x7fe33) {
        return None;
    }

    let war_number = match read_phase_war_number(payload, cursor) {
        Ok(war_number) => war_number,
        Err(error) => return Some(Err(error)),
    };
    match opcode {
        0x7fe1f => attack_city_sys.on_declar_war(war_number, context),
        0x7fe20 => attack_city_sys.on_attack_city_start(war_number, context),
        0x7fe21 => attack_city_sys.on_attack_city_time_out(war_number, context),
        0x7fe22 => attack_city_sys.on_attack_city_end(war_number, context),
        0x7fe23 => attack_city_sys.on_mass(war_number, context),
        0x7fe24 => attack_city_sys.on_clear_other_player(war_number, context),
        0x7fe25 => attack_city_sys.on_refresh_region(war_number, context),
        0x7fe2f => village_war_sys.on_delcare_war(war_number, context),
        0x7fe30 => village_war_sys.on_attack_village_start(war_number, context),
        0x7fe31 => village_war_sys.on_attack_village_out_time(war_number, context),
        0x7fe32 => village_war_sys.on_attack_village_end(war_number, context),
        0x7fe33 => village_war_sys.on_clear_player(war_number, context),
        _ => unreachable!("opcode отфильтрован перед чтением payload"),
    }
    trace!(opcode, war_number, "Обработана фаза войны");
    Some(Ok(()))
}

fn dispatch_game_player_quest_command(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), GamePlayerQuestCommandError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(GamePlayerQuestCommandError::MissingPlayerId)?;
    let quest_id = message
        .base_mut()
        .get_short()
        .ok_or(GamePlayerQuestCommandError::MissingQuestId)? as u16;
    let kind = if opcode == 0x7fe38 {
        GamePlayerQuestCommandKind::Add
    } else {
        GamePlayerQuestCommandKind::Remove
    };
    let player_found = game.find_player(player_id).is_some();
    if player_found {
        match kind {
            GamePlayerQuestCommandKind::Add => game.add_script_player_quest(player_id, quest_id),
            GamePlayerQuestCommandKind::Remove => {
                game.remove_script_player_quest(player_id, quest_id)
            }
        }
    }
    trace!(opcode, ?kind, player_id, quest_id, player_found, "Обработана команда задания игрока");
    Ok(())
}

fn dispatch_game_player_run_script(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    let script = message.base_mut().get_str_bytes(0x100).ok_or(
        FactionLifecycleDispatchError::UnexpectedEnd {
            field: "script path",
        },
    )?;
    let player_found = game.find_player(player_id).is_some();
    let player_alive = game
        .find_player(player_id)
        .is_some_and(|player| !player.is_dead());
    let queued_script_id = player_alive
        .then(|| game.queue_player_script(player_id, &script))
        .flatten();
    trace!(player_id, player_found, player_alive, ?queued_script_id, "Обработан запуск сценария игрока");
    Ok(())
}

fn dispatch_honor_rank_message(
    opcode: u32,
    message: &mut CMessage,
    game: &CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let requested_country = if opcode == 0x9012d {
        Some(
            message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "honor rank country",
                })?,
        )
    } else {
        None
    };
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        trace!(opcode, ?requested_country, "Honor-list команда не имеет игрока");
        return Ok(());
    };
    let sent = match opcode {
        0x9012a..=0x9012c => {
            game.send_player_honor_ranks(player_id, (opcode - 0x9012a) as i32)
        }
        0x9012d => game.send_total_honor_ranks_for_country(
            player_id,
            requested_country.expect("country прочитан для 0x9012D"),
        ),
        _ => unreachable!("honor-list opcode проверен dispatcher-ом"),
    };
    trace!(opcode, player_id, ?requested_country, sent, "Опубликован рейтинг чести");
    Ok(())
}

fn dispatch_client_quest_action(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        trace!(opcode, "Quest-команда не имеет игрока");
        return Ok(());
    };
    match opcode {
        0x90124 => {
            let first = message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "quest action first value",
                })?;
            let second = message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "quest action second value",
                })?;
            let mut request = CMessage::new(0x0006_0130);
            request.add_long(player_id);
            request.add_long(first);
            request.add_long(second);
            let delivery = request.send(game, false);
            trace!(opcode, player_id, first, second, ?delivery, "Quest-команда передана WorldServer");
        }
        0x90125 | 0x90126 => {
            message.add_long(player_id);
            message.set_message_type(if opcode == 0x90125 {
                0x0006_0131
            } else {
                0x0006_0132
            });
            let delivery = message.send(game, false);
            trace!(opcode, player_id, ?delivery, "Quest payload передан WorldServer");
        }
        0x90127 | 0x90128 => {
            let quest_id = message
                .base_mut()
                .get_short()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "quest terminal ID",
                })? as u16;
            let queued_script_id = if opcode == 0x90127 {
                game.queue_player_quest_complete_script(player_id, quest_id)
            } else {
                game.queue_player_quest_abandon_script(player_id, quest_id)
            };
            trace!(opcode, player_id, quest_id, ?queued_script_id, "Обработан терминальный сценарий задания");
        }
        _ => unreachable!("quest action opcode проверен dispatcher-ом"),
    }
    Ok(())
}

fn dispatch_organizing_client_relay(
    opcode: u32,
    message: &mut CMessage,
    game: &CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        trace!(opcode, "OrganSys relay не имеет игрока");
        return Ok(());
    };
    let output_opcode = match opcode {
        0x90102 => 0x0006_0104,
        0x90107..=0x90116 => 0x0006_010a + (opcode - 0x90107),
        0x90117..=0x90119 => 0x0006_011b + (opcode - 0x90117),
        0x9011c => 0x0006_0120,
        0x9011d => 0x0006_0121,
        0x9011e => 0x0006_0122,
        0x9011f => 0x0006_0123,
        0x90120 => 0x0006_0124,
        0x90121 => 0x0006_0128,
        _ => unreachable!("relay opcode проверен dispatcher-ом"),
    };
    let delivery = match opcode {
        0x90107 => {
            let first = message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "relay first value",
                })?;
            let second = message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "relay second value",
                })?;
            let mut request = CMessage::new(output_opcode as i32);
            request.add_long(player_id);
            request.add_long(second);
            request.add_long(first);
            request.send(game, false)
        }
        0x90108 | 0x90109 | 0x9010c | 0x9010d | 0x90113 | 0x90115 => {
            let value = message
                .base_mut()
                .get_long()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "relay value",
                })?;
            let mut request = CMessage::new(output_opcode as i32);
            request.add_long(player_id);
            request.add_long(value);
            request.send(game, false)
        }
        0x90102 | 0x9010a | 0x9010b | 0x9010e | 0x9010f => {
            let mut request = CMessage::new(output_opcode as i32);
            request.add_long(player_id);
            request.send(game, false)
        }
        0x90110..=0x90112
        | 0x90114
        | 0x90116..=0x90119
        | 0x9011c..=0x90121 => {
            message.add_long(player_id);
            message.set_message_type(output_opcode as i32);
            message.send(game, false)
        }
        _ => unreachable!("relay wire-form проверен dispatcher-ом"),
    };
    trace!(opcode, output_opcode, player_id, ?delivery, "OrganSys команда передана WorldServer");
    Ok(())
}

fn dispatch_organizing_client_response(
    opcode: u32,
    message: &mut CMessage,
    game: &CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let player_id = if opcode == 0x7fe09 {
        message.resolve_player_context(game);
        let Some(player_id) = message.player_id() else {
            trace!(opcode, "OrganSys context-ответ не имеет игрока");
            return Ok(());
        };
        player_id
    } else {
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                field: "client response player ID",
            })?
    };
    let output_opcode = match opcode {
        0x7fe02..=0x7fe05 | 0x7fe08..=0x7fe15 | 0x7fe1a..=0x7fe1c => {
            0x000b_ff00 + (opcode & 0xff)
        }
        0x7fe16..=0x7fe17 => 0x000b_ff17 + (opcode - 0x7fe16),
        0x7fe2b => 0x000b_ff27,
        _ => unreachable!("client response opcode проверен dispatcher-ом"),
    };
    message.set_message_type(output_opcode as i32);
    let delivery = message.send_to_player(game.net_server(), player_id);
    trace!(opcode, output_opcode, player_id, delivery, "OrganSys ответ отправлен игроку");
    Ok(())
}

/// Подключает всю достигнутую OrganSys family к живому `CGame` owner-у.
pub(crate) fn dispatch_game_organizing_message<Runtime: GameOrganizingWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameOrganizingMessageError>> {
    let opcode = message.message_type() as u32;
    if !matches!(
        opcode,
        0x90101
            | 0x90102
            | 0x90105
            | 0x90106
            | 0x90107..=0x90119
            | 0x9011a
            | 0x9011b
            | 0x9011c..=0x90121
            | 0x90122
            | 0x90123
            | 0x90124..=0x90128
            | 0x9012a..=0x9012d
            | 0x7fe01
            | 0x7fe02..=0x7fe05
            | 0x7fe06
            | 0x7fe07
            | 0x7fe08..=0x7fe17
            | 0x7fe18
            | 0x7fe19
            | 0x7fe1a..=0x7fe1c
            | 0x7fe1d
            | 0x7fe1e
            | 0x7fe2a
            | 0x7fe26
            | 0x7fe27
            | 0x7fe2b..=0x7fe2d
            | 0x7fe28
            | 0x7fe29
            | 0x7fe2e
            | 0x7fe1f..=0x7fe25
            | 0x7fe2f..=0x7fe33
            | 0x7fe34
            | 0x7fe35
            | 0x7fe36
            | 0x7fe37
            | 0x7fe38
            | 0x7fe39
            | 0x7fe3a
            | 0x7fe3c..=0x7fe3f
            | 0x7fe41
            | 0x7fe43..=0x7fe45
            | 0x7fe46..=0x7fe4a
    ) {
        return None;
    }

    if matches!(
        opcode,
        0x7fe02..=0x7fe05
            | 0x7fe08..=0x7fe17
            | 0x7fe1a..=0x7fe1c
            | 0x7fe2b
    ) {
        return Some(
            dispatch_organizing_client_response(opcode, message, game)
                .map_err(GameOrganizingMessageError::ClientResponse),
        );
    }

    if matches!(
        opcode,
        0x90102 | 0x90107..=0x90119 | 0x9011c..=0x90121
    ) {
        return Some(
            dispatch_organizing_client_relay(opcode, message, game)
                .map_err(GameOrganizingMessageError::ClientRelay),
        );
    }

    if matches!(opcode, 0x9012a..=0x9012d) {
        return Some(
            dispatch_honor_rank_message(opcode, message, game)
                .map_err(GameOrganizingMessageError::HonorRanks),
        );
    }

    if matches!(opcode, 0x90124..=0x90128) {
        return Some(
            dispatch_client_quest_action(opcode, message, game)
                .map_err(GameOrganizingMessageError::QuestActions),
        );
    }

    if matches!(opcode, 0x7fe38 | 0x7fe39) {
        return Some(
            dispatch_game_player_quest_command(opcode, message, game)
                .map_err(GameOrganizingMessageError::PlayerQuest),
        );
    }

    if opcode == 0x7fe3a {
        return Some(
            dispatch_game_player_run_script(message, game)
                .map_err(GameOrganizingMessageError::PlayerRunScript),
        );
    }

    if matches!(opcode, 0x90122 | 0x90123 | 0x7fe28 | 0x7fe29 | 0x7fe2e) {
        return Some(
            dispatch_region_tax_message(opcode, message, game)
                .map_err(GameOrganizingMessageError::RegionTax),
        );
    }

    if matches!(opcode, 0x7fe26 | 0x7fe27 | 0x7fe2c | 0x7fe2d) {
        return Some(
            dispatch_region_control_message(opcode, message, game)
                .map_err(GameOrganizingMessageError::RegionControl),
        );
    }

    if opcode == 0x7fe2a {
        return Some(
            dispatch_city_gate_response(message, game)
                .map_err(GameOrganizingMessageError::CityGate),
        );
    }

    if opcode == 0x7fe34 {
        return Some(
            dispatch_village_war_application_response(message, game)
                .map_err(GameOrganizingMessageError::VillageApplication),
        );
    }

    if opcode == 0x7fe37 {
        return Some(
            dispatch_war_application_response(message, game)
                .map_err(GameOrganizingMessageError::CityApplication),
        );
    }

    if opcode == 0x7fe1d {
        return Some(
            dispatch_faction_billboard_response(message, game)
                .map_err(GameOrganizingMessageError::FactionBillboard),
        );
    }

    if matches!(
        opcode,
        0x90101
            | 0x90105
            | 0x90106
            | 0x9011a
            | 0x9011b
            | 0x7fe01
            | 0x7fe06
            | 0x7fe07
            | 0x7fe18
            | 0x7fe19
            | 0x7fe1e
    ) {
        return Some(
            dispatch_faction_lifecycle_message(opcode, message, game, runtime)
                .map_err(GameOrganizingMessageError::FactionLifecycle),
        );
    }

    if matches!(opcode, 0x7fe46..=0x7fe4a) {
        return Some(
            dispatch_organizing_control_message(opcode, message, game)
                .map_err(GameOrganizingMessageError::Control),
        );
    }

    if matches!(
        opcode,
        0x7fe3c..=0x7fe3f | 0x7fe41 | 0x7fe43..=0x7fe45
    ) {
        return Some(
            dispatch_four_nation_phase_message(opcode, message, game)
                .map_err(GameOrganizingMessageError::Phase),
        );
    }

    let mut owners = game.take_war_startup_owners();
    let result = {
        let mut context = GameOrganizingWarContext { game };
        let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        if matches!(opcode, 0x7fe35 | 0x7fe36) {
            dispatch_war_faction_update(
                opcode,
                payload,
                cursor,
                &mut owners.attack_city,
                &mut owners.village,
                &mut context,
            )
            .expect("faction opcode проверен перед dispatcher-ом")
            .map_err(GameOrganizingMessageError::FactionUpdate)
        } else {
            dispatch_war_phase(
                opcode,
                payload,
                cursor,
                &mut owners.attack_city,
                &mut owners.village,
                &mut context,
            )
            .expect("phase opcode проверен перед dispatcher-ом")
            .map_err(GameOrganizingMessageError::Phase)
        }
    };
    game.restore_war_startup_owners(owners);
    Some(result)
}

fn dispatch_region_control_message(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let read_i32 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    match opcode {
        0x7fe26 => {
            game.collect_all_region_today_tax();
            trace!(opcode, "Собран суточный налог локальных регионов");
        }
        0x7fe27 => {
            let region_id = read_i32(message, "region ID")?;
            let faction_id = read_i32(message, "owned faction ID")?;
            let union_id = read_i32(message, "owned union ID")?;
            let country = message
                .base_mut()
                .get_byte()
                .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                    field: "region country",
                })?;
            let applied = if let Some(region) = game.find_region_mut(region_id) {
                region.base_mut().set_owned_city_org(faction_id, union_id);
                region.base_mut().country = country;
                true
            } else if let Some(region) = game.find_proxy_region_mut(region_id) {
                region.set_owned_city_org(faction_id, union_id);
                region.set_country(country);
                true
            } else {
                false
            };
            trace!(opcode, region_id, faction_id, union_id, country, applied, "Обновлён владелец региона");
        }
        0x7fe2c => {
            let region_id = read_i32(message, "region ID")?;
            let state = read_i32(message, "city state")?;
            let applied = game.find_region_mut(region_id).is_some_and(|region| {
                region.base_mut().set_city_state(state);
                true
            });
            trace!(opcode, region_id, state, applied, "Обновлено состояние города");
        }
        0x7fe2d => {
            let region_id = read_i32(message, "region ID")?;
            let amount = read_i32(message, "tax amount")? as u32;
            let applied = game.find_region(region_id).is_some();
            if applied {
                game.add_region_tax(region_id, amount);
            }
            trace!(opcode, region_id, amount, applied, "Налог добавлен владельцу региона");
        }
        _ => unreachable!("region-control opcode проверен dispatcher-ом"),
    }
    Ok(())
}

fn dispatch_faction_billboard_response(
    message: &mut CMessage,
    game: &CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    message.set_message_type(0x000b_ff1d);
    let delivery = message.send_to_player(game.net_server(), player_id);
    trace!(player_id, delivery, "Опубликован рейтинг фракций");
    Ok(())
}

fn dispatch_region_tax_message(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let read_i32 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    let read_u32 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .map(|value| value as u32)
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    let read_i64 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long64()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    match opcode {
        0x90122 | 0x90123 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let value = read_i32(message, "tax value")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let callback = game
                .submit_region_tax_session_result(player_id, session_id, password, value);
            trace!(opcode, player_id, session_id, ?callback, applied = callback == NetSessionCallbackOutcome::Delivered, "Обработан ответ налогового сеанса");
            Ok(())
        }
        0x7fe28 | 0x7fe29 => {
            let player_id = read_i32(message, "player ID")?;
            let region_id = read_i32(message, "region ID")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let kind = if opcode == 0x7fe28 {
                RegionTaxSessionKind::ObtainPayment
            } else {
                RegionTaxSessionKind::AdjustRate
            };
            let session_id = game.with_legacy_random_stream(|game, random| {
                game.start_region_tax_session(player_id, region_id, kind, |bound| {
                    random.random_below(bound)
                })
            });
            trace!(opcode, player_id, region_id, ?session_id, applied = session_id.is_some(), "Запущен налоговый сеанс");
            Ok(())
        }
        0x7fe2e => {
            let region_id = read_i32(message, "region ID")?;
            let today_total_tax = read_u32(message, "today tax")?;
            let total_tax = read_u32(message, "total tax")?;
            let current_tax_rate = read_i32(message, "current tax rate")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let applied = game.apply_proxy_region_tax_snapshot(
                region_id,
                today_total_tax,
                total_tax,
                current_tax_rate,
            );
            trace!(opcode, region_id, applied, "Применён налоговый снимок региона");
            Ok(())
        }
        _ => unreachable!("налоговый opcode проверен перед разбором"),
    }
}

fn dispatch_village_war_application_response(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    dispatch_war_application_response(message, game)
}

fn dispatch_war_application_response(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    let fee =
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                field: "village application fee",
            })?;
    if !message.base_mut().unread_bytes().is_empty() {
        return Err(FactionLifecycleDispatchError::InvalidPayload);
    }
    let money = game.apply_war_application_money(player_id, fee);
    trace!(player_id, fee, player_found = money.is_some(), ?money, "Обработан возврат платы за заявку войны");
    Ok(())
}

fn dispatch_city_gate_response(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    let region_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "region ID" })?;
    let gate_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "gate ID" })?;
    let operation = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "operation" })?;
    if !message.base_mut().unread_bytes().is_empty() {
        return Err(FactionLifecycleDispatchError::InvalidPayload);
    }
    let operated = game.operate_script_city_gate(region_id, gate_id, operation);
    let state = game.script_city_gate_state(region_id, gate_id);
    let notice_id = match (operation, state) {
        (0, 0) => Some(b"GS0042".as_slice()),
        (1, 1) => Some(b"GS0043".as_slice()),
        _ => None,
    };
    let notice_delivery = notice_id.map(|notice_id| {
        colored_player_notice_message(0xffff_ffff, 0xffff_0000, game.get_string_by_id(notice_id))
            .send_to_player(game.net_server(), player_id)
    });
    trace!(player_id, region_id, gate_id, operation, operated, ?notice_delivery, "Обработано управление городскими воротами");
    Ok(())
}

fn dispatch_faction_lifecycle_message<Runtime: GameClockContext>(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<(), FactionLifecycleDispatchError> {
    let read_i64 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long64()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    let read_i32 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    match opcode {
        0x90101 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let accepted = read_i32(message, "accepted")?;
            let faction_name = (accepted != 0).then(|| message.base_mut().get_str_bytes(20));
            let faction_name = match faction_name {
                Some(Some(name)) if !name.is_empty() => Some(name),
                Some(_) => return Err(FactionLifecycleDispatchError::InvalidPayload),
                None => None,
            };
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.submit_script_faction_creation(
                player_id,
                session_id,
                password,
                faction_name.as_deref(),
                runtime,
            );
            trace!(opcode, player_id, correlated, "Обработано создание фракции");
            Ok(())
        }
        0x7fe01 => {
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let player_id = read_i32(message, "player ID")?;
            let result = read_i32(message, "result")?;
            if !matches!(result, 0 | 1) || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated =
                game.finish_script_faction_creation(session_id, password, player_id, result);
            trace!(opcode, player_id, correlated, "Получен итог создания фракции");
            Ok(())
        }
        0x7fe06 => {
            let player_id = read_i32(message, "player ID")?;
            let faction_id = read_i32(message, "faction ID")?;
            let mut faction_logo_id = 0;
            let mut faction_level = 0;
            let mut faction_experience = 0;
            let mut faction_force = 0;
            let mut faction_contribute = 0;
            let mut faction_master_id = 0;
            let mut faction_name = Vec::new();
            let mut faction_title = Vec::new();
            let mut union_id = 0;
            let mut union_master_id = 0;
            let mut enemy_factions = std::collections::BTreeSet::new();
            let mut city_war_enemy_factions = std::collections::BTreeSet::new();
            let mut faction_owned_regions = Vec::new();
            if faction_id > 0 {
                faction_logo_id = read_i32(message, "faction logo ID")?;
                faction_level = message.base_mut().get_word().ok_or(
                    FactionLifecycleDispatchError::UnexpectedEnd {
                        field: "faction level",
                    },
                )?;
                faction_experience = read_i32(message, "faction experience")?;
                faction_force = read_i32(message, "faction force")?;
                faction_contribute = read_i32(message, "faction contribute")? as u32;
                faction_name = message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
                faction_title = message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
                faction_master_id = read_i32(message, "faction master ID")?;
                union_id = read_i32(message, "union ID")?;
                union_master_id = read_i32(message, "union master ID")?;
                for (field, destination) in [
                    ("enemy factions", &mut enemy_factions),
                    ("city-war enemy factions", &mut city_war_enemy_factions),
                ] {
                    let count = read_i32(message, field)?;
                    if count < 0 {
                        return Err(FactionLifecycleDispatchError::InvalidPayload);
                    }
                    for _ in 0..count {
                        destination.insert(read_i32(message, field)?);
                    }
                }
                let owned_count = read_i32(message, "owned regions")?;
                if owned_count < 0 {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                for _ in 0..owned_count {
                    let region_id = read_i32(message, "owned region ID")?;
                    let war_type = message.base_mut().get_word().ok_or(
                        FactionLifecycleDispatchError::UnexpectedEnd {
                            field: "owned region war type",
                        },
                    )?;
                    let reserved = message.base_mut().get_word().ok_or(
                        FactionLifecycleDispatchError::UnexpectedEnd {
                            field: "owned region reserved",
                        },
                    )?;
                    let mut wire = [0; 8];
                    wire[..4].copy_from_slice(&region_id.to_le_bytes());
                    wire[4..6].copy_from_slice(&war_type.to_le_bytes());
                    wire[6..].copy_from_slice(&reserved.to_le_bytes());
                    faction_owned_regions.push(wire);
                }
            }
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = if let Some(player) = game.find_player_mut(player_id) {
                player.restore_faction_identity(
                    faction_id,
                    faction_logo_id,
                    faction_level,
                    faction_experience,
                    faction_force,
                    faction_contribute,
                    faction_master_id,
                    &faction_name,
                    &faction_title,
                    union_id,
                    union_master_id,
                    enemy_factions,
                    city_war_enemy_factions,
                    faction_owned_regions,
                );
                true
            } else {
                false
            };
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff06);
                let _ = game.send_player_shape_around(player_id, None, message);
                message.send_to_player(game.net_server(), player_id)
            });
            trace!(opcode, player_id, correlated, ?delivery, "Обновлена принадлежность игрока к фракции");
            Ok(())
        }
        0x90105 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            // В точной ветви `0x90105` пароль пропускается: `OnDo` связывает
            // продолжение только по ID управляемого сеанса и текущему игроку.
            let _password = read_i32(message, "password")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.continue_script_faction_application(player_id, session_id);
            trace!(opcode, player_id, correlated, "Продолжен список заявок фракции");
            Ok(())
        }
        0x90106 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let discarded = read_i32(message, "discarded faction ID")?;
            let accepted = read_i32(message, "accepted")?;
            let faction_name = message
                .base_mut()
                .get_str_bytes(20)
                .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.select_script_faction_application(
                player_id,
                session_id,
                password,
                discarded,
                accepted,
                &faction_name,
            );
            trace!(opcode, player_id, correlated, "Выбрана заявка фракции");
            Ok(())
        }
        0x7fe07 => {
            let player_id = read_i32(message, "player ID")?;
            let total = read_i32(message, "total factions")?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if total < 0 {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            if total == 0 {
                if !message.base_mut().unread_bytes().is_empty() {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                let correlated =
                    game.finish_empty_script_faction_application(player_id, session_id, password);
                trace!(opcode, player_id, correlated, "Завершён пустой список заявок фракции");
                return Ok(());
            }
            let correlated =
                game.script_faction_application_is_active(player_id, session_id, password);
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff07);
                message.send_to_player(game.net_server(), player_id)
            });
            trace!(opcode, player_id, correlated, ?delivery, "Опубликован список заявок фракции");
            Ok(())
        }
        0x7fe1e => {
            let player_id = read_i32(message, "player ID")?;
            let money = read_i32(message, "money")?;
            let goods_name = message
                .base_mut()
                .get_str_bytes(100)
                .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
            if money < 0 || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated =
                game.apply_script_faction_upgrade_debit(player_id, money as u32, &goods_name);
            trace!(opcode, player_id, correlated, "Списаны ресурсы развития фракции");
            Ok(())
        }
        0x9011a => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.continue_script_faction_war_page(player_id, session_id, password);
            trace!(opcode, player_id, correlated, "Продолжена страница войны фракций");
            Ok(())
        }
        0x9011b => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let target_faction_id = read_i32(message, "target faction ID")?;
            let war_type = if target_faction_id > 0 {
                read_i32(message, "war type")?
            } else {
                0
            };
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = if target_faction_id > 0 {
                game.select_script_faction_war_target(
                    player_id,
                    session_id,
                    password,
                    target_faction_id,
                    war_type,
                    runtime,
                )
            } else {
                game.close_script_faction_war_declaration(player_id, session_id, password)
            };
            trace!(opcode, player_id, correlated, target_faction_id, war_type, "Обработан выбор цели войны фракций");
            Ok(())
        }
        0x7fe18 => {
            let player_id = read_i32(message, "player ID")?;
            let total = read_i32(message, "total factions")?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if total < 0 {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            if total == 0 {
                if !message.base_mut().unread_bytes().is_empty() {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                let correlated =
                    game.close_script_faction_war_declaration(player_id, session_id, password);
                trace!(opcode, player_id, correlated, "Закрыт пустой список войны фракций");
                return Ok(());
            }
            let correlated = game.script_faction_war_is_active(player_id, session_id, password);
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff19);
                message.send_to_player(game.net_server(), player_id)
            });
            trace!(opcode, player_id, correlated, ?delivery, "Опубликован список войны фракций");
            Ok(())
        }
        0x7fe19 => {
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let player_id = read_i32(message, "player ID")?;
            let money = read_i32(message, "money")?;
            if money < 0 || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let delivery = game.finish_script_faction_war_result(
                player_id,
                session_id,
                password,
                money as u32,
            );
            trace!(opcode, player_id, correlated = delivery.is_some(), ?delivery, "Завершён результат войны фракций");
            Ok(())
        }
        _ => unreachable!("faction lifecycle opcode проверен caller-ом"),
    }
}

fn dispatch_four_nation_phase_message(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), WarPhaseDispatchError> {
    let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let war_number = read_phase_war_number(payload, cursor)?;
    let schedule_exists = game
        .four_nation_war_sys()
        .setups()
        .get(war_number as usize)
        .is_some();
    let sign_up_counts = if opcode == 0x7fe3e && schedule_exists {
        let mut values = [0; 5];
        for value in &mut values {
            *value = read_phase_war_number(payload, cursor)?;
        }
        Some(values)
    } else {
        None
    };
    let mut owners = game.take_war_startup_owners();
    let (schedule_found, results) = {
        let mut context = GameOrganizingWarContext { game };
        match opcode {
            0x7fe3c => (
                owners.four_nation.on_war_start(war_number, &mut context),
                None,
            ),
            0x7fe3d => (
                owners
                    .four_nation
                    .on_sign_up_war_start(war_number, &mut context),
                None,
            ),
            0x7fe3e => (
                if let Some(sign_up_counts) = sign_up_counts {
                    owners
                        .four_nation
                        .on_sign_up_war_end(war_number, sign_up_counts, &mut context)
                } else {
                    false
                },
                None,
            ),
            0x7fe3f => (
                owners.four_nation.on_enter_start(war_number, &mut context),
                None,
            ),
            0x7fe41 => (
                owners
                    .four_nation
                    .on_refresh_region(war_number, &mut context),
                None,
            ),
            0x7fe43 => (
                owners.four_nation.on_war_end(war_number, &mut context),
                None,
            ),
            0x7fe44 => (
                owners.four_nation.on_clear_war(war_number, &mut context),
                None,
            ),
            0x7fe45 => {
                let results = owners
                    .four_nation
                    .take_war_results(war_number, &mut context);
                (
                    owners
                        .four_nation
                        .setups()
                        .get(war_number as usize)
                        .is_some(),
                    results,
                )
            }
            _ => unreachable!("FourNation phase opcode проверен outer dispatcher-ом"),
        }
    };
    game.restore_war_startup_owners(owners);

    let delivery = results.map(|values| {
        let mut response = CMessage::new(0x60319);
        for value in values {
            response.base_mut().add_ulong(value);
        }
        response.send(game, false)
    });
    trace!(opcode, war_number, schedule_found, has_results = results.is_some(), ?delivery, "Обработана фаза FourNation");
    Ok(())
}

fn dispatch_organizing_control_message(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), OrganizingControlDispatchError> {
    match opcode {
        0x7fe46 => {
            let player_id = read_control_i32(message, "exploit player ID")?;
            let increment = read_control_i32(message, "exploit increment")? as u32;
            let Some(previous_exploit) = game
                .find_player(player_id)
                .map(|player| player.base_properties().exploit)
            else {
                trace!(player_id, increment, "Игрок для изменения подвига FourNation не найден");
                return Ok(());
            };
            let maximum = game
                .country_param()
                .max_exploit()
                .ok_or(OrganizingControlDispatchError::CountryExploitLimitMissing { player_id })?;
            let advertised_exploit = previous_exploit.wrapping_add(increment);
            let mut property = CMessage::new(0xbf80c);
            property.base_mut().add_long(player_id);
            property.base_mut().add_long(player_id);
            property.base_mut().add_str(Some(c"dwExploit"));
            property.base_mut().add_ulong(advertised_exploit);
            let exploit_property_delivery = property.send_to_player(game.net_server(), player_id);
            let applied_exploit = {
                let player = game
                    .find_player_mut(player_id)
                    .expect("player проверен до exact exploit mutation");
                player.set_exploit(advertised_exploit, maximum)
            };
            let (combat_property_delivery, tao_zhuang_ran) = game
                .update_player_properties(player_id)
                .expect("player сохранён после exact exploit mutation");

            let notice_text = Some(format_four_nation_exploit_notice(
                game.get_string_by_id(b"GS1177"),
                increment as i32,
            ));
            let notice_delivery = notice_text.as_ref().map(|text| {
                let text = CString::new(text.as_slice())
                    .expect("legacy string prefix и decimal не содержат NUL");
                let mut notice = CMessage::new(0xbf806);
                notice.base_mut().add_ulong(u32::MAX);
                notice.base_mut().add_ulong(0xffff_0000);
                notice.base_mut().add_str(Some(&text));
                notice.send_to_player(game.net_server(), player_id)
            });
            trace!(player_id, increment, advertised_exploit, applied_exploit, exploit_property_delivery, combat_property_delivery, tao_zhuang_ran, ?notice_delivery, "Обновлён подвиг FourNation");
            Ok(())
        }
        0x7fe47 => {
            let player_id = read_control_i32(message, "war-time player ID")?;
            let time_ms = read_control_i32(message, "player war time")? as u32;
            let previous_time_ms = game
                .four_nation_war_sys_mut()
                .set_one_player_war_time(player_id, time_ms);
            trace!(player_id, time_ms, ?previous_time_ms, "Обновлено время игрока в FourNation");
            Ok(())
        }
        0x7fe48 => {
            let country_id = read_control_i32(message, "country ID")? as u8;
            let requested = read_control_i32(message, "country treasury")?;
            if game.country_handler().country(country_id).is_none() {
                trace!(country_id, requested, "Страна для обновления казны не найдена");
                return Ok(());
            }
            let maximum = game.country_param().max_country_treasury().ok_or(
                OrganizingControlDispatchError::CountryTreasuryLimitMissing { country_id },
            )?;
            let applied = if requested < 0 {
                0
            } else {
                requested.min(maximum)
            };
            let update = game
                .country_handler_mut()
                .country_mut(country_id)
                .expect("country проверен до exact treasury mutation")
                .set_country_treasury(applied);
            let delivery = update.send(game, false);
            trace!(country_id, requested, applied, ?delivery, "Обновлена казна страны");
            Ok(())
        }
        0x7fe49 => {
            let morale = read_control_i32(message, "FourNation morale")?;
            game.four_nation_war_sys_mut().set_morale(morale);
            trace!(morale, "Обновлена мораль FourNation");
            Ok(())
        }
        0x7fe4a => {
            let player_id = read_control_i32(message, "router player ID")?;
            let player_found = game.find_player(player_id).is_some();
            let delivery = player_found.then(|| {
                message.set_message_type(0xbff36);
                message.base_mut().update();
                message.send_to_player(game.net_server(), player_id)
            });
            trace!(player_id, player_found, ?delivery, "Обработан маршрут региона FourNation");
            Ok(())
        }
        _ => unreachable!("control opcode отфильтрован перед dispatcher-ом"),
    }
}

fn format_four_nation_exploit_notice(template: &[u8], increment: i32) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let Some(marker) = template.windows(2).position(|window| window == b"%d") else {
        return template[..template.len().min(0xff)].to_vec();
    };
    let value = increment.to_string();
    let mut result = Vec::with_capacity(template.len().saturating_add(value.len()));
    result.extend_from_slice(&template[..marker]);
    result.extend_from_slice(value.as_bytes());
    result.extend_from_slice(&template[marker + 2..]);
    result.truncate(0xff);
    result
}

fn format_legacy_integer_fields(
    template: &[u8],
    values: &[String],
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut result = Vec::with_capacity(template.len());
    let mut offset = 0;
    let mut value_index = 0;
    while offset < template.len() {
        if template[offset] != b'%' || offset + 1 >= template.len() {
            result.push(template[offset]);
            offset += 1;
            continue;
        }
        if template[offset + 1] == b'%' {
            result.push(b'%');
            offset += 2;
            continue;
        }
        let conversion_length = if template[offset + 1] == b'l'
            && template
                .get(offset + 2)
                .is_some_and(|byte| matches!(byte, b'd' | b'i' | b'u'))
        {
            3
        } else if matches!(template[offset + 1], b'd' | b'i' | b'u') {
            2
        } else {
            result.push(template[offset]);
            offset += 1;
            continue;
        };
        let Some(value) = values.get(value_index) else {
            result.extend_from_slice(&template[offset..offset + conversion_length]);
            offset += conversion_length;
            continue;
        };
        result.extend_from_slice(value.as_bytes());
        value_index += 1;
        offset += conversion_length;
    }
    result.truncate(maximum_bytes);
    result
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..length]
}

fn read_control_i32(
    message: &mut CMessage,
    field: &'static str,
) -> Result<i32, OrganizingControlDispatchError> {
    let base = message.base_mut();
    let offset = base.cursor();
    let available = base.as_wire_bytes().len().saturating_sub(offset);
    base.get_long()
        .ok_or(OrganizingControlDispatchError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        })
}

fn read_phase_war_number(payload: &[u8], cursor: &mut usize) -> Result<i32, WarPhaseDispatchError> {
    let offset = *cursor;
    let available = payload.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(payload, offset).map_err(|_| WarPhaseDispatchError::UnexpectedEnd { offset, needed: 4, available })?;
    let value = reader.read_i32().map_err(|_| WarPhaseDispatchError::UnexpectedEnd { offset, needed: 4, available })?;
    *cursor = reader.position();
    Ok(value)
}

struct GameOrganizingWarContext<'a> {
    game: &'a mut CGame,
}

struct CityWarEndContext<'a> {
    game: &'a mut CGame,
}

impl WarRegionClearContext for CityWarEndContext<'_> {
    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        WarRegionClearContext::send_contend_time(self.game, player_id, time);
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        WarRegionClearContext::set_region_player_contend_state(
            self.game, region_id, player_id, state,
        );
    }
}

enum ContendSchedule<'a> {
    AttackCity(&'a CAttackCitySys),
    Village(&'a CVillageWarSys),
}

struct ContendProjectionContext<'a> {
    game: &'a mut CGame,
    schedule: ContendSchedule<'a>,
    war_number: i32,
}

impl WarRegionContext for ContendProjectionContext<'_> {
    type MembershipError = std::convert::Infallible;

    fn player_faction_id(&mut self, player_id: i32) -> Option<i32> {
        self.game.find_player(player_id).map(|player| player.faction_id())
    }

    fn is_apply_war_faction(&mut self, faction_id: i32) -> Result<bool, Self::MembershipError> {
        match self.schedule {
            ContendSchedule::AttackCity(schedule) => Ok(
                schedule.is_already_declar_for_war(self.war_number, faction_id),
            ),
            ContendSchedule::Village(schedule) => {
                Ok(schedule.is_already_declar_for_war(self.war_number, faction_id))
            }
        }
    }

    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        let mut message = CMessage::new(0x000b_ff29);
        message.base_mut().add_long(time);
        let delivery = message.send_to_player(self.game.net_server(), player_id);
        tracing::trace!(player_id, time, delivery, "отправлено время war contender-а");
    }

    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool) {
        let region = self
            .game
            .find_player(player_id)
            .and_then(|player| player.server_region_id())
            .and_then(|region_id| self.game.find_region(region_id))
            .map(|owner| owner.base().recipients_snapshot());
        if let Some(region) = region {
            let _ = self
                .game
                .publish_war_player_contend_state_snapshot(&region, player_id, state);
        }
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        if self
            .game
            .find_player(player_id)
            .is_some_and(|player| player.server_region_id() == Some(region_id))
        {
            self.set_global_player_contend_state(player_id, state);
        }
    }
}

impl GameOrganizingWarContext<'_> {
    fn write_village_war_log(&self, string_id: &str, region_name: &str) {
        let text = format_legacy_text_fields(
            self.game.get_string_by_id(string_id.as_bytes()),
            &[region_name.as_bytes()],
            0xff,
        );
        put_string_to_file("war", &text);
    }

    fn write_city_war_log(&self, string_id: &str, war_number: i32, region_name: &str) {
        let text = format_legacy_mixed(
            self.game.get_string_by_id(string_id.as_bytes()),
            &[
                LegacyFormatArgument::Signed(war_number),
                LegacyFormatArgument::Bytes(region_name.as_bytes()),
            ],
            0xff,
        );
        put_string_to_file("war", &text);
    }

    fn lookup_region_then_proxy(&self, region_id: i32) -> Option<GameWarRegionHandle> {
        if self.game.find_region(region_id).is_some() {
            Some(GameWarRegionHandle::Local(region_id))
        } else {
            self.game
                .find_proxy_region(region_id)
                .map(|_| GameWarRegionHandle::Proxy(region_id))
        }
    }

    fn lookup_server_region(&self, region_id: i32) -> Option<GameWarRegionHandle> {
        self.game
            .find_region(region_id)
            .map(|_| GameWarRegionHandle::Local(region_id))
    }

    fn update_contenders(&mut self, region_id: i32, schedule: ContendSchedule<'_>) {
        let Some(mut owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let war = match &mut owner {
            ServerRegionOwner::Village(region) => &mut region.war,
            ServerRegionOwner::City(region) => &mut region.war,
            ServerRegionOwner::Nation(region) => &mut region.war,
            ServerRegionOwner::GodsBattle(region) => &mut region.war,
            ServerRegionOwner::Base(_) | ServerRegionOwner::Country(_) => {
                self.game.restore_region_owner(owner);
                return;
            }
        };
        let mut context = ContendProjectionContext {
            game: self.game,
            schedule,
            war_number: war.base.get_war_number(),
        };
        let _ = war.update_contend_player(&mut context);
        self.game.restore_region_owner(owner);
    }

    /// Exact `ServerNationRegion::OnRefreshRegion` tail: после state reset
    /// четыре magic-stone NPC проверяются и при отсутствии создаются в
    /// исходном порядке с `remember=true, sendAround=true`.
    fn refresh_four_nation_magic_stones(&mut self, region_id: i32) {
        let setups = [
            (b"GS1084".as_slice(), 0x20c, 0xfb, 0x35, 4),
            (b"GS1085".as_slice(), 0x20d, 0xf8, 0x1c1, 0),
            (b"GS1086".as_slice(), 0x20e, 0x25, 0xfc, 2),
            (b"GS1087".as_slice(), 0x20f, 0x1dc, 0x105, 6),
        ]
        .map(|(name_id, picture_id, x, y, direction)| ServerRegionNpcSetup {
            show_list: true,
            picture_id,
            left: x,
            top: y,
            right: x,
            bottom: y,
            count: 1,
            direction,
            time: 3_600_000,
            name: self.game.get_string_by_id(name_id).to_vec(),
            script: Vec::new(),
        });
        self.game
            .refresh_nation_magic_stone_npcs(region_id, setups);
    }

    fn kick_out_four_nation_players(
        &mut self,
        region: &mut super::super::servernationregion::ServerNationRegion,
    ) {
        let player_ids = region.war.base.registered_player_ids();
        let (area_width, area_height) = self.game.area_dimensions();
        for player_id in player_ids {
            let Some((country, eligible)) = self
                .game
                .find_player(player_id)
                .map(|player| (player.country(), player.can_start_nation_war_timing()))
            else {
                continue;
            };
            if !eligible {
                continue;
            }
            let Some(rect) = region.relive_rects().get(usize::from(country)).copied() else {
                report_four_nation_relive_block(
                    player_id,
                    FourNationReliveBlock::CountryOutsideRectangles { country },
                );
                continue;
            };
            let destination = match self.game.random_region_position_owned(
                &region.war.base.region,
                rect.left,
                rect.top,
                rect.right.wrapping_sub(rect.left),
                rect.bottom.wrapping_sub(rect.top),
            ) {
                Ok(destination) => destination,
                Err(block) => {
                    report_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::RandomPosition(block),
                    );
                    continue;
                }
            };

            let direction = self
                .game
                .find_player(player_id)
                .expect("Nation m_vPlayers ID проверен до random position")
                .shape()
                .get_direction();
            self.game.finish_player_business(player_id);
            {
                let player = self
                    .game
                    .find_player_mut(player_id)
                    .expect("end_business не удаляет Nation player owner");
                player.prepare_nation_relive();
            }
            let previous = {
                let player = self
                    .game
                    .find_player(player_id)
                    .expect("end_business не удаляет player owner");
                let x = match player.shape().get_tile_x() {
                    Ok(x) => x,
                    Err(block) => {
                        report_four_nation_relive_block(
                            player_id,
                            FourNationReliveBlock::Coordinate(block),
                        );
                        continue;
                    }
                };
                let y = match player.shape().get_tile_y() {
                    Ok(y) => y,
                    Err(block) => {
                        report_four_nation_relive_block(
                            player_id,
                            FourNationReliveBlock::Coordinate(block),
                        );
                        continue;
                    }
                };
                (x, y)
            };

            if previous != (destination.x, destination.y) {
                let mut movement = CMessage::new(0xbf603);
                movement.base_mut().add_long(400);
                movement.base_mut().add_long(player_id);
                movement.base_mut().add_long(destination.x);
                movement.base_mut().add_long(destination.y);
                movement.base_mut().add_long(0);
                let player = self
                    .game
                    .find_player(player_id)
                    .expect("Nation relive player остаётся live до around send");
                if let Err(block) = self.game.send_game_shape_around(
                    &region.war.base,
                    player.shape(),
                    None,
                    &movement,
                ) {
                    report_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::Coordinate(block),
                    );
                }

                let facts = player.nation_relive_position_facts(area_width, area_height);
                let result = {
                    let player = self
                        .game
                        .find_player_mut(player_id)
                        .expect("around send не удаляет player owner");
                    region.war.base.set_move_shape_tile_position(
                        player.nation_relive_shape_mut(),
                        destination.x,
                        destination.y,
                        facts,
                    )
                };
                if let Err(block) = result {
                    report_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::Position(block),
                    );
                    continue;
                }
            }

            let (wallet_gold, bank_gold) = {
                let player = self
                    .game
                    .find_player_mut(player_id)
                    .expect("Nation relive player остаётся live до direction/log");
                player.nation_relive_shape_mut().set_direction(direction);
                (player.money(), player.depot_money())
            };
            let _change_log = self.game.send_player_change_region_log(
                0,
                player_id,
                wallet_gold,
                bank_gold,
                region.war.base.id,
                previous.0,
                previous.1,
                region.war.base.id,
                destination.x,
                destination.y,
            );
        }
    }

    /// Exact `ServerNationRegion::OnClearWar`: active/sleeping/pet/carriage
    /// traversal сохраняет area storage order, подходящие monsters получают
    /// disappear packet и state `1`, sleeping state `1` дополнительно
    /// попадают в delete-list, затем удаляются до четырёх `GS1120` NPC.
    fn clear_four_nation_war(
        &mut self,
        region: &mut super::super::servernationregion::ServerNationRegion,
    ) {
        let width = region.war.base.region.width;
        let height = region.war.base.region.height;
        for monster_id in region.war.base.area_monster_ids() {
            let Some(monster) = region.war.base.find_monster_by_id(monster_id) else {
                continue;
            };
            if !monster.can_clear_from_nation_war() {
                continue;
            }
            let shape = monster.move_shape().shape();
            let tile_x = match shape.get_tile_x() {
                Ok(tile_x) => tile_x,
                Err(block) => {
                    report_four_nation_clear_block(FourNationClearBlock::MonsterCoordinate {
                        monster_id,
                        block,
                    });
                    continue;
                }
            };
            let tile_y = match shape.get_tile_y() {
                Ok(tile_y) => tile_y,
                Err(block) => {
                    report_four_nation_clear_block(FourNationClearBlock::MonsterCoordinate {
                        monster_id,
                        block,
                    });
                    continue;
                }
            };
            if tile_x < 0 || tile_x >= width || tile_y < 0 || tile_y >= height {
                continue;
            }

            let identity = shape.identity();
            let mut removal = CMessage::new(0xbf504);
            removal.base_mut().add_long(identity.object_type);
            removal.base_mut().add_long(identity.id);
            removal.base_mut().add_long(0);
            let _delivery =
                self.game
                    .send_shape_around_in_region(&region.war.base, shape, None, &removal);
            region
                .war
                .base
                .find_monster_by_id_mut(monster_id)
                .expect("around send не удаляет Nation monster owner")
                .stage_for_delete();
        }

        for monster_id in region.war.base.sleeping_monster_ids() {
            let Some(monster) = region.war.base.find_monster_by_id(monster_id) else {
                continue;
            };
            if monster.staged_for_delete() {
                region.war.base.stage_delete_shape(ShapeIdentity {
                    object_type: 600,
                    id: monster_id,
                    ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
                });
            }
        }

        let npc_name = self.game.get_string_by_id(b"GS1120").to_vec();
        for _ in 0..4 {
            let Some(npc_id) = region.war.base.find_owned_npc_id_by_name(&npc_name) else {
                continue;
            };
            let Some(npc) = region.war.base.find_npc_by_id(npc_id).filter(|npc| {
                npc.move_shape().shape().base_object().get_name() == npc_name.as_slice()
            }) else {
                report_four_nation_clear_block(FourNationClearBlock::NpcTraversalMismatch {
                    npc_id,
                });
                continue;
            };
            let shape = npc.move_shape().shape();
            let identity = shape.identity();
            let mut removal = CMessage::new(0xbf504);
            removal.base_mut().add_long(identity.object_type);
            removal.base_mut().add_long(identity.id);
            removal.base_mut().add_long(0);
            let _delivery =
                self.game
                    .send_shape_around_in_region(&region.war.base, shape, None, &removal);
            if let Err(block) = region.war.base.remove_owned_npc_by_id(identity.id) {
                report_four_nation_clear_block(FourNationClearBlock::NpcRemoval {
                    npc_id: identity.id,
                    block,
                });
            }
        }
    }

    fn on_war_declare(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Village(region) => {
                        let effect = region.on_war_declare(war_number);
                        self.write_village_war_log(effect.string_id, &effect.region_name);
                    }
                    ServerRegionOwner::City(region) => {
                        let effect = region.on_war_declare(war_number);
                        self.write_city_war_log(
                            effect.string_id,
                            effect.war_number,
                            &effect.region_name,
                        );
                    }
                    ServerRegionOwner::Nation(region) => region.war.on_war_declare(war_number),
                    ServerRegionOwner::GodsBattle(region) => region.war.on_war_declare(war_number),
                    ServerRegionOwner::Base(region) => region.on_war_declare(war_number),
                    ServerRegionOwner::Country(region) => region.base.on_war_declare(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_declare(war_number);
                }
            }
        }
    }

    fn on_war_start(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Village(region) => {
                        if let Some(effect) = region.on_war_start(war_number) {
                            self.write_village_war_log(effect.string_id, &effect.region_name);
                        }
                    }
                    ServerRegionOwner::City(region) => {
                        if let Some(effect) = region.on_war_start(war_number) {
                            self.write_city_war_log(
                                effect.string_id,
                                effect.war_number,
                                &effect.region_name,
                            );
                        }
                    }
                    region => region.base_mut().on_war_start(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_start();
                }
            }
        }
    }

    fn on_war_time_out(&mut self, region: GameWarRegionHandle, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        match owner {
            ServerRegionOwner::Village(region) => {
                let effect = region.on_war_time_out(war_number);
                self.game
                    .restore_region_owner(ServerRegionOwner::Village(region));
                self.game.send_village_timeout(
                    effect.war_number,
                    effect.region_id,
                    effect.flag_owner_faction_id,
                );
                self.write_village_war_log("GS0240", &effect.region_name);
            }
            ServerRegionOwner::City(mut region) => {
                let effect = region.on_war_time_out(war_number);
                self.game
                    .restore_region_owner(ServerRegionOwner::City(region));
                let Some(effect) = effect else {
                    return;
                };
                if let Some(victory) = effect.victory {
                    self.game.send_city_victory(
                        victory.war_number,
                        victory.region_id,
                        victory.faction_id,
                        victory.union_id,
                    );
                }
                self.write_city_war_log(
                    effect.log_string_id,
                    effect.war_number,
                    &effect.region_name,
                );
            }
            mut region => {
                region.base_mut().on_war_time_out(war_number);
                self.game.restore_region_owner(region);
            }
        }
    }

    fn on_war_end(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if matches!(
                    self.game.find_region(region_id),
                    Some(ServerRegionOwner::Village(_))
                ) {
                    self.game.end_village_war(region_id, war_number);
                    return;
                }
                if matches!(
                    self.game.find_region(region_id),
                    Some(ServerRegionOwner::City(_))
                ) {
                    let Some(owner) = self.game.take_region_owner(region_id) else {
                        return;
                    };
                    let ServerRegionOwner::City(mut city) = owner else {
                        self.game.restore_region_owner(owner);
                        return;
                    };
                    let effect = {
                        let mut context = CityWarEndContext { game: self.game };
                        city.on_war_end(war_number, &mut context)
                    };
                    self.game
                        .restore_region_owner(ServerRegionOwner::City(city));
                    if let Some(effect) = effect {
                        for update in effect.build_updates {
                            self.game.publish_build_update(update);
                        }
                        self.write_city_war_log(
                            effect.log.string_id,
                            effect.log.war_number,
                            &effect.log.region_name,
                        );
                    }
                    return;
                }
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Nation(region) => region.war.on_war_end(war_number),
                    ServerRegionOwner::GodsBattle(region) => region.war.on_war_end(war_number),
                    region => region.base_mut().on_war_end(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_end();
                }
            }
        }
    }

    fn on_war_mass(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::City(region) => {
                        if let Some(effect) = region.on_war_mass(war_number) {
                            self.write_city_war_log(
                                effect.string_id,
                                effect.war_number,
                                &effect.region_name,
                            );
                        }
                    }
                    region => region.base_mut().on_war_mass(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_mass();
                }
            }
        }
    }
}

impl WarFactionUpdateContext for GameOrganizingWarContext<'_> {
    fn update_attack_city_contend_player(&mut self, region_id: i32, schedules: &CAttackCitySys) {
        self.update_contenders(region_id, ContendSchedule::AttackCity(schedules));
    }

    fn update_village_contend_player(&mut self, region_id: i32, schedules: &CVillageWarSys) {
        self.update_contenders(region_id, ContendSchedule::Village(schedules));
    }
}

impl FourNationPhaseContext for GameOrganizingWarContext<'_> {
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn find_server_nation_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.game.find_region(region_id),
            Some(ServerRegionOwner::Nation(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        GameOrganizingWarContext::on_war_declare(self, region, war_number);
    }

    fn on_nation_war_declare(
        &mut self,
        region: Self::Region,
        war_number: i32,
        _sign_up_counts: [i32; 5],
    ) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.game.find_region_mut(region_id) {
            region.war.on_war_declare(war_number);
            region.reset_for_war_declare();
        }
    }

    fn on_war_mass(&mut self, region: Self::Region, war_number: i32) {
        if let GameWarRegionHandle::Local(region_id) = region
            && let Some(owner) = self.game.take_region_owner(region_id)
        {
            let ServerRegionOwner::Nation(mut region) = owner else {
                self.game.restore_region_owner(owner);
                GameOrganizingWarContext::on_war_mass(
                    self,
                    GameWarRegionHandle::Local(region_id),
                    war_number,
                );
                return;
            };
            region.war.base.on_war_mass(war_number);
            self.kick_out_four_nation_players(&mut region);
            self.game
                .restore_region_owner(ServerRegionOwner::Nation(region));
            return;
        }
        GameOrganizingWarContext::on_war_mass(self, region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        GameOrganizingWarContext::on_war_start(self, region, war_number);
    }

    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        match self.game.find_region_mut(region_id) {
            Some(ServerRegionOwner::Nation(region)) => {
                region.reset_for_region_refresh();
                self.refresh_four_nation_magic_stones(region_id);
            }
            Some(region) => region.base_mut().on_refresh_region(war_number),
            None => {}
        }
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        if let GameWarRegionHandle::Local(region_id) = region
            && let Some(owner) = self.game.take_region_owner(region_id)
        {
            let ServerRegionOwner::Nation(mut region) = owner else {
                self.game.restore_region_owner(owner);
                GameOrganizingWarContext::on_war_end(
                    self,
                    GameWarRegionHandle::Local(region_id),
                    war_number,
                );
                return;
            };

            add_game_log_text(b"ServerNationRegion::OnWarEnd");
            region.war.on_war_end(war_number);

            let end = CMessage::new(0xbf819);
            let _end_delivery = end.send_to_region(Some(&region.war.base), None, self.game);
            self.kick_out_four_nation_players(&mut region);

            let awards = region.take_player_war_awards(game_tick_milliseconds);
            for award in awards {
                let mut elapsed = CMessage::new(0x6031c);
                elapsed.base_mut().add_long(award.player_id);
                elapsed.base_mut().add_ulong(award.elapsed_time_ms);
                elapsed.base_mut().add_long(award.country);
                let _elapsed_delivery = elapsed.send(self.game, false);

                let log = format_legacy_integer_fields(
                    self.game.get_string_by_id(b"GS1134"),
                    &[
                        award.player_id.to_string(),
                        (award.elapsed_time_ms / 1000).to_string(),
                        award.exploit.to_string(),
                    ],
                    0x7f,
                );
                add_game_log_text(&log);

                let Some(previous_exploit) = self
                    .game
                    .find_player(award.player_id)
                    .map(|player| player.base_properties().exploit)
                else {
                    let mut offline = CMessage::new(0x6031a);
                    offline.base_mut().add_long(award.player_id);
                    offline.base_mut().add_ulong(award.exploit);
                    let _offline_delivery = offline.send(self.game, false);
                    continue;
                };

                let advertised_exploit = previous_exploit.wrapping_add(award.exploit);
                let mut property = CMessage::new(0xbf80c);
                property.base_mut().add_long(award.player_id);
                property.base_mut().add_long(award.player_id);
                property.base_mut().add_str(Some(c"dwExploit"));
                property.base_mut().add_ulong(advertised_exploit);
                let _property_delivery =
                    property.send_to_player(self.game.net_server(), award.player_id);
                {
                    let player = self
                        .game
                        .find_player_mut(award.player_id)
                        .expect("online award player проверен до mutation");
                    player.set_exploit_property_value(advertised_exploit)
                }
                let _property_update = self
                    .game
                    .update_player_properties(award.player_id)
                    .expect("online award player сохранён после exploit mutation");

                let notice = format_legacy_integer_fields(
                    self.game.get_string_by_id(b"GS1135"),
                    &[award.exploit.to_string()],
                    0xff,
                );
                let notice = CString::new(notice)
                    .expect("localized FourNation exploit notice обрезан до NUL");
                let mut message = CMessage::new(0xbf806);
                message.base_mut().add_ulong(u32::MAX);
                message.base_mut().add_ulong(0xffff_0000);
                message.base_mut().add_str(Some(&notice));
                let _notice_delivery =
                    message.send_to_player(self.game.net_server(), award.player_id);
            }

            region.reset_materialized_war_state();
            self.game
                .restore_region_owner(ServerRegionOwner::Nation(region));
            return;
        }
        GameOrganizingWarContext::on_war_end(self, region, war_number);
    }

    fn on_clear_war(&mut self, region: Self::Region, _war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.game.restore_region_owner(owner);
            return;
        };
        self.clear_four_nation_war(&mut region);
        self.game
            .restore_region_owner(ServerRegionOwner::Nation(region));
    }

    fn take_war_results(&mut self, region: Self::Region) -> [u32; 5] {
        let GameWarRegionHandle::Local(region_id) = region else {
            return [0; 5];
        };
        let Some(ServerRegionOwner::Nation(region)) = self.game.find_region_mut(region_id) else {
            return [0; 5];
        };
        region.take_stone_guard_results()
    }

    fn add_war_end_log(&mut self, war_number: i32) {
        let log = format_legacy_integer_fields(
            b"CFourNationWarSys::OnWarEnd,lWarID:%d",
            &[war_number.to_string()],
            0xff,
        );
        add_game_log_text(&log);
    }
}

impl AttackCityPhaseContext for GameOrganizingWarContext<'_> {
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_declare(region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_start(region, war_number);
    }

    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_time_out(region, war_number);
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_end(region, war_number);
    }

    fn on_war_mass(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_mass(region, war_number);
    }

    fn on_clear_other_player(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        self.game.clear_city_other_players(region_id, war_number);
    }

    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let ServerRegionOwner::City(region) = &owner else {
            self.game.restore_region_owner(owner);
            return;
        };
        let targets = region.on_refresh_region(war_number);
        self.game.restore_region_owner(owner);
        self.game
            .refresh_city_region_guards(region_id, targets);
    }
}

impl VillageWarPhaseContext for GameOrganizingWarContext<'_> {
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn owned_city_faction(&mut self, region: Self::Region) -> i32 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().owned_city_faction())
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.owned_city_org().0)
                .unwrap_or(0),
        }
    }

    fn owned_city_union(&mut self, region: Self::Region) -> i32 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().owned_city_union())
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.owned_city_org().1)
                .unwrap_or(0),
        }
    }

    fn set_owned_city_org(&mut self, region: Self::Region, faction_id: i32, union_id: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.game.find_region_mut(region_id) {
                    region.base_mut().set_owned_city_org(faction_id, union_id);
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.set_owned_city_org(faction_id, union_id);
                }
            }
        }
    }

    fn region_country(&self, region: Self::Region) -> u8 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().country)
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.country())
                .unwrap_or(0),
        }
    }

    fn set_region_country(&mut self, region: Self::Region, country: u8) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.game.find_region_mut(region_id) {
                    region.base_mut().country = country;
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.set_country(country);
                }
            }
        }
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_declare(region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_start(region, war_number);
    }

    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_time_out(region, war_number);
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_end(region, war_number);
    }

    fn send_clear_player_notice(&mut self, region: Self::Region) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let name = owner.name().to_vec();
        let text =
            format_legacy_text_fields(self.game.get_string_by_id(b"GS0127"), &[&name], 0x3ff);
        let _delivery = colored_player_notice_message(0xffda_edfe, 0, &text).send_to_region(
            Some(owner.base()),
            None,
            self.game,
        );
        self.game.restore_region_owner(owner);
    }

    fn start_clear_player_out(&mut self, region: Self::Region, delay_ms: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let now_ms = game_tick_milliseconds();
        if let Some(region) = self.game.find_region_mut(region_id) {
            region
                .base_mut()
                .start_clear_player_out_at(delay_ms, now_ms);
        }
    }
}
