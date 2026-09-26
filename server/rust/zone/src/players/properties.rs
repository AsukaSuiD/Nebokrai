//! ATTR-семья `CPlayer::UpdateProperty` живого игрока Zone: пересчёт
//! base+equipment/CiQing свойств (`MountAllEquip`/`MountCiQingEquip`),
//! производные battle-fairy addon-ов, `apply_*_state` формулы наложенных
//! состояний (undead/extended/change-body/ride), commit вычисленного снимка
//! в `tagProperty` wire и equipment-flash, client allocation `0x8FA01` и
//! серверный домен `GetCurrentTypeValue` с кадром `0xC010E`.
//!
//! Hub `CPlayer` старого пакета (`gameserver/appserver/player.rs`) бережёт
//! делегаты прежних сигнатур и передаёт заёмные проекции своих полей
//! ([`PlayerMountedPropertyParts`], [`PlayerEquipmentPropertyParts`],
//! [`PlayerCombatWireParts`]); владение полями остаётся у hub-а, циклической
//! зависимости на уровне типов нет. Входы Shared globesetup
//! (`GlobePlayerPropertyCoefficients`, base combat scales, critical rate)
//! приходят параметрами; внешние caller-ы игры (game.rs, playermessage.rs,
//! script/function.rs, other states) продолжают видеть прежние связанные
//! сигнатуры hub-а без правок.
//!
//! Машинная досверка P-ATTR: порядок `recompute_update_property` и точечная
//! арифметика сверены с телом `CPlayer::UpdateProperty` 0x4593E0
//! (`player-ctor.txt:184-995`), дампом `MountEquip` 0x442610
//! (`mount-equip.txt`) и `MountCiQingEquip` 0x447000
//! (`mount-ciqing-equip.txt`) той же точной пары. Обнаруженные
//! расхождения — три float-ассоциативности (две в BF prelude: growth и
//! порядок суммы; одна в MountEquip-cases: масштаб производных BF
//! последним) — исправлены по машинному разбору (см.
//! `refresh_battle_fairy_equipment_properties` и `apply_-cases ниже).
//! Внутреннее устройство `MountAllEquip` 0x453480 в корпус не входит:
//! двойной снимок recompute ×2 → BF pair → diff там остаётся PARTIAL
//! (противоречий к принятой модели не найдено).
//!
//! Исходный owner: `server/gameserver/appserver/player.cpp/.h`, точная пара
//! GameServer/gameserver.exe + GameServer.pdb.

use std::collections::BTreeMap;

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::resources::GlobePlayerPropertyCoefficients;

use crate::combat::{PlayerCombatProperties, truncate_original};
use crate::content::goods::{
    GAP_AGILITY_CORRECTION, GAP_ANIMA_BIND, GAP_ARMOR_CORRECTION, GAP_ATTACK_AVOID,
    GAP_ATTACK_SPEED_CORRECTION, GAP_BF_AGILITY, GAP_BF_AGILITY_BASE, GAP_BF_AGILITY_POTENTIAL,
    GAP_BF_ATTACK, GAP_BF_ATTACK_BASE, GAP_BF_ATTACK_POTENTIAL, GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST,
    GAP_BF_BLAST_POTENTIAL, GAP_BF_BRAVE, GAP_BF_BRAVE_BASE, GAP_BF_BRAVE_POTENTIAL,
    GAP_BF_CUT_HURT_SCALE, GAP_BF_HP, GAP_BF_LEVEL, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP,
    GAP_BF_PULLULATERATE, GAP_BF_SPRITE, GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_POTENTIAL,
    GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE, GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_STRENGH,
    GAP_BF_STRENGH_BASE, GAP_BF_STRENGH_POTENTIAL, GAP_BLAST_ATTACK, GAP_BLAST_ELEMENT_ATTACK,
    GAP_BREAK_ARMOUR, GAP_BREAK_BOUND, GAP_BREAK_ELEMENT, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
    GAP_CONSTITUTION_CORRECTION, GAP_DODGE_CORRECTION, GAP_ELEMENT_ATTACK_CORRECTION,
    GAP_ELEMENT_AVOID, GAP_ELEMENT_RESISTANCE_CORRECTION, GAP_EQUIP_ACTIVE, GAP_FAIRY_AGILITY,
    GAP_FAIRY_HP, GAP_FAIRY_STRENGTH, GAP_FAIRY_WAKAN, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_FULL_MISS, GAP_FUMO_PROPERTY, GAP_GOLD_POWER, GAP_GOODS_EQUIMENT_FLASH,
    GAP_GOODS_LIFE_TYPE, GAP_GOODS_MAXIMUM_DURABILITY, GAP_HIT_RATE_CORRECTION,
    GAP_HP_RESTORE_SPEED_CORRECTION, GAP_HP_UPPER_LIMIT_CORRECTION, GAP_MAXIMUM_ATTACK_CORRECTION,
    GAP_MINIMUM_ATTACK_CORRECTION, GAP_MP_RESTORE_SPEED_CORRECTION, GAP_MP_UPPER_LIMIT_CORRECTION,
    GAP_PUNCTURE, GAP_ROLE_MINIMUM_LEVEL_LIMIT, GAP_STIFFEN_PROBABILITY_CORRECTION,
    GAP_STRENGTH_CORRECTION, GAP_WAKAN_CORRECTION,
};
use crate::content::goodsfactory::CGoodsFactory;
use crate::effects::{ChangeBodyState, ExtendedState, RideState, UndeadState};
use crate::items::cequipmentcontainer::CEquipmentContainer;
use crate::items::cgoods::CGoods;
use crate::items::cvolumelimitgoodscontainer::CVolumeLimitGoodsContainer;
use crate::players::gamesave::{
    BASE_ATTACK_SPEED_OFFSET, BASE_CCH_OFFSET, BASE_CONSTITUTION_OFFSET, BASE_DEFENSE_OFFSET,
    BASE_DEXTERITY_OFFSET, BASE_DODGE_OFFSET, BASE_ELEMENT_RESISTANCE_OFFSET, BASE_HEALTH_OFFSET,
    BASE_HIT_OFFSET, BASE_HP_RECOVERY_OFFSET, BASE_INTELLIGENCE_OFFSET, BASE_KILL_COUNT_OFFSET,
    BASE_MANA_OFFSET, BASE_MAXIMUM_ATTACK_OFFSET, BASE_MAXIMUM_HP_OFFSET, BASE_MAXIMUM_MP_OFFSET,
    BASE_MINIMUM_ATTACK_OFFSET, BASE_MP_RECOVERY_OFFSET, BASE_PK_COUNT_OFFSET,
    BASE_STRENGTH_OFFSET, PLAYER_BASE_PROPERTY_WIRE_SIZE, PLAYER_COMBAT_PROPERTY_WIRE_SIZE,
    PlayerBaseProperties, read_player_wire_u16, read_player_wire_u32,
};

/// Итог полного `MountAllEquip` прохода: вычисленный combat snapshot и
/// объединённая CiQing-разность обязательного `0xC010E` отправителя.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPropertyRecompute {
    pub properties: PlayerCombatProperties,
    pub ci_qing_result_values: BTreeMap<u32, u32>,
}

/// Exact `GetPlayerAllProperties` diagnostic projection. Числа хранят raw
/// DWORD vararg bits: конкретный `%d`/`%u` шаблона определяет их signed view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerAllPropertiesDiagnosticSnapshot {
    pub name: Vec<u8>,
    pub summary_words: [u32; 15],
    pub base_combat_words: [u32; 15],
    pub current_combat_words: [u32; 20],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerStatAllocationState {
    pub sex: u8,
    pub occupation: u8,
    pub remain_point: u16,
    pub base_maximum_hp: u32,
    pub base_maximum_mp: u32,
    pub base_strength: u32,
    pub base_dexterity: u32,
    pub base_constitution: u32,
    pub base_intelligence: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerStatAllocationMutation {
    pub player_id: i32,
    pub selector: u8,
    pub stat_changed: bool,
    pub previous: PlayerStatAllocationState,
    pub current: PlayerStatAllocationState,
}

/// Заёмная read-проекция hub-полей для снимков `MountAllEquip`: base wire и
/// typed-части, экипировка, восемь CiQing-ячеек и флаг призванной боевой феи.
/// Владение остаётся у hub-а; проекция живёт только на время вызова.
pub struct PlayerMountedPropertyParts<'a> {
    pub equipment: &'a CEquipmentContainer,
    pub ci_qing: &'a CVolumeLimitGoodsContainer,
    pub base_properties: &'a PlayerBaseProperties,
    pub base_property_wire: &'a [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    pub battle_fairy_summoned: bool,
}

/// Заёмная mut-проекция hub-полей полного `UpdateProperty` прохода:
/// battle-fairy prelude пишет slot 10 экипировки, CiQing base-приложение —
/// сохранённый `tagBaseProperty`, а результат насыщенной разницы заменяет
/// `m_mapCiQingAddValue`. Владение и pending-флаг TaoZhuang остаются у hub-а.
pub struct PlayerEquipmentPropertyParts<'a> {
    pub equipment: &'a mut CEquipmentContainer,
    pub ci_qing: &'a mut CVolumeLimitGoodsContainer,
    pub base_properties: &'a mut PlayerBaseProperties,
    pub base_property_wire: &'a [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    pub battle_fairy_summoned: bool,
    pub ci_qing_add_values: &'a mut BTreeMap<u32, u32>,
    pub ci_qing_tao_zhuang_add_values: &'a BTreeMap<u32, u32>,
    pub equipment_changed: &'a mut bool,
}

impl PlayerEquipmentPropertyParts<'_> {
    /// Read-проекция тех же полей для двух `MountAllEquip` снимков внутри
    /// полного прохода; живёт только на длину одного вызова снимка.
    fn snapshot(&self) -> PlayerMountedPropertyParts<'_> {
        PlayerMountedPropertyParts {
            equipment: self.equipment,
            ci_qing: self.ci_qing,
            base_properties: self.base_properties,
            base_property_wire: self.base_property_wire,
            battle_fairy_summoned: self.battle_fairy_summoned,
        }
    }
}

/// Заёмная проекция hub-полей property commit-а: живой снимок, его 0x9c
/// wire-проекция, экипировка flash-чтения и 17 flash-ячеек с previous-table.
pub struct PlayerCombatWireParts<'a> {
    pub combat_properties: &'a mut PlayerCombatProperties,
    pub combat_property_wire: &'a mut [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    pub equipment: &'a CEquipmentContainer,
    pub flash_previous: &'a mut [u32; 17],
    pub flash_current: &'a mut [u32; 17],
    pub flash_changed: &'a mut bool,
}

// --- Общая формула одного предмета (MountEquip/MountEquipRide/MountFuMoProperty). ---

pub fn apply_equipment_goods_properties(
    properties: &mut PlayerCombatProperties,
    goods: &CGoods,
    factory: &CGoodsFactory,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: usize,
    include_fairy_properties: bool,
    active_level: Option<u8>,
) {
    fn add_u32(target: &mut u32, delta: i32) {
        *target = (i64::from(*target) + i64::from(delta)).clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn add_mount_u32(target: &mut u32, delta: i32) {
        let value = target.wrapping_add(delta as u32);
        *target = if delta < 0 && (value as i32) < 0 {
            0
        } else {
            value.min(i32::MAX as u32)
        };
    }
    fn add_mount_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if delta < 0 && value < 0 {
            0
        } else {
            value as u16
        };
    }
    // `AddPreItemToPlayer`, `MountFuMoProperty` и `ActiveEquip` перед FISTP
    // выставляют x87 RC=truncate; производные поля усекают полную сумму.
    fn scaled_delta(value: i32, coefficient: f32) -> i32 {
        ((value as f32) * coefficient).trunc() as i32
    }
    fn add_derived_u32(target: &mut u32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i64;
        *target = value.clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_derived_i32(target: &mut i32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if delta < 0 && value < 0 { 0 } else { value };
    }
    fn add_derived_u16(target: &mut u16, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn active_u32(current: u32, addition: i32, scale: f64) -> u32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        value.clamp(0, i64::from(i32::MAX)) as u32
    }
    fn active_signed(current: i32, addition: i32, scale: f64) -> i32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64 as u32;
        if (value as i32) < 0 { 0 } else { value as i32 }
    }
    fn active_u16(current: u16, addition: i32, scale: f64) -> u16 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        if value < 0 { 0 } else { value as u16 }
    }

    fn apply_active_equip(
        properties: &mut PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
        selector: i32,
        percentage: i32,
    ) {
        let scale = f64::from(percentage) * 0.01_f64;
        match selector {
            GAP_MINIMUM_ATTACK_CORRECTION
            | GAP_MAXIMUM_ATTACK_CORRECTION
            | GAP_ELEMENT_ATTACK_CORRECTION => {
                properties.minimum_attack = active_u32(
                    properties.minimum_attack,
                    goods.addon_property_value(factory, GAP_MINIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.maximum_attack = active_u32(
                    properties.maximum_attack,
                    goods.addon_property_value(factory, GAP_MAXIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.element_modify = active_signed(
                    properties.element_modify,
                    goods.addon_property_value(factory, GAP_ELEMENT_ATTACK_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ARMOR_CORRECTION | GAP_ELEMENT_RESISTANCE_CORRECTION => {
                properties.defense = active_u32(
                    properties.defense,
                    goods.addon_property_value(factory, GAP_ARMOR_CORRECTION, 1),
                    scale,
                );
                properties.element_resistance = active_u32(
                    properties.element_resistance,
                    goods.addon_property_value(factory, GAP_ELEMENT_RESISTANCE_CORRECTION, 1),
                    scale,
                );
            }
            GAP_HP_UPPER_LIMIT_CORRECTION => {
                properties.maximum_hp = active_u32(
                    properties.maximum_hp,
                    goods.addon_property_value(factory, GAP_HP_UPPER_LIMIT_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ATTACK_AVOID | GAP_ELEMENT_AVOID => {
                properties.attack_avoid = active_u16(
                    properties.attack_avoid,
                    goods.addon_property_value(factory, GAP_ATTACK_AVOID, 1),
                    scale,
                );
                properties.element_avoid = active_u16(
                    properties.element_avoid,
                    goods.addon_property_value(factory, GAP_ELEMENT_AVOID, 1),
                    scale,
                );
            }
            _ => {}
        }
    }

    let enabled = goods.enabled_addon_properties(factory);
    // Native equipment/ride owners вызывают positive, затем negative pass:
    // первый pass принимает неотрицательные addon-ы, второй — отрицательные.
    for positive_pass in [true, false] {
        for &stored_type in &enabled {
            if stored_type == GAP_EQUIP_ACTIVE {
                if positive_pass
                    && goods.has_addon_property_values(factory, GAP_ANIMA_BIND)
                    && goods.addon_property_value(factory, GAP_ANIMA_BIND, 1) != 0
                    && active_level.is_some_and(|level| {
                        goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                            <= i32::from(level)
                    })
                {
                    apply_active_equip(
                        properties,
                        goods,
                        factory,
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 1),
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 2),
                    );
                }
                continue;
            }
            let fumo = stored_type == GAP_FUMO_PROPERTY;
            let (property_type, delta) = if fumo {
                (
                    goods.addon_property_value(factory, stored_type, 1),
                    goods.addon_property_value(factory, stored_type, 2),
                )
            } else {
                (
                    stored_type,
                    goods.addon_property_value(factory, stored_type, 1),
                )
            };
            // GAP_FUMO_PROPERTY native-ветка существует только в первом
            // `MountEquipRide(true)` pass и уже внутри принимает signed delta.
            if (fumo && !positive_pass) || (!fumo && (delta >= 0) != positive_pass) {
                continue;
            }
            // MountEquip0x00442610 и MountEquipRide0x0043C5E0 складывают
            // low32 до signed negative gate. Отдельный MountFuMoProperty
            // здесь сохраняет прежний адаптер, не наследуя этот контракт.
            let add_direct_u32 = if fumo { add_u32 } else { add_mount_u32 };
            let add_direct_u16 = if fumo { add_u16 } else { add_mount_u16 };
            match property_type {
                GAP_MINIMUM_ATTACK_CORRECTION => {
                    add_direct_u32(&mut properties.minimum_attack, delta)
                }
                GAP_MAXIMUM_ATTACK_CORRECTION => {
                    add_direct_u32(&mut properties.maximum_attack, delta)
                }
                GAP_ELEMENT_ATTACK_CORRECTION => {
                    let value = properties.element_modify.wrapping_add(delta);
                    properties.element_modify = if delta < 0 && value < 0 { 0 } else { value };
                }
                GAP_ARMOR_CORRECTION => add_direct_u32(&mut properties.defense, delta),
                GAP_ATTACK_SPEED_CORRECTION => {
                    if fumo {
                        add_u16(&mut properties.attack_speed, delta);
                    } else {
                        properties.attack_speed =
                            properties.attack_speed.wrapping_add(delta as u16);
                    }
                }
                GAP_HIT_RATE_CORRECTION => add_direct_u16(&mut properties.hit, delta),
                GAP_FATAL_BLOW_RATE_CORRECTION => add_direct_u16(&mut properties.cch, delta),
                GAP_DODGE_CORRECTION => add_direct_u16(&mut properties.dodge, delta),
                GAP_ELEMENT_RESISTANCE_CORRECTION => {
                    add_direct_u32(&mut properties.element_resistance, delta)
                }
                GAP_HP_RESTORE_SPEED_CORRECTION => {
                    add_direct_u16(&mut properties.hp_recovery, delta)
                }
                GAP_MP_RESTORE_SPEED_CORRECTION => {
                    add_direct_u16(&mut properties.mp_recovery, delta)
                }
                GAP_STRENGTH_CORRECTION => {
                    add_direct_u32(&mut properties.strength, delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_AGILITY_CORRECTION => {
                    add_direct_u32(&mut properties.dexterity, delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_CONSTITUTION_CORRECTION => {
                    add_direct_u32(&mut properties.constitution, delta);
                    add_derived_u32(
                        &mut properties.maximum_hp,
                        delta,
                        coefficients.con_to_max_hp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.defense,
                        delta,
                        coefficients.con_to_defense[occupation],
                    );
                }
                GAP_WAKAN_CORRECTION => {
                    add_direct_u32(&mut properties.intelligence, delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_HP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_hp, delta),
                GAP_MP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_mp, delta),
                GAP_STIFFEN_PROBABILITY_CORRECTION => add_direct_u16(&mut properties.reank, delta),
                GAP_BURDEN_UPPER_LIMIT_CORRECTION => add_direct_u16(&mut properties.burden, delta),
                GAP_ATTACK_AVOID => add_direct_u16(&mut properties.attack_avoid, delta),
                GAP_ELEMENT_AVOID => add_direct_u16(&mut properties.element_avoid, delta),
                GAP_FULL_MISS => add_direct_u16(&mut properties.full_miss, delta),
                GAP_BLAST_ATTACK => add_direct_u16(&mut properties.blast_attack, delta),
                GAP_BLAST_ELEMENT_ATTACK => {
                    // Legacy case 96 берёт base из wBlastAttack, не из target.
                    let mut value = properties.blast_attack;
                    add_direct_u16(&mut value, delta);
                    properties.blast_element_attack = value;
                }
                GAP_FAIRY_STRENGTH if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_strength_to_player);
                    add_u32(&mut properties.strength, player_delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        player_delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        player_delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_FAIRY_AGILITY if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_agility_to_player);
                    add_u32(&mut properties.dexterity, player_delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        player_delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        player_delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_FAIRY_WAKAN if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_wakan_to_player);
                    add_u32(&mut properties.intelligence, player_delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        player_delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        player_delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        player_delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_FAIRY_HP if include_fairy_properties => add_u32(
                    &mut properties.maximum_hp,
                    scaled_delta(delta, coefficients.fairy_hp_to_player),
                ),
                _ => {}
            }
        }
    }
}

// --- Пересчёт base + mounted (MountAllEquip / MountCiQingEquip). ---

/// Базовая и equipment/CiQing половина `CPlayer::UpdateProperty` до
/// виртуального `CMoveShape::UpdateProperty`. Два signed addon-pass-а
/// сохраняют slot order `MountAllEquip`. Восемь recovery scalar-ов
/// каждый раз восстанавливаются из `CGlobeSetup::tagSetup` до применения
/// equipment, CiQing и state addon-ов, как остальные базовые свойства.
pub fn recompute_base_and_equipment_properties(
    parts: &PlayerMountedPropertyParts<'_>,
    coefficients: GlobePlayerPropertyCoefficients,
    base_combat_scales: [f32; 5],
    critical_rate: f32,
    goods_factory: &CGoodsFactory,
) -> PlayerCombatProperties {
    recompute_base_and_mounted_properties(
        parts,
        coefficients,
        base_combat_scales,
        critical_rate,
        goods_factory,
        true,
    )
}

/// Базовые cases `0x80..0x84` исходного `MountCiQingEquip`. Они не входят
/// в combat snapshot: owner меняет сохранённый `tagBaseProperty` в порядке
/// восьми ячеек, сперва для неотрицательных, затем для отрицательных
/// addon-ов каждого предмета. Durability здесь намеренно не проверяется.
pub fn apply_ci_qing_base_properties(
    parts: &mut PlayerEquipmentPropertyParts<'_>,
    factory: &CGoodsFactory,
) {
    fn add_property(target: &mut u32, delta: i32) {
        let next = target.wrapping_add(delta as u32);
        *target = if delta < 0 && (next as i32) < 0 {
            0
        } else {
            next
        };
    }

    for position in 0..parts.ci_qing.size() {
        let additions = {
            let Some(goods) = parts.ci_qing.get_goods(position) else {
                continue;
            };
            goods
                .enabled_addon_properties(factory)
                .into_iter()
                .map(|property_type| {
                    (
                        property_type,
                        goods.addon_property_value(factory, property_type, 1),
                    )
                })
                .collect::<Vec<_>>()
        };
        for positive_pass in [true, false] {
            for &(property_type, delta) in &additions {
                if (delta >= 0) != positive_pass {
                    continue;
                }
                match property_type {
                    GAP_BREAK_ARMOUR => {
                        add_property(&mut parts.base_properties.break_armour, delta)
                    }
                    GAP_PUNCTURE => add_property(&mut parts.base_properties.puncture, delta),
                    GAP_BREAK_ELEMENT => {
                        add_property(&mut parts.base_properties.break_element, delta)
                    }
                    GAP_BREAK_BOUND => add_property(&mut parts.base_properties.break_bound, delta),
                    GAP_GOLD_POWER => add_property(&mut parts.base_properties.power_of_gold, delta),
                    _ => {}
                }
            }
        }
    }
}

/// Первый снимок `MountAllEquip`: обычная экипировка уже применена, а
/// восемь CiQing-ячеек ещё нет. Он нужен для exact насыщенной разницы
/// `UpdateCiQingProperty` и не создаёт теневого player state.
pub fn recompute_without_ci_qing_properties(
    parts: &PlayerMountedPropertyParts<'_>,
    coefficients: GlobePlayerPropertyCoefficients,
    base_combat_scales: [f32; 5],
    critical_rate: f32,
    goods_factory: &CGoodsFactory,
) -> PlayerCombatProperties {
    recompute_base_and_mounted_properties(
        parts,
        coefficients,
        base_combat_scales,
        critical_rate,
        goods_factory,
        false,
    )
}

fn recompute_base_and_mounted_properties(
    parts: &PlayerMountedPropertyParts<'_>,
    coefficients: GlobePlayerPropertyCoefficients,
    base_combat_scales: [f32; 5],
    critical_rate: f32,
    goods_factory: &CGoodsFactory,
    include_ci_qing: bool,
) -> PlayerCombatProperties {
    let occupation = usize::from(parts.base_properties.occupation).min(2);
    let base_u16 = |offset| read_player_wire_u16(parts.base_property_wire, offset);
    let base_u32 = |offset| read_player_wire_u32(parts.base_property_wire, offset);
    let derived = |value: u32, coefficient: f32| ((value as f32) * coefficient).trunc() as u32;
    let mut properties = PlayerCombatProperties {
        maximum_hp: parts.base_properties.base_maximum_hp.wrapping_add(derived(
            parts.base_properties.base_constitution,
            coefficients.con_to_max_hp[occupation],
        )),
        maximum_mp: parts.base_properties.base_maximum_mp.wrapping_add(derived(
            parts.base_properties.base_intelligence,
            coefficients.int_to_max_mp[occupation],
        )),
        maximum_yp: parts.base_properties.maximum_yp,
        maximum_rp: parts.base_properties.maximum_rp,
        strength: parts.base_properties.base_strength,
        dexterity: parts.base_properties.base_dexterity,
        constitution: parts.base_properties.base_constitution,
        intelligence: parts.base_properties.base_intelligence,
        minimum_attack: base_u32(BASE_MINIMUM_ATTACK_OFFSET).wrapping_add(derived(
            parts.base_properties.base_dexterity,
            coefficients.dex_to_min_attack[occupation],
        )),
        maximum_attack: base_u32(BASE_MAXIMUM_ATTACK_OFFSET).wrapping_add(derived(
            parts.base_properties.base_strength,
            coefficients.str_to_max_attack[occupation],
        )),
        attack_speed: base_u16(BASE_ATTACK_SPEED_OFFSET),
        hit: base_u16(BASE_HIT_OFFSET),
        dodge: base_u16(BASE_DODGE_OFFSET),
        cch: base_u16(BASE_CCH_OFFSET),
        defense: base_u32(BASE_DEFENSE_OFFSET).wrapping_add(derived(
            parts.base_properties.base_constitution,
            coefficients.con_to_defense[occupation],
        )),
        element_resistance: base_u32(BASE_ELEMENT_RESISTANCE_OFFSET).wrapping_add(derived(
            parts.base_properties.base_intelligence,
            coefficients.int_to_resistant[occupation],
        )),
        hp_recovery: base_u16(BASE_HP_RECOVERY_OFFSET),
        mp_recovery: base_u16(BASE_MP_RECOVERY_OFFSET),
        burden: parts.base_properties.base_burden.wrapping_add(derived(
            parts.base_properties.base_strength,
            coefficients.str_to_burden[occupation],
        ) as u16),
        reank: derived(
            parts.base_properties.base_dexterity,
            coefficients.dex_to_stiff[occupation],
        ) as u16,
        element_modify: derived(
            parts.base_properties.base_intelligence,
            coefficients.int_to_element[occupation],
        ) as i32,
        blast_attack_scale_bits: base_combat_scales[0].to_bits(),
        blast_defense_scale_bits: base_combat_scales[1].to_bits(),
        element_blast_attack_scale_bits: base_combat_scales[2].to_bits(),
        element_blast_defense_scale_bits: base_combat_scales[3].to_bits(),
        full_miss_scale_bits: base_combat_scales[4].to_bits(),
        critical_rate_bits: critical_rate.to_bits(),
        resume_hp_peace: coefficients.resume_hp_peace,
        resume_mp_peace: coefficients.resume_mp_peace,
        resume_hp_fight: coefficients.resume_hp_fight,
        resume_mp_fight: coefficients.resume_mp_fight,
        restored_hp_peace: coefficients.restored_hp_peace,
        restored_mp_peace: coefficients.restored_mp_peace,
        restored_hp_fight: coefficients.restored_hp_fight,
        restored_mp_fight: coefficients.restored_mp_fight,
        battle_fairy_summoned: parts.battle_fairy_summoned,
        battle_fairy_recall: parts.base_properties.battle_fairy_recall,
        battle_fairy_died: parts.base_properties.battle_fairy_died,
        ..PlayerCombatProperties::default()
    };
    let mut active_levels = BTreeMap::<i32, u8>::new();
    for (_, goods) in parts.equipment.traversing_goods() {
        if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
            && goods.addon_property_value(goods_factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
        {
            continue;
        }
        if !goods.has_addon_property_values(goods_factory, GAP_ANIMA_BIND)
            || goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 1) == 0
        {
            continue;
        }
        let key = goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 2);
        let mut level =
            goods.addon_property_value(goods_factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1) as u8;
        if level > 99 {
            level = level.wrapping_add(10);
        }
        active_levels
            .entry(key)
            .and_modify(|current| *current = (*current).max(level))
            .or_insert(level);
    }
    for (column, goods) in parts.equipment.traversing_goods() {
        apply_equipment_goods_properties(
            &mut properties,
            goods,
            goods_factory,
            coefficients,
            occupation,
            true,
            active_levels.get(&(column.position() as i32)).copied(),
        );
    }
    if include_ci_qing {
        for position in 0..parts.ci_qing.size() {
            let Some(goods) = parts.ci_qing.get_goods(position) else {
                continue;
            };
            apply_equipment_goods_properties(
                &mut properties,
                goods,
                goods_factory,
                coefficients,
                occupation,
                true,
                active_levels.get(&(position as i32)).copied(),
            );
        }
    }
    properties
}

// --- Battle-fairy equipment prelude и property-pass-ы slot 10. ---

/// Мутирующая prelude исходного `CPlayer::UpdateProperty`: slot 10
/// пересчитывает производные battle-fairy addon-ы до `MountAllEquip`.
/// Значения `2` остаются instance-modifier-ами, а текущие HP/MP здесь
/// намеренно не зажимаются — native owner обновляет только максимумы.
/// Шесть FISTP-конверсий усекают полную сумму к нулю.
pub fn refresh_battle_fairy_equipment_properties(
    equipment: &mut CEquipmentContainer,
    factory: &CGoodsFactory,
) {
    let Some(goods) = equipment.get_goods_mut(10) else {
        return;
    };
    if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
        return;
    }
    let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1);
    let pullulate = goods.addon_property_value(factory, GAP_BF_PULLULATERATE, 1) as f32;
    // Машинный порядок `UpdateProperty` (0x4597A2..0x4597E7): сперва
    // fmul pullulate на f32 0.0001 ([0x64E9E8]), затем fmul (level-1) на
    // произведение и fadd f32 1.0 ([0x64DB50]). Декомпилят писал
    // `(level-1)*pullulate*0.0001`; float-ассоциативность здесь значима.
    let pullulate_scale = pullulate * 0.0001_f32;
    let growth = (level.wrapping_sub(1) as f32) * pullulate_scale + 1.0_f32;
    for (value_property, base_property, potential_property) in [
        (GAP_BF_BRAVE, GAP_BF_BRAVE_BASE, GAP_BF_BRAVE_POTENTIAL),
        (
            GAP_BF_AGILITY,
            GAP_BF_AGILITY_BASE,
            GAP_BF_AGILITY_POTENTIAL,
        ),
        (
            GAP_BF_SPRITUALISM,
            GAP_BF_SPRITUALISM_BASE,
            GAP_BF_SPRITUALISM_POTENTIAL,
        ),
        (
            GAP_BF_STRENGH,
            GAP_BF_STRENGH_BASE,
            GAP_BF_STRENGH_POTENTIAL,
        ),
        (GAP_BF_ATTACK, GAP_BF_ATTACK_BASE, GAP_BF_ATTACK_POTENTIAL),
        (GAP_BF_SPRITE, GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_POTENTIAL),
    ] {
        let base = goods.addon_property_value(factory, base_property, 1) as f32;
        let potential = goods.addon_property_value(factory, potential_property, 1) as f32;
        let modifier = goods.addon_property_value(factory, value_property, 2) as f32;
        // Машинный порядок суммы (тело `UpdateProperty`, шесть пар):
        // (potential + base*growth) + modifier до RC-guarded FISTP.
        let value = (potential + base * growth + modifier).trunc() as i32;
        let _ = goods.set_addon_property_value_core(value_property, 1, value);
    }
    let blast = goods
        .addon_property_value(factory, GAP_BF_BLAST_POTENTIAL, 1)
        .wrapping_add(goods.addon_property_value(factory, GAP_BF_BLAST, 2));
    let _ = goods.set_addon_property_value_core(GAP_BF_BLAST, 1, blast);
    let maximum_hp = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
    let maximum_mp = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
    let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, maximum_hp);
    let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, maximum_mp);
    let _ = goods.set_instance_addon_modifier(GAP_BF_CUT_HURT_SCALE, 1, 0);
}

/// `MountEquip` cases `0x9B/0x9C/0x9E..0xA1` после battle-fairy prelude.
/// Текущие BF-атрибуты хранятся в масштабе 1/10000; нулевое производное
/// значение восстанавливает base addon-ы, как native positive pass. Все
/// FISTP-преобразования используют truncate полной суммы с live property.
pub fn apply_battle_fairy_equipment_properties(
    equipment: &mut CEquipmentContainer,
    occupation: u8,
    mut properties: PlayerCombatProperties,
    coefficients: GlobePlayerPropertyCoefficients,
    factory: &CGoodsFactory,
) -> PlayerCombatProperties {
    fn add_u32(target: &mut u32, delta: i64) {
        *target = (i64::from(*target) + delta).clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_scaled_u32(target: &mut u32, delta: f64) {
        let value = (f64::from(*target) + delta).trunc() as i64;
        *target = value.clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_scaled_i32(target: &mut i32, delta: f64) {
        let value = (f64::from(*target) + delta).trunc() as i64;
        *target = value.clamp(0, i64::from(i32::MAX)) as i32;
    }
    fn add_scaled_u16(target: &mut u16, delta: f64) {
        let value = (f64::from(*target) + delta).trunc() as i64;
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn truncated_product(value: i32, coefficient: f32) -> i32 {
        (f64::from(value) * f64::from(coefficient)).trunc() as i32
    }
    fn scaled(value: i32) -> f64 {
        f64::from(value) * 0.0001_f64
    }

    let occupation = usize::from(occupation).min(2);
    let Some(goods) = equipment.get_goods_mut(10) else {
        return properties;
    };
    if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
        return properties;
    }
    let enabled = goods.enabled_addon_properties(factory);
    for property in enabled {
        match property {
            GAP_BF_ATTACK => {
                if goods.addon_property_value(factory, GAP_BF_ATTACK, 1) == 0 {
                    let base = goods.addon_property_value(factory, GAP_BF_ATTACK_BASE, 1);
                    let _ = goods.set_addon_property_value_core(GAP_BF_ATTACK, 1, base);
                }
            }
            GAP_BF_SPRITE => {
                if goods.addon_property_value(factory, GAP_BF_SPRITE, 1) == 0 {
                    let base = goods.addon_property_value(factory, GAP_BF_SPRITE_BASE, 1);
                    let _ = goods.set_addon_property_value_core(GAP_BF_SPRITE, 1, base);
                }
            }
            GAP_BF_BRAVE => {
                let base = goods.addon_property_value(factory, GAP_BF_BRAVE_BASE, 1);
                let current = goods.addon_property_value(factory, GAP_BF_BRAVE, 1);
                let base_effect =
                    truncated_product(base, coefficients.battle_fairy_brave_to_player);
                let current_effect =
                    truncated_product(current, coefficients.battle_fairy_brave_to_player);
                if current_effect == 0 {
                    let _ = goods.set_addon_property_value_core(GAP_BF_BRAVE, 1, base);
                    add_u32(&mut properties.strength, i64::from(base_effect));
                    continue;
                }
                if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                    continue;
                }
                let effect = scaled(current_effect);
                add_scaled_u32(&mut properties.strength, effect);
                // Производные в `MountEquip` (case 0x9E+): коэффициент
                // умножается на raw current_effect, масштаб 0.0001 — последним
                // (fld coef_f32; fmul qword ce; fmul qword [0x64DBD8]).
                add_scaled_u32(
                    &mut properties.maximum_attack,
                    f64::from(coefficients.str_to_max_attack[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
                add_scaled_u16(
                    &mut properties.burden,
                    f64::from(coefficients.str_to_burden[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
            }
            GAP_BF_AGILITY => {
                let base = goods.addon_property_value(factory, GAP_BF_AGILITY_BASE, 1);
                let current = goods.addon_property_value(factory, GAP_BF_AGILITY, 1);
                let base_effect =
                    truncated_product(base, coefficients.battle_fairy_agility_to_player);
                let current_effect =
                    truncated_product(current, coefficients.battle_fairy_agility_to_player);
                if current_effect == 0 {
                    let _ = goods.set_addon_property_value_core(GAP_BF_AGILITY, 1, base);
                    add_u32(&mut properties.dexterity, i64::from(base_effect));
                    continue;
                }
                if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                    continue;
                }
                let effect = scaled(current_effect);
                add_scaled_u32(&mut properties.dexterity, effect);
                add_scaled_u32(
                    &mut properties.minimum_attack,
                    f64::from(coefficients.dex_to_min_attack[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
                add_scaled_u16(
                    &mut properties.reank,
                    f64::from(coefficients.dex_to_stiff[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
            }
            GAP_BF_SPRITUALISM => {
                let base = goods.addon_property_value(factory, GAP_BF_SPRITUALISM_BASE, 1);
                let current = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
                let base_effect =
                    truncated_product(base, coefficients.battle_fairy_spiritualism_to_player);
                let current_effect =
                    truncated_product(current, coefficients.battle_fairy_spiritualism_to_player);
                if current_effect == 0 {
                    let _ = goods.set_addon_property_value_core(GAP_BF_SPRITUALISM, 1, base);
                    let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, base);
                    let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, base);
                    add_u32(&mut properties.intelligence, i64::from(base_effect));
                    continue;
                }
                if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                    continue;
                }
                let effect = scaled(current_effect);
                add_scaled_u32(&mut properties.intelligence, effect);
                add_scaled_i32(
                    &mut properties.element_modify,
                    f64::from(coefficients.int_to_element[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
                add_scaled_u32(
                    &mut properties.maximum_mp,
                    f64::from(coefficients.int_to_max_mp[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
                add_scaled_u32(
                    &mut properties.element_resistance,
                    f64::from(coefficients.int_to_resistant[occupation])
                        * f64::from(current_effect)
                        * 0.0001_f64,
                );
                clamp_battle_fairy_current(goods, factory, GAP_BF_MP, GAP_BF_MAX_MP);
            }
            GAP_BF_STRENGH => {
                let base = goods.addon_property_value(factory, GAP_BF_STRENGH_BASE, 1);
                let current = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
                let base_effect = truncated_product(base, coefficients.battle_fairy_strength_to_hp);
                let current_effect =
                    truncated_product(current, coefficients.battle_fairy_strength_to_hp);
                if current_effect == 0 {
                    let _ = goods.set_addon_property_value_core(GAP_BF_STRENGH, 1, base);
                    let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, base);
                    let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, base);
                    add_u32(&mut properties.maximum_hp, i64::from(base_effect));
                } else if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
                    add_scaled_u32(&mut properties.maximum_hp, scaled(current_effect));
                } else {
                    continue;
                }
                clamp_battle_fairy_current(goods, factory, GAP_BF_HP, GAP_BF_MAX_HP);
            }
            _ => {}
        }
    }
    properties
}

/// Канонический clamp BF current не выше maximum; hub-шов
/// `BattleFairyGearHost::clamp_headgear_current` зовёт его же.
pub fn clamp_battle_fairy_current(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    current_property: i32,
    maximum_property: i32,
) {
    let maximum = goods.addon_property_value(factory, maximum_property, 1);
    if maximum < goods.addon_property_value(factory, current_property, 1) {
        let _stored = goods.set_addon_property_value_core(current_property, 1, maximum);
    }
}

/// Два снимка `UpdateCiQingProperty` должны видеть один и тот же
/// pre-Mount BF goods state. Первый property-pass выполняется на clone,
/// второй оставляет canonical zero-init/clamp mutations в slot 10.
pub fn apply_battle_fairy_equipment_property_pair(
    equipment: &mut CEquipmentContainer,
    occupation: u8,
    previous: PlayerCombatProperties,
    current: PlayerCombatProperties,
    coefficients: GlobePlayerPropertyCoefficients,
    factory: &CGoodsFactory,
) -> (PlayerCombatProperties, PlayerCombatProperties) {
    let saved = equipment.get_goods(10).cloned();
    let previous = apply_battle_fairy_equipment_properties(
        equipment,
        occupation,
        previous,
        coefficients,
        factory,
    );
    if let (Some(saved), Some(goods)) = (saved, equipment.get_goods_mut(10)) {
        *goods = saved;
    }
    let current = apply_battle_fairy_equipment_properties(
        equipment,
        occupation,
        current,
        coefficients,
        factory,
    );
    (previous, current)
}

/// Полный `MountAllEquip` строит два снимка из одного BF goods state,
/// заменяет `m_mapCiQingAddValue` разностью и передаёт caller-у итог для
/// обязательного `SendResultToClient` до `OnChangeProperties`.
pub fn recompute_update_property(
    parts: &mut PlayerEquipmentPropertyParts<'_>,
    coefficients: GlobePlayerPropertyCoefficients,
    base_combat_scales: [f32; 5],
    critical_rate: f32,
    factory: &CGoodsFactory,
) -> PlayerPropertyRecompute {
    // Порядок присвоения флага — единственное известное отличие от машины
    // (`0x459B7B` ставит byte=1 после BF prelude, перед MountAllEquip);
    // наблюдаемо невидимо: флаг читается только DoneTaoZhuang-путём.
    *parts.equipment_changed = true;
    refresh_battle_fairy_equipment_properties(&mut *parts.equipment, factory);
    apply_ci_qing_base_properties(parts, factory);
    let previous = recompute_without_ci_qing_properties(
        &parts.snapshot(),
        coefficients,
        base_combat_scales,
        critical_rate,
        factory,
    );
    let current = recompute_base_and_equipment_properties(
        &parts.snapshot(),
        coefficients,
        base_combat_scales,
        critical_rate,
        factory,
    );
    let occupation = parts.base_properties.occupation;
    let (previous, current) = apply_battle_fairy_equipment_property_pair(
        &mut *parts.equipment,
        occupation,
        previous,
        current,
        coefficients,
        factory,
    );
    if let Some(values) = update_ci_qing_property_difference(
        &combat_type_values_from(previous),
        &combat_type_values_from(current),
    ) {
        *parts.ci_qing_add_values = values;
    }
    PlayerPropertyRecompute {
        properties: current,
        ci_qing_result_values: ci_qing_property_result(
            parts.ci_qing_add_values,
            parts.ci_qing_tao_zhuang_add_values,
        ),
    }
}

// --- apply_*_state формулы наложенных состояний поверх снимка. ---

/// `CNotDisappearAfterDead::OnUpdateProperties` применяет прямые поля до
/// базовых характеристик, а отрицательные STR/DEX/CON/INT проецирует как
/// разность производных старого и нового значения. IMUL сохраняет low32
/// до unsigned /100; каждый native setter ограничивает DWORD INT_MAX.
/// X87-преобразования усекают дробную часть; в отрицательных HP/MP/DEF-ветках сохраняется
/// исходное знаковое WORD-сужение. Поле usage `20_001` соответствует
/// `tagProperty.wHit +0x24`, а не соседнему `wAtcSpeed +0x32`.
pub fn apply_undead_state_properties(
    mut properties: PlayerCombatProperties,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: u8,
    state: &UndeadState,
) -> PlayerCombatProperties {
    fn clamp_wrapped_min_one(value: u32) -> u32 {
        if (value as i32) < 1 { 1 } else { value }
    }

    fn set_property(value: u32) -> u32 {
        value.min(i32::MAX as u32)
    }

    fn trunc_product(value: f64, coefficient: f32) -> i32 {
        truncate_original((value * f64::from(coefficient)).trunc())
    }

    fn ftol_word_product(value: f64, coefficient: f32) -> i16 {
        let value = (value * f64::from(coefficient)).trunc();
        // Только отрицательная INT→MP ветвь вызывает __ftol2 (FISTP64),
        // а затем использует AX. Integer-indefinite поэтому даёт WORD 0.
        if !value.is_finite()
            || value < -9_223_372_036_854_775_808.0
            || value >= 9_223_372_036_854_775_808.0
        {
            0
        } else {
            value as i64 as i16
        }
    }

    fn direct_value(current: u32, value: i16, percentage: bool) -> u32 {
        if value < 0 {
            let decrement = if percentage {
                (current.wrapping_mul(value.wrapping_neg() as i32 as u32) / 100) as i16
            } else {
                value.wrapping_neg()
            };
            let narrowed = (current as i16).wrapping_sub(decrement);
            if narrowed < 1 { 1 } else { narrowed as u32 }
        } else if value > 0 {
            let increment = if percentage {
                current.wrapping_mul(value as u32) / 100
            } else {
                value as u32
            };
            set_property(current.wrapping_add(increment))
        } else {
            current
        }
    }

    fn changed_stat(current: u32, value: i32, percentage: bool) -> (u32, u32, bool) {
        if value < 0 {
            let decrement = if percentage {
                current.wrapping_mul(value.wrapping_neg() as u32) / 100
            } else {
                value.wrapping_neg() as u32
            };
            let new = current.wrapping_sub(decrement);
            (clamp_wrapped_min_one(new), decrement, false)
        } else {
            let increment = if percentage {
                current.wrapping_mul(value as u32) / 100
            } else {
                value as u32
            };
            (
                set_property(current.wrapping_add(increment)),
                increment,
                true,
            )
        }
    }

    fn add_positive_derived(current: u32, delta: f64, coefficient: f32) -> u32 {
        set_property(current.wrapping_add(trunc_product(delta, coefficient) as u32))
    }

    fn add_derived_difference(current: u32, old: f64, new: f64, coefficient: f32) -> u32 {
        let difference =
            trunc_product(new, coefficient).wrapping_sub(trunc_product(old, coefficient));
        clamp_wrapped_min_one(current.wrapping_add(difference as u32))
    }

    fn add_short_derived_difference(current: u32, old: f64, new: f64, coefficient: f32) -> u32 {
        let difference = (trunc_product(new, coefficient) as i16)
            .wrapping_sub(trunc_product(old, coefficient) as i16);
        let narrowed = (current as i16).wrapping_add(difference);
        if narrowed < 1 { 1 } else { narrowed as u32 }
    }

    let occupation = usize::from(occupation).min(2);
    properties.maximum_hp = direct_value(properties.maximum_hp, state.maximum_hp, state.percentage);
    properties.maximum_mp = direct_value(properties.maximum_mp, state.maximum_mp, state.percentage);
    properties.defense = direct_value(properties.defense, state.defense, state.percentage);
    properties.element_resistance = direct_value(
        properties.element_resistance,
        state.element_resistance,
        state.percentage,
    );

    if state.strength != 0 {
        let old = properties.strength;
        let (new, delta, positive) = changed_stat(old, state.strength, state.percentage);
        properties.strength = new;
        properties.maximum_attack = if positive {
            add_positive_derived(
                properties.maximum_attack,
                f64::from(delta),
                coefficients.str_to_max_attack[occupation],
            )
        } else {
            add_derived_difference(
                properties.maximum_attack,
                f64::from(old),
                f64::from(new),
                coefficients.str_to_max_attack[occupation],
            )
        };
    }

    if state.dexterity != 0 {
        let old = properties.dexterity;
        let (new, delta, positive) = changed_stat(old, state.dexterity, state.percentage);
        properties.dexterity = new;
        properties.minimum_attack = if positive {
            add_positive_derived(
                properties.minimum_attack,
                f64::from(delta),
                coefficients.dex_to_min_attack[occupation],
            )
        } else {
            add_derived_difference(
                properties.minimum_attack,
                f64::from(old),
                f64::from(new),
                coefficients.dex_to_min_attack[occupation],
            )
        };
    }

    if state.constitution != 0 {
        let old = properties.constitution;
        let (new, delta, positive) = changed_stat(old, state.constitution, state.percentage);
        properties.constitution = new;
        if positive {
            let projected = if state.percentage {
                f64::from(delta as f32)
            } else {
                f64::from(delta)
            };
            properties.maximum_hp = add_positive_derived(
                properties.maximum_hp,
                projected,
                coefficients.con_to_max_hp[occupation],
            );
            properties.defense = add_positive_derived(
                properties.defense,
                projected,
                coefficients.con_to_defense[occupation],
            );
        } else {
            properties.maximum_hp = add_short_derived_difference(
                properties.maximum_hp,
                f64::from(old),
                f64::from(new),
                coefficients.con_to_max_hp[occupation],
            );
            properties.defense = add_short_derived_difference(
                properties.defense,
                f64::from(old as f32),
                f64::from(new as f32),
                coefficients.con_to_defense[occupation],
            );
        }
    }

    if state.intelligence != 0 {
        let old = properties.intelligence;
        let (new, delta, positive) = changed_stat(old, state.intelligence, state.percentage);
        properties.intelligence = new;
        if positive {
            // В абсолютной ветке оригинал повторно проецирует уже новое
            // полное INT; процентная ветка проецирует только приращение.
            let projected = if state.percentage {
                f64::from(delta as f32)
            } else {
                f64::from(new)
            };
            properties.maximum_mp = add_positive_derived(
                properties.maximum_mp,
                projected,
                coefficients.int_to_max_mp[occupation],
            );
            properties.element_resistance = add_positive_derived(
                properties.element_resistance,
                projected,
                coefficients.int_to_resistant[occupation],
            );
            properties.element_modify = properties.element_modify.wrapping_add(trunc_product(
                projected,
                coefficients.int_to_element[occupation],
            ));
        } else {
            let old_mp = ftol_word_product(f64::from(old), coefficients.int_to_max_mp[occupation]);
            let new_mp = ftol_word_product(
                f64::from(new as f32),
                coefficients.int_to_max_mp[occupation],
            );
            let maximum_mp =
                (properties.maximum_mp as i16).wrapping_add(new_mp.wrapping_sub(old_mp));
            properties.maximum_mp = maximum_mp.max(1) as u32;
            properties.element_resistance = add_derived_difference(
                properties.element_resistance,
                f64::from(old as f32),
                f64::from(new as f32),
                coefficients.int_to_resistant[occupation],
            );
            let difference = trunc_product(
                f64::from(new as f32),
                coefficients.int_to_element[occupation],
            )
            .wrapping_sub(trunc_product(
                f64::from(old as f32),
                coefficients.int_to_element[occupation],
            ));
            properties.element_modify = properties.element_modify.wrapping_add(difference);
            if properties.element_modify < 1 {
                properties.element_modify = 1;
            }
        }
    }

    properties.minimum_attack =
        direct_value(properties.minimum_attack, state.minimum_attack, false);
    properties.maximum_attack =
        direct_value(properties.maximum_attack, state.maximum_attack, false);
    if state.element_modify < 0 {
        let narrowed = (properties.element_modify as i16).wrapping_add(state.element_modify);
        properties.element_modify = if narrowed < 1 { 1 } else { i32::from(narrowed) };
    } else {
        properties.element_modify = properties
            .element_modify
            .wrapping_add(i32::from(state.element_modify));
    }
    properties.blast_attack = properties
        .blast_attack
        .wrapping_add(state.blast_attack as u16);
    properties.blast_element_attack = properties
        .blast_element_attack
        .wrapping_add(state.blast_element_attack as u16);
    properties.cch = properties.cch.wrapping_add(state.cch as u16);
    properties.full_miss = properties.full_miss.wrapping_add(state.full_miss as u16);
    properties.attack_avoid = properties
        .attack_avoid
        .wrapping_add(state.attack_avoid as u16);
    properties.element_avoid = properties
        .element_avoid
        .wrapping_add(state.element_avoid as u16);
    properties.hit = properties.hit.wrapping_add(state.hit as u16);
    properties.dodge = properties.dodge.wrapping_add(state.dodge as u16);
    properties
}

pub fn apply_extended_state_properties(
    mut properties: PlayerCombatProperties,
    state: &ExtendedState,
) -> PlayerCombatProperties {
    let add = |target: &mut u32, value: u16| {
        if value != 0 {
            *target = target.wrapping_add(u32::from(value)).min(i32::MAX as u32);
        }
    };
    add(&mut properties.maximum_hp, state.maximum_hp);
    add(&mut properties.maximum_mp, state.maximum_mp);
    add(&mut properties.minimum_attack, state.minimum_attack);
    add(&mut properties.maximum_attack, state.maximum_attack);
    add(&mut properties.defense, state.defense);
    add(&mut properties.element_resistance, state.element_resistance);
    properties.element_modify = properties
        .element_modify
        .wrapping_add(i32::from(state.element_modify));
    properties.cch = properties.cch.wrapping_add(state.cch);
    properties.full_miss = properties.full_miss.wrapping_add(state.full_miss);
    properties.attack_avoid = properties.attack_avoid.wrapping_add(state.attack_avoid);
    properties.element_avoid = properties.element_avoid.wrapping_add(state.element_avoid);
    properties.hit = properties.hit.wrapping_add(state.hit);
    properties.dodge = properties.dodge.wrapping_add(state.dodge);
    properties
}

pub fn apply_active_change_body_state_properties(
    mut properties: PlayerCombatProperties,
    state: &ChangeBodyState,
) -> PlayerCombatProperties {
    let add = |target: &mut u32, value: u32| {
        if value != 0 {
            *target = target.wrapping_add(value).min(i32::MAX as u32);
        }
    };
    add(&mut properties.maximum_hp, state.maximum_hp);
    add(&mut properties.maximum_mp, state.maximum_mp);
    add(&mut properties.minimum_attack, state.minimum_attack);
    add(&mut properties.maximum_attack, state.maximum_attack);
    add(&mut properties.defense, state.defense);
    add(&mut properties.element_resistance, state.element_resistance);
    if state.cch != 0 {
        properties.cch = properties.cch.wrapping_add(state.cch);
    }
    if state.blast_attack != 0 {
        properties.blast_attack = properties.blast_attack.wrapping_add(state.blast_attack);
    }
    if state.blast_element_attack != 0 {
        properties.blast_element_attack = properties
            .blast_element_attack
            .wrapping_add(state.blast_element_attack);
    }
    properties
}

/// CRideState::OnUpdateProperties0x004F9000 выбирает первый packet goods
/// по имени именно этого экземпляра, не по AI-cache. Общий расчёт делает
/// MountEquipRide(true), затем(false), не копируя state или CGoods.
pub fn apply_ride_state_properties(
    mut properties: PlayerCombatProperties,
    state: &RideState,
    coefficients: GlobePlayerPropertyCoefficients,
    goods_factory: &CGoodsFactory,
    packet: &CVolumeLimitGoodsContainer,
    occupation: u8,
) -> PlayerCombatProperties {
    if let Some(goods) = ride_goods(packet, state, goods_factory) {
        apply_equipment_goods_properties(
            &mut properties,
            goods,
            goods_factory,
            coefficients,
            usize::from(occupation).min(2),
            false,
            None,
        );
    }
    properties
}

fn ride_goods<'a>(
    packet: &'a CVolumeLimitGoodsContainer,
    state: &RideState,
    factory: &CGoodsFactory,
) -> Option<&'a CGoods> {
    let base_index = factory.query_goods_id_by_original_name(Some(state.goods_name()));
    packet
        .base()
        .traversing_goods()
        .find(|goods| goods.base_properties_index() == base_index)
}

// --- Commit вычисленного снимка: tagProperty wire и equipment flash. ---

/// Применяет результат виртуального `UpdateProperty` и синхронизирует
/// подтверждённые поля 0x9c-byte `tagProperty`, сохраняя неизвестные байты.
pub fn apply_recomputed_combat_properties(
    parts: &mut PlayerCombatWireParts<'_>,
    properties: PlayerCombatProperties,
    goods_factory: &CGoodsFactory,
) {
    *parts.combat_properties = properties;
    sync_combat_property_wire(parts.combat_properties, parts.combat_property_wire);
    refresh_equipment_flash(
        parts.equipment,
        parts.flash_current,
        parts.flash_changed,
        goods_factory,
    );
}

/// Чистая проекция полей живого снимка в подтверждённые offsets `tagProperty`;
/// остальные байты wire сохраняют прежнее значение.
pub fn sync_combat_property_wire(
    properties: &PlayerCombatProperties,
    wire: &mut [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
) {
    let write_u16 = |wire: &mut [u8], offset: usize, value: u16| {
        LegacyWriter::write_u16_at(wire, offset, value)
            .expect("combat wire offset проверен layout-константой");
    };
    let write_u32 = |wire: &mut [u8], offset: usize, value: u32| {
        LegacyWriter::write_u32_at(wire, offset, value)
            .expect("combat wire offset проверен layout-константой");
    };
    write_u32(wire, 0x00, properties.maximum_hp);
    write_u32(wire, 0x04, properties.maximum_mp);
    write_u16(wire, 0x08, properties.maximum_yp);
    write_u16(wire, 0x0a, properties.maximum_rp);
    write_u32(wire, 0x0c, properties.strength);
    write_u32(wire, 0x10, properties.dexterity);
    write_u32(wire, 0x14, properties.constitution);
    write_u32(wire, 0x18, properties.intelligence);
    write_u32(wire, 0x1c, properties.minimum_attack);
    write_u32(wire, 0x20, properties.maximum_attack);
    write_u16(wire, 0x32, properties.attack_speed);
    write_u16(wire, 0x24, properties.hit);
    write_u16(wire, 0x30, properties.dodge);
    write_u16(wire, 0x28, properties.cch);
    write_u16(wire, 0x26, properties.burden);
    write_u32(wire, 0x2c, properties.defense);
    write_u32(wire, 0x34, properties.element_resistance);
    write_u16(wire, 0x3c, properties.soul_resistance);
    write_u16(wire, 0x44, properties.add_soul_attack);
    write_u32(wire, 0x58, properties.blast_attack_scale_bits);
    write_u32(wire, 0x40, properties.add_element_attack);
    write_u16(wire, 0x38, properties.hp_recovery);
    write_u16(wire, 0x3a, properties.mp_recovery);
    write_u32(wire, 0x48, properties.element_modify as u32);
    write_u16(wire, 0x4c, properties.reank);
    write_u16(wire, 0x4e, properties.attack_avoid);
    write_u16(wire, 0x50, properties.element_avoid);
    write_u16(wire, 0x52, properties.full_miss);
    write_u16(wire, 0x54, properties.blast_attack);
    write_u16(wire, 0x56, properties.blast_element_attack);
    write_u32(wire, 0x5c, properties.blast_defense_scale_bits);
    write_u32(wire, 0x60, properties.element_blast_attack_scale_bits);
    write_u32(wire, 0x64, properties.element_blast_defense_scale_bits);
    write_u32(wire, 0x68, properties.full_miss_scale_bits);
    write_u32(wire, 0x6c, properties.critical_rate_bits);
    write_u32(wire, 0x70, properties.resume_hp_peace as u32);
    write_u32(wire, 0x74, properties.resume_mp_peace as u32);
    write_u32(wire, 0x78, properties.resume_hp_fight as u32);
    write_u32(wire, 0x7c, properties.resume_mp_fight as u32);
    write_u32(wire, 0x80, properties.restored_hp_peace as u32);
    write_u32(wire, 0x84, properties.restored_mp_peace as u32);
    write_u32(wire, 0x88, properties.restored_hp_fight as u32);
    write_u32(wire, 0x8c, properties.restored_mp_fight as u32);
    wire[0x90] = u8::from(properties.battle_fairy_summoned);
    wire[0x91] = u8::from(properties.battle_fairy_recall);
    wire[0x92] = u8::from(properties.battle_fairy_died);
}

/// `MountAllEquip -> SetCurFlash`: пересобирает 17 flash-ячеек после
/// каждого полного property commit. Исторический system clock заменён
/// `SystemTime`; damaged equipment намеренно сохраняет прежнее значение,
/// потому что original loop пропускает `SetCurFlash` для нулевой прочности.
fn refresh_equipment_flash(
    equipment: &CEquipmentContainer,
    flash_current: &mut [u32; 17],
    flash_changed: &mut bool,
    factory: &CGoodsFactory,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as u32);
    for position in 0..17usize {
        let Some(goods) = equipment.get_goods(position as u32) else {
            flash_current[position] = 0;
            *flash_changed = true;
            continue;
        };
        if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
            && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
        {
            continue;
        }
        let flash = if goods.query_attribute(GAP_GOODS_LIFE_TYPE) {
            match goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 2) {
                1 => {
                    let expiry = goods
                        .start_point(factory)
                        .wrapping_add(u64::from(goods.goods_lifetime(factory)));
                    if (expiry >> 32) as i32 == 0 && expiry as u32 <= now {
                        0
                    } else {
                        goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
                    }
                }
                2 => goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32,
                _ => continue,
            }
        } else {
            goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
        };
        flash_current[position] = flash;
        *flash_changed = true;
    }
}

/// `DoneFlash` забирает один pending snapshot и одновременно продвигает
/// previous-table. Повторный tick без нового `MountAllEquip` ничего не шлёт.
pub fn take_flash_update(
    flash_previous: &mut [u32; 17],
    flash_current: &mut [u32; 17],
    flash_changed: &mut bool,
) -> Option<[(u32, u32); 17]> {
    if !*flash_changed {
        return None;
    }
    let pairs = std::array::from_fn(|position| (flash_previous[position], flash_current[position]));
    *flash_previous = *flash_current;
    *flash_changed = false;
    Some(pairs)
}

// --- Client allocation 0x8FA01 и diagnostic projection. ---

pub const fn player_stat_allocation_state(
    base: &PlayerBaseProperties,
) -> PlayerStatAllocationState {
    PlayerStatAllocationState {
        sex: base.sex,
        occupation: base.occupation,
        remain_point: base.remain_point,
        base_maximum_hp: base.base_maximum_hp,
        base_maximum_mp: base.base_maximum_mp,
        base_strength: base.base_strength,
        base_dexterity: base.base_dexterity,
        base_constitution: base.base_constitution,
        base_intelligence: base.base_intelligence,
    }
}

/// Восстанавливает owned поля `m_BaseProperty`, участвующие в client
/// allocation `0x8FA01`; полный decoder игрока остаётся отдельным owner-ом.
pub const fn restore_player_stat_allocation_state(
    base: &mut PlayerBaseProperties,
    state: PlayerStatAllocationState,
) {
    base.sex = state.sex;
    base.occupation = state.occupation;
    base.remain_point = state.remain_point;
    base.base_maximum_hp = state.base_maximum_hp;
    base.base_maximum_mp = state.base_maximum_mp;
    base.base_strength = state.base_strength;
    base.base_dexterity = state.base_dexterity;
    base.base_constitution = state.base_constitution;
    base.base_intelligence = state.base_intelligence;
}

/// Exact mutation-tail `0x8FA01`: DEX/CON/INT используют legacy STR gate;
/// неизвестный selector всё равно расходует одно очко и ведёт к recompute.
pub fn allocate_stat_point(
    base: &mut PlayerBaseProperties,
    player_id: i32,
    selector: u8,
    constitution_hp: u16,
    intelligence_mp: u16,
) -> Option<PlayerStatAllocationMutation> {
    if base.remain_point == 0 {
        return None;
    }
    let previous = player_stat_allocation_state(base);
    let strength_gate = base.base_strength < i32::MAX as u32;
    let stat_changed = match selector {
        0 if strength_gate => {
            base.base_strength = base.base_strength.wrapping_add(1);
            true
        }
        1 if strength_gate => {
            base.base_dexterity = base.base_dexterity.wrapping_add(1);
            true
        }
        2 => {
            if strength_gate {
                base.base_constitution = base.base_constitution.wrapping_add(1);
            }
            base.base_maximum_hp = base
                .base_maximum_hp
                .wrapping_add(u32::from(constitution_hp));
            strength_gate
        }
        3 => {
            if strength_gate {
                base.base_intelligence = base.base_intelligence.wrapping_add(1);
            }
            base.base_maximum_mp = base
                .base_maximum_mp
                .wrapping_add(u32::from(intelligence_mp));
            strength_gate
        }
        _ => false,
    };
    base.remain_point = base.remain_point.wrapping_sub(1);
    Some(PlayerStatAllocationMutation {
        player_id,
        selector,
        stat_changed,
        previous,
        current: player_stat_allocation_state(base),
    })
}

/// Selector `3009` читает не производную Rust-модель, а те же concrete
/// `tagBaseProperty`/`tagProperty` slots, которые использует EXE. Поэтому
/// неизвестные, но загруженные и пересчитанные поля сохраняются в выводе.
/// `base_wire` — уже synchronized снимок hub-а.
pub fn player_all_properties_diagnostic_snapshot(
    name: &[u8],
    occupation: u8,
    level: u8,
    experience: u32,
    money: u32,
    base_wire: &[u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    combat_wire: &[u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
) -> PlayerAllPropertiesDiagnosticSnapshot {
    let base = base_wire;
    let current = combat_wire;
    let base_u16 = |offset| u32::from(read_player_wire_u16(base, offset));
    let base_u32 = |offset| read_player_wire_u32(base, offset);
    let current_u16 = |offset| u32::from(read_player_wire_u16(current, offset));
    let current_u32 = |offset| read_player_wire_u32(current, offset);
    PlayerAllPropertiesDiagnosticSnapshot {
        name: name.to_vec(),
        summary_words: [
            u32::from(occupation),
            u32::from(level),
            experience,
            base_u32(BASE_HEALTH_OFFSET),
            current_u32(0x00),
            base_u32(BASE_MANA_OFFSET),
            current_u32(0x04),
            base_u16(0xac),
            current_u16(0x0a),
            base_u32(BASE_MAXIMUM_HP_OFFSET),
            base_u32(BASE_MAXIMUM_MP_OFFSET),
            base_u16(0xba),
            base_u16(BASE_PK_COUNT_OFFSET),
            base_u32(BASE_KILL_COUNT_OFFSET),
            money,
        ],
        base_combat_words: [
            base_u32(BASE_STRENGTH_OFFSET),
            base_u32(BASE_DEXTERITY_OFFSET),
            base_u32(BASE_CONSTITUTION_OFFSET),
            base_u32(BASE_INTELLIGENCE_OFFSET),
            base_u32(0xcc),
            base_u32(0xd0),
            base_u16(0xd4),
            base_u16(0xd6),
            base_u16(0xd8),
            base_u32(0xdc),
            base_u16(0xe0),
            base_u16(0xe2),
            base_u32(0xe4),
            base_u16(0xe8),
            base_u16(0xea),
        ],
        current_combat_words: [
            current_u32(0x0c),
            current_u32(0x10),
            current_u32(0x14),
            current_u32(0x18),
            current_u32(0x1c),
            current_u32(0x20),
            current_u16(0x24),
            current_u16(0x26),
            current_u16(0x28),
            current_u32(0x2c),
            current_u16(0x30),
            i32::from(read_player_wire_u16(current, 0x32) as i16) as u32,
            current_u32(0x34),
            current_u16(0x38),
            current_u16(0x3a),
            current_u16(0x3c),
            current_u32(0x40),
            current_u16(0x44),
            current_u32(0x48),
            current_u16(0x4c),
        ],
    }
}

// --- Серверный домен GetCurrentTypeValue и кадр 0xC010E. ---

/// Серверный домен `GetCurrentTypeValue` — ровно 15 ключей `2..0x10`:
/// 2/3/4/5 = strength/dexterity/constitution/intelligence, 6/7 = min/max
/// attack, 8 = element_modify, 9 = cch, 0xa = defense,
/// 0xb = element_resistance, 0xc/0xd = max HP/MP, 0xe/0xf = blast attack /
/// blast element attack, 0x10 = full_miss. Ключи в провод не уходят:
/// клиент (CiQing-ветвь `0x53990E`) читает 15 позиционных DWORD после
/// `(type, id)`, порядок задаётся возрастанием ключей BTreeMap. Соседний
/// домен `0x0e..0x60` живёт отдельно в `apply_tao_zhuang_properties`
/// (серверный `AddPreItemToPlayer`) и здесь не используется.
pub fn combat_type_values_from(properties: PlayerCombatProperties) -> BTreeMap<u32, u32> {
    BTreeMap::from([
        (0x02, properties.strength),
        (0x03, properties.dexterity),
        (0x04, properties.constitution),
        (0x05, properties.intelligence),
        (0x06, properties.minimum_attack),
        (0x07, properties.maximum_attack),
        (0x08, properties.element_modify as u32),
        (0x09, u32::from(properties.cch)),
        (0x0a, properties.defense),
        (0x0b, properties.element_resistance),
        (0x0c, properties.maximum_hp),
        (0x0d, properties.maximum_mp),
        (0x0e, u32::from(properties.blast_attack)),
        (0x0f, u32::from(properties.blast_element_attack)),
        (0x10, u32::from(properties.full_miss)),
    ])
}

pub fn update_ci_qing_property_difference(
    previous: &BTreeMap<u32, u32>,
    current: &BTreeMap<u32, u32>,
) -> Option<BTreeMap<u32, u32>> {
    if previous.len() != current.len() {
        return None;
    }
    let mut destination = BTreeMap::new();
    for ((_, &previous), (&property, &current)) in previous.iter().zip(current) {
        destination.insert(property, current.saturating_sub(previous));
    }
    Some(destination)
}

pub fn ci_qing_property_result(
    add_values: &BTreeMap<u32, u32>,
    tao_zhuang_add_values: &BTreeMap<u32, u32>,
) -> BTreeMap<u32, u32> {
    let mut result = add_values.clone();
    for (&property, &value) in tao_zhuang_add_values {
        let current = result.entry(property).or_default();
        *current = current.wrapping_add(value);
    }
    result
}
