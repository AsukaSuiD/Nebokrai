//! CBloodLossState (0x21D), gameserver.exe/GameServer.pdb,
//! исходный owner appserver/skills/bloodlossstate.cpp.
//! Ctor 0x005E3820 задаёт start/count0 без часов; default 0x005E38E0 обнуляет
//! остальные поля. Begin 0x005E3BD0 и AI 0x005E3F90 разделяют periodicattack:
//! условный base clock → actual U/S → loop1 visual → count reset → регистрация;
//! clock → absolute deadline → actual S/IsDied → count → clock → count++ → удар.
//! Vtable 0x0065F2AC: property=true, S-only SetRegion 0x005E3B30;
//! End 0x005FD420: optional visual1 → свежий S → RemoveState без state.ended.
//! Save 0x005E3B40 чистый, Load 0x005E3C70 читает Master40 до clock.
//! Запись ровно 64 байта: ID/Master40/remaining/frequency/два float-бита/
//! min WORD/max WORD. Старый размер 68 не соответствовал EXE.
//! CalculateAttackPower 0x005E3E30: RNG(abs(min-max)+1) → modifier/min →
//! __ftol2 low DWORD → clamp0 → actual S.GetCCH → RNG100 → critical rate.
//! GetCCH player 0x0044A380 читает WORD; базовый 0x004E69B0 возвращает 0.
//! Критический множитель читается только при успешном броске; FISTP dword
//! сохраняет отдельную overflow-семантику. Подтверждённая x87-арифметика
//! выражена через f64 и общие truncate helpers без промежуточного f32.
//! Payload один в общей арене; независимый seed содержит лишь входы формулы.
//! Неустановленные координатные/typed Begin callers остаются RAW ниже.

use super::bloodloss::BLOOD_LOSS_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::StateKey;
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
    as begin_primary_blood_loss_state;

pub(crate) const BLOOD_LOSS_STATE_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BloodLossState {
    core: PeriodicAttackCore,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
}

pub(crate) struct BloodLossAttackSeed {
    damage_factor: f32,
    damage_modifier: f32,
    minimum_attack: u16,
    maximum_attack: u16,
}

impl BloodLossState {
    pub(crate) const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
    ) -> Self {
        Self {
            core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms),
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
        }
    }

    pub(crate) const fn skill_id(&self) -> u32 { BLOOD_LOSS_SKILL_ID }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (core, mut reader) = PeriodicAttackCore::decode(payload, offset, BLOOD_LOSS_SKILL_ID, now)?;
        Ok(Self {
            core,
            damage_factor_bits: reader.read_u32()?,
            damage_modifier_bits: reader.read_u32()?,
            minimum_attack: reader.read_u16()?,
            maximum_attack: reader.read_u16()?,
        })
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        encode_periodic_state(self, now).try_into().expect("запись потери крови содержит 64 байта")
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        encode_periodic_state_for_install(self).try_into().expect("запись потери крови содержит 64 байта")
    }
}

impl PeriodicAttackState for BloodLossState {
    const STATE_ID: u32 = BLOOD_LOSS_SKILL_ID;
    const RECORD_BYTES: usize = BLOOD_LOSS_STATE_BYTES;
    type AttackSeed = BloodLossAttackSeed;

    fn core(&self) -> &PeriodicAttackCore { &self.core }
    fn core_mut(&mut self) -> &mut PeriodicAttackCore { &mut self.core }

    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) {
        writer.write_u32(self.damage_factor_bits);
        writer.write_u32(self.damage_modifier_bits);
        writer.write_u16(self.minimum_attack);
        writer.write_u16(self.maximum_attack);
    }

    fn attack_seed(&self) -> Self::AttackSeed {
        BloodLossAttackSeed {
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: f32::from_bits(self.damage_modifier_bits),
            minimum_attack: self.minimum_attack,
            maximum_attack: self.maximum_attack,
        }
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
    ).max(0);
    let critical_chance = if target.object_type == 400 {
        game.find_player(target.id).map_or(0, |player| player.combat_properties().cch)
    } else {
        0
    };
    let critical = game.skill_random_below(100) < i32::from(critical_chance);
    if critical {
        damage = truncate_original(f64::from(damage) * f64::from(game.globe_setup().critical_rate()));
    }
    periodic_attack_information(master, seed.damage_factor, critical, AttackPower {
        kind: AttackPowerType::Physical,
        hp_damage: damage,
        mp_damage: 0,
    })
}

pub(crate) fn update_blood_loss_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    update_periodic_attack_state::<BloodLossState, Runtime>(
        game, region_id, holder, key, runtime, calculate_blood_loss_attack,
    )
}

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:75
// RVA: 0x001E39E0
// ADDRESS: 005e39e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:89
// RVA: 0x001E3A80
// ADDRESS: 005e3a80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
