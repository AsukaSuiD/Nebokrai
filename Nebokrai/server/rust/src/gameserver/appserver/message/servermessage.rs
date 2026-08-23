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
//! QuestSystem `0x16` сохраняет exact scalar/script/map mutation order и
//! публикует runtime lookup-owner до финального startup log.
//! CountryParam `0x18` сохраняет scalar prefix, пять ordered maps и известные
//! technology wire-quirks до точного startup log.
//! CountryHandler `0x19` заменяет прежние country owners, декодирует byte-count
//! minister records и публикует ordered lookup до финального startup log.
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
use crate::gameserver::appserver::country::countryhandler::{
    CountryHandlerDecodeError, CountryHandlerDecodeReport,
};
use crate::gameserver::appserver::country::countryparam::{
    CountryParamDecodeReport, CountryParamInputBlock,
};
use crate::gameserver::appserver::goods::cbattlefairyproperty::BattleFairyComposeDecodeError;
use crate::gameserver::gameserver::game::{CGame, GameNetworkInitializationError};
use crate::gameserver::gameserver::playerranks::PlayerRanksDecodeError;
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::nets::netserver::mynetclient::CMyNetClient;
use crate::public::ciqing::{CiQingDecodeError, CiQingDecodeReport, CiQingSerializationBlock};
use crate::public::dakongxiangqian::{DaKongDecodeError, DaKongDecodeReport};
use crate::public::dupliregionsetup::DupliRegionDecodeError;
use crate::public::equipmentcomposelist::{
    EquipmentComposeDecodeError, EquipmentComposeDecodeReport,
};
use crate::public::taozhuangsetup::{TaoZhuangDecodeError, TaoZhuangSerializationBlock};
use crate::public::wordsfilter::{WordsFilterDecodeError, WordsFilterDecodeReport};
use crate::setup::cbattlefairyexpconfig::{BattleFairyExpDecodeError, BattleFairyExpDecodeReport};
use crate::setup::changebody::ChangeBodyDecodeError;
use crate::setup::contributesetup::ContributeSetupDecodeError;
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
use crate::setup::newskillmonsterlist::{NewSkillMonsterDecodeError, NewSkillMonsterDecodeReport};
use crate::setup::playerlist::{PlayerListDecodeError, PlayerListDecodeReport};
use crate::setup::preciousboxconf::PreciousBoxDecodeError;
use crate::setup::prisonconf::PrisonConfDecodeError;
use crate::setup::questsystem::{QuestSystemDecodeError, QuestSystemDecodeReport};
use crate::setup::regionsetup::RegionSetupDecodeError;
use crate::setup::synthesis::{SynthesisDecodeError, SynthesisDecodeReport};
use crate::setup::tradelist::TradeListDecodeError;

const BILLING_REGISTRATION: i32 = 0x000E_F101;
const CLIENT_SERVER_START_SELECTOR: i32 = 0x3b;
const PLAYER_LIST_SELECTOR: i32 = 0x01;
const TRADE_LIST_SELECTOR: i32 = 0x03;
const INCREMENT_SHOP_SELECTOR: i32 = 0x04;
const CONTRIBUTE_SETUP_SELECTOR: i32 = 0x05;
const GLOBE_SETUP_SELECTOR: i32 = 0x07;
const LOG_SYSTEM_SELECTOR: i32 = 0x08;
const GM_LIST_SELECTOR: i32 = 0x09;
const REGION_SETUP_SELECTOR: i32 = 0x11;
const HIT_LEVEL_SELECTOR: i32 = 0x14;
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
const DA_KONG_SELECTOR: i32 = 0x2b;
const BATTLE_FAIRY_EXP_SELECTOR: i32 = 0x2c;
const BATTLE_FAIRY_COMBINE_SELECTOR: i32 = 0x2d;
const EQUIPMENT_COMPOSE_SELECTOR: i32 = 0x30;
const WORDS_FILTER_SELECTOR: i32 = 0x31;
const JJC_REGION_LEVEL_SELECTOR: i32 = 0x32;
const TAO_ZHUANG_SELECTOR: i32 = 0x34;
const CI_QING_LING_BAO_SELECTOR: i32 = 0x35;
const THING_SETUP_SELECTOR: i32 = 0x36;
const GODS_BATTLE_SELECTOR: i32 = 0x39;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOwnedStartupSnapshotReport {
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
    RegionSetup {
        entries: usize,
    },
    HitLevel {
        entries: usize,
    },
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
    PlayerList(PlayerListDecodeError),
    TradeList(TradeListDecodeError),
    IncrementShop(IncrementShopDecodeError),
    ContributeSetup(ContributeSetupDecodeError),
    GlobeSetup(GlobeSetupDecodeError),
    LogSystem(LogSystemDecodeError),
    GmList(GmListDecodeError),
    RegionSetup(RegionSetupDecodeError),
    HitLevel(HitLevelDecodeError),
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
            Self::PlayerList(error) => error.fmt(formatter),
            Self::TradeList(error) => error.fmt(formatter),
            Self::IncrementShop(error) => error.fmt(formatter),
            Self::ContributeSetup(error) => error.fmt(formatter),
            Self::GlobeSetup(error) => error.fmt(formatter),
            Self::LogSystem(error) => error.fmt(formatter),
            Self::GmList(error) => error.fmt(formatter),
            Self::RegionSetup(error) => error.fmt(formatter),
            Self::HitLevel(error) => error.fmt(formatter),
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
            Self::PlayerList(error) => Some(error),
            Self::TradeList(error) => Some(error),
            Self::IncrementShop(error) => Some(error),
            Self::ContributeSetup(error) => Some(error),
            Self::GlobeSetup(error) => Some(error),
            Self::LogSystem(error) => Some(error),
            Self::GmList(error) => Some(error),
            Self::RegionSetup(error) => Some(error),
            Self::HitLevel(error) => Some(error),
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
pub(crate) fn dispatch_game_owned_startup_snapshot(
    selector: i32,
    message: &mut CMessage,
    game: &mut CGame,
    mut add_log_text: impl FnMut(&[u8]),
    mut put_string_to_file: impl FnMut(&str, &[u8]),
) -> Option<Result<GameOwnedStartupSnapshotReport, GameOwnedStartupSnapshotError>> {
    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    match selector {
        PLAYER_LIST_SELECTOR => {
            let report = match game
                .player_list_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::PlayerList(error))),
            };
            add_log_text(b"Initial SI_PLAYERLIST...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::PlayerList(report)))
        }
        TRADE_LIST_SELECTOR => {
            let entries = match game.trade_list_mut().decord_from_byte_array(source, cursor) {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::TradeList(error))),
            };
            add_log_text(b"Initial SI_TRADELIST...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::TradeList { entries }))
        }
        INCREMENT_SHOP_SELECTOR => {
            let entries = match game
                .increment_shop_list_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::IncrementShop(error)));
                }
            };
            add_log_text(b"Initial SI_INCREMENTSHOPLIST...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::IncrementShop {
                entries,
            }))
        }
        CONTRIBUTE_SETUP_SELECTOR => {
            let entries = match game
                .contribute_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::ContributeSetup(error)));
                }
            };
            add_log_text(b"Initial SI_CONTRIBUTEITEM...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::ContributeSetup {
                entries,
            }))
        }
        GLOBE_SETUP_SELECTOR => {
            let decoded = {
                let (globe_setup, region_router) = game.globe_setup_and_region_router_mut();
                match globe_setup.decord_from_byte_array(region_router, source, cursor) {
                    Ok(report) => report,
                    Err(error) => {
                        return Some(Err(GameOwnedStartupSnapshotError::GlobeSetup(error)));
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
            Some(Ok(GameOwnedStartupSnapshotReport::GlobeSetup {
                decoded,
                goods_ai_broadcast,
                auction_forced_disabled,
            }))
        }
        LOG_SYSTEM_SELECTOR => {
            let report = match game.log_system_mut().decord_from_byte_array(source, cursor) {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::LogSystem(error))),
            };
            game.da_kong_xiang_qian_mut().set_key(report.da_kong_log);
            add_log_text(b"Initial SI_LOGSYSTEM...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::LogSystem {
                entries: report.items,
                da_kong_log: report.da_kong_log,
            }))
        }
        GM_LIST_SELECTOR => {
            let report = match game.gm_list_mut().decord_from_byte_array(source, cursor) {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::GmList(error))),
            };
            add_log_text(b"Initial SI_GMLIST...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::GmList(report)))
        }
        REGION_SETUP_SELECTOR => {
            let entries = match game
                .region_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::RegionSetup(error))),
            };
            add_log_text(b"Initial SI_REGIONLEVELSETUP...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::RegionSetup { entries }))
        }
        HIT_LEVEL_SELECTOR => {
            let entries = match game
                .hit_level_setup_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::HitLevel(error))),
            };
            add_log_text(b"Initial SI_HITLEVEL...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::HitLevel { entries }))
        }
        QUEST_SYSTEM_SELECTOR => {
            let report = match game
                .quest_system_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::QuestSystem(error)));
                }
            };
            add_log_text(b"Initial SI_QUEST...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::QuestSystem(report)))
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
        COUNTRY_PARAM_SELECTOR => {
            let report = match game
                .country_param_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::CountryParam(error)));
                }
            };
            add_log_text(b"Initial SI_COUNTRYPARAM...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::CountryParam(report)))
        }
        COUNTRY_HANDLER_SELECTOR => {
            let report = match game
                .country_handler_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::CountryHandler(
                        error,
                    )));
                }
            };
            add_log_text(b"Initial SI_COUNTRY...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::CountryHandler(report)))
        }
        DUPLI_REGION_SELECTOR => {
            let Some(setup) = game.dupli_region_setup_mut() else {
                return Some(Err(GameOwnedStartupSnapshotError::OwnerUnavailable {
                    selector,
                }));
            };
            if let Err(error) = setup.decord_from_byte_array(source, cursor) {
                return Some(Err(GameOwnedStartupSnapshotError::DupliRegions(error)));
            }
            let entries = setup.entries().len();
            add_log_text(b"Initial SI_DUPLIREGIONSETUP...OK!");
            Some(Ok(GameOwnedStartupSnapshotReport::DupliRegions { entries }))
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
        FAIRY_EXP_SELECTOR => {
            let report = match game
                .fairy_exp_conf_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => return Some(Err(GameOwnedStartupSnapshotError::FairyExp(error))),
            };
            add_log_text(b"Initial SI_FAIRY_EXP...ok!");
            Some(Ok(GameOwnedStartupSnapshotReport::FairyExp(report)))
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
        BATTLE_FAIRY_EXP_SELECTOR => {
            let report = match game
                .battle_fairy_exp_config_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::BattleFairyExp(error)));
                }
            };
            add_log_text(b"Initial SI_BATLLE_FAIRY_CONF...ok\xA3\xA1");
            Some(Ok(GameOwnedStartupSnapshotReport::BattleFairyExp(report)))
        }
        BATTLE_FAIRY_COMBINE_SELECTOR => {
            let entries = match game
                .battle_fairy_property_mut()
                .decord_byte_array_combine(source, cursor)
            {
                Ok(entries) => entries,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::BattleFairyCombine(
                        error,
                    )));
                }
            };
            add_log_text(b"Initial SI_BATLLE_FAIRY_COMBINE...ok!");
            Some(Ok(GameOwnedStartupSnapshotReport::BattleFairyCombine {
                entries,
            }))
        }
        EQUIPMENT_COMPOSE_SELECTOR => {
            let report = match game
                .equipment_compose_list_mut()
                .decord_from_byte_array(source, cursor)
            {
                Ok(report) => report,
                Err(error) => {
                    return Some(Err(GameOwnedStartupSnapshotError::EquipmentCompose(error)));
                }
            };
            Some(Ok(GameOwnedStartupSnapshotReport::EquipmentCompose(report)))
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
