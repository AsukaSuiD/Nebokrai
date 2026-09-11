//! `CNotDisappearAfterDead`, GameServer.exe + GameServer.pdb,
//! исходный owner `appserver/other states/notdisappearafterdead.cpp`.
//! OnUpdateProperties (vtable 0x0065E2AC +0x24 →0x005D6580) получает
//! GetSufferer: NULL даёт 0, только player 400 получает формулу, прочие — 1.
//! Visual, End, IsEnded и часы в этой функции не участвуют. Типизированная
//! арифметика находится в CPlayer::apply_undead_state_properties; вызов
//! конкретного экземпляра выполняет общий property-проход states/state.rs.
//! Проценты используют low32 IMUL, затем unsigned /100 (0x005D65D6,
//! 0x005D67FA), включая wrapping negation отрицательных параметров.
//! Вызванные setters 0x0042ACF0..0x0042AE10 ограничивают каждую unsigned
//! сумму INT_MAX. Отрицательные direct/CON HP/DEF/INT MP сохраняют signed
//! WORD-сужение, а STR/DEX и INT resistance/element — signed DWORD floor1.
//! FILD/FMUL не заменены ранним округлением всего выражения в f32:
//! STR/DEX используют полные целые; CON(-) сохраняет f32 только для DEF,
//! CON/INT(+) percentage — для delta. INT(-) MP использует __ftol2/FISTP64
//! и младший WORD (old полный, new f32), прочие производные INT(-) — f32.
//! Absolute INT(+) повторно проецирует полное новое INT после native cap.
//! Временный negation самого payload восстанавливается до возврата; между
//! этими записями только чистые player getters/setters, поэтому Rust считает
//! тот же результат без промежуточной мутации живого состояния.
//! Остальные ещё не перенесённые тела ниже остаются RAW-комментариями.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::CNotDisappearAfterDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:21
// RVA: 0x001D62A0
// ADDRESS: 005d62a0
// PROTOTYPE: undefined __thiscall CNotDisappearAfterDead(tagNotDisappearAfterDead * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::CNotDisappearAfterDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:30
// RVA: 0x001D62E0
// ADDRESS: 005d62e0
// PROTOTYPE: undefined __thiscall CNotDisappearAfterDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::~CNotDisappearAfterDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:38
// RVA: 0x001D6310
// ADDRESS: 005d6310
// PROTOTYPE: void __thiscall ~CNotDisappearAfterDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:760
// RVA: 0x001D6360
// ADDRESS: 005d6360
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:778
// RVA: 0x001D6420
// ADDRESS: 005d6420
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:876
// RVA: 0x001D64F0
// ADDRESS: 005d64f0
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:889
// RVA: 0x001D6530
// ADDRESS: 005d6530
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:745
// RVA: 0x001D7910
// ADDRESS: 005d7910
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDeadVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:902
// RVA: 0x001D79C0
// ADDRESS: 005d79c0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::use_item
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:842
// RVA: 0x001D7B20
// ADDRESS: 005d7b20
// PROTOTYPE: int __thiscall use_item(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:815
// RVA: 0x001D7C80
// ADDRESS: 005d7c80
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer
