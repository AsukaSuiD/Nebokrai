//! Достигнутый контракт навыка боевого духа `CLifeShield`.
//!
//! Навык `544` расходует `GAP_BF_MP` экипированного боевого духа, немедленно
//! рассылает изменённый товар, затем создаёт упорядоченное защитное состояние.
//! Любое завершение состояния добавляет краткоживущий `CCureState`.

pub(crate) const LIFE_SHIELD_SKILL_ID: u32 = 544;
pub(crate) const LIFE_SHIELD_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const LIFE_SHIELD_VISUAL_OBJECT_TYPE: i32 = 700;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_STATE_HP: u32 = 10_010;
pub(crate) const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
pub(crate) const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.h

// ============================================================================
// FUNCTION: CLifeShield::CLifeShield
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:20
// RVA: 0x00118100
// ADDRESS: 00518100
// PROTOTYPE: undefined __thiscall CLifeShield(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::~CLifeShield
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:28
// RVA: 0x00118170
// ADDRESS: 00518170
// PROTOTYPE: void __thiscall ~CLifeShield(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:122
// RVA: 0x00118190
// ADDRESS: 00518190
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:139
// RVA: 0x00118260
// ADDRESS: 00518260
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:104
// RVA: 0x00118360
// ADDRESS: 00518360
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShieldEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:311
// RVA: 0x00118430
// ADDRESS: 00518430
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:39
// RVA: 0x00118850
// ADDRESS: 00518850
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLifeShield::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshield.cpp:174
// RVA: 0x00118A60
// ADDRESS: 00518a60
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//













// COMPONENT_VARIANT_END: GameServer
