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
//! и сброса flags. Detail отвечает промахом до мутаций, mismatch map переводит
//! в offline, совпавший маршрут кодирует полный снимок до online-записи. Return
//! декодирует subtype-1, отвечает LoginServer и снимает login/online до friend
//! оповещений. Tiberius и owned snapshots заменяют ADO/STL без перестановки.
//!
//! Все ветви диспетчера перенесены в `realm/app/logmessage.rs` (список
//! персонажей — в `realm/app/player_base.rs`); здесь остаются dispatcher,
//! адаптеры швов и re-export типов исходов.

use crate::dbaccess::worlddb::rsplayer::TiberiusRsPlayer;
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::CMessage;
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
use crate::worldserver::appworld::player::{CPlayer, PlayerPropertyCoefficients};
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::worldserver::game::{CGame, legacy_tick_ms};
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
const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;

pub(crate) use nebokrai_realm::app::logmessage::{
    WorldCreateRoleOutcome, WorldPlayerSelectOutcome, WorldRestoreRoleOutcome,
};

pub(crate) use nebokrai_realm::app::player_base::WorldPlayerBaseOutcome;
pub(crate) use nebokrai_realm::app::logmessage::{
    WorldAccountDisconnectOutcome, WorldAccountLoginCleanupOutcome, WorldDeleteRoleOutcome,
    WorldPlayerDetailOutcome, WorldPlayerReturnOutcome,
};

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
        PLAYER_DETAIL_REQUEST => {
            let mut log_wrapper = |bytes: &[u8]| {
                let _ = add_error_log_text(bytes);
            };
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                nebokrai_realm::app::logmessage::on_player_detail(
                    game,
                    organizing,
                    registry,
                    coefficients,
                    &mut log_wrapper,
                    message,
                ),
            ))
        }
        PLAYER_RETURN_REQUEST => {
            let mut log_wrapper = |bytes: &[u8]| {
                let _ = add_error_log_text(bytes);
            };
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
                nebokrai_realm::app::logmessage::on_player_return(
                    game,
                    organizing,
                    session_factory,
                    registry,
                    coefficients,
                    &mut log_wrapper,
                    message,
                ),
            ))
        }
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
            let mut get_tick = legacy_tick_ms;
            let mut log_wrapper = |bytes: &[u8]| {
                let _ = add_error_log_text(bytes);
            };
            WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerSelect(
                nebokrai_realm::app::logmessage::on_player_select(
                    game,
                    organizing,
                    registry,
                    coefficients,
                    rs_player,
                    player_database,
                    load_player_largess,
                    &mut get_tick,
                    &mut log_wrapper,
                    message,
                )
                .await,
            ))
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
