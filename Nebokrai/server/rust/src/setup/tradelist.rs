//! Торговые списки NPC исторического Miracle.
//!
//! Контракт World `LoadTradeList` и `AddToByteArray`
//!:; Game lookup/decoder не входят в этот owner и остаются
//! Точная пара:
//! Исходный владелец PDB:
//!
//! Loader сначала безусловно очищает map, затем для каждого `*` читает
//! StringTable ID NPC и signed число товаров. Имя NPC разрешается сразу;
//! missing ID становится пустой строкой. Каждая найденная `#`-запись содержит
//! page/x/y, original-name товара и amount: четыре signed `long` сужаются до
//! младшего байта, а original-name lookup сохраняет исходный нулевой ID при
//! отсутствии. Если `#` не найден, slot просто пропускается, как в EXE.
//! Duplicate localized NPC name заменяет прежний список.
//!
//! Wire — signed count ordered map-а, затем для каждого NPC C-строка с NUL,
//! signed goods count и точные восьмибайтные записи
//! `[page, x, y, amount, goods_id_le]`. Legacy строки хранятся byte-exact, а
//! не как обязательный UTF-8; `BTreeMap<Vec<u8>, _>` сохраняет unsigned
//! лексикографический порядок `std::string`. `std::fs`, `Vec` и `Drop`
//! заменяют CRFile/STL plumbing, не меняя формат.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

/// Точная layout-проекция `tagTrade::tagGoods` без C++ padding-зависимости.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TradeGoods {
    pub(crate) page: u8,
    pub(crate) position_x: u8,
    pub(crate) position_y: u8,
    pub(crate) amount: u8,
    pub(crate) goods_id: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Trade {
    npc_name: Vec<u8>,
    goods: Vec<TradeGoods>,
}

/// Safe owner исходного process-global `m_mapTradeList`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CTradeList {
    trades: BTreeMap<Vec<u8>, Trade>,
}

impl CTradeList {
 /// `rfOpen` failure уже оставляет очищенную карту.
    pub(crate) fn clear(&mut self) {
        self.trades.clear();
    }

 /// Очищает прежний state до открытия, как исходный loader.
    pub(crate) fn load_from_file<ResolveNpcName, ResolveGoodsId>(
        &mut self,
        path: impl AsRef<Path>,
        resolve_npc_name: &mut ResolveNpcName,
        resolve_goods_id: &mut ResolveGoodsId,
    ) -> Result<usize, TradeListFileLoadError>
    where
        ResolveNpcName: FnMut(&[u8]) -> Option<Vec<u8>>,
        ResolveGoodsId: FnMut(&[u8]) -> u32,
    {
        self.trades.clear();
        let source = std::fs::read(path).map_err(TradeListFileLoadError::Io)?;
        self.load_from_bytes(&source, resolve_npc_name, resolve_goods_id)
            .map_err(TradeListFileLoadError::Format)
    }

 /// Загружает formatted token-stream с оригинал merge/replace семантикой.
    pub(crate) fn load_from_bytes<ResolveNpcName, ResolveGoodsId>(
        &mut self,
        source: &[u8],
        resolve_npc_name: &mut ResolveNpcName,
        resolve_goods_id: &mut ResolveGoodsId,
    ) -> Result<usize, TradeListFormatError>
    where
        ResolveNpcName: FnMut(&[u8]) -> Option<Vec<u8>>,
        ResolveGoodsId: FnMut(&[u8]) -> u32,
    {
        self.trades.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty())
            .peekable();
        let mut applied = 0;

        while read_to(&mut tokens, b"*") {
            let npc_name_id = next_token(&mut tokens, "StringTable ID NPC")?;
            let goods_count = read_i32(&mut tokens, "число товаров NPC")?;
            let resolved_name = resolve_npc_name(npc_name_id).unwrap_or_default();
            let npc_name = truncate_at_nul(&resolved_name).to_vec();
            let mut goods = Vec::new();

            for _ in 0..goods_count.max(0) {
                if !read_to(&mut tokens, b"#") {
                    if tokens.peek().is_none() {
                        break;
                    }
                    continue;
                }
                let page = read_i32(&mut tokens, "страница товара")? as u8;
                let position_x = read_i32(&mut tokens, "позиция X товара")? as u8;
                let position_y = read_i32(&mut tokens, "позиция Y товара")? as u8;
                let original_name = next_token(&mut tokens, "original-name товара")?;
                let amount = read_i32(&mut tokens, "количество товара")? as u8;
                let goods_id = resolve_goods_id(truncate_at_nul(original_name));
                goods.push(TradeGoods {
                    page,
                    position_x,
                    position_y,
                    amount,
                    goods_id,
                });
            }

            self.trades
                .insert(npc_name.clone(), Trade { npc_name, goods });
            applied += 1;
        }
        Ok(applied)
    }

 /// Дописывает оригинал World `count + ordered trade records` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), TradeListSerializeError> {
        let trade_count =
            i32::try_from(self.trades.len()).map_err(|_| TradeListSerializeError::TradeCount {
                count: self.trades.len(),
            })?;
        destination.extend_from_slice(&trade_count.to_le_bytes());

        for trade in self.trades.values() {
            if trade.npc_name.contains(&0) {
                return Err(TradeListSerializeError::NpcNameContainsNul);
            }
            destination.extend_from_slice(&trade.npc_name);
            destination.push(0);
            let goods_count = i32::try_from(trade.goods.len()).map_err(|_| {
                TradeListSerializeError::GoodsCount {
                    npc_name: trade.npc_name.clone(),
                    count: trade.goods.len(),
                }
            })?;
            destination.extend_from_slice(&goods_count.to_le_bytes());
            for goods in &trade.goods {
                destination.extend_from_slice(&[
                    goods.page,
                    goods.position_x,
                    goods.position_y,
                    goods.amount,
                ]);
                destination.extend_from_slice(&goods.goods_id.to_le_bytes());
            }
        }
        Ok(())
    }
}

/// Ошибка безопасного formatted parser-а после уже применённых полных блоков.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TradeListFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for TradeListFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является signed long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for TradeListFormatError {}

#[derive(Debug)]
pub(crate) enum TradeListFileLoadError {
    Io(std::io::Error),
    Format(TradeListFormatError),
}

impl fmt::Display for TradeListFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for TradeListFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

/// Невозможный для signed-count wire state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TradeListSerializeError {
    TradeCount { count: usize },
    NpcNameContainsNul,
    GoodsCount { npc_name: Vec<u8>, count: usize },
}

impl fmt::Display for TradeListSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TradeCount { count } => write!(
                formatter,
                "trade map содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NpcNameContainsNul => formatter.write_str("имя NPC содержит внутренний NUL"),
            Self::GoodsCount { npc_name, count } => write!(
                formatter,
                "NPC {} содержит {count} товаров вне signed 32-битного диапазона",
                String::from_utf8_lossy(npc_name)
            ),
        }
    }
}

impl Error for TradeListSerializeError {}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], TradeListFormatError> {
    tokens
        .next()
        .ok_or(TradeListFormatError::UnexpectedEnd { field })
}

fn read_i32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i32, TradeListFormatError> {
    let token = next_token(tokens, field)?;
    let text = std::str::from_utf8(token).map_err(|_| TradeListFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<i32>()
        .map_err(|_| TradeListFormatError::InvalidLong {
            field,
            token: token.to_vec(),
        })
}

fn truncate_at_nul(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

// оставшихся call-site деталей, а не как Rust-реализация.
