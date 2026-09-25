//! Login lifecycle `OnLogMessage` из `logmessage.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Перенесённые ветви диспетчера `OnLogMessage`: списки восстановления и
//! удаления, вход и отключение аккаунта изменяются в исходном порядке до
//! wire-ответа LoginServer; форма payload не меняется. Create-role выполняет
//! limit, sex/occupation, country, filter и name checks до выдачи ID и
//! equipment. Select сначала проверяет live map, frozen save-map и лишь затем
//! DB; direct-маршрут клона остаётся одним вызовом у владельца игры. Список
//! персонажей `0x4FB01` живёт в соседнем `player_base.rs`; остальные пока не
//! перенесённые ветви остаются у владельца старого пакета.

use nebokrai_shared::resources::GlobeSetupSnapshot;

use crate::app::world_game_view::{
    WorldCountryView, WorldCreateRoleDbView, WorldCreateRoleLaunchFailure,
    WorldCreateRoleLaunchGate, WorldCreateRoleOrganizingView, WorldDeleteRoleCountryGate,
    WorldDeleteRoleDbView, WorldGameView, WorldLoginTimeoutTeamExit, WorldPlayerLoadRequestBlock,
    WorldPlayerLoadRequestOutcome, WorldPlayerSelectDbView, WorldPlayerSelectGameView,
    WorldPlayerSelectRouteError, WorldPlayerSelectRouteOutcome,
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
