//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::CC2SContainerObjectMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:23
// RVA: 0x001BB7D0
// ADDRESS: 005bb7d0
// PROTOTYPE: undefined __thiscall CC2SContainerObjectMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CC2SContainerObjectMove::Receive
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: player↔enhancement и player↔equipment-session decode/owner normalization в message/containermessage.rs
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:44
// RVA: 0x001BB890
// ADDRESS: 005bb890
// PROTOTYPE: int __thiscall Receive(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::GetSource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:325
// RVA: 0x001BBCD0
// ADDRESS: 005bbcd0
// PROTOTYPE: CBaseObject * __thiscall GetSource(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::GetDestination
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:356
// RVA: 0x001BBD60
// ADDRESS: 005bbd60
// PROTOTYPE: CBaseObject * __thiscall GetDestination(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::GetGoods
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: packet/equipment enhancement и equipment-session select, same-source clear и cross-container remove в CPlayer/CGame
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:374
// RVA: 0x001BBDF0
// ADDRESS: 005bbdf0
// PROTOTYPE: CGoods * __thiscall GetGoods(CS2CContainerObjectMove * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::SwapGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:1031
// RVA: 0x001BC620
// ADDRESS: 005bc620
// PROTOTYPE: int __thiscall SwapGoods(CGoods * param_1, CS2CContainerObjectMove * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::RollBack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:1134
// RVA: 0x001BC880
// ADDRESS: 005bc880
// PROTOTYPE: void __thiscall RollBack(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::PutGoods
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: enhancement и typed upgrade/DaKong/compose session shadow placement, shadow→packet/equipment add/rollback в CPlayer/CGame
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:645
// RVA: 0x001BCAB0
// ADDRESS: 005bcab0
// PROTOTYPE: int __thiscall PutGoods(CGoods * param_1, CS2CContainerObjectMove * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CC2SContainerObjectMove::Move
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: полные packet/equipment ↔ enhancement/equipment-session и двусторонние auction-listing transfers, включая burden/rollback/equipment effects; остальные container routes RAW ниже
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message packaging\cc2scontainerobjectmove.cpp:1225
// RVA: 0x001BD230
// ADDRESS: 005bd230
// PROTOTYPE: int __thiscall Move(CS2CContainerObjectMove * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
