//! Диспетчер server-сообщений WorldServer (`appworld/message/servermessage.cpp`,
//! свободная `OnServerMessage`): сам диспетчер `on_server_message`,
//! decode-helpers, `on_game_server_connected`, семейство
//! `continue_game_server_*_configuration` и reconnect-цепочка. Источник
//! контракта — та же точная пара, что у [`crate::app::world_message`].
//!
//! Внутрипроцессный reconnect заменяет GameServer client в той же FIFO-позиции:
//! старое соединение закрывается и уничтожается до публикации нового; уже
//! выполненная замена не откатывается при ошибке последующей отправки.
//! Первичная ветвь `0x5FA01` назначает map ID и отправляет GameServer всю
//! конфигурацию в исходном порядке; ошибки отдельных `Send` не прерывают
//! цепочку. Reconnect сначала очищает удалённую сторону, затем восстанавливает
//! полные player records и завершает CD-key snapshot. Неизвестный тип записи
//! потребляет только свой tag. Runtime spawn request `0x5FA0B` валидируется
//! вместе с source/region route, меняет только тип на `0x7F80A` и передаёт
//! неизменный payload целевому GameServer. Короткие числовые поля дают
//! legacy-ноль без сдвига курсора; безопасные проверки длины останавливают
//! только чтение, которое в оригинале выходило бы за буфер.
//!
//! Ветвь `0x3FC02` (synthetic close GameServer) разобрана по машинному коду
//! (VERIFIED): `GetGameServer(K)`, not-found AddLogText, found — `connected=0`,
//! при auction slot — `0x80403` + `SendAll`; обе концовки завершают
//! `OnGameServerLost(K)`. Win32-listbox UI-эффекты вне wire сознательно не
//! воспроизводятся (здесь и `LB_ADDSTRING` ветви `0x5FA01`), player-listbox
//! владельца игры подставлен no-op sink-ом. Миграция игроков offline и
//! login-нотификация `0x1FE03` остаются inherent-методом владельца игры через
//! шов `WorldServerMessageGameView`. Legacy-неинициализированный port
//! моделируется `Option`: при `None` модель печатает `%d` нулём и несёт
//! исходный `Option` в отчёте.
//!
//! Игровой контекст диспетчер получает через [`WorldGameView`]; organizing-
//! переходы игрока и save-материализацию — через [`WorldServerMessageGameView`]
//! с ассоциированными типами владельцев старого пакета.
//! `CountryWarSys`/`CCountryHandler`, остающиеся в старом пакете,
//! сериализуются вызывающей стороной в исходной позиции. Цитируемые
//! data-контракты игры лежат в `crate::app::worldserver`, reconnect
//! restart-итоги — в `crate::app::loginreconnectworker`, network `CMessage` —
//! в `crate::app::world_message`, `CountryHandlerSerializeError` — в
//! `crate::organizations`.
//!
//! Доказательства: docs/reconstruction/realm-services.md#world-диспетчер-servermessage
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;

use nebokrai_shared::network::{CBaseMessage, ServerCommandHandle};

use crate::activities::attackcitysys::CAttackCitySys;
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarSerializationBlock};
use crate::activities::rsgodsbattle::{
    GodsBattleNpcFactionSnapshot, RsGodsBattleNotice, RsGodsBattleOwner, TiberiusRsGodsBattle,
};
use crate::activities::villagewarsys::CVillageWarSys;
use crate::app::loginreconnectworker::WorldLoginReconnectThreadRestart;
use crate::app::world_client::CMyNetClient;
use crate::app::world_game_view::{
    WorldCompletedSaveResponseMaterialization, WorldGameServerDisconnectionState, WorldGameView,
    WorldServerMessageGameView,
};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::worldserver::{
    AddLogTextDisposition, WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldGenerateDbDataBlock,
    WorldGenerateDbDataReport, WorldGameServerLostReport, WorldGlobeVariablesDelivery,
    WorldInitialRegionSnapshot,
    WorldInitialRegionSnapshotBlock, WorldInitialRegionSnapshotKind, WorldOnlinePlayerAppendOutcome,
    WorldPingGameServerInfo, WorldPlayerSaveResponseProgress, WorldReceivedPlayerDataRead,
    WorldReceivedPlayerDataUpdate, WorldReconnectedPlayerDecode, WorldRegionChangePlayerTransition,
    WorldRegionChangeTeamUpdate, WorldRegionParamDecodeOutcome, WorldGameServerLookupError,
    WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest, WorldServerSnapshotPlayerDecode,
    WorldServerSnapshotPlayerOwner,
};
use crate::rankings::honordb::HonorRanksType;
use crate::rankings::honorranks::{CHonorRanks, HonorRanksSerializationBlock};
use crate::characters::player::{
    PlayerCodecError, PlayerMurderCounterUpdate, PlayerPropertyCoefficients,
};
use crate::rankings::playerranks::{CPlayerRanks, PlayerRanksSerializationBlock};
use crate::content::battlefairyproperty::{BattleFairyComposeWireError, CBattleFairyProperty};
use crate::content::cgoodsfactory::{GoodsRegistrySerializeError, serialize_goods_registry};
use crate::content::countryparam::{CCountryParam, CountryParamSerializationBlock};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::skillfactory::{CSkillFactory, SkillFactorySerializeError};
use crate::content::variablelist::{
    CVariableList, VariableListSerializationBlock, VariableSetOutcome,
};
use crate::organizations::countryhandler::CountryHandlerSerializeError;
use crate::persistence::rssetup::WorldTdsClient;
use crate::sessions::csessionfactory::CSessionFactory;
use nebokrai_shared::resources::{
    BattleFairyExpSerializeError, CBattleFairyExpConfig, CCiQingSetup, CContributeSetup,
    CDupliRegionSetup, CEmotion, CFairyExpConf, CGMList, CGodsBattleConf, CHitLevelSetup,
    CIncrementShopList, CLingBaoSetup, CLogSystem, CPlayerList, CQuestSystem, CRegionSetup,
    CSynthesis, CTaoZhuangSetup, CThingSetup, CTradeList, CWordsFilter,
    CiQingSerializationBlock, ContributeSetupSerializeError, DupliRegionSerializeError,
    EmotionSerializeError, EquipmentComposeList, EquipmentComposeSerializeError,
    GmListSerializationBlock, GodsBattleFactionXydUpdate, GodsBattleNpcFactionUpdate,
    GodsBattleSerializeError, GlobeSetupSnapshot, GoodsDestroySerializeError, GoodsDestroySetup,
    HitLevelSerializeError, HonorElimilateConfig, IncrementShopSerializeError,
    LingBaoSerializationBlock, LogSystemSerializeError, MonsterDropRegistry, MonsterListSerializeError,
    MonsterRegistry, NewSkillMonsterConf, NewSkillMonsterSerializeError, PlayerListSerializeError,
    PreciousBoxConf, PreciousBoxSerializeError, PrisonConf, PrisonConfSerializeError,
    QuestSystemSerializationBlock, RegionRouter, RegionRouterSerializeError,
    RegionSetupSerializeError, SynthesisSerializeError, TaoZhuangSerializationBlock,
    ThingSetupCodecError, TradeListSerializeError, WordsFilterSerializeError,
    serialize_monster_list,
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

/// Typed-снимок входа initial-prefix: сами владельцы (`StringTable`,
/// `CWordsFilter`, `thing setup`) остаются у игры, обработчик получает только
/// ссылки на чтение в исходной позиции вызова.
pub struct WorldGameServerInitialConfigurationPrefix<'a> {
    pub da_kong_xiang_qian: &'a [u8],
    pub goods_registry: &'a GoodsBasePropertiesRegistry,
    pub string_table: &'a [u8],
    pub words_filter: &'a CWordsFilter,
    pub thing_setup: &'a CThingSetup,
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

// Типы ниже цитируют data-контракты игры из `crate::app::worldserver`.

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

// Диспетчерная связка outcome/dispatch и пара CountryHandler configuration;
// их transitive-контракты (reconnect restart-итоги, network `CMessage`,
// `CountryHandlerSerializeError`) размещены в realm.

/// Наблюдаемые эффекты внутреннего `0x3FC01`, опубликованного
/// `CMyNetClient::OnClose`.
#[derive(Debug)]
pub struct WorldLoginServerClosed {
    pub log: AddLogTextDisposition,
    pub reconnect: WorldLoginReconnectThreadRestart,
}

/// Наблюдаемые эффекты ветви `0x3FC02` — синтетического disconnect-сообщения,
/// опубликованного `CMyServerClient::OnClose` (`0x42BC50`). Машинный порядок
/// `0x4ADD6F...0x4ADE8C`: decode map identity → registry disconnect-мутация →
/// операторский лог → auction-бroadcast → терминал `OnGameServerLost`.
#[derive(Debug)]
pub struct WorldGameServerDisconnected {
    pub map_id: i32,
    pub map_id_complete: bool,
    pub disposition: WorldGameServerDisconnectedDisposition,
    pub lost: WorldGameServerLostReport,
}

#[derive(Debug)]
pub enum WorldGameServerDisconnectedDisposition {
    /// Registry не знает такого identity: ветвь `0x4ADE76`, AddLogText
    /// `!!!!!Unknown GameServer Lost!!!!!!!!![index = %d]`.
    UnknownGameServer { log: AddLogTextDisposition },
    /// Запись найдена и помечена `connected = 0`: listbox-снятие вне wire не
    /// воспроизводится, AddLogText `GameServer %s [%d] lost!`; auction-slot
    /// (`index == 5`) дополнительно получает broadcast `0x80403 + AddLong(0)`.
    Lost {
        disconnection: WorldGameServerDisconnectionState,
        log: AddLogTextDisposition,
        auction_broadcast: Option<WorldGameServerAuctionBroadcast>,
    },
}

#[derive(Debug)]
pub enum WorldServerMessageOutcome {
    NoOp { request_type: i32 },
    GameServerConnection(WorldGameServerConnectionReport),
    GameServerBroadcast(WorldGameServerBroadcast),
    GameServerDisconnected(WorldGameServerDisconnected),
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


// Fn-уровень диспетчера: decode-helpers, reconnect/registration семейство и
// initial-configuration цепочка. Доступ к владельцу игры — через
// `WorldGameView`/`WorldServerMessageGameView` и typed снимки конфигов;
// wire-формат, signedness, byte-exact строки и частичные эффекты сохраняются.

fn decode_spawn_routing_i32(payload: &[u8], cursor: &mut usize) -> Option<i32> {
    let bytes: [u8; 4] = payload
        .get(*cursor..cursor.checked_add(4)?)?
        .try_into()
        .ok()?;
    *cursor += 4;
    Some(i32::from_le_bytes(bytes))
}

fn skip_spawn_routing_string(payload: &[u8], cursor: &mut usize) -> bool {
    let Some(remaining) = payload.get(*cursor..) else {
        return false;
    };
    let Some(length) = remaining.iter().take(256).position(|byte| *byte == 0) else {
        return false;
    };
    *cursor += length + 1;
    true
}

fn skip_spawn_routing_script(payload: &[u8], cursor: &mut usize) -> bool {
    let Some(&marker) = payload.get(*cursor) else {
        return false;
    };
    *cursor += 1;
    marker == 0 || skip_spawn_routing_string(payload, cursor)
}

fn decode_world_spawn_routing(payload: &[u8]) -> Option<WorldSpawnRoutingCommand> {
    let mut cursor = 0_usize;
    let kind = *payload.get(cursor)?;
    cursor += 1;
    if kind > 1 {
        return None;
    }
    let region_id = decode_spawn_routing_i32(payload, &mut cursor)?;
    if region_id <= 0 || !skip_spawn_routing_string(payload, &mut cursor) {
        return None;
    }
    if kind == 0 {
        let count = decode_spawn_routing_i32(payload, &mut cursor)?;
        let left = decode_spawn_routing_i32(payload, &mut cursor)?;
        let top = decode_spawn_routing_i32(payload, &mut cursor)?;
        let right = decode_spawn_routing_i32(payload, &mut cursor)?;
        let bottom = decode_spawn_routing_i32(payload, &mut cursor)?;
        if !(0..=4096).contains(&count)
            || i32::try_from(i64::from(right) - i64::from(left)).is_err()
            || i32::try_from(i64::from(bottom) - i64::from(top)).is_err()
            || !skip_spawn_routing_script(payload, &mut cursor)
        {
            return None;
        }
    } else {
        let _graphics_id = decode_spawn_routing_i32(payload, &mut cursor)?;
        let count = decode_spawn_routing_i32(payload, &mut cursor)?;
        if !(0..=4096).contains(&count) {
            return None;
        }
        for _ in 0..5 {
            let _ = decode_spawn_routing_i32(payload, &mut cursor)?;
        }
        if !skip_spawn_routing_script(payload, &mut cursor) {
            return None;
        }
        let _lifetime = decode_spawn_routing_i32(payload, &mut cursor)?;
    }
    (cursor == payload.len()).then_some(WorldSpawnRoutingCommand { kind, region_id })
}

/// Операторский лог подключения GameServer на LoginServer (`0x1FE05`).
pub fn game_server_connected_log(
    game: &(impl WorldGameView + ?Sized),
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

/// Внутрипроцессная замена LoginServer client в той же FIFO-позиции: старое
/// соединение закрывается и уничтожается до публикации нового, после чего
/// ставятся CD-key snapshot и регистрация. Уже выполненная замена не
/// откатывается при ошибке последующей отправки.
pub fn on_login_client_reconnected(
    game: &mut (impl WorldGameView + ?Sized),
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

/// Регистрация GameServer по адресу (`0x5FA01`): sync-флаг, порт и IP из
/// payload, затем адресный lookup, маршрут socket→map, auction-флаг для
/// аукционного сервера, globe-переменные и операторский лог. Исходная
/// валидация legacy IPv4 сохранена: стандартный `inet_addr` синтаксис принят,
/// числовые legacy-формы отклоняются, остальное даёт `u32::MAX`.
pub fn on_game_server_connected(
    game: &mut (impl WorldGameView + ?Sized),
    message: &mut CMessage,
    auction_enabled: Option<bool>,
) -> WorldGameServerConnectionReport {
    let decoded_sync_flag = message.base_mut().get_char();
    let sync_flag = decoded_sync_flag.unwrap_or(0);
    let decoded_port = message.base_mut().get_long();
    let port = decoded_port.unwrap_or(0) as u32;
    let ip = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("ненулевая GetStr-граница задана точным owner-ом");
    let socket_id = message.socket_id();

    let mut report = WorldGameServerConnectionReport {
        sync_flag,
        sync_flag_complete: decoded_sync_flag.is_some(),
        port,
        port_complete: decoded_port.is_some(),
        ip,
        socket_id,
        game_server_index: None,
        previous_connected: None,
        route_assignment: None,
        connected_notice: false,
        auction: None,
        globe_variables: None,
        login_log: None,
        reconnect: None,
        initial_configuration: None,
        continuation: WorldGameServerConnectionContinuation::NotConfigured,
    };

    let connection = match game.connect_game_server_by_address(&report.ip, report.port) {
        Ok(Some(connection)) => connection,
        Ok(None) => return report,
        Err(error) => {
            let WorldGameServerLookupError::PortUnavailable { index } = error;
            report.continuation = WorldGameServerConnectionContinuation::RegistryPortUnavailable {
                game_server_index: index,
            };
            return report;
        }
    };
    report.game_server_index = Some(connection.index);
    report.previous_connected = Some(connection.previous_connected);

    let Some(sender) = game.current_game_server_sender() else {
        report.continuation = WorldGameServerConnectionContinuation::NetworkOwnerUnavailable;
        return report;
    };
    report.route_assignment = Some(sender.set_client_map_id(socket_id, connection.index as i32));
    report.connected_notice = true;

    if connection.index == 5 {
        let Some(enabled) = auction_enabled else {
            report.continuation = WorldGameServerConnectionContinuation::AuctionStateUnavailable;
            return report;
        };
        let mut notice = CMessage::new(0x0008_0403);
        notice.base_mut().add_ulong(u32::from(enabled));
        report.auction = Some(WorldGameServerAuctionBroadcast {
            message_type: 0x0008_0403,
            enabled,
            delivery: notice.send_all(Some(&sender)),
        });
    }

    report.globe_variables = Some(game.send_globe_variables_to_game_server(socket_id));
    let peer_ipv4 = match observed_inet_addr(&report.ip) {
        Ok(peer_ipv4) => peer_ipv4,
        Err(()) => {
            report.continuation = WorldGameServerConnectionContinuation::LegacyIpv4SyntaxUnknown;
            return report;
        }
    };
    report.login_log = Some(game_server_connected_log(game, peer_ipv4, connection.index));
    report.continuation = if sync_flag == 0 {
        WorldGameServerConnectionContinuation::InitialConfigurationPending {
            socket_id,
            game_server_index: connection.index,
        }
    } else {
        let remaining_payload = {
            let base = message.base_mut();
            base.as_wire_bytes()[base.cursor()..].to_vec()
        };
        WorldGameServerConnectionContinuation::ReconnectPlayerDataPending {
            socket_id,
            game_server_index: connection.index,
            remaining_payload,
        }
    };
    report
}

/// Диспетчер server-сообщений World (`OnServerMessage`): полный набор ветвей
/// исходного `match` в исходном порядке. Организационные переходы игрока и
/// save-материализация остаются у владельца игры через ассоциированные типы
/// шва; остальные обращения — узкие делегации [`WorldGameView`].
#[allow(
    clippy::too_many_arguments,
    reason = "исходный handler повторно обращался к тем же singleton/static владельцам"
)]
pub async fn on_server_message<Game, Materialization>(
    game: &mut Game,
    mut message: CMessage,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    organizing: &mut Game::OrganizingContext,
    honor_ranks: &mut CHonorRanks,
    materialization: &mut Materialization,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    session_factory: &mut CSessionFactory,
    general_variables: Option<&mut CVariableList>,
    globe_setup: &GlobeSetupSnapshot,
    gods_battle: &mut CGodsBattleConf,
    rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
    mut gods_battle_database: Option<&mut WorldTdsClient>,
) -> WorldServerMessageDispatch
where
    Game: WorldServerMessageGameView,
    Materialization: WorldCompletedSaveResponseMaterialization<Game> + ?Sized,
{
    match message.message_type() {
        0x0003_FC01 => {
            let log = add_log_text(b"========= LoginServer closed =========");
            let reconnect = game.create_connect_login_thread(tokio::runtime::Handle::current());
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::LoginServerClosed(
                WorldLoginServerClosed { log, reconnect },
            ))
        }
        0x0003_FC02 => {
            let decoded = message.base_mut().get_long();
            let map_id = decoded.unwrap_or(0);
            let disposition = match game.disconnect_game_server(map_id as u32) {
                None => {
                    let log = add_log_text(
                        format!("!!!!!Unknown GameServer Lost!!!!!!!!![index = {map_id}]")
                            .as_bytes(),
                    );
                    WorldGameServerDisconnectedDisposition::UnknownGameServer { log }
                }
                Some(disconnection) => {
                    // sprintf `%s [%d]` строился только для поиска строки в
                    // Win32 listbox; сам UI-эффект вне wire не воспроизводится.
                    let mut text = b"GameServer ".to_vec();
                    text.extend_from_slice(&disconnection.ip);
                    text.extend_from_slice(
                        format!(" [{}] lost!", disconnection.port.unwrap_or(0)).as_bytes(),
                    );
                    let log = add_log_text(&text);
                    let auction_broadcast = (disconnection.index == 5).then(|| {
                        let mut notice = CMessage::new(0x0008_0403);
                        notice.base_mut().add_ulong(0);
                        let sender = game.current_game_server_sender();
                        WorldGameServerAuctionBroadcast {
                            message_type: 0x0008_0403,
                            enabled: false,
                            delivery: notice.send_all(sender.as_ref()),
                        }
                    });
                    WorldGameServerDisconnectedDisposition::Lost {
                        disconnection,
                        log,
                        auction_broadcast,
                    }
                }
            };
            // Обе концовки исходной ветви завершают `OnGameServerLost(K)`;
            // player-listbox add подставлен no-op sink-ом (UI-эффект вне wire).
            let lost = game.on_game_server_lost(organizing, map_id as u32, &mut |_| {});
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GameServerDisconnected(
                WorldGameServerDisconnected {
                    map_id,
                    map_id_complete: decoded.is_some(),
                    disposition,
                    lost,
                },
            ))
        }
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
        0x0005_FA0B => {
            let source_map_id = message.map_id();
            let command = decode_world_spawn_routing(message.base_mut().unread_bytes());
            let source_is_current = source_map_id != 5
                && u32::try_from(source_map_id)
                    .ok()
                    .and_then(|map_id| game.game_server(map_id))
                    .is_some_and(|server| server.connected);
            let Some(command) = command else {
                return WorldServerMessageDispatch::Handled(
                    WorldServerMessageOutcome::SpawnRouted(WorldSpawnRoutingOutcome::Rejected {
                        source_map_id,
                        region_id: None,
                        reason: "некорректный spawn-routing payload",
                    }),
                );
            };
            if !source_is_current {
                return WorldServerMessageDispatch::Handled(
                    WorldServerMessageOutcome::SpawnRouted(WorldSpawnRoutingOutcome::Rejected {
                        source_map_id,
                        region_id: Some(command.region_id),
                        reason: "источник не является подключённым GameServer",
                    }),
                );
            }
            let target_map_id = game.game_server_number_by_region_id(command.region_id);
            let target_available = target_map_id > 0
                && target_map_id != 5
                && u32::try_from(target_map_id)
                    .ok()
                    .and_then(|map_id| game.game_server(map_id))
                    .is_some_and(|server| server.connected);
            if !game.has_materialized_region(command.region_id) || !target_available {
                return WorldServerMessageDispatch::Handled(
                    WorldServerMessageOutcome::SpawnRouted(WorldSpawnRoutingOutcome::Rejected {
                        source_map_id,
                        region_id: Some(command.region_id),
                        reason: "владелец целевого региона недоступен",
                    }),
                );
            }
            message.set_message_type(0x0007_f80a);
            let delivery = game.send_msg_to_game_server(target_map_id, &message);
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::SpawnRouted(
                WorldSpawnRoutingOutcome::Forwarded {
                    source_map_id,
                    target_map_id,
                    region_id: command.region_id,
                    kind: command.kind,
                    delivery,
                },
            ))
        }
        0x0005_FA01 => {
            let mut report =
                on_game_server_connected(game, &mut message, Some(globe_setup.auction_enabled()));
            if let WorldGameServerConnectionContinuation::ReconnectPlayerDataPending {
                socket_id,
                game_server_index,
                remaining_payload,
            } = report.continuation.clone()
            {
                report.reconnect = Some(continue_game_server_reconnect(
                    game,
                    socket_id,
                    &remaining_payload,
                    registry,
                    organizing,
                    coefficients,
                ));
                report.continuation =
                    WorldGameServerConnectionContinuation::ReconnectPlayerDataComplete {
                        socket_id,
                        game_server_index,
                    };
            }
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GameServerConnection(
                report,
            ))
        }
        0x0005_FA02 => {
            let decoded_player_id = message.base_mut().get_long();
            let player_id = decoded_player_id.unwrap_or(0);
            let decoded_target_region = message.base_mut().get_long();
            let target_region_id = decoded_target_region.unwrap_or(0);
            let socket_id = message.socket_id();
            let target_region_found = game.region_assignment_exists(target_region_id);
            let target_game_server = game
                .region_game_server_route(target_region_id)
                .filter(|route| route.connected)
                .map(|route| (route.index, route.ip, route.port));

            let disposition = if target_game_server.is_none() {
                WorldRegionChangeDisposition::TargetUnavailable {
                    target_region_found,
                    delivery: send_region_change_failure(game, socket_id, player_id),
                }
            } else if game.online_player_by_id(player_id as u32).is_none() {
                WorldRegionChangeDisposition::OnlinePlayerMissing {
                    delivery: send_region_change_failure(game, socket_id, player_id),
                    operator_notice: true,
                }
            } else {
                let decoded = [
                    message.base_mut().get_long(),
                    message.base_mut().get_long(),
                    message.base_mut().get_long(),
                    message.base_mut().get_long(),
                    message.base_mut().get_long(),
                ];
                let values = decoded.map(|value| value.unwrap_or(0));
                let prefix = WorldRegionChangePrefix {
                    tile_x: values[0],
                    tile_y: values[1],
                    direction: values[2],
                    use_goods: values[3],
                    range: values[4],
                    complete: decoded.map(|value| value.is_some()),
                };
                let cursor_before_decode = message.base_mut().cursor();
                let transition = {
                    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                    game.transition_online_player_region(
                        organizing,
                        player_id as u32,
                        target_region_id,
                        prefix.tile_x,
                        prefix.tile_y,
                        prefix.direction,
                        source,
                        cursor,
                        registry,
                        coefficients,
                    )
                };
                let cursor_after_decode = message.base_mut().cursor();
                match transition {
                    Err(error) => WorldRegionChangeDisposition::DecodeBlocked {
                        prefix,
                        cursor_before_decode,
                        cursor_after_decode,
                        error,
                    },
                    Ok(None) => WorldRegionChangeDisposition::OnlinePlayerDisappeared { prefix },
                    Ok(Some(transition)) => {
                        let (target_game_server_index, target_ip, target_port) =
                            target_game_server.expect("target проверен до player mutation");
                        match target_port {
                            None => WorldRegionChangeDisposition::TargetPortUnavailable {
                                prefix,
                                cursor_before_decode,
                                cursor_after_decode,
                                transition,
                                target_game_server_index,
                            },
                            Some(target_port) => {
                                let mut response = CMessage::new(0x0007_F802);
                                response.base_mut().add_char(1);
                                response.base_mut().add_long(player_id);
                                add_legacy_c_string(response.base_mut(), &target_ip);
                                response.base_mut().add_ulong(target_port);
                                let sender = game.current_game_server_sender();
                                let delivery = response.send_to_socket(sender.as_ref(), socket_id);
                                let team_session_id =
                                    game.get_team_session_id(transition.team_id as u32);
                                let team_update = game.set_team_player_owner_region(
                                    session_factory,
                                    team_session_id,
                                    transition.owner_type,
                                    transition.owner_id,
                                    transition.target_region_id,
                                );
                                WorldRegionChangeDisposition::Changed {
                                    prefix,
                                    cursor_before_decode,
                                    cursor_after_decode,
                                    transition,
                                    target_game_server_index,
                                    target_ip,
                                    target_port,
                                    message_type: 0x0007_F802,
                                    delivery,
                                    team_session_id,
                                    team_update,
                                }
                            }
                        }
                    }
                }
            };

            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::RegionChanged(
                WorldRegionChangeMessage {
                    player_id,
                    player_id_complete: decoded_player_id.is_some(),
                    target_region_id,
                    target_region_complete: decoded_target_region.is_some(),
                    socket_id,
                    disposition,
                },
            ))
        }
        0x0005_FA03 => {
            let message_size = i32::from_le_bytes(
                message.as_wire_bytes()[..4]
                    .try_into()
                    .expect("World message всегда содержит полный header"),
            );
            let game_server_index = message.map_id();
            let decoded_marker = message.base_mut().get_char();
            let marker = decoded_marker.unwrap_or(0);
            let decoded_player_count = message.base_mut().get_long();
            let advertised_player_count = decoded_player_count.unwrap_or(0);

            let disposition = 'batch: {
                let mut packets = Vec::new();
                let mut exhausted_noop_entries = 0;
                if marker != -1 && advertised_player_count > 0 {
                    let mut index = 0;
                    while index < advertised_player_count {
                        let decoded_packet_type = message.base_mut().get_long();
                        let packet_type = decoded_packet_type.unwrap_or(0);
                        if packet_type != 1 {
                            packets.push(WorldPlayerSavePacket::Ignored {
                                index,
                                packet_type,
                                packet_type_complete: decoded_packet_type.is_some(),
                            });
                            if decoded_packet_type.is_none() {
                                exhausted_noop_entries =
                                    advertised_player_count.wrapping_sub(index).wrapping_sub(1);
                                break;
                            }
                            index = index.wrapping_add(1);
                            continue;
                        }

                        let decoded_player_id = message.base_mut().get_long();
                        let requested_player_id = decoded_player_id.unwrap_or(0);
                        let cursor_before_decode = message.base_mut().cursor();
                        let decode = {
                            let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                            game.decord_server_snapshot_player(
                                requested_player_id as u32,
                                source,
                                cursor,
                                registry,
                                coefficients,
                            )
                        };
                        let cursor_after_decode = message.base_mut().cursor();
                        let decode = match decode {
                            Ok(decode) => decode,
                            Err(error) => {
                                break 'batch WorldPlayerSaveBatchDisposition::DecodeBlocked {
                                    packets,
                                    index,
                                    requested_player_id,
                                    player_id_complete: decoded_player_id.is_some(),
                                    cursor_before_decode,
                                    cursor_after_decode,
                                    error,
                                };
                            }
                        };
                        let missing_player_notice = match decode.owner {
                            WorldServerSnapshotPlayerOwner::Existing => None,
                            WorldServerSnapshotPlayerOwner::Created { .. } => {
                                let mut notice = game
                                    .map_player(decode.decoded_player_id as u32)
                                    .map(|player| {
                                        let name = player.get_name();
                                        let end = name
                                            .iter()
                                            .position(|byte| *byte == 0)
                                            .unwrap_or(name.len());
                                        name[..end].to_vec()
                                    })
                                    .unwrap_or_default();
                                notice.extend_from_slice(
                                    format!(
                                        "({}) does not exist in WorldServer!!!!!",
                                        decode.decoded_player_id
                                    )
                                    .as_bytes(),
                                );
                                Some(notice)
                            }
                        };
                        packets.push(WorldPlayerSavePacket::Player {
                            index,
                            requested_player_id,
                            player_id_complete: decoded_player_id.is_some(),
                            cursor_before_decode,
                            cursor_after_decode,
                            decode,
                            missing_player_notice,
                        });
                        index = index.wrapping_add(1);
                    }
                }

                let completion = (marker == -1).then(|| {
                    let text = format!(
                        "Received saved data from GameServer{game_server_index} successfully, player:{advertised_player_count}, data packets size:{message_size}"
                    );
                    WorldPlayerSaveCompletion {
                        game_server_index,
                        advertised_player_count,
                        message_size,
                        log: add_log_text(text.as_bytes()),
                    }
                });
                let progress = game.record_player_save_response(completion.is_some());
                let materialization = if progress.save_triggered {
                    let variables = general_variables
                        .as_deref()
                        .expect("World message loop запускается после загрузки general variables");
                    match materialization.materialize_completed_save_response_snapshot(
                        game,
                        registry,
                        organizing,
                        coefficients,
                        variables,
                        honor_ranks,
                        gods_battle,
                    ) {
                        Ok(report) => WorldPlayerSaveMaterialization::Launched(report),
                        Err(block) => WorldPlayerSaveMaterialization::Blocked(block),
                    }
                } else {
                    WorldPlayerSaveMaterialization::NotTriggered
                };
                WorldPlayerSaveBatchDisposition::Processed {
                    packets,
                    exhausted_noop_entries,
                    completion,
                    progress,
                    materialization,
                }
            };

            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::PlayerSaveBatch(
                WorldPlayerSaveBatchMessage {
                    marker,
                    marker_complete: decoded_marker.is_some(),
                    advertised_player_count,
                    player_count_complete: decoded_player_count.is_some(),
                    disposition,
                },
            ))
        }
        0x0005_FA04 => {
            let requested_name = message
                .base_mut()
                .get_str_bytes(0x18)
                .expect("ненулевая GetStr-граница задана точным owner-ом");
            let text = message
                .base_mut()
                .get_str_bytes(0x400)
                .expect("ненулевая GetStr-граница задана точным owner-ом");
            let decoded = [
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ];
            let values = decoded.map(|value| value.unwrap_or(0));
            let values_complete = decoded.map(|value| value.is_some());
            let target_player_id = values[2];
            let resolved_named_player_id = game.online_player_id_by_name(&requested_name);

            let disposition = if let Some(named_player_id) = game
                .online_player_by_id(resolved_named_player_id)
                .map(|player| player.get_id())
            {
                let game_server_number = game.game_server_number_by_player_id(named_player_id);
                if game_server_number == 0 {
                    WorldPlayerNameMessageDisposition::Suppressed(
                        WorldPlayerNameMessageSuppression::NamedPlayerRouteMissing,
                    )
                } else {
                    let (displayed_target, target_online) = game
                        .online_player_by_id(target_player_id as u32)
                        .map_or_else(
                            || (format!("uid[{target_player_id}]").into_bytes(), false),
                            |player| {
                                let name = player.get_name();
                                let end = name
                                    .iter()
                                    .position(|byte| *byte == 0)
                                    .unwrap_or(name.len());
                                (name[..end].to_vec(), true)
                            },
                        );
                    let mut forwarded = CMessage::new(0x0007_F804);
                    forwarded.base_mut().add_long(named_player_id);
                    add_legacy_c_string(forwarded.base_mut(), &text);
                    forwarded.base_mut().add_long(values[0]);
                    forwarded.base_mut().add_long(values[1]);
                    add_legacy_c_string(forwarded.base_mut(), &displayed_target);
                    let delivery = game.send_msg_to_game_server(game_server_number, &forwarded);
                    WorldPlayerNameMessageDisposition::NamedPlayer {
                        player_id: named_player_id,
                        game_server_number,
                        displayed_target,
                        target_online,
                        message_type: 0x0007_F804,
                        delivery,
                    }
                }
            } else if let Some(target_id) = game
                .online_player_by_id(target_player_id as u32)
                .map(|player| player.get_id())
            {
                let game_server_number = game.game_server_number_by_player_id(target_id);
                if game_server_number == 0 {
                    WorldPlayerNameMessageDisposition::Suppressed(
                        WorldPlayerNameMessageSuppression::TargetRouteMissing,
                    )
                } else {
                    let mut failure = CMessage::new(0x0007_F804);
                    failure.base_mut().add_long(0);
                    add_legacy_c_string(failure.base_mut(), &requested_name);
                    failure.base_mut().add_long(target_player_id);
                    let delivery = game.send_msg_to_game_server(game_server_number, &failure);
                    WorldPlayerNameMessageDisposition::MissingNamedPlayer {
                        target_player_id,
                        game_server_number,
                        message_type: 0x0007_F804,
                        delivery,
                    }
                }
            } else {
                WorldPlayerNameMessageDisposition::Suppressed(
                    WorldPlayerNameMessageSuppression::TargetPlayerNotOnline,
                )
            };

            WorldServerMessageDispatch::Handled(
                WorldServerMessageOutcome::PlayerNameMessageRelayed(WorldPlayerNameMessageRelay {
                    requested_name,
                    text,
                    values,
                    values_complete,
                    resolved_named_player_id,
                    disposition,
                }),
            )
        }
        0x0005_FA05 => {
            let decoded_type = message.base_mut().get_long();
            let variable_type = decoded_type.unwrap_or(0);
            let name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("ненулевая GetStr-граница задана точным owner-ом");
            let value = match variable_type {
                1 => {
                    let decoded = message.base_mut().get_long();
                    Some(WorldGeneralVariableValue::Integer {
                        value: decoded.unwrap_or(0),
                        complete: decoded.is_some(),
                    })
                }
                3 => Some(WorldGeneralVariableValue::String(
                    message
                        .base_mut()
                        .get_str_bytes(0x100)
                        .expect("ненулевая GetStr-граница задана точным owner-ом"),
                )),
                _ => None,
            };

            let disposition = if value.is_none() {
                WorldGeneralVariableUpdateDisposition::UnsupportedTypeLegacyUndefined
            } else {
                match general_variables {
                    None => WorldGeneralVariableUpdateDisposition::VariableListUnavailable,
                    Some(variables) => {
                        let mutation = match value.as_ref().expect("type `1/3` имеет value") {
                            WorldGeneralVariableValue::Integer { value, .. } => {
                                variables.set_zero_index_integer(&name, *value)
                            }
                            WorldGeneralVariableValue::String(value) => {
                                variables.set_string(&name, value)
                            }
                        };
                        if matches!(mutation, VariableSetOutcome::Updated { .. }) {
                            let mut response = CMessage::new(0x0007_F805);
                            response.base_mut().add_long(variable_type);
                            add_legacy_c_string(response.base_mut(), &name);
                            match value.as_ref().expect("type `1/3` имеет value") {
                                WorldGeneralVariableValue::Integer { value, .. } => {
                                    response.base_mut().add_long(*value);
                                }
                                WorldGeneralVariableValue::String(value) => {
                                    add_legacy_c_string(response.base_mut(), value);
                                }
                            }
                            let sender = game.current_game_server_sender();
                            let delivery = response.send_all(sender.as_ref());
                            WorldGeneralVariableUpdateDisposition::Broadcast {
                                mutation,
                                message_type: 0x0007_F805,
                                delivery,
                            }
                        } else {
                            WorldGeneralVariableUpdateDisposition::MutationRejected(mutation)
                        }
                    }
                }
            };

            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GeneralVariableUpdated(
                WorldGeneralVariableUpdate {
                    variable_type,
                    type_complete: decoded_type.is_some(),
                    name,
                    value,
                    disposition,
                },
            ))
        }
        0x0005_FA06 => {
            let decoded = [
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ];
            let fields = decoded.map(|value| value.unwrap_or(0));
            let fields_complete = decoded.map(|value| value.is_some());
            let player_id = fields[3];
            let game_server_number = game.game_server_number_by_player_id(player_id);
            let disposition = if game_server_number == 0 {
                game.increment_online_player_murder_counters(player_id as u32)
                    .map_or(
                        WorldMurderReportDisposition::MissingOnlinePlayer,
                        WorldMurderReportDisposition::CountersIncremented,
                    )
            } else {
                message.set_message_type(0x0007_F806);
                let delivery = game.send_msg_to_game_server(game_server_number, &message);
                WorldMurderReportDisposition::Relayed {
                    message_type: 0x0007_F806,
                    delivery,
                }
            };
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::MurderReported(
                WorldMurderReport {
                    fields,
                    fields_complete,
                    game_server_number,
                    disposition,
                },
            ))
        }
        0x0005_FA07 => {
            let decoded_region = message.base_mut().get_long();
            let region_id = decoded_region.unwrap_or(0);
            let cursor_before_decode = message.base_mut().cursor();
            let outcome = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                game.decode_region_param_from_game_server(region_id, source, cursor)
            };
            let cursor_after_decode = message.base_mut().cursor();
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::RegionParametersUpdated(
                WorldRegionParameterUpdate {
                    region_id,
                    region_complete: decoded_region.is_some(),
                    cursor_before_decode,
                    cursor_after_decode,
                    outcome,
                },
            ))
        }
        0x0005_FA09 => {
            let decoded_subtype = message.base_mut().get_char();
            let subtype = decoded_subtype.unwrap_or(0);
            let game_server_index = message.map_id();
            let disposition = match subtype {
                0 => {
                    let decoded_count = message.base_mut().get_long();
                    let declared_online_players = decoded_count.unwrap_or(0);
                    let counter = game.reset_received_player_data(game_server_index);
                    WorldPlayerDataSyncDisposition::Started {
                        declared_online_players,
                        payload_complete: decoded_count.is_some(),
                        counter,
                        operator_notice: true,
                    }
                }
                1 => {
                    let counter = game.increment_received_player_data(game_server_index);
                    let decoded_player_id = message.base_mut().get_long();
                    let requested_player_id = decoded_player_id.unwrap_or(0);
                    let cursor_before_decode = message.base_mut().cursor();
                    let decoded = {
                        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                        game.decord_server_snapshot_player(
                            requested_player_id as u32,
                            source,
                            cursor,
                            registry,
                            coefficients,
                        )
                    };
                    let cursor_after_decode = message.base_mut().cursor();
                    let missing_player_notice = matches!(
                        &decoded,
                        Ok(player) if matches!(
                            player.owner,
                            WorldServerSnapshotPlayerOwner::Created { .. }
                        )
                    );
                    WorldPlayerDataSyncDisposition::Player {
                        counter,
                        requested_player_id,
                        player_id_complete: decoded_player_id.is_some(),
                        cursor_before_decode,
                        cursor_after_decode,
                        decoded,
                        missing_player_notice,
                    }
                }
                2 => {
                    let decoded_count = message.base_mut().get_long();
                    let declared_sent_players = decoded_count.unwrap_or(0);
                    let received_players = game.received_player_data(game_server_index);
                    WorldPlayerDataSyncDisposition::Finished {
                        declared_sent_players,
                        payload_complete: decoded_count.is_some(),
                        received_players,
                        operator_notice: true,
                    }
                }
                _ => WorldPlayerDataSyncDisposition::Ignored,
            };
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::PlayerDataSynchronized(
                WorldPlayerDataSync {
                    subtype,
                    subtype_complete: decoded_subtype.is_some(),
                    game_server_index,
                    disposition,
                },
            ))
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
        0x0005_FA0C => {
            let world_number = game.configured_world_number();
            let map_id = message.map_id();
            let decoded = message.base_mut().get_long();
            let value = decoded.unwrap_or(0);
            let payload_complete = decoded.is_some();

            let relay = match world_number {
                None => {
                    // До успешного LoadSetup старый
                    // dwNumber был неинициализирован. Реакция его чтения не
                    // назначается; исходное GetLong уже выполнено выше.
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
        0x0005_FA0F => {
            let decoded_subtype = message.base_mut().get_char();
            let subtype = decoded_subtype.unwrap_or(0);
            let socket_id = message.socket_id();
            let disposition = match subtype {
                0 => {
                    let (faction_a_xyd, faction_b_xyd) = gods_battle.faction_xyd();
                    let mut response = CMessage::new(0x0007_F80E);
                    response.base_mut().add_ulong(faction_a_xyd);
                    response.base_mut().add_ulong(faction_b_xyd);
                    let sender = game.current_game_server_sender();
                    let delivery = response.send_to_socket(sender.as_ref(), socket_id);
                    WorldGodsBattleDisposition::Query {
                        faction_a_xyd,
                        faction_b_xyd,
                        message_type: 0x0007_F80E,
                        socket_id,
                        delivery,
                    }
                }
                1 => {
                    let decoded_faction = message.base_mut().get_long();
                    let faction = decoded_faction.unwrap_or(0);
                    let decoded_xyd = message.base_mut().get_long();
                    let xyd = decoded_xyd.unwrap_or(0) as u32;
                    let update = gods_battle.set_faction_xyd(faction, xyd);
                    let (faction_a_xyd, faction_b_xyd) = gods_battle.faction_xyd();
                    let mut response = CMessage::new(0x0007_F80E);
                    response.base_mut().add_char(1);
                    response.base_mut().add_ulong(faction_a_xyd);
                    response.base_mut().add_ulong(faction_b_xyd);
                    let sender = game.current_game_server_sender();
                    let delivery = response.send_to_socket(sender.as_ref(), socket_id);
                    WorldGodsBattleDisposition::UpdateFaction {
                        faction,
                        faction_complete: decoded_faction.is_some(),
                        xyd,
                        xyd_complete: decoded_xyd.is_some(),
                        update,
                        faction_a_xyd,
                        faction_b_xyd,
                        response_marker: 1,
                        message_type: 0x0007_F80E,
                        socket_id,
                        delivery,
                    }
                }
                2 => {
                    let name = message
                        .base_mut()
                        .get_str_bytes(0x80)
                        .expect("ненулевая GetStr-граница задана точным owner-ом");
                    let decoded_faction = message.base_mut().get_long();
                    let faction = decoded_faction.unwrap_or(0);
                    let update = gods_battle.set_npc_faction(&name, faction);
                    let snapshots: Vec<_> = gods_battle
                        .npc_names()
                        .iter()
                        .map(|npc| GodsBattleNpcFactionSnapshot {
                            faction: npc.faction,
                            name: npc.name.clone(),
                        })
                        .collect();
                    let save = match rs_gods_battle {
                        None => WorldGodsBattleNpcSave::DatabaseOwnerUnavailable,
                        Some(database_owner) => {
                            let notice_checkpoint = database_owner.notice_checkpoint();
                            let save_returned =
                                database_owner.save_npc_faction_autonomous(&snapshots).await;
                            let notices = database_owner.drain_notices_after(notice_checkpoint);
                            WorldGodsBattleNpcSave::Completed {
                                snapshot_records: snapshots.len(),
                                save_returned,
                                notices,
                            }
                        }
                    };
                    WorldGodsBattleDisposition::UpdateNpc {
                        name,
                        faction,
                        faction_complete: decoded_faction.is_some(),
                        update,
                        save,
                    }
                }
                _ => WorldGodsBattleDisposition::Ignored,
            };
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GodsBattle(
                WorldGodsBattleMessage {
                    subtype,
                    subtype_complete: decoded_subtype.is_some(),
                    disposition,
                },
            ))
        }
        0x0005_FA10 => {
            let socket_id = message.socket_id();
            let disposition = match rs_gods_battle {
                None => WorldGodsBattleTopTenDisposition::DatabaseOwnerUnavailable,
                Some(database_owner) => {
                    let notice_checkpoint = database_owner.notice_checkpoint();
                    let mut payload = Vec::new();
                    let faction_five_succeeded = database_owner
                        .get_top_ten_szl_players(
                            5,
                            &mut payload,
                            gods_battle_database.as_deref_mut(),
                        )
                        .await;
                    if !faction_five_succeeded {
                        let notices = database_owner.drain_notices_after(notice_checkpoint);
                        WorldGodsBattleTopTenDisposition::FactionFiveFailed { notices }
                    } else {
                        let faction_five_payload_bytes = payload.len();
                        let faction_six_succeeded = database_owner
                            .get_top_ten_szl_players(
                                6,
                                &mut payload,
                                gods_battle_database.as_deref_mut(),
                            )
                            .await;
                        if !faction_six_succeeded {
                            let notices = database_owner.drain_notices_after(notice_checkpoint);
                            WorldGodsBattleTopTenDisposition::FactionSixFailed {
                                faction_five_payload_bytes,
                                notices,
                            }
                        } else {
                            let faction_six_payload_bytes =
                                payload.len() - faction_five_payload_bytes;
                            payload.extend_from_slice(&0_i32.to_le_bytes());
                            let mut response = CMessage::new(0x0007_F80F);
                            response.base_mut().add(&payload);
                            let sender = game.current_game_server_sender();
                            let delivery = response.send_to_socket(sender.as_ref(), socket_id);
                            let notices = database_owner.drain_notices_after(notice_checkpoint);
                            WorldGodsBattleTopTenDisposition::Sent {
                                faction_five_payload_bytes,
                                faction_six_payload_bytes,
                                terminal_marker: 0,
                                payload_bytes: payload.len(),
                                message_type: 0x0007_F80F,
                                delivery,
                                notices,
                            }
                        }
                    }
                }
            };
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GodsBattleTopTen(
                WorldGodsBattleTopTenMessage {
                    socket_id,
                    disposition,
                },
            ))
        }
        request_type => {
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::NoOp { request_type })
        }
    }
}

/// Initial-prefix `0x5FA01` sync-ветки: DaKongXiangQian, StringTable,
/// words filter, goods registry и thing setup в исходном порядке; thing setup
/// рассылается всем GameServer. Ошибка сериализации очередного владельца
/// прекращает цепочку до его send, уже поставленные сообщения не откатываются.
pub fn continue_game_server_initial_configuration_prefix(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    snapshots: WorldGameServerInitialConfigurationPrefix<'_>,
) -> WorldInitialConfigurationPrefixReport {
    let mut deliveries = Vec::with_capacity(5);
    deliveries.push(send_initial_configuration_to_socket(
        sender,
        socket_id,
        0x2B,
        snapshots.da_kong_xiang_qian,
    ));
    deliveries.push(send_initial_configuration_to_socket(
        sender,
        socket_id,
        0x2F,
        snapshots.string_table,
    ));
    let language_notice = true;

    let words_filter_notice = if snapshots.words_filter.is_valid() {
        let mut words_filter = Vec::new();
        if let Err(error) = snapshots.words_filter.add_to_byte_array(&mut words_filter) {
            return WorldInitialConfigurationPrefixReport {
                deliveries,
                language_notice,
                words_filter_notice: false,
                completion: WorldInitialConfigurationPrefixCompletion::WordsFilter(error),
            };
        }
        deliveries.push(send_initial_configuration_to_socket(
            sender,
            socket_id,
            0x31,
            &words_filter,
        ));
        true
    } else {
        false
    };

    let mut goods = Vec::new();
    if let Err(error) = serialize_goods_registry(snapshots.goods_registry, &mut goods) {
        return WorldInitialConfigurationPrefixReport {
            deliveries,
            language_notice,
            words_filter_notice,
            completion: WorldInitialConfigurationPrefixCompletion::GoodsRegistry(error),
        };
    }
    deliveries.push(send_initial_configuration_to_socket(
        sender, socket_id, 0, &goods,
    ));
    let mut thing_setup = Vec::new();
    if let Err(error) = snapshots.thing_setup.add_to_byte_array(&mut thing_setup) {
        return WorldInitialConfigurationPrefixReport {
            deliveries,
            language_notice,
            words_filter_notice,
            completion: WorldInitialConfigurationPrefixCompletion::ThingSetup(error),
        };
    }
    deliveries.push(send_initial_configuration_to_all(sender, 0x36, &thing_setup));

    WorldInitialConfigurationPrefixReport {
        deliveries,
        language_notice,
        words_filter_notice,
        completion: WorldInitialConfigurationPrefixCompletion::MonsterListPending { socket_id },
    }
}

/// Шаг 2: monster list с drop-registry в исходном двухвладельческом порядке.
pub fn continue_game_server_monster_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    monsters: &MonsterRegistry,
    drop_goods: &MonsterDropRegistry,
) -> WorldMonsterConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = serialize_monster_list(monsters, drop_goods, &mut payload) {
        return WorldMonsterConfigurationReport {
            delivery: None,
            completion: WorldMonsterConfigurationCompletion::MonsterList(error),
        };
    }

    WorldMonsterConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 2, &payload,
        )),
        completion: WorldMonsterConfigurationCompletion::HitLevelSetupPending { socket_id },
    }
}

/// Шаг 0x14: hit level setup.
pub fn continue_game_server_hit_level_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    hit_level_setup: &CHitLevelSetup,
) -> WorldHitLevelConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = hit_level_setup.add_to_byte_array(&mut payload) {
        return WorldHitLevelConfigurationReport {
            delivery: None,
            completion: WorldHitLevelConfigurationCompletion::HitLevelSetup(error),
        };
    }

    WorldHitLevelConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x14, &payload,
        )),
        completion: WorldHitLevelConfigurationCompletion::PlayerListPending { socket_id },
    }
}

/// Шаг 1: player list.
pub fn continue_game_server_player_list_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    player_list: &CPlayerList,
) -> WorldPlayerListConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = player_list.add_to_byte_array(&mut payload) {
        return WorldPlayerListConfigurationReport {
            delivery: None,
            completion: WorldPlayerListConfigurationCompletion::PlayerList(error),
        };
    }

    WorldPlayerListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 1, &payload,
        )),
        completion: WorldPlayerListConfigurationCompletion::EmotionPending { socket_id },
    }
}

/// Шаг 0x15: emotion table.
pub fn continue_game_server_emotion_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    emotion: &CEmotion,
) -> WorldEmotionConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = emotion.serialize(&mut payload) {
        return WorldEmotionConfigurationReport {
            delivery: None,
            completion: WorldEmotionConfigurationCompletion::Emotion(error),
        };
    }

    WorldEmotionConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x15, &payload,
        )),
        completion: WorldEmotionConfigurationCompletion::SkillFactoryPending { socket_id },
    }
}

/// Шаг 6: skill factory.
pub fn continue_game_server_skill_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    skills: &CSkillFactory,
) -> WorldSkillConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = skills.serialize(&mut payload) {
        return WorldSkillConfigurationReport {
            delivery: None,
            completion: WorldSkillConfigurationCompletion::SkillFactory(error),
        };
    }

    WorldSkillConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 6, &payload,
        )),
        completion: WorldSkillConfigurationCompletion::TradeListPending { socket_id },
    }
}

/// Шаг 3: trade list.
pub fn continue_game_server_trade_list_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    trade_list: &CTradeList,
) -> WorldTradeListConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = trade_list.add_to_byte_array(&mut payload) {
        return WorldTradeListConfigurationReport {
            delivery: None,
            completion: WorldTradeListConfigurationCompletion::TradeList(error),
        };
    }

    WorldTradeListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 3, &payload,
        )),
        completion: WorldTradeListConfigurationCompletion::IncrementShopListPending { socket_id },
    }
}

/// Шаг 4: increment shop list.
pub fn continue_game_server_increment_shop_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    increment_shop_list: &CIncrementShopList,
) -> WorldIncrementShopConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = increment_shop_list.add_to_byte_array(&mut payload) {
        return WorldIncrementShopConfigurationReport {
            delivery: None,
            completion: WorldIncrementShopConfigurationCompletion::IncrementShop(error),
        };
    }

    WorldIncrementShopConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 4, &payload,
        )),
        completion: WorldIncrementShopConfigurationCompletion::ContributeSetupPending { socket_id },
    }
}

/// Шаг 5: contribute setup.
pub fn continue_game_server_contribute_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    contribute_setup: &CContributeSetup,
) -> WorldContributeConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = contribute_setup.add_to_byte_array(&mut payload) {
        return WorldContributeConfigurationReport {
            delivery: None,
            completion: WorldContributeConfigurationCompletion::ContributeSetup(error),
        };
    }

    WorldContributeConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 5, &payload,
        )),
        completion: WorldContributeConfigurationCompletion::PrisonConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x1D: prison configuration.
pub fn continue_game_server_prison_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    prison_conf: &PrisonConf,
) -> WorldPrisonConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = prison_conf.add_to_byte_array(&mut payload) {
        return WorldPrisonConfigurationReport {
            delivery: None,
            completion: WorldPrisonConfigurationCompletion::PrisonConf(error),
        };
    }

    WorldPrisonConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x1D, &payload,
        )),
        completion: WorldPrisonConfigurationCompletion::PreciousBoxConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x1E: precious box configuration.
pub fn continue_game_server_precious_box_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    precious_boxes: &PreciousBoxConf,
) -> WorldPreciousBoxConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = precious_boxes.add_to_byte_array(&mut payload) {
        return WorldPreciousBoxConfigurationReport {
            delivery: None,
            completion: WorldPreciousBoxConfigurationCompletion::PreciousBox(error),
        };
    }

    WorldPreciousBoxConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x1E, &payload,
        )),
        completion: WorldPreciousBoxConfigurationCompletion::FairyExpConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x20: fairy exp configuration.
pub fn continue_game_server_fairy_exp_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    fairy_exp: &CFairyExpConf,
) -> WorldFairyExpConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = fairy_exp.add_to_byte_array(&mut payload) {
        return WorldFairyExpConfigurationReport {
            delivery: None,
            completion: WorldFairyExpConfigurationCompletion::FairyExp(error),
        };
    }

    WorldFairyExpConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x20, &payload,
        )),
        completion: WorldFairyExpConfigurationCompletion::SynthesisConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x21: synthesis configuration.
pub fn continue_game_server_synthesis_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    synthesis: &CSynthesis,
) -> WorldSynthesisConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = synthesis.add_to_byte_array(&mut payload) {
        return WorldSynthesisConfigurationReport {
            delivery: None,
            completion: WorldSynthesisConfigurationCompletion::Synthesis(error),
        };
    }

    WorldSynthesisConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x21, &payload,
        )),
        completion: WorldSynthesisConfigurationCompletion::EquipmentComposeConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x30: equipment compose list.
pub fn continue_game_server_equipment_compose_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    equipment_compose_list: &EquipmentComposeList,
) -> WorldEquipmentComposeConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = equipment_compose_list.add_to_byte_array(&mut payload) {
        return WorldEquipmentComposeConfigurationReport {
            delivery: None,
            completion: WorldEquipmentComposeConfigurationCompletion::EquipmentCompose(error),
        };
    }

    WorldEquipmentComposeConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x30, &payload,
        )),
        completion:
            WorldEquipmentComposeConfigurationCompletion::NewSkillMonsterConfigurationPending {
                socket_id,
            },
    }
}

/// Шаг 0x22: new skill monster configuration.
pub fn continue_game_server_new_skill_monster_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    new_skill_monsters: &NewSkillMonsterConf,
) -> WorldNewSkillMonsterConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = new_skill_monsters.add_to_byte_array(&mut payload) {
        return WorldNewSkillMonsterConfigurationReport {
            delivery: None,
            completion: WorldNewSkillMonsterConfigurationCompletion::NewSkillMonster(error),
        };
    }

    WorldNewSkillMonsterConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x22, &payload,
        )),
        completion: WorldNewSkillMonsterConfigurationCompletion::GoodsDestroyConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x23: goods destroy setup.
pub fn continue_game_server_goods_destroy_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    goods_destroy: &GoodsDestroySetup,
) -> WorldGoodsDestroyConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = goods_destroy.add_to_byte_array(&mut payload) {
        return WorldGoodsDestroyConfigurationReport {
            delivery: None,
            completion: WorldGoodsDestroyConfigurationCompletion::GoodsDestroy(error),
        };
    }

    WorldGoodsDestroyConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x23, &payload,
        )),
        completion: WorldGoodsDestroyConfigurationCompletion::GlobeSetupConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 7: globe setup поверх region router.
pub fn continue_game_server_globe_setup_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    globe_setup: &GlobeSetupSnapshot,
    region_router: &RegionRouter,
) -> WorldGlobeSetupConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = globe_setup.add_to_byte_array(region_router, &mut payload) {
        return WorldGlobeSetupConfigurationReport {
            delivery: None,
            completion: WorldGlobeSetupConfigurationCompletion::RegionRouter(error),
        };
    }

    WorldGlobeSetupConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 7, &payload,
        )),
        completion: WorldGlobeSetupConfigurationCompletion::LogSystemConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 8: log system.
pub fn continue_game_server_log_system_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    log_system: &CLogSystem,
) -> WorldLogSystemConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = log_system.add_to_byte_array(&mut payload) {
        return WorldLogSystemConfigurationReport {
            delivery: None,
            completion: WorldLogSystemConfigurationCompletion::LogSystem(error),
        };
    }

    WorldLogSystemConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 8, &payload,
        )),
        completion: WorldLogSystemConfigurationCompletion::CountryParamConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x18: country parameters.
pub fn continue_game_server_country_param_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    country_param: &CCountryParam,
) -> WorldCountryParamConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = country_param.add_to_byte_array(&mut payload) {
        return WorldCountryParamConfigurationReport {
            delivery: None,
            completion: WorldCountryParamConfigurationCompletion::CountryParam(error),
        };
    }

    WorldCountryParamConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x18, &payload,
        )),
        completion: WorldCountryParamConfigurationCompletion::CountryHandlerConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x19: country handler. Таблица остаётся в старом пакете; вызывающая
/// сторона сериализует её в исходной позиции и передаёт payload outcome.
pub fn continue_game_server_country_handler_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    payload: Result<Vec<u8>, CountryHandlerSerializeError>,
) -> WorldCountryHandlerConfigurationReport {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => {
            return WorldCountryHandlerConfigurationReport {
                delivery: None,
                completion: WorldCountryHandlerConfigurationCompletion::CountryHandler(error),
            };
        }
    };

    WorldCountryHandlerConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x19, &payload,
        )),
        completion: WorldCountryHandlerConfigurationCompletion::GodsBattleConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x39: gods battle configuration.
pub fn continue_game_server_gods_battle_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    gods_battle: &CGodsBattleConf,
) -> WorldGodsBattleConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = gods_battle.add_to_byte_array(&mut payload) {
        return WorldGodsBattleConfigurationReport {
            delivery: None,
            completion: WorldGodsBattleConfigurationCompletion::GodsBattle(error),
        };
    }

    WorldGodsBattleConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x39, &payload,
        )),
        completion: WorldGodsBattleConfigurationCompletion::RegionSnapshotsPending { socket_id },
    }
}

/// Region snapshots `0x7F801`: assigned-регионы целевого GameServer с
/// subtype `0x0E` и паузой 100 мс после каждого, proxy-регионы с subtype
/// `0x0F` без паузы. Порядок обхода и остановка на первой ошибке снимка
/// принадлежат visitor-у владельца игры; отправленные сообщения не
/// откатываются.
pub fn continue_game_server_region_configurations<Snapshots, Delay>(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    mut snapshots: Snapshots,
    mut delay: Delay,
) -> WorldRegionConfigurationReport
where
    Snapshots: FnMut(
        &mut dyn FnMut(WorldInitialRegionSnapshot),
    ) -> Result<(), WorldInitialRegionSnapshotBlock>,
    Delay: FnMut(u32),
{
    let mut deliveries = Vec::new();
    let traversal = snapshots(&mut |snapshot: WorldInitialRegionSnapshot| {
        let delay_after_ms = match snapshot.kind {
            WorldInitialRegionSnapshotKind::Assigned { .. } => Some(100),
            WorldInitialRegionSnapshotKind::Proxy => None,
        };
        let mut message = CMessage::new(0x0007_F801);
        let (subtype, payload_length) = match snapshot.kind {
            WorldInitialRegionSnapshotKind::Assigned { region_type } => {
                message.base_mut().add_long(0x0E);
                message.base_mut().add_long(region_type);
                (0x0E, snapshot.payload.len() + 4)
            }
            WorldInitialRegionSnapshotKind::Proxy => {
                message.base_mut().add_long(0x0F);
                (0x0F, snapshot.payload.len())
            }
        };
        message.base_mut().add(&snapshot.payload);
        let delivery = message.send_to_socket(sender, socket_id);
        deliveries.push(WorldRegionConfigurationDelivery {
            map_key: snapshot.map_key,
            region_id: snapshot.region_id,
            kind: snapshot.kind,
            delivery: WorldInitialConfigurationDelivery {
                subtype,
                payload_length,
                target: WorldInitialConfigurationTarget::Socket(socket_id),
                delivery,
            },
            delay_after_ms,
        });
        if let Some(milliseconds) = delay_after_ms {
            delay(milliseconds);
        }
    });

    let completion = match traversal {
        Ok(()) => WorldRegionConfigurationCompletion::RegionSetupConfigurationPending { socket_id },
        Err(error) => WorldRegionConfigurationCompletion::RegionSnapshot(error),
    };
    WorldRegionConfigurationReport {
        deliveries,
        completion,
    }
}

/// Шаг 0x11: region setup.
pub fn continue_game_server_region_setup_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    region_setup: &CRegionSetup,
) -> WorldRegionSetupConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = region_setup.add_to_byte_array(&mut payload) {
        return WorldRegionSetupConfigurationReport {
            delivery: None,
            completion: WorldRegionSetupConfigurationCompletion::RegionSetup(error),
        };
    }

    WorldRegionSetupConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x11, &payload,
        )),
        completion: WorldRegionSetupConfigurationCompletion::DupliRegionSetupPending { socket_id },
    }
}

/// Шаг 0x1A: dupli region setup.
pub fn continue_game_server_dupli_region_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    dupli_region_setup: &CDupliRegionSetup,
) -> WorldDupliRegionConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = dupli_region_setup.add_to_byte_array(&mut payload) {
        return WorldDupliRegionConfigurationReport {
            delivery: None,
            completion: WorldDupliRegionConfigurationCompletion::DupliRegionSetup(error),
        };
    }

    WorldDupliRegionConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x1A, &payload,
        )),
        completion: WorldDupliRegionConfigurationCompletion::HonorEliminateConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x26: honor eliminate config, всегда 8-байтовый owned-снимок.
pub fn continue_game_server_honor_eliminate_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    honor_eliminate: HonorElimilateConfig,
) -> WorldHonorEliminateConfigurationReport {
    let mut payload = Vec::with_capacity(8);
    honor_eliminate.add_to_byte_array(&mut payload);
    WorldHonorEliminateConfigurationReport {
        delivery: send_initial_configuration_to_socket(sender, socket_id, 0x26, &payload),
        completion: WorldHonorEliminateConfigurationCompletion::HonorRanksPending { socket_id },
    }
}

/// Шаги 0x27–0x2A: honor ranks по четырём периодам; total-проход несёт
/// дополнительный нулевой префикс перед payload. Ошибка периода прекращает
/// цепочку, уже отправленные проходы не откатываются.
pub fn continue_game_server_honor_ranks_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    honor_ranks: &CHonorRanks,
) -> WorldHonorRanksConfigurationReport {
    const PASSES: [(HonorRanksType, i32, bool); 4] = [
        (HonorRanksType::Day, 0x27, false),
        (HonorRanksType::Week, 0x28, false),
        (HonorRanksType::Month, 0x29, false),
        (HonorRanksType::Total, 0x2A, true),
    ];

    let mut deliveries = Vec::with_capacity(PASSES.len());
    for (rank_type, subtype, has_total_prefix) in PASSES {
        let mut payload = Vec::new();
        if let Err(error) = honor_ranks.add_history_to_byte_array(&mut payload, rank_type, None) {
            return WorldHonorRanksConfigurationReport {
                deliveries,
                completion: WorldHonorRanksConfigurationCompletion::HonorRanks(error),
            };
        }

        let delivery = if has_total_prefix {
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(subtype);
            message.base_mut().add_long(0);
            message.base_mut().add(&payload);
            WorldInitialConfigurationDelivery {
                subtype,
                payload_length: payload.len() + 4,
                target: WorldInitialConfigurationTarget::Socket(socket_id),
                delivery: message.send_to_socket(sender, socket_id),
            }
        } else {
            send_initial_configuration_to_socket(sender, socket_id, subtype, &payload)
        };
        deliveries.push(WorldHonorRanksConfigurationDelivery {
            rank_type,
            delivery,
        });
    }

    WorldHonorRanksConfigurationReport {
        deliveries,
        completion: WorldHonorRanksConfigurationCompletion::FunctionListPending { socket_id },
    }
}

/// Шаги 0x0A/0x0B: сырые function/variable script lists, отсутствующий файл
/// пропускается без записи; файл, не помещающийся в legacy length, прекращает
/// цепочку до своего send.
pub fn continue_game_server_raw_script_lists_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    function_list: Option<&[u8]>,
    variable_list: Option<&[u8]>,
) -> WorldRawScriptListsConfigurationReport {
    let mut deliveries = Vec::with_capacity(2);
    for (kind, subtype, data) in [
        (WorldRawScriptListKind::Function, 0x0A, function_list),
        (WorldRawScriptListKind::Variable, 0x0B, variable_list),
    ] {
        let Some(data) = data else { continue };
        let length = match i32::try_from(data.len()) {
            Ok(length) => length,
            Err(_) => {
                return WorldRawScriptListsConfigurationReport {
                    deliveries,
                    completion: WorldRawScriptListsConfigurationCompletion::FileSize(
                        WorldRawScriptListConfigurationBlock {
                            kind,
                            length: data.len(),
                        },
                    ),
                };
            }
        };
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(subtype);
        message.base_mut().add_long(length);
        message.base_mut().add(data);
        deliveries.push(WorldRawScriptListConfigurationDelivery {
            kind,
            delivery: WorldInitialConfigurationDelivery {
                subtype,
                payload_length: data.len() + 4,
                target: WorldInitialConfigurationTarget::Socket(socket_id),
                delivery: message.send_to_socket(sender, socket_id),
            },
        });
    }

    WorldRawScriptListsConfigurationReport {
        deliveries,
        completion: WorldRawScriptListsConfigurationCompletion::GeneralVariableListPending {
            socket_id,
        },
    }
}

/// Шаг 0x0C: general variable list; отсутствующий список пропускается молча.
pub fn continue_game_server_general_variable_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    variables: Option<&CVariableList>,
) -> WorldGeneralVariableConfigurationReport {
    let Some(variables) = variables else {
        return WorldGeneralVariableConfigurationReport {
            delivery: None,
            completion: WorldGeneralVariableConfigurationCompletion::ScriptFilesPending {
                socket_id,
            },
        };
    };
    let mut payload = Vec::new();
    if let Err(error) = variables.add_to_byte_array(&mut payload) {
        return WorldGeneralVariableConfigurationReport {
            delivery: None,
            completion: WorldGeneralVariableConfigurationCompletion::VariableList(error),
        };
    }

    WorldGeneralVariableConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x0C, &payload,
        )),
        completion: WorldGeneralVariableConfigurationCompletion::ScriptFilesPending { socket_id },
    }
}

/// Шаг 0x0D: script-файлы `(path, data)` в обходном порядке владельца; файл,
/// не помещающийся в legacy length, прекращает цепочку до своего send.
pub fn continue_game_server_script_files_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    script_files: &[(&[u8], &[u8])],
) -> WorldScriptFilesConfigurationReport {
    let mut deliveries = Vec::new();
    for (path, data) in script_files {
        let declared_length = match i32::try_from(data.len()) {
            Ok(length) => length,
            Err(_) => {
                return WorldScriptFilesConfigurationReport {
                    deliveries,
                    completion: WorldScriptFilesConfigurationCompletion::FileSize(
                        WorldScriptFileConfigurationBlock {
                            path: path.to_vec(),
                            length: data.len(),
                        },
                    ),
                };
            }
        };
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x0D);
        message.base_mut().add(path);
        message.base_mut().add_byte(0);
        message.base_mut().add_long(declared_length);
        message.base_mut().add(data);
        message.base_mut().add_byte(0);
        deliveries.push(WorldScriptFileConfigurationDelivery {
            path: path.to_vec(),
            declared_length,
            delivery: WorldInitialConfigurationDelivery {
                subtype: 0x0D,
                payload_length: path.len() + 1 + 4 + data.len() + 1,
                target: WorldInitialConfigurationTarget::Socket(socket_id),
                delivery: message.send_to_socket(sender, socket_id),
            },
        });
    }

    WorldScriptFilesConfigurationReport {
        deliveries,
        completion: WorldScriptFilesConfigurationCompletion::QuestSystemPending { socket_id },
    }
}

/// Шаг 0x16: quest system.
pub fn continue_game_server_quest_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    quest_system: &CQuestSystem,
) -> WorldQuestConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = quest_system.add_to_byte_array(&mut payload) {
        return WorldQuestConfigurationReport {
            delivery: None,
            completion: WorldQuestConfigurationCompletion::QuestSystem(error),
        };
    }
    WorldQuestConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x16, &payload,
        )),
        completion: WorldQuestConfigurationCompletion::PlayerRanksPending { socket_id },
    }
}

/// Шаг 0x17: player ranks.
pub fn continue_game_server_player_ranks_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    player_ranks: &CPlayerRanks,
) -> WorldPlayerRanksConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = player_ranks.add_to_byte_array(&mut payload) {
        return WorldPlayerRanksConfigurationReport {
            delivery: None,
            completion: WorldPlayerRanksConfigurationCompletion::PlayerRanks(error),
        };
    }

    WorldPlayerRanksConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x17, &payload,
        )),
        completion: WorldPlayerRanksConfigurationCompletion::GmListPending { socket_id },
    }
}

/// Шаг 9: GM list.
pub fn continue_game_server_gm_list_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    game_server_index: u32,
    gm_list: &CGMList,
) -> WorldGmListConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = gm_list.add_to_byte_array(&mut payload) {
        return WorldGmListConfigurationReport {
            delivery: None,
            completion: WorldGmListConfigurationCompletion::GmList(error),
        };
    }

    WorldGmListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 9, &payload,
        )),
        completion: WorldGmListConfigurationCompletion::GameServerIndexPending {
            socket_id,
            game_server_index,
        },
    }
}

/// Шаг 0x12: однобайтовый индекс GameServer.
pub fn continue_game_server_index_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    game_server_index: u32,
) -> WorldGameServerIndexConfigurationReport {
    WorldGameServerIndexConfigurationReport {
        delivery: send_initial_configuration_to_socket(
            sender,
            socket_id,
            0x12,
            &[game_server_index as u8],
        ),
        completion: WorldGameServerIndexConfigurationCompletion::FourNationWarPending { socket_id },
    }
}

/// Шаг 0x25: four nation war.
pub fn continue_game_server_four_nation_war_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    four_nation_war: &CFourNationWarSys,
) -> WorldFourNationWarConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = four_nation_war.add_to_byte_array(&mut payload) {
        return WorldFourNationWarConfigurationReport {
            delivery: None,
            completion: WorldFourNationWarConfigurationCompletion::FourNationWar(error),
        };
    }

    WorldFourNationWarConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x25, &payload,
        )),
        completion: WorldFourNationWarConfigurationCompletion::BattleFairyExpConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x2C: battle fairy exp configuration.
pub fn continue_game_server_battle_fairy_exp_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    battle_fairy_exp: &CBattleFairyExpConfig,
) -> WorldBattleFairyExpConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = battle_fairy_exp.add_to_byte_array(&mut payload) {
        return WorldBattleFairyExpConfigurationReport {
            delivery: None,
            completion: WorldBattleFairyExpConfigurationCompletion::BattleFairyExp(error),
        };
    }

    WorldBattleFairyExpConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x2C, &payload,
        )),
        completion: WorldBattleFairyExpConfigurationCompletion::BattleFairyPropertyPending {
            socket_id,
        },
    }
}

/// Шаг 0x2D: battle fairy property (combine-сериализация).
pub fn continue_game_server_battle_fairy_property_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    battle_fairy_property: &CBattleFairyProperty,
) -> WorldBattleFairyPropertyConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = battle_fairy_property.serialize_combine(&mut payload) {
        return WorldBattleFairyPropertyConfigurationReport {
            delivery: None,
            completion: WorldBattleFairyPropertyConfigurationCompletion::BattleFairyProperty(error),
        };
    }

    WorldBattleFairyPropertyConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x2D, &payload,
        )),
        completion:
            WorldBattleFairyPropertyConfigurationCompletion::CiQingLingBaoConfigurationPending {
                socket_id,
            },
    }
}

/// Шаг 0x35: ciqing setup сразу за ним ling bao в одном payload.
pub fn continue_game_server_ciqing_ling_bao_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    ci_qing_setup: &CCiQingSetup,
    ling_bao: &CLingBaoSetup,
) -> WorldCiQingLingBaoConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = ci_qing_setup.add_byte_to_array(&mut payload) {
        return WorldCiQingLingBaoConfigurationReport {
            delivery: None,
            completion: WorldCiQingLingBaoConfigurationCompletion::CiQing(error),
        };
    }
    if let Err(error) = ling_bao.add_byte_ling_bao(&mut payload) {
        return WorldCiQingLingBaoConfigurationReport {
            delivery: None,
            completion: WorldCiQingLingBaoConfigurationCompletion::LingBao(error),
        };
    }

    WorldCiQingLingBaoConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x35, &payload,
        )),
        completion: WorldCiQingLingBaoConfigurationCompletion::TaoZhuangConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x34: tao zhuang setup.
pub fn continue_game_server_tao_zhuang_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    tao_zhuang_setup: &CTaoZhuangSetup,
) -> WorldTaoZhuangConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = tao_zhuang_setup.add_byte_to_array(&mut payload) {
        return WorldTaoZhuangConfigurationReport {
            delivery: None,
            completion: WorldTaoZhuangConfigurationCompletion::TaoZhuang(error),
        };
    }

    WorldTaoZhuangConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x34, &payload,
        )),
        completion: WorldTaoZhuangConfigurationCompletion::AttackCityConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x1B: attack city; исходный bool успеха сериализации игнорировался.
pub fn continue_game_server_attack_city_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    attack_city: &CAttackCitySys,
) -> WorldAttackCityConfigurationReport {
    let mut payload = Vec::new();
    let _legacy_success = attack_city.add_to_byte_array(&mut payload);
    WorldAttackCityConfigurationReport {
        delivery: send_initial_configuration_to_socket(sender, socket_id, 0x1B, &payload),
        completion: WorldAttackCityConfigurationCompletion::VillageWarConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x1C: village war; исходный bool успеха сериализации игнорировался.
pub fn continue_game_server_village_war_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    village_war: &CVillageWarSys,
) -> WorldVillageWarConfigurationReport {
    let mut payload = Vec::new();
    let _legacy_success = village_war.add_to_byte_array(&mut payload);
    WorldVillageWarConfigurationReport {
        delivery: send_initial_configuration_to_socket(sender, socket_id, 0x1C, &payload),
        completion: WorldVillageWarConfigurationCompletion::CountryWarConfigurationPending {
            socket_id,
        },
    }
}

/// Шаг 0x1F: country war. Система остаётся в старом пакете; вызывающая
/// сторона сериализует её (с исходным игнорируемым bool успеха) в той же
/// позиции и передаёт payload.
pub fn continue_game_server_country_war_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    payload: &[u8],
) -> WorldCountryWarConfigurationReport {
    WorldCountryWarConfigurationReport {
        delivery: send_initial_configuration_to_socket(sender, socket_id, 0x1F, payload),
        completion: WorldCountryWarConfigurationCompletion::GameServerIdentityPending { socket_id },
    }
}

/// Финал цепочки (subtype 0x3B): login server id и world number одним
/// 8-байтовым payload; отсутствующий world number прекращает цепочку без
/// отправки.
pub fn finish_game_server_initial_configuration(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    world_number: Option<u32>,
    login_server_id: i32,
) -> WorldGameServerIdentityReport {
    let Some(world_number) = world_number else {
        return WorldGameServerIdentityReport {
            delivery: None,
            completion: WorldGameServerIdentityCompletion::MissingWorldNumber { socket_id },
        };
    };

    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&login_server_id.to_le_bytes());
    payload.extend_from_slice(&world_number.to_le_bytes());
    WorldGameServerIdentityReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender, socket_id, 0x3B, &payload,
        )),
        completion: WorldGameServerIdentityCompletion::InitialConfigurationComplete { socket_id },
    }
}

fn send_initial_configuration_to_socket(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    subtype: i32,
    payload: &[u8],
) -> WorldInitialConfigurationDelivery {
    let mut message = CMessage::new(0x0007_F801);
    message.base_mut().add_long(subtype);
    message.base_mut().add(payload);
    WorldInitialConfigurationDelivery {
        subtype,
        payload_length: payload.len(),
        target: WorldInitialConfigurationTarget::Socket(socket_id),
        delivery: message.send_to_socket(sender, socket_id),
    }
}

fn send_initial_configuration_to_all(
    sender: Option<&ServerCommandHandle>,
    subtype: i32,
    payload: &[u8],
) -> WorldInitialConfigurationDelivery {
    let mut message = CMessage::new(0x0007_F801);
    message.base_mut().add_long(subtype);
    message.base_mut().add(payload);
    WorldInitialConfigurationDelivery {
        subtype,
        payload_length: payload.len(),
        target: WorldInitialConfigurationTarget::AllGameServers,
        delivery: message.send_all(sender),
    }
}

/// Reconnect-цепочка `0x5FA01` sync-ветки: acknowledgement `0x8040E` ставится
/// до разбора записей; неизвестный тип записи потребляет только свой tag;
/// каждый игрок декодируется владельцем и публикуется online; завершение —
/// CD-key snapshot LoginServer-у независимо от исхода отдельных записей.
pub fn continue_game_server_reconnect<Game>(
    game: &mut Game,
    socket_id: i32,
    payload: &[u8],
    registry: &GoodsBasePropertiesRegistry,
    organizing: &mut Game::OrganizingContext,
    coefficients: &PlayerPropertyCoefficients,
) -> WorldGameServerReconnectReport
where
    Game: WorldServerMessageGameView,
{
    let mut cursor = 0;
    let decoded_count = read_reconnect_long(payload, &mut cursor);
    let declared_count = decoded_count.unwrap_or(0);

    let acknowledgement_message = CMessage::new(0x0008_040E);
    let sender = game.current_game_server_sender();
    let acknowledgement = acknowledgement_message.send_to_socket(sender.as_ref(), socket_id);
    let mut records = Vec::new();

    let completion = if declared_count > 0
        && (declared_count as usize).saturating_mul(4) > payload.len().saturating_sub(cursor)
    {
        WorldGameServerReconnectCompletion::InvalidElementCount {
            declared_count,
            minimum_bytes: (declared_count as usize).saturating_mul(4),
            available_bytes: payload.len().saturating_sub(cursor),
        }
    } else {
        'decode: {
            for record_index in 0..declared_count.max(0) as usize {
                let Some(packet_type) = read_reconnect_long(payload, &mut cursor) else {
                    break 'decode WorldGameServerReconnectCompletion::UnexpectedEnd {
                        record_index,
                        field: "packet type",
                    };
                };
                if packet_type != 1 {
                    records.push(WorldGameServerReconnectRecord::Skipped { packet_type });
                    continue;
                }

                let Some(requested_player_id) = read_reconnect_long(payload, &mut cursor) else {
                    break 'decode WorldGameServerReconnectCompletion::UnexpectedEnd {
                        record_index,
                        field: "player ID",
                    };
                };
                let decoded = match game.decord_reconnected_player(
                    requested_player_id as u32,
                    payload,
                    &mut cursor,
                    registry,
                    coefficients,
                ) {
                    Ok(decoded) => decoded,
                    Err(error) => {
                        break 'decode WorldGameServerReconnectCompletion::PlayerCodec {
                            record_index,
                            error,
                        };
                    }
                };
                let online = game.append_online_player_id(organizing, decoded.decoded_player_id);
                let trailing = read_reconnect_long(payload, &mut cursor);
                records.push(WorldGameServerReconnectRecord::Player {
                    packet_type,
                    decoded,
                    online,
                    trailing_value: trailing.unwrap_or(0),
                    trailing_complete: trailing.is_some(),
                });
                if trailing.is_none() {
                    break 'decode WorldGameServerReconnectCompletion::UnexpectedEnd {
                        record_index,
                        field: "trailing long",
                    };
                }
            }
            WorldGameServerReconnectCompletion::CdkeySnapshot(game.send_cdkey_to_login_server())
        }
    };

    WorldGameServerReconnectReport {
        socket_id,
        declared_count,
        count_complete: decoded_count.is_some(),
        acknowledgement,
        records,
        completion,
    }
}

fn read_reconnect_long(source: &[u8], cursor: &mut usize) -> Option<i32> {
    let end = (*cursor).checked_add(4)?;
    let bytes: [u8; 4] = source.get(*cursor..end)?.try_into().ok()?;
    *cursor = end;
    Some(i32::from_le_bytes(bytes))
}

fn observed_inet_addr(ip: &[u8]) -> Result<u32, ()> {
    if let Ok(text) = std::str::from_utf8(ip)
        && let Ok(address) = text.parse::<Ipv4Addr>()
    {
        return Ok(u32::from_le_bytes(address.octets()));
    }
    if looks_like_legacy_numeric_ipv4(ip) {
        return Err(());
    }
    Ok(u32::MAX)
}

fn looks_like_legacy_numeric_ipv4(ip: &[u8]) -> bool {
    if ip.is_empty() {
        return false;
    }
    let parts = ip.split(|byte| *byte == b'.').collect::<Vec<_>>();
    (1..=4).contains(&parts.len())
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part.iter().all(u8::is_ascii_digit)
                    || (part.starts_with(b"0x") || part.starts_with(b"0X"))
                        && part.len() > 2
                        && part[2..].iter().all(u8::is_ascii_hexdigit))
        })
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

fn send_region_change_failure(
    game: &(impl WorldGameView + ?Sized),
    socket_id: i32,
    player_id: i32,
) -> Result<i32, SendMessageError> {
    let mut response = CMessage::new(0x0007_F802);
    response.base_mut().add_char(0);
    response.base_mut().add_long(player_id);
    let sender = game.current_game_server_sender();
    response.send_to_socket(sender.as_ref(), socket_id)
}
