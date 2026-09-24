//! Достигнутая owner-часть GameServer `CPlug`, перенесённая в Zone `sessions/`.
//!
//! Constructor/`SetOwner`/`GetOwner` RVA
//! `0x0007B0F0/0x0007B130/0x0007B280` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `server/gameserver/appserver/session/cplug.cpp`. PDB фиксирует
//! `m_dwPlugType +0x44`, ended `+0x48`, session `+0x4C`, owner type/id
//! `+0x50/+0x54`; `GetOwner` принимает только type `400` и ищет player по
//! signed owner ID.
//!
//! Base object и пять достигнутых scalar-полей выражены safe Rust storage;
//! signed ended-dword сохраняется без нормализации на wire,
//! Обратный вызов разрешения owner заменяет исходный nullable `CMoveShape*`:
//! тип возврата принадлежит владеющему resolver-у, гейт type `400` и signed
//! owner ID сохранены здесь. Terminal `Exit` проверяет живую session через
//! factory-owner и только после session state dispatch фиксирует ended.

use crate::regions::baseobject::CBaseObject;

const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CPlug {
    base_object: CBaseObject,
    plug_type: u32,
    ended: i32,
    session_id: i32,
    owner_type: i32,
    owner_id: i32,
}

impl CPlug {
    pub const fn new() -> Self {
        Self {
            base_object: CBaseObject::with_reached_constructor_defaults(),
            plug_type: 0,
            ended: 0,
            session_id: 0,
            owner_type: 0,
            owner_id: 0,
        }
    }

    pub const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub const fn set_id(&mut self, plug_id: i32) {
        self.base_object.set_id(plug_id);
    }

    pub const fn set_plug_type(&mut self, plug_type: u32) {
        self.plug_type = plug_type;
    }

    pub const fn set_session(&mut self, session_id: i32) {
        self.session_id = session_id;
    }

    pub const fn id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub const fn has_owner(&self, owner_type: i32, owner_id: i32) -> bool {
        self.owner_type == owner_type && self.owner_id == owner_id
    }

    pub const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub const fn session_id(&self) -> i32 {
        self.session_id
    }

    pub const fn is_ended(&self) -> bool {
        self.ended != 0
    }

    pub const fn ended_state(&self) -> i32 {
        self.ended
    }

    pub const fn set_ended_state(&mut self, ended: i32) {
        self.ended = ended;
    }

    pub const fn mark_ended(&mut self) {
        self.ended = 1;
    }

    pub fn get_owner<'a, T>(&self, lookup: &impl Fn(i32) -> Option<&'a T>) -> Option<&'a T> {
        if self.owner_type != PLAYER_TYPE {
            return None;
        }
        lookup(self.owner_id)
    }
}
