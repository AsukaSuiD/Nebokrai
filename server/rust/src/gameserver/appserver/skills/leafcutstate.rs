//! Применение периодических ударов LeafCut к живой цели Game.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! `appserver/skills/leafcutstate.cpp` и `leafcutstate.h`.
//! Данные трёх состояний и кодек перенесены в `zone/effects/leafcut.rs`.
//! Расчёт с общим RNG, критическим шансом цели и три компонента урона
//! остаются здесь вместе с применением к фигуре.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::periodicattack::{
    PeriodicAttackCore, PeriodicAttackState, periodic_attack_information,
    update_periodic_attack_state,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use crate::gameserver::appserver::states::periodicattack::begin_primary_periodic_attack_state as begin_primary_leaf_cut_state;
pub(crate) use nebokrai_zone::effects::{
    LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID, LeafCutAttackSeed, LeafCutState,
};

impl<const ID: u32> PeriodicAttackState for LeafCutState<ID>
where
    Self: AppliedState,
{
    type AttackSeed = LeafCutAttackSeed;

    fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        LeafCutState::core_mut(self)
    }
    fn attack_seed(&self) -> Self::AttackSeed {
        LeafCutState::attack_seed(self)
    }
}
fn calculate_leaf_cut_attack(
    game: &mut CGame,
    (_target_region, target): (i32, ShapeIdentity),
    master: MasterInfo,
    seed: LeafCutAttackSeed,
) -> AttackInformation {
    let minimum = i32::from(seed.minimum_attack);
    let maximum = i32::from(seed.maximum_attack);
    let span = minimum.abs_diff(maximum).wrapping_add(1) as i32;
    let rolled = game.skill_random_below(span);
    // В x87-цепочке нет промежуточного f32: __ftol2 отдаёт младший DWORD,
    // и только затем его знаковое значение ограничивается снизу нулём.
    let physical = truncate_original_i64_low(
        f64::from(rolled) + f64::from(seed.damage_modifier) + f64::from(minimum),
    )
    .max(0);
    let mut attack = periodic_attack_information(
        master,
        seed.damage_factor,
        false,
        AttackPower {
            kind: AttackPowerType::Physical,
            hp_damage: physical,
            mp_damage: 0,
        },
    );
    attack.damages.extend([
        AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: i32::from(seed.element_attack),
            mp_damage: 0,
        },
        AttackPower {
            kind: AttackPowerType::Soul,
            hp_damage: i32::from(seed.soul_attack),
            mp_damage: 0,
        },
    ]);
    let critical_chance = if target.object_type == 400 {
        game.find_player(target.id)
            .map_or(0, |player| player.combat_properties().cch)
    } else {
        0
    };
    // Второй бросок выполняется и при нулевом шансе. Он относится к цели,
    // а не к MasterInfo; критический множитель читается только после успеха.
    attack.critical = game.skill_random_below(100) < i32::from(critical_chance);
    if attack.critical {
        for damage in &mut attack.damages {
            // Критический пересчёт использует FISTP dword, а не __ftol2;
            // повторного ограничения отрицательного результата здесь нет.
            damage.hp_damage = truncate_original(
                f64::from(damage.hp_damage) * f64::from(game.globe_setup().critical_rate()),
            );
        }
    }
    attack
}

pub(crate) fn update_leaf_cut_state<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool
where
    LeafCutState<ID>: AppliedState,
{
    update_periodic_attack_state::<LeafCutState<ID>, Runtime>(
        game,
        region_id,
        holder,
        key,
        runtime,
        calculate_leaf_cut_attack,
    )
}

// Для координатного Begin (0x005FC690) и typed Begin (0x005FC730)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
