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
//! timed/AI/package effects и rollback loss. Конкретные player message/skill
//! callbacks, fairy/battle-fairy и codec ниже остаются RAW до materialization
//! связанных owners; достигнутое ядро не выдаётся за весь контейнер.

use std::collections::BTreeMap;

use super::ccontainer::ContainerListenerHandle;
use super::cgoodscontainer::{CGoodsContainer, GoodsContainerMode};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_BODY, EQUIP_PLACE_BOOT, EQUIP_PLACE_FAIRY, EQUIP_PLACE_FROCK, EQUIP_PLACE_GLOVE,
    EQUIP_PLACE_HAND, EQUIP_PLACE_HEAD, EQUIP_PLACE_HEADGEAR, EQUIP_PLACE_JEWELRY,
    EQUIP_PLACE_LING_BAO, EQUIP_PLACE_MANTEAU, EQUIP_PLACE_MEDAL, EQUIP_PLACE_ORNAMENTS,
    EQUIP_PLACE_POSTERIOR, EQUIP_PLACE_TALISMAN, EQUIP_PLACE_WING, GAP_BF_BFEQUIPEMENT,
    GAP_GOODS_PACKAGE_EXTENTION, GOODS_TYPE_EQUIPMENT,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;

pub(crate) const EQUIPMENT_COLUMN_LIMIT: u32 = 17;
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

#[must_use = "в test-mode detached товары должны получить нового владельца"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentReleaseReport {
    pub(crate) garbage_collected: Vec<ShapeIdentity>,
    pub(crate) detached: Vec<(EquipmentColumn, CGoods)>,
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
    pub(crate) requires_player_callback: bool,
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
    pub(crate) requires_player_callback: bool,
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
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
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
        let requires_player_callback =
            owner_type == PLAYER_OWNER_TYPE && runtime.owner_player.is_some();
        let package_extension_delta = if requires_player_callback
            && runtime.pack_add_enabled
            && goods.query_attribute(GAP_GOODS_PACKAGE_EXTENTION)
            && goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) == 2
        {
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
            requires_player_callback,
            package_extension_delta,
            listeners: self.base.base().listeners().to_vec(),
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
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
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

    fn does_equip_place_fit(column: EquipmentColumn, equip_place: i32) -> bool {
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
        EquipmentRemoveOutcome::Removed(EquipmentRemovedReport {
            goods,
            event: EquipmentRemovedEvent {
                owner_type: self.base.owner_type(),
                owner_id: self.base.owner_id(),
                column,
                partial_effects,
                requires_player_callback: self.base.owner_type() == PLAYER_OWNER_TYPE
                    && runtime.owner_player_present,
                listeners: self.base.base().listeners().to_vec(),
            },
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
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
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
        let removed_report = match self.remove(old_goods_id, runtime.remove) {
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

    /// Internal self-callback должен быть применён dispatcher-ом перед
    /// перечисленными external listeners для каждого report-а.
    pub(crate) fn clear(&mut self) -> Vec<EquipmentClearedGoods> {
        let listeners = self.base.base().listeners().to_vec();
        std::mem::take(&mut self.equipment)
            .into_iter()
            .map(|(column, goods)| EquipmentClearedGoods {
                column,
                goods,
                listeners: listeners.clone(),
            })
            .collect()
    }

    pub(crate) fn release(&mut self) -> EquipmentReleaseReport {
        let mode = self.base.container_mode();
        let equipment = std::mem::take(&mut self.equipment);
        let mut report = EquipmentReleaseReport::default();
        match mode {
            GoodsContainerMode::Normal => {
                report.garbage_collected = equipment
                    .into_values()
                    .map(|goods| goods.identity())
                    .collect();
            }
            GoodsContainerMode::Test => {
                report.detached = equipment.into_iter().collect();
            }
        }
        self.base.release();
        report
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp

// ============================================================================
// FUNCTION: CEquipmentContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:46
// RVA: 0x000ED180
// ADDRESS: 004ed180
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:127
// RVA: 0x000ED480
// ADDRESS: 004ed480
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:153
// RVA: 0x000ED5A0
// ADDRESS: 004ed5a0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::TraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:171
// RVA: 0x000ED680
// ADDRESS: 004ed680
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetContentsWeight
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:191
// RVA: 0x000ED700
// ADDRESS: 004ed700
// PROTOTYPE: ulong __thiscall GetContentsWeight(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:208
// RVA: 0x000ED770
// ADDRESS: 004ed770
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:396
// RVA: 0x000ED800
// ADDRESS: 004ed800
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:699
// RVA: 0x000ED890
// ADDRESS: 004ed890
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::IsFull
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:714
// RVA: 0x000ED900
// ADDRESS: 004ed900
// PROTOTYPE: int __thiscall IsFull(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:736
// RVA: 0x000ED970
// ADDRESS: 004ed970
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:760
// RVA: 0x000ED9D0
// ADDRESS: 004ed9d0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:781
// RVA: 0x000EDA60
// ADDRESS: 004eda60
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:799
// RVA: 0x000EDAF0
// ADDRESS: 004edaf0
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoodsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:963
// RVA: 0x000EDB30
// ADDRESS: 004edb30
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1003
// RVA: 0x000EDBB0
// ADDRESS: 004edbb0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::~CEquipmentContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:39
// RVA: 0x000EE410
// ADDRESS: 004ee410
// PROTOTYPE: void __thiscall ~CEquipmentContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::FairyExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1075
// RVA: 0x000EE4A0
// ADDRESS: 004ee4a0
// PROTOTYPE: EExpUpResult __thiscall FairyExpUp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::BFLevelUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1149
// RVA: 0x000EE860
// ADDRESS: 004ee860
// PROTOTYPE: eExpUpResult __thiscall BFLevelUp(int param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::CEquipmentContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:25
// RVA: 0x000EEBF0
// ADDRESS: 004eebf0
// PROTOTYPE: undefined __thiscall CEquipmentContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Swap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:227
// RVA: 0x000EECB0
// ADDRESS: 004eecb0
// PROTOTYPE: int __thiscall Swap(EQUIPMENT_COLUMN param_1, CGoods * param_2, CGoods * * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:418
// RVA: 0x000EEF90
// ADDRESS: 004eef90
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:492
// RVA: 0x000EF2F0
// ADDRESS: 004ef2f0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:813
// RVA: 0x000EF5B0
// ADDRESS: 004ef5b0
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:829
// RVA: 0x000EF6C0
// ADDRESS: 004ef6c0
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:894
// RVA: 0x000EF9A0
// ADDRESS: 004ef9a0
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1024
// RVA: 0x000EFC30
// ADDRESS: 004efc30
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
