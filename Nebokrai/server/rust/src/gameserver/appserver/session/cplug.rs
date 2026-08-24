//! Достигнутая owner-часть GameServer `CPlug`.
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
//! Полный polymorphic plug/session lifecycle ниже остаётся RAW. Base object и
//! пять достигнутых scalar-полей выражены safe Rust storage; `Option<&CPlayer>`
//! заменяет исходный nullable `CMoveShape*`, сохраняя успешный RTTI-контракт.

use crate::gameserver::appserver::baseobject::CBaseObject;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::gameserver::game::CGame;

const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPlug {
    base_object: CBaseObject,
    plug_type: u32,
    ended: bool,
    session_id: i32,
    owner_type: i32,
    owner_id: i32,
}

impl CPlug {
    pub(crate) const fn new() -> Self {
        Self {
            base_object: CBaseObject::with_reached_constructor_defaults(),
            plug_type: 0,
            ended: false,
            session_id: 0,
            owner_type: 0,
            owner_id: 0,
        }
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub(crate) const fn set_id(&mut self, plug_id: i32) {
        self.base_object.set_id(plug_id);
    }

    pub(crate) const fn id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub(crate) const fn has_owner(&self, owner_type: i32, owner_id: i32) -> bool {
        self.owner_type == owner_type && self.owner_id == owner_id
    }

    pub(crate) fn get_owner<'a>(&self, game: &'a CGame) -> Option<&'a CPlayer> {
        if self.owner_type != PLAYER_TYPE {
            return None;
        }
        game.find_player(self.owner_id)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp

// IMPLEMENTED: constructor, base-only destructor и `SetOwner` материализованы
// выше; покрытые raw-блоки удалены.

// ============================================================================
// FUNCTION: CPlug::SetSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:189
// RVA: 0x0007B160
// ADDRESS: 0047b160
// PROTOTYPE: void __thiscall SetSession(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlug::GetSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:197
// RVA: 0x0007B170
// ADDRESS: 0047b170
// PROTOTYPE: CSession * __thiscall GetSession(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlug::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:227
// RVA: 0x0007B180
// ADDRESS: 0047b180
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlug::ChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:89
// RVA: 0x0007B1E0
// ADDRESS: 0047b1e0
// PROTOTYPE: int __thiscall ChangeState(long param_1, uchar * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlug::Exit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:104
// RVA: 0x0007B220
// ADDRESS: 0047b220
// PROTOTYPE: int __thiscall Exit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlug::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:236
// RVA: 0x0007B260
// ADDRESS: 0047b260
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `GetOwner` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CPlug::IsPlugAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cplug.cpp:41
// RVA: 0x0007B2E0
// ADDRESS: 0047b2e0
// PROTOTYPE: int __thiscall IsPlugAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
