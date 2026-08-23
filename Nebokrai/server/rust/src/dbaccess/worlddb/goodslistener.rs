//! Listener сохранения вещей `CGoodsListener` WorldServer.
//!
//! Constructor, destructor и `OnTraversingContainer` входят в контракт owner-а
//! из WorldServer EXE/PDB; остальные функции в этот owner не входят.
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
//! `m_btPlace` непосредственно перед каждым действующим обходом. `Option<u8>`
//! не придумывает исходный байт: вызов callback с подходящими объектами до
//! `set_place` остаётся локальным `typed boundary`. Не читаемый
//! `m_byContainerType` не представлен в Rust API.
//!
//! Callback сначала выполняет два RTTI cast-а, обнуляет
//! `unsigned long dwPosition`, вызывает virtual `QueryGoodsPosition`, игнорирует
//! его bool и передаёт младший byte результата в `SaveGoods`. Frozen Rust-
//! snapshot заранее хранит уже доказанный position конкретного container-а:
//! это узкий compatibility-layer без RTTI/raw pointer, а не новая политика
//! traversal. Null/не-goods объекты отсутствуют только в materialized safe-
//! state; исходный общий callback не объявляется универсально заменённым.
//!
//! Действующие `CEquipmentContainer`, `CAmountLimitGoodsContainer` и `CWallet`
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
 /// Младший байт оригинал `unsigned long` результата `QueryGoodsPosition`.
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

 /// Сохраняет одну действующую вещь и возвращает исходный callback `int`.
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
