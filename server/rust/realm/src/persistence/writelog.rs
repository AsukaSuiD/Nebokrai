//! Payload-контракт FIFO журнала WorldServer Realm.
//!
//! Источник — точная пара `Nworldserver.exe`/`WorldServer.pdb`
//! (RSDS 289F1FB3-96A0-4FF4-8B5D-1FD17B50B751).

use nebokrai_shared::values::{CGuid, TagTime};

use crate::auction::auctionlog::AuctionLogNode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldIncrementLogWrite {
    pub context_id: Vec<u8>,
    pub entry_type: u8,
    pub money: i32,
    pub description: Vec<u8>,
    pub player_id: i32,
    pub player_account: Vec<u8>,
    pub player_level: u8,
    pub item_name: Vec<u8>,
    pub item_amount: i32,
    pub ip_address: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct WorldCarriageLogWrite {
    pub player_id: i32,
    pub carriage_id: i32,
    pub region_id: i32,
    pub coordinate_x: i16,
    pub coordinate_y: i16,
    pub event_type: i32,
    pub event_time: TagTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlainLogWrite {
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub player_account: Vec<u8>,
    pub content: Vec<u8>,
    pub log_type: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCiqingLogWrite {
    pub player_id: i32,
    pub in_out: i32,
    pub entry_type: i32,
    pub base_index: i32,
    pub amount: i32,
}

#[derive(Clone, Debug)]
pub struct WorldFairyLogWrite {
    pub player_id: i32,
    pub event_time: TagTime,
    pub event: WorldFairyLogEvent,
}

#[derive(Clone, Debug)]
pub enum WorldFairyLogEvent {
    Grow {
        is_jing_po: i32,
        goods_id: Vec<u8>,
        goods_name: Vec<u8>,
        current_level: i32,
    },
    Take {
        goods_id: CGuid,
        goods_name: Vec<u8>,
        current_level: i32,
        grow_rate_raw: u32,
        main_fetch: i32,
        combined_times: i32,
    },
    Implantation {
        goods_name: Vec<u8>,
        goods_id: CGuid,
        previous_level: i32,
        current_level: i32,
        west_diamond: i32,
    },
    Incubate {
        goods_id: CGuid,
        goods_name: Vec<u8>,
    },
    Syncretize {
        main_goods_id: CGuid,
        main_goods_name: Vec<u8>,
        main_level: i32,
        main_grow_rate_raw: u32,
        secondary_goods_id: CGuid,
        secondary_goods_name: Vec<u8>,
        secondary_level: i32,
        secondary_grow_rate_raw: u32,
        west_patch: i32,
        child_goods_id: CGuid,
        child_goods_name: Vec<u8>,
        child_main_ability: i32,
        child_syncretize_times: i32,
        child_grow_rate_raw: u32,
    },
}

#[derive(Clone, Debug)]
pub struct WorldAuctionLogWrite {
    pub record: AuctionLogNode,
    pub log_time: TagTime,
}

#[derive(Clone, Debug)]
pub struct WorldAuctionSaleLogWrite {
    pub event_time: TagTime,
    pub event: WorldAuctionSaleLogEvent,
}

#[derive(Clone, Debug)]
pub enum WorldAuctionSaleLogEvent {
    Oper {
        player_id: u32,
        base_index: u32,
        guid: CGuid,
        amount: u32,
        money: u32,
        time_type: u32,
        fee: u32,
    },
    Cancel {
        player_id: u32,
        guid: CGuid,
    },
    Receive {
        player_id: u32,
        amount: u32,
        guid: CGuid,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlayerProgressLogWrite {
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub event: WorldPlayerProgressLogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldPlayerProgressLogEvent {
    Level {
        experience: i32,
        old_level: u8,
        current_level: u8,
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
    Experience {
        experience: i32,
        map_id: i32,
        position_x: i32,
        position_y: i32,
        log_type: u8,
    },
    Died {
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlayerRelationLogWrite {
    pub first_player_id: i32,
    pub first_player_name: Vec<u8>,
    pub second_player_id: i32,
    pub second_player_name: Vec<u8>,
    pub event: WorldPlayerRelationLogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldPlayerRelationLogEvent {
    Team {
        map_id: i32,
        wire_position_x: i32,
        position_y: i32,
        log_type: u8,
    },
    Killer {
        map_id: i32,
        position_x: i32,
        position_y: i32,
        log_type: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGoodsTradeLogWrite {
    pub log_type: u8,
    pub seller_id: i32,
    pub seller_name: Vec<u8>,
    pub seller_current_money: i32,
    pub seller_map_id: i32,
    pub seller_position_x: i32,
    pub seller_position_y: i32,
    pub purchaser_id: i32,
    pub purchaser_name: Vec<u8>,
    pub purchaser_current_money: i32,
    pub purchaser_map_id: i32,
    pub purchaser_position_x: i32,
    pub purchaser_position_y: i32,
    pub goods_id: CGuid,
    pub goods_name: Vec<u8>,
    pub price: i32,
    pub amount: i32,
    pub buyer_ip_address: Vec<u8>,
    pub seller_ip_address: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGoodsLogWrite {
    pub log_type: u8,
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub pk_count: u16,
    pub current_money: i32,
    pub current_bank: i32,
    pub goods_id: CGuid,
    pub goods_name: Vec<u8>,
    pub goods_amount: i32,
    pub price: i32,
    pub map_id: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub ip_address: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGoodsCraftLogWrite {
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub event: WorldGoodsCraftLogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldGoodsCraftLogEvent {
    Upgrade {
        goods_id: CGuid,
        goods_name: Vec<u8>,
        gems: [(CGuid, Vec<u8>); 4],
        map_id: i32,
        position_x: i32,
        position_y: i32,
        log_type: u8,
    },
    GemExchange {
        destination_gem_id: CGuid,
        destination_gem_name: Vec<u8>,
        source_gem_id: CGuid,
        source_gem_name: Vec<u8>,
        source_gem_amount: i32,
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
    JewelryMade {
        goods_id: CGuid,
        goods_name: Vec<u8>,
        material_id: CGuid,
        material_name: Vec<u8>,
        jade_id: CGuid,
        jade_name: Vec<u8>,
        jade_amount: i32,
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldChatLogWrite {
    pub sender_id: i32,
    pub sender_name: Vec<u8>,
    pub map_id: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub receiver_id: i32,
    pub receiver_name: Vec<u8>,
    pub content: Vec<u8>,
    pub log_type: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldChangeMapLogWrite {
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub money: i32,
    pub bank: i32,
    pub source_map_id: i32,
    pub source_position_x: i32,
    pub source_position_y: i32,
    pub destination_map_id: i32,
    pub destination_position_x: i32,
    pub destination_position_y: i32,
    pub log_type: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPlayerDeleteLogWrite {
    pub player_id: i32,
    pub player_name: Vec<u8>,
    pub ip_address: Vec<u8>,
}

/// Owned-поля SQL-записей организационной системы. Один enum сохраняет общий
/// FIFO, а варианты различают таблицы и их исходный positional контракт.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldFactionLogWrite {
    Faction {
        faction_id: i32,
        faction_name: Vec<u8>,
        player_id: i32,
        player_name: Vec<u8>,
        log_type: i32,
    },
    Member {
        member_id: i32,
        member_name: Vec<u8>,
        manager_id: i32,
        manager_name: Vec<u8>,
        faction_id: i32,
        faction_name: Vec<u8>,
        log_type: i32,
    },
    Title {
        member_id: i32,
        member_name: Vec<u8>,
        old_title: Vec<u8>,
        new_title: Vec<u8>,
        manager_id: i32,
        manager_name: Vec<u8>,
        faction_id: i32,
        faction_name: Vec<u8>,
    },
    Purview {
        member_id: i32,
        member_name: Vec<u8>,
        purview: i32,
        manager_id: i32,
        manager_name: Vec<u8>,
        faction_id: i32,
        faction_name: Vec<u8>,
        log_type: i32,
    },
    Level {
        faction_id: i32,
        faction_name: Vec<u8>,
        level: i32,
        master_id: i32,
        master_name: Vec<u8>,
    },
    Experience {
        faction_id: i32,
        faction_name: Vec<u8>,
        member_id: i32,
        member_name: Vec<u8>,
        before_experience: i32,
        experience: i32,
    },
    Master {
        old_master_id: i32,
        old_master_name: Vec<u8>,
        new_master_id: i32,
        new_master_name: Vec<u8>,
        faction_id: i32,
        faction_name: Vec<u8>,
    },
}

#[derive(Clone, Debug)]
pub enum WorldWriteLogCommand {
    IncrementLog(WorldIncrementLogWrite),
    LargessLog(LargessWriteLog),
    CarriageLog(WorldCarriageLogWrite),
    PlainLog(WorldPlainLogWrite),
    CiqingLog(WorldCiqingLogWrite),
    FairyLog(WorldFairyLogWrite),
    AuctionLog(WorldAuctionLogWrite),
    AuctionSaleLog(WorldAuctionSaleLogWrite),
 /// SQL `CAuctionLog::CollectNoNotice`, созданный только из typed GUID и
 /// opttype аукционного журнала; сохраняет FIFO-публикацию owner-а.
    AuctionNoticeSql(String),
    PlayerProgressLog(WorldPlayerProgressLogWrite),
    PlayerRelationLog(WorldPlayerRelationLogWrite),
    GoodsTradeLog(WorldGoodsTradeLogWrite),
    GoodsLog(WorldGoodsLogWrite),
    GoodsCraftLog(WorldGoodsCraftLogWrite),
    ChatLog(WorldChatLogWrite),
    LegacyEmptyChatSql { log_type: u8 },
    ChangeMapLog(WorldChangeMapLogWrite),
    PlayerDeleteLog(WorldPlayerDeleteLogWrite),
    FactionLog(WorldFactionLogWrite),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LargessWriteLog {
    pub account: Vec<u8>,
    pub player_id: i32,
    pub send_time: Vec<u8>,
    pub goods_id: Vec<u8>,
    pub goods_index: u32,
    pub goods_name: Vec<u8>,
    pub goods_level: i32,
    pub send_num: i32,
    pub sent_num: i32,
    pub current_sent_num: i32,
    pub result: Vec<u8>,
}
