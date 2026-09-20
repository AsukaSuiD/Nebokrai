//! YuanBao-вариант однослотового currency container GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cyuanbao.cpp`. Layout и порядок
//! операций совпадают с `CWallet`, но допустимый catalog index берётся из
//! `YUANBAO`. Общий storage/lifecycle и достигнутый persisted codec реализованы
//! в `cwallet` marker-адаптером; собственная `CS2CContainerObjectMove` граница
//! ниже остаётся RAW.

use super::cwallet::{CSingleCurrencyContainer, CurrencyKind};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct YuanBaoCurrency;

impl CurrencyKind for YuanBaoCurrency {
    const VALIDATE_EMPTY_GOODS: bool = true;

    fn goods_index(factory: &CGoodsFactory) -> u32 {
        factory.get_yuan_bao_index()
    }
}

pub(crate) type CYuanBao = CSingleCurrencyContainer<YuanBaoCurrency>;

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp

// ============================================================================
// FUNCTION: CYuanBao::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:186
// RVA: 0x000D63E0
// ADDRESS: 004d63e0
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:199
// RVA: 0x000D6400
// ADDRESS: 004d6400
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::CYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:19
// RVA: 0x000D6470
// ADDRESS: 004d6470
// PROTOTYPE: undefined __thiscall CYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::~CYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:35
// RVA: 0x000D64D0
// ADDRESS: 004d64d0
// PROTOTYPE: void __thiscall ~CYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::DecreaseGoldCoins
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:429
// RVA: 0x000D6670
// ADDRESS: 004d6670
// PROTOTYPE: int __thiscall DecreaseGoldCoins(ulong param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:56
// RVA: 0x000D6800
// ADDRESS: 004d6800
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:42
// RVA: 0x000D69A0
// ADDRESS: 004d69a0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::AddGoldCoins
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:371
// RVA: 0x000D69F0
// ADDRESS: 004d69f0
// PROTOTYPE: int __thiscall AddGoldCoins(ulong param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cyuanbao.cpp:222
// RVA: 0x000D6C10
// ADDRESS: 004d6c10
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
