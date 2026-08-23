//! YuanBao-специализация однослотовой currency shadow GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cshadowyuanbao.cpp`. Собственные
//! executable отличия от `CShadowWallet` ограничены catalog
//! `YUANBAO` и source extend `5`; storage, partial-mutation order и listener
//! callbacks переиспользуют общий typed adapter. Нижний pseudocode
//! оставлен как первичная документация.

use super::cshadowwallet::{CShadowCurrencyContainer, ShadowCurrencyKind};
use super::cyuanbao::YuanBaoCurrency;

impl ShadowCurrencyKind for YuanBaoCurrency {
    const SOURCE_CONTAINER_EXTEND_ID: i32 = 5;
}

pub(crate) type CShadowYuanBao = CShadowCurrencyContainer<YuanBaoCurrency>;

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cshadowyuanbao.cpp

// ============================================================================
// FUNCTION: CShadowYuanBao::CShadowYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cshadowyuanbao.cpp:20
// RVA: 0x002049B0
// ADDRESS: 006049b0
// PROTOTYPE: undefined __thiscall CShadowYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShadowYuanBao::~CShadowYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cshadowyuanbao.cpp:33
// RVA: 0x00204A10
// ADDRESS: 00604a10
// PROTOTYPE: void __thiscall ~CShadowYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShadowYuanBao::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cshadowyuanbao.cpp:48
// RVA: 0x00204AD0
// ADDRESS: 00604ad0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShadowYuanBao::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cshadowyuanbao.cpp:40
// RVA: 0x00204D50
// ADDRESS: 00604d50
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
