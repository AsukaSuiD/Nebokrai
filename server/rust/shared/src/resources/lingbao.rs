//! LingBao `CLingBaoSetup` из WorldServer и GameServer: loader/serializer
//! подтверждены парой WorldServer EXE/PDB, decoder `DecodeFromArrayLingBao`
//! — парой GameServer (в `server/rust/src/manifest/`). Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp/.h`.
//!
//! Wire: signed multimap count, key C-string, ticket и три vector-секции с
//! records по 8, 5 и 3 `u32`. Равные byte-keys сохраняют insertion order:
//! исходный `_Insert` при равном ключе уходит вправо и вставляет новый node
//! без выхода по совпадению, поэтому gameplay поиск сканирует map парой
//! (ключ, ticket).
//!
//! Loader очищает map, игнорирует labels позиционного token stream и при
//! missing resource оставляет state пустым; повреждённый хвост останавливается
//! после последней полной записи. Game decoder тоже сначала очищает multimap
//! и публикует только полные records; динамическая byte-string заменяет старый
//! 256-byte stack buffer без воспроизведения overflow. Отрицательный signed
//! count, который оригинал принял бы за огромный unsigned loop, Rust трактует
//! как пустой вход (`max(0)`), не перенося unbounded-read.
//! Установленный экземпляр и его потребители остаются у владельца роли.
//! Доказательства: docs/reconstruction/shared-technical.md#lingbao-clingbaosetup

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::protocol::LegacyReader;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LingBaoFirstNode {
    pub node_type: u32,
    pub use_ticket: u32,
    pub probability_1: u32,
    pub add_value_1: u32,
    pub probability_2: u32,
    pub add_value_2: u32,
    pub probability_3: u32,
    pub add_value_3: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LingBaoSecondNode {
    pub node_type: u32,
    pub probability: u32,
    pub use_ticket: u32,
    pub add_value_min: u32,
    pub add_value_max: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LingBaoThirdNode {
    pub node_type: u32,
    pub use_ticket: u32,
    pub use_ticket_max: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LingBaoNodeInfo {
    pub ticket: u32,
    pub first: Vec<LingBaoFirstNode>,
    pub second: Vec<LingBaoSecondNode>,
    pub third: Vec<LingBaoThirdNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CLingBaoSetup {
    entries: BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>>,
}

impl CLingBaoSetup {
    pub fn insert(&mut self, name: Vec<u8>, info: LingBaoNodeInfo) {
        self.entries.entry(name).or_default().push(info);
    }

    pub fn entries(&self) -> &BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>> {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn load_from_bytes(&mut self, source: Option<&[u8]>) -> LingBaoLoadReport {
        self.clear();
        let Some(source) = source else {
            return LingBaoLoadReport {
                missing_resource: true,
            };
        };

        let mut parser = LingBaoTokenParser::new(source);
        let Some(total) = parser.labeled_u32() else {
            return LingBaoLoadReport::default();
        };
        for _ in 0..total {
            let (Some(_label), Some(original_name), Some(_transform_label), Some(transforms)) =
                (parser.token(), parser.token(), parser.token(), parser.u32())
            else {
                break;
            };
            for ticket in 1..=transforms {
                let Some(first_count) = parser.two_labels_and_u32() else {
                    return LingBaoLoadReport::default();
                };
                let mut first = Vec::new();
                for _ in 0..first_count {
                    let Some(node) = parser.first_node() else {
                        return LingBaoLoadReport::default();
                    };
                    first.push(node);
                }

                let Some(second_count) = parser.labeled_u32() else {
                    return LingBaoLoadReport::default();
                };
                let mut second = Vec::new();
                for _ in 0..second_count {
                    let Some(node) = parser.second_node() else {
                        return LingBaoLoadReport::default();
                    };
                    second.push(node);
                }

                let Some(third_count) = parser.labeled_u32() else {
                    return LingBaoLoadReport::default();
                };
                let mut third = Vec::new();
                for _ in 0..third_count {
                    let Some(node) = parser.third_node() else {
                        return LingBaoLoadReport::default();
                    };
                    third.push(node);
                }

                self.insert(
                    original_name.to_vec(),
                    LingBaoNodeInfo {
                        ticket,
                        first,
                        second,
                        third,
                    },
                );
            }
        }
        LingBaoLoadReport::default()
    }

    pub fn add_byte_ling_bao(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), LingBaoSerializationBlock> {
        let total_count = self
            .entries
            .values()
            .try_fold(0usize, |total, group| total.checked_add(group.len()));
        let total_count = total_count
            .ok_or(LingBaoSerializationBlock::EntryCountOutOfRange { count: usize::MAX })?;
        let count = i32::try_from(total_count)
            .map_err(|_| LingBaoSerializationBlock::EntryCountOutOfRange { count: total_count })?;

        let mut payload = Vec::new();
        payload.extend_from_slice(&count.to_le_bytes());
        let mut entry_index = 0usize;
        for (name, group) in &self.entries {
            for info in group {
                if name.contains(&0) {
                    return Err(LingBaoSerializationBlock::NameContainsNul { entry_index });
                }
                payload.extend_from_slice(name);
                payload.push(0);
                payload.extend_from_slice(&info.ticket.to_le_bytes());
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::First,
                    info.first.len(),
                )?;
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::Second,
                    info.second.len(),
                )?;
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::Third,
                    info.third.len(),
                )?;
                for node in &info.first {
                    write_ling_bao_fields(
                        &mut payload,
                        &[
                            node.node_type,
                            node.use_ticket,
                            node.probability_1,
                            node.add_value_1,
                            node.probability_2,
                            node.add_value_2,
                            node.probability_3,
                            node.add_value_3,
                        ],
                    );
                }
                for node in &info.second {
                    write_ling_bao_fields(
                        &mut payload,
                        &[
                            node.node_type,
                            node.probability,
                            node.use_ticket,
                            node.add_value_min,
                            node.add_value_max,
                        ],
                    );
                }
                for node in &info.third {
                    write_ling_bao_fields(
                        &mut payload,
                        &[node.node_type, node.use_ticket, node.use_ticket_max],
                    );
                }
                entry_index += 1;
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }

    /// Декодирует Game projection непосредственно после CiQing payload.
    pub fn decode_from_array_ling_bao(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), LingBaoDecodeError> {
        self.clear();
        let count = read_wire_i32(source, cursor, "entry count")?;
        for _ in 0..count.max(0) {
            let name = read_wire_c_string(source, cursor, "entry name")?;
            let ticket = read_wire_u32(source, cursor, "ticket")?;
            let first_count = read_wire_i32(source, cursor, "first-node count")?;
            let second_count = read_wire_i32(source, cursor, "second-node count")?;
            let third_count = read_wire_i32(source, cursor, "third-node count")?;

            let mut first = Vec::new();
            for _ in 0..first_count.max(0) {
                first.push(LingBaoFirstNode {
                    node_type: read_wire_u32(source, cursor, "first node type")?,
                    use_ticket: read_wire_u32(source, cursor, "first use ticket")?,
                    probability_1: read_wire_u32(source, cursor, "first probability 1")?,
                    add_value_1: read_wire_u32(source, cursor, "first add value 1")?,
                    probability_2: read_wire_u32(source, cursor, "first probability 2")?,
                    add_value_2: read_wire_u32(source, cursor, "first add value 2")?,
                    probability_3: read_wire_u32(source, cursor, "first probability 3")?,
                    add_value_3: read_wire_u32(source, cursor, "first add value 3")?,
                });
            }

            let mut second = Vec::new();
            for _ in 0..second_count.max(0) {
                second.push(LingBaoSecondNode {
                    node_type: read_wire_u32(source, cursor, "second node type")?,
                    probability: read_wire_u32(source, cursor, "second probability")?,
                    use_ticket: read_wire_u32(source, cursor, "second use ticket")?,
                    add_value_min: read_wire_u32(source, cursor, "second add value min")?,
                    add_value_max: read_wire_u32(source, cursor, "second add value max")?,
                });
            }

            let mut third = Vec::new();
            for _ in 0..third_count.max(0) {
                third.push(LingBaoThirdNode {
                    node_type: read_wire_u32(source, cursor, "third node type")?,
                    use_ticket: read_wire_u32(source, cursor, "third use ticket")?,
                    use_ticket_max: read_wire_u32(source, cursor, "third use ticket max")?,
                });
            }

            self.insert(
                name,
                LingBaoNodeInfo {
                    ticket,
                    first,
                    second,
                    third,
                },
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LingBaoLoadReport {
    pub missing_resource: bool,
}

struct LingBaoTokenParser<'a> {
    remaining: &'a [u8],
}

impl<'a> LingBaoTokenParser<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self { remaining: source }
    }

    fn token(&mut self) -> Option<&'a [u8]> {
        let start = self
            .remaining
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())?;
        self.remaining = &self.remaining[start..];
        let end = self
            .remaining
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(self.remaining.len());
        let (token, remaining) = self.remaining.split_at(end);
        self.remaining = remaining;
        Some(token)
    }

    fn u32(&mut self) -> Option<u32> {
        legacy_u32(self.token()?)
    }

    fn labeled_u32(&mut self) -> Option<u32> {
        self.token()?;
        self.u32()
    }

    fn two_labels_and_u32(&mut self) -> Option<u32> {
        self.token()?;
        self.token()?;
        self.u32()
    }

    fn first_node(&mut self) -> Option<LingBaoFirstNode> {
        self.token()?;
        self.token()?;
        let node_type = self.u32()?;
        let use_ticket = self.labeled_u32()?;
        self.token()?;
        let add_value_1 = self.u32()?;
        let add_value_2 = self.u32()?;
        let add_value_3 = self.u32()?;
        self.token()?;
        let probability_1 = self.u32()?;
        let probability_2 = self.u32()?;
        let probability_3 = self.u32()?;
        Some(LingBaoFirstNode {
            node_type,
            use_ticket,
            probability_1,
            add_value_1,
            probability_2,
            add_value_2,
            probability_3,
            add_value_3,
        })
    }

    fn second_node(&mut self) -> Option<LingBaoSecondNode> {
        self.token()?;
        self.token()?;
        let node_type = self.u32()?;
        let probability = self.labeled_u32()?;
        let use_ticket = self.labeled_u32()?;
        let add_value_min = self.labeled_u32()?;
        let add_value_max = self.labeled_u32()?;
        Some(LingBaoSecondNode {
            node_type,
            probability,
            use_ticket,
            add_value_min,
            add_value_max,
        })
    }

    fn third_node(&mut self) -> Option<LingBaoThirdNode> {
        self.token()?;
        self.token()?;
        let node_type = self.u32()?;
        let use_ticket = self.labeled_u32()?;
        let use_ticket_max = self.labeled_u32()?;
        Some(LingBaoThirdNode {
            node_type,
            use_ticket,
            use_ticket_max,
        })
    }
}

fn legacy_u32(token: &[u8]) -> Option<u32> {
    let mut bytes = token.iter().copied().peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut parsed = false;
    let mut result = 0_u32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(u32::from(digit));
    }
    parsed.then_some(if negative {
        0_u32.wrapping_sub(result)
    } else {
        result
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LingBaoNodeSection {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LingBaoSerializationBlock {
    EntryCountOutOfRange {
        count: usize,
    },
    NodeCountOutOfRange {
        entry_index: usize,
        section: LingBaoNodeSection,
        count: usize,
    },
    NameContainsNul {
        entry_index: usize,
    },
}

impl fmt::Display for LingBaoSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCountOutOfRange { count } => write!(
                formatter,
                "CLingBaoSetup содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NodeCountOutOfRange {
                entry_index,
                section,
                count,
            } => write!(
                formatter,
                "CLingBaoSetup #{entry_index} {section:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul { entry_index } => {
                write!(
                    formatter,
                    "CLingBaoSetup #{entry_index}: имя содержит внутренний NUL"
                )
            }
        }
    }
}

impl Error for LingBaoSerializationBlock {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LingBaoDecodeError {
    pub field: &'static str,
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

impl fmt::Display for LingBaoDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "LingBao snapshot, поле {} на {}: нужно {}, доступно {}",
            self.field, self.offset, self.needed, self.available
        )
    }
}

impl Error for LingBaoDecodeError {}

fn read_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, LingBaoDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block| LingBaoDecodeError {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, LingBaoDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(|block| LingBaoDecodeError {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, LingBaoDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    if offset > source.len() {
        return Err(LingBaoDecodeError {
            field,
            offset,
            needed: 1,
            available,
        });
    }
    LegacyReader::read_c_string_from(source, cursor, available)
        .map(|value| value.to_vec())
        .map_err(|_| LingBaoDecodeError {
            field,
            offset,
            needed: available.saturating_add(1),
            available,
        })
}

fn write_ling_bao_count(
    destination: &mut Vec<u8>,
    entry_index: usize,
    section: LingBaoNodeSection,
    count: usize,
) -> Result<(), LingBaoSerializationBlock> {
    let count =
        i32::try_from(count).map_err(|_| LingBaoSerializationBlock::NodeCountOutOfRange {
            entry_index,
            section,
            count,
        })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_ling_bao_fields(destination: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}
