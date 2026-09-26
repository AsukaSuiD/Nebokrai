//! Данные четырёх состояний периодического ядовитого урона Zone.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/{poisonarrowstate,spiderpoisonstate,spriteburnstate,kerosenestate}.cpp/.h`.
//! Все четыре используют общие writer и getter срока; первые три читаются
//! одним reader-ом, Kerosene — отдельным, который берёт время после полей
//! записи.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use crate::combat::MasterInfo;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyWriter};

use super::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};

pub const POISON_STATE_BYTES: usize = 56;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoisonState<const ID: u32> {
    core: PeriodicAttackCore,
    hp_loss: u32,
}

impl<const ID: u32> PoisonState<ID> {
    pub const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_loss: u32,
    ) -> Self {
        Self {
            core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms),
            hp_loss,
        }
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (mut core, mut reader) = if ID == 0xf1 {
            PeriodicAttackCore::decode_deferred_start(payload, offset, ID)?
        } else {
            PeriodicAttackCore::decode(payload, offset, ID, now)?
        };
        let hp_loss = reader.read_u32()?;
        // У Kerosene часы обновляются после полного чтения записи.
        if ID == 0xf1 {
            core.finish_loading_at(now());
        }
        Ok(Self { core, hp_loss })
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; POISON_STATE_BYTES] {
        encode_periodic_state(self, now)
            .try_into()
            .expect("запись яда содержит 56 байт")
    }

    pub fn encoded_for_install(&self) -> [u8; POISON_STATE_BYTES] {
        encode_periodic_state_for_install(self)
            .try_into()
            .expect("запись яда содержит 56 байт")
    }

    pub const fn skill_id(&self) -> u32 {
        ID
    }
    pub const fn master(&self) -> MasterInfo {
        self.core.master()
    }
    pub const fn hp_loss(&self) -> u32 {
        self.hp_loss
    }
    pub fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        &mut self.core
    }
    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }
}

impl<const ID: u32> PeriodicAttackRecord for PoisonState<ID> {
    const STATE_ID: u32 = ID;
    const RECORD_BYTES: usize = POISON_STATE_BYTES;

    fn core(&self) -> &PeriodicAttackCore {
        &self.core
    }
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) {
        writer.write_u32(self.hp_loss);
    }
}
