//! Владелец входного GameServer dispatcher-а `OnServerMessage`.
//!
//! Весь dispatcher RVA `0x0009D300` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме цепочек
//! сообщения `0x7F801` для достигнутых typed startup snapshots,
//! AttackCity/Village и terminal selector `0x3B`, а также полной typed Billing
//! reconnect ветви `0x6F904`; они имеют статус `IMPLEMENTED`. Точная пара
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
//! HonorEliminate `0x26` отдельно сохраняет оба подтверждённых sink-а:
//! `AddLogText` и `PutStringToFile("HonorCompositior", ...)`; payload проверен
//! по exact EXE и runtime-логам.
//! GodsBattle `0x39` перед финальным startup log сохраняет decoder-local GBK
//! warning и `PutStringToFile("godsbattleLog", ...)` в их исходных позициях.
//! GlobeSetup `0x07` сохраняет вложенный router decode, DaKong key, byte
//! broadcast `0xBF736`, conditional auction disable и только затем глобальные
//! area dimensions с финальным startup log.
//! Вместе с LogSystem `0x08` и GM-list `0x09` он входит в общий живой
//! runtime-configuration pass, сохраняя state/network/log ordering всей группы.
//! QuestSystem `0x16` сохраняет exact scalar/script/map mutation order и
//! публикует runtime lookup-owner до финального startup log.
//! Вместе с Game ID `0x12`, hit-level `0x14` и emotion `0x15` он входит в
//! живую player-rule resource family; player ranks `0x17` остаётся отдельно
//! из-за честной allocation-error границы owner-а.
//! CountryParam `0x18` сохраняет scalar prefix, пять ordered maps и известные
//! technology wire-quirks до точного startup log.
//! CountryHandler `0x19` заменяет прежние country owners, декодирует byte-count
//! minister records и публикует ordered lookup до финального startup log.
//! Оба country state selector-а `0x18/0x19` входят в один FIFO pass, чтобы
//! параметры и canonical country owners сохраняли согласованный startup цикл.
//! Пространственные owners `0x0F/0x10/0x11/0x1A` также проходят один FIFO
//! pass: proxy публикуется только после полного decode, reload сохраняет
//! ранний region miss, а region-level и duplicate registries — исходные
//! replacement/partial-decode и success-log границы.
//! CEmotion `0x15` накладывает signed ID/value records без очистки общего map и
//! публикует runtime repeated-emotion lookup до финального startup log.
//! Goods list `0x00` заменяет ID/original-name/name registry из парного
//! WorldServer wire и только после полного decode пишет точный startup log.
//! Monster list `0x02` аналогично заменяет monster/drop registries, пишет log
//! и лишь затем запускает refresh уже живых monster base-property ссылок.
//! Skill list `0x06` очищает и заново публикует composite-key registry из
//! парного WorldServer wire, затем пишет точный startup log.
//! Goods/monster/skill registry family `0x00/0x02/0x06` входит в живой FIFO
//! одним selector-pass; monster success после log обновляет lookup-связность
//! уже опубликованных region monster owners, как исходный dispatcher.
//! Player/economy catalogs `0x01/0x03/0x04/0x05` тем же путём публикуют
//! player templates, trade, increment-shop и contribution owners; каждый
//! сохраняет собственный clear/partial-decode и exact success-log контракт.
//! Proxy region `0x0F` создаёт отдельный owner, полностью декодирует короткий
//! proxy wire и map-assignment-ом публикует его до точного startup log.
//! Region selector `0x0E` маршрутизирует все шесть concrete subtype-ов через
//! общий base decoder, публикует ordered `CGame` owner и лишь затем обновляет
//! startup totals и GodsBattle region-set.
//! Runtime selector `0x10` ищет concrete owner по ID и перечитывает только его
//! setup/forbid-goods tail; miss не читает tail и не пишет log.
//! FourNationWar `0x25` декодирует exact 196-byte setup records и пять rects,
//! затем проецирует war state и relive rectangles в доступные nation regions.
//! Script resources `0x0A..0x0D` сохраняют signed lengths, bounded path,
//! function/general parser callbacks и разные duplicate-owner контракты.
//! Game ID selector `0x12` сохраняет raw byte для старшего байта team ID.
//! Language table selector `0x2F` и runtime refresh `0x7F807` используют один
//! clear/decode/log/cursor контракт `CGame::CreateStringTable`.
//! Battle-fairy resource family `0x20/0x2C/0x2D/0x30` входит в реальный
//! server FIFO через общий selector owner: обе exp-таблицы, combine recipes и
//! equipment-compose maps публикуются с исходными partial decode и log order.
//! Honor ranks `0x27..0x2A` декодируют четыре country-list, а total branch
//! перед финальным log передаёт точную player-reset семантику context-owner-у.
//!
//! Terminal selector сначала вызывает `InitNetServer`, затем читает login и
//! world ID и присваивает их даже после ошибки Host. Rust сохраняет этот
//! partial-effect порядок: malformed хвост возвращается отдельно, не откатывая
//! уже выполненный network init и не подставляя нулевые identity. Dialog/log
//! вызовы возвращаются ordered typed effects на внешней runtime-границе.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityRegionContext, CAttackCitySys,
};
use super::super::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationGameDecodeError, FourNationGameStartupContext, FourNationRect,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarRegionContext,
};
use crate::gameserver::appserver::country::countryhandler::{
    CountryHandlerDecodeError, CountryHandlerDecodeReport,
};
use crate::gameserver::appserver::country::countryparam::{
    CountryParamDecodeReport, CountryParamInputBlock,
};
use crate::gameserver::appserver::country::countrywarsys::{
    CountryWarDecodeError, CountryWarStartupContext, CountryWarSys,
};
use crate::gameserver::appserver::goods::cbattlefairyproperty::BattleFairyComposeDecodeError;
use crate::gameserver::appserver::goods::cgoodsfactory::{
    GoodsFactoryDecodeError, GoodsFactoryDecodeReport,
};
use crate::gameserver::appserver::proxyserverregion::{CProxyServerRegion, ProxyRegionDecodeError};
use crate::gameserver::appserver::servercityregion::{
    CServerCityRegion, CityRegionDecodeContext, CityRegionDecodeError,
};
use crate::gameserver::appserver::servercountryregion::{
    CServerCountryRegion, CountryRegionDecodeContext, CountryRegionDecodeError,
};
use crate::gameserver::appserver::servergodsbattleregion::CServerGodsBattleRegion;
use crate::gameserver::appserver::servernationregion::ServerNationRegion;
use crate::gameserver::appserver::serverregion::ServerRegionSetupDecodeError;
use crate::gameserver::appserver::serverregion::{CServerRegion, ServerRegionDecodeError};
use crate::gameserver::appserver::servervillageregion::CServerVillageRegion;
use crate::gameserver::appserver::serverwarregion::WarRegionDecodeError;
use crate::gameserver::appserver::skills::skillfactory::{
    SkillFactoryDecodeError, SkillFactoryDecodeReport,
};
use crate::gameserver::gameserver::game::{
    CGame, GameNetworkInitializationError, GameScriptResourceContext, GameSingleFilePublication,
    MonsterBasePropertyRefreshReport, ServerRegionOwner,
};
use crate::gameserver::gameserver::honorranks::{HonorRanksDecodeError, HonorRanksDecodeReport};
use crate::gameserver::gameserver::playerranks::PlayerRanksDecodeError;
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::nets::netserver::mynetclient::CMyNetClient;
use crate::public::ciqing::{CiQingDecodeError, CiQingDecodeReport, CiQingSerializationBlock};
use crate::public::dakongxiangqian::{DaKongDecodeError, DaKongDecodeReport};
use crate::public::dupliregionsetup::DupliRegionDecodeError;
use crate::public::equipmentcomposelist::{
    EquipmentComposeDecodeError, EquipmentComposeDecodeReport,
};
use crate::public::mystringtable::{MyStringTableDecodeError, MyStringTableDecodeReport};
use crate::public::taozhuangsetup::{TaoZhuangDecodeError, TaoZhuangSerializationBlock};
use crate::public::wordsfilter::{WordsFilterDecodeError, WordsFilterDecodeReport};
use crate::setup::cbattlefairyexpconfig::{BattleFairyExpDecodeError, BattleFairyExpDecodeReport};
use crate::setup::changebody::ChangeBodyDecodeError;
use crate::setup::contributesetup::ContributeSetupDecodeError;
use crate::setup::emotion::{EmotionDecodeError, EmotionDecodeReport};
use crate::setup::globesetup::{GlobeSetupDecodeError, GlobeSetupDecodeReport};
use crate::setup::gmlist::{GmListDecodeError, GmListDecodeReport};
use crate::setup::godsbattleconf::{GodsBattleDecodeError, GodsBattleDecodeReport};
use crate::setup::goodsdestructionconfig::{GoodsDestroyDecodeError, GoodsDestroyDecodeReport};
use crate::setup::hitlevelsetup::HitLevelDecodeError;
use crate::setup::honorelimilateconfig::HonorEliminateDecodeError;
use crate::setup::incrementshoplist::IncrementShopDecodeError;
use crate::setup::leitingsetup::ThingSetupCodecError;
use crate::setup::lingbao::{LingBaoDecodeError, LingBaoDecodeReport};
use crate::setup::logsystem::LogSystemDecodeError;
use crate::setup::monsterlist::{MonsterListDecodeError, MonsterListDecodeReport};
use crate::setup::newskillmonsterlist::{NewSkillMonsterDecodeError, NewSkillMonsterDecodeReport};
use crate::setup::playerlist::{PlayerListDecodeError, PlayerListDecodeReport};
use crate::setup::preciousboxconf::PreciousBoxDecodeError;
use crate::setup::prisonconf::PrisonConfDecodeError;
use crate::setup::questsystem::{QuestSystemDecodeError, QuestSystemDecodeReport};
use crate::setup::regionsetup::RegionSetupDecodeError;
use crate::setup::synthesis::{SynthesisDecodeError, SynthesisDecodeReport};
use crate::setup::tradelist::TradeListDecodeError;

const BILLING_REGISTRATION: i32 = 0x000E_F101;
const SERVER_STARTUP_MESSAGE: i32 = 0x0007_F801;
const PLAYER_COUNT_IF_WORLD_CONNECTED_MESSAGE: i32 = 0x0007_F809;
const PLAYER_COUNT_MESSAGE: i32 = 0x0007_F80B;
const PLAYER_COUNT_IF_WORLD_CONNECTED_RESPONSE: i32 = 0x0005_FA0A;
const PLAYER_COUNT_RESPONSE: i32 = 0x0005_FA0C;
const CLIENT_SERVER_START_SELECTOR: i32 = 0x3b;
const GOODS_LIST_SELECTOR: i32 = 0x00;
const PLAYER_LIST_SELECTOR: i32 = 0x01;
const MONSTER_LIST_SELECTOR: i32 = 0x02;
const TRADE_LIST_SELECTOR: i32 = 0x03;
const INCREMENT_SHOP_SELECTOR: i32 = 0x04;
const CONTRIBUTE_SETUP_SELECTOR: i32 = 0x05;
const SKILL_LIST_SELECTOR: i32 = 0x06;
const GLOBE_SETUP_SELECTOR: i32 = 0x07;
const LOG_SYSTEM_SELECTOR: i32 = 0x08;
const GM_LIST_SELECTOR: i32 = 0x09;
const FUNCTION_LIST_SELECTOR: i32 = 0x0a;
const VARIABLE_LIST_SELECTOR: i32 = 0x0b;
const GENERAL_VARIABLE_SELECTOR: i32 = 0x0c;
const SCRIPT_FILE_SELECTOR: i32 = 0x0d;
const REGION_SELECTOR: i32 = 0x0e;
const PROXY_REGION_SELECTOR: i32 = 0x0f;
const REGION_RELOAD_SELECTOR: i32 = 0x10;
const REGION_SETUP_SELECTOR: i32 = 0x11;
const ID_INDEX_SELECTOR: i32 = 0x12;
const HIT_LEVEL_SELECTOR: i32 = 0x14;
const EMOTION_SELECTOR: i32 = 0x15;
const QUEST_SYSTEM_SELECTOR: i32 = 0x16;
const PLAYER_RANKS_SELECTOR: i32 = 0x17;
const COUNTRY_PARAM_SELECTOR: i32 = 0x18;
const COUNTRY_HANDLER_SELECTOR: i32 = 0x19;
const DUPLI_REGION_SELECTOR: i32 = 0x1a;
const PRISON_CONF_SELECTOR: i32 = 0x1d;
const PRECIOUS_BOX_CONF_SELECTOR: i32 = 0x1e;
const FAIRY_EXP_SELECTOR: i32 = 0x20;
const SYNTHESIS_SELECTOR: i32 = 0x21;
const NEW_SKILL_MONSTER_SELECTOR: i32 = 0x22;
const GOODS_DESTROY_SELECTOR: i32 = 0x23;
const CHANGE_BODY_SELECTOR: i32 = 0x24;
const HONOR_ELIMINATE_SELECTOR: i32 = 0x26;
const DAYS_HONOR_RANK_SELECTOR: i32 = 0x27;
const WEEKS_HONOR_RANK_SELECTOR: i32 = 0x28;
const MONTHS_HONOR_RANK_SELECTOR: i32 = 0x29;
const TOTAL_HONOR_RANK_SELECTOR: i32 = 0x2a;
const DA_KONG_SELECTOR: i32 = 0x2b;
const BATTLE_FAIRY_EXP_SELECTOR: i32 = 0x2c;
const BATTLE_FAIRY_COMBINE_SELECTOR: i32 = 0x2d;
const STRING_TABLE_SELECTOR: i32 = 0x2f;
const EQUIPMENT_COMPOSE_SELECTOR: i32 = 0x30;
const WORDS_FILTER_SELECTOR: i32 = 0x31;
const JJC_REGION_LEVEL_SELECTOR: i32 = 0x32;
const TAO_ZHUANG_SELECTOR: i32 = 0x34;
const CI_QING_LING_BAO_SELECTOR: i32 = 0x35;
const THING_SETUP_SELECTOR: i32 = 0x36;
const GODS_BATTLE_SELECTOR: i32 = 0x39;
const STRING_TABLE_REFRESH_MESSAGE: i32 = 0x0007_f807;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameClientServerStartNetwork {
    Started,
    MissingNetworkSetup,
    HostFailed { detail: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameClientServerStartEffect {
    FailureDialog {
        message: &'static [u8],
        title: &'static [u8],
    },
    Log(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameClientServerStartReport {
    pub(crate) network: GameClientServerStartNetwork,
    pub(crate) effects: Vec<GameClientServerStartEffect>,
    pub(crate) server_ids: Result<GameServerIds, GameClientServerStartPayloadError>,
    pub(crate) applied_login_id: Option<i32>,
    pub(crate) applied_world_id: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameStringTableSource {
    Startup,
    RuntimeRefresh,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameStringTableMessageReport {
    pub(crate) source: GameStringTableSource,
    pub(crate) decoded: MyStringTableDecodeReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerCountResponseKind {
    WorldConnected,
    Unconditional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerCountResponseOutcome {
    MissingWorldClient,
    Sent(Result<i32, SendMessageError>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerCountResponseReport {
    pub(crate) kind: GamePlayerCountResponseKind,
    pub(crate) player_count: Option<u32>,
    pub(crate) outcome: GamePlayerCountResponseOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameBattleFairyStartupReport {
    FairyExp(BattleFairyExpDecodeReport),
    BattleFairyExp(BattleFairyExpDecodeReport),
    Combine { entries: usize },
    EquipmentCompose(EquipmentComposeDecodeReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameBattleFairyStartupError {
    FairyExp(BattleFairyExpDecodeError),
    BattleFairyExp(BattleFairyExpDecodeError),
    Combine(BattleFairyComposeDecodeError),
    EquipmentCompose(EquipmentComposeDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameBattleFairyStartupMessageReport {
    pub(crate) decoded: GameBattleFairyStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCombatRegistryStartupReport {
    Goods(GoodsFactoryDecodeReport),
    Monsters(MonsterListStartupReport),
    Skills(SkillFactoryDecodeReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCombatRegistryStartupError {
    Goods(GoodsFactoryDecodeError),
    Monsters(MonsterListDecodeError),
    Skills(SkillFactoryDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameCombatRegistryStartupMessageReport {
    pub(crate) decoded: GameCombatRegistryStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerEconomyStartupReport {
    PlayerTemplates(PlayerListDecodeReport),
    TradeList { entries: usize },
    IncrementShop { entries: usize },
    ContributionItems { entries: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerEconomyStartupError {
    PlayerTemplates(PlayerListDecodeError),
    TradeList(TradeListDecodeError),
    IncrementShop(IncrementShopDecodeError),
    ContributionItems(ContributeSetupDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerEconomyStartupMessageReport {
    pub(crate) decoded: GamePlayerEconomyStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameRuntimeConfigurationStartupReport {
    GlobeSetup {
        decoded: GlobeSetupDecodeReport,
        goods_ai_broadcast: Result<i32, SendMessageError>,
        auction_forced_disabled: bool,
    },
    LogSystem {
        entries: usize,
        da_kong_log: bool,
    },
    GmList(GmListDecodeReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRuntimeConfigurationStartupError {
    GlobeSetup(GlobeSetupDecodeError),
    LogSystem(LogSystemDecodeError),
    GmList(GmListDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRuntimeConfigurationStartupMessageReport {
    pub(crate) decoded: GameRuntimeConfigurationStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerRuleStartupReport {
    IdIndex { value: u8 },
    HitLevel { entries: usize },
    Emotion(EmotionDecodeReport),
    QuestSystem(QuestSystemDecodeReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerRuleStartupError {
    IdIndex(GameIdIndexDecodeError),
    HitLevel(HitLevelDecodeError),
    Emotion(EmotionDecodeError),
    QuestSystem(QuestSystemDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerRuleStartupMessageReport {
    pub(crate) decoded: GamePlayerRuleStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCountryStateStartupReport {
    Parameters(CountryParamDecodeReport),
    Countries(CountryHandlerDecodeReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCountryStateStartupError {
    Parameters(CountryParamInputBlock),
    Countries(CountryHandlerDecodeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameCountryStateStartupMessageReport {
    pub(crate) decoded: GameCountryStateStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameSpatialStartupReport {
    ProxyRegion { region_id: i32, replaced: bool },
    RegionReload(RegionSetupReloadReport),
    RegionSetup { entries: usize },
    DupliRegions { entries: usize },
}

#[derive(Clone, Debug)]
pub(crate) enum GameSpatialStartupError {
    ProxyRegion(ProxyRegionDecodeError),
    RegionReload(RegionSetupReloadError),
    RegionSetup(RegionSetupDecodeError),
    OwnerUnavailable { selector: i32 },
    DupliRegions(Arc<DupliRegionDecodeError>),
}

impl PartialEq for GameSpatialStartupError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ProxyRegion(left), Self::ProxyRegion(right)) => left == right,
            (Self::RegionReload(left), Self::RegionReload(right)) => left == right,
            (Self::RegionSetup(left), Self::RegionSetup(right)) => left == right,
            (
                Self::OwnerUnavailable { selector: left },
                Self::OwnerUnavailable { selector: right },
            ) => left == right,
            (Self::DupliRegions(left), Self::DupliRegions(right)) => {
                dupli_region_decode_errors_equal(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for GameSpatialStartupError {}

fn dupli_region_decode_errors_equal(
    left: &DupliRegionDecodeError,
    right: &DupliRegionDecodeError,
) -> bool {
    match (left, right) {
        (
            DupliRegionDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            DupliRegionDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (DupliRegionDecodeError::Allocation(_), DupliRegionDecodeError::Allocation(_)) => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameSpatialStartupMessageReport {
    pub(crate) decoded: GameSpatialStartupReport,
    pub(crate) log_effects: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameServerMessageReport {
    ClientServerStart(GameClientServerStartReport),
    StringTable(GameStringTableMessageReport),
    PlayerCount(GamePlayerCountResponseReport),
    BattleFairyStartup(GameBattleFairyStartupMessageReport),
    CombatRegistryStartup(GameCombatRegistryStartupMessageReport),
    PlayerEconomyStartup(GamePlayerEconomyStartupMessageReport),
    RuntimeConfigurationStartup(GameRuntimeConfigurationStartupMessageReport),
    PlayerRuleStartup(GamePlayerRuleStartupMessageReport),
    CountryStateStartup(GameCountryStateStartupMessageReport),
    SpatialStartup(GameSpatialStartupMessageReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameServerMessageError {
    StartupSelector(GameClientServerStartPayloadError),
    StringTable(MyStringTableDecodeError),
    BattleFairyStartup(GameBattleFairyStartupError),
    CombatRegistryStartup(GameCombatRegistryStartupError),
    PlayerEconomyStartup(GamePlayerEconomyStartupError),
    RuntimeConfigurationStartup(GameRuntimeConfigurationStartupError),
    PlayerRuleStartup(GamePlayerRuleStartupError),
    CountryStateStartup(GameCountryStateStartupError),
    SpatialStartup(GameSpatialStartupError),
}

/// Маршрутизирует уже материализованные ветви `OnServerMessage`: общий
/// startup selector читается один раз, неизвестные selector-ы не двигают
/// cursor, а обе language-table точки входят в один `CGame` owner.
pub(crate) fn dispatch_server_message(
    message: &mut CMessage,
    game: &mut CGame,
    mut now_ms: impl FnMut() -> u32,
) -> Option<Result<GameServerMessageReport, GameServerMessageError>> {
    if message.message_type() == STRING_TABLE_REFRESH_MESSAGE {
        return Some(dispatch_string_table_message(
            message,
            game,
            GameStringTableSource::RuntimeRefresh,
        ));
    }
    if let Some(report) = dispatch_player_count_message(message.message_type(), game) {
        return Some(Ok(GameServerMessageReport::PlayerCount(report)));
    }
    if message.message_type() != SERVER_STARTUP_MESSAGE {
        return None;
    }
    let selector = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        let mut probe = *cursor;
        match read_start_long(wire, &mut probe) {
            Ok(selector) => selector,
            Err(selector) => {
                return Some(Err(GameServerMessageError::StartupSelector(selector)));
            }
        }
    };
    match selector {
        CLIENT_SERVER_START_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("terminal selector проверен без изменения cursor");
            Some(Ok(GameServerMessageReport::ClientServerStart(
                dispatch_client_server_start(consumed_selector, message, game, now_ms())
                    .expect("selector 0x3B проверен outer dispatcher-ом"),
            )))
        }
        STRING_TABLE_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("language-table selector проверен без изменения cursor");
            debug_assert_eq!(consumed_selector, STRING_TABLE_SELECTOR);
            Some(dispatch_string_table_message(
                message,
                game,
                GameStringTableSource::Startup,
            ))
        }
        FAIRY_EXP_SELECTOR
        | BATTLE_FAIRY_EXP_SELECTOR
        | BATTLE_FAIRY_COMBINE_SELECTOR
        | EQUIPMENT_COMPOSE_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("battle-fairy selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_battle_fairy_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("battle-fairy selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::BattleFairyStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::BattleFairyStartup(
                GameBattleFairyStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        GOODS_LIST_SELECTOR | MONSTER_LIST_SELECTOR | SKILL_LIST_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("combat-registry selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_combat_registry_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("combat-registry selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::CombatRegistryStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::CombatRegistryStartup(
                GameCombatRegistryStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        PLAYER_LIST_SELECTOR
        | TRADE_LIST_SELECTOR
        | INCREMENT_SHOP_SELECTOR
        | CONTRIBUTE_SETUP_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("player/economy selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_player_economy_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("player/economy selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::PlayerEconomyStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::PlayerEconomyStartup(
                GamePlayerEconomyStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        GLOBE_SETUP_SELECTOR | LOG_SYSTEM_SELECTOR | GM_LIST_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("runtime-configuration selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_runtime_configuration_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| log_effects.push(text.to_vec()),
                )
                .expect("runtime-configuration selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::RuntimeConfigurationStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::RuntimeConfigurationStartup(
                GameRuntimeConfigurationStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        ID_INDEX_SELECTOR | HIT_LEVEL_SELECTOR | EMOTION_SELECTOR | QUEST_SYSTEM_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("player-rule selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_player_rule_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("player-rule selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::PlayerRuleStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::PlayerRuleStartup(
                GamePlayerRuleStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        COUNTRY_PARAM_SELECTOR | COUNTRY_HANDLER_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("country-state selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_country_state_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("country-state selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::CountryStateStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::CountryStateStartup(
                GameCountryStateStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        PROXY_REGION_SELECTOR
        | REGION_RELOAD_SELECTOR
        | REGION_SETUP_SELECTOR
        | DUPLI_REGION_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("spatial selector проверен без изменения cursor");
            let mut log_effects = Vec::new();
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_spatial_startup(consumed_selector, wire, cursor, game, |text| {
                    log_effects.push(text.to_vec())
                })
                .expect("spatial selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::SpatialStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            Some(Ok(GameServerMessageReport::SpatialStartup(
                GameSpatialStartupMessageReport {
                    decoded,
                    log_effects,
                },
            )))
        }
        _ => None,
    }
}

fn decode_spatial_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GameSpatialStartupReport, GameSpatialStartupError>> {
    match selector {
        PROXY_REGION_SELECTOR => {
            let mut region = CProxyServerRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true) {
                return Some(Err(GameSpatialStartupError::ProxyRegion(error)));
            }
            let region_id = region.get_id();
            let replaced = game.add_proxy_region(region);
            add_log_text(b"Add Proxy Region : (%d) %s ...OK!");
            Some(Ok(GameSpatialStartupReport::ProxyRegion {
                region_id,
                replaced,
            }))
        }
        REGION_RELOAD_SELECTOR => {
            let region_id = match read_initial_region_i32(source, cursor) {
                Ok(region_id) => region_id,
                Err(error) => {
                    return Some(Err(GameSpatialStartupError::RegionReload(
                        RegionSetupReloadError::RegionId(error),
                    )));
                }
            };
            let Some(region) = game.find_region_mut(region_id) else {
                return Some(Ok(GameSpatialStartupReport::RegionReload(
                    RegionSetupReloadReport {
                        region_id,
                        found: false,
                    },
                )));
            };
            if let Err(error) = region
                .base_mut()
                .decord_setup_from_byte_array(source, cursor, true)
            {
                return Some(Err(GameSpatialStartupError::RegionReload(
                    RegionSetupReloadError::Setup(error),
                )));
            }
            add_log_text(b"Reload Region : (%d)%s Setup...OK!");
            Some(Ok(GameSpatialStartupReport::RegionReload(
                RegionSetupReloadReport {
                    region_id,
                    found: true,
                },
            )))
        }
        REGION_SETUP_SELECTOR => {
            let entries = match game
                .region_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameSpatialStartupError::RegionSetup(error))),
            };
            add_log_text(b"Initial SI_REGIONLEVELSETUP...OK!");
            Some(Ok(GameSpatialStartupReport::RegionSetup { entries }))
        }
        DUPLI_REGION_SELECTOR => {
            let Some(setup) = game.dupli_region_setup_mut() else {
                return Some(Err(GameSpatialStartupError::OwnerUnavailable { selector }));
            };
            if let Err(error) = setup.decord_from_byte_array(source, cursor) {
                return Some(Err(GameSpatialStartupError::DupliRegions(Arc::new(error))));
            }
            let entries = setup.entries().len();
            add_log_text(b"Initial SI_DUPLIREGIONSETUP...OK!");
            Some(Ok(GameSpatialStartupReport::DupliRegions { entries }))
        }
        _ => None,
    }
}

fn decode_country_state_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GameCountryStateStartupReport, GameCountryStateStartupError>> {
    let result = match selector {
        COUNTRY_PARAM_SELECTOR => game
            .country_param_mut()
            .decord_from_byte_array(source, cursor)
            .map(GameCountryStateStartupReport::Parameters)
            .map_err(GameCountryStateStartupError::Parameters),
        COUNTRY_HANDLER_SELECTOR => game
            .country_handler_mut()
            .decord_from_byte_array(source, cursor)
            .map(GameCountryStateStartupReport::Countries)
            .map_err(GameCountryStateStartupError::Countries),
        _ => return None,
    };
    if result.is_ok() {
        add_log_text(match selector {
            COUNTRY_PARAM_SELECTOR => b"Initial SI_COUNTRYPARAM...OK!".as_slice(),
            COUNTRY_HANDLER_SELECTOR => b"Initial SI_COUNTRY...OK!".as_slice(),
            _ => unreachable!("selector отфильтрован перед country log lookup"),
        });
    }
    Some(result)
}

fn decode_player_rule_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GamePlayerRuleStartupReport, GamePlayerRuleStartupError>> {
    match selector {
        ID_INDEX_SELECTOR => {
            let offset = *cursor;
            let Some(&value) = source.get(offset) else {
                return Some(Err(GamePlayerRuleStartupError::IdIndex(
                    GameIdIndexDecodeError {
                        offset,
                        available: source.len().saturating_sub(offset),
                    },
                )));
            };
            *cursor += 1;
            game.set_id_index(value);
            Some(Ok(GamePlayerRuleStartupReport::IdIndex { value }))
        }
        HIT_LEVEL_SELECTOR => {
            let entries = game
                .hit_level_setup_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GamePlayerRuleStartupError::HitLevel);
            if entries.is_ok() {
                add_log_text(b"Initial SI_HITLEVEL...OK!");
            }
            Some(entries.map(|entries| GamePlayerRuleStartupReport::HitLevel { entries }))
        }
        EMOTION_SELECTOR => {
            let report = game
                .emotion_mut()
                .unserialize(source, cursor)
                .map_err(GamePlayerRuleStartupError::Emotion);
            if report.is_ok() {
                add_log_text(b"Initial SI_EMOTION...OK!");
            }
            Some(report.map(GamePlayerRuleStartupReport::Emotion))
        }
        QUEST_SYSTEM_SELECTOR => {
            let report = game
                .quest_system_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GamePlayerRuleStartupError::QuestSystem);
            if report.is_ok() {
                add_log_text(b"Initial SI_QUEST...OK!");
            }
            Some(report.map(GamePlayerRuleStartupReport::QuestSystem))
        }
        _ => None,
    }
}

fn decode_runtime_configuration_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GameRuntimeConfigurationStartupReport, GameRuntimeConfigurationStartupError>> {
    match selector {
        GLOBE_SETUP_SELECTOR => {
            let decoded = {
                let (globe_setup, region_router) = game.globe_setup_and_region_router_mut();
                match globe_setup.decord_from_byte_array(region_router, source, cursor) {
                    Ok(decoded) => decoded,
                    Err(error) => {
                        return Some(Err(GameRuntimeConfigurationStartupError::GlobeSetup(error)));
                    }
                }
            };
            game.da_kong_xiang_qian_mut().set_key(decoded.da_kong_key);
            let mut notice = CMessage::new(0x000B_F736);
            notice.add_byte(u8::from(decoded.goods_ai_enabled));
            let goods_ai_broadcast = notice.send_all(game.current_net_server());
            let auction_forced_disabled = !decoded.auction_enabled;
            if auction_forced_disabled {
                game.force_auction_disabled();
            }
            game.set_area_dimensions(decoded.area_width, decoded.area_height);
            add_log_text(b"Initial SI_GLOBESETUP...OK!");
            Some(Ok(GameRuntimeConfigurationStartupReport::GlobeSetup {
                decoded,
                goods_ai_broadcast,
                auction_forced_disabled,
            }))
        }
        LOG_SYSTEM_SELECTOR => {
            let report = match game.log_system_mut().decord_from_byte_array(source, cursor) {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameRuntimeConfigurationStartupError::LogSystem(error)));
                }
            };
            game.da_kong_xiang_qian_mut().set_key(report.da_kong_log);
            add_log_text(b"Initial SI_LOGSYSTEM...OK!");
            Some(Ok(GameRuntimeConfigurationStartupReport::LogSystem {
                entries: report.items,
                da_kong_log: report.da_kong_log,
            }))
        }
        GM_LIST_SELECTOR => {
            let report = game
                .gm_list_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameRuntimeConfigurationStartupError::GmList);
            if report.is_ok() {
                add_log_text(b"Initial SI_GMLIST...OK!");
            }
            Some(report.map(GameRuntimeConfigurationStartupReport::GmList))
        }
        _ => None,
    }
}

fn decode_player_economy_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GamePlayerEconomyStartupReport, GamePlayerEconomyStartupError>> {
    let result = match selector {
        PLAYER_LIST_SELECTOR => game
            .player_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(GamePlayerEconomyStartupReport::PlayerTemplates)
            .map_err(GamePlayerEconomyStartupError::PlayerTemplates),
        TRADE_LIST_SELECTOR => game
            .trade_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| GamePlayerEconomyStartupReport::TradeList { entries })
            .map_err(GamePlayerEconomyStartupError::TradeList),
        INCREMENT_SHOP_SELECTOR => game
            .increment_shop_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| GamePlayerEconomyStartupReport::IncrementShop { entries })
            .map_err(GamePlayerEconomyStartupError::IncrementShop),
        CONTRIBUTE_SETUP_SELECTOR => game
            .contribute_setup_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| GamePlayerEconomyStartupReport::ContributionItems { entries })
            .map_err(GamePlayerEconomyStartupError::ContributionItems),
        _ => return None,
    };
    if result.is_ok() {
        let log = match selector {
            PLAYER_LIST_SELECTOR => b"Initial SI_PLAYERLIST...OK!".as_slice(),
            TRADE_LIST_SELECTOR => b"Initial SI_TRADELIST...OK!".as_slice(),
            INCREMENT_SHOP_SELECTOR => b"Initial SI_INCREMENTSHOPLIST...OK!".as_slice(),
            CONTRIBUTE_SETUP_SELECTOR => b"Initial SI_CONTRIBUTEITEM...OK!".as_slice(),
            _ => unreachable!("selector отфильтрован перед log lookup"),
        };
        add_log_text(log);
    }
    Some(result)
}

fn decode_combat_registry_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GameCombatRegistryStartupReport, GameCombatRegistryStartupError>> {
    match selector {
        GOODS_LIST_SELECTOR => {
            let report = game
                .goods_factory_mut()
                .unserialize(source, cursor)
                .map_err(GameCombatRegistryStartupError::Goods);
            if report.is_ok() {
                add_log_text(b"Initial SI_GOODSLIST...OK!");
            }
            Some(report.map(GameCombatRegistryStartupReport::Goods))
        }
        MONSTER_LIST_SELECTOR => {
            let decoded = match game
                .decode_monster_list(source, cursor)
                .map_err(GameCombatRegistryStartupError::Monsters)
            {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            add_log_text(b"Initial SI_MONSTERLIST...OK!");
            let refreshed = game.refresh_all_monster_base_property();
            Some(Ok(GameCombatRegistryStartupReport::Monsters(
                MonsterListStartupReport { decoded, refreshed },
            )))
        }
        SKILL_LIST_SELECTOR => {
            let report = game
                .skill_factory_mut()
                .rebuild(source, cursor)
                .map_err(GameCombatRegistryStartupError::Skills);
            if report.is_ok() {
                add_log_text(b"Initial SI_SKILLLIST...OK!");
            }
            Some(report.map(GameCombatRegistryStartupReport::Skills))
        }
        _ => None,
    }
}

fn decode_battle_fairy_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<GameBattleFairyStartupReport, GameBattleFairyStartupError>> {
    match selector {
        FAIRY_EXP_SELECTOR => {
            let report = game
                .fairy_exp_conf_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameBattleFairyStartupError::FairyExp);
            if report.is_ok() {
                add_log_text(b"Initial SI_FAIRY_EXP...ok!");
            }
            Some(report.map(GameBattleFairyStartupReport::FairyExp))
        }
        BATTLE_FAIRY_EXP_SELECTOR => {
            let report = game
                .battle_fairy_exp_config_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameBattleFairyStartupError::BattleFairyExp);
            if report.is_ok() {
                add_log_text(b"Initial SI_BATLLE_FAIRY_CONF...ok\xA3\xA1");
            }
            Some(report.map(GameBattleFairyStartupReport::BattleFairyExp))
        }
        BATTLE_FAIRY_COMBINE_SELECTOR => {
            let entries = game
                .battle_fairy_property_mut()
                .decord_byte_array_combine(source, cursor)
                .map_err(GameBattleFairyStartupError::Combine);
            if entries.is_ok() {
                add_log_text(b"Initial SI_BATLLE_FAIRY_COMBINE...ok!");
            }
            Some(entries.map(|entries| GameBattleFairyStartupReport::Combine { entries }))
        }
        EQUIPMENT_COMPOSE_SELECTOR => Some(
            game.equipment_compose_list_mut()
                .decord_from_byte_array(source, cursor)
                .map(GameBattleFairyStartupReport::EquipmentCompose)
                .map_err(GameBattleFairyStartupError::EquipmentCompose),
        ),
        _ => None,
    }
}

fn dispatch_player_count_message(
    message_type: i32,
    game: &CGame,
) -> Option<GamePlayerCountResponseReport> {
    let (kind, response_type, requires_world_client) = match message_type {
        PLAYER_COUNT_IF_WORLD_CONNECTED_MESSAGE => (
            GamePlayerCountResponseKind::WorldConnected,
            PLAYER_COUNT_IF_WORLD_CONNECTED_RESPONSE,
            true,
        ),
        PLAYER_COUNT_MESSAGE => (
            GamePlayerCountResponseKind::Unconditional,
            PLAYER_COUNT_RESPONSE,
            false,
        ),
        _ => return None,
    };
    if requires_world_client && game.world_client().is_none() {
        return Some(GamePlayerCountResponseReport {
            kind,
            player_count: None,
            outcome: GamePlayerCountResponseOutcome::MissingWorldClient,
        });
    }

    let player_count = game.player_count();
    let mut response = CMessage::new(response_type);
    response.add_ulong(player_count);
    Some(GamePlayerCountResponseReport {
        kind,
        player_count: Some(player_count),
        outcome: GamePlayerCountResponseOutcome::Sent(response.send(game, false)),
    })
}

fn dispatch_string_table_message(
    message: &mut CMessage,
    game: &mut CGame,
    source: GameStringTableSource,
) -> Result<GameServerMessageReport, GameServerMessageError> {
    let mut log_effects = Vec::new();
    let decoded = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        game.create_string_table(wire, cursor, |text| log_effects.push(text.to_vec()))
            .map_err(GameServerMessageError::StringTable)?
    };
    Ok(GameServerMessageReport::StringTable(
        GameStringTableMessageReport {
            source,
            decoded,
            log_effects,
        },
    ))
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

    let network = match game.init_net_server(now_ms) {
        Ok(()) => GameClientServerStartNetwork::Started,
        Err(GameNetworkInitializationError::MissingNetworkSetup) => {
            GameClientServerStartNetwork::MissingNetworkSetup
        }
        Err(GameNetworkInitializationError::Host(error)) => {
            GameClientServerStartNetwork::HostFailed {
                detail: error.to_string(),
            }
        }
    };
    let mut effects = Vec::new();
    if network != GameClientServerStartNetwork::Started {
        effects.push(GameClientServerStartEffect::FailureDialog {
            message: b"Can't init NetServer!",
            title: b"Message",
        });
        effects.push(GameClientServerStartEffect::Log(
            b"==========Initial NetServer FAILED==========".to_vec(),
        ));
    }
    let (monsters, npcs) = game.initial_region_totals();
    effects.push(GameClientServerStartEffect::Log(
        format!("GS : Monster={monsters} Npc={npcs}!").into_bytes(),
    ));
    effects.push(GameClientServerStartEffect::Log(
        b"GameServer As Client Server SUCCESS!".to_vec(),
    ));
    let (server_ids, applied_login_id, applied_world_id) = read_and_apply_server_ids(message, game);
    Some(GameClientServerStartReport {
        network,
        effects,
        server_ids,
        applied_login_id,
        applied_world_id,
    })
}

fn read_and_apply_server_ids(
    message: &mut CMessage,
    game: &mut CGame,
) -> (
    Result<GameServerIds, GameClientServerStartPayloadError>,
    Option<i32>,
    Option<i32>,
) {
    let login = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match read_start_long(wire, cursor) {
            Ok(login) => login,
            Err(error) => return (Err(error), None, None),
        }
    };
    game.set_login_server_id(login);

    let world = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match read_start_long(wire, cursor) {
            Ok(world) => world,
            Err(error) => return (Err(error), Some(login), None),
        }
    };
    game.set_world_server_id(world);
    (Ok(GameServerIds { login, world }), Some(login), Some(world))
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

pub(crate) trait InitialRegionStartupContext:
    CityRegionDecodeContext + CountryRegionDecodeContext
{
    fn total_region_monsters(&self) -> i32;
    fn total_region_npcs(&self) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InitialRegionSubtypeInputBlock {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InitialRegionStartupError<RuntimeError> {
    SubtypeInput(InitialRegionSubtypeInputBlock),
    UnknownSubtype(i32),
    Base(ServerRegionDecodeError<RuntimeError>),
    War(WarRegionDecodeError<ServerRegionDecodeError<RuntimeError>>),
    City(CityRegionDecodeError<ServerRegionDecodeError<RuntimeError>>),
    Country(CountryRegionDecodeError<ServerRegionDecodeError<RuntimeError>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InitialRegionStartupReport {
    pub(crate) subtype: i32,
    pub(crate) region_id: i32,
    pub(crate) added_to_region_list: bool,
    pub(crate) replaced: bool,
    pub(crate) total_monsters: i32,
    pub(crate) total_npcs: i32,
    pub(crate) gods_battle_registered: bool,
}

/// Выполняет startup case `0x0E`, включая allocation/decode, display-list
/// gate, map assignment, totals snapshot и GodsBattle registration.
pub(crate) fn dispatch_initial_region_startup<Context, AddRegionList, AddLogText>(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
    mut add_region_list: AddRegionList,
    mut add_log_text: AddLogText,
) -> Option<Result<InitialRegionStartupReport, InitialRegionStartupError<Context::RuntimeError>>>
where
    Context: InitialRegionStartupContext,
    AddRegionList: FnMut(&[u8], i32),
    AddLogText: FnMut(&[u8]),
{
    if selector != REGION_SELECTOR {
        return None;
    }

    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let subtype = match read_initial_region_i32(source, cursor) {
        Ok(subtype) => subtype,
        Err(error) => return Some(Err(InitialRegionStartupError::SubtypeInput(error))),
    };
    let owner = match subtype {
        0 => {
            let mut region = CServerRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::Base(error)));
            }
            ServerRegionOwner::Base(region)
        }
        1 => {
            let mut region = CServerVillageRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::War(error)));
            }
            ServerRegionOwner::Village(region)
        }
        2 => {
            let mut region = CServerCityRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::City(error)));
            }
            ServerRegionOwner::City(region)
        }
        3 => {
            let mut region = CServerCountryRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::Country(error)));
            }
            ServerRegionOwner::Country(region)
        }
        4 => {
            let mut region = ServerNationRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::War(error)));
            }
            ServerRegionOwner::Nation(region)
        }
        5 => {
            let mut region = CServerGodsBattleRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true, context) {
                return Some(Err(InitialRegionStartupError::War(error)));
            }
            ServerRegionOwner::GodsBattle(region)
        }
        subtype => return Some(Err(InitialRegionStartupError::UnknownSubtype(subtype))),
    };

    let region_id = owner.region_id();
    let added_to_region_list = game.find_region(region_id).is_none();
    if added_to_region_list {
        add_region_list(owner.name(), region_id);
    }
    let gods_battle = owner.is_gods_battle();
    let replaced = game.add_region(owner);
    add_log_text(b"Start Region : (%d) %s [m=%d n=%d] ...OK!");

    let total_monsters = context.total_region_monsters();
    let total_npcs = context.total_region_npcs();
    game.set_initial_region_totals(total_monsters, total_npcs);
    let gods_battle_registered =
        gods_battle && game.gods_battle_mgr_mut().add_region_set(region_id);

    Some(Ok(InitialRegionStartupReport {
        subtype,
        region_id,
        added_to_region_list,
        replaced,
        total_monsters,
        total_npcs,
        gods_battle_registered,
    }))
}

fn read_initial_region_i32(
    source: &[u8],
    cursor: &mut usize,
) -> Result<i32, InitialRegionSubtypeInputBlock> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(InitialRegionSubtypeInputBlock {
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(InitialRegionSubtypeInputBlock {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .expect("region subtype занимает четыре байта"),
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSetupReloadError {
    RegionId(InitialRegionSubtypeInputBlock),
    Setup(ServerRegionSetupDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionSetupReloadReport {
    pub(crate) region_id: i32,
    pub(crate) found: bool,
}

pub(crate) fn dispatch_region_setup_reload(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<RegionSetupReloadReport, RegionSetupReloadError>> {
    if selector != REGION_RELOAD_SELECTOR {
        return None;
    }

    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    Some(
        match decode_spatial_startup(selector, source, cursor, game, &mut add_log_text)
            .expect("region-reload selector проверен dispatcher-ом")
        {
            Ok(GameSpatialStartupReport::RegionReload(report)) => Ok(report),
            Err(GameSpatialStartupError::RegionReload(error)) => Err(error),
            _ => unreachable!("selector 0x10 возвращает только region-reload variant"),
        },
    )
}

pub(crate) fn dispatch_monster_list_startup(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<MonsterListStartupReport, MonsterListDecodeError>> {
    if selector != MONSTER_LIST_SELECTOR {
        return None;
    }

    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    Some(
        match decode_combat_registry_startup(selector, source, cursor, game, &mut add_log_text)
            .expect("monster-list selector проверен dispatcher-ом")
        {
            Ok(GameCombatRegistryStartupReport::Monsters(report)) => Ok(report),
            Err(GameCombatRegistryStartupError::Monsters(error)) => Err(error),
            Ok(_) | Err(_) => unreachable!("selector 0x02 возвращает только monster-list variant"),
        },
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterListStartupReport {
    pub(crate) decoded: MonsterListDecodeReport,
    pub(crate) refreshed: MonsterBasePropertyRefreshReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameScriptResourceDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    NegativeLength {
        resource: &'static str,
        declared: i32,
    },
}

impl fmt::Display for GameScriptResourceDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                required,
                available,
            } => write!(
                formatter,
                "script resource обрывается на {field} в {offset}: нужно {required}, доступно {available}"
            ),
            Self::NegativeLength { resource, declared } => write!(
                formatter,
                "script resource {resource} содержит отрицательную длину {declared}"
            ),
        }
    }
}

impl Error for GameScriptResourceDecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameIdIndexDecodeError {
    pub(crate) offset: usize,
    pub(crate) available: usize,
}

impl fmt::Display for GameIdIndexDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Game ID index отсутствует в {0}: нужно 1, доступно {1}",
            self.offset, self.available
        )
    }
}

impl Error for GameIdIndexDecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOwnedStartupSnapshotReport {
    GoodsList(GoodsFactoryDecodeReport),
    PlayerList(PlayerListDecodeReport),
    TradeList {
        entries: usize,
    },
    IncrementShop {
        entries: usize,
    },
    ContributeSetup {
        entries: usize,
    },
    SkillList(SkillFactoryDecodeReport),
    GlobeSetup {
        decoded: GlobeSetupDecodeReport,
        goods_ai_broadcast: Result<i32, SendMessageError>,
        auction_forced_disabled: bool,
    },
    LogSystem {
        entries: usize,
        da_kong_log: bool,
    },
    GmList(GmListDecodeReport),
    FunctionList {
        declared_length: i32,
        publication: GameSingleFilePublication,
    },
    VariableList {
        declared_length: i32,
        publication: GameSingleFilePublication,
    },
    GeneralVariable {
        start_offset: usize,
    },
    ScriptFile {
        path_bytes: usize,
        declared_length: i32,
        replaced: bool,
    },
    ProxyRegion {
        region_id: i32,
        replaced: bool,
    },
    RegionSetup {
        entries: usize,
    },
    IdIndex {
        value: u8,
    },
    HitLevel {
        entries: usize,
    },
    Emotion(EmotionDecodeReport),
    QuestSystem(QuestSystemDecodeReport),
    PlayerRanks {
        entries: usize,
    },
    CountryParam(CountryParamDecodeReport),
    CountryHandler(CountryHandlerDecodeReport),
    DupliRegions {
        entries: usize,
    },
    PrisonConf {
        entries: usize,
    },
    PreciousBoxConf {
        entries: usize,
    },
    FairyExp(BattleFairyExpDecodeReport),
    Synthesis(SynthesisDecodeReport),
    NewSkillMonster(NewSkillMonsterDecodeReport),
    GoodsDestroy(GoodsDestroyDecodeReport),
    ChangeBody {
        entries: usize,
    },
    HonorEliminate {
        level_difference: i32,
        minimum_level: i32,
    },
    DaKong(DaKongDecodeReport),
    BattleFairyExp(BattleFairyExpDecodeReport),
    BattleFairyCombine {
        entries: usize,
    },
    StringTable(MyStringTableDecodeReport),
    EquipmentCompose(EquipmentComposeDecodeReport),
    WordsFilter(WordsFilterDecodeReport),
    JjcRegionLevel {
        entries: usize,
    },
    TaoZhuang {
        skill_ids: usize,
        items: usize,
        broadcast: Result<i32, SendMessageError>,
    },
    CiQingLingBao {
        ci_qing: CiQingDecodeReport,
        ling_bao: LingBaoDecodeReport,
        broadcast: Result<i32, SendMessageError>,
    },
    ThingSetup {
        entries: usize,
    },
    GodsBattle(GodsBattleDecodeReport),
}

#[derive(Debug)]
pub(crate) enum GameOwnedStartupSnapshotError {
    OwnerUnavailable { selector: i32 },
    GoodsList(GoodsFactoryDecodeError),
    PlayerList(PlayerListDecodeError),
    TradeList(TradeListDecodeError),
    IncrementShop(IncrementShopDecodeError),
    ContributeSetup(ContributeSetupDecodeError),
    SkillList(SkillFactoryDecodeError),
    GlobeSetup(GlobeSetupDecodeError),
    LogSystem(LogSystemDecodeError),
    GmList(GmListDecodeError),
    ScriptResource(GameScriptResourceDecodeError),
    ProxyRegion(ProxyRegionDecodeError),
    RegionSetup(RegionSetupDecodeError),
    IdIndex(GameIdIndexDecodeError),
    HitLevel(HitLevelDecodeError),
    Emotion(EmotionDecodeError),
    QuestSystem(QuestSystemDecodeError),
    PlayerRanks(PlayerRanksDecodeError),
    CountryParam(CountryParamInputBlock),
    CountryHandler(CountryHandlerDecodeError),
    DupliRegions(DupliRegionDecodeError),
    PrisonConf(PrisonConfDecodeError),
    PreciousBoxConf(PreciousBoxDecodeError),
    FairyExp(BattleFairyExpDecodeError),
    Synthesis(SynthesisDecodeError),
    NewSkillMonster(NewSkillMonsterDecodeError),
    GoodsDestroy(GoodsDestroyDecodeError),
    ChangeBody(ChangeBodyDecodeError),
    HonorEliminate(HonorEliminateDecodeError),
    DaKong(DaKongDecodeError),
    BattleFairyExp(BattleFairyExpDecodeError),
    BattleFairyCombine(BattleFairyComposeDecodeError),
    StringTable(MyStringTableDecodeError),
    EquipmentCompose(EquipmentComposeDecodeError),
    WordsFilter(WordsFilterDecodeError),
    JjcRegionLevel(JjcRegionLevelDecodeError),
    TaoZhuang(TaoZhuangDecodeError),
    TaoZhuangSerialize(TaoZhuangSerializationBlock),
    CiQing(CiQingDecodeError),
    LingBao(LingBaoDecodeError),
    CiQingSerialize(CiQingSerializationBlock),
    ThingSetup(ThingSetupCodecError),
    GodsBattle(GodsBattleDecodeError),
}

impl fmt::Display for GameOwnedStartupSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerUnavailable { selector } => {
                write!(
                    formatter,
                    "startup owner selector {selector:#x} ещё не создан CGame::Init"
                )
            }
            Self::GoodsList(error) => error.fmt(formatter),
            Self::PlayerList(error) => error.fmt(formatter),
            Self::TradeList(error) => error.fmt(formatter),
            Self::IncrementShop(error) => error.fmt(formatter),
            Self::ContributeSetup(error) => error.fmt(formatter),
            Self::SkillList(error) => error.fmt(formatter),
            Self::GlobeSetup(error) => error.fmt(formatter),
            Self::LogSystem(error) => error.fmt(formatter),
            Self::GmList(error) => error.fmt(formatter),
            Self::ScriptResource(error) => error.fmt(formatter),
            Self::ProxyRegion(error) => error.fmt(formatter),
            Self::RegionSetup(error) => error.fmt(formatter),
            Self::IdIndex(error) => error.fmt(formatter),
            Self::HitLevel(error) => error.fmt(formatter),
            Self::Emotion(error) => error.fmt(formatter),
            Self::QuestSystem(error) => error.fmt(formatter),
            Self::PlayerRanks(error) => error.fmt(formatter),
            Self::CountryParam(error) => error.fmt(formatter),
            Self::CountryHandler(error) => error.fmt(formatter),
            Self::DupliRegions(error) => error.fmt(formatter),
            Self::PrisonConf(error) => error.fmt(formatter),
            Self::PreciousBoxConf(error) => error.fmt(formatter),
            Self::FairyExp(error) => error.fmt(formatter),
            Self::Synthesis(error) => error.fmt(formatter),
            Self::NewSkillMonster(error) => error.fmt(formatter),
            Self::GoodsDestroy(error) => error.fmt(formatter),
            Self::ChangeBody(error) => error.fmt(formatter),
            Self::HonorEliminate(error) => error.fmt(formatter),
            Self::DaKong(error) => error.fmt(formatter),
            Self::BattleFairyExp(error) => error.fmt(formatter),
            Self::BattleFairyCombine(error) => error.fmt(formatter),
            Self::StringTable(error) => error.fmt(formatter),
            Self::EquipmentCompose(error) => error.fmt(formatter),
            Self::WordsFilter(error) => error.fmt(formatter),
            Self::JjcRegionLevel(error) => error.fmt(formatter),
            Self::TaoZhuang(error) => error.fmt(formatter),
            Self::TaoZhuangSerialize(error) => error.fmt(formatter),
            Self::CiQing(error) => error.fmt(formatter),
            Self::LingBao(error) => error.fmt(formatter),
            Self::CiQingSerialize(error) => error.fmt(formatter),
            Self::ThingSetup(error) => error.fmt(formatter),
            Self::GodsBattle(error) => error.fmt(formatter),
        }
    }
}

impl Error for GameOwnedStartupSnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OwnerUnavailable { .. } => None,
            Self::GoodsList(error) => Some(error),
            Self::PlayerList(error) => Some(error),
            Self::TradeList(error) => Some(error),
            Self::IncrementShop(error) => Some(error),
            Self::ContributeSetup(error) => Some(error),
            Self::SkillList(error) => Some(error),
            Self::GlobeSetup(error) => Some(error),
            Self::LogSystem(error) => Some(error),
            Self::GmList(error) => Some(error),
            Self::ScriptResource(error) => Some(error),
            Self::ProxyRegion(error) => Some(error),
            Self::RegionSetup(error) => Some(error),
            Self::IdIndex(error) => Some(error),
            Self::HitLevel(error) => Some(error),
            Self::Emotion(error) => Some(error),
            Self::QuestSystem(error) => Some(error),
            Self::PlayerRanks(error) => Some(error),
            Self::CountryParam(error) => Some(error),
            Self::CountryHandler(error) => Some(error),
            Self::DupliRegions(error) => Some(error),
            Self::PrisonConf(error) => Some(error),
            Self::PreciousBoxConf(error) => Some(error),
            Self::FairyExp(error) => Some(error),
            Self::Synthesis(error) => Some(error),
            Self::NewSkillMonster(error) => Some(error),
            Self::GoodsDestroy(error) => Some(error),
            Self::ChangeBody(error) => Some(error),
            Self::HonorEliminate(error) => Some(error),
            Self::DaKong(error) => Some(error),
            Self::BattleFairyExp(error) => Some(error),
            Self::BattleFairyCombine(error) => Some(error),
            Self::StringTable(error) => Some(error),
            Self::EquipmentCompose(error) => Some(error),
            Self::WordsFilter(error) => Some(error),
            Self::JjcRegionLevel(error) => Some(error),
            Self::TaoZhuang(error) => Some(error),
            Self::TaoZhuangSerialize(error) => Some(error),
            Self::CiQing(error) => Some(error),
            Self::LingBao(error) => Some(error),
            Self::CiQingSerialize(error) => Some(error),
            Self::ThingSetup(error) => Some(error),
            Self::GodsBattle(error) => Some(error),
        }
    }
}

/// Декодирует startup snapshots, чьи state owners уже принадлежат `CGame`.
pub(crate) fn dispatch_game_owned_startup_snapshot<Context: GameScriptResourceContext>(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    script_context: &mut Context,
    mut add_log_text: impl FnMut(&[u8]),
    mut put_string_to_file: impl FnMut(&str, &[u8]),
) -> Option<Result<GameOwnedStartupSnapshotReport, GameOwnedStartupSnapshotError>> {
    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    if matches!(
        selector,
        PROXY_REGION_SELECTOR | REGION_SETUP_SELECTOR | DUPLI_REGION_SELECTOR
    ) && let Some(result) =
        decode_spatial_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GameSpatialStartupReport::ProxyRegion {
                region_id,
                replaced,
            }) => Ok(GameOwnedStartupSnapshotReport::ProxyRegion {
                region_id,
                replaced,
            }),
            Ok(GameSpatialStartupReport::RegionSetup { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::RegionSetup { entries })
            }
            Ok(GameSpatialStartupReport::DupliRegions { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::DupliRegions { entries })
            }
            Err(GameSpatialStartupError::ProxyRegion(error)) => {
                Err(GameOwnedStartupSnapshotError::ProxyRegion(error))
            }
            Err(GameSpatialStartupError::RegionSetup(error)) => {
                Err(GameOwnedStartupSnapshotError::RegionSetup(error))
            }
            Err(GameSpatialStartupError::OwnerUnavailable { selector }) => {
                Err(GameOwnedStartupSnapshotError::OwnerUnavailable { selector })
            }
            Err(GameSpatialStartupError::DupliRegions(error)) => {
                Err(GameOwnedStartupSnapshotError::DupliRegions(
                    Arc::try_unwrap(error)
                        .expect("dupli decode error ещё не разделён между reports"),
                ))
            }
            Ok(GameSpatialStartupReport::RegionReload(_))
            | Err(GameSpatialStartupError::RegionReload(_)) => {
                unreachable!("selector 0x0F/0x11/0x1A не возвращает reload variant")
            }
        });
    }
    if let Some(result) =
        decode_country_state_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GameCountryStateStartupReport::Parameters(report)) => {
                Ok(GameOwnedStartupSnapshotReport::CountryParam(report))
            }
            Ok(GameCountryStateStartupReport::Countries(report)) => {
                Ok(GameOwnedStartupSnapshotReport::CountryHandler(report))
            }
            Err(GameCountryStateStartupError::Parameters(error)) => {
                Err(GameOwnedStartupSnapshotError::CountryParam(error))
            }
            Err(GameCountryStateStartupError::Countries(error)) => {
                Err(GameOwnedStartupSnapshotError::CountryHandler(error))
            }
        });
    }
    if let Some(result) =
        decode_player_rule_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GamePlayerRuleStartupReport::IdIndex { value }) => {
                Ok(GameOwnedStartupSnapshotReport::IdIndex { value })
            }
            Ok(GamePlayerRuleStartupReport::HitLevel { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::HitLevel { entries })
            }
            Ok(GamePlayerRuleStartupReport::Emotion(report)) => {
                Ok(GameOwnedStartupSnapshotReport::Emotion(report))
            }
            Ok(GamePlayerRuleStartupReport::QuestSystem(report)) => {
                Ok(GameOwnedStartupSnapshotReport::QuestSystem(report))
            }
            Err(GamePlayerRuleStartupError::IdIndex(error)) => {
                Err(GameOwnedStartupSnapshotError::IdIndex(error))
            }
            Err(GamePlayerRuleStartupError::HitLevel(error)) => {
                Err(GameOwnedStartupSnapshotError::HitLevel(error))
            }
            Err(GamePlayerRuleStartupError::Emotion(error)) => {
                Err(GameOwnedStartupSnapshotError::Emotion(error))
            }
            Err(GamePlayerRuleStartupError::QuestSystem(error)) => {
                Err(GameOwnedStartupSnapshotError::QuestSystem(error))
            }
        });
    }
    if let Some(result) =
        decode_runtime_configuration_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GameRuntimeConfigurationStartupReport::GlobeSetup {
                decoded,
                goods_ai_broadcast,
                auction_forced_disabled,
            }) => Ok(GameOwnedStartupSnapshotReport::GlobeSetup {
                decoded,
                goods_ai_broadcast,
                auction_forced_disabled,
            }),
            Ok(GameRuntimeConfigurationStartupReport::LogSystem {
                entries,
                da_kong_log,
            }) => Ok(GameOwnedStartupSnapshotReport::LogSystem {
                entries,
                da_kong_log,
            }),
            Ok(GameRuntimeConfigurationStartupReport::GmList(report)) => {
                Ok(GameOwnedStartupSnapshotReport::GmList(report))
            }
            Err(GameRuntimeConfigurationStartupError::GlobeSetup(error)) => {
                Err(GameOwnedStartupSnapshotError::GlobeSetup(error))
            }
            Err(GameRuntimeConfigurationStartupError::LogSystem(error)) => {
                Err(GameOwnedStartupSnapshotError::LogSystem(error))
            }
            Err(GameRuntimeConfigurationStartupError::GmList(error)) => {
                Err(GameOwnedStartupSnapshotError::GmList(error))
            }
        });
    }
    if let Some(result) =
        decode_player_economy_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GamePlayerEconomyStartupReport::PlayerTemplates(report)) => {
                Ok(GameOwnedStartupSnapshotReport::PlayerList(report))
            }
            Ok(GamePlayerEconomyStartupReport::TradeList { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::TradeList { entries })
            }
            Ok(GamePlayerEconomyStartupReport::IncrementShop { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::IncrementShop { entries })
            }
            Ok(GamePlayerEconomyStartupReport::ContributionItems { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::ContributeSetup { entries })
            }
            Err(GamePlayerEconomyStartupError::PlayerTemplates(error)) => {
                Err(GameOwnedStartupSnapshotError::PlayerList(error))
            }
            Err(GamePlayerEconomyStartupError::TradeList(error)) => {
                Err(GameOwnedStartupSnapshotError::TradeList(error))
            }
            Err(GamePlayerEconomyStartupError::IncrementShop(error)) => {
                Err(GameOwnedStartupSnapshotError::IncrementShop(error))
            }
            Err(GamePlayerEconomyStartupError::ContributionItems(error)) => {
                Err(GameOwnedStartupSnapshotError::ContributeSetup(error))
            }
        });
    }
    if matches!(selector, GOODS_LIST_SELECTOR | SKILL_LIST_SELECTOR)
        && let Some(result) =
            decode_combat_registry_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GameCombatRegistryStartupReport::Goods(report)) => {
                Ok(GameOwnedStartupSnapshotReport::GoodsList(report))
            }
            Ok(GameCombatRegistryStartupReport::Skills(report)) => {
                Ok(GameOwnedStartupSnapshotReport::SkillList(report))
            }
            Err(GameCombatRegistryStartupError::Goods(error)) => {
                Err(GameOwnedStartupSnapshotError::GoodsList(error))
            }
            Err(GameCombatRegistryStartupError::Skills(error)) => {
                Err(GameOwnedStartupSnapshotError::SkillList(error))
            }
            Ok(GameCombatRegistryStartupReport::Monsters(_))
            | Err(GameCombatRegistryStartupError::Monsters(_)) => {
                unreachable!("selector 0x00/0x06 не возвращает monster-list variant")
            }
        });
    }
    if let Some(result) =
        decode_battle_fairy_startup(selector, source, cursor, game, &mut add_log_text)
    {
        return Some(match result {
            Ok(GameBattleFairyStartupReport::FairyExp(report)) => {
                Ok(GameOwnedStartupSnapshotReport::FairyExp(report))
            }
            Ok(GameBattleFairyStartupReport::BattleFairyExp(report)) => {
                Ok(GameOwnedStartupSnapshotReport::BattleFairyExp(report))
            }
            Ok(GameBattleFairyStartupReport::Combine { entries }) => {
                Ok(GameOwnedStartupSnapshotReport::BattleFairyCombine { entries })
            }
            Ok(GameBattleFairyStartupReport::EquipmentCompose(report)) => {
                Ok(GameOwnedStartupSnapshotReport::EquipmentCompose(report))
            }
            Err(GameBattleFairyStartupError::FairyExp(error)) => {
                Err(GameOwnedStartupSnapshotError::FairyExp(error))
            }
            Err(GameBattleFairyStartupError::BattleFairyExp(error)) => {
                Err(GameOwnedStartupSnapshotError::BattleFairyExp(error))
            }
            Err(GameBattleFairyStartupError::Combine(error)) => {
                Err(GameOwnedStartupSnapshotError::BattleFairyCombine(error))
            }
            Err(GameBattleFairyStartupError::EquipmentCompose(error)) => {
                Err(GameOwnedStartupSnapshotError::EquipmentCompose(error))
            }
        });
    }
    match selector {
        FUNCTION_LIST_SELECTOR => {
            let declared_length = match read_script_resource_length(source, cursor, "FunctionList")
            {
                Ok(length) => length,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let data = match take_script_resource_bytes(
                source,
                cursor,
                declared_length as usize,
                "FunctionList data",
            ) {
                Ok(data) => data.to_vec(),
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let publication = game.set_function_file_data(data, script_context);
            add_log_text(b"FunctionList...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::FunctionList {
                declared_length,
                publication,
            }))
        }
        VARIABLE_LIST_SELECTOR => {
            let declared_length = match read_script_resource_length(source, cursor, "VariableList")
            {
                Ok(length) => length,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let data = match take_script_resource_bytes(
                source,
                cursor,
                declared_length as usize,
                "VariableList data",
            ) {
                Ok(data) => data.to_vec(),
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let publication = game.set_variable_file_data(data);
            add_log_text(b"VariableList...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::VariableList {
                declared_length,
                publication,
            }))
        }
        GENERAL_VARIABLE_SELECTOR => {
            let start_offset = *cursor;
            game.set_general_variable_file_data(source, start_offset, script_context);
            add_log_text(b"GeneralVariableList...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::GeneralVariable {
                start_offset,
            }))
        }
        SCRIPT_FILE_SELECTOR => {
            let path = read_script_resource_path(source, cursor, 0x104);
            let declared_length = match read_script_resource_length(source, cursor, "ScriptFile") {
                Ok(length) => length,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let data = match take_script_resource_bytes(
                source,
                cursor,
                declared_length as usize,
                "ScriptFile data",
            ) {
                Ok(data) => data.to_vec(),
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ScriptResource(error)));
                }
            };
            let path_bytes = path.len();
            let replaced = game.set_script_file_data(path, data);
            Some(Ok(GameOwnedStartupSnapshotReport::ScriptFile {
                path_bytes,
                declared_length,
                replaced,
            }))
        }
        PLAYER_RANKS_SELECTOR => {
            let Some(ranks) = game.player_ranks_mut() else {
                return Some(Err(GameOwnedStartupSnapshotError::OwnerUnavailable {
                    selector,
                }));
            };
            if let Err(error) = ranks.decord_from_byte_array(source, cursor) {
                return Some(Err(GameOwnedStartupSnapshotError::PlayerRanks(error)));
            }
            Some(Ok(GameOwnedStartupSnapshotReport::PlayerRanks {
                entries: ranks.ranks().len(),
            }))
        }
        PRISON_CONF_SELECTOR => {
            let entries = match game
                .prison_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::PrisonConf(error))),
            };
            add_log_text(b"Initial SI_PRISON_CONF...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::PrisonConf { entries }))
        }
        PRECIOUS_BOX_CONF_SELECTOR => {
            let entries = match game
                .precious_box_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::PreciousBoxConf(error)));
                }
            };
            add_log_text(b"Initial SI_PRECIOUSBOX_CONF...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::PreciousBoxConf {
                entries,
            }))
        }
        SYNTHESIS_SELECTOR => {
            let report = match game.synthesis_mut().decord_from_byte_array(source, cursor) {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::Synthesis(error))),
            };
            add_log_text(b"Initial SI_SYNTHESIS...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::Synthesis(report)))
        }
        NEW_SKILL_MONSTER_SELECTOR => {
            let report = match game
                .new_skill_monster_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::NewSkillMonster(error)));
                }
            };
            add_log_text(b"Initial SI_NEWSKILL_MONSTER_CONF...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::NewSkillMonster(report)))
        }
        GOODS_DESTROY_SELECTOR => {
            let report = match game
                .goods_destroy_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::GoodsDestroy(error))),
            };
            add_log_text(b"Initial SI_GOODS_DESTROY_CONF...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::GoodsDestroy(report)))
        }
        CHANGE_BODY_SELECTOR => {
            let entries = match game
                .change_body_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::ChangeBody(error))),
            };
            add_log_text(b"Initial SI_CHANGE_BODY...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::ChangeBody { entries }))
        }
        HONOR_ELIMINATE_SELECTOR => {
            let config = game.honor_eliminate_config_mut();
            if let Err(error) = config.decord_from_byte_array(source, cursor) {
                return Some(Err(GameOwnedStartupSnapshotError::HonorEliminate(error)));
            }
            let level_difference = config.level_difference;
            let minimum_level = config.minimum_level;
            add_log_text(b"Inital SI_HONOR_ELIMILATE_CONF...ok!");
            put_string_to_file(
                "HonorCompositior",
                b"Inital SI_HONOR_ELIMILATE_CONF...ok\xA3\xA1",
            );
            Some(Ok(GameOwnedStartupSnapshotReport::HonorEliminate {
                level_difference,
                minimum_level,
            }))
        }
        DA_KONG_SELECTOR => {
            let report = match game
                .da_kong_xiang_qian_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::DaKong(error))),
            };
            Some(Ok(GameOwnedStartupSnapshotReport::DaKong(report)))
        }
        STRING_TABLE_SELECTOR => {
            let report = match game.create_string_table(source, cursor, &mut add_log_text) {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::StringTable(error)));
                }
            };
            Some(Ok(GameOwnedStartupSnapshotReport::StringTable(report)))
        }
        WORDS_FILTER_SELECTOR => {
            let report = match game.words_filter_mut().from_byte_array(source, cursor) {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::WordsFilter(error)));
                }
            };
            add_log_text(b"Initial SI_WORDSFILTER...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::WordsFilter(report)))
        }
        JJC_REGION_LEVEL_SELECTOR => {
            game.clear_jjc_level_data();
            let count = match read_jjc_level_i32(source, cursor) {
                Ok(count) => count,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::JjcRegionLevel(error)));
                }
            };
            for _ in 0..count.max(0) {
                let key = match read_jjc_level_i32(source, cursor) {
                    Ok(key) => key,
                    Err(error) => {
                        return Some(Err(GameOwnedStartupSnapshotError::JjcRegionLevel(error)));
                    }
                };
                let value = match read_jjc_level_i32(source, cursor) {
                    Ok(value) => value,
                    Err(error) => {
                        return Some(Err(GameOwnedStartupSnapshotError::JjcRegionLevel(error)));
                    }
                };
                game.insert_jjc_level_data(key, value);
            }
            let entries = game.jjc_level_data().len();
            add_log_text(b"Initial SI_JJCREGIONLEVELSETUP...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::JjcRegionLevel {
                entries,
            }))
        }
        TAO_ZHUANG_SELECTOR => {
            let decoded = match game.tao_zhuang_setup_mut().decode_from_byte(source, cursor) {
                Ok(decoded) => decoded,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::TaoZhuang(error)));
                }
            };
            add_log_text(b"Add TaoZhuangSetup....OK");

            let mut payload = Vec::new();
            if let Err(error) = game.tao_zhuang_setup().add_byte_to_array(&mut payload) {
                return Some(Err(GameOwnedStartupSnapshotError::TaoZhuangSerialize(
                    error,
                )));
            }
            let mut notice = CMessage::new(0x000B_F81A);
            notice.base_mut().add(&payload);
            let broadcast = notice.send_all(game.current_net_server());
            Some(Ok(GameOwnedStartupSnapshotReport::TaoZhuang {
                skill_ids: decoded.skill_ids,
                items: decoded.items,
                broadcast,
            }))
        }
        CI_QING_LING_BAO_SELECTOR => {
            let ci_qing = match game.ci_qing_setup_mut().de_byte_from_array(source, cursor) {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::CiQing(error))),
            };
            let ling_bao = match game
                .ling_bao_setup_mut()
                .decode_from_array_ling_bao(source, cursor)
            {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::LingBao(error))),
            };

            let mut payload = Vec::new();
            if let Err(error) = game.ci_qing_setup().add_byte_to_array(&mut payload) {
                return Some(Err(GameOwnedStartupSnapshotError::CiQingSerialize(error)));
            }
            let mut notice = CMessage::new(0x000C_010D);
            notice.base_mut().add(&payload);
            let broadcast = notice.send_all(game.current_net_server());
            add_log_text(b"Add CiQingSetup...ok!");
            Some(Ok(GameOwnedStartupSnapshotReport::CiQingLingBao {
                ci_qing,
                ling_bao,
                broadcast,
            }))
        }
        THING_SETUP_SELECTOR => {
            if let Err(error) = game
                .thing_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                return Some(Err(GameOwnedStartupSnapshotError::ThingSetup(error)));
            }
            let entries = game.thing_setup().all_things().len();
            let decoded_line = format!("GS Leiting Decord:line {entries}");
            add_log_text(decoded_line.as_bytes());
            add_log_text(b"Initial Strictest Enforcement...ok!");
            Some(Ok(GameOwnedStartupSnapshotReport::ThingSetup { entries }))
        }
        GODS_BATTLE_SELECTOR => {
            let report = match game.gods_battle_mgr_mut().decord_from_byte_array(
                source,
                cursor,
                &mut add_log_text,
                &mut put_string_to_file,
            ) {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::GodsBattle(error)));
                }
            };
            add_log_text(b"Initial SI_GODSBATTLE_SETUP...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::GodsBattle(report)))
        }
        _ => None,
    }
}

pub(crate) trait HonorRankPlayerResetContext {
    /// Идёт по ordered `CGame::s_mapPlayer`: nullable pointer пишет diagnostic;
    /// для живого игрока всегда обнуляет days, по mask `2/4` — weeks/months,
    /// затем вызывает `AdjustHonorRank`, если rank-of-nobility ненулевой.
    fn reset_total_honor_eliminate(
        &mut self,
        reset_mask: u32,
        put_string_to_file: &mut dyn FnMut(&str, &[u8]),
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HonorRankStartupReport {
    pub(crate) decoded: HonorRanksDecodeReport,
    pub(crate) reset_mask: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HonorRankStartupError {
    MissingResetMask { offset: usize, available: usize },
    Decode(HonorRanksDecodeError),
}

impl fmt::Display for HonorRankStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResetMask { offset, available } => write!(
                formatter,
                "total honor reset mask обрывается в {offset}: нужно 4, доступно {available}"
            ),
            Self::Decode(error) => error.fmt(formatter),
        }
    }
}

impl Error for HonorRankStartupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingResetMask { .. } => None,
            Self::Decode(error) => Some(error),
        }
    }
}

/// Обрабатывает четыре honor-rank startup snapshots `0x27..0x2A`.
pub(crate) fn dispatch_honor_rank_startup_setup<Context: HonorRankPlayerResetContext>(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
    mut put_string_to_file: impl FnMut(&str, &[u8]),
) -> Result<Option<HonorRankStartupReport>, HonorRankStartupError> {
    let (rank_type, log_text, has_reset_mask) = match selector {
        DAYS_HONOR_RANK_SELECTOR => (
            0,
            b"Initial SI_DAYS_HONOR_ELIMILATE_RANK ...ok!\xA3\xA1".as_slice(),
            false,
        ),
        WEEKS_HONOR_RANK_SELECTOR => (
            1,
            b"Initial SI_WEEKS_HONOR_ELIMILATE_RANK...ok!\xA3\xA1".as_slice(),
            false,
        ),
        MONTHS_HONOR_RANK_SELECTOR => (
            2,
            b"Initial SI_MOHTHS_HONOR_ELIMILATE_RANK...ok!\xA3\xA1".as_slice(),
            false,
        ),
        TOTAL_HONOR_RANK_SELECTOR => (
            3,
            b"Initial SI_TOTAL_HONOR_ELIMILATE_RANK...ok!\xA3\xA1".as_slice(),
            true,
        ),
        _ => return Ok(None),
    };

    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let reset_mask = if has_reset_mask {
        Some(read_honor_reset_mask(source, cursor)?)
    } else {
        None
    };
    let decoded = game
        .honor_ranks_mut()
        .decord_from_byte_array(source, cursor, rank_type, -1)
        .map_err(HonorRankStartupError::Decode)?;

    if let Some(reset_mask) = reset_mask {
        context.reset_total_honor_eliminate(reset_mask, &mut put_string_to_file);
    }
    put_string_to_file("HonorRanksLog", log_text);
    Ok(Some(HonorRankStartupReport {
        decoded,
        reset_mask,
    }))
}

fn read_honor_reset_mask(source: &[u8], cursor: &mut usize) -> Result<u32, HonorRankStartupError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(HonorRankStartupError::MissingResetMask { offset, available });
    };
    *cursor += 4;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("total honor reset mask содержит четыре байта"),
    ))
}

/// Наблюдаемый итог reconnect-ветви Billing `0x6F904`.
#[derive(Debug)]
pub(crate) struct GameBillingClientReplacement {
    pub(crate) previous_client_closed: bool,
    pub(crate) registration: Result<i32, SendMessageError>,
    pub(crate) connected_notice: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcRegionLevelDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for JjcRegionLevelDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "JJC region-level snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for JjcRegionLevelDecodeError {}

fn read_jjc_level_i32(source: &[u8], cursor: &mut usize) -> Result<i32, JjcRegionLevelDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(JjcRegionLevelDecodeError {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .expect("размер JJC region-level scalar уже проверен"),
    ))
}

fn read_script_resource_length(
    source: &[u8],
    cursor: &mut usize,
    resource: &'static str,
) -> Result<i32, GameScriptResourceDecodeError> {
    let bytes = take_script_resource_bytes(source, cursor, 4, "resource length")?;
    let declared = i32::from_le_bytes(
        bytes
            .try_into()
            .expect("длина script resource содержит ровно четыре байта"),
    );
    if declared < 0 {
        return Err(GameScriptResourceDecodeError::NegativeLength { resource, declared });
    }
    Ok(declared)
}

fn take_script_resource_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], GameScriptResourceDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(required) else {
        return Err(GameScriptResourceDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(GameScriptResourceDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

fn read_script_resource_path(source: &[u8], cursor: &mut usize, maximum: usize) -> Vec<u8> {
    let mut path = Vec::new();
    for _ in 0..maximum {
        let Some(&byte) = source.get(*cursor) else {
            return path;
        };
        *cursor += 1;
        if byte == 0 {
            return path;
        }
        path.push(byte);
    }
    Vec::new()
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

    /// Ищет только non-null country region без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Ищет main region с proxy fallback и принимает только nation region.
    fn find_nation_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    fn reset_nation_war_state(&mut self, region: Self::Region, index: i32, state: i32);

    fn set_nation_relive_rects(&mut self, region: Self::Region, rects: [FourNationRect; 5]);

    fn add_log_text(&mut self, text: &'static str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarScheduleSetupError {
    AttackCity(AttackCityDecodeError),
    Village(VillageWarDecodeError),
    Country(CountryWarDecodeError),
    FourNation(FourNationGameDecodeError),
}

impl fmt::Display for WarScheduleSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttackCity(error) => write!(formatter, "AttackCity snapshot: {error}"),
            Self::Village(error) => write!(formatter, "Village snapshot: {error}"),
            Self::Country(error) => write!(formatter, "CountryWar snapshot: {error}"),
            Self::FourNation(error) => write!(formatter, "FourNationWar snapshot: {error}"),
        }
    }
}

impl Error for WarScheduleSetupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AttackCity(error) => Some(error),
            Self::Village(error) => Some(error),
            Self::Country(error) => Some(error),
            Self::FourNation(error) => Some(error),
        }
    }
}

/// Обрабатывает доказанные war startup selectors `0x1B/0x1C/0x1F/0x25`.
pub(crate) fn dispatch_war_startup_setup<Context: WarScheduleSetupContext>(
    selector: i32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    country_war_sys: &mut CountryWarSys,
    four_nation_war_sys: &mut CFourNationWarSys,
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
        0x1f => {
            country_war_sys
                .decord_from_byte_array(payload, cursor)
                .map_err(WarScheduleSetupError::Country)?;
            {
                let mut adapter = CountryWarContextAdapter(context);
                country_war_sys.init_country_region_state(&mut adapter);
            }
            context.add_log_text("Initial SI_COUNTRYWAR...OK!");
            Ok(true)
        }
        0x25 => {
            four_nation_war_sys
                .decord_from_byte_array(payload, cursor)
                .map_err(WarScheduleSetupError::FourNation)?;
            {
                let mut adapter = FourNationWarContextAdapter(context);
                let _ = four_nation_war_sys.init_war_state(&mut adapter);
            }
            context.add_log_text("Initial SI_FOURNATIONWARSYS_SETUP..OK!!");
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

struct CountryWarContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarScheduleSetupContext> CountryWarStartupContext
    for CountryWarContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }
}

struct FourNationWarContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarScheduleSetupContext> FourNationGameStartupContext
    for FourNationWarContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_nation_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_nation_region_then_proxy(region_id)
    }

    fn reset_nation_war_state(&mut self, region: Self::Region, index: i32, state: i32) {
        self.0.reset_nation_war_state(region, index, state);
    }

    fn set_nation_relive_rects(&mut self, region: Self::Region, rects: [FourNationRect; 5]) {
        self.0.set_nation_relive_rects(region, rects);
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
