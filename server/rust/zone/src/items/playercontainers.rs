//! Единый каталог extend-id плеер-контейнеров GameServer (дизайн-решение D4),
//! заведённый в Zone `items/` — у владельца типов контейнеров.
//!
//! Волна Z-C1 вводит каталог как типизированный enum: каждому live-контейнеру
//! `CPlayer` соответствует wire-номер, которым message boundary адресует его в
//! `*_container_extend_id` полях. Значения собраны из уже установленных в коде
//! констант и маршрутов, а не из новой машинной сверки этой волны: equipment `2`
//! (`EQUIPMENT_CONTAINER_EXTEND_ID`), fairy `0x0b` (`FAIRY_CONTAINER_EXTEND_ID`),
//! battle fairy `0x0c` и packet-фасадные ci_qing `16`/`17` — `CPlayer` старого
//! пакета, shadow wallet/YuanBao `4`/`5` — session-контейнеры, маршруты
//! банковской и наземной валюты `4↔8`/`15→4`/`3|4|5` — машинно подтверждённые
//! таблицы Zone `trade/currency.rs` (статус MATCH их порции).
//!
//! Область применения пока ограничена перенесёнными этой волной файлами:
//! currency и bank owners документируют свой extend-id ссылкой на вариант ниже.
//! Литералы других семейств (message-роутеры, shadow/session-упаковки, owners
//! depot/fairy/equipment/enhancement и auction) здесь сознательно не мигрируют —
//! они переходят на каталог каждый своей волной. Trade-рамка (`plug << 8 | kind`)
//! и session-упаковки extend-id каталогом не унифицируются: осознанный отказ,
//! зафиксированный порцией T; их таблицы остаются в Zone `trade/`.

/// Live-контейнер `CPlayer`, адресуемый extend-id на message boundary.
///
/// Покрывает только плеер-контейнеры исходной карты адресации; номера вне
/// таблицы (включая `0` и резервы исходной нумерации) контейнера не обозначают
/// и отклоняются [`PlayerContainerKind::from_extend_id`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlayerContainerKind {
    /// Основная сумка player (`CVolumeLimitGoodsContainer`-владелец).
    Packet,
    /// Экипировка (`CEquipmentContainer`-владелец, `EQUIPMENT_CONTAINER_EXTEND_ID`).
    Equipment,
    /// Однослотовый hand-контейнер ground drop/pickup.
    Hand,
    /// Однослотовый wallet золота (`CWallet`).
    Wallet,
    /// Однослотовый инкремент-кошелёк (`CYuanBao`).
    YuanBao,
    /// Однослотовый JiFen-контейнер (`CJiFen`).
    JiFen,
    /// Банковская ячейка золота (`CBank` lock-gate).
    Bank,
    /// Склад (`CDepot`-владелец).
    Depot,
    /// Контейнер заточки enhancement (`set_container_extend_id(10)` фасада).
    Enhancement,
    /// Феи (`CFairyContainer`-владелец, `FAIRY_CONTAINER_EXTEND_ID`).
    Fairy,
    /// Боевые феи (`BATTLE_FAIRY_CONTAINER_EXTEND_ID` фасада).
    BattleFairy,
    /// Товары аукциона (auction goods).
    AuctionGoods,
    /// Деньги аукциона (auction wallet `CYuanBao`-семейства).
    AuctionWallet,
    /// CiQing packet-фасада `CPlayer` (`compose_container == false`).
    CiQing,
    /// CiQing compose packet-фасада `CPlayer` (`compose_container == true`).
    CiQingCompose,
}

impl PlayerContainerKind {
    /// Wire-номер контейнера исходной адресации.
    pub const fn extend_id(self) -> i32 {
        match self {
            Self::Packet => 1,
            Self::Equipment => 2,
            Self::Hand => 3,
            Self::Wallet => 4,
            Self::YuanBao => 5,
            Self::JiFen => 6,
            Self::Bank => 8,
            Self::Depot => 9,
            Self::Enhancement => 10,
            Self::Fairy => 11,
            Self::BattleFairy => 12,
            Self::AuctionGoods => 14,
            Self::AuctionWallet => 15,
            Self::CiQing => 16,
            Self::CiQingCompose => 17,
        }
    }

    /// Разбор wire-номера; номера вне каталога отклоняются.
    pub const fn from_extend_id(extend_id: i32) -> Option<Self> {
        match extend_id {
            1 => Some(Self::Packet),
            2 => Some(Self::Equipment),
            3 => Some(Self::Hand),
            4 => Some(Self::Wallet),
            5 => Some(Self::YuanBao),
            6 => Some(Self::JiFen),
            8 => Some(Self::Bank),
            9 => Some(Self::Depot),
            10 => Some(Self::Enhancement),
            11 => Some(Self::Fairy),
            12 => Some(Self::BattleFairy),
            14 => Some(Self::AuctionGoods),
            15 => Some(Self::AuctionWallet),
            16 => Some(Self::CiQing),
            17 => Some(Self::CiQingCompose),
            _ => None,
        }
    }
}
