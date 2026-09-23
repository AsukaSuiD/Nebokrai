//! Живое применение CBloodLossState в переходном Game.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! `appserver/skills/bloodlossstate.cpp` и `bloodlossstate.h`.
//! Данные и кодек перенесены в `zone/effects/bloodloss.rs`.
//! `CalculateAttackPower` VA `0x005E3E30` использует игровой RNG и текущую цель.
//! Vtable `+0x1C` ведёт к End VA `0x005FD420`: visual, свежий S, RemoveState.
//! Две перегрузки Begin VA `0x005E39E0/0x005E3A80` целиком не проверены.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::StateKey;
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

pub(crate) use crate::gameserver::appserver::states::periodicattack::begin_primary_periodic_attack_state as begin_primary_blood_loss_state;
pub(crate) use nebokrai_zone::effects::{
    BLOOD_LOSS_STATE_BYTES, BloodLossAttackSeed, BloodLossState,
};

impl PeriodicAttackState for BloodLossState {
    type AttackSeed = BloodLossAttackSeed;

    fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        BloodLossState::core_mut(self)
    }
    fn attack_seed(&self) -> Self::AttackSeed {
        BloodLossState::attack_seed(self)
    }
}

fn calculate_blood_loss_attack(
    game: &mut CGame,
    (_target_region, target): (i32, ShapeIdentity),
    master: MasterInfo,
    seed: BloodLossAttackSeed,
) -> AttackInformation {
    let minimum = i32::from(seed.minimum_attack);
    let maximum = i32::from(seed.maximum_attack);
    let span = minimum.abs_diff(maximum).wrapping_add(1) as i32;
    let rolled = game.skill_random_below(span);
    let mut damage = truncate_original_i64_low(
        f64::from(rolled) + f64::from(seed.damage_modifier) + f64::from(minimum),
    )
    .max(0);
    let critical_chance = if target.object_type == 400 {
        game.find_player(target.id)
            .map_or(0, |player| player.combat_properties().cch)
    } else {
        0
    };
    let critical = game.skill_random_below(100) < i32::from(critical_chance);
    if critical {
        damage =
            truncate_original(f64::from(damage) * f64::from(game.globe_setup().critical_rate()));
    }
    periodic_attack_information(
        master,
        seed.damage_factor,
        critical,
        AttackPower {
            kind: AttackPowerType::Physical,
            hp_damage: damage,
            mp_damage: 0,
        },
    )
}

pub(crate) fn update_blood_loss_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    update_periodic_attack_state::<BloodLossState, Runtime>(
        game,
        region_id,
        holder,
        key,
        runtime,
        calculate_blood_loss_attack,
    )
}
