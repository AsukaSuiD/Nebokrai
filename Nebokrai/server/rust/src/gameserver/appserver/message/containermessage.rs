//! Диспетчер сообщений о переносе объектов между игровыми контейнерами.
//!
//! Источник: `gameserver.exe`, `GameServer.pdb` и исходный владелец
//! `server/gameserver/appserver/message/containermessage.cpp`. Диспетчер сохраняет
//! точную нормализацию полей `0x90301`, проверки владельца и состояния игрока,
//! порядок удаления, добавления, отката и клиентских, World- и Billing-отправок.
//! Частичный успех и необратимое удаление остаются в специализированных
//! результатах владельцев только там, где вызывающая сторона выбирает дальнейшее
//! действие. Сведения об уже выполненных операциях публикуются через `tracing` и
//! не возвращаются накопительными отчётами. Отложенных эффектов в этом владельце
//! нет, поэтому `GameEffectJournal` здесь намеренно не используется.
//!
//! Неизвестным остаётся полный снимок игрока `0x6080E`: его исходный владелец пока
//! недоступен, поэтому диспетчер не имитирует эту отправку.

use crate::gameserver::appserver::container::ccontainer::ContainerListenerHandle;
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cdepot::{DepotGoodsAddBlock, DepotGoodsAddOutcome};
use crate::gameserver::appserver::container::cequipmentcontainer::EquipmentRemovedEvent;
use crate::gameserver::appserver::container::cfairycontainer::FairyContainerRemoveOutcome;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::{
    ShadowPresenceReport, ShadowRemovedReport,
};
use crate::gameserver::appserver::container::cvolumelimitgoodscontainer::VolumeGoodsAddOutcome;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::{
    BattleFairyEquipmentMutationReport, CPlayer, EnhancementSelectionReport,
    PlayerEquipmentAddReport, PlayerEquipmentDelivery, PlayerEquipmentRemoveEffect,
    PlayerEquipmentRemoveReport, PlayerProgress,
};
use crate::gameserver::appserver::session::csessionfactory::{
    EquipmentSessionShadowAddBlock, EquipmentSessionShadowAdded, PersonalShopShadowAddBlock,
    PersonalShopShadowAdded,
};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    AuctionGoodsInventoryBlock, AuctionGoodsInventoryRollback, BankCurrencyTransferBlock,
    BattleFairyStorageRemoval, BattleFairyTransferAddition, BattleFairyTransferBlock, CGame,
    CiQingComposeStorageRemoval, CiQingComposeTransferAddition, CiQingComposeTransferBlock,
    DepotStorageRemoval, DepotStorageTransferAddition, DepotStorageTransferBlock,
    FairyStorageRemoval, FairyStorageTransferAddition, FairyStorageTransferBlock,
    GameContainerMessageRuntime, GroundGoodsMoveBlock, HandAuctionListingBlock,
    HandContainerMoveBlock, PlayerHandMoveBlock,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const CONTAINER_OBJECT_MOVE: u32 = 0x0009_0301;
const CLIENT_CONTAINER_OBJECT_MOVE: i32 = 0x000c_0101;
const PLAYER_CONTAINER_TYPE: i32 = 400;
const GOODS_OBJECT_TYPE: i32 = 700;
const ENHANCEMENT_EXTEND_ID: i32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameContainerMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContainerObjectMoveRequest {
    pub(crate) source_container_type: i32,
    pub(crate) source_container_id: i32,
    pub(crate) source_container_extend_id: i32,
    pub(crate) source_position: u32,
    pub(crate) destination_container_type: i32,
    pub(crate) destination_container_id: i32,
    pub(crate) destination_container_extend_id: i32,
    pub(crate) destination_position: u32,
    pub(crate) object_type: i32,
    pub(crate) object_id: CGuid,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementMoveReceiveBlock {
    InvalidObjectType,
    ZeroAmount,
    InvalidExtendId,
    SameContainer,
    ForbiddenRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnhancementMessageRoute {
    BankCurrencyTransfer,
    DepotStorageTransfer,
    HandContainerMove,
    PlayerHandMove,
    FairyStorageTransfer,
    BattleFairyTransfer,
    CiQingComposeTransfer,
    AuctionGoodsInventoryReturn,
    AuctionMoneyReturn,
    HandAuctionListingTransfer,
    GroundDrop,
    GroundPickup,
    EnhancementSelect,
    EnhancementClear,
    EnhancementTransfer,
    EquipmentSessionSelect,
    EquipmentSessionClear,
    EquipmentSessionTransfer,
    PersonalShopSelect,
    PersonalShopClear,
    AuctionListingMove,
    AuctionListingWithdrawal,
    TradeOfferAdd,
    TradeOfferRemove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingTransferRemoval {
    Packet {
        owner_type: i32,
        owner_id: i32,
        position: u32,
        amount: u32,
        listeners: Vec<ContainerListenerHandle>,
    },
    Equipment {
        event: EquipmentRemovedEvent,
        effects: Vec<PlayerEquipmentRemoveEffect>,
        deliveries: Vec<PlayerEquipmentDelivery>,
    },
    AuctionGoods {
        owner_type: i32,
        owner_id: i32,
        position: u32,
        amount: u32,
        listeners: Vec<ContainerListenerHandle>,
    },
    Depot(DepotStorageRemoval),
    Fairy(FairyStorageRemoval),
    BattleFairy(BattleFairyStorageRemoval),
    Compose(CiQingComposeStorageRemoval),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuctionListingTransferReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) removal: AuctionListingTransferRemoval,
    pub(crate) destination: VolumeGoodsAddOutcome,
    pub(crate) listing_slot_zero_was_empty: bool,
    pub(crate) previous_last_operated: (u32, u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingTransferBlock {
    UnsupportedSourceContainer { extend_id: i32 },
    InvalidDestinationPosition { position: u32 },
    MissingSourceGoods,
    InvalidCurrency,
    MissingBaseProperties,
    AuctionLimitRejected,
    PacketRemovalFailed,
    AuctionGoodsRemovalFailed,
    DepotRemovalFailed,
    FairyRemovalFailed(FairyContainerRemoveOutcome),
    InvalidBattleFairyCell { position: u32 },
    BattleFairyRemovalFailed(BattleFairyEquipmentMutationReport),
    ComposeRemovalFailed,
    EquipmentRemovalFailed(PlayerEquipmentRemoveReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuctionListingWithdrawalRemoval {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingWithdrawalOutcome {
    Moved {
        addition: CiQingComposeTransferAddition,
        destination_position: u32,
        destination_goods: ShapeIdentity,
        amount: u32,
    },
    RolledBack {
        rejected: CiQingComposeTransferAddition,
        restored: VolumeGoodsAddOutcome,
    },
    GoodsCollected {
        rejected: CiQingComposeTransferAddition,
        rollback: VolumeGoodsAddOutcome,
        goods: ShapeIdentity,
        notification_delivery: i32,
    },
}

#[must_use = "withdrawal report сохраняет listing ownership, destination effects и rollback"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuctionListingWithdrawalReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) removal: AuctionListingWithdrawalRemoval,
    pub(crate) outcome: AuctionListingWithdrawalOutcome,
    pub(crate) previous_last_operated: Option<(u32, u32)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingWithdrawalBlock {
    UnsupportedDestinationContainer { extend_id: i32 },
    MissingSourceGoods,
    BurdenExceeded,
    InvalidBattleFairyCell { position: u32 },
    RemovalFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementTransferRemoval {
    Packet {
        owner_type: i32,
        owner_id: i32,
        position: u32,
        amount: u32,
        listeners: Vec<ContainerListenerHandle>,
    },
    Equipment {
        event: EquipmentRemovedEvent,
        effects: Vec<PlayerEquipmentRemoveEffect>,
        deliveries: Vec<PlayerEquipmentDelivery>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementTransferAddition {
    Packet(VolumeGoodsAddOutcome),
    Equipment(PlayerEquipmentAddReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementTransferOutcome {
    Moved(EnhancementTransferAddition),
    RolledBack {
        rejected: EnhancementTransferAddition,
        restored: EnhancementTransferAddition,
    },
    GoodsCollected {
        rejected: EnhancementTransferAddition,
        rollback: EnhancementTransferAddition,
        goods: ShapeIdentity,
        notification_delivery: i32,
    },
}

#[must_use = "transfer report сохраняет ownership, equipment effects, shadow removal и rollback"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementTransferReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: crate::gameserver::appserver::container::ccontainer::PreviousContainer,
    pub(crate) removal: EnhancementTransferRemoval,
    pub(crate) shadow:
        crate::gameserver::appserver::container::cgoodsshadowcontainer::ShadowRemovedReport,
    pub(crate) delete_shadow_delivery: i32,
    pub(crate) outcome: EnhancementTransferOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementTransferBlock {
    MissingPlayer,
    MissingShadow,
    MissingSourceGoods,
    UnsupportedSourceContainer { extend_id: i32 },
    UnsupportedDestinationContainer { extend_id: i32 },
    PacketRemovalFailed,
    EquipmentRemovalFailed(crate::gameserver::appserver::player::PlayerEquipmentRemoveReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionSelectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) added: EquipmentSessionShadowAdded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionSelectionBlock {
    MissingPlayer,
    MissingGoods,
    UnsupportedSourceContainer { extend_id: i32 },
    SourceMismatch,
    Shadow(EquipmentSessionShadowAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionClearBlock {
    MissingPlayer,
    MissingShadow,
    SourceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopSelectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) added: PersonalShopShadowAdded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShopSelectionBlock {
    MissingPlayer,
    MissingGoods,
    UnsupportedSourceContainer { extend_id: i32 },
    SourceMismatch,
    ShopOpened,
    Shadow(PersonalShopShadowAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShopClearBlock {
    MissingPlayer,
    MissingShadow,
    SourceMismatch,
    ShopOpened,
}

pub(crate) fn dispatch_game_container_message<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<(), GameContainerMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type != CONTAINER_OBJECT_MOVE {
        return None;
    }

    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        tracing::trace!(message_type, region_id, "у перемещения контейнера нет игрока");
        return Some(Ok(()));
    };

    let start_cursor = message.base_mut().cursor();
    let (request, route) = match decode_container_object_move(message) {
        Ok(mut request) => {
            if request.source_container_type == PLAYER_CONTAINER_TYPE {
                request.source_container_id = player_id;
            }
            if request.destination_container_type == PLAYER_CONTAINER_TYPE {
                request.destination_container_id = player_id;
            }
            if let Some(region_id) = region_id
                && let Some(region) = game.find_region(region_id)
                && let Some(player) = game.find_player(player_id)
            {
                let tile_x = player.shape().get_tile_x().ok();
                let tile_y = player.shape().get_tile_y().ok();
                let position = tile_x.zip(tile_y).map(|(x, y)| {
                    region.base().region.width.wrapping_mul(y).wrapping_add(x) as u32
                });
                if request.source_container_type == 200
                    && request.destination_container_type == PLAYER_CONTAINER_TYPE
                {
                    request.source_container_id = region_id;
                    request.source_container_extend_id = 0;
                    if let Some(position) = position {
                        request.source_position = position;
                    }
                    if let Some(goods) = region.base().find_ground_goods(request.object_id) {
                        request.amount = goods.amount();
                    }
                } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                    && request.destination_container_type == 200
                {
                    request.destination_container_id = region_id;
                    request.destination_container_extend_id = 0;
                    if let Some(position) = position {
                        request.destination_position = position;
                    }
                }
            }
            if request.source_container_type == PLAYER_CONTAINER_TYPE
                && matches!(request.source_container_extend_id, 3 | 4 | 5)
            {
                request.source_position = 0;
            }
            let source_is_reached = game
                .find_player(player_id)
                .is_some_and(|player| match request.source_container_extend_id {
                    3 => player.hand().get_goods(0).is_some(),
                    4 | 5 => player
                        .ground_currency_goods(request.source_container_extend_id)
                        .is_some(),
                    _ => false,
                });
            let ground_base_index = region_id
                .and_then(|region_id| game.find_region(region_id))
                .and_then(|region| region.base().find_ground_goods(request.object_id))
                .map(|goods| goods.base_properties_index());
            let ground_is_currency = ground_base_index.is_some_and(|index| {
                index == game.goods_factory().get_gold_coin_index()
                    || index == game.goods_factory().get_yuan_bao_index()
            });
            let destination_is_reached = (request.destination_container_extend_id == 3
                && request.destination_position == 0
                && !ground_is_currency)
                || (ground_is_currency
                    && !matches!(request.destination_container_extend_id, 1 | 2));
            let route = if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && (matches!(request.source_container_extend_id, 1 | 2 | 3 | 9 | 11 | 12)
                    && request.destination_container_extend_id == 17
                    || request.source_container_extend_id == 17
                        && matches!(
                            request.destination_container_extend_id,
                            1 | 2 | 3 | 9 | 11 | 12
                        )) {
                EnhancementMessageRoute::CiQingComposeTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && (matches!(request.source_container_extend_id, 1 | 2 | 3 | 9 | 11)
                    && request.destination_container_extend_id == 12
                    || request.source_container_extend_id == 12
                        && matches!(request.destination_container_extend_id, 1 | 2 | 3 | 9 | 11))
            {
                EnhancementMessageRoute::BattleFairyTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && (matches!(request.source_container_extend_id, 1 | 2 | 3 | 9)
                    && request.destination_container_extend_id == 11
                    || request.source_container_extend_id == 11
                        && matches!(request.destination_container_extend_id, 1 | 2 | 3 | 9))
            {
                EnhancementMessageRoute::FairyStorageTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && matches!(request.source_container_extend_id, 1 | 2 | 9)
                && request.destination_container_extend_id == 3
            {
                EnhancementMessageRoute::PlayerHandMove
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == 3
                && matches!(request.destination_container_extend_id, 1 | 2 | 9)
            {
                EnhancementMessageRoute::HandContainerMove
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && matches!(
                    (
                        request.source_container_extend_id,
                        request.destination_container_extend_id
                    ),
                    (3, 13) | (13, 3)
                )
            {
                EnhancementMessageRoute::HandAuctionListingTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == 15
                && request.destination_container_extend_id == 4
            {
                EnhancementMessageRoute::AuctionMoneyReturn
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && matches!(
                    (
                        request.source_container_extend_id,
                        request.destination_container_extend_id
                    ),
                    (4, 8) | (8, 4)
                )
            {
                EnhancementMessageRoute::BankCurrencyTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && matches!(
                    (
                        request.source_container_extend_id,
                        request.destination_container_extend_id
                    ),
                    (1, 2) | (2, 1) | (1, 9) | (2, 9) | (9, 1) | (9, 2)
                )
            {
                EnhancementMessageRoute::DepotStorageTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == 14
                && matches!(
                    request.destination_container_extend_id,
                    1 | 2 | 3 | 9 | 11 | 12 | 17
                )
            {
                EnhancementMessageRoute::AuctionGoodsInventoryReturn
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == 200
                && (matches!(request.source_container_extend_id, 1 | 2 | 9 | 11)
                    || source_is_reached)
            {
                EnhancementMessageRoute::GroundDrop
            } else if request.source_container_type == 200
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && (matches!(request.destination_container_extend_id, 1 | 2 | 9 | 11)
                    || destination_is_reached)
            {
                EnhancementMessageRoute::GroundPickup
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_extend_id == 13
                && matches!(
                    request.source_container_extend_id,
                    1 | 2 | 9 | 11 | 12 | 14 | 17
                )
            {
                EnhancementMessageRoute::AuctionListingMove
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == 13
                && matches!(
                    request.destination_container_extend_id,
                    1 | 2 | 9 | 11 | 12 | 17
                )
            {
                EnhancementMessageRoute::AuctionListingWithdrawal
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == 10
                && game
                    .session_factory()
                    .query_trader(request.destination_container_extend_id >> 8)
                    .is_some()
            {
                EnhancementMessageRoute::TradeOfferAdd
            } else if request.source_container_type == 10
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && game
                    .session_factory()
                    .query_trader(request.source_container_extend_id >> 8)
                    .is_some()
            {
                EnhancementMessageRoute::TradeOfferRemove
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_extend_id == ENHANCEMENT_EXTEND_ID
            {
                EnhancementMessageRoute::EnhancementSelect
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == ENHANCEMENT_EXTEND_ID
            {
                let original = game.find_player(player_id).and_then(|player| {
                    player
                        .enhancement_original_container(request.source_position, request.object_id)
                });
                if original.is_some_and(|original| {
                    matches!(original.container_extend_id, 1 | 2)
                        && original.container_type == PLAYER_CONTAINER_TYPE
                        && original.container_id == player_id
                        && original.container_extend_id == request.destination_container_extend_id
                        && original.goods_position == request.destination_position
                }) {
                    EnhancementMessageRoute::EnhancementClear
                } else if original.is_some_and(|original| {
                    matches!(original.container_extend_id, 1 | 2)
                        && original.container_type == PLAYER_CONTAINER_TYPE
                        && original.container_id == player_id
                }) && matches!(request.destination_container_extend_id, 1 | 2)
                {
                    EnhancementMessageRoute::EnhancementTransfer
                } else {
                    let (_, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                    *cursor = start_cursor;
                    return None;
                }
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == 10
                && matches!(request.source_container_extend_id, 1 | 2)
                && game.session_factory().is_personal_shop_seller_container(
                    request.destination_container_id,
                    request.destination_container_extend_id,
                    player_id,
                )
            {
                EnhancementMessageRoute::PersonalShopSelect
            } else if request.source_container_type == 10
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && game.session_factory().is_personal_shop_seller_container(
                    request.source_container_id,
                    request.source_container_extend_id,
                    player_id,
                )
            {
                let original = game.session_factory().personal_shop_shadow_original(
                    request.source_container_id,
                    request.source_container_extend_id,
                    player_id,
                    request.object_id,
                );
                if original.is_some_and(|original| {
                    original.container_type == PLAYER_CONTAINER_TYPE
                        && original.container_id == player_id
                        && original.container_extend_id == request.destination_container_extend_id
                        && original.goods_position == request.destination_position
                }) {
                    EnhancementMessageRoute::PersonalShopClear
                } else {
                    let (_, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                    *cursor = start_cursor;
                    return None;
                }
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == 10
                && matches!(request.source_container_extend_id, 1 | 2)
            {
                EnhancementMessageRoute::EquipmentSessionSelect
            } else if request.source_container_type == 10
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
            {
                let original = game.session_factory().equipment_session_shadow_original(
                    request.source_container_id,
                    request.source_container_extend_id,
                    player_id,
                    request.object_id,
                );
                if original.is_some_and(|original| {
                    original.container_type == PLAYER_CONTAINER_TYPE
                        && original.container_id == player_id
                        && original.container_extend_id == request.destination_container_extend_id
                        && original.goods_position == request.destination_position
                }) {
                    EnhancementMessageRoute::EquipmentSessionClear
                } else if original.is_some_and(|original| {
                    original.container_type == PLAYER_CONTAINER_TYPE
                        && original.container_id == player_id
                        && matches!(original.container_extend_id, 1 | 2)
                }) && matches!(request.destination_container_extend_id, 1 | 2)
                {
                    EnhancementMessageRoute::EquipmentSessionTransfer
                } else {
                    let (_, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                    *cursor = start_cursor;
                    return None;
                }
            } else {
                let (_, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                *cursor = start_cursor;
                return None;
            };
            (request, route)
        }
        Err(error) => return Some(Err(error)),
    };

    let trace = |outcome: &'static str| {
        tracing::trace!(message_type, player_id, region_id, ?request, outcome, "перемещение контейнера обработано");
    };
    let Some(player) = game.find_player(player_id) else {
        trace("игрок не найден");
        return Some(Ok(()));
    };
    if player.in_changing_server() {
        trace("игрок меняет сервер");
        return Some(Ok(()));
    }
    if player.in_changing_region() {
        trace("игрок меняет регион");
        return Some(Ok(()));
    }
    if region_id.is_none() {
        trace("регион не найден");
        return Some(Ok(()));
    }
    if player.current_progress() == PlayerProgress::Synthesis {
        let text = game.get_string_by_id(b"GS1013").to_vec();
        let delivery = send_notify(game, player_id, &text, 0xffff_0000, 0);
        tracing::trace!(message_type, player_id, delivery, "перемещение запрещено во время синтеза");
        return Some(Ok(()));
    }
    if CMoveShape::is_died(player.health()) {
        let delivery = send_notify(
            game,
            player_id,
            b"you can`t pick up prop after died! ",
            0xffff_ffff,
            0,
        );
        tracing::trace!(message_type, player_id, delivery, "перемещение запрещено после смерти");
        return Some(Ok(()));
    }
    if request.object_type != GOODS_OBJECT_TYPE {
        trace("неверный тип объекта");
        return Some(Ok(()));
    }
    if request.amount == 0 {
        trace("нулевое количество");
        return Some(Ok(()));
    }
    if (request.source_container_type == PLAYER_CONTAINER_TYPE
        && !(0..=17).contains(&request.source_container_extend_id))
        || (request.destination_container_type == PLAYER_CONTAINER_TYPE
            && !(0..=17).contains(&request.destination_container_extend_id))
    {
        trace("неверный идентификатор расширения");
        return Some(Ok(()));
    }
    if request.source_container_type == request.destination_container_type
        && request.source_container_id == request.destination_container_id
        && request.source_container_extend_id == request.destination_container_extend_id
    {
        trace("источник и назначение совпадают");
        return Some(Ok(()));
    }
    if route == EnhancementMessageRoute::EnhancementSelect
        && (request.source_container_extend_id == 4
            || request.source_container_extend_id == 5
            || matches!(request.source_container_extend_id, 8 | 15))
    {
        trace("маршрут перемещения запрещён");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::AuctionMoneyReturn {
        let transfer = game.transfer_player_bank_currency(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "AuctionMoneyReturned", "перемещение контейнера выполнено"),
            Err(reason) => {
                let receive_rejected = matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRejectedBeforeRemoval { .. }
                );
                let mut notification_count = 0usize;
                if matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRejectedBeforeRemoval { .. }
                        | BankCurrencyTransferBlock::AuctionCapacityRejectedAfterRemoval { .. }
                        | BankCurrencyTransferBlock::AuctionCapacityRollbackFailed { .. }
                ) {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM015"),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                if matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRollbackFailed { .. }
                        | BankCurrencyTransferBlock::RollbackFailed { .. }
                ) {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM019"),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                let delivery = (!receive_rejected).then(|| send_rollback(game, player_id));
                tracing::trace!(?reason, delivery, notifications = notification_count, "возврат аукционных денег отклонён");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::BankCurrencyTransfer {
        let transfer = game.transfer_player_bank_currency(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "BankCurrencyMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "перевод валюты банка отменён");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::CiQingComposeTransfer {
        let transfer = game.transfer_player_ci_qing_compose_goods(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "CiQingComposeMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    CiQingComposeTransferBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    CiQingComposeTransferBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    CiQingComposeTransferBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    CiQingComposeTransferBlock::BurdenRolledBack { .. }
                    | CiQingComposeTransferBlock::BurdenRollbackFailed { .. } => Some(b"GS0259"),
                    CiQingComposeTransferBlock::DestinationRejectRollbackFailed { .. }
                    | CiQingComposeTransferBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    CiQingComposeTransferBlock::RolledBack { rejected, .. } => match rejected {
                        CiQingComposeTransferAddition::Player(addition) => {
                            depot_add_rejection_notice(addition)
                        }
                        _ => None,
                    },
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение состава CiQing отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::BattleFairyTransfer {
        let transfer = game.transfer_player_battle_fairy_goods(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "BattleFairyMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    BattleFairyTransferBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    BattleFairyTransferBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    BattleFairyTransferBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    BattleFairyTransferBlock::BurdenRolledBack { .. }
                    | BattleFairyTransferBlock::BurdenRollbackFailed { .. } => Some(b"GS0259"),
                    BattleFairyTransferBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    BattleFairyTransferBlock::RolledBack { rejected, .. } => match rejected {
                        BattleFairyTransferAddition::Player(addition) => {
                            depot_add_rejection_notice(addition)
                        }
                        _ => None,
                    },
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение боевой феи отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::FairyStorageTransfer {
        let transfer = game.transfer_player_fairy_goods(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "FairyStorageMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    FairyStorageTransferBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    FairyStorageTransferBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    FairyStorageTransferBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    FairyStorageTransferBlock::BurdenRolledBack { .. }
                    | FairyStorageTransferBlock::BurdenRollbackFailed { .. } => Some(b"GS0259"),
                    FairyStorageTransferBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    FairyStorageTransferBlock::RolledBack { rejected, .. } => match rejected {
                        FairyStorageTransferAddition::Player(addition) => {
                            depot_add_rejection_notice(addition)
                        }
                        _ => None,
                    },
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение феи отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::PlayerHandMove {
        let transfer = game.move_player_goods_to_hand(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "PlayerHandMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    PlayerHandMoveBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    PlayerHandMoveBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    PlayerHandMoveBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    PlayerHandMoveBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение в руку отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::HandAuctionListingTransfer {
        let transfer = game.transfer_hand_auction_listing_goods(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "HandAuctionListingMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    HandAuctionListingBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    HandAuctionListingBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    HandAuctionListingBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    HandAuctionListingBlock::BurdenRolledBack { .. } => Some(b"GS0259"),
                    HandAuctionListingBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение руки и лота отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::HandContainerMove {
        let transfer = game.move_hand_goods_to_player_container(
            player_id,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "HandContainerMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let rejected = match &reason {
                    HandContainerMoveBlock::SwapBusy { rejected, .. }
                    | HandContainerMoveBlock::SwapFailed { rejected, .. }
                    | HandContainerMoveBlock::RollbackFailed { rejected, .. } => Some(rejected),
                    _ => None,
                };
                let notice_id: Option<&[u8]> = match &reason {
                    HandContainerMoveBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    HandContainerMoveBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    HandContainerMoveBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    HandContainerMoveBlock::BurdenExceeded { .. } => Some(b"GS0259"),
                    HandContainerMoveBlock::SwapBusy { .. } => Some(b"GS0335"),
                    HandContainerMoveBlock::RollbackFailed { .. } => Some(b"GPM019"),
                    _ => match rejected {
                        Some(DepotStorageTransferAddition::Depot(
                            DepotGoodsAddOutcome::Rejected(
                                DepotGoodsAddBlock::ExtensionSlotOccupied { .. },
                            ),
                        )) => Some(b"KR007"),
                        Some(DepotStorageTransferAddition::Depot(
                            DepotGoodsAddOutcome::Rejected(
                                DepotGoodsAddBlock::ExtensionItemRequired { .. },
                            ),
                        )) => Some(b"KR008"),
                        Some(DepotStorageTransferAddition::Depot(
                            DepotGoodsAddOutcome::Rejected(
                                DepotGoodsAddBlock::InvalidExtensionKind { .. },
                            ),
                        )) => Some(b"KR009"),
                        _ => None,
                    },
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение из руки отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::AuctionGoodsInventoryReturn {
        let transfer = game.return_auction_goods_to_inventory(
            player_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "AuctionGoodsInventoryMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let mut notification_count = 0usize;
                let progress_notice: Option<&[u8]> = match &reason {
                    AuctionGoodsInventoryBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                        Some(b"GS0113")
                    }
                    AuctionGoodsInventoryBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                        Some(b"GS0114")
                    }
                    AuctionGoodsInventoryBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                        Some(b"GS0115")
                    }
                    _ => None,
                };
                if let Some(notice_id) = progress_notice {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                if matches!(&reason, AuctionGoodsInventoryBlock::BurdenExceeded { .. }) {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GS0259"),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                if let AuctionGoodsInventoryBlock::RolledBack { rejected, .. } = &reason
                    && let CiQingComposeTransferAddition::Player(rejected) = rejected
                    && let Some(notice_id) = depot_add_rejection_notice(rejected)
                {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                let rollback = match &reason {
                    AuctionGoodsInventoryBlock::BurdenExceeded { rollback, .. }
                    | AuctionGoodsInventoryBlock::PacketPositionUnavailable { rollback, .. }
                    | AuctionGoodsInventoryBlock::DestinationRejectsSourceSlot {
                        rollback, ..
                    }
                    | AuctionGoodsInventoryBlock::RolledBack { rollback, .. } => Some(rollback),
                    _ => None,
                };
                if matches!(rollback, Some(AuctionGoodsInventoryRollback::Failed { .. })) {
                    let _ = send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM019"),
                        0xffff_ffff,
                        0,
                    );
                    notification_count += 1;
                }
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notifications = notification_count, "возврат аукционного предмета отменён");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::DepotStorageTransfer {
        let transfer = game.transfer_player_depot_goods(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "DepotStorageMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let rejected = match &reason {
                    DepotStorageTransferBlock::RolledBack { rejected, .. }
                    | DepotStorageTransferBlock::RollbackFailed { rejected, .. } => Some(rejected),
                    _ => None,
                };
                let notice_id: Option<&[u8]> = if matches!(
                    &reason,
                    DepotStorageTransferBlock::BurdenRollbackCompleted { .. }
                        | DepotStorageTransferBlock::BurdenRollbackFailed { .. }
                ) {
                    Some(b"GS0259")
                } else {
                    match &reason {
                        DepotStorageTransferBlock::PartialMoveBusy(PlayerProgress::OpenStall) => {
                            Some(b"GS0113")
                        }
                        DepotStorageTransferBlock::PartialMoveBusy(PlayerProgress::Trading) => {
                            Some(b"GS0114")
                        }
                        DepotStorageTransferBlock::PartialMoveBusy(PlayerProgress::Upgrade) => {
                            Some(b"GS0115")
                        }
                        _ => match rejected {
                            Some(DepotStorageTransferAddition::Depot(
                                DepotGoodsAddOutcome::Rejected(
                                    DepotGoodsAddBlock::ExtensionSlotOccupied { .. },
                                ),
                            )) => Some(b"KR007"),
                            Some(DepotStorageTransferAddition::Depot(
                                DepotGoodsAddOutcome::Rejected(
                                    DepotGoodsAddBlock::ExtensionItemRequired { .. },
                                ),
                            )) => Some(b"KR008"),
                            Some(DepotStorageTransferAddition::Depot(
                                DepotGoodsAddOutcome::Rejected(
                                    DepotGoodsAddBlock::InvalidExtensionKind { .. },
                                ),
                            )) => Some(b"KR009"),
                            _ => None,
                        },
                    }
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение хранилища отменено");
            }
        }
        return Some(Ok(()));
    }

    if matches!(
        route,
        EnhancementMessageRoute::GroundDrop | EnhancementMessageRoute::GroundPickup
    ) {
        if route == EnhancementMessageRoute::GroundPickup
            && (!(0..=11).contains(&request.destination_container_extend_id)
                || request.destination_container_extend_id == 5)
        {
            trace("неверный контейнер назначения для наземного предмета");
            return Some(Ok(()));
        }
        let region_id = region_id.expect("ground route проверен после current region resolve");
        if route == EnhancementMessageRoute::GroundPickup {
            let progress = game
                .find_player(player_id)
                .map(CPlayer::current_progress)
                .unwrap_or_default();
            let notice_id: Option<&[u8]> = match progress {
                PlayerProgress::OpenStall => Some(b"GS0108"),
                PlayerProgress::Trading => Some(b"GS0109"),
                PlayerProgress::Upgrade => Some(b"GS0110"),
                _ => None,
            };
            if let Some(notice_id) = notice_id {
                let notification_delivery = Some(send_notify(
                    game,
                    player_id,
                    game.get_string_by_id(notice_id),
                    0xffff_ffff,
                    0,
                ));
                let delivery = send_rollback(game, player_id);
                tracing::trace!(reason = ?GroundGoodsMoveBlock::PickupBusy, delivery, notification_delivery, "подбор предмета запрещён текущим состоянием");
                return Some(Ok(()));
            }
        }
        let transfer = if route == EnhancementMessageRoute::GroundDrop {
            game.drop_player_goods_to_region(
                player_id,
                region_id,
                request.source_container_extend_id,
                request.source_position,
                request.object_id,
                request.amount,
                context,
            )
        } else {
            game.pick_up_ground_goods_to_player(
                player_id,
                region_id,
                request.source_position,
                request.object_id,
                request.destination_container_extend_id,
                request.destination_position,
                context,
            )
        };
        match transfer {
            Ok(transfer) => tracing::trace!(?transfer, outcome = "GroundGoodsMoved", "перемещение контейнера выполнено"),
            Err(reason) => {
                let notice_id: Option<&[u8]> = match &reason {
                    GroundGoodsMoveBlock::PickupProtected => Some(b"GS0112"),
                    GroundGoodsMoveBlock::BurdenExceeded => Some(b"GS0259"),
                    GroundGoodsMoveBlock::DropBusy(PlayerProgress::OpenStall) => Some(b"GS0113"),
                    GroundGoodsMoveBlock::DropBusy(PlayerProgress::Trading) => Some(b"GS0114"),
                    GroundGoodsMoveBlock::DropBusy(PlayerProgress::Upgrade) => Some(b"GS0115"),
                    GroundGoodsMoveBlock::DepotAdditionRejected(addition) => {
                        depot_add_rejection_notice(addition)
                    }
                    _ => None,
                };
                let notification_delivery = notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "перемещение наземного предмета отменено");
            }
        }
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::AuctionListingMove {
        let transfer = game.move_player_goods_to_auction_listing(
            player_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_position,
            context,
        );
        let transfer = match transfer {
            Ok(transfer) => transfer,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "перемещение лота отменено");
                return Some(Ok(()));
            }
        };
        let move_delivery = send_auction_listing_move_moved(game, player_id, request, &transfer);
        let snapshot_refresh_required =
            transfer.listing_slot_zero_was_empty && request.destination_position == 0;
        tracing::trace!(?transfer, move_delivery, snapshot_refresh_required, "лот перемещён");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::AuctionListingWithdrawal {
        let withdrawal = game.withdraw_player_auction_listing_goods(
            player_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        let withdrawal = match withdrawal {
            Ok(withdrawal) => withdrawal,
            Err(reason) => {
                let notification_delivery =
                    (reason == AuctionListingWithdrawalBlock::BurdenExceeded).then(|| {
                        send_notify(
                            game,
                            player_id,
                            game.get_string_by_id(b"GS0259"),
                            0xffff_ffff,
                            0,
                        )
                    });
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, notification_delivery, "снятие лота отменено");
                return Some(Ok(()));
            }
        };
        let move_delivery = match &withdrawal.outcome {
            AuctionListingWithdrawalOutcome::Moved { .. } => {
                send_auction_listing_withdrawal_moved(game, player_id, request, &withdrawal)
            }
            AuctionListingWithdrawalOutcome::RolledBack { .. }
            | AuctionListingWithdrawalOutcome::GoodsCollected { .. } => {
                send_rollback(game, player_id)
            }
        };
        let notification_delivery = match &withdrawal.outcome {
            AuctionListingWithdrawalOutcome::RolledBack { rejected, .. }
            | AuctionListingWithdrawalOutcome::GoodsCollected { rejected, .. } => {
                let notice_id = match rejected {
                    CiQingComposeTransferAddition::Player(addition) => {
                        depot_add_rejection_notice(addition)
                    }
                    _ => None,
                };
                notice_id.map(|notice_id| {
                    send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    )
                })
            }
            AuctionListingWithdrawalOutcome::Moved { .. } => None,
        };
        tracing::trace!(?withdrawal, move_delivery, notification_delivery, "лот снят");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::TradeOfferAdd {
        let source_goods = game
            .find_player(player_id)
            .and_then(|player| {
                player.trade_source_goods(
                    request.source_container_extend_id,
                    request.source_position,
                    request.object_id,
                )
            })
            .cloned();
        let added = game.record_player_trade_offer(
            player_id,
            request.destination_container_id,
            request.destination_container_extend_id,
            request.destination_position,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
        );
        let added = match added {
            Ok(added) => added,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "добавление предложения обмена отменено");
                return Some(Ok(()));
            }
        };
        let identity = source_goods
            .as_ref()
            .map(CGoods::identity)
            .unwrap_or(ShapeIdentity {
                object_type: GOODS_OBJECT_TYPE,
                id: 0,
                ex_id: request.object_id,
            });
        let payload = source_goods
            .as_ref()
            .map(|goods| context.encode_goods_for_old_client(goods))
            .unwrap_or_default();
        let owners = game.player_trade_owner_ids(request.destination_container_id);
        if let Some(removed) = &added.replaced {
            for owner_id in &owners {
                let _ = send_enhancement_shadow_deleted(game, *owner_id, identity, removed);
            }
        }
        for owner_id in &owners {
            let _ = send_shadow_presence(game, *owner_id, identity, &added.presence, &payload);
        }
        game.reset_player_trade_ready(request.destination_container_id);
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?added, owners = owners.len(), move_delivery, "предложение обмена добавлено");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::TradeOfferRemove {
        let removed = game.remove_player_trade_offer(
            player_id,
            request.source_container_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.destination_container_extend_id,
            request.destination_position,
        );
        let removed = match removed {
            Ok(removed) => removed,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "удаление предложения обмена отменено");
                return Some(Ok(()));
            }
        };
        let identity = game
            .find_player(player_id)
            .and_then(|player| {
                player.trade_source_goods(
                    request.destination_container_extend_id,
                    request.destination_position,
                    request.object_id,
                )
            })
            .map(CGoods::identity)
            .unwrap_or(ShapeIdentity {
                object_type: GOODS_OBJECT_TYPE,
                id: 0,
                ex_id: request.object_id,
            });
        let owners = game.player_trade_owner_ids(request.source_container_id);
        for owner_id in &owners {
            let _ = send_enhancement_shadow_deleted(game, *owner_id, identity, &removed.removed);
        }
        game.reset_player_trade_ready(request.source_container_id);
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?removed, owners = owners.len(), move_delivery, "предложение обмена удалено");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::EquipmentSessionClear {
        let goods_identity = game
            .find_player(player_id)
            .and_then(|player| player.get_goods_by_id(request.object_id))
            .map(|goods| goods.identity());
        let removed = game.clear_player_equipment_session_selection(
            player_id,
            request.source_container_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
        );
        let removed = match removed {
            Ok(removed) => removed,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "очистка сессии снаряжения отменена");
                return Some(Ok(()));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            goods_identity.expect("clear validation сохраняет live source goods"),
            &removed.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?removed, delete_shadow_delivery, move_delivery, "сессия снаряжения очищена");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::PersonalShopClear {
        let goods_identity = game
            .find_player(player_id)
            .and_then(|player| player.get_goods_by_id(request.object_id))
            .map(CGoods::identity);
        let removed = game.clear_player_personal_shop_selection(
            player_id,
            request.source_container_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
        );
        let removed = match removed {
            Ok(removed) => removed,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "очистка выбора личной лавки отменена");
                return Some(Ok(()));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            goods_identity.expect("shop clear validation сохраняет live source goods"),
            &removed.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?removed, delete_shadow_delivery, move_delivery, "выбор личной лавки очищен");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::EquipmentSessionTransfer {
        let transfer = game.transfer_player_equipment_session_goods(
            player_id,
            request.source_container_id,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        let transfer = match transfer {
            Ok(transfer) => transfer,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "перемещение сессии снаряжения отменено");
                return Some(Ok(()));
            }
        };
        let move_delivery = match &transfer.outcome {
            EnhancementTransferOutcome::Moved(_) => {
                send_enhancement_transfer_moved(game, player_id, request, &transfer)
            }
            EnhancementTransferOutcome::RolledBack { .. }
            | EnhancementTransferOutcome::GoodsCollected { .. } => send_rollback(game, player_id),
        };
        tracing::trace!(?transfer, move_delivery, "предмет сессии снаряжения перемещён");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::EnhancementClear {
        let deselection = game.clear_player_enhancement_selection(
            player_id,
            request.source_position,
            request.object_id,
            request.amount,
        );
        let deselection = match deselection {
            Ok(deselection) => deselection,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "очистка усиления отменена");
                return Some(Ok(()));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            deselection.goods,
            &deselection.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?deselection, delete_shadow_delivery, move_delivery, "выбор усиления очищен");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::EnhancementTransfer {
        let transfer = game.transfer_player_enhancement_goods(
            player_id,
            request.source_position,
            request.object_id,
            request.amount,
            request.destination_container_extend_id,
            request.destination_position,
            context,
        );
        let transfer = match transfer {
            Ok(transfer) => transfer,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "перемещение усиления отменено");
                return Some(Ok(()));
            }
        };
        let move_delivery = match &transfer.outcome {
            EnhancementTransferOutcome::Moved(_) => {
                send_enhancement_transfer_moved(game, player_id, request, &transfer)
            }
            EnhancementTransferOutcome::RolledBack { .. }
            | EnhancementTransferOutcome::GoodsCollected { .. } => send_rollback(game, player_id),
        };
        tracing::trace!(?transfer, move_delivery, "предмет усиления перемещён");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::EquipmentSessionSelect {
        let selection = game.select_player_equipment_session_goods(
            player_id,
            request.destination_container_id,
            request.destination_container_extend_id,
            request.destination_position,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
        );
        let selection = match selection {
            Ok(selection) => selection,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "выбор предмета сессии снаряжения отменён");
                return Some(Ok(()));
            }
        };
        let goods = game
            .find_player(player_id)
            .and_then(|player| player.get_goods_by_id(selection.goods.ex_id))
            .expect("equipment-session shadow сохраняет live source goods");
        let old_client_payload = context.encode_goods_for_old_client(goods);
        let add_shadow_delivery = send_shadow_presence(
            game,
            player_id,
            selection.goods,
            &selection.added.shadow.presence,
            &old_client_payload,
        );
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?selection, payload_bytes = old_client_payload.len(), add_shadow_delivery, move_delivery, "предмет сессии снаряжения выбран");
        return Some(Ok(()));
    }

    if route == EnhancementMessageRoute::PersonalShopSelect {
        let selection = game.select_player_personal_shop_goods(
            player_id,
            request.destination_container_id,
            request.destination_container_extend_id,
            request.destination_position,
            request.source_container_extend_id,
            request.source_position,
            request.object_id,
            request.amount,
        );
        let selection = match selection {
            Ok(selection) => selection,
            Err(reason) => {
                let delivery = send_rollback(game, player_id);
                tracing::trace!(?reason, delivery, "выбор предмета личной лавки отменён");
                return Some(Ok(()));
            }
        };
        let goods = game
            .find_player(player_id)
            .and_then(|player| player.get_goods_by_id(selection.goods.ex_id))
            .expect("personal-shop shadow сохраняет live source goods");
        let old_client_payload = context.encode_goods_for_old_client(goods);
        let add_shadow_delivery = send_shadow_presence(
            game,
            player_id,
            selection.goods,
            &selection.added.shadow.presence,
            &old_client_payload,
        );
        let move_delivery = send_rollback(game, player_id);
        tracing::trace!(?selection, payload_bytes = old_client_payload.len(), add_shadow_delivery, move_delivery, "предмет личной лавки выбран");
        return Some(Ok(()));
    }

    let selection = game.select_player_enhancement_goods(
        player_id,
        request.source_container_extend_id,
        request.source_position,
        request.object_id,
        request.amount,
    );
    let selection = match selection {
        Ok(selection) => selection,
        Err(reason) => {
            let delivery = send_rollback(game, player_id);
            tracing::trace!(?reason, delivery, "выбор усиления отменён");
            return Some(Ok(()));
        }
    };

    let goods = game
        .find_player(player_id)
        .and_then(|player| player.get_goods_by_id(selection.goods.ex_id))
        .expect("enhancement shadow сохраняет live source goods");
    let old_client_payload = context.encode_goods_for_old_client(goods);
    let add_shadow_delivery = send_add_shadow(game, player_id, &selection, &old_client_payload);
    let move_delivery = send_move_result(game, player_id, request, &selection);
    tracing::trace!(?selection, payload_bytes = old_client_payload.len(), add_shadow_delivery, move_delivery, "предмет усиления выбран");
    Some(Ok(()))
}

fn decode_container_object_move(
    message: &mut CMessage,
) -> Result<ContainerObjectMoveRequest, GameContainerMessageError> {
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameContainerMessageError::MissingField(field))
    };
    let source_container_type = read_long(message, "source container type")?;
    let source_container_id = read_long(message, "source container ID")?;
    let source_container_extend_id = read_long(message, "source container extend ID")?;
    let source_position = read_long(message, "source position")? as u32;
    let destination_container_type = read_long(message, "destination container type")?;
    let destination_container_id = read_long(message, "destination container ID")?;
    let destination_container_extend_id = read_long(message, "destination container extend ID")?;
    let destination_position = read_long(message, "destination position")? as u32;
    let object_type = read_long(message, "object type")?;
    let object_id_cursor = message.base_mut().cursor();
    let object_id = match message.base_mut().get_guid() {
        Some(guid) => guid,
        None if message.base_mut().cursor() == object_id_cursor + 1 => CGuid::GUID_INVALID,
        None => return Err(GameContainerMessageError::MissingField("object GUID")),
    };
    let amount = read_long(message, "amount")? as u32;
    Ok(ContainerObjectMoveRequest {
        source_container_type,
        source_container_id,
        source_container_extend_id,
        source_position,
        destination_container_type,
        destination_container_id,
        destination_container_extend_id,
        destination_position,
        object_type,
        object_id,
        amount,
    })
}

fn send_notify(game: &CGame, player_id: i32, text: &[u8], first: u32, second: u32) -> i32 {
    let mut message = CMessage::new(0x000b_f806);
    message.add_ulong(first);
    message.add_ulong(second);
    message.base_mut().add(text);
    message.add_byte(0);
    message.send_to_player(game.net_server(), player_id)
}

fn depot_add_rejection_notice(addition: &DepotStorageTransferAddition) -> Option<&'static [u8]> {
    match addition {
        DepotStorageTransferAddition::Depot(DepotGoodsAddOutcome::Rejected(
            DepotGoodsAddBlock::ExtensionSlotOccupied { .. },
        )) => Some(b"KR007"),
        DepotStorageTransferAddition::Depot(DepotGoodsAddOutcome::Rejected(
            DepotGoodsAddBlock::ExtensionItemRequired { .. },
        )) => Some(b"KR008"),
        DepotStorageTransferAddition::Depot(DepotGoodsAddOutcome::Rejected(
            DepotGoodsAddBlock::InvalidExtensionKind { .. },
        )) => Some(b"KR009"),
        _ => None,
    }
}

fn send_rollback(game: &CGame, player_id: i32) -> i32 {
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(0);
    message.send_to_player(game.net_server(), player_id)
}

pub(crate) fn send_enhancement_shadow_deleted(
    game: &CGame,
    player_id: i32,
    goods: ShapeIdentity,
    removed: &ShadowRemovedReport,
) -> i32 {
    let presence = &removed.presence;
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(3);
    message.add_long(presence.owner_type);
    message.add_long(presence.owner_id);
    message.add_long(presence.container_extend_id);
    message.add_ulong(presence.position);
    message.add_long(0);
    message.add_long(0);
    message.add_long(0);
    message.add_ulong(0);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(goods.ex_id);
    message.add_long(0);
    message.base_mut().add_guid(CGuid::GUID_INVALID);
    message.add_ulong(0);
    message.send_to_player(game.net_server(), player_id)
}

fn send_enhancement_transfer_moved(
    game: &CGame,
    player_id: i32,
    request: ContainerObjectMoveRequest,
    transfer: &EnhancementTransferReport,
) -> i32 {
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(1);
    message.add_long(transfer.source.container_type);
    message.add_long(transfer.source.container_id);
    message.add_long(transfer.source.container_extend_id);
    message.add_ulong(transfer.source.goods_position);
    message.add_long(request.destination_container_type);
    message.add_long(request.destination_container_id);
    message.add_long(request.destination_container_extend_id);
    message.add_ulong(request.destination_position);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(transfer.goods.ex_id);
    message.add_ulong(request.amount);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(transfer.goods.ex_id);
    message.add_ulong(request.amount);
    message.send_to_player(game.net_server(), player_id)
}

fn send_auction_listing_move_moved(
    game: &CGame,
    player_id: i32,
    request: ContainerObjectMoveRequest,
    transfer: &AuctionListingTransferReport,
) -> i32 {
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(1);
    message.add_long(request.source_container_type);
    message.add_long(request.source_container_id);
    message.add_long(request.source_container_extend_id);
    message.add_ulong(request.source_position);
    message.add_long(request.destination_container_type);
    message.add_long(request.destination_container_id);
    message.add_long(request.destination_container_extend_id);
    message.add_ulong(request.destination_position);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(transfer.goods.ex_id);
    message.add_ulong(request.amount);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(transfer.goods.ex_id);
    message.add_ulong(request.amount);
    message.send_to_player(game.net_server(), player_id)
}

fn send_auction_listing_withdrawal_moved(
    game: &CGame,
    player_id: i32,
    request: ContainerObjectMoveRequest,
    withdrawal: &AuctionListingWithdrawalReport,
) -> i32 {
    let AuctionListingWithdrawalOutcome::Moved {
        destination_position,
        destination_goods,
        amount,
        ..
    } = &withdrawal.outcome
    else {
        unreachable!("withdrawal move packet требует успешный destination add")
    };
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(1);
    message.add_long(request.source_container_type);
    message.add_long(request.source_container_id);
    message.add_long(request.source_container_extend_id);
    message.add_ulong(request.source_position);
    message.add_long(request.destination_container_type);
    message.add_long(request.destination_container_id);
    message.add_long(request.destination_container_extend_id);
    message.add_ulong(*destination_position);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(withdrawal.goods.ex_id);
    message.add_ulong(request.amount);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(destination_goods.ex_id);
    message.add_ulong(*amount);
    message.send_to_player(game.net_server(), player_id)
}

pub(crate) fn send_enhancement_goods_collected(
    game: &CGame,
    player_id: i32,
    goods_name: &[u8],
    amount: u32,
) -> i32 {
    let template = game.get_string_by_id(b"GPM019");
    let text = format_goods_collected_notification(template, goods_name, amount);
    send_notify(game, player_id, &text, 0xffff_ffff, 0)
}

fn format_goods_collected_notification(template: &[u8], goods_name: &[u8], amount: u32) -> Vec<u8> {
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let goods_name = goods_name
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let amount = amount.to_string();
    let mut output = Vec::with_capacity(template.len());
    let mut offset = 0usize;
    while offset < template.len() && output.len() < 0x22b {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        match template.get(offset + 1).copied() {
            Some(b'%') => output.push(b'%'),
            Some(b's') => output.extend_from_slice(goods_name),
            Some(b'd' | b'i' | b'u') => output.extend_from_slice(amount.as_bytes()),
            _ => {
                output.push(b'%');
                offset += 1;
                continue;
            }
        }
        offset += 2;
    }
    output.truncate(0x22b);
    output
}

fn send_add_shadow(
    game: &CGame,
    player_id: i32,
    selection: &EnhancementSelectionReport,
    payload: &[u8],
) -> i32 {
    send_shadow_presence(
        game,
        player_id,
        selection.goods,
        &selection.shadow.presence,
        payload,
    )
}

pub(crate) fn send_shadow_presence(
    game: &CGame,
    player_id: i32,
    goods: ShapeIdentity,
    presence: &ShadowPresenceReport,
    payload: &[u8],
) -> i32 {
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(2);
    message.add_long(0);
    message.add_long(0);
    message.add_long(0);
    message.add_ulong(0);
    message.add_long(presence.owner_type);
    message.add_long(presence.owner_id);
    message.add_long(presence.container_extend_id);
    message.add_ulong(presence.position);
    message.add_long(0);
    message.base_mut().add_guid(CGuid::GUID_INVALID);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(goods.ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(payload);
    message.send_to_player(game.net_server(), player_id)
}

fn send_move_result(
    game: &CGame,
    player_id: i32,
    _request: ContainerObjectMoveRequest,
    _selection: &EnhancementSelectionReport,
) -> i32 {
    // Shadow Add сначала возвращает тот же goods в PreviousContainer. Поэтому
    // response до `Send` содержит одинаковые source/destination owner, slot и
    // GUID; exact `NormalizeSelfMove` сворачивает его в однобайтовый rollback.
    send_rollback(game, player_id)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp

// ============================================================================
// FUNCTION: OnContainerMessage
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: полные packet/equipment ↔ enhancement, equipment-session upgrade/DaKong/compose и двусторонние auction-listing routes `0x90301`; остальные routes RAW ниже
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp:14
// RVA: 0x00086240
// ADDRESS: 00486240
// PROTOTYPE: void __cdecl OnContainerMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00499691
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp
// RVA: 0x00099691
// ADDRESS: 00499691
// PROTOTYPE: undefined Catch@00499691()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004997a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp
// RVA: 0x000997A1
// ADDRESS: 004997a1
// PROTOTYPE: undefined Catch@004997a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
