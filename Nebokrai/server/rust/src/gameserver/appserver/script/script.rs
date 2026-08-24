//! Script resource registry GameServer.
//!
//! `CScript::LoadFunction(nullptr, data)` из точного EXE/PDB читает
//! непрерывный `FunctionList`, преобразует caption через `atoi` и сохраняет
//! text → numeric ID в ordered `std::map`. `BTreeMap` является прямой safe
//! заменой lookup/order semantics; expression VM и instance execution ниже
//! остаются RAW.

use std::collections::BTreeMap;

use super::variablelist::section_records;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CScriptFunctionRegistry {
    functions: BTreeMap<Vec<u8>, i32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ScriptFunctionLoadReport {
    pub(crate) declared_functions: usize,
    pub(crate) replaced_names: usize,
}

impl CScriptFunctionRegistry {
    pub(crate) fn load(&mut self, source: &[u8]) -> ScriptFunctionLoadReport {
        self.functions.clear();
        let mut report = ScriptFunctionLoadReport::default();
        for (caption, name) in section_records(source, b"FunctionList") {
            let id = legacy_atoi(caption);
            if self.functions.insert(name.to_vec(), id).is_some() {
                report.replaced_names += 1;
            }
            report.declared_functions += 1;
        }
        report
    }

    pub(crate) fn query(&self, name: &[u8]) -> Option<i32> {
        self.functions.get(visible_c_string(name)).copied()
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.functions.len();
        self.functions.clear();
        count
    }
}

fn visible_c_string(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

pub(crate) fn legacy_atoi(value: &[u8]) -> i32 {
    let value = visible_c_string(value);
    let value = value
        .get(
            value
                .iter()
                .position(|byte| !byte.is_ascii_whitespace())
                .unwrap_or(value.len())..,
        )
        .unwrap_or_default();
    let (negative, digits) = match value.first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let magnitude =
        digits
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .fold(0_i64, |current, byte| {
                current
                    .saturating_mul(10)
                    .saturating_add(i64::from(*byte - b'0'))
            });
    let signed = if negative { -magnitude } else { magnitude };
    signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp

// ============================================================================
// FUNCTION: stRunScript::stRunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.h:267
// RVA: 0x00024AB0
// ADDRESS: 00424ab0
// PROTOTYPE: undefined __thiscall stRunScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::ReleaseGeneralVariable` замкнут в `CGame::release` через owned `CVariableList`.

// ============================================================================
// FUNCTION: CScript::SetVariableList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:391
// RVA: 0x00024AF0
// ADDRESS: 00424af0
// PROTOTYPE: void __thiscall SetVariableList(CVariableList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateVariableList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:399
// RVA: 0x00024B10
// ADDRESS: 00424b10
// PROTOTYPE: void __thiscall UpdateVariableList(CVariableList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GotoNextLine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:410
// RVA: 0x00024B30
// ADDRESS: 00424b30
// PROTOTYPE: int __thiscall GotoNextLine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::LoadScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:521
// RVA: 0x00024B90
// ADDRESS: 00424b90
// PROTOTYPE: bool __thiscall LoadScript(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::ReadCmd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:575
// RVA: 0x00024CB0
// ADDRESS: 00424cb0
// PROTOTYPE: bool __thiscall ReadCmd(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::IsOperation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1061
// RVA: 0x00024F00
// ADDRESS: 00424f00
// PROTOTYPE: bool __thiscall IsOperation(char param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::OperationNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1073
// RVA: 0x00024F30
// ADDRESS: 00424f30
// PROTOTYPE: int __thiscall OperationNum(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Prew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1111
// RVA: 0x00024FA0
// ADDRESS: 00424fa0
// PROTOTYPE: int __thiscall Prew(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::PrewString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1124
// RVA: 0x00024FD0
// ADDRESS: 00424fd0
// PROTOTYPE: int __thiscall PrewString(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetFunctionName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1521
// RVA: 0x00025000
// ADDRESS: 00425000
// PROTOTYPE: char * __thiscall GetFunctionName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::DumpString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2174
// RVA: 0x00025080
// ADDRESS: 00425080
// PROTOTYPE: int __cdecl DumpString(char * * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateToWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2192
// RVA: 0x000250E0
// ADDRESS: 004250e0
// PROTOTYPE: bool __cdecl UpdateToWorldServer(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateToWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2202
// RVA: 0x00025170
// ADDRESS: 00425170
// PROTOTYPE: bool __cdecl UpdateToWorldServer(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::DispatchCommand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2213
// RVA: 0x00025200
// ADDRESS: 00425200
// PROTOTYPE: long __thiscall DispatchCommand(int param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::LoadGeneralVariable` замкнут в startup decoder-е owned `CVariableList`.

// ============================================================================
// FUNCTION: CScript::SetPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:384
// RVA: 0x000252B0
// ADDRESS: 004252b0
// PROTOTYPE: void __thiscall SetPlayer(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::JumpTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:433
// RVA: 0x000252D0
// ADDRESS: 004252d0
// PROTOTYPE: int __thiscall JumpTo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::JumpToNextBlock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:467
// RVA: 0x00025420
// ADDRESS: 00425420
// PROTOTYPE: int __thiscall JumpToNextBlock(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Count
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1139
// RVA: 0x000255E0
// ADDRESS: 004255e0
// PROTOTYPE: int __thiscall Count(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelectAllScripByPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:96
// RVA: 0x00025FA0
// ADDRESS: 00425fa0
// PROTOTYPE: long __cdecl DelectAllScripByPlayer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelPlayerTalkBoxScrip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:116
// RVA: 0x00026090
// ADDRESS: 00426090
// PROTOTYPE: long __cdecl DelPlayerTalkBoxScrip(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelectPlayerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:162
// RVA: 0x000261C0
// ADDRESS: 004261c0
// PROTOTYPE: long __cdecl DelectPlayerScript(CPlayer * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ScriptIfExit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:187
// RVA: 0x00026300
// ADDRESS: 00426300
// PROTOTYPE: bool __cdecl ScriptIfExit(CPlayer * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelectPlayerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:139
// RVA: 0x000263A0
// ADDRESS: 004263a0
// PROTOTYPE: long __cdecl DelectPlayerScript(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ScriptContinue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:204
// RVA: 0x00026480
// ADDRESS: 00426480
// PROTOTYPE: long __cdecl ScriptContinue(long param_1, CPlayer * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::~CScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:317
// RVA: 0x000264E0
// ADDRESS: 004264e0
// PROTOTYPE: void __thiscall ~CScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::CScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:268
// RVA: 0x00026620
// ADDRESS: 00426620
// PROTOTYPE: undefined __thiscall CScript(CPlayer * param_1, CRegion * param_2, CNpc * param_3, CGUID * param_4, ulong param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:724
// RVA: 0x00026780
// ADDRESS: 00426780
// PROTOTYPE: void __thiscall Check(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::ComputeVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:915
// RVA: 0x00026D20
// ADDRESS: 00426d20
// PROTOTYPE: bool __thiscall ComputeVar(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::RunLine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:678
// RVA: 0x00027AA0
// ADDRESS: 00427aa0
// PROTOTYPE: void __thiscall RunLine(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetIntParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1546
// RVA: 0x00027BF0
// ADDRESS: 00427bf0
// PROTOTYPE: int __thiscall GetIntParam(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetStringParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1705
// RVA: 0x00027E80
// ADDRESS: 00427e80
// PROTOTYPE: char * __thiscall GetStringParam(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: RunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:29
// RVA: 0x00028840
// ADDRESS: 00428840
// PROTOTYPE: long __cdecl RunScript(stRunScript * param_1, char * param_2, tagPOINT * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::ReleaseFunction` замкнут в `CGame::release` через owned function registry.

// ============================================================================
// FUNCTION: CScript::RunStep
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2030
// RVA: 0x00028D80
// ADDRESS: 00428d80
// PROTOTYPE: SCRIPTRETURN __thiscall RunStep(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ScriptLoop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:63
// RVA: 0x00029000
// ADDRESS: 00429000
// PROTOTYPE: long __cdecl ScriptLoop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::LoadFunction` материализован выше как `CScriptFunctionRegistry::load`.

// ============================================================================
// FUNCTION: CVariableList::`scalar_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp
// RVA: 0x000AE4E0
// ADDRESS: 004ae4e0
// PROTOTYPE: void * __thiscall `scalar_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
