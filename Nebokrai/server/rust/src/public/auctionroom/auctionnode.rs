//! Узел товара аукциона `CGoodsNode`: начальное состояние, очистка и
//! двусторонний межсерверный byte-array.
//!
//! Статус владельца: `IMPLEMENTED` для `CGoodsNode`, `~CGoodsNode`, `Clear`,
//! `UnSerialize`, `Serialize` и достигнутых переводов состояния из
//! `CAuctionRoom::AddItemToAuctionRoom`, `DelItemFromAuctionRoom`,
//! `DelItemFromAuctionRoomByPreBuy`, `DelItemFromAuctionRoomForSucessed` и
//! `DoneDelList`; `Clone` и остальная доменная семантика ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально).
//!
//! Исходные `.cpp/.h`:
//! `h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp` и
//! `e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.cpp/.h`.
//! Точные пары и существенные RVA:
//! - MiscServer: `miscserver.exe + miscserver.pdb`, constructor `0x0000BCA0`,
//!   destructor `0x0000B9E0`, `Clear` `0x0000BA90`, `UnSerialize` `0x0000BB40`,
//!   `Serialize` `0x0000B8B0`;
//! - GameServer: `gameserver.exe + GameServer.pdb`, constructor `0x000D93B0`,
//!   destructor `0x000D9110`, `Clear` `0x000D91C0`, `UnSerialize` `0x000D9270`,
//!   `Serialize` `0x000D8FE0`;
//! - WorldServer: `Nworldserver.exe + WorldServer.pdb`, constructor
//!   `0x000DFBE0`, destructor `0x000DF880`, `Clear` `0x000DF9F0`,
//!   `UnSerialize` `0x000DFAA0`, `Serialize` `0x000DF8C0`.
//!
//! Все три варианта совпадают. Точные PDB задают размер старого класса `0x478`,
//! 256-байтовые account/name, 16-байтовый `CGUID`, `AuctionInfo` размером
//! `0x22c` и 32-битный `GoodsState`: `STATE_NONE..STATE_PRE_BUY = 0..5`.
//! `UnSerialize` сначала вызывает `Clear`, затем последовательно читает bool,
//! little-endian 32-битные поля, две NUL-terminated строки, GUID с байтом
//! присутствия, byte-exact `AuctionInfo`, длину и байты товара. Курсор включает
//! завершающий NUL и GUID-marker.
//! `Serialize` во всех трёх компонентах совпадает и пишет ровно обратную
//! последовательность. Старый `vector<unsigned char>` заменён новым owned
//! `Vec<u8>`; строки включают первый NUL, GUID сохраняет marker `0/0x10`, а
//! длина goods-вектора остаётся 32-битной.
//!
//! Фиксированные C-массивы представлены `[u8; 0x100]`, непрозрачный пока
//! `AuctionInfo` сохраняется byte-exact wrapper-ом над `[u8; 0x22c]`,
//! `std::vector<unsigned char>` — `Vec<u8>`, а GUID — готовым `CGuid`.
//! Достигнутое поле `dwBuyerId` PDB задаёт как unsigned 32-bit по offset
//! `0x224`. Снимки полей для DB-записи ниже читают только подтверждённые
//! аргументы exact `CDbMisc`; остальные байты структуры не переинтерпретируются
//! заранее. У неинициализированных constructor-ом `m_btGoodsType/m_dwLvLimit`
//! нет придуманного default и TDS-owner останавливает такой вызов до SQL.
//! Rust-владение заменяет destructor,
//! allocator и exception cleanup. Форма `long&` сознательно заменена
//! `&mut usize`, а неинициализированные constructor-ом `m_btGoodsType` и
//! `m_dwLvLimit` — `Option`: `Clear` буквально не меняет их, успешный
//! `UnSerialize` задаёт `Some`. Никакой исходный default для этих полей не
//! придуман.
//!
//! Старые helpers не знали длину источника и могли читать за его концом либо
//! писать строку длиннее 256 байт. Безопасная граница возвращает typed-ошибку,
//! сохраняя уже выполненные `Clear` и успешные последовательные присваивания.
//! Наблюдаемая реакция оригинального процесса на эти UB-входы не объявляется
//! fail-closed контрактом; конкретные неизвестности локализованы у проверок.
//! Реализованные тела и их прямой compiler/STL cleanup удалены, незатронутый
//! аукционный псевдокод сохранён ниже.

use std::error::Error;
use std::fmt;

use super::guid::CGuid;

const LEGACY_STRING_CAPACITY: usize = 0x100;
const AUCTION_INFO_SIZE: usize = 0x22c;
const AUCTION_SELLER_NAME_OFFSET: usize = 0x000;
const AUCTION_MONEY_SELLER_OFFSET: usize = 0x100;
const AUCTION_TIME_SELLER_OFFSET: usize = 0x104;
const AUCTION_SELLER_ID_OFFSET: usize = 0x108;
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

#[derive(Clone, Copy, Eq, PartialEq)]
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
    /// сужения до известных состояний: exact `LoadGoodsByOwnerId` просто
    /// копировал `GoodsState` из строки в node.
    pub(crate) const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }
}

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
        copy_legacy_database_string(
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
        copy_legacy_database_string(
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
    /// Это узкая граница `CDbMisc::LoadGoodsByOwnerId`: exact owner заполнял
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
    /// Exact owner заполнял только amount, GUID, base-index, `STATE_BACK` и
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

    /// Создаёт byte-exact результат старого `Serialize`.
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
    pub(super) const fn add_ticket(&self) -> u32 {
        self.add_ticket
    }

    /// Возвращает исходный unsigned owner id без изменения битов.
    pub(crate) const fn owner_id(&self) -> u32 {
        self.owner_id
    }

    /// Возвращает exact `m_bDb` для World auction relay.
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

    /// Собирает аргументы exact `exec addnewGoods` без переинтерпретации
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

    /// Возвращает старый byte money type.
    pub(super) const fn money_type(&self) -> u8 {
        self.money_type
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
    pub(super) const fn goods_name(&self) -> &[u8; LEGACY_STRING_CAPACITY] {
        &self.goods_name
    }

    /// Выполняет доказанное присваивание `m_GoodsState = STATE_AUCTION`.
    pub(super) fn mark_as_auction(&mut self) {
        self.goods_state = GoodsState::AUCTION;
    }

    /// Проверяет точное состояние `STATE_AUCTION`.
    pub(super) fn is_auction(&self) -> bool {
        self.goods_state == GoodsState::AUCTION
    }

    /// Возвращает достигнутое `DoneDelList` состояние без переинтерпретации.
    pub(crate) const fn goods_state(&self) -> GoodsState {
        self.goods_state
    }

    /// Возвращает достигнутый unsigned индекс исходного auction-node.
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

    /// Возвращает достигнутый `DoneDelList` флаг наличия ставки.
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

/// Безопасная замена `_snprintf(buffer, 0x100, "%s", database_text)`.
///
/// Нормальная строка сохраняется byte-for-byte и с первым NUL. Старый MSVC
/// мог оставить усечённый buffer без NUL, а последующий `Serialize` читать за
/// его границей; это внутренний дефект без доказанного контракта, поэтому Rust
/// всегда ставит terminator в последней ячейке.
fn copy_legacy_database_string(destination: &mut [u8; LEGACY_STRING_CAPACITY], source: &[u8]) {
    destination.fill(0);
    let visible = source
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(source.len());
    let copied = visible.min(LEGACY_STRING_CAPACITY - 1);
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
                // BLOCKED_MISSING_FACT: helpers Misc RVA 0x00005830, Game
                // 0x0007AC80 и World 0x000A3190 после 256 байт продолжали
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
            // BLOCKED_MISSING_FACT: прямые scalar-read в `UnSerialize` сначала
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

// COMPONENT_VARIANT_BEGIN: MiscServer
// Точная пара: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SHA-256 EXE: F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65
// SHA-256 PDB: ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7
// Исходный владелец PDB: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp

// ============================================================================
// FUNCTION: Catch@00409277
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00009277
// ADDRESS: 00409277
// PROTOTYPE: undefined __stdcall Catch@00409277(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040930e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x0000930E
// ADDRESS: 0040930e
// PROTOTYPE: undefined __stdcall Catch@0040930e(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L87944
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023180
// ADDRESS: 00423180
// PROTOTYPE: undefined __stdcall $L87944(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L95669
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000231A0
// ADDRESS: 004231a0
// PROTOTYPE: undefined __stdcall $L95669(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L100099
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000231C0
// ADDRESS: 004231c0
// PROTOTYPE: undefined __stdcall $L100099(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L100476
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000231E0
// ADDRESS: 004231e0
// PROTOTYPE: undefined __stdcall $L100476(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L100477
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000231E8
// ADDRESS: 004231e8
// PROTOTYPE: undefined __stdcall $L100477(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L100683
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023200
// ADDRESS: 00423200
// PROTOTYPE: undefined __stdcall $L100683(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L100684
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023208
// ADDRESS: 00423208
// PROTOTYPE: undefined __stdcall $L100684(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L102981
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023220
// ADDRESS: 00423220
// PROTOTYPE: undefined __stdcall $L102981(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L102982
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023228
// ADDRESS: 00423228
// PROTOTYPE: undefined __stdcall $L102982(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L102983
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023230
// ADDRESS: 00423230
// PROTOTYPE: undefined __stdcall $L102983(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L102984
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023238
// ADDRESS: 00423238
// PROTOTYPE: undefined __stdcall $L102984(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L102985
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023240
// ADDRESS: 00423240
// PROTOTYPE: undefined __stdcall $L102985(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L105778
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023260
// ADDRESS: 00423260
// PROTOTYPE: undefined __stdcall $L105778(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L107657
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023280
// ADDRESS: 00423280
// PROTOTYPE: undefined __stdcall $L107657(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L107658
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023288
// ADDRESS: 00423288
// PROTOTYPE: undefined __stdcall $L107658(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L107948
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232A0
// ADDRESS: 004232a0
// PROTOTYPE: undefined __stdcall $L107948(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L107949
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232A8
// ADDRESS: 004232a8
// PROTOTYPE: undefined __stdcall $L107949(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109363
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232C0
// ADDRESS: 004232c0
// PROTOTYPE: undefined __stdcall $L109363(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109364
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232C8
// ADDRESS: 004232c8
// PROTOTYPE: undefined __stdcall $L109364(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109365
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232D3
// ADDRESS: 004232d3
// PROTOTYPE: undefined __stdcall $L109365(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109366
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232DE
// ADDRESS: 004232de
// PROTOTYPE: undefined __stdcall $L109366(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109367
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232E9
// ADDRESS: 004232e9
// PROTOTYPE: undefined __stdcall $L109367(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109368
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232F4
// ADDRESS: 004232f4
// PROTOTYPE: undefined __stdcall $L109368(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109369
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000232FF
// ADDRESS: 004232ff
// PROTOTYPE: undefined __stdcall $L109369(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109370
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x0002330A
// ADDRESS: 0042330a
// PROTOTYPE: undefined __stdcall $L109370(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L109371
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023315
// ADDRESS: 00423315
// PROTOTYPE: undefined __stdcall $L109371(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L110989
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023330
// ADDRESS: 00423330
// PROTOTYPE: undefined __stdcall $L110989(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L110990
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023338
// ADDRESS: 00423338
// PROTOTYPE: undefined __stdcall $L110990(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113025
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023350
// ADDRESS: 00423350
// PROTOTYPE: undefined __stdcall $L113025(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113026
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023358
// ADDRESS: 00423358
// PROTOTYPE: undefined __stdcall $L113026(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113214
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023370
// ADDRESS: 00423370
// PROTOTYPE: undefined __stdcall $L113214(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113215
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023378
// ADDRESS: 00423378
// PROTOTYPE: undefined __stdcall $L113215(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113855
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023390
// ADDRESS: 00423390
// PROTOTYPE: undefined __stdcall $L113855(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113856
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x00023398
// ADDRESS: 00423398
// PROTOTYPE: undefined __stdcall $L113856(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113857
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233A3
// ADDRESS: 004233a3
// PROTOTYPE: undefined __stdcall $L113857(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113858
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233AE
// ADDRESS: 004233ae
// PROTOTYPE: undefined __stdcall $L113858(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113859
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233B9
// ADDRESS: 004233b9
// PROTOTYPE: undefined __stdcall $L113859(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113860
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233C4
// ADDRESS: 004233c4
// PROTOTYPE: undefined __stdcall $L113860(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113861
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233CF
// ADDRESS: 004233cf
// PROTOTYPE: undefined __stdcall $L113861(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L113862
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233DA
// ADDRESS: 004233da
// PROTOTYPE: undefined __stdcall $L113862(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L115616
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\auctionroom\auctionnode.cpp
// RVA: 0x000233F0
// ADDRESS: 004233f0
// PROTOTYPE: undefined __stdcall $L115616(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: MiscServer

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.cpp

// ============================================================================
// FUNCTION: CGoodsNode::SetAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h:99
// RVA: 0x0002AB40
// ADDRESS: 0042ab40
// PROTOTYPE: void __thiscall SetAccount(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsNode::SetGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h:127
// RVA: 0x0002AB60
// ADDRESS: 0042ab60
// PROTOTYPE: void __thiscall SetGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsNode::SetGoodsName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h:137
// RVA: 0x0002ABB0
// ADDRESS: 0042abb0
// PROTOTYPE: void __thiscall SetGoodsName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsNode::ComputerEndTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.cpp:150
// RVA: 0x000D8FC0
// ADDRESS: 004d8fc0
// PROTOTYPE: uint __thiscall ComputerEndTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsNode::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.cpp:44
// RVA: 0x000D9460
// ADDRESS: 004d9460
// PROTOTYPE: bool __thiscall Clone(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h

// ============================================================================
// FUNCTION: CGoodsNode::SetGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionnode.h:127
// RVA: 0x000EF5F0
// ADDRESS: 004ef5f0
// PROTOTYPE: void __thiscall SetGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
