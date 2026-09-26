//! Позиционное owning-ядро `CEquipmentContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cequipmentcontainer.cpp`. Семнадцать
//! колонок `0..=16`, их deterministic enum-order, lookup-ы, подсчёты, weight,
//! `Clear` и mode-зависимый `Release` материализованы по RVA
//! `0x000ED180..0x000EDB30` и `0x000EEBF0`. `BTreeMap` заменяет legacy
//! `std::map`, а owned `CGoods` — сырые указатели без изменения порядка.
//!
//! Exact EXE сравнивает число непустых колонок именно с 17. Вопреки позднему
//! архивному донору, `Clear` и `Release` не обнуляют `m_nExpantPkgNum`;
//! `Release` очищает external listeners через `CGoodsContainer::Release`, а
//! внутренний self-listener представлен прямым derived callback-ом и потому
//! не хранится как самоссылка. `Clear` возвращает ordered reports вместе с
//! владением товарами, а `Release` в test-mode возвращает detached товары;
//! это безопасная замена legacy pointer lifetime/`GarbageCollect`.
//!
//! Add/remove/swap сохраняют player facts как явный runtime-вход, partial
//! timed/AI/package effects, typed skill/property/message callbacks и rollback
//! loss. Timed-prefix передаёт изменяемый товар в GoodsAI до проверки equip-place и
//! захвата слота; назначенный ticket сохраняется и при последующем отказе.
//! Ordinary/battle-fairy growth замкнут через свойства `CGoods`, включая
//! old-client update и необратимый delete/add transition; конкретный goods
//! payload codec и player dispatcher остаются callback-границами связанных
//! owners. Add-report отдельно хранит факт применения package-extension, так
//! как native пишет `PackExpand` log и при нулевой дельте. Внешний equipment
//! codec сохраняет wire-order и partial decode.
//! Достигнутое ядро не выдаётся за весь контейнер.

use std::collections::BTreeMap;

use super::ccontainer::ContainerListenerHandle;
use super::cgoodscontainer::CGoodsContainer;
use crate::gameserver::appserver::goods::cbattlefairyproperty::{
    BattleFairyExpBlock, BattleFairyExpReport, BattleFairyExpUpResult, BattleFairyPlayerFacts,
};
use crate::gameserver::appserver::goods::cgoods::{CGoods, GoodsBasePropertyBlock, GoodsDecodeError};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_BODY, EQUIP_PLACE_BOOT, EQUIP_PLACE_FAIRY, EQUIP_PLACE_FROCK, EQUIP_PLACE_GLOVE,
    EQUIP_PLACE_HAND, EQUIP_PLACE_HEAD, EQUIP_PLACE_HEADGEAR, EQUIP_PLACE_JEWELRY,
    EQUIP_PLACE_LING_BAO, EQUIP_PLACE_MANTEAU, EQUIP_PLACE_MEDAL, EQUIP_PLACE_ORNAMENTS,
    EQUIP_PLACE_POSTERIOR, EQUIP_PLACE_TALISMAN, EQUIP_PLACE_WING, GAP_BF_BATTLE_FAIRY,
    GAP_BF_BFEQUIPEMENT, GAP_BF_LEVEL, GAP_GOODS_PACKAGE_EXTENTION, GAP_WEAPON_LEVEL,
    GOODS_TYPE_EQUIPMENT,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::goods::fairyproperties::{
    FairyExpBlock, FairyExpReport, FairyExpRuntime, FairyExpUpResult,
};
use crate::gameserver::appserver::gameeffectjournal::GameEffectJournal;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::ShapeIdentity;
use nebokrai_shared::values::CGuid;
use thiserror::Error;

pub(crate) const EQUIPMENT_COLUMN_LIMIT: u32 = 17;
pub(crate) const EQUIPMENT_AROUND_UPDATE_MESSAGE_TYPE: u32 = 0x0b_f720;
pub(crate) const EQUIPMENT_FAIRY_UPDATE_MESSAGE_TYPE: u32 = 0x0b_f918;
pub(crate) const EQUIPMENT_CONTAINER_EXTEND_ID: u32 = 2;
const PLAYER_OWNER_TYPE: i32 = 400;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EquipmentColumn {
    Head = 0,
    Body = 1,
    Hand = 2,
    Glove = 3,
    Boot = 4,
    Jewelry = 5,
    OrnamentsOne = 6,
    OrnamentsTwo = 7,
    Medal = 8,
    Posterior = 9,
    Headgear = 10,
    Talisman = 11,
    Frock = 12,
    Wing = 13,
    Manteau = 14,
    Fairy = 15,
    LingBao = 16,
}

impl EquipmentColumn {
    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        Some(match position {
            0 => Self::Head,
            1 => Self::Body,
            2 => Self::Hand,
            3 => Self::Glove,
            4 => Self::Boot,
            5 => Self::Jewelry,
            6 => Self::OrnamentsOne,
            7 => Self::OrnamentsTwo,
            8 => Self::Medal,
            9 => Self::Posterior,
            10 => Self::Headgear,
            11 => Self::Talisman,
            12 => Self::Frock,
            13 => Self::Wing,
            14 => Self::Manteau,
            15 => Self::Fairy,
            16 => Self::LingBao,
            _ => return None,
        })
    }

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }
}

#[must_use = "report сохраняет порядок remove-callback-ов и владение товаром"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentClearedGoods {
    pub(crate) column: EquipmentColumn,
    pub(crate) goods: CGoods,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentAddPartialEffects {
    pub(crate) start_point_initialized: bool,
    pub(crate) registered_with_goods_ai: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentAddBlock {
    MissingGoods,
    BattleFairyEquipment,
    MountRejected {
        result: i32,
    },
    Occupied {
        column: EquipmentColumn,
    },
    MissingBaseProperties {
        index: u32,
    },
    NotEquipment {
        goods_type: i32,
    },
    InvalidPosition {
        position: u32,
    },
    EquipPlaceMismatch {
        column: EquipmentColumn,
        equip_place: i32,
    },
    UnsupportedEquipPlace {
        equip_place: i32,
    },
    NoFreeOrnamentSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentOwnerPlayerFacts {
    /// Exact `CanMountEquip` должен вернуть magic success-code 9.
    pub(crate) can_mount_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentAddRuntimeFacts {
    /// `None` означает, что lookup игрока по owner id не дал объект.
    pub(crate) owner_player: Option<EquipmentOwnerPlayerFacts>,
    pub(crate) pack_add_enabled: bool,
    pub(crate) now: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentAroundUpdate {
    pub(crate) owner_id: i32,
    pub(crate) column: EquipmentColumn,
    pub(crate) added: bool,
    pub(crate) base_properties_index: u32,
    pub(crate) weapon_level: u32,
    /// Add исключает owner-а из around-send, Remove передаёт null exclusion.
    pub(crate) exclude_owner: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentPlayerAddedEffects {
    pub(crate) add_war_soul_skill: bool,
    pub(crate) recompute_properties: bool,
    pub(crate) around_update: EquipmentAroundUpdate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentPlayerRemovedEffects {
    pub(crate) clear_war_soul_status: bool,
    pub(crate) delete_war_soul_skill: bool,
    pub(crate) recompute_without_removed_slot: bool,
    pub(crate) clamp_hp_and_mp: bool,
    pub(crate) around_update: EquipmentAroundUpdate,
}

#[must_use = "report содержит обязательные internal/player/listener эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentAddedReport {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) column: EquipmentColumn,
    pub(crate) identity: ShapeIdentity,
    pub(crate) base_properties_index: u32,
    pub(crate) amount: u32,
    pub(crate) partial_effects: EquipmentAddPartialEffects,
    pub(crate) player_effects: Option<EquipmentPlayerAddedEffects>,
    pub(crate) package_extension_applied: bool,
    pub(crate) package_extension_delta: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "blocked add может уже содержать timed/AI partial effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentAddBlockedReport {
    pub(crate) reason: EquipmentAddBlock,
    pub(crate) partial_effects: EquipmentAddPartialEffects,
}

#[must_use = "blocked add может уже содержать timed/AI partial effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentAddOutcome {
    Added(EquipmentAddedReport),
    Blocked(EquipmentAddBlockedReport),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentRemovePartialEffects {
    /// Значение уже вычтено из `expanded_package_num` с wrapping semantics.
    pub(crate) package_extension_subtracted: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentRemoveRuntimeFacts {
    /// Lookup игрока по owner id для derived remove-callback-а.
    pub(crate) owner_player_present: bool,
    pub(crate) pack_add_enabled: bool,
    /// Результат player-wide `GetGoodsById` после QueryAttribute/value1==2.
    pub(crate) player_goods_package_extension: Option<u32>,
    /// Уже вычисленный exact skill gate `battle fairy && Can...()==0`.
    pub(crate) active_war_soul_blocks_headgear: bool,
}

#[must_use = "report сохраняет removed ownership и callback ordering"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentRemovedEvent {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) column: EquipmentColumn,
    pub(crate) partial_effects: EquipmentRemovePartialEffects,
    pub(crate) player_effects: Option<EquipmentPlayerRemovedEffects>,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "report сохраняет removed ownership и callback ordering"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentRemovedReport {
    pub(crate) event: EquipmentRemovedEvent,
    pub(crate) goods: CGoods,
}

#[must_use = "missing/blocked remove может уже изменить package-extension счётчик"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentRemoveOutcome {
    Removed(EquipmentRemovedReport),
    Missing {
        partial_effects: EquipmentRemovePartialEffects,
    },
    BlockedByActiveWarSoul {
        column: EquipmentColumn,
        partial_effects: EquipmentRemovePartialEffects,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSwapRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    /// `None` означает отсутствие player-а для pre-remove capacity guard.
    pub(crate) active_unused_package_slots: Option<u32>,
    pub(crate) remove: EquipmentRemoveRuntimeFacts,
    pub(crate) incoming_add: EquipmentAddRuntimeFacts,
    /// Новый clock/player snapshot для legacy rollback `Add(old)`.
    pub(crate) rollback_add: EquipmentAddRuntimeFacts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSwapBlock {
    MissingIncoming,
    MissingBaseProperties {
        index: u32,
    },
    NotEquipment {
        goods_type: i32,
    },
    EquipPlaceMismatch {
        column: EquipmentColumn,
        equip_place: i32,
    },
    Empty {
        column: EquipmentColumn,
    },
    InsufficientPackageCapacity {
        required: u32,
        available: u32,
    },
}

#[must_use = "swap outcome содержит все remove/add/rollback partial effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSwapOutcome {
    Blocked(EquipmentSwapBlock),
    RemovalFailed(EquipmentRemoveOutcome),
    Swapped {
        outgoing: CGoods,
        removed: EquipmentRemovedEvent,
        added: EquipmentAddedReport,
    },
    IncomingRejectedAndRestored {
        removed: EquipmentRemovedEvent,
        incoming: EquipmentAddBlockedReport,
        restored: EquipmentAddedReport,
    },
    IncomingRejectedAndOldGoodsCollected {
        removed: EquipmentRemovedEvent,
        incoming: EquipmentAddBlockedReport,
        rollback: EquipmentAddBlockedReport,
        garbage_collected: ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum EquipmentContainerCodecError {
    #[error(transparent)]
    Goods(#[from] GoodsDecodeError),
    #[error("equipment container обрывается на {field} в {offset}: доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        available: usize,
    },
    #[error("equipment container получил отрицательное число goods {count}")]
    NegativeGoodsCount {
        count: i32,
    },
}

#[must_use = "entry outcome может вернуть rejected owned goods"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentUnserializedEntry {
    DecoderReturnedNull {
        position: u32,
    },
    Added {
        position: u32,
        add: EquipmentAddedReport,
        to_add_extension_delta: u32,
        write_pack_expand_log: bool,
    },
    Rejected {
        position: u32,
        goods: CGoods,
        add: EquipmentAddBlockedReport,
    },
}

#[must_use = "report содержит cleared ownership и все применённые entries"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentUnserializeReport {
    pub(crate) cleared: Vec<EquipmentClearedGoods>,
    pub(crate) entries: Vec<EquipmentUnserializedEntry>,
}

#[must_use = "failure сохраняет уже очищенный и частично заполненный state"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentUnserializeFailure {
    pub(crate) error: EquipmentContainerCodecError,
    pub(crate) report: EquipmentUnserializeReport,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EquipmentFairyExpRuntimeFacts {
    pub(crate) grow_log_enabled: bool,
    pub(crate) egg_max_level: u32,
    pub(crate) upgrade_rate: f32,
    pub(crate) remove: EquipmentRemoveRuntimeFacts,
    pub(crate) replacement_add: EquipmentAddRuntimeFacts,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EquipmentBattleFairyExpRuntimeFacts {
    pub(crate) player: Option<BattleFairyPlayerFacts>,
    pub(crate) remove: EquipmentRemoveRuntimeFacts,
    pub(crate) replacement_add: EquipmentAddRuntimeFacts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentFairyMoveOperation {
    DeleteObject,
    NewObject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentFairyObjectMove {
    pub(crate) operation: EquipmentFairyMoveOperation,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) column: EquipmentColumn,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentFairyGoodsUpdate {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentFairyTransitionEffect {
    ObjectMove(EquipmentFairyObjectMove),
    GarbageCollected(ShapeIdentity),
}

#[must_use = "transition сохраняет removal/add reports и порядок move/GC effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentFairyTransition {
    ReplacementCreationFailed {
        goods_index: u32,
    },
    RemovalFailed {
        removal: EquipmentRemoveOutcome,
        effects: Vec<EquipmentFairyTransitionEffect>,
    },
    ReplacementRejected {
        removed: EquipmentRemovedEvent,
        add: EquipmentAddBlockedReport,
        effects: Vec<EquipmentFairyTransitionEffect>,
    },
    Changed {
        removed: EquipmentRemovedEvent,
        added: EquipmentAddedReport,
        effects: Vec<EquipmentFairyTransitionEffect>,
    },
}

#[must_use = "outcome содержит property logs, update payload или transition effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentFairyExpOutcome {
    NoChange(FairyExpReport),
    Updated {
        exp: FairyExpReport,
        update: EquipmentFairyGoodsUpdate,
    },
    StateChanged {
        exp: FairyExpReport,
        transition: EquipmentFairyTransition,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentFairyExpBlock {
    Properties(FairyExpBlock),
    ReplacementProperties(GoodsBasePropertyBlock),
    ReplacementFairyPropertiesMissing,
}

#[must_use = "outcome содержит level logs, update payload или transition effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentBattleFairyExpOutcome {
    NoChange(BattleFairyExpReport),
    Updated {
        exp: BattleFairyExpReport,
        update: EquipmentFairyGoodsUpdate,
    },
    StateChanged {
        exp: BattleFairyExpReport,
        transition: EquipmentFairyTransition,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentBattleFairyExpBlock {
    Properties(BattleFairyExpBlock),
    ReplacementProperties(GoodsBasePropertyBlock),
    ReplacementBattleFairyPropertiesMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentContainer {
    base: CGoodsContainer,
    equipment: BTreeMap<EquipmentColumn, CGoods>,
    expanded_package_num: u32,
}

impl Default for CEquipmentContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            equipment: BTreeMap::new(),
            expanded_package_num: 0,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn expanded_package_num(&self) -> u32 {
        self.expanded_package_num
    }

    /// Player callback adapter временно возвращает pre-add snapshot, чтобы
    /// `PropertiesChanged` и around-send предшествовали native PackAdd tail-у.
    pub(crate) const fn set_expanded_package_num_snapshot(&mut self, value: u32) {
        self.expanded_package_num = value;
    }

    pub(crate) fn occupied_count(&self) -> u32 {
        self.equipment.len() as u32
    }

    /// Exact `IsFull` использует equality, а не `>=`.
    pub(crate) fn is_full(&self) -> bool {
        self.occupied_count() == EQUIPMENT_COLUMN_LIMIT
    }

    pub(crate) fn is_slot_empty(&self, column: EquipmentColumn) -> bool {
        !self.equipment.contains_key(&column)
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.equipment
            .get(&EquipmentColumn::from_position(position)?)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.equipment
            .get_mut(&EquipmentColumn::from_position(position)?)
    }

    pub(crate) fn find(&self, goods_id: CGuid) -> Option<&CGoods> {
        self.equipment
            .values()
            .find(|goods| goods.identity().ex_id == goods_id)
    }

    pub(crate) fn find_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        self.equipment
            .values_mut()
            .find(|goods| goods.identity().ex_id == goods_id)
    }

    pub(crate) fn query_goods_position_by_id(&self, goods_id: CGuid) -> Option<EquipmentColumn> {
        self.equipment
            .iter()
            .find_map(|(column, goods)| (goods.identity().ex_id == goods_id).then_some(*column))
    }

    /// Typed replacement исходного pointer-identity overload-а.
    pub(crate) fn query_goods_position(&self, goods: &CGoods) -> Option<EquipmentColumn> {
        self.equipment
            .iter()
            .find_map(|(column, stored)| std::ptr::eq(stored, goods).then_some(*column))
    }

    pub(crate) fn get_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.equipment
            .values()
            .find(|goods| goods.base_properties_index() == base_properties_index)
    }

    pub(crate) fn get_goods_by_base_properties(&self, base_properties_index: u32) -> Vec<&CGoods> {
        self.equipment
            .values()
            .filter(|goods| goods.base_properties_index() == base_properties_index)
            .collect()
    }

    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.get_first_goods(base_properties_index).is_some()
    }

    pub(crate) fn traversing_goods(&self) -> Vec<(EquipmentColumn, &CGoods)> {
        self.equipment
            .iter()
            .map(|(column, goods)| (*column, goods))
            .collect()
    }

    pub(crate) fn contents_weight(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .values()
            .fold(0, |total, goods| total.wrapping_add(goods.weight(factory)))
    }

    /// Exact owner считает только товары с живой catalog-записью.
    pub(crate) fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .values()
            .filter(|goods| {
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some()
            })
            .count() as u32
    }

    /// Exact `FairyExpUp` работает с headgear column, несмотря на отдельную
    /// legacy колонку `EC_FAIRY`. Пустой input не делает даже lookup-а.
    pub(crate) fn fairy_exp_up(
        &mut self,
        experience: u32,
        factory: &CGoodsFactory,
        runtime: EquipmentFairyExpRuntimeFacts,
        fairy_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Result<EquipmentFairyExpOutcome, EquipmentFairyExpBlock> {
        let mut remaining = experience;
        if remaining == 0 {
            return Ok(EquipmentFairyExpOutcome::NoChange(FairyExpReport {
                result: FairyExpUpResult::None,
                remaining_experience: remaining,
                effects: GameEffectJournal::default(),
            }));
        }

        let owner_id = self.base.owner_id();
        let Some(goods) = self.equipment.get_mut(&EquipmentColumn::Headgear) else {
            return Ok(EquipmentFairyExpOutcome::NoChange(FairyExpReport {
                result: FairyExpUpResult::None,
                remaining_experience: remaining,
                effects: GameEffectJournal::default(),
            }));
        };
        let fairy_guid = goods.identity().ex_id.to_string().into_bytes();
        let fairy_name = goods.name().to_vec();
        let fairy_runtime = FairyExpRuntime {
            player_id: owner_id,
            fairy_guid: &fairy_guid,
            fairy_name: &fairy_name,
            log_value: 0,
            suppress_grow_log: false,
            grow_log_enabled: runtime.grow_log_enabled,
            egg_max_level: runtime.egg_max_level,
            upgrade_rate: runtime.upgrade_rate,
        };
        let Some(exp) = goods
            .fairy_exp_up(
                &mut remaining,
                fairy_runtime,
                &mut *fairy_threshold_for_level,
            )
            .map_err(EquipmentFairyExpBlock::Properties)?
        else {
            return Ok(EquipmentFairyExpOutcome::NoChange(FairyExpReport {
                result: FairyExpUpResult::None,
                remaining_experience: remaining,
                effects: GameEffectJournal::default(),
            }));
        };
        if exp.result <= FairyExpUpResult::None {
            return Ok(EquipmentFairyExpOutcome::NoChange(exp));
        }
        goods
            .save_fairy_properties(factory)
            .expect("equipped goods сохраняет живую catalog entry");
        if exp.result != FairyExpUpResult::ChangeState {
            return Ok(EquipmentFairyExpOutcome::Updated {
                update: Self::fairy_goods_update(owner_id, goods, encode_old_client),
                exp,
            });
        }

        let ripe_id = goods
            .fairy_properties()
            .expect("успешный fairy exp report требует property owner")
            .ripe_id;
        let Some(mut replacement) = create_goods(ripe_id) else {
            return Ok(EquipmentFairyExpOutcome::StateChanged {
                exp,
                transition: EquipmentFairyTransition::ReplacementCreationFailed {
                    goods_index: ripe_id,
                },
            });
        };
        replacement
            .copy_fairy_addon_properties_from(goods, factory, &mut *fairy_threshold_for_level)
            .map_err(EquipmentFairyExpBlock::ReplacementProperties)?;
        let Some(replacement_fairy) = replacement.fairy_properties_mut() else {
            return Err(EquipmentFairyExpBlock::ReplacementFairyPropertiesMissing);
        };
        replacement_fairy.fairy_state = 2;
        if !replacement
            .save_fairy_properties(factory)
            .map_err(EquipmentFairyExpBlock::ReplacementProperties)?
        {
            return Err(EquipmentFairyExpBlock::ReplacementFairyPropertiesMissing);
        }
        let old_goods_id = goods.identity().ex_id;
        let transition = self.replace_headgear_fairy(
            old_goods_id,
            replacement,
            factory,
            runtime.remove,
            runtime.replacement_add,
            register_with_goods_ai,
            encode_old_client,
        );
        Ok(EquipmentFairyExpOutcome::StateChanged { exp, transition })
    }

    /// Exact BF guard — addon 172/value 1 == 1. Текущее `LevelUp` всегда
    /// возвращает `LevelUp`, но unreachable transition branch owner-а
    /// сохранён явно для совместимости с исходным enum-контрактом.
    pub(crate) fn battle_fairy_level_up(
        &mut self,
        experience: u32,
        factory: &CGoodsFactory,
        runtime: EquipmentBattleFairyExpRuntimeFacts,
        battle_threshold_for_level: &mut dyn FnMut(u32, u32) -> u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Result<EquipmentBattleFairyExpOutcome, EquipmentBattleFairyExpBlock> {
        let mut remaining = experience;
        if remaining == 0 {
            return Ok(EquipmentBattleFairyExpOutcome::NoChange(
                Self::empty_battle_fairy_exp_report(remaining),
            ));
        }
        let Some(goods) = self.equipment.get_mut(&EquipmentColumn::Headgear) else {
            return Ok(EquipmentBattleFairyExpOutcome::NoChange(
                Self::empty_battle_fairy_exp_report(remaining),
            ));
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return Ok(EquipmentBattleFairyExpOutcome::NoChange(
                Self::empty_battle_fairy_exp_report(remaining),
            ));
        }
        let Some(exp) = goods
            .battle_fairy_exp_up(
                factory,
                runtime.player,
                &mut remaining,
                &mut *battle_threshold_for_level,
            )
            .map_err(EquipmentBattleFairyExpBlock::Properties)?
        else {
            return Ok(EquipmentBattleFairyExpOutcome::NoChange(
                Self::empty_battle_fairy_exp_report(remaining),
            ));
        };
        if exp.result <= BattleFairyExpUpResult::None {
            return Ok(EquipmentBattleFairyExpOutcome::NoChange(exp));
        }
        goods
            .save_battle_fairy_property(factory)
            .expect("equipped goods сохраняет живую catalog entry");
        let owner_id = self.base.owner_id();
        if exp.result != BattleFairyExpUpResult::ChangeState {
            return Ok(EquipmentBattleFairyExpOutcome::Updated {
                update: Self::fairy_goods_update(owner_id, goods, encode_old_client),
                exp,
            });
        }

        let change_id = goods
            .battle_fairy_property()
            .expect("успешный battle-fairy exp report требует property owner")
            .change_id;
        let Some(mut replacement) = create_goods(change_id) else {
            return Ok(EquipmentBattleFairyExpOutcome::StateChanged {
                exp,
                transition: EquipmentFairyTransition::ReplacementCreationFailed {
                    goods_index: change_id,
                },
            });
        };
        replacement
            .copy_battle_fairy_addon_properties_from(
                goods,
                factory,
                &mut *battle_threshold_for_level,
            )
            .map_err(EquipmentBattleFairyExpBlock::ReplacementProperties)?;
        let Some(replacement_battle_fairy) = replacement.battle_fairy_property_mut() else {
            return Err(EquipmentBattleFairyExpBlock::ReplacementBattleFairyPropertiesMissing);
        };
        replacement_battle_fairy.module = Some(3);
        if !replacement
            .save_battle_fairy_property(factory)
            .map_err(EquipmentBattleFairyExpBlock::ReplacementProperties)?
        {
            return Err(EquipmentBattleFairyExpBlock::ReplacementBattleFairyPropertiesMissing);
        }
        let old_goods_id = goods.identity().ex_id;
        let transition = self.replace_headgear_fairy(
            old_goods_id,
            replacement,
            factory,
            runtime.remove,
            runtime.replacement_add,
            register_with_goods_ai,
            encode_old_client,
        );
        Ok(EquipmentBattleFairyExpOutcome::StateChanged { exp, transition })
    }

    fn empty_battle_fairy_exp_report(remaining_experience: u32) -> BattleFairyExpReport {
        BattleFairyExpReport {
            result: BattleFairyExpUpResult::None,
            remaining_experience,
            goods_update: None,
        }
    }

    fn fairy_goods_update(
        player_id: i32,
        goods: &CGoods,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> EquipmentFairyGoodsUpdate {
        EquipmentFairyGoodsUpdate {
            message_type: EQUIPMENT_FAIRY_UPDATE_MESSAGE_TYPE,
            player_id,
            goods: goods.identity(),
            old_client_payload: encode_old_client(goods),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn replace_headgear_fairy(
        &mut self,
        old_goods_id: CGuid,
        replacement: CGoods,
        factory: &CGoodsFactory,
        remove_runtime: EquipmentRemoveRuntimeFacts,
        add_runtime: EquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> EquipmentFairyTransition {
        let replacement_identity = replacement.identity();
        let removed = match self.remove(old_goods_id, factory, remove_runtime) {
            EquipmentRemoveOutcome::Removed(removed) => removed,
            removal => {
                return EquipmentFairyTransition::RemovalFailed {
                    removal,
                    effects: vec![EquipmentFairyTransitionEffect::GarbageCollected(
                        replacement_identity,
                    )],
                };
            }
        };
        let EquipmentRemovedReport {
            event: removed_event,
            goods: old_goods,
        } = removed;
        let old_identity = old_goods.identity();
        let mut effects = vec![
            EquipmentFairyTransitionEffect::ObjectMove(EquipmentFairyObjectMove {
                operation: EquipmentFairyMoveOperation::DeleteObject,
                owner_type: removed_event.owner_type,
                owner_id: removed_event.owner_id,
                column: removed_event.column,
                container_extend_id: EQUIPMENT_CONTAINER_EXTEND_ID,
                goods: old_identity,
                amount: old_goods.amount(),
                old_client_payload: Vec::new(),
            }),
            EquipmentFairyTransitionEffect::GarbageCollected(old_identity),
        ];
        drop(old_goods);

        let mut incoming = Some(replacement);
        match self.add_preferred(&mut incoming, factory, add_runtime, register_with_goods_ai) {
            EquipmentAddOutcome::Blocked(add) => {
                let rejected = incoming
                    .take()
                    .expect("rejected fairy replacement сохраняет ownership");
                effects.push(EquipmentFairyTransitionEffect::GarbageCollected(
                    rejected.identity(),
                ));
                EquipmentFairyTransition::ReplacementRejected {
                    removed: removed_event,
                    add,
                    effects,
                }
            }
            EquipmentAddOutcome::Added(added) => {
                let added_goods = self
                    .equipment
                    .get(&added.column)
                    .expect("successful Add публикует replacement в map");
                effects.push(EquipmentFairyTransitionEffect::ObjectMove(
                    EquipmentFairyObjectMove {
                        operation: EquipmentFairyMoveOperation::NewObject,
                        owner_type: added.owner_type,
                        owner_id: added.owner_id,
                        column: added.column,
                        container_extend_id: EQUIPMENT_CONTAINER_EXTEND_ID,
                        goods: added_goods.identity(),
                        amount: added_goods.amount(),
                        old_client_payload: encode_old_client(added_goods),
                    },
                ));
                EquipmentFairyTransition::Changed {
                    removed: removed_event,
                    added,
                    effects,
                }
            }
        }
    }

    /// Exact positional `Add` сохраняет необычный порядок: BF/mount/occupied/
    /// catalog проверки, затем timed mutation и AI-registration, и только
    /// после них проверку соответствия equip-place. Поэтому late block
    /// возвращает уже применённые partial effects и не забирает `incoming`.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        runtime: EquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
    ) -> EquipmentAddOutcome {
        let Some(goods) = incoming.as_mut() else {
            return Self::blocked_add(EquipmentAddBlock::MissingGoods, Default::default());
        };
        if goods.addon_property_value(factory, GAP_BF_BFEQUIPEMENT, 1) == 1 {
            return Self::blocked_add(EquipmentAddBlock::BattleFairyEquipment, Default::default());
        }
        if self.base.owner_type() == PLAYER_OWNER_TYPE
            && let Some(player) = runtime.owner_player
            && player.can_mount_result != 9
        {
            return Self::blocked_add(
                EquipmentAddBlock::MountRejected {
                    result: player.can_mount_result,
                },
                Default::default(),
            );
        }

        let column = EquipmentColumn::from_position(position);
        if let Some(column) = column
            && self.equipment.contains_key(&column)
        {
            return Self::blocked_add(EquipmentAddBlock::Occupied { column }, Default::default());
        }

        let base_properties_index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(base_properties_index) else {
            return Self::blocked_add(
                EquipmentAddBlock::MissingBaseProperties {
                    index: base_properties_index,
                },
                Default::default(),
            );
        };
        if properties.goods_type() != GOODS_TYPE_EQUIPMENT {
            return Self::blocked_add(
                EquipmentAddBlock::NotEquipment {
                    goods_type: properties.goods_type(),
                },
                Default::default(),
            );
        }

        let mut partial_effects = EquipmentAddPartialEffects::default();
        partial_effects.start_point_initialized =
            goods.initialize_equipment_start_point(factory, runtime.now);
        if runtime.owner_player.is_some() {
            register_with_goods_ai(goods);
            partial_effects.registered_with_goods_ai = true;
        }

        let Some(column) = column else {
            return Self::blocked_add(
                EquipmentAddBlock::InvalidPosition { position },
                partial_effects,
            );
        };
        let equip_place = properties.equip_place();
        if !Self::does_equip_place_fit(column, equip_place) {
            return Self::blocked_add(
                EquipmentAddBlock::EquipPlaceMismatch {
                    column,
                    equip_place,
                },
                partial_effects,
            );
        }

        let goods = incoming
            .take()
            .expect("incoming проверен и не изменяется до commit");
        let identity = goods.identity();
        let amount = goods.amount();
        let owner_type = self.base.owner_type();
        let owner_id = self.base.owner_id();
        let player_effects = (owner_type == PLAYER_OWNER_TYPE && runtime.owner_player.is_some())
            .then(|| EquipmentPlayerAddedEffects {
                add_war_soul_skill: column == EquipmentColumn::Headgear
                    && goods.has_addon_property_values(factory, GAP_BF_LEVEL),
                recompute_properties: true,
                around_update: EquipmentAroundUpdate {
                    owner_id,
                    column,
                    added: true,
                    base_properties_index,
                    weapon_level: goods.addon_property_value(factory, GAP_WEAPON_LEVEL, 1) as u32,
                    exclude_owner: true,
                },
            });
        let package_extension_applied = player_effects.is_some()
            && runtime.pack_add_enabled
            && goods.query_attribute(GAP_GOODS_PACKAGE_EXTENTION)
            && goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) == 2;
        let package_extension_delta = if package_extension_applied {
            goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2) as u32
        } else {
            0
        };

        let replaced = self.equipment.insert(column, goods);
        debug_assert!(replaced.is_none());
        self.expanded_package_num = self
            .expanded_package_num
            .wrapping_add(package_extension_delta);

        EquipmentAddOutcome::Added(EquipmentAddedReport {
            owner_type,
            owner_id,
            column,
            identity,
            base_properties_index,
            amount,
            partial_effects,
            player_effects,
            package_extension_applied,
            package_extension_delta,
            listeners: self.base.base().listener_snapshot(),
        })
    }

    /// Auto-add выбирает фиксированную колонку до вызова positional owner-а;
    /// для ornaments проверяет сначала 6, затем 7 и при обеих занятых не
    /// запускает BF/mount/timed/AI ветви.
    pub(crate) fn add_preferred(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        runtime: EquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
    ) -> EquipmentAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return Self::blocked_add(EquipmentAddBlock::MissingGoods, Default::default());
        };
        let base_properties_index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(base_properties_index) else {
            return Self::blocked_add(
                EquipmentAddBlock::MissingBaseProperties {
                    index: base_properties_index,
                },
                Default::default(),
            );
        };
        if properties.goods_type() != GOODS_TYPE_EQUIPMENT {
            return Self::blocked_add(
                EquipmentAddBlock::NotEquipment {
                    goods_type: properties.goods_type(),
                },
                Default::default(),
            );
        }
        let equip_place = properties.equip_place();
        let Some(column) = self.preferred_column(equip_place) else {
            let reason = if equip_place == EQUIP_PLACE_ORNAMENTS {
                EquipmentAddBlock::NoFreeOrnamentSlot
            } else {
                EquipmentAddBlock::UnsupportedEquipPlace { equip_place }
            };
            return Self::blocked_add(reason, Default::default());
        };
        self.add_at(
            column.position(),
            incoming,
            factory,
            runtime,
            register_with_goods_ai,
        )
    }

    fn blocked_add(
        reason: EquipmentAddBlock,
        partial_effects: EquipmentAddPartialEffects,
    ) -> EquipmentAddOutcome {
        EquipmentAddOutcome::Blocked(EquipmentAddBlockedReport {
            reason,
            partial_effects,
        })
    }

    fn preferred_column(&self, equip_place: i32) -> Option<EquipmentColumn> {
        Some(match equip_place {
            EQUIP_PLACE_HEAD => EquipmentColumn::Head,
            EQUIP_PLACE_BODY => EquipmentColumn::Body,
            EQUIP_PLACE_HAND => EquipmentColumn::Hand,
            EQUIP_PLACE_GLOVE => EquipmentColumn::Glove,
            EQUIP_PLACE_BOOT => EquipmentColumn::Boot,
            EQUIP_PLACE_ORNAMENTS => {
                if self.is_slot_empty(EquipmentColumn::OrnamentsOne) {
                    EquipmentColumn::OrnamentsOne
                } else if self.is_slot_empty(EquipmentColumn::OrnamentsTwo) {
                    EquipmentColumn::OrnamentsTwo
                } else {
                    return None;
                }
            }
            EQUIP_PLACE_MEDAL => EquipmentColumn::Medal,
            EQUIP_PLACE_POSTERIOR => EquipmentColumn::Posterior,
            EQUIP_PLACE_JEWELRY => EquipmentColumn::Jewelry,
            EQUIP_PLACE_HEADGEAR => EquipmentColumn::Headgear,
            EQUIP_PLACE_TALISMAN => EquipmentColumn::Talisman,
            EQUIP_PLACE_FROCK => EquipmentColumn::Frock,
            EQUIP_PLACE_WING => EquipmentColumn::Wing,
            EQUIP_PLACE_MANTEAU => EquipmentColumn::Manteau,
            EQUIP_PLACE_FAIRY => EquipmentColumn::Fairy,
            EQUIP_PLACE_LING_BAO => EquipmentColumn::LingBao,
            _ => return None,
        })
    }

    pub(crate) fn does_equip_place_fit(column: EquipmentColumn, equip_place: i32) -> bool {
        match equip_place {
            EQUIP_PLACE_HEAD => column == EquipmentColumn::Head,
            EQUIP_PLACE_BODY => column == EquipmentColumn::Body,
            EQUIP_PLACE_HAND => column == EquipmentColumn::Hand,
            EQUIP_PLACE_GLOVE => column == EquipmentColumn::Glove,
            EQUIP_PLACE_BOOT => column == EquipmentColumn::Boot,
            EQUIP_PLACE_ORNAMENTS => matches!(
                column,
                EquipmentColumn::OrnamentsOne | EquipmentColumn::OrnamentsTwo
            ),
            EQUIP_PLACE_MEDAL => column == EquipmentColumn::Medal,
            EQUIP_PLACE_POSTERIOR => column == EquipmentColumn::Posterior,
            EQUIP_PLACE_JEWELRY => column == EquipmentColumn::Jewelry,
            EQUIP_PLACE_HEADGEAR => column == EquipmentColumn::Headgear,
            EQUIP_PLACE_TALISMAN => column == EquipmentColumn::Talisman,
            EQUIP_PLACE_FROCK => column == EquipmentColumn::Frock,
            EQUIP_PLACE_WING => column == EquipmentColumn::Wing,
            EQUIP_PLACE_MANTEAU => column == EquipmentColumn::Manteau,
            EQUIP_PLACE_FAIRY => column == EquipmentColumn::Fairy,
            EQUIP_PLACE_LING_BAO => column == EquipmentColumn::LingBao,
            _ => false,
        }
    }

    /// Exact `Remove(CGUID)` сначала применяет player-wide package-extension
    /// decrement, затем вычисляет позицию и только после этого может отказать
    /// headgear из-за активной war-soul skill. Поэтому даже missing/blocked
    /// исходы несут уже применённый partial effect.
    pub(crate) fn remove(
        &mut self,
        goods_id: CGuid,
        factory: &CGoodsFactory,
        runtime: EquipmentRemoveRuntimeFacts,
    ) -> EquipmentRemoveOutcome {
        let mut partial_effects = EquipmentRemovePartialEffects::default();
        if runtime.pack_add_enabled
            && runtime.owner_player_present
            && let Some(extension) = runtime.player_goods_package_extension
        {
            self.expanded_package_num = self.expanded_package_num.wrapping_sub(extension);
            partial_effects.package_extension_subtracted = extension;
        }

        let Some(column) = self.query_goods_position_by_id(goods_id) else {
            return EquipmentRemoveOutcome::Missing { partial_effects };
        };
        if column == EquipmentColumn::Headgear && runtime.active_war_soul_blocks_headgear {
            return EquipmentRemoveOutcome::BlockedByActiveWarSoul {
                column,
                partial_effects,
            };
        }

        let goods = self
            .equipment
            .remove(&column)
            .expect("column разрешена непосредственно перед erase");
        let owner_type = self.base.owner_type();
        let owner_id = self.base.owner_id();
        let player_effects = (owner_type == PLAYER_OWNER_TYPE && runtime.owner_player_present)
            .then(|| {
                let changes_war_soul = column == EquipmentColumn::Headgear
                    && goods.has_addon_property_values(factory, GAP_BF_LEVEL);
                EquipmentPlayerRemovedEffects {
                    clear_war_soul_status: changes_war_soul,
                    delete_war_soul_skill: changes_war_soul,
                    recompute_without_removed_slot: true,
                    clamp_hp_and_mp: true,
                    around_update: EquipmentAroundUpdate {
                        owner_id,
                        column,
                        added: false,
                        base_properties_index: goods.base_properties_index(),
                        weapon_level: goods.addon_property_value(factory, GAP_WEAPON_LEVEL, 1)
                            as u32,
                        exclude_owner: false,
                    },
                }
            });
        EquipmentRemoveOutcome::Removed(EquipmentRemovedReport {
            event: EquipmentRemovedEvent {
                owner_type,
                owner_id,
                column,
                partial_effects,
                player_effects,
                listeners: self.base.base().listener_snapshot(),
            },
            goods,
        })
    }

    /// Exact `Swap` использует public `Remove`/`Add`, поэтому их callbacks,
    /// wrapping package-counter и late timed/AI effects наблюдаемы также на
    /// неуспехе. Старый товар garbage-collect-ится только если rollback Add
    /// тоже отказал.
    pub(crate) fn swap(
        &mut self,
        column: EquipmentColumn,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        runtime: EquipmentSwapRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
    ) -> EquipmentSwapOutcome {
        let Some(incoming_goods) = incoming.as_ref() else {
            return EquipmentSwapOutcome::Blocked(EquipmentSwapBlock::MissingIncoming);
        };
        let base_properties_index = incoming_goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(base_properties_index) else {
            return EquipmentSwapOutcome::Blocked(EquipmentSwapBlock::MissingBaseProperties {
                index: base_properties_index,
            });
        };
        if properties.goods_type() != GOODS_TYPE_EQUIPMENT {
            return EquipmentSwapOutcome::Blocked(EquipmentSwapBlock::NotEquipment {
                goods_type: properties.goods_type(),
            });
        }
        let equip_place = properties.equip_place();
        if !Self::does_equip_place_fit(column, equip_place) {
            return EquipmentSwapOutcome::Blocked(EquipmentSwapBlock::EquipPlaceMismatch {
                column,
                equip_place,
            });
        }

        let Some(old_goods) = self.equipment.get(&column) else {
            return EquipmentSwapOutcome::Blocked(EquipmentSwapBlock::Empty { column });
        };
        if runtime.pack_add_enabled
            && let Some(available) = runtime.active_unused_package_slots
            && old_goods.query_attribute(GAP_GOODS_PACKAGE_EXTENTION)
            && old_goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) == 2
        {
            let required =
                old_goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2) as u32;
            if available < required {
                return EquipmentSwapOutcome::Blocked(
                    EquipmentSwapBlock::InsufficientPackageCapacity {
                        required,
                        available,
                    },
                );
            }
        }

        let old_goods_id = old_goods.identity().ex_id;
        let removed_report = match self.remove(old_goods_id, factory, runtime.remove) {
            EquipmentRemoveOutcome::Removed(report) => report,
            failed => return EquipmentSwapOutcome::RemovalFailed(failed),
        };
        let EquipmentRemovedReport {
            event: removed,
            goods: old_goods,
        } = removed_report;

        match self.add_at(
            column.position(),
            incoming,
            factory,
            runtime.incoming_add,
            register_with_goods_ai,
        ) {
            EquipmentAddOutcome::Added(added) => EquipmentSwapOutcome::Swapped {
                outgoing: old_goods,
                removed,
                added,
            },
            EquipmentAddOutcome::Blocked(incoming_block) => {
                let mut rollback_goods = Some(old_goods);
                match self.add_at(
                    column.position(),
                    &mut rollback_goods,
                    factory,
                    runtime.rollback_add,
                    register_with_goods_ai,
                ) {
                    EquipmentAddOutcome::Added(restored) => {
                        EquipmentSwapOutcome::IncomingRejectedAndRestored {
                            removed,
                            incoming: incoming_block,
                            restored,
                        }
                    }
                    EquipmentAddOutcome::Blocked(rollback) => {
                        let garbage_collected = rollback_goods
                            .take()
                            .expect("failed rollback Add сохраняет old goods")
                            .identity();
                        EquipmentSwapOutcome::IncomingRejectedAndOldGoodsCollected {
                            removed,
                            incoming: incoming_block,
                            rollback,
                            garbage_collected,
                        }
                    }
                }
            }
        }
    }

    /// Outer wire-owner `Serialize`: count включает только товары с живой
    /// catalog-записью, затем идут signed i32 column и payload `CGoods`.
    pub(crate) fn serialize_with<Encode>(
        &self,
        destination: &mut Vec<u8>,
        factory: &CGoodsFactory,
        include_ex_data: bool,
        mut encode_goods: Encode,
    ) where
        Encode: FnMut(&CGoods, bool, &mut Vec<u8>),
    {
        LegacyWriter::new(destination).write_i32(self.goods_amount(factory) as i32);
        for (column, goods) in &self.equipment {
            if factory
                .query_goods_base_properties(goods.base_properties_index())
                .is_none()
            {
                continue;
            }
            LegacyWriter::new(destination).write_i32(column.position() as i32);
            encode_goods(goods, include_ex_data, destination);
        }
    }

    /// Outer `Unserialize` очищает container до чтения count, продолжает после
    /// null decoder-а или rejected Add и добавляет отдельный positive-only
    /// `bToAdd` package delta уже после успешного Add.
    pub(crate) fn unserialize_with<Decode, Runtime>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        to_add_enabled: bool,
        mut decode_goods: Decode,
        mut runtime_for_goods: Runtime,
        register_with_goods_ai: &mut dyn FnMut(&mut CGoods),
        on_cleared: &mut dyn FnMut(EquipmentColumn, &CGoods, &[ContainerListenerHandle]),
    ) -> Result<EquipmentUnserializeReport, EquipmentUnserializeFailure>
    where
        Decode: FnMut(
            &[u8],
            &mut usize,
        ) -> Result<Option<CGoods>, EquipmentContainerCodecError>,
        Runtime: FnMut(&CGoods) -> EquipmentAddRuntimeFacts,
    {
        let mut report = EquipmentUnserializeReport {
            cleared: self.clear_observing(on_cleared),
            entries: Vec::new(),
        };
        let count = match Self::read_codec_i32(source, cursor, "goods count") {
            Ok(count) => count,
            Err(error) => return Err(EquipmentUnserializeFailure { error, report }),
        };
        if count < 0 {
            return Err(EquipmentUnserializeFailure {
                error: EquipmentContainerCodecError::NegativeGoodsCount { count },
                report,
            });
        }

        for _ in 0..count as u32 {
            let position = match Self::read_codec_i32(source, cursor, "goods position") {
                Ok(position) => position as u32,
                Err(error) => return Err(EquipmentUnserializeFailure { error, report }),
            };
            let Some(goods) = (match decode_goods(source, cursor) {
                Ok(goods) => goods,
                Err(error) => return Err(EquipmentUnserializeFailure { error, report }),
            }) else {
                report
                    .entries
                    .push(EquipmentUnserializedEntry::DecoderReturnedNull { position });
                continue;
            };
            let runtime = runtime_for_goods(&goods);
            let mut incoming = Some(goods);
            match self.add_at(
                position,
                &mut incoming,
                factory,
                runtime,
                register_with_goods_ai,
            ) {
                EquipmentAddOutcome::Blocked(add) => {
                    report.entries.push(EquipmentUnserializedEntry::Rejected {
                        position,
                        goods: incoming
                            .take()
                            .expect("rejected equipment Add сохраняет decoded goods"),
                        add,
                    });
                }
                EquipmentAddOutcome::Added(add) => {
                    let to_add_extension = if to_add_enabled {
                        self.equipment
                            .get(&add.column)
                            .filter(|goods| goods.query_attribute(GAP_GOODS_PACKAGE_EXTENTION))
                            .filter(|goods| {
                                goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1)
                                    == 2
                            })
                            .map(|goods| {
                                goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2)
                            })
                    } else {
                        None
                    };
                    let to_add_extension_delta = to_add_extension
                        .filter(|extension| 0 < *extension)
                        .map_or(0, |extension| extension as u32);
                    self.expanded_package_num = self
                        .expanded_package_num
                        .wrapping_add(to_add_extension_delta);
                    report.entries.push(EquipmentUnserializedEntry::Added {
                        position,
                        add,
                        to_add_extension_delta,
                        write_pack_expand_log: to_add_extension.is_some()
                            && runtime.owner_player.is_some(),
                    });
                }
            }
        }
        Ok(report)
    }

    fn read_codec_i32(
        source: &[u8],
        cursor: &mut usize,
        field: &'static str,
    ) -> Result<i32, EquipmentContainerCodecError> {
        let offset = *cursor;
        let Some(end) = offset.checked_add(4) else {
            return Err(EquipmentContainerCodecError::UnexpectedEnd {
                field,
                offset,
                available: source.len().saturating_sub(offset),
            });
        };
        let mut reader = LegacyReader::at(source, offset).map_err(|_| {
            EquipmentContainerCodecError::UnexpectedEnd {
                field,
                offset,
                available: source.len().saturating_sub(offset),
            }
        })?;
        match reader.read_i32() {
            Ok(value) => {
                *cursor = reader.position();
                Ok(value)
            }
            Err(_) => {
                // Исторический владелец сначала продвигал указатель, затем обнаруживал обрыв.
                *cursor = end;
                Err(EquipmentContainerCodecError::UnexpectedEnd {
                    field,
                    offset,
                    available: source.len().saturating_sub(offset),
                })
            }
        }
    }

    /// Internal self-callback должен быть применён dispatcher-ом перед
    /// перечисленными external listeners для каждого report-а.
    pub(crate) fn clear(&mut self) -> Vec<EquipmentClearedGoods> {
        self.clear_observing(&mut |_, _, _| {})
    }

    /// Observer вызывается пока текущий goods ещё доступен в своей колонке;
    /// после возврата ownership переносится в ordered report.
    fn clear_observing(
        &mut self,
        observer: &mut dyn FnMut(EquipmentColumn, &CGoods, &[ContainerListenerHandle]),
    ) -> Vec<EquipmentClearedGoods> {
        let listeners = self.base.base().listener_snapshot();
        let columns: Vec<_> = self.equipment.keys().copied().collect();
        let mut reports = Vec::with_capacity(columns.len());
        for column in columns {
            observer(
                column,
                self.equipment
                    .get(&column)
                    .expect("clear column взята из текущей map"),
                &listeners,
            );
            let goods = self
                .equipment
                .remove(&column)
                .expect("clear observer не меняет owning map");
            reports.push(EquipmentClearedGoods {
                column,
                goods,
                listeners: listeners.clone(),
            });
        }
        reports
    }

}
