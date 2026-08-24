//! Достигнутая session-state часть GameServer `CPersonalShopSeller`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и owner
//! `server/gameserver/appserver/session/cpersonalshopseller.cpp` подтверждают
//! constructor и `OnPlugInserted`: seller владеет пустым именем, закрытым
//! shop-флагом и shadow-контейнером 8×6, который получает owner `(10, plug)` и
//! extend ID `plug << 8`. Его listener identity — сам plug; player packet и
//! equipment подписываются тем же identity при insertion. Safe Rust storage
//! заменяет указатели/RTTI, не меняя этих наблюдаемых связей. Торговые операции
//! и terminal lifecycle остаются RAW ниже до своих message-сценариев.

use crate::gameserver::appserver::container::ccontainer::ContainerListenerHandle;
use crate::gameserver::appserver::container::cvolumelimitgoodsshadowcontainer::CVolumeLimitGoodsShadowContainer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPersonalShopSeller {
    shop_name: Vec<u8>,
    goods: CVolumeLimitGoodsShadowContainer,
    shop_opened: bool,
}

impl CPersonalShopSeller {
    pub(crate) fn inserted(plug_id: i32) -> Self {
        let mut goods = CVolumeLimitGoodsShadowContainer::new();
        goods.set_container_dimensions(8, 6);
        goods
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(10, plug_id);
        goods
            .base_mut()
            .base_mut()
            .set_container_extend_id(plug_id.wrapping_shl(8));
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let _self_listener = goods
            .base_mut()
            .base_mut()
            .base_mut()
            .base_mut()
            .add_listener(listener);
        Self {
            shop_name: Vec::new(),
            goods,
            shop_opened: false,
        }
    }

    pub(crate) const fn goods(&self) -> &CVolumeLimitGoodsShadowContainer {
        &self.goods
    }

    pub(crate) fn shop_name(&self) -> &[u8] {
        &self.shop_name
    }

    pub(crate) const fn shop_opened(&self) -> bool {
        self.shop_opened
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp

// ============================================================================
// FUNCTION: CPersonalShopSeller::GetShopName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:265
// RVA: 0x000EA3B0
// ADDRESS: 004ea3b0
// PROTOTYPE: char * __thiscall GetShopName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::GetContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:160
// RVA: 0x001045C0
// ADDRESS: 005045c0
// PROTOTYPE: CContainer * __thiscall GetContainer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::CloseDown
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:232
// RVA: 0x001045D0
// ADDRESS: 005045d0
// PROTOTYPE: int __thiscall CloseDown(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::IsPlugAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:47
// RVA: 0x00104780
// ADDRESS: 00504780
// PROTOTYPE: int __thiscall IsPlugAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::OnPlugEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:115
// RVA: 0x00104890
// ADDRESS: 00504890
// PROTOTYPE: int __thiscall OnPlugEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::OpenForBusiness
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:215
// RVA: 0x00104990
// ADDRESS: 00504990
// PROTOTYPE: int __thiscall OpenForBusiness(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::GetGoodsPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:291
// RVA: 0x00104B40
// ADDRESS: 00504b40
// PROTOTYPE: int __thiscall GetGoodsPrice(CGUID * param_1, ulong * param_2, ulong * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::GetValidGoodsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:724
// RVA: 0x00104B90
// ADDRESS: 00504b90
// PROTOTYPE: ulong __thiscall GetValidGoodsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::PurchaseStep2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:749
// RVA: 0x00104C50
// ADDRESS: 00504c50
// PROTOTYPE: bool __thiscall PurchaseStep2(CGUID * param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::GetGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:682
// RVA: 0x00105040
// ADDRESS: 00505040
// PROTOTYPE: int __thiscall GetGoodsList(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:147
// RVA: 0x00105160
// ADDRESS: 00505160
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::SetShopName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:251
// RVA: 0x001051E0
// ADDRESS: 005051e0
// PROTOTYPE: int __thiscall SetShopName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::Purchase
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:308
// RVA: 0x00105210
// ADDRESS: 00505210
// PROTOTYPE: CGoods * __thiscall Purchase(CGUID * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:180
// RVA: 0x00106070
// ADDRESS: 00506070
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::~CPersonalShopSeller
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:40
// RVA: 0x00106410
// ADDRESS: 00506410
// PROTOTYPE: void __thiscall ~CPersonalShopSeller(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPersonalShopSeller::SetGoodsPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cpersonalshopseller.cpp:272
// RVA: 0x00106680
// ADDRESS: 00506680
// PROTOTYPE: int __thiscall SetGoodsPrice(CGUID * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
