//! Диспетчер сообщений WorldServer, BillingServer и ресурсов запуска.
//!
//! Источник: `gameserver.exe`, `GameServer.pdb` и исходный владелец
//! `appserver/message/servermessage.cpp`. Реализованные ветви сохраняют точное
//! чтение селекторов и полей, порядок применения ресурсов, число и порядок
//! вызовов генератора случайных чисел, жизненный цикл соединений и игроков, а
//! также последовательность client, World и Billing-отправок.
//!
//! Синхронно выполненные мутации и отправки диагностируются через `tracing` в
//! месте возникновения. Диспетчер возвращает только успех либо типизированную
//! ошибку; списки кадров игроков, результатов порождения и деревья startup-
//! отчётов до верхнего цикла не передаются. Декодирующие результаты сохраняются
//! локально только там, где следующий шаг использует их для настройки владельца.
//!
//! Typed dispatcher покрывает полный достигнутый switch `OnServerMessage`.
//! Внутренние pointer-as-long события повторного подключения заменены owned
//! `GameServerEvent`, сохраняя исходный close/replace/register/control-send
//! порядок. Для обрезанных legacy-буферов сохраняется безопасный отказ без
//! назначения побочных эффектов неопределённому чтению исходной программы;
//! неизвестные коды и селекторы диспетчер не интерпретирует.

use std::sync::Arc;
use thiserror::Error;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityRegionContext, CAttackCitySys,
};
use super::super::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationGameDecodeError, FourNationGameStartupContext, FourNationRect,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarRegionContext,
};
use crate::gameserver::appserver::country::countryhandler::CountryHandlerDecodeError;
use crate::gameserver::appserver::country::countryparam::CountryParamInputBlock;
use crate::gameserver::appserver::country::countrywarsys::{
    CountryWarDecodeError, CountryWarStartupContext, CountryWarSys,
};
use crate::gameserver::appserver::goods::cbattlefairyproperty::BattleFairyComposeDecodeError;
use crate::gameserver::appserver::goods::cgoodsfactory::GoodsFactoryDecodeError;
use crate::gameserver::appserver::legacycodec::LegacyReader;
use crate::gameserver::appserver::proxyserverregion::{CProxyServerRegion, ProxyRegionDecodeError};
use crate::gameserver::appserver::script::variablelist::GameVariableSnapshotError;
use crate::gameserver::appserver::servercityregion::{
    CServerCityRegion, CityRegionDecodeContext, CityRegionDecodeError,
};
use crate::gameserver::appserver::servercountryregion::{
    CServerCountryRegion, CountryRegionDecodeContext, CountryRegionDecodeError,
};
use crate::gameserver::appserver::servergodsbattleregion::{
    CServerGodsBattleRegion, GodsBattleTopTenDecodeError,
};
use crate::gameserver::appserver::servernationregion::ServerNationRegion;
use crate::gameserver::appserver::serverregion::{
    CServerRegion, ServerRegionDecodeError, ServerRegionNpcSetup, ServerRegionSetupDecodeError,
};
use crate::gameserver::appserver::servervillageregion::CServerVillageRegion;
use crate::gameserver::appserver::serverwarregion::WarRegionDecodeError;
use crate::gameserver::appserver::skills::skillfactory::SkillFactoryDecodeError;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GameNetworkInitializationError, GodsBattleNpcSpawnContext,
    ServerRegionOwner,
    colored_player_notice_message, format_legacy_text_fields,
};
use crate::gameserver::gameserver::honorranks::HonorRanksDecodeError;
use crate::gameserver::gameserver::playerranks::PlayerRanksDecodeError;
use crate::nets::netserver::message::{CMessage, GameServerAroundRuntime, SendMessageError};
use crate::nets::netserver::mynetclient::CMyNetClient;
use crate::public::ciqing::{CiQingDecodeError, CiQingSerializationBlock};
use crate::public::dakongxiangqian::DaKongDecodeError;
use crate::public::dupliregionsetup::DupliRegionDecodeError;
use crate::public::equipmentcomposelist::EquipmentComposeDecodeError;
use crate::public::mystringtable::MyStringTableDecodeError;
use crate::public::taozhuangsetup::{TaoZhuangDecodeError, TaoZhuangSerializationBlock};
use crate::public::tools::add_game_log_text;
use crate::public::wordsfilter::WordsFilterDecodeError;
use crate::setup::cbattlefairyexpconfig::BattleFairyExpDecodeError;
use crate::setup::changebody::ChangeBodyDecodeError;
use crate::setup::contributesetup::ContributeSetupDecodeError;
use crate::setup::emotion::EmotionDecodeError;
use crate::setup::globesetup::GlobeSetupDecodeError;
use crate::setup::gmlist::GmListDecodeError;
use crate::setup::godsbattleconf::GodsBattleDecodeError;
use crate::setup::goodsdestructionconfig::GoodsDestroyDecodeError;
use crate::setup::hitlevelsetup::HitLevelDecodeError;
use crate::setup::honorelimilateconfig::HonorEliminateDecodeError;
use crate::setup::incrementshoplist::IncrementShopDecodeError;
use crate::setup::leitingsetup::ThingSetupCodecError;
use crate::setup::lingbao::LingBaoDecodeError;
use crate::setup::logsystem::LogSystemDecodeError;
use crate::setup::monsterlist::MonsterListDecodeError;
use crate::setup::newskillmonsterlist::NewSkillMonsterDecodeError;
use crate::setup::playerlist::PlayerListDecodeError;
use crate::setup::preciousboxconf::PreciousBoxDecodeError;
use crate::setup::prisonconf::PrisonConfDecodeError;
use crate::setup::questsystem::QuestSystemDecodeError;
use crate::setup::regionsetup::RegionSetupDecodeError;
use crate::setup::synthesis::SynthesisDecodeError;
use crate::setup::tradelist::TradeListDecodeError;

const BILLING_REGISTRATION: i32 = 0x000E_F101;
const WORLD_SERVER_CLOSED: i32 = 0x0006_F901;
const BILLING_SERVER_CLOSED: i32 = 0x0006_F903;
const SERVER_STARTUP_MESSAGE: i32 = 0x0007_F801;
const WORLD_REGION_CHANGE_RESPONSE: i32 = 0x0007_F802;
const WORLD_PLAYER_SAVE_REQUEST: i32 = 0x0007_F803;
const WORLD_PLAYER_NOTICE_RESPONSE: i32 = 0x0007_F804;
const GENERAL_VARIABLE_UPDATE_RESPONSE: i32 = 0x0007_F805;
const MURDERER_UPDATE_RESPONSE: i32 = 0x0007_F806;
const WORLD_PLAYER_DATA_REQUEST: i32 = 0x0007_F808;
const RUNTIME_SPAWN_RESPONSE: i32 = 0x0007_F80A;
const PLAYER_COUNT_IF_WORLD_CONNECTED_MESSAGE: i32 = 0x0007_F809;
const PLAYER_COUNT_MESSAGE: i32 = 0x0007_F80B;
const PLAYER_COUNT_IF_WORLD_CONNECTED_RESPONSE: i32 = 0x0005_FA0A;
const PLAYER_COUNT_RESPONSE: i32 = 0x0005_FA0C;
const WORLD_PLAYER_DATA_RESPONSE: i32 = 0x0005_FA09;
const WORLD_PLAYER_SAVE_RESPONSE: i32 = 0x0005_FA03;
const GODS_BATTLE_TOP_TEN_RESPONSE: i32 = 0x0007_F80F;
const GODS_BATTLE_XYD_RESPONSE: i32 = 0x0007_F80E;
const GODS_BATTLE_TOP_TEN_CLIENT: i32 = 0x000B_F740;
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
const ATTACK_CITY_SELECTOR: i32 = 0x1b;
const VILLAGE_WAR_SELECTOR: i32 = 0x1c;
const PRISON_CONF_SELECTOR: i32 = 0x1d;
const PRECIOUS_BOX_CONF_SELECTOR: i32 = 0x1e;
const COUNTRY_WAR_SELECTOR: i32 = 0x1f;
const FAIRY_EXP_SELECTOR: i32 = 0x20;
const SYNTHESIS_SELECTOR: i32 = 0x21;
const NEW_SKILL_MONSTER_SELECTOR: i32 = 0x22;
const GOODS_DESTROY_SELECTOR: i32 = 0x23;
const CHANGE_BODY_SELECTOR: i32 = 0x24;
const FOUR_NATION_WAR_SELECTOR: i32 = 0x25;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameStringTableSource {
    Startup,
    RuntimeRefresh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameBattleFairyStartupError {
    FairyExp(BattleFairyExpDecodeError),
    BattleFairyExp(BattleFairyExpDecodeError),
    Combine(BattleFairyComposeDecodeError),
    EquipmentCompose(EquipmentComposeDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCombatRegistryStartupError {
    Goods(GoodsFactoryDecodeError),
    Monsters(MonsterListDecodeError),
    Skills(SkillFactoryDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerEconomyStartupError {
    PlayerTemplates(PlayerListDecodeError),
    TradeList(TradeListDecodeError),
    IncrementShop(IncrementShopDecodeError),
    ContributionItems(ContributeSetupDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRuntimeConfigurationStartupError {
    GlobeSetup(GlobeSetupDecodeError),
    LogSystem(LogSystemDecodeError),
    GmList(GmListDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerRuleStartupError {
    IdIndex(GameIdIndexDecodeError),
    HitLevel(HitLevelDecodeError),
    Emotion(EmotionDecodeError),
    QuestSystem(QuestSystemDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameCountryStateStartupError {
    Parameters(CountryParamInputBlock),
    Countries(CountryHandlerDecodeError),
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

#[derive(Clone, Debug)]
pub(crate) enum GameEnvironmentConfigurationStartupError {
    Prison(PrisonConfDecodeError),
    PreciousBoxes(Arc<PreciousBoxDecodeError>),
}

impl PartialEq for GameEnvironmentConfigurationStartupError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Prison(left), Self::Prison(right)) => left == right,
            (Self::PreciousBoxes(left), Self::PreciousBoxes(right)) => {
                precious_box_decode_errors_equal(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for GameEnvironmentConfigurationStartupError {}

fn precious_box_decode_errors_equal(
    left: &PreciousBoxDecodeError,
    right: &PreciousBoxDecodeError,
) -> bool {
    match (left, right) {
        (
            PreciousBoxDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            PreciousBoxDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (
            PreciousBoxDecodeError::Allocation {
                field: left_field, ..
            },
            PreciousBoxDecodeError::Allocation {
                field: right_field, ..
            },
        ) => left_field == right_field,
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub(crate) enum GameMutationRulesStartupError {
    Synthesis(Arc<SynthesisDecodeError>),
    NewSkillMonsters(Arc<NewSkillMonsterDecodeError>),
    GoodsDestruction(Arc<GoodsDestroyDecodeError>),
    ChangeBody(Arc<ChangeBodyDecodeError>),
}

impl PartialEq for GameMutationRulesStartupError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Synthesis(left), Self::Synthesis(right)) => {
                synthesis_decode_errors_equal(left, right)
            }
            (Self::NewSkillMonsters(left), Self::NewSkillMonsters(right)) => {
                new_skill_monster_decode_errors_equal(left, right)
            }
            (Self::GoodsDestruction(left), Self::GoodsDestruction(right)) => {
                goods_destroy_decode_errors_equal(left, right)
            }
            (Self::ChangeBody(left), Self::ChangeBody(right)) => {
                change_body_decode_errors_equal(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for GameMutationRulesStartupError {}

fn synthesis_decode_errors_equal(
    left: &SynthesisDecodeError,
    right: &SynthesisDecodeError,
) -> bool {
    match (left, right) {
        (
            SynthesisDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            SynthesisDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (
            SynthesisDecodeError::UnterminatedString {
                offset: left_offset,
                available: left_available,
            },
            SynthesisDecodeError::UnterminatedString {
                offset: right_offset,
                available: right_available,
            },
        ) => left_offset == right_offset && left_available == right_available,
        (
            SynthesisDecodeError::Allocation {
                field: left_field, ..
            },
            SynthesisDecodeError::Allocation {
                field: right_field, ..
            },
        ) => left_field == right_field,
        _ => false,
    }
}

fn new_skill_monster_decode_errors_equal(
    left: &NewSkillMonsterDecodeError,
    right: &NewSkillMonsterDecodeError,
) -> bool {
    match (left, right) {
        (
            NewSkillMonsterDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            NewSkillMonsterDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (
            NewSkillMonsterDecodeError::UnterminatedString {
                offset: left_offset,
                available: left_available,
            },
            NewSkillMonsterDecodeError::UnterminatedString {
                offset: right_offset,
                available: right_available,
            },
        ) => left_offset == right_offset && left_available == right_available,
        (
            NewSkillMonsterDecodeError::Allocation {
                skill_id: left_skill_id,
                ..
            },
            NewSkillMonsterDecodeError::Allocation {
                skill_id: right_skill_id,
                ..
            },
        ) => left_skill_id == right_skill_id,
        _ => false,
    }
}

fn goods_destroy_decode_errors_equal(
    left: &GoodsDestroyDecodeError,
    right: &GoodsDestroyDecodeError,
) -> bool {
    match (left, right) {
        (
            GoodsDestroyDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            GoodsDestroyDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (
            GoodsDestroyDecodeError::UnterminatedString {
                offset: left_offset,
                available: left_available,
            },
            GoodsDestroyDecodeError::UnterminatedString {
                offset: right_offset,
                available: right_available,
            },
        ) => left_offset == right_offset && left_available == right_available,
        (
            GoodsDestroyDecodeError::Allocation {
                list: left_list, ..
            },
            GoodsDestroyDecodeError::Allocation {
                list: right_list, ..
            },
        ) => left_list == right_list,
        _ => false,
    }
}

fn change_body_decode_errors_equal(
    left: &ChangeBodyDecodeError,
    right: &ChangeBodyDecodeError,
) -> bool {
    match (left, right) {
        (
            ChangeBodyDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            ChangeBodyDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (ChangeBodyDecodeError::Allocation(_), ChangeBodyDecodeError::Allocation(_)) => true,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameLookupFilterStartupError {
    DaKong(DaKongDecodeError),
    WordsFilter(WordsFilterDecodeError),
    JjcRegionLevels(JjcRegionLevelDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameEquipmentEnhancementStartupError {
    TaoZhuangDecode(TaoZhuangDecodeError),
    TaoZhuangSerialize(TaoZhuangSerializationBlock),
    CiQingDecode(CiQingDecodeError),
    LingBaoDecode(LingBaoDecodeError),
    CiQingSerialize(CiQingSerializationBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameWorldEventStartupError {
    Things(ThingSetupCodecError),
    GodsBattle(GodsBattleDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameHonorStartupError {
    EliminateConfiguration(HonorEliminateDecodeError),
    Ranks(HonorRankStartupError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameScriptStartupError {
    Resource(GameScriptResourceDecodeError),
    GeneralVariables(GameVariableSnapshotError),
}

#[derive(Clone, Debug)]
pub(crate) enum GamePlayerRanksStartupError {
    OwnerUnavailable { selector: i32 },
    Decode(Arc<PlayerRanksDecodeError>),
}

impl PartialEq for GamePlayerRanksStartupError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::OwnerUnavailable { selector: left },
                Self::OwnerUnavailable { selector: right },
            ) => left == right,
            (Self::Decode(left), Self::Decode(right)) => {
                player_ranks_decode_errors_equal(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for GamePlayerRanksStartupError {}

fn player_ranks_decode_errors_equal(
    left: &PlayerRanksDecodeError,
    right: &PlayerRanksDecodeError,
) -> bool {
    match (left, right) {
        (
            PlayerRanksDecodeError::UnexpectedEnd {
                offset: left_offset,
                needed: left_needed,
                available: left_available,
            },
            PlayerRanksDecodeError::UnexpectedEnd {
                offset: right_offset,
                needed: right_needed,
                available: right_available,
            },
        ) => {
            left_offset == right_offset
                && left_needed == right_needed
                && left_available == right_available
        }
        (
            PlayerRanksDecodeError::MissingStringTerminator {
                offset: left_offset,
            },
            PlayerRanksDecodeError::MissingStringTerminator {
                offset: right_offset,
            },
        ) => left_offset == right_offset,
        (PlayerRanksDecodeError::Allocation(_), PlayerRanksDecodeError::Allocation(_)) => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameServerMessageError {
    StartupSelector(GameClientServerStartPayloadError),
    StringTable(MyStringTableDecodeError),
    GodsBattleTopTen(GodsBattleTopTenDecodeError),
    GodsBattleXydUnexpectedEnd { field: &'static str },
    RegionChangeUnexpectedEnd { field: &'static str },
    GeneralVariableUpdateUnexpectedEnd { field: &'static str },
    WorldPlayerNoticeUnexpectedEnd { field: &'static str },
    MurdererUpdateUnexpectedEnd { field: &'static str },
    RuntimeSpawnUnexpectedEnd { field: &'static str },
    BattleFairyStartup(GameBattleFairyStartupError),
    CombatRegistryStartup(GameCombatRegistryStartupError),
    PlayerEconomyStartup(GamePlayerEconomyStartupError),
    RuntimeConfigurationStartup(GameRuntimeConfigurationStartupError),
    PlayerRuleStartup(GamePlayerRuleStartupError),
    CountryStateStartup(GameCountryStateStartupError),
    SpatialStartup(GameSpatialStartupError),
    EnvironmentConfigurationStartup(GameEnvironmentConfigurationStartupError),
    MutationRulesStartup(GameMutationRulesStartupError),
    LookupFilterStartup(GameLookupFilterStartupError),
    EquipmentEnhancementStartup(GameEquipmentEnhancementStartupError),
    WorldEventStartup(GameWorldEventStartupError),
    PlayerRanksStartup(GamePlayerRanksStartupError),
    HonorStartup(GameHonorStartupError),
    ScriptStartup(GameScriptStartupError),
    InitialRegionStartup(InitialRegionStartupError),
    WarStartup(WarScheduleSetupError),
}

/// Маршрутизирует уже материализованные ветви `OnServerMessage`: общий
/// startup selector читается один раз, неизвестные selector-ы не двигают
/// cursor, а обе language-table точки входят в один `CGame` owner.
pub(crate) fn dispatch_server_message<Context>(
    message: &mut CMessage,
    game: &mut CGame,
    script_context: &mut Context,
    mut now_ms: impl FnMut(&mut Context) -> u32,
) -> Option<Result<(), GameServerMessageError>>
where
    Context: GameMainLoopRuntime,
{
    if message.message_type() == WORLD_SERVER_CLOSED {
        add_game_log_text(b"WorldServer closed");
        let reconnect = game.schedule_world_reconnect_task();
        game.jjc_on_world_closed();
        tracing::warn!(?reconnect, "соединение с WorldServer закрыто");
        return Some(Ok(()));
    }
    if message.message_type() == BILLING_SERVER_CLOSED {
        add_game_log_text(
            b"[WARNNING] BillingServer closed : Billing Function CANNOT Start NOW!!!!",
        );
        let reconnect = game.schedule_billing_reconnect_task();
        tracing::warn!(?reconnect, "соединение с BillingServer закрыто");
        return Some(Ok(()));
    }
    if message.message_type() == WORLD_PLAYER_SAVE_REQUEST {
        let id_index = game.id_index();
        let advertised_players = game.player_count();
        for player_id in game.ordered_player_ids() {
            let Some(player) = game.find_player(player_id) else {
                continue;
            };
            let mut snapshot = Vec::new();
            let encoded = game.encode_player_game_save(player, &mut snapshot, script_context);
            let snapshot_bytes = snapshot.len();

            let mut frame = CMessage::new(WORLD_PLAYER_SAVE_RESPONSE);
            frame.add_byte(id_index);
            frame.add_long(1);
            frame.add_long(1);
            frame.add_long(player_id);
            frame.base_mut().add(&snapshot);
            frame.base_mut().update();
            let delivery = frame.send(game, false);
            tracing::trace!(player_id, encoded, snapshot_bytes, ?delivery, "снимок игрока отправлен World при сохранении");
        }

        let mut terminal = CMessage::new(WORLD_PLAYER_SAVE_RESPONSE);
        terminal.add_byte(u8::MAX);
        terminal.add_long(advertised_players as i32);
        let terminal_delivery = terminal.send(game, false);
        tracing::trace!(id_index, advertised_players, ?terminal_delivery, "сохранение игроков завершено");
        return Some(Ok(()));
    }
    if message.message_type() == WORLD_REGION_CHANGE_RESPONSE {
        let Some(accepted) = message.base_mut().get_char() else {
            return Some(Err(GameServerMessageError::RegionChangeUnexpectedEnd {
                field: "accepted",
            }));
        };
        let Some(player_id) = message.base_mut().get_long() else {
            return Some(Err(GameServerMessageError::RegionChangeUnexpectedEnd {
                field: "player id",
            }));
        };
        if game.find_player(player_id).is_none() {
            tracing::warn!(player_id, accepted, "ответ смены региона получен для отсутствующего игрока");
            return Some(Ok(()));
        }
        if accepted == 0 {
            let player = game
                .find_player_mut(player_id)
                .expect("region-change player проверен до rejection");
            let previous_changing_server = player.in_changing_server();
            let previous_changing_region = player.in_changing_region();
            player.cancel_server_region_change();
            tracing::debug!(player_id, previous_changing_server, previous_changing_region, "смена региона отклонена World");
            return Some(Ok(()));
        }
        let address = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("0x100 не достигает zero-size GetStr boundary");
        let Some(port) = message.base_mut().get_long().map(|value| value as u32) else {
            return Some(Err(GameServerMessageError::RegionChangeUnexpectedEnd {
                field: "target port",
            }));
        };
        let (captain, team_id) = {
            let player = game
                .find_player(player_id)
                .expect("region-change player проверен до success wire");
            (player.is_team_captain(), player.team_id())
        };
        let mut response = CMessage::new(0x000b_f506);
        response.base_mut().add(&address);
        response.add_byte(0);
        response.add_ulong(port);
        response.add_byte(u8::from(captain));
        response.add_long(team_id);
        let client_delivery = response.send_to_player(game.net_server(), player_id);
        game.on_player_lost(player_id, script_context);
        tracing::trace!(player_id, address_bytes = address.len(), port, captain, team_id, client_delivery, "смена региона подтверждена World");
        return Some(Ok(()));
    }
    if message.message_type() == RUNTIME_SPAWN_RESPONSE {
        let Some(kind) = message.base_mut().get_char() else {
            return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                field: "spawn kind",
            }));
        };
        if kind != 0 && kind != 1 {
            tracing::debug!(kind, "неизвестный вид runtime-создания проигнорирован");
            return Some(Ok(()));
        }
        let Some(region_id) = message.base_mut().get_long() else {
            return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                field: "region id",
            }));
        };

        if kind == 0 {
            let name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("0x100 не достигает zero-size GetStr boundary");
            let Some(requested_count) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster count",
                }));
            };
            let Some(left) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster range left",
                }));
            };
            let Some(top) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster range top",
                }));
            };
            let Some(right) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster range right",
                }));
            };
            let Some(bottom) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster range bottom",
                }));
            };
            let Some(has_script) = message.base_mut().get_char() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: "monster script marker",
                }));
            };
            let script = if has_script != 0 {
                message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .expect("0x100 не достигает zero-size GetStr boundary")
            } else {
                Vec::new()
            };
            let Some(mut owner) = game.take_region_owner(region_id) else {
                tracing::warn!(kind, region_id, "runtime-создание монстров пропущено: регион отсутствует");
                return Some(Ok(()));
            };
            let range_width = right.wrapping_sub(left);
            let range_height = bottom.wrapping_sub(top);
            // Exact case вызывает GetRandomPosInRange один раз до проверки
            // count и отбрасывает результат, сохраняя расход RNG.
            let initial_position = owner.base().region.get_random_pos_in_range(
                left,
                top,
                range_width,
                range_height,
                script_context,
            );
            let (area_width, area_height) = game.area_dimensions();
            let mut completed = 0_i32;
            if initial_position.is_ok() {
                let mut remaining = requested_count;
                while remaining > 0 {
                    let position = match owner.base().region.get_random_pos_in_range(
                        left,
                        top,
                        range_width,
                        range_height,
                        script_context,
                    ) {
                        Ok(position) => position,
                        Err(block) => {
                            tracing::warn!(kind, region_id, requested_count, completed, ?block, "runtime-создание монстров остановлено: позиция недоступна");
                            break;
                        }
                    };
                    match game.find_monster_property_by_origin_name(&name).cloned() {
                        None => tracing::warn!(kind, region_id, ?position, name_bytes = name.len(), "runtime-создание монстра пропущено: свойства отсутствуют"),
                        Some(property) => match owner.base_mut().add_monster(
                            &property,
                            position.x,
                            position.y,
                            -1,
                            true,
                            false,
                            now_ms(script_context),
                            area_width,
                            area_height,
                            game.skill_factory(),
                            script_context,
                        ) {
                            Ok(monster_id) => {
                                owner
                                    .base_mut()
                                    .find_monster_by_id_mut(monster_id)
                                    .expect("успешный AddMonster публикует owned monster")
                                    .set_script_file(&script);
                                tracing::trace!(kind, region_id, ?position, monster_id, "runtime-монстр создан");
                            }
                            Err(block) => {
                                tracing::warn!(kind, region_id, ?position, ?block, "runtime-монстр не создан");
                            }
                        },
                    }
                    completed = completed.wrapping_add(1);
                    remaining = remaining.wrapping_sub(1);
                }
            }
            game.restore_region_owner(owner);
            tracing::trace!(kind, region_id, requested_count, completed, ?initial_position, name_bytes = name.len(), script_bytes = script.len(), "runtime-создание монстров обработано");
            return Some(Ok(()));
        }

        let name = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("0x100 не достигает zero-size GetStr boundary");
        let fields = [
            "NPC picture id",
            "NPC count",
            "NPC range left",
            "NPC range top",
            "NPC range right",
            "NPC range bottom",
            "NPC direction",
        ];
        let mut values = [0_i32; 7];
        for (index, value) in values.iter_mut().enumerate() {
            let Some(decoded) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                    field: fields[index],
                }));
            };
            *value = decoded;
        }
        let Some(has_script) = message.base_mut().get_char() else {
            return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                field: "NPC script marker",
            }));
        };
        let script = if has_script != 0 {
            message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("0x100 не достигает zero-size GetStr boundary")
        } else {
            Vec::new()
        };
        let Some(time) = message.base_mut().get_long() else {
            return Some(Err(GameServerMessageError::RuntimeSpawnUnexpectedEnd {
                field: "NPC lifetime",
            }));
        };
        let setup = ServerRegionNpcSetup {
            // Exact tagNpc ctor не инициализирует этот прочитанный AddNpc-ом
            // byte. Safe Rust не воспроизводит stack garbage и выбирает zero.
            show_list: false,
            picture_id: values[0],
            count: values[1],
            left: values[2],
            top: values[3],
            right: values[4],
            bottom: values[5],
            direction: values[6],
            time,
            name,
            script,
        };
        let Some(mut owner) = game.take_region_owner(region_id) else {
            tracing::warn!(kind, region_id, "runtime-создание NPC пропущено: регион отсутствует");
            return Some(Ok(()));
        };
        let spawn = game.add_region_npc_with_clock(
            &mut owner,
            &setup,
            true,
            true,
            script_context,
            |context| now_ms(context),
        );
        game.restore_region_owner(owner);
        tracing::trace!(kind, region_id, ?setup, ?spawn, "runtime-создание NPC обработано");
        return Some(Ok(()));
    }
    if message.message_type() == MURDERER_UPDATE_RESPONSE {
        let mut ignored_prefix = [0_i32; 3];
        for (index, field) in ignored_prefix.iter_mut().enumerate() {
            let Some(value) = message.base_mut().get_long() else {
                const FIELDS: [&str; 3] =
                    ["ignored prefix 0", "ignored prefix 1", "ignored prefix 2"];
                return Some(Err(GameServerMessageError::MurdererUpdateUnexpectedEnd {
                    field: FIELDS[index],
                }));
            };
            *field = value;
        }
        let Some(player_id) = message.base_mut().get_long() else {
            return Some(Err(GameServerMessageError::MurdererUpdateUnexpectedEnd {
                field: "player id",
            }));
        };
        if game.find_player(player_id).is_none() {
            message.set_message_type(0x0005_FA06);
            let world_delivery = message.send(game, false);
            tracing::warn!(?ignored_prefix, player_id, ?world_delivery, "обновление убийцы возвращено World: игрок отсутствует");
        } else {
            let pk_count_per_kill = game.globe_setup().pk_count_per_kill();
            let (pk_count, kill_count) = game
                .find_player_mut(player_id)
                .expect("player existence проверен перед synchronous mutation")
                .apply_confirmed_kill(pk_count_per_kill, || now_ms(script_context));
            let mut around = CMessage::new(0x000B_F70E);
            around.add_long(player_id);
            around.base_mut().add_word(pk_count);
            around.add_ulong(kill_count);
            let around_runtime = GameServerAroundRuntime::new(
                game,
                game.session_factory(),
                game.globe_setup().area_width(),
                game.globe_setup().area_height(),
            );
            let around_delivery = around_runtime.and_then(|around_runtime| {
                let player = game.find_player(player_id)?;
                let region = player
                    .server_region_id()
                    .and_then(|region_id| game.find_region(region_id))
                    .map(ServerRegionOwner::base);
                Some(around.send_to_around(region, player.shape(), None, &around_runtime))
            });
            tracing::trace!(?ignored_prefix, player_id, pk_count, kill_count, ?around_delivery, "счётчики убийцы обновлены");
        }
        return Some(Ok(()));
    }
    if message.message_type() == WORLD_PLAYER_NOTICE_RESPONSE {
        let Some(target_or_mode) = message.base_mut().get_long() else {
            return Some(Err(
                GameServerMessageError::WorldPlayerNoticeUnexpectedEnd {
                    field: "target player or offline marker",
                },
            ));
        };
        if target_or_mode == 0 {
            let target_name = message
                .base_mut()
                .get_str_bytes(0x18)
                .expect("0x18 не достигает zero-size GetStr boundary");
            let Some(source_player_id) = message.base_mut().get_long() else {
                return Some(Err(
                    GameServerMessageError::WorldPlayerNoticeUnexpectedEnd {
                        field: "offline source player id",
                    },
                ));
            };
            let text =
                format_legacy_text_fields(game.get_string_by_id(b"GS0332 "), &[&target_name], 0xff);
            let delivery = colored_player_notice_message(0xffff_ffff, 0, &text)
                .send_to_player(game.net_server(), source_player_id);
            tracing::trace!(source_player_id, target_name_bytes = target_name.len(), text_bytes = text.len(), delivery, "уведомление об отсутствующей цели отправлено");
        } else {
            let text = message
                .base_mut()
                .get_str_bytes(0x400)
                .expect("0x400 не достигает zero-size GetStr boundary");
            let Some(color) = message.base_mut().get_long() else {
                return Some(Err(
                    GameServerMessageError::WorldPlayerNoticeUnexpectedEnd {
                        field: "online notice color",
                    },
                ));
            };
            let Some(message_type) = message.base_mut().get_long() else {
                return Some(Err(
                    GameServerMessageError::WorldPlayerNoticeUnexpectedEnd {
                        field: "online notice type",
                    },
                ));
            };
            let source_name = message
                .base_mut()
                .get_str_bytes(0x18)
                .expect("0x18 не достигает zero-size GetStr boundary");
            let mut formatted = if source_name.is_empty() {
                text.clone()
            } else {
                let mut formatted = Vec::with_capacity(source_name.len() + 1 + text.len());
                formatted.extend_from_slice(&source_name);
                formatted.push(b':');
                formatted.extend_from_slice(&text);
                formatted
            };
            formatted.truncate(0x3ff);
            let delivery =
                colored_player_notice_message(color as u32, message_type as u32, &formatted)
                    .send_to_player(game.net_server(), target_or_mode);
            tracing::trace!(target_player_id = target_or_mode, source_name_bytes = source_name.len(), text_bytes = text.len(), color, message_type, formatted_bytes = formatted.len(), delivery, "уведомление World отправлено игроку");
        }
        return Some(Ok(()));
    }
    if message.message_type() == GENERAL_VARIABLE_UPDATE_RESPONSE {
        let Some(value_tag) = message.base_mut().get_long() else {
            return Some(Err(
                GameServerMessageError::GeneralVariableUpdateUnexpectedEnd { field: "value tag" },
            ));
        };
        let name = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("0x100 не достигает zero-size GetStr boundary");
        match value_tag {
            1 => {
                let Some(value) = message.base_mut().get_long() else {
                    return Some(Err(
                        GameServerMessageError::GeneralVariableUpdateUnexpectedEnd {
                            field: "integer value",
                        },
                    ));
                };
                game.set_general_variable_integer(&name, value);
                tracing::trace!(value_tag, name_bytes = name.len(), value, "целая общая переменная обновлена");
            }
            3 => {
                let value = message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .expect("0x100 не достигает zero-size GetStr boundary");
                game.set_general_variable_string(&name, &value);
                tracing::trace!(value_tag, name_bytes = name.len(), value_bytes = value.len(), "строковая общая переменная обновлена");
            }
            _ => tracing::debug!(value_tag, name_bytes = name.len(), "неизвестный вид общей переменной проигнорирован"),
        }
        return Some(Ok(()));
    }
    if message.message_type() == STRING_TABLE_REFRESH_MESSAGE {
        return Some(dispatch_string_table_message(
            message,
            game,
            GameStringTableSource::RuntimeRefresh,
        ));
    }
    if message.message_type() == WORLD_PLAYER_DATA_REQUEST {
        let declared_players = game.player_count();
        let mut begin = CMessage::new(WORLD_PLAYER_DATA_RESPONSE);
        begin.add_byte(0);
        begin.add_long(declared_players as i32);
        let begin_delivery = begin.send(game, false);

        let mut sent_players = 0_u32;
        for player_id in game.ordered_player_ids() {
            let Some(player) = game.find_player(player_id) else {
                continue;
            };
            let mut snapshot = Vec::new();
            let encoded = game.encode_player_game_save(player, &mut snapshot, script_context);
            let client_ip = player.client_ip();
            let snapshot_bytes = snapshot.len();

            let mut frame = CMessage::new(WORLD_PLAYER_DATA_RESPONSE);
            frame.add_byte(1);
            frame.add_long(player_id);
            frame.base_mut().add(&snapshot);
            frame.add_ulong(client_ip);
            frame.base_mut().update();
            let delivery = frame.send(game, false);
            sent_players = sent_players.wrapping_add(1);
            tracing::trace!(player_id, encoded, snapshot_bytes, client_ip, ?delivery, "снимок данных игрока отправлен World");
        }

        let mut finish = CMessage::new(WORLD_PLAYER_DATA_RESPONSE);
        finish.add_byte(2);
        finish.add_long(sent_players as i32);
        let finish_delivery = finish.send(game, false);
        tracing::trace!(declared_players, ?begin_delivery, sent_players, ?finish_delivery, "снимок данных игроков завершён");
        return Some(Ok(()));
    }
    if dispatch_player_count_message(message.message_type(), game).is_some() {
        return Some(Ok(()));
    }
    if message.message_type() == GODS_BATTLE_XYD_RESPONSE {
        let Some(marker) = message.base_mut().get_char() else {
            return Some(Err(GameServerMessageError::GodsBattleXydUnexpectedEnd {
                field: "marker",
            }));
        };
        let apply = if marker == 1 {
            let Some(faction_a) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::GodsBattleXydUnexpectedEnd {
                    field: "faction A XYD",
                }));
            };
            let Some(faction_b) = message.base_mut().get_long() else {
                return Some(Err(GameServerMessageError::GodsBattleXydUnexpectedEnd {
                    field: "faction B XYD",
                }));
            };
            Some(game.apply_gods_battle_xyd(faction_a as u32, faction_b as u32))
        } else {
            None
        };
        tracing::trace!(marker, ?apply, "значения XYD битвы богов обработаны");
        return Some(Ok(()));
    }
    if message.message_type() == GODS_BATTLE_TOP_TEN_RESPONSE {
        let entries = {
            let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
            match game.gods_battle_mgr().decode_top_ten(source, cursor) {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameServerMessageError::GodsBattleTopTen(error)));
                }
            }
        };
        let player_id = game.gods_battle_mgr().pending_top_ten_player_id();
        let mut response = CMessage::new(GODS_BATTLE_TOP_TEN_CLIENT);
        response.add_ulong(entries.len() as u32);
        for entry in &entries {
            response.add_ulong(entry.faction as u32);
            response.base_mut().add(&entry.name);
            response.base_mut().add_byte(0);
            response.add_ulong(entry.szl);
            response.add_ulong(entry.level);
        }
        let delivery = response.send_to_player(game.net_server(), player_id);
        tracing::trace!(player_id, entries = entries.len(), delivery, "десятка битвы богов отправлена игроку");
        return Some(Ok(()));
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
            dispatch_client_server_start(
                consumed_selector,
                message,
                game,
                now_ms(script_context),
            )
            .expect("selector 0x3B проверен outer dispatcher-ом");
            Some(Ok(()))
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
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_battle_fairy_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("battle-fairy selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::BattleFairyStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        GOODS_LIST_SELECTOR | MONSTER_LIST_SELECTOR | SKILL_LIST_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("combat-registry selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_combat_registry_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("combat-registry selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::CombatRegistryStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        PLAYER_LIST_SELECTOR
        | TRADE_LIST_SELECTOR
        | INCREMENT_SHOP_SELECTOR
        | CONTRIBUTE_SETUP_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("player/economy selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_player_economy_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("player/economy selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::PlayerEconomyStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        GLOBE_SETUP_SELECTOR | LOG_SYSTEM_SELECTOR | GM_LIST_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("runtime-configuration selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_runtime_configuration_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
                )
                .expect("runtime-configuration selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::RuntimeConfigurationStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        ID_INDEX_SELECTOR | HIT_LEVEL_SELECTOR | EMOTION_SELECTOR | QUEST_SYSTEM_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("player-rule selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_player_rule_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("player-rule selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::PlayerRuleStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        COUNTRY_PARAM_SELECTOR | COUNTRY_HANDLER_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("country-state selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_country_state_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("country-state selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::CountryStateStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        REGION_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("region selector проверен без изменения cursor");
            let decoded = match dispatch_initial_region_startup(
                consumed_selector,
                message,
                game,
                script_context,
                |name, region_id| tracing::trace!(name = %String::from_utf8_lossy(name), region_id, "регион добавлен в список запуска"),
                |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
            )
            .expect("region selector проверен outer dispatcher-ом")
            .map_err(GameServerMessageError::InitialRegionStartup)
            {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        ATTACK_CITY_SELECTOR
        | VILLAGE_WAR_SELECTOR
        | COUNTRY_WAR_SELECTOR
        | FOUR_NATION_WAR_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("war selector проверен без изменения cursor");
            let mut owners = game.take_war_startup_owners();
            let decoded = {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                dispatch_war_startup_setup(
                    consumed_selector,
                    wire,
                    cursor,
                    &mut owners.attack_city,
                    &mut owners.village,
                    &mut owners.country,
                    &mut owners.four_nation,
                    game,
                    |text| tracing::info!(text, "сообщение загрузки GameServer"),
                )
                .expect("war selector проверен outer dispatcher-ом")
            };
            game.restore_war_startup_owners(owners);
            match decoded {
                Ok(decoded) => { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) },
                Err(error) => Some(Err(GameServerMessageError::WarStartup(error))),
            }
        }
        PROXY_REGION_SELECTOR
        | REGION_RELOAD_SELECTOR
        | REGION_SETUP_SELECTOR
        | DUPLI_REGION_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("spatial selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_spatial_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("spatial selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::SpatialStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        PRISON_CONF_SELECTOR | PRECIOUS_BOX_CONF_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("environment-configuration selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_environment_configuration_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
                )
                .expect("environment-configuration selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::EnvironmentConfigurationStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            tracing::trace!(selector = consumed_selector, ?decoded, "настройки окружения GameServer применены");
            Some(Ok(()))
        }
        SYNTHESIS_SELECTOR
        | NEW_SKILL_MONSTER_SELECTOR
        | GOODS_DESTROY_SELECTOR
        | CHANGE_BODY_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("mutation-rules selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_mutation_rules_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("mutation-rules selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::MutationRulesStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        DA_KONG_SELECTOR | WORDS_FILTER_SELECTOR | JJC_REGION_LEVEL_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("lookup/filter selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_lookup_filter_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("lookup/filter selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::LookupFilterStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        TAO_ZHUANG_SELECTOR | CI_QING_LING_BAO_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("equipment-enhancement selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_equipment_enhancement_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
                )
                .expect("equipment-enhancement selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::EquipmentEnhancementStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        THING_SETUP_SELECTOR | GODS_BATTLE_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("world-event selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_world_event_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
                    |path, text| tracing::info!(path, text = %String::from_utf8_lossy(text), "файловое сообщение загрузки GameServer"),
                )
                .expect("world-event selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::WorldEventStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        PLAYER_RANKS_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("player-ranks selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_player_ranks_startup(consumed_selector, wire, cursor, game)
                    .expect("player-ranks selector проверен outer dispatcher-ом")
                    .map_err(GameServerMessageError::PlayerRanksStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            tracing::trace!(selector = consumed_selector, ?decoded, "рейтинги игроков применены");
            Some(Ok(()))
        }
        HONOR_ELIMINATE_SELECTOR
        | DAYS_HONOR_RANK_SELECTOR
        | WEEKS_HONOR_RANK_SELECTOR
        | MONTHS_HONOR_RANK_SELECTOR
        | TOTAL_HONOR_RANK_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("honor selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_honor_startup(
                    consumed_selector,
                    wire,
                    cursor,
                    game,
                    |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"),
                    |path, text| tracing::info!(path, text = %String::from_utf8_lossy(text), "файловое сообщение загрузки GameServer"),
                )
                .expect("honor selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::HonorStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        FUNCTION_LIST_SELECTOR
        | VARIABLE_LIST_SELECTOR
        | GENERAL_VARIABLE_SELECTOR
        | SCRIPT_FILE_SELECTOR => {
            let consumed_selector = message
                .base_mut()
                .get_long()
                .expect("script-resource selector проверен без изменения cursor");
            let decoded = match {
                let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                decode_script_startup(consumed_selector, wire, cursor, game, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
                .expect("script-resource selector проверен outer dispatcher-ом")
                .map_err(GameServerMessageError::ScriptStartup)
            } {
                Ok(decoded) => decoded,
                Err(error) => return Some(Err(error)),
            };
            { tracing::trace!(selector = consumed_selector, ?decoded, "ресурс запуска GameServer применён"); Some(Ok(())) }
        }
        _ => None,
    }
}

fn decode_script_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<(), GameScriptStartupError>> {
    match selector {
        FUNCTION_LIST_SELECTOR | VARIABLE_LIST_SELECTOR => {
            let resource = if selector == FUNCTION_LIST_SELECTOR {
                "FunctionList"
            } else {
                "VariableList"
            };
            let declared_length = match read_script_resource_length(source, cursor, resource) {
                Ok(length) => length,
                Err(error) => return Some(Err(GameScriptStartupError::Resource(error))),
            };
            let data = match take_script_resource_bytes(
                source,
                cursor,
                declared_length as usize,
                if selector == FUNCTION_LIST_SELECTOR {
                    "FunctionList data"
                } else {
                    "VariableList data"
                },
            ) {
                Ok(data) => data.to_vec(),
                Err(error) => return Some(Err(GameScriptStartupError::Resource(error))),
            };
            if selector == FUNCTION_LIST_SELECTOR {
                let publication = game.set_function_file_data(data);
                add_log_text(b"FunctionList...OK!");
                tracing::trace!(declared_length, ?publication, "список функций загружен");
                Some(Ok(()))
            } else {
                let publication = game.set_variable_file_data(data);
                add_log_text(b"VariableList...OK!");
                tracing::trace!(declared_length, ?publication, "список переменных загружен");
                Some(Ok(()))
            }
        }
        GENERAL_VARIABLE_SELECTOR => {
            let start_offset = *cursor;
            match game.set_general_variable_file_data(source, *cursor) {
                Ok(()) => {}
                Err(error) => {
                    return Some(Err(GameScriptStartupError::GeneralVariables(error)));
                }
            }
            add_log_text(b"GeneralVariableList...OK!");
            tracing::trace!(start_offset, "общие переменные загружены");
            Some(Ok(()))
        }
        SCRIPT_FILE_SELECTOR => {
            let path = read_script_resource_path(source, cursor, 0x104);
            let declared_length = match read_script_resource_length(source, cursor, "ScriptFile") {
                Ok(length) => length,
                Err(error) => return Some(Err(GameScriptStartupError::Resource(error))),
            };
            let data = match take_script_resource_bytes(
                source,
                cursor,
                declared_length as usize,
                "ScriptFile data",
            ) {
                Ok(data) => data.to_vec(),
                Err(error) => return Some(Err(GameScriptStartupError::Resource(error))),
            };
            let path_bytes = path.len();
            let replaced = game.set_script_file_data(path, data);
            tracing::trace!(path_bytes, declared_length, replaced, "файл сценария загружен");
            Some(Ok(()))
        }
        _ => None,
    }
}

fn decode_honor_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
    mut put_string_to_file: impl FnMut(&str, &[u8]),
) -> Option<Result<(), GameHonorStartupError>> {
    if selector == HONOR_ELIMINATE_SELECTOR {
        let config = game.honor_eliminate_config_mut();
        if let Err(error) = config.decord_from_byte_array(source, cursor) {
            return Some(Err(GameHonorStartupError::EliminateConfiguration(error)));
        }
        let level_difference = config.level_difference;
        let minimum_level = config.minimum_level;
        add_log_text(b"Inital SI_HONOR_ELIMILATE_CONF...ok!");
        put_string_to_file(
            "HonorCompositior",
            b"Inital SI_HONOR_ELIMILATE_CONF...ok\xA3\xA1",
        );
        tracing::trace!(level_difference, minimum_level, "настройки исключения чести загружены");
        return Some(Ok(()));
    }
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
        _ => return None,
    };
    let reset_mask = if has_reset_mask {
        match read_honor_reset_mask(source, cursor) {
            Ok(mask) => Some(mask),
            Err(error) => return Some(Err(GameHonorStartupError::Ranks(error))),
        }
    } else {
        None
    };
    match game
        .honor_ranks_mut()
        .decord_from_byte_array(source, cursor, rank_type, -1)
    {
        Ok(()) => {}
        Err(error) => {
            return Some(Err(GameHonorStartupError::Ranks(
                HonorRankStartupError::Decode(error),
            )));
        }
    }
    if let Some(mask) = reset_mask {
        game.reset_total_honor_eliminate(mask);
    }
    put_string_to_file("HonorRanksLog", log_text);
    tracing::trace!(rank_type, ?reset_mask, "рейтинг чести загружен");
    Some(Ok(()))
}

fn decode_player_ranks_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
) -> Option<Result<(), GamePlayerRanksStartupError>> {
    if selector != PLAYER_RANKS_SELECTOR {
        return None;
    }
    let Some(ranks) = game.player_ranks_mut() else {
        return Some(Err(GamePlayerRanksStartupError::OwnerUnavailable {
            selector,
        }));
    };
    if let Err(error) = ranks.decord_from_byte_array(source, cursor) {
        return Some(Err(GamePlayerRanksStartupError::Decode(Arc::new(error))));
    }
    tracing::trace!(entries = ranks.ranks().len(), "рейтинги игроков загружены");
    Some(Ok(()))
}

fn decode_world_event_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
    mut put_string_to_file: impl FnMut(&str, &[u8]),
) -> Option<Result<(), GameWorldEventStartupError>> {
    match selector {
        THING_SETUP_SELECTOR => {
            if let Err(error) = game
                .thing_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                return Some(Err(GameWorldEventStartupError::Things(error)));
            }
            let entries = game.thing_setup().all_things().len();
            let decoded_line = format!("GS Leiting Decord:line {entries}");
            add_log_text(decoded_line.as_bytes());
            add_log_text(b"Initial Strictest Enforcement...ok!");
            { tracing::trace!(entries, "настройки мировых объектов загружены"); Some(Ok(())) }
        }
        GODS_BATTLE_SELECTOR => {
            match game.gods_battle_mgr_mut().decord_from_byte_array(
                source,
                cursor,
                &mut add_log_text,
                &mut put_string_to_file,
            ) {
                Ok(()) => {}
                Err(error) => return Some(Err(GameWorldEventStartupError::GodsBattle(error))),
            }
            add_log_text(b"Initial SI_GODSBATTLE_SETUP...OK!");
            { tracing::trace!("настройки битвы богов загружены"); Some(Ok(())) }
        }
        _ => None,
    }
}

fn decode_equipment_enhancement_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<(), GameEquipmentEnhancementStartupError>> {
    match selector {
        TAO_ZHUANG_SELECTOR => {
            match game.tao_zhuang_setup_mut().decode_from_byte(source, cursor) {
                Ok(()) => {}
                Err(error) => {
                    return Some(Err(GameEquipmentEnhancementStartupError::TaoZhuangDecode(
                        error,
                    )));
                }
            }
            add_log_text(b"Add TaoZhuangSetup....OK");
            let mut payload = Vec::new();
            if let Err(error) = game.tao_zhuang_setup().add_byte_to_array(&mut payload) {
                return Some(Err(
                    GameEquipmentEnhancementStartupError::TaoZhuangSerialize(error),
                ));
            }
            let mut notice = CMessage::new(0x000B_F81A);
            notice.base_mut().add(&payload);
            let broadcast = notice.send_all(game.current_net_server());
            tracing::trace!(?broadcast, "настройки TaoZhuang загружены");
            Some(Ok(()))
        }
        CI_QING_LING_BAO_SELECTOR => {
            match game.ci_qing_setup_mut().de_byte_from_array(source, cursor) {
                Ok(()) => {}
                Err(error) => {
                    return Some(Err(GameEquipmentEnhancementStartupError::CiQingDecode(
                        error,
                    )));
                }
            }
            match game
                .ling_bao_setup_mut()
                .decode_from_array_ling_bao(source, cursor)
            {
                Ok(()) => {}
                Err(error) => {
                    return Some(Err(GameEquipmentEnhancementStartupError::LingBaoDecode(
                        error,
                    )));
                }
            }
            let mut payload = Vec::new();
            if let Err(error) = game.ci_qing_setup().add_byte_to_array(&mut payload) {
                return Some(Err(GameEquipmentEnhancementStartupError::CiQingSerialize(
                    error,
                )));
            }
            let mut notice = CMessage::new(0x000C_010D);
            notice.base_mut().add(&payload);
            let broadcast = notice.send_all(game.current_net_server());
            add_log_text(b"Add CiQingSetup...ok!");
            tracing::trace!(?broadcast, "настройки CiQing и LingBao загружены");
            Some(Ok(()))
        }
        _ => None,
    }
}

fn decode_lookup_filter_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<(), GameLookupFilterStartupError>> {
    match selector {
        DA_KONG_SELECTOR => Some(
            game.da_kong_xiang_qian_mut()
                .decord_from_byte_array(source, cursor)
                .map(|()| tracing::trace!("настройки DaKong загружены"))
                .map_err(GameLookupFilterStartupError::DaKong),
        ),
        WORDS_FILTER_SELECTOR => {
            let result = game
                .words_filter_mut()
                .from_byte_array(source, cursor)
                .map(|()| tracing::trace!("фильтр слов загружен"))
                .map_err(GameLookupFilterStartupError::WordsFilter);
            if result.is_ok() {
                add_log_text(b"Initial SI_WORDSFILTER...OK!");
            }
            Some(result)
        }
        JJC_REGION_LEVEL_SELECTOR => {
            game.clear_jjc_level_data();
            let count = match read_jjc_level_i32(source, cursor) {
                Ok(count) => count,
                Err(error) => {
                    return Some(Err(GameLookupFilterStartupError::JjcRegionLevels(error)));
                }
            };
            for _ in 0..count.max(0) {
                let key = match read_jjc_level_i32(source, cursor) {
                    Ok(key) => key,
                    Err(error) => {
                        return Some(Err(GameLookupFilterStartupError::JjcRegionLevels(error)));
                    }
                };
                let value = match read_jjc_level_i32(source, cursor) {
                    Ok(value) => value,
                    Err(error) => {
                        return Some(Err(GameLookupFilterStartupError::JjcRegionLevels(error)));
                    }
                };
                game.insert_jjc_level_data(key, value);
            }
            let entries = game.jjc_level_data().len();
            add_log_text(b"Initial SI_JJCREGIONLEVELSETUP...OK!");
            { tracing::trace!(entries, "уровни регионов JJC загружены"); Some(Ok(())) }
        }
        _ => None,
    }
}

fn decode_mutation_rules_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<(), GameMutationRulesStartupError>> {
    let result = match selector {
        SYNTHESIS_SELECTOR => game
            .synthesis_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("правила синтеза загружены"))
            .map_err(|error| GameMutationRulesStartupError::Synthesis(Arc::new(error))),
        NEW_SKILL_MONSTER_SELECTOR => game
            .new_skill_monster_conf_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("правила новых навыков монстров загружены"))
            .map_err(|error| GameMutationRulesStartupError::NewSkillMonsters(Arc::new(error))),
        GOODS_DESTROY_SELECTOR => game
            .goods_destroy_setup_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("правила уничтожения предметов загружены"))
            .map_err(|error| GameMutationRulesStartupError::GoodsDestruction(Arc::new(error))),
        CHANGE_BODY_SELECTOR => game
            .change_body_conf_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| tracing::trace!(entries, "правила смены тела загружены"))
            .map_err(|error| GameMutationRulesStartupError::ChangeBody(Arc::new(error))),
        _ => return None,
    };
    if result.is_ok() {
        add_log_text(match selector {
            SYNTHESIS_SELECTOR => b"Initial SI_SYNTHESIS...OK!".as_slice(),
            NEW_SKILL_MONSTER_SELECTOR => b"Initial SI_NEWSKILL_MONSTER_CONF...OK!".as_slice(),
            GOODS_DESTROY_SELECTOR => b"Initial SI_GOODS_DESTROY_CONF...OK!".as_slice(),
            CHANGE_BODY_SELECTOR => b"Initial SI_CHANGE_BODY...OK!".as_slice(),
            _ => unreachable!("selector отфильтрован перед mutation-rules log lookup"),
        });
    }
    Some(result)
}

fn decode_environment_configuration_startup(
    selector: i32,
    source: &[u8],
    cursor: &mut usize,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
) -> Option<Result<(), GameEnvironmentConfigurationStartupError>> {
    match selector {
        PRISON_CONF_SELECTOR => {
            let entries = match game
                .prison_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameEnvironmentConfigurationStartupError::Prison(error)));
                }
            };
            add_log_text(b"Initial SI_PRISON_CONF...OK!");
            tracing::trace!(entries, "настройки тюрьмы загружены");
            Some(Ok(()))
        }
        PRECIOUS_BOX_CONF_SELECTOR => {
            let entries = match game
                .precious_box_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(
                        GameEnvironmentConfigurationStartupError::PreciousBoxes(Arc::new(error)),
                    ));
                }
            };
            add_log_text(b"Initial SI_PRECIOUSBOX_CONF...OK!");
            tracing::trace!(entries, "настройки драгоценных сундуков загружены");
            Some(Ok(()))
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
) -> Option<Result<(), GameSpatialStartupError>> {
    match selector {
        PROXY_REGION_SELECTOR => {
            let mut region = CProxyServerRegion::default();
            if let Err(error) = region.decord_from_byte_array(source, cursor, true) {
                return Some(Err(GameSpatialStartupError::ProxyRegion(error)));
            }
            let region_id = region.get_id();
            let replaced = game.add_proxy_region(region);
            add_log_text(b"Add Proxy Region : (%d) %s ...OK!");
            tracing::trace!(region_id, replaced, "прокси-регион загружен");
            Some(Ok(()))
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
                tracing::warn!(region_id, "настройки региона не перезагружены: регион отсутствует");
                return Some(Ok(()));
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
            tracing::trace!(region_id, "настройки региона перезагружены");
            Some(Ok(()))
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
            { tracing::trace!(entries, "общие настройки регионов загружены"); Some(Ok(())) }
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
            { tracing::trace!(entries, "настройки дублируемых регионов загружены"); Some(Ok(())) }
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
) -> Option<Result<(), GameCountryStateStartupError>> {
    let result = match selector {
        COUNTRY_PARAM_SELECTOR => game
            .country_param_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("параметры стран загружены"))
            .map_err(GameCountryStateStartupError::Parameters),
        COUNTRY_HANDLER_SELECTOR => game
            .country_handler_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("состояния стран загружены"))
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
) -> Option<Result<(), GamePlayerRuleStartupError>> {
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
            { tracing::trace!(value, "индекс GameServer загружен"); Some(Ok(())) }
        }
        HIT_LEVEL_SELECTOR => {
            let entries = game
                .hit_level_setup_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GamePlayerRuleStartupError::HitLevel);
            if entries.is_ok() {
                add_log_text(b"Initial SI_HITLEVEL...OK!");
            }
            Some(entries.map(|entries| tracing::trace!(entries, "уровни попадания загружены")))
        }
        EMOTION_SELECTOR => {
            let report = game
                .emotion_mut()
                .unserialize(source, cursor)
                .map_err(GamePlayerRuleStartupError::Emotion);
            if report.is_ok() {
                add_log_text(b"Initial SI_EMOTION...OK!");
            }
            Some(report.map(|()| tracing::trace!("эмоции загружены")))
        }
        QUEST_SYSTEM_SELECTOR => {
            let report = game
                .quest_system_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GamePlayerRuleStartupError::QuestSystem);
            if report.is_ok() {
                add_log_text(b"Initial SI_QUEST...OK!");
            }
            Some(report.map(|()| tracing::trace!("система заданий загружена")))
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
) -> Option<Result<(), GameRuntimeConfigurationStartupError>> {
    match selector {
        GLOBE_SETUP_SELECTOR => {
            {
                let (globe_setup, region_router) = game.globe_setup_and_region_router_mut();
                match globe_setup.decord_from_byte_array(region_router, source, cursor) {
                    Ok(()) => {}
                    Err(error) => {
                        return Some(Err(GameRuntimeConfigurationStartupError::GlobeSetup(error)));
                    }
                }
            }
            let da_kong_key = game.globe_setup().da_kong_key();
            let goods_ai_enabled = game.globe_setup().goods_ai_enabled();
            let auction_enabled = game.globe_setup().auction_enabled();
            let area_width = game.globe_setup().area_width();
            let area_height = game.globe_setup().area_height();
            game.da_kong_xiang_qian_mut().set_key(da_kong_key);
            let mut notice = CMessage::new(0x000B_F736);
            notice.add_byte(u8::from(goods_ai_enabled));
            let goods_ai_broadcast = notice.send_all(game.current_net_server());
            let auction_forced_disabled = !auction_enabled;
            if auction_forced_disabled {
                game.force_auction_disabled();
            }
            game.set_area_dimensions(area_width, area_height);
            add_log_text(b"Initial SI_GLOBESETUP...OK!");
            tracing::trace!(da_kong_key, goods_ai_enabled, auction_enabled, area_width, area_height, ?goods_ai_broadcast, auction_forced_disabled, "глобальные настройки загружены");
            Some(Ok(()))
        }
        LOG_SYSTEM_SELECTOR => {
            match game.log_system_mut().decord_from_byte_array(source, cursor) {
                Ok(()) => {}
                Err(error) => {
                    return Some(Err(GameRuntimeConfigurationStartupError::LogSystem(error)));
                }
            }
            let da_kong_log = game.log_system().da_kong_log_enabled();
            game.da_kong_xiang_qian_mut()
                .set_log_key(da_kong_log);
            add_log_text(b"Initial SI_LOGSYSTEM...OK!");
            tracing::trace!(da_kong_log, "настройки журналирования загружены");
            Some(Ok(()))
        }
        GM_LIST_SELECTOR => {
            let report = game
                .gm_list_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameRuntimeConfigurationStartupError::GmList);
            if report.is_ok() {
                add_log_text(b"Initial SI_GMLIST...OK!");
            }
            Some(report.map(|()| tracing::trace!("список GM загружен")))
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
) -> Option<Result<(), GamePlayerEconomyStartupError>> {
    let result = match selector {
        PLAYER_LIST_SELECTOR => game
            .player_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(|()| tracing::trace!("шаблоны игроков загружены"))
            .map_err(GamePlayerEconomyStartupError::PlayerTemplates),
        TRADE_LIST_SELECTOR => game
            .trade_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| tracing::trace!(entries, "торговый список загружен"))
            .map_err(GamePlayerEconomyStartupError::TradeList),
        INCREMENT_SHOP_SELECTOR => game
            .increment_shop_list_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| tracing::trace!(entries, "список магазина прироста загружен"))
            .map_err(GamePlayerEconomyStartupError::IncrementShop),
        CONTRIBUTE_SETUP_SELECTOR => game
            .contribute_setup_mut()
            .decord_from_byte_array(source, cursor)
            .map(|entries| tracing::trace!(entries, "предметы вклада загружены"))
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
) -> Option<Result<(), GameCombatRegistryStartupError>> {
    match selector {
        GOODS_LIST_SELECTOR => {
            let report = game
                .goods_factory_mut()
                .unserialize(source, cursor)
                .map_err(GameCombatRegistryStartupError::Goods);
            if report.is_ok() {
                add_log_text(b"Initial SI_GOODSLIST...OK!");
            }
            Some(report.map(|()| tracing::trace!("реестр предметов загружен")))
        }
        MONSTER_LIST_SELECTOR => {
            match game
                .decode_monster_list(source, cursor)
                .map_err(GameCombatRegistryStartupError::Monsters)
            {
                Ok(()) => {}
                Err(error) => return Some(Err(error)),
            }
            add_log_text(b"Initial SI_MONSTERLIST...OK!");
            game.refresh_all_monster_base_property();
            tracing::trace!("реестр монстров загружен");
            Some(Ok(()))
        }
        SKILL_LIST_SELECTOR => {
            let report = game
                .skill_factory_mut()
                .rebuild(source, cursor)
                .map_err(GameCombatRegistryStartupError::Skills);
            if report.is_ok() {
                add_log_text(b"Initial SI_SKILLLIST...OK!");
            }
            Some(report.map(|()| tracing::trace!("реестр навыков загружен")))
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
) -> Option<Result<(), GameBattleFairyStartupError>> {
    match selector {
        FAIRY_EXP_SELECTOR => {
            let report = game
                .fairy_exp_conf_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameBattleFairyStartupError::FairyExp);
            if report.is_ok() {
                add_log_text(b"Initial SI_FAIRY_EXP...ok!");
            }
            Some(report.map(|()| tracing::trace!("опыт фей загружен")))
        }
        BATTLE_FAIRY_EXP_SELECTOR => {
            let report = game
                .battle_fairy_exp_config_mut()
                .decord_from_byte_array(source, cursor)
                .map_err(GameBattleFairyStartupError::BattleFairyExp);
            if report.is_ok() {
                add_log_text(b"Initial SI_BATLLE_FAIRY_CONF...ok\xA3\xA1");
            }
            Some(report.map(|()| tracing::trace!("опыт боевых фей загружен")))
        }
        BATTLE_FAIRY_COMBINE_SELECTOR => {
            let entries = game
                .battle_fairy_property_mut()
                .decord_byte_array_combine(source, cursor)
                .map_err(GameBattleFairyStartupError::Combine);
            if entries.is_ok() {
                add_log_text(b"Initial SI_BATLLE_FAIRY_COMBINE...ok!");
            }
            Some(entries.map(|entries| tracing::trace!(entries, "правила объединения боевых фей загружены")))
        }
        EQUIPMENT_COMPOSE_SELECTOR => Some(
            game.equipment_compose_list_mut()
                .decord_from_byte_array(source, cursor)
                .map(|()| tracing::trace!("правила составления снаряжения загружены"))
                .map_err(GameBattleFairyStartupError::EquipmentCompose),
        ),
        _ => None,
    }
}

fn dispatch_player_count_message(
    message_type: i32,
    game: &CGame,
) -> Option<()> {
    let (kind, response_type, requires_world_client) = match message_type {
        PLAYER_COUNT_IF_WORLD_CONNECTED_MESSAGE => (
            "при подключённом WorldServer",
            PLAYER_COUNT_IF_WORLD_CONNECTED_RESPONSE,
            true,
        ),
        PLAYER_COUNT_MESSAGE => (
            "безусловный",
            PLAYER_COUNT_RESPONSE,
            false,
        ),
        _ => return None,
    };
    if requires_world_client && game.world_client().is_none() {
        tracing::debug!(kind, "число игроков не отправлено: WorldServer не подключён");
        return Some(());
    }

    let player_count = game.player_count();
    let mut response = CMessage::new(response_type);
    response.add_ulong(player_count);
    let delivery = response.send(game, false);
    tracing::trace!(kind, player_count, ?delivery, "число игроков отправлено World");
    Some(())
}

fn dispatch_string_table_message(
    message: &mut CMessage,
    game: &mut CGame,
    source: GameStringTableSource,
) -> Result<(), GameServerMessageError> {
    {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        game.create_string_table(wire, cursor, |text| tracing::info!(text = %String::from_utf8_lossy(text), "сообщение загрузки GameServer"))
            .map_err(GameServerMessageError::StringTable)?
    }
    tracing::trace!(?source, "таблица строк GameServer применена");
    Ok(())
}

/// Выполняет terminal startup selector `0x3B` в исходном порядке side effects.
pub(crate) fn dispatch_client_server_start(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    now_ms: u32,
) -> Option<()> {
    if selector != CLIENT_SERVER_START_SELECTOR {
        return None;
    }

    let network_started = match game.init_net_server(now_ms) {
        Ok(()) => true,
        Err(GameNetworkInitializationError::MissingNetworkSetup) => {
            tracing::error!("сетевой запуск GameServer не выполнен: настройки отсутствуют");
            false
        }
        Err(GameNetworkInitializationError::Host(error)) => {
            tracing::error!(%error, "сетевой запуск GameServer не выполнен");
            false
        }
    };
    if !network_started {
        tracing::error!(dialog_message = "Can't init NetServer!", dialog_title = "Message", "инициализация сетевого владельца завершилась ошибкой");
    }
    let (monsters, npcs) = game.initial_region_totals();
    tracing::info!(monsters, npcs, network_started, "GameServer запущен как клиентский сервер");
    let (server_ids, applied_login_id, applied_world_id) = read_and_apply_server_ids(message, game);
    tracing::trace!(?server_ids, ?applied_login_id, ?applied_world_id, "идентификаторы серверов применены");
    Some(())
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
    let mut reader = LegacyReader::at(wire, offset).map_err(|_| {
        GameClientServerStartPayloadError { offset, needed: 4, available }
    })?;
    let value = reader.read_i32().map_err(|_| {
        GameClientServerStartPayloadError { offset, needed: 4, available }
    })?;
    *cursor = reader.position();
    Ok(value)
}

pub(crate) trait InitialRegionStartupContext:
    CityRegionDecodeContext + CountryRegionDecodeContext + GodsBattleNpcSpawnContext
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InitialRegionSubtypeInputBlock {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InitialRegionStartupError {
    SubtypeInput(InitialRegionSubtypeInputBlock),
    UnknownSubtype(i32),
    Base(ServerRegionDecodeError),
    War(WarRegionDecodeError<ServerRegionDecodeError>),
    City(CityRegionDecodeError<ServerRegionDecodeError>),
    Country(CountryRegionDecodeError<ServerRegionDecodeError>),
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
) -> Option<Result<(), InitialRegionStartupError>>
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
    let (area_width, area_height) = game.area_dimensions();
    let owner = match subtype {
        0 => {
            let mut region = CServerRegion::default();
            if let Err(error) = region.decord_from_byte_array(
                source,
                cursor,
                true,
                area_width,
                area_height,
                game.monster_registry(),
                game.skill_factory(),
                context,
            ) {
                return Some(Err(InitialRegionStartupError::Base(error)));
            }
            ServerRegionOwner::Base(region)
        }
        1 => {
            let mut region = CServerVillageRegion::default();
            if let Err(error) = region.decord_from_byte_array(
                source,
                cursor,
                true,
                area_width,
                area_height,
                game.monster_registry(),
                game.skill_factory(),
                context,
            ) {
                return Some(Err(InitialRegionStartupError::War(error)));
            }
            ServerRegionOwner::Village(region)
        }
        2 => {
            let mut region = CServerCityRegion::default();
            if let Err(error) = region.decord_from_byte_array(
                source,
                cursor,
                true,
                area_width,
                area_height,
                game.monster_registry(),
                game.skill_factory(),
                context,
            ) {
                return Some(Err(InitialRegionStartupError::City(error)));
            }
            ServerRegionOwner::City(region)
        }
        3 => {
            let mut region = CServerCountryRegion::default();
            if let Err(error) = region.decord_from_byte_array(
                source,
                cursor,
                true,
                area_width,
                area_height,
                game.monster_registry(),
                game.skill_factory(),
                context,
            ) {
                return Some(Err(InitialRegionStartupError::Country(error)));
            }
            ServerRegionOwner::Country(region)
        }
        4 => {
            let mut region = ServerNationRegion::default();
            if let Err(error) = region.decord_from_byte_array(
                source,
                cursor,
                true,
                area_width,
                area_height,
                game.monster_registry(),
                game.skill_factory(),
                context,
            ) {
                return Some(Err(InitialRegionStartupError::War(error)));
            }
            ServerRegionOwner::Nation(region)
        }
        5 => {
            let mut region = CServerGodsBattleRegion::default();
            if let Err(error) =
                game.decode_initial_gods_battle_region(&mut region, source, cursor, true, context)
            {
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
    let (spawned_monsters, spawned_npcs) = owner.base().total_spawned_shapes();
    let replaced = game.add_region(owner);
    add_log_text(b"Start Region : (%d) %s [m=%d n=%d] ...OK!");

    let (previous_monsters, previous_npcs) = game.initial_region_totals();
    let total_monsters = previous_monsters.wrapping_add(spawned_monsters);
    let total_npcs = previous_npcs.wrapping_add(spawned_npcs);
    game.set_initial_region_totals(total_monsters, total_npcs);
    let gods_battle_registered =
        gods_battle && game.gods_battle_mgr_mut().add_region_set(region_id);

    tracing::trace!(
        subtype,
        region_id,
        added_to_region_list,
        replaced,
        total_monsters,
        total_npcs,
        gods_battle_registered,
        "начальный регион загружен"
    );
    Some(Ok(()))
}

fn read_initial_region_i32(
    source: &[u8],
    cursor: &mut usize,
) -> Result<i32, InitialRegionSubtypeInputBlock> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(source, offset).map_err(|_| {
        InitialRegionSubtypeInputBlock { offset, needed: 4, available }
    })?;
    let value = reader.read_i32().map_err(|_| {
        InitialRegionSubtypeInputBlock { offset, needed: 4, available }
    })?;
    *cursor = reader.position();
    Ok(value)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSetupReloadError {
    RegionId(InitialRegionSubtypeInputBlock),
    Setup(ServerRegionSetupDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum GameScriptResourceDecodeError {
    #[error("script resource обрывается на {field} в {offset}: нужно {required}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    #[error("script resource {resource} содержит отрицательную длину {declared}")]
    NegativeLength {
        resource: &'static str,
        declared: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("Game ID index отсутствует в {offset}: нужно 1, доступно {available}")]
pub(crate) struct GameIdIndexDecodeError {
    pub(crate) offset: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum HonorRankStartupError {
    #[error("total honor reset mask обрывается в {offset}: нужно 4, доступно {available}")]
    MissingResetMask { offset: usize, available: usize },
    #[error(transparent)]
    Decode(#[from] HonorRanksDecodeError),
}

fn read_honor_reset_mask(source: &[u8], cursor: &mut usize) -> Result<u32, HonorRankStartupError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(source, offset)
        .map_err(|_| HonorRankStartupError::MissingResetMask { offset, available })?;
    let value = reader
        .read_u32()
        .map_err(|_| HonorRankStartupError::MissingResetMask { offset, available })?;
    *cursor = reader.position();
    Ok(value)
}

/// Наблюдаемый итог reconnect-ветви Billing `0x6F904`.
#[derive(Debug)]
pub(crate) struct GameBillingClientReplacement {
    pub(crate) previous_client_closed: bool,
    pub(crate) registration: Result<i32, SendMessageError>,
    pub(crate) connected_notice: bool,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("JJC region-level snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
pub(crate) struct JjcRegionLevelDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

fn read_jjc_level_i32(source: &[u8], cursor: &mut usize) -> Result<i32, JjcRegionLevelDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(source, offset)
        .map_err(|_| JjcRegionLevelDecodeError { offset, needed: 4, available })?;
    let value = reader
        .read_i32()
        .map_err(|_| JjcRegionLevelDecodeError { offset, needed: 4, available })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_script_resource_length(
    source: &[u8],
    cursor: &mut usize,
    resource: &'static str,
) -> Result<i32, GameScriptResourceDecodeError> {
    let offset = *cursor;
    let mut reader = script_resource_reader(source, offset, "resource length", 4)?;
    let declared = reader
        .read_i32()
        .map_err(|block| script_resource_error("resource length", block))?;
    *cursor = reader.position();
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
    let mut reader = script_resource_reader(source, *cursor, field, required)?;
    let bytes = reader
        .read_bytes(required)
        .map_err(|block| script_resource_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn script_resource_reader<'source>(source: &'source [u8], cursor: usize, field: &'static str, required: usize) -> Result<LegacyReader<'source>, GameScriptResourceDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| GameScriptResourceDecodeError::UnexpectedEnd { field, offset: block.offset, required, available: block.available })
}

fn script_resource_error(field: &'static str, block: crate::gameserver::appserver::legacycodec::LegacyReadBlock) -> GameScriptResourceDecodeError {
    GameScriptResourceDecodeError::UnexpectedEnd { field, offset: block.offset, required: block.needed, available: block.available }
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
    add_game_log_text(b"Reconnect to BillingServer Success!");
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
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum WarScheduleSetupError {
    #[error("AttackCity snapshot: {0}")]
    AttackCity(#[source] AttackCityDecodeError),
    #[error("Village snapshot: {0}")]
    Village(#[source] VillageWarDecodeError),
    #[error("CountryWar snapshot: {0}")]
    Country(#[source] CountryWarDecodeError),
    #[error("FourNationWar snapshot: {0}")]
    FourNation(#[source] FourNationGameDecodeError),
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
    mut add_log_text: impl FnMut(&'static str),
) -> Option<Result<(), WarScheduleSetupError>> {
    match selector {
        ATTACK_CITY_SELECTOR => {
            if let Err(error) = attack_city_sys.decord_from_byte_array(payload, cursor) {
                return Some(Err(WarScheduleSetupError::AttackCity(error)));
            }
            {
                let mut adapter = AttackCityContextAdapter(context);
                attack_city_sys.init_city_region_state(&mut adapter)
            }
            add_log_text("Initial SI_ATTACKCITYSYS_SETUP...OK!");
            tracing::trace!("система осады города загружена");
            Some(Ok(()))
        }
        VILLAGE_WAR_SELECTOR => {
            if let Err(error) = village_war_sys.decord_from_byte_array(payload, cursor) {
                return Some(Err(WarScheduleSetupError::Village(error)));
            }
            {
                let mut adapter = VillageWarContextAdapter(context);
                village_war_sys.init_village_region_state(&mut adapter)
            }
            add_log_text("Initial SI_VILLAGEWARSYS_SETUP...OK!");
            tracing::trace!("система деревенской войны загружена");
            Some(Ok(()))
        }
        COUNTRY_WAR_SELECTOR => {
            if let Err(error) = country_war_sys.decord_from_byte_array(payload, cursor) {
                return Some(Err(WarScheduleSetupError::Country(error)));
            }
            {
                let mut adapter = CountryWarContextAdapter(context);
                country_war_sys.init_country_region_state(&mut adapter)
            }
            add_log_text("Initial SI_COUNTRYWAR...OK!");
            tracing::trace!("система войны стран загружена");
            Some(Ok(()))
        }
        FOUR_NATION_WAR_SELECTOR => {
            match four_nation_war_sys.decord_from_byte_array(payload, cursor) {
                Ok(()) => {}
                Err(error) => return Some(Err(WarScheduleSetupError::FourNation(error))),
            }
            {
                let mut adapter = FourNationWarContextAdapter(context);
                four_nation_war_sys.init_war_state(&mut adapter)
            }
            add_log_text("Initial SI_FOURNATIONWARSYS_SETUP..OK!!");
            tracing::trace!("система войны четырёх государств загружена");
            Some(Ok(()))
        }
        _ => None,
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
