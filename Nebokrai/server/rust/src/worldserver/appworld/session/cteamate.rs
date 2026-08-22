//! Участник World-команды `CTeamate`.
//!
//! Все десять функций owner-а восстановлены по точной паре
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`: `SetOwnerRegionID`
//! RVA `0x000DEB80`, `OnChangeState` `0x000DEBB0`, `PlayerStillExisted`
//! `0x000DEBD0`, `IsPlugAvailable` `0x000DEBE0`, `GetOwnerName` `0x000DED00`,
//! `Serialize` `0x000DED20`, constructor/destructor `0x000DED90/0x000DEDD0`,
//! `SetOwnerName` `0x000DEE70`, `Unserialize` `0x000DEEA0`. Исходный owner:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\session\cteamate.cpp`.
//!
//! Constructor задаёт plug type `5`, region/timestamp `0`, existence `1` и
//! пустое byte-exact имя. `SetOwnerRegionID` сначала меняет поле, затем
//! синхронно публикует state `6`; safe Rust сохраняет это через немедленно
//! дренируемый factory effect базового `CPlug`. Проверка существования хранит
//! исходный минутный unsigned gate, повторные вызовы `timeGetTime` и пакет
//! `0x7FD09 [plug ID, owner type, owner ID]`; добавленные Linux-донором pending
//! map/peer validation и требование успешной отправки в EXE отсутствуют.
//!
//! Wire suffix — region `long`, имя и NUL после базового plug header. Owned
//! `Vec<u8>` заменяет `std::string`, не навязывает UTF-8 и обрезает setter по
//! первому NUL. Старый `_GetStringFromByteArray` писал в `char[256]`; safe
//! decoder принимает только найденный в этой границе NUL и возвращает `0` при
//! коротком/переполненном входе, сохраняя уже прочитанные поля и cursor.

use crate::nets::networld::message::CMessage;
use crate::worldserver::appworld::session::cplug::{CPlug, read_i32};
use crate::worldserver::appworld::session::csessionfactory::{
    WorldPlugOwner, WorldPlugSessionEffect, WorldTeamateOwner,
};
use crate::worldserver::worldserver::game::{CGame, legacy_tick_ms};

const TEAMATE_PLUG_TYPE: u32 = 5;
const EXISTENCE_QUERY_INTERVAL_MS: u32 = 60_000;

/// Конкретный factory-owned участник команды.
pub(crate) struct CTeamate {
    plug: CPlug,
    owner_region_id: i32,
    owner_name: Vec<u8>,
    last_queried_timestamp_ms: u32,
    player_still_existed: i32,
}

impl CTeamate {
    pub(crate) const fn new() -> Self {
        let mut plug = CPlug::new();
        plug.set_plug_type(TEAMATE_PLUG_TYPE);
        Self {
            plug,
            owner_region_id: 0,
            owner_name: Vec::new(),
            last_queried_timestamp_ms: 0,
            player_still_existed: 1,
        }
    }

    pub(crate) const fn owner_region_id(&self) -> i32 {
        self.owner_region_id
    }

    pub(crate) const fn object_id(&self) -> i32 {
        self.plug.object_id()
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.plug.owner_type()
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.plug.owner_id()
    }

    pub(crate) const fn set_session(&mut self, session_id: i32) {
        self.plug.set_session(session_id);
    }

    pub(crate) const fn is_plug_ended(&self) -> i32 {
        self.plug.is_plug_ended()
    }

    pub(crate) fn owner_name(&self) -> &[u8] {
        &self.owner_name
    }

    /// Выполняет точный минутный availability/probe lifecycle.
    pub(crate) fn is_plug_available(&mut self, game: &CGame) -> i32 {
        let now = legacy_tick_ms();
        if self.last_queried_timestamp_ms == 0 {
            self.player_still_existed = 1;
            self.last_queried_timestamp_ms = now;
            return 1;
        }
        if now <= self
            .last_queried_timestamp_ms
            .wrapping_add(EXISTENCE_QUERY_INTERVAL_MS)
        {
            return 1;
        }

        let result = if self.player_still_existed == 0 {
            0
        } else {
            if let Some(game_server) = game.player_game_server(self.plug.owner_id()) {
                let map_id = game_server.index as i32;
                self.player_still_existed = 0;
                let mut message = CMessage::new(0x0007_FD09);
                message.base_mut().add_long(self.plug.object_id());
                message.base_mut().add_long(self.plug.owner_type());
                message.base_mut().add_long(self.plug.owner_id());
                let _ = game.send_msg_to_game_server(map_id, &message);
            }
            1
        };
        self.last_queried_timestamp_ms = legacy_tick_ms();
        result
    }

    /// Исходный callback только пытается найти plug в той же session и
    /// независимо от результата возвращает `1`.
    pub(crate) const fn on_change_state(&self, _plug_id: i32) -> i32 {
        1
    }

    pub(crate) fn serialize(&self, output: &mut Vec<u8>) -> i32 {
        if self.plug.serialize(output) == 0 {
            return 0;
        }
        output.extend_from_slice(&self.owner_region_id.to_le_bytes());
        output.extend_from_slice(&self.owner_name);
        output.push(0);
        1
    }

    fn unserialize_suffix(&mut self, stream: &[u8], offset: &mut i32) -> i32 {
        if self.plug.unserialize(stream, offset) == 0 {
            return 0;
        }
        let Some(region_id) = read_i32(stream, offset) else {
            return 0;
        };
        self.owner_region_id = region_id;

        let attempted_offset = *offset;
        let Ok(start) = usize::try_from(attempted_offset) else {
            return 0;
        };
        let Some(legacy_end) = start.checked_add(0x100) else {
            return 0;
        };
        let search_end = stream.len().min(legacy_end);
        let Some(relative_nul) = stream[start..search_end]
            .iter()
            .position(|byte| *byte == 0)
        else {
            *offset = i32::try_from(search_end).unwrap_or(i32::MAX);
            return 0;
        };
        let end = start + relative_nul;
        self.owner_name.clear();
        self.owner_name.extend_from_slice(&stream[start..end]);
        let Some(next) = end.checked_add(1).and_then(|value| i32::try_from(value).ok()) else {
            return 0;
        };
        *offset = next;
        1
    }
}

impl WorldPlugOwner for CTeamate {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32) {
        self.plug.assign_factory_identity(object_type, object_id);
    }

    fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.plug.set_owner(owner_type, owner_id);
    }

    fn object_id(&self) -> i32 {
        self.object_id()
    }

    fn owner_type(&self) -> i32 {
        self.owner_type()
    }

    fn owner_id(&self) -> i32 {
        self.owner_id()
    }

    fn set_session(&mut self, session_id: i32) {
        self.set_session(session_id);
    }

    fn is_plug_available(&mut self, game: &CGame) -> i32 {
        self.is_plug_available(game)
    }

    fn is_plug_ended(&self) -> i32 {
        self.is_plug_ended()
    }

    fn on_change_state(&mut self, plug_id: i32, _state: i32, _value: &[u8]) -> i32 {
        CTeamate::on_change_state(self, plug_id)
    }

    fn serialize(&self, output: &mut Vec<u8>) -> i32 {
        CTeamate::serialize(self, output)
    }

    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32 {
        self.unserialize_suffix(stream, offset)
    }

    fn as_teamate_mut(&mut self) -> Option<&mut dyn WorldTeamateOwner> {
        Some(self)
    }
}

impl WorldTeamateOwner for CTeamate {
    fn exit(&mut self) {
        self.plug.exit();
    }

    fn set_owner_region_id(&mut self, region_id: i32) {
        self.owner_region_id = region_id;
        self.plug.change_state(6, &region_id.to_le_bytes());
    }

    fn set_owner_name(&mut self, owner_name: &[u8]) {
        let prefix_length = owner_name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(owner_name.len());
        self.owner_name.clear();
        self.owner_name
            .extend_from_slice(&owner_name[..prefix_length]);
    }

    fn player_still_existed(&mut self, existed: i32) {
        self.player_still_existed = existed;
    }

    fn owner_region_id(&self) -> i32 {
        self.owner_region_id
    }

    fn owner_name(&self) -> &[u8] {
        &self.owner_name
    }

    fn take_session_effects(&mut self) -> Vec<WorldPlugSessionEffect> {
        self.plug.take_session_effects()
    }

    fn confirm_exit(&mut self) {
        self.plug.confirm_exit();
    }
}
