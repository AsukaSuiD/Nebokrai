//! Владелец wallet-container исторического `WorldServer`.
//!
//! Статус constructor-state `CWallet::CWallet` RVA `0x000D5F60`,
//! destructor ownership RVA `0x000D5FC0` и `GetGoldCoinsAmount` RVA
//! `0x000D5F40` — `IMPLEMENTED`; игровые операции и остальной корпус ниже
//! остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:18,34,349`.
//!
//! Exact PDB задаёт единственное собственное поле `m_pGoldCoins` по `+0x24`;
//! inherited `CGoodsContainer` хранит signed owner type/ID по `+0x14/+0x18`.
//! Constructor начинает с owner `0/0` и null goods, а destructor сначала
//! выполняет base `Release`, затем `GarbageCollect`. Rust `Option<Box<CGoods>>`
//! заменяет только nullable pointer и deleting-destructor: обычный `Drop`
//! уничтожает товар один раз. Встроенный secondary listener регистрируется
//! constructor-ом, но оба его callbacks сведены линкером к доказанному
//! `mov eax,1; ret 0xC` по RVA `0x000DBD10`; отдельного состояния он не имеет.

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;

use super::super::goods::cgoods::{CGoods, GoodsDbSnapshotBlock};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;

/// Достигнутое состояние исходного `CWallet`, не копия его 32-битного ABI.
pub(crate) struct CWallet {
    pub(super) owner_type: i32,
    pub(super) owner_id: i32,
    pub(super) gold_coins: Option<Box<CGoods>>,
}

impl CWallet {
    /// Создаёт точные defaults base-owner-а и пустого wallet slot-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            owner_type: 0,
            owner_id: 0,
            gold_coins: None,
        }
    }

    /// Возвращает количество единственного gold-coins товара либо ноль.
    pub(crate) const fn get_gold_coins_amount(&self) -> u32 {
        match &self.gold_coins {
            Some(goods) => goods.get_amount(),
            None => 0,
        }
    }

    /// Замораживает nullable wallet-slot для DB traversal с позицией `0`.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, GoodsDbSnapshotBlock> {
        self.gold_coins
            .iter()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: 0,
                })
            })
            .collect()
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp

// ============================================================================
// FUNCTION: CWallet::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:211
// RVA: 0x000D5E80
// ADDRESS: 004d5e80
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:224
// RVA: 0x000D5EA0
// ADDRESS: 004d5ea0
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::GetGoldCoinsAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:349
// RVA: 0x000D5F40
// ADDRESS: 004d5f40
// PROTOTYPE: ulong __thiscall GetGoldCoinsAmount(void)
//
// IMPLEMENTED выше; null slot возвращает unsigned ноль.

// ============================================================================
// FUNCTION: CWallet::CWallet
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:18
// RVA: 0x000D5F60
// ADDRESS: 004d5f60
// PROTOTYPE: undefined __thiscall CWallet(void)
//
// IMPLEMENTED выше; no-op embedded listener не требует отдельного Rust-state.

// ============================================================================
// FUNCTION: CWallet::~CWallet
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:34
// RVA: 0x000D5FC0
// ADDRESS: 004d5fc0
// PROTOTYPE: void __thiscall ~CWallet(void)
//
// IMPLEMENTED через `Option<Box<CGoods>>` и обычный Rust `Drop`.

// ============================================================================
// FUNCTION: CWallet::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:83
// RVA: 0x000D6090
// ADDRESS: 004d6090
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:55
// RVA: 0x000D61F0
// ADDRESS: 004d61f0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::AddGoldCoinOfLargess
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:422
// RVA: 0x000D6290
// ADDRESS: 004d6290
// PROTOTYPE: int __thiscall AddGoldCoinOfLargess(CGoods * param_1, ulong param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:41
// RVA: 0x000D63B0
// ADDRESS: 004d63b0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp:247
// RVA: 0x000D63F0
// ADDRESS: 004d63f0
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00535620
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp
// RVA: 0x00135620
// ADDRESS: 00535620
// PROTOTYPE: undefined Unwind@00535620()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053562b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cwallet.cpp
// RVA: 0x0013562B
// ADDRESS: 0053562b
// PROTOTYPE: undefined Unwind@0053562b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
