//! Владелец `CVariableList` исторического WorldServer из `variablelist.cpp`.
//!
//! Статус `SaveVarData` RVA `0x000A1930` — `IMPLEMENTED`; остальные функции
//! файла ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp`.
//!
//! PDB задаёт `CVariableList` как `m_lVarNum: signed long` и
//! `m_pVarList: stVariable*`, а 16-байтный `stVariable` содержит `Name` по
//! `+0`, signed `Array` по `+4` и две union-пары `Value/strValue` и
//! `SValue/strSValue` по `+8/+0xc`. Достигнутый `GetOneVar` `0x000A19C0`
//! отдаёт имя и два уже форматированных ANSI-значения: `%d` для scalar,
//! `\"%s\"` для string и пустые значения для положительного `Array`.
//! `VariableListSaveSource` сохраняет именно этот узкий byte-exact view; сам
//! layout и остальные операции списка будут материализованы в их владельце.
//!
//! `SaveVarData` не имел собственной DB-семантики: копировал тот же ADO
//! connection, получал `GetGame()->m_pRsGenVar`, вызывал
//! `CRsGenVar::Save(this, connection)` и возвращал его `bool`. Rust заменяет
//! singleton явными owner/connection аргументами и исключает `nullptr` через
//! ссылки, но не меняет порядок, соединение либо результат. Условный
//! `BLOCKED_MISSING_FACT` переполнения исходного SQL scratch-буфера проходит
//! наружу отдельно и не маскируется придуманным `false`.

use crate::dbaccess::worlddb::rsgenvar::{GenVarSaveOutcome, RsGenVarOwner};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

/// Три byte-exact строки, которые исходный `GetOneVar` отдавал DB-owner-у.
pub(crate) struct VariableSaveRow {
    pub(crate) name: Vec<u8>,
    pub(crate) initial_value: Vec<u8>,
    pub(crate) current_value: Vec<u8>,
}

/// Узкий read-only view достигнутой части `CVariableList`.
pub(crate) trait VariableListSaveSource {
    /// Число выполняемых исходным signed-циклом итераций.
    fn variable_count(&self) -> usize;

    /// Возвращает результат `GetOneVar` для доказанно допустимого индекса.
    fn save_row(&self, index: usize) -> VariableSaveRow;
}

/// Делегирует сохранение тому же DB-owner-у на том же активном connection.
pub(crate) async fn save_var_data<S: VariableListSaveSource, O: RsGenVarOwner>(
    variables: &S,
    database: &mut O,
    active_transaction: &mut WorldTdsClient,
) -> GenVarSaveOutcome {
    database.save(variables, active_transaction).await
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::~tagCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0003FED0
// ADDRESS: 0043fed0
// PROTOTYPE: void __thiscall ~tagCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::tagCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0003FFD0
// ADDRESS: 0043ffd0
// PROTOTYPE: undefined __thiscall tagCompose(tagCompose * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440286
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040286
// ADDRESS: 00440286
// PROTOTYPE: undefined Catch@00440286()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440388
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040388
// ADDRESS: 00440388
// PROTOTYPE: undefined Catch@00440388()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044066a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0004066A
// ADDRESS: 0044066a
// PROTOTYPE: undefined Catch@0044066a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440736
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040736
// ADDRESS: 00440736
// PROTOTYPE: undefined Catch@00440736()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::CVariableList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:18
// RVA: 0x000A1090
// ADDRESS: 004a1090
// PROTOTYPE: undefined __thiscall CVariableList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:29
// RVA: 0x000A10B0
// ADDRESS: 004a10b0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetArrayName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:510
// RVA: 0x000A1160
// ADDRESS: 004a1160
// PROTOTYPE: void __thiscall GetArrayName(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:577
// RVA: 0x000A11B0
// ADDRESS: 004a11b0
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:601
// RVA: 0x000A1240
// ADDRESS: 004a1240
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadVarList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:177
// RVA: 0x000A1320
// ADDRESS: 004a1320
// PROTOTYPE: void __thiscall LoadVarList(char * param_1, char * param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadVarData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:447
// RVA: 0x000A1630
// ADDRESS: 004a1630
// PROTOTYPE: void __thiscall LoadVarData(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadOneVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:60
// RVA: 0x000A1680
// ADDRESS: 004a1680
// PROTOTYPE: void __thiscall LoadOneVar(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetOneVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:144
// RVA: 0x000A19C0
// ADDRESS: 004a19c0
// PROTOTYPE: void __thiscall GetOneVar(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:644
// RVA: 0x000A1B10
// ADDRESS: 004a1b10
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@0052e010
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0012E010
// ADDRESS: 0052e010
// PROTOTYPE: undefined Unwind@0052e010()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e030
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0012E030
// ADDRESS: 0052e030
// PROTOTYPE: undefined Unwind@0052e030()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
