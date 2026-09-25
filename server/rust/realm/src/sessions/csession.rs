//! Базовая session `CSession`, перенесённая в Realm `sessions/`, подтверждённая
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Сохраняются full signed lifecycle-флаги, unsigned wrapping tick/lifetime,
//! list-order plug-ов, wire header `[type,min,max,remaining_lifetime]` и
//! безусловные повторные `End/Abort` callbacks. `Vec<WorldSessionPlug>`
//! заменяет `std::list<long>` и дополнительно к ID кэширует неизменяемую owner
//! identity: это позволяет Rust-owner-у выполнять virtual owner lookup без
//! обратного указателя в factory и не меняет порядок или wire. После удаления
//! plug-а обход продолжается со следующей записью.
//!
//! Обращения к plug registry и `CGame`, которые C++ выполнял через globals,
//! представлены ordered `WorldSessionEffect`. `CSessionFactory` забирает их
//! сразу после virtual-вызова; очередь не является новым игровым состоянием.

use rustix::time::{ClockId, clock_gettime};

use crate::regions::baseobject::CBaseObject;
use crate::sessions::csessionfactory::WorldSessionOwner;

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldSessionPlug {
    pub plug_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSessionEffect {
    TeamStarted { team_id: u32 },
    TeamEnded { team_id: u32 },
    KickPlayer { team_id: u32, player_id: i32 },
    PlugState {
        team_id: u32,
        plug_id: i32,
        state: i32,
        value: Vec<u8>,
        include_sender: bool,
    },
}

pub struct CSession {
    object: CBaseObject,
    session_type: u32,
    ended: i32,
    started: i32,
    aborted: i32,
    maximum_plugs: u32,
    minimum_plugs: u32,
    starting_timestamp_ms: u32,
    lifetime_ms: u32,
    plugs: Vec<WorldSessionPlug>,
}

impl CSession {
    pub const fn new(
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
    ) -> Self {
        Self {
            object: CBaseObject::with_reached_constructor_defaults(),
            session_type: 0,
            ended: 0,
            started: 0,
            aborted: 0,
            maximum_plugs,
            minimum_plugs,
            starting_timestamp_ms: 0,
            lifetime_ms,
            plugs: Vec::new(),
        }
    }

    pub fn assign_factory_identity(&mut self, object_type: i32, object_id: i32) {
        self.object.set_type(object_type);
        self.object.set_id(object_id);
    }

    pub const fn object_id(&self) -> i32 {
        self.object.get_id()
    }

    pub const fn set_session_type(&mut self, session_type: u32) {
        self.session_type = session_type;
    }

    pub fn plugs(&self) -> &[WorldSessionPlug] {
        &self.plugs
    }

    pub fn plugs_mut(&mut self) -> &mut Vec<WorldSessionPlug> {
        &mut self.plugs
    }

    pub const fn maximum_plugs(&self) -> u32 {
        self.maximum_plugs
    }

    pub const fn started(&self) -> i32 {
        self.started
    }

    pub const fn ended_flag(&self) -> i32 {
        self.ended
    }

    pub const fn aborted(&self) -> i32 {
        self.aborted
    }

    pub fn start(&mut self) -> i32 {
        self.starting_timestamp_ms = legacy_tick_ms();
        self.started = 1;
        1
    }

    pub const fn is_session_ended(&self) -> i32 {
        ((self.started != 0) && (self.ended != 0)) as i32
    }

    pub fn is_session_available(&self) -> i32 {
        (self.minimum_plugs as usize <= self.plugs.len()) as i32
    }

    pub fn end(&mut self) -> i32 {
        self.starting_timestamp_ms = 0;
        self.ended = 1;
        1
    }

    pub fn abort(&mut self) -> i32 {
        self.starting_timestamp_ms = 0;
        self.aborted = 1;
        1
    }

    pub fn ai(&mut self) -> bool {
        if self.started != 1 || self.ended != 0 || self.aborted != 0 {
            return false;
        }
        if self.lifetime_ms != 0
            && self.starting_timestamp_ms.wrapping_add(self.lifetime_ms) <= legacy_tick_ms()
        {
            self.end();
            return true;
        }
        false
    }

    pub fn can_insert_plug(&self) -> bool {
        self.started != 0
            && self.ended == 0
            && self.aborted == 0
            && self.plugs.len() < self.maximum_plugs as usize
    }

    pub fn insert_plug_identity(&mut self, plug: WorldSessionPlug) {
        self.plugs.push(plug);
    }

    pub fn remove_plug_id(&mut self, plug_id: i32) {
        if let Some(index) = self.plugs.iter().position(|plug| plug.plug_id == plug_id) {
            self.plugs.remove(index);
        }
    }

    pub fn query_plug_by_owner(&self, owner_type: i32, owner_id: i32) -> Option<i32> {
        self.plugs
            .iter()
            .find(|plug| plug.owner_type == owner_type && plug.owner_id == owner_id)
            .map(|plug| plug.plug_id)
    }

    pub fn query_plug_by_id(&self, plug_id: i32) -> Option<i32> {
        self.plugs
            .iter()
            .any(|plug| plug.plug_id == plug_id)
            .then_some(plug_id)
    }

    pub fn serialize(&self, output: &mut Vec<u8>) -> i32 {
        let remaining_lifetime = if self.lifetime_ms == 0 {
            0
        } else {
            self.starting_timestamp_ms
                .wrapping_add(self.lifetime_ms)
                .wrapping_sub(legacy_tick_ms())
        };
        output.extend_from_slice(&self.session_type.to_le_bytes());
        output.extend_from_slice(&self.minimum_plugs.to_le_bytes());
        output.extend_from_slice(&self.maximum_plugs.to_le_bytes());
        output.extend_from_slice(&remaining_lifetime.to_le_bytes());
        1
    }

    pub const fn unserialize(&mut self, stream: Option<&[u8]>) -> i32 {
        stream.is_some() as i32
    }
}

impl WorldSessionOwner for CSession {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32) {
        self.assign_factory_identity(object_type, object_id);
    }

    fn is_session_available(&mut self) -> i32 {
        CSession::is_session_available(self)
    }

    fn abort(&mut self) -> i32 {
        CSession::abort(self)
    }

    fn is_session_ended(&mut self) -> i32 {
        CSession::is_session_ended(self)
    }

    fn end(&mut self) -> i32 {
        CSession::end(self)
    }

    fn ai(&mut self) {
        let _ = CSession::ai(self);
    }

    fn insert_plug(&mut self, _plug_id: i32) -> i32 {
        0
    }

    fn on_plug_change_state(
        &mut self,
        plug_id: i32,
        _state: i32,
        _value: &[u8],
        _recursive: i32,
    ) -> i32 {
        self.query_plug_by_id(plug_id).is_some() as i32
    }

    fn unserialize(&mut self, stream: &[u8], _offset: &mut i32) -> i32 {
        self.unserialize(Some(stream))
    }

    fn insert_plug_identity(&mut self, plug: WorldSessionPlug) {
        CSession::insert_plug_identity(self, plug);
    }

    fn can_insert_plug(&self) -> bool {
        CSession::can_insert_plug(self)
    }

    fn session_plugs(&self) -> &[WorldSessionPlug] {
        CSession::plugs(self)
    }

    fn remove_plug_id(&mut self, plug_id: i32) {
        CSession::remove_plug_id(self, plug_id);
    }

    fn should_traverse_plugs(&self) -> bool {
        self.started == 1 && self.ended == 0 && self.aborted == 0
    }
}
