//! Базовый plug-owner исторического WorldServer.
//!
//! Все девять функций `CPlug` из
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\session\cplug.cpp`
//! восстановлены по точной паре `Nworldserver.exe + WorldServer.pdb`:
//! constructor/destructor RVA `0x000DE9E0/0x000DEA00`, `SetOwner`
//! `0x000DEA10`, `IsPlugEnded` `0x000DEA30`, `SetSession/GetSession`
//! `0x000DEA60/0x000DEA70`, `Serialize` `0x000DEA80`, `ChangeState`
//! `0x000DEAE0`, `Exit` `0x000DEB20`, `Unserialize` `0x000DEB60`.
//!
//! Wire сохраняет четыре little-endian `long`: plug type, owner type, owner
//! ID и полный signed ended-флаг. Factory header уже читает первые три поля,
//! поэтому virtual `Unserialize` читает только ended. Исходные
//! `ChangeState/Exit` синхронно искали session по сохранённому ID и вызывали
//! `OnPlugChangeState`; Rust ставит тот же вызов в короткую owned-очередь,
//! которую `CSessionFactory::with_teamate` немедленно доставляет в единственный
//! session registry. Так устраняется global raw pointer, но сохраняются
//! порядок вызова и правило `Exit`: ended назначается только при найденной
//! session, независимо от результата её virtual handler-а.
//!
//! Ручной STL byte-array заменён `Vec<u8>`. Короткий либо переполненный offset
//! вместо unchecked чтения не обращается за границы; cursor сохраняет уже
//! выполненное wrapping `+4`, а virtual result становится `0`.

use crate::worldserver::appworld::baseobject::CBaseObject;
use crate::worldserver::appworld::session::csessionfactory::WorldPlugSessionEffect;

/// Достигнутое base-состояние всех session plug-ов.
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
    /// Воспроизводит constructor defaults до derived plug type assignment.
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

    /// Дописывает исходный plug header и всегда возвращает `1`.
    pub(crate) fn serialize(&self, output: &mut Vec<u8>) -> i32 {
        output.extend_from_slice(&self.plug_type.to_le_bytes());
        output.extend_from_slice(&self.owner_type.to_le_bytes());
        output.extend_from_slice(&self.owner_id.to_le_bytes());
        output.extend_from_slice(&self.ended.to_le_bytes());
        1
    }

    /// Ставит немедленный state-call сохранённой session.
    pub(crate) fn change_state(&mut self, state: i32, value: &[u8]) {
        self.session_effects.push(WorldPlugSessionEffect {
            session_id: self.session_id,
            plug_id: self.object.get_id(),
            state,
            value: value.to_vec(),
            end_after_session_lookup: false,
        });
    }

    /// Ставит state `1`; ended будет назначен factory только при живой session.
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

    /// Читает единственный virtual suffix `m_bPlugEnded`.
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
