//! Полный owner `CFairyContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cfairycontainer.cpp`. Контейнер
//! владеет ordinary-fairy товарами через `CVolumeLimitGoodsContainer`:
//! auto-add сначала выбирает ячейку base-owner-а, positional add сохраняет
//! отдельное правило `0..12` для headgear с fairy property и исключение
//! `13/FZ0885`, а remove блокирует фею с ненулевым hatch timer. Codec suffix,
//! инкубация, общий рост, state transition и синкретизация материализованы с
//! точным порядком проверок, wrapping arithmetic и partial mutation.
//!
//! `CVolumeLimitGoodsContainer`, owned `CGoods`, `Vec` effects и явные
//! clock/config/player facts заменяют vtable dispatch, raw pointers и globals,
//! не меняя lock/stack/listener/message semantics. Пять hatch timer-ов codec
//! suffix сохраняют partial decode; state/syncretize не откатывают уже
//! выполненные remove/add и возвращают detached ownership на отказе. Достигнутые
//! state/amount client packets и incubate/syncretize World logs исполняет
//! canonical `CGame`, поэтому здесь остаются только ordered typed effects.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCleared, AmountLimitGoodsRelease, AmountLimitGoodsTaken,
};
use super::ccontainer::ContainerListenerHandle;
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddBlock, VolumeGoodsAddOutcome,
    VolumeGoodsCodecError, VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_BF_BATTLE_FAIRY, GAP_PARTICULAR_ATTRIBUTE,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::goods::fairyproperties::{
    FairyExpBlock, FairyExpReport, FairyExpRuntime, FairyExpUpResult,
};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;
use thiserror::Error;

const FAIRY_SPECIAL_POSITION: u32 = 13;
const FAIRY_SPECIAL_ORIGINAL_NAME: &[u8] = b"FZ0885";
const HATCHER_POSITIONS: std::ops::Range<u32> = 5..10;
const FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0b;
const FAIRY_EXP_POSITIONS: std::ops::Range<u32> = 0..5;
const FAIRY_GOODS_UPDATE_MESSAGE_TYPE: u32 = 0x0b_f918;
const FAIRY_WORLD_LOG_MESSAGE_TYPE: u32 = 0x06_0210;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerAddBlock {
    MissingGoods,
    BattleFairy,
    MissingBaseProperties { index: u32 },
    InvalidFairyPosition { position: u32 },
    InvalidFairyGoods { position: u32 },
}

#[must_use = "outcome определяет ownership incoming и base listener effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerAddOutcome {
    Base(VolumeGoodsAddOutcome),
    Rejected(FairyContainerAddBlock),
}

#[must_use = "remove может быть заблокирован живым hatch timer"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerRemoveOutcome {
    Missing,
    Hatching {
        position: Option<u32>,
        goods_id: CGuid,
    },
    Removed(VolumeGoodsRemoveOutcome),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum FairyContainerCodecError {
    #[error(transparent)]
    Base(#[from] VolumeGoodsCodecError),
    #[error("fairy container hatch position {position} обрывается в {offset}: доступно {available}")]
    UnexpectedEnd {
        position: u32,
        offset: usize,
        available: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerUnserializeReport {
    pub(crate) cleared: AmountLimitGoodsCleared,
    pub(crate) base_result: bool,
    pub(crate) restored_hatch_positions: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerUnserializeFailure {
    pub(crate) error: FairyContainerCodecError,
    pub(crate) report: FairyContainerUnserializeReport,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyState {
    Egg = 0,
    Young = 1,
    Ripe = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerMoveOperation {
    DeleteObject,
    NewObject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerObjectMove {
    pub(crate) operation: FairyContainerMoveOperation,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: Option<u32>,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyStateChangeEffect {
    ObjectMove(FairyContainerObjectMove),
    GarbageCollected(ShapeIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerRemovedEvent {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: Option<u32>,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "state change может оставить detached replacement после partial mutation"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyStateChangeOutcome {
    MissingGoods,
    MissingProperties,
    ReplacementCreationFailed {
        goods_index: u32,
    },
    ReplacementPropertiesMissing {
        replacement: CGoods,
    },
    RemovalFailed {
        removal: FairyContainerRemoveOutcome,
        replacement: CGoods,
    },
    ReplacementRejected {
        removed: FairyContainerRemovedEvent,
        add: FairyContainerAddOutcome,
        replacement: Option<CGoods>,
        effects: Vec<FairyStateChangeEffect>,
    },
    Changed {
        removed: FairyContainerRemovedEvent,
        added: AmountLimitGoodsAdded,
        goods: ShapeIdentity,
        effects: Vec<FairyStateChangeEffect>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyIncubateLog {
    pub(crate) message_type: u32,
    pub(crate) log_type: i32,
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) goods_name: Vec<u8>,
}

#[must_use = "hatcher entry содержит полный state transition и optional world log"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyHatcherEntry {
    pub(crate) position: u32,
    pub(crate) transition: FairyStateChangeOutcome,
    pub(crate) incubate_log: Option<FairyIncubateLog>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerGoodsUpdate {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

#[must_use = "exp entry содержит ordered property logs и delivery effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerExpEntry {
    Updated {
        position: u32,
        exp: FairyExpReport,
        update: FairyContainerGoodsUpdate,
    },
    StateChanged {
        position: u32,
        exp: FairyExpReport,
        transition: FairyStateChangeOutcome,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyImplantDelivery {
    Update(FairyContainerGoodsUpdate),
    StateChanged(FairyStateChangeOutcome),
}

#[must_use = "implantation report сохраняет remaining exp и итоговый goods для расхода vigour/crystal"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyImplantReport {
    pub(crate) old_level: u32,
    pub(crate) exp: FairyExpReport,
    pub(crate) goods: ShapeIdentity,
    pub(crate) goods_name: Vec<u8>,
    pub(crate) resulting_level: u32,
    pub(crate) old_client_payload: Vec<u8>,
    pub(crate) delivery: FairyImplantDelivery,
}

#[must_use = "failure сохраняет уже обработанный prefix позиций"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerExpFailure {
    pub(crate) position: u32,
    pub(crate) error: FairyExpBlock,
    pub(crate) entries: Vec<FairyContainerExpEntry>,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairySyncreticProperty {
    FairyAttribute = 0,
    GrowingRate = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FairySyncreticResult {
    #[default]
    Unknown = 0,
    NoRipeFairyOne = 1,
    NoRipeFairyTwo = 2,
    DifferentMainProperty = 3,
    SyncreticTimesError = 4,
    NotEnoughFragment = 5,
    NotEnoughMoney = 6,
    NotEnoughExperience = 7,
    Failed = 8,
    Successful = 9,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FairySyncretizeConfig {
    pub(crate) needed_goods: u32,
    pub(crate) needed_experience: u32,
    pub(crate) needed_money: u32,
    pub(crate) rate_a: f32,
    pub(crate) rate_b: f32,
    pub(crate) rate_c: f32,
    pub(crate) rate_d: f32,
    pub(crate) rate_e: f32,
    pub(crate) rate_f: f32,
    pub(crate) rate_g: f32,
    pub(crate) rate_h: f32,
    pub(crate) rate_n: f32,
    pub(crate) rate_y: f32,
    pub(crate) log_enabled: bool,
}

#[derive(Debug)]
pub(crate) struct FairySyncretizePlayer {
    pub(crate) id: i32,
    pub(crate) experience: u32,
    pub(crate) money: u32,
    pub(crate) vigour: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyContainerAmountChange {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FairySyncretizePlayerUpdate {
    pub(crate) player_id: i32,
    pub(crate) experience: u32,
    pub(crate) vigour: u32,
    pub(crate) money: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairySyncretizeLog {
    pub(crate) message_type: u32,
    pub(crate) log_type: i32,
    pub(crate) player_id: i32,
    pub(crate) primary_guid: CGuid,
    pub(crate) primary_name: Vec<u8>,
    pub(crate) primary_level: u32,
    pub(crate) primary_growing_rate: u32,
    pub(crate) secondary_guid: CGuid,
    pub(crate) secondary_name: Vec<u8>,
    pub(crate) secondary_level: u32,
    pub(crate) secondary_growing_rate: u32,
    pub(crate) needed_goods: u32,
    pub(crate) result_guid: CGuid,
    pub(crate) result_name: Vec<u8>,
    pub(crate) main_ability: u32,
    pub(crate) combinated_times: u32,
    pub(crate) growing_rate: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairySyncretizeFragmentEffect {
    AmountChanged(FairyContainerAmountChange),
    Removed {
        event: FairyContainerRemovedEvent,
        effects: Vec<FairyStateChangeEffect>,
    },
    RemovalFailed(FairyContainerRemoveOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairySyncretizeRemoval {
    Removed {
        event: FairyContainerRemovedEvent,
        effects: Vec<FairyStateChangeEffect>,
    },
    Failed(FairyContainerRemoveOutcome),
}

#[must_use = "report сохраняет необратимые state/remove/fragment/player effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairySyncretizeReport {
    pub(crate) result: FairySyncreticResult,
    pub(crate) state_change: Option<FairyStateChangeOutcome>,
    pub(crate) secondary_removal: Option<FairySyncretizeRemoval>,
    pub(crate) fragment_effect: Option<FairySyncretizeFragmentEffect>,
    pub(crate) player_update: Option<FairySyncretizePlayerUpdate>,
    pub(crate) log: Option<FairySyncretizeLog>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFairyContainer {
    base: CVolumeLimitGoodsContainer,
}

impl Default for CFairyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CFairyContainer {
    /// Полный persisted codec fairy owner-а: positional base payload и пять
    /// hatch timer-ов образуют один неделимый wire-блок player GameSave.
    pub(crate) fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
        if !self.base.serialize(destination, factory) {
            return false;
        }
        for position in HATCHER_POSITIONS {
            let hatch_start_time = self
                .base
                .get_goods(position)
                .and_then(CGoods::fairy_properties)
                .map_or(0, |fairy| fairy.hatch_start_time);
            LegacyWriter::new(destination).write_u32(hatch_start_time);
        }
        true
    }

    pub(crate) fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<Vec<u32>, FairyContainerCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        self.base.unserialize(
            source,
            cursor,
            factory,
            ordinary_threshold,
            battle_threshold,
        )?;
        let mut restored = Vec::new();
        for position in HATCHER_POSITIONS {
            let offset = *cursor;
            let available = source.len().saturating_sub(offset);
            let mut reader = LegacyReader::at(source, offset).map_err(|_| FairyContainerCodecError::UnexpectedEnd { position, offset, available })?;
            let hatch_start_time = reader.read_u32().map_err(|_| FairyContainerCodecError::UnexpectedEnd { position, offset, available })?;
            *cursor = reader.position();
            if hatch_start_time == 0 {
                continue;
            }
            let Some(fairy) = self
                .base
                .get_goods_mut(position)
                .and_then(CGoods::fairy_properties_mut)
            else {
                continue;
            };
            fairy.hatch_start_time = hatch_start_time;
            restored.push(position);
        }
        Ok(restored)
    }

    pub(crate) fn new() -> Self {
        Self {
            base: CVolumeLimitGoodsContainer::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CVolumeLimitGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.base
    }

    pub(crate) fn clear(&mut self) -> AmountLimitGoodsCleared {
        self.base.clear_goods()
    }

    pub(crate) fn release(&mut self) -> AmountLimitGoodsRelease {
        self.base.release()
    }

    pub(crate) fn add(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> FairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::MissingGoods);
        };
        let Some(position) = self.base.find_position_for_goods(goods, factory) else {
            return FairyContainerAddOutcome::Base(VolumeGoodsAddOutcome::Rejected(
                VolumeGoodsAddBlock::NoSpace,
            ));
        };
        self.add_at(position, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> FairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::MissingGoods);
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1 {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::BattleFairy);
        }
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return FairyContainerAddOutcome::Rejected(
                FairyContainerAddBlock::MissingBaseProperties { index },
            );
        };

        let accepted = if position < FAIRY_SPECIAL_POSITION {
            properties.equip_place() == EQUIP_PLACE_HEADGEAR && goods.fairy_properties().is_some()
        } else if position == FAIRY_SPECIAL_POSITION {
            properties.original_name() == FAIRY_SPECIAL_ORIGINAL_NAME
        } else {
            return FairyContainerAddOutcome::Rejected(
                FairyContainerAddBlock::InvalidFairyPosition { position },
            );
        };
        if !accepted {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::InvalidFairyGoods {
                position,
            });
        }

        FairyContainerAddOutcome::Base(self.base.add_goods_at(
            position,
            incoming,
            factory,
            owner_progress_allows,
        ))
    }

    pub(crate) fn remove(&mut self, goods_id: CGuid) -> FairyContainerRemoveOutcome {
        let position = self.base.query_goods_position(goods_id);
        let Some(goods) = self.base.base().find(goods_id) else {
            return FairyContainerRemoveOutcome::Missing;
        };
        if goods
            .fairy_properties()
            .is_some_and(|fairy| fairy.hatch_start_time != 0)
        {
            return FairyContainerRemoveOutcome::Hatching { position, goods_id };
        }
        self.base
            .remove_goods(goods_id)
            .map(FairyContainerRemoveOutcome::Removed)
            .unwrap_or(FairyContainerRemoveOutcome::Missing)
    }

    /// Positional `Remove(position, amount)` сохраняет тот же hatch-lock,
    /// после чего делегирует full/partial ownership общему volume owner-у.
    pub(crate) fn take<Create>(
        &mut self,
        position: u32,
        amount: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> FairyContainerRemoveOutcome
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let Some(goods) = self.base.get_goods(position) else {
            return FairyContainerRemoveOutcome::Missing;
        };
        if goods
            .fairy_properties()
            .is_some_and(|fairy| fairy.hatch_start_time != 0)
        {
            return FairyContainerRemoveOutcome::Hatching {
                position: Some(position),
                goods_id: goods.identity().ex_id,
            };
        }
        self.base
            .take_goods(position, amount, factory, create_goods)
            .map(FairyContainerRemoveOutcome::Removed)
            .unwrap_or(FairyContainerRemoveOutcome::Missing)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fairy_change_state(
        &mut self,
        goods_id: CGuid,
        state: FairyState,
        destination_position: u32,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> FairyStateChangeOutcome {
        let Some(old_goods) = self.base.base().find(goods_id) else {
            return FairyStateChangeOutcome::MissingGoods;
        };
        let Some(old_fairy) = old_goods.fairy_properties() else {
            return FairyStateChangeOutcome::MissingProperties;
        };
        let replacement_index = match state {
            FairyState::Egg => old_fairy.young_id,
            FairyState::Young => old_fairy.ripe_id,
            FairyState::Ripe => old_fairy.egg_id,
        };
        let Some(mut replacement) = create_goods(replacement_index) else {
            return FairyStateChangeOutcome::ReplacementCreationFailed {
                goods_index: replacement_index,
            };
        };
        if replacement
            .copy_fairy_addon_properties_from(old_goods, factory, &mut *fairy_threshold_for_level)
            .is_err()
        {
            return FairyStateChangeOutcome::ReplacementPropertiesMissing { replacement };
        }
        let Some(replacement_fairy) = replacement.fairy_properties_mut() else {
            return FairyStateChangeOutcome::ReplacementPropertiesMissing { replacement };
        };
        match state {
            FairyState::Egg => {
                replacement_fairy.fairy_state = FairyState::Young as u32;
                replacement_fairy.strength =
                    hatch_stat(replacement_fairy.strength, replacement_fairy.base_strength);
                replacement_fairy.agility =
                    hatch_stat(replacement_fairy.agility, replacement_fairy.base_agility);
                replacement_fairy.wakan =
                    hatch_stat(replacement_fairy.wakan, replacement_fairy.base_wakan);
                replacement_fairy.hp = hatch_stat(replacement_fairy.hp, replacement_fairy.base_hp);
            }
            FairyState::Young => replacement_fairy.fairy_state = FairyState::Ripe as u32,
            FairyState::Ripe => replacement_fairy.fairy_state = FairyState::Egg as u32,
        }
        if replacement.save_fairy_properties(factory).ok() != Some(true) {
            return FairyStateChangeOutcome::ReplacementPropertiesMissing { replacement };
        }
        if state == FairyState::Ripe {
            let _ = replacement.set_addon_property_value_core(GAP_PARTICULAR_ATTRIBUTE, 1, 0x122);
        }

        let old_position = self.base.query_goods_position(goods_id);
        self.base
            .base_mut()
            .find_mut(goods_id)
            .and_then(CGoods::fairy_properties_mut)
            .expect("old fairy проверена до создания replacement")
            .hatch_start_time = 0;
        let removal = self.remove(goods_id);
        let FairyContainerRemoveOutcome::Removed(removed) = removal else {
            return FairyStateChangeOutcome::RemovalFailed {
                removal,
                replacement,
            };
        };
        let taken = match removed {
            VolumeGoodsRemoveOutcome::Removed(taken)
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
        };
        let AmountLimitGoodsTaken::Removed(removed) = taken else {
            unreachable!("full remove by GUID cannot split goods")
        };
        let removed_identity = removed.goods.identity();
        let removed_event = FairyContainerRemovedEvent {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position,
            amount: removed.amount,
            listeners: removed.listeners,
        };
        let mut effects = vec![
            FairyStateChangeEffect::ObjectMove(FairyContainerObjectMove {
                operation: FairyContainerMoveOperation::DeleteObject,
                owner_type: removed_event.owner_type,
                owner_id: removed_event.owner_id,
                position: old_position,
                container_extend_id: FAIRY_CONTAINER_EXTEND_ID,
                goods: removed_identity,
                amount: removed_event.amount,
                old_client_payload: Vec::new(),
            }),
            FairyStateChangeEffect::GarbageCollected(removed_identity),
        ];
        drop(removed.goods);

        let mut incoming = Some(replacement);
        let add = self.add_at(
            destination_position,
            &mut incoming,
            factory,
            owner_progress_allows,
        );
        let FairyContainerAddOutcome::Base(VolumeGoodsAddOutcome::Added(added)) = add else {
            return FairyStateChangeOutcome::ReplacementRejected {
                removed: removed_event,
                add,
                replacement: incoming,
                effects,
            };
        };
        let added_goods = self
            .base
            .get_goods(destination_position)
            .expect("successful fairy Add публикует destination goods");
        let added_identity = added_goods.identity();
        effects.push(FairyStateChangeEffect::ObjectMove(
            FairyContainerObjectMove {
                operation: FairyContainerMoveOperation::NewObject,
                owner_type: added.owner_type,
                owner_id: added.owner_id,
                position: Some(destination_position),
                container_extend_id: FAIRY_CONTAINER_EXTEND_ID,
                goods: added_identity,
                amount: added_goods.amount(),
                old_client_payload: encode_old_client(added_goods),
            },
        ));
        FairyStateChangeOutcome::Changed {
            removed: removed_event,
            added,
            goods: added_identity,
            effects,
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Исходный параметр owner не читал; callback заменяет внутренний
    /// `timeGetTime()` и вызывается отдельно для каждого живого timer-а.
    pub(crate) fn check_hatcher(
        &mut self,
        current_tick: &mut dyn FnMut() -> u32,
        hatch_duration: u32,
        incubate_log_enabled: bool,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Vec<FairyHatcherEntry> {
        let mut entries = Vec::new();
        for position in HATCHER_POSITIONS {
            let Some((goods_id, hatch_start_time)) =
                self.base.get_goods(position).and_then(|goods| {
                    goods
                        .fairy_properties()
                        .filter(|fairy| fairy.hatch_start_time != 0)
                        .map(|fairy| (goods.identity().ex_id, fairy.hatch_start_time))
                })
            else {
                continue;
            };
            if current_tick() < hatch_start_time.wrapping_add(hatch_duration) {
                continue;
            }
            let transition = self.fairy_change_state(
                goods_id,
                FairyState::Egg,
                position,
                factory,
                owner_progress_allows,
                fairy_threshold_for_level,
                create_goods,
                encode_old_client,
            );
            let incubate_log = incubate_log_enabled.then(|| {
                self.base.get_goods(position).map(|goods| FairyIncubateLog {
                    message_type: FAIRY_WORLD_LOG_MESSAGE_TYPE,
                    log_type: 3,
                    player_id: self.base.base().base().owner_id(),
                    goods: goods.identity(),
                    goods_name: goods.name().to_vec(),
                })
            });
            entries.push(FairyHatcherEntry {
                position,
                transition,
                incubate_log: incubate_log.flatten(),
            });
        }
        entries
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fairy_exp_up(
        &mut self,
        experience: u32,
        grow_log_enabled: bool,
        egg_max_level: u32,
        upgrade_rate: f32,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Result<Vec<FairyContainerExpEntry>, FairyContainerExpFailure> {
        enum Delivery {
            None,
            Update(FairyContainerGoodsUpdate),
            ChangeState(CGuid),
        }

        let owner_id = self.base.base().base().owner_id();
        let mut entries = Vec::new();
        for position in FAIRY_EXP_POSITIONS {
            let (exp, delivery) = {
                let Some(goods) = self.base.get_goods_mut(position) else {
                    continue;
                };
                if goods.fairy_properties().is_none() {
                    continue;
                }
                let fairy_guid = goods.identity().ex_id.to_string().into_bytes();
                let fairy_name = goods.name().to_vec();
                let mut remaining = experience;
                let Some(exp) = goods
                    .fairy_exp_up(
                        &mut remaining,
                        FairyExpRuntime {
                            player_id: owner_id,
                            fairy_guid: &fairy_guid,
                            fairy_name: &fairy_name,
                            log_value: 0,
                            suppress_grow_log: false,
                            grow_log_enabled,
                            egg_max_level,
                            upgrade_rate,
                        },
                        &mut *fairy_threshold_for_level,
                    )
                    .map_err(|error| FairyContainerExpFailure {
                        position,
                        error,
                        entries: std::mem::take(&mut entries),
                    })?
                else {
                    continue;
                };
                if exp.result <= FairyExpUpResult::None {
                    (exp, Delivery::None)
                } else {
                    goods
                        .save_fairy_properties(factory)
                        .expect("stored fairy сохраняет живую catalog entry");
                    if exp.result == FairyExpUpResult::ChangeState {
                        (exp, Delivery::ChangeState(goods.identity().ex_id))
                    } else {
                        (
                            exp,
                            Delivery::Update(FairyContainerGoodsUpdate {
                                message_type: FAIRY_GOODS_UPDATE_MESSAGE_TYPE,
                                player_id: owner_id,
                                goods: goods.identity(),
                                old_client_payload: encode_old_client(goods),
                            }),
                        )
                    }
                }
            };

            match delivery {
                Delivery::None => {}
                Delivery::Update(update) => entries.push(FairyContainerExpEntry::Updated {
                    position,
                    exp,
                    update,
                }),
                Delivery::ChangeState(goods_id) => {
                    let transition = self.fairy_change_state(
                        goods_id,
                        FairyState::Young,
                        position,
                        factory,
                        owner_progress_allows,
                        fairy_threshold_for_level,
                        create_goods,
                        encode_old_client,
                    );
                    entries.push(FairyContainerExpEntry::StateChanged {
                        position,
                        exp,
                        transition,
                    });
                }
            }
        }
        Ok(entries)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn implant_exp(
        &mut self,
        experience: u32,
        grow_log_enabled: bool,
        egg_max_level: u32,
        upgrade_rate: f32,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Result<Option<FairyImplantReport>, FairyExpBlock> {
        let owner_id = self.base.base().base().owner_id();
        let (old_level, exp, change_state) = {
            let Some(goods) = self.base.get_goods_mut(0) else {
                return Ok(None);
            };
            let Some(old_level) = goods.fairy_properties().map(|fairy| fairy.level) else {
                return Ok(None);
            };
            let fairy_guid = goods.identity().ex_id.to_string().into_bytes();
            let fairy_name = goods.name().to_vec();
            let mut remaining = experience;
            let Some(exp) = goods.fairy_exp_up(
                &mut remaining,
                FairyExpRuntime {
                    player_id: owner_id,
                    fairy_guid: &fairy_guid,
                    fairy_name: &fairy_name,
                    log_value: 0,
                    suppress_grow_log: false,
                    grow_log_enabled,
                    egg_max_level,
                    upgrade_rate,
                },
                &mut *fairy_threshold_for_level,
            )?
            else {
                return Ok(None);
            };
            if exp.result > FairyExpUpResult::None {
                goods
                    .save_fairy_properties(factory)
                    .expect("stored implantation fairy сохраняет catalog entry");
            }
            let change_state =
                (exp.result == FairyExpUpResult::ChangeState).then_some(goods.identity().ex_id);
            (old_level, exp, change_state)
        };

        let delivery = if let Some(goods_id) = change_state {
            FairyImplantDelivery::StateChanged(self.fairy_change_state(
                goods_id,
                FairyState::Young,
                10,
                factory,
                owner_progress_allows,
                fairy_threshold_for_level,
                create_goods,
                encode_old_client,
            ))
        } else {
            let goods = self
                .base
                .get_goods(0)
                .expect("implantation source остаётся в slot 0 без state change");
            FairyImplantDelivery::Update(FairyContainerGoodsUpdate {
                message_type: FAIRY_GOODS_UPDATE_MESSAGE_TYPE,
                player_id: owner_id,
                goods: goods.identity(),
                old_client_payload: encode_old_client(goods),
            })
        };
        let resulting_goods = match &delivery {
            FairyImplantDelivery::Update(_) => self.base.get_goods(0),
            FairyImplantDelivery::StateChanged(FairyStateChangeOutcome::Changed { .. }) => {
                self.base.get_goods(10)
            }
            FairyImplantDelivery::StateChanged(_) => None,
        };
        let Some(resulting_goods) = resulting_goods else {
            return Ok(None);
        };
        let resulting_level = resulting_goods
            .fairy_properties()
            .map_or(old_level, |fairy| fairy.level);
        let old_client_payload = match &delivery {
            FairyImplantDelivery::Update(update) => update.old_client_payload.clone(),
            FairyImplantDelivery::StateChanged(_) => encode_old_client(resulting_goods),
        };
        Ok(Some(FairyImplantReport {
            old_level,
            exp,
            goods: resulting_goods.identity(),
            goods_name: resulting_goods.name().to_vec(),
            resulting_level,
            old_client_payload,
            delivery,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fairy_syncretize(
        &mut self,
        property: FairySyncreticProperty,
        successful_roll: bool,
        player: Option<&mut FairySyncretizePlayer>,
        config: FairySyncretizeConfig,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> FairySyncretizeReport {
        const PRIMARY_POSITION: u32 = 11;
        const SECONDARY_POSITION: u32 = 12;
        const FRAGMENT_POSITION: u32 = 13;

        let Some(primary_goods) = self.base.get_goods(PRIMARY_POSITION) else {
            return Self::empty_syncretize_report(FairySyncreticResult::NoRipeFairyOne);
        };
        let Some(primary) = primary_goods
            .fairy_properties()
            .filter(|fairy| fairy.fairy_state == FairyState::Ripe as u32)
            .cloned()
        else {
            return Self::empty_syncretize_report(FairySyncreticResult::NoRipeFairyOne);
        };
        let primary_identity = primary_goods.identity();
        let primary_name = legacy_name_31(primary_goods.name());

        let Some(secondary_goods) = self.base.get_goods(SECONDARY_POSITION) else {
            return Self::empty_syncretize_report(FairySyncreticResult::NoRipeFairyTwo);
        };
        let Some(secondary) = secondary_goods
            .fairy_properties()
            .filter(|fairy| fairy.fairy_state == FairyState::Ripe as u32)
            .cloned()
        else {
            return Self::empty_syncretize_report(FairySyncreticResult::NoRipeFairyTwo);
        };
        let secondary_identity = secondary_goods.identity();
        let secondary_name = legacy_name_31(secondary_goods.name());
        if primary.main_ability != secondary.main_ability {
            return Self::empty_syncretize_report(FairySyncreticResult::DifferentMainProperty);
        }
        if primary.max_combinated_times
            <= primary
                .combinated_times
                .wrapping_add(secondary.combinated_times)
        {
            return Self::empty_syncretize_report(FairySyncreticResult::SyncreticTimesError);
        }

        let Some(fragment) = self.base.get_goods(FRAGMENT_POSITION) else {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughFragment);
        };
        if fragment.amount() < config.needed_goods {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughFragment);
        }
        let Some(fragment_properties) =
            factory.query_goods_base_properties(fragment.base_properties_index())
        else {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughFragment);
        };
        if fragment_properties.original_name() != FAIRY_SPECIAL_ORIGINAL_NAME {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughFragment);
        }
        let fragment_identity = fragment.identity();

        let Some(player) = player else {
            return Self::empty_syncretize_report(FairySyncreticResult::Unknown);
        };
        let consume_times = secondary.combinated_times.wrapping_add(1);
        let required_experience = consume_times.wrapping_mul(config.needed_experience);
        if player.experience < required_experience {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughExperience);
        }
        let required_money = consume_times.wrapping_mul(config.needed_money);
        if player.money < required_money {
            return Self::empty_syncretize_report(FairySyncreticResult::NotEnoughMoney);
        }

        let primary_goods = self
            .base
            .get_goods_mut(PRIMARY_POSITION)
            .expect("primary position не менялась после validation");
        let primary_mut = primary_goods
            .fairy_properties_mut()
            .expect("primary fairy проверена до mutation");
        if !successful_roll {
            let (primary_rate, secondary_rate) = match property {
                FairySyncreticProperty::FairyAttribute => (config.rate_c, config.rate_d),
                FairySyncreticProperty::GrowingRate => (config.rate_a, config.rate_b),
            };
            primary_mut.base_strength = blend_syncretic_base(
                primary_mut.base_strength,
                secondary.base_strength,
                consume_times,
                primary_rate,
                secondary_rate,
            );
            primary_mut.base_agility = blend_syncretic_base(
                primary_mut.base_agility,
                secondary.base_agility,
                consume_times,
                primary_rate,
                secondary_rate,
            );
            primary_mut.base_wakan = blend_syncretic_base(
                primary_mut.base_wakan,
                secondary.base_wakan,
                consume_times,
                primary_rate,
                secondary_rate,
            );
            primary_mut.base_hp = blend_syncretic_base(
                primary_mut.base_hp,
                secondary.base_hp,
                consume_times,
                primary_rate,
                secondary_rate,
            );
        } else {
            let (primary_rate, secondary_rate) = match property {
                FairySyncreticProperty::FairyAttribute => (config.rate_g, config.rate_h),
                FairySyncreticProperty::GrowingRate => (config.rate_e, config.rate_f),
            };
            primary_mut.growing_rate = blend_syncretic_base(
                primary_mut.growing_rate,
                secondary.growing_rate,
                consume_times,
                primary_rate,
                secondary_rate,
            );
        }
        let primary_times = primary_mut.combinated_times.wrapping_add(1);
        primary_mut.strength = blend_syncretic_visible(
            primary_mut.strength,
            primary_times,
            secondary.strength,
            consume_times,
            config.rate_n,
            config.rate_y,
            false,
        );
        primary_mut.agility = blend_syncretic_visible(
            primary_mut.agility,
            primary_times,
            secondary.agility,
            consume_times,
            config.rate_n,
            config.rate_y,
            false,
        );
        primary_mut.wakan = blend_syncretic_visible(
            primary_mut.wakan,
            primary_times,
            secondary.wakan,
            consume_times,
            config.rate_n,
            config.rate_y,
            true,
        );
        primary_mut.hp = blend_syncretic_visible(
            primary_mut.hp,
            primary_times,
            secondary.hp,
            consume_times,
            config.rate_n,
            config.rate_y,
            false,
        );
        primary_mut.fairy_state = FairyState::Egg as u32;
        primary_mut.level = 1;
        if primary_mut.experience().is_some() {
            primary_mut.link_experience(0);
        }
        primary_mut.combinated_times = primary_mut.combinated_times.wrapping_add(consume_times);
        primary_goods
            .save_fairy_properties(factory)
            .expect("primary fairy сохраняет живую catalog entry");

        let state_change = self.fairy_change_state(
            primary_identity.ex_id,
            FairyState::Ripe,
            PRIMARY_POSITION,
            factory,
            owner_progress_allows,
            fairy_threshold_for_level,
            create_goods,
            encode_old_client,
        );
        let mut report = Self::empty_syncretize_report(FairySyncreticResult::Unknown);
        let state_changed = matches!(state_change, FairyStateChangeOutcome::Changed { .. });
        report.state_change = Some(state_change);
        if !state_changed {
            return report;
        }

        let secondary_removal = self.delete_syncretize_goods(secondary_identity.ex_id);
        let secondary_removed = matches!(secondary_removal, FairySyncretizeRemoval::Removed { .. });
        report.secondary_removal = Some(secondary_removal);
        if !secondary_removed {
            return report;
        }

        if self
            .base
            .get_goods(FRAGMENT_POSITION)
            .is_some_and(|goods| config.needed_goods < goods.amount())
        {
            let owner_type = self.base.base().base().owner_type();
            let owner_id = self.base.base().base().owner_id();
            let fragment = self
                .base
                .get_goods_mut(FRAGMENT_POSITION)
                .expect("fragment проверен непосредственно перед mutation");
            fragment.set_amount(fragment.amount().wrapping_sub(config.needed_goods));
            report.fragment_effect = Some(FairySyncretizeFragmentEffect::AmountChanged(
                FairyContainerAmountChange {
                    owner_type,
                    owner_id,
                    position: FRAGMENT_POSITION,
                    container_extend_id: FAIRY_CONTAINER_EXTEND_ID,
                    goods: fragment_identity,
                    amount: fragment.amount(),
                },
            ));
        } else {
            let removal = self.delete_syncretize_goods(fragment_identity.ex_id);
            let fragment_removed = matches!(removal, FairySyncretizeRemoval::Removed { .. });
            report.fragment_effect = Some(match removal {
                FairySyncretizeRemoval::Removed { event, effects } => {
                    FairySyncretizeFragmentEffect::Removed { event, effects }
                }
                FairySyncretizeRemoval::Failed(removal) => {
                    FairySyncretizeFragmentEffect::RemovalFailed(removal)
                }
            });
            if !fragment_removed {
                return report;
            }
        }

        player.money = player.money.wrapping_sub(required_money);
        player.experience = player.experience.wrapping_sub(required_experience);
        report.player_update = Some(FairySyncretizePlayerUpdate {
            player_id: player.id,
            experience: player.experience,
            vigour: player.vigour,
            money: player.money,
        });
        report.result = if successful_roll {
            FairySyncreticResult::Successful
        } else {
            FairySyncreticResult::Failed
        };

        if config.log_enabled
            && let Some(result_goods) = self.base.get_goods(PRIMARY_POSITION)
            && let Some(result_fairy) = result_goods.fairy_properties()
        {
            report.log = Some(FairySyncretizeLog {
                message_type: FAIRY_WORLD_LOG_MESSAGE_TYPE,
                log_type: 4,
                player_id: self.base.base().base().owner_id(),
                primary_guid: primary_identity.ex_id,
                primary_name,
                primary_level: primary.level,
                primary_growing_rate: primary.growing_rate,
                secondary_guid: secondary_identity.ex_id,
                secondary_name,
                secondary_level: secondary.level,
                secondary_growing_rate: secondary.growing_rate,
                needed_goods: config.needed_goods,
                result_guid: result_goods.identity().ex_id,
                result_name: legacy_name_31(result_goods.name()),
                main_ability: primary.main_ability,
                combinated_times: result_fairy.combinated_times,
                growing_rate: result_fairy.growing_rate,
            });
        }
        report
    }

    fn empty_syncretize_report(result: FairySyncreticResult) -> FairySyncretizeReport {
        FairySyncretizeReport {
            result,
            state_change: None,
            secondary_removal: None,
            fragment_effect: None,
            player_update: None,
            log: None,
        }
    }

    fn delete_syncretize_goods(&mut self, goods_id: CGuid) -> FairySyncretizeRemoval {
        let removal = self.remove(goods_id);
        let FairyContainerRemoveOutcome::Removed(removed) = removal else {
            return FairySyncretizeRemoval::Failed(removal);
        };
        let taken = match removed {
            VolumeGoodsRemoveOutcome::Removed(taken)
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
        };
        let AmountLimitGoodsTaken::Removed(removed) = taken else {
            unreachable!("full syncretize remove cannot split goods")
        };
        let identity = removed.goods.identity();
        let event = FairyContainerRemovedEvent {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position,
            amount: removed.amount,
            listeners: removed.listeners,
        };
        let effects = vec![
            FairyStateChangeEffect::ObjectMove(FairyContainerObjectMove {
                operation: FairyContainerMoveOperation::DeleteObject,
                owner_type: event.owner_type,
                owner_id: event.owner_id,
                position: event.position,
                container_extend_id: FAIRY_CONTAINER_EXTEND_ID,
                goods: identity,
                amount: event.amount,
                old_client_payload: Vec::new(),
            }),
            FairyStateChangeEffect::GarbageCollected(identity),
        ];
        drop(removed.goods);
        FairySyncretizeRemoval::Removed { event, effects }
    }

    /// Base payload сохраняет собственного owner-а; fairy suffix всегда
    /// дописывает пять little-endian hatch значений для ячеек `5..9`, даже
    /// когда base serializer вернул `false`.
    pub(crate) fn serialize_with<SerializeBase>(
        &self,
        destination: &mut Vec<u8>,
        include_ex_data: bool,
        serialize_base: SerializeBase,
    ) -> bool
    where
        SerializeBase: FnOnce(&CVolumeLimitGoodsContainer, &mut Vec<u8>, bool) -> bool,
    {
        let result = serialize_base(&self.base, destination, include_ex_data);
        for position in HATCHER_POSITIONS {
            let hatch_start_time = self
                .base
                .get_goods(position)
                .and_then(CGoods::fairy_properties)
                .map_or(0, |fairy| fairy.hatch_start_time);
            LegacyWriter::new(destination).write_u32(hatch_start_time);
        }
        result
    }

    /// Exact owner очищает container до base decoder-а. Suffix применяет
    /// ненулевые timer-ы по одному и при обрыве сохраняет уже восстановленный
    /// prefix state.
    pub(crate) fn unserialize_with<UnserializeBase>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_ex_data: bool,
        unserialize_base: UnserializeBase,
    ) -> Result<FairyContainerUnserializeReport, FairyContainerUnserializeFailure>
    where
        UnserializeBase: FnOnce(&mut CVolumeLimitGoodsContainer, &[u8], &mut usize, bool) -> bool,
    {
        let cleared = self.base.clear_goods();
        let base_result = unserialize_base(&mut self.base, source, cursor, include_ex_data);
        let mut report = FairyContainerUnserializeReport {
            cleared,
            base_result,
            restored_hatch_positions: Vec::new(),
        };
        for position in HATCHER_POSITIONS {
            let offset = *cursor;
            let Some(end) = offset.checked_add(4) else {
                return Err(FairyContainerUnserializeFailure {
                    error: FairyContainerCodecError::UnexpectedEnd {
                        position,
                        offset,
                        available: source.len().saturating_sub(offset),
                    },
                    report,
                });
            };
            let available = source.len().saturating_sub(offset);
            let mut reader = match LegacyReader::at(source, offset) {
                Ok(reader) => reader,
                Err(_) => {
                    *cursor = end;
                    return Err(FairyContainerUnserializeFailure {
                        error: FairyContainerCodecError::UnexpectedEnd {
                            position,
                            offset,
                            available,
                        },
                        report,
                    });
                }
            };
            let hatch_start_time = match reader.read_u32() {
                Ok(value) => {
                    *cursor = reader.position();
                    value
                }
                Err(_) => {
                    // Этот владелец, в отличие от основного кодека, продвигает курсор до ошибки.
                    *cursor = end;
                    return Err(FairyContainerUnserializeFailure {
                        error: FairyContainerCodecError::UnexpectedEnd { position, offset, available },
                        report,
                    });
                }
            };
            if hatch_start_time == 0 {
                continue;
            }
            let Some(fairy) = self
                .base
                .get_goods_mut(position)
                .and_then(CGoods::fairy_properties_mut)
            else {
                continue;
            };
            fairy.hatch_start_time = hatch_start_time;
            report.restored_hatch_positions.push(position);
        }
        Ok(report)
    }
}

fn hatch_stat(value: u32, base: u32) -> u32 {
    ((value as f64 + base as f64 * 0.0001_f64).round() as i64 as i32) as u32
}

fn blend_syncretic_base(
    primary: u32,
    secondary: u32,
    secondary_times: u32,
    primary_rate: f32,
    secondary_rate: f32,
) -> u32 {
    let primary = primary as f32;
    let secondary = secondary as f32;
    round_syncretic(
        primary + primary * primary_rate + secondary * secondary_times as f32 * secondary_rate,
    )
}

fn blend_syncretic_visible(
    primary: u32,
    primary_times: u32,
    secondary: u32,
    secondary_times: u32,
    primary_rate: f32,
    secondary_rate: f32,
    secondary_term_first: bool,
) -> u32 {
    let primary_term = primary as f32 * primary_times as f32 * primary_rate;
    let secondary_term = secondary as f32 * secondary_times as f32 * secondary_rate;
    let combined = if secondary_term_first {
        secondary_term + primary_term
    } else {
        primary_term + secondary_term
    };
    round_syncretic(combined)
}

fn round_syncretic(value: f32) -> u32 {
    (value.round() as i64 as i32) as u32
}

fn legacy_name_31(name: &[u8]) -> Vec<u8> {
    let visible = &name[..name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len())];
    visible[..visible.len().min(31)].to_vec()
}
