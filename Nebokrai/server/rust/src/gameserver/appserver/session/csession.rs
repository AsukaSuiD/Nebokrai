//! Достигнутая plug-list storage-часть GameServer `CSession`.
//!
//! PDB `GameServer/GameServer.pdb` фиксирует `std::list<long> m_lPlugs` по
//! `+0x64`; inline `GetPlugList` RVA `0x00070910` экспортирован из точного
//! source-owner `server/gameserver/appserver/area.cpp` и материализован там.
//! `Vec<i32>` сохраняет порядок обхода. Normal equipment-session materializes
//! constructor defaults, Start gate и InsertPlug capacity/state prefix;
//! team lifecycle ниже остаётся RAW. `from_plug_ids` является assembly-
//! границей уже восстановленного registry state. Для normal equipment-session
//! материализован terminal `End`: ended/remove state и ordered обход plug IDs;
//! concrete plug callback/registry lookup выполняет `CSessionFactory`.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSession {
    plug_ids: Vec<i32>,
    minimum_plugs: u32,
    maximum_plugs: u32,
    lifetime: u32,
    started: bool,
    ended: bool,
    aborted: bool,
    remove_requested: bool,
}

impl CSession {
    pub(crate) const fn from_plug_ids(plug_ids: Vec<i32>) -> Self {
        Self {
            plug_ids,
            minimum_plugs: 0,
            maximum_plugs: u32::MAX,
            lifetime: 0,
            started: true,
            ended: false,
            aborted: false,
            remove_requested: false,
        }
    }

    pub(crate) const fn normal(minimum_plugs: u32, maximum_plugs: u32, lifetime: u32) -> Self {
        Self {
            plug_ids: Vec::new(),
            minimum_plugs,
            maximum_plugs,
            lifetime,
            started: false,
            ended: false,
            aborted: false,
            remove_requested: false,
        }
    }

    pub(crate) const fn start(&mut self) -> bool {
        if self.started || self.ended || self.aborted {
            return false;
        }
        self.started = true;
        true
    }

    pub(crate) fn insert_plug(&mut self, plug_id: i32) -> bool {
        if !self.started
            || self.ended
            || self.aborted
            || self.maximum_plugs as usize <= self.plug_ids.len()
        {
            return false;
        }
        self.plug_ids.push(plug_id);
        true
    }

    pub(crate) fn plug_ids_storage(&self) -> &[i32] {
        &self.plug_ids
    }

    pub(crate) fn end(&mut self) -> Vec<i32> {
        self.ended = true;
        self.remove_requested = true;
        self.plug_ids.clone()
    }

    pub(crate) fn abort(&mut self) -> Vec<i32> {
        self.aborted = true;
        self.remove_requested = true;
        self.plug_ids.clone()
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.started && self.ended
    }

    pub(crate) const fn remove_requested(&self) -> bool {
        self.remove_requested
    }

    pub(crate) const fn is_available_prefix(&self) -> bool {
        self.started && !self.ended && !self.aborted
    }

    pub(crate) const fn minimum_plugs(&self) -> u32 {
        self.minimum_plugs
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp

// ============================================================================
// FUNCTION: Catch@004788c9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp
// RVA: 0x000788C9
// ADDRESS: 004788c9
// PROTOTYPE: undefined Catch@004788c9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00478aa6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp
// RVA: 0x00078AA6
// ADDRESS: 00478aa6
// PROTOTYPE: undefined Catch@00478aa6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::Start
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:107
// RVA: 0x0007B310
// ADDRESS: 0047b310
// PROTOTYPE: int __thiscall Start(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `remove_requested` materialизует remove flag для concrete
// equipment-session sweep; покрытое RAW-тело удалено.
// IMPLEMENTED: `is_ended` сохраняет exact started && ended predicate;
// покрытое RAW-тело удалено.
// ============================================================================
// FUNCTION: CSession::OnPlugInserted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:332
// RVA: 0x0007B370
// ADDRESS: 0047b370
// PROTOTYPE: int __thiscall OnPlugInserted(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::OnPlugAborted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:351
// RVA: 0x0007B3B0
// ADDRESS: 0047b3b0
// PROTOTYPE: int __thiscall OnPlugAborted(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::OnPlugEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:372
// RVA: 0x0007B400
// ADDRESS: 0047b400
// PROTOTYPE: int __thiscall OnPlugEnded(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:443
// RVA: 0x0007B450
// ADDRESS: 0047b450
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: terminal `End` state и ordered plug traversal материализованы
// выше; concrete virtual callback остаётся у соответствующего plug owner-а.

// ============================================================================
// FUNCTION: CSession::Abort
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:139
// RVA: 0x0007B520
// ADDRESS: 0047b520
// PROTOTYPE: int __thiscall Abort(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::OnPlugChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:177
// RVA: 0x0007B570
// ADDRESS: 0047b570
// PROTOTYPE: int __thiscall OnPlugChangeState(long param_1, long param_2, uchar * param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::IsSessionAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:221
// RVA: 0x0007B610
// ADDRESS: 0047b610
// PROTOTYPE: int __thiscall IsSessionAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::QueryPlugByOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:291
// RVA: 0x0007B660
// ADDRESS: 0047b660
// PROTOTYPE: CPlug * __thiscall QueryPlugByOwner(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::QueryPlugByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:313
// RVA: 0x0007B6C0
// ADDRESS: 0047b6c0
// PROTOTYPE: CPlug * __thiscall QueryPlugByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:393
// RVA: 0x0007B6F0
// ADDRESS: 0047b6f0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::SendNotification
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:421
// RVA: 0x0007B760
// ADDRESS: 0047b760
// PROTOTYPE: void __thiscall SendNotification(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::~CSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:47
// RVA: 0x0007B7D0
// ADDRESS: 0047b7d0
// PROTOTYPE: void __thiscall ~CSession(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:54
// RVA: 0x0007B830
// ADDRESS: 0047b830
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::CSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:18
// RVA: 0x0007B930
// ADDRESS: 0047b930
// PROTOTYPE: undefined __thiscall CSession(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSession::InsertPlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csession.cpp:253
// RVA: 0x0007B9D0
// ADDRESS: 0047b9d0
// PROTOTYPE: int __thiscall InsertPlug(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
