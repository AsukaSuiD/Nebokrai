//! Player-owned team plug `CTeamate` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/session/cteamate.cpp`. Материализован достигнутый invite/join
//! prefix: plug identity, player owner, region/name snapshot и wire Serialize,
//! который `OnPlugInserted` вкладывает в клиентский `0xBFD03`, а local exit
//! доводит до player membership и `0xBFD05`. Достигнутые allocation/chat
//! callbacks материализуют `0xBFD08/09` из typed session owner-ов. Lose/restore,
//! AI и остальные change-state ветви сохранены ниже как RAW.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTeamate {
    plug_id: i32,
    owner_id: i32,
    owner_region_id: i32,
    owner_name: Vec<u8>,
}

impl CTeamate {
    pub(crate) fn new(
        plug_id: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Self {
        Self {
            plug_id,
            owner_id,
            owner_region_id,
            owner_name: owner_name
                .split(|byte| *byte == 0)
                .next()
                .unwrap_or_default()
                .to_vec(),
        }
    }

    pub(crate) const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) fn serialize(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&5_i32.to_le_bytes());
        output.extend_from_slice(&400_i32.to_le_bytes());
        output.extend_from_slice(&self.owner_id.to_le_bytes());
        output.extend_from_slice(&0_i32.to_le_bytes());
        output.extend_from_slice(&self.owner_region_id.to_le_bytes());
        output.extend_from_slice(&self.owner_name);
        output.push(0);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp

// ============================================================================
// FUNCTION: CTeamate::SetOwnerRegionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:107
// RVA: 0x000EA270
// ADDRESS: 004ea270
// PROTOTYPE: void __thiscall SetOwnerRegionID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnPlugEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:332
// RVA: 0x000EA2A0
// ADDRESS: 004ea2a0
// PROTOTYPE: int __thiscall OnPlugEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:131
// RVA: 0x000EA3C0
// ADDRESS: 004ea3c0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:163
// RVA: 0x000EA430
// ADDRESS: 004ea430
// PROTOTYPE: int __thiscall OnChangeState(long param_1, long param_2, uchar * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::CTeamate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:16
// RVA: 0x000EA850
// ADDRESS: 004ea850
// PROTOTYPE: undefined __thiscall CTeamate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::~CTeamate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:22
// RVA: 0x000EA880
// ADDRESS: 004ea880
// PROTOTYPE: void __thiscall ~CTeamate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::SetOwnerName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:118
// RVA: 0x000EA8E0
// ADDRESS: 004ea8e0
// PROTOTYPE: void __thiscall SetOwnerName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:146
// RVA: 0x000EA910
// ADDRESS: 004ea910
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnPlugInserted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:294
// RVA: 0x000EA9C0
// ADDRESS: 004ea9c0
// PROTOTYPE: int __thiscall OnPlugInserted(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::IsPlugAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:33
// RVA: 0x000EAB20
// ADDRESS: 004eab20
// PROTOTYPE: int __thiscall IsPlugAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b863c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B863C
// ADDRESS: 005b863c
// PROTOTYPE: undefined Catch@005b863c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b86fd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B86FD
// ADDRESS: 005b86fd
// PROTOTYPE: undefined Catch@005b86fd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b8ed6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B8ED6
// ADDRESS: 005b8ed6
// PROTOTYPE: undefined Catch@005b8ed6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
