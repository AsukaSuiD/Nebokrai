//! Статус корпуса смешанный: `App::init/mainLoop` заменены прямым Linux
//! bootstrap в `main.rs` и `cgame.rs`; остальные GUI/operator-функции сохраняют
//! `UNKNOWN` (исследовательский декомпилят хранится локально) до отдельных решений владельца.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

// COMPONENT_VARIANT_BEGIN: AuthServer
// Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SHA-256 EXE: AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15
// SHA-256 PDB: 26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5
// Исходный владелец PDB: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp

// ============================================================================
// FUNCTION: App::addLSItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:83
// RVA: 0x00001030
// ADDRESS: 00401030
// PROTOTYPE: void __thiscall addLSItem(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::delLSItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:91
// RVA: 0x00001090
// ADDRESS: 00401090
// PROTOTYPE: void __thiscall delLSItem(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::onResize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:104
// RVA: 0x000010F0
// ADDRESS: 004010f0
// PROTOTYPE: void __thiscall onResize(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::App
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:24
// RVA: 0x00001210
// ADDRESS: 00401210
// PROTOTYPE: void __thiscall App(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::~App
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:28
// RVA: 0x000012A0
// ADDRESS: 004012a0
// PROTOTYPE: void __thiscall ~App(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::onMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:145
// RVA: 0x000015A0
// ADDRESS: 004015a0
// PROTOTYPE: long __thiscall onMessage(uint param_1, uint param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: App::addLogText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:114
// RVA: 0x00001D10
// ADDRESS: 00401d10
// PROTOTYPE: void __thiscall addLogText(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: _RtlUnwind@16
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp
// RVA: 0x0002A024
// ADDRESS: 0042a024
// PROTOTYPE: void __stdcall _RtlUnwind@16(PVOID TargetFrame, PVOID TargetIp, PEXCEPTION_RECORD ExceptionRecord, PVOID ReturnValue)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\app.cpp:13
// RVA: 0x0002B600
// ADDRESS: 0042b600
// PROTOTYPE: void __cdecl $E1(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: AuthServer
