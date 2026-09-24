//! Общие владельцы исходного каталога `public`.

#[allow(
    dead_code,
    reason = "CiQing serializer подключён к combined initial-config до точного text loader-а"
)]
pub(crate) mod ciqing;

#[allow(
    dead_code,
    reason = "CRC подключён до восстановления межсерверного envelope и ServerUpdate"
)]
pub(crate) mod crc32static;

#[allow(
    dead_code,
    reason = "auction rooms подключены к MiscServer M2M и GameServer World reconciliation"
)]
#[path = "auctionroom/aucitionroom.rs"]
pub(crate) mod aucitionroom;
#[allow(
    dead_code,
    reason = "CAuctionLog materialized before its World auction handler and LogDB loader"
)]
#[path = "auctionroom/auctionlog.rs"]
pub(crate) mod auctionlog;
#[allow(
    dead_code,
    reason = "CGoodsNode подключён к AddItemToAuctionRoom и auction-handler 0x14ED01"
)]
#[path = "auctionroom/auctionnode.rs"]
pub(crate) mod auctionnode;

pub(crate) mod md5;
#[allow(
    dead_code,
    reason = "CWordsFilter подключён к CGame до reload и initial-config consumers"
)]
pub(crate) mod wordsfilter;

#[allow(
    dead_code,
    reason = "CNetSessionManager подключён к World MainLoop; доменные async producers достигаются следующими owner-проходами"
)]
pub(crate) mod netsession;
#[allow(
    dead_code,
    reason = "CNetSessionManager подключён к World/Game MainLoop; доменные async producers ещё достигаются"
)]
pub(crate) mod netsessionmanager;

#[allow(
    dead_code,
    reason = "tagTime подключён для calendar-ветви CTimer; остальная арифметика owner-а ещё raw"
)]
pub(crate) mod date;

#[allow(
    dead_code,
    reason = "DaKong owner подключён к World reload и initial-config до runtime query прохода"
)]
pub(crate) mod dakongxiangqian;

#[allow(
    dead_code,
    reason = "duplicate-region serializer подключён к initial-config до точного loader-а и RNG"
)]
pub(crate) mod dupliregionsetup;

#[allow(
    dead_code,
    reason = "equipment-compose serializer подключён к initial-config до text loader-а"
)]
pub(crate) mod equipmentcomposelist;

#[allow(
    dead_code,
    reason = "TaoZhuang serializer подключён к initial-config до точного text loader-а"
)]
pub(crate) mod taozhuangsetup;

#[allow(
    dead_code,
    reason = "CTimer подключён к World MainLoop до materialization всех registration callers"
)]
pub(crate) mod timer;

pub(crate) mod clientresource;
pub(crate) mod readwrite;
pub(crate) mod tools;
