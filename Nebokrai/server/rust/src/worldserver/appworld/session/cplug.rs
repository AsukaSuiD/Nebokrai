//! Базовый `CPlug` из WorldServer, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire содержит plug type, owner type/ID и signed ended flag. Factory уже
//! читает первые три поля, поэтому virtual decode получает только ended.
//!
//! `ChangeState` и `Exit` синхронно доставляют `OnPlugChangeState` найденной
//! session; `Exit` ставит ended только при найденной session независимо от
//! результата handler-а. Короткий input сохраняет wrapping сдвиг cursor и
//! возвращает ноль.

use crate::worldserver::appworld::baseobject::CBaseObject;
use crate::worldserver::appworld::session::csessionfactory::WorldPlugSessionEffect;

pub(crate) struct CPlug {
    object: CBaseObject,
    session_id: i32,
    owner_type: i32,
    owner_id: i32,
    plug_type: u32,
    ended: i32,
    session_effects: Vec<WorldPlugSessionEffect>,
}

impl CPlug {
    pub(crate) const fn new() -> Self {
        Self {
            object: CBaseObject::with_reached_constructor_defaults(),
            session_id: 0,
            owner_type: 0,
            owner_id: 0,
            plug_type: 0,
            ended: 0,
            session_effects: Vec::new(),
        }
    }

    pub(crate) fn assign_factory_identity(&mut self, object_type: i32, object_id: i32) {
        self.object.set_type(object_type);
        self.object.set_id(object_id);
    }

    pub(crate) const fn object_id(&self) -> i32 {
        self.object.get_id()
    }

    pub(crate) const fn set_plug_type(&mut self, plug_type: u32) {
        self.plug_type = plug_type;
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) const fn set_session(&mut self, session_id: i32) {
        self.session_id = session_id;
    }

    pub(crate) const fn session_id(&self) -> i32 {
        self.session_id
    }

    pub(crate) const fn is_plug_ended(&self) -> i32 {
        self.ended
    }

    pub(crate) fn serialize(&self, output: &mut Vec<u8>) -> i32 {
        output.extend_from_slice(&self.plug_type.to_le_bytes());
        output.extend_from_slice(&self.owner_type.to_le_bytes());
        output.extend_from_slice(&self.owner_id.to_le_bytes());
        output.extend_from_slice(&self.ended.to_le_bytes());
        1
    }

    pub(crate) fn change_state(&mut self, state: i32, value: &[u8]) {
        self.session_effects.push(WorldPlugSessionEffect {
            session_id: self.session_id,
            plug_id: self.object.get_id(),
            state,
            value: value.to_vec(),
            end_after_session_lookup: false,
        });
    }

    pub(crate) fn exit(&mut self) {
        self.session_effects.push(WorldPlugSessionEffect {
            session_id: self.session_id,
            plug_id: self.object.get_id(),
            state: 1,
            value: Vec::new(),
            end_after_session_lookup: true,
        });
    }

    pub(crate) fn take_session_effects(&mut self) -> Vec<WorldPlugSessionEffect> {
        std::mem::take(&mut self.session_effects)
    }

    pub(crate) const fn confirm_exit(&mut self) {
        self.ended = 1;
    }

    pub(crate) fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32 {
        let Some(ended) = read_i32(stream, offset) else {
            return 0;
        };
        self.ended = ended;
        1
    }
}

pub(super) fn read_i32(stream: &[u8], offset: &mut i32) -> Option<i32> {
    let attempted_offset = *offset;
    *offset = offset.wrapping_add(4);
    let start = usize::try_from(attempted_offset).ok()?;
    let end = start.checked_add(4)?;
    let bytes: [u8; 4] = stream.get(start..end)?.try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}
