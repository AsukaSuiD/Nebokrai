//! Общий payload состояний PoisonArrow, SpiderPoison, SpriteBurn и Kerosene.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые owners appserver/skills.
//! Их запись содержит 56 байт и один DWORD урона HP. Тип урона —
//! Poison, включая SpriteBurn и Kerosene; MP не изменяется. Конкретный ID сохраняет
//! отдельный вариант общей арены, без дополнительного хранилища.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyWriter};
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
        let (mut core, mut reader) = if ID == 0xf1 {
            PeriodicAttackCore::decode_deferred_start(payload, offset, ID)?
        } else {
            PeriodicAttackCore::decode(payload, offset, ID, now)?
        };
        let hp_loss = reader.read_u32()?;
        // Kerosene начинает отсчёт после полного чтения записи, остальные
        // варианты — после MasterInfo, до срока, частоты и HP.
        if ID == 0xf1 { core.finish_loading_at(now()); }
        Ok(Self { core, hp_loss })
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
    // SpriteBurn может ударить и после истечения срока: проверка времени
    // следует за попыткой атаки, даже когда частота пропускает удар.
    const CHECK_LIFETIME_AFTER_ATTACK: bool = ID == 0x1a6;
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
                // Только PoisonArrow ограничивает отрицательный signed HP.
                hp_damage: if ID == 0x21e { (hp_loss as i32).max(0) } else { hp_loss as i32 },
                mp_damage: 0,
            })
        },
    )
}
