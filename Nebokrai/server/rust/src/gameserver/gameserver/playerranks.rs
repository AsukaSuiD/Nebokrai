//! Рейтинг игроков исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `gameserver/playerranks.cpp`, подтверждает list insertion-order, signed
//! player IDs, NUL-terminated names, 16-битные occupation/level и map cooldown
//! по `timeGetTime`. `Initialize` безусловно успешен; process singleton заменён
//! прямым владением `CGame`.
//!
//! `Vec` и `BTreeMap` заменяют MSVC list/tree plumbing. Decoder очищает старый
//! список до чтения count и на safe malformed input сохраняет уже прочитанный
//! prefix. Двухсекундный cooldown использует wrapping `u32` timestamps и удаляет
//! expired keys в sorted order. Compatibility quirk serializer-а сохранён:
//! ограниченный declared count не ограничивает фактический обход rank list.

use std::collections::{BTreeMap, TryReserveError};
use std::error::Error;
use std::fmt;

use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::mynetserver::CMyNetServer;

const PLAYER_RANKS_MESSAGE: i32 = 0x000B_FF30;
const REQUEST_COOLDOWN_MS: u32 = 2_000;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerRankEntry {
    pub(crate) player_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) occupation: u16,
    pub(crate) level: u16,
    pub(crate) faction_name: Vec<u8>,
}

#[derive(Debug, Default)]
pub(crate) struct CPlayerRanks {
    ranks: Vec<PlayerRankEntry>,
    request_expirations_ms: BTreeMap<i32, u32>,
}

impl CPlayerRanks {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn initialize(&mut self) -> bool {
        true
    }

    /// Сохраняет quirk exact EXE: header count может быть меньше числа реально
    /// записанных records, потому что исходный loop не использовал limit.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        maximum_rank_count: i32,
    ) -> Result<(), PlayerRanksSerializeError> {
        let total = u32::try_from(self.ranks.len())
            .map_err(|_| PlayerRanksSerializeError::CountOutsideLegacyRange)?;
        let declared = total.min(maximum_rank_count as u32);
        destination.extend_from_slice(&declared.to_le_bytes());
        for rank in &self.ranks {
            destination.extend_from_slice(&rank.player_id.to_le_bytes());
            append_legacy_c_string(destination, &rank.name);
            destination.extend_from_slice(&rank.occupation.to_le_bytes());
            destination.extend_from_slice(&rank.level.to_le_bytes());
            append_legacy_c_string(destination, &rank.faction_name);
        }
        Ok(())
    }

    pub(crate) fn get_specify_player_rank(&self, player_id: u32) -> u32 {
        self.ranks
            .iter()
            .position(|rank| rank.player_id as u32 == player_id)
            .and_then(|index| u32::try_from(index + 1).ok())
            .unwrap_or(0)
    }

    /// Очищает прежний list немедленно и сохраняет успешно decoded prefix.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerRanksDecodeError> {
        self.ranks.clear();
        let count = read_i32(source, cursor)?;
        if count <= 0 {
            return Ok(());
        }
        self.ranks
            .try_reserve(count as usize)
            .map_err(PlayerRanksDecodeError::Allocation)?;
        for _ in 0..count {
            let player_id = read_i32(source, cursor)?;
            let name = read_legacy_c_string(source, cursor)?;
            let occupation = read_u16(source, cursor)?;
            let level = read_u16(source, cursor)?;
            let faction_name = read_legacy_c_string(source, cursor)?;
            self.ranks.push(PlayerRankEntry {
                player_id,
                name,
                occupation,
                level,
                faction_name,
            });
        }
        Ok(())
    }

    pub(crate) fn on_player_get_ranks(
        &mut self,
        player_id: i32,
        maximum_rank_count: i32,
        now_ms: u32,
        net_server: &CMyNetServer,
    ) -> Result<PlayerRanksRequestOutcome, PlayerRanksSerializeError> {
        let mut suppressed = false;
        self.request_expirations_ms
            .retain(|stored_player_id, expiry| {
                if now_ms < *expiry {
                    if *stored_player_id == player_id {
                        suppressed = true;
                    }
                    true
                } else {
                    false
                }
            });
        if suppressed {
            return Ok(PlayerRanksRequestOutcome::SuppressedByCooldown);
        }

        let mut payload = Vec::new();
        self.add_to_byte_array(&mut payload, maximum_rank_count)?;
        let mut message = CMessage::new(PLAYER_RANKS_MESSAGE);
        message.base_mut().add(&payload);
        let queue_result = message.send_to_player(net_server, player_id);
        self.request_expirations_ms
            .insert(player_id, now_ms.wrapping_add(REQUEST_COOLDOWN_MS));
        Ok(PlayerRanksRequestOutcome::Sent { queue_result })
    }

    pub(crate) fn ranks(&self) -> &[PlayerRankEntry] {
        &self.ranks
    }

    pub(crate) fn request_expirations(&self) -> &BTreeMap<i32, u32> {
        &self.request_expirations_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksSerializeError {
    CountOutsideLegacyRange,
}

impl fmt::Display for PlayerRanksSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("число Game player ranks не представимо unsigned long")
    }
}

impl Error for PlayerRanksSerializeError {}

#[derive(Debug)]
pub(crate) enum PlayerRanksDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    MissingStringTerminator {
        offset: usize,
    },
    Allocation(TryReserveError),
}

impl fmt::Display for PlayerRanksDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "player ranks обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::MissingStringTerminator { offset } => {
                write!(
                    formatter,
                    "player ranks string с {offset} не завершена нулём"
                )
            }
            Self::Allocation(_) => formatter.write_str("не удалось выделить Game player ranks"),
        }
    }
}

impl Error for PlayerRanksDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksRequestOutcome {
    SuppressedByCooldown,
    Sent { queue_result: i32 },
}

fn append_legacy_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let prefix = value
        .iter()
        .position(|byte| *byte == 0)
        .map_or(value, |end| &value[..end]);
    destination.extend_from_slice(prefix);
    destination.push(0);
}

fn read_i32(source: &[u8], cursor: &mut usize) -> Result<i32, PlayerRanksDecodeError> {
    Ok(i32::from_le_bytes(read_array(source, cursor)?))
}

fn read_u16(source: &[u8], cursor: &mut usize) -> Result<u16, PlayerRanksDecodeError> {
    Ok(u16::from_le_bytes(read_array(source, cursor)?))
}

fn read_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], PlayerRanksDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(PlayerRanksDecodeError::UnexpectedEnd {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes.try_into().expect("размер rank scalar уже проверен"))
}

fn read_legacy_c_string(
    source: &[u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, PlayerRanksDecodeError> {
    let offset = *cursor;
    let remaining = source
        .get(offset..)
        .ok_or(PlayerRanksDecodeError::UnexpectedEnd {
            offset,
            needed: 1,
            available: 0,
        })?;
    let Some(length) = remaining.iter().position(|byte| *byte == 0) else {
        return Err(PlayerRanksDecodeError::MissingStringTerminator { offset });
    };
    *cursor += length + 1;
    Ok(remaining[..length].to_vec())
}
