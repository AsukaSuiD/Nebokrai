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
//! `LegacyReader` и `LegacyWriter` поверх `bytes` отвечают только за границы,
//! курсор и little-endian primitives; signed count и partial prefix сохранены.
//! Reached `6051 / RequestPlayerRanks` вызывается живым `CScript` через
//! `CGame`: NPC/player gate и аргумент остаются у script owner-а, а этот owner
//! получает единый process tick, применяет cooldown и отправляет `0xBFF30`.

use std::collections::{BTreeMap, TryReserveError};
use thiserror::Error;

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
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
        let mut writer = LegacyWriter::new(destination);
        writer.write_u32(declared);
        for rank in &self.ranks {
            writer.write_i32(rank.player_id);
            writer.write_c_string(&rank.name);
            writer.write_u16(rank.occupation);
            writer.write_u16(rank.level);
            writer.write_c_string(&rank.faction_name);
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

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum PlayerRanksSerializeError {
    #[error("число Game player ranks не представимо unsigned long")]
    CountOutsideLegacyRange,
}

#[derive(Debug, Error)]
pub(crate) enum PlayerRanksDecodeError {
    #[error("player ranks обрывается на {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("player ranks string с {offset} не завершена нулём")]
    MissingStringTerminator {
        offset: usize,
    },
    #[error("не удалось выделить Game player ranks")]
    Allocation(#[source] TryReserveError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksRequestOutcome {
    SuppressedByCooldown,
    Sent { queue_result: i32 },
}

fn read_i32(source: &[u8], cursor: &mut usize) -> Result<i32, PlayerRanksDecodeError> {
    let mut reader = rank_reader(source, *cursor, 4)?;
    let value = reader.read_i32().map_err(map_rank_block)?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u16(source: &[u8], cursor: &mut usize) -> Result<u16, PlayerRanksDecodeError> {
    let mut reader = rank_reader(source, *cursor, 2)?;
    let value = reader.read_u16().map_err(map_rank_block)?;
    *cursor = reader.position();
    Ok(value)
}

fn rank_reader(
    source: &[u8],
    cursor: usize,
    needed: usize,
) -> Result<LegacyReader<'_>, PlayerRanksDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| PlayerRanksDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn map_rank_block(block: LegacyReadBlock) -> PlayerRanksDecodeError {
    PlayerRanksDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

fn read_legacy_c_string(
    source: &[u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, PlayerRanksDecodeError> {
    let offset = *cursor;
    let mut reader = rank_reader(source, offset, 1)?;
    let maximum = reader.remaining();
    let value = reader
        .read_c_string(maximum)
        .map_err(|_| PlayerRanksDecodeError::MissingStringTerminator { offset })?;
    *cursor = reader.position();
    Ok(value.to_vec())
}
