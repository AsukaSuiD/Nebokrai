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
//! execution: global gate, точные различия notification для missing catalog,
//! порядок remove `body → stone → material`, RNG-result и ownership
//! созданного товара выполняются в `CPlayer`; этот owner даёт recipe,
//! positional storage и `LoadBFDefualtProperty` callback в том же порядке.
//!
//! Автоматический overload читает неинициализированный `m_eBFEquipPlace` у
//! catalog owner-а. Rust выражает этот UB как typed block, а не выбирает
//! логичную ячейку из позднего C++-донора. Upgrade/summon и остальные
//! player-integrated методы ниже остаются RAW.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsCleared, AmountLimitGoodsRelease, AmountLimitGoodsTaken,
};
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome, VolumeGoodsRemoveOutcome,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDefaultAddonWrite {
    pub(crate) property_type: i32,
    pub(crate) value_id: u32,
    pub(crate) value: i32,
    /// У setter-а нет registry fallback: false означает, что catalog-addon
    /// существовал только в статическом описании и instance не изменён.
    pub(crate) stored: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDefaultGoodsUpdate {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

/// Результат `LoadBFDefualtProperty`: callback регистрирует skill в owner-е
/// player до следующей addon-записи, а отчёт сохраняет только успешно
/// найденные через `GetSkill` registrations.
#[must_use = "report содержит обязательные skill-state и goods-update effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDefaultPropertyReport {
    pub(crate) attempted_writes: Vec<BattleFairyDefaultAddonWrite>,
    pub(crate) registered_skills: Vec<BattleFairyDefaultSkill>,
    pub(crate) goods_update: BattleFairyDefaultGoodsUpdate,
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
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingBaseProperties { index },
            );
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
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::GoodsRejected { cell },
            );
        }
        let property_effect =
            applies_property.then_some(BattleFairyPropertyAddEffect { cell, delta: 1 });
        BattleFairyContainerAddOutcome::Stored {
            base: self
                .base
                .add_goods_at(cell.position(), incoming, factory, owner_progress_allows),
            property_effect,
        }
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

    /// Exact `LoadBFDefualtProperty` меняет только товар принадлежащего
    /// существующему player-а. Базовые HP/MP копируются в current и maximum,
    /// затем выставляются три стартовых skill ID и три talent ID.
    pub(crate) fn load_default_properties<Register>(
        player_id: Option<i32>,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        register_skill: &mut Register,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyDefaultPropertyReport>
    where
        Register: FnMut(BattleFairyDefaultSkill) -> bool,
    {
        let player_id = player_id?;
        let strength = goods.addon_property_value(factory, GAP_BF_STRENGH_BASE, 1);
        let spiritualism = goods.addon_property_value(factory, GAP_BF_SPRITUALISM_BASE, 1);
        let mut attempted_writes: Vec<_> = [
            (GAP_BF_HP, 1, strength),
            (GAP_BF_MAX_HP, 1, strength),
            (GAP_BF_MP, 1, spiritualism),
            (GAP_BF_MAX_MP, 1, spiritualism),
        ]
        .into_iter()
        .map(
            |(property_type, value_id, value)| BattleFairyDefaultAddonWrite {
                property_type,
                value_id,
                value,
                stored: goods.set_addon_property_value_core(property_type, value_id, value),
            },
        )
        .collect();

        let mut registered_skills = Vec::with_capacity(3);
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
            attempted_writes.push(BattleFairyDefaultAddonWrite {
                property_type,
                value_id: 2,
                value: skill.id as i32,
                stored: goods.set_addon_property_value_core(property_type, 2, skill.id as i32),
            });
            if register_skill(skill) {
                registered_skills.push(skill);
            }
        }

        attempted_writes.extend(
            [
                (GAP_BF_SKY, 2, 0x3c0),
                (GAP_BF_EARTH, 2, 0x3c1),
                (GAP_BF_MAN, 2, 0x3c2),
            ]
            .into_iter()
            .map(
                |(property_type, value_id, value)| BattleFairyDefaultAddonWrite {
                    property_type,
                    value_id,
                    value,
                    stored: goods.set_addon_property_value_core(property_type, value_id, value),
                },
            ),
        );
        Some(BattleFairyDefaultPropertyReport {
            attempted_writes,
            registered_skills,
            goods_update: BattleFairyDefaultGoodsUpdate {
                message_type: crate::gameserver::appserver::goods::cbattlefairyproperty::BATTLE_FAIRY_GOODS_UPDATE_MESSAGE_TYPE,
                player_id,
                goods: goods.identity(),
                old_client_payload: encode_old_client(goods),
            },
        })
    }

    /// Actual `BatllteFairyCombine` различает отсутствие properties fetch-body
    /// (`ZHGS0060`) от check-only opcode, который возвращает `ZHGS0056`.
    pub(crate) fn battle_fairy_combine_recipe(
        &self,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> Result<BattleFairyCompose, BattleFairyCombineExecutionNotification> {
        let material = self
            .base
            .get_goods(BattleFairyCell::Material.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingMaterial)?;
        let material_properties = factory
            .query_goods_base_properties(material.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::MissingMaterial)?;
        let fetch_stone = self
            .base
            .get_goods(BattleFairyCell::FetchStone.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchStone)?;
        let fetch_stone_properties = factory
            .query_goods_base_properties(fetch_stone.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchStone)?;
        let fetch_body = self
            .base
            .get_goods(BattleFairyCell::FetchBody.position())
            .ok_or(BattleFairyCombineExecutionNotification::MissingFetchBody)?;
        let fetch_body_properties = factory
            .query_goods_base_properties(fetch_body.base_properties_index())
            .ok_or(BattleFairyCombineExecutionNotification::CannotSummon)?;
        compose
            .iter()
            .find(|recipe| {
                recipe.fetch_stone == fetch_stone_properties.original_name()
                    && recipe.fetch_body == fetch_body_properties.original_name()
                    && recipe.material == material_properties.original_name()
            })
            .cloned()
            .ok_or(BattleFairyCombineExecutionNotification::CannotSummon)
    }

    /// `BatllteFairyCombine` удаляет input именно в порядке body, stone,
    /// material. Returned identity остаётся доступной для `OT_DELETE_OBJECT`;
    /// detached `CGoods` затем уничтожается тем же owner-ом.
    pub(crate) fn remove_battle_fairy_combine_input(
        &mut self,
        cell: BattleFairyCell,
    ) -> Option<BattleFairyCombineRemovedInput> {
        let identity = self.base.get_goods(cell.position())?.identity();
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp

// ============================================================================
// FUNCTION: CBattleFairyContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:68
// RVA: 0x000DB670
// ADDRESS: 004db670
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:334
// RVA: 0x000DB690
// ADDRESS: 004db690
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:74
// RVA: 0x000FD460
// ADDRESS: 004fd460
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:339
// RVA: 0x000FD470
// ADDRESS: 004fd470
// PROTOTYPE: CBaseObject * __thiscall Remove(ulong param_1, ulong param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1815
// RVA: 0x000FD4A0
// ADDRESS: 004fd4a0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1821
// RVA: 0x000FD4B0
// ADDRESS: 004fd4b0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:81
// RVA: 0x000FD510
// ADDRESS: 004fd510
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetSuccessResult
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1148
// RVA: 0x000FD840
// ADDRESS: 004fd840
// PROTOTYPE: ulong __thiscall GetSuccessResult(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetFailResult
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1216
// RVA: 0x000FD9A0
// ADDRESS: 004fd9a0
// PROTOTYPE: ulong __thiscall GetFailResult(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1269
// RVA: 0x000FDAB0
// ADDRESS: 004fdab0
// PROTOTYPE: ulong __thiscall GetProbability(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetUpgradePrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1316
// RVA: 0x000FDBF0
// ADDRESS: 004fdbf0
// PROTOTYPE: ulong __thiscall GetUpgradePrice(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::CheckBattleFairyCombine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:495
// RVA: 0x000FE780
// ADDRESS: 004fe780
// PROTOTYPE: eCombineResult __thiscall CheckBattleFairyCombine(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:773
// RVA: 0x000FECB0
// ADDRESS: 004fecb0
// PROTOTYPE: bool __thiscall DeleteGoods(eBattleFairy_Place_Cell param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::ResetPotential
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:800
// RVA: 0x000FEE20
// ADDRESS: 004fee20
// PROTOTYPE: void __thiscall ResetPotential(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::AllocatePotential
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:924
// RVA: 0x000FF480
// ADDRESS: 004ff480
// PROTOTYPE: void __thiscall AllocatePotential(int param_1, GOODS_ADDON_PROPERTIES param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::IsValidateUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1327
// RVA: 0x00100030
// ADDRESS: 00500030
// PROTOTYPE: bool __thiscall IsValidateUpgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Upgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1394
// RVA: 0x001002E0
// ADDRESS: 005002e0
// PROTOTYPE: bool __thiscall Upgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::ResetSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1851
// RVA: 0x00101530
// ADDRESS: 00501530
// PROTOTYPE: void __thiscall ResetSkill(int param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::SummonBF
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2096
// RVA: 0x00101CB0
// ADDRESS: 00501cb0
// PROTOTYPE: void __thiscall SummonBF(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::BFPropertyAdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2286
// RVA: 0x001020E0
// ADDRESS: 005020e0
// PROTOTYPE: void __thiscall BFPropertyAdd(eBattleFairy_Place_Cell param_1, CGoods * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::LoadBFDefualtProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2475
// RVA: 0x00102BC0
// ADDRESS: 00502bc0
// PROTOTYPE: void __thiscall LoadBFDefualtProperty(int param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:157
// RVA: 0x00103080
// ADDRESS: 00503080
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:290
// RVA: 0x00103310
// ADDRESS: 00503310
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::BatllteFairyCombine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:577
// RVA: 0x001034C0
// ADDRESS: 005034c0
// PROTOTYPE: bool __thiscall BatllteFairyCombine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::CBattleFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:46
// RVA: 0x00103FC0
// ADDRESS: 00503fc0
// PROTOTYPE: undefined __thiscall CBattleFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::~CBattleFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:61
// RVA: 0x00104190
// ADDRESS: 00504190
// PROTOTYPE: void __thiscall ~CBattleFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
