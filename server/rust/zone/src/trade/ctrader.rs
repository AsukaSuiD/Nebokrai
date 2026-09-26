//! Скалярные правила рамки двустороннего обмена GameServer `CTrader`,
//! перенесённые в Zone `trade/`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/session/ctrader.cpp` (правила рамки и
//! предложений) и `server/gameserver/appserver/game.cpp` (маршрутизация
//! источника предложения и сверка количества на commit). Мгновенный владелец
//! трёх shadow-контейнеров `(goods, Gold, YuanBao)` и конкретных
//! shadow-отчётов остаётся прежним `appserver/session/ctrader.rs`
//! (`CPersonalShopSeller`-прецедент: container-владеющий plug живёт у старого
//! пакета, потому что контейнеры и их listener-отчёты ещё принадлежат
//! контейнерной порции). Registry поиск plug-ов выполняет `CSessionFactory`,
//! wire доставку — message runtime caller-ы. Здесь — объявленные kind индексов
//! рамки, скалярная приёмка предложения до занятия ячейки, таблица
//! допустимых player-контейнеров источника, сверка количества и правила
//! обратимости stack-merge при rollback.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинный статус — `MATCH` по подсемейству trade разведки «Zone
//! player: машинная разведка trade/auction/bank/ground currency» от 26
//! сентября 2026 (CTrader трёх shadow-контейнеров; reset ready у обоих
//! участников при любой смене рамки `OnObjectAdded`; commit `0x1B9470` —
//! 1241 insn, per-goods AmountChange → packet Add → move publish; частичный
//! отказ → Clear(temp) + RollBack(map) полностью и немедленно):
//!
//! | правило | машинный факт | здесь | статус |
//! |---|---|---|---|
//! | extend-id рамки | `plug << 8 \| index` (`0/1/2` = goods/Gold/YuanBao) | [`trade_container_extend_id`], [`trade_extend_id_parts`] | семья `MATCH` |
//! | приёмка goods | currency в goods → NoTrade flag `0x20` → позиция за объёмом → занятость ячейки | [`trade_goods_offer_block`] | семья `MATCH` |
//! | приёмка валюты | несовпадение index ожидаемой валюты, единственная позиция `0` | [`trade_currency_offer_block`] | семья `MATCH` |
//! | источник предложения | goods ← packet `1`/equipment `2`, Gold ← wallet `4`, YuanBao ← increment `5` | [`TradeSourceContainer`], [`trade_offer_player_container_allowed`] | семья `MATCH` |
//! | сверка количества | packet снимает часть стека (`>=`), экипировка целиком (`==`) | [`trade_offer_amount_satisfies`] | семья `MATCH` |
//! | rollback stack-merge | полный merge (`amount == original`) обратим вычитанием; иначе `None` | [`trade_rollback_merge_reversible`] | семья `MATCH` |
//!
//! Риск-ноты переноса (разведка порции T):
//!
//! - Reset ready: `record_offer`/`remove_offer`/`clear` прежнего owner
//!   немедленно сбрасывают собственный `ready`, а `CGame` затем сбрасывает
//!   готовность ОБОИХ участников и рассылает `0xBF716`. Между своим локальным
//!   сбросом и внешним сбросом второго участника есть micro-окно исходного
//!   порядка — оно является контрактом очерёдности оригинала и сознательно не
//!   уплотняется.
//! - Rollback в commit: когда доставленный предмет слился со стеком
//!   получателя НЕ полностью (исходный `RollBack` получает неслитый остаток),
//!   обратного удаления части стека в Rust-владельце нет — как и у
//!   release-формы оригинала вопрос о частичном unmerge остаётся
//!   [`trade_rollback_merge_reversible`] == `false` (возврат `None` caller-а).
//!
//! Швы переноса: числовые index рамки и скалярные предикаты перенесены
//! буквально (pub/пути нормализованы); контейнерные операции `record_offer`
//! (occupy/record/clear) и отчёты `ShadowPresenceReport/ShadowRemovedReport`
//! остаются у прежнего владельца, который вызывает правила ниже в исходном
//! порядке. Константы сессии обмена материализованы здесь же, потому что
//! `CSessionFactory` создаёт рамку именно с этими параметрами.

/// Owner type shadow-контейнеров рамки: `(10, session_id)` исходного owner.
pub const SESSION_OWNER_TYPE: i32 = 10;

/// Объём рамки товаров `CTrader`: 32 ячейки исходного ctor.
pub const TRADE_GOODS_CELLS: u32 = 32;

/// Particular-флаг запрета обмена товара в рамку goods (`GAP 0x13`).
pub const TRADE_GOODS_NO_TRADE_FLAG: u32 = 0x20;

/// Параметры `CSession::normal` обмена: минимум/максимум plug-ов и lifetime.
pub const TRADE_SESSION_MINIMUM_PLUGS: u32 = 2;
pub const TRADE_SESSION_MAXIMUM_PLUGS: u32 = 2;
pub const TRADE_SESSION_LIFETIME: u32 = 0;

/// Plug type обоих участников обмена (пригласивший и отвечающий).
pub const TRADE_PLUG_TYPE: u32 = 1;

/// Контейнер рамки `CTrader`, индекс которого кодируется младшим байтом
/// extend-id (`plug << 8 | index`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraderContainerKind {
    Goods,
    Gold,
    YuanBao,
}

impl TraderContainerKind {
    pub const fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Goods),
            1 => Some(Self::Gold),
            2 => Some(Self::YuanBao),
            _ => None,
        }
    }

    pub const fn index(self) -> i32 {
        match self {
            Self::Goods => 0,
            Self::Gold => 1,
            Self::YuanBao => 2,
        }
    }
}

/// Отказ записи предложения `record_offer` до/вместо фактической записи
/// shadow исходного `CTrader`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraderOfferBlock {
    InvalidContainer,
    InvalidPosition,
    MissingGoods,
    CurrencyInGoodsContainer,
    InvalidCurrency,
    NoTrade,
    Occupied,
    ShadowRejected,
}

/// Player-контейнер источника предложения обмена; та же extend-id таблица
/// читается живым фасадом `CPlayer::trade_source_goods` при сверке и commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeSourceContainer {
    Packet,
    Equipment,
    Wallet,
    YuanBao,
}

impl TradeSourceContainer {
    pub const fn from_extend_id(extend_id: i32) -> Option<Self> {
        match extend_id {
            1 => Some(Self::Packet),
            2 => Some(Self::Equipment),
            4 => Some(Self::Wallet),
            5 => Some(Self::YuanBao),
            _ => None,
        }
    }

    pub const fn extend_id(self) -> i32 {
        match self {
            Self::Packet => 1,
            Self::Equipment => 2,
            Self::Wallet => 4,
            Self::YuanBao => 5,
        }
    }
}

/// Упаковка extend-id рамки исходного ctor: `plug << 8 | kind.index()`.
pub const fn trade_container_extend_id(plug_id: i32, kind: TraderContainerKind) -> i32 {
    plug_id.wrapping_shl(8) | kind.index()
}

/// Разбор extend-id рамки: plug id старшими битами и kind младшим байтом.
pub const fn trade_extend_id_parts(extend_id: i32) -> (i32, Option<TraderContainerKind>) {
    (extend_id >> 8, TraderContainerKind::from_index(extend_id & 0xff))
}

/// Приёмка предложения и его снятия проходит одну таблицу: goods предлагается
/// только из packet `1`/equipment `2`, Gold — из wallet `4`, YuanBao — из
/// increment-контейнера `5`. Обе ветви request-валидации `CGame` (add и
/// remove) машинно используют одну и ту же таблицу.
pub const fn trade_offer_player_container_allowed(
    kind: TraderContainerKind,
    player_container_extend_id: i32,
) -> bool {
    match kind {
        TraderContainerKind::Goods => matches!(player_container_extend_id, 1 | 2),
        TraderContainerKind::Gold => player_container_extend_id == 4,
        TraderContainerKind::YuanBao => player_container_extend_id == 5,
    }
}

/// Первый отказ `record_offer`: нулевое или превышающее живой источник
/// количество не создаёт предложение.
pub const fn trade_offer_missing_goods(amount: u32, goods_amount: u32) -> bool {
    amount == 0 || amount > goods_amount
}

/// Скалярная приёмка рамки goods `record_offer` до занятия ячейки: currency
/// в goods-рамке и particular-флаг `0x20` отклоняются до проверки позиции против
/// объёма контейнера. Занятость ячейки (`Occupied`) и запись shadow остаются
/// контейнерной операцией прежнего owner.
pub const fn trade_goods_offer_block(
    base_properties_index: u32,
    gold_coin_index: u32,
    yuan_bao_index: u32,
    particular_attribute: u32,
    position: u32,
    container_size: u32,
) -> Option<TraderOfferBlock> {
    if base_properties_index == gold_coin_index || base_properties_index == yuan_bao_index {
        Some(TraderOfferBlock::CurrencyInGoodsContainer)
    } else if particular_attribute & TRADE_GOODS_NO_TRADE_FLAG != 0 {
        Some(TraderOfferBlock::NoTrade)
    } else if position >= container_size {
        Some(TraderOfferBlock::InvalidPosition)
    } else {
        None
    }
}

/// Скалярная приёмка рамки валюты `record_offer`: ожидаемый index валюты
/// выбирается по kind (Gold иначе YuanBao — ветка `else` исходного кода), а
/// допустима единственная позиция `0`. Перезапись прежней записи, очистка и
/// запись shadow — контейнерные операции прежнего owner.
pub const fn trade_currency_offer_block(
    kind: TraderContainerKind,
    base_properties_index: u32,
    gold_coin_index: u32,
    yuan_bao_index: u32,
    position: u32,
) -> Option<TraderOfferBlock> {
    let expected = match kind {
        TraderContainerKind::Gold => gold_coin_index,
        _ => yuan_bao_index,
    };
    if base_properties_index != expected {
        Some(TraderOfferBlock::InvalidCurrency)
    } else if position != 0 {
        Some(TraderOfferBlock::InvalidPosition)
    } else {
        None
    }
}

/// Сверка количества источника с предложением при валидации и commit
/// `0x1B9470`: источник из packet снимается частично (`>=`), источник из
/// экипировки — целиком (`==`).
pub const fn trade_offer_amount_satisfies(
    source_container_extend_id: i32,
    source_amount: u32,
    offer_amount: u32,
) -> bool {
    if source_container_extend_id == 1 {
        source_amount >= offer_amount
    } else {
        source_amount == offer_amount
    }
}

/// Обратимость stack-merge исходного `RollBack` commit `0x1B9470`: доставка,
/// слившаяся со стеком получателя ПОЛНОСТЬЮ (`merged amount == original
/// amount`), откатывается вычитанием исходного количества. Неполный merge
/// владелец не обращает (caller получает `None`) — risk-note этого файла.
pub const fn trade_rollback_merge_reversible(merged_amount: u32, original_amount: u32) -> bool {
    merged_amount == original_amount
}
