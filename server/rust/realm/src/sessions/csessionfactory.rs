//! Фабрика сессий `CSessionFactory` из WorldServer, перенесённая в Realm
//! `sessions/`; подтверждённая `worldserver.exe` и `worldserver.pdb`.
//!
//! Registry хранит sessions/plugs по signed ID и создаёт `CSession`, `CTeam`,
//! `CTeamate` и шесть plug types. `AI` удаляет null, abort-ит недоступную,
//! end-ит завершённую session и только иначе вызывает её `AI`.
//!
//! Порядок MSVC hash-map наблюдаем через callbacks, поэтому узкий registry
//! воспроизводит bucket `key ^ 0xDEADBEEF` и signed key order. Duplicate ID
//! заменяет owner; Rust освобождает вытесненный объект вместо утечки.
//!
//! Unserialize сначала wrapping увеличивает offset, читает ID, создаёт owner
//! и делегирует virtual body; false удаляет созданный объект. Null stream даёт
//! ноль без изменения offset, короткий input сохраняет уже выполненный сдвиг.
//! В командном plug-цикле отказ любого plug-а валит unserialize всей сессии:
//! созданная сессия уничтожается GC, а запись `team_id → session_id` в карте
//! CGame не откатывается (зомби-запись оригинала, см. `unserialize_session`).
//! Машинное основание таких особенностей — точная пара `Nworldserver.exe`
//! (`F3AC454D…`) + `WorldServer.pdb` (RSDS match).

use std::error::Error;
use std::fmt;

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::CMessage;
use crate::sessions::csession::{WorldSessionEffect, WorldSessionPlug};
use crate::sessions::cteam::CTeam;
use crate::sessions::cteamate::CTeamate;

const LEGACY_HASH_XOR: u32 = 0xDEAD_BEEF;
const TYPE_SESSION: i32 = 10;
const TYPE_PLUG: i32 = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum WorldSessionType {
    Normal = 0,
    Team = 1,
}

impl WorldSessionType {
    const fn from_legacy(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Team),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum WorldPlugType {
    Normal = 0,
    Trader = 1,
    PersonalShopSeller = 2,
    PersonalShopBuyer = 3,
    EquipmentUpgrade = 4,
    Teamate = 5,
}

impl WorldPlugType {
    const fn from_legacy(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Trader),
            2 => Some(Self::PersonalShopSeller),
            3 => Some(Self::PersonalShopBuyer),
            4 => Some(Self::EquipmentUpgrade),
            5 => Some(Self::Teamate),
            _ => None,
        }
    }
}

pub trait WorldSessionOwner {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32);
    fn is_session_available(&mut self) -> i32;
    fn abort(&mut self) -> i32;
    fn is_session_ended(&mut self) -> i32;
    fn end(&mut self) -> i32;
    fn ai(&mut self);
    fn ai_prefix(&mut self) {
        self.ai();
    }
    fn ai_suffix(&mut self) {}
    fn insert_plug(&mut self, plug_id: i32) -> i32;
    fn on_plug_change_state(
        &mut self,
        plug_id: i32,
        state: i32,
        value: &[u8],
        recursive: i32,
    ) -> i32;
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;
    fn take_effects(&mut self) -> Vec<WorldSessionEffect> {
        Vec::new()
    }
    fn take_pending_unserialize_plugs(&mut self) -> u32 {
        0
    }
    fn insert_plug_identity(&mut self, _plug: WorldSessionPlug) {}
    fn can_insert_plug(&self) -> bool {
        false
    }
    fn session_plugs(&self) -> &[WorldSessionPlug] {
        &[]
    }
    fn remove_plug_id(&mut self, _plug_id: i32) {}
    fn should_traverse_plugs(&self) -> bool {
        false
    }

    fn as_team_mut(&mut self) -> Option<&mut dyn WorldTeamSessionOwner> {
        None
    }
}

pub trait WorldTeamSessionOwner {
    fn query_plug_by_owner(&mut self, owner_type: i32, owner_id: i32) -> Option<i32>;
    fn set_leader(&mut self, player_id: i32);
    fn kick_player(&mut self, player_id: i32);
    fn serialize(&mut self, output: &mut Vec<u8>);
    fn allocation_scheme(&mut self) -> i32;
    fn set_allocation_scheme(&mut self, scheme: i32);
    fn on_plug_change_state(&mut self, plug_id: i32, state: i32, value: &[u8]);
}

pub trait WorldPlugOwner {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32);
    fn set_owner(&mut self, owner_type: i32, owner_id: i32);
    fn object_id(&self) -> i32;
    fn owner_type(&self) -> i32;
    fn owner_id(&self) -> i32;
    fn set_session(&mut self, session_id: i32);
    fn is_plug_available(&mut self, game: &dyn WorldGameView) -> i32;
    fn is_plug_ended(&self) -> i32;
    fn on_change_state(&mut self, plug_id: i32, state: i32, value: &[u8]) -> i32;
    fn serialize(&self, output: &mut Vec<u8>) -> i32;
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;

    fn as_teamate_mut(&mut self) -> Option<&mut dyn WorldTeamateOwner> {
        None
    }
}

pub trait WorldTeamateOwner {
    fn exit(&mut self);
    fn set_owner_region_id(&mut self, region_id: i32);
    fn set_owner_name(&mut self, owner_name: &[u8]);
    fn player_still_existed(&mut self, existed: i32);
    fn owner_region_id(&self) -> i32;
    fn owner_name(&self) -> &[u8];

    fn take_session_effects(&mut self) -> Vec<WorldPlugSessionEffect> {
        Vec::new()
    }

    fn confirm_exit(&mut self) {}
}

pub struct WorldPlugSessionEffect {
    pub session_id: i32,
    pub plug_id: i32,
    pub state: i32,
    pub value: Vec<u8>,
    pub end_after_session_lookup: bool,
}

struct LegacyMsvcHashEntry<T> {
    key: i32,
    value: Option<T>,
}

struct LegacyMsvcHashRegistry<T> {
    entries: Vec<LegacyMsvcHashEntry<T>>,
    mask: u32,
    max_index: u32,
    bucket_vector_slots: u32,
}

impl<T> LegacyMsvcHashRegistry<T> {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            mask: 1,
            max_index: 1,
            bucket_vector_slots: 9,
        }
    }

    fn position(&self, key: i32) -> Option<usize> {
        self.entries.iter().position(|entry| entry.key == key)
    }

    fn get(&self, key: i32) -> Option<&T> {
        self.position(key)
            .and_then(|index| self.entries[index].value.as_ref())
    }

    fn get_mut(&mut self, key: i32) -> Option<&mut T> {
        let index = self.position(key)?;
        self.entries[index].value.as_mut()
    }

    fn insert_or_replace(&mut self, key: i32, value: Option<T>) -> Option<T> {
        if let Some(index) = self.position(key) {
            return std::mem::replace(&mut self.entries[index].value, value);
        }

        self.grow_before_insert_if_needed();
        let order = self.order_key(key);
        let index = self
            .entries
            .partition_point(|entry| self.order_key(entry.key) < order);
        self.entries
            .insert(index, LegacyMsvcHashEntry { key, value });
        None
    }

    fn grow_before_insert_if_needed(&mut self) {
        if self.max_index > (self.entries.len() as u32) >> 2 {
            return;
        }

        if self.max_index < self.bucket_vector_slots - 1 {
            if self.mask < self.max_index {
                self.mask = self.mask.wrapping_mul(2).wrapping_add(1);
            }
        } else {
            self.mask = self.bucket_vector_slots.wrapping_mul(2).wrapping_sub(3);
            self.bucket_vector_slots = self.bucket_vector_slots.wrapping_mul(2).wrapping_sub(1);
        }
        self.max_index = self.max_index.wrapping_add(1);

        let mask = self.mask;
        let max_index = self.max_index;
        self.entries
            .sort_by_key(|entry| (legacy_bucket(entry.key, mask, max_index), entry.key));
    }

    fn order_key(&self, key: i32) -> (u32, i32) {
        (legacy_bucket(key, self.mask, self.max_index), key)
    }
}

fn legacy_bucket(key: i32, mask: u32, max_index: u32) -> u32 {
    let mut bucket = ((key as u32) ^ LEGACY_HASH_XOR) & mask;
    if max_index <= bucket {
        bucket = bucket.wrapping_sub((mask >> 1).wrapping_add(1));
    }
    bucket
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSessionFactoryAiEvent {
    NullRemoved { session_id: i32 },
    UnavailableRemoved { session_id: i32, abort_result: i32 },
    EndedRemoved { session_id: i32, end_result: i32 },
    Ran { session_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldSessionFactoryAiReport {
    pub events: Vec<WorldSessionFactoryAiEvent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSessionFactoryGarbageCollect {
    UnknownObjectType,
    Missing,
    Removed {
        object_type: i32,
        object_id: i32,
        had_owner: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldSessionFactoryInputBlock {
    pub attempted_offset: i32,
    pub assigned_offset: i32,
    pub stream_length: usize,
}

impl fmt::Display for WorldSessionFactoryInputBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSessionFactory читает DWORD с offset {}, после присваивания {}; длина stream {} не покрывает старый unchecked read",
            self.attempted_offset, self.assigned_offset, self.stream_length
        )
    }
}

impl Error for WorldSessionFactoryInputBlock {}

pub struct CSessionFactory {
    next_session_id: i32,
    next_plug_id: i32,
    sessions: LegacyMsvcHashRegistry<Box<dyn WorldSessionOwner>>,
    plugs: LegacyMsvcHashRegistry<Box<dyn WorldPlugOwner>>,
}

impl CSessionFactory {
    pub const fn new() -> Self {
        Self {
            next_session_id: 1,
            next_plug_id: 1,
            sessions: LegacyMsvcHashRegistry::new(),
            plugs: LegacyMsvcHashRegistry::new(),
        }
    }

    pub fn query_session(&self, session_id: i32) -> Option<&dyn WorldSessionOwner> {
        self.sessions
            .get(session_id)
            .map(|session| session.as_ref())
    }

    pub fn query_plug(&self, plug_id: i32) -> Option<&dyn WorldPlugOwner> {
        self.plugs.get(plug_id).map(|plug| plug.as_ref())
    }

    pub fn insert_plug(&mut self, game: &mut dyn WorldGameView, session_id: i32, plug_id: i32) -> i32 {
        let Some(plug) = self.plugs.get_mut(plug_id) else {
            return 0;
        };
        let identity = WorldSessionPlug {
            plug_id,
            owner_type: plug.owner_type(),
            owner_id: plug.owner_id(),
        };
        let Some(session) = self.sessions.get_mut(session_id) else {
            return 0;
        };
        if !session.can_insert_plug() || plug.is_plug_available(game) == 0 {
            return 0;
        }
        plug.set_session(session_id);
        session.insert_plug_identity(identity);
        let _ = session.on_plug_change_state(plug_id, 0, &[], 0);
        self.drain_session_effects(game, session_id);
        1
    }

    pub fn is_team(&mut self, session_id: i32) -> bool {
        self.sessions
            .get_mut(session_id)
            .and_then(|session| session.as_team_mut())
            .is_some()
    }

    // Документированная граница (достижимость UNKNOWN, намеренно не чинить):
    // в машине arm `0x60008` диспетчера игнорирует результат virtual Serialize
    // (`0x4AB25A`, eax не тестируется) и отправляет `0x7FD08` даже с частично
    // заполненным буфером; Rust при plug-miss / serialize == 0 не шлёт вовсе.
    // Отказ здесь недостижим в перенесённых потоках — GC всегда идёт с unlink,
    // поэтому plug-список сессии не содержит висячих ID; расхождение оставлено.
    pub fn serialize_team(&mut self, session_id: i32) -> Option<Vec<u8>> {
        let (mut output, plug_ids) = {
            let session = self.sessions.get_mut(session_id)?;
            let plug_ids = session
                .session_plugs()
                .iter()
                .map(|plug| plug.plug_id)
                .collect::<Vec<_>>();
            let mut output = Vec::new();
            session.as_team_mut()?.serialize(&mut output);
            (output, plug_ids)
        };
        for plug_id in plug_ids {
            let plug = self.plugs.get(plug_id)?;
            if plug.serialize(&mut output) == 0 {
                return None;
            }
        }
        Some(output)
    }

    pub fn with_team<ResultValue>(
        &mut self,
        game: &mut dyn WorldGameView,
        session_id: i32,
        operation: impl FnOnce(&mut dyn WorldTeamSessionOwner) -> ResultValue,
    ) -> Option<ResultValue> {
        let result = self.sessions.get_mut(session_id)?.as_team_mut().map(operation)?;
        self.drain_session_effects(game, session_id);
        Some(result)
    }

    pub fn with_teamate<ResultValue>(
        &mut self,
        game: &mut dyn WorldGameView,
        plug_id: i32,
        operation: impl FnOnce(&mut dyn WorldTeamateOwner) -> ResultValue,
    ) -> Option<ResultValue> {
        let (result, effects) = {
            let teamate = self.plugs.get_mut(plug_id)?.as_teamate_mut()?;
            let result = operation(teamate);
            (result, teamate.take_session_effects())
        };
        let mut affected_sessions = Vec::new();
        for effect in effects {
            affected_sessions.push(effect.session_id);
            let session_found = self.sessions.get_mut(effect.session_id).is_some_and(|session| {
                session.on_plug_change_state(
                    effect.plug_id,
                    effect.state,
                    &effect.value,
                    0,
                );
                true
            });
            if session_found && effect.end_after_session_lookup {
                if let Some(teamate) = self
                    .plugs
                    .get_mut(plug_id)
                    .and_then(|plug| plug.as_teamate_mut())
                {
                    teamate.confirm_exit();
                }
            }
        }
        affected_sessions.sort_unstable();
        affected_sessions.dedup();
        for session_id in affected_sessions {
            self.drain_session_effects(game, session_id);
        }
        Some(result)
    }

    pub fn end_session(&mut self, game: &mut dyn WorldGameView, session_id: i32) -> Option<i32> {
        let result = self.sessions.get_mut(session_id).map(|session| session.end())?;
        self.drain_session_effects(game, session_id);
        Some(result)
    }

    fn drain_session_effects(&mut self, game: &mut dyn WorldGameView, session_id: i32) {
        loop {
            let effects = self
                .sessions
                .get_mut(session_id)
                .map(|session| session.take_effects())
                .unwrap_or_default();
            if effects.is_empty() {
                break;
            }
            for effect in effects {
                match effect {
                    WorldSessionEffect::TeamStarted { team_id } => {
                        game.publish_team_session(team_id, session_id);
                    }
                    WorldSessionEffect::TeamEnded { team_id } => {
                        let mut message = CMessage::new(0x0007_FD02);
                        message.base_mut().add_long(team_id as i32);
                        let _ = message.send_all(game.current_game_server_sender().as_ref());
                        game.remove_team_session(team_id);
                    }
                    WorldSessionEffect::KickPlayer { team_id, player_id } => {
                        if let Some(map_id) = game
                            .player_game_server(player_id)
                            .map(|server| server.index as i32)
                        {
                            let mut message = CMessage::new(0x0007_FD06);
                            message.base_mut().add_long(team_id as i32);
                            message.base_mut().add_long(player_id);
                            let _ = game.send_msg_to_game_server(map_id, &message);
                        }
                    }
                    WorldSessionEffect::PlugState {
                        team_id,
                        plug_id,
                        state,
                        value,
                        include_sender,
                    } => self.dispatch_team_plug_state(
                        game,
                        session_id,
                        team_id,
                        plug_id,
                        state,
                        &value,
                        include_sender,
                    ),
                }
            }
        }
    }

    fn dispatch_team_plug_state(
        &mut self,
        game: &dyn WorldGameView,
        session_id: i32,
        team_id: u32,
        plug_id: i32,
        state: i32,
        value: &[u8],
        include_sender: bool,
    ) {
        let plug_ids: Vec<i32> = self
            .sessions
            .get(session_id)
            .map(|session| {
                session
                    .session_plugs()
                    .iter()
                    .map(|plug| plug.plug_id)
                    .collect()
            })
            .unwrap_or_default();
        // Членство source в списке сессии моделирует сеансовый `QueryPlugByID`
        // (`0x4DD910`) + RTTI-guard, которые `CTeam::OnPlugChangeState`
        // (`0x4DE3C0`) применяет в ветвях state 0/1/2/6/8/9. State 5
        // (`0x7FD0A`, SetAllocationScheme) — исключение: ветвь `0x4DE417`
        // довольствуется base-guard `0x4DE3F2` → `CSession::OnPlugChangeState`
        // (`0x4DD810`: только глобальный QueryPlug + IsPlugAvailable) и
        // членства не требует, поэтому при числовом совпадении quirk-ового
        // player leader ID с чужим plug ID машина доставляет; ниже guard
        // global-exists + available сохраняется, exclusion по source map тоже.
        // Машинное основание: точная пара `Nworldserver.exe` (`F3AC454D…`) +
        // `WorldServer.pdb` (RSDS match).
        if state != 5 && !plug_ids.contains(&plug_id) {
            return;
        }
        let Some(source) = self.plugs.get_mut(plug_id) else {
            return;
        };
        if source.is_plug_available(game) == 0 {
            return;
        }

        for target_id in plug_ids.iter().copied() {
            if target_id == plug_id && !include_sender {
                continue;
            }
            let Some(target) = self.plugs.get_mut(target_id) else {
                continue;
            };
            if target.is_plug_available(game) != 0 && target.is_plug_ended() == 0 {
                let _ = target.on_change_state(plug_id, state, value);
            }
        }

        let source_identity = self.plugs.get(plug_id).map(|plug| {
            (plug.owner_type(), plug.owner_id())
        });
        match state {
            0 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                let Some((region_id, owner_name)) = self.with_teamate_snapshot(plug_id) else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD03);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(region_id);
                message.base_mut().add(&owner_name);
                message.base_mut().add_byte(0);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            1 | 2 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD04);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            4 => {
                let Some(player_id) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let player_exists = self.sessions.get(session_id).is_some_and(|session| {
                    session
                        .session_plugs()
                        .iter()
                        .any(|plug| plug.owner_type == 400 && plug.owner_id == player_id)
                });
                if player_id == 0 || !player_exists {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD07);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(player_id);
                self.send_to_team_game_servers(
                    game,
                    &plug_ids,
                    if include_sender { 0 } else { plug_id },
                    &message,
                );
            }
            5 => {
                let Some(scheme) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD0A);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(scheme);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            6 => {
                let Some(region_id) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if region_id == 0 || self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD05);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(region_id);
                self.send_to_team_game_servers(game, &plug_ids, -1, &message);
            }
            8 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD0B);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add(value);
                message.base_mut().add_byte(0);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            9 => {
                let Some(state_bits) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD0C);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(state_bits);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            _ => {}
        }
    }

    fn with_teamate_snapshot(&mut self, plug_id: i32) -> Option<(i32, Vec<u8>)> {
        let teamate = self.plugs.get_mut(plug_id)?.as_teamate_mut()?;
        let region_id = teamate.owner_region_id();
        let owner_name = teamate.owner_name().to_vec();
        Some((region_id, owner_name))
    }

    fn send_to_team_game_servers(
        &self,
        game: &dyn WorldGameView,
        plug_ids: &[i32],
        source_plug_id: i32,
        message: &CMessage,
    ) {
        let source_map = self
            .plugs
            .get(source_plug_id)
            .and_then(|plug| game.player_game_server(plug.owner_id()))
            .map(|server| server.index);
        let mut maps = Vec::new();
        for plug_id in plug_ids.iter().copied() {
            if plug_id == source_plug_id {
                continue;
            }
            let Some(map_id) = self
                .plugs
                .get(plug_id)
                .and_then(|plug| game.player_game_server(plug.owner_id()))
                .map(|server| server.index)
            else {
                continue;
            };
            if Some(map_id) == source_map || maps.contains(&map_id) {
                continue;
            }
            maps.push(map_id);
            let _ = game.send_msg_to_game_server(map_id as i32, message);
        }
    }

    pub fn create_session(
        &mut self,
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
        session_type: i32,
    ) -> i32 {
        let Some(session_type) = WorldSessionType::from_legacy(session_type) else {
            return 0;
        };
        let mut session: Box<dyn WorldSessionOwner> = match session_type {
            WorldSessionType::Normal => Box::new(
                crate::sessions::csession::CSession::new(
                    minimum_plugs,
                    maximum_plugs,
                    lifetime_ms,
                ),
            ),
            WorldSessionType::Team => Box::new(CTeam::new(
                minimum_plugs,
                maximum_plugs,
                lifetime_ms,
            )),
        };

        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        session.assign_factory_identity(TYPE_SESSION, session_id);
        drop(self.sessions.insert_or_replace(session_id, Some(session)));
        session_id
    }

    pub fn create_plug(
        &mut self,
        plug_type: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> i32 {
        let Some(plug_type) = WorldPlugType::from_legacy(plug_type) else {
            return 0;
        };
        if plug_type != WorldPlugType::Teamate {
            return 0;
        }
        let mut plug: Box<dyn WorldPlugOwner> = Box::new(CTeamate::new());

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        plug.assign_factory_identity(TYPE_PLUG, plug_id);
        plug.set_owner(owner_type, owner_id);
        drop(self.plugs.insert_or_replace(plug_id, Some(plug)));
        plug_id
    }

    pub fn ai(&mut self, game: &mut dyn WorldGameView) -> WorldSessionFactoryAiReport {
        let mut events = Vec::new();
        let mut index = 0;
        while index < self.sessions.entries.len() {
            let session_id = self.sessions.entries[index].key;
            if self.sessions.entries[index].value.is_none() {
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::NullRemoved { session_id });
                continue;
            }

            let available = self.sessions.entries[index]
                .value
                .as_mut()
                .expect("null session отсеян до virtual-вызова")
                .is_session_available();
            if available == 0 {
                let abort_result = self.sessions.entries[index]
                    .value
                    .as_mut()
                    .expect("session остаётся живой до Abort")
                    .abort();
                self.drain_session_effects(game, session_id);
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                }
                let session = self.sessions.entries[index]
                    .value
                    .take()
                    .expect("session остаётся живой до deleting destructor");
                drop(session);
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::UnavailableRemoved {
                    session_id,
                    abort_result,
                });
                continue;
            }

            let ended = self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой после availability")
                .is_session_ended();
            if ended != 0 {
                let end_result = self.sessions.entries[index]
                    .value
                    .as_mut()
                    .expect("session остаётся живой до End")
                    .end();
                self.drain_session_effects(game, session_id);
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                }
                let session = self.sessions.entries[index]
                    .value
                    .take()
                    .expect("session остаётся живой до deleting destructor");
                drop(session);
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::EndedRemoved {
                    session_id,
                    end_result,
                });
                continue;
            }

            self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой до AI prefix")
                .ai_prefix();
            self.drain_session_effects(game, session_id);

            let traverse = self.sessions.entries[index]
                .value
                .as_ref()
                .is_some_and(|session| session.should_traverse_plugs());
            if traverse {
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let disposition = match self.plugs.get_mut(plug_id) {
                        None => 0,
                        Some(plug) => {
                            if plug.is_plug_available(game) == 0 {
                                2
                            } else if plug.is_plug_ended() != 0 {
                                1
                            } else {
                                3
                            }
                        }
                    };
                    match disposition {
                        0 => {
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                session.remove_plug_id(plug_id);
                            }
                        }
                        state @ (1 | 2) => {
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                let _ = session.on_plug_change_state(plug_id, state, &[], 0);
                            }
                            self.drain_session_effects(game, session_id);
                            let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                session.remove_plug_id(plug_id);
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой до AI suffix")
                .ai_suffix();
            self.drain_session_effects(game, session_id);
            events.push(WorldSessionFactoryAiEvent::Ran { session_id });
            index += 1;
        }
        WorldSessionFactoryAiReport { events }
    }

    pub fn garbage_collect(
        &mut self,
        object_type: i32,
        object_id: i32,
    ) -> WorldSessionFactoryGarbageCollect {
        let had_owner = match object_type {
            TYPE_SESSION => {
                let Some(index) = self.sessions.position(object_id) else {
                    return WorldSessionFactoryGarbageCollect::Missing;
                };
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let Some(plug_index) = self.plugs.position(plug_id) else {
                        continue;
                    };
                    let plug = self.plugs.entries[plug_index].value.take();
                    drop(plug);
                    self.plugs.entries.remove(plug_index);
                }
                let owner = self.sessions.entries[index].value.take();
                let had_owner = owner.is_some();
                drop(owner);
                self.sessions.entries.remove(index);
                had_owner
            }
            TYPE_PLUG => {
                let Some(index) = self.plugs.position(object_id) else {
                    return WorldSessionFactoryGarbageCollect::Missing;
                };
                let owner = self.plugs.entries[index].value.take();
                let had_owner = owner.is_some();
                drop(owner);
                self.plugs.entries.remove(index);
                had_owner
            }
            _ => return WorldSessionFactoryGarbageCollect::UnknownObjectType,
        };
        WorldSessionFactoryGarbageCollect::Removed {
            object_type,
            object_id,
            had_owner,
        }
    }

    pub fn unserialize_plug(
        &mut self,
        stream: Option<&[u8]>,
        offset: &mut i32,
    ) -> Result<i32, WorldSessionFactoryInputBlock> {
        let Some(stream) = stream else {
            return Ok(0);
        };
        let plug_type = read_i32(stream, offset)?;
        let owner_type = read_i32(stream, offset)?;
        let owner_id = read_i32(stream, offset)?;
        let plug_id = self.create_plug(plug_type, owner_type, owner_id);
        let Some(plug) = self.plugs.get_mut(plug_id) else {
            return Ok(0);
        };
        if plug.unserialize(stream, offset) != 0 {
            return Ok(plug_id);
        }
        let _ = self.garbage_collect(TYPE_PLUG, plug_id);
        Ok(0)
    }

    pub fn unserialize_session(
        &mut self,
        game: &mut dyn WorldGameView,
        stream: Option<&[u8]>,
        offset: &mut i32,
    ) -> Result<i32, WorldSessionFactoryInputBlock> {
        let Some(stream) = stream else {
            return Ok(0);
        };
        let session_type = read_i32(stream, offset)?;
        let minimum_plugs = read_u32(stream, offset)?;
        let maximum_plugs = read_u32(stream, offset)?;
        let lifetime_ms = read_u32(stream, offset)?;
        let session_id = self.create_session(
            minimum_plugs,
            maximum_plugs,
            lifetime_ms,
            session_type,
        );
        let (unserialized, plug_count) = {
            let Some(session) = self.sessions.get_mut(session_id) else {
                return Ok(0);
            };
            let unserialized = session.unserialize(stream, offset);
            let plug_count = session.take_pending_unserialize_plugs();
            (unserialized, plug_count)
        };
        if unserialized != 0 {
            self.drain_session_effects(game, session_id);
            for _ in 0..plug_count {
                let plug_id = match self.unserialize_plug(Some(stream), offset) {
                    Ok(plug_id) => plug_id,
                    Err(block) => {
                        // Спутник отказа plug-а в bounded-окне короткого input
                        // (машина читала бы unchecked): созданная сессия
                        // разрушается тем же GC, что и при чистом отказе ниже;
                        // частичная сессия в registry не остаётся.
                        let _ = self.garbage_collect(TYPE_SESSION, session_id);
                        return Err(block);
                    }
                };
                if plug_id == 0 {
                    // Машинный контракт (точная пара `Nworldserver.exe`
                    // `F3AC454D…` + `WorldServer.pdb`): plug-цикл тела
                    // `CTeam::Unserialize` (`0x4DE103-0x4DE12B`) при отказе
                    // UnserializePlug — например чужой plug_type ≠ 5 — валит
                    // весь unserialize: `test eax, eax; je` (`0x4DE10D-0x4DE10F`)
                    // = return 0 из virtual body. UnserializeSession тогда
                    // разрушает созданную сессию deleting dtor + hash erase
                    // (`0x47C610-0x47C62F`) и возвращает 0; plug-и, вставленные
                    // до отказа, погибают вместе с сессией. Запись
                    // `team_id → session_id` в карте CGame НЕ откатывается:
                    // Start/OnSessionStarted выполнен до plug-цикла (здесь —
                    // дренирован выше), запись остаётся зомби навсегда.
                    // Особенность оригинала, исправлять нельзя.
                    let _ = self.garbage_collect(TYPE_SESSION, session_id);
                    return Ok(0);
                }
                let _ = self.insert_plug(game, session_id, plug_id);
            }
            return Ok(session_id);
        }
        let _ = self.garbage_collect(TYPE_SESSION, session_id);
        Ok(0)
    }
}

fn read_i32(stream: &[u8], offset: &mut i32) -> Result<i32, WorldSessionFactoryInputBlock> {
    read_dword(stream, offset).map(i32::from_le_bytes)
}

fn read_u32(stream: &[u8], offset: &mut i32) -> Result<u32, WorldSessionFactoryInputBlock> {
    read_dword(stream, offset).map(u32::from_le_bytes)
}

fn read_dword(stream: &[u8], offset: &mut i32) -> Result<[u8; 4], WorldSessionFactoryInputBlock> {
    let attempted_offset = *offset;
    *offset = offset.wrapping_add(4);
    let Ok(start) = usize::try_from(attempted_offset) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    let Some(end) = start.checked_add(4) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    let Some(bytes) = stream.get(start..end) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    Ok(bytes
        .try_into()
        .expect("ровно четыре bytes проверены slice-границей"))
}
