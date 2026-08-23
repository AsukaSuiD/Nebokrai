//! Increment-shop initial configuration исторического Miracle.
//!
//! Контракт World `AddToByteArray`, `Release`
//! и `LoadItems`:; singleton
//! plumbing и Game decoder не входят в этот owner и остаются. Точная пара:
//! Исходный владелец PDB:
//!
//! Owner хранит `multimap<unsigned char, Item*>`: wire начинается с signed
//! count, затем идёт unsigned page-key, первые `0x18` байт Item, description и
//! localized goods key как две C-строки; после всех записей следует C-строка
//! affiche. Равные page-key сохраняют insertion order, поэтому Rust использует
//! `BTreeMap<u8, Vec<Item>>`.
//!
//! EXE копировал весь 24-байтный prefix, хотя loader заполнял только category,
//! четыре `u32` и icon: два байта после category и три после icon оставались
//! heap-мусором. Точный Game decoder пропускает эти reserved-позиции. Rust
//! сохраняет размер/layout, но пишет нули, устраняя внутреннюю утечку без
//! изменения принимаемых полей. `LoadItems` сначала освобождает прежнее
//! состояние, затем для каждой `#`-позиции читает восемь whitespace-полей.
//! Page/category вне `0..=255`, неразрешённый основной товар, ненулевой
//! неразрешённый товар скидки, пустое описание либо отсутствующее отображаемое
//! имя завершают load с уже внесённой частичной картой. Overlap менее `1`
//! заменяется на `1`; price/icon сохраняют сужение signed long до legacy
//! `u32/u8`. Второй проход между `<AfficheStart>` и `<AfficheEnd>` добавляет
//! каждую целую строку с `\n`. Safe owner не допускает null Item slot-ов и
//! освобождает весь registry через `Drop`, а не воспроизводит ошибочный ручной
//! lifetime.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::public::readwrite::read_to;

const ITEM_WIRE_LENGTH: usize = 0x18;

/// Значимые поля первых `0x18` байт legacy Item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IncrementShopItem {
    pub(crate) category: u16,
    pub(crate) overlapped_amount: u32,
    pub(crate) goods_id: u32,
    pub(crate) yuan_bao_price: u32,
    pub(crate) deduction_goods_id: u32,
    pub(crate) icon_id: u8,
    pub(crate) description: Vec<u8>,
    pub(crate) key: Vec<u8>,
}

/// Safe owner исходного singleton state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CIncrementShopList {
    items: BTreeMap<u8, Vec<IncrementShopItem>>,
    affiche: Vec<u8>,
}

/// Единственная serial lookup-граница `CGoodsFactory` loader-а.
pub(crate) enum IncrementShopGoodsQuery<'name> {
    OriginalName(&'name [u8]),
    DisplayName(u32),
}

pub(crate) enum IncrementShopGoodsResult {
    Id(u32),
    Name(Option<Vec<u8>>),
}

/// Результат загрузки: предупреждения original `AddLogText` до success-log.
pub(crate) struct IncrementShopLoadReport {
    pub(crate) warnings: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopLoadError {
    UnexpectedEnd,
    InvalidTotalSort(i32),
    InvalidSubSort(i32),
    GoodsNotFound(Vec<u8>),
    DiKouGoodsNotFound(Vec<u8>),
    ItemDescriptionNotFound(Vec<u8>),
    ItemNameNotFound(Vec<u8>),
}

impl IncrementShopLoadError {
    pub(crate) fn log_payload(&self) -> Vec<u8> {
        match self {
            Self::UnexpectedEnd => Vec::new(),
            Self::InvalidTotalSort(value) => {
                let mut payload = value.to_string().into_bytes();
                payload.extend_from_slice(b" Total Sort INVALID, Ignore This Setup.");
                payload
            }
            Self::InvalidSubSort(value) => {
                let mut payload = value.to_string().into_bytes();
                payload.extend_from_slice(b" Sub Sort INVALID, Ignore This Setup.");
                payload
            }
            Self::GoodsNotFound(token) => with_suffix(token, b" Goods Not Found, Ignore This Setup."),
            Self::DiKouGoodsNotFound(token) => {
                with_suffix(token, b" DiKouGoods Not Found, Ignore This Setup.")
            }
            Self::ItemDescriptionNotFound(token) => {
                with_suffix(token, b" item desc Not Found. ignore this item!")
            }
            Self::ItemNameNotFound(token) => {
                with_suffix(token, b" item name Not Found. ignore this item!")
            }
        }
    }
}

impl CIncrementShopList {
 /// Эквивалент multimap insertion в конец диапазона равного page-key.
    pub(crate) fn insert(&mut self, page: u8, item: IncrementShopItem) {
        self.items.entry(page).or_default().push(item);
    }

    pub(crate) fn set_affiche(&mut self, affiche: &[u8]) {
        self.affiche.clear();
        self.affiche.extend_from_slice(truncate_at_nul(affiche));
    }

 /// Сохраняет исходный порядок очистки, а ownership освобождает все Item.
    pub(crate) fn release(&mut self) {
        self.affiche.clear();
        self.items.clear();
    }

 /// Загружает точный formatted stream `incrementshoplist.ini`.
 ///
 /// Две ветви lookup принадлежат уже загруженному `CGoodsFactory`. Result
 /// передаёт исходный bool и отложенные `AddLogText`, не скрывая уже
 /// применённую часть state.
    pub(crate) fn load_from_bytes<ResolveGoods>(
        &mut self,
        source: &[u8],
        resolve_goods: &mut ResolveGoods,
    ) -> Result<IncrementShopLoadReport, IncrementShopLoadError>
    where
        ResolveGoods: for<'name> FnMut(IncrementShopGoodsQuery<'name>) -> IncrementShopGoodsResult,
    {
        self.release();
        let mut warnings = Vec::new();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty())
            .peekable();

        while read_to(&mut tokens, b"#") {
            let Some(page) = read_signed_long(&mut tokens) else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            if !(0..=u8::MAX as i32).contains(&page) {
                return Err(IncrementShopLoadError::InvalidTotalSort(page));
            }

            let Some(category) = read_signed_long(&mut tokens) else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            if !(0..=u8::MAX as i32).contains(&category) {
                return Err(IncrementShopLoadError::InvalidSubSort(category));
            }

            let Some(mut overlapped_amount) = read_signed_long(&mut tokens) else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            if overlapped_amount < 1 {
                let mut message = overlapped_amount.to_string().into_bytes();
                message.extend_from_slice(b" Overlapped Num INVALID, set 1 compulsively.");
 // Loader пишет это предупреждение, но продолжает с единицей.
                warnings.push(message);
                overlapped_amount = 1;
            }

            let Some(goods_original_name) = tokens.next() else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            let goods_id = query_goods_id(resolve_goods, goods_original_name);
            if goods_id == 0 {
                return Err(IncrementShopLoadError::GoodsNotFound(goods_original_name.to_vec()));
            }

            let Some(yuan_bao_price) = read_signed_long(&mut tokens) else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            let Some(deduction_original_name) = tokens.next() else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            let deduction_goods_id = query_goods_id(resolve_goods, deduction_original_name);
            if deduction_original_name != b"0" && deduction_goods_id == 0 {
                return Err(IncrementShopLoadError::DiKouGoodsNotFound(
                    deduction_original_name.to_vec(),
                ));
            }

            let Some(icon_id) = read_signed_long(&mut tokens) else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            let Some(description) = tokens.next() else {
                return Err(IncrementShopLoadError::UnexpectedEnd);
            };
            if description.is_empty() {
                return Err(IncrementShopLoadError::ItemDescriptionNotFound(
                    description.to_vec(),
                ));
            }
            let Some(key) = query_goods_name(resolve_goods, goods_id) else {
                return Err(IncrementShopLoadError::ItemNameNotFound(description.to_vec()));
            };

            self.insert(
                page as u8,
                IncrementShopItem {
                    category: category as u16,
                    overlapped_amount: overlapped_amount as u32,
                    goods_id,
                    yuan_bao_price: yuan_bao_price as u32,
                    deduction_goods_id,
                    icon_id: icon_id as u8,
                    description: description.to_vec(),
                    key: truncate_at_nul(&key).to_vec(),
                },
            );
        }

        self.load_affiche(source);
        Ok(IncrementShopLoadReport { warnings })
    }

    fn load_affiche(&mut self, source: &[u8]) {
        self.affiche.clear();
        let mut started = false;
        for raw_line in source.split(|byte| *byte == b'\n') {
            let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
            if !started {
                started = line
                    .split(u8::is_ascii_whitespace)
                    .any(|token| token == b"<AfficheStart>");
                continue;
            }
            if line
                .split(u8::is_ascii_whitespace)
                .any(|token| token == b"<AfficheEnd>")
            {
                break;
            }
            self.affiche.extend_from_slice(line);
            self.affiche.push(b'\n');
        }
    }

 /// Дописывает оригинал `count + items + affiche` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), IncrementShopSerializeError> {
        let item_count = self
            .items
            .values()
            .try_fold(0_usize, |count, items| count.checked_add(items.len()))
            .ok_or(IncrementShopSerializeError::ItemCountOverflow)?;
        let item_count = i32::try_from(item_count)
            .map_err(|_| IncrementShopSerializeError::ItemCountOverflow)?;
        destination.extend_from_slice(&item_count.to_le_bytes());

        for (&page, items) in &self.items {
            for item in items {
                ensure_c_string(&item.description, IncrementShopStringField::Description)?;
                ensure_c_string(&item.key, IncrementShopStringField::Key)?;
                destination.push(page);

                let prefix_start = destination.len();
                destination.extend_from_slice(&item.category.to_le_bytes());
                destination.extend_from_slice(&[0; 2]);
                destination.extend_from_slice(&item.overlapped_amount.to_le_bytes());
                destination.extend_from_slice(&item.goods_id.to_le_bytes());
                destination.extend_from_slice(&item.yuan_bao_price.to_le_bytes());
                destination.extend_from_slice(&item.deduction_goods_id.to_le_bytes());
                destination.push(item.icon_id);
                destination.extend_from_slice(&[0; 3]);
                debug_assert_eq!(destination.len() - prefix_start, ITEM_WIRE_LENGTH);

                destination.extend_from_slice(&item.description);
                destination.push(0);
                destination.extend_from_slice(&item.key);
                destination.push(0);
            }
        }

        ensure_c_string(&self.affiche, IncrementShopStringField::Affiche)?;
        destination.extend_from_slice(&self.affiche);
        destination.push(0);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopStringField {
    Description,
    Key,
    Affiche,
}

impl fmt::Display for IncrementShopStringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Description => "описание increment-shop товара",
            Self::Key => "поисковый ключ increment-shop товара",
            Self::Affiche => "increment-shop affiche",
        })
    }
}

/// Safe boundary для state, не представимого legacy C-string/count wire.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopSerializeError {
    ItemCountOverflow,
    StringContainsNul { field: IncrementShopStringField },
}

impl fmt::Display for IncrementShopSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemCountOverflow => formatter
                .write_str("increment-shop item count не помещается в signed 32-битный диапазон"),
            Self::StringContainsNul { field } => {
                write!(formatter, "{field} содержит внутренний NUL")
            }
        }
    }
}

impl Error for IncrementShopSerializeError {}

fn ensure_c_string(
    value: &[u8],
    field: IncrementShopStringField,
) -> Result<(), IncrementShopSerializeError> {
    if value.contains(&0) {
        return Err(IncrementShopSerializeError::StringContainsNul { field });
    }
    Ok(())
}

fn truncate_at_nul(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

fn read_signed_long<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>) -> Option<i32> {
    let token = tokens.next()?;
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn query_goods_id(
    resolve_goods: &mut impl for<'name> FnMut(IncrementShopGoodsQuery<'name>) -> IncrementShopGoodsResult,
    original_name: &[u8],
) -> u32 {
    match resolve_goods(IncrementShopGoodsQuery::OriginalName(original_name)) {
        IncrementShopGoodsResult::Id(id) => id,
        IncrementShopGoodsResult::Name(_) => 0,
    }
}

fn query_goods_name(
    resolve_goods: &mut impl for<'name> FnMut(IncrementShopGoodsQuery<'name>) -> IncrementShopGoodsResult,
    goods_id: u32,
) -> Option<Vec<u8>> {
    match resolve_goods(IncrementShopGoodsQuery::DisplayName(goods_id)) {
        IncrementShopGoodsResult::Name(name) => name,
        IncrementShopGoodsResult::Id(_) => None,
    }
}

fn with_suffix(value: &[u8], suffix: &[u8]) -> Vec<u8> {
    let mut payload = value.to_vec();
    payload.extend_from_slice(suffix);
    payload
}

// Game decoder-а, а не как Rust-реализация.
