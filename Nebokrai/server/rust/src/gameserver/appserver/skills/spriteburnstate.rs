//! Каноническое периодическое состояние `CSpriteBurnState` (`0x1a6`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spriteburnstate.cpp`. Состояние хранит снимок
//! `tagMasterInfo`, использует отдельные чтения часов для срока и частоты,
//! строгую границу `>` и увеличивает счётчик перед каждым огненным ударом.
//! Замена, визуальные пакеты и применение урона идут через реальных владельцев
//! игрока либо монстра; координатные перегрузки `Begin` остаются RAW ниже.
//! Клиентский срок направлен на общий exact-owner `0x00606320` с двумя
//! отдельными чтениями wrapping clock для положительного остатка.

use super::spriteburn::SPRITE_BURN_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const LEGACY_UNKNOWN_SKILL_ID: u32 = i32::MAX as u32;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SpriteBurnStateTick {
    Pending,
    Attack(AttackInformation),
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpriteBurnState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    attack_count: u32,
}

impl SpriteBurnState {
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
        SPRITE_BURN_SKILL_ID
    }

    pub(crate) const fn master(self) -> MasterInfo {
        self.master
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
    ) -> SpriteBurnStateTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms || target_dead {
            return SpriteBurnStateTick::Ended;
        }
        let delay = self.frequency_ms.wrapping_mul(self.attack_count);
        if frequency_now_ms.wrapping_sub(self.started_at_ms) <= delay {
            return SpriteBurnStateTick::Pending;
        }
        self.attack_count = self.attack_count.wrapping_add(1);
        SpriteBurnStateTick::Attack(AttackInformation {
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
                hp_damage: self.hp_loss as i32,
                mp_damage: 0,
            }],
        })
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_sprite_burn_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SpriteBurnState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_ulong(state.client_state_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn send_sprite_burn_state_visual_in_region(
    game: &CGame,
    region: &crate::gameserver::appserver::serverregion::CServerRegion,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SpriteBurnState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_ulong(state.client_state_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_game_position_around(region, tile_x, tile_y, &message);
}

pub(crate) fn install_sprite_burn_state(
    game: &mut CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    target: ShapeIdentity,
    state: SpriteBurnState,
    now_ms: u32,
) {
    let replaced = match target.object_type {
        400 => game.find_player_mut(target.id).and_then(|player| {
            let identity = player.shape().identity();
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            Some((player.replace_sprite_burn_state(state), identity, x, y))
        }),
        MONSTER_TYPE => region.find_monster_by_id_mut(target.id).and_then(|monster| {
            let identity = monster.move_shape().shape().identity();
            let x = monster.move_shape().shape().get_tile_x().ok()?;
            let y = monster.move_shape().shape().get_tile_y().ok()?;
            Some((
                monster.move_shape_mut().replace_sprite_burn_state(state),
                identity,
                x,
                y,
            ))
        }),
        _ => None,
    };
    let Some((previous, identity, x, y)) = replaced else {
        return;
    };
    if let Some(previous) = previous {
        send_sprite_burn_state_visual_in_region(
            game, region, identity, x, y, previous, false, now_ms,
        );
    }
    send_sprite_burn_state_visual_in_region(
        game, region, identity, x, y, state, true, now_ms,
    );
}

pub(crate) fn update_player_sprite_burn_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(mut state) = game
        .find_player_mut(player_id)
        .and_then(CPlayer::take_sprite_burn_state_for_ai)
    else {
        return false;
    };
    let Some((identity, x, y, region_id, dead)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            player.server_region_id()?,
            player.is_dead(),
        ))
    }) else {
        return false;
    };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    match state.tick(lifetime_now_ms, frequency_now_ms, dead) {
        SpriteBurnStateTick::Pending => {
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_sprite_burn_state(state);
            }
        }
        SpriteBurnStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_sprite_burn_state(state);
            }
            if master.master_type == MONSTER_TYPE {
                game.apply_monster_periodic_state_attack(master, identity, region_id, attack, runtime);
            } else {
                game.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime);
            }
        }
        SpriteBurnStateTick::Ended => {
            if let Some(player) = game.find_player_mut(player_id) {
                player.finish_periodic_attack_state(state.skill_id());
            }
            send_sprite_burn_state_visual(
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

pub(crate) fn update_monster_sprite_burn_state<Runtime: GameMainLoopRuntime>(
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
            let state = monster.move_shape_mut().take_sprite_burn_state_for_ai()?;
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
        SpriteBurnStateTick::Pending => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster.move_shape_mut().replace_sprite_burn_state(state);
                }
                game.restore_region_owner(owner);
            }
        }
        SpriteBurnStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster.move_shape_mut().replace_sprite_burn_state(state);
                }
                game.restore_region_owner(owner);
            }
            if master.master_type == MONSTER_TYPE {
                game.apply_monster_periodic_state_attack(master, identity, region_id, attack, runtime);
            } else {
                game.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime);
            }
        }
        SpriteBurnStateTick::Ended => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    monster.move_shape_mut().finish_periodic_attack_state(state.skill_id());
                }
                game.restore_region_owner(owner);
            }
            send_sprite_burn_state_visual(
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

pub(crate) fn finish_player_sprite_burn_state_on_cure(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_sprite_burn_state()?;
        Some((
            state,
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else {
        return false;
    };
    send_sprite_burn_state_visual(
        game, region_id, identity, tile_x, tile_y, state, false, now_ms,
    );
    true
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp

// ============================================================================
// FUNCTION: CSpriteBurnState::CSpriteBurnState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:30
// RVA: 0x00206220
// ADDRESS: 00606220
// PROTOTYPE: undefined __thiscall CSpriteBurnState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::~CSpriteBurnState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:43
// RVA: 0x002062B0
// ADDRESS: 006062b0
// PROTOTYPE: void __thiscall ~CSpriteBurnState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:69
// RVA: 0x00206350
// ADDRESS: 00606350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:83
// RVA: 0x002063F0
// ADDRESS: 006063f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:55
// RVA: 0x002064C0
// ADDRESS: 006064c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:226
// RVA: 0x00206560
// ADDRESS: 00606560
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:161
// RVA: 0x002066A0
// ADDRESS: 006066a0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, tagAttackInformation * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburnstate.cpp:113
// RVA: 0x00206740
// ADDRESS: 00606740
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
