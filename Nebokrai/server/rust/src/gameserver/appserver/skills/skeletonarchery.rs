//! Стрельба скелета `CSkeletonArchery` (`0x1a1`) для владельца-монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/skeletonarchery.cpp`. Модуль сохраняет проверку прямого
//! пути, запрет движения при подготовке, время полёта по первой преграде,
//! текущую клетку объектной цели в момент прилёта, формулу и порядок RNG.
//! Общая допустимость цели, защита и применение удара принадлежат узкому
//! `monsterattack`; `CGame` только возвращает регион между ударами и применяет
//! межвладельческие смерти. Варианты игрока с проверкой оружия, координатные
//! перегрузки и автоматический повтор остаются RAW до реального вызывающего пути.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:134
// RVA: 0x00138600
// ADDRESS: 00538600
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:152
// RVA: 0x001386D0
// ADDRESS: 005386d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:115
// RVA: 0x001387F0
// ADDRESS: 005387f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// IMPLEMENTED: объектная ветвь владельца-монстра материализована ниже;
// вариант игрока сохранён в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArcheryEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: действия подготовки и выстрела владельца-монстра
// материализованы ниже; клиентские ошибки игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:590
// RVA: 0x001388B0
// ADDRESS: 005388b0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: ограничение прямого пути владельца-монстра материализовано
// ниже; проверка оружия относится к недостигнутому вызывающему пути игрока.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:43
// RVA: 0x00138D90
// ADDRESS: 00538d90
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: формула владельца-монстра и оба RNG-вызова материализованы ниже;
// дополнительные атаки и критический удар игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:503
// RVA: 0x00138F50
// ADDRESS: 00538f50
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:478
// RVA: 0x001391A0
// ADDRESS: 005391a0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// IMPLEMENTED: удар источника-монстра по одной цели материализован ниже;
// разрешения игрока сохранены в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:452
// RVA: 0x001392B0
// ADDRESS: 005392b0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// IMPLEMENTED: упорядоченный проход клетки источником-монстром материализован ниже;
// неизвестные разновидности `CMoveShape` сохранены в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: объектная последовательность владельца-монстра, подготовка,
// полёт и прилёт материализованы ниже; автоматический повтор сохранён в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:205
// RVA: 0x001393A0
// ADDRESS: 005393a0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

pub(crate) const SKELETON_ARCHERY_SKILL_ID: u32 = 0x1a1;
