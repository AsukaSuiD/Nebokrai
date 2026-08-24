//! Достигнутая lookup-часть GameServer `CSessionFactory`.
//!
//! `QuerySession` RVA `0x000780C0` и `QueryPlug` RVA `0x00078190` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `server/gameserver/appserver/session/csessionfactory.cpp`. PDB подтверждает
//! две static `hash_map<long, CSession*/CPlug*>`; оба lookup возвращают null
//! при отсутствии ключа.
//!
//! Один owned `CSessionFactory`, подключённый к `CGame`, заменяет две
//! process-static maps, а `BTreeMap`
//! является deterministic replacement: hash iteration этими функциями не
//! наблюдается, только exact-key lookup. `register_*`
//! материализует достигнутый registry storage; `goodsmessage 0x8FC25`
//! выполняет ordered session plug lookup по owner type/ID. Но это не
//! объявляет реализованными RAW `CreateSession/CreatePlug/InsertPlug`, их
//! lifecycle и garbage collection ниже.

use std::collections::BTreeMap;

use super::cequipmentcompose::CEquipmentCompose;
use super::cplug::CPlug;
use super::csession::CSession;

#[derive(Debug, Default)]
pub(crate) struct CSessionFactory {
    sessions: BTreeMap<i32, CSession>,
    plugs: BTreeMap<i32, CPlug>,
    equipment_compose_plugs: BTreeMap<i32, CEquipmentCompose>,
}

impl CSessionFactory {
    pub(crate) fn register_session(
        &mut self,
        session_id: i32,
        session: CSession,
    ) -> Option<CSession> {
        self.sessions.insert(session_id, session)
    }

    pub(crate) fn register_plug(&mut self, plug_id: i32, mut plug: CPlug) -> Option<CPlug> {
        plug.set_id(plug_id);
        self.plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_session(&self, session_id: i32) -> Option<&CSession> {
        self.sessions.get(&session_id)
    }

    pub(crate) fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        self.plugs.get(&plug_id)
    }

    pub(crate) fn query_session_plug_by_owner(
        &self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> Option<&CPlug> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .find_map(|plug_id| {
                self.plugs
                    .get(plug_id)
                    .filter(|plug| plug.has_owner(owner_type, owner_id))
            })
    }

    pub(crate) fn register_equipment_compose_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentCompose,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_compose_plug(&self, plug_id: i32) -> Option<&CEquipmentCompose> {
        self.equipment_compose_plugs.get(&plug_id)
    }

    pub(crate) fn query_equipment_compose_plug_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CEquipmentCompose> {
        self.equipment_compose_plugs.get_mut(&plug_id)
    }

    pub(crate) fn take_equipment_compose_plug(
        &mut self,
        plug_id: i32,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.remove(&plug_id)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp

// ============================================================================
// FUNCTION: CSessionFactory::query_session_by_owner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:81
// RVA: 0x00078060
// ADDRESS: 00478060
// PROTOTYPE: CSession * __cdecl query_session_by_owner(OBJECT_TYPE param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `QuerySession` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CSessionFactory::GarbageCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:200
// RVA: 0x000780F0
// ADDRESS: 004780f0
// PROTOTYPE: void __cdecl GarbageCollect(OBJECT_TYPE param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `QueryPlug` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CSessionFactory::InsertPlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:104
// RVA: 0x000781C0
// ADDRESS: 004781c0
// PROTOTYPE: int __cdecl InsertPlug(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:119
// RVA: 0x000785D0
// ADDRESS: 004785d0
// PROTOTYPE: void __cdecl AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::CreateSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:37
// RVA: 0x00078DA0
// ADDRESS: 00478da0
// PROTOTYPE: long __cdecl CreateSession(ulong param_1, ulong param_2, ulong param_3, SESSION_TYPE param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::CreatePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:244
// RVA: 0x00078EC0
// ADDRESS: 00478ec0
// PROTOTYPE: long __cdecl CreatePlug(PLUG_TYPE param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:297
// RVA: 0x000790D0
// ADDRESS: 004790d0
// PROTOTYPE: long __cdecl UnserializePlug(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializeSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:324
// RVA: 0x000791B0
// ADDRESS: 004791b0
// PROTOTYPE: long __cdecl UnserializeSession(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
