//! Владелец навыка `CChuckStone` (`0x19D`). Достигнутый путь владельца-монстра
//! использует общий снарядный механизм `monsterprojectile`; проверки оружия,
//! варианты игрока и автоматический повтор ниже остаются RAW до появления
//! настоящего вызова.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp

// ============================================================================
// FUNCTION: CChuckStone::GetAffectRangeMin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:187
// RVA: 0x001387A0
// ADDRESS: 005387a0
// PROTOTYPE: long __thiscall GetAffectRangeMin(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::CChuckStone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:22
// RVA: 0x0013CCE0
// ADDRESS: 0053cce0
// PROTOTYPE: undefined __thiscall CChuckStone(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::~CChuckStone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:33
// RVA: 0x0013CD50
// ADDRESS: 0053cd50
// PROTOTYPE: void __thiscall ~CChuckStone(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:134
// RVA: 0x0013CD70
// ADDRESS: 0053cd70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:152
// RVA: 0x0013CE40
// ADDRESS: 0053ce40
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:573
// RVA: 0x0013CF10
// ADDRESS: 0053cf10
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:115
// RVA: 0x0013CF80
// ADDRESS: 0053cf80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStoneEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:588
// RVA: 0x0013D040
// ADDRESS: 0053d040
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:43
// RVA: 0x0013D520
// ADDRESS: 0053d520
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:501
// RVA: 0x0013D6E0
// ADDRESS: 0053d6e0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:476
// RVA: 0x0013D930
// ADDRESS: 0053d930
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:450
// RVA: 0x0013DA40
// ADDRESS: 0053da40
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:205
// RVA: 0x0013DB30
// ADDRESS: 0053db30
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChuckStone::End
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован в `monsterprojectile`;
// отличающиеся ветви игрока и недостигнутого вызова сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:170
// RVA: 0x0016A330
// ADDRESS: 0056a330
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: GameServer

pub(crate) const CHUCK_STONE_SKILL_ID: u32 = 0x19d;
