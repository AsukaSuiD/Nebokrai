//! Storage-prefix `CFairyContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cfairycontainer.cpp`. Контейнер
//! владеет ordinary-fairy товарами через `CVolumeLimitGoodsContainer`:
//! auto-add сначала выбирает ячейку base-owner-а, positional add сохраняет
//! отдельное правило `0..12` для headgear с fairy property и исключение
//! `13/FZ0885`, а remove блокирует фею с ненулевым hatch timer.
//!
//! `CVolumeLimitGoodsContainer` и owned `CGoods` заменяют vtable dispatch/raw
//! pointers, не меняя lock/stack/listener semantics base-owner-а. Пять hatch
//! timer-ов codec suffix сохранены с partial decode, а state change — с
//! необратимым remove/add и detached replacement на отказе. Hatch/exp traversal
//! и syncretize ниже пока остаются RAW.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCleared, AmountLimitGoodsTaken,
};
use super::ccontainer::ContainerListenerHandle;
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddBlock, VolumeGoodsAddOutcome,
    VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_BF_BATTLE_FAIRY, GAP_PARTICULAR_ATTRIBUTE,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;

const FAIRY_SPECIAL_POSITION: u32 = 13;
const FAIRY_SPECIAL_ORIGINAL_NAME: &[u8] = b"FZ0885";
const HATCHER_POSITIONS: std::ops::Range<u32> = 5..10;
const FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0b;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerCodecError {
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
pub(crate) struct CFairyContainer {
    base: CVolumeLimitGoodsContainer,
}

impl Default for CFairyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CFairyContainer {
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
            destination.extend_from_slice(&hatch_start_time.to_le_bytes());
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
            *cursor = end;
            let Some(bytes) = source.get(offset..end) else {
                return Err(FairyContainerUnserializeFailure {
                    error: FairyContainerCodecError::UnexpectedEnd {
                        position,
                        offset,
                        available: source.len().saturating_sub(offset),
                    },
                    report,
                });
            };
            let hatch_start_time = u32::from_le_bytes(
                bytes
                    .try_into()
                    .expect("fairy hatch suffix содержит четыре байта"),
            );
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp

// ============================================================================
// FUNCTION: CFairyContainer::CFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:22
// RVA: 0x000DB650
// ADDRESS: 004db650
// PROTOTYPE: undefined __thiscall CFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:41
// RVA: 0x000DB680
// ADDRESS: 004db680
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::~CFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:26
// RVA: 0x000DB6A0
// ADDRESS: 004db6a0
// PROTOTYPE: void __thiscall ~CFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:88
// RVA: 0x000DB700
// ADDRESS: 004db700
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairyChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:443
// RVA: 0x000DB750
// ADDRESS: 004db750
// PROTOTYPE: CGoods * __thiscall FairyChangeState(CGoods * param_1, EFairyState param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:518
// RVA: 0x000DBB60
// ADDRESS: 004dbb60
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:538
// RVA: 0x000DBBC0
// ADDRESS: 004dbbc0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::CheckHatcher
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:151
// RVA: 0x000DBC50
// ADDRESS: 004dbc50
// PROTOTYPE: void __thiscall CheckHatcher(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:46
// RVA: 0x000DBDE0
// ADDRESS: 004dbde0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairyExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:106
// RVA: 0x000DBF00
// ADDRESS: 004dbf00
// PROTOTYPE: void __thiscall FairyExpUp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairySyncretize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:186
// RVA: 0x000DC190
// ADDRESS: 004dc190
// PROTOTYPE: ESyncreticResult __thiscall FairySyncretize(ESyncreticProperty param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
