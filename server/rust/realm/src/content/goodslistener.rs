//! Listener `CGoodsListener` из WorldServer.
//!
//! Перед каждым обходом `SaveGoodsFiled` назначает byte place. Callback
//! запрашивает позицию товара, игнорирует bool и передаёт младший byte вместе
//! с player в `CDBGoods::SaveGoods`. В safe snapshot позиция уже вычислена
//! конкретным контейнером, поэтому RTTI и raw pointers не повторяются.
//!
//! Неинициализированный place представлен `None`; callback до `set_place`
//! блокируется. Equipment, amount-container и wallet игнорируют возвращаемый
//! `int`, но listener сохраняет его без собственной политики остановки.
//!
//! Listener сам не создаёт player-значение, поэтому player-тип — только
//! параметр `PlayerT` границы `DbGoodsOwner<PlayerT>`.

use crate::content::dbgoods::{DbGoodsOwner, GoodsSaveBlock, GoodsSaveOutcome, GoodsSaveSnapshot};
use crate::content::goodsdb::GoodsObjectSnapshot;
use crate::persistence::rssetup::WorldTdsClient;

pub struct TraversedGoods {
    pub goods: GoodsObjectSnapshot,
    pub position: u8,
}

pub struct GoodsContainerTraversalSnapshot<'snapshot> {
    pub objects: &'snapshot [TraversedGoods],
}

#[derive(Debug)]
pub enum GoodsTraversalBlock {
    UninitializedPlace,
    SaveGoods(GoodsSaveBlock),
}

#[derive(Debug)]
pub enum GoodsTraversalOutcome {
    Returned(i32),
    BlockedMissingFact(GoodsTraversalBlock),
}

pub struct GoodsListener<'owner, 'transaction, PlayerT, G: DbGoodsOwner<PlayerT> + ?Sized> {
    player_id: Option<i32>,
    place: Option<u8>,
    goods_owner: &'owner mut G,
    active_transaction: &'transaction mut WorldTdsClient,
    phantom: std::marker::PhantomData<fn() -> PlayerT>,
}

impl<'owner, 'transaction, PlayerT, G: DbGoodsOwner<PlayerT> + ?Sized>
    GoodsListener<'owner, 'transaction, PlayerT, G>
{
    pub fn new(
        player_id: Option<i32>,
        goods_owner: &'owner mut G,
        active_transaction: &'transaction mut WorldTdsClient,
    ) -> Self {
        Self {
            player_id,
            place: None,
            goods_owner,
            active_transaction,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn set_place(&mut self, place: u8) {
        self.place = Some(place);
    }

    pub async fn on_traversing_container(
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
    pub async fn traverse<PlayerT, G: DbGoodsOwner<PlayerT> + ?Sized>(
        &self,
        listener: &mut GoodsListener<'_, '_, PlayerT, G>,
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
