//! Входной container dispatcher исторического GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/message/containermessage.cpp`. Материализованы
//! полные player packet/equipment ↔ enhancement-shadow и equipment-session
//! upgrade/DaKong/compose, personal-shop seller и входной auction-listing
//! проходы `0x90301`:
//! одиннадцать wire-полей, outer changing/region/progress/death guards,
//! Receive-нормализация owner ID, точный source position/GUID/amount,
//! запрет stackable goods, однослотовый AddShadow, last-operated state и обе
//! адресные `0xC0101` публикации. Вторая self-move публикация нормализуется в
//! `OT_ROLL_BACK`, сохраняя исходный goods в его source slot. Shadow не
//! забирает ownership исходного goods; исходный select remove→re-add свёрнут
//! в атомарную metadata-запись, поскольку предмет и owner не меняются.
//! Обратный same-original-slot путь удаляет только shadow. Перенос в другой
//! packet/equipment slot выполняет полный ownership pass: source remove,
//! player/equipment callbacks, `OT_DELETE_OBJECT`, destination add, rollback
//! в исходный slot и `GPM019` перед garbage collection при двойном отказе.
//! Equipment-session wire сохраняет настоящий session ID и кодирует plug в
//! старших 24 битах extend ID; selection/clear публикуют Add/DeleteShadow и
//! self-move rollback, transfer использует тот же ownership/effects контракт.
//! Player trade использует тот же session-owned wire: три trader container-а,
//! source metadata без раннего ownership transfer, Add/DeleteShadow обоим
//! участникам и сброс обеих ready-state при каждом изменении предложения.
//! Auction-listing принимает полный предмет из packet/equipment/depot/auction-return,
//! а обратный маршрут возвращает его в packet/equipment/depot после exact carried
//! burden gate; оба сохраняют equipment callbacks, depot lock/anchor/audit,
//! destination rollback и client move;
//! полный persisted-player snapshot `0x6080E` остаётся у недоступного owner-а.
//! Packet/equipment/hand/depot/ordinary-fairy↔ground ветвь того же `0x90301` теперь достигает
//! concrete region goods owner-а: Receive нормализует region/position/amount,
//! сохраняет exact pickup/progress/burden guards, protection notice,
//! equipment property/around effects, one-slot hand split/stack, depot
//! lock/anchor, fairy hatch/lock и независимые ground/depot audits, проводит remove/add с rollback и
//! возвращает container listeners вместе с self/around `0xC0101`.
//! Currency-ветвь сохраняет отдельную `Move`-нормализацию: ground
//! gold/YuanBao при non-packet/equipment destination попадают в
//! wallet/YuanBao extend `4/5`, а source currency containers сохраняют
//! свой balance/object ownership и partial split. JiFen extend `6`
//! отсутствует в точном `GetGoods` switch этого `0x90301` owner-а и остаётся
//! вне данного runtime-маршрута. Wallet↔bank gold transfer достигает тех же
//! positional split/stack owner-ов только после реального password unlock;
//! locked destination выполняет полный rollback в source balance, а success
//! до client move публикует World bank audit с reason `9/10`.
//! Packet/equipment↔depot direct move и compatible stack проходят через тот
//! же dispatcher после password unlock: burden rollback, extension-anchor
//! guards, equipment effects, GoodsAI, audit `7/8` и self wire достигают live
//! owners. Direct packet↔equipment использует тот же ownership pass без depot
//! audit: partial progress guards, exact post-remove burden, equipment effects,
//! occupied-slot rollback и self wire остаются наблюдаемыми.
//! Depot↔ordinary-fairy/battle-fairy/CiQing-compose использует те же lock и
//! extension-anchor owners, World audit `7/8`, special-container effects и
//! rollback; burden добавляется только при возврате в carried containers.
//! Hand↔packet/equipment и depot→hand обычный Put сохраняют positional
//! split, equipment callbacks, one-slot stack, depot lock/anchor/audit и
//! двусторонний rollback. Для
//! source-hand→packet/equipment/depot `OT_SWITCH_OBJECT` выполняется после
//! отказа Put: occupied destination меняется с предметом руки, displaced
//! возвращается в hand (либо exact garbage collection), а switch wire доходит
//! до runtime.
//! Packet/equipment/hand↔ordinary-fairy (`11`) использует тот же ownership owner:
//! fairy positional filters и hatch-lock, packet split, equipment callbacks,
//! burden после fairy remove, rollback и self `0xC0101` наблюдаемы целиком.
//! Packet/equipment/hand↔battle-fairy (`12`) дополнительно сохраняет ранние
//! `BFPropertyAdd`, partial material/gem remove, property/goods-update
//! deliveries и их повторный rollback add до итогового move/rollback wire.
//! Packet/equipment/hand↔CiQing compose (`17`) достигает persisted трёхслотового
//! owner-а: positional/automatic stack, partial remove, burden, equipment
//! callbacks, rollback и self wire связаны целиком. Сохранён exact quirk
//! `PutGoods`: source slot `2` запрещает помещение в compose уже после
//! source remove, поэтому наблюдаемы remove/add rollback effects.
//! Auction-return storage (`14`) имеет только исходящий generic route:
//! packet destination заново выбирает `FindPositionForGoods`, очищает bind
//! value-id `2`, equipment сохраняет positional add, depot — lock/anchor/audit;
//! burden, partial guards, equipment callbacks, rollback и self wire доходят
//! до live owner-ов.
//! Auction wallet (`15`) аналогично имеет только исходящий путь в wallet `4`:
//! exact capacity gate выполняется и в Receive по полному auction balance, и
//! повторно после source removal; partial currency ownership, rollback,
//! last-operated state, `GPM015/GPM019` и self wire сохраняют исходный порядок.
//! Hand↔auction listing (`3↔13`) проходит отдельным direct owner-путём:
//! partial one-slot removal, slot-0 `AuctionLimit`, burden обратного переноса,
//! positional stack, rollback и `0xC0101` больше не выпадают в RAW handler.
//!
//! Остальные container paths owner-а остаются RAW ниже и после восстановления
//! cursor продолжают проходить через прежнюю общую handler-границу.

use crate::gameserver::appserver::container::ccontainer::ContainerListenerHandle;
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cdepot::{DepotGoodsAddBlock, DepotGoodsAddOutcome};
use crate::gameserver::appserver::container::cequipmentcontainer::EquipmentRemovedEvent;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::{
    ShadowPresenceReport, ShadowRemovedReport,
};
use crate::gameserver::appserver::container::cvolumelimitgoodscontainer::VolumeGoodsAddOutcome;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::message::containermessage::EnhancementMoveReceiveBlock::{
    InvalidExtendId, InvalidObjectType, SameContainer, ZeroAmount,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::{
    CPlayer, EnhancementDeselectionBlock, EnhancementDeselectionReport, EnhancementSelectionBlock,
    EnhancementSelectionReport, PlayerEquipmentAddReport, PlayerEquipmentDelivery,
    PlayerEquipmentRemoveEffect, PlayerEquipmentRemoveReport, PlayerProgress,
};
use crate::gameserver::appserver::session::csessionfactory::{
    EquipmentSessionShadowAddBlock, EquipmentSessionShadowAdded, EquipmentSessionShadowRemoved,
    PersonalShopShadowAddBlock, PersonalShopShadowAdded, PersonalShopShadowRemoved,
};
use crate::gameserver::appserver::session::ctrader::{TraderOfferAdded, TraderOfferRemoved};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    AuctionGoodsInventoryBlock, AuctionGoodsInventoryReport, AuctionGoodsInventoryRollback,
    BankCurrencyTransferBlock, BankCurrencyTransferReport, BattleFairyTransferAddition,
    BattleFairyTransferBlock, BattleFairyTransferReport, CGame, CiQingComposeTransferAddition,
    CiQingComposeTransferBlock, CiQingComposeTransferReport, DepotStorageRemoval,
    DepotStorageTransferAddition, DepotStorageTransferBlock, DepotStorageTransferReport,
    FairyStorageTransferAddition, FairyStorageTransferBlock, FairyStorageTransferReport,
    GameContainerMessageRuntime, GroundGoodsMoveBlock, GroundGoodsMoveReport,
    HandAuctionListingBlock, HandAuctionListingReport, HandContainerMoveBlock,
    HandContainerMoveReport, PlayerHandMoveBlock, PlayerHandMoveReport,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameContainerMessageOutcome {
    MissingPlayer,
    MissingRegion,
    ChangingServer,
    ChangingRegion,
    Synthesis {
        notification_delivery: i32,
    },
    Died {
        notification_delivery: i32,
    },
    ReceiveRejected(EnhancementMoveReceiveBlock),
    RolledBack {
        reason: EnhancementSelectionBlock,
        delivery: i32,
    },
    EnhancementSelected {
        selection: EnhancementSelectionReport,
        old_client_payload: Vec<u8>,
        add_shadow_delivery: i32,
        move_delivery: i32,
    },
    ClearRolledBack {
        reason: EnhancementDeselectionBlock,
        delivery: i32,
    },
    EnhancementCleared {
        deselection: EnhancementDeselectionReport,
        delete_shadow_delivery: i32,
        move_delivery: i32,
    },
    TransferRolledBack {
        reason: EnhancementTransferBlock,
        delivery: i32,
    },
    EnhancementTransferred {
        transfer: EnhancementTransferReport,
        move_delivery: i32,
    },
    EquipmentSessionSelectRolledBack {
        reason: EquipmentSessionSelectionBlock,
        delivery: i32,
    },
    EquipmentSessionSelected {
        selection: EquipmentSessionSelectionReport,
        old_client_payload: Vec<u8>,
        add_shadow_delivery: i32,
        move_delivery: i32,
    },
    EquipmentSessionClearRolledBack {
        reason: EquipmentSessionClearBlock,
        delivery: i32,
    },
    EquipmentSessionCleared {
        removed: EquipmentSessionShadowRemoved,
        delete_shadow_delivery: i32,
        move_delivery: i32,
    },
    EquipmentSessionTransferred {
        transfer: EnhancementTransferReport,
        move_delivery: i32,
    },
    PersonalShopSelected {
        selection: PersonalShopSelectionReport,
        old_client_payload: Vec<u8>,
        add_shadow_delivery: i32,
        move_delivery: i32,
    },
    PersonalShopSelectRolledBack {
        reason: PersonalShopSelectionBlock,
        delivery: i32,
    },
    PersonalShopCleared {
        removed: PersonalShopShadowRemoved,
        delete_shadow_delivery: i32,
        move_delivery: i32,
    },
    PersonalShopClearRolledBack {
        reason: PersonalShopClearBlock,
        delivery: i32,
    },
    AuctionListingMoved {
        transfer: AuctionListingTransferReport,
        move_delivery: i32,
        snapshot_refresh_required: bool,
    },
    AuctionListingRolledBack {
        reason: AuctionListingTransferBlock,
        delivery: i32,
    },
    AuctionListingWithdrawn {
        withdrawal: AuctionListingWithdrawalReport,
        move_delivery: i32,
        notification_delivery: Option<i32>,
    },
    AuctionListingWithdrawalRolledBack {
        reason: AuctionListingWithdrawalBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    TradeOfferAdded {
        added: TraderOfferAdded,
        add_shadow_deliveries: Vec<i32>,
        replaced_shadow_deliveries: Vec<i32>,
        ready_deliveries: Vec<i32>,
        move_delivery: i32,
    },
    TradeOfferRemoved {
        removed: TraderOfferRemoved,
        delete_shadow_deliveries: Vec<i32>,
        ready_deliveries: Vec<i32>,
        move_delivery: i32,
    },
    TradeOfferRolledBack {
        reason: crate::gameserver::gameserver::game::PlayerTradeOfferBlock,
        delivery: i32,
    },
    GroundGoodsMoved(GroundGoodsMoveReport),
    GroundGoodsRolledBack {
        reason: GroundGoodsMoveBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    BankCurrencyMoved(BankCurrencyTransferReport),
    BankCurrencyRolledBack {
        reason: BankCurrencyTransferBlock,
        delivery: i32,
    },
    DepotStorageMoved(DepotStorageTransferReport),
    DepotStorageRolledBack {
        reason: DepotStorageTransferBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    HandContainerMoved(HandContainerMoveReport),
    HandContainerRolledBack {
        reason: HandContainerMoveBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    PlayerHandMoved(PlayerHandMoveReport),
    PlayerHandRolledBack {
        reason: PlayerHandMoveBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    FairyStorageMoved(FairyStorageTransferReport),
    FairyStorageRolledBack {
        reason: FairyStorageTransferBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    BattleFairyMoved(BattleFairyTransferReport),
    BattleFairyRolledBack {
        reason: BattleFairyTransferBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    CiQingComposeMoved(CiQingComposeTransferReport),
    CiQingComposeRolledBack {
        reason: CiQingComposeTransferBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
    AuctionGoodsInventoryMoved(AuctionGoodsInventoryReport),
    AuctionGoodsInventoryRolledBack {
        reason: AuctionGoodsInventoryBlock,
        delivery: i32,
        notification_deliveries: Vec<i32>,
    },
    AuctionMoneyReturned(BankCurrencyTransferReport),
    AuctionMoneyReturnRejected {
        reason: BankCurrencyTransferBlock,
        delivery: Option<i32>,
        notification_deliveries: Vec<i32>,
    },
    HandAuctionListingMoved(HandAuctionListingReport),
    HandAuctionListingRolledBack {
        reason: HandAuctionListingBlock,
        delivery: i32,
        notification_delivery: Option<i32>,
    },
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuctionListingTransferReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) removal: AuctionListingTransferRemoval,
    pub(crate) destination: VolumeGoodsAddOutcome,
    pub(crate) listing_slot_zero_was_empty: bool,
    pub(crate) previous_last_operated: (u32, u32),
    pub(crate) audit_deliveries: Vec<i32>,
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
        addition: DepotStorageTransferAddition,
        destination_position: u32,
        destination_goods: ShapeIdentity,
        amount: u32,
    },
    RolledBack {
        rejected: DepotStorageTransferAddition,
        restored: VolumeGoodsAddOutcome,
    },
    GoodsCollected {
        rejected: DepotStorageTransferAddition,
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
    pub(crate) audit_deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingWithdrawalBlock {
    UnsupportedDestinationContainer { extend_id: i32 },
    MissingSourceGoods,
    BurdenExceeded,
    RemovalFailed,
}

#[must_use = "container report сохраняет request, mutation и ordered client effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameContainerMessageReport {
    pub(crate) message_type: u32,
    pub(crate) socket_id: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) request: Option<ContainerObjectMoveRequest>,
    pub(crate) outcome: GameContainerMessageOutcome,
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
) -> Option<Result<GameContainerMessageReport, GameContainerMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type != CONTAINER_OBJECT_MOVE {
        return None;
    }

    message.resolve_player_context(game);
    let socket_id = message.socket_id();
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        return Some(Ok(GameContainerMessageReport {
            message_type,
            socket_id,
            player_id: None,
            region_id,
            request: None,
            outcome: GameContainerMessageOutcome::MissingPlayer,
        }));
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
                && (matches!(request.source_container_extend_id, 1 | 2 | 3 | 9)
                    && request.destination_container_extend_id == 17
                    || request.source_container_extend_id == 17
                        && matches!(request.destination_container_extend_id, 1 | 2 | 3 | 9))
            {
                EnhancementMessageRoute::CiQingComposeTransfer
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && (matches!(request.source_container_extend_id, 1 | 2 | 3 | 9)
                    && request.destination_container_extend_id == 12
                    || request.source_container_extend_id == 12
                        && matches!(request.destination_container_extend_id, 1 | 2 | 3 | 9))
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
                && matches!(request.destination_container_extend_id, 1 | 2 | 9)
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
                && matches!(request.source_container_extend_id, 1 | 2 | 9 | 14)
            {
                EnhancementMessageRoute::AuctionListingMove
            } else if request.source_container_type == PLAYER_CONTAINER_TYPE
                && request.destination_container_type == PLAYER_CONTAINER_TYPE
                && request.source_container_extend_id == 13
                && matches!(request.destination_container_extend_id, 1 | 2 | 9)
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

    let report = |outcome| GameContainerMessageReport {
        message_type,
        socket_id,
        player_id: Some(player_id),
        region_id,
        request: Some(request),
        outcome,
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(report(GameContainerMessageOutcome::MissingPlayer)));
    };
    if player.in_changing_server() {
        return Some(Ok(report(GameContainerMessageOutcome::ChangingServer)));
    }
    if player.in_changing_region() {
        return Some(Ok(report(GameContainerMessageOutcome::ChangingRegion)));
    }
    if region_id.is_none() {
        return Some(Ok(report(GameContainerMessageOutcome::MissingRegion)));
    }
    if player.current_progress() == PlayerProgress::Synthesis {
        let text = game.get_string_by_id(b"GS1013").to_vec();
        let delivery = send_notify(game, player_id, &text, 0xffff_0000, 0);
        return Some(Ok(report(GameContainerMessageOutcome::Synthesis {
            notification_delivery: delivery,
        })));
    }
    if CMoveShape::is_died(player.health()) {
        let delivery = send_notify(
            game,
            player_id,
            b"you can`t pick up prop after died! ",
            0xffff_ffff,
            0,
        );
        return Some(Ok(report(GameContainerMessageOutcome::Died {
            notification_delivery: delivery,
        })));
    }
    if request.object_type != GOODS_OBJECT_TYPE {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            InvalidObjectType,
        ))));
    }
    if request.amount == 0 {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            ZeroAmount,
        ))));
    }
    if (request.source_container_type == PLAYER_CONTAINER_TYPE
        && !(0..=17).contains(&request.source_container_extend_id))
        || (request.destination_container_type == PLAYER_CONTAINER_TYPE
            && !(0..=17).contains(&request.destination_container_extend_id))
    {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            InvalidExtendId,
        ))));
    }
    if request.source_container_type == request.destination_container_type
        && request.source_container_id == request.destination_container_id
        && request.source_container_extend_id == request.destination_container_extend_id
    {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            SameContainer,
        ))));
    }
    if route == EnhancementMessageRoute::EnhancementSelect
        && (request.source_container_extend_id == 4
            || request.source_container_extend_id == 5
            || matches!(request.source_container_extend_id, 8 | 15))
    {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            EnhancementMoveReceiveBlock::ForbiddenRoute,
        ))));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::AuctionMoneyReturned(transfer),
            Err(reason) => {
                let receive_rejected = matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRejectedBeforeRemoval { .. }
                );
                let mut notification_deliveries = Vec::new();
                if matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRejectedBeforeRemoval { .. }
                        | BankCurrencyTransferBlock::AuctionCapacityRejectedAfterRemoval { .. }
                        | BankCurrencyTransferBlock::AuctionCapacityRollbackFailed { .. }
                ) {
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM015"),
                        0xffff_ffff,
                        0,
                    ));
                }
                if matches!(
                    &reason,
                    BankCurrencyTransferBlock::AuctionCapacityRollbackFailed { .. }
                        | BankCurrencyTransferBlock::RollbackFailed { .. }
                ) {
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM019"),
                        0xffff_ffff,
                        0,
                    ));
                }
                GameContainerMessageOutcome::AuctionMoneyReturnRejected {
                    reason,
                    delivery: (!receive_rejected).then(|| send_rollback(game, player_id)),
                    notification_deliveries,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::BankCurrencyMoved(transfer),
            Err(reason) => GameContainerMessageOutcome::BankCurrencyRolledBack {
                reason,
                delivery: send_rollback(game, player_id),
            },
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::CiQingComposeMoved(transfer),
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
                GameContainerMessageOutcome::CiQingComposeRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::BattleFairyMoved(transfer),
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
                GameContainerMessageOutcome::BattleFairyRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::FairyStorageMoved(transfer),
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
                GameContainerMessageOutcome::FairyStorageRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::PlayerHandMoved(transfer),
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
                GameContainerMessageOutcome::PlayerHandRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::HandAuctionListingMoved(transfer),
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
                GameContainerMessageOutcome::HandAuctionListingRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::HandContainerMoved(transfer),
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
                GameContainerMessageOutcome::HandContainerRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::AuctionGoodsInventoryMoved(transfer),
            Err(reason) => {
                let mut notification_deliveries = Vec::new();
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
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    ));
                }
                if matches!(&reason, AuctionGoodsInventoryBlock::BurdenExceeded { .. }) {
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GS0259"),
                        0xffff_ffff,
                        0,
                    ));
                }
                if let AuctionGoodsInventoryBlock::RolledBack { rejected, .. } = &reason
                    && let Some(notice_id) = depot_add_rejection_notice(rejected)
                {
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(notice_id),
                        0xffff_ffff,
                        0,
                    ));
                }
                let rollback = match &reason {
                    AuctionGoodsInventoryBlock::BurdenExceeded { rollback, .. }
                    | AuctionGoodsInventoryBlock::PacketPositionUnavailable { rollback, .. }
                    | AuctionGoodsInventoryBlock::RolledBack { rollback, .. } => Some(rollback),
                    _ => None,
                };
                if matches!(rollback, Some(AuctionGoodsInventoryRollback::Failed { .. })) {
                    notification_deliveries.push(send_notify(
                        game,
                        player_id,
                        game.get_string_by_id(b"GPM019"),
                        0xffff_ffff,
                        0,
                    ));
                }
                GameContainerMessageOutcome::AuctionGoodsInventoryRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_deliveries,
                }
            }
        })));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::DepotStorageMoved(transfer),
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
                GameContainerMessageOutcome::DepotStorageRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
    }

    if matches!(
        route,
        EnhancementMessageRoute::GroundDrop | EnhancementMessageRoute::GroundPickup
    ) {
        if route == EnhancementMessageRoute::GroundPickup
            && (!(0..=11).contains(&request.destination_container_extend_id)
                || request.destination_container_extend_id == 5)
        {
            return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
                InvalidExtendId,
            ))));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::GroundGoodsRolledBack {
                        reason: GroundGoodsMoveBlock::PickupBusy,
                        delivery,
                        notification_delivery,
                    },
                )));
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
        return Some(Ok(report(match transfer {
            Ok(transfer) => GameContainerMessageOutcome::GroundGoodsMoved(transfer),
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
                GameContainerMessageOutcome::GroundGoodsRolledBack {
                    reason,
                    delivery: send_rollback(game, player_id),
                    notification_delivery,
                }
            }
        })));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::AuctionListingRolledBack { reason, delivery },
                )));
            }
        };
        let move_delivery = send_auction_listing_move_moved(game, player_id, request, &transfer);
        let snapshot_refresh_required =
            transfer.listing_slot_zero_was_empty && request.destination_position == 0;
        return Some(Ok(report(
            GameContainerMessageOutcome::AuctionListingMoved {
                transfer,
                move_delivery,
                snapshot_refresh_required,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::AuctionListingWithdrawalRolledBack {
                        reason,
                        delivery,
                        notification_delivery,
                    },
                )));
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
                depot_add_rejection_notice(rejected).map(|notice_id| {
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
        return Some(Ok(report(
            GameContainerMessageOutcome::AuctionListingWithdrawn {
                withdrawal,
                move_delivery,
                notification_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::TradeOfferRolledBack { reason, delivery },
                )));
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
        let replaced_shadow_deliveries = added.replaced.as_ref().map_or_else(Vec::new, |removed| {
            owners
                .iter()
                .map(|owner_id| send_enhancement_shadow_deleted(game, *owner_id, identity, removed))
                .collect()
        });
        let add_shadow_deliveries = owners
            .iter()
            .map(|owner_id| {
                send_shadow_presence(game, *owner_id, identity, &added.presence, &payload)
            })
            .collect();
        let ready_deliveries = game.reset_player_trade_ready(request.destination_container_id);
        let move_delivery = send_rollback(game, player_id);
        return Some(Ok(report(GameContainerMessageOutcome::TradeOfferAdded {
            added,
            add_shadow_deliveries,
            replaced_shadow_deliveries,
            ready_deliveries,
            move_delivery,
        })));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::TradeOfferRolledBack { reason, delivery },
                )));
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
        let delete_shadow_deliveries = owners
            .iter()
            .map(|owner_id| {
                send_enhancement_shadow_deleted(game, *owner_id, identity, &removed.removed)
            })
            .collect();
        let ready_deliveries = game.reset_player_trade_ready(request.source_container_id);
        let move_delivery = send_rollback(game, player_id);
        return Some(Ok(report(GameContainerMessageOutcome::TradeOfferRemoved {
            removed,
            delete_shadow_deliveries,
            ready_deliveries,
            move_delivery,
        })));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::EquipmentSessionClearRolledBack {
                        reason,
                        delivery,
                    },
                )));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            goods_identity.expect("clear validation сохраняет live source goods"),
            &removed.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        return Some(Ok(report(
            GameContainerMessageOutcome::EquipmentSessionCleared {
                removed,
                delete_shadow_delivery,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::PersonalShopClearRolledBack { reason, delivery },
                )));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            goods_identity.expect("shop clear validation сохраняет live source goods"),
            &removed.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        return Some(Ok(report(
            GameContainerMessageOutcome::PersonalShopCleared {
                removed,
                delete_shadow_delivery,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::TransferRolledBack { reason, delivery },
                )));
            }
        };
        let move_delivery = match &transfer.outcome {
            EnhancementTransferOutcome::Moved(_) => {
                send_enhancement_transfer_moved(game, player_id, request, &transfer)
            }
            EnhancementTransferOutcome::RolledBack { .. }
            | EnhancementTransferOutcome::GoodsCollected { .. } => send_rollback(game, player_id),
        };
        return Some(Ok(report(
            GameContainerMessageOutcome::EquipmentSessionTransferred {
                transfer,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(GameContainerMessageOutcome::ClearRolledBack {
                    reason,
                    delivery,
                })));
            }
        };
        let delete_shadow_delivery = send_enhancement_shadow_deleted(
            game,
            player_id,
            deselection.goods,
            &deselection.removed,
        );
        let move_delivery = send_rollback(game, player_id);
        return Some(Ok(report(
            GameContainerMessageOutcome::EnhancementCleared {
                deselection,
                delete_shadow_delivery,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::TransferRolledBack { reason, delivery },
                )));
            }
        };
        let move_delivery = match &transfer.outcome {
            EnhancementTransferOutcome::Moved(_) => {
                send_enhancement_transfer_moved(game, player_id, request, &transfer)
            }
            EnhancementTransferOutcome::RolledBack { .. }
            | EnhancementTransferOutcome::GoodsCollected { .. } => send_rollback(game, player_id),
        };
        return Some(Ok(report(
            GameContainerMessageOutcome::EnhancementTransferred {
                transfer,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::EquipmentSessionSelectRolledBack {
                        reason,
                        delivery,
                    },
                )));
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
        return Some(Ok(report(
            GameContainerMessageOutcome::EquipmentSessionSelected {
                selection,
                old_client_payload,
                add_shadow_delivery,
                move_delivery,
            },
        )));
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
                return Some(Ok(report(
                    GameContainerMessageOutcome::PersonalShopSelectRolledBack { reason, delivery },
                )));
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
        return Some(Ok(report(
            GameContainerMessageOutcome::PersonalShopSelected {
                selection,
                old_client_payload,
                add_shadow_delivery,
                move_delivery,
            },
        )));
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
            return Some(Ok(report(GameContainerMessageOutcome::RolledBack {
                reason,
                delivery,
            })));
        }
    };

    let goods = game
        .find_player(player_id)
        .and_then(|player| player.get_goods_by_id(selection.goods.ex_id))
        .expect("enhancement shadow сохраняет live source goods");
    let old_client_payload = context.encode_goods_for_old_client(goods);
    let add_shadow_delivery = send_add_shadow(game, player_id, &selection, &old_client_payload);
    let move_delivery = send_move_result(game, player_id, request, &selection);
    Some(Ok(report(
        GameContainerMessageOutcome::EnhancementSelected {
            selection,
            old_client_payload,
            add_shadow_delivery,
            move_delivery,
        },
    )))
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
