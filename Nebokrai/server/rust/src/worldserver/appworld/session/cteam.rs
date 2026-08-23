//! Конкретная session команды WorldServer из точной пары EXE/PDB.
//!
//! Owner сохраняет delay `125`, минутный leader-check, allocation `0`, team
//! wire, порядок Start-before-plug-count при unserialize и opcodes
//! `0x7FD02..0x7FD0C` через ordered effects фабрики.
//!
//! Важная совместимая странность owner-а: `SetAllocationScheme`
//! передаёт в `OnPlugChangeState` player leader ID как будто это factory plug
//! ID. Этот quirk сохраняется, потому что влияет на наблюдаемую доставку.
//! Owner identity кэшируется рядом с ordered plug ID, а обращения к
//! factory/game выполняются немедленными typed effects.

use crate::worldserver::appworld::session::csession::{
    CSession, WorldSessionEffect, WorldSessionPlug,
};
use crate::worldserver::appworld::session::csessionfactory::{
    WorldSessionOwner, WorldTeamSessionOwner,
};
use crate::worldserver::worldserver::game::legacy_tick_ms;

const PLAYER_OWNER_TYPE: i32 = 400;
const TEAM_CHECK_INTERVAL_MS: u32 = 60_000;

pub(crate) struct CTeam {
    session: CSession,
    team_id: u32,
    password: Vec<u8>,
    team_name: Vec<u8>,
    team_leader_id: i32,
    last_checked_timestamp_ms: u32,
    allocation_scheme: i32,
    queried_plugs: Vec<(i32, i32)>,
    delay: i32,
    effects: Vec<WorldSessionEffect>,
    pending_unserialize_plugs: u32,
}

impl CTeam {
    pub(crate) const fn new(
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
    ) -> Self {
        let mut session = CSession::new(minimum_plugs, maximum_plugs, lifetime_ms);
        session.set_session_type(1);
        Self {
            session,
            team_id: 0,
            password: Vec::new(),
            team_name: Vec::new(),
            team_leader_id: 0,
            last_checked_timestamp_ms: 0,
            allocation_scheme: 0,
            queried_plugs: Vec::new(),
            delay: 125,
            effects: Vec::new(),
            pending_unserialize_plugs: 0,
        }
    }

    pub(crate) fn session(&self) -> &CSession {
        &self.session
    }

    pub(crate) fn session_mut(&mut self) -> &mut CSession {
        &mut self.session
    }

    pub(crate) fn insert_plug_identity(&mut self, plug: WorldSessionPlug) {
        self.session.insert_plug_identity(plug);
    }

    pub(crate) fn take_effects(&mut self) -> Vec<WorldSessionEffect> {
        std::mem::take(&mut self.effects)
    }

    pub(crate) fn take_pending_unserialize_plugs(&mut self) -> u32 {
        std::mem::take(&mut self.pending_unserialize_plugs)
    }

    pub(crate) fn serialize_header(&self, output: &mut Vec<u8>) -> i32 {
        self.session.serialize(output);
        output.extend_from_slice(&self.team_id.to_le_bytes());
        output.extend_from_slice(&self.team_name);
        output.push(0);
        output.extend_from_slice(&self.password);
        output.push(0);
        output.extend_from_slice(&self.team_leader_id.to_le_bytes());
        output.extend_from_slice(&(self.session.plugs().len() as u32).to_le_bytes());
        1
    }

    fn start(&mut self) -> i32 {
        self.session.start();
        self.effects.push(WorldSessionEffect::TeamStarted {
            team_id: self.team_id,
        });
        1
    }

    fn end(&mut self) -> i32 {
        self.session.end();
        self.effects.push(WorldSessionEffect::TeamEnded {
            team_id: self.team_id,
        });
        1
    }

    fn abort(&mut self) -> i32 {
        self.session.abort();
        self.effects.push(WorldSessionEffect::TeamEnded {
            team_id: self.team_id,
        });
        1
    }

    fn queue_state(&mut self, plug_id: i32, state: i32, value: &[u8], include_sender: bool) {
        self.effects.push(WorldSessionEffect::PlugState {
            team_id: self.team_id,
            plug_id,
            state,
            value: value.to_vec(),
            include_sender,
        });
    }

    fn run_session_ai_prefix(&mut self) {
        if self.session.ai() {
            self.effects.push(WorldSessionEffect::TeamEnded {
                team_id: self.team_id,
            });
        }
    }

    fn run_ai_suffix(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.last_checked_timestamp_ms == 0 {
            self.last_checked_timestamp_ms = legacy_tick_ms();
            return;
        }
        if self.last_checked_timestamp_ms.wrapping_add(TEAM_CHECK_INTERVAL_MS) <= legacy_tick_ms() {
            if self
                .session
                .query_plug_by_owner(PLAYER_OWNER_TYPE, self.team_leader_id)
                .is_none()
            {
                if let Some(first) = self.session.plugs().first().copied() {
                    self.set_leader(first.owner_id);
                }
            }
            self.last_checked_timestamp_ms = legacy_tick_ms();
        }
    }

    fn unserialize_team(&mut self, stream: &[u8], offset: &mut i32) -> i32 {
        let Some(team_id) = read_u32(stream, offset) else {
            return 0;
        };
        self.team_id = team_id;
        let Some(team_name) = read_legacy_string(stream, offset) else {
            return 0;
        };
        self.team_name = team_name;
        let Some(password) = read_legacy_string(stream, offset) else {
            return 0;
        };
        self.password = password;
        let Some(leader_id) = read_i32(stream, offset) else {
            return 0;
        };
        self.team_leader_id = leader_id;
        self.start();
        let Some(plug_count) = read_u32(stream, offset) else {
            return 0;
        };
        self.pending_unserialize_plugs = plug_count;
        1
    }
}

impl WorldSessionOwner for CTeam {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32) {
        self.session.assign_factory_identity(object_type, object_id);
    }

    fn is_session_available(&mut self) -> i32 {
        if self.delay > 0 {
            1
        } else {
            self.session.is_session_available()
        }
    }

    fn abort(&mut self) -> i32 {
        self.abort()
    }

    fn is_session_ended(&mut self) -> i32 {
        self.session.is_session_ended()
    }

    fn end(&mut self) -> i32 {
        self.end()
    }

    fn ai(&mut self) {
        self.run_session_ai_prefix();
        self.run_ai_suffix();
    }

    fn ai_prefix(&mut self) {
        self.run_session_ai_prefix();
    }

    fn ai_suffix(&mut self) {
        self.run_ai_suffix();
    }

    fn insert_plug(&mut self, _plug_id: i32) -> i32 {
        0
    }

    fn on_plug_change_state(
        &mut self,
        plug_id: i32,
        state: i32,
        value: &[u8],
        recursive: i32,
    ) -> i32 {
        self.queue_state(plug_id, state, value, recursive != 0);
        1
    }

    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32 {
        self.unserialize_team(stream, offset)
    }

    fn as_team_mut(&mut self) -> Option<&mut dyn WorldTeamSessionOwner> {
        Some(self)
    }

    fn take_effects(&mut self) -> Vec<WorldSessionEffect> {
        self.take_effects()
    }

    fn take_pending_unserialize_plugs(&mut self) -> u32 {
        self.take_pending_unserialize_plugs()
    }

    fn insert_plug_identity(&mut self, plug: WorldSessionPlug) {
        self.insert_plug_identity(plug);
    }

    fn can_insert_plug(&self) -> bool {
        self.session.can_insert_plug()
    }

    fn session_plugs(&self) -> &[WorldSessionPlug] {
        self.session.plugs()
    }

    fn remove_plug_id(&mut self, plug_id: i32) {
        self.session.remove_plug_id(plug_id);
    }

    fn should_traverse_plugs(&self) -> bool {
        self.session.started() == 1
            && self.session.ended_flag() == 0
            && self.session.aborted() == 0
    }
}

impl WorldTeamSessionOwner for CTeam {
    fn query_plug_by_owner(&mut self, owner_type: i32, owner_id: i32) -> Option<i32> {
        self.session.query_plug_by_owner(owner_type, owner_id)
    }

    fn set_leader(&mut self, player_id: i32) {
        let Some(new_plug_id) = self
            .session
            .query_plug_by_owner(PLAYER_OWNER_TYPE, player_id)
        else {
            return;
        };
        if self.session.is_session_ended() != 0 {
            return;
        }
        let mut source_plug_id = new_plug_id;
        let mut include_sender = true;
        if self.team_leader_id != 0 {
            if let Some(old_plug_id) = self
                .session
                .query_plug_by_owner(PLAYER_OWNER_TYPE, self.team_leader_id)
            {
                source_plug_id = old_plug_id;
                include_sender = false;
            }
        }
        self.team_leader_id = player_id;
        self.queue_state(
            source_plug_id,
            4,
            &player_id.to_le_bytes(),
            include_sender,
        );
    }

    fn kick_player(&mut self, player_id: i32) {
        self.effects.push(WorldSessionEffect::KickPlayer {
            team_id: self.team_id,
            player_id,
        });
    }

    fn serialize(&mut self, output: &mut Vec<u8>) {
        self.serialize_header(output);
    }

    fn allocation_scheme(&mut self) -> i32 {
        self.allocation_scheme
    }

    fn set_allocation_scheme(&mut self, scheme: i32) {
        self.allocation_scheme = scheme;
        self.queue_state(
            self.team_leader_id,
            5,
            &scheme.to_le_bytes(),
            false,
        );
    }

    fn on_plug_change_state(&mut self, plug_id: i32, state: i32, value: &[u8]) {
        self.queue_state(plug_id, state, value, false);
    }
}

fn read_i32(stream: &[u8], offset: &mut i32) -> Option<i32> {
    read_u32(stream, offset).map(|value| value as i32)
}

fn read_u32(stream: &[u8], offset: &mut i32) -> Option<u32> {
    let start_value = *offset;
    *offset = offset.wrapping_add(4);
    let start = usize::try_from(start_value).ok()?;
    let end = start.checked_add(4)?;
    let bytes: [u8; 4] = stream.get(start..end)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_legacy_string(stream: &[u8], offset: &mut i32) -> Option<Vec<u8>> {
    let start = usize::try_from(*offset).ok()?;
    let search_end = stream.len().min(start.checked_add(0x100)?);
    let relative_nul = stream.get(start..search_end)?.iter().position(|byte| *byte == 0)?;
    let end = start + relative_nul;
    *offset = i32::try_from(end.checked_add(1)?).ok()?;
    Some(stream[start..end].to_vec())
}
