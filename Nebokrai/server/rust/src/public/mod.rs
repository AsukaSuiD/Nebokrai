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
    reason = "GUID подключён к сборке до восстановления сообщений, БД и игровых владельцев"
)]
pub(crate) mod guid;

#[allow(
    dead_code,
    reason = "CAuctionRoom подключён для восстановления MiscServer M2M до остальных auction-владельцев"
)]
#[path = "auctionroom/aucitionroom.rs"]
pub(crate) mod aucitionroom;
#[allow(
    dead_code,
    reason = "CGoodsNode подключён к AddItemToAuctionRoom и auction-handler 0x14ED01"
)]
#[path = "auctionroom/auctionnode.rs"]
pub(crate) mod auctionnode;
#[allow(
    dead_code,
    reason = "stPlayerOptNode хранит доказанное use-self поле, которое текущий фильтр не читает"
)]
#[path = "auctionroom/auctionroom.rs"]
pub(crate) mod auctionroom;
#[allow(
    dead_code,
    reason = "CAuctionLog materialized before its World auction handler and LogDB loader"
)]
#[path = "auctionroom/auctionlog.rs"]
pub(crate) mod auctionlog;

pub(crate) mod md5;
pub(crate) mod mystringtable;
pub(crate) mod stringtable;
#[allow(
    dead_code,
    reason = "CharCodeFilter materialized for World name filtering before its snapshot serializer"
)]
pub(crate) mod char_code_filter;
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
    reason = "CNetSessionManager подключён к World MainLoop; Game call sites остаются в сыром компоненте"
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

pub(crate) mod readwrite;
pub(crate) mod tools;
#[allow(
    dead_code,
    reason = "CRFile подключён перед восстановлением CClientResource/package-opening"
)]
pub(crate) mod rfile;
#[allow(
    dead_code,
    reason = "FilesInfo подключён перед materialization CClientResource и package-opening"
)]
pub(crate) mod filesinfo;
