//! Storage/query/lock core `CAmountLimitGoodsContainer` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/camountlimitgoodscontainer.cpp`.
//! Legacy `stdext::hash_map` хранит GUID→owned `CGoods`, но его traversal идёт
//! по внутреннему list в insertion order. `Vec<CGoods>` сохраняет этот порядок;
//! duplicate GUID заменяет value на прежней позиции. Линейный GUID lookup
//! заменяет хеш-индекс как зрелая стандартная реализация при том же результате.
//!
//! Constructor RVA `0x000FCD60` задаёт limit `1`, пустые goods/locks и
//! регистрирует собственный listener-subobject. В Rust его OnObjectAdded/
//! OnObjectRemoved structurally принадлежат concrete owner-у и не требуют
//! самоссылочного pointer handle. Read-only traversal, lock/unlock и lookup
//! перенесены буквально; locked goods скрыты от public find/get. Add/remove,
//! listener messages, player AI tree, codec и mode-dependent release ниже
//! остаются RAW до замыкания соседних owners.

use super::cgoodscontainer::CGoodsContainer;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CAmountLimitGoodsContainer {
    base: CGoodsContainer,
    goods: Vec<CGoods>,
    locked_goods: Vec<CGuid>,
    goods_amount_limit: u32,
}

impl Default for CAmountLimitGoodsContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CAmountLimitGoodsContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            goods: Vec::new(),
            locked_goods: Vec::new(),
            goods_amount_limit: 1,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.base.set_owner(owner_type, owner_id);
    }

    pub(crate) const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.goods_amount_limit = limit;
    }

    pub(crate) const fn goods_amount_limit(&self) -> u32 {
        self.goods_amount_limit
    }

    pub(crate) fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.goods.iter().fold(0u32, |amount, goods| {
            amount.wrapping_add(u32::from(
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some(),
            ))
        })
    }

    pub(crate) fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.goods_amount_limit <= self.goods_amount(factory)
    }

    pub(crate) fn is_locked(&self, ex_id: CGuid) -> bool {
        self.locked_goods.contains(&ex_id)
    }

    pub(crate) fn lock(&mut self, ex_id: CGuid) -> bool {
        if self.locked_goods.contains(&ex_id) {
            return false;
        }
        self.locked_goods.push(ex_id);
        true
    }

    pub(crate) fn unlock(&mut self, ex_id: CGuid) -> bool {
        let Some(index) = self.locked_goods.iter().position(|locked| *locked == ex_id) else {
            return false;
        };
        self.locked_goods.remove(index);
        true
    }

    pub(crate) fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        if self.locked_goods.contains(&ex_id) {
            return None;
        }
        self.goods
            .iter()
            .find(|goods| goods.identity().ex_id == ex_id)
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.goods_amount_limit <= position {
            return None;
        }
        self.goods
            .get(position as usize)
            .filter(|goods| !self.is_locked(goods.identity().ex_id))
    }

    pub(crate) fn get_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.goods.iter().find(|goods| {
            goods.base_properties_index() == base_properties_index
                && !self.is_locked(goods.identity().ex_id)
        })
    }

    pub(crate) fn get_goods_by_base_properties(&self, base_properties_index: u32) -> Vec<&CGoods> {
        self.goods
            .iter()
            .filter(|goods| {
                goods.base_properties_index() == base_properties_index
                    && !self.is_locked(goods.identity().ex_id)
            })
            .collect()
    }

    pub(crate) fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.goods
            .iter()
            .position(|goods| goods.identity().ex_id == ex_id)
            .map(|position| position as u32)
    }

    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.goods
            .iter()
            .any(|goods| goods.base_properties_index() == base_properties_index)
    }

    pub(crate) fn contents_weight(&self, factory: &CGoodsFactory) -> u32 {
        self.goods.iter().fold(0u32, |weight, goods| {
            weight.wrapping_add(goods.weight(factory))
        })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:518
// RVA: 0x000D6570
// ADDRESS: 004d6570
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:496
// RVA: 0x000DD480
// ADDRESS: 004dd480
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::TraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:143
// RVA: 0x000FC680
// ADDRESS: 004fc680
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:251
// RVA: 0x000FC790
// ADDRESS: 004fc790
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:452
// RVA: 0x000FC820
// ADDRESS: 004fc820
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:580
// RVA: 0x000FC8E0
// ADDRESS: 004fc8e0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:387
// RVA: 0x000FC980
// ADDRESS: 004fc980
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:42
// RVA: 0x000FCDF0
// ADDRESS: 004fcdf0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:79
// RVA: 0x000FCEB0
// ADDRESS: 004fceb0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:104
// RVA: 0x000FCFE0
// ADDRESS: 004fcfe0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:125
// RVA: 0x000FD0E0
// ADDRESS: 004fd0e0
// PROTOTYPE: int __thiscall Clone(CGoodsContainer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:600
// RVA: 0x000FD150
// ADDRESS: 004fd150
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::~CAmountLimitGoodsContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp:35
// RVA: 0x000FD1A0
// ADDRESS: 004fd1a0
// PROTOTYPE: void __thiscall ~CAmountLimitGoodsContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005044a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodscontainer.cpp
// RVA: 0x001044A1
// ADDRESS: 005044a1
// PROTOTYPE: undefined Catch@005044a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
