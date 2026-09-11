//! Каноническое периодическое состояние `CLeafCutState3` (`0x8F`).
//! Периодический AI изменяет payload по поколенческому ключу общей арены.
//! Чистый tick завершается до межвладельческого удара; состояние не вынимается
//! и остаётся доступным вложенному End/Clear. Удар использует независимый снимок.
//! Прямой End: vtable 0x0065FD44, слот +0x1C → 0x005FD420: visual
//! с фазой 1 → GetSufferer (+0x18, 0x005DBFD0) → RemoveState (0x004CDAB0).
//! Это не CState::End: записи IsEnded и проверок времени/HP в нём нет.
//! Runtime Begin и StartAllStates связывают sufferer с holder; MasterInfo
//! остаётся источником атаки, а не владельцем удаляемого ключа. End работает
//! с опубликованной формой и точным ключом; ошибка доставки не отменяет удаление.
//! После RemoveState перенесённый player UpdateProperty вызывается явно;
//! monster читает изменения состояния в живых getters. Остальные property
//! overrides не подменяются выдуманным callback.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate3.cpp`. Класс наследует формулу, два чтения
//! часов и 68-байтовый DB-кодек `CLeafCutState`, но хранится отдельным
//! состоянием и сохраняет собственный ID. Обёртка не создаёт второй источник
//! истины: жизненный цикл принадлежит `CanonicalStateStorage`, а его runtime
//! tick выполняется этим owner-ом до межвладельческого применения атаки.
//! Унаследованный клиентский срок сохраняет два чтения exact-owner-а
//! `CBloodLossState::GetRemainedTime` по `0x00606320`.

use super::leafcutstate::{LeafCutState, LeafCutStateTick};
use crate::gameserver::appserver::legacycodec::LegacyReadBlock;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_3_STATE_ID: u32 = 0x8f;
pub(crate) const LEAF_CUT_3_STATE_BYTES: usize = 68;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState3(LeafCutState);

impl LeafCutState3 {
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
            LEAF_CUT_3_STATE_ID,
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

    pub(crate) const fn skill_id(self) -> u32 { LEAF_CUT_3_STATE_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.0.master() }
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 { self.0.client_state_time(now_milliseconds) }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { self.0.serialized_span() }
    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        self.0.shift_serialized_offset_for_insert(inserted_offset, amount);
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { self.0.shift_serialized_offset_after(removed_offset, amount); }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.0.activate_loaded(now_ms); }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { LeafCutState::decode_with_id(payload, offset, now_ms, LEAF_CUT_3_STATE_ID).map(Self) }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) { self.0.append_serialized(payload, now_ms); }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool { self.0.write_serialized_at(payload, offset, now_ms) }
    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { self.0.update_serialized_runtime(payload, now_ms); }
    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, frequency_now_ms: u32, target_dead: bool, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> LeafCutStateTick { self.0.tick(lifetime_now_ms, frequency_now_ms, target_dead, critical_chance, critical_rate, random) }
}

pub(crate) fn send_leaf_cut_3_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: LeafCutState3,
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

pub(crate) fn end_leaf_cut_3_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state_id) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<LeafCutState3>(key))
        .map(|state| state.skill_id())
    else {
        return false;
    };
    let mut message = CMessage::new(STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state_id as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<LeafCutState3>(key, LEAF_CUT_3_STATE_BYTES))
        .is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

pub(crate) fn update_player_leaf_cut_3_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let target = game.find_player(player_id).and_then(|player| {
        player.move_shape().applied_state::<LeafCutState3>(key)?;
        let shape = player.move_shape().shape();
        Some((
            shape.identity(),
            player.server_region_id()?,
            player.is_dead(),
            player.combat_properties().cch,
        ))
    });
    let Some((identity, region_id, dead, critical_chance)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let prepared = game.with_player_state_random::<LeafCutState3, _>(player_id, key, |state, random| {
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
            let _ = end_leaf_cut_3_state(game, region_id, identity, key);
            let _ = game.publish_player_states(player_id);
        }
    }
    true
}

pub(crate) fn update_monster_leaf_cut_3_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let Some(owner) = game.take_region_owner(region_id) else { return false };
    let target = owner.base().find_monster_by_id(monster_id).and_then(|monster| {
        monster.move_shape().applied_state::<LeafCutState3>(key)?;
        let shape = monster.move_shape().shape();
        Some((shape.identity(), monster.hit_points() == 0))
    });
    game.restore_region_owner(owner);
    let Some((identity, dead)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let prepared = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().applied_state_mut::<LeafCutState3>(key)?;
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
            let _ = end_leaf_cut_3_state(game, region_id, identity, key);
        }
    }
    true
}
