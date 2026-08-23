//! Достигнутая send/receive dispatch storage-часть `CGame` GameServer.
//!
//! PDB подтверждает nullable `s_pNetClientOfWS +0x8`,
//! `s_pNetClientOfBS +0xC`, `s_pNetServer +0x10`, ordered
//! `s_mapPlayer +0x14` типа `long -> CPlayer*` и ordered
//! `m_mTeamSessionID +0x188` типа `unsigned long -> long`. `FindPlayer` RVA
//! `0x00014930`, `GetTeamSessionID` RVA `0x00013690` и достигнутые
//! `CMessage` sends `0x00013910..0x00014923` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `server/gameserver/gameserver/game.h/.cpp`.
//!
//! `BTreeMap` сохраняет наблюдаемый ordered-map lookup, owned `CPlayer`
//! заменяет сырой pointer только в достигнутой runtime-проекции, а
//! `CMyNetServer/CMyNetClient` остаются отдельными historical owners. Полный
//! `ProcessMessage` RVA `0x00005830` атомарно забирает FIFO строго в порядке
//! World, Billing, accepted clients и для каждого элемента вызывает
//! `CMessage::Run`; это имеет статус `IMPLEMENTED`. `InitNetServer` RVA
//! `0x000020D0`, `InitNetClientOfWS/BS` RVA `0x00002C60/0x00002DE0`,
//! `ReConnectWorldServer/BillingServer` RVA `0x0000B7B0/0x0000B8D0`, retry
//! entries RVA `0x0000BAD0/0x0000BB80` и task owners RVA
//! `0x0000BDA0/0x0000BE10` также материализованы через общий Linux transport;
//! точный return listener-а подтверждён машинным кодом. Из `Release` RVA
//! `0x00009FD0` перенесён только начальный stop/join reconnect workers. Полный
//! позиционный разбор `LoadSetup/LoadSetupEx` RVA `0x00009960/0x00009160`
//! материализован с исходными defaults, частичной мутацией и игнорированием
//! labels. Открытие listener-а заменяет поздней проверкой bind старый
//! `FindWindow` single-instance guard; недоказанный default billing bind-port
//! остаётся typed-границей. `Release`, resource loaders и прочие maps ниже
//! остаются RAW;
//! `Init` связан через обязательный World client, общий MSVCRT RNG и sequence
//! registry до необязательной Billing-попытки, затем создаёт DupliRegion,
//! move-check, ranks и GoodsWar owners. GoodsWar constructor сохраняет
//! немедленный World request. Tokio/socket types заменяют ненаблюдаемые
//! `CBaseMessage::Initial` и `CMySocket::MySocketInit`; следующий незакрытый
//! шаг — resource/runtime owners после завершённого `Init`.
//! `with_send_state/register_*/attach_*` являются явной assembly-границей
//! baseline и не снимают их псевдокод. Network setup передаётся отдельной
//! post-`LoadSetup*` проекцией. Windows thread handles заменены owned Tokio
//! tasks с тем же порядком exit/sleep/retry/join. `SO_SNDBUF=0` и World
//! reconnect player snapshot остаются локальными границами; nullable раннюю
//! ветвь `CMessage::SendAll` принимает отдельно как `Option`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rustix::system::uname;

use crate::gameserver::appserver::goods::cbattlefairyproperty::CBattleFairyProperty;
use crate::gameserver::appserver::goodswarmember::CGoodsWarMember;
use crate::gameserver::appserver::message::sequencestring::{
    CSequenceRegistry, SequenceRegistryInitializationError,
};
use crate::gameserver::appserver::message::servermessage::on_billing_client_reconnected;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::{
    MoveCheckCellRegistry, ShapeIdentity, ShapeResolver, ShapeView,
};
use crate::gameserver::gameserver::playerranks::CPlayerRanks;
use crate::nets::clients::ClientConnectError;
use crate::nets::mysocket::legacy_ipv4_word;
use crate::nets::netserver::message::{CMessage, GameMessageHandlers, SendMessageError};
use crate::nets::netserver::mynetclient::{
    CMyNetClient, GameClientIoError, GameClientIoStep, ServerType,
};
use crate::nets::netserver::mynetserver::{
    CMyNetServer, GameServerEvent, GameServerEventPublisher,
};
use crate::nets::servers::ServerHostError;
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::dupliregionsetup::CDupliRegionSetup;
use crate::public::equipmentcomposelist::EquipmentComposeList;
use crate::setup::cbattlefairyexpconfig::CBattleFairyExpConfig;
use crate::setup::changebody::CChangeBodyConf;
use crate::setup::contributesetup::CContributeSetup;
use crate::setup::fairyexpconf::CFairyExpConf;
use crate::setup::gmlist::CGMList;
use crate::setup::goodsdestructionconfig::GoodsDestroySetup;
use crate::setup::hitlevelsetup::CHitLevelSetup;
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::incrementshoplist::CIncrementShopList;
use crate::setup::leitingsetup::CThingSetup;
use crate::setup::logsystem::CLogSystem;
use crate::setup::newskillmonsterlist::NewSkillMonsterConf;
use crate::setup::playerlist::CPlayerList;
use crate::setup::preciousboxconf::PreciousBoxConf;
use crate::setup::prisonconf::PrisonConf;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::setup::tradelist::CTradeList;
use crate::transport::bind_tcp_ipv4;

const PLAYER_TYPE: i32 = 400;
const DEFAULT_SOCKET_TYPE: i32 = 1;
const WORLD_REGISTRATION: i32 = 0x0005_FA01;
const BILLING_REGISTRATION: i32 = 0x000E_F101;
const RECONNECT_RETRY_DELAY: Duration = Duration::from_millis(8_000);

#[derive(Clone, Debug, Eq, PartialEq)]
struct GameSetup {
    world_host: Vec<u8>,
    world_port: u32,
    billing_host: Vec<u8>,
    billing_port: u32,
    billing_backup_host: Vec<u8>,
    billing_backup_port: u32,
    billing_bind_ip: Vec<u8>,
    billing_bind_port: Option<u32>,
    listen_port: u32,
    local_ip: Vec<u8>,
    check_network: bool,
    maximum_bytes_per_second: u32,
    maximum_message_length: u32,
    forbid_time_ms: u32,
    check_message_content: bool,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
    refresh_info_time_ms: u32,
    save_info_time_ms: u32,
    watch_runtime_info: bool,
    watch_runtime_time_ms: u32,
    enter_time: u32,
    message_validate_time_ms: u32,
    sequence_count: u32,
}

impl Default for GameSetup {
    fn default() -> Self {
        Self {
            world_host: b"127.0.0.1".to_vec(),
            world_port: 0x1fa4,
            billing_host: b"127.0.0.1".to_vec(),
            billing_port: 0x1f98,
            billing_backup_host: b"127.0.0.1".to_vec(),
            billing_backup_port: 0x1f98,
            billing_bind_ip: Vec::new(),
            billing_bind_port: None,
            listen_port: 0x092b,
            local_ip: b"127.0.0.1".to_vec(),
            check_network: true,
            maximum_bytes_per_second: 5_000,
            maximum_message_length: 0x1_9000,
            forbid_time_ms: 10,
            check_message_content: true,
            maximum_clients: 500,
            maximum_in_flight_sends: 3,
            permitted_send_bytes: 0xc800,
            refresh_info_time_ms: 1_000,
            save_info_time_ms: 60_000,
            watch_runtime_info: false,
            watch_runtime_time_ms: 60_000,
            enter_time: 5,
            message_validate_time_ms: 0,
            sequence_count: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GameSetupEx {
    maximum_block_connections: i32,
    first_receive_timeout_ms: i32,
    maximum_yuan_bao: Option<i32>,
    maximum_money: Option<i32>,
}

impl Default for GameSetupEx {
    fn default() -> Self {
        Self {
            maximum_block_connections: 10,
            first_receive_timeout_ms: 4_000,
            maximum_yuan_bao: None,
            maximum_money: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameSetupLoadReport {
    pub(crate) parsed_pairs: usize,
    pub(crate) stopped_at_pair: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct GameSetupOpenError {
    pub(crate) path: PathBuf,
    pub(crate) source: io::Error,
}

impl fmt::Display for GameSetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не удалось прочитать {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for GameSetupOpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRuntimePaths {
    pub(crate) setup: PathBuf,
    pub(crate) setup_ex: PathBuf,
}

impl GameRuntimePaths {
    pub(crate) fn from_runtime_directory(directory: impl AsRef<Path>) -> Self {
        let directory = directory.as_ref();
        Self {
            setup: directory.join("setup.ini"),
            setup_ex: directory.join("setupex.ini"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum GameSetupExLoad {
    Loaded(GameSetupLoadReport),
    Unavailable(GameSetupOpenError),
}

#[derive(Debug)]
pub(crate) struct GameRuntimeSetupReport {
    pub(crate) setup: GameSetupLoadReport,
    pub(crate) setup_ex: GameSetupExLoad,
}

#[derive(Debug)]
pub(crate) enum GameRuntimeSetupError {
    Setup(GameSetupOpenError),
    MissingField(&'static str),
}

impl fmt::Display for GameRuntimeSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(error) => error.fmt(formatter),
            Self::MissingField(field) => {
                write!(formatter, "GameServer setup не определил поле {field}")
            }
        }
    }
}

impl std::error::Error for GameRuntimeSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup(error) => Some(error),
            Self::MissingField(_) => None,
        }
    }
}

impl GameSetup {
    fn parse_positional(&mut self, bytes: &[u8]) -> GameSetupLoadReport {
        let mut tokens = GameSetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.report();
                };
                let Some(value) = $parser(raw) else {
                    return tokens.report();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_game_setup_number::<$type>(raw));
            };
        }

        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_game_setup_bool);
            };
        }

        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_bytes!(world_host);
        read_number!(world_port, u32);
        read_bytes!(billing_host);
        read_number!(billing_port, u32);
        read_bytes!(billing_backup_host);
        read_number!(billing_backup_port, u32);
        read_bytes!(billing_bind_ip);
        read_value!(billing_bind_port, |raw| {
            parse_game_setup_number::<u32>(raw).map(Some)
        });
        read_number!(listen_port, u32);
        read_bytes!(local_ip);
        read_bool!(check_network);
        read_number!(maximum_bytes_per_second, u32);
        read_number!(maximum_message_length, u32);
        read_number!(forbid_time_ms, u32);
        read_bool!(check_message_content);
        read_number!(maximum_clients, i32);
        read_number!(maximum_in_flight_sends, i32);
        read_number!(permitted_send_bytes, i32);
        read_number!(refresh_info_time_ms, u32);
        read_number!(save_info_time_ms, u32);
        read_bool!(watch_runtime_info);
        read_number!(watch_runtime_time_ms, u32);
        read_number!(enter_time, u32);
        read_number!(message_validate_time_ms, u32);
        read_number!(sequence_count, u32);
        tokens.report()
    }

    fn network_setup(
        &self,
        setup_ex: &GameSetupEx,
    ) -> Result<GameNetworkSetup, GameRuntimeSetupError> {
        let billing_bind_port = self
            .billing_bind_port
            .ok_or(GameRuntimeSetupError::MissingField("_bind_port_for_bs"))?;
        Ok(GameNetworkSetup::new(
            GameUpstreamEndpoint::new(&self.world_host, self.world_port),
            GameBillingPlan::new(
                GameUpstreamEndpoint::new(&self.billing_host, self.billing_port),
                GameUpstreamEndpoint::new(&self.billing_backup_host, self.billing_backup_port),
                &self.billing_bind_ip,
                billing_bind_port,
            ),
            &self.local_ip,
            GameListenerPlan::new(
                self.listen_port,
                self.check_network,
                self.maximum_bytes_per_second,
                self.forbid_time_ms,
                self.maximum_message_length,
                self.maximum_clients,
                self.maximum_in_flight_sends,
                self.permitted_send_bytes,
                setup_ex.maximum_block_connections,
                setup_ex.first_receive_timeout_ms,
            ),
            self.check_message_content,
        ))
    }
}

impl GameSetupEx {
    fn parse_positional(&mut self, bytes: &[u8]) -> GameSetupLoadReport {
        let mut tokens = GameSetupTokens::new(bytes);

        macro_rules! read_number {
            ($field:ident) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.report();
                };
                let Some(value) = parse_game_setup_number::<i32>(raw) else {
                    return tokens.report();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        read_number!(maximum_block_connections);
        read_number!(first_receive_timeout_ms);

        let Some(raw) = tokens.next_value() else {
            return tokens.report();
        };
        let Some(value) = parse_game_setup_number::<i32>(raw) else {
            return tokens.report();
        };
        self.maximum_yuan_bao = Some(value);
        tokens.parsed();

        let Some(raw) = tokens.next_value() else {
            return tokens.report();
        };
        let Some(value) = parse_game_setup_number::<i32>(raw) else {
            return tokens.report();
        };
        self.maximum_money = Some(value);
        tokens.parsed();
        tokens.report()
    }
}

struct GameSetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> GameSetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    fn report(&self) -> GameSetupLoadReport {
        GameSetupLoadReport {
            parsed_pairs: self.parsed_pairs,
            stopped_at_pair: (self.parsed_pairs < self.attempted_pairs)
                .then_some(self.attempted_pairs),
        }
    }
}

fn parse_game_setup_number<T: FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_game_setup_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameUpstreamEndpoint {
    host: Vec<u8>,
    port: u32,
}

impl GameUpstreamEndpoint {
    pub(crate) fn new(host: &[u8], port: u32) -> Self {
        Self {
            host: host.to_vec(),
            port,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameBillingPlan {
    primary: GameUpstreamEndpoint,
    backup: GameUpstreamEndpoint,
    bind_ip: Vec<u8>,
    bind_port: u32,
}

impl GameBillingPlan {
    pub(crate) fn new(
        primary: GameUpstreamEndpoint,
        backup: GameUpstreamEndpoint,
        bind_ip: &[u8],
        bind_port: u32,
    ) -> Self {
        Self {
            primary,
            backup,
            bind_ip: bind_ip.to_vec(),
            bind_port,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameListenerPlan {
    listen_port: u32,
    check_network: bool,
    maximum_bytes_per_second: u32,
    forbid_time_ms: u32,
    maximum_message_length: u32,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
    maximum_block_connections: i32,
    first_receive_timeout_ms: i32,
}

impl GameListenerPlan {
    #[allow(
        clippy::too_many_arguments,
        reason = "план сохраняет десять достигнутых setup-полей Game listener"
    )]
    pub(crate) const fn new(
        listen_port: u32,
        check_network: bool,
        maximum_bytes_per_second: u32,
        forbid_time_ms: u32,
        maximum_message_length: u32,
        maximum_clients: i32,
        maximum_in_flight_sends: i32,
        permitted_send_bytes: i32,
        maximum_block_connections: i32,
        first_receive_timeout_ms: i32,
    ) -> Self {
        Self {
            listen_port,
            check_network,
            maximum_bytes_per_second,
            forbid_time_ms,
            maximum_message_length,
            maximum_clients,
            maximum_in_flight_sends,
            permitted_send_bytes,
            maximum_block_connections,
            first_receive_timeout_ms,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameNetworkSetup {
    world: GameUpstreamEndpoint,
    billing: GameBillingPlan,
    local_ip: Vec<u8>,
    listener: GameListenerPlan,
    check_message_content: bool,
}

impl GameNetworkSetup {
    pub(crate) fn new(
        world: GameUpstreamEndpoint,
        billing: GameBillingPlan,
        local_ip: &[u8],
        listener: GameListenerPlan,
        check_message_content: bool,
    ) -> Self {
        Self {
            world,
            billing,
            local_ip: local_ip.to_vec(),
            listener,
            check_message_content,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameUpstreamDirection {
    World,
    BillingPrimary,
    BillingBackup,
}

#[derive(Debug)]
pub(crate) enum GameClientInitializationFailure {
    MissingNetworkSetup,
    MissingNetworkServerOwner,
    AddressTooLong {
        direction: GameUpstreamDirection,
        length: usize,
    },
    AddressEncodingUnsupported {
        direction: GameUpstreamDirection,
    },
    AddressResolution {
        direction: GameUpstreamDirection,
    },
    BillingBindAddressResolution,
    Bind(io::Error),
    Connect {
        direction: GameUpstreamDirection,
        source: ClientConnectError,
    },
}

#[derive(Debug)]
pub(crate) struct GameConnectAttempt {
    pub(crate) direction: GameUpstreamDirection,
    pub(crate) failure: Option<GameClientInitializationFailure>,
}

#[derive(Debug)]
pub(crate) enum GameClientInitialization {
    Connected {
        endpoint: SocketAddrV4,
        used_billing_backup: bool,
        registration: Result<i32, SendMessageError>,
        attempts: Vec<GameConnectAttempt>,
    },
    Failed {
        attempts: Vec<GameConnectAttempt>,
    },
}

#[derive(Debug)]
pub(crate) enum GameReconnectPublication {
    Published {
        endpoint: SocketAddrV4,
        used_billing_backup: bool,
        attempts: Vec<GameConnectAttempt>,
    },
    Failed {
        attempts: Vec<GameConnectAttempt>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReconnectWorkerEnd {
    Published,
    ExitRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReconnectTaskStartError {
    MissingNetworkSetup,
    MissingNetworkServerOwner,
}

struct GameReconnectTask {
    exit_requested: Arc<AtomicBool>,
    handle: tokio::task::JoinHandle<GameReconnectWorkerEnd>,
}

#[derive(Debug)]
pub(crate) enum GameNetworkInitializationError {
    MissingNetworkSetup,
    Host(ServerHostError),
}

#[derive(Debug)]
pub(crate) struct GameInitializationThroughBillingReport {
    pub(crate) setup: GameRuntimeSetupReport,
    pub(crate) world: GameClientInitialization,
    pub(crate) sequence_elements: usize,
    pub(crate) billing: GameClientInitialization,
}

#[derive(Debug)]
pub(crate) struct GameInitializationReport {
    pub(crate) through_billing: GameInitializationThroughBillingReport,
    pub(crate) move_check_cells: usize,
    pub(crate) player_ranks_initialized: bool,
    pub(crate) goods_war_request: Result<i32, SendMessageError>,
}

#[derive(Debug)]
pub(crate) enum GameInitializationThroughBillingError {
    Setup(GameRuntimeSetupError),
    WorldUnavailable {
        setup: GameRuntimeSetupReport,
        connection: GameClientInitialization,
    },
    Sequence(SequenceRegistryInitializationError),
}

impl fmt::Display for GameInitializationThroughBillingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(error) => error.fmt(formatter),
            Self::WorldUnavailable { .. } => {
                formatter.write_str("GameServer не подключился к обязательному WorldServer")
            }
            Self::Sequence(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for GameInitializationThroughBillingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup(error) => Some(error),
            Self::WorldUnavailable { .. } => None,
            Self::Sequence(error) => Some(error),
        }
    }
}

pub(crate) struct CGame {
    setup: GameSetup,
    setup_ex: GameSetupEx,
    random_state: u32,
    sequence_registry: CSequenceRegistry,
    player_list: CPlayerList,
    trade_list: CTradeList,
    thing_setup: CThingSetup,
    increment_shop_list: CIncrementShopList,
    contribute_setup: CContributeSetup,
    log_system: CLogSystem,
    gm_list: CGMList,
    da_kong_xiang_qian: CDaKongXiangQian,
    region_setup: CRegionSetup,
    hit_level_setup: CHitLevelSetup,
    prison_conf: PrisonConf,
    precious_box_conf: PreciousBoxConf,
    fairy_exp_conf: CFairyExpConf,
    battle_fairy_exp_config: CBattleFairyExpConfig,
    battle_fairy_property: CBattleFairyProperty,
    equipment_compose_list: EquipmentComposeList,
    synthesis: CSynthesis,
    new_skill_monster_conf: NewSkillMonsterConf,
    goods_destroy_setup: GoodsDestroySetup,
    change_body_conf: CChangeBodyConf,
    honor_eliminate_config: HonorElimilateConfig,
    dupli_region_setup: Option<CDupliRegionSetup>,
    move_check_cells: MoveCheckCellRegistry,
    player_ranks: Option<CPlayerRanks>,
    goods_war: Option<CGoodsWarMember>,
    login_server_id: i32,
    world_server_id: i32,
    network_setup: Option<GameNetworkSetup>,
    world_client: Option<CMyNetClient>,
    billing_client: Option<CMyNetClient>,
    net_server: Option<CMyNetServer>,
    world_reconnect_task: Option<GameReconnectTask>,
    billing_reconnect_task: Option<GameReconnectTask>,
    players: BTreeMap<i32, CPlayer>,
    team_session_ids: BTreeMap<u32, i32>,
}

impl CGame {
    /// Создаёт достигнутую process-owned проекцию `CGame` с подтверждёнными
    /// setup defaults; ещё не материализованные gameplay owners не подменяет.
    pub(crate) fn new() -> Self {
        Self {
            setup: GameSetup::default(),
            setup_ex: GameSetupEx::default(),
            random_state: 1,
            sequence_registry: CSequenceRegistry::default(),
            player_list: CPlayerList::default(),
            trade_list: CTradeList::default(),
            thing_setup: CThingSetup::default(),
            increment_shop_list: CIncrementShopList::default(),
            contribute_setup: CContributeSetup::default(),
            log_system: CLogSystem::default(),
            gm_list: CGMList::default(),
            da_kong_xiang_qian: CDaKongXiangQian::default(),
            region_setup: CRegionSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            prison_conf: PrisonConf::default(),
            precious_box_conf: PreciousBoxConf::default(),
            fairy_exp_conf: CFairyExpConf::default(),
            battle_fairy_exp_config: CBattleFairyExpConfig::default(),
            battle_fairy_property: CBattleFairyProperty::default(),
            equipment_compose_list: EquipmentComposeList::default(),
            synthesis: CSynthesis::default(),
            new_skill_monster_conf: NewSkillMonsterConf::default(),
            goods_destroy_setup: GoodsDestroySetup::default(),
            change_body_conf: CChangeBodyConf::default(),
            honor_eliminate_config: HonorElimilateConfig::default(),
            dupli_region_setup: None,
            move_check_cells: MoveCheckCellRegistry::new(),
            player_ranks: None,
            goods_war: None,
            login_server_id: 0,
            world_server_id: 0,
            network_setup: None,
            world_client: None,
            billing_client: None,
            net_server: None,
            world_reconnect_task: None,
            billing_reconnect_task: None,
            players: BTreeMap::new(),
            team_session_ids: BTreeMap::new(),
        }
    }

    pub(crate) fn with_send_state(net_server: CMyNetServer) -> Self {
        let mut game = Self::new();
        game.net_server = Some(net_server);
        game
    }

    /// Создаёт pre-network assembly после уже выполненного `LoadSetup*`.
    pub(crate) fn with_network_setup(network_setup: GameNetworkSetup) -> Self {
        let mut game = Self::new();
        game.network_setup = Some(network_setup);
        game
    }

    /// Читает обязательный `setup.ini`, затем необязательный `setupex.ini` и
    /// публикует единый network plan в исходной позиции перед `InitNetServer`.
    /// Некорректная пара останавливает последующие extraction-ы, сохраняя уже
    /// применённые значения и constructor defaults оставшихся полей.
    pub(crate) fn load_runtime_setup(
        &mut self,
        paths: &GameRuntimePaths,
    ) -> Result<GameRuntimeSetupReport, GameRuntimeSetupError> {
        // Производный plan не должен пережить неуспешную повторную загрузку.
        self.network_setup = None;
        let setup_bytes = fs::read(&paths.setup).map_err(|source| {
            GameRuntimeSetupError::Setup(GameSetupOpenError {
                path: paths.setup.clone(),
                source,
            })
        })?;
        let setup = self.setup.parse_positional(&setup_bytes);

        let setup_ex = match fs::read(&paths.setup_ex) {
            Ok(bytes) => GameSetupExLoad::Loaded(self.setup_ex.parse_positional(&bytes)),
            Err(source) => GameSetupExLoad::Unavailable(GameSetupOpenError {
                path: paths.setup_ex.clone(),
                source,
            }),
        };
        self.network_setup = Some(self.setup.network_setup(&self.setup_ex)?);
        Ok(GameRuntimeSetupReport { setup, setup_ex })
    }

    /// Выполняет достигнутый `Init` от первого RNG seed до Billing-попытки.
    /// World failure завершает цепочку; Billing failure только остаётся в
    /// отчёте, как исходное предупреждение с последующим продолжением.
    pub(crate) async fn init_through_billing(
        &mut self,
        paths: &GameRuntimePaths,
        wall_time_seconds: u32,
        sequence_seed_ms: u32,
    ) -> Result<GameInitializationThroughBillingReport, GameInitializationThroughBillingError> {
        self.random_state = wall_time_seconds;
        let _discarded_roll = game_legacy_random(&mut self.random_state, 100);

        let setup = self
            .load_runtime_setup(paths)
            .map_err(GameInitializationThroughBillingError::Setup)?;
        let world = self.init_world_client().await;
        if matches!(&world, GameClientInitialization::Failed { .. }) {
            return Err(GameInitializationThroughBillingError::WorldUnavailable {
                setup,
                connection: world,
            });
        }

        self.random_state = sequence_seed_ms;
        let sequence_count = self.setup.sequence_count;
        let random_state = &mut self.random_state;
        self.sequence_registry
            .initialize(sequence_count, || next_msvc_rand(random_state))
            .map_err(GameInitializationThroughBillingError::Sequence)?;
        let sequence_elements = self.sequence_registry.len();

        let billing = self.init_billing_client().await;
        Ok(GameInitializationThroughBillingReport {
            setup,
            world,
            sequence_elements,
            billing,
        })
    }

    /// Завершает точный хвост `CGame::Init` после Billing-попытки.
    pub(crate) async fn init(
        &mut self,
        paths: &GameRuntimePaths,
        wall_time_seconds: u32,
        sequence_seed_ms: u32,
    ) -> Result<GameInitializationReport, GameInitializationThroughBillingError> {
        let through_billing = self
            .init_through_billing(paths, wall_time_seconds, sequence_seed_ms)
            .await?;

        self.dupli_region_setup = Some(CDupliRegionSetup::default());
        self.move_check_cells.initialize();
        let move_check_cells = self.move_check_cells.total_len();

        let mut player_ranks = CPlayerRanks::new();
        let player_ranks_initialized = player_ranks.initialize();
        self.player_ranks = Some(player_ranks);

        let goods_war = CGoodsWarMember::new();
        let goods_war_request = goods_war.request_initial_state(self);
        self.goods_war = Some(goods_war);

        Ok(GameInitializationReport {
            through_billing,
            move_check_cells,
            player_ranks_initialized,
            goods_war_request,
        })
    }

    pub(crate) fn net_server(&self) -> &CMyNetServer {
        self.net_server
            .as_ref()
            .expect("Game send-family достигается после InitNetServer")
    }

    /// Возвращает опубликованный listener-owner фактическому network runtime.
    pub(crate) fn current_net_server_mut(&mut self) -> Option<&mut CMyNetServer> {
        self.net_server.as_mut()
    }

    pub(crate) const fn world_client(&self) -> Option<&CMyNetClient> {
        self.world_client.as_ref()
    }

    pub(crate) const fn billing_client(&self) -> Option<&CMyNetClient> {
        self.billing_client.as_ref()
    }

    /// Сохраняет assignment `s_pNetClientOfWS`, затем exact server type writer.
    pub(crate) fn attach_world_client(&mut self, client: CMyNetClient) -> Option<CMyNetClient> {
        let previous = self.world_client.replace(client);
        self.world_client
            .as_mut()
            .expect("World client только что присвоен")
            .set_server_type(ServerType::World);
        previous
    }

    /// Сохраняет assignment `s_pNetClientOfBS`, затем exact server type writer.
    pub(crate) fn attach_billing_client(&mut self, client: CMyNetClient) -> Option<CMyNetClient> {
        let previous = self.billing_client.replace(client);
        self.billing_client
            .as_mut()
            .expect("Billing client только что присвоен")
            .set_server_type(ServerType::Billing);
        previous
    }

    /// Закрывает прежний Billing owner до присваивания reconnect replacement.
    pub(crate) fn replace_billing_client(&mut self, client: CMyNetClient) -> bool {
        let mut previous = self.billing_client.take();
        if let Some(previous) = previous.as_mut() {
            let _legacy_result = previous.close();
        }
        let previous_closed = previous.is_some();
        drop(previous);
        self.attach_billing_client(client);
        previous_closed
    }

    pub(crate) fn current_billing_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        self.billing_client.as_mut()
    }

    /// Сохраняет обе identity из terminal startup-пакета после попытки Host.
    pub(crate) const fn set_server_ids(&mut self, login_server_id: i32, world_server_id: i32) {
        self.login_server_id = login_server_id;
        self.world_server_id = world_server_id;
    }

    pub(crate) const fn server_ids(&self) -> (i32, i32) {
        (self.login_server_id, self.world_server_id)
    }

    pub(crate) const fn sequence_registry(&self) -> &CSequenceRegistry {
        &self.sequence_registry
    }

    pub(crate) const fn player_list(&self) -> &CPlayerList {
        &self.player_list
    }

    pub(crate) const fn player_list_mut(&mut self) -> &mut CPlayerList {
        &mut self.player_list
    }

    pub(crate) const fn trade_list(&self) -> &CTradeList {
        &self.trade_list
    }

    pub(crate) const fn trade_list_mut(&mut self) -> &mut CTradeList {
        &mut self.trade_list
    }

    pub(crate) const fn thing_setup(&self) -> &CThingSetup {
        &self.thing_setup
    }

    pub(crate) const fn thing_setup_mut(&mut self) -> &mut CThingSetup {
        &mut self.thing_setup
    }

    pub(crate) const fn increment_shop_list_mut(&mut self) -> &mut CIncrementShopList {
        &mut self.increment_shop_list
    }

    pub(crate) const fn contribute_setup_mut(&mut self) -> &mut CContributeSetup {
        &mut self.contribute_setup
    }

    pub(crate) const fn log_system_mut(&mut self) -> &mut CLogSystem {
        &mut self.log_system
    }

    pub(crate) const fn gm_list_mut(&mut self) -> &mut CGMList {
        &mut self.gm_list
    }

    pub(crate) const fn da_kong_xiang_qian_mut(&mut self) -> &mut CDaKongXiangQian {
        &mut self.da_kong_xiang_qian
    }

    pub(crate) const fn region_setup_mut(&mut self) -> &mut CRegionSetup {
        &mut self.region_setup
    }

    pub(crate) const fn hit_level_setup_mut(&mut self) -> &mut CHitLevelSetup {
        &mut self.hit_level_setup
    }

    pub(crate) const fn prison_conf_mut(&mut self) -> &mut PrisonConf {
        &mut self.prison_conf
    }

    pub(crate) const fn precious_box_conf_mut(&mut self) -> &mut PreciousBoxConf {
        &mut self.precious_box_conf
    }

    pub(crate) const fn fairy_exp_conf_mut(&mut self) -> &mut CFairyExpConf {
        &mut self.fairy_exp_conf
    }

    pub(crate) const fn battle_fairy_exp_config_mut(&mut self) -> &mut CBattleFairyExpConfig {
        &mut self.battle_fairy_exp_config
    }

    pub(crate) const fn battle_fairy_property_mut(&mut self) -> &mut CBattleFairyProperty {
        &mut self.battle_fairy_property
    }

    pub(crate) const fn equipment_compose_list(&self) -> &EquipmentComposeList {
        &self.equipment_compose_list
    }

    pub(crate) const fn equipment_compose_list_mut(&mut self) -> &mut EquipmentComposeList {
        &mut self.equipment_compose_list
    }

    pub(crate) const fn synthesis_mut(&mut self) -> &mut CSynthesis {
        &mut self.synthesis
    }

    pub(crate) const fn new_skill_monster_conf_mut(&mut self) -> &mut NewSkillMonsterConf {
        &mut self.new_skill_monster_conf
    }

    pub(crate) const fn goods_destroy_setup_mut(&mut self) -> &mut GoodsDestroySetup {
        &mut self.goods_destroy_setup
    }

    pub(crate) const fn change_body_conf_mut(&mut self) -> &mut CChangeBodyConf {
        &mut self.change_body_conf
    }

    pub(crate) const fn honor_eliminate_config_mut(&mut self) -> &mut HonorElimilateConfig {
        &mut self.honor_eliminate_config
    }

    pub(crate) const fn dupli_region_setup(&self) -> Option<&CDupliRegionSetup> {
        self.dupli_region_setup.as_ref()
    }

    pub(crate) const fn dupli_region_setup_mut(&mut self) -> Option<&mut CDupliRegionSetup> {
        self.dupli_region_setup.as_mut()
    }

    pub(crate) const fn move_check_cells(&self) -> &MoveCheckCellRegistry {
        &self.move_check_cells
    }

    pub(crate) const fn player_ranks(&self) -> Option<&CPlayerRanks> {
        self.player_ranks.as_ref()
    }

    pub(crate) const fn player_ranks_mut(&mut self) -> Option<&mut CPlayerRanks> {
        self.player_ranks.as_mut()
    }

    pub(crate) const fn goods_war(&self) -> Option<&CGoodsWarMember> {
        self.goods_war.as_ref()
    }

    pub(crate) const fn goods_war_mut(&mut self) -> Option<&mut CGoodsWarMember> {
        self.goods_war.as_mut()
    }

    /// Публикует listener-owner до `Host`, затем сохраняет setup-порядок.
    pub(crate) fn init_net_server(
        &mut self,
        now_ms: u32,
    ) -> Result<(), GameNetworkInitializationError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameNetworkInitializationError::MissingNetworkSetup)?;
        self.net_server = Some(CMyNetServer::new(now_ms));
        let server = self
            .net_server
            .as_mut()
            .expect("Game server owner только что опубликован");
        server
            .host(setup.listener.listen_port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(GameNetworkInitializationError::Host)?;

        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }
        server.configure_after_host(
            setup.listener.check_network,
            setup.listener.maximum_in_flight_sends,
            setup.listener.maximum_bytes_per_second,
            setup.listener.maximum_clients,
            setup.check_message_content,
            setup.listener.forbid_time_ms,
            setup.listener.maximum_message_length,
            setup.listener.permitted_send_bytes,
        );
        server.configure_accept_limits_after_host(
            setup.listener.maximum_block_connections,
            setup.listener.first_receive_timeout_ms,
        );
        Ok(())
    }

    /// Пересоздаёт initial World client и ставит исходную регистрацию.
    pub(crate) async fn init_world_client(&mut self) -> GameClientInitialization {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return GameClientInitialization::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
                }],
            };
        };
        self.close_and_remove_world_client();
        self.attach_world_client(CMyNetClient::new());

        let mut attempts = Vec::with_capacity(1);
        let result = connect_game_client(
            self.world_client
                .as_mut()
                .expect("World client только что опубликован"),
            &setup.world,
            GameUpstreamDirection::World,
            None,
            0,
        )
        .await;
        let endpoint = match result {
            Ok(endpoint) => {
                attempts.push(GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: None,
                });
                endpoint
            }
            Err(failure) => {
                attempts.push(GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(failure),
                });
                self.close_and_remove_world_client();
                return GameClientInitialization::Failed { attempts };
            }
        };

        self.world_client
            .as_mut()
            .expect("подключённый World client остаётся опубликованным")
            .enable_control_send();
        let mut registration = CMessage::new(WORLD_REGISTRATION);
        registration.add_byte(0);
        registration.add_ulong(setup.listener.listen_port);
        add_legacy_c_string(registration.base_mut(), &setup.local_ip);
        let registration = registration.send(self, false);
        GameClientInitialization::Connected {
            endpoint,
            used_billing_backup: false,
            registration,
            attempts,
        }
    }

    /// Пересоздаёт Billing client, пробуя master и затем backup endpoint.
    pub(crate) async fn init_billing_client(&mut self) -> GameClientInitialization {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return GameClientInitialization::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::BillingPrimary,
                    failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
                }],
            };
        };
        self.close_and_remove_billing_client();
        self.attach_billing_client(CMyNetClient::new());

        let bind_ip = match resolve_billing_bind_ipv4(&setup.billing.bind_ip) {
            Ok(address) => address,
            Err(failure) => {
                self.close_and_remove_billing_client();
                return GameClientInitialization::Failed {
                    attempts: vec![GameConnectAttempt {
                        direction: GameUpstreamDirection::BillingPrimary,
                        failure: Some(failure),
                    }],
                };
            }
        };
        let mut attempts = Vec::with_capacity(2);
        let plans = [
            (
                &setup.billing.primary,
                GameUpstreamDirection::BillingPrimary,
            ),
            (&setup.billing.backup, GameUpstreamDirection::BillingBackup),
        ];
        let mut connected = None;
        for (endpoint_plan, direction) in plans {
            let result = connect_game_client(
                self.billing_client
                    .as_mut()
                    .expect("Billing client остаётся опубликованным между попытками"),
                endpoint_plan,
                direction,
                Some(bind_ip),
                setup.billing.bind_port,
            )
            .await;
            match result {
                Ok(endpoint) => {
                    attempts.push(GameConnectAttempt {
                        direction,
                        failure: None,
                    });
                    connected = Some((endpoint, direction));
                    break;
                }
                Err(failure) => attempts.push(GameConnectAttempt {
                    direction,
                    failure: Some(failure),
                }),
            }
        }
        let Some((endpoint, direction)) = connected else {
            self.close_and_remove_billing_client();
            return GameClientInitialization::Failed { attempts };
        };

        self.billing_client
            .as_mut()
            .expect("подключённый Billing client остаётся опубликованным")
            .enable_control_send();
        let registration = CMessage::new(BILLING_REGISTRATION).send_to_bs(self, false);
        GameClientInitialization::Connected {
            endpoint,
            used_billing_backup: direction == GameUpstreamDirection::BillingBackup,
            registration,
            attempts,
        }
    }

    /// Подключает новый World owner и публикует typed reconnect event.
    pub(crate) async fn reconnect_world_server(&self) -> GameReconnectPublication {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return missing_setup_reconnect(GameUpstreamDirection::World);
        };
        let publisher = self.net_server.as_ref().map(CMyNetServer::event_publisher);
        reconnect_world_with(setup, publisher).await
    }

    /// Подключает новый Billing owner и публикует typed reconnect event.
    pub(crate) async fn reconnect_billing_server(&self) -> GameReconnectPublication {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return missing_setup_reconnect(GameUpstreamDirection::BillingPrimary);
        };
        let publisher = self.net_server.as_ref().map(CMyNetServer::event_publisher);
        reconnect_billing_with(setup, publisher).await
    }

    /// Останавливает прежний World worker, закрывает текущий канал и запускает retry entry.
    pub(crate) async fn create_connect_world_task(
        &mut self,
    ) -> Result<(), GameReconnectTaskStartError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameReconnectTaskStartError::MissingNetworkSetup)?;
        let publisher = self
            .net_server
            .as_ref()
            .map(CMyNetServer::event_publisher)
            .ok_or(GameReconnectTaskStartError::MissingNetworkServerOwner)?;

        stop_reconnect_task(&mut self.world_reconnect_task).await;
        if let Some(client) = self.world_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }

        let exit_requested = Arc::new(AtomicBool::new(false));
        let worker_exit = Arc::clone(&exit_requested);
        let handle =
            tokio::spawn(
                async move { run_world_reconnect_task(setup, publisher, worker_exit).await },
            );
        self.world_reconnect_task = Some(GameReconnectTask {
            exit_requested,
            handle,
        });
        Ok(())
    }

    /// Останавливает прежний Billing worker, закрывает текущий канал и запускает retry entry.
    pub(crate) async fn create_connect_billing_task(
        &mut self,
    ) -> Result<(), GameReconnectTaskStartError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameReconnectTaskStartError::MissingNetworkSetup)?;
        let publisher = self
            .net_server
            .as_ref()
            .map(CMyNetServer::event_publisher)
            .ok_or(GameReconnectTaskStartError::MissingNetworkServerOwner)?;

        stop_reconnect_task(&mut self.billing_reconnect_task).await;
        if let Some(client) = self.billing_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }

        let exit_requested = Arc::new(AtomicBool::new(false));
        let worker_exit = Arc::clone(&exit_requested);
        let handle =
            tokio::spawn(
                async move { run_billing_reconnect_task(setup, publisher, worker_exit).await },
            );
        self.billing_reconnect_task = Some(GameReconnectTask {
            exit_requested,
            handle,
        });
        Ok(())
    }

    /// Материализует начальный порядок остановки reconnect workers из `Release`.
    pub(crate) async fn stop_reconnect_tasks(&mut self) {
        stop_reconnect_task(&mut self.world_reconnect_task).await;
        stop_reconnect_task(&mut self.billing_reconnect_task).await;
    }

    /// Выполняет один awaitable I/O шаг текущего World направления.
    pub(crate) async fn run_world_io_once(
        &mut self,
    ) -> Result<GameClientIoStep, GameClientIoError> {
        self.world_client
            .as_mut()
            .ok_or(GameClientIoError::NotConnected)?
            .run_io_once()
            .await
    }

    /// Выполняет один awaitable I/O шаг текущего Billing направления.
    pub(crate) async fn run_billing_io_once(
        &mut self,
    ) -> Result<GameClientIoStep, GameClientIoError> {
        self.billing_client
            .as_mut()
            .ok_or(GameClientIoError::NotConnected)?
            .run_io_once()
            .await
    }

    fn close_and_remove_world_client(&mut self) -> bool {
        let mut client = self.world_client.take();
        if let Some(client) = client.as_mut() {
            let _legacy_result = client.close();
        }
        client.is_some()
    }

    fn close_and_remove_billing_client(&mut self) -> bool {
        let mut client = self.billing_client.take();
        if let Some(client) = client.as_mut() {
            let _legacy_result = client.close();
        }
        client.is_some()
    }

    pub(crate) fn register_player(&mut self, player: CPlayer) -> Option<CPlayer> {
        self.players.insert(player.player_id(), player)
    }

    pub(crate) fn register_team_session(&mut self, team_id: u32, session_id: i32) -> Option<i32> {
        self.team_session_ids.insert(team_id, session_id)
    }

    pub(crate) fn get_team_session_id(&self, team_id: u32) -> i32 {
        self.team_session_ids.get(&team_id).copied().unwrap_or(0)
    }

    pub(crate) fn find_player(&self, player_id: i32) -> Option<&CPlayer> {
        self.players.get(&player_id)
    }

    /// Исполняет один исходный snapshot входящих FIFO в порядке WS, BS, GS.
    pub(crate) fn process_messages(&mut self, handlers: &mut dyn GameMessageHandlers) -> i32 {
        if let Some(client) = &self.world_client {
            for mut message in client.take_all_messages() {
                message.run(self, handlers);
            }
        }
        if let Some(client) = &self.billing_client {
            for mut message in client.take_all_messages() {
                message.run(self, handlers);
            }
        }
        let server_events = self
            .net_server
            .as_ref()
            .map(CMyNetServer::take_all_events)
            .unwrap_or_default();
        for event in server_events {
            match event {
                GameServerEvent::Message(mut message) => {
                    message.run(self, handlers);
                }
                GameServerEvent::WorldClientReconnected(client) => {
                    handlers.handle_world_client_reconnected(self, client);
                }
                GameServerEvent::BillingClientReconnected(client) => {
                    let _legacy_ignored = on_billing_client_reconnected(self, client);
                }
            }
        }
        1
    }
}

impl Default for CGame {
    fn default() -> Self {
        Self::new()
    }
}

fn missing_setup_reconnect(direction: GameUpstreamDirection) -> GameReconnectPublication {
    GameReconnectPublication::Failed {
        attempts: vec![GameConnectAttempt {
            direction,
            failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
        }],
    }
}

fn next_msvc_rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(214_013).wrapping_add(2_531_011);
    (*state >> 16) & 0x7fff
}

fn game_legacy_random(state: &mut u32, upper_bound: i32) -> i32 {
    if upper_bound <= 0 {
        return 0;
    }
    loop {
        let value = (i64::from(next_msvc_rand(state)) * i64::from(upper_bound) / 0x7fff) as i32;
        if value != upper_bound || value <= 0 {
            return value;
        }
    }
}

async fn stop_reconnect_task(task: &mut Option<GameReconnectTask>) {
    let Some(task) = task.take() else {
        return;
    };
    task.exit_requested.store(true, Ordering::Release);
    let _legacy_ignored = task.handle.await;
}

async fn run_world_reconnect_task(
    setup: GameNetworkSetup,
    publisher: GameServerEventPublisher,
    exit_requested: Arc<AtomicBool>,
) -> GameReconnectWorkerEnd {
    loop {
        if exit_requested.load(Ordering::Acquire) {
            return GameReconnectWorkerEnd::ExitRequested;
        }
        tokio::time::sleep(RECONNECT_RETRY_DELAY).await;
        if matches!(
            reconnect_world_with(setup.clone(), Some(publisher.clone())).await,
            GameReconnectPublication::Published { .. }
        ) {
            return GameReconnectWorkerEnd::Published;
        }
    }
}

async fn run_billing_reconnect_task(
    setup: GameNetworkSetup,
    publisher: GameServerEventPublisher,
    exit_requested: Arc<AtomicBool>,
) -> GameReconnectWorkerEnd {
    loop {
        if exit_requested.load(Ordering::Acquire) {
            return GameReconnectWorkerEnd::ExitRequested;
        }
        tokio::time::sleep(RECONNECT_RETRY_DELAY).await;
        if matches!(
            reconnect_billing_with(setup.clone(), Some(publisher.clone())).await,
            GameReconnectPublication::Published { .. }
        ) {
            return GameReconnectWorkerEnd::Published;
        }
    }
}

async fn reconnect_world_with(
    setup: GameNetworkSetup,
    publisher: Option<GameServerEventPublisher>,
) -> GameReconnectPublication {
    let Some(publisher) = publisher else {
        return GameReconnectPublication::Failed {
            attempts: vec![GameConnectAttempt {
                direction: GameUpstreamDirection::World,
                failure: Some(GameClientInitializationFailure::MissingNetworkServerOwner),
            }],
        };
    };
    let mut client = CMyNetClient::new();
    client.set_server_type(ServerType::World);
    let endpoint = match connect_game_client(
        &mut client,
        &setup.world,
        GameUpstreamDirection::World,
        None,
        0,
    )
    .await
    {
        Ok(endpoint) => endpoint,
        Err(failure) => {
            let _legacy_result = client.close();
            return GameReconnectPublication::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(failure),
                }],
            };
        }
    };
    publisher.publish_reconnected_world_client(client);
    GameReconnectPublication::Published {
        endpoint,
        used_billing_backup: false,
        attempts: vec![GameConnectAttempt {
            direction: GameUpstreamDirection::World,
            failure: None,
        }],
    }
}

async fn reconnect_billing_with(
    setup: GameNetworkSetup,
    publisher: Option<GameServerEventPublisher>,
) -> GameReconnectPublication {
    let Some(publisher) = publisher else {
        return GameReconnectPublication::Failed {
            attempts: vec![GameConnectAttempt {
                direction: GameUpstreamDirection::BillingPrimary,
                failure: Some(GameClientInitializationFailure::MissingNetworkServerOwner),
            }],
        };
    };
    let bind_ip = match resolve_billing_bind_ipv4(&setup.billing.bind_ip) {
        Ok(address) => address,
        Err(failure) => {
            return GameReconnectPublication::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::BillingPrimary,
                    failure: Some(failure),
                }],
            };
        }
    };
    let mut client = CMyNetClient::new();
    client.set_server_type(ServerType::Billing);
    let mut attempts = Vec::with_capacity(2);
    let mut connected = None;
    for (endpoint_plan, direction) in [
        (
            &setup.billing.primary,
            GameUpstreamDirection::BillingPrimary,
        ),
        (&setup.billing.backup, GameUpstreamDirection::BillingBackup),
    ] {
        match connect_game_client(
            &mut client,
            endpoint_plan,
            direction,
            Some(bind_ip),
            setup.billing.bind_port,
        )
        .await
        {
            Ok(endpoint) => {
                attempts.push(GameConnectAttempt {
                    direction,
                    failure: None,
                });
                connected = Some((endpoint, direction));
                break;
            }
            Err(failure) => attempts.push(GameConnectAttempt {
                direction,
                failure: Some(failure),
            }),
        }
    }
    let Some((endpoint, direction)) = connected else {
        let _legacy_result = client.close();
        return GameReconnectPublication::Failed { attempts };
    };
    publisher.publish_reconnected_billing_client(client);
    GameReconnectPublication::Published {
        endpoint,
        used_billing_backup: direction == GameUpstreamDirection::BillingBackup,
        attempts,
    }
}

async fn connect_game_client(
    client: &mut CMyNetClient,
    endpoint_plan: &GameUpstreamEndpoint,
    direction: GameUpstreamDirection,
    bind_ip: Option<Ipv4Addr>,
    bind_port: u32,
) -> Result<SocketAddrV4, GameClientInitializationFailure> {
    let socket =
        bind_tcp_ipv4(bind_ip, bind_port).map_err(GameClientInitializationFailure::Bind)?;
    let endpoint = resolve_game_endpoint(endpoint_plan, direction)?;
    client
        .connect(socket, endpoint)
        .await
        .map_err(|source| GameClientInitializationFailure::Connect { direction, source })?;
    Ok(endpoint)
}

fn resolve_game_endpoint(
    endpoint: &GameUpstreamEndpoint,
    direction: GameUpstreamDirection,
) -> Result<SocketAddrV4, GameClientInitializationFailure> {
    let host = legacy_c_string_prefix(&endpoint.host);
    if host.len() > 63 {
        // BLOCKED_MISSING_FACT: Game CClient::Connect RVA 0x00019CE0 копировал
        // hostname без проверки в `char[64]`; переполнение не имитируется.
        return Err(GameClientInitializationFailure::AddressTooLong {
            direction,
            length: host.len(),
        });
    }
    let host = std::str::from_utf8(host)
        .map_err(|_| GameClientInitializationFailure::AddressEncodingUnsupported { direction })?;
    (host, endpoint.port as u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| {
            addresses.find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
        })
        .ok_or(GameClientInitializationFailure::AddressResolution { direction })
}

fn resolve_billing_bind_ipv4(raw: &[u8]) -> Result<Ipv4Addr, GameClientInitializationFailure> {
    std::str::from_utf8(legacy_c_string_prefix(raw))
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(GameClientInitializationFailure::BillingBindAddressResolution)
}

fn add_legacy_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn resolve_first_local_ipv4() -> Option<Ipv4Addr> {
    let hostname = uname();
    let hostname = hostname.nodename().to_str().ok()?;
    (hostname, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|address| match address {
            SocketAddr::V4(address) => Some(*address.ip()),
            SocketAddr::V6(_) => None,
        })
}

impl ShapeResolver for CGame {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        if identity.object_type != PLAYER_TYPE {
            return None;
        }
        let player = self.find_player(identity.id)?;
        let view = player.shape_view()?;
        (view.identity.object_type == identity.object_type && view.identity.id == identity.id)
            .then_some(view)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h

// ============================================================================
// FUNCTION: Catch@00401323
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x00001323
// ADDRESS: 00401323
// PROTOTYPE: undefined Catch@00401323()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040134e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0000134E
// ADDRESS: 0040134e
// PROTOTYPE: undefined FUN_0040134e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004013bd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x000013BD
// ADDRESS: 004013bd
// PROTOTYPE: undefined Catch@004013bd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `InitNetServer` материализован выше;
// exact эпилог возвращает `0` после Host-error и `1` после setup-записей.

// ============================================================================
// FUNCTION: CGame::KickPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:980
// RVA: 0x000022F0
// ADDRESS: 004022f0
// PROTOTYPE: bool __thiscall KickPlayer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SetFunctionFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1017
// RVA: 0x00002310
// ADDRESS: 00402310
// PROTOTYPE: void __thiscall SetFunctionFileData(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SetVariableFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1025
// RVA: 0x00002340
// ADDRESS: 00402340
// PROTOTYPE: void __thiscall SetVariableFileData(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SetGeneralVariableFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1033
// RVA: 0x00002370
// ADDRESS: 00402370
// PROTOTYPE: void __thiscall SetGeneralVariableFileData(uchar * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SetAuctionState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1439
// RVA: 0x00002390
// ADDRESS: 00402390
// PROTOTYPE: void __thiscall SetAuctionState(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `InitNetClientOfWS/BS` материализованы выше с исходными
// registration packets и master -> backup Billing порядком.

// ============================================================================
// FUNCTION: CGame::CreateStringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1361
// RVA: 0x00003000
// ADDRESS: 00403000
// PROTOTYPE: bool __thiscall CreateStringTable(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:948
// RVA: 0x00003A60
// ADDRESS: 00403a60
// PROTOTYPE: CPlayer * __thiscall FindPlayer(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindPlayerByAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:964
// RVA: 0x00003B20
// ADDRESS: 00403b20
// PROTOTYPE: CPlayer * __thiscall FindPlayerByAccount(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SendTopInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1105
// RVA: 0x00003BE0
// ADDRESS: 00403be0
// PROTOTYPE: void __thiscall SendTopInfoToClient(long param_1, long param_2, long param_3, long param_4, basic_string<char,std::char_traits<char>,std::allocator<char>_> param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReCollectBaiTanInGs
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1480
// RVA: 0x00003CB0
// ADDRESS: 00403cb0
// PROTOTYPE: void __thiscall ReCollectBaiTanInGs(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00403cfa
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1493
// RVA: 0x00003CFA
// ADDRESS: 00403cfa
// PROTOTYPE: undefined Catch@00403cfa()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00403d13
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1483
// RVA: 0x00003D13
// ADDRESS: 00403d13
// PROTOTYPE: undefined FUN_00403d13()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:742
// RVA: 0x00005080
// ADDRESS: 00405080
// PROTOTYPE: int __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1001
// RVA: 0x000050F0
// ADDRESS: 004050f0
// PROTOTYPE: CServerRegion * __thiscall FindRegion(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SaveCityRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1085
// RVA: 0x000051B0
// ADDRESS: 004051b0
// PROTOTYPE: void __thiscall SaveCityRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `ProcessMessage` материализован выше; static scratch deque
// заменён тремя последовательными owned snapshot-очередями.

// ============================================================================
// FUNCTION: CGame::tagSetup::tagSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:166
// RVA: 0x00008810
// ADDRESS: 00408810
// PROTOTYPE: void __thiscall tagSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::RemoveSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1221
// RVA: 0x00008970
// ADDRESS: 00408970
// PROTOTYPE: void __thiscall RemoveSequenceMap(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CleanSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1237
// RVA: 0x000089D0
// ADDRESS: 004089d0
// PROTOTYPE: void __thiscall CleanSequenceMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadSetupEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:189
// RVA: 0x00009160
// ADDRESS: 00409160
// PROTOTYPE: bool __thiscall LoadSetupEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReloadSetupEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:207
// RVA: 0x00009310
// ADDRESS: 00409310
// PROTOTYPE: bool __thiscall ReloadSetupEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AppendSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1203
// RVA: 0x00009340
// ADDRESS: 00409340
// PROTOTYPE: bool __thiscall AppendSequenceMap(uint param_1, CSequenceString * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AppendValidateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1280
// RVA: 0x000093D0
// ADDRESS: 004093d0
// PROTOTYPE: void __thiscall AppendValidateTime(uint param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:116
// RVA: 0x00009960
// ADDRESS: 00409960
// PROTOTYPE: bool __thiscall LoadSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReLoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:178
// RVA: 0x00009DD0
// ADDRESS: 00409dd0
// PROTOTYPE: bool __thiscall ReLoadSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::Init
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:217
// RVA: 0x00009E70
// ADDRESS: 00409e70
// PROTOTYPE: int __thiscall Init(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:293
// RVA: 0x00009FD0
// ADDRESS: 00409fd0
// PROTOTYPE: int __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a090
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:326
// RVA: 0x0000A090
// ADDRESS: 0040a090
// PROTOTYPE: undefined FUN_0040a090()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a15b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:341
// RVA: 0x0000A15B
// ADDRESS: 0040a15b
// PROTOTYPE: undefined Catch@0040a15b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a2a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:379
// RVA: 0x0000A2A0
// ADDRESS: 0040a2a0
// PROTOTYPE: undefined FUN_0040a2a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a2d3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:384
// RVA: 0x0000A2D3
// ADDRESS: 0040a2d3
// PROTOTYPE: undefined Catch@0040a2d3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a370
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:394
// RVA: 0x0000A370
// ADDRESS: 0040a370
// PROTOTYPE: undefined FUN_0040a370()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a3aa
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:399
// RVA: 0x0000A3AA
// ADDRESS: 0040a3aa
// PROTOTYPE: undefined Catch@0040a3aa()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a48a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:415
// RVA: 0x0000A48A
// ADDRESS: 0040a48a
// PROTOTYPE: undefined Catch@0040a48a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::RefreashAllMonsterBaseProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1052
// RVA: 0x0000A890
// ADDRESS: 0040a890
// PROTOTYPE: void __thiscall RefreashAllMonsterBaseProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::RemoveValidateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1293
// RVA: 0x0000AA40
// ADDRESS: 0040aa40
// PROTOTYPE: void __thiscall RemoveValidateTime(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AddRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:986
// RVA: 0x0000ACF0
// ADDRESS: 0040acf0
// PROTOTYPE: void __thiscall AddRegion(long param_1, CServerRegion * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AddProxyRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:991
// RVA: 0x0000AD10
// ADDRESS: 0040ad10
// PROTOTYPE: void __thiscall AddProxyRegion(long param_1, CProxyServerRegion * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindProxyRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:996
// RVA: 0x0000AD30
// ADDRESS: 0040ad30
// PROTOTYPE: CServerRegion * __thiscall FindProxyRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::~CGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:110
// RVA: 0x0000AF50
// ADDRESS: 0040af50
// PROTOTYPE: void __thiscall ~CGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:83
// RVA: 0x0000B260
// ADDRESS: 0040b260
// PROTOTYPE: void __thiscall CGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::SetScriptFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1038
// RVA: 0x0000B4B0
// ADDRESS: 0040b4b0
// PROTOTYPE: void __thiscall SetScriptFileData(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:76
// RVA: 0x0000B750
// ADDRESS: 0040b750
// PROTOTYPE: CGame * __cdecl GetGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: обе `ReConnect*` функции и retry thread-entry материализованы
// выше; C++ exception funclets заменены typed outcomes Rust.

// ============================================================================
// FUNCTION: CGame::RunAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1397
// RVA: 0x0000BC30
// ADDRESS: 0040bc30
// PROTOTYPE: void __thiscall RunAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CreateConnectWorldThread` RVA `0x0000BDA0` и
// `CreateConnectBillingThread` RVA `0x0000BE10` материализованы выше как owned
// Tokio tasks; начальный stop/join обеих задач из `Release` также перенесён.

// ============================================================================
// FUNCTION: CGame::MainLoop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:827
// RVA: 0x0000BE80
// ADDRESS: 0040be80
// PROTOTYPE: int __thiscall MainLoop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GameThreadFunc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1151
// RVA: 0x0000C130
// ADDRESS: 0040c130
// PROTOTYPE: uint __cdecl GameThreadFunc(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `GetTeamSessionID` и `FindPlayer` материализованы выше;
// покрытые raw-блоки удалены.

// ============================================================================
// FUNCTION: GetStringByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:417
// RVA: 0x0001D5F0
// ADDRESS: 0041d5f0
// PROTOTYPE: char * __cdecl GetStringByID(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetScriptFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:107
// RVA: 0x00028C00
// ADDRESS: 00428c00
// PROTOTYPE: char * __thiscall GetScriptFileData(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:91
// RVA: 0x00040BF0
// ADDRESS: 00440bf0
// PROTOTYPE: CServerRegion * __thiscall FindRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetTeamID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:284
// RVA: 0x0008BEB0
// ADDRESS: 0048beb0
// PROTOTYPE: ulong __thiscall GetTeamID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L82323
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3A0
// ADDRESS: 0062a3a0
// PROTOTYPE: undefined $L82323()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L83447
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3C0
// ADDRESS: 0062a3c0
// PROTOTYPE: undefined $L83447()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L83127
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3E0
// ADDRESS: 0062a3e0
// PROTOTYPE: undefined $L83127()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0024A5F0
// ADDRESS: 0064a5f0
// PROTOTYPE: void __cdecl $E2(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0024A600
// ADDRESS: 0064a600
// PROTOTYPE: void __cdecl $E5(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
