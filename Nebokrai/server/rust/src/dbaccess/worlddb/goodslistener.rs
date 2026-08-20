//! Listener сохранения вещей `CGoodsListener` исторического WorldServer.
//!
//! Конструктор RVA `0x00119240`, деструктор RVA `0x001192C0` и
//! `OnTraversingContainer` RVA `0x00119320` имеют статус `IMPLEMENTED`;
//! остальные функции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp`.
//!
//! PDB задаёт layout исходного listener-а размером `0x14`: connection по
//! `+0x4`, nullable `CPlayer*` по `+0x8`, `m_btPlace` по `+0xC`, не читаемый
//! callback-ом `m_byContainerType` по `+0xD` и встроенный `CDBGoods` по `+0x10`.
//! Единственный project-call конструктора находится в `CDBGoods::SaveGoodsFiled`
//! после проверки connection на null; Rust-ссылка сохраняет этот доказанный
//! live-path. COM `AddRef/Release`, пустое состояние встроенного DB-owner-а и
//! compiler cleanup заменены borrow-ами и `Drop`, а не Windows FFI.
//!
//! Конструктор не инициализировал два byte-поля, но `SaveGoodsFiled` назначает
//! `m_btPlace` непосредственно перед каждым достигнутым обходом. `Option<u8>`
//! не придумывает исходный байт: вызов callback с подходящими объектами до
//! `set_place` остаётся локальным `BLOCKED_MISSING_FACT`. Не читаемый
//! `m_byContainerType` не представлен в Rust API.
//!
//! Exact `0x00519320..0x005193BC` сначала выполняет два RTTI cast-а, обнуляет
//! `unsigned long dwPosition`, вызывает virtual `QueryGoodsPosition`, игнорирует
//! его bool и передаёт младший byte результата в `SaveGoods`. Frozen Rust-
//! snapshot заранее хранит уже доказанный position конкретного container-а:
//! это узкий compatibility-layer без RTTI/raw pointer, а не новая политика
//! traversal. Null/не-goods объекты отсутствуют только в materialized safe-
//! state; исходный общий callback не объявляется универсально заменённым.
//!
//! Достигнутые `CEquipmentContainer`, `CAmountLimitGoodsContainer` и `CWallet`
//! игнорируют возврат listener-а при traversal. Это наблюдение принадлежит
//! следующему владельцу `SaveGoodsFiled`: текущий API возвращает точный `int`,
//! но сам не назначает политику остановки обхода. ADO, STL и `Unwind@...` ниже
//! являются library/compiler noise и не получают отдельных Rust-аналогов.

use crate::dbaccess::worlddb::dbgoods::{
    DbGoodsOwner, GoodsObjectSnapshot, GoodsSaveBlock, GoodsSaveOutcome, GoodsSaveSnapshot,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

/// Один уже позиционированный элемент concrete goods-container traversal.
pub(crate) struct TraversedGoods {
    pub(crate) goods: GoodsObjectSnapshot,
    /// Младший байт exact `unsigned long` результата `QueryGoodsPosition`.
    pub(crate) position: u8,
}

/// Caller-owned снимок одного синхронного `TraversingContainer`.
pub(crate) struct GoodsContainerTraversalSnapshot<'snapshot> {
    /// Элементы уже расположены в доказанном порядке конкретного container-а.
    pub(crate) objects: &'snapshot [TraversedGoods],
}

/// Неразрешённая legacy-граница callback-а.
#[derive(Debug)]
pub(crate) enum GoodsTraversalBlock {
    /// В исходном конструкторе `m_btPlace` оставался неинициализированным.
    UninitializedPlace,
    /// Вложенный `CDBGoods::SaveGoods` достиг собственной неизвестной границы.
    SaveGoods(GoodsSaveBlock),
}

/// Точный `int` callback-а либо локальная граница неопределённого исходного byte.
#[derive(Debug)]
pub(crate) enum GoodsTraversalOutcome {
    Returned(i32),
    BlockedMissingFact(GoodsTraversalBlock),
}

/// Linux/Rust-замена живого состояния `CGoodsListener`.
pub(crate) struct GoodsListener<'owner, 'transaction, G: DbGoodsOwner + ?Sized> {
    player_id: Option<i32>,
    place: Option<u8>,
    goods_owner: &'owner mut G,
    active_transaction: &'transaction mut WorldTdsClient,
}

impl<'owner, 'transaction, G: DbGoodsOwner + ?Sized> GoodsListener<'owner, 'transaction, G> {
    /// Создаёт listener; отсутствие player сохраняет исходную null-ветку.
    pub(crate) fn new(
        player_id: Option<i32>,
        goods_owner: &'owner mut G,
        active_transaction: &'transaction mut WorldTdsClient,
    ) -> Self {
        Self {
            player_id,
            place: None,
            goods_owner,
            active_transaction,
        }
    }

    /// Соответствует присваиванию `m_btPlace` владельцем перед traversal.
    pub(crate) fn set_place(&mut self, place: u8) {
        self.place = Some(place);
    }

    /// Сохраняет одну достигнутую вещь и возвращает исходный callback `int`.
    pub(crate) async fn on_traversing_container(
        &mut self,
        object: &TraversedGoods,
    ) -> GoodsTraversalOutcome {
        let Some(player_id) = self.player_id else {
            return GoodsTraversalOutcome::Returned(0);
        };
        let Some(place) = self.place else {
            return GoodsTraversalOutcome::BlockedMissingFact(
                GoodsTraversalBlock::UninitializedPlace,
            );
        };

        let snapshot = GoodsSaveSnapshot {
            player_id,
            goods: &object.goods,
            place,
            position: object.position,
        };
        match self
            .goods_owner
            .save_goods(&snapshot, self.active_transaction)
            .await
        {
            GoodsSaveOutcome::Saved => GoodsTraversalOutcome::Returned(1),
            GoodsSaveOutcome::Failed => GoodsTraversalOutcome::Returned(0),
            GoodsSaveOutcome::BlockedMissingFact(block) => {
                GoodsTraversalOutcome::BlockedMissingFact(GoodsTraversalBlock::SaveGoods(block))
            }
        }
    }
}

impl GoodsContainerTraversalSnapshot<'_> {
    /// Вызывает listener для всех элементов, как исходный void traversal.
    pub(crate) async fn traverse<G: DbGoodsOwner + ?Sized>(
        &self,
        listener: &mut GoodsListener<'_, '_, G>,
    ) -> Result<(), GoodsTraversalBlock> {
        for object in self.objects {
            match listener.on_traversing_container(object).await {
                GoodsTraversalOutcome::Returned(_) => {}
                GoodsTraversalOutcome::BlockedMissingFact(block) => return Err(block),
            }
        }
        Ok(())
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp

// ============================================================================
// FUNCTION: Field20::GetActualSize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6600
// ADDRESS: 004e6600
// PROTOTYPE: long __thiscall GetActualSize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Recordset15::Open
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E66B0
// ADDRESS: 004e66b0
// PROTOTYPE: long __thiscall Open(_variant_t * param_1, _variant_t * param_2, CursorTypeEnum param_3, LockTypeEnum param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Recordset15::Update
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6730
// ADDRESS: 004e6730
// PROTOTYPE: long __thiscall Update(_variant_t * param_1, _variant_t * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Recordset15::PutCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E67A0
// ADDRESS: 004e67a0
// PROTOTYPE: void __thiscall PutCollect(_variant_t * param_1, _variant_t * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Fields15::GetItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E69F0
// ADDRESS: 004e69f0
// PROTOTYPE: _com_ptr_t<_com_IIID<Field,&struct___s_GUID_const__GUID_00000569_0000_0010_8000_00aa006d2ea4>_> __thiscall GetItem(_variant_t * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Recordset15::GetFields
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6BB0
// ADDRESS: 004e6bb0
// PROTOTYPE: _com_ptr_t<_com_IIID<Fields,&struct___s_GUID_const__GUID_00000564_0000_0010_8000_00aa006d2ea4>_> __thiscall GetFields(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Connection15::Execute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6C60
// ADDRESS: 004e6c60
// PROTOTYPE: _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> __thiscall Execute(_bstr_t param_1, tagVARIANT * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::tagLargess::~tagLargess
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6CF0
// ADDRESS: 004e6cf0
// PROTOTYPE: void __thiscall ~tagLargess(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWorldRegion::tagNpc::~tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6DD0
// ADDRESS: 004e6dd0
// PROTOTYPE: void __thiscall ~tagNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::tagLargess::tagLargess
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E6E20
// ADDRESS: 004e6e20
// PROTOTYPE: undefined __thiscall tagLargess(tagLargess * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e8c86
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x000E8C86
// ADDRESS: 004e8c86
// PROTOTYPE: undefined Catch@004e8c86()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00535be6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x00135BE6
// ADDRESS: 00535be6
// PROTOTYPE: undefined Unwind@00535be6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535bf1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x00135BF1
// ADDRESS: 00535bf1
// PROTOTYPE: undefined Unwind@00535bf1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@00535e80
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\goodslistener.cpp
// RVA: 0x00135E80
// ADDRESS: 00535e80
// PROTOTYPE: undefined Unwind@00535e80()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: WorldServer
