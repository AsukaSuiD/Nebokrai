//! Каноническое периодическое состояние `CPoisonArrowState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonarrowstate.cpp`. Состояние хранит снимок владельца,
//! использует два отдельных чтения часов на шаг и строгую проверку `>` для
//! срока и очередного удара. Формирование атаки с типом `Poison` и сообщений
//! `0xBFE03/0xBFE04`
//! и полный такт lifecycle принадлежат этому модулю; `CGame` только применяет
//! рассчитанную атаку к независимому владельцу игрока или монстра и выполняет
//! доставку.
//! Не достигнуты восстановление из DB и координатные перегрузки `Begin`; их RAW
//! сохранён ниже без второго изменяемого представления состояния.

use super::poisonarrow::POISON_ARROW_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const LEGACY_UNKNOWN_SKILL_ID: u32 = i32::MAX as u32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PoisonArrowStateTick {
    Pending,
    Attack(AttackInformation),
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PoisonArrowState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    attack_count: u32,
}

impl PoisonArrowState {
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_loss: u32,
    ) -> Self {
        Self {
            master,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            hp_loss,
            attack_count: 0,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        POISON_ARROW_SKILL_ID
    }

    pub(crate) const fn master(self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms {
            0
        } else {
            self.keep_time_ms.wrapping_sub(elapsed) as i32
        }
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
    ) -> PoisonArrowStateTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms || target_dead {
            return PoisonArrowStateTick::Ended;
        }
        let delay = self.frequency_ms.wrapping_mul(self.attack_count);
        if frequency_now_ms.wrapping_sub(self.started_at_ms) <= delay {
            return PoisonArrowStateTick::Pending;
        }
        self.attack_count = self.attack_count.wrapping_add(1);
        PoisonArrowStateTick::Attack(self.attack())
    }

    fn attack(self) -> AttackInformation {
        AttackInformation {
            skill_id: LEGACY_UNKNOWN_SKILL_ID,
            skill_level: 0,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 0,
            damage_factor: 1.0,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Poison,
                hp_damage: (self.hp_loss as i32).max(0),
                mp_damage: 0,
            }],
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической around-доставки")]
pub(crate) fn send_poison_arrow_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: PoisonArrowState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin {
        STATE_BEGIN_MESSAGE
    } else {
        STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn update_player_poison_arrow_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(mut state) = game
        .find_player_mut(player_id)
        .and_then(CPlayer::take_poison_arrow_state_for_ai)
    else {
        return false;
    };
    let target = game.find_player(player_id).and_then(|player| {
        Some((
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            player.server_region_id()?,
            player.is_dead(),
        ))
    });
    let Some((identity, x, y, region_id, dead)) = target else {
        return false;
    };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    match state.tick(lifetime_now_ms, frequency_now_ms, dead) {
        PoisonArrowStateTick::Pending => {
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_poison_arrow_state(state);
            }
        }
        PoisonArrowStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_poison_arrow_state(state);
            }
            game.apply_owned_skill_attack_to_player(
                master,
                player_id,
                region_id,
                attack,
                runtime,
            );
        }
        PoisonArrowStateTick::Ended => {
            if let Some(player) = game.find_player_mut(player_id) {
                player.finish_periodic_attack_state(state.skill_id());
            }
            send_poison_arrow_state_visual(
                game,
                region_id,
                identity,
                x,
                y,
                state,
                false,
                lifetime_now_ms,
            );
            let _ = game.publish_player_states(player_id);
        }
    }
    true
}

pub(crate) fn update_monster_poison_arrow_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else {
        return false;
    };
    let state_and_target = owner
        .base_mut()
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| {
            let state = monster
                .move_shape_mut()
                .take_poison_arrow_state_for_ai()?;
            let shape = monster.move_shape().shape();
            Some((
                state,
                shape.identity(),
                shape.get_tile_x().ok()?,
                shape.get_tile_y().ok()?,
                monster.hit_points() == 0,
            ))
        });
    game.restore_region_owner(owner);
    let Some((mut state, identity, x, y, dead)) = state_and_target else {
        return false;
    };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    match state.tick(lifetime_now_ms, frequency_now_ms, dead) {
        PoisonArrowStateTick::Pending => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster
                        .move_shape_mut()
                        .replace_poison_arrow_state(state);
                }
                game.restore_region_owner(owner);
            }
        }
        PoisonArrowStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster
                        .move_shape_mut()
                        .replace_poison_arrow_state(state);
                }
                game.restore_region_owner(owner);
            }
            game.apply_owned_skill_attack_to_monster(
                master,
                monster_id,
                region_id,
                attack,
                runtime,
            );
        }
        PoisonArrowStateTick::Ended => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    monster
                        .move_shape_mut()
                        .finish_periodic_attack_state(state.skill_id());
                }
                game.restore_region_owner(owner);
            }
            send_poison_arrow_state_visual(
                game,
                region_id,
                identity,
                x,
                y,
                state,
                false,
                lifetime_now_ms,
            );
        }
    }
    true
}

// Остаются недостигнутыми создание состояния для DB-восстановления,
// его чтение и координатные overload-ы Begin.
// ============================================================================
// FUNCTION: CPoisonArrowState::CPoisonArrowState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:30
// RVA: 0x001E31F0
// ADDRESS: 005e31f0
// PROTOTYPE: undefined __thiscall CPoisonArrowState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPoisonArrowState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:68
// RVA: 0x001E32F0
// ADDRESS: 005e32f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPoisonArrowState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:82
// RVA: 0x001E3390
// ADDRESS: 005e3390
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPoisonArrowState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:210
// RVA: 0x001E3500
// ADDRESS: 005e3500
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
