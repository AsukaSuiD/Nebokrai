//! Login lifecycle `OnLogMessage` из `logmessage.cpp`, подтверждённый
//! `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Ветви диспетчера `OnLogMessage`: списки восстановления и
//! удаления, вход и отключение аккаунта изменяются в исходном порядке до
//! wire-ответа LoginServer; форма payload не меняется. Create-role выполняет
//! limit, sex/occupation, country, filter и name checks до выдачи ID и
//! equipment. Select сначала проверяет live map, frozen save-map и лишь затем
//! DB; direct-маршрут клона остаётся одним вызовом у владельца игры.
//! Player-detail фиксирует промах login-маршрута статусом до мутаций,
//! несовпадение map переводит игрока в offline, а совпавший маршрут кодирует
//! полный снимок до повторной публикации online. Player-return декодирует
//! subtype-`1`, отвечает LoginServer и снимает login/online до уведомления
//! друзей. Список персонажей `0x4FB01` живёт в соседнем `player_base.rs`.
//! Здесь же сам match-диспетчер `OnLogMessage`: маршрут Log-сообщений идёт
//! из единственного callsite-а `process_world_message` через dyn-швы.

use nebokrai_shared::resources::GlobeSetupSnapshot;

use crate::app::player_base::WorldPlayerBaseOutcome;
use crate::app::world_game_view::{
    WorldCountryView, WorldCreateRoleDbView, WorldCreateRoleLaunchFailure,
    WorldCreateRoleLaunchGate, WorldCreateRoleOrganizingView, WorldDeleteRoleCountryGate,
    WorldDeleteRoleDbView, WorldGameView, WorldLoginTimeoutTeamExit, WorldPlayerDetailGameView,
    WorldPlayerLoadRequestBlock, WorldPlayerLoadRequestOutcome, WorldPlayerReturnGameView,
    WorldPlayerSelectDbView, WorldPlayerSelectGameView, WorldPlayerSelectRouteError,
    WorldPlayerSelectRouteOutcome, WorldReturnedPlayerDecode,
};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::player::{
    CPlayer, PlayerBaseWireSnapshot, PlayerCodecError, PlayerPropertyCoefficients,
};
use crate::content::countryparam::CCountryParam;
use crate::content::cgoodsfactory::GoodsOriginalNameIndex;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::organizations::organizingctrl::{
    OrganizingDeleteRoleBlock, OrganizingDeleteRoleOutcome, WorldDeleteRoleOrganizingGate,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::persistence::rsplayer::RsPlayerOwner;
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::writelog::{WorldPlayerDeleteLogWrite, WorldWriteLogCommand};
use crate::sessions::csessionfactory::CSessionFactory;
use nebokrai_shared::resources::CPlayerList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCreateRoleRequest {
    pub name: Vec<u8>,
    pub sex: u8,
    pub occupation: u8,
    pub head_picture: u8,
    pub face_picture: u8,
    pub country: u8,
    pub account: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCreateRoleFailureStage {
    DatabaseCount,
    CharacterLimit,
    OccupationSex,
    Country,
    WordsFilter,
    DuplicateName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCreateRoleAppendCollision {
    DuplicateCreationId,
    ExistingMapOwner,
}

pub const PLAYER_BASE_REQUEST: i32 = 0x0004_FB01;
pub const CREATE_ROLE_REQUEST: i32 = 0x0004_FB04;
pub const CREATE_ROLE_RESPONSE: i32 = 0x0001_FF03;
pub const CREATE_ROLE_INVALID_STATUS: i8 = 0x01;
pub const CREATE_ROLE_LIMIT_STATUS: i8 = 0x02;
pub const CREATE_ROLE_FILTER_STATUS: i8 = 0x03;
pub const CREATE_ROLE_DUPLICATE_STATUS: i8 = 0x05;
pub const CREATE_ROLE_SUCCESS_STATUS: i8 = 0x00;

pub const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
pub const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
pub const RESTORE_ROLE_STATUS: i8 = 0x15;
pub const ACCOUNT_LOGIN_CLEANUP_REQUEST: i32 = 0x0004_FB06;
pub const ACCOUNT_DISCONNECT_REQUEST: i32 = 0x0004_FB07;
pub const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
pub const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;
pub const PLAYER_SELECT_REQUEST: i32 = 0x0004_FB05;
pub const PLAYER_SELECT_RESPONSE: i32 = 0x0001_FF01;
pub const PLAYER_SELECT_REJECTED_STATUS: i8 = 0x1C;
pub const PLAYER_DETAIL_REQUEST: i32 = 0x0005_FB01;
pub const PLAYER_RETURN_REQUEST: i32 = 0x0005_FB02;
pub const PLAYER_DETAIL_RESPONSE: i32 = 0x0007_F901;
pub const PLAYER_FRIEND_OFFLINE_RESPONSE: i32 = 0x0007_F905;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRestoreRoleOutcome {
    pub account: Vec<u8>,
    pub player_id: u32,
    pub player_id_complete: bool,
    pub response_type: i32,
    pub status: i8,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldAccountLoginCleanupOutcome {
    NotFound { account: Vec<u8> },
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
pub enum WorldAccountDisconnectOutcome {
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

/// Ветвь `0x4FB03` без DB и без cross-owner: сплыть списки в хранении до
/// wire-ответа с конечным owner-полем пакета без изменения формы payload.
pub fn on_restore_role(
    game: &mut dyn WorldGameView,
    mut message: CMessage,
) -> WorldRestoreRoleOutcome {
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
    WorldRestoreRoleOutcome {
        account,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        response_type: RESTORE_ROLE_RESPONSE,
        status: RESTORE_ROLE_STATUS,
        wire,
        delivery,
    }
}

/// Ветвь `0x4FB06`: выход из команды и снятие отметок выполняются после
/// lookup по account в исходном порядке мутаций.
pub fn on_account_login_cleanup(
    game: &mut dyn WorldGameView,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldAccountLoginCleanupOutcome {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let Some(player) = game.login_player_by_account(&account) else {
        return WorldAccountLoginCleanupOutcome::NotFound { account };
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

    WorldAccountLoginCleanupOutcome::Cleaned {
        account,
        player_id,
        team_id: player.team_id,
        team_session_id,
        team_exit,
        player_load_removed,
        login_removed,
        offline_inserted,
    }
}

/// Ветвь `0x4FB07`: два маршрута завершения — online-игроку отправляется
/// пакет на его GameServer, затем оба маршрута снимают login-метку до ack
/// LoginServer.
pub fn on_account_disconnect(
    game: &mut dyn WorldGameView,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldAccountDisconnectOutcome {
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
        return WorldAccountDisconnectOutcome::OnlineRouted {
            account,
            player_id,
            game_server_index: player.game_server_index,
            response_type: ACCOUNT_DISCONNECT_GAME_RESPONSE,
            wire,
            delivery,
            team_id: player.team_id,
            team_session_id,
            team_exit,
        };
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
    }
}

pub const DELETE_ROLE_REQUEST: i32 = 0x0004_FB02;
pub const DELETE_ROLE_RESPONSE: i32 = 0x0001_FF03;
pub const DELETE_ROLE_REJECTED_STATUS: i8 = 0x13;
pub const DELETE_ROLE_SCHEDULED_STATUS: i8 = 0x14;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerDeleteLogOutcome {
    pub player_name: Vec<u8>,
    pub ip_address: Vec<u8>,
    pub queue_length_after: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldDeleteRoleDisposition {
    OrganizingRejected { code: i32 },
    AlreadyScheduled { deletion_time: i32 },
    Scheduled {
        deletion_time: i32,
        restore_was_present: bool,
        deletion_days: i8,
        delete_log: Option<WorldPlayerDeleteLogOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldDeleteRoleOutcome {
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

fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

/// Ветвь `0x4FB02`: `_time` снимается до DB/organizing gates; organizing
/// gate и страновый has_job проходят узкими dyn-швами, три DB-запроса —
/// boxed future по ADR-0013; журнал удаления ставится в FIFO только после
/// success-ответа в очередь LoginServer.
#[allow(clippy::too_many_arguments, reason = "границы один к одному соответствуют owner-ам ветки delete-role")]
pub async fn on_delete_role(
    game: &mut dyn WorldGameView,
    organizing: &mut dyn WorldDeleteRoleOrganizingGate,
    country_gate: &mut dyn WorldDeleteRoleCountryGate,
    organizing_parameters: &COrganizingParam,
    globe_setup: &GlobeSetupSnapshot,
    db: &mut dyn WorldDeleteRoleDbView,
    mut player_database: Option<&mut WorldTdsClient>,
    delete_log_enabled: bool,
    mut request: CMessage,
) -> WorldDeleteRoleOutcome {
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

    let country = db
        .get_player_country_by_id(player_id, player_database.as_deref_mut())
        .await;
    let country_has_job = if country == 0 {
        false
    } else {
        country_gate.country_has_job(&*game, country, player_id as i32)
    };
    let organizing_outcome = match organizing.apply_delete_role(
        &*game,
        organizing_parameters,
        player_id as i32,
        country_has_job,
    ) {
        Ok(outcome) => outcome,
        Err(source) => {
            return WorldDeleteRoleOutcome::OrganizingBlocked {
                account,
                player_id,
                ip_address,
                source,
            };
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
        existing_deletion_time = db
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
        let player_name = db
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
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "typed outcome сохраняет все наблюдаемые поля одного exact ответа"
)]
fn send_delete_role_response(
    game: &mut dyn WorldGameView,
    account: Vec<u8>,
    player_id: u32,
    ip_address: Vec<u8>,
    organizing: OrganizingDeleteRoleOutcome,
    disposition: WorldDeleteRoleDisposition,
    status: i8,
    trailing_value: i32,
) -> WorldDeleteRoleOutcome {
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
    }
}

fn send_create_role_failure<G: WorldGameView + ?Sized>(
    game: &mut G,
    request: WorldCreateRoleRequest,
    player_count: Option<u8>,
    stage: WorldCreateRoleFailureStage,
    status: i8,
) -> WorldCreateRoleOutcome {
    let mut response = CMessage::new(CREATE_ROLE_RESPONSE);
    response.base_mut().add_char(status);
    {
        let account = &request.account;
        let visible = account.iter().position(|byte| *byte == 0).unwrap_or(account.len());
        response.base_mut().add(&account[..visible]);
        response.base_mut().add_char(0);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldCreateRoleOutcome::Failed {
        request,
        player_count,
        stage,
        status,
        response_type: CREATE_ROLE_RESPONSE,
        wire,
        delivery,
    }
}

/// Конечный outcome ветки создания роли. Failed повторяет исходные статусы
/// `0x01..0x05`; Blocked идентичен изоляции технического дефекта до любого
/// ответа LoginServer.
#[derive(Debug)]
pub enum WorldCreateRoleOutcome {
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
        source: WorldCreateRoleLaunchFailure,
    },
    Created {
        request: WorldCreateRoleRequest,
        player_count_before: u8,
        player_id: u32,
        snapshot: PlayerBaseWireSnapshot,
        response_type: i32,
        status: i8,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Ветвь `0x4FB04`: полная цепочка проверки до выделения ID и launch-а.
/// Порядок именно src: сначала счётчик DB+creation, лимит `maximum_characters`,
/// sex/occupation пара и страна через game-view, фильтр слов, шесть проверок
/// занятости имени в точном порядке (map → db-creation → db-data → db-exist
/// через DB-шов → организация через launch-gate), затем launch владельца игры
/// и ответ со снимком нового игрока. Box-alloc `CPlayer` и всю мутацию карты
/// и очереди выполняет launch-gate; обработчик не владеет setup-таблицами,
/// кроме тех что уже в репо realm (player_list, registry, original_name_index,
/// country_parameters, coefficients, globe_setup).
#[allow(clippy::too_many_arguments, reason = "границы один к одному соответствуют owner-ам ветки create-role")]
pub async fn on_create_role<G: WorldGameView + WorldCreateRoleLaunchGate>(
    game: &mut G,
    organizing_view: &dyn WorldCreateRoleOrganizingView,
    country_view: &dyn WorldCountryView,
    db: &mut dyn WorldCreateRoleDbView,
    player_list: &mut CPlayerList,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    country_parameters: &mut CCountryParam,
    coefficients: &PlayerPropertyCoefficients,
    globe_setup: &GlobeSetupSnapshot,
    mut player_database: Option<&mut WorldTdsClient>,
    random: &mut dyn FnMut(i32) -> i32,
    add_log_text: &mut dyn FnMut(&[u8]),
    mut message: CMessage,
) -> WorldCreateRoleOutcome {
    let name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
    let sex = message.base_mut().get_byte().unwrap_or(0);
    let occupation = message.base_mut().get_byte().unwrap_or(0);
    let head_picture = message.base_mut().get_byte().unwrap_or(0);
    let face_picture = message.base_mut().get_byte().unwrap_or(0);
    let country = message.base_mut().get_byte().unwrap_or(0);
    let account = message.base_mut().get_str_bytes(0x14).unwrap_or_default();
    let request = WorldCreateRoleRequest {
        name: name.clone(),
        sex,
        occupation,
        head_picture,
        face_picture,
        country,
        account: account.clone(),
    };

    // Счётчик creation+DB ≤ maximum_characters из live globe.
    let creation_count = game.creation_player_count_in_cdkey(&account);
    let Some(player_count) = db
        .get_player_count_in_cdkey(&account, creation_count, player_database.as_deref_mut())
        .await
    else {
        return send_create_role_failure(game, request, None, WorldCreateRoleFailureStage::DatabaseCount, CREATE_ROLE_INVALID_STATUS);
    };
    if i16::from(player_count) >= globe_setup.maximum_characters() {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::CharacterLimit, CREATE_ROLE_LIMIT_STATUS);
    }

    // Исходная формула допустимых (sex, occupation) копирована буквально.
    if !matches!((sex, occupation), (0, 0) | (1, 1) | (2, 0)) {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::OccupationSex, CREATE_ROLE_INVALID_STATUS);
    }

    if !country_view.country_exists(country) {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::Country, CREATE_ROLE_INVALID_STATUS);
    }

    // Фильтр слов работает на клоне имени, а его результат отбрасывается —
    // как в исходном обработчике (`checked_name`): дальнейшие проверки
    // занятости и launch используют исходное прочитанное имя без подмены.
    let mut checked_name = name.clone();
    if !game.check_create_role_name(&mut checked_name, false, true) {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::WordsFilter, CREATE_ROLE_FILTER_STATUS);
    }

    // Шесть ступеней занятости имени, в исходном порядке и с той же изоляцией
    // технического дефекта как Blocked до любого ответа.
    let duplicate = match game.creation_player_by_name(&name) {
        Ok(found) => found,
        Err(source) => {
            return WorldCreateRoleOutcome::Blocked {
                request,
                source: WorldCreateRoleLaunchFailure::PlayerName(source),
            };
        }
    };
    if duplicate {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
    }

    let duplicate = match game.is_name_exist_in_map_player(&name) {
        Ok(found) => found,
        Err(source) => {
            return WorldCreateRoleOutcome::Blocked {
                request,
                source: WorldCreateRoleLaunchFailure::PlayerName(source),
            };
        }
    };
    if duplicate {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
    }

    let duplicate = match game.is_name_exist_in_db_creation(&name) {
        Ok(found) => found,
        Err(source) => {
            return WorldCreateRoleOutcome::Blocked {
                request,
                source: WorldCreateRoleLaunchFailure::PlayerName(source),
            };
        }
    };
    if duplicate {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
    }

    let duplicate = match game.is_name_exist_in_db_data(&name) {
        Ok(found) => found,
        Err(source) => {
            return WorldCreateRoleOutcome::Blocked {
                request,
                source: WorldCreateRoleLaunchFailure::PlayerName(source),
            };
        }
    };
    if duplicate {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
    }

    if db.is_name_exist(&name, player_database.as_deref_mut()).await {
        return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
    }

    match game.is_name_exit_in_faction(organizing_view, &name) {
        Ok(true) => {
            return send_create_role_failure(game, request, Some(player_count), WorldCreateRoleFailureStage::DuplicateName, CREATE_ROLE_DUPLICATE_STATUS);
        }
        Ok(false) => {}
        Err(source) => {
            return WorldCreateRoleOutcome::Blocked { request, source };
        }
    }

    // Выделение ID и вся мутация выполняются в launch-gate в исходной
    // позиции (после set_creation_service_defaults, до set_id) — обработчик
    // не потребляет sequence счётчика на путях отказа.
    match game.launch_creation_player(
        &WorldCreateRoleRequest {
            name,
            sex,
            occupation,
            head_picture,
            face_picture,
            country,
            account,
        },
        player_list,
        registry,
        original_name_index,
        country_parameters,
        coefficients,
        globe_setup,
        random,
        add_log_text,
    ) {
        Ok(report) => {
            let mut response = CMessage::new(CREATE_ROLE_RESPONSE);
            response.base_mut().add_char(CREATE_ROLE_SUCCESS_STATUS);
            {
                let base = response.base_mut();
                let account_src = request.account.clone();
                let visible = account_src.iter().position(|b| *b == 0).unwrap_or(account_src.len());
                base.add(&account_src[..visible]);
                base.add_char(0);
                base.add_ulong(report.player_id);
                let name = &report.snapshot.name;
                let visible = name.iter().position(|b| *b == 0).unwrap_or(name.len());
                base.add(&name[..visible]);
                base.add_char(0);
            }
            response.base_mut().add_short(i16::from(report.snapshot.level));
            response.base_mut().add_byte(report.snapshot.sex);
            response.base_mut().add_byte(report.snapshot.occupation);
            response.base_mut().add_byte(report.snapshot.country);
            response.base_mut().add_byte(report.snapshot.head);
            for equipment_id in report.snapshot.equipment_ids {
                response.base_mut().add_ulong(equipment_id);
            }
            for equipment_level in report.snapshot.equipment_levels {
                response.base_mut().add_byte(equipment_level);
            }
            response.base_mut().add_long(report.snapshot.region_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send(
                game.current_login_client().map(|client| client.send_queue()),
                false,
            );
            WorldCreateRoleOutcome::Created {
                request,
                player_count_before: player_count,
                player_id: report.player_id,
                snapshot: report.snapshot,
                response_type: CREATE_ROLE_RESPONSE,
                status: CREATE_ROLE_SUCCESS_STATUS,
                wire,
                delivery,
            }
        }
        Err(source) => WorldCreateRoleOutcome::Blocked { request, source },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlayerSelectRequest {
    pub player_id: u32,
    pub account: Vec<u8>,
    pub client_ip: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerSelectValidationOwner {
    LiveMap,
    FrozenDbMap,
    PersistentDatabase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerSelectCloneOwner {
    LiveMap,
    FrozenDbMap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerSelectRejectReason {
    InvalidBinding,
    Deleted,
}

#[derive(Debug)]
pub enum WorldPlayerSelectBlock {
    Clone(PlayerCodecError),
    Route(WorldPlayerSelectRouteError),
    LoadRequest(WorldPlayerLoadRequestBlock),
}

/// Конечный outcome ветки выбора роли. Rejected повторяет исходный отказ
/// `0x1C` до любой мутации; Routed и Queued фиксируют direct-маршрут клона и
/// постановку player-load FIFO; Blocked изолирует технический дефект до
/// ответа LoginServer. Строка журнала InvalidBinding отправляется через
/// `add_log_text` в форме прежнего обработчика без переноса disposition —
/// аналог wrapper-а ветки create-role.
#[derive(Debug)]
pub enum WorldPlayerSelectOutcome {
    Rejected {
        request: WorldPlayerSelectRequest,
        reason: WorldPlayerSelectRejectReason,
        response_type: i32,
        status: i8,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Routed {
        request: WorldPlayerSelectRequest,
        validation_owner: WorldPlayerSelectValidationOwner,
        clone_owner: WorldPlayerSelectCloneOwner,
        route: WorldPlayerSelectRouteOutcome,
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

fn append_c_string(message: &mut nebokrai_shared::network::CBaseMessage, value: &[u8]) {
    let visible = value.iter().position(|byte| *byte == 0).unwrap_or(value.len());
    message.add(&value[..visible]);
    message.add_char(0);
}

#[allow(
    clippy::too_many_arguments,
    reason = "typed outcome сохраняет все наблюдаемые поля одного exact ответа"
)]
fn send_player_select_rejection<G: WorldGameView + ?Sized>(
    game: &mut G,
    request: WorldPlayerSelectRequest,
    reason: WorldPlayerSelectRejectReason,
    log_payload: Option<&[u8]>,
    add_log_text: &mut dyn FnMut(&[u8]),
) -> WorldPlayerSelectOutcome {
    let mut response = CMessage::new(PLAYER_SELECT_RESPONSE);
    response.base_mut().add_char(PLAYER_SELECT_REJECTED_STATUS);
    append_c_string(response.base_mut(), &request.account);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    if let Some(payload) = log_payload {
        add_log_text(payload);
    }
    WorldPlayerSelectOutcome::Rejected {
        request,
        reason,
        response_type: PLAYER_SELECT_RESPONSE,
        status: PLAYER_SELECT_REJECTED_STATUS,
        wire,
        delivery,
    }
}

/// Ветвь `0x4FB05`: сначала live map, затем frozen save-map и лишь потом DB
/// для связки ID↔cdkey; отказ `0x1C` до любой мутации. Clone из live map или
/// frozen savedb-FIFO уходит в direct-маршрут владельца игры — Largess/login/
/// map публикуются до friends и сброса flags внутри game-шва; при полном miss
/// ставится player-load FIFO. Порядок вызовов и предикатов скопирован из
/// исходного обработчика буквально; organizing-context и clone/mаршрут
/// наследуют общий списковый view без второй реализации.
#[allow(clippy::too_many_arguments, reason = "границы один к одному соответствуют owner-ам ветки select")]
pub async fn on_player_select<G: WorldPlayerSelectGameView>(
    game: &mut G,
    organizing: &mut G::OrganizingContext,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    db: &mut dyn WorldPlayerSelectDbView,
    mut player_database: Option<&mut WorldTdsClient>,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    get_tick: &mut dyn FnMut() -> u32,
    add_log_text: &mut dyn FnMut(&[u8]),
    mut message: CMessage,
) -> WorldPlayerSelectOutcome {
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
    } else if db
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
            add_log_text,
        );
    };

    if !game.is_restore_player_exist(request.player_id) {
        let mut deletion_time = game.deletion_player_time(request.player_id);
        if deletion_time == 0 {
            deletion_time = db
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
                add_log_text,
            );
        }
    }

    let (clone_owner, player) = match game.clone_map_player_for_base(
        request.player_id,
        registry,
        organizing,
        coefficients,
    ) {
        Ok(Some(player)) => (Some(WorldPlayerSelectCloneOwner::LiveMap), Some(player)),
        Ok(None) => match game.clone_saving_player_for_base(
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
                return WorldPlayerSelectOutcome::Blocked {
                    request,
                    validation_owner,
                    source: WorldPlayerSelectBlock::Clone(source),
                };
            }
        },
        Err(source) => {
            return WorldPlayerSelectOutcome::Blocked {
                request,
                validation_owner,
                source: WorldPlayerSelectBlock::Clone(source),
            };
        }
    };

    if let (Some(clone_owner), Some(player)) = (clone_owner, player) {
        let route = game.route_select_player(
            organizing,
            request.player_id,
            request.client_ip,
            &request.account,
            Some(player),
            load_player_largess,
            get_tick,
        );
        return match route {
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
        };
    }

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
    }
}

/// Единичное offline-оповещение друга ветки player_return: `0x7F905` уходит
/// на game server друга, найденный по online-маршруту в live-карте.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldReturnedPlayerFriendOutcome {
    pub friend_player_id: u32,
    pub game_server_index: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

/// Конечный outcome ветки возврата игрока. PlayerOnlyOnWorld повторяет
/// журнальный путь subtype-`0` без wire; DecodeBlocked изолирует дефект
/// subtype-`1` decode до любого ответа; PlayerMissing — промах live снимка;
/// Released фиксирует ответ LoginServer `0x7F903` и исходный порядок снятия
/// login/online/offline, team exit и friend-уведомлений. Организационный
/// исход online-снятия свёрнут швом в число удалённых вхождений —
/// владелец организаций старого пакета ради одной ветки не переносится.
#[derive(Debug)]
pub enum WorldPlayerReturnOutcome {
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
        online_removed_occurrences: usize,
        offline_inserted: bool,
        team_exit: Option<WorldLoginTimeoutTeamExit>,
        friends: Vec<WorldReturnedPlayerFriendOutcome>,
    },
}

/// Конечный outcome ветки player detail. Rejected повторяет статус-ответ
/// `-1`/`0` до любой мутации; WrongGameServer — отказ `-2` с переводом
/// login/online/offline до сброса faction-data; PromotedOnline кодирует
/// полный mapped-снимок в тело исходного сообщения и публикует online после
/// wire-ответа на source socket; Serialization* изолируют дефект encode до
/// ответа. Организационные исходы снятия/постановки online свёрнуты швом в
/// число удалённых вхождений и флаг вставки, без переноса владельца
/// организаций старого пакета.
#[derive(Debug)]
pub enum WorldPlayerDetailOutcome {
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
        online_removed_occurrences: usize,
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
        online_inserted: bool,
    },
    SerializationBlocked {
        player_id: u32,
        error: PlayerCodecError,
    },
    SerializationOwnerMissing {
        player_id: u32,
    },
}

/// Ветвь `0x5FB02`: subtype-`1` decode выполняет pet cleanup и ранний
/// offline-переход у владельца игры до снимка; subtype-`0` пишет строку
/// журнала без wire. Снятие login/online/offline и team exit идут в исходной
/// последовательности после ответа LoginServer `0x7F903`, friend-присутствие
/// `0x7F905` рассылается последним циклом. Порядок вызовов и предикатов
/// скопирован из исходного обработчика буквально.
#[allow(clippy::too_many_arguments, reason = "границы один к одному соответствуют owner-ам ветки return")]
pub fn on_player_return<G: WorldPlayerReturnGameView>(
    game: &mut G,
    organizing: &mut G::OrganizingContext,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_log_text: &mut dyn FnMut(&[u8]),
    mut message: CMessage,
) -> WorldPlayerReturnOutcome {
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
                return WorldPlayerReturnOutcome::DecodeBlocked { player_id, error };
            }
        }
    } else {
        if subtype == 0 {
            let line = format!("Player {} Only On WorldServer!", player_id);
            add_log_text(line.as_bytes());
        }
        None
    };

    let Some(player) = game.returned_player_snapshot(player_id) else {
        return if subtype == 0 {
            WorldPlayerReturnOutcome::PlayerOnlyOnWorld { player_id }
        } else {
            WorldPlayerReturnOutcome::PlayerMissing { player_id, subtype }
        };
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
    let online_removed_occurrences = game.remove_online_player(organizing, owner_id);
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

    WorldPlayerReturnOutcome::Released {
        player_id,
        subtype,
        decode,
        login_wire,
        login_delivery,
        login_removed,
        online_removed_occurrences,
        offline_inserted,
        team_exit,
        friends,
    }
}

fn send_player_detail_status<G: WorldGameView + ?Sized>(
    game: &G,
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

/// Ветвь `0x5FB01`: промах login-маршрута отвечает статусом `-1`/`0`
/// (online/offline различение) и строкой журнала до любой мутации;
/// несовпадение assigned map и source map отвечает `-2`, переводит login/
/// online в offline и сбрасывает faction-data. Совпавший маршрут меняет тип
/// исходного сообщения на `0x7F901`, дописывает полный mapped-снимок и лишь
/// после wire-ответа на source socket снимает login/offline и публикует
/// online. Порядок вызовов скопирован из исходного обработчика буквально.
pub fn on_player_detail<G: WorldPlayerDetailGameView>(
    game: &mut G,
    organizing: &mut G::OrganizingContext,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_log_text: &mut dyn FnMut(&[u8]),
    mut message: CMessage,
) -> WorldPlayerDetailOutcome {
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
        add_log_text(error_text.as_bytes());
        return WorldPlayerDetailOutcome::Rejected {
            player_id,
            status,
            source_socket_id,
            response_type: PLAYER_DETAIL_RESPONSE,
            wire,
            delivery,
        };
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
        let online_removed_occurrences = game.remove_online_player(organizing, owner_id);
        let offline_inserted = game.append_offline_player_id(owner_id);
        let faction_data_reset = game.reset_map_player_faction_data(player.map_key);
        let error_text = format!(
            "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> GameServer Invalid",
            player_id
        );
        add_log_text(error_text.as_bytes());
        return WorldPlayerDetailOutcome::WrongGameServer {
            player_id,
            owner_id: player.owner_id,
            source_map_id,
            assigned_map_id,
            source_socket_id,
            response_type: PLAYER_DETAIL_RESPONSE,
            wire,
            delivery,
            login_removed,
            online_removed_occurrences,
            offline_inserted,
            faction_data_reset,
        };
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
            return WorldPlayerDetailOutcome::SerializationOwnerMissing { player_id };
        }
        Err(error) => {
            return WorldPlayerDetailOutcome::SerializationBlocked { player_id, error };
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
    let online_inserted = game.append_online_player_id(organizing, player.owner_id);

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
        online_inserted,
    }
}

/// Match-поверхность исходного `OnLogMessage` диспетчера `logmessage.cpp`:
/// opcode выбирает ветвь, нераспознанный opcode фиксируется `NoOp` без
/// побочных эффектов, маршрут `Handled/Pending` сохранён для цепочки
/// владельцев старого пакета (`process_world_message`). Порядок рукавов
/// повторяет прежний диспетчер буквально.
#[derive(Debug)]
pub enum WorldLogMessageOutcome {
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

pub enum WorldLogMessageDispatch {
    Handled(WorldLogMessageOutcome),
    Pending(CMessage),
}

/// Диспетчер `OnLogMessage` целиком: один вызов ветви на сообщение в прежнем
/// порядке match-рукавов. Страновые gate/view и организационный view приходят
/// dyn-швами от callsite-а, чтобы владельцы таблиц старого пакета не входили
/// в сигнатуру; `delete_log_enabled` и обвязки журнала/tick сохраняют прежние
/// точки снятия `_time` и errno-log по `_time`.
#[allow(
    clippy::too_many_arguments,
    reason = "границы один к одному соответствуют owner-ам прежнего dispatcher-адаптера старого пакета"
)]
pub async fn on_log_message<G, D>(
    game: &mut G,
    organizing: &mut G::OrganizingContext,
    organizing_parameters: &COrganizingParam,
    country_gate: &mut dyn WorldDeleteRoleCountryGate,
    country_view: &dyn WorldCountryView,
    country_parameters: &mut CCountryParam,
    player_list: &mut CPlayerList,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    globe_setup: &GlobeSetupSnapshot,
    db: &mut D,
    player_database: Option<&mut WorldTdsClient>,
    delete_log_enabled: bool,
    add_log_text: &mut dyn FnMut(&[u8]),
    get_tick: &mut dyn FnMut() -> u32,
    random: &mut dyn FnMut(i32) -> i32,
    message: CMessage,
) -> WorldLogMessageDispatch
where
    G: WorldPlayerSelectGameView
        + WorldPlayerReturnGameView
        + WorldPlayerDetailGameView
        + WorldCreateRoleLaunchGate,
    G::OrganizingContext: WorldDeleteRoleOrganizingGate + WorldCreateRoleOrganizingView,
    D: RsPlayerOwner<CPlayer>
        + WorldDeleteRoleDbView
        + WorldCreateRoleDbView
        + WorldPlayerSelectDbView,
{
    match message.message_type() {
        PLAYER_BASE_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::PlayerBase(
                crate::app::player_base::on_player_base(
                    game,
                    organizing,
                    registry,
                    coefficients,
                    globe_setup,
                    db,
                    player_database,
                    message,
                )
                .await,
            ),
        ),
        DELETE_ROLE_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::DeleteRole(
                on_delete_role(
                    game,
                    organizing,
                    country_gate,
                    organizing_parameters,
                    globe_setup,
                    db,
                    player_database,
                    delete_log_enabled,
                    message,
                )
                .await,
            ))
        }
        PLAYER_DETAIL_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                on_player_detail(game, organizing, registry, coefficients, add_log_text, message),
            ))
        }
        PLAYER_RETURN_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
                on_player_return(
                    game,
                    organizing,
                    session_factory,
                    registry,
                    coefficients,
                    add_log_text,
                    message,
                ),
            ))
        }
        RESTORE_ROLE_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::RestoreRole(
                on_restore_role(game, message),
            ))
        }
        CREATE_ROLE_REQUEST => {
            let organizing_view: &dyn WorldCreateRoleOrganizingView = &*organizing;
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::CreateRole(
                on_create_role(
                    game,
                    organizing_view,
                    country_view,
                    db,
                    player_list,
                    registry,
                    original_name_index,
                    country_parameters,
                    coefficients,
                    globe_setup,
                    player_database,
                    random,
                    add_log_text,
                    message,
                )
                .await,
            ))
        }
        PLAYER_SELECT_REQUEST => {
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
                on_player_select(
                    game,
                    organizing,
                    registry,
                    coefficients,
                    db,
                    player_database,
                    load_player_largess,
                    get_tick,
                    add_log_text,
                    message,
                )
                .await,
            ))
        }
        ACCOUNT_LOGIN_CLEANUP_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountLoginCleanup(
                on_account_login_cleanup(game, session_factory, message),
            ),
        ),
        ACCOUNT_DISCONNECT_REQUEST => WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountDisconnect(
                on_account_disconnect(game, session_factory, message),
            ),
        ),
        request_type => WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::NoOp {
            request_type,
        }),
    }
}
