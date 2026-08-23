//! Торговые списки `CTradeList` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/tradelist.cpp`.
//!
//! Loader очищает map, читает `*` NPC и следующие `#` goods records. StringTable
//! miss даёт пустое имя, unknown goods — нулевой ID; duplicate NPC заменяет
//! список. Числовые page/x/y/amount сужаются до byte.
//!
//! Wire пишет ordered NPC C-строки, signed counts и восьмибайтные goods records.
//! `BTreeMap<Vec<u8>, _>` сохраняет byte-лексикографический порядок. Game decoder
//! очищает map до count и публикует NPC только после полного goods-list;
//! safe truncation поэтому сохраняет лишь завершённый prefix.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CTradeList {
    trades: BTreeMap<Vec<u8>, Trade>,
}

impl CTradeList {
    pub(crate) fn clear(&mut self) {
        self.trades.clear();
    }

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

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, TradeListDecodeError> {
        self.trades.clear();
        let trade_count = read_wire_i32(source, cursor)?;
        if trade_count <= 0 {
            return Ok(0);
        }

        for _ in 0..trade_count {
            let npc_name = read_wire_c_string(source, cursor)?;
            let goods_count = read_wire_i32(source, cursor)?;
            let mut goods = Vec::new();
            for _ in 0..goods_count.max(0) {
                let compact = read_wire_array::<4>(source, cursor)?;
                goods.push(TradeGoods {
                    page: compact[0],
                    position_x: compact[1],
                    position_y: compact[2],
                    amount: compact[3],
                    goods_id: read_wire_u32(source, cursor)?,
                });
            }
            self.trades
                .insert(npc_name.clone(), Trade { npc_name, goods });
        }
        Ok(self.trades.len())
    }

    pub(crate) fn len(&self) -> usize {
        self.trades.len()
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TradeListDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    MissingStringTerminator {
        offset: usize,
    },
}

impl fmt::Display for TradeListDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "TradeList snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::MissingStringTerminator { offset } => write!(
                formatter,
                "TradeList NPC name с {offset} не завершено нулём"
            ),
        }
    }
}

impl Error for TradeListDecodeError {}

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

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, TradeListDecodeError> {
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, TradeListDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], TradeListDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(TradeListDecodeError::UnexpectedEnd {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер TradeList scalar уже проверен"))
}

fn read_wire_c_string(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, TradeListDecodeError> {
    let offset = *cursor;
    let remaining = source
        .get(offset..)
        .ok_or(TradeListDecodeError::UnexpectedEnd {
            offset,
            needed: 1,
            available: 0,
        })?;
    let Some(length) = remaining.iter().position(|byte| *byte == 0) else {
        return Err(TradeListDecodeError::MissingStringTerminator { offset });
    };
    *cursor += length + 1;
    Ok(remaining[..length].to_vec())
}

// оставшихся call-site деталей, а не как Rust-реализация.
