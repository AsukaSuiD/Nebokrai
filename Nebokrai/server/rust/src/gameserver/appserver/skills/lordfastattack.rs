//! Быстрая атака владыки `CLordFastAttack` (`0x1f5`) для пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordfastattack.cpp`. Достигнутый путь совпадает с
//! `CMonsterFastAttack`: начало, задержка, два удара и их RNG-последовательность
//! проходят общий узкий семейный механизм с собственным ID. Варианты игрока и
//! координатные перегрузки остаются в исходном материале ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.h

// ============================================================================
// FUNCTION: CLordFastAttack::End
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение пути монстра выполняет `finish_base_attack_cast`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:163
// RVA: 0x00112B50
// ADDRESS: 00512b50
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::CLordFastAttack
// STATUS: PARTIALLY_IMPLEMENTED
// Идентификатор достигнутого пути задаёт `LORD_FAST_ATTACK_SKILL_ID`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:18
// RVA: 0x00130380
// ADDRESS: 00530380
// PROTOTYPE: undefined __thiscall CLordFastAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::~CLordFastAttack
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение пути монстра выполняет `finish_base_attack_cast`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:28
// RVA: 0x001303F0
// ADDRESS: 005303f0
// PROTOTYPE: void __thiscall ~CLordFastAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:125
// RVA: 0x00130410
// ADDRESS: 00530410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:144
// RVA: 0x001304E0
// ADDRESS: 005304e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный путь монстра проходит общий механизм в `monsterbaseattack`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:105
// RVA: 0x001305E0
// ADDRESS: 005305e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttackEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1 пути монстра формирует общий механизм быстрой атаки.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:381
// RVA: 0x001306A0
// ADDRESS: 005306a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Перезарядка, дальность и блокировка движения пути монстра выполняются
// общим механизмом ближней атаки.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:37
// RVA: 0x00130BB0
// ADDRESS: 00530bb0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра и точная последовательность RNG выполняются общим механизмом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:313
// RVA: 0x00130D60
// ADDRESS: 00530d60
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Удары монстра проходят общий упорядоченный владелец применения.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:289
// RVA: 0x00130FB0
// ADDRESS: 00530fb0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Задержка, первый и второй удары пути монстра выполняются общим механизмом;
// ветвь игрока остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:179
// RVA: 0x001310C0
// ADDRESS: 005310c0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: GameServer

pub(crate) const LORD_FAST_ATTACK_SKILL_ID: u32 = 0x1f5;
