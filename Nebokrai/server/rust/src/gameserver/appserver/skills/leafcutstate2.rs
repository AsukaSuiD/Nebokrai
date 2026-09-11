//! Каноническое периодическое состояние `CLeafCutState2` (`0x80`).
//! Периодический AI изменяет payload по поколенческому ключу общей арены.
//! Чистый tick завершается до межвладельческого удара; состояние не вынимается
//! и остаётся доступным вложенному End/Clear. Снимок нужен только пакету End.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate2.cpp`. Формула, два чтения часов и два
//! вызова MSVCRT RNG совпадают с подтверждённой основой `CLeafCutState`, но
//! состояние имеет отдельную идентичность и lifecycle. Exact vtable направляет
//! `Serialize/Unserialize` на общую пару `0x005F0820/0x005EBF20`, поэтому
//! состояние использует тот же 68-байтный DB-кодек с собственным ID. Чистый tick
//! и точное завершение принадлежат этому owner-у; payload остаётся в общей арене.
//! Унаследованный клиентский срок сохраняет два чтения exact-owner-а
//! `CBloodLossState::GetRemainedTime` по `0x00606320`.

use super::leafcutstate::{LeafCutState, LeafCutStateTick};
use crate::gameserver::appserver::legacycodec::LegacyReadBlock;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_2_STATE_ID: u32 = 0x80;
pub(crate) const LEAF_CUT_2_STATE_BYTES: usize = 68;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState2(LeafCutState);

impl LeafCutState2 {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
        element_attack: u16,
        soul_attack: u16,
    ) -> Self {
        Self(LeafCutState::new_with_id(
            LEAF_CUT_2_STATE_ID,
            master,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            damage_factor,
            damage_modifier,
            minimum_attack,
            maximum_attack,
            element_attack,
            soul_attack,
        ))
    }

    pub(crate) const fn skill_id(self) -> u32 { LEAF_CUT_2_STATE_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.0.master() }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { LeafCutState::decode_with_id(payload, offset, now_ms, LEAF_CUT_2_STATE_ID).map(Self) }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.0.activate_loaded(now_ms); }
    pub(crate) fn encoded(self, now_ms: u32) -> Vec<u8> { self.0.encoded(now_ms) }
    pub(crate) fn encoded_for_install(self) -> Vec<u8> { self.0.encoded_for_install() }
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 { self.0.client_state_time(now_milliseconds) }
    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> LeafCutStateTick {
        self.0.tick(
            lifetime_now_ms,
            frequency_now_ms,
            target_dead,
            critical_chance,
            critical_rate,
            random,
        )
    }
}

pub(crate) fn send_leaf_cut_2_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: LeafCutState2,
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

pub(crate) fn update_player_leaf_cut_2_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    key: crate::gameserver::appserver::moveshape::StateKey,
    runtime: &mut Runtime,
) -> bool {
    let target = game.find_player(player_id).and_then(|player| {
        player.move_shape().applied_state::<LeafCutState2>(key)?;
        let shape = player.move_shape().shape();
        Some((shape.identity(), shape.get_tile_x().ok()?, shape.get_tile_y().ok()?,
            player.server_region_id()?, player.is_dead(), player.combat_properties().cch))
    });
    let Some((identity, x, y, region_id, dead, critical_chance)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let prepared = game.with_player_state_random::<LeafCutState2, _>(player_id, key, |state, random| {
        let tick = state.tick(lifetime_now_ms, frequency_now_ms, dead, critical_chance, critical_rate, random);
        (tick, *state)
    });
    let Some((tick, state)) = prepared else { return false };
    match tick {
        LeafCutStateTick::Pending => {}
        LeafCutStateTick::Attack(attack) => {
            let master = state.master();
            game.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime);
        }
        LeafCutStateTick::Ended => {
            if let Some(player) = game.find_player_mut(player_id) {
                let move_shape = player.move_shape_mut();
                let _ = move_shape.remove_applied_state_record::<LeafCutState2>(key, LEAF_CUT_2_STATE_BYTES);
            }
            send_leaf_cut_2_state_visual(game, region_id, identity, x, y, state, false, lifetime_now_ms);
            let _ = game.publish_player_states(player_id);
        }
    }
    true
}

pub(crate) fn update_monster_leaf_cut_2_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    key: crate::gameserver::appserver::moveshape::StateKey,
    runtime: &mut Runtime,
) -> bool {
    let Some(owner) = game.take_region_owner(region_id) else { return false };
    let target = owner.base().find_monster_by_id(monster_id).and_then(|monster| {
        monster.move_shape().applied_state::<LeafCutState2>(key)?;
        let shape = monster.move_shape().shape();
        Some((shape.identity(), shape.get_tile_x().ok()?, shape.get_tile_y().ok()?,
            monster.hit_points() == 0))
    });
    game.restore_region_owner(owner);
    let Some((identity, x, y, dead)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let prepared = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().applied_state_mut::<LeafCutState2>(key)?;
        let tick = state.tick(lifetime_now_ms, frequency_now_ms, dead, 0, critical_rate, &mut |maximum| game.skill_random_below(maximum));
        Some((tick, *state))
    });
    game.restore_region_owner(owner);
    let Some((tick, state)) = prepared else { return false };
    match tick {
        LeafCutStateTick::Pending => {}
        LeafCutStateTick::Attack(attack) => {
            let master = state.master();
            game.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime);
        }
        LeafCutStateTick::Ended => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let move_shape = monster.move_shape_mut();
                    let _ = move_shape.remove_applied_state_record::<LeafCutState2>(key, LEAF_CUT_2_STATE_BYTES);
                }
                game.restore_region_owner(owner);
            }
            send_leaf_cut_2_state_visual(game, region_id, identity, x, y, state, false, lifetime_now_ms);
        }
    }
    true
}
