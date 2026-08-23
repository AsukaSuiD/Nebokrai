//! WorldServer dispatcher-owner `OnLogMessage`.
//!
//! Источник контракта — `WorldServer/Nworldserver.exe` и
//! `WorldServer/WorldServer.pdb`. Owner обрабатывает player lifecycle
//! `0x5FB01/0x5FB02`, player-list `0x4FB01`, delete-role `0x4FB02`,
//! restore-role `0x4FB03`, create-role `0x4FB04`, select-player `0x4FB05` и
//! account cleanup `0x4FB06/0x4FB07`. Неизвестный opcode возвращается без
//! побочных эффектов и не передаётся следующему dispatcher-у.
//!
//! Lifecycle сохраняет порядок переходов между login, offline и online,
//! вызов team-exit и адресную отправку GameServer. Успешный online-маршрут
//! отправляет полный `CPlayer` до изменения списков. Два дополнительных long
//! после player ID читаются и игнорируются; они не меняют исходный payload
//! success-ответа. Offline snapshot сначала очищает transient pet/faction
//! state, затем отвечает LoginServer, обновляет списки и только после этого
//! уведомляет online-друзей в исходном порядке.
//!
//! Restore удаляет первое совпадение из live deletion-list, добавляет уникальный
//! ID в хвост restore-list и посылает `0x1FF04/0x15` без priority. Account
//! cleanup сначала предпочитает действующий online-маршрут; только при его
//! отсутствии удаляет первый login account и отвечает `0x1FF06`.
//!
//! `CRsPlayer::OpenPlayerBase` сначала выполняет отдельный `SELECT ID`, затем
//! кодирует account и wrapping-сумму DB/creation rows. Нулевой результат
//! содержит дополнительный `long(0)`. Ненулевой путь читает
//! `SELECT * ... ORDER BY id`, публикует DB rows перед creation rows, подменяет
//! scalar-ы действующей map/save-копией и вычисляет deletion-status с
//! приоритетом restore, live deletion, DB `DelDate`. N+1 lookup `DelDate` и
//! SQL-порядок сохранены.
//!
//! Delete-role снимает время до organizing/DB gates. Ошибки `1..=4` отвечают
//! `0x1FF03/0x13`; повторное удаление использует тот же ответ с нулём. Успех
//! снимает restore, добавляет live deletion time и отвечает `0x14` с signed
//! byte `dwDelDays`; optional delete-log ставится в FIFO только после send.
//! Faction-проверка использует `GetMembers`, union-ветвь отвязывает faction и
//! даёт код `3`; код `2` недостижим в этом owner-е.
//!
//! Create-role строго выполняет limit, RU sex/occupation, country,
//! `WordsFilter` и шесть name lookup-ов. Первая terminal-проверка определяет
//! ответ `0x18/0x19/0x1A/0x17`. Успех применяет default property, выдаёт новый
//! player ID, добавляет начальную экипировку и лишь затем помещает игрока в
//! creation-list; ответ сохраняет исходный порядок identity, equipment и
//! region. Select-player проверяет binding через live map, frozen save-map и
//! только затем DB. При miss он ставит fixed account record в player-load FIFO;
//! direct clone публикует Largess/login/map до friends и сброса flags.
//!
//! `VecDeque`, owned wire snapshots и параметризованный Tiberius заменяют
//! list/deque nodes, ADO/COM, `_sprintf` и временные C++ buffers. Они не меняют
//! wire, SQL-порядок, очереди и частичные lifecycle-эффекты.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::dbaccess::worlddb::rsplayer::{
    PlayerBaseDatabaseRow, RsPlayerOwner, TiberiusRsPlayer,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::playerlist::CPlayerList;
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::country::country::{
    CountryExileTextArgument, CountryHasJobContext,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::message::writelogmessage::{
    WorldPlayerDeleteLogWrite, WorldWriteLogCommand,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, OrganizingDeleteRoleBlock, OrganizingDeleteRoleOutcome,
    OrganizingNameLookupBlock,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::union::UnionFormatArgument;
use crate::worldserver::appworld::player::{
    CPlayer, PlayerBaseWireSnapshot, PlayerCodecError, PlayerDbProjectionBlock,
    PlayerDefaultPropertyBlock, PlayerDefaultPropertyReport, PlayerPropertyCoefficients,
};
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::worldserver::game::{
    CGame, WorldCreationPlayerAppendOutcome, WorldLoadedPlayerRouteOrder,
    WorldLoginTimeoutTeamExit,
    WorldOnlinePlayerAppendOutcome, WorldOnlinePlayerRemoveOutcome, WorldOriginGoodsBlock,
    WorldOriginGoodsReport, WorldPlayerLoadRequestBlock,
    WorldPlayerLoadRequestOutcome, WorldPlayerNameLookupError,
    WorldProcessPlayerDataQueueError, WorldProcessPlayerDataQueueOutcome,
    WorldReturnedPlayerDecode, legacy_tick_ms,
};
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const PLAYER_BASE_REQUEST: i32 = 0x0004_FB01;
const DELETE_ROLE_REQUEST: i32 = 0x0004_FB02;
const PLAYER_DETAIL_REQUEST: i32 = 0x0005_FB01;
const PLAYER_RETURN_REQUEST: i32 = 0x0005_FB02;
const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
const CREATE_ROLE_REQUEST: i32 = 0x0004_FB04;
const PLAYER_SELECT_REQUEST: i32 = 0x0004_FB05;
const ACCOUNT_LOGIN_CLEANUP_REQUEST: i32 = 0x0004_FB06;
const ACCOUNT_DISCONNECT_REQUEST: i32 = 0x0004_FB07;
const PLAYER_BASE_RESPONSE: i32 = 0x0001_FF02;
const DELETE_ROLE_RESPONSE: i32 = 0x0001_FF03;
const DELETE_ROLE_REJECTED_STATUS: i8 = 0x13;
const DELETE_ROLE_SCHEDULED_STATUS: i8 = 0x14;
const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
const RESTORE_ROLE_STATUS: i8 = 0x15;
const CREATE_ROLE_RESPONSE: i32 = 0x0001_FF05;
const CREATE_ROLE_DUPLICATE_STATUS: i8 = 0x17;
const CREATE_ROLE_INVALID_STATUS: i8 = 0x18;
const CREATE_ROLE_LIMIT_STATUS: i8 = 0x19;
const CREATE_ROLE_FILTER_STATUS: i8 = 0x1A;
const CREATE_ROLE_SUCCESS_STATUS: i8 = 0x1B;
const PLAYER_SELECT_RESPONSE: i32 = 0x0001_FF01;
const PLAYER_SELECT_REJECTED_STATUS: i8 = 0x1C;
const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;
const PLAYER_DETAIL_RESPONSE: i32 = 0x0007_F901;
const PLAYER_FRIEND_OFFLINE_RESPONSE: i32 = 0x0007_F905;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRestoreRoleOutcome {
    pub(crate) account: Vec<u8>,
    pub(crate) player_id: u32,
    pub(crate) player_id_complete: bool,
    pub(crate) response_type: i32,
    pub(crate) status: i8,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCreateRoleRequest {
    pub(crate) name: Vec<u8>,
    pub(crate) sex: u8,
    pub(crate) occupation: u8,
    pub(crate) head_picture: u8,
    pub(crate) face_picture: u8,
    pub(crate) country: u8,
    pub(crate) account: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCreateRoleFailureStage {
    DatabaseCount,
    CharacterLimit,
    OccupationSex,
    Country,
    WordsFilter,
    DuplicateName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCreateRoleAppendCollision {
    DuplicateCreationId,
    ExistingMapOwner,
}

#[derive(Debug)]
pub(crate) enum WorldCreateRoleBlock {
    PlayerName(WorldPlayerNameLookupError),
    OrganizingName(OrganizingNameLookupBlock),
    DefaultProperty(PlayerDefaultPropertyBlock),
    OriginGoods(WorldOriginGoodsBlock),
    AppendCollision(WorldCreateRoleAppendCollision),
    Snapshot(PlayerDbProjectionBlock),
    PublishedPlayerMissing { player_id: u32 },
}

#[derive(Debug)]
pub(crate) enum WorldCreateRoleOutcome {
    Failed {
        request: WorldCreateRoleRequest,
        player_count: Option<u8>,
        stage: WorldCreateRoleFailureStage,
        status: i8,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Blocked {
        request: WorldCreateRoleRequest,
        source: WorldCreateRoleBlock,
    },
    Created {
        request: WorldCreateRoleRequest,
        player_count_before: u8,
        player_id: u32,
        defaults: PlayerDefaultPropertyReport,
        origin_goods: WorldOriginGoodsReport,
        snapshot: PlayerBaseWireSnapshot,
        response_type: i32,
        status: i8,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerSelectRequest {
    pub(crate) player_id: u32,
    pub(crate) account: Vec<u8>,
    pub(crate) client_ip: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerSelectValidationOwner {
    LiveMap,
    FrozenDbMap,
    PersistentDatabase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerSelectCloneOwner {
    LiveMap,
    FrozenDbMap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerSelectRejectReason {
    InvalidBinding,
    Deleted,
}

#[derive(Debug)]
pub(crate) enum WorldPlayerSelectBlock {
    Clone(PlayerCodecError),
    Route(WorldProcessPlayerDataQueueError),
    LoadRequest(WorldPlayerLoadRequestBlock),
}

#[derive(Debug)]
pub(crate) enum WorldPlayerSelectOutcome {
    Rejected {
        request: WorldPlayerSelectRequest,
        reason: WorldPlayerSelectRejectReason,
        response_type: i32,
        status: i8,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        log: Option<AddLogTextDisposition>,
    },
    Routed {
        request: WorldPlayerSelectRequest,
        validation_owner: WorldPlayerSelectValidationOwner,
        clone_owner: WorldPlayerSelectCloneOwner,
        route: WorldProcessPlayerDataQueueOutcome,
    },
    Queued {
        request: WorldPlayerSelectRequest,
        validation_owner: WorldPlayerSelectValidationOwner,
        queue: WorldPlayerLoadRequestOutcome,
    },
    Blocked {
        request: WorldPlayerSelectRequest,
        validation_owner: WorldPlayerSelectValidationOwner,
        source: WorldPlayerSelectBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerBaseOutcome {
    pub(crate) account: Vec<u8>,
    pub(crate) succeeded: bool,
    pub(crate) declared_count: Option<u8>,
    pub(crate) emitted_rows: i32,
    pub(crate) response_type: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerDeleteLogOutcome {
    pub(crate) player_name: Vec<u8>,
    pub(crate) ip_address: Vec<u8>,
    pub(crate) queue_length_after: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldDeleteRoleDisposition {
    OrganizingRejected {
        code: i32,
    },
    AlreadyScheduled {
        deletion_time: i32,
    },
    Scheduled {
        deletion_time: i32,
        restore_was_present: bool,
        deletion_days: i8,
        delete_log: Option<WorldPlayerDeleteLogOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldDeleteRoleOutcome {
    OrganizingBlocked {
        account: Vec<u8>,
        player_id: u32,
        ip_address: Vec<u8>,
        source: OrganizingDeleteRoleBlock,
    },
    Responded {
        account: Vec<u8>,
        player_id: u32,
        ip_address: Vec<u8>,
        organizing: OrganizingDeleteRoleOutcome,
        disposition: WorldDeleteRoleDisposition,
        response_type: i32,
        status: i8,
        trailing_value: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldAccountLoginCleanupOutcome {
    NotFound {
        account: Vec<u8>,
    },
    Cleaned {
        account: Vec<u8>,
        player_id: u32,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
        player_load_removed: bool,
        login_removed: bool,
        offline_inserted: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldAccountDisconnectOutcome {
    OnlineRouted {
        account: Vec<u8>,
        player_id: u32,
        game_server_index: u32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
    },
    LoginAcknowledged {
        account: Vec<u8>,
        released_player_id: Option<u32>,
        team_id: Option<i32>,
        team_session_id: Option<i32>,
        team_exit: Option<WorldLoginTimeoutTeamExit>,
        login_removed: bool,
        offline_inserted: bool,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerDetailOutcome {
    Rejected {
        player_id: u32,
        status: i32,
        source_socket_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    WrongGameServer {
        player_id: u32,
        owner_id: i32,
        source_map_id: i32,
        assigned_map_id: i32,
        source_socket_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        faction_data_reset: bool,
    },
    PromotedOnline {
        player_id: u32,
        owner_id: i32,
        source_map_id: i32,
        source_socket_id: i32,
        response_type: i32,
        encoded_bytes: usize,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        offline_removal_completed: bool,
        online_append: WorldOnlinePlayerAppendOutcome,
    },
    SerializationBlocked {
        player_id: u32,
        error: PlayerCodecError,
    },
    SerializationOwnerMissing {
        player_id: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldReturnedPlayerFriendOutcome {
    pub(crate) friend_player_id: u32,
    pub(crate) game_server_index: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerReturnOutcome {
    PlayerOnlyOnWorld { player_id: u32 },
    DecodeBlocked { player_id: u32, error: PlayerCodecError },
    PlayerMissing { player_id: u32, subtype: i32 },
    Released {
        player_id: u32,
        subtype: i32,
        decode: Option<WorldReturnedPlayerDecode>,
        login_wire: Vec<u8>,
        login_delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        team_exit: Option<WorldLoginTimeoutTeamExit>,
        friends: Vec<WorldReturnedPlayerFriendOutcome>,
    },
}

#[derive(Debug)]
pub(crate) enum WorldLogMessageOutcome {
 /// Default полного `OnLogMessage` без чтения и side effects.
    NoOp {
        request_type: i32,
    },
    PlayerBase(WorldPlayerBaseOutcome),
    DeleteRole(WorldDeleteRoleOutcome),
    RestoreRole(WorldRestoreRoleOutcome),
    CreateRole(WorldCreateRoleOutcome),
    PlayerSelect(WorldPlayerSelectOutcome),
    AccountLoginCleanup(WorldAccountLoginCleanupOutcome),
    AccountDisconnect(WorldAccountDisconnectOutcome),
    PlayerDetail(WorldPlayerDetailOutcome),
    PlayerReturn(WorldPlayerReturnOutcome),
}

pub(crate) enum WorldLogMessageDispatch {
    Handled(WorldLogMessageOutcome),
    Pending(CMessage),
}

pub(crate) async fn on_log_message(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    country_handler: &CCountryHandler,
    country_parameters: &mut CCountryParam,
    player_list: &mut CPlayerList,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    globe_setup: &GlobeSetupSnapshot,
    rs_player: &mut TiberiusRsPlayer,
    player_database: Option<&mut WorldTdsClient>,
    delete_log_enabled: bool,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    random: &mut dyn FnMut(i32) -> i32,
    message: CMessage,
) -> WorldLogMessageDispatch {
    match message.message_type() {
        PLAYER_BASE_REQUEST => {
            player_base(
                game,
                organizing,
                registry,
                coefficients,
                globe_setup,
                rs_player,
                player_database,
                message,
            )
            .await
        }
        DELETE_ROLE_REQUEST => {
            delete_role(
                game,
                organizing,
                organizing_parameters,
                country_handler,
                globe_setup,
                rs_player,
                player_database,
                delete_log_enabled,
                message,
            )
            .await
        }
        PLAYER_DETAIL_REQUEST => player_detail(
            game,
            organizing,
            registry,
            coefficients,
            add_error_log_text,
            message,
        ),
        PLAYER_RETURN_REQUEST => player_return(
            game,
            organizing,
            session_factory,
            registry,
            coefficients,
            add_error_log_text,
            message,
        ),
        RESTORE_ROLE_REQUEST => restore_role(game, message),
        CREATE_ROLE_REQUEST => {
            create_role(
                game,
                organizing,
                country_handler,
                country_parameters,
                player_list,
                registry,
                original_name_index,
                coefficients,
                globe_setup,
                rs_player,
                player_database,
                random,
                add_error_log_text,
                message,
            )
            .await
        }
        PLAYER_SELECT_REQUEST => {
            player_select(
                game,
                organizing,
                registry,
                coefficients,
                rs_player,
                player_database,
                load_player_largess,
                add_error_log_text,
                message,
            )
            .await
        }
        ACCOUNT_LOGIN_CLEANUP_REQUEST => {
            account_login_cleanup(game, session_factory, message)
        }
        ACCOUNT_DISCONNECT_REQUEST => account_disconnect(game, session_factory, message),
        request_type => WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::NoOp {
            request_type,
        }),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "exact select-player связывает live/save/DB owners и transport"
)]
async fn player_select(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let request = WorldPlayerSelectRequest {
        player_id: message.base_mut().get_long().unwrap_or(0) as u32,
        account: message
            .base_mut()
            .get_str_bytes(0x14)
            .unwrap_or_default(),
        client_ip: message.base_mut().get_long().unwrap_or(0) as u32,
    };

    let validation_owner = if game.validate_player_id_in_cdkey(
        &request.account,
        request.player_id,
    ) {
        Some(WorldPlayerSelectValidationOwner::LiveMap)
    } else if game.validate_db_player_id_in_cdkey(&request.account, request.player_id) {
        Some(WorldPlayerSelectValidationOwner::FrozenDbMap)
    } else if rs_player
        .validate_player_id_in_cdkey(
            &request.account,
            request.player_id,
            player_database.as_deref_mut(),
        )
        .await
    {
        Some(WorldPlayerSelectValidationOwner::PersistentDatabase)
    } else {
        None
    };

    let Some(validation_owner) = validation_owner else {
        let mut log_payload = format!(
            "==[L2W Invalid Request]== ID <{}> Not Bound To Cdkey <",
            request.player_id,
        )
        .into_bytes();
        let account_end = request
            .account
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(request.account.len());
        log_payload.extend_from_slice(&request.account[..account_end]);
        log_payload.extend_from_slice(b"> !");
        return send_player_select_rejection(
            game,
            request,
            WorldPlayerSelectRejectReason::InvalidBinding,
            Some(&log_payload),
            add_error_log_text,
        );
    };

    if !game.is_restore_player_exist(request.player_id) {
        let mut deletion_time = game.deletion_player_time(request.player_id);
        if deletion_time == 0 {
            deletion_time = rs_player
                .get_player_deletion_date(
                    request.player_id,
                    player_database.as_deref_mut(),
                )
                .await;
        }
        if deletion_time != 0 {
            return send_player_select_rejection(
                game,
                request,
                WorldPlayerSelectRejectReason::Deleted,
                None,
                add_error_log_text,
            );
        }
    }

    let (clone_owner, player) = match game.clone_map_player(
        request.player_id,
        registry,
        organizing,
        coefficients,
    ) {
        Ok(Some(player)) => (Some(WorldPlayerSelectCloneOwner::LiveMap), Some(player)),
        Ok(None) => match game.clone_saving_player(
            request.player_id,
            registry,
            organizing,
            coefficients,
        ) {
            Ok(Some(player)) => {
                (Some(WorldPlayerSelectCloneOwner::FrozenDbMap), Some(player))
            }
            Ok(None) => (None, None),
            Err(source) => {
                return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
                    WorldPlayerSelectOutcome::Blocked {
                        request,
                        validation_owner,
                        source: WorldPlayerSelectBlock::Clone(source),
                    },
                ));
            }
        },
        Err(source) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
                WorldPlayerSelectOutcome::Blocked {
                    request,
                    validation_owner,
                    source: WorldPlayerSelectBlock::Clone(source),
                },
            ));
        }
    };

    if let (Some(clone_owner), Some(player)) = (clone_owner, player) {
        let route = game.route_loaded_player(
            organizing,
            0,
            0,
            request.player_id,
            request.client_ip,
            &request.account,
            Some(player),
            WorldLoadedPlayerRouteOrder::Direct,
            load_player_largess,
            legacy_tick_ms,
        );
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
            match route {
                Ok(route) => WorldPlayerSelectOutcome::Routed {
                    request,
                    validation_owner,
                    clone_owner,
                    route,
                },
                Err(source) => WorldPlayerSelectOutcome::Blocked {
                    request,
                    validation_owner,
                    source: WorldPlayerSelectBlock::Route(source),
                },
            },
        ));
    }

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
        match game.push_player_load_request(
            &request.account,
            request.player_id,
            request.client_ip,
        ) {
            Ok(queue) => WorldPlayerSelectOutcome::Queued {
                request,
                validation_owner,
                queue,
            },
            Err(source) => WorldPlayerSelectOutcome::Blocked {
                request,
                validation_owner,
                source: WorldPlayerSelectBlock::LoadRequest(source),
            },
        },
    ))
}

fn send_player_select_rejection(
    game: &CGame,
    request: WorldPlayerSelectRequest,
    reason: WorldPlayerSelectRejectReason,
    log_payload: Option<&[u8]>,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
) -> WorldLogMessageDispatch {
    let mut response = CMessage::new(PLAYER_SELECT_RESPONSE);
    response.base_mut().add_char(PLAYER_SELECT_REJECTED_STATUS);
    append_c_string(response.base_mut(), &request.account);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    let log = log_payload.map(add_error_log_text);
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
        WorldPlayerSelectOutcome::Rejected {
            request,
            reason,
            response_type: PLAYER_SELECT_RESPONSE,
            status: PLAYER_SELECT_REJECTED_STATUS,
            wire,
            delivery,
            log,
        },
    ))
}

#[allow(
    clippy::too_many_arguments,
    reason = "exact create-role связывает setup, DB, organizing, goods и transport owners"
)]
async fn create_role(
    game: &mut CGame,
    organizing: &COrganizingCtrl,
    country_handler: &CCountryHandler,
    country_parameters: &mut CCountryParam,
    player_list: &mut CPlayerList,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    globe_setup: &GlobeSetupSnapshot,
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    random: &mut dyn FnMut(i32) -> i32,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let request = WorldCreateRoleRequest {
        name: message
            .base_mut()
            .get_str_bytes(0x32)
            .unwrap_or_default(),
        sex: message.base_mut().get_byte().unwrap_or(0),
        occupation: message.base_mut().get_byte().unwrap_or(0),
        head_picture: message.base_mut().get_byte().unwrap_or(0),
        face_picture: message.base_mut().get_byte().unwrap_or(0),
        country: message.base_mut().get_byte().unwrap_or(0),
        account: message
            .base_mut()
            .get_str_bytes(0x14)
            .unwrap_or_default(),
    };

    let creation_count = game.creation_player_count_in_cdkey(&request.account);
    let Some(player_count) = rs_player
        .get_player_count_in_cdkey(
            &request.account,
            creation_count,
            player_database.as_deref_mut(),
        )
        .await
    else {
        return send_create_role_failure(
            game,
            request,
            None,
            WorldCreateRoleFailureStage::DatabaseCount,
            CREATE_ROLE_INVALID_STATUS,
        );
    };
    if i16::from(player_count) >= globe_setup.maximum_characters() {
        return send_create_role_failure(
            game,
            request,
            Some(player_count),
            WorldCreateRoleFailureStage::CharacterLimit,
            CREATE_ROLE_LIMIT_STATUS,
        );
    }

    if !matches!(
        (request.sex, request.occupation),
        (0, 0) | (1, 1) | (2, 0)
    ) {
        return send_create_role_failure(
            game,
            request,
            Some(player_count),
            WorldCreateRoleFailureStage::OccupationSex,
            CREATE_ROLE_INVALID_STATUS,
        );
    }
    if country_handler.get_country(request.country).is_none() {
        return send_create_role_failure(
            game,
            request,
            Some(player_count),
            WorldCreateRoleFailureStage::Country,
            CREATE_ROLE_INVALID_STATUS,
        );
    }

    let mut checked_name = request.name.clone();
    if !game.check_create_role_name(&mut checked_name, false, true) {
        return send_create_role_failure(
            game,
            request,
            Some(player_count),
            WorldCreateRoleFailureStage::WordsFilter,
            CREATE_ROLE_FILTER_STATUS,
        );
    }

    let duplicate = match game.creation_player_by_name(&request.name) {
        Ok(found) => found.is_some(),
        Err(source) => {
            return create_role_blocked(request, WorldCreateRoleBlock::PlayerName(source));
        }
    };
    let duplicate = if duplicate {
        true
    } else {
        match game.is_name_exist_in_map_player(&request.name) {
            Ok(found) => found,
            Err(source) => {
                return create_role_blocked(request, WorldCreateRoleBlock::PlayerName(source));
            }
        }
    };
    let duplicate = if duplicate {
        true
    } else {
        match game.is_name_exist_in_db_creation(&request.name) {
            Ok(found) => found,
            Err(source) => {
                return create_role_blocked(request, WorldCreateRoleBlock::PlayerName(source));
            }
        }
    };
    let duplicate = if duplicate {
        true
    } else {
        match game.is_name_exist_in_db_data(&request.name) {
            Ok(found) => found,
            Err(source) => {
                return create_role_blocked(request, WorldCreateRoleBlock::PlayerName(source));
            }
        }
    };
    let duplicate = if duplicate {
        true
    } else {
        rs_player
            .is_name_exist(&request.name, player_database.as_deref_mut())
            .await
    };
    let duplicate = if duplicate {
        true
    } else {
        match game.is_name_exit_in_faction(organizing, &request.name) {
            Ok(found) => found,
            Err(source) => {
                return create_role_blocked(
                    request,
                    WorldCreateRoleBlock::OrganizingName(source),
                );
            }
        }
    };
    if duplicate {
        return send_create_role_failure(
            game,
            request,
            Some(player_count),
            WorldCreateRoleFailureStage::DuplicateName,
            CREATE_ROLE_DUPLICATE_STATUS,
        );
    }

    let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
    let defaults = match player.load_default_property(
        request.sex,
        request.occupation,
        request.country,
        country_parameters,
        game.dupli_region_setup(),
        player_list,
        globe_setup,
        game.thing_setup(),
        coefficients,
        |region_id| game.creation_region_base(region_id),
        random,
        || TagTime::local_now().day_of_week,
        legacy_time_seconds,
    ) {
        Ok(report) => report,
        Err(source) => {
            return create_role_blocked(
                request,
                WorldCreateRoleBlock::DefaultProperty(source),
            );
        }
    };
    player.set_creation_identity(
        &request.name,
        &request.account,
        request.head_picture,
        request.face_picture,
    );
    player.set_creation_service_defaults();
    let player_id = game.allocate_player_id();
    player.set_id(player_id);
    let origin_goods = match game.add_origin_goods_to_player(
        &mut player,
        player_list,
        registry,
        original_name_index,
        random,
    ) {
        Ok(report) => report,
        Err(source) => {
            return create_role_blocked(request, WorldCreateRoleBlock::OriginGoods(source));
        }
    };

    match game.append_creation_player(player, |entry| {
        let text = entry.to_string();
        let _ = add_log_text(text.as_bytes());
    }) {
        WorldCreationPlayerAppendOutcome::Inserted { .. } => {}
        WorldCreationPlayerAppendOutcome::DuplicateReleased { .. } => {
            return create_role_blocked(
                request,
                WorldCreateRoleBlock::AppendCollision(
                    WorldCreateRoleAppendCollision::DuplicateCreationId,
                ),
            );
        }
        WorldCreationPlayerAppendOutcome::ExistingMapOwnerKept { incoming, .. } => {
            drop(incoming);
            return create_role_blocked(
                request,
                WorldCreateRoleBlock::AppendCollision(
                    WorldCreateRoleAppendCollision::ExistingMapOwner,
                ),
            );
        }
    }

    let Some(created_player) = game.map_player(player_id as u32) else {
        return create_role_blocked(
            request,
            WorldCreateRoleBlock::PublishedPlayerMissing {
                player_id: player_id as u32,
            },
        );
    };
    let snapshot = match created_player.player_base_wire_snapshot() {
        Ok(snapshot) => snapshot,
        Err(source) => {
            return create_role_blocked(request, WorldCreateRoleBlock::Snapshot(source));
        }
    };

    let mut response = CMessage::new(CREATE_ROLE_RESPONSE);
    response.base_mut().add_char(CREATE_ROLE_SUCCESS_STATUS);
    append_c_string(response.base_mut(), &request.account);
    response.base_mut().add_ulong(snapshot.id as u32);
    append_c_string(response.base_mut(), &snapshot.name);
    response.base_mut().add_short(i16::from(snapshot.level));
    response.base_mut().add_byte(snapshot.sex);
    response.base_mut().add_byte(snapshot.occupation);
    response.base_mut().add_byte(snapshot.country);
    response.base_mut().add_byte(snapshot.head);
    for equipment_id in snapshot.equipment_ids {
        response.base_mut().add_ulong(equipment_id);
    }
    for equipment_level in snapshot.equipment_levels {
        response.base_mut().add_byte(equipment_level);
    }
    response.base_mut().add_long(snapshot.region_id);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::CreateRole(
        WorldCreateRoleOutcome::Created {
            request,
            player_count_before: player_count,
            player_id: player_id as u32,
            defaults,
            origin_goods,
            snapshot,
            response_type: CREATE_ROLE_RESPONSE,
            status: CREATE_ROLE_SUCCESS_STATUS,
            wire,
            delivery,
        },
    ))
}

fn send_create_role_failure(
    game: &CGame,
    request: WorldCreateRoleRequest,
    player_count: Option<u8>,
    stage: WorldCreateRoleFailureStage,
    status: i8,
) -> WorldLogMessageDispatch {
    let mut response = CMessage::new(CREATE_ROLE_RESPONSE);
    response.base_mut().add_char(status);
    append_c_string(response.base_mut(), &request.account);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::CreateRole(
        WorldCreateRoleOutcome::Failed {
            request,
            player_count,
            stage,
            status,
            response_type: CREATE_ROLE_RESPONSE,
            wire,
            delivery,
        },
    ))
}

fn create_role_blocked(
    request: WorldCreateRoleRequest,
    source: WorldCreateRoleBlock,
) -> WorldLogMessageDispatch {
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::CreateRole(
        WorldCreateRoleOutcome::Blocked { request, source },
    ))
}

fn append_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    let visible = value.iter().position(|byte| *byte == 0).unwrap_or(value.len());
    message.add(&value[..visible]);
    message.add_char(0);
}

fn legacy_time_seconds() -> u32 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as u32,
        Err(error) => 0_u32.wrapping_sub(error.duration().as_secs() as u32),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlayerBaseWireRow {
    player_id: u32,
    name: Vec<u8>,
    level: u8,
    occupation: u8,
    sex: u8,
    country: u8,
    head: u8,
    equipment_ids: [u32; 11],
    equipment_levels: [u8; 11],
    region_id: i32,
    deletion_status: i8,
}

impl PlayerBaseWireRow {
    fn from_database(row: PlayerBaseDatabaseRow, deletion_status: i8) -> Self {
        Self {
            player_id: row.id,
            name: row.name,
            level: row.level,
            occupation: row.occupation,
            sex: row.sex,
            country: row.country,
            head: row.head,
            equipment_ids: row.equipment_ids,
            equipment_levels: row.equipment_levels,
            region_id: row.region_id,
            deletion_status,
        }
    }

    fn from_snapshot(
        player_id: u32,
        snapshot: PlayerBaseWireSnapshot,
        deletion_status: i8,
    ) -> Self {
        Self {
            player_id,
            name: snapshot.name,
            level: snapshot.level,
            occupation: snapshot.occupation,
            sex: snapshot.sex,
            country: snapshot.country,
            head: snapshot.head,
            equipment_ids: snapshot.equipment_ids,
            equipment_levels: snapshot.equipment_levels,
            region_id: snapshot.region_id,
            deletion_status,
        }
    }

    fn append_to(self, row_index: i32, response: &mut CMessage) {
        response.base_mut().add_short(row_index as i16);
        response.base_mut().add_ulong(self.player_id);
        response.base_mut().add(c_string_prefix(&self.name));
        response.base_mut().add_char(0);
        response.base_mut().add_byte(self.level);
        response.base_mut().add_byte(self.occupation);
        response.base_mut().add_byte(self.sex);
        response.base_mut().add_byte(self.country);
        response.base_mut().add_byte(self.head);
        for equipment_id in self.equipment_ids {
            response.base_mut().add_ulong(equipment_id);
        }
        for equipment_level in self.equipment_levels {
            response.base_mut().add_byte(equipment_level);
        }
        response.base_mut().add_long(self.region_id);
        response.base_mut().add_char(self.deletion_status);
    }
}

fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn remaining_deletion_days(deletion_days: u32, deletion_time: i32) -> i8 {
    let now = chrono::Local::now().timestamp();
    let elapsed_seconds = now - i64::from(deletion_time);
    let elapsed_days = (elapsed_seconds as f64 / 86_400.0) as i32;
    let remaining = (deletion_days as u8 as i8).wrapping_sub(elapsed_days as u8 as i8);
    remaining.max(0)
}

struct DeleteRoleCountryEffects<'a> {
    game: &'a CGame,
    globe_setup: &'a GlobeSetupSnapshot,
}

impl CountryHasJobContext for DeleteRoleCountryEffects<'_> {
    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(value) => UnionFormatArgument::Text(value),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "границы один к одному соответствуют exact OnLogMessage/OnDeleteRole owner-ам"
)]
async fn delete_role(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    country_handler: &CCountryHandler,
    globe_setup: &GlobeSetupSnapshot,
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    delete_log_enabled: bool,
    mut request: CMessage,
) -> WorldLogMessageDispatch {
    let account = request
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let player_id = request.base_mut().get_long().unwrap_or(0) as u32;
    let ip_raw = request.base_mut().get_long().unwrap_or(0) as u32;
    let ip_address = format!(
        "{}.{}.{}.{}",
        ip_raw & 0xff,
        (ip_raw >> 8) & 0xff,
        (ip_raw >> 16) & 0xff,
        (ip_raw >> 24) & 0xff,
    )
    .into_bytes();
 // `_time` расположен до DB/organizing gates; в deletion-list
 // сохранялись младшие 32 бита Windows `long`.
    let deletion_time = chrono::Local::now().timestamp() as i32;

    let country = rs_player
        .get_player_country_by_id(player_id, player_database.as_deref_mut())
        .await;
    let country_has_job = if country == 0 {
        false
    } else if let Some(country_state) = country_handler.get_country(country) {
        let mut effects = DeleteRoleCountryEffects {
            game: &*game,
            globe_setup,
        };
        country_state.has_job(player_id as i32, &mut effects) != 0
    } else {
        false
    };
    let organizing_outcome = match organizing.on_delete_role(
        game,
        organizing_parameters,
        player_id as i32,
        country_has_job,
    ) {
        Ok(outcome) => outcome,
        Err(source) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::DeleteRole(
                WorldDeleteRoleOutcome::OrganizingBlocked {
                    account,
                    player_id,
                    ip_address,
                    source,
                },
            ));
        }
    };

    let organizing_code = organizing_outcome.legacy_code();
    if organizing_code != 0 {
        return send_delete_role_response(
            game,
            account,
            player_id,
            ip_address,
            organizing_outcome,
            WorldDeleteRoleDisposition::OrganizingRejected {
                code: organizing_code,
            },
            DELETE_ROLE_REJECTED_STATUS,
            organizing_code,
        );
    }

    let mut existing_deletion_time = game.deletion_player_time(player_id);
    if existing_deletion_time == 0 {
        existing_deletion_time = rs_player
            .get_player_deletion_date(player_id, player_database.as_deref_mut())
            .await;
    }
    if existing_deletion_time != 0 {
        return send_delete_role_response(
            game,
            account,
            player_id,
            ip_address,
            organizing_outcome,
            WorldDeleteRoleDisposition::AlreadyScheduled {
                deletion_time: existing_deletion_time,
            },
            DELETE_ROLE_REJECTED_STATUS,
            0,
        );
    }

    let restore_was_present = game.is_restore_player_exist(player_id);
    game.delete_restore_player(player_id);
    game.append_deletion_player(player_id, deletion_time);
    let deletion_days = globe_setup.deletion_days() as u8 as i8;
    let mut response = CMessage::new(DELETE_ROLE_RESPONSE);
    response.base_mut().add_char(DELETE_ROLE_SCHEDULED_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(c_string_prefix(&account));
    response.base_mut().add_char(0);
    response.base_mut().add_char(deletion_days);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );

 // optional log начинается только после постановки success-ответа в
 // LoginServer queue. Parameter binding заменяет `_sprintf`, не меняя FIFO.
    let delete_log = if delete_log_enabled {
        let player_name = rs_player
            .get_player_name_by_id(player_id, player_database.as_deref_mut())
            .await;
        let queue_length_after = game.push_write_log_command(
            WorldWriteLogCommand::PlayerDeleteLog(WorldPlayerDeleteLogWrite {
                player_id: player_id as i32,
                player_name: player_name.clone(),
                ip_address: ip_address.clone(),
            }),
        );
        Some(WorldPlayerDeleteLogOutcome {
            player_name,
            ip_address: ip_address.clone(),
            queue_length_after,
        })
    } else {
        None
    };

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::DeleteRole(
        WorldDeleteRoleOutcome::Responded {
            account,
            player_id,
            ip_address,
            organizing: organizing_outcome,
            disposition: WorldDeleteRoleDisposition::Scheduled {
                deletion_time,
                restore_was_present,
                deletion_days,
                delete_log,
            },
            response_type: DELETE_ROLE_RESPONSE,
            status: DELETE_ROLE_SCHEDULED_STATUS,
            trailing_value: i32::from(deletion_days),
            wire,
            delivery,
        },
    ))
}

#[allow(
    clippy::too_many_arguments,
    reason = "typed outcome сохраняет все наблюдаемые поля одного exact ответа"
)]
fn send_delete_role_response(
    game: &CGame,
    account: Vec<u8>,
    player_id: u32,
    ip_address: Vec<u8>,
    organizing: OrganizingDeleteRoleOutcome,
    disposition: WorldDeleteRoleDisposition,
    status: i8,
    trailing_value: i32,
) -> WorldLogMessageDispatch {
    let mut response = CMessage::new(DELETE_ROLE_RESPONSE);
    response.base_mut().add_char(status);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(c_string_prefix(&account));
    response.base_mut().add_char(0);
    response.base_mut().add_long(trailing_value);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::DeleteRole(
        WorldDeleteRoleOutcome::Responded {
            account,
            player_id,
            ip_address,
            organizing,
            disposition,
            response_type: DELETE_ROLE_RESPONSE,
            status,
            trailing_value,
            wire,
            delivery,
        },
    ))
}

fn send_player_base(
    game: &CGame,
    account: Vec<u8>,
    succeeded: bool,
    declared_count: Option<u8>,
    emitted_rows: i32,
    response: CMessage,
) -> WorldLogMessageDispatch {
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerBase(
        WorldPlayerBaseOutcome {
            account,
            succeeded,
            declared_count,
            emitted_rows,
            response_type: PLAYER_BASE_RESPONSE,
            wire,
            delivery,
        },
    ))
}

fn send_player_base_failure(
    game: &CGame,
    account: Vec<u8>,
    declared_count: Option<u8>,
) -> WorldLogMessageDispatch {
    let mut response = CMessage::new(PLAYER_BASE_RESPONSE);
    response.base_mut().add_char(0);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    send_player_base(game, account, false, declared_count, 0, response)
}

async fn player_base(
    game: &mut CGame,
    organizing: &COrganizingCtrl,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    globe_setup: &GlobeSetupSnapshot,
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    mut request: CMessage,
) -> WorldLogMessageDispatch {
    let account = request
        .base_mut()
        .get_str_bytes(0x14)
        .unwrap_or_default();
    let Some(database_count) = rs_player
        .get_player_count_in_db_by_cdkey(&account, player_database.as_deref_mut())
        .await
    else {
        return send_player_base_failure(game, account, None);
    };

    let creation_count = game.creation_player_count_in_cdkey(&account);
    let declared_count = database_count.wrapping_add(creation_count);
    let mut response = CMessage::new(PLAYER_BASE_RESPONSE);
    response.base_mut().add_char(1);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    response.base_mut().add_word(u16::from(declared_count));
    if declared_count == 0 {
        response.base_mut().add_char(1);
        response.base_mut().add(&account);
        response.base_mut().add_char(0);
        response.base_mut().add_long(0);
        return send_player_base(game, account, true, Some(0), 0, response);
    }

    let database_rows = match rs_player
        .open_player_base_in_db(&account, player_database.as_deref_mut())
        .await
    {
        Ok(rows) => rows,
        Err(_) => return send_player_base_failure(game, account, Some(declared_count)),
    };

    let mut row_index = 0_i32;
    for database_row in database_rows {
        let player_id = database_row.id;
        let deletion_status = if game.is_restore_player_exist(player_id) {
            -1
        } else {
            let mut deletion_time = game.deletion_player_time(player_id);
            if deletion_time == 0 {
                deletion_time = rs_player
                    .get_player_deletion_date(player_id, player_database.as_deref_mut())
                    .await;
            }
            if deletion_time == 0 {
                -1
            } else {
                remaining_deletion_days(globe_setup.deletion_days(), deletion_time)
            }
        };

        let runtime = match game.clone_map_player(
            player_id,
            registry,
            organizing,
            coefficients,
        ) {
            Ok(Some(player)) => match player.player_base_wire_snapshot() {
                Ok(snapshot) => Some(snapshot),
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Ok(None) => match game.clone_saving_player(
                player_id,
                registry,
                organizing,
                coefficients,
            ) {
                Ok(Some(player)) => match player.player_base_wire_snapshot() {
                    Ok(snapshot) => Some(snapshot),
                    Err(_) => {
                        return send_player_base_failure(game, account, Some(declared_count));
                    }
                },
                Ok(None) => None,
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Err(_) => return send_player_base_failure(game, account, Some(declared_count)),
        };
        let row = runtime.map_or_else(
            || PlayerBaseWireRow::from_database(database_row, deletion_status),
            |snapshot| {
                PlayerBaseWireRow::from_snapshot(player_id, snapshot, deletion_status)
            },
        );
        row.append_to(row_index, &mut response);
        row_index = row_index.wrapping_add(1);
    }

    let creation_player_ids = game.creation_player_ids_by_cdkey(&account);
    for player_id in creation_player_ids {
        let snapshot = match game.clone_map_player(
            player_id,
            registry,
            organizing,
            coefficients,
        ) {
            Ok(Some(player)) => match player.player_base_wire_snapshot() {
                Ok(snapshot) if snapshot.id != 0 => snapshot,
                Ok(_) => continue,
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Ok(None) => continue,
            Err(_) => return send_player_base_failure(game, account, Some(declared_count)),
        };
        let deletion_time = game.deletion_player_time(player_id);
        let deletion_status = if deletion_time == 0 {
            -1
        } else {
            remaining_deletion_days(globe_setup.deletion_days(), deletion_time)
        };
        PlayerBaseWireRow::from_snapshot(player_id, snapshot, deletion_status)
            .append_to(row_index, &mut response);
        row_index = row_index.wrapping_add(1);
    }

    send_player_base(
        game,
        account,
        true,
        Some(declared_count),
        row_index,
        response,
    )
}

fn player_return(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
    let subtype = message.base_mut().get_long().unwrap_or(0);
    let decode = if subtype == 1 {
        let decoded = {
            let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
            game.decord_returned_player(
                organizing,
                player_id,
                source,
                cursor,
                registry,
                coefficients,
            )
        };
        match decoded {
            Ok(decoded) => Some(decoded),
            Err(error) => {
                return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
                    WorldPlayerReturnOutcome::DecodeBlocked { player_id, error },
                ));
            }
        }
    } else {
        if subtype == 0 {
            let line = format!("Player {} Only On WorldServer!", player_id);
            let _ = add_error_log_text(line.as_bytes());
        }
        None
    };

    let Some(player) = game.returned_player_snapshot(player_id) else {
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
            if subtype == 0 {
                WorldPlayerReturnOutcome::PlayerOnlyOnWorld { player_id }
            } else {
                WorldPlayerReturnOutcome::PlayerMissing { player_id, subtype }
            },
        ));
    };

    let mut login = CMessage::new(ACCOUNT_DISCONNECT_LOGIN_RESPONSE);
    login.base_mut().add(&player.account);
    login.base_mut().add_char(0);
    login.base_mut().add(&player.name);
    login.base_mut().add_char(0);
    login.base_mut().add_byte(player.level);
    let login_wire = login.as_wire_bytes().to_vec();
    let login_delivery = login.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );

    let owner_id = player.owner_id as u32;
    let login_removed = game.remove_login_player(owner_id);
    let online_removal = game.remove_online_player(organizing, owner_id);
    let offline_inserted = game.append_offline_player_id(owner_id);
    let team_exit = if player.team_id == 0 {
        None
    } else {
        let session_id = game.get_team_session_id(player.team_id as u32);
        Some(game.exit_team_player(
            session_factory,
            session_id,
            player.owner_type,
            player.owner_id,
        ))
    };

    let mut friends = Vec::new();
    for friend_name in player.friend_names {
        let friend_player_id = game.online_player_id_by_name(&friend_name);
        if friend_player_id == 0 {
            continue;
        }
        let game_server_index = game.game_server_number_by_player_id(friend_player_id as i32);
        let mut presence = CMessage::new(PLAYER_FRIEND_OFFLINE_RESPONSE);
        presence.base_mut().add_ulong(friend_player_id);
        presence.base_mut().add(&player.name);
        presence.base_mut().add_char(0);
        let wire = presence.as_wire_bytes().to_vec();
        let delivery = game.send_msg_to_game_server(game_server_index, &presence);
        friends.push(WorldReturnedPlayerFriendOutcome {
            friend_player_id,
            game_server_index,
            wire,
            delivery,
        });
    }

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
        WorldPlayerReturnOutcome::Released {
            player_id,
            subtype,
            decode,
            login_wire,
            login_delivery,
            login_removed,
            online_removal,
            offline_inserted,
            team_exit,
            friends,
        },
    ))
}

fn player_detail(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
    let _ = message.base_mut().get_long();
    let _ = message.base_mut().get_long();

    let Some(player) = game.login_player_route_snapshot(player_id) else {
        let (status, error_text) = if game.online_player_by_id(player_id).is_some() {
            (
                -1,
                format!(
                    "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> Charactor Is Online!",
                    player_id
                ),
            )
        } else {
            (
                0,
                format!(
                    "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> Charactor Is Offline",
                    player_id
                ),
            )
        };
        let (wire, delivery) = send_player_detail_status(
            game,
            source_socket_id,
            status,
            player_id,
        );
        let _ = add_error_log_text(error_text.as_bytes());
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
            WorldPlayerDetailOutcome::Rejected {
                player_id,
                status,
                source_socket_id,
                response_type: PLAYER_DETAIL_RESPONSE,
                wire,
                delivery,
            },
        ));
    };

    let assigned_map_id = game.game_server_number_by_region_id(player.region_id);
    if assigned_map_id != source_map_id {
        let (wire, delivery) = send_player_detail_status(
            game,
            source_socket_id,
            -2,
            player_id,
        );
        let owner_id = player.owner_id as u32;
        let login_removed = game.remove_login_player(owner_id);
        let online_removal = game.remove_online_player(organizing, owner_id);
        let offline_inserted = game.append_offline_player_id(owner_id);
        let faction_data_reset = game.reset_map_player_faction_data(player.map_key);
        let error_text = format!(
            "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> GameServer Invalid",
            player_id
        );
        let _ = add_error_log_text(error_text.as_bytes());
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
            WorldPlayerDetailOutcome::WrongGameServer {
                player_id,
                owner_id: player.owner_id,
                source_map_id,
                assigned_map_id,
                source_socket_id,
                response_type: PLAYER_DETAIL_RESPONSE,
                wire,
                delivery,
                login_removed,
                online_removal,
                offline_inserted,
                faction_data_reset,
            },
        ));
    }

    message.set_message_type(PLAYER_DETAIL_RESPONSE);
    let encoded = match game.encode_map_player_full_snapshot(
        organizing,
        player.map_key,
        registry,
        coefficients,
    ) {
        Ok(Some(encoded)) => encoded,
        Ok(None) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                WorldPlayerDetailOutcome::SerializationOwnerMissing { player_id },
            ));
        }
        Err(error) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                WorldPlayerDetailOutcome::SerializationBlocked { player_id, error },
            ));
        }
    };
    let encoded_bytes = encoded.len();
    message.base_mut().add(&encoded);
    let wire = message.as_wire_bytes().to_vec();
    let sender = game.current_game_server_sender();
    let delivery = message.send_to_socket(sender.as_ref(), source_socket_id);
    let owner_id = player.owner_id as u32;
    let login_removed = game.remove_login_player(owner_id);
    game.remove_offline_player(owner_id);
    let online_append = game.append_online_player_id(organizing, player.owner_id);

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
        WorldPlayerDetailOutcome::PromotedOnline {
            player_id,
            owner_id: player.owner_id,
            source_map_id,
            source_socket_id,
            response_type: PLAYER_DETAIL_RESPONSE,
            encoded_bytes,
            wire,
            delivery,
            login_removed,
            offline_removal_completed: true,
            online_append,
        },
    ))
}

fn send_player_detail_status(
    game: &CGame,
    socket_id: i32,
    status: i32,
    player_id: u32,
) -> (Vec<u8>, Result<i32, SendMessageError>) {
    let mut response = CMessage::new(PLAYER_DETAIL_RESPONSE);
    response.base_mut().add_long(status);
    response.base_mut().add_ulong(player_id);
    let wire = response.as_wire_bytes().to_vec();
    let sender = game.current_game_server_sender();
    let delivery = response.send_to_socket(sender.as_ref(), socket_id);
    (wire, delivery)
}

fn restore_role(game: &mut CGame, mut message: CMessage) -> WorldLogMessageDispatch {
    let account = message
        .base_mut()
        .get_str_bytes(0x14)
        .expect("literal 0x14 исключает zero-capacity GetStr");
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0) as u32;

    game.delete_deletion_player(player_id);
    game.append_restore_player(player_id);

    let mut response = CMessage::new(RESTORE_ROLE_RESPONSE);
    response.base_mut().add_char(RESTORE_ROLE_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::RestoreRole(
        WorldRestoreRoleOutcome {
            account,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            response_type: RESTORE_ROLE_RESPONSE,
            status: RESTORE_ROLE_STATUS,
            wire,
            delivery,
        },
    ))
}

fn account_login_cleanup(
    game: &mut CGame,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let Some(player) = game.login_player_by_account(&account) else {
        return WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountLoginCleanup(
                WorldAccountLoginCleanupOutcome::NotFound { account },
            ),
        );
    };

    let team_session_id = game.get_team_session_id(player.team_id as u32);
    let team_exit = game.exit_team_player(
        session_factory,
        team_session_id,
        player.owner_type,
        player.owner_id,
    );
    let player_id = player.owner_id as u32;
    let player_load_removed = game.remove_player_load_data(player.owner_id);
    let login_removed = game.remove_login_player(player_id);
    let offline_inserted = game.append_offline_player_id(player_id);

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::AccountLoginCleanup(
        WorldAccountLoginCleanupOutcome::Cleaned {
            account,
            player_id,
            team_id: player.team_id,
            team_session_id,
            team_exit,
            player_load_removed,
            login_removed,
            offline_inserted,
        },
    ))
}

fn account_disconnect(
    game: &mut CGame,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");

    if let Some(player) = game.online_player_route_by_account(&account) {
        let player_id = player.owner_id as u32;
        let mut response = CMessage::new(ACCOUNT_DISCONNECT_GAME_RESPONSE);
        response.base_mut().add_ulong(player_id);
        let wire = response.as_wire_bytes().to_vec();
        let delivery = game.send_msg_to_game_server(
            player.game_server_index as i32,
            &response,
        );
        let team_session_id = game.get_team_session_id(player.team_id as u32);
        let team_exit = game.exit_team_player(
            session_factory,
            team_session_id,
            player.owner_type,
            player.owner_id,
        );
        return WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountDisconnect(
                WorldAccountDisconnectOutcome::OnlineRouted {
                    account,
                    player_id,
                    game_server_index: player.game_server_index,
                    response_type: ACCOUNT_DISCONNECT_GAME_RESPONSE,
                    wire,
                    delivery,
                    team_id: player.team_id,
                    team_session_id,
                    team_exit,
                },
            ),
        );
    }

    let mut released_player_id = None;
    let mut team_id = None;
    let mut team_session_id = None;
    let mut team_exit = None;
    let mut login_removed = false;
    let mut offline_inserted = false;
    if let Some(player) = game.login_player_by_account(&account) {
        let session_id = game.get_team_session_id(player.team_id as u32);
        let exit = game.exit_team_player(
            session_factory,
            session_id,
            player.owner_type,
            player.owner_id,
        );
        let player_id = player.owner_id as u32;
        login_removed = game.remove_login_player(player_id);
        offline_inserted = game.append_offline_player_id(player_id);
        released_player_id = Some(player_id);
        team_id = Some(player.team_id);
        team_session_id = Some(session_id);
        team_exit = Some(exit);
    }

    let mut response = CMessage::new(ACCOUNT_DISCONNECT_LOGIN_RESPONSE);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    response.base_mut().add_char(0);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::AccountDisconnect(
        WorldAccountDisconnectOutcome::LoginAcknowledged {
            account,
            released_player_id,
            team_id,
            team_session_id,
            team_exit,
            login_removed,
            offline_inserted,
            response_type: ACCOUNT_DISCONNECT_LOGIN_RESPONSE,
            wire,
            delivery,
        },
    ))
}
