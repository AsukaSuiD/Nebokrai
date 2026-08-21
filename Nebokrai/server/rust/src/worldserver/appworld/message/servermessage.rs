//! Обработчики server-семейств исторического WorldServer.
//!
//! Статус владельца: `IMPLEMENTED` для `gameserv_conn_log`,
//! внутрипроцессного события `0x3FC03`,
//! регистрации GameServer и reconnect player-data хвоста в `0x5FA01`,
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
//! Начальная часть `0x5FA01` читает signed `char`, 32-битный порт и ограниченную
//! C-строку IP, ищет её в настроенном реестре GameServer, ставит
//! `bConnected`, затем строго по порядку назначает socket map-ID, достигает
//! операторского сообщения о подключении, условно рассылает `0x80403` для
//! индекса `5`,
//! посылает `0x7F80D` и вызывает `gameserv_conn_log`. Неизвестный адрес
//! прекращает ветку после чтения payload. Отсутствующий сетевой owner и ещё не
//! материализованный `CGlobeSetup::bAuction` выражены позиционными безопасными
//! границами после уже выполненных эффектов, а не выдуманными значениями.
//!
//! Реальные `serverSetup.ini` содержат канонический `95.78.126.81` и hostname
//! `miracle_misc`: первый переводится в x86 IPv4 word, второй точно даёт
//! `INADDR_NONE`. Для иных числовых форм, которые старый `inet_addr` мог читать
//! как octal/hex/сокращённый адрес, отправка `0x1FE05` останавливается отдельной
//! границей после `0x7F80D`; строгий parser стандартной библиотеки не выдаётся
//! за полную Winsock-грамматику. Нулевой sync flag возвращает обязанность
//! продолжить большую цепочку начальной конфигурации; ненулевой — отдельный
//! хвост `count -> 0x8040E -> player snapshots -> CD-key snapshot`. Ни один
//! хвост не объявляется исполненным частично. Reconnect сначала безусловно
//! отправляет пустой `0x8040E`, затем для каждого packet type `1` декодирует
//! полный `CPlayer`, восстанавливает map/offline/online ownership и после
//! цикла отправляет CD-key snapshot. Неизвестный packet type потребляет только
//! собственный `long`, как исходник. Единственное техническое отличие — после
//! уже достигнутой отправки `0x8040E` заведомо невозможный по оставшимся байтам
//! положительный count останавливается typed-границей: это не меняет wire и
//! порядок наблюдаемых эффектов корректного сообщения, но не даёт входу
//! вызвать неограниченный пустой цикл.
//!
//! Начальная конфигурация материализована до следующего owner-а
//! `CMonsterList`: `0x7F801/0x2B` DaKong, `0x2F` StringTable, условный
//! `0x31` валидного WordsFilter, `0` фабрики товаров и broadcast `0x36`
//! ThingSetup. Первые четыре адресных сообщения уходят только новому socket,
//! а `0x36` — всем GameServer, даже уже подключённым. Результаты send исходник
//! игнорировал, поэтому ошибка очереди записывается в отчёт и не переставляет
//! последующие пакеты. `CGoodsFactory::Serialize` уже заменён его точным
//! serializer-ом; ещё сырые owner-ы передают готовые byte snapshots и тем
//! самым не объявляются восстановленными. Невозможный безопасный state
//! registry товаров останавливает цепочку после уже отправленного prefix-а,
//! вместо старого null-dereference либо выхода за 32-битный размер.
//! Следующий `CMonsterList` уже использует восстановленный общий setup-owner:
//! оба ordered registry кодируются им в пакет `0x7F801/2`, после чего точной
//! следующей границей остаётся `CHitLevelSetup`. Это адресный send только
//! подключившемуся socket; его результат также не управляет продолжением.
//! `CHitLevelSetup` затем строит `0x7F801/0x14` из своего восстановленного
//! `count + 12-byte records` owner-а и переводит ветку к `CPlayerList`.
//! `CPlayerList` кодирует пять доказанных секций и адресно отправляет subtype
//! `1`; следующая граница ветки — process-global `CEmotion::Serialize`.
//! Восстановленный `CEmotion` отправляется следом как `0x7F801/0x15`.
//! `CSkillFactory` затем сохраняет ordered slot framing и исторические восемь
//! байт padding каждого record-а, но обнуляет прежний heap-мусор, и адресно
//! отправляется как subtype `6`. `CTradeList` сохраняет следующий ordered
//! `C-string + count + 8-byte goods records` payload и уходит subtype `3`.
//! Singleton `CIncrementShopList` следом кодирует ordered multimap, точные
//! 24-байтные item-prefix-ы и affiche как subtype `4`. `CContributeSetup`
//! затем передаёт одиннадцать positional scalars и contribution items subtype
//! `5`. `PrisonConf` сохраняет signed-char key order и компактные десятибайтные
//! записи как subtype `0x1D`. Следующие восстановленные owner-ы доводят цепочку
//! через `PreciousBoxConf`, fairy exp, synthesis, equipment compose, new-skill
//! monsters и goods-destroy до общего `CGlobeSetup + CRegionRouter` subtype
//! `7`. `CLogSystem` затем сохраняет 64-байтный ABI snapshot и ordered signed
//! item set в subtype `8`. Уже восстановленный `CCountryParam` следом отправляет
//! 39 scalar-полей и пять ordered map-секций subtype `0x18`. После него
//! country-map с вложенными `CCountry` records уходит subtype `0x19` через
//! `CCountryHandler`. `CGodsBattleConf` затем передаёт семь positional секций
//! subtype `0x39`; явный старый `Update` уже является инвариантом каждого
//! `CBaseMessage::add`. Ordered region map следом отправляет назначенные этому
//! GameServer регионы как subtype `0x0E + region type + full snapshot`, а
//! остальные — как subtype `0x0F + proxy snapshot`. Старый `Sleep(100)` после
//! каждого назначенного региона выражен injected delay-callback-ом: wire-order
//! и точка задержки сохранены без навязывания Rust-слою конкретного runtime-а.
//! Общий `CRegionSetup` затем сохраняет signed count и ordered 12-байтные
//! records в subtype `0x11`. `CDupliRegionSetup` следом передаёт insertion-order
//! пары region/duplicate-region subtype `0x1A`. `HonorElimilateConfig` затем
//! отправляет два signed scalar-а subtype `0x26`. Четыре history-среза
//! `CHonorRanks` идут subtype `0x27..0x2A`; total-пакет сохраняет отдельный
//! positional ноль перед тем же rank payload. Nullable function/variable
//! file-data следуют subtype `10/11` как `signed size + exact raw bytes`, без
//! C-string преобразования. Nullable general `CVariableList` затем передаёт
//! `count + payload length + tagged values` subtype `12`. Ordered script-file
//! map следует отдельными subtype `13`: C-string path, signed `lstrlenA` data
//! и C-string data. `CQuestSystem` затем передаёт setup и ordered quest records
//! subtype `0x16`. `CPlayerRanks` следом отправляет insertion-order рейтинг
//! subtype `0x17`. `CGMList` затем передаёт два ordered name/level map-а и god
//! passport subtype `9`. GameServer index следом передаётся subtype `0x12` как
//! точный narrowing cast `u32 -> u8`; старшее содержимое исходного `dwIndex`
//! отбрасывается, как в EXE. Следующая граница — `CFourNationWarSys` subtype
//! `0x25`.
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

use crate::dbaccess::worlddb::rsplayer::HonorRanksType;
use crate::nets::basemessage::CBaseMessage;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::networld::mynetclient::CMyNetClient;
use crate::nets::servers::ServerCommandHandle;
use crate::public::dupliregionsetup::{CDupliRegionSetup, DupliRegionSerializeError};
use crate::public::equipmentcomposelist::{
    EquipmentComposeList, EquipmentComposeSerializeError,
};
use crate::setup::cbattlefairyexpconfig::{
    BattleFairyExpSerializeError, CBattleFairyExpConfig,
};
use crate::setup::contributesetup::{CContributeSetup, ContributeSetupSerializeError};
use crate::setup::emotion::{CEmotion, EmotionSerializeError};
use crate::setup::goodsdestructionconfig::{GoodsDestroySerializeError, GoodsDestroySetup};
use crate::setup::gmlist::{CGMList, GmListSerializationBlock};
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::godsbattleconf::{CGodsBattleConf, GodsBattleSerializeError};
use crate::setup::hitlevelsetup::{CHitLevelSetup, HitLevelSerializeError};
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::incrementshoplist::{CIncrementShopList, IncrementShopSerializeError};
use crate::setup::logsystem::{CLogSystem, LogSystemSerializeError};
use crate::setup::monsterlist::{
    MonsterDropRegistry, MonsterListSerializeError, MonsterRegistry, serialize_monster_list,
};
use crate::setup::newskillmonsterlist::{
    NewSkillMonsterConf, NewSkillMonsterSerializeError,
};
use crate::setup::playerlist::{CPlayerList, PlayerListSerializeError};
use crate::setup::preciousboxconf::{PreciousBoxConf, PreciousBoxSerializeError};
use crate::setup::prisonconf::{PrisonConf, PrisonConfSerializeError};
use crate::setup::questsystem::{CQuestSystem, QuestSystemSerializationBlock};
use crate::setup::regionsetup::{CRegionSetup, RegionSetupSerializeError};
use crate::setup::regionrouter::{RegionRouter, RegionRouterSerializeError};
use crate::setup::synthesis::{CSynthesis, SynthesisSerializeError};
use crate::setup::tradelist::{CTradeList, TradeListSerializeError};
use crate::worldserver::appworld::country::country::CountryKingSaveLimits;
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParamSerializationBlock,
};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryHandlerSerializeError,
};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsRegistrySerializeError, serialize_goods_registry,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::CFactionWarSys;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::player::{PlayerCodecError, PlayerPropertyCoefficients};
use crate::worldserver::appworld::script::variablelist::{
    CVariableList, VariableListSerializationBlock,
};
use crate::worldserver::appworld::skills::skillfactory::{
    CSkillFactory, SkillFactorySerializeError,
};
use crate::worldserver::worldserver::game::{
    CGame, WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldGameServerLookupError,
    WorldGenerateDbDataBlock, WorldGenerateDbDataReport, WorldGlobeVariablesDelivery,
    WorldInitialRegionSnapshot, WorldInitialRegionSnapshotBlock, WorldInitialRegionSnapshotKind,
    WorldOnlinePlayerAppendOutcome, WorldPingGameServerInfo, WorldReconnectedPlayerDecode,
    WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest, prepare_save_thread_launch,
};
use crate::worldserver::worldserver::honorranks::{CHonorRanks, HonorRanksSerializationBlock};
use crate::worldserver::worldserver::playerranks::{
    CPlayerRanks, PlayerRanksSerializationBlock,
};

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
    GameServerConnection(WorldGameServerConnectionReport),
    GameServerBroadcast(WorldGameServerBroadcast),
    GameServerPingResponseRecorded(WorldGameServerPingResponse),
    GameServerPingStarted(WorldGameServerPingStart),
    LoginServerTupleRelay(WorldLoginServerTupleRelay),
    LoginServerIdentityAssigned(WorldLoginServerIdentity),
    OpaqueFieldsRead(WorldOpaqueServerFields),
    RegionMessageRelayed(WorldRegionMessageRelay),
}

/// Следующая точная позиция ветки `0x5FA01` после достигнутой начальной части.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameServerConnectionContinuation {
    NotConfigured,
    NetworkOwnerUnavailable,
    AuctionStateUnavailable,
    LegacyIpv4SyntaxUnknown,
    InitialConfigurationPending {
        socket_id: i32,
        game_server_index: u32,
    },
    ReconnectPlayerDataPending {
        socket_id: i32,
        game_server_index: u32,
        remaining_payload: Vec<u8>,
    },
    RegistryPortUnavailable {
        game_server_index: u32,
    },
}

/// Условный broadcast auction-state для специального GameServer `5`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerAuctionBroadcast {
    pub(crate) message_type: i32,
    pub(crate) enabled: bool,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Наблюдаемые эффекты достигнутой части `OnServerMessage(0x5FA01)`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerConnectionReport {
    pub(crate) sync_flag: i8,
    pub(crate) sync_flag_complete: bool,
    pub(crate) port: u32,
    pub(crate) port_complete: bool,
    pub(crate) ip: Vec<u8>,
    pub(crate) socket_id: i32,
    pub(crate) game_server_index: Option<u32>,
    pub(crate) previous_connected: Option<bool>,
    pub(crate) route_assignment: Option<i32>,
    pub(crate) connected_notice: bool,
    pub(crate) auction: Option<WorldGameServerAuctionBroadcast>,
    pub(crate) globe_variables: Option<WorldGlobeVariablesDelivery>,
    pub(crate) login_log: Option<WorldGameServerConnectedLog>,
    pub(crate) continuation: WorldGameServerConnectionContinuation,
}

/// Уже сериализованные owner-снимки достигнутого prefix-а initial-config.
pub(crate) struct WorldGameServerInitialConfigurationPrefix<'a> {
    pub(crate) da_kong_xiang_qian: &'a [u8],
    pub(crate) string_table: &'a [u8],
    pub(crate) valid_words_filter: Option<&'a [u8]>,
    pub(crate) goods_registry: &'a GoodsBasePropertiesRegistry,
    pub(crate) thing_setup: &'a [u8],
}

/// Получатель одного `0x7F801` initial-config сообщения.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldInitialConfigurationTarget {
    Socket(i32),
    AllGameServers,
}

/// Наблюдаемый результат одного send в initial-config prefix-е.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldInitialConfigurationDelivery {
    pub(crate) subtype: i32,
    pub(crate) payload_length: usize,
    pub(crate) target: WorldInitialConfigurationTarget,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Следующая точная позиция после достигнутого initial-config prefix-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldInitialConfigurationPrefixCompletion {
    GoodsRegistry(GoodsRegistrySerializeError),
    MonsterListPending { socket_id: i32 },
}

/// Отчёт prefix-а ветки `0x5FA01` с нулевым sync flag.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldInitialConfigurationPrefixReport {
    pub(crate) deliveries: Vec<WorldInitialConfigurationDelivery>,
    pub(crate) language_notice: bool,
    pub(crate) words_filter_notice: bool,
    pub(crate) completion: WorldInitialConfigurationPrefixCompletion,
}

/// Следующая позиция ветки после сериализации общего monster owner-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMonsterConfigurationCompletion {
    MonsterList(MonsterListSerializeError),
    HitLevelSetupPending { socket_id: i32 },
}

/// Отчёт отправки `CMonsterList` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMonsterConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldMonsterConfigurationCompletion,
}

/// Следующая позиция ветки после `CHitLevelSetup`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHitLevelConfigurationCompletion {
    HitLevelSetup(HitLevelSerializeError),
    PlayerListPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x14` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldHitLevelConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldHitLevelConfigurationCompletion,
}

/// Следующая позиция ветки после `CPlayerList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerListConfigurationCompletion {
    PlayerList(PlayerListSerializeError),
    EmotionPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/1` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerListConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldPlayerListConfigurationCompletion,
}

/// Следующая позиция ветки после `CEmotion`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldEmotionConfigurationCompletion {
    Emotion(EmotionSerializeError),
    SkillFactoryPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x15` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldEmotionConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldEmotionConfigurationCompletion,
}

/// Следующая позиция ветки после `CSkillFactory`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldSkillConfigurationCompletion {
    SkillFactory(SkillFactorySerializeError),
    TradeListPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/6` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSkillConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldSkillConfigurationCompletion,
}

/// Следующая позиция ветки после `CTradeList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldTradeListConfigurationCompletion {
    TradeList(TradeListSerializeError),
    IncrementShopListPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/3` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldTradeListConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldTradeListConfigurationCompletion,
}

/// Следующая позиция ветки после `CIncrementShopList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldIncrementShopConfigurationCompletion {
    IncrementShop(IncrementShopSerializeError),
    ContributeSetupPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/4` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldIncrementShopConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldIncrementShopConfigurationCompletion,
}

/// Следующая позиция ветки после `CContributeSetup`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldContributeConfigurationCompletion {
    ContributeSetup(ContributeSetupSerializeError),
    PrisonConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/5` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldContributeConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldContributeConfigurationCompletion,
}

/// Следующая позиция ветки после `PrisonConf`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPrisonConfigurationCompletion {
    PrisonConf(PrisonConfSerializeError),
    PreciousBoxConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x1D` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPrisonConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldPrisonConfigurationCompletion,
}

/// Следующая позиция ветки после `PreciousBoxConf`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPreciousBoxConfigurationCompletion {
    PreciousBox(PreciousBoxSerializeError),
    FairyExpConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x1E` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPreciousBoxConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldPreciousBoxConfigurationCompletion,
}

/// Следующая позиция ветки после `CFairyExpConf`/base serializer-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldFairyExpConfigurationCompletion {
    FairyExp(BattleFairyExpSerializeError),
    SynthesisConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x20` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFairyExpConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldFairyExpConfigurationCompletion,
}

/// Следующая позиция ветки после `CSynthesis`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldSynthesisConfigurationCompletion {
    Synthesis(SynthesisSerializeError),
    EquipmentComposeConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x21` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSynthesisConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldSynthesisConfigurationCompletion,
}

/// Следующая позиция ветки после `EquipmentComposeList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldEquipmentComposeConfigurationCompletion {
    EquipmentCompose(EquipmentComposeSerializeError),
    NewSkillMonsterConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x30` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldEquipmentComposeConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldEquipmentComposeConfigurationCompletion,
}

/// Следующая позиция ветки после `CNewSkillMonserConf`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldNewSkillMonsterConfigurationCompletion {
    NewSkillMonster(NewSkillMonsterSerializeError),
    GoodsDestroyConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x22` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldNewSkillMonsterConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldNewSkillMonsterConfigurationCompletion,
}

/// Следующая позиция ветки после `CGoodsDestroySetup`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGoodsDestroyConfigurationCompletion {
    GoodsDestroy(GoodsDestroySerializeError),
    GlobeSetupConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x23` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGoodsDestroyConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldGoodsDestroyConfigurationCompletion,
}

/// Следующая позиция ветки после общего `CGlobeSetup + CRegionRouter` payload.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGlobeSetupConfigurationCompletion {
    RegionRouter(RegionRouterSerializeError),
    LogSystemConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/7` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGlobeSetupConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldGlobeSetupConfigurationCompletion,
}

/// Следующая позиция ветки после `CLogSystem`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLogSystemConfigurationCompletion {
    LogSystem(LogSystemSerializeError),
    CountryParamConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/8` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldLogSystemConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldLogSystemConfigurationCompletion,
}

/// Следующая позиция ветки после `CCountryParam`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryParamConfigurationCompletion {
    CountryParam(CountryParamSerializationBlock),
    CountryHandlerConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x18` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryParamConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldCountryParamConfigurationCompletion,
}

/// Следующая позиция ветки после `CCountryHandler`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryHandlerConfigurationCompletion {
    CountryHandler(CountryHandlerSerializeError),
    GodsBattleConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x19` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryHandlerConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldCountryHandlerConfigurationCompletion,
}

/// Следующая позиция ветки после `CGodsBattleConf`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGodsBattleConfigurationCompletion {
    GodsBattle(GodsBattleSerializeError),
    RegionSnapshotsPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x39` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGodsBattleConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldGodsBattleConfigurationCompletion,
}

/// Наблюдаемая отправка одного элемента ordered region map.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionConfigurationDelivery {
    pub(crate) map_key: i32,
    pub(crate) region_id: i32,
    pub(crate) kind: WorldInitialRegionSnapshotKind,
    pub(crate) delivery: WorldInitialConfigurationDelivery,
    pub(crate) delay_after_ms: Option<u32>,
}

/// Следующая точная позиция ветки после initial-config region traversal.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionConfigurationCompletion {
    RegionSnapshot(WorldInitialRegionSnapshotBlock),
    RegionSetupConfigurationPending { socket_id: i32 },
}

/// Частичный или полный отчёт ordered region snapshot прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionConfigurationReport {
    pub(crate) deliveries: Vec<WorldRegionConfigurationDelivery>,
    pub(crate) completion: WorldRegionConfigurationCompletion,
}

/// Следующая позиция ветки после общего `CRegionSetup`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionSetupConfigurationCompletion {
    RegionSetup(RegionSetupSerializeError),
    DupliRegionSetupPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x11` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionSetupConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldRegionSetupConfigurationCompletion,
}

/// Следующая позиция ветки после `CDupliRegionSetup`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldDupliRegionConfigurationCompletion {
    DupliRegionSetup(DupliRegionSerializeError),
    HonorEliminateConfigurationPending { socket_id: i32 },
}

/// Отчёт отправки `0x7F801/0x1A` новому GameServer.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldDupliRegionConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldDupliRegionConfigurationCompletion,
}

/// Следующая позиция ветки после `HonorElimilateConfig`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorEliminateConfigurationCompletion {
    HonorRanksPending { socket_id: i32 },
}

/// Отчёт фиксированной отправки `HonorElimilateConfig` subtype `0x26`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorEliminateConfigurationReport {
    pub(crate) delivery: WorldInitialConfigurationDelivery,
    pub(crate) completion: WorldHonorEliminateConfigurationCompletion,
}

/// Одна из четырёх initial-config history-таблиц `CHonorRanks`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorRanksConfigurationDelivery {
    pub(crate) rank_type: HonorRanksType,
    pub(crate) delivery: WorldInitialConfigurationDelivery,
}

/// Следующая точная позиция после четырёх history-таблиц.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorRanksConfigurationCompletion {
    HonorRanks(HonorRanksSerializationBlock),
    FunctionListPending { socket_id: i32 },
}

/// Частичный или полный отчёт отправки subtype `0x27..0x2A`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorRanksConfigurationReport {
    pub(crate) deliveries: Vec<WorldHonorRanksConfigurationDelivery>,
    pub(crate) completion: WorldHonorRanksConfigurationCompletion,
}

/// Один из двух nullable raw script-list owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldRawScriptListKind {
    Function,
    Variable,
}

/// Невозможный для исходного signed `long` размер файла.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldRawScriptListConfigurationBlock {
    pub(crate) kind: WorldRawScriptListKind,
    pub(crate) length: usize,
}

/// Одна реально состоявшаяся nullable-отправка subtype `0x0A/0x0B`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRawScriptListConfigurationDelivery {
    pub(crate) kind: WorldRawScriptListKind,
    pub(crate) delivery: WorldInitialConfigurationDelivery,
}

/// Следующая позиция ветки после function/variable raw file-data.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldRawScriptListsConfigurationCompletion {
    FileSize(WorldRawScriptListConfigurationBlock),
    GeneralVariableListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRawScriptListsConfigurationReport {
    pub(crate) deliveries: Vec<WorldRawScriptListConfigurationDelivery>,
    pub(crate) completion: WorldRawScriptListsConfigurationCompletion,
}

/// Следующая позиция после nullable general variable-list.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGeneralVariableConfigurationCompletion {
    VariableList(VariableListSerializationBlock),
    ScriptFilesPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGeneralVariableConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldGeneralVariableConfigurationCompletion,
}

/// Невозможная signed length одного `lstrlenA(script_data)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldScriptFileConfigurationBlock {
    pub(crate) path: Vec<u8>,
    pub(crate) length: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldScriptFileConfigurationDelivery {
    pub(crate) path: Vec<u8>,
    pub(crate) declared_length: i32,
    pub(crate) delivery: WorldInitialConfigurationDelivery,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldScriptFilesConfigurationCompletion {
    FileSize(WorldScriptFileConfigurationBlock),
    QuestSystemPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldScriptFilesConfigurationReport {
    pub(crate) deliveries: Vec<WorldScriptFileConfigurationDelivery>,
    pub(crate) completion: WorldScriptFilesConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldQuestConfigurationCompletion {
    QuestSystem(QuestSystemSerializationBlock),
    PlayerRanksPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldQuestConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldQuestConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerRanksConfigurationCompletion {
    PlayerRanks(PlayerRanksSerializationBlock),
    GmListPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerRanksConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldPlayerRanksConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmListConfigurationCompletion {
    GmList(GmListSerializationBlock),
    GameServerIndexPending {
        socket_id: i32,
        game_server_index: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGmListConfigurationReport {
    pub(crate) delivery: Option<WorldInitialConfigurationDelivery>,
    pub(crate) completion: WorldGmListConfigurationCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGameServerIndexConfigurationCompletion {
    FourNationWarPending { socket_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerIndexConfigurationReport {
    pub(crate) delivery: WorldInitialConfigurationDelivery,
    pub(crate) completion: WorldGameServerIndexConfigurationCompletion,
}

/// Один элемент reconnect-хвоста после обязательного packet type.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGameServerReconnectRecord {
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

/// Точная достигнутая точка завершения reconnect player-data хвоста.
#[derive(Debug)]
pub(crate) enum WorldGameServerReconnectCompletion {
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

/// Полный отчёт продолжения reconnect-ветки `0x5FA01` после общего prefix-а.
#[derive(Debug)]
pub(crate) struct WorldGameServerReconnectReport {
    pub(crate) socket_id: i32,
    pub(crate) declared_count: i32,
    pub(crate) count_complete: bool,
    pub(crate) acknowledgement: Result<i32, SendMessageError>,
    pub(crate) records: Vec<WorldGameServerReconnectRecord>,
    pub(crate) completion: WorldGameServerReconnectCompletion,
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
        0x0005_FA01 => {
            WorldServerMessageDispatch::Handled(WorldServerMessageOutcome::GameServerConnection(
                on_game_server_connected(game, &mut message, None),
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

/// Исполняет начало `0x5FA01`; auction-state остаётся у отдельного setup-owner-а.
pub(crate) fn on_game_server_connected(
    game: &mut CGame,
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

/// Отправляет точный prefix начальной конфигурации до `CMonsterList`.
pub(crate) fn continue_game_server_initial_configuration_prefix(
    game: &CGame,
    socket_id: i32,
    snapshots: WorldGameServerInitialConfigurationPrefix<'_>,
) -> WorldInitialConfigurationPrefixReport {
    let sender = game.current_game_server_sender();
    let mut deliveries = Vec::with_capacity(5);
    deliveries.push(send_initial_configuration_to_socket(
        sender.as_ref(),
        socket_id,
        0x2B,
        snapshots.da_kong_xiang_qian,
    ));
    deliveries.push(send_initial_configuration_to_socket(
        sender.as_ref(),
        socket_id,
        0x2F,
        snapshots.string_table,
    ));
    let language_notice = true;

    let words_filter_notice = if let Some(words_filter) = snapshots.valid_words_filter {
        deliveries.push(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x31,
            words_filter,
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
        sender.as_ref(),
        socket_id,
        0,
        &goods,
    ));
    deliveries.push(send_initial_configuration_to_all(
        sender.as_ref(),
        0x36,
        snapshots.thing_setup,
    ));

    WorldInitialConfigurationPrefixReport {
        deliveries,
        language_notice,
        words_filter_notice,
        completion: WorldInitialConfigurationPrefixCompletion::MonsterListPending { socket_id },
    }
}

/// Кодирует и отправляет точный `0x7F801/2` monster-list packet.
pub(crate) fn continue_game_server_monster_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldMonsterConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            2,
            &payload,
        )),
        completion: WorldMonsterConfigurationCompletion::HitLevelSetupPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CHitLevelSetup` initial-config packet.
pub(crate) fn continue_game_server_hit_level_configuration(
    game: &CGame,
    socket_id: i32,
    hit_levels: &CHitLevelSetup,
) -> WorldHitLevelConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = hit_levels.add_to_byte_array(&mut payload) {
        return WorldHitLevelConfigurationReport {
            delivery: None,
            completion: WorldHitLevelConfigurationCompletion::HitLevelSetup(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldHitLevelConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x14,
            &payload,
        )),
        completion: WorldHitLevelConfigurationCompletion::PlayerListPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CPlayerList` initial-config packet.
pub(crate) fn continue_game_server_player_list_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldPlayerListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            1,
            &payload,
        )),
        completion: WorldPlayerListConfigurationCompletion::EmotionPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CEmotion` initial-config packet.
pub(crate) fn continue_game_server_emotion_configuration(
    game: &CGame,
    socket_id: i32,
    emotions: &CEmotion,
) -> WorldEmotionConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = emotions.serialize(&mut payload) {
        return WorldEmotionConfigurationReport {
            delivery: None,
            completion: WorldEmotionConfigurationCompletion::Emotion(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldEmotionConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x15,
            &payload,
        )),
        completion: WorldEmotionConfigurationCompletion::SkillFactoryPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CSkillFactory` initial-config packet.
pub(crate) fn continue_game_server_skill_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldSkillConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            6,
            &payload,
        )),
        completion: WorldSkillConfigurationCompletion::TradeListPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CTradeList` initial-config packet.
pub(crate) fn continue_game_server_trade_list_configuration(
    game: &CGame,
    socket_id: i32,
    trades: &CTradeList,
) -> WorldTradeListConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = trades.add_to_byte_array(&mut payload) {
        return WorldTradeListConfigurationReport {
            delivery: None,
            completion: WorldTradeListConfigurationCompletion::TradeList(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldTradeListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            3,
            &payload,
        )),
        completion: WorldTradeListConfigurationCompletion::IncrementShopListPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CIncrementShopList` initial-config packet.
pub(crate) fn continue_game_server_increment_shop_configuration(
    game: &CGame,
    socket_id: i32,
    increment_shop: &CIncrementShopList,
) -> WorldIncrementShopConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = increment_shop.add_to_byte_array(&mut payload) {
        return WorldIncrementShopConfigurationReport {
            delivery: None,
            completion: WorldIncrementShopConfigurationCompletion::IncrementShop(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldIncrementShopConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            4,
            &payload,
        )),
        completion: WorldIncrementShopConfigurationCompletion::ContributeSetupPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CContributeSetup` initial-config packet.
pub(crate) fn continue_game_server_contribute_configuration(
    game: &CGame,
    socket_id: i32,
    contribute: &CContributeSetup,
) -> WorldContributeConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = contribute.add_to_byte_array(&mut payload) {
        return WorldContributeConfigurationReport {
            delivery: None,
            completion: WorldContributeConfigurationCompletion::ContributeSetup(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldContributeConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            5,
            &payload,
        )),
        completion: WorldContributeConfigurationCompletion::PrisonConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `PrisonConf` initial-config packet.
pub(crate) fn continue_game_server_prison_configuration(
    game: &CGame,
    socket_id: i32,
    prison: &PrisonConf,
) -> WorldPrisonConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = prison.add_to_byte_array(&mut payload) {
        return WorldPrisonConfigurationReport {
            delivery: None,
            completion: WorldPrisonConfigurationCompletion::PrisonConf(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldPrisonConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x1D,
            &payload,
        )),
        completion: WorldPrisonConfigurationCompletion::PreciousBoxConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `PreciousBoxConf` initial-config packet.
pub(crate) fn continue_game_server_precious_box_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldPreciousBoxConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x1E,
            &payload,
        )),
        completion: WorldPreciousBoxConfigurationCompletion::FairyExpConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует общий base-state `CFairyExpConf` и отправляет exact packet.
pub(crate) fn continue_game_server_fairy_exp_configuration(
    game: &CGame,
    socket_id: i32,
    fairy_exp: &CBattleFairyExpConfig,
) -> WorldFairyExpConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = fairy_exp.add_to_byte_array(&mut payload) {
        return WorldFairyExpConfigurationReport {
            delivery: None,
            completion: WorldFairyExpConfigurationCompletion::FairyExp(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldFairyExpConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x20,
            &payload,
        )),
        completion: WorldFairyExpConfigurationCompletion::SynthesisConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CSynthesis` initial-config packet.
pub(crate) fn continue_game_server_synthesis_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldSynthesisConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x21,
            &payload,
        )),
        completion: WorldSynthesisConfigurationCompletion::EquipmentComposeConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `EquipmentComposeList` initial-config packet.
pub(crate) fn continue_game_server_equipment_compose_configuration(
    game: &CGame,
    socket_id: i32,
    equipment_compose: &EquipmentComposeList,
) -> WorldEquipmentComposeConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = equipment_compose.add_to_byte_array(&mut payload) {
        return WorldEquipmentComposeConfigurationReport {
            delivery: None,
            completion: WorldEquipmentComposeConfigurationCompletion::EquipmentCompose(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldEquipmentComposeConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x30,
            &payload,
        )),
        completion:
            WorldEquipmentComposeConfigurationCompletion::NewSkillMonsterConfigurationPending {
                socket_id,
            },
    }
}

/// Кодирует и отправляет точный `CNewSkillMonserConf` initial-config packet.
pub(crate) fn continue_game_server_new_skill_monster_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldNewSkillMonsterConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x22,
            &payload,
        )),
        completion: WorldNewSkillMonsterConfigurationCompletion::GoodsDestroyConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CGoodsDestroySetup` initial-config packet.
pub(crate) fn continue_game_server_goods_destroy_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldGoodsDestroyConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x23,
            &payload,
        )),
        completion: WorldGoodsDestroyConfigurationCompletion::GlobeSetupConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует единый `CGlobeSetup + CRegionRouter` payload и отправляет packet.
pub(crate) fn continue_game_server_globe_setup_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldGlobeSetupConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            7,
            &payload,
        )),
        completion: WorldGlobeSetupConfigurationCompletion::LogSystemConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CLogSystem` initial-config packet.
pub(crate) fn continue_game_server_log_system_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldLogSystemConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            8,
            &payload,
        )),
        completion: WorldLogSystemConfigurationCompletion::CountryParamConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CCountryParam` initial-config packet.
pub(crate) fn continue_game_server_country_param_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldCountryParamConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x18,
            &payload,
        )),
        completion: WorldCountryParamConfigurationCompletion::CountryHandlerConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CCountryHandler` initial-config packet.
pub(crate) fn continue_game_server_country_handler_configuration(
    game: &CGame,
    socket_id: i32,
    country_handler: &CCountryHandler,
) -> WorldCountryHandlerConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = country_handler.add_to_byte_array(&mut payload) {
        return WorldCountryHandlerConfigurationReport {
            delivery: None,
            completion: WorldCountryHandlerConfigurationCompletion::CountryHandler(error),
        };
    }

    let sender = game.current_game_server_sender();
    WorldCountryHandlerConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x19,
            &payload,
        )),
        completion: WorldCountryHandlerConfigurationCompletion::GodsBattleConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CGodsBattleConf` initial-config packet.
pub(crate) fn continue_game_server_gods_battle_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldGodsBattleConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x39,
            &payload,
        )),
        completion: WorldGodsBattleConfigurationCompletion::RegionSnapshotsPending { socket_id },
    }
}

/// Отправляет initial-config снимки регионов в точном signed map-order.
pub(crate) fn continue_game_server_region_configurations<Delay>(
    game: &CGame,
    socket_id: i32,
    game_server_index: u32,
    mut delay: Delay,
) -> WorldRegionConfigurationReport
where
    Delay: FnMut(u32),
{
    let sender = game.current_game_server_sender();
    let mut deliveries = Vec::new();
    let traversal = game.visit_initial_region_snapshots(
        game_server_index,
        |snapshot: WorldInitialRegionSnapshot| {
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
            let delivery = message.send_to_socket(sender.as_ref(), socket_id);
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
        },
    );

    let completion = match traversal {
        Ok(()) => WorldRegionConfigurationCompletion::RegionSetupConfigurationPending {
            socket_id,
        },
        Err(error) => WorldRegionConfigurationCompletion::RegionSnapshot(error),
    };
    WorldRegionConfigurationReport {
        deliveries,
        completion,
    }
}

/// Кодирует и отправляет точный общий `CRegionSetup` initial-config packet.
pub(crate) fn continue_game_server_region_setup_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldRegionSetupConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x11,
            &payload,
        )),
        completion: WorldRegionSetupConfigurationCompletion::DupliRegionSetupPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `CDupliRegionSetup` initial-config packet.
pub(crate) fn continue_game_server_dupli_region_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldDupliRegionConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x1A,
            &payload,
        )),
        completion: WorldDupliRegionConfigurationCompletion::HonorEliminateConfigurationPending {
            socket_id,
        },
    }
}

/// Кодирует и отправляет точный `HonorElimilateConfig` initial-config packet.
pub(crate) fn continue_game_server_honor_eliminate_configuration(
    game: &CGame,
    socket_id: i32,
    honor_eliminate: HonorElimilateConfig,
) -> WorldHonorEliminateConfigurationReport {
    let mut payload = Vec::with_capacity(8);
    honor_eliminate.add_to_byte_array(&mut payload);
    let sender = game.current_game_server_sender();
    WorldHonorEliminateConfigurationReport {
        delivery: send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x26,
            &payload,
        ),
        completion: WorldHonorEliminateConfigurationCompletion::HonorRanksPending { socket_id },
    }
}

/// Отправляет четыре точных history-среза `CHonorRanks`.
pub(crate) fn continue_game_server_honor_ranks_configuration(
    game: &CGame,
    socket_id: i32,
    honor_ranks: &CHonorRanks,
) -> WorldHonorRanksConfigurationReport {
    const PASSES: [(HonorRanksType, i32, bool); 4] = [
        (HonorRanksType::Day, 0x27, false),
        (HonorRanksType::Week, 0x28, false),
        (HonorRanksType::Month, 0x29, false),
        (HonorRanksType::Total, 0x2A, true),
    ];

    let sender = game.current_game_server_sender();
    let mut deliveries = Vec::with_capacity(PASSES.len());
    for (rank_type, subtype, has_total_prefix) in PASSES {
        let mut payload = Vec::new();
        if let Err(error) =
            honor_ranks.add_history_to_byte_array(&mut payload, rank_type, None)
        {
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
                delivery: message.send_to_socket(sender.as_ref(), socket_id),
            }
        } else {
            send_initial_configuration_to_socket(
                sender.as_ref(),
                socket_id,
                subtype,
                &payload,
            )
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

/// Отправляет nullable function/variable file-data без C-string преобразования.
pub(crate) fn continue_game_server_raw_script_lists_configuration(
    game: &CGame,
    socket_id: i32,
) -> WorldRawScriptListsConfigurationReport {
    let sender = game.current_game_server_sender();
    let mut deliveries = Vec::with_capacity(2);
    for (kind, subtype, data) in [
        (
            WorldRawScriptListKind::Function,
            0x0A,
            game.function_list_file_data(),
        ),
        (
            WorldRawScriptListKind::Variable,
            0x0B,
            game.variable_list_file_data(),
        ),
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
                delivery: message.send_to_socket(sender.as_ref(), socket_id),
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

/// Отправляет nullable общий `CVariableList` subtype `0x0C`.
pub(crate) fn continue_game_server_general_variable_configuration(
    game: &CGame,
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

    let sender = game.current_game_server_sender();
    WorldGeneralVariableConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x0C,
            &payload,
        )),
        completion: WorldGeneralVariableConfigurationCompletion::ScriptFilesPending { socket_id },
    }
}

/// Отправляет ordered `m_mapScript_FileData` как отдельные subtype `0x0D`.
pub(crate) fn continue_game_server_script_files_configuration(
    game: &CGame,
    socket_id: i32,
) -> WorldScriptFilesConfigurationReport {
    let sender = game.current_game_server_sender();
    let mut deliveries = Vec::new();
    for (path, data) in game.initial_script_files() {
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
                delivery: message.send_to_socket(sender.as_ref(), socket_id),
            },
        });
    }

    WorldScriptFilesConfigurationReport {
        deliveries,
        completion: WorldScriptFilesConfigurationCompletion::QuestSystemPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CQuestSystem` subtype `0x16`.
pub(crate) fn continue_game_server_quest_configuration(
    game: &CGame,
    socket_id: i32,
    quests: &CQuestSystem,
) -> WorldQuestConfigurationReport {
    let mut payload = Vec::new();
    if let Err(error) = quests.add_to_byte_array(&mut payload) {
        return WorldQuestConfigurationReport {
            delivery: None,
            completion: WorldQuestConfigurationCompletion::QuestSystem(error),
        };
    }
    let sender = game.current_game_server_sender();
    WorldQuestConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x16,
            &payload,
        )),
        completion: WorldQuestConfigurationCompletion::PlayerRanksPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CPlayerRanks` subtype `0x17`.
pub(crate) fn continue_game_server_player_ranks_configuration(
    game: &CGame,
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
    let sender = game.current_game_server_sender();
    WorldPlayerRanksConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x17,
            &payload,
        )),
        completion: WorldPlayerRanksConfigurationCompletion::GmListPending { socket_id },
    }
}

/// Кодирует и отправляет точный `CGMList` subtype `9`.
pub(crate) fn continue_game_server_gm_list_configuration(
    game: &CGame,
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
    let sender = game.current_game_server_sender();
    WorldGmListConfigurationReport {
        delivery: Some(send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            9,
            &payload,
        )),
        completion: WorldGmListConfigurationCompletion::GameServerIndexPending {
            socket_id,
            game_server_index,
        },
    }
}

/// Отправляет subtype `0x12` с точным однобайтовым cast-ом `dwIndex`.
pub(crate) fn continue_game_server_index_configuration(
    game: &CGame,
    socket_id: i32,
    game_server_index: u32,
) -> WorldGameServerIndexConfigurationReport {
    let sender = game.current_game_server_sender();
    WorldGameServerIndexConfigurationReport {
        delivery: send_initial_configuration_to_socket(
            sender.as_ref(),
            socket_id,
            0x12,
            &[game_server_index as u8],
        ),
        completion: WorldGameServerIndexConfigurationCompletion::FourNationWarPending {
            socket_id,
        },
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

/// Продолжает точный reconnect player-data хвост после общего prefix-а.
pub(crate) fn continue_game_server_reconnect(
    game: &mut CGame,
    socket_id: i32,
    payload: &[u8],
    registry: &GoodsBasePropertiesRegistry,
    organizing: &mut COrganizingCtrl,
    coefficients: &PlayerPropertyCoefficients,
) -> WorldGameServerReconnectReport {
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
