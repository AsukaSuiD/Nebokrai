//! CPoisonArrowState (0x21E) и CSpiderPoisonState (0x191),
//! GameServer.exe/GameServer.pdb, owners skills/poisonarrowstate.cpp и
//! skills/spiderpoisonstate.cpp. Общие Begin/AI/End/codec-prefix делегированы
//! periodicattack.rs; типовые ID сохраняют два существующих варианта арены.
//! Хвост Save56 — один DWORD HP. CalculateAttackPower различается:
//! Arrow 0x005E3690 обнуляет отрицательный signed HP, Spider 0x005E9610
//! передаёт его без ограничения. Обе атаки имеют тип Poison и MP0.
//! Конкретные адреса lifecycle и неизвестные overload — в owner-файлах.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::periodicattack::{
    PeriodicAttackCore, PeriodicAttackState, encode_periodic_state,
    encode_periodic_state_for_install, periodic_attack_information, update_periodic_attack_state,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use crate::gameserver::appserver::states::periodicattack::begin_primary_periodic_attack_state
    as begin_primary_poison_state;

pub(crate) const POISON_STATE_BYTES: usize = 56;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PoisonState<const ID: u32> {
    core: PeriodicAttackCore,
    hp_loss: u32,
}

impl<const ID: u32> PoisonState<ID> {
    pub(crate) const fn new(
        master: MasterInfo, keep_time_ms: u32, frequency_ms: u32, hp_loss: u32,
    ) -> Self {
        Self { core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms), hp_loss }
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (core, mut reader) = PeriodicAttackCore::decode(payload, offset, ID, now)?;
        Ok(Self { core, hp_loss: reader.read_u32()? })
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; POISON_STATE_BYTES]
    where Self: AppliedState,
    {
        encode_periodic_state(self, now).try_into().expect("запись яда содержит 56 байт")
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; POISON_STATE_BYTES]
    where Self: AppliedState,
    {
        encode_periodic_state_for_install(self).try_into().expect("запись яда содержит 56 байт")
    }

    pub(crate) const fn skill_id(&self) -> u32 { ID }
    pub(crate) const fn master(&self) -> MasterInfo { self.core.master() }
    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }
}

impl<const ID: u32> PeriodicAttackState for PoisonState<ID>
where Self: AppliedState,
{
    const STATE_ID: u32 = ID;
    const RECORD_BYTES: usize = POISON_STATE_BYTES;
    type AttackSeed = u32;

    fn core(&self) -> &PeriodicAttackCore { &self.core }
    fn core_mut(&mut self) -> &mut PeriodicAttackCore { &mut self.core }
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) { writer.write_u32(self.hp_loss); }
    fn attack_seed(&self) -> u32 { self.hp_loss }
}

pub(crate) fn update_poison_state<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, runtime: &mut Runtime,
) -> bool
where PoisonState<ID>: AppliedState,
{
    update_periodic_attack_state::<PoisonState<ID>, Runtime>(
        game, region_id, holder, key, runtime, |_game, _target, master, hp_loss| {
            periodic_attack_information(master, 1.0, false, AttackPower {
                kind: AttackPowerType::Poison,
                hp_damage: if ID == 0x21e { (hp_loss as i32).max(0) } else { hp_loss as i32 },
                mp_damage: 0,
            })
        },
    )
}
