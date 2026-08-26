//! LingBao `CLingBaoSetup` из WorldServer и GameServer.
//!
//! Loader/serializer подтверждены парой
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, decoder
//! `DecodeFromArrayLingBao` —
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp/.h`.
//!
//! Wire пишет signed multimap count, key C-string, ticket и три vector-секции
//! с records по 8, 5 и 3 `u32`. Равные byte-keys сохраняют insertion order.
//!
//! Loader очищает map, игнорирует labels позиционного token stream и при
//! missing resource оставляет state пустым. Повреждённый хвост останавливается
//! после последней полной записи.
//! Game decoder также сначала очищает multimap и публикует только полные
//! records; равные names остаются в insertion order. Динамическая byte-string
//! заменяет старый 256-byte stack buffer без воспроизведения overflow.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoFirstNode {
    pub(crate) node_type: u32,
    pub(crate) use_ticket: u32,
    pub(crate) probability_1: u32,
    pub(crate) add_value_1: u32,
    pub(crate) probability_2: u32,
    pub(crate) add_value_2: u32,
    pub(crate) probability_3: u32,
    pub(crate) add_value_3: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoSecondNode {
    pub(crate) node_type: u32,
    pub(crate) probability: u32,
    pub(crate) use_ticket: u32,
    pub(crate) add_value_min: u32,
    pub(crate) add_value_max: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoThirdNode {
    pub(crate) node_type: u32,
    pub(crate) use_ticket: u32,
    pub(crate) use_ticket_max: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoNodeInfo {
    pub(crate) ticket: u32,
    pub(crate) first: Vec<LingBaoFirstNode>,
    pub(crate) second: Vec<LingBaoSecondNode>,
    pub(crate) third: Vec<LingBaoThirdNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CLingBaoSetup {
    entries: BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>>,
}

impl CLingBaoSetup {
    pub(crate) fn insert(&mut self, name: Vec<u8>, info: LingBaoNodeInfo) {
        self.entries.entry(name).or_default().push(info);
    }

    pub(crate) fn entries(&self) -> &BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>> {
        &self.entries
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn load_from_bytes(&mut self, source: Option<&[u8]>) -> LingBaoLoadReport {
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
            let (Some(_label), Some(original_name), Some(_transform_label), Some(transforms)) = (
                parser.token(),
                parser.token(),
                parser.token(),
                parser.u32(),
            ) else {
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

    pub(crate) fn add_byte_ling_bao(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), LingBaoSerializationBlock> {
        let total_count = self.entries.values().try_fold(0usize, |total, group| {
            total.checked_add(group.len())
        });
        let total_count = total_count.ok_or(LingBaoSerializationBlock::EntryCountOutOfRange {
            count: usize::MAX,
        })?;
        let count = i32::try_from(total_count).map_err(|_| {
            LingBaoSerializationBlock::EntryCountOutOfRange { count: total_count }
        })?;

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
    pub(crate) fn decode_from_array_ling_bao(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), LingBaoDecodeError> {
        self.clear();
        let count = read_wire_i32(source, cursor, "entry count")?;
        let mut entries = 0;
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
            entries += 1;
        }
        tracing::trace!(entries, "настройки LingBao декодированы");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoLoadReport {
    pub(crate) missing_resource: bool,
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
    parsed.then_some(if negative { 0_u32.wrapping_sub(result) } else { result })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LingBaoNodeSection {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LingBaoSerializationBlock {
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
                write!(formatter, "CLingBaoSetup #{entry_index}: имя содержит внутренний NUL")
            }
        }
    }
}

impl Error for LingBaoSerializationBlock {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LingBaoDecodeError {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
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
    Ok(i32::from_le_bytes(read_wire_array(
        source, cursor, field,
    )?))
}

fn read_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, LingBaoDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(
        source, cursor, field,
    )?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], LingBaoDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(LingBaoDecodeError {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер LingBao scalar уже проверен"))
}

fn read_wire_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, LingBaoDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(relative_end) = source
        .get(offset..)
        .and_then(|tail| tail.iter().position(|byte| *byte == 0))
    else {
        return Err(LingBaoDecodeError {
            field,
            offset,
            needed: available.saturating_add(1),
            available,
        });
    };
    let end = offset + relative_end;
    *cursor = end + 1;
    Ok(source[offset..end].to_vec())
}

fn write_ling_bao_count(
    destination: &mut Vec<u8>,
    entry_index: usize,
    section: LingBaoNodeSection,
    count: usize,
) -> Result<(), LingBaoSerializationBlock> {
    let count = i32::try_from(count).map_err(|_| {
        LingBaoSerializationBlock::NodeCountOutOfRange {
            entry_index,
            section,
            count,
        }
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_ling_bao_fields(destination: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}
