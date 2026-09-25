//! Наблюдаемые outcome/report-типы диспетчера server-сообщений WorldServer
//! (`appworld/message/servermessage.cpp`), подтверждённые `worldserver.exe` и
//! `worldserver.pdb`. Здесь только data-контракты без связей с владельцем игры;
//! сам диспетчер `OnServerMessage`, decode-helpers и fn-обработчики остаются в
//! старом `appworld/message/servermessage.rs`, который реэкспортирует эти типы
//! для переходных потребителей.
//!
//! Вторая волна добавила типы, чьи поля цитируют data-контракты бывшего
//! `worldserver/worldserver/game.rs`: сами контракты (CD-key snapshot, reconnect
//! records, region transition/snapshot, save-пайплайн, ping/region decode)
//! теперь в `crate::app::worldserver`. Финальная типовая волна добавила
//! диспетчерную связку
//! `WorldLoginServerClosed`/`WorldServerMessageOutcome`/`WorldServerMessageDispatch`
//! (reconnect restart-итоги живут в `crate::app::loginreconnectworker`, network
//! `CMessage` — в `crate::app::world_message`) и пару
//! `WorldCountryHandlerConfiguration*` (`CountryHandlerSerializeError` переехал
//! к `CountrySerializeError` в `crate::organizations`). Формы wire-полей и
//! смысл disposition-вариантов не меняются.

use std::error::Error;
use std::fmt;

use crate::activities::fournationwarsys::FourNationWarSerializationBlock;
use crate::activities::rsgodsbattle::RsGodsBattleNotice;
use crate::app::loginreconnectworker::WorldLoginReconnectThreadRestart;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::worldserver::{
    AddLogTextDisposition, WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldGenerateDbDataBlock,
    WorldGenerateDbDataReport, WorldGlobeVariablesDelivery, WorldInitialRegionSnapshotBlock,
    WorldInitialRegionSnapshotKind, WorldOnlinePlayerAppendOutcome, WorldPingGameServerInfo,
    WorldPlayerSaveResponseProgress, WorldReceivedPlayerDataRead, WorldReceivedPlayerDataUpdate,
    WorldReconnectedPlayerDecode, WorldRegionChangePlayerTransition, WorldRegionChangeTeamUpdate,
    WorldRegionParamDecodeOutcome, WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest,
    WorldServerSnapshotPlayerDecode,
};
use crate::characters::honordb::HonorRanksType;
use crate::characters::honorranks::HonorRanksSerializationBlock;
use crate::characters::player::{PlayerCodecError, PlayerMurderCounterUpdate};
use crate::characters::playerranks::PlayerRanksSerializationBlock;
use crate::content::battlefairyproperty::BattleFairyComposeWireError;
use crate::content::cgoodsfactory::GoodsRegistrySerializeError;
use crate::content::countryparam::CountryParamSerializationBlock;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::skillfactory::SkillFactorySerializeError;
use crate::content::variablelist::{VariableListSerializationBlock, VariableSetOutcome};
use crate::organizations::countryhandler::CountryHandlerSerializeError;
use nebokrai_shared::resources::{
    BattleFairyExpSerializeError, CiQingSerializationBlock, ContributeSetupSerializeError,
    DupliRegionSerializeError, EmotionSerializeError, EquipmentComposeSerializeError,
    GmListSerializationBlock, GodsBattleFactionXydUpdate, GodsBattleNpcFactionUpdate,
    GodsBattleSerializeError, GoodsDestroySerializeError, HitLevelSerializeError,
    IncrementShopSerializeError, LingBaoSerializationBlock, LogSystemSerializeError,
    MonsterListSerializeError, NewSkillMonsterSerializeError, PlayerListSerializeError,
    PreciousBoxSerializeError, PrisonConfSerializeError, QuestSystemSerializationBlock,
    RegionRouterSerializeError, RegionSetupSerializeError, SynthesisSerializeError,
    TaoZhuangSerializationBlock, ThingSetupCodecError, TradeListSerializeError,
    WordsFilterSerializeError,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerConnectedLog {
    pub peer_ipv4: u32,
    pub game_server_index: u32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSpawnRoutingOutcome {
    Rejected {
        source_map_id: i32,
        region_id: Option<i32>,
        reason: &'static str,
    },
    Forwarded {
        source_map_id: i32,
        target_map_id: i32,
        region_id: i32,
        kind: u8,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldSpawnRoutingCommand {
    pub kind: u8,
    pub region_id: i32,
}

#[derive(Debug)]
pub struct WorldGodsBattleMessage {
    pub subtype: i8,
    pub subtype_complete: bool,
    pub disposition: WorldGodsBattleDisposition,
}

#[derive(Debug)]
pub enum WorldGodsBattleDisposition {
    Query {
        faction_a_xyd: u32,
        faction_b_xyd: u32,
        message_type: i32,
        socket_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
    UpdateFaction {
        faction: i32,
        faction_complete: bool,
        xyd: u32,
        xyd_complete: bool,
        update: GodsBattleFactionXydUpdate,
        faction_a_xyd: u32,
        faction_b_xyd: u32,
        response_marker: i8,
        message_type: i32,
        socket_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
    UpdateNpc {
        name: Vec<u8>,
        faction: i32,
        faction_complete: bool,
        update: Option<GodsBattleNpcFactionUpdate>,
        save: WorldGodsBattleNpcSave,
    },
    Ignored,
}

#[derive(Debug)]
pub enum WorldGodsBattleNpcSave {
    DatabaseOwnerUnavailable,
    Completed {
        snapshot_records: usize,
        save_returned: bool,
        notices: Vec<RsGodsBattleNotice>,
    },
}

#[derive(Debug)]
pub struct WorldGodsBattleTopTenMessage {
    pub socket_id: i32,
    pub disposition: WorldGodsBattleTopTenDisposition,
}

#[derive(Debug)]
pub enum WorldGodsBattleTopTenDisposition {
    DatabaseOwnerUnavailable,
    FactionFiveFailed {
        notices: Vec<RsGodsBattleNotice>,
    },
    FactionSixFailed {
        faction_five_payload_bytes: usize,
        notices: Vec<RsGodsBattleNotice>,
    },
    Sent {
        faction_five_payload_bytes: usize,
        faction_six_payload_bytes: usize,
        terminal_marker: i32,
        payload_bytes: usize,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
        notices: Vec<RsGodsBattleNotice>,
    },
}

#[derive(Debug)]
pub struct WorldGeneralVariableUpdate {
    pub variable_type: i32,
    pub type_complete: bool,
    pub name: Vec<u8>,
    pub value: Option<WorldGeneralVariableValue>,
    pub disposition: WorldGeneralVariableUpdateDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldGeneralVariableValue {
    Integer { value: i32, complete: bool },
    String(Vec<u8>),
}

#[derive(Debug)]
pub enum WorldGeneralVariableUpdateDisposition {
    UnsupportedTypeLegacyUndefined,
    VariableListUnavailable,
    MutationRejected(VariableSetOutcome),
    Broadcast {
        mutation: VariableSetOutcome,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRegionChangePrefix {
    pub tile_x: i32,
    pub tile_y: i32,
    pub direction: i32,
    pub use_goods: i32,
    pub range: i32,
    pub complete: [bool; 5],
}

#[derive(Debug)]
pub struct WorldPlayerSaveCompletion {
    pub game_server_index: i32,
    pub advertised_player_count: i32,
    pub message_size: i32,
    pub log: AddLogTextDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMurderReport {
    pub fields: [i32; 4],
    pub fields_complete: [bool; 4],
    pub game_server_number: i32,
    pub disposition: WorldMurderReportDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldMurderReportDisposition {
    Relayed {
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
    CountersIncremented(PlayerMurderCounterUpdate),
    MissingOnlinePlayer,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerNameMessageRelay {
    pub requested_name: Vec<u8>,
    pub text: Vec<u8>,
    pub values: [i32; 3],
    pub values_complete: [bool; 3],
    pub resolved_named_player_id: u32,
    pub disposition: WorldPlayerNameMessageDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPlayerNameMessageDisposition {
    Suppressed(WorldPlayerNameMessageSuppression),
    MissingNamedPlayer {
        target_player_id: i32,
        game_server_number: i32,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
    NamedPlayer {
        player_id: i32,
        game_server_number: i32,
        displayed_target: Vec<u8>,
        target_online: bool,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerNameMessageSuppression {
    TargetPlayerNotOnline,
    TargetRouteMissing,
    NamedPlayerRouteMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldGameServerConnectionContinuation {
    NotConfigured,
    NetworkOwnerUnavailable,
    AuctionStateUnavailable,
    LegacyIpv4SyntaxUnknown,
    InitialConfigurationPending {
        socket_id: i32,
        game_server_index: u32,
    },
    InitialConfigurationComplete {
        socket_id: i32,
        game_server_index: u32,
    },
    InitialConfigurationBlocked {
        socket_id: i32,
        game_server_index: u32,
        owner: &'static str,
    },
    ReconnectPlayerDataPending {
        socket_id: i32,
        game_server_index: u32,
        remaining_payload: Vec<u8>,
    },
    ReconnectPlayerDataComplete {
        socket_id: i32,
        game_server_index: u32,
    },
    RegistryPortUnavailable { game_server_index: u32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerAuctionBroadcast {
    pub message_type: i32,
    pub enabled: bool,
    pub delivery: Result<i32, SendMessageError>,
}

pub struct WorldGameServerInitialConfigurationPrefix<'a> {
    pub da_kong_xiang_qian: &'a [u8],
    pub goods_registry: &'a GoodsBasePropertiesRegistry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldInitialConfigurationTarget {
    Socket(i32),
    AllGameServers,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldInitialConfigurationDelivery {
    pub subtype: i32,
    pub payload_length: usize,
    pub target: WorldInitialConfigurationTarget,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldInitialConfigurationRunCompletion {
    Complete,
    Blocked { owner: &'static str },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldInitialConfigurationRunReport {
    pub deliveries: Vec<WorldInitialConfigurationDelivery>,
    pub completion: WorldInitialConfigurationRunCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldInitialConfigurationPrefixCompletion {
    WordsFilter(WordsFilterSerializeError),
    GoodsRegistry(GoodsRegistrySerializeError),
    ThingSetup(ThingSetupCodecError),
    MonsterListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldInitialConfigurationPrefixReport {
    pub deliveries: Vec<WorldInitialConfigurationDelivery>,
    pub language_notice: bool,
    pub words_filter_notice: bool,
    pub completion: WorldInitialConfigurationPrefixCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldMonsterConfigurationCompletion {
    MonsterList(MonsterListSerializeError),
    HitLevelSetupPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMonsterConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldMonsterConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldHitLevelConfigurationCompletion {
    HitLevelSetup(HitLevelSerializeError),
    PlayerListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldHitLevelConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldHitLevelConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPlayerListConfigurationCompletion {
    PlayerList(PlayerListSerializeError),
    EmotionPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerListConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldPlayerListConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldEmotionConfigurationCompletion {
    Emotion(EmotionSerializeError),
    SkillFactoryPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldEmotionConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldEmotionConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSkillConfigurationCompletion {
    SkillFactory(SkillFactorySerializeError),
    TradeListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldSkillConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldSkillConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldTradeListConfigurationCompletion {
    TradeList(TradeListSerializeError),
    IncrementShopListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldTradeListConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldTradeListConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldIncrementShopConfigurationCompletion {
    IncrementShop(IncrementShopSerializeError),
    ContributeSetupPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldIncrementShopConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldIncrementShopConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldContributeConfigurationCompletion {
    ContributeSetup(ContributeSetupSerializeError),
    PrisonConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldContributeConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldContributeConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPrisonConfigurationCompletion {
    PrisonConf(PrisonConfSerializeError),
    PreciousBoxConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPrisonConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldPrisonConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPreciousBoxConfigurationCompletion {
    PreciousBox(PreciousBoxSerializeError),
    FairyExpConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPreciousBoxConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldPreciousBoxConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldFairyExpConfigurationCompletion {
    FairyExp(BattleFairyExpSerializeError),
    SynthesisConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFairyExpConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldFairyExpConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSynthesisConfigurationCompletion {
    Synthesis(SynthesisSerializeError),
    EquipmentComposeConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldSynthesisConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldSynthesisConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldEquipmentComposeConfigurationCompletion {
    EquipmentCompose(EquipmentComposeSerializeError),
    NewSkillMonsterConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldEquipmentComposeConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldEquipmentComposeConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldNewSkillMonsterConfigurationCompletion {
    NewSkillMonster(NewSkillMonsterSerializeError),
    GoodsDestroyConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldNewSkillMonsterConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldNewSkillMonsterConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGoodsDestroyConfigurationCompletion {
    GoodsDestroy(GoodsDestroySerializeError),
    GlobeSetupConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGoodsDestroyConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGoodsDestroyConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGlobeSetupConfigurationCompletion {
    RegionRouter(RegionRouterSerializeError),
    LogSystemConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGlobeSetupConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGlobeSetupConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldLogSystemConfigurationCompletion {
    LogSystem(LogSystemSerializeError),
    CountryParamConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldLogSystemConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldLogSystemConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryParamConfigurationCompletion {
    CountryParam(CountryParamSerializationBlock),
    CountryHandlerConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryParamConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldCountryParamConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGodsBattleConfigurationCompletion {
    GodsBattle(GodsBattleSerializeError),
    RegionSnapshotsPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGodsBattleConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGodsBattleConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldRegionSetupConfigurationCompletion {
    RegionSetup(RegionSetupSerializeError),
    DupliRegionSetupPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRegionSetupConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldRegionSetupConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldDupliRegionConfigurationCompletion {
    DupliRegionSetup(DupliRegionSerializeError),
    HonorEliminateConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldDupliRegionConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldDupliRegionConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldHonorEliminateConfigurationCompletion {
    HonorRanksPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldHonorEliminateConfigurationReport {
    pub delivery: WorldInitialConfigurationDelivery,
    pub completion: WorldHonorEliminateConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldHonorRanksConfigurationDelivery {
    pub rank_type: HonorRanksType,
    pub delivery: WorldInitialConfigurationDelivery,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldHonorRanksConfigurationCompletion {
    HonorRanks(HonorRanksSerializationBlock),
    FunctionListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldHonorRanksConfigurationReport {
    pub deliveries: Vec<WorldHonorRanksConfigurationDelivery>,
    pub completion: WorldHonorRanksConfigurationCompletion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRawScriptListKind {
    Function,
    Variable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRawScriptListConfigurationBlock {
    pub kind: WorldRawScriptListKind,
    pub length: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRawScriptListConfigurationDelivery {
    pub kind: WorldRawScriptListKind,
    pub delivery: WorldInitialConfigurationDelivery,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldRawScriptListsConfigurationCompletion {
    FileSize(WorldRawScriptListConfigurationBlock),
    GeneralVariableListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRawScriptListsConfigurationReport {
    pub deliveries: Vec<WorldRawScriptListConfigurationDelivery>,
    pub completion: WorldRawScriptListsConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGeneralVariableConfigurationCompletion {
    VariableList(VariableListSerializationBlock),
    ScriptFilesPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGeneralVariableConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGeneralVariableConfigurationCompletion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldScriptFileConfigurationBlock {
    pub path: Vec<u8>,
    pub length: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldScriptFileConfigurationDelivery {
    pub path: Vec<u8>,
    pub declared_length: i32,
    pub delivery: WorldInitialConfigurationDelivery,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldScriptFilesConfigurationCompletion {
    FileSize(WorldScriptFileConfigurationBlock),
    QuestSystemPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldScriptFilesConfigurationReport {
    pub deliveries: Vec<WorldScriptFileConfigurationDelivery>,
    pub completion: WorldScriptFilesConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldQuestConfigurationCompletion {
    QuestSystem(QuestSystemSerializationBlock),
    PlayerRanksPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldQuestConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldQuestConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPlayerRanksConfigurationCompletion {
    PlayerRanks(PlayerRanksSerializationBlock),
    GmListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerRanksConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldPlayerRanksConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGmListConfigurationCompletion {
    GmList(GmListSerializationBlock),
    GameServerIndexPending {
        socket_id: i32,
        game_server_index: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGmListConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGmListConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGameServerIndexConfigurationCompletion {
    FourNationWarPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerIndexConfigurationReport {
    pub delivery: WorldInitialConfigurationDelivery,
    pub completion: WorldGameServerIndexConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldFourNationWarConfigurationCompletion {
    FourNationWar(FourNationWarSerializationBlock),
    BattleFairyExpConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFourNationWarConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldFourNationWarConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldBattleFairyExpConfigurationCompletion {
    BattleFairyExp(BattleFairyExpSerializeError),
    BattleFairyPropertyPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldBattleFairyExpConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldBattleFairyExpConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldBattleFairyPropertyConfigurationCompletion {
    BattleFairyProperty(BattleFairyComposeWireError),
    CiQingLingBaoConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldBattleFairyPropertyConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldBattleFairyPropertyConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCiQingLingBaoConfigurationCompletion {
    CiQing(CiQingSerializationBlock),
    LingBao(LingBaoSerializationBlock),
    TaoZhuangConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCiQingLingBaoConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldCiQingLingBaoConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldTaoZhuangConfigurationCompletion {
    TaoZhuang(TaoZhuangSerializationBlock),
    AttackCityConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldTaoZhuangConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldTaoZhuangConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldAttackCityConfigurationCompletion {
    VillageWarConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldAttackCityConfigurationReport {
    pub delivery: WorldInitialConfigurationDelivery,
    pub completion: WorldAttackCityConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldVillageWarConfigurationCompletion {
    CountryWarConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldVillageWarConfigurationReport {
    pub delivery: WorldInitialConfigurationDelivery,
    pub completion: WorldVillageWarConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryWarConfigurationCompletion {
    GameServerIdentityPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryWarConfigurationReport {
    pub delivery: WorldInitialConfigurationDelivery,
    pub completion: WorldCountryWarConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGameServerIdentityCompletion {
    InitialConfigurationComplete { socket_id: i32 },
    MissingWorldNumber { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerIdentityReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldGameServerIdentityCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerBroadcast {
    pub message_type: i32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerPingStart {
    pub cleared_responses: usize,
    pub started_at_ms: u32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldLoginServerTupleRelay {
    WorldNumberUnavailable {
        map_id: i32,
        value: i32,
        payload_complete: bool,
    },
    Suppressed {
        world_number: u32,
        map_id: i32,
        value: i32,
        payload_complete: bool,
    },
    Forwarded {
        world_number: u32,
        map_id: i32,
        value: i32,
        payload_complete: bool,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldOpaqueServerFields {
    pub value: i32,
    pub numeric_complete: bool,
    pub text: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRegionMessageRelay {
    pub ignored_selector: i8,
    pub selector_complete: bool,
    pub region_id: i32,
    pub region_complete: bool,
    pub game_server_number: i32,
    pub message_type: i32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLoginServerIdentity {
    pub previous_login_server_id: i32,
    pub login_server_id: i32,
    pub payload_complete: bool,
}

// Типы второй волны: их поля цитируют data-контракты бывшего
// `worldserver/worldserver/game.rs`, теперь размещённые в
// `nebokrai_realm::app::worldserver`.

#[derive(Debug)]
pub struct WorldLoginClientReplacement {
    pub previous_client_closed: bool,
    pub connected_notice: bool,
    pub cdkey_snapshot: WorldCdkeySnapshot,
    pub registration: Result<i32, SendMessageError>,
}

#[derive(Debug)]
pub struct WorldCompletedSaveResponseLaunchReport {
    pub snapshot: WorldGenerateDbDataReport,
    pub launch: WorldSaveThreadLaunchRequest,
    pub resulting_handle: WorldSaveThreadHandleState,
}

#[derive(Debug)]
pub struct WorldRegionChangeMessage {
    pub player_id: i32,
    pub player_id_complete: bool,
    pub target_region_id: i32,
    pub target_region_complete: bool,
    pub socket_id: i32,
    pub disposition: WorldRegionChangeDisposition,
}

#[derive(Debug)]
pub enum WorldRegionChangeDisposition {
    TargetUnavailable {
        target_region_found: bool,
        delivery: Result<i32, SendMessageError>,
    },
    OnlinePlayerMissing {
        delivery: Result<i32, SendMessageError>,
        operator_notice: bool,
    },
    DecodeBlocked {
        prefix: WorldRegionChangePrefix,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        error: PlayerCodecError,
    },
    OnlinePlayerDisappeared {
        prefix: WorldRegionChangePrefix,
    },
    TargetPortUnavailable {
        prefix: WorldRegionChangePrefix,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        transition: WorldRegionChangePlayerTransition,
        target_game_server_index: u32,
    },
    Changed {
        prefix: WorldRegionChangePrefix,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        transition: WorldRegionChangePlayerTransition,
        target_game_server_index: u32,
        target_ip: Vec<u8>,
        target_port: u32,
        message_type: i32,
        delivery: Result<i32, SendMessageError>,
        team_session_id: i32,
        team_update: WorldRegionChangeTeamUpdate,
    },
}

#[derive(Debug)]
pub enum WorldPlayerSavePacket {
    Ignored {
        index: i32,
        packet_type: i32,
        packet_type_complete: bool,
    },
    Player {
        index: i32,
        requested_player_id: i32,
        player_id_complete: bool,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        decode: WorldServerSnapshotPlayerDecode,
        missing_player_notice: Option<Vec<u8>>,
    },
}

#[derive(Debug)]
pub enum WorldPlayerSaveMaterialization {
    NotTriggered,
    Launched(WorldCompletedSaveResponseLaunchReport),
    Blocked(WorldGenerateDbDataBlock),
}

#[derive(Debug)]
pub enum WorldPlayerSaveBatchDisposition {
    DecodeBlocked {
        packets: Vec<WorldPlayerSavePacket>,
        index: i32,
        requested_player_id: i32,
        player_id_complete: bool,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        error: PlayerCodecError,
    },
    Processed {
        packets: Vec<WorldPlayerSavePacket>,
        exhausted_noop_entries: i32,
        completion: Option<WorldPlayerSaveCompletion>,
        progress: WorldPlayerSaveResponseProgress,
        materialization: WorldPlayerSaveMaterialization,
    },
}

#[derive(Debug)]
pub struct WorldPlayerSaveBatchMessage {
    pub marker: i8,
    pub marker_complete: bool,
    pub advertised_player_count: i32,
    pub player_count_complete: bool,
    pub disposition: WorldPlayerSaveBatchDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRegionParameterUpdate {
    pub region_id: i32,
    pub region_complete: bool,
    pub cursor_before_decode: usize,
    pub cursor_after_decode: usize,
    pub outcome: WorldRegionParamDecodeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlayerDataSync {
    pub subtype: i8,
    pub subtype_complete: bool,
    pub game_server_index: i32,
    pub disposition: WorldPlayerDataSyncDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldPlayerDataSyncDisposition {
    Started {
        declared_online_players: i32,
        payload_complete: bool,
        counter: WorldReceivedPlayerDataUpdate,
        operator_notice: bool,
    },
    Player {
        counter: WorldReceivedPlayerDataUpdate,
        requested_player_id: i32,
        player_id_complete: bool,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        decoded: Result<WorldServerSnapshotPlayerDecode, PlayerCodecError>,
        missing_player_notice: bool,
    },
    Finished {
        declared_sent_players: i32,
        payload_complete: bool,
        received_players: WorldReceivedPlayerDataRead,
        operator_notice: bool,
    },
    Ignored,
}

#[derive(Debug)]
pub struct WorldGameServerConnectionReport {
    pub sync_flag: i8,
    pub sync_flag_complete: bool,
    pub port: u32,
    pub port_complete: bool,
    pub ip: Vec<u8>,
    pub socket_id: i32,
    pub game_server_index: Option<u32>,
    pub previous_connected: Option<bool>,
    pub route_assignment: Option<i32>,
    pub connected_notice: bool,
    pub auction: Option<WorldGameServerAuctionBroadcast>,
    pub globe_variables: Option<WorldGlobeVariablesDelivery>,
    pub login_log: Option<WorldGameServerConnectedLog>,
    pub reconnect: Option<WorldGameServerReconnectReport>,
    pub initial_configuration: Option<WorldInitialConfigurationRunReport>,
    pub continuation: WorldGameServerConnectionContinuation,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRegionConfigurationDelivery {
    pub map_key: i32,
    pub region_id: i32,
    pub kind: WorldInitialRegionSnapshotKind,
    pub delivery: WorldInitialConfigurationDelivery,
    pub delay_after_ms: Option<u32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldRegionConfigurationCompletion {
    RegionSnapshot(WorldInitialRegionSnapshotBlock),
    RegionSetupConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRegionConfigurationReport {
    pub deliveries: Vec<WorldRegionConfigurationDelivery>,
    pub completion: WorldRegionConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldGameServerReconnectRecord {
    Skipped {
        packet_type: i32,
    },
    Player {
        packet_type: i32,
        decoded: WorldReconnectedPlayerDecode,
        online: WorldOnlinePlayerAppendOutcome,
        trailing_value: i32,
        trailing_complete: bool,
    },
}

#[derive(Debug)]
pub enum WorldGameServerReconnectCompletion {
    InvalidElementCount {
        declared_count: i32,
        minimum_bytes: usize,
        available_bytes: usize,
    },
    UnexpectedEnd {
        record_index: usize,
        field: &'static str,
    },
    PlayerCodec {
        record_index: usize,
        error: PlayerCodecError,
    },
    CdkeySnapshot(Result<Option<WorldCdkeySnapshot>, WorldCdkeySnapshotError>),
}

#[derive(Debug)]
pub struct WorldGameServerReconnectReport {
    pub socket_id: i32,
    pub declared_count: i32,
    pub count_complete: bool,
    pub acknowledgement: Result<i32, SendMessageError>,
    pub records: Vec<WorldGameServerReconnectRecord>,
    pub completion: WorldGameServerReconnectCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameServerPingResponse {
    pub response: WorldPingGameServerInfo,
    pub response_count: usize,
    pub payload_complete: bool,
}

#[derive(Debug)]
pub struct WorldServerMessageError {
    pub previous_client_closed: bool,
    pub connected_notice: bool,
    pub source: WorldCdkeySnapshotError,
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

// Финальная типовая волна: диспетчерная связка outcome/dispatch и пара
// CountryHandler configuration. Их transitive-контракты (reconnect
// restart-итоги, network `CMessage`, `CountryHandlerSerializeError`) уже
// размещены в realm.

/// Наблюдаемые эффекты внутреннего `0x3FC01`, опубликованного
/// `CMyNetClient::OnClose`.
#[derive(Debug)]
pub struct WorldLoginServerClosed {
    pub log: AddLogTextDisposition,
    pub reconnect: WorldLoginReconnectThreadRestart,
}

#[derive(Debug)]
pub enum WorldServerMessageOutcome {
    NoOp { request_type: i32 },
    GameServerConnection(WorldGameServerConnectionReport),
    GameServerBroadcast(WorldGameServerBroadcast),
    GameServerPingResponseRecorded(WorldGameServerPingResponse),
    GameServerPingStarted(WorldGameServerPingStart),
    GodsBattle(WorldGodsBattleMessage),
    GodsBattleTopTen(WorldGodsBattleTopTenMessage),
    GeneralVariableUpdated(WorldGeneralVariableUpdate),
    LoginServerClosed(WorldLoginServerClosed),
    LoginServerTupleRelay(WorldLoginServerTupleRelay),
    LoginServerIdentityAssigned(WorldLoginServerIdentity),
    MurderReported(WorldMurderReport),
    OpaqueFieldsRead(WorldOpaqueServerFields),
    PlayerDataSynchronized(WorldPlayerDataSync),
    PlayerNameMessageRelayed(WorldPlayerNameMessageRelay),
    PlayerSaveBatch(WorldPlayerSaveBatchMessage),
    RegionParametersUpdated(WorldRegionParameterUpdate),
    RegionChanged(WorldRegionChangeMessage),
    RegionMessageRelayed(WorldRegionMessageRelay),
    SpawnRouted(WorldSpawnRoutingOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryHandlerConfigurationCompletion {
    CountryHandler(CountryHandlerSerializeError),
    GodsBattleConfigurationPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryHandlerConfigurationReport {
    pub delivery: Option<WorldInitialConfigurationDelivery>,
    pub completion: WorldCountryHandlerConfigurationCompletion,
}

pub enum WorldServerMessageDispatch {
    Handled(WorldServerMessageOutcome),
    Pending(CMessage),
}
