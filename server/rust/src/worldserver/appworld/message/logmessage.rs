//! Login/player lifecycle `OnLogMessage` из `logmessage.cpp`, сопоставленный
//! с парой `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Ветки `0x4FB01..0x4FB07` и `0x5FB01..0x5FB02` сохраняют переходы
//! login/offline/online, create/delete/restore/select и account cleanup.
//! Online player отправляется GameServer до изменения списков; offline snapshot
//! отвечает LoginServer до списков и уведомлений друзей.
//!
//! Player list объединяет DB rows перед creation rows и накладывает live/save
//! состояние. Delete-role снимает время до faction/DB gates; restore/deletion
//! списки меняются в исходном порядке. Create-role выполняет limit, RU sex/
//! occupation, country, filter и name checks до выдачи ID и equipment.
//!
//! Select сначала проверяет live map, frozen save-map и лишь затем DB. При miss
//! ставится player-load FIFO; direct clone публикует Largess/login/map до friends
//! и сброса flags. Tiberius и owned snapshots заменяют ADO/STL без перестановки.

use crate::dbaccess::worlddb::rsplayer::{
    RsPlayerOwner, TiberiusRsPlayer,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::setup::globesetup::GlobeSetupSnapshot;
use nebokrai_shared::resources::CPlayerList;
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::country::country::{
    CountryExileTextArgument, CountryHasJobContext,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::union::UnionFormatArgument;
use crate::worldserver::appworld::player::{CPlayer, PlayerCodecError, PlayerPropertyCoefficients};
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::worldserver::game::{
    CGame, WorldLoadedPlayerRouteOrder, WorldLoginTimeoutTeamExit,
    WorldOnlinePlayerAppendOutcome, WorldOnlinePlayerRemoveOutcome, WorldPlayerLoadRequestBlock,
    WorldPlayerLoadRequestOutcome, WorldProcessPlayerDataQueueError,
    WorldProcessPlayerDataQueueOutcome, WorldReturnedPlayerDecode, legacy_tick_ms,
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
const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
const RESTORE_ROLE_STATUS: i8 = 0x15;
const PLAYER_SELECT_RESPONSE: i32 = 0x0001_FF01;
const PLAYER_SELECT_REJECTED_STATUS: i8 = 0x1C;
const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;
const PLAYER_DETAIL_RESPONSE: i32 = 0x0007_F901;
const PLAYER_FRIEND_OFFLINE_RESPONSE: i32 = 0x0007_F905;

pub(crate) use nebokrai_realm::app::logmessage::{WorldCreateRoleOutcome, WorldRestoreRoleOutcome};

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

pub(crate) use nebokrai_realm::app::player_base::WorldPlayerBaseOutcome;
pub(crate) use nebokrai_realm::app::logmessage::{
    WorldAccountDisconnectOutcome, WorldAccountLoginCleanupOutcome, WorldDeleteRoleOutcome,
};

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
        PLAYER_BASE_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::PlayerBase(
                nebokrai_realm::app::player_base::on_player_base(
                    game,
                    organizing,
                    registry,
                    coefficients,
                    globe_setup,
                    rs_player,
                    player_database,
                    message,
                )
                .await,
            ),
        ),
        DELETE_ROLE_REQUEST => {
            let mut country_gate = DeleteRoleCountryGateBridge {
                country_handler,
                globe_setup,
            };
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::DeleteRole(
                nebokrai_realm::app::logmessage::on_delete_role(
                    game,
                    organizing,
                    &mut country_gate,
                    organizing_parameters,
                    globe_setup,
                    rs_player,
                    player_database,
                    delete_log_enabled,
                    message,
                )
                .await,
            ))
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
        RESTORE_ROLE_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::RestoreRole(
                nebokrai_realm::app::logmessage::on_restore_role(game, message),
            ))
        }
        CREATE_ROLE_REQUEST => {
            let country_view_adapter = CreateRoleCountryViewAdapter {
                handler: country_handler,
            };
            let organizing_view_adapter = CreateRoleOrganizingViewAdapter { organizing };
            let mut log_wrapper = |bytes: &[u8]| {
                let _ = add_error_log_text(bytes);
            };
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::CreateRole(
                nebokrai_realm::app::logmessage::on_create_role(
                    game,
                    &organizing_view_adapter,
                    &country_view_adapter,
                    rs_player,
                    player_list,
                    registry,
                    original_name_index,
                    country_parameters,
                    coefficients,
                    globe_setup,
                    player_database,
                    random,
                    &mut log_wrapper,
                    message,
                )
                .await,
            ))
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
        ACCOUNT_LOGIN_CLEANUP_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountLoginCleanup(
                nebokrai_realm::app::logmessage::on_account_login_cleanup(
                    game, session_factory, message,
                ),
            ),
        ),
        ACCOUNT_DISCONNECT_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountDisconnect(
                nebokrai_realm::app::logmessage::on_account_disconnect(
                    game, session_factory, message,
                ),
            ),
        ),
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


fn append_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    let visible = value.iter().position(|byte| *byte == 0).unwrap_or(value.len());
    message.add(&value[..visible]);
    message.add_char(0);
}


struct DeleteRoleCountryEffects<'a> {
    game: &'a (dyn nebokrai_realm::app::world_game_view::WorldGameView + 'a),
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

/// Адаптер странового gate ветви delete-role: повторяет исходную цепочку
/// `country_handler.get_country → country_state.has_job` буквально, игра
/// приходит через короткий перезайм view от обработчика.
struct DeleteRoleCountryGateBridge<'a> {
    country_handler: &'a CCountryHandler,
    globe_setup: &'a GlobeSetupSnapshot,
}

impl nebokrai_realm::app::world_game_view::WorldDeleteRoleCountryGate
    for DeleteRoleCountryGateBridge<'_>
{
    fn country_has_job(
        &mut self,
        game: &dyn nebokrai_realm::app::world_game_view::WorldGameView,
        country: u8,
        player_id: i32,
    ) -> bool {
        let Some(country_state) = self.country_handler.get_country(country) else {
            return false;
        };
        let mut effects = DeleteRoleCountryEffects {
            game,
            globe_setup: self.globe_setup,
        };
        country_state.has_job(player_id, &mut effects) != 0
    }
}

/// Адаптер страновой таблицы create-role: повторяет `get_country(...).is_some()`
/// прежнего обработчика без переноса самого `CCountryHandler`.
struct CreateRoleCountryViewAdapter<'a> {
    handler: &'a CCountryHandler,
}

impl nebokrai_realm::app::world_game_view::WorldCountryView for CreateRoleCountryViewAdapter<'_> {
    fn country_exists(&self, country: u8) -> bool {
        self.handler.get_country(country).is_some()
    }
}

/// Адаптер организационного lookup-а create-role: делегирует
/// `COrganizingCtrl::organizing_by_name(...).map(|m| m.is_some())` с тем же
/// отклонением технического дефекта в seam-блок.
struct CreateRoleOrganizingViewAdapter<'a> {
    organizing: &'a COrganizingCtrl,
}

impl nebokrai_realm::app::world_game_view::WorldCreateRoleOrganizingView
    for CreateRoleOrganizingViewAdapter<'_>
{
    fn name_exists(
        &self,
        name: &[u8],
    ) -> Result<bool, nebokrai_realm::app::world_game_view::WorldCreateRoleOrganizingLookupBlock>
    {
        match self.organizing.organizing_by_name(name) {
            Ok(matched) => Ok(matched.is_some()),
            Err(_source) => Err(
                nebokrai_realm::app::world_game_view::WorldCreateRoleOrganizingLookupBlock::NullOwner,
            ),
        }
    }
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
