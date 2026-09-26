//! Правила и агрегат рамки двустороннего обмена GameServer `CTrader`,
//! перенесённые в Zone `trade/`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/session/ctrader.cpp` (агрегат, правила рамки и
//! предложений) и `server/gameserver/appserver/game.cpp` (маршрутизация
//! источника предложения и сверка количества на commit). Агрегат `CTrader` с
//! тремя trade-shadow container-ами `(goods, Gold, YuanBao)`, ready-state,
//! source metadata, terminal clear и отчётами `TraderOfferAdded`/
//! `TraderOfferRemoved` перенесён сюда волной Z-C4: все его поля — типы Zone
//! items (волна Z-C3) и скаляры, методы принимают `CGoods`/`CGoodsFactory` по
//! ссылке (Zone items/content), хранимого доступа к живому `CPlayer`/`CGame`
//! нет — hub-форма не требуется, и агрегат следует карте в Zone вместо
//! прецедента container-owner старого пакета. `CGame` по-прежнему выполняет
//! достигнутую двухфазную проверку и ownership transaction: исходные goods
//! остаются у player до commit, затем переходят в packet второго участника;
//! при частичном отказе они удаляются у получателя и возвращаются в packet
//! владельца, как исходный `RollBack`. Registry поиск plug-ов выполняет
//! `CSessionFactory`, wire доставку — message runtime caller-ы. Здесь — сам
//! агрегат, объявленные kind индексов рамки, скалярная приёмка предложения до
//! занятия ячейки, таблица допустимых player-контейнеров источника, сверка
//! количества и правила обратимости stack-merge при rollback.
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
//! - Reset ready: `record_offer`/`remove_offer`/`clear` агрегата немедленно
//!   сбрасывают собственный `ready`, а `CGame` затем сбрасывает готовность
//!   ОБОИХ участников и рассылает `0xBF716`. Между своим локальным сбросом и
//!   внешним сбросом второго участника есть micro-окно исходного порядка — оно
//!   является контрактом очерёдности оригинала и сознательно не уплотняется.
//! - Rollback в commit: когда доставленный предмет слился со стеком
//!   получателя НЕ полностью (исходный `RollBack` получает неслитый остаток),
//!   обратного удаления части стека в Rust-владельце commit-прохода (`CGame`)
//!   нет — как и у release-формы оригинала вопрос о частичном unmerge остаётся
//!   [`trade_rollback_merge_reversible`] == `false` (возврат `None` caller-а).
//!
//! Швы переноса: числовые index рамки, скалярные предикаты и агрегат
//! перенесены буквально (pub/пути нормализованы); shadow-контейнеры и их
//! отчёты `GoodsShadow`/`PlacedShadowGoods`/`ShadowPresenceReport`/
//! `ShadowRemovedReport` — Zone `items/` владельцы (волна Z-C3), `CGoods` —
//! Zone `items/cgoods.rs`, реестр `CGoodsFactory` — Zone
//! `content/goodsfactory.rs` (волна Z-G0b), `CGuid` — Shared. Двухфазная
//! ownership transaction commit/rollback и reset готовности второго участника
//! остаются у `CGame`. Константы сессии обмена материализованы здесь же,
//! потому что `CSessionFactory` создаёт рамку именно с этими параметрами.

use crate::content::goods::GAP_PARTICULAR_ATTRIBUTE;
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::ccontainer::PreviousContainer;
use crate::items::cgoods::CGoods;
use crate::items::cgoodsshadowcontainer::{
    GoodsShadow, PlacedShadowGoods, ShadowPresenceReport, ShadowRemovedReport,
};
use crate::items::cshadowwallet::CShadowWallet;
use crate::items::cshadowyuanbao::CShadowYuanBao;
use crate::items::cvolumelimitgoodsshadowcontainer::CVolumeLimitGoodsShadowContainer;
use nebokrai_shared::values::CGuid;

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

/// Подтверждённая запись предложения рамки `CTrader::record_offer`: presence
/// shadow и отчёт о вытесненной записи рамки валюты для listener-следствий
/// caller-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraderOfferAdded {
    pub plug_id: i32,
    pub kind: TraderContainerKind,
    pub position: u32,
    pub record: GoodsShadow,
    pub presence: ShadowPresenceReport,
    pub replaced: Option<ShadowRemovedReport>,
}

/// Подтверждённое снятие предложения рамки `CTrader::remove_offer`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraderOfferRemoved {
    pub plug_id: i32,
    pub kind: TraderContainerKind,
    pub position: u32,
    pub removed: ShadowRemovedReport,
}

/// Агрегат рамки двустороннего обмена исходного `CTrader`: три trade-shadow
/// container-а `(goods 32 ячейки, Gold, YuanBao)` с owner `(10, session_id)` и
/// extend-id `plug << 8 | kind`, ready-state и source metadata одного
/// участника. Ready немедленно сбрасывается любой сменой рамки (risk-note
/// этого файла); сброс второго участника и wire — у `CGame`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTrader {
    plug_id: i32,
    session_id: i32,
    owner_id: i32,
    goods: CVolumeLimitGoodsShadowContainer,
    gold: CShadowWallet,
    yuan_bao: CShadowYuanBao,
    ready: bool,
}

impl CTrader {
    pub fn inserted(plug_id: i32, session_id: i32, owner_id: i32) -> Self {
        let mut goods = CVolumeLimitGoodsShadowContainer::new();
        goods.set_container_volume(TRADE_GOODS_CELLS);
        goods
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(SESSION_OWNER_TYPE, session_id);
        goods
            .base_mut()
            .base_mut()
            .set_container_extend_id(trade_container_extend_id(plug_id, TraderContainerKind::Goods));

        let mut gold = CShadowWallet::new();
        gold.set_owner(SESSION_OWNER_TYPE, session_id);
        gold.set_container_extend_id(trade_container_extend_id(plug_id, TraderContainerKind::Gold));
        let mut yuan_bao = CShadowYuanBao::new();
        yuan_bao.set_owner(SESSION_OWNER_TYPE, session_id);
        yuan_bao.set_container_extend_id(trade_container_extend_id(
            plug_id,
            TraderContainerKind::YuanBao,
        ));
        Self {
            plug_id,
            session_id,
            owner_id,
            goods,
            gold,
            yuan_bao,
            ready: false,
        }
    }

    pub const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub const fn session_id(&self) -> i32 {
        self.session_id
    }

    pub const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub const fn ready(&self) -> bool {
        self.ready
    }

    pub const fn set_trade_state(&mut self, ready: bool) {
        self.ready = ready;
    }

    pub fn goods_offers(&self) -> Vec<GoodsShadow> {
        self.goods
            .base()
            .base()
            .shadows()
            .values()
            .copied()
            .collect()
    }

    pub fn gold_amount(&self) -> u32 {
        self.gold.currency_amount()
    }

    pub fn yuan_bao_amount(&self) -> u32 {
        self.yuan_bao.currency_amount()
    }

    pub fn currency_offer(&self, kind: TraderContainerKind) -> Option<GoodsShadow> {
        let container = match kind {
            TraderContainerKind::Gold => self.gold.base(),
            TraderContainerKind::YuanBao => self.yuan_bao.base(),
            TraderContainerKind::Goods => return None,
        };
        container.base().shadows().values().next().copied()
    }

    pub fn record_offer(
        &mut self,
        kind: TraderContainerKind,
        position: u32,
        goods: &CGoods,
        amount: u32,
        previous: PreviousContainer,
        factory: &CGoodsFactory,
    ) -> Result<TraderOfferAdded, TraderOfferBlock> {
        if trade_offer_missing_goods(amount, goods.amount()) {
            return Err(TraderOfferBlock::MissingGoods);
        }
        let base_index = goods.base_properties_index();
        let expected_gold = factory.get_gold_coin_index();
        let expected_yuan_bao = factory.get_yuan_bao_index();
        let placed = PlacedShadowGoods {
            identity: goods.identity().ex_id,
            position: previous.goods_position,
            base_properties_index: base_index,
            amount,
        };
        let (record, presence, replaced) = match kind {
            TraderContainerKind::Goods => {
                if let Some(block) = trade_goods_offer_block(
                    base_index,
                    expected_gold,
                    expected_yuan_bao,
                    goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) as u32,
                    position,
                    self.goods.size(),
                ) {
                    return Err(block);
                }
                if !self.goods.is_space_enough(position) {
                    return Err(TraderOfferBlock::Occupied);
                }
                let added = self
                    .goods
                    .base_mut()
                    .record_placed_goods(previous, placed)
                    .map_err(|_| TraderOfferBlock::ShadowRejected)?;
                if !self
                    .goods
                    .occupy_cell(position, added.recorded.record.goods_id)
                {
                    let _ = self.goods.remove_shadow(added.recorded.record.goods_id);
                    return Err(TraderOfferBlock::Occupied);
                }
                (added.recorded.record, added.presence, None)
            }
            TraderContainerKind::Gold | TraderContainerKind::YuanBao => {
                if let Some(block) = trade_currency_offer_block(
                    kind,
                    base_index,
                    expected_gold,
                    expected_yuan_bao,
                    position,
                ) {
                    return Err(block);
                }
                let container = if kind == TraderContainerKind::Gold {
                    self.gold.base_mut()
                } else {
                    self.yuan_bao.base_mut()
                };
                let replaced_id = container.base().shadows().keys().next().copied();
                let replaced = replaced_id.and_then(|id| container.base_mut().remove_shadow(id));
                container.clear();
                container.set_goods_amount_limit(1);
                let added = container
                    .record_placed_goods(previous, placed)
                    .map_err(|_| TraderOfferBlock::ShadowRejected)?;
                (added.recorded.record, added.presence, replaced)
            }
        };
        self.ready = false;
        Ok(TraderOfferAdded {
            plug_id: self.plug_id,
            kind,
            position,
            record,
            presence,
            replaced,
        })
    }

    pub fn remove_offer(
        &mut self,
        kind: TraderContainerKind,
        position: u32,
        goods_id: CGuid,
    ) -> Option<TraderOfferRemoved> {
        let removed = match kind {
            TraderContainerKind::Goods => {
                if self.goods.query_goods_position(goods_id)? != position {
                    return None;
                }
                self.goods.remove_shadow(goods_id)?
            }
            TraderContainerKind::Gold => {
                if position != 0 {
                    return None;
                }
                self.gold.base_mut().base_mut().remove_shadow(goods_id)?
            }
            TraderContainerKind::YuanBao => {
                if position != 0 {
                    return None;
                }
                self.yuan_bao
                    .base_mut()
                    .base_mut()
                    .remove_shadow(goods_id)?
            }
        };
        self.ready = false;
        Some(TraderOfferRemoved {
            plug_id: self.plug_id,
            kind,
            position,
            removed,
        })
    }

    pub fn clear(&mut self) -> usize {
        self.ready = false;
        self.goods.clear() + self.gold.clear() + self.yuan_bao.clear()
    }
}

// Полный достигнутый CTrader lifecycle исполняется typed owner-ами: агрегат
// здесь, двухфазная transaction commit/rollback — у `CGame`; отдельной
// сохранённой RAW-копии замещённых функций в owner-файле нет.
