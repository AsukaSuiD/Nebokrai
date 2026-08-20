//! Обработчики server-семейств исторического WorldServer.
//!
//! Статус владельца: `IMPLEMENTED` для `gameserv_conn_log`,
//! внутрипроцессного события `0x3FC03`,
//! snapshot/cleanup хвоста `0x5FA03`, обычных opcode `0x4FC01..=0x4FC03` и
//! `0x5FA0A..=0x5FA0D` из
//! `OnServerMessage` RVA `0x000ADCF0`;
//! остальные ветви остаются `UNKNOWN` (исследовательский декомпилят хранится локально) ниже. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\servermessage.cpp:87`.
//!
//! Старый reconnect передавал `CMyNetClient*` как `long` внутри сообщения.
//! Rust получает тот же элемент общей FIFO как typed event: сначала вызывает
//! `Close` прежнего owner-а и уничтожает его, затем публикует новый, выдаёт
//! операторское подтверждение, ставит CD-key snapshot, приоритетную регистрацию
//! `0x1FE01 + dwNumber + strName\0` и лишь после попытки `Send` включает
//! control-send. Результат обоих send исходник игнорировал; typed outcome
//! сохраняет их без изменения порядка. `Option`, owned client и `Drop` заменяют
//! nullable pointer, integer-pointer и ручной deleting destructor.
//!
//! Если доказанный инвариант CD-key snapshot нарушен, replacement и позиция
//! операторского подтверждения уже достигнуты, но регистрацию и control-send
//! исходный код ещё не выполнял. Ошибка сохраняет этот частичный эффект, не
//! выбирая реакцию старого null-dereference. Полное сырьё реализованной ветви
//! удалено; соседние server-opcode остаются рабочим материалом.
//!
//! `gameserv_conn_log` строит `0x1FE05 + peer IPv4 word + GameServer index`
//! и неприоритетно ставит его текущему nullable LoginServer client. Оба поля
//! остаются беззнаковыми 32-битными словами; отсутствие client сохраняет
//! исходный нулевой результат, а проигнорированный `Send` доступен вызывающему.
//!
//! `0x4FC03` читает один signed Windows `long` и без дополнительных проверок
//! присваивает его `CGame::_login_server_id`. Готовый `CBaseMessage::get_long`
//! сдвигает cursor только при наличии всех четырёх little-endian bytes; короткий
//! payload сохраняет принятую legacy-замену нулём и неподвижный cursor. Typed
//! outcome сообщает, были ли байты фактически прочитаны, не меняя единственный
//! исходный побочный эффект. Остальные server-opcode возвращаются owned
//! вызывающему и не выдаются за исполненные.
//!
//! `0x4FC02` не читает payload и не меняет `CGame`: создаёт пустое сообщение
//! `0x7F80B` и вызывает общий World `SendAll`. Nullable `s_pNetServer` уже
//! выражен `Option` и даёт исходный `0`; живой `ServerCommandHandle` синхронно
//! копирует CRC-envelope до возврата. Игнорировавшийся исходником результат
//! доступен typed outcome, а `Drop` заменяет stack-destructor сообщения.
//!
//! `0x4FC01` сначала ставит ping-флаг, полностью очищает накопленные ответы и
//! только затем запоминает отдельный wrapping `timeGetTime`. После этих мутаций
//! ветвь рассылает пустой `0x7F809`; ошибка готового `SendAll` доступна typed
//! outcome и не откатывает уже начатый цикл ping.
//!
//! Ответ `0x5FA0A` без проверки ping-флага читает signed player count,
//! форматирует metadata IPv4 от младшего к старшему octet, копирует signed
//! map ID и добавляет один `tagPingGameServerInfo`. Короткий payload сохраняет
//! legacy-ноль и неподвижный cursor; запись всё равно добавляется.
//!
//! `0x5FA0C` независимо читает signed payload после снимков setup world number
//! и metadata map ID. Только два ненулевых ID порождают неприоритетный
//! `0x1FE07 + world + map + value` текущему nullable Login client. Исходно
//! неинициализированный до setup `dwNumber` остаётся отдельной safe-границей
//! после уже доказанного чтения payload, а не получает выдуманный ноль.
//!
//! `0x5FA0D` только читает signed `long`, затем byte-exact строку с границей
//! `0x80`. Поля не получают недоказанного доменного имени; отсутствие send,
//! operator-log и мутаций `CGame` сохранено буквально.
//!
//! `0x5FA0B` читает и игнорирует один signed `char`, затем signed region ID,
//! находит назначенный GameServer двумя ordered map lookup, меняет opcode того
//! же входного сообщения на `0x7F80A` и безусловно вызывает `SendToMapID`, в
//! том числе с legacy-нулём при отсутствии любой ступени. Короткие getters
//! сохраняют ноль и неподвижный cursor; typed outcome отдельно сообщает
//! полноту обоих чтений и исходно игнорировавшийся результат отправки.
//!
//! Reached хвост `0x5FA03` после равенства response-count сначала уже сбросил
//! `m_nDBResponsed`, затем выполняет полный `GenerateDBData` и строго
//! `ClearMapPlayerForOffline -> ClearRestorePlayer -> ClearCreationPlayer ->
//! ClearDeletionPlayer -> ClearOfflinePlayer`. Этот owner хранит собственную
//! caller-оркестрацию отдельно от совпадающей ветви `CGame::Run`. После cleanup
//! прежний handle-state закрывается и возвращается точная одноразовая
//! `SaveThreadFunc` launch-обязанность; системный thread не создаётся.

use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;

use crate::nets::basemessage::CBaseMessage;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::networld::mynetclient::CMyNetClient;
use crate::worldserver::appworld::country::country::CountryKingSaveLimits;
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::organizingsystem::factionwarsys::CFactionWarSys;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::player::PlayerPropertyCoefficients;
use crate::worldserver::worldserver::game::{
    CGame, WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldGenerateDbDataBlock,
    WorldGenerateDbDataReport, WorldPingGameServerInfo, WorldSaveThreadHandleState,
    WorldSaveThreadLaunchRequest, prepare_save_thread_launch,
};
use crate::worldserver::worldserver::honorranks::CHonorRanks;

/// Наблюдаемый итог typed-замены LoginServer client из ветки `0x3FC03`.
#[derive(Debug)]
pub(crate) struct WorldLoginClientReplacement {
    /// Был ли прежний owner закрыт и уничтожен перед присваиванием нового.
    pub(crate) previous_client_closed: bool,
    /// Соответствует позиции `AddLogText("Connect To LoginServer SUCCESS!")`.
    pub(crate) connected_notice: bool,
    /// Поставленный перед регистрацией полный список online account.
    pub(crate) cdkey_snapshot: WorldCdkeySnapshot,
    /// Исходно игнорировавшийся результат приоритетной регистрации мира.
    pub(crate) registration: Result<i32, SendMessageError>,
}

/// Наблюдаемый результат `gameserv_conn_log`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerConnectedLog {
    pub(crate) peer_ipv4: u32,
    pub(crate) game_server_index: u32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Snapshot/cleanup хвост `0x5FA03` и достигнутый launch call-site.
#[derive(Debug)]
pub(crate) struct WorldCompletedSaveResponseLaunchReport {
    pub(crate) snapshot: WorldGenerateDbDataReport,
    pub(crate) launch: WorldSaveThreadLaunchRequest,
}

/// Результат исполненного обычного opcode `OnServerMessage`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldServerMessageOutcome {
    GameServerBroadcast(WorldGameServerBroadcast),
    GameServerPingResponseRecorded(WorldGameServerPingResponse),
    GameServerPingStarted(WorldGameServerPingStart),
    LoginServerTupleRelay(WorldLoginServerTupleRelay),
    LoginServerIdentityAssigned(WorldLoginServerIdentity),
    OpaqueFieldsRead(WorldOpaqueServerFields),
    RegionMessageRelayed(WorldRegionMessageRelay),
}

/// Итог пустого broadcast из ветки `0x4FC02`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerBroadcast {
    /// Полный opcode построенного исходящего сообщения.
    pub(crate) message_type: i32,
    /// Исходно игнорировавшийся результат `CMessage::SendAll`.
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Наблюдаемый итог запуска цикла ping из ветки `0x4FC01`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerPingStart {
    /// Число накопленных ответов, удалённых до снятия нового tick.
    pub(crate) cleared_responses: usize,
    /// Записанный wrapping millisecond tick нового цикла.
    pub(crate) started_at_ms: u32,
    /// Исходно игнорировавшийся результат `CMessage::SendAll`.
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Наблюдаемый итог принятого ответа GameServer из ветки `0x5FA0A`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerPingResponse {
    /// Полная семантическая копия элемента, добавленного в vector.
    pub(crate) response: WorldPingGameServerInfo,
    /// Размер vector после безусловного `push_back`.
    pub(crate) response_count: usize,
    /// `false` означает legacy-ноль без сдвига cursor короткого payload.
    pub(crate) payload_complete: bool,
}

/// Typed-результат условной пересылки tuple из ветки `0x5FA0C`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLoginServerTupleRelay {
    /// Setup ещё не назначил исходно неинициализированный `dwNumber`.
    WorldNumberUnavailable {
        map_id: i32,
        value: i32,
        payload_complete: bool,
    },
    /// Нулевой world либо map ID подавил создание исходящего сообщения.
    Suppressed {
        world_number: u32,
        map_id: i32,
        value: i32,
        payload_complete: bool,
    },
    /// Оба ID ненулевые и попытка неприоритетной отправки выполнена.
    Forwarded {
        world_number: u32,
        map_id: i32,
        value: i32,
        payload_complete: bool,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Два намеренно безымянных поля, прочитанных веткой `0x5FA0D`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldOpaqueServerFields {
    /// Signed `long`, включая legacy-ноль короткого payload.
    pub(crate) value: i32,
    /// Был ли numeric getter способен сдвинуть cursor на четыре bytes.
    pub(crate) numeric_complete: bool,
    /// Byte-exact результат готового ограниченного `GetStr(..., 0x80)`.
    pub(crate) text: Vec<u8>,
}

/// Наблюдаемый итог региональной пересылки из ветки `0x5FA0B`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionMessageRelay {
    /// Прочитанное, но не использованное исходником первое поле.
    pub(crate) ignored_selector: i8,
    /// Был ли `GetChar` способен сдвинуть cursor на один byte.
    pub(crate) selector_complete: bool,
    /// Signed region ID, включая legacy-ноль короткого payload.
    pub(crate) region_id: i32,
    /// Был ли `GetLong` способен сдвинуть cursor на четыре bytes.
    pub(crate) region_complete: bool,
    /// Найденный `dwIndex` GameServer либо исходный ноль.
    pub(crate) game_server_number: i32,
    /// Полный opcode того же входного сообщения после мутации.
    pub(crate) message_type: i32,
    /// Исходно игнорировавшийся результат `CMessage::SendToMapID`.
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Наблюдаемый итог ветки `0x4FC03`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldLoginServerIdentity {
    /// Значение поля до безусловного присваивания.
    pub(crate) previous_login_server_id: i32,
    /// Новый signed `long`, включая legacy-ноль короткого payload.
    pub(crate) login_server_id: i32,
    /// `false` означает, что `GetLong` не сдвинул cursor и вернул legacy-ноль.
    pub(crate) payload_complete: bool,
}

/// Узкая диспетчеризация уже выбранного server-owner-а.
pub(crate) enum WorldServerMessageDispatch {
    Handled(WorldServerMessageOutcome),
    Pending(CMessage),
}

/// Безопасная граница после уже выполненной замены LoginServer owner-а.
#[derive(Debug)]
pub(crate) struct WorldServerMessageError {
    /// Был ли прежний owner закрыт до достижения ошибки snapshot.
    pub(crate) previous_client_closed: bool,
    /// Операторское подтверждение уже находится перед вызовом snapshot.
    pub(crate) connected_notice: bool,
    source: WorldCdkeySnapshotError,
}

impl fmt::Display for WorldServerMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "LoginServer client заменён, но CD-key snapshot не построен: {}",
            self.source
        )
    }
}

impl Error for WorldServerMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Выполняет точный helper `gameserv_conn_log` перед продолжением `0x5FA01`.
pub(crate) fn game_server_connected_log(
    game: &CGame,
    peer_ipv4: u32,
    game_server_index: u32,
) -> WorldGameServerConnectedLog {
    let mut message = CMessage::new(0x0001_FE05);
    message.base_mut().add_ulong(peer_ipv4);
    message.base_mut().add_ulong(game_server_index);
    let delivery = message.send(
        game.current_login_client().map(CMyNetClient::send_queue),
        false,
    );
    WorldGameServerConnectedLog {
        peer_ipv4,
        game_server_index,
        delivery,
    }
}

/// Выполняет только доказанную ветку `OnServerMessage(0x3FC03)`.
pub(crate) fn on_login_client_reconnected(
    game: &mut CGame,
    client: CMyNetClient,
) -> Result<WorldLoginClientReplacement, WorldServerMessageError> {
    let previous_client_closed = game.replace_login_client(client);

    // Эта позиция является typed-эквивалентом операторского AddLogText.
    let connected_notice = true;
    let cdkey_snapshot = game
        .send_cdkey_to_login_server()
        .map_err(|source| WorldServerMessageError {
            previous_client_closed,
            connected_notice,
            source,
        })?
        .expect("новый LoginServer client уже опубликован");

    let mut registration = CMessage::new(0x0001_FE01);
    registration
        .base_mut()
        .add_ulong(game.world_number_after_cdkey_snapshot());
    add_legacy_c_string(registration.base_mut(), game.world_name());
    let registration = registration.send(
        game.current_login_client().map(CMyNetClient::send_queue),
        true,
    );
    game.current_login_client_mut()
        .expect("новый LoginServer client остаётся опубликованным")
        .enable_control_send();

    Ok(WorldLoginClientReplacement {
        previous_client_closed,
        connected_notice,
        cdkey_snapshot,
        registration,
    })
}

/// Выполняет snapshot/cleanup хвост завершённой ветви `0x5FA03`.
///
/// Счётчик DB-ответов уже сброшен caller-ом. Handle replacement выполняется
/// только после успешных snapshot/cleanup и не создаёт системный thread.
#[allow(
    clippy::too_many_arguments,
    reason = "исходный handler повторно обращался к тем же singleton/static владельцам"
)]
pub(crate) fn materialize_completed_save_response_snapshot(
    game: &mut CGame,
    registry: &GoodsBasePropertiesRegistry,
    organizing_ctrl: &mut COrganizingCtrl,
    coefficients: &PlayerPropertyCoefficients,
    faction_war_sys: &CFactionWarSys,
    country_handler: &CCountryHandler,
    country_limits: CountryKingSaveLimits,
    honor_ranks: &mut CHonorRanks,
    save_thread_handle: &mut WorldSaveThreadHandleState,
) -> Result<WorldCompletedSaveResponseLaunchReport, WorldGenerateDbDataBlock> {
    let snapshot = game.generate_db_data(
        registry,
        organizing_ctrl,
        coefficients,
        faction_war_sys,
        country_handler,
        country_limits,
        honor_ranks,
    )?;
    game.clear_map_player_for_offline();
    game.clear_restore_player();
    game.clear_creation_player();
    game.clear_deletion_player();
    game.clear_offline_player();
    let launch = prepare_save_thread_launch(save_thread_handle);
    Ok(WorldCompletedSaveResponseLaunchReport { snapshot, launch })
}

/// Исполняет только уже восстановленные обычные ветви `OnServerMessage`.
pub(crate) fn on_server_message(
    game: &mut CGame,
    mut message: CMessage,
) -> WorldServerMessageDispatch {
    match message.message_type() {
        0x0004_FC01 => {
            let (cleared_responses, started_at_ms) = game.begin_game_server_ping();
            let ping = CMessage::new(0x0007_F809);
            let sender = game.current_game_server_sender();
            let delivery = ping.send_all(sender.as_ref());
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GameServerPingStarted(
                WorldGameServerPingStart {
                    cleared_responses,
                    started_at_ms,
                    delivery,
                },
            ))
        }
        0x0004_FC02 => {
            let broadcast = CMessage::new(0x0007_F80B);
            let sender = game.current_game_server_sender();
            let delivery = broadcast.send_all(sender.as_ref());
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GameServerBroadcast(
                WorldGameServerBroadcast {
                    message_type: 0x0007_F80B,
                    delivery,
                },
            ))
        }
        0x0004_FC03 => {
            let decoded = message.base_mut().get_long();
            let login_server_id = decoded.unwrap_or(0);
            let previous_login_server_id = game.assign_login_server_id(login_server_id);
            WorldServerMessageDispatch::Handled(
                WorldServerMessageOutcome::LoginServerIdentityAssigned(WorldLoginServerIdentity {
                    previous_login_server_id,
                    login_server_id,
                    payload_complete: decoded.is_some(),
                }),
            )
        }
        0x0005_FA0A => {
            let decoded = message.base_mut().get_long();
            let player_count = decoded.unwrap_or(0);
            let ip = format_legacy_ipv4(message.ip());
            let map_id = message.map_id();
            let response = WorldPingGameServerInfo {
                ip,
                map_id,
                player_count,
            };
            let response_count = game.record_game_server_ping(response.clone());
            WorldServerMessageDispatch::Handled(
                WorldServerMessageOutcome::GameServerPingResponseRecorded(
                    WorldGameServerPingResponse {
                        response,
                        response_count,
                        payload_complete: decoded.is_some(),
                    },
                ),
            )
        }
        0x0005_FA0B => {
            let selector = message.base_mut().get_char();
            let decoded_region = message.base_mut().get_long();
            let region_id = decoded_region.unwrap_or(0);
            let game_server_number = game.game_server_number_by_region_id(region_id);
            message.set_message_type(0x0007_F80A);
            let delivery = game.send_msg_to_game_server(game_server_number, &message);
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::RegionMessageRelayed(
                WorldRegionMessageRelay {
                    ignored_selector: selector.unwrap_or(0),
                    selector_complete: selector.is_some(),
                    region_id,
                    region_complete: decoded_region.is_some(),
                    game_server_number,
                    message_type: 0x0007_F80A,
                    delivery,
                },
            ))
        }
        0x0005_FA0C => {
            let world_number = game.configured_world_number();
            let map_id = message.map_id();
            let decoded = message.base_mut().get_long();
            let value = decoded.unwrap_or(0);
            let payload_complete = decoded.is_some();

            let relay = match world_number {
                None => {
                    // BLOCKED_MISSING_FACT: до успешного LoadSetup старый
                    // dwNumber был неинициализирован. Реакция его чтения не
                    // назначается; доказанное GetLong уже выполнено выше.
                    WorldLoginServerTupleRelay::WorldNumberUnavailable {
                        map_id,
                        value,
                        payload_complete,
                    }
                }
                Some(world_number) if world_number == 0 || map_id == 0 => {
                    WorldLoginServerTupleRelay::Suppressed {
                        world_number,
                        map_id,
                        value,
                        payload_complete,
                    }
                }
                Some(world_number) => {
                    let mut forwarded = CMessage::new(0x0001_FE07);
                    forwarded.base_mut().add_ulong(world_number);
                    forwarded.base_mut().add_long(map_id);
                    forwarded.base_mut().add_long(value);
                    let delivery = forwarded.send(
                        game.current_login_client().map(CMyNetClient::send_queue),
                        false,
                    );
                    WorldLoginServerTupleRelay::Forwarded {
                        world_number,
                        map_id,
                        value,
                        payload_complete,
                        message_type: 0x0001_FE07,
                        delivery,
                    }
                }
            };
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::LoginServerTupleRelay(
                relay,
            ))
        }
        0x0005_FA0D => {
            let decoded = message.base_mut().get_long();
            let value = decoded.unwrap_or(0);
            let text = message
                .base_mut()
                .get_str_bytes(0x80)
                .expect("ненулевая граница GetStr всегда допустима");
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::OpaqueFieldsRead(
                WorldOpaqueServerFields {
                    value,
                    numeric_complete: decoded.is_some(),
                    text,
                },
            ))
        }
        _ => WorldServerMessageDispatch::Pending(message),
    }
}

fn format_legacy_ipv4(raw: u32) -> Vec<u8> {
    Ipv4Addr::from(raw.to_le_bytes()).to_string().into_bytes()
}

fn add_legacy_c_string(message: &mut CBaseMessage, bytes: &[u8]) {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    message.add(&bytes[..end]);
    message.add_byte(0);
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\servermessage.cpp

// ============================================================================
// FUNCTION: OnServerMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\servermessage.cpp:87
// RVA: 0x000ADCF0
// ADDRESS: 004adcf0
// PROTOTYPE: void __cdecl OnServerMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
