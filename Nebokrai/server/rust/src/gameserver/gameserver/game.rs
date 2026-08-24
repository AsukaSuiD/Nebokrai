//! Достигнутая send/receive dispatch storage-часть `CGame` GameServer.
//!
//! PDB подтверждает nullable `s_pNetClientOfWS +0x8`,
//! `s_pNetClientOfBS +0xC`, `s_pNetServer +0x10`, ordered
//! `s_mapPlayer +0x14` типа `long -> CPlayer*` и ordered
//! `m_JJcLevelData` типа `long -> long` в startup dispatcher-е и
//! `m_mTeamSessionID +0x188` типа `unsigned long -> long`. `FindPlayer` RVA
//! `0x00014930`, `GetTeamSessionID` RVA `0x00013690`, `GetTeamID` RVA
//! `0x0008BEB0` и достигнутые
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
//! остаётся typed-границей. `Release` и остальные resource/runtime owners ниже
//! остаются RAW;
//! `Init` связан через обязательный World client, общий MSVCRT RNG и sequence
//! registry до необязательной Billing-попытки, затем создаёт DupliRegion,
//! move-check, ranks и GoodsWar owners. GodsBattle startup snapshot хранится
//! отдельным `CGodsBattleMgr` historical owner-ом. GoodsWar constructor
//! сохраняет немедленный World request. Tokio/socket types заменяют ненаблюдаемые
//! `CBaseMessage::Initial` и `CMySocket::MySocketInit`; следующий незакрытый
//! шаг — resource/runtime owners после завершённого `Init`.
//! QuestSystem, CountryParam, CountryHandler, AttackCity, VillageWar,
//! CountryWarSys, CFourNationWarSys и CEmotion process singletons хранятся
//! owned-полями `CGame`, сохраняя exact startup wire и runtime lookup-
//! контракты без отдельных global allocations.
//! Function/variable/script file buffers принадлежат `CGame`; script parser
//! globals остаются явной context-границей. Повторный function/variable setter
//! безопасно материализует исходный freed-owner контракт как `None`; старый
//! `length + 1` NUL-padding заменён bounded `Vec` и C-string prefix adapter-ом.
//! `MyStringTable` также принадлежит `CGame`: reload сначала очищает map,
//! публикует decoded prefix и сохраняет пустой fallback `GetStringByID`.
//! `CHonorRanks` хранит четыре rank-type × четыре country snapshots; поля
//! полного игрока для total-reset остаются отдельной assembly-границей.
//! `CSkillFactory` принадлежит `CGame`: startup selector `0x06` заменяет весь
//! composite-key registry, а будущие skill/state owners получают те же runtime
//! lookup-ы без process-global raw pointers.
//! `CGoodsFactory` аналогично хранит startup selector `0x00`, включая оба
//! byte-name index-а для последующего container/goods lifecycle.
//! Goods, monster и skill registries `0x00/0x02/0x06` теперь публикуются из
//! реального World FIFO; monster registry после точного log выполняет полный
//! refresh lookup-связности уже живых region monster-ов.
//! Player templates, trade list, increment shop и contribution setup
//! `0x01/0x03/0x04/0x05` входят туда же одной resource-группой с исходными
//! partial publication, cursor и success-log границами.
//! GlobeSetup, LogSystem и GM-list `0x07/0x08/0x09` также достигаются из FIFO:
//! router/DaKong/auction/area mutations, Goods-AI broadcast и permission
//! registry публикуются в подтверждённом порядке до соответствующих logs.
//! Game ID, hit-level, emotion и quest resources `0x12/0x14/0x15/0x16`
//! проходят общий player-rule FIFO pass с точными partial/cursor/log
//! контрактами; player-ranks `0x17` соседним owner-ом сохраняет missing-init и
//! исходный allocation source.
//! CountryParam и CountryHandler `0x18/0x19` публикуются одним country-state
//! FIFO pass, сохраняя scalar/map partial mutation, replacement и exact logs.
//! Proxy/reload/region-level/dupli selectors `0x0F/0x10/0x11/0x1A` проходят
//! общий spatial FIFO pass с canonical region owners, ранним reload miss и
//! сохранённой allocation-error причиной duplicate registry.
//! Prison/PreciousBox `0x1D/0x1E` входят в общий environment-configuration
//! FIFO pass с clear/partial-decode owners и сохранённой причиной allocation
//! failure вложенных box-списков.
//! Synthesis/new-skill/goods-destruction/change-body `0x21..0x24` проходят
//! общий mutation-rules FIFO pass с полными decode reports, partial registries,
//! точными logs и сохранёнными allocation-error sources.
//! DaKong/WordsFilter/JJC levels `0x2B/0x31/0x32` проходят общий lookup/filter
//! FIFO pass с исходными clear/append, partial publication и success logs.
//! TaoZhuang и CiQing/LingBao `0x34/0x35` проходят общий enhancement FIFO pass:
//! owners сериализуются в client wire, broadcast-ятся и логируются в exact order.
//! Leiting/GodsBattle `0x36/0x39` проходят общий world-event FIFO pass с
//! dynamic/internal/final logs и typed file-audit effects в исходном порядке.
//! Honor configuration/ranks `0x26..0x2A` проходят полный FIFO pass; total
//! snapshot сбрасывает counters canonical player map и возвращает точные
//! AdjustHonorRank script-effects для внешнего script runtime.
//! Function/variable/general/script-file resources `0x0A..0x0D` получают
//! parser callbacks от того же `GameMainLoopRuntime`, который исполняет Script
//! stage, и публикуются из живого FIFO с duplicate-owner семантикой.
//! OrganSys war opcodes `0x7FE1F..0x7FE36` тем же FIFO меняют owned
//! AttackCity/Village schedules, concrete local/proxy region phases и
//! contender state с сохранением City/Village message/log side effects.
//! Control tail `0x7FE46..0x7FE4A` продолжает этот route: FourNation
//! exploit/time выполняют player property/client effects и owned time-map,
//! country treasury clamp публикует exact World `0x60314`, morale меняется в
//! owned schedule, а router response переиспользует входной wire как `0xBFF36`.
//! Nation combat callback-ы продолжают эту вертикаль: `OnBeenHurted` хранит
//! first-hit flags и exact World notices, `OnDied` исполняет morale/fail
//! packets, regional notices и одноразовый YuYingShi через concrete `AddNpc`.
//! Script-facing carriage return отдельно сохраняет saturating treasure-box
//! count, wrapping morale bonus и немедленный regional `0xBF818` snapshot.
//! Nation contend проходит через те же canonical player/region owners: enter,
//! cancel, damage и AI timeout исполняют `0xBFF29`, localized notices, захват
//! с morale/top-info и три concrete treasure-box `AddNpc` в исходном порядке.
//! Связанный player-state проход сохраняет `SetContendState` around-wire,
//! Nation `OnDied` с half-duration penalty, обе relive-копии `0xBFF2A` и
//! wrapping periodic countdown. Неопределённый `AL` раннего cancel-return
//! остаётся typed outcome, а не подменяется придуманным уведомлением.
//! Перед contend-таймером тот же Nation AI исполняет строгий gate четырёх
//! стражей и адмирала: локализованный stone NPC получает explosion/removal
//! around-пакеты, удаляется из spatial owner-а и заменяется monster-ом в
//! фиксированной точке.
//! CountryWar `0x7FF17..0x7FF22` продолжает тот же lifecycle: мутирует
//! country-region phases/results, выполняет clear через concrete runtime и
//! переиспользует входной message для all/country-filtered client broadcast.
//! GoodsWar `0x7FF20/21` тем же country route публикует member/faction/count
//! snapshots в process-owned owner и сохраняет World delete notification.
//! Battle-fairy combine теперь замыкает game player-map с GlobeSetup gate и
//! maximum fetch power, exact Game RNG, обеими exp-таблицами, goods/skill
//! registry и явным old-client serializer-ом; он возвращает ordered адресные
//! effects, потому что transport encoder этого семейства ещё отдельный owner.
//! Обе exp-таблицы, combine recipes и equipment-compose maps теперь также
//! достигаются из World startup FIFO selector-ами `0x20/0x2C/0x2D/0x30`,
//! сохраняя partial publication, cursor и точные startup log-effects.
//! Equipment add/remove проведены через canonical player registry до war-soul
//! state/skills, property callbacks, remove vitals clamp и typed around
//! `0xBF720`; полный virtual property owner остаётся caller adapter-ом.
//! Один `MainLoop` turn сохраняет static DWORD clocks как owned process state,
//! exact Script→AI→Message→Session→NetSession→Auction order, optional profile
//! reads, refresh/watch gates и wrapping pacing. Ещё не материализованные
//! concrete owner-ы подключаются через обязательный runtime trait; message
//! dispatch уже исполняется живым `CGame`.
//! `Release` сохраняет player-save/catch, city-save, registry/script/factory,
//! network и singleton teardown order; Rust `clear/take/Drop` заменяет manual
//! delete, а ещё внешние process-global owners вызываются typed runtime-ом.
//! Неопределённый security-cookie-derived int не объявляется результатом.
//! `GameThreadFunc` связан с достигнутыми `Init`, повторным `MainLoop` и
//! безусловным `Release`; COM, exit-event и window-close заменены platform
//! callbacks до появления конкретного Linux process runtime.
//! `RunAuction` хранит process-static tick, auction state/time и primary
//! ordered goods owner в `CGame`; exact World `0x80403` caller мутирует
//! его из живого message FIFO, а `MainLoop` публикует `0x60807/0x60808`
//! с исходными strict/wrapping time gates без внешнего auction hook-а.
//! Game-variant `CNetSessionManager` также принадлежит `CGame`: `MainLoop`
//! напрямую выполняет ordered timeout/callback pass, а `Release` очищает
//! его после socket cleanup в исходной lifecycle-позиции.
//! GM silence `0x7FC0B/0x7FC0E` достигает canonical player map из
//! `ProcessMessage`: byte-name lookup, lazy expiry и оба World response-а
//! исполняются до оставшегося внешним GM route owner-а. Адресный `0x7FC0F`
//! там же формирует player system message с local listener IP, а `0x7FC0D`
//! выбирает один из двух player wire и публикует его через `SendAll`.
//! Requester-localized `0x7FC0C` безопасно сохраняет подтверждённый
//! `GS0033/GS0034` `%s/%d/%s` contract и адресный `0xBF806` результат.
//! Country-filtered `0x7FC13` обходит ту же canonical player map в signed
//! ID-order и адресно публикует `0xBF806` каждому совпавшему country byte.
//! Mass-kick `0x7FC09` сохраняет requester, обходит остальные player ID в том
//! же порядке и ставит exact `QuitClientByMapID`; legacy `KickPlayer` при этом
//! всегда возвращает `false` независимо от queue result.
//! Named kick `0x7FC06` использует byte-exact ordered `FindPlayer(char*)`,
//! ставит тот же close side effect и только затем отвечает WorldServer.
//! Around-kick `0x7FC07` связывает name lookup, player father-region и
//! 7×7 `GetShape` scan с consecutive-only `list::unique`, ordered kicks и
//! итоговым World `0x5FD02`; повреждённые pointer/coordinate facts заменены
//! typed block outcome без выдуманного успешного ответа. Общий resolver
//! дополнен owned monster/NPC; любой ещё не owned goods/other registry entry
//! блокирует scan, чтобы не пропустить более ранний non-player `GetShape`.
//! Presence feedback `0x7FC08` сохраняет signed-char outcome, локализует
//! `GS0025/GS0026` с одним byte-string аргументом и отвечает requester-у.
//! Region-kick `0x7FC0A` связывает `FindRegion`, physical row-major area
//! registry и ordered `QuitClientByMapID`, исключая requester без дедупликации.
//! Входящий `0x5FF15` проверяет target player до чтения остатка payload и
//! публикует каждую list-строку отдельным адресным system message.
//! Полная GMA family `0x800xx` теперь входит в реальный FIFO message pass:
//! address kick возвращает World `0x60401`, count — `0x60402`, а unknown
//! сохраняет исходный warning без фиктивного handler success.
//! Depot family `0x7FBxx/0x8FExx` проходит тот же FIFO до generic route:
//! player guards, password mutation, bank/depot locks и адресные
//! `0xBFB07/0xBFB08` выполняются одним owner-ом.
//! Terminal server startup `0x7F801/0x3B` из FIFO действительно поднимает
//! client-facing listener, сохраняет ordered dialog/log effects и только
//! после них читает login/world IDs; другие startup selectors не перехватывает.
//! Language table проходит тот же server dispatcher как startup `0x2F` и
//! runtime `0x7F807`: обе ветви очищают один `CGame` owner, сохраняют partial
//! decode/error, точный cursor и ordered log-effect в process report.
//! Запросы числа игроков `0x7F809/0x7F80B` читают canonical player map и
//! публикуют World responses `0x5FA0A/0x5FA0C`; первая ветвь сохраняет ранний
//! nullable-client guard, вторая доходит до обычной nullable send-семантики.
//! `CMonsterList` хранит monster/drop registries selector-а `0x02`; runtime
//! lookup по original name становится общей базой concrete monster spawn.
//! `s_mapProxyRegion` теперь является owned ordered registry: `AddProxyRegion`
//! `0x0000AD10` сохраняет map-assignment, а `FindProxyRegion` `0x0000AD30` —
//! lookup/null. Proxy snapshot `0x0F` публикуется целиком до startup log.
//! `s_mapRegion` хранит concrete enum всех шести subtype-ов; ID/name lookup,
//! replace assignment, startup monster/NPC totals и GodsBattle registration
//! связаны с selector-ом `0x0E`, не стирая subtype state через base slicing.
//! `with_send_state/register_*/attach_*` являются явной assembly-границей
//! baseline и не снимают их псевдокод. Network setup передаётся отдельной
//! post-`LoadSetup*` проекцией. Windows thread handles заменены owned Tokio
//! tasks с тем же порядком exit/sleep/retry/join. `SO_SNDBUF=0` и World
//! reconnect player snapshot остаются локальными границами; nullable раннюю
//! ветвь `CMessage::SendAll` принимает отдельно как `Option`.

use std::collections::BTreeMap;
use std::ffi::CString;
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

use crate::gameserver::appserver::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck,
};
use crate::gameserver::appserver::country::countryhandler::CCountryHandler;
use crate::gameserver::appserver::country::countryparam::CCountryParam;
use crate::gameserver::appserver::country::countrywarsys::CountryWarSys;
use crate::gameserver::appserver::goods::cbattlefairyproperty::CBattleFairyProperty;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::goodswarmember::{
    CGoodsWarMember, GameGoodsWarMessageError, GameGoodsWarMessageReport,
    dispatch_game_goods_war_message,
};
use crate::gameserver::appserver::message::countrymessage::{
    CountryWarMessageDispatchError, GameCountryWarMessageReport, GameCountryWarRuntime,
    dispatch_game_country_war_message,
};
use crate::gameserver::appserver::message::depotmessage::{
    DepotMessageReport, dispatch_depot_message,
};
use crate::gameserver::appserver::message::gmamessage::{
    GmaMessageError, GmaMessageReport, dispatch_gma_message,
};
use crate::gameserver::appserver::message::gmmessage::{
    GmMessageError, GmMessageReport, dispatch_gm_message,
};
use crate::gameserver::appserver::message::onmsg_w2s_auction::{
    WorldAuctionStateMessageError, WorldAuctionStateMessageReport, dispatch_world_auction_state,
};
use crate::gameserver::appserver::message::organsysmessage::{
    GameOrganizingWarMessageError, GameOrganizingWarMessageReport, GameOrganizingWarRuntime,
    dispatch_game_organizing_war_message,
};
use crate::gameserver::appserver::message::sequencestring::{
    CSequenceRegistry, SequenceRegistryInitializationError,
};
use crate::gameserver::appserver::message::servermessage::on_billing_client_reconnected;
use crate::gameserver::appserver::message::servermessage::{
    GameServerMessageError, GameServerMessageReport, InitialRegionStartupContext,
    WarScheduleSetupContext, dispatch_server_message,
};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::organizingsystem::attackcitysys::CAttackCitySys;
use crate::gameserver::appserver::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationRect,
};
use crate::gameserver::appserver::organizingsystem::villagewarsys::CVillageWarSys;
use crate::gameserver::appserver::player::{
    BattleFairyCombineReport, BattleFairyDeathReport, BattleFairyEquipmentMutationReport,
    BattleFairyFollowReport, BattleFairySkillRequest, BattleFairySkillRequestFacts,
    BattleFairySkillRequestReport, BattleFairySkillResetReport, BattleFairySummonReport,
    BattleFairyWarSoulAction, CPlayer, PlayerCombatProperties, PlayerEquipmentAddReport,
    PlayerEquipmentAddRuntimeFacts, PlayerEquipmentRemoveReport, PlayerEquipmentRemoveRuntimeFacts,
    PlayerHonorResetReport,
};
use crate::gameserver::appserver::proxyserverregion::CProxyServerRegion;
use crate::gameserver::appserver::servercityregion::CServerCityRegion;
use crate::gameserver::appserver::servercountryregion::CServerCountryRegion;
use crate::gameserver::appserver::servercountryregion::CountryBattleStateBlock;
use crate::gameserver::appserver::servergodsbattleregion::{
    CGodsBattleMgr, CServerGodsBattleRegion,
};
use crate::gameserver::appserver::servernationregion::{
    NationCarriageReturnOutcome, NationContend, NationContendArithmeticBlock,
    NationContendCancelOutcome, NationContendCaptureMutation, NationContendDamageMutation,
    NationMonsterDamageNotice, NationMoraleMutation, ServerNationRegion,
    classify_nation_morale_target,
};
use crate::gameserver::appserver::serverregion::{
    CServerRegion, RegionMembershipBlock, ServerRegionMonsterContext, ServerRegionNpcContext,
    ServerRegionNpcSetup, ServerRegionNpcSpawnBlock, ServerRegionNpcSpawnReport,
};
use crate::gameserver::appserver::servervillageregion::CServerVillageRegion;
use crate::gameserver::appserver::shape::{
    CShape, MoveCheckCellRegistry, ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapeResolver,
    ShapeView,
};
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::gameserver::honorranks::CHonorRanks;
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
use crate::public::aucitionroom::CAuctionRoom;
use crate::public::auctionnode::CGoodsNode;
use crate::public::ciqing::CCiQingSetup;
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::dupliregionsetup::CDupliRegionSetup;
use crate::public::equipmentcomposelist::EquipmentComposeList;
use crate::public::guid::CGuid;
use crate::public::mystringtable::{
    MyStringTable, MyStringTableDecodeError, MyStringTableDecodeReport,
};
use crate::public::netsessionmanager::{
    CNetSessionManager, NetSessionManagerVariant, NetSessionRunReport,
};
use crate::public::taozhuangsetup::CTaoZhuangSetup;
use crate::public::wordsfilter::CWordsFilter;
use crate::setup::cbattlefairyexpconfig::CBattleFairyExpConfig;
use crate::setup::changebody::CChangeBodyConf;
use crate::setup::contributesetup::CContributeSetup;
use crate::setup::emotion::CEmotion;
use crate::setup::fairyexpconf::CFairyExpConf;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::gmlist::CGMList;
use crate::setup::goodsdestructionconfig::GoodsDestroySetup;
use crate::setup::hitlevelsetup::CHitLevelSetup;
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::incrementshoplist::CIncrementShopList;
use crate::setup::leitingsetup::CThingSetup;
use crate::setup::lingbao::CLingBaoSetup;
use crate::setup::logsystem::CLogSystem;
use crate::setup::monsterlist::{
    MonsterDropRegistry, MonsterListDecodeError, MonsterListDecodeReport, MonsterProperties,
    MonsterRegistry, decode_monster_list, get_monster_property_by_origin_name,
};
use crate::setup::newskillmonsterlist::NewSkillMonsterConf;
use crate::setup::playerlist::CPlayerList;
use crate::setup::preciousboxconf::PreciousBoxConf;
use crate::setup::prisonconf::PrisonConf;
use crate::setup::questsystem::CQuestSystem;
use crate::setup::regionrouter::RegionRouter;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::setup::tradelist::CTradeList;
use crate::transport::bind_tcp_ipv4;

const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const DEFAULT_SOCKET_TYPE: i32 = 1;
const WORLD_REGISTRATION: i32 = 0x0005_FA01;
const BILLING_REGISTRATION: i32 = 0x000E_F101;
const GAME_RELEASE_PLAYER_SAVE_MESSAGE: i32 = 0x0005_FB02;
const GAME_AUCTION_GOODS_SYNC_MESSAGE: i32 = 0x0006_0807;
const GAME_AUCTION_STATE_REQUEST_MESSAGE: i32 = 0x0006_0808;
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

pub(crate) trait GameScriptResourceContext {
    /// Выполняет исходный `CScript::LoadFunction(nullptr, data)`.
    fn load_function_list(&mut self, data: &[u8]);

    /// Создаёт новый global `CVariableList` и декодирует его с локальной копией
    /// cursor, не меняя внешний message cursor.
    fn load_general_variables(&mut self, source: &[u8], cursor: usize);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameSingleFilePublication {
    Published,
    RepeatedOwnerFreed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterBasePropertyRefreshReport {
    pub(crate) monsters: usize,
    pub(crate) resolved: usize,
    pub(crate) missing: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionOwner {
    Base(CServerRegion),
    Village(CServerVillageRegion),
    City(CServerCityRegion),
    Country(CServerCountryRegion),
    Nation(ServerNationRegion),
    GodsBattle(CServerGodsBattleRegion),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameWarRegionHandle {
    Local(i32),
    Proxy(i32),
}

pub(crate) struct GameWarStartupOwners {
    pub(crate) attack_city: CAttackCitySys,
    pub(crate) village: CVillageWarSys,
    pub(crate) country: CountryWarSys,
    pub(crate) four_nation: CFourNationWarSys,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleTopTenRequestReport {
    pub(crate) player_id: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

pub(crate) trait NationCombatContext: ServerRegionNpcContext {
    fn now_milliseconds(&mut self) -> u32;
    fn add_log_text(&mut self, text: &[u8]);
    fn put_debug_string(&mut self, text: &[u8]);

    /// Выполняет concrete player-origin `CMessage::SendToAround`, включая
    /// соседние areas и удалённых team members исходного runtime-а.
    fn send_nation_player_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock>;
}

pub(crate) trait NationContendContext:
    NationCombatContext + ServerRegionMonsterContext
{
    fn run_base_region_ai(&mut self, region: &mut CServerRegion);

    /// Выполняет concrete `CMessage::SendToAround` для двух magic-stone
    /// сообщений до virtual удаления исходного NPC.
    fn send_nation_magic_stone_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        message: &CMessage,
    ) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMonsterDamageOutcome {
    AttackerMissing,
    AttackerUnavailable,
    MonsterMissing,
    MonsterUnavailable,
    MonsterPropertyMissing,
    CountryOutsideNation,
    SameCountry,
    NoFirstHitNotice,
    Notice(NationMonsterDamageNotice),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMonsterDamageReport {
    pub(crate) region_id: i32,
    pub(crate) monster_id: i32,
    pub(crate) attacker_player_id: i32,
    pub(crate) outcome: NationMonsterDamageOutcome,
    pub(crate) world_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationCarriageReturnReport {
    pub(crate) region_id: i32,
    pub(crate) requested_country: i32,
    pub(crate) outcome: NationCarriageReturnOutcome,
    pub(crate) morale_delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationContendEnterOutcome {
    PlayerMissing,
    PlayerUnavailable,
    CountryAlreadyOwnsSymbol,
    CountryContenderExists { contender_player_id: i32 },
    Entered { first_for_country: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendEnterReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: NationContendEnterOutcome,
    pub(crate) deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<Result<i32, ShapeCoordinateBlock>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendCancelReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: Option<NationContendCancelOutcome>,
    pub(crate) delivery: Option<i32>,
    pub(crate) state_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendCancelAllReport {
    pub(crate) time_deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<(i32, Result<i32, ShapeCoordinateBlock>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendDamageReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) mutation: Result<Option<NationContendDamageMutation>, NationContendArithmeticBlock>,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationContendCompletionOutcome {
    PlayerMissing,
    PlayerUnavailable,
    CountryOutsideNation,
    Captured(NationContendCaptureMutation),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMagicStoneTransitionOutcome {
    NpcMissing,
    NpcNameAmbiguous { matches: usize },
    NpcCoordinateBlocked(ShapeCoordinateBlock),
    NpcRemovalBlocked(RegionMembershipBlock),
    MonsterPropertyMissing,
    MonsterSpawnBlocked(RegionMembershipBlock),
    Spawned { monster_id: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMagicStoneTransitionReport {
    pub(crate) country: u8,
    pub(crate) npc_id: Option<i32>,
    pub(crate) explosion_delivery: Option<i32>,
    pub(crate) removal_delivery: Option<i32>,
    pub(crate) outcome: NationMagicStoneTransitionOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendAiReport {
    pub(crate) region_id: i32,
    pub(crate) magic_stone_transitions: Vec<NationMagicStoneTransitionReport>,
    pub(crate) progress_deliveries: Vec<(i32, i32, i32)>,
    pub(crate) completed: Option<NationContend>,
    pub(crate) completion_outcome: Option<NationContendCompletionOutcome>,
    pub(crate) completion_deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<(i32, Result<i32, ShapeCoordinateBlock>)>,
    pub(crate) top_info_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) treasure_spawns: Vec<Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationPlayerDeathContendOutcome {
    NotContending,
    MissingReset,
    RemovedLegacyReturnIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerDeathReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) timing_finished: bool,
    pub(crate) contend_outcome: NationPlayerDeathContendOutcome,
    pub(crate) contend_state_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) contend_time_delivery: Option<i32>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) died_state_time_ms: i32,
    pub(crate) died_state_start_time_ms: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerDiedStatePublication {
    pub(crate) player_id: i32,
    pub(crate) state: bool,
    pub(crate) self_delivery: i32,
    pub(crate) around_delivery: Result<i32, ShapeCoordinateBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationPlayerDiedStateTick {
    Inactive,
    Waiting {
        elapsed_ms: u32,
    },
    Advanced {
        elapsed_ms: u32,
        remaining_ms: i32,
        time_delivery: Option<i32>,
    },
    Expired {
        elapsed_ms: u32,
        time_delivery: Option<i32>,
        state_publication: NationPlayerDiedStatePublication,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMonsterDeathOutcome {
    KillerMissing,
    MonsterMissing,
    MonsterPropertyMissing,
    Unclassified,
    CountryOutsideNation,
    MoraleChanged(NationMoraleMutation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMonsterDeathReport {
    pub(crate) region_id: i32,
    pub(crate) monster_id: i32,
    pub(crate) killer_player_id: i32,
    pub(crate) outcome: NationMonsterDeathOutcome,
    pub(crate) morale_delivery: Option<i32>,
    pub(crate) first_guard_delivery: Option<i32>,
    pub(crate) nation_fail_deliveries: Vec<Result<i32, SendMessageError>>,
    pub(crate) yu_ying_shi_spawns: Vec<NationYuYingShiSpawnReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationYuYingShiSpawnReport {
    pub(crate) country: u8,
    pub(crate) notify_country: u8,
    pub(crate) spawn: Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock>,
    pub(crate) region_delivery: Option<i32>,
    pub(crate) country_deliveries: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GameMainLoopProfile {
    pub(crate) script_ms: u32,
    pub(crate) ai_ms: u32,
    pub(crate) message_ms: u32,
    pub(crate) session_ms: u32,
    pub(crate) net_session_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopRuntimeLog {
    Compact {
        elapsed_ms: u32,
        ai_calls: i32,
    },
    Profiled {
        elapsed_ms: u32,
        ai_calls: i32,
        profile: GameMainLoopProfile,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopStage {
    RefreshInfo,
    RuntimeLog,
    Script,
    Ai,
    Message,
    Session,
    NetSession,
    Auction,
    Wait { duration_ms: u32 },
    LagWarning { resync_tick_ms: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopOutcome {
    Continue,
    ExitRequested,
}

#[must_use = "MainLoop report сохраняет ordering, pacing и legacy return"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameMainLoopReport<RegionRuntimeError> {
    pub(crate) outcome: GameMainLoopOutcome,
    pub(crate) return_value: i32,
    pub(crate) sampled_tick_ms: u32,
    pub(crate) ai_tick: i32,
    pub(crate) stages: Vec<GameMainLoopStage>,
    pub(crate) next_deadline_ms: Option<u32>,
    pub(crate) signed_lag_ms: Option<i32>,
    pub(crate) messages: Option<GameProcessMessagesReport<RegionRuntimeError>>,
    pub(crate) net_sessions: Option<NetSessionRunReport>,
    pub(crate) auction: Option<GameAuctionRunReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameAuctionRunOutcome {
    FeatureDisabled,
    TickNotDue { elapsed_ms: u32 },
    Processed { state_expired: bool },
}

#[must_use = "RunAuction report сохраняет time gates и оба World effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameAuctionRunReport {
    pub(crate) outcome: GameAuctionRunOutcome,
    pub(crate) sampled_tick_ms: Option<u32>,
    pub(crate) sampled_wall_time_seconds: Option<u32>,
    pub(crate) synchronized_goods: Vec<CGuid>,
    pub(crate) goods_sync: Option<Result<i32, SendMessageError>>,
    pub(crate) state_request: Option<Result<i32, SendMessageError>>,
}

#[must_use = "ProcessMessage report сохраняет server, auction, GM, GMA и depot effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameProcessMessagesReport<RegionRuntimeError> {
    pub(crate) legacy_return: i32,
    pub(crate) auction_states:
        Vec<Result<WorldAuctionStateMessageReport, WorldAuctionStateMessageError>>,
    pub(crate) gm_messages: Vec<Result<GmMessageReport, GmMessageError>>,
    pub(crate) gma_messages: Vec<Result<GmaMessageReport, GmaMessageError>>,
    pub(crate) depot_messages: Vec<DepotMessageReport>,
    pub(crate) organizing_war_messages:
        Vec<Result<GameOrganizingWarMessageReport, GameOrganizingWarMessageError>>,
    pub(crate) country_war_messages: Vec<
        Result<
            GameCountryWarMessageReport,
            CountryWarMessageDispatchError<CountryBattleStateBlock>,
        >,
    >,
    pub(crate) goods_war_messages: Vec<Result<GameGoodsWarMessageReport, GameGoodsWarMessageError>>,
    pub(crate) server_messages:
        Vec<Result<GameServerMessageReport, GameServerMessageError<RegionRuntimeError>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameKickPlayerReport {
    pub(crate) player_id: i32,
    pub(crate) command_result: i32,
    pub(crate) legacy_return: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameKickAroundOutcome {
    TargetMissing,
    ServerRegionMissing,
    CoordinateBlocked(ShapeCoordinateBlock),
    RegionMissing,
    UnresolvedShape(ShapeIdentity),
    LookupBlocked(RegionMembershipBlock),
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameKickAroundReport {
    pub(crate) outcome: GameKickAroundOutcome,
    pub(crate) target_player_id: Option<i32>,
    pub(crate) server_region_id: Option<i32>,
    pub(crate) window_origin: Option<(i32, i32)>,
    pub(crate) matched_player_ids: Vec<i32>,
    pub(crate) kicks: Vec<GameKickPlayerReport>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GameMainLoopState {
    initialized: bool,
    current_tick_ms: u32,
    runtime_log_tick_ms: u32,
    refresh_info_tick_ms: u32,
    calls_since_runtime_log: i32,
    ai_tick: i32,
    profile: GameMainLoopProfile,
    pacing_initialized: bool,
    pacing_deadline_ms: u32,
}

/// Concrete Script/region/AI/session owners подключаются сюда по мере их
/// материализации; message routing уже исполняется самим `CGame`, а region
/// decoder получает тот же live factory-контекст без отдельного shadow state.
pub(crate) trait GameMainLoopRuntime:
    GameMessageHandlers
    + GameScriptResourceContext
    + InitialRegionStartupContext
    + GameOrganizingWarRuntime
    + GameCountryWarRuntime
{
    fn exit_requested(&self) -> bool;
    fn tick_interval_ms(&self) -> u32;
    fn get_tick_ms(&mut self) -> u32;
    fn wall_time_seconds(&mut self) -> u32;
    fn refresh_info_text(&mut self, game: &CGame);
    fn add_runtime_log(&mut self, log: GameMainLoopRuntimeLog);
    fn script_loop(&mut self, game: &mut CGame);
    fn ai(&mut self, game: &mut CGame);
    fn session_factory_ai(&mut self, game: &mut CGame);
    fn wait(&mut self, duration_ms: u32);
    fn output_debug(&mut self, message: &'static str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseExternalOwner {
    ScriptFunctions,
    GeneralVariables,
    SocketRuntime,
    PkSystem,
    BaseMessageRuntime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseDebug {
    ServerExiting,
    PlayerSaveFailed {
        player_id: i32,
        processed: usize,
        total: usize,
    },
    PlayersSaved {
        processed: usize,
        total: usize,
    },
    CityRegionSaved,
    PlayersAndRegionsCleared,
    ProxyRegionsCleared,
    ScriptDataCleared,
    ServerExited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseEvent {
    Debug(GameReleaseDebug),
    ReconnectTasksStopped,
    PlayerSave {
        player_id: i32,
        saved: bool,
    },
    CityRegionSaved,
    PlayersCleared {
        count: usize,
    },
    RegionsCleared {
        count: usize,
    },
    ProxyRegionsCleared {
        count: usize,
    },
    ScriptDataCleared {
        function_list: bool,
        variable_list: bool,
        files: usize,
    },
    ExternalOwner(GameReleaseExternalOwner),
    SkillFactoryCleared,
    GoodsFactoryReleased,
    NetworkServerWorkerStopped {
        present: bool,
    },
    WorldClientReleased {
        present: bool,
    },
    BillingClientReleased {
        present: bool,
    },
    NetworkServerReleased {
        present: bool,
    },
    NetSessionsReleased {
        count: usize,
    },
    IncrementShopReleased,
    QuestSystemReleased,
    SequenceRegistryCleared {
        count: usize,
    },
    PlayerRanksReleased {
        present: bool,
    },
    WordsFilterReleased,
    HonorRanksReleased,
    GoodsWarReleased {
        present: bool,
    },
    CountryHandlerReleased,
    CountryParamReleased,
    AttackCitySystemReleased {
        schedules: usize,
    },
    VillageWarSystemReleased {
        schedules: usize,
    },
}

#[must_use = "Release report сохраняет полный достигнутый teardown ordering"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameReleaseReport {
    pub(crate) events: Vec<GameReleaseEvent>,
    /// `Release` объявлен как int, но достигнутый tail возвращает результат
    /// security-cookie thunk; gameplay caller значение игнорирует.
    pub(crate) legacy_return: Option<i32>,
}

pub(crate) trait GameReleaseRuntime {
    fn put_debug_string(&mut self, message: GameReleaseDebug);
    fn save_player(&mut self, player: &CPlayer, message_type: i32, save_flag: i32) -> bool;
    fn save_city_region(&mut self, game: &CGame, region_id: i32);
    fn release_external_owner(&mut self, owner: GameReleaseExternalOwner);
    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer);
}

pub(crate) trait GameThreadRuntime: GameMainLoopRuntime + GameReleaseRuntime {
    fn runtime_paths(&self) -> GameRuntimePaths;
    fn sequence_seed_ms(&mut self) -> u32;
    fn initialize_com(&mut self);
    fn signal_game_thread_exit(&mut self);
    fn post_process_close(&mut self);
    fn uninitialize_com(&mut self);
}

#[must_use = "GameThread report сохраняет Init/MainLoop/Release lifecycle"]
#[derive(Debug)]
pub(crate) struct GameThreadReport {
    pub(crate) initialization:
        Result<GameInitializationReport, GameInitializationThroughBillingError>,
    pub(crate) main_loop_calls: usize,
    pub(crate) release: GameReleaseReport,
}

impl ServerRegionOwner {
    pub(crate) const fn base(&self) -> &CServerRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => &region.war.base,
            Self::City(region) => &region.war.base,
            Self::Country(region) => &region.base,
            Self::Nation(region) => &region.war.base,
            Self::GodsBattle(region) => &region.war.base,
        }
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CServerRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => &mut region.war.base,
            Self::City(region) => &mut region.war.base,
            Self::Country(region) => &mut region.base,
            Self::Nation(region) => &mut region.war.base,
            Self::GodsBattle(region) => &mut region.war.base,
        }
    }

    pub(crate) const fn region_id(&self) -> i32 {
        self.base().id
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.base().region.get_name()
    }

    pub(crate) const fn is_gods_battle(&self) -> bool {
        matches!(self, Self::GodsBattle(_))
    }

    /// Сохраняет virtual dispatch: war-derived owners дополнительно сбрасывают
    /// symbol state, base/country используют `CServerRegion` реализацию.
    pub(crate) fn reset_war_state(&mut self, war_number: i32, state: i32) {
        match self {
            Self::Base(region) => region.reset_war_state(war_number, state),
            Self::Village(region) => region.war.reset_war_state(war_number, state),
            Self::City(region) => region.war.reset_war_state(war_number, state),
            Self::Country(region) => region.base.reset_war_state(war_number, state),
            Self::Nation(region) => region.war.reset_war_state(war_number, state),
            Self::GodsBattle(region) => region.war.reset_war_state(war_number, state),
        }
    }
}

impl WarScheduleSetupContext for CGame {
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        if self.find_region(region_id).is_some() {
            Some(GameWarRegionHandle::Local(region_id))
        } else {
            self.find_proxy_region(region_id)
                .map(|_| GameWarRegionHandle::Proxy(region_id))
        }
    }

    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.find_region_mut(region_id) {
                    region.reset_war_state(war_number, state);
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.find_proxy_region_mut(region_id) {
                    region.reset_war_state(war_number, state);
                }
            }
        }
    }

    fn region_country(&self, region: Self::Region) -> u8 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .find_region(region_id)
                .map(|region| region.base().country)
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .find_proxy_region(region_id)
                .map(CProxyServerRegion::country)
                .unwrap_or(0),
        }
    }

    fn set_region_country(&mut self, region: Self::Region, country: u8) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.find_region_mut(region_id) {
                    region.base_mut().country = country;
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.find_proxy_region_mut(region_id) {
                    region.set_country(country);
                }
            }
        }
    }

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.find_region(region_id),
            Some(ServerRegionOwner::Country(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn find_nation_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.find_region(region_id),
            Some(ServerRegionOwner::Nation(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn reset_nation_war_state(&mut self, region: Self::Region, index: i32, state: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) {
            region.war.reset_war_state(index, state);
        }
    }

    fn set_nation_relive_rects(&mut self, region: Self::Region, rects: [FourNationRect; 5]) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) {
            region.set_relive_rects(rects);
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
    goods_factory: CGoodsFactory,
    skill_factory: CSkillFactory,
    monster_registry: MonsterRegistry,
    monster_drop_registry: MonsterDropRegistry,
    thing_setup: CThingSetup,
    increment_shop_list: CIncrementShopList,
    contribute_setup: CContributeSetup,
    log_system: CLogSystem,
    gm_list: CGMList,
    da_kong_xiang_qian: CDaKongXiangQian,
    globe_setup: GlobeSetupSnapshot,
    region_router: RegionRouter,
    area_width: i32,
    area_height: i32,
    auction_room: CAuctionRoom<CGoodsNode>,
    auction_now: bool,
    auction_last_check_seconds: u32,
    auction_tick_ms: u32,
    function_list_file_data: Option<Vec<u8>>,
    variable_list_file_data: Option<Vec<u8>>,
    script_file_data: BTreeMap<Vec<u8>, Vec<u8>>,
    string_table: MyStringTable,
    quest_system: CQuestSystem,
    country_param: CCountryParam,
    country_handler: CCountryHandler,
    attack_city_sys: CAttackCitySys,
    village_war_sys: CVillageWarSys,
    country_war_sys: CountryWarSys,
    four_nation_war_sys: CFourNationWarSys,
    emotion: CEmotion,
    region_setup: CRegionSetup,
    hit_level_setup: CHitLevelSetup,
    prison_conf: PrisonConf,
    precious_box_conf: PreciousBoxConf,
    fairy_exp_conf: CFairyExpConf,
    battle_fairy_exp_config: CBattleFairyExpConfig,
    battle_fairy_property: CBattleFairyProperty,
    equipment_compose_list: EquipmentComposeList,
    words_filter: CWordsFilter,
    jjc_level_data: BTreeMap<i32, i32>,
    tao_zhuang_setup: CTaoZhuangSetup,
    ci_qing_setup: CCiQingSetup,
    ling_bao_setup: CLingBaoSetup,
    gods_battle_mgr: CGodsBattleMgr,
    synthesis: CSynthesis,
    new_skill_monster_conf: NewSkillMonsterConf,
    goods_destroy_setup: GoodsDestroySetup,
    change_body_conf: CChangeBodyConf,
    honor_eliminate_config: HonorElimilateConfig,
    dupli_region_setup: Option<CDupliRegionSetup>,
    move_check_cells: MoveCheckCellRegistry,
    player_ranks: Option<CPlayerRanks>,
    honor_ranks: CHonorRanks,
    goods_war: Option<CGoodsWarMember>,
    id_index: u8,
    team_id_counter: u32,
    login_server_id: i32,
    world_server_id: i32,
    network_setup: Option<GameNetworkSetup>,
    world_client: Option<CMyNetClient>,
    billing_client: Option<CMyNetClient>,
    net_server: Option<CMyNetServer>,
    world_reconnect_task: Option<GameReconnectTask>,
    billing_reconnect_task: Option<GameReconnectTask>,
    net_session_manager: CNetSessionManager,
    players: BTreeMap<i32, CPlayer>,
    regions: BTreeMap<i32, ServerRegionOwner>,
    proxy_regions: BTreeMap<i32, CProxyServerRegion>,
    initial_total_monsters: i32,
    initial_total_npcs: i32,
    team_session_ids: BTreeMap<u32, i32>,
    main_loop_state: GameMainLoopState,
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
            goods_factory: CGoodsFactory::default(),
            skill_factory: CSkillFactory::default(),
            monster_registry: MonsterRegistry::new(),
            monster_drop_registry: MonsterDropRegistry::new(),
            thing_setup: CThingSetup::default(),
            increment_shop_list: CIncrementShopList::default(),
            contribute_setup: CContributeSetup::default(),
            log_system: CLogSystem::default(),
            gm_list: CGMList::default(),
            da_kong_xiang_qian: CDaKongXiangQian::default(),
            globe_setup: GlobeSetupSnapshot::default(),
            region_router: RegionRouter::default(),
            area_width: 15,
            area_height: 15,
            auction_room: CAuctionRoom::new(),
            auction_now: false,
            auction_last_check_seconds: 0,
            auction_tick_ms: 0,
            function_list_file_data: None,
            variable_list_file_data: None,
            script_file_data: BTreeMap::new(),
            string_table: MyStringTable::new(),
            quest_system: CQuestSystem::default(),
            country_param: CCountryParam::default(),
            country_handler: CCountryHandler::default(),
            attack_city_sys: CAttackCitySys::default(),
            village_war_sys: CVillageWarSys::default(),
            country_war_sys: CountryWarSys::default(),
            four_nation_war_sys: CFourNationWarSys::default(),
            emotion: CEmotion::default(),
            region_setup: CRegionSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            prison_conf: PrisonConf::default(),
            precious_box_conf: PreciousBoxConf::default(),
            fairy_exp_conf: CFairyExpConf::default(),
            battle_fairy_exp_config: CBattleFairyExpConfig::default(),
            battle_fairy_property: CBattleFairyProperty::default(),
            equipment_compose_list: EquipmentComposeList::default(),
            words_filter: CWordsFilter::default(),
            jjc_level_data: BTreeMap::new(),
            tao_zhuang_setup: CTaoZhuangSetup::default(),
            ci_qing_setup: CCiQingSetup::default(),
            ling_bao_setup: CLingBaoSetup::default(),
            gods_battle_mgr: CGodsBattleMgr::default(),
            synthesis: CSynthesis::default(),
            new_skill_monster_conf: NewSkillMonsterConf::default(),
            goods_destroy_setup: GoodsDestroySetup::default(),
            change_body_conf: CChangeBodyConf::default(),
            honor_eliminate_config: HonorElimilateConfig::default(),
            dupli_region_setup: None,
            move_check_cells: MoveCheckCellRegistry::new(),
            player_ranks: None,
            honor_ranks: CHonorRanks::default(),
            goods_war: None,
            id_index: 0,
            team_id_counter: 1,
            login_server_id: 0,
            world_server_id: 0,
            network_setup: None,
            world_client: None,
            billing_client: None,
            net_server: None,
            world_reconnect_task: None,
            billing_reconnect_task: None,
            net_session_manager: CNetSessionManager::new(NetSessionManagerVariant::GameServer),
            players: BTreeMap::new(),
            regions: BTreeMap::new(),
            proxy_regions: BTreeMap::new(),
            initial_total_monsters: 0,
            initial_total_npcs: 0,
            team_session_ids: BTreeMap::new(),
            main_loop_state: GameMainLoopState::default(),
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
        // C++ constructor запоминал `_time` при создании process singleton-а.
        // Safe owner получает первый доказанный wall-clock на входе `Init`;
        // пока state=false, эта разница ненаблюдаема, а true-ветвь всегда
        // перезаписывает timestamp через exact `SetAuctionState`.
        self.auction_last_check_seconds = wall_time_seconds;
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

    pub(crate) const fn current_net_server(&self) -> Option<&CMyNetServer> {
        self.net_server.as_ref()
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

    /// Terminal startup присваивает login ID до чтения world ID; раздельные
    /// setter-ы сохраняют partial effect malformed хвоста.
    pub(crate) const fn set_login_server_id(&mut self, login_server_id: i32) {
        self.login_server_id = login_server_id;
    }

    pub(crate) const fn set_world_server_id(&mut self, world_server_id: i32) {
        self.world_server_id = world_server_id;
    }

    pub(crate) const fn server_ids(&self) -> (i32, i32) {
        (self.login_server_id, self.world_server_id)
    }

    pub(crate) const fn set_id_index(&mut self, id_index: u8) {
        self.id_index = id_index;
    }

    pub(crate) const fn id_index(&self) -> u8 {
        self.id_index
    }

    /// Выдаёт исходный 24-bit team counter с Game index в старшем байте.
    /// Process-static C++ counter хранится здесь, поскольку `CGame` — singleton.
    pub(crate) fn get_team_id(&mut self, session_id: i32) -> u32 {
        if session_id == 0 {
            return 0;
        }

        let result = self.team_id_counter | (u32::from(self.id_index) << 24);
        self.team_id_counter += 1;
        if self.team_id_counter >= 0x00ff_ffff {
            self.team_id_counter = 1;
        }
        result
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

    pub(crate) const fn goods_factory(&self) -> &CGoodsFactory {
        &self.goods_factory
    }

    pub(crate) const fn goods_factory_mut(&mut self) -> &mut CGoodsFactory {
        &mut self.goods_factory
    }

    /// Создаёт достигнутый GameServer goods core; исходный код игнорировал
    /// ошибку `CoCreateGuid`, поэтому failure оставляет нулевой GUID.
    pub(crate) fn create_goods_core(&mut self, goods_index: u32) -> Option<CGoods> {
        let random_state = &mut self.random_state;
        self.goods_factory.create_goods_core(
            goods_index,
            |upper_bound| game_legacy_random(random_state, upper_bound),
            || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
        )
    }

    pub(crate) const fn skill_factory(&self) -> &CSkillFactory {
        &self.skill_factory
    }

    pub(crate) const fn skill_factory_mut(&mut self) -> &mut CSkillFactory {
        &mut self.skill_factory
    }

    pub(crate) fn decode_monster_list(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<MonsterListDecodeReport, MonsterListDecodeError> {
        decode_monster_list(
            &mut self.monster_registry,
            &mut self.monster_drop_registry,
            source,
            cursor,
        )
    }

    pub(crate) fn find_monster_property_by_origin_name(
        &self,
        origin_name: &[u8],
    ) -> Option<&MonsterProperties> {
        get_monster_property_by_origin_name(&self.monster_registry, origin_name)
    }

    /// Эквивалент `RefreashAllMonsterBaseProperty`: old raw pointers заменены
    /// key lookup-ами, поэтому проход подтверждает состояние всех live owners.
    pub(crate) fn refresh_all_monster_base_property(&self) -> MonsterBasePropertyRefreshReport {
        let mut report = MonsterBasePropertyRefreshReport::default();
        for region in self.regions.values() {
            for key in region.base().monster_base_property_keys() {
                report.monsters = report.monsters.wrapping_add(1);
                if self.find_monster_property_by_origin_name(key).is_some() {
                    report.resolved = report.resolved.wrapping_add(1);
                } else {
                    report.missing = report.missing.wrapping_add(1);
                }
            }
        }
        report
    }

    /// Сохраняет исходный `std::map::operator[] = pointer`: повторный ID
    /// заменяет опубликованный proxy-owner.
    pub(crate) fn add_proxy_region(&mut self, region: CProxyServerRegion) -> bool {
        self.proxy_regions.insert(region.get_id(), region).is_some()
    }

    pub(crate) fn find_proxy_region(&self, region_id: i32) -> Option<&CProxyServerRegion> {
        self.proxy_regions.get(&region_id)
    }

    pub(crate) fn find_proxy_region_mut(
        &mut self,
        region_id: i32,
    ) -> Option<&mut CProxyServerRegion> {
        self.proxy_regions.get_mut(&region_id)
    }

    pub(crate) fn take_war_startup_owners(&mut self) -> GameWarStartupOwners {
        GameWarStartupOwners {
            attack_city: std::mem::take(&mut self.attack_city_sys),
            village: std::mem::take(&mut self.village_war_sys),
            country: std::mem::take(&mut self.country_war_sys),
            four_nation: std::mem::take(&mut self.four_nation_war_sys),
        }
    }

    pub(crate) fn restore_war_startup_owners(&mut self, owners: GameWarStartupOwners) {
        self.attack_city_sys = owners.attack_city;
        self.village_war_sys = owners.village;
        self.country_war_sys = owners.country;
        self.four_nation_war_sys = owners.four_nation;
    }

    pub(crate) fn add_region(&mut self, mut region: ServerRegionOwner) -> bool {
        if let ServerRegionOwner::Nation(nation) = &mut region {
            nation.set_country_names(std::array::from_fn(|country| {
                self.globe_setup
                    .country_name(country as u8)
                    .unwrap_or_default()
                    .to_vec()
            }));
        }
        self.regions.insert(region.region_id(), region).is_some()
    }

    pub(crate) fn find_region(&self, region_id: i32) -> Option<&ServerRegionOwner> {
        self.regions.get(&region_id)
    }

    pub(crate) fn find_region_mut(&mut self, region_id: i32) -> Option<&mut ServerRegionOwner> {
        self.regions.get_mut(&region_id)
    }

    pub(crate) fn take_region_owner(&mut self, region_id: i32) -> Option<ServerRegionOwner> {
        self.regions.remove(&region_id)
    }

    pub(crate) fn restore_region_owner(&mut self, region: ServerRegionOwner) {
        let replaced = self.regions.insert(region.region_id(), region);
        debug_assert!(
            replaced.is_none(),
            "scoped region owner не заменяет live owner"
        );
    }

    /// Script function `9304 / kScriptFunctionNationWarSendPlayerId`:
    /// father-region берётся у current script player, а timing запускается
    /// для переданного player ID только в concrete local Nation owner.
    pub(crate) fn script_nation_war_send_player_id(
        &mut self,
        script_player_id: i32,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some(region_id) = self
            .find_player(script_player_id)
            .and_then(CPlayer::server_region_id)
        else {
            return false;
        };
        self.start_nation_war_player_timing(region_id, player_id, now_ms)
    }

    /// Exact `ServerNationRegion::OnPlayerTimgingStart`, включая
    /// morale snapshot `0xBF818` до мутации timing record.
    pub(crate) fn start_nation_war_player_timing(
        &mut self,
        region_id: i32,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some((country, can_start)) = self
            .find_player(player_id)
            .map(|player| (player.country(), player.can_start_nation_war_timing()))
        else {
            return false;
        };
        if !can_start {
            return false;
        }
        let Some((morale, failed)) = self.find_region(region_id).and_then(|region| match region {
            ServerRegionOwner::Nation(region) => Some((*region.morale(), *region.nation_failed())),
            _ => None,
        }) else {
            return false;
        };

        let snapshot = self.four_nation_morale_snapshot(morale, failed);
        let _delivery = snapshot.send_to_player(self.net_server(), player_id);

        let timing_exists = self
            .find_region(region_id)
            .and_then(|region| match region {
                ServerRegionOwner::Nation(region) => Some(region.has_player_timing(player_id)),
                _ => None,
            })
            .unwrap_or(false);
        let now_ms = now_ms();
        let country_figure = if timing_exists {
            0
        } else {
            self.country_handler_mut()
                .country_mut(country)
                .map(|country| country.identity_for_player(player_id))
                .unwrap_or_default()
        };
        let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) else {
            return false;
        };
        region.start_player_timing(
            player_id,
            i32::from(country),
            i32::from(country_figure),
            now_ms,
        );
        true
    }

    /// Caller-side `CPlayer::OnLost`: changing-server ветвь не закрывает
    /// nation clock; ordinary loss передаёт `died=false`.
    pub(crate) fn finish_nation_war_timing_on_player_lost(
        &mut self,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some((region_id, changing_server)) = self
            .find_player(player_id)
            .and_then(|player| Some((player.server_region_id()?, player.in_changing_server())))
        else {
            return false;
        };
        if changing_server {
            return false;
        }
        self.finish_nation_war_player_timing(region_id, player_id, false, now_ms)
    }

    fn finish_nation_war_player_timing(
        &mut self,
        region_id: i32,
        player_id: i32,
        died: bool,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) else {
            return false;
        };
        region
            .finish_player_timing(player_id, died, now_ms)
            .is_some()
    }

    /// Script primitive `NationWar_CarriageBackTown` reaches the same typed
    /// Nation owner that combat callbacks use; non-Nation region keeps the
    /// original dynamic-cast no-op as `None`.
    pub(crate) fn script_nation_carriage_back_town(
        &mut self,
        region_id: i32,
        country: i32,
    ) -> Option<NationCarriageReturnReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let outcome = region.carriage_back_town(country);
        let morale_delivery =
            matches!(outcome, NationCarriageReturnOutcome::Applied(_)).then(|| {
                self.four_nation_morale_snapshot(*region.morale(), *region.nation_failed())
                    .send_to_region(Some(&region.war.base), None, self)
            });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationCarriageReturnReport {
            region_id,
            requested_country: country,
            outcome,
            morale_delivery,
        })
    }

    pub(crate) fn nation_enter_contend<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        max_time: u32,
        context: &mut Context,
    ) -> Option<NationContendEnterReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mut deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        let outcome = match self.find_player(player_id) {
            None => NationContendEnterOutcome::PlayerMissing,
            Some(player) if !player.can_attack_nation_monster() => {
                NationContendEnterOutcome::PlayerUnavailable
            }
            Some(player) => {
                let country = player.country();
                if region.flag_belong_to_id() == i32::from(country) {
                    deliveries.push(
                        self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS1130")),
                    );
                    NationContendEnterOutcome::CountryAlreadyOwnsSymbol
                } else if let Some(contender_player_id) =
                    region.contender_for_country(i32::from(country))
                {
                    if let Some(contender) = self.find_player(contender_player_id) {
                        let text = format_legacy_text_fields(
                            self.get_string_by_id(b"GS1131"),
                            &[contender.shape().base_object().get_name()],
                            0xff,
                        );
                        deliveries.push(self.send_nation_player_notice(player_id, &text));
                        NationContendEnterOutcome::CountryContenderExists {
                            contender_player_id,
                        }
                    } else {
                        self.finish_nation_contend_entry(
                            &mut region,
                            player_id,
                            country,
                            max_time,
                            context,
                            &mut deliveries,
                            &mut state_deliveries,
                        )
                    }
                } else {
                    self.finish_nation_contend_entry(
                        &mut region,
                        player_id,
                        country,
                        max_time,
                        context,
                        &mut deliveries,
                        &mut state_deliveries,
                    )
                }
            }
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendEnterReport {
            region_id,
            player_id,
            outcome,
            deliveries,
            state_deliveries,
        })
    }

    pub(crate) fn nation_cancel_contend_by_player_id<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationContendCancelReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let (outcome, delivery, state_delivery) = if self.find_player(player_id).is_none() {
            (None, None, None)
        } else {
            let outcome = region.cancel_contend_by_player_id(player_id);
            let (delivery, state_delivery) =
                if matches!(outcome, NationContendCancelOutcome::MissingReset) {
                    let state_delivery = self.set_nation_player_contend_state(
                        &region.war.base,
                        player_id,
                        false,
                        context,
                    );
                    (
                        Some(self.send_nation_contend_time(player_id, 0)),
                        state_delivery,
                    )
                } else {
                    (None, None)
                };
            (Some(outcome), delivery, state_delivery)
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendCancelReport {
            region_id,
            player_id,
            outcome,
            delivery,
            state_delivery,
        })
    }

    pub(crate) fn nation_cancel_all_contenders<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        context: &mut Context,
    ) -> Option<NationContendCancelAllReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mut time_deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        for player_id in region.contender_player_ids() {
            time_deliveries.push(self.send_nation_contend_time(player_id, 0));
            if let Some(delivery) =
                self.set_nation_player_contend_state(&region.war.base, player_id, false, context)
            {
                state_deliveries.push((player_id, delivery));
            }
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendCancelAllReport {
            time_deliveries,
            state_deliveries,
        })
    }

    fn finish_nation_contend_entry<Context: NationCombatContext>(
        &mut self,
        region: &mut ServerNationRegion,
        player_id: i32,
        country: u8,
        max_time: u32,
        context: &mut Context,
        deliveries: &mut Vec<i32>,
        state_deliveries: &mut Vec<Result<i32, ShapeCoordinateBlock>>,
    ) -> NationContendEnterOutcome {
        if matches!(
            region.cancel_contend_by_player_id(player_id),
            NationContendCancelOutcome::MissingReset
        ) {
            if let Some(delivery) =
                self.set_nation_player_contend_state(&region.war.base, player_id, false, context)
            {
                state_deliveries.push(delivery);
            }
            deliveries.push(self.send_nation_contend_time(player_id, 0));
        }
        let first_for_country = region.add_contend(
            player_id,
            i32::from(country),
            max_time as i32,
            context.now_milliseconds(),
        );
        if let Some(delivery) =
            self.set_nation_player_contend_state(&region.war.base, player_id, true, context)
        {
            state_deliveries.push(delivery);
        }
        deliveries.push(self.send_nation_contend_time(player_id, 0));
        if first_for_country && (1..=4).contains(&country) {
            let text = format_legacy_text_fields(
                self.get_string_by_id(b"GS1133"),
                &[region.country_name(country)],
                0xff,
            );
            deliveries.push(
                nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &text)
                    .send_to_region(Some(&region.war.base), None, self),
            );
        }
        deliveries
            .push(self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS1132")));
        NationContendEnterOutcome::Entered { first_for_country }
    }

    pub(crate) fn nation_contender_damaged(
        &mut self,
        region_id: i32,
        player_id: i32,
        damage: i32,
    ) -> Option<NationContendDamageReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mutation = match self.find_player(player_id) {
            Some(player) if player.can_attack_nation_monster() => region.damage_contender(
                player_id,
                damage,
                player.maximum_health(),
                self.globe_setup.contend_damage_time_factor(),
            ),
            _ => Ok(None),
        };
        let delivery = mutation.as_ref().ok().and_then(|mutation| {
            mutation.map(|mutation| self.send_nation_contend_time(player_id, mutation.percentage))
        });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendDamageReport {
            region_id,
            player_id,
            mutation,
            delivery,
        })
    }

    pub(crate) fn nation_contend_ai<Context: NationContendContext>(
        &mut self,
        region_id: i32,
        context: &mut Context,
    ) -> Option<Result<NationContendAiReport, NationContendArithmeticBlock>> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        context.run_base_region_ai(&mut region.war.base);
        let mut magic_stone_transitions = Vec::new();
        for country in region.take_due_magic_stone_transitions() {
            magic_stone_transitions.push(self.replace_nation_magic_stone(
                &mut region,
                country,
                context,
            ));
        }
        let advance = match region.advance_contenders(context.now_milliseconds()) {
            Ok(advance) => advance.unwrap_or_default(),
            Err(error) => {
                self.restore_region_owner(ServerRegionOwner::Nation(region));
                return Some(Err(error));
            }
        };
        let mut progress_deliveries = Vec::with_capacity(advance.progress.len());
        for (player_id, percentage) in advance.progress {
            let delivery = self.send_nation_contend_time(player_id, percentage);
            progress_deliveries.push((player_id, percentage, delivery));
        }

        let completed = advance.completed;
        let mut completion_outcome = None;
        let mut completion_deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        let mut top_info_delivery = None;
        let mut treasure_spawns = Vec::new();
        if let Some(contender) = completed {
            completion_deliveries.push(self.send_nation_contend_time(contender.player_id, 100));
            context.add_log_text(self.get_string_by_id(b"GS1072"));
            completion_outcome = Some(match self.find_player(contender.player_id) {
                None => NationContendCompletionOutcome::PlayerMissing,
                Some(player) if !player.can_attack_nation_monster() => {
                    NationContendCompletionOutcome::PlayerUnavailable
                }
                Some(player) => {
                    let country = player.country();
                    match region.capture_contend_symbol(country) {
                        None => NationContendCompletionOutcome::CountryOutsideNation,
                        Some(capture) => {
                            for cancelled_player_id in &capture.cancelled_player_ids {
                                completion_deliveries
                                    .push(self.send_nation_contend_time(*cancelled_player_id, 0));
                                if let Some(delivery) = self.set_nation_player_contend_state(
                                    &region.war.base,
                                    *cancelled_player_id,
                                    false,
                                    context,
                                ) {
                                    state_deliveries.push((*cancelled_player_id, delivery));
                                }
                            }
                            context.add_log_text(self.get_string_by_id(b"GS1073"));
                            completion_deliveries.push(
                                self.four_nation_morale_snapshot(
                                    *region.morale(),
                                    *region.nation_failed(),
                                )
                                .send_to_region(
                                    Some(&region.war.base),
                                    None,
                                    self,
                                ),
                            );
                            completion_deliveries.push(
                                nation_colored_text_message(
                                    0xbf806,
                                    0xffff_ffff,
                                    0xffff_0000,
                                    &format_legacy_text_fields(
                                        self.get_string_by_id(b"GS1082"),
                                        &[region.country_name(country)],
                                        0xff,
                                    ),
                                )
                                .send_to_region(
                                    Some(&region.war.base),
                                    None,
                                    self,
                                ),
                            );
                            let mut top = CMessage::new(0xbf804);
                            top.add_long(0);
                            top.add_long(-1);
                            top.add_long(1);
                            top.add_long(1);
                            add_legacy_c_string(
                                top.base_mut(),
                                &format_legacy_text_fields(
                                    self.get_string_by_id(b"GS1083"),
                                    &[region.country_name(country)],
                                    0xff,
                                ),
                            );
                            top_info_delivery = Some(top.send_all(self.current_net_server()));
                            for (name_id, script, x, y) in [
                                (
                                    b"GS1025".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_01.script".as_slice(),
                                    0xfb,
                                    0x102,
                                ),
                                (
                                    b"GS1190".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_02.script".as_slice(),
                                    0xf6,
                                    0xfd,
                                ),
                                (
                                    b"GS1189".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_03.script".as_slice(),
                                    0xfb,
                                    0xf9,
                                ),
                            ] {
                                treasure_spawns.push(self.spawn_nation_treasure_box(
                                    &mut region,
                                    name_id,
                                    script,
                                    x,
                                    y,
                                    context,
                                ));
                            }
                            NationContendCompletionOutcome::Captured(capture)
                        }
                    }
                }
            });
            region.clear_contenders();
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(Ok(NationContendAiReport {
            region_id,
            magic_stone_transitions,
            progress_deliveries,
            completed,
            completion_outcome,
            completion_deliveries,
            state_deliveries,
            top_info_delivery,
            treasure_spawns,
        }))
    }

    fn replace_nation_magic_stone<Context: NationContendContext>(
        &self,
        region: &mut ServerNationRegion,
        country: u8,
        context: &mut Context,
    ) -> NationMagicStoneTransitionReport {
        let (npc_name_id, monster_name_id, tile_x, tile_y) = match country {
            1 => (b"GS1084".as_slice(), b"GS1142".as_slice(), 0xfb, 0x35),
            2 => (b"GS1085".as_slice(), b"GS1139".as_slice(), 0xf8, 0x1c1),
            3 => (b"GS1086".as_slice(), b"GS1140".as_slice(), 0x25, 0xfc),
            4 => (b"GS1087".as_slice(), b"GS1141".as_slice(), 0x1dc, 0x105),
            _ => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcMissing,
                };
            }
        };
        let npc_name = self.get_string_by_id(npc_name_id);
        let npc = match region.war.base.find_npc_by_name(npc_name) {
            Ok(Some(npc)) => npc,
            Ok(None) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcMissing,
                };
            }
            Err(block) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcNameAmbiguous {
                        matches: block.matches,
                    },
                };
            }
        };
        let shape = npc.move_shape().shape();
        let npc_id = shape.identity().id;
        let npc_tile_x = match shape.get_tile_x() {
            Ok(tile_x) => tile_x,
            Err(error) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: Some(npc_id),
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcCoordinateBlocked(error),
                };
            }
        };
        let npc_tile_y = match shape.get_tile_y() {
            Ok(tile_y) => tile_y,
            Err(error) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: Some(npc_id),
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcCoordinateBlocked(error),
                };
            }
        };

        let mut explosion = CMessage::new(0xbf50a);
        explosion.add_long(2_000_000);
        explosion
            .base_mut()
            .add(&(npc_tile_x as f32 + 0.5).to_le_bytes());
        explosion
            .base_mut()
            .add(&(npc_tile_y as f32 + 0.5).to_le_bytes());
        let explosion_delivery =
            Some(context.send_nation_magic_stone_around(&region.war.base, shape, &explosion));

        let identity = shape.identity();
        let mut removal = CMessage::new(0xbf504);
        removal.add_long(identity.object_type);
        removal.add_long(identity.id);
        removal.add_long(0);
        let removal_delivery =
            Some(context.send_nation_magic_stone_around(&region.war.base, shape, &removal));

        if let Err(error) = region.war.base.remove_owned_npc_by_id(npc_id) {
            return NationMagicStoneTransitionReport {
                country,
                npc_id: Some(npc_id),
                explosion_delivery,
                removal_delivery,
                outcome: NationMagicStoneTransitionOutcome::NpcRemovalBlocked(error),
            };
        }

        let monster_name = self.get_string_by_id(monster_name_id);
        let Some(property) = self
            .find_monster_property_by_origin_name(monster_name)
            .cloned()
        else {
            return NationMagicStoneTransitionReport {
                country,
                npc_id: Some(npc_id),
                explosion_delivery,
                removal_delivery,
                outcome: NationMagicStoneTransitionOutcome::MonsterPropertyMissing,
            };
        };
        let (area_width, area_height) = self.area_dimensions();
        let outcome = match region.war.base.add_monster(
            &property,
            tile_x,
            tile_y,
            -1,
            true,
            false,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        ) {
            Ok(monster_id) => NationMagicStoneTransitionOutcome::Spawned { monster_id },
            Err(error) => NationMagicStoneTransitionOutcome::MonsterSpawnBlocked(error),
        };
        NationMagicStoneTransitionReport {
            country,
            npc_id: Some(npc_id),
            explosion_delivery,
            removal_delivery,
            outcome,
        }
    }

    fn spawn_nation_treasure_box<Context: NationCombatContext>(
        &self,
        region: &mut ServerNationRegion,
        name_id: &[u8],
        script: &[u8],
        x: i32,
        y: i32,
        context: &mut Context,
    ) -> Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock> {
        let setup = ServerRegionNpcSetup {
            show_list: true,
            picture_id: 0x104,
            left: x,
            top: y,
            right: x,
            bottom: y,
            count: 1,
            direction: -1,
            time: 3_600_000,
            name: self.get_string_by_id(name_id).to_vec(),
            script: script.to_vec(),
        };
        let (area_width, area_height) = self.area_dimensions();
        region.war.base.add_npc(
            &setup,
            true,
            true,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        )
    }

    fn set_nation_player_contend_state<Context: NationCombatContext>(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        contend_state: bool,
        context: &mut Context,
    ) -> Option<Result<i32, ShapeCoordinateBlock>> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_contend_state(contend_state) {
            return None;
        }
        let mut message = CMessage::new(0xbff28);
        message.add_long(player_id);
        message.add_byte(u8::from(contend_state));
        let player = self
            .find_player(player_id)
            .expect("player сохранён между mutation и synchronous around-send");
        Some(context.send_nation_player_around(region, player.shape(), None, &message))
    }

    fn publish_player_died_state<Context: NationCombatContext>(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        state: bool,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStatePublication> {
        self.find_player_mut(player_id)?
            .set_city_war_died_state(state);
        let mut message = CMessage::new(0xbff2a);
        message.add_long(player_id);
        message.add_byte(u8::from(state));
        let self_delivery = message.send_to_player(self.net_server(), player_id);
        let player = self
            .find_player(player_id)
            .expect("player сохранён между self и synchronous around-send");
        let around_delivery =
            context.send_nation_player_around(region, player.shape(), Some(player_id), &message);
        Some(NationPlayerDiedStatePublication {
            player_id,
            state,
            self_delivery,
            around_delivery,
        })
    }

    fn set_player_died_state_time(&mut self, player_id: i32, time_ms: i32) -> Option<i32> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_city_war_died_state_time_ms(time_ms) {
            return None;
        }
        let mut message = CMessage::new(0xbff2b);
        message.add_long(player_id);
        message.add_long(time_ms);
        Some(message.send_to_player(self.net_server(), player_id))
    }

    /// Полный достигнутый Nation-prefix `CPlayer::OnDied`: закрывает active
    /// war clock, исполняет virtual contend cancel, затем устанавливает
    /// половину global death penalty без setter-wire.
    pub(crate) fn player_died_in_nation_region<Context: NationCombatContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDeathReport> {
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let timing_finished = region
            .finish_player_timing(player_id, true, || context.now_milliseconds())
            .is_some();
        let was_contending = self
            .find_player(player_id)
            .is_some_and(CPlayer::contend_state);
        let mut contend_state_delivery = None;
        let mut contend_time_delivery = None;
        let mut notice_delivery = None;
        let contend_outcome = if !was_contending {
            NationPlayerDeathContendOutcome::NotContending
        } else {
            match region.cancel_contend_by_player_id(player_id) {
                NationContendCancelOutcome::MissingReset => {
                    contend_state_delivery = self.set_nation_player_contend_state(
                        &region.war.base,
                        player_id,
                        false,
                        context,
                    );
                    contend_time_delivery = Some(self.send_nation_contend_time(player_id, 0));
                    notice_delivery = Some(
                        self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS0136")),
                    );
                    NationPlayerDeathContendOutcome::MissingReset
                }
                NationContendCancelOutcome::Removed { .. } => {
                    NationPlayerDeathContendOutcome::RemovedLegacyReturnIndeterminate
                }
            }
        };
        let died_state_time_ms = self
            .globe_setup
            .died_state_time_seconds()
            .wrapping_mul(1000)
            / 2;
        let died_state_start_time_ms = (died_state_time_ms > 0).then(|| context.now_milliseconds());
        if let Some(player) = self.find_player_mut(player_id) {
            player.begin_city_war_death_countdown(
                died_state_time_ms,
                died_state_start_time_ms.unwrap_or_default(),
            );
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationPlayerDeathReport {
            region_id,
            player_id,
            timing_finished,
            contend_outcome,
            contend_state_delivery,
            contend_time_delivery,
            notice_delivery,
            died_state_time_ms,
            died_state_start_time_ms,
        })
    }

    /// Две достижимые `OnRelive` ветви сходятся в этом exact tail: positive
    /// remaining time активирует state и публикует self, затем around.
    pub(crate) fn publish_nation_died_state_after_relive<Context: NationCombatContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStatePublication> {
        let player = self.find_player(player_id)?;
        if player.city_war_died_state_time_ms() <= 0 {
            return None;
        }
        let region_id = player.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let publication = self.publish_player_died_state(owner.base(), player_id, true, context);
        self.restore_region_owner(owner);
        publication
    }

    /// Exact death-state tail `CPlayer::PeriodicalUpdate`: `timeGetTime`
    /// читается всегда, threshold строго `>1000`, DWORD elapsed wraps, а
    /// сравнение выполняется после signed cast.
    pub(crate) fn periodical_update_nation_died_state<Context: NationCombatContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStateTick> {
        let now_ms = context.now_milliseconds();
        let player = self.find_player(player_id)?;
        let time_ms = player.city_war_died_state_time_ms();
        if time_ms <= 0 {
            return Some(NationPlayerDiedStateTick::Inactive);
        }
        let elapsed_ms = now_ms.wrapping_sub(player.died_state_start_time_ms());
        if elapsed_ms <= 1000 {
            return Some(NationPlayerDiedStateTick::Waiting { elapsed_ms });
        }
        self.find_player_mut(player_id)?
            .restart_died_state_clock(now_ms);
        if (elapsed_ms as i32) < time_ms {
            let remaining_ms = time_ms.wrapping_sub(elapsed_ms as i32);
            let time_delivery = self.set_player_died_state_time(player_id, remaining_ms);
            return Some(NationPlayerDiedStateTick::Advanced {
                elapsed_ms,
                remaining_ms,
                time_delivery,
            });
        }
        let time_delivery = self.set_player_died_state_time(player_id, 0);
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let state_publication =
            self.publish_player_died_state(owner.base(), player_id, false, context);
        self.restore_region_owner(owner);
        let state_publication = state_publication?;
        Some(NationPlayerDiedStateTick::Expired {
            elapsed_ms,
            time_delivery,
            state_publication,
        })
    }

    fn send_nation_contend_time(&self, player_id: i32, percentage: i32) -> i32 {
        let mut message = CMessage::new(0xbff29);
        message.add_long(percentage);
        message.send_to_player(self.net_server(), player_id)
    }

    fn send_nation_player_notice(&self, player_id: i32, text: &[u8]) -> i32 {
        nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, text)
            .send_to_player(self.net_server(), player_id)
    }

    /// Reached `CMonster::OnBeenHurted` branch: только player damage (`400`)
    /// вызывает Nation first-hit owner.
    pub(crate) fn monster_on_been_hurted(
        &mut self,
        region_id: i32,
        monster_id: i32,
        attacker_type: i32,
        attacker_id: i32,
    ) -> Option<NationMonsterDamageReport> {
        (attacker_type == 400)
            .then(|| self.nation_monster_damaged(region_id, monster_id, attacker_id))?
    }

    pub(crate) fn nation_monster_damaged(
        &mut self,
        region_id: i32,
        monster_id: i32,
        attacker_player_id: i32,
    ) -> Option<NationMonsterDamageReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };

        let outcome = match region.war.base.find_monster_by_id(monster_id) {
            None => NationMonsterDamageOutcome::MonsterMissing,
            Some(monster) if !monster.can_trigger_nation_damage() => {
                NationMonsterDamageOutcome::MonsterUnavailable
            }
            Some(monster) => {
                let Some(race) = monster
                    .base_property_key()
                    .and_then(|key| self.find_monster_property_by_origin_name(key))
                    .map(|property| property.race)
                else {
                    self.restore_region_owner(ServerRegionOwner::Nation(region));
                    return Some(NationMonsterDamageReport {
                        region_id,
                        monster_id,
                        attacker_player_id,
                        outcome: NationMonsterDamageOutcome::MonsterPropertyMissing,
                        world_delivery: None,
                    });
                };
                let monster_original_name = monster.original_name().to_vec();
                let Some(attacker) = self.find_player(attacker_player_id) else {
                    self.restore_region_owner(ServerRegionOwner::Nation(region));
                    return Some(NationMonsterDamageReport {
                        region_id,
                        monster_id,
                        attacker_player_id,
                        outcome: NationMonsterDamageOutcome::AttackerMissing,
                        world_delivery: None,
                    });
                };
                if !attacker.can_attack_nation_monster() {
                    NationMonsterDamageOutcome::AttackerUnavailable
                } else if u32::from(attacker.country()) == race {
                    NationMonsterDamageOutcome::SameCountry
                } else {
                    match u8::try_from(race).ok().and_then(|defender_country| {
                        region.register_monster_first_hit(
                            &monster_original_name,
                            defender_country,
                            attacker.country(),
                            |id| self.get_string_by_id(id).to_vec(),
                        )
                    }) {
                        Some(notice) => NationMonsterDamageOutcome::Notice(notice),
                        None if !(1..=4).contains(&race) => {
                            NationMonsterDamageOutcome::CountryOutsideNation
                        }
                        None => NationMonsterDamageOutcome::NoFirstHitNotice,
                    }
                }
            }
        };
        let world_delivery = match outcome {
            NationMonsterDamageOutcome::Notice(notice) => Some(
                self.nation_first_hit_notice_message(&region, notice)
                    .send(self, false),
            ),
            _ => None,
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationMonsterDamageReport {
            region_id,
            monster_id,
            attacker_player_id,
            outcome,
            world_delivery,
        })
    }

    /// Reached `CMonster::OnDied` Nation callback. Context оставляет снаружи
    /// только уже существующие spatial/AI callbacks полноценного `AddNpc` и
    /// process log sinks; state и все client/World packets исполняются здесь.
    pub(crate) fn monster_on_died<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        monster_id: i32,
        killer_player_id: i32,
        context: &mut Context,
    ) -> Option<NationMonsterDeathReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        if region.war.base.find_monster_by_id(monster_id).is_some() {
            context.add_log_text(b"ServerNationRegion::OnMonsterDie");
        }
        let outcome = if !region
            .war
            .base
            .registered_player_ids()
            .contains(&killer_player_id)
        {
            NationMonsterDeathOutcome::KillerMissing
        } else {
            match region.war.base.find_monster_by_id(monster_id) {
                None => NationMonsterDeathOutcome::MonsterMissing,
                Some(monster) => {
                    let Some(race) = monster
                        .base_property_key()
                        .and_then(|key| self.find_monster_property_by_origin_name(key))
                        .map(|property| property.race)
                    else {
                        self.restore_region_owner(ServerRegionOwner::Nation(region));
                        return Some(NationMonsterDeathReport {
                            region_id,
                            monster_id,
                            killer_player_id,
                            outcome: NationMonsterDeathOutcome::MonsterPropertyMissing,
                            morale_delivery: None,
                            first_guard_delivery: None,
                            nation_fail_deliveries: Vec::new(),
                            yu_ying_shi_spawns: Vec::new(),
                        });
                    };
                    let target = classify_nation_morale_target(monster.original_name(), |id| {
                        self.get_string_by_id(id).to_vec()
                    });
                    match target {
                        None => NationMonsterDeathOutcome::Unclassified,
                        Some(target) => {
                            let attacker_country = self
                                .find_player(killer_player_id)
                                .expect("killer проверен в Nation m_vPlayers")
                                .country();
                            match u8::try_from(race).ok().and_then(|defender_country| {
                                region.apply_monster_morale(
                                    target,
                                    defender_country,
                                    attacker_country,
                                )
                            }) {
                                Some(mutation) => {
                                    NationMonsterDeathOutcome::MoraleChanged(mutation)
                                }
                                None => NationMonsterDeathOutcome::CountryOutsideNation,
                            }
                        }
                    }
                }
            }
        };
        if matches!(outcome, NationMonsterDeathOutcome::KillerMissing) {
            context.put_debug_string(self.get_string_by_id(b"GS1128"));
        }

        let mut first_guard_delivery = None;
        let mut nation_fail_deliveries = Vec::new();
        let mut yu_ying_shi_spawns = Vec::new();
        if let NationMonsterDeathOutcome::MoraleChanged(mutation) = outcome {
            if mutation.first_guard_notice {
                let defender = region.country_name(mutation.defender_country);
                let attacker = region.country_name(mutation.attacker_country);
                let text = format_legacy_text_fields(
                    self.get_string_by_id(b"GS1121"),
                    &[defender, attacker, attacker],
                    0xff,
                );
                first_guard_delivery = Some(
                    nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &text)
                        .send_to_region(Some(&region.war.base), None, self),
                );
            }

            if mutation.check_morale_spawn
                && region.mark_yu_ying_shi_due_to_morale(mutation.defender_country)
            {
                yu_ying_shi_spawns.push(self.spawn_nation_yu_ying_shi(
                    &mut region,
                    mutation.defender_country,
                    mutation.attacker_country,
                    context,
                ));
            }
            if mutation.check_admiral_spawn
                && region.mark_yu_ying_shi_due_to_admiral(mutation.defender_country)
            {
                yu_ying_shi_spawns.push(self.spawn_nation_yu_ying_shi(
                    &mut region,
                    mutation.defender_country,
                    mutation.defender_country,
                    context,
                ));
            }

            if mutation.nation_failed {
                let defender = region.country_name(mutation.defender_country);
                let attacker = region.country_name(mutation.attacker_country);
                let (template, arguments): (&[u8], &[&[u8]]) =
                    if mutation.defender_country == mutation.attacker_country {
                        (self.get_string_by_id(b"GS1122"), &[defender])
                    } else {
                        (self.get_string_by_id(b"GS1123"), &[defender, attacker])
                    };
                let mut text = format_legacy_text_fields(template, arguments, 0xff);
                if region.treasure_box_count(mutation.defender_country) != 0 {
                    text.extend_from_slice(legacy_c_string_prefix(
                        self.get_string_by_id(b"GS1124"),
                    ));
                    text.truncate(0xff);
                }
                nation_fail_deliveries.push(nation_world_notice_message(&text).send(self, false));
                let mut failure = CMessage::new(0x6031d);
                failure.add_long(i32::from(mutation.defender_country));
                failure.add_long(i32::from(mutation.attacker_country));
                nation_fail_deliveries.push(failure.send(self, false));
            }

            context.add_log_text(self.get_string_by_id(b"GS1129"));
        }
        let morale_delivery =
            matches!(outcome, NationMonsterDeathOutcome::MoraleChanged(_)).then(|| {
                self.four_nation_morale_snapshot(*region.morale(), *region.nation_failed())
                    .send_to_region(Some(&region.war.base), None, self)
            });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationMonsterDeathReport {
            region_id,
            monster_id,
            killer_player_id,
            outcome,
            morale_delivery,
            first_guard_delivery,
            nation_fail_deliveries,
            yu_ying_shi_spawns,
        })
    }

    fn nation_first_hit_notice_message(
        &self,
        region: &ServerNationRegion,
        notice: NationMonsterDamageNotice,
    ) -> CMessage {
        let (defender_country, attacker_country, template_id, stone) = match notice {
            NationMonsterDamageNotice::StoneGuard {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1125".as_slice(),
                true,
            ),
            NationMonsterDamageNotice::JinWeiJun {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1126".as_slice(),
                false,
            ),
            NationMonsterDamageNotice::MagicStone {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1127".as_slice(),
                false,
            ),
        };
        let defender = region.country_name(defender_country);
        let attacker = region.country_name(attacker_country);
        let arguments: &[&[u8]] = if stone {
            &[defender, attacker]
        } else {
            &[attacker, defender, defender]
        };
        let text = format_legacy_text_fields(self.get_string_by_id(template_id), arguments, 0xff);
        if stone {
            let mut message = CMessage::new(0x5fd09);
            message.add_byte(1);
            message.add_byte(defender_country);
            add_legacy_c_string(message.base_mut(), &text);
            message
        } else {
            nation_world_notice_message(&text)
        }
    }

    fn spawn_nation_yu_ying_shi<Context: NationCombatContext>(
        &self,
        region: &mut ServerNationRegion,
        country: u8,
        notify_country: u8,
        context: &mut Context,
    ) -> NationYuYingShiSpawnReport {
        let (script, x, y, coordinates): (&[u8], i32, i32, &[u8]) = match country {
            1 => (
                b"scripts/npc/npc_siguoyuyingshi_11000.script",
                214,
                71,
                b"[214,71]",
            ),
            2 => (
                b"scripts/npc/npc_siguoyuyingshi_12000.script",
                214,
                434,
                b"[214,434]",
            ),
            3 => (
                b"scripts/npc/npc_siguoyuyingshi_13000.script",
                50,
                217,
                b"[50,217]",
            ),
            4 => (
                b"scripts/npc/npc_siguoyuyingshi_14000.script",
                461,
                290,
                b"[461,290]",
            ),
            _ => unreachable!("YuYingShi gate принимает только страны 1..=4"),
        };
        let setup = ServerRegionNpcSetup {
            show_list: true,
            picture_id: 0x207,
            left: x,
            top: y,
            right: x,
            bottom: y,
            count: 1,
            direction: -1,
            time: 3_600_000,
            name: self.get_string_by_id(b"GS1136").to_vec(),
            script: script.to_vec(),
        };
        let (area_width, area_height) = self.area_dimensions();
        let spawn = region.war.base.add_npc(
            &setup,
            true,
            true,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        );
        if spawn.is_err() {
            return NationYuYingShiSpawnReport {
                country,
                notify_country,
                spawn,
                region_delivery: None,
                country_deliveries: Vec::new(),
            };
        }

        let country_name = region.country_name(country);
        let region_text = format_legacy_text_fields(
            self.get_string_by_id(b"GS1137"),
            &[country_name, country_name],
            0xff,
        );
        let region_delivery = Some(
            nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffa2_44ff, &region_text)
                .send_to_region(Some(&region.war.base), None, self),
        );
        let country_text =
            format_legacy_text_fields(self.get_string_by_id(b"GS1138"), &[coordinates], 0x7f);
        let country_message =
            nation_colored_text_message(0xbf811, 0xffff_ff00, 0xff00_0000, &country_text);
        let country_deliveries = (0..3)
            .map(|_| {
                country_message.send_to_region_contry_player(
                    Some(&region.war.base),
                    i32::from(notify_country),
                    self,
                )
            })
            .collect();
        NationYuYingShiSpawnReport {
            country,
            notify_country,
            spawn,
            region_delivery,
            country_deliveries,
        }
    }

    fn four_nation_morale_snapshot(&self, morale: [i32; 5], failed: [bool; 5]) -> CMessage {
        let colors = [0xfffc_0000, 0xffd1_eefe, 0xffe2_bf18, 0xff78_fcdb];
        let string_ids: [[&[u8]; 2]; 4] = [
            [b"GS1075", b"GS1074"],
            [b"GS1077", b"GS1076"],
            [b"GS1079", b"GS1078"],
            [b"GS1081", b"GS1080"],
        ];
        let mut snapshot = CMessage::new(0xbf818);
        for index in 0..4 {
            snapshot.base_mut().add_ulong(colors[index]);
            let text = format_single_legacy_i32(
                self.get_string_by_id(string_ids[index][usize::from(failed[index + 1])]),
                morale[index + 1],
                0xff,
            );
            let text = CString::new(text).expect("localized morale text обрезан до NUL");
            snapshot.base_mut().add_str(Some(&text));
        }
        snapshot
    }

    pub(crate) fn find_region_by_name(&self, name: &[u8]) -> Option<&ServerRegionOwner> {
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.regions
            .values()
            .find(|region| region.name() == &name[..end])
    }

    pub(crate) const fn set_initial_region_totals(&mut self, monsters: i32, npcs: i32) {
        self.initial_total_monsters = monsters;
        self.initial_total_npcs = npcs;
    }

    pub(crate) const fn initial_region_totals(&self) -> (i32, i32) {
        (self.initial_total_monsters, self.initial_total_npcs)
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

    pub(crate) const fn gm_list(&self) -> &CGMList {
        &self.gm_list
    }

    pub(crate) const fn da_kong_xiang_qian_mut(&mut self) -> &mut CDaKongXiangQian {
        &mut self.da_kong_xiang_qian
    }

    pub(crate) const fn globe_setup(&self) -> &GlobeSetupSnapshot {
        &self.globe_setup
    }

    pub(crate) const fn region_router(&self) -> &RegionRouter {
        &self.region_router
    }

    pub(crate) const fn globe_setup_and_region_router_mut(
        &mut self,
    ) -> (&mut GlobeSetupSnapshot, &mut RegionRouter) {
        (&mut self.globe_setup, &mut self.region_router)
    }

    pub(crate) const fn area_dimensions(&self) -> (i32, i32) {
        (self.area_width, self.area_height)
    }

    pub(crate) const fn set_area_dimensions(&mut self, width: i32, height: i32) {
        self.area_width = width;
        self.area_height = height;
    }

    pub(crate) const fn auction_now(&self) -> bool {
        self.auction_now
    }

    pub(crate) const fn auction_last_check_seconds(&self) -> u32 {
        self.auction_last_check_seconds
    }

    pub(crate) const fn auction_room(&self) -> &CAuctionRoom<CGoodsNode> {
        &self.auction_room
    }

    pub(crate) const fn auction_room_mut(&mut self) -> &mut CAuctionRoom<CGoodsNode> {
        &mut self.auction_room
    }

    /// Возвращает process-owned Game variant для async producer-ов.
    pub(crate) const fn net_session_manager(&self) -> &CNetSessionManager {
        &self.net_session_manager
    }

    /// Exact `CGame::SetAuctionState`: false не меняет saved wall-clock,
    /// true публикует полученный caller-ом `_time` sample.
    pub(crate) const fn set_auction_state(&mut self, enabled: bool, wall_time_seconds: u32) {
        self.auction_now = enabled;
        if enabled {
            self.auction_last_check_seconds = wall_time_seconds;
        }
    }

    /// Точный false-only effect Globe decoder-а без обновления timestamp.
    pub(crate) const fn force_auction_disabled(&mut self) {
        self.set_auction_state(false, self.auction_last_check_seconds);
    }

    pub(crate) fn set_function_file_data<Context: GameScriptResourceContext>(
        &mut self,
        data: Vec<u8>,
        context: &mut Context,
    ) -> GameSingleFilePublication {
        if self.function_list_file_data.is_some() {
            self.function_list_file_data.take();
            return GameSingleFilePublication::RepeatedOwnerFreed;
        }
        self.function_list_file_data = Some(data);
        let published = self
            .function_list_file_data
            .as_deref()
            .expect("function list только что опубликован");
        context.load_function_list(legacy_c_string_prefix(published));
        GameSingleFilePublication::Published
    }

    pub(crate) fn set_variable_file_data(&mut self, data: Vec<u8>) -> GameSingleFilePublication {
        if self.variable_list_file_data.is_some() {
            self.variable_list_file_data.take();
            return GameSingleFilePublication::RepeatedOwnerFreed;
        }
        self.variable_list_file_data = Some(data);
        GameSingleFilePublication::Published
    }

    pub(crate) fn set_general_variable_file_data<Context: GameScriptResourceContext>(
        &mut self,
        source: &[u8],
        cursor: usize,
        context: &mut Context,
    ) {
        context.load_general_variables(source, cursor);
    }

    pub(crate) fn set_script_file_data(&mut self, path: Vec<u8>, data: Vec<u8>) -> bool {
        self.script_file_data
            .insert(legacy_c_string_prefix(&path).to_vec(), data)
            .is_some()
    }

    pub(crate) fn function_file_data(&self) -> Option<&[u8]> {
        self.function_list_file_data.as_deref()
    }

    pub(crate) fn variable_file_data(&self) -> Option<&[u8]> {
        self.variable_list_file_data.as_deref()
    }

    pub(crate) fn script_file_data(&self, path: &[u8]) -> Option<&[u8]> {
        self.script_file_data
            .get(legacy_c_string_prefix(path))
            .map(Vec::as_slice)
    }

    /// Очищает и декодирует language table, пишет exact log и лишь затем
    /// сдвигает внешний message cursor на consumed length.
    pub(crate) fn create_string_table(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        mut add_log_text: impl FnMut(&[u8]),
    ) -> Result<MyStringTableDecodeReport, MyStringTableDecodeError> {
        let start = *cursor;
        self.string_table.table_mut().free();
        let payload = source.get(start..).unwrap_or_default();
        let report = self.string_table.from_byte_array(payload)?;
        if report.unique_entries == 0 {
            add_log_text(b"WARNING : Received a NULL language packet from WorldServer!");
        } else {
            add_log_text(b"Received Language packet from WorldServer OK!");
        }
        *cursor = start
            .checked_add(report.consumed)
            .expect("MyStringTable consumed length помещается в message cursor");
        Ok(report)
    }

    /// Exact `GetStringByID` fallback: отсутствующий key возвращает пустую строку.
    pub(crate) fn get_string_by_id(&self, id: &[u8]) -> &[u8] {
        self.string_table
            .table()
            .get_string_by_id(id)
            .unwrap_or_default()
    }

    pub(crate) const fn quest_system(&self) -> &CQuestSystem {
        &self.quest_system
    }

    pub(crate) const fn quest_system_mut(&mut self) -> &mut CQuestSystem {
        &mut self.quest_system
    }

    pub(crate) const fn country_param(&self) -> &CCountryParam {
        &self.country_param
    }

    pub(crate) const fn country_param_mut(&mut self) -> &mut CCountryParam {
        &mut self.country_param
    }

    pub(crate) const fn country_handler(&self) -> &CCountryHandler {
        &self.country_handler
    }

    pub(crate) const fn country_handler_mut(&mut self) -> &mut CCountryHandler {
        &mut self.country_handler
    }

    pub(crate) const fn attack_city_sys(&self) -> &CAttackCitySys {
        &self.attack_city_sys
    }

    pub(crate) const fn attack_city_sys_mut(&mut self) -> &mut CAttackCitySys {
        &mut self.attack_city_sys
    }

    pub(crate) const fn village_war_sys(&self) -> &CVillageWarSys {
        &self.village_war_sys
    }

    pub(crate) const fn village_war_sys_mut(&mut self) -> &mut CVillageWarSys {
        &mut self.village_war_sys
    }

    pub(crate) const fn country_war_sys(&self) -> &CountryWarSys {
        &self.country_war_sys
    }

    pub(crate) const fn country_war_sys_mut(&mut self) -> &mut CountryWarSys {
        &mut self.country_war_sys
    }

    pub(crate) const fn four_nation_war_sys(&self) -> &CFourNationWarSys {
        &self.four_nation_war_sys
    }

    pub(crate) const fn four_nation_war_sys_mut(&mut self) -> &mut CFourNationWarSys {
        &mut self.four_nation_war_sys
    }

    pub(crate) const fn emotion(&self) -> &CEmotion {
        &self.emotion
    }

    pub(crate) const fn emotion_mut(&mut self) -> &mut CEmotion {
        &mut self.emotion
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

    pub(crate) const fn words_filter(&self) -> &CWordsFilter {
        &self.words_filter
    }

    pub(crate) const fn words_filter_mut(&mut self) -> &mut CWordsFilter {
        &mut self.words_filter
    }

    pub(crate) const fn jjc_level_data(&self) -> &BTreeMap<i32, i32> {
        &self.jjc_level_data
    }

    pub(crate) fn clear_jjc_level_data(&mut self) {
        self.jjc_level_data.clear();
    }

    pub(crate) fn insert_jjc_level_data(&mut self, key: i32, value: i32) -> bool {
        if self.jjc_level_data.contains_key(&key) {
            return false;
        }
        self.jjc_level_data.insert(key, value);
        true
    }

    pub(crate) const fn tao_zhuang_setup(&self) -> &CTaoZhuangSetup {
        &self.tao_zhuang_setup
    }

    pub(crate) const fn tao_zhuang_setup_mut(&mut self) -> &mut CTaoZhuangSetup {
        &mut self.tao_zhuang_setup
    }

    pub(crate) const fn ci_qing_setup(&self) -> &CCiQingSetup {
        &self.ci_qing_setup
    }

    pub(crate) const fn ci_qing_setup_mut(&mut self) -> &mut CCiQingSetup {
        &mut self.ci_qing_setup
    }

    pub(crate) const fn ling_bao_setup(&self) -> &CLingBaoSetup {
        &self.ling_bao_setup
    }

    pub(crate) const fn ling_bao_setup_mut(&mut self) -> &mut CLingBaoSetup {
        &mut self.ling_bao_setup
    }

    pub(crate) const fn gods_battle_mgr(&self) -> &CGodsBattleMgr {
        &self.gods_battle_mgr
    }

    pub(crate) const fn gods_battle_mgr_mut(&mut self) -> &mut CGodsBattleMgr {
        &mut self.gods_battle_mgr
    }

    /// Script primitive at call-site `0x4C3D96`: пустой `0x5FA10` сначала
    /// уходит WorldServer-у, затем single pending requester перезаписывается.
    pub(crate) fn request_gods_battle_top_ten(
        &mut self,
        player_id: i32,
    ) -> GodsBattleTopTenRequestReport {
        let request = CMessage::new(0x5fa10);
        let delivery = request.send(self, false);
        self.gods_battle_mgr.record_top_ten_request(player_id);
        GodsBattleTopTenRequestReport {
            player_id,
            delivery,
        }
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

    pub(crate) const fn honor_ranks(&self) -> &CHonorRanks {
        &self.honor_ranks
    }

    pub(crate) const fn honor_ranks_mut(&mut self) -> &mut CHonorRanks {
        &mut self.honor_ranks
    }

    pub(crate) const fn goods_war(&self) -> Option<&CGoodsWarMember> {
        self.goods_war.as_ref()
    }

    pub(crate) const fn goods_war_mut(&mut self) -> Option<&mut CGoodsWarMember> {
        self.goods_war.as_mut()
    }

    pub(crate) fn take_goods_war(&mut self) -> Option<CGoodsWarMember> {
        self.goods_war.take()
    }

    pub(crate) fn restore_goods_war(&mut self, goods_war: CGoodsWarMember) {
        self.goods_war = Some(goods_war);
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

    /// Полный достигнутый `Release` teardown. Manual deletes заменены Drop и
    /// `take/clear`, а отсутствующие process-global owners вызываются строго в
    /// исходной позиции через runtime; неизвестный legacy int не выдумывается.
    pub(crate) async fn release<Runtime: GameReleaseRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameReleaseReport {
        let mut events = Vec::new();

        let debug = GameReleaseDebug::ServerExiting;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        self.stop_reconnect_tasks().await;
        events.push(GameReleaseEvent::ReconnectTasksStopped);

        let player_ids: Vec<i32> = self.players.keys().copied().collect();
        let total_players = player_ids.len();
        for (index, player_id) in player_ids.into_iter().enumerate() {
            let saved = self.players.get(&player_id).is_some_and(|player| {
                runtime.save_player(player, GAME_RELEASE_PLAYER_SAVE_MESSAGE, 1)
            });
            if !saved {
                let debug = GameReleaseDebug::PlayerSaveFailed {
                    player_id,
                    processed: index,
                    total: total_players,
                };
                runtime.put_debug_string(debug.clone());
                events.push(GameReleaseEvent::Debug(debug));
                // Safe Result-граница заменяет native exception catch, который
                // немедленно erase-ил проблемный map node и продолжал обход.
                self.players.remove(&player_id);
            }
            events.push(GameReleaseEvent::PlayerSave { player_id, saved });
        }
        let debug = GameReleaseDebug::PlayersSaved {
            processed: total_players,
            total: total_players,
        };
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        runtime.save_city_region(self, 0);
        events.push(GameReleaseEvent::CityRegionSaved);
        let debug = GameReleaseDebug::CityRegionSaved;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let players = self.players.len();
        self.players.clear();
        events.push(GameReleaseEvent::PlayersCleared { count: players });
        let regions = self.regions.len();
        self.regions.clear();
        events.push(GameReleaseEvent::RegionsCleared { count: regions });
        let debug = GameReleaseDebug::PlayersAndRegionsCleared;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let proxy_regions = self.proxy_regions.len();
        self.proxy_regions.clear();
        events.push(GameReleaseEvent::ProxyRegionsCleared {
            count: proxy_regions,
        });
        let debug = GameReleaseDebug::ProxyRegionsCleared;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let function_list = self.function_list_file_data.take().is_some();
        let variable_list = self.variable_list_file_data.take().is_some();
        let script_files = self.script_file_data.len();
        self.script_file_data.clear();
        events.push(GameReleaseEvent::ScriptDataCleared {
            function_list,
            variable_list,
            files: script_files,
        });
        for owner in [
            GameReleaseExternalOwner::ScriptFunctions,
            GameReleaseExternalOwner::GeneralVariables,
        ] {
            runtime.release_external_owner(owner);
            events.push(GameReleaseEvent::ExternalOwner(owner));
        }
        let debug = GameReleaseDebug::ScriptDataCleared;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        self.skill_factory.clear_skill_cache();
        events.push(GameReleaseEvent::SkillFactoryCleared);
        self.goods_factory.release();
        events.push(GameReleaseEvent::GoodsFactoryReleased);

        let network_server_present = self.net_server.is_some();
        if let Some(server) = self.net_server.as_mut() {
            runtime.exit_network_server_worker(server);
        }
        events.push(GameReleaseEvent::NetworkServerWorkerStopped {
            present: network_server_present,
        });

        let world_client_present = self.world_client.is_some();
        if let Some(client) = self.world_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }
        let billing_client_present = self.billing_client.is_some();
        if let Some(client) = self.billing_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }
        self.world_client = None;
        events.push(GameReleaseEvent::WorldClientReleased {
            present: world_client_present,
        });
        self.billing_client = None;
        events.push(GameReleaseEvent::BillingClientReleased {
            present: billing_client_present,
        });
        let network_server_present = self.net_server.take().is_some();
        events.push(GameReleaseEvent::NetworkServerReleased {
            present: network_server_present,
        });

        runtime.release_external_owner(GameReleaseExternalOwner::SocketRuntime);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::SocketRuntime,
        ));
        let released_net_sessions = self.net_session_manager.release();
        events.push(GameReleaseEvent::NetSessionsReleased {
            count: released_net_sessions,
        });

        self.increment_shop_list.release();
        events.push(GameReleaseEvent::IncrementShopReleased);
        runtime.release_external_owner(GameReleaseExternalOwner::PkSystem);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::PkSystem,
        ));
        self.quest_system = CQuestSystem::default();
        events.push(GameReleaseEvent::QuestSystemReleased);
        runtime.release_external_owner(GameReleaseExternalOwner::BaseMessageRuntime);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::BaseMessageRuntime,
        ));

        let sequence_count = self.sequence_registry.len();
        self.sequence_registry.clear();
        events.push(GameReleaseEvent::SequenceRegistryCleared {
            count: sequence_count,
        });
        let player_ranks_present = self.player_ranks.take().is_some();
        events.push(GameReleaseEvent::PlayerRanksReleased {
            present: player_ranks_present,
        });
        self.words_filter.clear();
        events.push(GameReleaseEvent::WordsFilterReleased);
        self.honor_ranks = CHonorRanks::default();
        events.push(GameReleaseEvent::HonorRanksReleased);
        let goods_war_present = self.goods_war.take().is_some();
        events.push(GameReleaseEvent::GoodsWarReleased {
            present: goods_war_present,
        });
        self.country_handler = CCountryHandler::default();
        events.push(GameReleaseEvent::CountryHandlerReleased);
        self.country_param = CCountryParam::default();
        events.push(GameReleaseEvent::CountryParamReleased);

        let attack_city_schedules = self.attack_city_sys.attacks.len();
        self.attack_city_sys = CAttackCitySys::default();
        events.push(GameReleaseEvent::AttackCitySystemReleased {
            schedules: attack_city_schedules,
        });
        let village_war_schedules = self.village_war_sys.village_wars.len();
        self.village_war_sys = CVillageWarSys::default();
        events.push(GameReleaseEvent::VillageWarSystemReleased {
            schedules: village_war_schedules,
        });

        let debug = GameReleaseDebug::ServerExited;
        runtime.put_debug_string(debug.clone());
        events.push(GameReleaseEvent::Debug(debug));
        GameReleaseReport {
            events,
            legacy_return: None,
        }
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

    /// Exact `s_mapPlayer.size()` для GMA `0x80002`; x86 `size_type` — DWORD.
    pub(crate) fn player_count(&self) -> u32 {
        u32::try_from(self.players.len()).expect("x86 player map не может превысить DWORD")
    }

    pub(crate) fn reset_total_honor_eliminate(
        &mut self,
        reset_mask: u32,
    ) -> Vec<PlayerHonorResetReport> {
        self.players
            .values_mut()
            .map(|player| player.reset_total_honor_eliminate(reset_mask))
            .collect()
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

    pub(crate) fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.players.get_mut(&player_id)
    }

    /// Exact `CGame::KickPlayer`: queue side effect сохраняется, публичный
    /// bool исходника всегда остаётся `false`.
    pub(crate) fn kick_player(&self, player_id: i32) -> GameKickPlayerReport {
        let command_result = self.net_server().command_handle().quit_by_map_id(player_id);
        GameKickPlayerReport {
            player_id,
            command_result,
            legacy_return: false,
        }
    }

    /// Exact `OnGMMessage 0x7FC09` recipient pass: requester остаётся,
    /// остальные canonical player ID закрываются в signed map-order.
    pub(crate) fn kick_players_except(
        &self,
        preserved_player_id: i32,
    ) -> Vec<GameKickPlayerReport> {
        self.players
            .keys()
            .copied()
            .filter(|player_id| *player_id != preserved_player_id)
            .map(|player_id| self.kick_player(player_id))
            .collect()
    }

    /// Exact `OnGMMessage 0x7FC0A`: region последовательно обходит все area,
    /// а найденные `CPlayer` передаются в `KickPlayer` в исходном порядке.
    pub(crate) fn kick_players_in_region_except(
        &self,
        region_id: i32,
        preserved_player_id: i32,
    ) -> Vec<GameKickPlayerReport> {
        let Some(region) = self.regions.get(&region_id) else {
            return Vec::new();
        };
        let mut player_ids = Vec::new();
        region.base().find_all_player_ids(&mut player_ids);
        player_ids
            .into_iter()
            .filter(|player_id| *player_id != preserved_player_id)
            .map(|player_id| self.kick_player(player_id))
            .collect()
    }

    /// Exact `OnGMMessage 0x7FC07` до network response: X-major 7×7 scan
    /// использует region `GetShape`, а `Vec::dedup` повторяет именно
    /// consecutive-only семантику исходного `std::list::unique`.
    pub(crate) fn kick_players_around_name(&self, name: &[u8]) -> GameKickAroundReport {
        let Some(player) = self.find_player_by_name(name) else {
            return game_kick_around_block(GameKickAroundOutcome::TargetMissing, None, None);
        };
        let target_player_id = player.player_id();
        let Some(server_region_id) = player.server_region_id() else {
            return game_kick_around_block(
                GameKickAroundOutcome::ServerRegionMissing,
                Some(target_player_id),
                None,
            );
        };
        let tile_x = match player.shape().get_tile_x() {
            Ok(tile_x) => tile_x,
            Err(block) => {
                return game_kick_around_block(
                    GameKickAroundOutcome::CoordinateBlocked(block),
                    Some(target_player_id),
                    Some(server_region_id),
                );
            }
        };
        let tile_y = match player.shape().get_tile_y() {
            Ok(tile_y) => tile_y,
            Err(block) => {
                return game_kick_around_block(
                    GameKickAroundOutcome::CoordinateBlocked(block),
                    Some(target_player_id),
                    Some(server_region_id),
                );
            }
        };
        let Some(region) = self.regions.get(&server_region_id) else {
            return game_kick_around_block(
                GameKickAroundOutcome::RegionMissing,
                Some(target_player_id),
                Some(server_region_id),
            );
        };
        let region = region.base();
        if let Some(identity) = region
            .registered_shape_identities()
            .into_iter()
            .find(|identity| self.resolve_shape(*identity).is_none())
        {
            return game_kick_around_block(
                GameKickAroundOutcome::UnresolvedShape(identity),
                Some(target_player_id),
                Some(server_region_id),
            );
        }
        let window_x = gm_kick_window_origin(tile_x, region.region.width);
        let window_y = gm_kick_window_origin(tile_y, region.region.height);
        let (area_width, area_height) = self.area_dimensions();
        let mut matched_player_ids = Vec::new();
        let window_end_x = window_x.wrapping_add(7);
        let window_end_y = window_y.wrapping_add(7);
        let mut scan_x = window_x;
        while scan_x < window_end_x {
            let mut scan_y = window_y;
            while scan_y < window_end_y {
                let shape = match region.get_shape(scan_x, scan_y, area_width, area_height, self) {
                    Ok(shape) => shape,
                    Err(block) => {
                        return GameKickAroundReport {
                            outcome: GameKickAroundOutcome::LookupBlocked(block),
                            target_player_id: Some(target_player_id),
                            server_region_id: Some(server_region_id),
                            window_origin: Some((window_x, window_y)),
                            matched_player_ids,
                            kicks: Vec::new(),
                        };
                    }
                };
                if let Some(shape) = shape.filter(|shape| shape.identity.object_type == PLAYER_TYPE)
                {
                    matched_player_ids.push(shape.identity.id);
                }
                scan_y = scan_y.wrapping_add(1);
            }
            scan_x = scan_x.wrapping_add(1);
        }
        matched_player_ids.dedup();
        let kicks = matched_player_ids
            .iter()
            .copied()
            .map(|player_id| self.kick_player(player_id))
            .collect();
        GameKickAroundReport {
            outcome: GameKickAroundOutcome::Completed,
            target_player_id: Some(target_player_id),
            server_region_id: Some(server_region_id),
            window_origin: Some((window_x, window_y)),
            matched_player_ids,
            kicks,
        }
    }

    /// Exact name lookup + `KickPlayer` side effect для GM `0x7FC06`.
    pub(crate) fn kick_player_by_name(&self, name: &[u8]) -> Option<GameKickPlayerReport> {
        let player_id = self.find_player_by_name(name)?.player_id();
        Some(self.kick_player(player_id))
    }

    /// Exact recipient pass `OnGMMessage 0x7FC13`: unsigned `long` country
    /// сравнивается с promoted player byte, обход сохраняет signed ID-order.
    pub(crate) fn player_ids_in_country(&self, country: u32) -> Vec<i32> {
        self.players
            .iter()
            .filter_map(|(player_id, player)| {
                (u32::from(player.country()) == country).then_some(*player_id)
            })
            .collect()
    }

    /// CountryWar start обходит canonical player map один раз и выбирает обе
    /// участвующие страны, не группируя получателей по стране.
    pub(crate) fn player_ids_in_countries(&self, countries: [u8; 2]) -> Vec<i32> {
        self.players
            .iter()
            .filter_map(|(player_id, player)| {
                countries.contains(&player.country()).then_some(*player_id)
            })
            .collect()
    }

    /// Exact `FindPlayer(char const*)`: обходит canonical player map по
    /// signed ID-order и сравнивает byte-exact C-string имя.
    pub(crate) fn find_player_by_name(&self, name: &[u8]) -> Option<&CPlayer> {
        let name = legacy_c_string_prefix(name);
        self.players
            .values()
            .find(|player| legacy_c_string_prefix(player.shape().base_object().get_name()) == name)
    }

    /// Замыкает GM silence mutation от ordered name lookup до exact
    /// `SetSilence` timestamp. Отсутствующий player не создаёт state.
    pub(crate) fn silence_player_by_name(
        &mut self,
        name: &[u8],
        minutes: i32,
        now_milliseconds: impl FnOnce() -> u32,
    ) -> Option<i32> {
        let player_id = self.find_player_by_name(name)?.player_id();
        self.players
            .get_mut(&player_id)
            .expect("FindPlayer name lookup вернул canonical player")
            .set_silence(minutes, now_milliseconds());
        Some(player_id)
    }

    /// Один ordered pass исходного GM silence query. Clock читается
    /// только для player-а с ненулевым silence, как в `IsInSilence`.
    /// Caller выполняет два pass-а: сначала capacity, затем payload.
    pub(crate) fn silenced_player_names_pass(
        &mut self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<Vec<u8>> {
        let player_ids: Vec<i32> = self.players.keys().copied().collect();
        let mut names = Vec::new();
        for player_id in player_ids {
            let player = self
                .players
                .get_mut(&player_id)
                .expect("snapshot ID принадлежит canonical player map");
            if player.silence_minutes() == 0 {
                continue;
            }
            if player.is_in_silence(now_milliseconds()) {
                names.push(player.shape().base_object().get_name().to_vec());
            }
        }
        names
    }

    /// Замыкает caller `goodsmessage` с runtime player-map и рецептом,
    /// опубликованным WorldServer selector-ом `SI_BATLLE_FAIRY_COMBINE`.
    /// Отсутствующий player не создаёт уведомления или пакет, как outer
    /// lookup исходного `CheckBattleFairyCombine`.
    pub(crate) fn check_battle_fairy_combine(&self, player_id: i32) -> BattleFairyCombineCheck {
        self.find_player(player_id)
            .map_or_else(BattleFairyCombineCheck::default, |player| {
                player.check_battle_fairy_combine(
                    &self.goods_factory,
                    self.battle_fairy_property.compose(),
                )
            })
    }

    /// Исполняемый caller combine из `goodsmessage` после lookup player-а.
    /// Отсутствующий player, как и исходный outer lookup, не посылает packet.
    /// Old-client serializer остаётся explicit transport boundary: его нельзя
    /// заменить пустым payload без изменения `OT_NEW_OBJECT/0xbf918`.
    pub(crate) fn combine_battle_fairy(
        &mut self,
        player_id: i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyCombineReport> {
        let battle_fairy_enabled = self.globe_setup.battle_fairy_enabled();
        let maximum_fetch_power = self.globe_setup.maximum_fetch_power();
        let (
            players,
            random_state,
            goods_factory,
            skill_factory,
            fairy_exp_conf,
            battle_fairy_exp_config,
            battle_fairy_property,
        ) = (
            &mut self.players,
            &mut self.random_state,
            &self.goods_factory,
            &self.skill_factory,
            &self.fairy_exp_conf,
            &self.battle_fairy_exp_config,
            &self.battle_fairy_property,
        );
        let player = players.get_mut(&player_id)?;
        let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
        let mut create_goods = |goods_index, random: &mut dyn FnMut(i32) -> i32| {
            goods_factory.create_goods(
                goods_index,
                |upper_bound| random(upper_bound),
                || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
            )
        };
        Some(player.combine_battle_fairy(
            battle_fairy_enabled,
            maximum_fetch_power,
            goods_factory,
            battle_fairy_property.compose(),
            skill_factory,
            &mut random,
            &mut create_goods,
            encode_old_client,
        ))
    }

    /// Исполняемый entry point для `goodsmessage` opcode `0x8FC2C/0x8FC2D`.
    /// Player сохраняет порядок guards и broadcast effects, а region map
    /// меняется здесь, потому что `CGame` — первый живой owner обоих runtime
    /// объектов. Transport остаётся explicit consumer ordered report-а.
    pub(crate) fn summon_battle_fairy(
        &mut self,
        player_id: i32,
        mode: i32,
    ) -> Option<BattleFairySummonReport> {
        let battle_fairy_enabled = self.globe_setup.battle_fairy_enabled();
        let mut report = {
            let player = self.players.get_mut(&player_id)?;
            player.summon_battle_fairy(battle_fairy_enabled, mode, &self.goods_factory)
        };
        let Some(action) = report.spatial_action else {
            return Some(report);
        };
        let spatial_applied = report.region_id.is_some_and(|region_id| {
            let Some(region) = self.regions.get_mut(&region_id) else {
                return false;
            };
            match action {
                BattleFairyWarSoulAction::SetPosition { previous, target } => region
                    .base_mut()
                    .set_war_soul_position(player_id as u32, previous, target),
                BattleFairyWarSoulAction::Delete { previous, .. } => region
                    .base_mut()
                    .delete_war_soul(player_id as u32, previous),
            }
        });
        report.spatial_applied = spatial_applied;
        if let Some(player) = self.players.get_mut(&player_id) {
            player.apply_war_soul_action(action, spatial_applied);
        }
        Some(report)
    }

    /// Замыкает positional add battle-fairy container-а с player property и
    /// загруженными GlobeSetup coefficients. Old-client codec остаётся
    /// transport boundary и вызывается только на подтверждённом update path.
    pub(crate) fn add_battle_fairy_goods(
        &mut self,
        player_id: i32,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyEquipmentMutationReport> {
        let coefficients = self.globe_setup.player_property_coefficients();
        self.players.get_mut(&player_id).map(|player| {
            player.add_battle_fairy_goods(
                cell,
                incoming,
                &self.goods_factory,
                coefficients,
                owner_progress_allows,
                encode_old_client,
            )
        })
    }

    /// Полный player-owned tail `CEquipmentContainer::Remove`: callback
    /// materializes reached результат ещё отдельного virtual
    /// `PropertiesChanged`, наблюдая player уже без removed slot-а.
    pub(crate) fn remove_player_equipment(
        &mut self,
        player_id: i32,
        ex_id: CGuid,
        runtime: PlayerEquipmentRemoveRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> Option<PlayerEquipmentRemoveReport> {
        let (players, goods_factory, skill_factory) =
            (&mut self.players, &self.goods_factory, &self.skill_factory);
        players.get_mut(&player_id).map(|player| {
            player.remove_equipment_goods(
                ex_id,
                goods_factory,
                skill_factory,
                runtime,
                recompute_properties,
            )
        })
    }

    /// Полный player-owned tail positional `CEquipmentContainer::Add`; оба
    /// callback-а вызываются в native порядке относительно container commit.
    pub(crate) fn add_player_equipment(
        &mut self,
        player_id: i32,
        position: u32,
        incoming: &mut Option<CGoods>,
        runtime: PlayerEquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> Option<PlayerEquipmentAddReport> {
        let (players, goods_factory, skill_factory) =
            (&mut self.players, &self.goods_factory, &self.skill_factory);
        players.get_mut(&player_id).map(|player| {
            player.add_equipment_goods(
                position,
                incoming,
                goods_factory,
                skill_factory,
                runtime,
                register_with_goods_ai,
                recompute_properties,
            )
        })
    }

    /// Замыкает remove по GUID с тем же player/equipment state и сохраняет
    /// подтверждённую двойную публикацию `0xBF918` после property removal.
    pub(crate) fn remove_battle_fairy_goods(
        &mut self,
        player_id: i32,
        ex_id: CGuid,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyEquipmentMutationReport> {
        let coefficients = self.globe_setup.player_property_coefficients();
        self.players.get_mut(&player_id).map(|player| {
            player.remove_battle_fairy_goods(
                ex_id,
                &self.goods_factory,
                coefficients,
                encode_old_client,
            )
        })
    }

    /// Исполняемый entry point goods-message `0x8FC2A`; decoder передаёт пары
    /// property/client-points без предварительного масштабирования.
    pub(crate) fn allocate_battle_fairy_potential(
        &mut self,
        player_id: i32,
        allocations: &[(i32, i32)],
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyPotentialAllocationReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let coefficients = self.globe_setup.player_property_coefficients();
        self.players.get_mut(&player_id).map(|player| {
            player.allocate_battle_fairy_potential(
                enabled,
                allocations,
                &self.goods_factory,
                coefficients,
                encode_old_client,
            )
        })
    }

    /// Полный runtime entry point goods-message `0x8FC28`: общий Game RNG,
    /// live log gates, factory, player wallet и positional BF-container
    /// исполняются в одном mutable snapshot-е.
    pub(crate) fn upgrade_battle_fairy_equipment(
        &mut self,
        player_id: i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyUpgradeReport> {
        let log_gates = crate::gameserver::appserver::player::BattleFairyUpgradeLogGates {
            success: self.log_system.goods_upgrade_success_enabled(),
            failure: self.log_system.goods_upgrade_failure_enabled(),
            lost_target: self.log_system.goods_lost_by_upgrade_enabled(),
        };
        let (players, random_state, goods_factory) = (
            &mut self.players,
            &mut self.random_state,
            &self.goods_factory,
        );
        let player = players.get_mut(&player_id)?;
        let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
        Some(player.upgrade_battle_fairy_equipment(
            goods_factory,
            log_gates,
            &mut random,
            encode_old_client,
        ))
    }

    /// Исполняемый entry point goods-message `0x8FC2B`: reset item ищется и
    /// расходуется в owned player packet до potential/player mutations.
    pub(crate) fn reset_battle_fairy_potential(
        &mut self,
        player_id: i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyPotentialResetReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        self.players.get_mut(&player_id).map(|player| {
            player.reset_battle_fairy_potential(enabled, &self.goods_factory, encode_old_client)
        })
    }

    /// Runtime entry point `CBattleFairyContainer::ResetSkill`, общий для
    /// script-functions распределения обычного/special skill и прямого caller-а
    /// с расходом reset item. RNG принадлежит одному `CGame` sequence.
    pub(crate) fn reset_battle_fairy_skill(
        &mut self,
        player_id: i32,
        position: i32,
        consume_item: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairySkillResetReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let (players, random_state, goods_factory, skill_factory) = (
            &mut self.players,
            &mut self.random_state,
            &self.goods_factory,
            &self.skill_factory,
        );
        let player = players.get_mut(&player_id)?;
        let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
        Some(player.reset_battle_fairy_skill(
            enabled,
            position,
            consume_item,
            goods_factory,
            skill_factory,
            &mut random,
            encode_old_client,
        ))
    }

    /// Runtime entry point уже декодированного `skillmessage 0x90005`.
    /// Facts оставляют explicit boundaries для ещё сырого `CPlayerAI`,
    /// `SymbolIsAttackAble` и monster registry, не выдавая player-only resolver
    /// текущего `CGame` за полный region lookup.
    pub(crate) fn request_battle_fairy_skill(
        &self,
        player_id: i32,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
    ) -> Option<BattleFairySkillRequestReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        self.players.get(&player_id).map(|player| {
            player.request_battle_fairy_skill(
                enabled,
                request,
                facts,
                &self.goods_factory,
                &self.skill_factory,
            )
        })
    }

    /// Завершает periodic `ComputeWarSoulXY` tick через тот же region area-map,
    /// который обслуживает summon/recall. Skill restored-state остаётся exact
    /// fact ещё не перенесённого concrete `CSkill`.
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        player_id: i32,
        current_war_soul_skill_restored: Option<bool>,
    ) -> Option<BattleFairyFollowReport> {
        let mut report = self
            .players
            .get_mut(&player_id)?
            .compute_war_soul_xy(current_war_soul_skill_restored);
        let Some(action) = report.spatial_action else {
            return Some(report);
        };
        let spatial_applied = report.region_id.is_some_and(|region_id| {
            let Some(region) = self.regions.get_mut(&region_id) else {
                return false;
            };
            match action {
                BattleFairyWarSoulAction::SetPosition { previous, target } => region
                    .base_mut()
                    .set_war_soul_position(player_id as u32, previous, target),
                BattleFairyWarSoulAction::Delete { .. } => false,
            }
        });
        report.spatial_applied = spatial_applied;
        if let Some(player) = self.players.get_mut(&player_id) {
            player.apply_war_soul_action(action, spatial_applied);
        }
        Some(report)
    }

    /// Выполняет periodic HP-death prefix `CPlayer::AI` над игроком из
    /// canonical ordered registry. Возвращаемый effect оставляет virtual
    /// `PropertiesChanged` явной границей до полного property owner-а.
    pub(crate) fn refresh_battle_fairy_death(
        &mut self,
        player_id: i32,
    ) -> Option<BattleFairyDeathReport> {
        let factory = &self.goods_factory;
        self.players
            .get_mut(&player_id)
            .map(|player| player.refresh_battle_fairy_death(factory))
    }

    /// Один exact `CGame::RunAuction` pass. Feature gate не читает часы;
    /// strict 999-ms gate читает wall-clock только при срабатывании.
    /// Goods snapshot и оба World sends сохраняют исходный порядок.
    pub(crate) fn run_auction<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameAuctionRunReport {
        if !self.globe_setup.auction_enabled() {
            return GameAuctionRunReport {
                outcome: GameAuctionRunOutcome::FeatureDisabled,
                sampled_tick_ms: None,
                sampled_wall_time_seconds: None,
                synchronized_goods: Vec::new(),
                goods_sync: None,
                state_request: None,
            };
        }

        let sampled_tick_ms = runtime.get_tick_ms();
        let elapsed_ms = sampled_tick_ms.wrapping_sub(self.auction_tick_ms);
        if elapsed_ms <= 999 {
            return GameAuctionRunReport {
                outcome: GameAuctionRunOutcome::TickNotDue { elapsed_ms },
                sampled_tick_ms: Some(sampled_tick_ms),
                sampled_wall_time_seconds: None,
                synchronized_goods: Vec::new(),
                goods_sync: None,
                state_request: None,
            };
        }
        self.auction_tick_ms = sampled_tick_ms;

        let synchronized_goods = if self.auction_now {
            self.auction_room.count_goods()
        } else {
            Vec::new()
        };
        let goods_sync = if self.auction_now {
            let mut message = CMessage::new(GAME_AUCTION_GOODS_SYNC_MESSAGE);
            message.add_ulong(synchronized_goods.len() as u32);
            for guid in &synchronized_goods {
                message.base_mut().add_guid(*guid);
            }
            Some(message.send(self, false))
        } else {
            None
        };

        let sampled_wall_time_seconds = runtime.wall_time_seconds();
        let state_expired =
            self.auction_last_check_seconds.wrapping_add(4) < sampled_wall_time_seconds;
        if state_expired {
            self.auction_now = false;
        }
        let state_request = CMessage::new(GAME_AUCTION_STATE_REQUEST_MESSAGE).send(self, false);

        GameAuctionRunReport {
            outcome: GameAuctionRunOutcome::Processed { state_expired },
            sampled_tick_ms: Some(sampled_tick_ms),
            sampled_wall_time_seconds: Some(sampled_wall_time_seconds),
            synchronized_goods,
            goods_sync,
            state_request: Some(state_request),
        }
    }

    /// Один exact `CGame::MainLoop` turn. Wrapping DWORD clocks, strict
    /// interval comparisons, profiling reads и pacing deadline сохраняют
    /// наблюдаемый Win32 порядок; wait заменён platform callback-ом.
    pub(crate) fn main_loop<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameMainLoopReport<Runtime::RuntimeError> {
        let mut state = self.main_loop_state;
        if !state.initialized {
            state.current_tick_ms = runtime.get_tick_ms();
            state.runtime_log_tick_ms = state.current_tick_ms;
            state.refresh_info_tick_ms = state.current_tick_ms;
            state.initialized = true;
        }

        state.current_tick_ms = runtime.get_tick_ms();
        state.calls_since_runtime_log = state.calls_since_runtime_log.wrapping_add(1);
        let mut stages = Vec::new();

        let refresh_elapsed = state
            .current_tick_ms
            .wrapping_sub(state.refresh_info_tick_ms);
        if self.setup.refresh_info_time_ms < refresh_elapsed {
            state.refresh_info_tick_ms = state.current_tick_ms;
            runtime.refresh_info_text(self);
            stages.push(GameMainLoopStage::RefreshInfo);
        }

        let runtime_elapsed = state
            .current_tick_ms
            .wrapping_sub(state.runtime_log_tick_ms);
        if self.setup.watch_runtime_time_ms < runtime_elapsed {
            let log = if self.setup.watch_runtime_info {
                let profile = state.profile;
                state.profile = GameMainLoopProfile::default();
                GameMainLoopRuntimeLog::Profiled {
                    elapsed_ms: runtime_elapsed,
                    ai_calls: state.calls_since_runtime_log,
                    profile,
                }
            } else {
                GameMainLoopRuntimeLog::Compact {
                    elapsed_ms: runtime_elapsed,
                    ai_calls: state.calls_since_runtime_log,
                }
            };
            runtime.add_runtime_log(log);
            stages.push(GameMainLoopStage::RuntimeLog);
            state.calls_since_runtime_log = 0;
            state.runtime_log_tick_ms = state.current_tick_ms;
        }

        if runtime.exit_requested() {
            self.main_loop_state = state;
            return GameMainLoopReport {
                outcome: GameMainLoopOutcome::ExitRequested,
                return_value: 0,
                sampled_tick_ms: state.current_tick_ms,
                ai_tick: state.ai_tick,
                stages,
                next_deadline_ms: state.pacing_initialized.then_some(state.pacing_deadline_ms),
                signed_lag_ms: None,
                messages: None,
                net_sessions: None,
                auction: None,
            };
        }

        state.ai_tick = state.ai_tick.wrapping_add(1);
        let messages;
        let net_sessions;
        if self.setup.watch_runtime_info {
            let started = runtime.get_tick_ms();
            runtime.script_loop(self);
            state.profile.script_ms = state
                .profile
                .script_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Script);

            let started = runtime.get_tick_ms();
            runtime.ai(self);
            state.profile.ai_ms = state
                .profile
                .ai_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Ai);

            let started = runtime.get_tick_ms();
            messages = self.process_messages(runtime);
            state.profile.message_ms = state
                .profile
                .message_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Message);

            let started = runtime.get_tick_ms();
            runtime.session_factory_ai(self);
            state.profile.session_ms = state
                .profile
                .session_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Session);

            let started = runtime.get_tick_ms();
            net_sessions = self.net_session_manager.run();
            state.profile.net_session_ms = state
                .profile
                .net_session_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::NetSession);
        } else {
            runtime.script_loop(self);
            stages.push(GameMainLoopStage::Script);
            runtime.ai(self);
            stages.push(GameMainLoopStage::Ai);
            messages = self.process_messages(runtime);
            stages.push(GameMainLoopStage::Message);
            runtime.session_factory_ai(self);
            stages.push(GameMainLoopStage::Session);
            net_sessions = self.net_session_manager.run();
            stages.push(GameMainLoopStage::NetSession);
        }

        let auction = self.run_auction(runtime);
        stages.push(GameMainLoopStage::Auction);

        if !state.pacing_initialized {
            state.pacing_deadline_ms = runtime.get_tick_ms();
            state.pacing_initialized = true;
        }
        let pacing_tick_ms = runtime.get_tick_ms();
        state.current_tick_ms = pacing_tick_ms;
        let interval_ms = runtime.tick_interval_ms();
        if pacing_tick_ms.wrapping_sub(state.pacing_deadline_ms) < interval_ms {
            let duration_ms = state
                .pacing_deadline_ms
                .wrapping_sub(pacing_tick_ms)
                .wrapping_add(interval_ms);
            runtime.wait(duration_ms);
            stages.push(GameMainLoopStage::Wait { duration_ms });
        }

        state.pacing_deadline_ms = state.pacing_deadline_ms.wrapping_add(interval_ms);
        let signed_lag_ms = pacing_tick_ms.wrapping_sub(state.pacing_deadline_ms) as i32;
        if 1_000 < signed_lag_ms {
            runtime.output_debug("warning!!! 1 second not call AI()\n");
            let resync_tick_ms = runtime.get_tick_ms();
            state.pacing_deadline_ms = resync_tick_ms;
            stages.push(GameMainLoopStage::LagWarning { resync_tick_ms });
        }

        self.main_loop_state = state;
        GameMainLoopReport {
            outcome: GameMainLoopOutcome::Continue,
            return_value: 1,
            sampled_tick_ms: pacing_tick_ms,
            ai_tick: state.ai_tick,
            stages,
            next_deadline_ms: Some(state.pacing_deadline_ms),
            signed_lag_ms: Some(signed_lag_ms),
            messages: Some(messages),
            net_sessions: Some(net_sessions),
            auction: Some(auction),
        }
    }

    /// Исполняет один исходный snapshot входящих FIFO в порядке WS, BS, GS.
    pub(crate) fn process_messages<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameProcessMessagesReport<Runtime::RuntimeError> {
        let mut auction_states = Vec::new();
        let mut gm_messages = Vec::new();
        let mut gma_messages = Vec::new();
        let mut depot_messages = Vec::new();
        let mut organizing_war_messages = Vec::new();
        let mut country_war_messages = Vec::new();
        let mut goods_war_messages = Vec::new();
        let mut server_messages = Vec::new();
        let world_messages = self
            .world_client
            .as_ref()
            .map(CMyNetClient::take_all_messages)
            .unwrap_or_default();
        for mut message in world_messages {
            self.run_incoming_message(
                &mut message,
                runtime,
                &mut auction_states,
                &mut gm_messages,
                &mut gma_messages,
                &mut depot_messages,
                &mut organizing_war_messages,
                &mut country_war_messages,
                &mut goods_war_messages,
                &mut server_messages,
            );
        }
        let billing_messages = self
            .billing_client
            .as_ref()
            .map(CMyNetClient::take_all_messages)
            .unwrap_or_default();
        for mut message in billing_messages {
            self.run_incoming_message(
                &mut message,
                runtime,
                &mut auction_states,
                &mut gm_messages,
                &mut gma_messages,
                &mut depot_messages,
                &mut organizing_war_messages,
                &mut country_war_messages,
                &mut goods_war_messages,
                &mut server_messages,
            );
        }
        let server_events = self
            .net_server
            .as_ref()
            .map(CMyNetServer::take_all_events)
            .unwrap_or_default();
        for event in server_events {
            match event {
                GameServerEvent::Message(mut message) => {
                    self.run_incoming_message(
                        &mut message,
                        runtime,
                        &mut auction_states,
                        &mut gm_messages,
                        &mut gma_messages,
                        &mut depot_messages,
                        &mut organizing_war_messages,
                        &mut country_war_messages,
                        &mut goods_war_messages,
                        &mut server_messages,
                    );
                }
                GameServerEvent::WorldClientReconnected(client) => {
                    runtime.handle_world_client_reconnected(self, client);
                }
                GameServerEvent::BillingClientReconnected(client) => {
                    let _legacy_ignored = on_billing_client_reconnected(self, client);
                }
            }
        }
        GameProcessMessagesReport {
            legacy_return: 1,
            auction_states,
            gm_messages,
            gma_messages,
            depot_messages,
            organizing_war_messages,
            country_war_messages,
            goods_war_messages,
            server_messages,
        }
    }

    fn run_incoming_message<Runtime: GameMainLoopRuntime>(
        &mut self,
        message: &mut CMessage,
        runtime: &mut Runtime,
        auction_states: &mut Vec<
            Result<WorldAuctionStateMessageReport, WorldAuctionStateMessageError>,
        >,
        gm_messages: &mut Vec<Result<GmMessageReport, GmMessageError>>,
        gma_messages: &mut Vec<Result<GmaMessageReport, GmaMessageError>>,
        depot_messages: &mut Vec<DepotMessageReport>,
        organizing_war_messages: &mut Vec<
            Result<GameOrganizingWarMessageReport, GameOrganizingWarMessageError>,
        >,
        country_war_messages: &mut Vec<
            Result<
                GameCountryWarMessageReport,
                CountryWarMessageDispatchError<CountryBattleStateBlock>,
            >,
        >,
        goods_war_messages: &mut Vec<Result<GameGoodsWarMessageReport, GameGoodsWarMessageError>>,
        server_messages: &mut Vec<
            Result<GameServerMessageReport, GameServerMessageError<Runtime::RuntimeError>>,
        >,
    ) {
        if let Some(report) =
            dispatch_server_message(message, self, runtime, |runtime| runtime.get_tick_ms())
        {
            server_messages.push(report);
        } else if let Some(report) =
            dispatch_world_auction_state(message, self, || runtime.wall_time_seconds())
        {
            auction_states.push(report);
        } else if let Some(report) = dispatch_gm_message(message, self, || runtime.get_tick_ms()) {
            gm_messages.push(report);
        } else if let Some(report) = dispatch_gma_message(message, self) {
            gma_messages.push(report);
        } else if let Some(report) = dispatch_depot_message(message, self) {
            depot_messages.push(report);
        } else if let Some(report) = dispatch_game_organizing_war_message(message, self, runtime) {
            organizing_war_messages.push(report);
        } else if let Some(report) = dispatch_game_country_war_message(message, self, runtime) {
            country_war_messages.push(report);
        } else if let Some(report) = dispatch_game_goods_war_message(message, self) {
            goods_war_messages.push(report);
        } else {
            message.run(self, runtime);
        }
    }
}

/// Safe process-owned замена `GameThreadFunc`: COM/platform notifications
/// остаются runtime callbacks, а `CGame` всегда проходит Release даже после
/// неуспешного Init, как исходный ненулевой singleton `GetGame`.
pub(crate) async fn game_thread_func<Runtime: GameThreadRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
) -> GameThreadReport {
    runtime.initialize_com();
    let paths = runtime.runtime_paths();
    let wall_time_seconds = runtime.wall_time_seconds();
    let sequence_seed_ms = runtime.sequence_seed_ms();
    let initialization = game.init(&paths, wall_time_seconds, sequence_seed_ms).await;

    let mut main_loop_calls = 0usize;
    if initialization.is_ok() {
        loop {
            let turn = game.main_loop(runtime);
            main_loop_calls = main_loop_calls.wrapping_add(1);
            if turn.return_value == 0 {
                break;
            }
        }
    }

    let release = game.release(runtime).await;
    runtime.signal_game_thread_exit();
    runtime.post_process_close();
    runtime.uninitialize_com();
    GameThreadReport {
        initialization,
        main_loop_calls,
        release,
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

fn nation_colored_text_message(
    message_type: i32,
    first_color: u32,
    second_color: u32,
    text: &[u8],
) -> CMessage {
    let mut message = CMessage::new(message_type);
    message.add_ulong(first_color);
    message.add_ulong(second_color);
    add_legacy_c_string(message.base_mut(), text);
    message
}

fn nation_world_notice_message(text: &[u8]) -> CMessage {
    let mut message = CMessage::new(0x5fd06);
    message.add_long(0);
    message.add_long(0);
    message.add_ulong(0xffff_ffff);
    message.add_ulong(0xff00_6ee1);
    add_legacy_c_string(message.base_mut(), text);
    message
}

/// Bounded replacement for the reached `__snprintf` `%s` subset. The EXE
/// buffers reserve one byte for NUL (`0x100`/`0x80`), hence visible limits
/// `0xff` and `0x7f` at callers.
fn format_legacy_text_fields(
    template: &[u8],
    arguments: &[&[u8]],
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() && output.len() < maximum_bytes {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        match template.get(offset + 1).copied() {
            Some(b'%') => {
                output.push(b'%');
                offset += 2;
            }
            Some(b's') => {
                let Some(argument) = arguments.get(argument_index) else {
                    output.extend_from_slice(&template[offset..]);
                    break;
                };
                let remaining = maximum_bytes.saturating_sub(output.len());
                let argument = legacy_c_string_prefix(argument);
                output.extend_from_slice(&argument[..argument.len().min(remaining)]);
                argument_index += 1;
                offset += 2;
            }
            _ => {
                output.push(b'%');
                offset += 1;
            }
        }
    }
    output.truncate(maximum_bytes);
    output
}

fn game_kick_around_block(
    outcome: GameKickAroundOutcome,
    target_player_id: Option<i32>,
    server_region_id: Option<i32>,
) -> GameKickAroundReport {
    GameKickAroundReport {
        outcome,
        target_player_id,
        server_region_id,
        window_origin: None,
        matched_player_ids: Vec::new(),
        kicks: Vec::new(),
    }
}

fn gm_kick_window_origin(tile: i32, extent: i32) -> i32 {
    if tile.wrapping_add(3) >= extent {
        extent.wrapping_sub(7)
    } else if tile.wrapping_sub(3) <= 0 {
        0
    } else {
        tile.wrapping_sub(3)
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn format_single_legacy_i32(template: &[u8], value: i32, maximum_bytes: usize) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let Some(marker) = template.windows(2).position(|window| window == b"%d") else {
        return template[..template.len().min(maximum_bytes)].to_vec();
    };
    let value = value.to_string();
    let mut result = Vec::with_capacity(template.len().saturating_add(value.len()));
    result.extend_from_slice(&template[..marker]);
    result.extend_from_slice(value.as_bytes());
    result.extend_from_slice(&template[marker + 2..]);
    result.truncate(maximum_bytes);
    result
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
        match identity.object_type {
            PLAYER_TYPE => {
                let player = self.find_player(identity.id)?;
                let view = player.shape_view()?;
                (view.identity.object_type == identity.object_type
                    && view.identity.id == identity.id)
                    .then_some(view)
            }
            MONSTER_TYPE => {
                let monster = self
                    .regions
                    .values()
                    .find_map(|region| region.base().find_monster_by_id(identity.id))?;
                let property =
                    self.find_monster_property_by_origin_name(monster.base_property_key()?)?;
                shape_view(monster.move_shape().shape(), CMonster::figure(property))
            }
            NPC_TYPE => {
                let npc = self
                    .regions
                    .values()
                    .find_map(|region| region.base().find_npc_by_id(identity.id))?;
                shape_view(npc.move_shape().shape(), ShapeFigure::default())
            }
            _ => None,
        }
    }
}

fn shape_view(
    shape: &crate::gameserver::appserver::shape::CShape,
    figure: ShapeFigure,
) -> Option<ShapeView> {
    Some(ShapeView {
        identity: shape.identity(),
        tile_x: shape.get_tile_x().ok()?,
        tile_y: shape.get_tile_y().ok()?,
        figure,
    })
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

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `CGame::KickPlayer` RVA `0x000022F0`
// материализован выше с exact `QuitClientByMapID` side effect и постоянным
// `false` return; покрытый raw удалён.

// `SetFunctionFileData`, `SetVariableFileData` и `SetGeneralVariableFileData`
// материализованы выше с подтверждённой семантикой владения и повторной публикации.

// IMPLEMENTED: `SetAuctionState` RVA `0x00002390` материализован выше
// и связан с exact World `0x80403` caller-ом; покрытый raw удалён.

// IMPLEMENTED: `InitNetClientOfWS/BS` материализованы выше с исходными
// registration packets и master -> backup Billing порядком.

// `CreateStringTable` материализован выше с clear/decode/log/cursor порядком.

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `FindPlayer(char const*)` RVA `0x00003A60`
// материализован выше как ordered byte-name lookup; покрытый raw удалён.
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

// IMPLEMENTED: достигнутый `Release` teardown материализован выше. Широкий
// RAW-блок и split funclets ниже сохранены как доказательство ещё не
// материализованных player serializer-а, SaveCityRegion, validate-time map и
// exception-specific debug paths; он не считается полностью заменённым.

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

// `SetScriptFileData` материализован выше: ключ ограничен первой NUL,
// повторная публикация заменяет прежний owned buffer.

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

// IMPLEMENTED: `RunAuction` RVA `0x0000BC30` материализован выше и вызывается
// напрямую из `MainLoop`; покрытый raw удалён.

// IMPLEMENTED: `CreateConnectWorldThread` RVA `0x0000BDA0` и
// `CreateConnectBillingThread` RVA `0x0000BE10` материализованы выше как owned
// Tokio tasks; начальный stop/join обеих задач из `Release` также перенесён.

// IMPLEMENTED: `GetTeamSessionID` и `FindPlayer` материализованы выше;
// покрытые raw-блоки удалены.

// `GetStringByID` материализован выше с исходным empty-string fallback.

// `GetScriptFileData` материализован выше как lookup без вставки отсутствующего ключа.

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
