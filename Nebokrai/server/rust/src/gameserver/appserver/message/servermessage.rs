//! Владелец входного GameServer dispatcher-а `OnServerMessage`.
//!
//! Весь dispatcher RVA `0x0009D300` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме цепочек
//! сообщения `0x7F801` для AttackCity/Village и terminal selector `0x3B`, а
//! также полной typed Billing reconnect ветви `0x6F904`; они имеют статус
//! `IMPLEMENTED`. Точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходник
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\servermessage.cpp`.
//!
//! Оба case передают текущий payload cursor парному decoder-у, затем всегда на
//! успешном legacy payload вызывают initial region-state owner и только после
//! него пишут точный startup log. Decoder-result оригинал игнорировал, но exact
//! EXE доказал его безусловный `true`. Safe short-buffer не достигает init/log:
//! старый безразмерный pointer имел неизвестный UB, которому Rust не назначает
//! побочные эффекты. Остальные selector-ы возвращаются caller-у как `false` и
//! этим helper-ом не интерпретируются; полный switch не имитируется. Billing
//! handoff закрывает старый owner, публикует новый, приоритетно ставит
//! регистрацию и только затем включает control-send. Парная World-ветвь
//! остаётся RAW до материализации полного player snapshot.
//!
//! Terminal selector сначала вызывает `InitNetServer`, затем читает login и
//! world ID и присваивает их даже после ошибки Host. Rust сохраняет этот
//! partial-effect порядок: malformed хвост возвращается отдельно, не откатывая
//! уже выполненный network init и не подставляя нулевые identity.

use std::error::Error;
use std::fmt;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityRegionContext, CAttackCitySys,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarRegionContext,
};
use crate::gameserver::gameserver::game::{CGame, GameNetworkInitializationError};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::nets::netserver::mynetclient::CMyNetClient;

const BILLING_REGISTRATION: i32 = 0x000E_F101;
const CLIENT_SERVER_START_SELECTOR: i32 = 0x3b;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameServerIds {
    pub(crate) login: i32,
    pub(crate) world: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameClientServerStartPayloadError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Debug)]
pub(crate) struct GameClientServerStartReport {
    pub(crate) network: Result<(), GameNetworkInitializationError>,
    pub(crate) server_ids: Result<GameServerIds, GameClientServerStartPayloadError>,
}

/// Выполняет terminal startup selector `0x3B` в исходном порядке side effects.
pub(crate) fn dispatch_client_server_start(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    now_ms: u32,
) -> Option<GameClientServerStartReport> {
    if selector != CLIENT_SERVER_START_SELECTOR {
        return None;
    }

    let network = game.init_net_server(now_ms);
    let server_ids = read_server_ids(message);
    if let Ok(server_ids) = server_ids {
        game.set_server_ids(server_ids.login, server_ids.world);
    }
    Some(GameClientServerStartReport {
        network,
        server_ids,
    })
}

fn read_server_ids(
    message: &mut CMessage,
) -> Result<GameServerIds, GameClientServerStartPayloadError> {
    let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let login = read_start_long(wire, cursor)?;
    let world = read_start_long(wire, cursor)?;
    Ok(GameServerIds { login, world })
}

fn read_start_long(
    wire: &[u8],
    cursor: &mut usize,
) -> Result<i32, GameClientServerStartPayloadError> {
    let offset = *cursor;
    let available = wire.len().saturating_sub(offset);
    let Some(bytes) = wire.get(offset..offset.saturating_add(4)) else {
        return Err(GameClientServerStartPayloadError {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("server ID содержит четыре байта"),
    ))
}

/// Наблюдаемый итог reconnect-ветви Billing `0x6F904`.
#[derive(Debug)]
pub(crate) struct GameBillingClientReplacement {
    pub(crate) previous_client_closed: bool,
    pub(crate) registration: Result<i32, SendMessageError>,
    pub(crate) connected_notice: bool,
}

/// Выполняет полную самодостаточную ветвь Billing reconnect handoff.
pub(crate) fn on_billing_client_reconnected(
    game: &mut CGame,
    client: CMyNetClient,
) -> GameBillingClientReplacement {
    let previous_client_closed = game.replace_billing_client(client);
    let registration = CMessage::new(BILLING_REGISTRATION).send_to_bs(game, true);
    game.current_billing_client_mut()
        .expect("reconnect Billing client только что опубликован")
        .enable_control_send();
    GameBillingClientReplacement {
        previous_client_closed,
        registration,
        connected_notice: true,
    }
}

pub(crate) trait WarScheduleSetupContext {
    type Region: Copy;

    /// Ищет сначала `CGame::s_mapRegion`, затем nullable proxy fallback.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32);

    fn region_country(&self, region: Self::Region) -> u8;

    fn set_region_country(&mut self, region: Self::Region, country: u8);

    fn add_log_text(&mut self, text: &'static str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarScheduleSetupError {
    AttackCity(AttackCityDecodeError),
    Village(VillageWarDecodeError),
}

impl fmt::Display for WarScheduleSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttackCity(error) => write!(formatter, "AttackCity snapshot: {error}"),
            Self::Village(error) => write!(formatter, "Village snapshot: {error}"),
        }
    }
}

impl Error for WarScheduleSetupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AttackCity(error) => Some(error),
            Self::Village(error) => Some(error),
        }
    }
}

/// Обрабатывает только доказанные schedule selectors `0x1B/0x1C`.
pub(crate) fn dispatch_war_schedule_setup<Context: WarScheduleSetupContext>(
    selector: i32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    context: &mut Context,
) -> Result<bool, WarScheduleSetupError> {
    match selector {
        0x1b => {
            attack_city_sys
                .decord_from_byte_array(payload, cursor)
                .map_err(WarScheduleSetupError::AttackCity)?;
            {
                let mut adapter = AttackCityContextAdapter(context);
                attack_city_sys.init_city_region_state(&mut adapter);
            }
            context.add_log_text("Initial SI_ATTACKCITYSYS_SETUP...OK!");
            Ok(true)
        }
        0x1c => {
            village_war_sys
                .decord_from_byte_array(payload, cursor)
                .map_err(WarScheduleSetupError::Village)?;
            {
                let mut adapter = VillageWarContextAdapter(context);
                village_war_sys.init_village_region_state(&mut adapter);
            }
            context.add_log_text("Initial SI_VILLAGEWARSYS_SETUP...OK!");
            Ok(true)
        }
        _ => Ok(false),
    }
}

struct AttackCityContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarScheduleSetupContext> AttackCityRegionContext
    for AttackCityContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_region_then_proxy(region_id)
    }

    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32) {
        self.0.reset_war_state(region, war_number, state);
    }
}

struct VillageWarContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarScheduleSetupContext> VillageWarRegionContext
    for VillageWarContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_region_then_proxy(region_id)
    }

    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32) {
        self.0.reset_war_state(region, war_number, state);
    }

    fn region_country(&self, region: Self::Region) -> u8 {
        self.0.region_country(region)
    }

    fn set_region_country(&mut self, region: Self::Region, country: u8) {
        self.0.set_region_country(region, country);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\servermessage.cpp

// ============================================================================
// FUNCTION: OnServerMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\servermessage.cpp:75
// RVA: 0x0009D300
// ADDRESS: 0049d300
// PROTOTYPE: void __cdecl OnServerMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
