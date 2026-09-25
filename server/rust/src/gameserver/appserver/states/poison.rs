//! Применение ядовитого периодического состояния к живой фигуре Game.
//! Данные и кодек: `zone/effects/poison.rs`.

use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::periodicattack::{
    PeriodicAttackCore, PeriodicAttackState, periodic_attack_information,
    update_periodic_attack_state,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use crate::gameserver::appserver::states::periodicattack::begin_primary_periodic_attack_state as begin_primary_poison_state;
pub(crate) use nebokrai_zone::effects::PoisonState;

impl<const ID: u32> PeriodicAttackState for PoisonState<ID>
where
    Self: AppliedState,
{
    // SpriteBurn может ударить и после истечения срока: проверка времени
    // следует за попыткой атаки, даже когда частота пропускает удар.
    const CHECK_LIFETIME_AFTER_ATTACK: bool = ID == 0x1a6;
    type AttackSeed = u32;

    fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        PoisonState::core_mut(self)
    }
    fn attack_seed(&self) -> u32 {
        self.hp_loss()
    }
}

pub(crate) fn update_poison_state<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool
where
    PoisonState<ID>: AppliedState,
{
    update_periodic_attack_state::<PoisonState<ID>, Runtime>(
        game,
        region_id,
        holder,
        key,
        runtime,
        |_game, _target, master, hp_loss| {
            periodic_attack_information(
                master,
                1.0,
                false,
                AttackPower {
                    kind: AttackPowerType::Poison,
                    // Только PoisonArrow ограничивает отрицательный signed HP.
                    hp_damage: if ID == 0x21e {
                        (hp_loss as i32).max(0)
                    } else {
                        hp_loss as i32
                    },
                    mp_damage: 0,
                },
            )
        },
    )
}
