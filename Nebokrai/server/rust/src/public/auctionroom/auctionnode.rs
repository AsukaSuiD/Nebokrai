//! Межсерверный узел аукционного товара `CGoodsNode`.
//! Источники контракта — точные пары Misc/Game/World EXE/PDB.
//!
//! Во всех вариантах декодирование сначала очищает владельца, затем читает
//! скалярные поля, две C-строки, маркер GUID, `AuctionInfo`, длину и байты
//! предмета; кодирование пишет обратную последовательность. Курсор включает
//! NUL и маркер GUID. Владеющие векторы и массивы фиксированного размера
//! заменяют буферы и управление временем жизни C++, сохраняя сетевой формат,
//! переходы состояния и последствия частичного декодирования.

use std::error::Error;
use std::fmt;

use super::guid::CGuid;

const LEGACY_STRING_CAPACITY: usize = 0x100;
const AUCTION_INFO_SIZE: usize = 0x22c;
const AUCTION_SELLER_NAME_OFFSET: usize = 0x000;
const AUCTION_MONEY_SELLER_OFFSET: usize = 0x100;
const AUCTION_TIME_SELLER_OFFSET: usize = 0x104;
const AUCTION_SELLER_ID_OFFSET: usize = 0x108;
const AUCTION_SELLER_IP_OFFSET: usize = 0x10c;
const AUCTION_BUYER_NAME_OFFSET: usize = 0x11c;
const AUCTION_MONEY_BUYER_OFFSET: usize = 0x21c;
const AUCTION_TIME_BUYER_OFFSET: usize = 0x220;
const AUCTION_BUYER_ID_OFFSET: usize = 0x224;

/// Ошибка безопасной границы чтения старого неограниченного byte-array.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsNodeUnserializeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    LegacyStringOverflow {
        field: &'static str,
        first_out_of_bounds_offset: usize,
    },
    GoodsAllocationFailed {
        length: usize,
    },
}

/// Неинициализированная либо непредставимая граница старого `Serialize`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsNodeSerializeError {
    MissingGoodsType,
    MissingLevelLimit,
    LegacyStringWithoutTerminator { field: &'static str },
    GoodsLengthOutsideLegacyRange { length: usize },
}

impl fmt::Display for GoodsNodeUnserializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
            Self::LegacyStringOverflow {
                field,
                first_out_of_bounds_offset,
            } => write!(
                formatter,
                "поле {field} вышло за старый 256-байтовый буфер на offset {first_out_of_bounds_offset}"
            ),
            Self::GoodsAllocationFailed { length } => write!(
                formatter,
                "не удалось выделить старый goods-буфер длиной {length} байт"
            ),
        }
    }
}

impl Error for GoodsNodeUnserializeError {}

impl fmt::Display for GoodsNodeSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGoodsType => {
                formatter.write_str("поле m_btGoodsType не было инициализировано")
            }
            Self::MissingLevelLimit => {
                formatter.write_str("поле m_dwLvLimit не было инициализировано")
            }
            Self::LegacyStringWithoutTerminator { field } => {
                write!(
                    formatter,
                    "поле {field} не содержит NUL в старом char[0x100]"
                )
            }
            Self::GoodsLengthOutsideLegacyRange { length } => write!(
                formatter,
                "goods-буфер длиной {length} не представим старым unsigned long"
            ),
        }
    }
}

impl Error for GoodsNodeSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsState(i32);

impl GoodsState {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const AUCTION: Self = Self(1);
    pub(crate) const SUCESSED: Self = Self(2);
    pub(crate) const BACK: Self = Self(3);
    pub(crate) const UNDO: Self = Self(4);
    pub(crate) const PRE_BUY: Self = Self(5);

    pub(crate) const fn raw(self) -> i32 {
        self.0
    }

    /// Сохраняет любой 32-битный database discriminant без преждевременного
    /// сужения до известных состояний: оригинал `LoadGoodsByOwnerId` просто
    /// копировал `GoodsState` из строки в node.
    pub(crate) const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuctionInfo {
    bytes: [u8; AUCTION_INFO_SIZE],
}

impl AuctionInfo {
    const fn new() -> Self {
        Self {
            bytes: [0; AUCTION_INFO_SIZE],
        }
    }

    fn clear(&mut self) {
        self.bytes.fill(0);
    }

    fn as_bytes_mut(&mut self) -> &mut [u8; AUCTION_INFO_SIZE] {
        &mut self.bytes
    }

    const fn as_bytes(&self) -> &[u8; AUCTION_INFO_SIZE] {
        &self.bytes
    }

    fn buyer_id(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_BUYER_ID_OFFSET..AUCTION_BUYER_ID_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwBuyerId помещается в AuctionInfo"),
        )
    }

    fn set_buyer_id(&mut self, buyer_id: u32) {
        self.bytes[AUCTION_BUYER_ID_OFFSET..AUCTION_BUYER_ID_OFFSET + 4]
            .copy_from_slice(&buyer_id.to_le_bytes());
    }

    fn buyer_name(&self) -> &[u8] {
        &self.bytes[AUCTION_BUYER_NAME_OFFSET..AUCTION_MONEY_BUYER_OFFSET]
    }

    fn seller_name(&self) -> &[u8] {
        &self.bytes[AUCTION_SELLER_NAME_OFFSET..AUCTION_MONEY_SELLER_OFFSET]
    }

    fn seller_ip(&self) -> &[u8] {
        &self.bytes[AUCTION_SELLER_IP_OFFSET..AUCTION_BUYER_NAME_OFFSET]
    }

    fn money_seller(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_MONEY_SELLER_OFFSET..AUCTION_MONEY_SELLER_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwMoneySeller помещается в AuctionInfo"),
        )
    }

    fn seller_id(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_SELLER_ID_OFFSET..AUCTION_SELLER_ID_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwSellerId помещается в AuctionInfo"),
        )
    }

    fn time_seller(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_TIME_SELLER_OFFSET..AUCTION_TIME_SELLER_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwTimeSeller помещается в AuctionInfo"),
        )
    }

    fn money_buyer(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_MONEY_BUYER_OFFSET..AUCTION_MONEY_BUYER_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwMoneyBuyer помещается в AuctionInfo"),
        )
    }

    fn time_buyer(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[AUCTION_TIME_BUYER_OFFSET..AUCTION_TIME_BUYER_OFFSET + 4]
                .try_into()
                .expect("PDB offset dwTimeBuyer помещается в AuctionInfo"),
        )
    }

    fn set_database_fields(
        &mut self,
        seller_name: &[u8],
        money_seller: u32,
        time_seller: u32,
        seller_id: u32,
        buyer_name: &[u8],
        money_buyer: u32,
        time_buyer: u32,
        buyer_id: u32,
    ) {
        copy_legacy_database_string::<LEGACY_STRING_CAPACITY>(
            (&mut self.bytes[AUCTION_SELLER_NAME_OFFSET..AUCTION_MONEY_SELLER_OFFSET])
                .try_into()
                .expect("PDB seller-name диапазон имеет длину 0x100"),
            seller_name,
        );
        self.bytes[AUCTION_MONEY_SELLER_OFFSET..AUCTION_MONEY_SELLER_OFFSET + 4]
            .copy_from_slice(&money_seller.to_le_bytes());
        self.bytes[AUCTION_TIME_SELLER_OFFSET..AUCTION_TIME_SELLER_OFFSET + 4]
            .copy_from_slice(&time_seller.to_le_bytes());
        self.bytes[AUCTION_SELLER_ID_OFFSET..AUCTION_SELLER_ID_OFFSET + 4]
            .copy_from_slice(&seller_id.to_le_bytes());
        copy_legacy_database_string::<LEGACY_STRING_CAPACITY>(
            (&mut self.bytes[AUCTION_BUYER_NAME_OFFSET..AUCTION_MONEY_BUYER_OFFSET])
                .try_into()
                .expect("PDB buyer-name диапазон имеет длину 0x100"),
            buyer_name,
        );
        self.bytes[AUCTION_MONEY_BUYER_OFFSET..AUCTION_MONEY_BUYER_OFFSET + 4]
            .copy_from_slice(&money_buyer.to_le_bytes());
        self.bytes[AUCTION_TIME_BUYER_OFFSET..AUCTION_TIME_BUYER_OFFSET + 4]
            .copy_from_slice(&time_buyer.to_le_bytes());
        self.set_buyer_id(buyer_id);
    }
}

/// Owned-состояние исходного `CGoodsNode`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGoodsNode {
    db: bool,
    add_ticket: u32,
    account: [u8; LEGACY_STRING_CAPACITY],
    owner_id: u32,
    auction_time: u32,
    money_type: u8,
    goods_type: Option<u8>,
    npc_price: i32,
    amount: i32,
    goods_state: GoodsState,
    offer_price: bool,
    guid: CGuid,
    level_limit: Option<u32>,
    goods_name: [u8; LEGACY_STRING_CAPACITY],
    base_index: u32,
    auction_info: AuctionInfo,
    goods_bytes: Vec<u8>,
}

/// Значения одной основной строки `Auction`, которые `CDbMisc` переносил в
/// `CGoodsNode` до сериализации вложенного `CGoods`.
///
/// Поле IP продавца в `AuctionInfo` в этом SQL не читается. Оно остаётся
/// нулевым после `DbNote::DbNote -> CGoodsNode::Clear`, как в точном owner-е.
/// Строки уже должны быть в Windows-1251; helper ниже безопасно выражает
/// старый `_snprintf(char[0x100], "%s")` и не переносит его UB при усечении.
pub(crate) struct AuctionDatabaseNodeFields {
    pub(crate) add_ticket: u32,
    pub(crate) account: Vec<u8>,
    pub(crate) owner_id: u32,
    pub(crate) auction_time: u32,
    pub(crate) money_type: u8,
    pub(crate) goods_type: u8,
    pub(crate) npc_price: i32,
    pub(crate) amount: i32,
    pub(crate) goods_state: GoodsState,
    pub(crate) offer_price: bool,
    pub(crate) guid: CGuid,
    pub(crate) base_index: u32,
    pub(crate) level_limit: u32,
    pub(crate) goods_name: Vec<u8>,
    pub(crate) seller_name: Vec<u8>,
    pub(crate) money_seller: u32,
    pub(crate) time_seller: u32,
    pub(crate) seller_id: u32,
    pub(crate) buyer_name: Vec<u8>,
    pub(crate) money_buyer: u32,
    pub(crate) time_buyer: u32,
    pub(crate) buyer_id: u32,
}

/// Поля одного точного `MakeCurAucNode` после успешного удаления товара из
/// двухъячеечного аукционного контейнера игрока.
pub(crate) struct AuctionListingNodeFields<'value> {
    pub(crate) account: &'value [u8],
    pub(crate) owner_id: u32,
    pub(crate) auction_time: u32,
    pub(crate) goods_type: u8,
    pub(crate) npc_price: i32,
    pub(crate) amount: i32,
    pub(crate) guid: CGuid,
    pub(crate) level_limit: u32,
    pub(crate) goods_name: &'value [u8],
    pub(crate) base_index: u32,
    pub(crate) seller_money: u32,
    pub(crate) seller_time: u32,
    pub(crate) seller_ip: &'value [u8],
    pub(crate) end_time: u32,
    pub(crate) goods_bytes: Vec<u8>,
}

/// Поля одного успешного шага `CPlayer::AutoAddAuctionGoods`.
pub(crate) struct AuctionAutomaticNodeFields<'value> {
    pub(crate) account: &'value [u8],
    pub(crate) owner_id: u32,
    pub(crate) money_type: u8,
    pub(crate) goods_type: u8,
    pub(crate) npc_price: i32,
    pub(crate) amount: i32,
    pub(crate) guid: CGuid,
    pub(crate) level_limit: u32,
    pub(crate) goods_name: &'value [u8],
    pub(crate) base_index: u32,
    pub(crate) seller_time: u32,
    pub(crate) end_time: u32,
    pub(crate) goods_bytes: Vec<u8>,
}

/// Подтверждённые PDB-поля узла, которые нужны четырём SQL write-переходам
/// `CDbMisc`.
///
/// Строка buyer name остаётся срезом фиксированного legacy-буфера: TDS-владелец обязан
/// остановиться на первом NUL, как старый `%s`, и отдельно обработать отсутствие
/// terminator вместо чтения за границей `char[0x100]`.
pub(crate) struct AuctionDatabaseWriteFields<'node> {
    pub(crate) guid: CGuid,
    pub(crate) buyer_id: u32,
    pub(crate) buyer_name: &'node [u8],
    pub(crate) money_type: u8,
    pub(crate) seller_money: u32,
    pub(crate) seller_id: u32,
    pub(crate) goods_state: GoodsState,
}

/// PDB-подтверждённые поля одного вызова `CDbMisc::InsertItemToDb`.
///
/// В отличие от materialized DB-view, текст остаётся заимствованным
/// fixed-size legacy-буфером. SQL-владелец останавливается на первом NUL, как
/// исходный `%s`, но не переносит его чтение за границу массива.
pub(crate) struct AuctionDatabaseInsertFields<'node> {
    pub(crate) add_ticket: u32,
    pub(crate) account: &'node [u8],
    pub(crate) owner_id: u32,
    pub(crate) auction_time: u32,
    pub(crate) money_type: u8,
    pub(crate) goods_type: Option<u8>,
    pub(crate) npc_price: i32,
    pub(crate) amount: i32,
    pub(crate) goods_state: GoodsState,
    pub(crate) offer_price: bool,
    pub(crate) goods_name: &'node [u8],
    pub(crate) level_limit: Option<u32>,
    pub(crate) base_index: u32,
    pub(crate) seller_name: &'node [u8],
    pub(crate) money_seller: u32,
    pub(crate) time_seller: u32,
    pub(crate) seller_id: u32,
    pub(crate) buyer_name: &'node [u8],
    pub(crate) money_buyer: u32,
    pub(crate) time_buyer: u32,
    pub(crate) buyer_id: u32,
}

impl Default for CGoodsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl CGoodsNode {
    /// Создаёт узел в состоянии исходного constructor + `Clear`.
    pub(crate) fn new() -> Self {
        let mut node = Self {
            db: false,
            add_ticket: 0,
            account: [0; LEGACY_STRING_CAPACITY],
            owner_id: 0,
            auction_time: 0,
            money_type: 0,
            goods_type: None,
            npc_price: 0,
            amount: 0,
            goods_state: GoodsState::NONE,
            offer_price: false,
            guid: CGuid::GUID_INVALID,
            level_limit: None,
            goods_name: [0; LEGACY_STRING_CAPACITY],
            base_index: 0,
            auction_info: AuctionInfo::new(),
            goods_bytes: Vec::new(),
        };
        node.clear();
        node
    }

    /// Создаёт результат одной materialized DB-строки `Auction`.
    ///
    /// Это узкая граница `CDbMisc::LoadGoodsByOwnerId`: оригинал owner заполнял
    /// note после `CreateGoodsNoProbability`, а его goods-byte-array назначал
    /// позднее, когда все joined `AuctionGoods` строки были применены.
    pub(crate) fn from_auction_database(
        fields: AuctionDatabaseNodeFields,
        goods_bytes: Vec<u8>,
    ) -> Self {
        let mut node = Self::new();
        node.add_ticket = fields.add_ticket;
        copy_legacy_database_string(&mut node.account, &fields.account);
        node.owner_id = fields.owner_id;
        node.auction_time = fields.auction_time;
        node.money_type = fields.money_type;
        node.goods_type = Some(fields.goods_type);
        node.npc_price = fields.npc_price;
        node.amount = fields.amount;
        node.goods_state = fields.goods_state;
        node.offer_price = fields.offer_price;
        node.guid = fields.guid;
        node.base_index = fields.base_index;
        node.level_limit = Some(fields.level_limit);
        copy_legacy_database_string(&mut node.goods_name, &fields.goods_name);
        node.auction_info.set_database_fields(
            &fields.seller_name,
            fields.money_seller,
            fields.time_seller,
            fields.seller_id,
            &fields.buyer_name,
            fields.money_buyer,
            fields.time_buyer,
            fields.buyer_id,
        );
        node.goods_bytes = goods_bytes;
        node
    }

    /// Создаёт `LoadMoneyById`-note после сериализации возвращаемого gold.
    ///
    /// Оригинал owner заполнял только amount, GUID, base-index, `STATE_BACK` и
    /// goods-byte-array. `m_btGoodsType` и `m_dwLvLimit` оставались прежней
    /// неинициализированной внутренней областью, но этот path никогда не
    /// вызывает `CGoodsNode::Serialize`: `DoneOutList` извлекает только
    /// вложенный `CGoods`. Rust оставляет их `None`, поэтому ошибочный новый
    /// serialize безопасно выявляется вместо чтения мусора.
    pub(crate) fn from_auction_money_return(
        amount: i32,
        guid: CGuid,
        base_index: u32,
        goods_bytes: Vec<u8>,
    ) -> Self {
        let mut node = Self::new();
        node.amount = amount;
        node.guid = guid;
        node.base_index = base_index;
        node.goods_state = GoodsState::BACK;
        node.goods_bytes = goods_bytes;
        node
    }

    /// Создаёт ожидающий узел продажи в точном порядке итоговых полей
    /// `MakeCurAucNode`; клиентский путь всегда выставляет цену в YuanBao.
    pub(crate) fn from_player_listing(fields: AuctionListingNodeFields<'_>) -> Self {
        let mut node = Self::new();
        node.add_ticket = fields.end_time;
        copy_legacy_database_string(&mut node.account, fields.account);
        node.owner_id = fields.owner_id;
        node.auction_time = fields.auction_time;
        node.money_type = 1;
        node.goods_type = Some(fields.goods_type);
        node.npc_price = fields.npc_price;
        node.amount = fields.amount;
        node.goods_state = GoodsState::AUCTION;
        node.guid = fields.guid;
        node.level_limit = Some(fields.level_limit);
        copy_legacy_database_string(&mut node.goods_name, fields.goods_name);
        node.base_index = fields.base_index;
        node.auction_info.set_database_fields(
            fields.account,
            fields.seller_money,
            fields.seller_time,
            fields.owner_id,
            &[],
            0,
            0,
            0,
        );
        copy_legacy_database_string::<16>(
            (&mut node.auction_info.bytes[AUCTION_SELLER_IP_OFFSET..AUCTION_BUYER_NAME_OFFSET])
                .try_into()
                .expect("PDB seller-ip диапазон имеет длину 0x10"),
            fields.seller_ip,
        );
        node.goods_bytes = fields.goods_bytes;
        node
    }

    /// Создаёт узел, которым `AutoAddAuctionGoods` заменяет
    /// `m_CurrentAucNode`; этот сценарный путь не задаёт IP продавца.
    pub(crate) fn from_automatic_goods(fields: AuctionAutomaticNodeFields<'_>) -> Self {
        let mut node = Self::new();
        node.add_ticket = fields.end_time;
        copy_legacy_database_string(&mut node.account, fields.account);
        node.owner_id = fields.owner_id;
        node.auction_time = 0x3840;
        node.money_type = fields.money_type;
        node.goods_type = Some(fields.goods_type);
        node.npc_price = fields.npc_price;
        node.amount = fields.amount;
        node.goods_state = GoodsState::AUCTION;
        node.guid = fields.guid;
        node.level_limit = Some(fields.level_limit);
        copy_legacy_database_string(&mut node.goods_name, fields.goods_name);
        node.base_index = fields.base_index;
        node.auction_info.set_database_fields(
            fields.account,
            1,
            fields.seller_time,
            fields.owner_id,
            &[],
            0,
            0,
            0,
        );
        node.goods_bytes = fields.goods_bytes;
        node
    }

    /// Очищает ровно поля исходного `Clear` и освобождает goods-буфер.
    ///
    /// `goods_type` и `level_limit` сохраняются: старый метод не присваивал им
    /// значений ни при повторной очистке, ни в constructor-е.
    pub(crate) fn clear(&mut self) {
        self.db = true;
        self.add_ticket = 0;
        self.account.fill(0);
        self.owner_id = 0;
        self.auction_time = 0;
        self.money_type = 0;
        self.npc_price = 0;
        self.amount = 1;
        self.goods_state = GoodsState::NONE;
        self.offer_price = false;
        self.guid = CGuid::GUID_INVALID;
        self.base_index = 0;
        self.goods_name.fill(0);
        self.auction_info.clear();
        self.goods_bytes = Vec::new();
    }

    /// Читает один узел из старого byte-array, начиная с переданного offset.
    ///
    /// При безопасной ошибке уже выполненные очистка, сдвиги курсора и
    /// успешные присваивания сохраняются; реакция старого UB не имитируется.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), GoodsNodeUnserializeError> {
        self.clear();
        let mut reader = LegacyByteArrayReader::new(source, cursor);

        self.db = reader.read_u8("m_bDb")? != 0;
        self.add_ticket = reader.read_u32("m_dwAddTicket")?;
        reader.read_c_string("m_strAccount", &mut self.account)?;
        self.owner_id = reader.read_u32("m_dwOwerId")?;
        self.auction_time = reader.read_u32("m_dwAuctionTime")?;
        self.money_type = reader.read_u8("m_btMoneyType")?;
        self.goods_type = Some(reader.read_u8("m_btGoodsType")?);
        self.npc_price = reader.read_i32("m_i32NpcPrice")?;
        self.amount = reader.read_i32("m_i32Amout")?;
        self.goods_state = GoodsState(reader.read_i32("m_GoodsState")?);
        self.offer_price = reader.read_u8("m_bOfferPrice")? != 0;
        self.guid = reader.read_guid("m_guid")?;
        self.base_index = reader.read_u32("m_dwBaseIndex")?;
        self.level_limit = Some(reader.read_u32("m_dwLvLimit")?);
        reader.read_c_string("m_strGoodsName", &mut self.goods_name)?;
        reader.read_exact("m_AucInfo", self.auction_info.as_bytes_mut())?;

        let goods_length = reader.read_u32("m_vecGoodsByte.size")? as usize;
        reader.ensure_available("m_vecGoodsByte", goods_length)?;
        let mut goods_bytes = Vec::new();
        goods_bytes.try_reserve_exact(goods_length).map_err(|_| {
            GoodsNodeUnserializeError::GoodsAllocationFailed {
                length: goods_length,
            }
        })?;
        let goods_source = reader.take("m_vecGoodsByte", goods_length)?;
        goods_bytes.extend_from_slice(goods_source);
        self.goods_bytes = goods_bytes;
        Ok(())
    }

    /// Создаёт byte-оригинал результат старого `Serialize`.
    ///
    /// Ошибка оставляет локально заблокированными только поля, которые старый
    /// constructor не задавал, либо недостижимую для 32-bit vector длину.
    pub(crate) fn serialize(&self) -> Result<Vec<u8>, GoodsNodeSerializeError> {
        let goods_type = self
            .goods_type
            .ok_or(GoodsNodeSerializeError::MissingGoodsType)?;
        let level_limit = self
            .level_limit
            .ok_or(GoodsNodeSerializeError::MissingLevelLimit)?;
        let account = legacy_c_string(&self.account, "m_strAccount")?;
        let goods_name = legacy_c_string(&self.goods_name, "m_strGoodsName")?;
        let goods_length = u32::try_from(self.goods_bytes.len()).map_err(|_| {
            GoodsNodeSerializeError::GoodsLengthOutsideLegacyRange {
                length: self.goods_bytes.len(),
            }
        })?;

        let mut output = Vec::new();
        output.push(u8::from(self.db));
        output.extend_from_slice(&self.add_ticket.to_le_bytes());
        output.extend_from_slice(account);
        output.extend_from_slice(&self.owner_id.to_le_bytes());
        output.extend_from_slice(&self.auction_time.to_le_bytes());
        output.push(self.money_type);
        output.push(goods_type);
        output.extend_from_slice(&self.npc_price.to_le_bytes());
        output.extend_from_slice(&self.amount.to_le_bytes());
        output.extend_from_slice(&self.goods_state.0.to_le_bytes());
        output.push(u8::from(self.offer_price));
        if self.guid.is_invalid() {
            output.push(0);
        } else {
            output.push(0x10);
            output.extend_from_slice(self.guid.as_legacy_bytes());
        }
        output.extend_from_slice(&self.base_index.to_le_bytes());
        output.extend_from_slice(&level_limit.to_le_bytes());
        output.extend_from_slice(goods_name);
        output.extend_from_slice(self.auction_info.as_bytes());
        output.extend_from_slice(&goods_length.to_le_bytes());
        output.extend_from_slice(&self.goods_bytes);
        Ok(output)
    }

    /// Возвращает GUID, по которому комната индексирует этот узел.
    pub(crate) const fn guid(&self) -> CGuid {
        self.guid
    }

    /// Возвращает ticket вторичного временного индекса.
    pub(crate) const fn add_ticket(&self) -> u32 {
        self.add_ticket
    }

    pub(crate) const fn money_type(&self) -> u8 {
        self.money_type
    }

    pub(crate) const fn auction_time(&self) -> u32 {
        self.auction_time
    }

    pub(crate) const fn npc_price(&self) -> i32 {
        self.npc_price
    }

    pub(crate) fn seller_money(&self) -> u32 {
        self.auction_info.money_seller()
    }

    pub(crate) fn seller_id(&self) -> u32 {
        self.auction_info.seller_id()
    }

    pub(crate) fn seller_name(&self) -> &[u8] {
        self.auction_info.seller_name()
    }

    pub(crate) fn seller_ip(&self) -> &[u8] {
        self.auction_info.seller_ip()
    }

    /// Возвращает исходный unsigned owner id без изменения битов.
    pub(crate) const fn owner_id(&self) -> u32 {
        self.owner_id
    }

    /// Возвращает оригинал `m_bDb` для World auction relay.
    pub(crate) const fn is_db(&self) -> bool {
        self.db
    }

    /// Возвращает PDB-подтверждённый `m_AucInfo.dwBuyerId`.
    pub(crate) fn buyer_id(&self) -> u32 {
        self.auction_info.buyer_id()
    }

    /// Выполняет единственное доказанное присваивание `dwBuyerId`.
    pub(crate) fn set_buyer_id(&mut self, buyer_id: u32) {
        self.auction_info.set_buyer_id(buyer_id);
    }

    /// Собирает только поля, которые исходный `CDbMisc` передавал в свои
    /// отдельные SQL write-переходы.
    pub(crate) fn database_write_fields(&self) -> AuctionDatabaseWriteFields<'_> {
        AuctionDatabaseWriteFields {
            guid: self.guid,
            buyer_id: self.auction_info.buyer_id(),
            buyer_name: self.auction_info.buyer_name(),
            money_type: self.money_type,
            seller_money: self.auction_info.money_seller(),
            seller_id: self.auction_info.seller_id(),
            goods_state: self.goods_state,
        }
    }

    /// Собирает аргументы оригинал `exec addnewGoods` без переинтерпретации
    /// неиспользуемых байтов `AuctionInfo`.
    pub(crate) fn database_insert_fields(&self) -> AuctionDatabaseInsertFields<'_> {
        AuctionDatabaseInsertFields {
            add_ticket: self.add_ticket,
            account: &self.account,
            owner_id: self.owner_id,
            auction_time: self.auction_time,
            money_type: self.money_type,
            goods_type: self.goods_type,
            npc_price: self.npc_price,
            amount: self.amount,
            goods_state: self.goods_state,
            offer_price: self.offer_price,
            goods_name: &self.goods_name,
            level_limit: self.level_limit,
            base_index: self.base_index,
            seller_name: self.auction_info.seller_name(),
            money_seller: self.auction_info.money_seller(),
            time_seller: self.auction_info.time_seller(),
            seller_id: self.auction_info.seller_id(),
            buyer_name: self.auction_info.buyer_name(),
            money_buyer: self.auction_info.money_buyer(),
            time_buyer: self.auction_info.time_buyer(),
            buyer_id: self.auction_info.buyer_id(),
        }
    }

    /// Возвращает тип товара либо неинициализированную старую границу.
    pub(super) const fn goods_type(&self) -> Option<u8> {
        self.goods_type
    }

    /// Возвращает level limit либо неинициализированную старую границу.
    pub(super) const fn level_limit(&self) -> Option<u32> {
        self.level_limit
    }

    /// Возвращает фиксированный старый буфер имени товара.
    pub(crate) const fn goods_name(&self) -> &[u8; LEGACY_STRING_CAPACITY] {
        &self.goods_name
    }

    /// Выполняет доказанное присваивание `m_GoodsState = STATE_AUCTION`.
    pub(crate) fn mark_as_auction(&mut self) {
        self.goods_state = GoodsState::AUCTION;
    }

    /// Exact `SendBuyAucNode(false)`/offline-buyer rollback перед serialize.
    pub(crate) fn prepare_return_to_auction(&mut self) {
        self.goods_state = GoodsState::AUCTION;
        self.db = false;
        self.auction_info.set_buyer_id(0);
    }

    /// Проверяет точное состояние `STATE_AUCTION`.
    pub(super) fn is_auction(&self) -> bool {
        self.goods_state == GoodsState::AUCTION
    }

    /// Возвращает действующее `DoneDelList` состояние без переинтерпретации.
    pub(crate) const fn goods_state(&self) -> GoodsState {
        self.goods_state
    }

    /// Возвращает действующий unsigned индекс исходного auction-node.
    pub(crate) const fn base_index(&self) -> u32 {
        self.base_index
    }

    /// Возвращает исходное signed количество с сохранением опечатки `Amout`.
    pub(crate) const fn amount(&self) -> i32 {
        self.amount
    }

    /// Заимствует вложенный byte-array полного `CGoods`.
    pub(crate) fn goods_bytes(&self) -> &[u8] {
        &self.goods_bytes
    }

    /// Возвращает действующий `DoneDelList` флаг наличия ставки.
    pub(super) const fn offer_price(&self) -> bool {
        self.offer_price
    }

    /// Переводит узел в `STATE_UNDO`.
    pub(super) fn mark_as_undo(&mut self) {
        self.goods_state = GoodsState::UNDO;
    }

    /// Переводит узел в `STATE_PRE_BUY`.
    pub(super) fn mark_as_pre_buy(&mut self) {
        self.goods_state = GoodsState::PRE_BUY;
    }

    /// Переводит узел в `STATE_SUCESSED` с исходной опечаткой имени.
    pub(super) fn mark_as_sucessed(&mut self) {
        self.goods_state = GoodsState::SUCESSED;
    }

    /// Переводит узел в `STATE_BACK`.
    pub(super) fn mark_as_back(&mut self) {
        self.goods_state = GoodsState::BACK;
    }

    /// Возвращает reached-поле `AuctionInfo::dwBuyerId`.
    pub(super) fn auction_buyer_id(&self) -> u32 {
        self.auction_info.buyer_id()
    }

    /// Перезаписывает только `AuctionInfo::dwBuyerId`, сохраняя остальные bytes.
    pub(super) fn set_auction_buyer_id(&mut self, buyer_id: u32) {
        self.auction_info.set_buyer_id(buyer_id);
    }
}

fn legacy_c_string<'value>(
    value: &'value [u8; LEGACY_STRING_CAPACITY],
    field: &'static str,
) -> Result<&'value [u8], GoodsNodeSerializeError> {
    let terminator = value
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(GoodsNodeSerializeError::LegacyStringWithoutTerminator { field })?;
    Ok(&value[..=terminator])
}

/// Безопасная замена старого копирования C-строки в фиксированный buffer.
///
/// Нормальная строка сохраняется byte-for-byte и с первым NUL. Старый MSVC
/// мог оставить усечённый buffer без NUL, а последующий `Serialize` читать за
/// его границей; это внутренний дефект без доказанного контракта, поэтому Rust
/// всегда ставит terminator в последней ячейке.
fn copy_legacy_database_string<const CAPACITY: usize>(
    destination: &mut [u8; CAPACITY],
    source: &[u8],
) {
    destination.fill(0);
    let visible = source
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(source.len());
    let copied = visible.min(CAPACITY.saturating_sub(1));
    destination[..copied].copy_from_slice(&source[..copied]);
}

struct LegacyByteArrayReader<'source, 'cursor> {
    source: &'source [u8],
    cursor: &'cursor mut usize,
}

impl<'source, 'cursor> LegacyByteArrayReader<'source, 'cursor> {
    fn new(source: &'source [u8], cursor: &'cursor mut usize) -> Self {
        Self { source, cursor }
    }

    fn read_u8(&mut self, field: &'static str) -> Result<u8, GoodsNodeUnserializeError> {
        Ok(self.take(field, 1)?[0])
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, GoodsNodeUnserializeError> {
        let bytes: [u8; 4] = self
            .take(field, 4)?
            .try_into()
            .expect("take вернул ровно четыре байта");
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i32(&mut self, field: &'static str) -> Result<i32, GoodsNodeUnserializeError> {
        let bytes: [u8; 4] = self
            .take(field, 4)?
            .try_into()
            .expect("take вернул ровно четыре байта");
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_guid(&mut self, field: &'static str) -> Result<CGuid, GoodsNodeUnserializeError> {
        if self.read_u8(field)? == 0 {
            return Ok(CGuid::GUID_INVALID);
        }

        let bytes: [u8; 16] = self
            .take(field, 16)?
            .try_into()
            .expect("take вернул ровно шестнадцать байт");
        Ok(CGuid::from_legacy_bytes(bytes))
    }

    fn read_c_string(
        &mut self,
        field: &'static str,
        destination: &mut [u8; LEGACY_STRING_CAPACITY],
    ) -> Result<(), GoodsNodeUnserializeError> {
        let mut destination_offset = 0;
        loop {
            let source_offset = *self.cursor;
            let byte = self.read_u8(field)?;
            let Some(slot) = destination.get_mut(destination_offset) else {
                // typed boundary: helpers Misc, Game
                // и World после 256 байт продолжали
                // писать за char[0x100]. Достижимость и реакция процесса на
                // такую строку не доказаны; unsafe не вводится.
                return Err(GoodsNodeUnserializeError::LegacyStringOverflow {
                    field,
                    first_out_of_bounds_offset: source_offset,
                });
            };
            *slot = byte;
            destination_offset += 1;
            if byte == 0 {
                return Ok(());
            }
        }
    }

    fn read_exact<const SIZE: usize>(
        &mut self,
        field: &'static str,
        destination: &mut [u8; SIZE],
    ) -> Result<(), GoodsNodeUnserializeError> {
        destination.copy_from_slice(self.take(field, SIZE)?);
        Ok(())
    }

    fn take(
        &mut self,
        field: &'static str,
        size: usize,
    ) -> Result<&'source [u8], GoodsNodeUnserializeError> {
        self.ensure_available(field, size)?;
        let offset = *self.cursor;
        let end = offset + size;
        let bytes = &self.source[offset..end];
        *self.cursor = end;
        Ok(bytes)
    }

    fn ensure_available(
        &self,
        field: &'static str,
        size: usize,
    ) -> Result<(), GoodsNodeUnserializeError> {
        let offset = *self.cursor;
        let available = self.source.len().saturating_sub(offset);
        let Some(end) = offset.checked_add(size) else {
            return Err(GoodsNodeUnserializeError::UnexpectedEnd {
                field,
                offset,
                needed: size,
                available,
            });
        };
        if end > self.source.len() {
            // typed boundary: прямые scalar-read в `UnSerialize` сначала
            // сдвигали `long&`, а helpers строк/буферов тоже не знали длину
            // источника. Безопасная граница не назначает чтению за концом
            // наблюдаемого результата и не двигает курсор через отсутствующее.
            return Err(GoodsNodeUnserializeError::UnexpectedEnd {
                field,
                offset,
                needed: size,
                available,
            });
        }
        Ok(())
    }
}
