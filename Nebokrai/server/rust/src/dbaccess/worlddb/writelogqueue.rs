//! FIFO-owner `CWriteLogQueue` исторического WorldServer.
//!
//! `GetSize`, `PopWriteLogData`, constructor, `Clear`, destructor и
//! `PushWriteLogData` — `IMPLEMENTED`: `Arc<Mutex<VecDeque<_>>>` заменяет
//! Win32 critical section, MSVC deque и сырые `char*`, а typed
//! `WorldWriteLogCommand` владеет данными до pop. Очередь намеренно не вводит
//! новый limit: exact проверял `_dwLimit=0xFFFF` только для null pointer, тогда
//! как все штатные non-null SQL обходили gate; typed Rust null-команды не имеет.
//! `Clear` и `Drop` освобождают весь хвост безопасно вместо переноса ошибочного
//! ручного pointer/deque cleanup. DB batch и thread lifecycle остаются в
//! `worldserver/worldserver/writelogworker.rs`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::worldserver::appworld::message::writelogmessage::WorldWriteLogCommand;

#[derive(Clone, Default)]
pub(crate) struct WorldWriteLogQueue {
    commands: Arc<Mutex<VecDeque<WorldWriteLogCommand>>>,
}

impl WorldWriteLogQueue {
    pub(crate) fn push(&self, command: WorldWriteLogCommand) -> usize {
        let mut commands = self.lock();
        commands.push_back(command);
        commands.len()
    }

    pub(crate) fn pop(&self) -> Option<WorldWriteLogCommand> {
        self.lock().pop_front()
    }

    pub(crate) fn len(&self) -> usize {
        self.lock().len()
    }

    pub(crate) fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<WorldWriteLogCommand>> {
        self.commands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp

// ============================================================================
// FUNCTION: CMyAdoBase::`scalar_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp
// RVA: 0x000EB570
// ADDRESS: 004eb570
// PROTOTYPE: void * __thiscall `scalar_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Command15::CreateParameter
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp
// RVA: 0x000EB790
// ADDRESS: 004eb790
// PROTOTYPE: _com_ptr_t<_com_IIID<_Parameter,&struct___s_GUID_const__GUID_0000050c_0000_0010_8000_00aa006d2ea4>_> __thiscall CreateParameter(_bstr_t param_1, DataTypeEnum param_2, ParameterDirectionEnum param_3, long param_4, _variant_t * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ebceb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp
// RVA: 0x000EBCEB
// ADDRESS: 004ebceb
// PROTOTYPE: undefined Catch@004ebceb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::GetSize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:53
// RVA: 0x000EC840
// ADDRESS: 004ec840
// PROTOTYPE: uint __thiscall GetSize(void)
//
// IMPLEMENTED_OWNER: `WorldWriteLogQueue::len` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::PopWriteLogData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:36
// RVA: 0x000EC860
// ADDRESS: 004ec860
// PROTOTYPE: char * __thiscall PopWriteLogData(void)
//
// IMPLEMENTED_OWNER: `WorldWriteLogQueue::pop` выше возвращает owned command.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::CWriteLogQueue
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:14
// RVA: 0x000EC8E0
// ADDRESS: 004ec8e0
// PROTOTYPE: undefined __thiscall CWriteLogQueue(void)
//
// не нуждаются в исходном фактически не применявшемся limit `0xFFFF`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:62
// RVA: 0x000EC910
// ADDRESS: 004ec910
// PROTOTYPE: void __thiscall Clear(void)
//
// IMPLEMENTED_OWNER: `WorldWriteLogQueue::clear` выше освобождает весь FIFO.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::~CWriteLogQueue
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:19
// RVA: 0x000EC9E0
// ADDRESS: 004ec9e0
// PROTOTYPE: void __thiscall ~CWriteLogQueue(void)
//
// IMPLEMENTED_OWNER: `Arc`, `Mutex` и `VecDeque` освобождаются Rust `Drop`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWriteLogQueue::PushWriteLogData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\writelogqueue.cpp:25
// RVA: 0x000ECC40
// ADDRESS: 004ecc40
// PROTOTYPE: bool __thiscall PushWriteLogData(char * param_1)
//
// IMPLEMENTED_OWNER: `WorldWriteLogQueue::push` выше; typed command не может
// представить исходный null pointer, единственный путь limit-check-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//








// COMPONENT_VARIANT_END: WorldServer
