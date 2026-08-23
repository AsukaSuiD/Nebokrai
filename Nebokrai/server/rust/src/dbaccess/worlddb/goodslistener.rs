//! Listener `CGoodsListener` из WorldServer, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Перед каждым обходом `SaveGoodsFiled` назначает byte place. Callback
//! запрашивает позицию товара, игнорирует bool и передаёт младший byte вместе
//! с player в `CDBGoods::SaveGoods`. В safe snapshot позиция уже вычислена
//! конкретным контейнером, поэтому RTTI и raw pointers не повторяются.
//!
//! Неинициализированный place представлен `None`; callback до `set_place`
//! блокируется. Equipment, amount-container и wallet игнорируют возвращаемый
//! `int`, но listener сохраняет его без собственной политики остановки.

use crate::dbaccess::worlddb::dbgoods::{
    DbGoodsOwner, GoodsObjectSnapshot, GoodsSaveBlock, GoodsSaveOutcome, GoodsSaveSnapshot,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

pub(crate) struct TraversedGoods {
    pub(crate) goods: GoodsObjectSnapshot,
    pub(crate) position: u8,
}

pub(crate) struct GoodsContainerTraversalSnapshot<'snapshot> {
    pub(crate) objects: &'snapshot [TraversedGoods],
}

#[derive(Debug)]
pub(crate) enum GoodsTraversalBlock {
    UninitializedPlace,
    SaveGoods(GoodsSaveBlock),
}

#[derive(Debug)]
pub(crate) enum GoodsTraversalOutcome {
    Returned(i32),
    BlockedMissingFact(GoodsTraversalBlock),
}

pub(crate) struct GoodsListener<'owner, 'transaction, G: DbGoodsOwner + ?Sized> {
    player_id: Option<i32>,
    place: Option<u8>,
    goods_owner: &'owner mut G,
    active_transaction: &'transaction mut WorldTdsClient,
}

impl<'owner, 'transaction, G: DbGoodsOwner + ?Sized> GoodsListener<'owner, 'transaction, G> {
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

    pub(crate) fn set_place(&mut self, place: u8) {
        self.place = Some(place);
    }

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
