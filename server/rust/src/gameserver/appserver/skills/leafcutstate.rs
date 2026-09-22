//! Периодический физический, стихийный и душевный урон трёх рассечений.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcutstate.cpp.
//!
//! Варианты отличаются ID и остаются независимыми объектами общей арены.
//! Хвост 68-байтной записи содержит два float-битовых DWORD и четыре WORD;
//! префикс, Begin, End, перезапуск и таймеры принадлежат periodicattack.
//! Параметры урона фиксируются при наложении, критический шанс берётся у
//! фактического Sufferer при ударе. Состояние не извлекается ради вызова AI.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::periodicattack::{
    PeriodicAttackCore, PeriodicAttackState, encode_periodic_state,
    encode_periodic_state_for_install, periodic_attack_information, update_periodic_attack_state,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use crate::gameserver::appserver::states::periodicattack::begin_primary_periodic_attack_state
    as begin_primary_leaf_cut_state;

pub(crate) const LEAF_CUT_STATE_ID: u32 = 0x6b;
pub(crate) const LEAF_CUT_STATE_BYTES: usize = 68;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState<const ID: u32 = LEAF_CUT_STATE_ID> {
    core: PeriodicAttackCore,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
    element_attack: u16,
    soul_attack: u16,
}

pub(crate) struct LeafCutAttackSeed {
    damage_factor: f32,
    damage_modifier: f32,
    minimum_attack: u16,
    maximum_attack: u16,
    element_attack: u16,
    soul_attack: u16,
}

impl<const ID: u32> LeafCutState<ID> {
    #[allow(clippy::too_many_arguments, reason = "параметры снимка атаки сохраняют контракт состояния")]
    pub(crate) const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
        element_attack: u16,
        soul_attack: u16,
    ) -> Self {
        Self {
            core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms),
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
            element_attack,
            soul_attack,
        }
    }

    pub(crate) const fn skill_id(&self) -> u32 { ID }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (core, mut reader) = PeriodicAttackCore::decode(payload, offset, ID, now)?;
        Ok(Self {
            core,
            damage_factor_bits: reader.read_u32()?,
            damage_modifier_bits: reader.read_u32()?,
            minimum_attack: reader.read_u16()?,
            maximum_attack: reader.read_u16()?,
            element_attack: reader.read_u16()?,
            soul_attack: reader.read_u16()?,
        })
    }
}

impl<const ID: u32> LeafCutState<ID>
where Self: AppliedState,
{
    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; LEAF_CUT_STATE_BYTES] {
        encode_periodic_state(self, now).try_into().expect("запись рассечения содержит 68 байт")
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; LEAF_CUT_STATE_BYTES] {
        encode_periodic_state_for_install(self).try_into().expect("запись рассечения содержит 68 байт")
    }
}

impl<const ID: u32> PeriodicAttackState for LeafCutState<ID>
where Self: AppliedState,
{
    const STATE_ID: u32 = ID;
    const RECORD_BYTES: usize = LEAF_CUT_STATE_BYTES;
    type AttackSeed = LeafCutAttackSeed;

    fn core(&self) -> &PeriodicAttackCore { &self.core }
    fn core_mut(&mut self) -> &mut PeriodicAttackCore { &mut self.core }

    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) {
        writer.write_u32(self.damage_factor_bits);
        writer.write_u32(self.damage_modifier_bits);
        writer.write_u16(self.minimum_attack);
        writer.write_u16(self.maximum_attack);
        writer.write_u16(self.element_attack);
        writer.write_u16(self.soul_attack);
    }

    fn attack_seed(&self) -> Self::AttackSeed {
        LeafCutAttackSeed {
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: f32::from_bits(self.damage_modifier_bits),
            minimum_attack: self.minimum_attack,
            maximum_attack: self.maximum_attack,
            element_attack: self.element_attack,
            soul_attack: self.soul_attack,
        }
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
    ).max(0);
    let mut attack = periodic_attack_information(master, seed.damage_factor, false, AttackPower {
        kind: AttackPowerType::Physical,
        hp_damage: physical,
        mp_damage: 0,
    });
    attack.damages.extend([
        AttackPower { kind: AttackPowerType::Element, hp_damage: i32::from(seed.element_attack), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(seed.soul_attack), mp_damage: 0 },
    ]);
    let critical_chance = if target.object_type == 400 {
        game.find_player(target.id).map_or(0, |player| player.combat_properties().cch)
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
where LeafCutState<ID>: AppliedState,
{
    update_periodic_attack_state::<LeafCutState<ID>, Runtime>(
        game, region_id, holder, key, runtime, calculate_leaf_cut_attack,
    )
}

// Для координатного Begin (0x005FC690) и typed Begin (0x005FC730)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
