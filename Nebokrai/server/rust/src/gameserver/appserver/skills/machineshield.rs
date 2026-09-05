//! Достигнутый контракт самонакладываемого `CMachineShield`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/machineshield.cpp`. Навык `222` сохраняет двойную
//! проверку и необратимый расход MP, пакеты применения, замену состояния и
//! время восстановления. Общие с `CManaShield` стадии исполняет узкий
//! `selfshield`, а параметры и тип состояния остаются у этого owner-а.

pub(crate) const MACHINE_SHIELD_SKILL_ID: u32 = 222;
pub(crate) const MACHINE_SHIELD_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_STATE_HP: u32 = 10_010;
pub(crate) const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
pub(crate) const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

use super::machineshieldstate::{send_machine_shield_state_visual, MachineShieldState};
use super::selfshield::SelfShieldOwner;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::gameserver::game::CGame;

pub(crate) struct MachineShieldOwner;

impl SelfShieldOwner for MachineShieldOwner {
    type State = MachineShieldState;
    type Extra = ();

    const SKILL_ID: u32 = MACHINE_SHIELD_SKILL_ID;
    const EFFECT_MESSAGE: i32 = MACHINE_SHIELD_EFFECT_MESSAGE;

    fn read_extra(_properties: &CSkillBaseProperties) -> Self::Extra {}

    fn create_state(
        started_at_ms: u32,
        keep_time_ms: u32,
        life: i32,
        hp_factor: u16,
        mp_factor: u16,
        (): Self::Extra,
    ) -> Self::State {
        MachineShieldState::new(started_at_ms, keep_time_ms, life, hp_factor, mp_factor)
    }

    fn replace_state(player: &mut CPlayer, state: Self::State) -> Option<Self::State> {
        player.replace_machine_shield_state(state)
    }

    fn send_state_visual(
        game: &mut CGame,
        player_id: i32,
        state: Self::State,
        begin: bool,
        now_milliseconds: impl FnMut() -> u32,
    ) {
        send_machine_shield_state_visual(game, player_id, state, begin, now_milliseconds);
    }







}
