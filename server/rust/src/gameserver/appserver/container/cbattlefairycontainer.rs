//! Позиционный storage-prefix `CBattleFairyContainer` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cbattlefairycontainer.cpp`.
//! Материализованы 17 фиксированных ячеек и exact positional add-фильтры по
//! goods type/addon marker. Gear-слоты публикуют ранний `BFPropertyAdd(+1)`
//! effect до base Add, поэтому отказ storage не отменяет этот effect. Проверка
//! combine сверяет original-name material/fetch stone/fetch body в порядке
//! записи compose, публикуя legacy packet `0xbf92c`; его `fistp` использует
//! truncation к нулю. Gem success/fail/probability и upgrade-price queries
//! сохраняют positional RNG, signed clamp и неинициализированный cached price
//! как `Option`. Полный combine теперь разделяет check-only `0xbf92c` и
//! execution opcode `0x8fc27`: global gate, валидация goods-presence ×3 затем
//! base-properties ×3 с машинными notification `(56/57/58, затем 56/57/60)`,
//! молчаливый execution no-match (tail 0x503F81), порядок remove
//! `body → stone → material`, RNG-result и ownership
//! созданного товара выполняются в `CPlayer`; этот owner даёт recipe,
//! positional storage и `LoadBFDefualtProperty` callback в том же порядке.
//! Gear add/remove теперь замкнуты через `CPlayer`: ранний `BFPropertyAdd`
//! остаётся partial effect даже при отказе base Add, а Remove применяет `-1`
//! только после успешного отделения goods. Сам контейнер публикует typed
//! storage/result state; GlobeSetup-формулы и client update принадлежат
//! player/game owner-ам.
//! Upgrade target/gems также проходят через player/game owner: container
//! сохраняет positional queries, stack decrement и полный remove ownership.
//! Ошибочный повторный native pointer guard в optional-gem tail заменён
//! независимой безопасной обработкой ячеек `13..=16`; порядок не меняется.
//! `ResetSkill` перенесён в `CPlayer/CGame`, где одновременно доступны
//! equipment, packet, skill storage, общий RNG и ordered network effects;
//! восемь constructor-owned incompatible pairs подтверждены EXE immediate-ами.
//!
//! Автоматический overload читает неинициализированный `m_eBFEquipPlace` у
//! catalog owner-а. Rust выражает этот UB как typed block, а не выбирает
//! логичную ячейку из позднего C++-донора. Остальные ещё не подключённые
//! player-integrated методы ещё требуют реконструкции; полный декомпилят хранится локально.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsCleared, AmountLimitGoodsRelease, AmountLimitGoodsTaken,
};
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome, VolumeGoodsCodecError,
    VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::goods::cbattlefairyproperty::BattleFairyCompose;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_BATTLE_FAIRY, GAP_BF_BFEQUIPEMENT, GAP_BF_CLOTH, GAP_BF_DEFUALT_SKLL, GAP_BF_EARTH,
    GAP_BF_FETCH_BODY, GAP_BF_FETCH_STONE, GAP_BF_GEM, GAP_BF_GLOVE, GAP_BF_HP,
    GAP_BF_HUOXIESHU_SKILL, GAP_BF_HUXINJING, GAP_BF_JEWELLERY, GAP_BF_LINGZHISHU_SKILL,
    GAP_BF_MAN, GAP_BF_MATERIAL, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_PIFENG,
    GAP_BF_SKY, GAP_BF_SPRITUALISM_BASE, GAP_BF_STRENGH_BASE, GAP_BF_WEAPON, GAP_BF_XIEZI,
    GAP_BF_YAODAI, GAP_GEM_PROBABILITY, GAP_GEM_TYPE, GAP_GEM_UPGRADE_FAILED_RESULT,
    GAP_GEM_UPGRADE_SUCCEED_RESULT, GAP_GOODS_UPGRADE_PRICE, GOODS_TYPE_CONSUMABLE,
    GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCell {
    Weapon = 0,
    Body = 1,
    Huxinjing = 2,
    Jewelry = 3,
    Glove = 4,
    Pifeng = 5,
    Yaodai = 6,
    Xiezi = 7,
    Material = 8,
    FetchStone = 9,
    FetchBody = 10,
    Battle = 11,
    Equipment = 12,
    GemBase = 13,
    GemOne = 14,
    GemTwo = 15,
    GemThree = 16,
}

impl BattleFairyCell {
    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        Some(match position {
            0 => Self::Weapon,
            1 => Self::Body,
            2 => Self::Huxinjing,
            3 => Self::Jewelry,
            4 => Self::Glove,
            5 => Self::Pifeng,
            6 => Self::Yaodai,
            7 => Self::Xiezi,
            8 => Self::Material,
            9 => Self::FetchStone,
            10 => Self::FetchBody,
            11 => Self::Battle,
            12 => Self::Equipment,
            13 => Self::GemBase,
            14 => Self::GemOne,
            15 => Self::GemTwo,
            16 => Self::GemThree,
            _ => return None,
        })
    }

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPropertyAddEffect {
    pub(crate) cell: BattleFairyCell,
    pub(crate) delta: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeConsumedGem {
    pub(crate) cell: BattleFairyCell,
    pub(crate) goods: ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removed: bool,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

pub(crate) const BATTLE_FAIRY_COMBINE_MESSAGE_TYPE: u32 = 0x0b_f92c;
const SKILL_HUOXIESHU: u32 = 546;
const SKILL_LINGZHISHU: u32 = 547;
const SKILL_BATTLEFAIRY_BASE_ATTACK: u32 = 548;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineResult {
    #[default]
    None = 0,
    CanCombine = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineNotification {
    MissingMaterial,
    MissingFetchStone,
    MissingFetchBody,
    CannotSummon,
}

impl BattleFairyCombineNotification {
    pub(crate) const fn string_id(self) -> &'static str {
        match self {
            Self::MissingMaterial => "ZHGS0056",
            Self::MissingFetchStone => "ZHGS0057",
            Self::MissingFetchBody => "ZHGS0058",
            Self::CannotSummon => "ZHGS0059",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineAvailability {
    pub(crate) message_type: u32,
    pub(crate) deplete_fetch: u32,
    pub(crate) truncated_success_rate: u32,
}

#[must_use = "report содержит адресованные игроку notification или availability effect"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineCheck {
    pub(crate) result: BattleFairyCombineResult,
    pub(crate) player_id: Option<i32>,
    pub(crate) notification: Option<BattleFairyCombineNotification>,
    pub(crate) availability: Option<BattleFairyCombineAvailability>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDefaultSkill {
    pub(crate) id: u32,
    pub(crate) level: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDefaultGoodsUpdate {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineExecutionNotification {
    MissingMaterial,
    MissingFetchStone,
    MissingFetchBody,
    CannotSummon,
}

impl BattleFairyCombineExecutionNotification {
    pub(crate) const fn string_id(self) -> &'static str {
        match self {
            Self::MissingMaterial => "ZHGS0056",
            Self::MissingFetchStone => "ZHGS0057",
            Self::MissingFetchBody => "ZHGS0058",
            Self::CannotSummon => "ZHGS0060",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineRemovedInput {
    pub(crate) cell: BattleFairyCell,
    pub(crate) goods: ShapeIdentity,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyContainerAddBlock {
    MissingGoods,
    MissingBaseProperties { index: u32 },
    AutomaticAddRequiresEquipment { goods_type: i32 },
    UninitializedBattleFairyEquipPlace,
    InvalidPosition { position: u32 },
    GoodsRejected { cell: BattleFairyCell },
}

#[must_use = "outcome сохраняет ранний BFPropertyAdd effect и ownership incoming"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyContainerAddOutcome {
    Stored {
        base: VolumeGoodsAddOutcome,
        property_effect: Option<BattleFairyPropertyAddEffect>,
    },
    Rejected(BattleFairyContainerAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBattleFairyContainer {
    base: CVolumeLimitGoodsContainer,
    upgrade_price: Option<u32>,
}

impl Default for CBattleFairyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CBattleFairyContainer {
    pub(crate) fn new() -> Self {
        Self {
            base: CVolumeLimitGoodsContainer::new(),
            upgrade_price: None,
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

    pub(crate) fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
        self.base.serialize(destination, factory)
    }

    pub(crate) fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<(), VolumeGoodsCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let _cleared = self.clear();
        self.base.unserialize(
            source,
            cursor,
            factory,
            ordinary_threshold,
            battle_threshold,
        )
    }

    pub(crate) fn add(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingGoods,
            );
        };
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingBaseProperties { index },
            );
        };
        if properties.goods_type() != GOODS_TYPE_EQUIPMENT {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::AutomaticAddRequiresEquipment {
                    goods_type: properties.goods_type(),
                },
            );
        }
        let Some(raw_place) = properties.battle_fairy_equip_place() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::UninitializedBattleFairyEquipPlace,
            );
        };
        let Some(cell) = BattleFairyCell::from_position(raw_place as u32) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::InvalidPosition {
                    position: raw_place as u32,
                },
            );
        };
        self.add_at(cell, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_at(
        &mut self,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingGoods,
            );
        };
        let applies_property = match self.validate_add_at(cell, goods, factory) {
            Ok(applies_property) => applies_property,
            Err(block) => return BattleFairyContainerAddOutcome::Rejected(block),
        };
        let property_effect =
            applies_property.then_some(BattleFairyPropertyAddEffect { cell, delta: 1 });
        BattleFairyContainerAddOutcome::Stored {
            base: self
                .base
                .add_goods_at(cell.position(), incoming, factory, owner_progress_allows),
            property_effect,
        }
    }

    /// Позволяет player owner-у исполнить подтверждённый ранний
    /// `BFPropertyAdd(+1)` до base storage mutation, сохраняя единый validator
    /// positional add и его exact rejection mapping.
    pub(crate) fn property_effect_before_add(
        &self,
        cell: BattleFairyCell,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> Option<BattleFairyPropertyAddEffect> {
        self.validate_add_at(cell, goods, factory)
            .ok()
            .filter(|applies_property| *applies_property)
            .map(|_| BattleFairyPropertyAddEffect { cell, delta: 1 })
    }

    fn validate_add_at(
        &self,
        cell: BattleFairyCell,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> Result<bool, BattleFairyContainerAddBlock> {
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return Err(BattleFairyContainerAddBlock::MissingBaseProperties { index });
        };
        let value =
            |property_type, value_id| goods.addon_property_value(factory, property_type, value_id);
        let (allowed, applies_property) = match properties.goods_type() {
            GOODS_TYPE_EQUIPMENT => match cell {
                BattleFairyCell::Battle => (value(GAP_BF_BATTLE_FAIRY, 1) == 1, false),
                BattleFairyCell::Weapon => (value(GAP_BF_WEAPON, 1) == 1, true),
                BattleFairyCell::Huxinjing => (value(GAP_BF_HUXINJING, 1) == 1, true),
                BattleFairyCell::Body => (value(GAP_BF_CLOTH, 1) == 1, true),
                BattleFairyCell::Jewelry => (value(GAP_BF_JEWELLERY, 1) == 1, true),
                BattleFairyCell::Pifeng => (value(GAP_BF_PIFENG, 1) == 1, true),
                BattleFairyCell::Yaodai => (value(GAP_BF_YAODAI, 1) == 1, true),
                BattleFairyCell::Xiezi => (value(GAP_BF_XIEZI, 1) == 1, true),
                BattleFairyCell::Glove => (value(GAP_BF_GLOVE, 1) == 1, true),
                BattleFairyCell::Equipment => (value(GAP_BF_BFEQUIPEMENT, 2) == 1, false),
                _ => (false, false),
            },
            GOODS_TYPE_USELESS => match cell {
                BattleFairyCell::Material => (value(GAP_BF_MATERIAL, 1) == 1, false),
                BattleFairyCell::FetchBody => (value(GAP_BF_FETCH_BODY, 1) == 1, false),
                BattleFairyCell::FetchStone => (value(GAP_BF_FETCH_STONE, 1) == 1, false),
                _ => (false, false),
            },
            GOODS_TYPE_CONSUMABLE if value(GAP_BF_GEM, 1) == 1 => match cell {
                BattleFairyCell::GemBase => (value(GAP_GEM_TYPE, 1) == 1, false),
                BattleFairyCell::GemOne | BattleFairyCell::GemTwo | BattleFairyCell::GemThree => {
                    (value(GAP_GEM_TYPE, 1) == 2, false)
                }
                _ => (false, false),
            },
            _ => (false, false),
        };
        if !allowed {
            return Err(BattleFairyContainerAddBlock::GoodsRejected { cell });
        }
        Ok(applies_property)
    }

    /// Gem-base (13) задаёт начальный result без roll-а; улучшения из `14..16`
    /// рассматриваются только при строго большем result и делают отдельный
    /// `random(100) <= threshold` вызов в positional order.
    pub(crate) fn success_result(
        &self,
        factory: &CGoodsFactory,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> u32 {
        let mut result = self
            .base
            .get_goods(BattleFairyCell::GemBase.position())
            .map_or(0, |goods| {
                goods.addon_property_value(factory, GAP_GEM_UPGRADE_SUCCEED_RESULT, 1) as u32
            });
        for cell in [
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ] {
            let Some(goods) = self.base.get_goods(cell.position()) else {
                continue;
            };
            let candidate =
                goods.addon_property_value(factory, GAP_GEM_UPGRADE_SUCCEED_RESULT, 1) as u32;
            if result < candidate
                && random(100)
                    <= goods.addon_property_value(factory, GAP_GEM_UPGRADE_SUCCEED_RESULT, 2)
            {
                result = candidate;
            }
        }
        result
    }

    pub(crate) fn fail_result(&self, factory: &CGoodsFactory) -> u32 {
        [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ]
        .into_iter()
        .filter_map(|cell| self.base.get_goods(cell.position()))
        .map(|goods| goods.addon_property_value(factory, GAP_GEM_UPGRADE_FAILED_RESULT, 1) as u32)
        .filter(|result| *result != 0)
        .fold(4, u32::min)
    }

    /// Сумма идёт `13,14,15,16,12` с wrapping u32. Отрицательный signed view
    /// обнуляется до clamp-а `100`, как в exact owner-е.
    pub(crate) fn probability(&self, factory: &CGoodsFactory) -> u32 {
        let total = [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
            BattleFairyCell::Equipment,
        ]
        .into_iter()
        .filter_map(|cell| self.base.get_goods(cell.position()))
        .fold(0u32, |total, goods| {
            total.wrapping_add(goods.addon_property_value(factory, GAP_GEM_PROBABILITY, 1) as u32)
        });
        if (total as i32) < 0 {
            0
        } else {
            total.min(100)
        }
    }

    /// При пустой ячейке exact возвращает `0`, но не меняет сохранённое поле.
    pub(crate) fn upgrade_price(&mut self, factory: &CGoodsFactory) -> u32 {
        let Some(goods) = self.base.get_goods(BattleFairyCell::Equipment.position()) else {
            return 0;
        };
        let price = goods.addon_property_value(factory, GAP_GOODS_UPGRADE_PRICE, 1) as u32;
        self.upgrade_price = Some(price);
        price
    }

    pub(crate) const fn stored_upgrade_price(&self) -> Option<u32> {
        self.upgrade_price
    }

    pub(crate) fn delete_upgrade_target(
        &mut self,
    ) -> Option<(ShapeIdentity, VolumeGoodsRemoveOutcome)> {
        let goods = self.base.get_goods(BattleFairyCell::Equipment.position())?;
        let identity = goods.identity();
        let removal = self.base.remove_goods(identity.ex_id)?;
        Some((identity, removal))
    }

    /// Расходует одну единицу gem в exact positional tail upgrade-а. Stack
    /// остаётся тем же object-ом; amount `1` отделяет ownership всей ячейки.
    pub(crate) fn consume_upgrade_gem(
        &mut self,
        cell: BattleFairyCell,
    ) -> Option<BattleFairyUpgradeConsumedGem> {
        if !matches!(
            cell,
            BattleFairyCell::GemBase
                | BattleFairyCell::GemOne
                | BattleFairyCell::GemTwo
                | BattleFairyCell::GemThree
        ) {
            return None;
        }
        let position = cell.position();
        let goods = self.base.get_goods(position)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        if previous_amount < 2 {
            let removal = self.base.remove_goods(identity.ex_id)?;
            return Some(BattleFairyUpgradeConsumedGem {
                cell,
                goods: identity,
                previous_amount,
                remaining_amount: 0,
                removed: true,
                removal: Some(removal),
            });
        }
        self.base
            .get_goods_mut(position)
            .expect("gem identity получен из той же positional ячейки")
            .set_amount(previous_amount.wrapping_sub(1));
        Some(BattleFairyUpgradeConsumedGem {
            cell,
            goods: identity,
            previous_amount,
            remaining_amount: previous_amount.wrapping_sub(1),
            removed: false,
            removal: None,
        })
    }

    /// Exact `LoadBFDefualtProperty` меняет только товар принадлежащего
    /// существующему player-а. Базовые HP/MP копируются в current и maximum,
    /// затем выставляются три стартовых skill ID и три talent ID.
    pub(crate) fn load_default_properties<Register>(
        player_id: Option<i32>,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        register_skill: &mut Register,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyDefaultGoodsUpdate>
    where
        Register: FnMut(BattleFairyDefaultSkill) -> bool,
    {
        let player_id = player_id?;
        let strength = goods.addon_property_value(factory, GAP_BF_STRENGH_BASE, 1);
        let spiritualism = goods.addon_property_value(factory, GAP_BF_SPRITUALISM_BASE, 1);
        for (property_type, value_id, value) in [
            (GAP_BF_HP, 1, strength),
            (GAP_BF_MAX_HP, 1, strength),
            (GAP_BF_MP, 1, spiritualism),
            (GAP_BF_MAX_MP, 1, spiritualism),
        ] {
            let stored = goods.set_addon_property_value_core(property_type, value_id, value);
            tracing::trace!(property_type, value_id, value, stored, "свойство боевой феи инициализировано");
        }

        for (property_type, skill) in [
            (
                GAP_BF_HUOXIESHU_SKILL,
                BattleFairyDefaultSkill {
                    id: SKILL_HUOXIESHU,
                    level: 1,
                },
            ),
            (
                GAP_BF_LINGZHISHU_SKILL,
                BattleFairyDefaultSkill {
                    id: SKILL_LINGZHISHU,
                    level: 1,
                },
            ),
            (
                GAP_BF_DEFUALT_SKLL,
                BattleFairyDefaultSkill {
                    id: SKILL_BATTLEFAIRY_BASE_ATTACK,
                    level: 1,
                },
            ),
        ] {
            let stored =
                goods.set_addon_property_value_core(property_type, 2, skill.id as i32);
            let registered = register_skill(skill);
            tracing::trace!(property_type, skill_id = skill.id, stored, registered, "начальный навык боевой феи обработан");
        }

        for (property_type, value_id, value) in [
            (GAP_BF_SKY, 2, 0x3c0),
            (GAP_BF_EARTH, 2, 0x3c1),
            (GAP_BF_MAN, 2, 0x3c2),
        ] {
            let stored = goods.set_addon_property_value_core(property_type, value_id, value);
            tracing::trace!(property_type, value_id, value, stored, "талант боевой феи инициализирован");
        }
        Some(BattleFairyDefaultGoodsUpdate {
            message_type: crate::gameserver::appserver::goods::cbattlefairyproperty::BATTLE_FAIRY_GOODS_UPDATE_MESSAGE_TYPE,
            player_id,
            goods: goods.identity(),
            old_client_payload: encode_old_client(goods),
        })
    }

    /// Actual `BatllteFairyCombine` (opcode 0x8FC27) сначала проверяет
    /// goods-presence всех трёх ячеек material/fetch-stone/fetch-body
    /// (`ZHGS0056`/`ZHGS0057`/`ZHGS0058`), затем base-properties тех же трёх
    /// в той же последовательности (`ZHGS0056`/`ZHGS0057`/`ZHGS0060`). При
    /// несовпадении рецепта execution-обработчик машинно завершается молча
    /// (tail 0x503F81): `Ok(None)` не публикует notification, который
    /// присущ только check-only opcode.
    pub(crate) fn battle_fairy_combine_recipe(
        &self,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> Result<Option<BattleFairyCompose>, BattleFairyCombineExecutionNotification> {
        let material = self
            .base
            .get_goods(BattleFairyCell::Material.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingMaterial)?;
        let fetch_stone = self
            .base
            .get_goods(BattleFairyCell::FetchStone.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchStone)?;
        let fetch_body = self
            .base
            .get_goods(BattleFairyCell::FetchBody.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchBody)?;
        let material_properties = factory
            .query_goods_base_properties(material.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::MissingMaterial)?;
        let fetch_stone_properties = factory
            .query_goods_base_properties(fetch_stone.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchStone)?;
        let fetch_body_properties = factory
            .query_goods_base_properties(fetch_body.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::CannotSummon)?;
        Ok(compose
            .iter()
            .find(|recipe| {
                recipe.fetch_stone == fetch_stone_properties.original_name()
                    && recipe.fetch_body == fetch_body_properties.original_name()
                    && recipe.material == material_properties.original_name()
            })
            .cloned())
    }

    /// `BatllteFairyCombine` удаляет input именно в порядке body, stone,
    /// material. Returned identity остаётся доступной для `OT_DELETE_OBJECT`;
    /// detached `CGoods` затем уничтожается тем же owner-ом.
    pub(crate) fn remove_battle_fairy_combine_input(
        &mut self,
        cell: BattleFairyCell,
    ) -> Option<BattleFairyCombineRemovedInput> {
        let goods = self.base.get_goods(cell.position())?;
        let identity = goods.identity();
        let amount = goods.amount();
        let outcome = self.base.remove_goods(identity.ex_id)?;
        match outcome {
            VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed))
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(AmountLimitGoodsTaken::Removed(
                removed,
            )) => {
                let _released = removed.goods;
                Some(BattleFairyCombineRemovedInput {
                    cell,
                    goods: identity,
                    amount,
                })
            }
            VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Split(_))
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(AmountLimitGoodsTaken::Split(_)) => {
                None
            }
        }
    }

    /// Проверяет только публично наблюдаемый запрос combine. Оригинал сначала
    /// ищет переданного игрока в `CGame::s_mapPlayer`: когда его нет, не
    /// публикуется ни packet, ни notification. Не найденные catalog properties
    /// классифицируются так же, как отсутствующий material (`ZHGS0056`).
    pub(crate) fn check_battle_fairy_combine(
        &self,
        player_id: Option<i32>,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> BattleFairyCombineCheck {
        let mut check = BattleFairyCombineCheck {
            player_id,
            ..BattleFairyCombineCheck::default()
        };
        if player_id.is_none() {
            return check;
        }

        let Some(material) = self.base.get_goods(BattleFairyCell::Material.position()) else {
            check.notification = Some(BattleFairyCombineNotification::MissingMaterial);
            return check;
        };
        let Some(fetch_stone) = self.base.get_goods(BattleFairyCell::FetchStone.position()) else {
            check.notification = Some(BattleFairyCombineNotification::MissingFetchStone);
            return check;
        };
        let Some(fetch_body) = self.base.get_goods(BattleFairyCell::FetchBody.position()) else {
            check.notification = Some(BattleFairyCombineNotification::MissingFetchBody);
            return check;
        };

        let Some(material_properties) =
            factory.query_goods_base_properties(material.base_properties_index())
        else {
            check.notification = Some(BattleFairyCombineNotification::MissingMaterial);
            return check;
        };
        let Some(fetch_stone_properties) =
            factory.query_goods_base_properties(fetch_stone.base_properties_index())
        else {
            check.notification = Some(BattleFairyCombineNotification::MissingMaterial);
            return check;
        };
        let Some(fetch_body_properties) =
            factory.query_goods_base_properties(fetch_body.base_properties_index())
        else {
            check.notification = Some(BattleFairyCombineNotification::MissingMaterial);
            return check;
        };

        let Some(recipe) = compose.iter().find(|recipe| {
            recipe.fetch_stone == fetch_stone_properties.original_name()
                && recipe.fetch_body == fetch_body_properties.original_name()
                && recipe.material == material_properties.original_name()
        }) else {
            check.notification = Some(BattleFairyCombineNotification::CannotSummon);
            return check;
        };

        check.result = BattleFairyCombineResult::CanCombine;
        check.availability = Some(BattleFairyCombineAvailability {
            message_type: BATTLE_FAIRY_COMBINE_MESSAGE_TYPE,
            deplete_fetch: recipe.deplete_fetch,
            // `fnstcw; or ah, 0x0c; fldcw; fistp` принудительно выбирает
            // rounding toward zero, не обычное округление до ближайшего.
            truncated_success_rate: x87_fistp_truncating(recipe.success_rate) as u32,
        });
        check
    }
}

/// `fistp dword` возвращает integer-indefinite `INT_MIN` для NaN и переполнения;
/// простое Rust-приведение насыщало бы такие значения и меняло wire payload.
fn x87_fistp_truncating(value: f32) -> i32 {
    if !value.is_finite() || value >= 2_147_483_648.0 || value < -2_147_483_648.0 {
        i32::MIN
    } else {
        value.trunc() as i32
    }
}
