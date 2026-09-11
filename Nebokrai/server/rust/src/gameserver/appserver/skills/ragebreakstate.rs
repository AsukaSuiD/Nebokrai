//! Каноническое состояние подготовки яростного удара `CRageBreakState` (`0x6E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/ragebreakstate.cpp`. Новое наложение заменяет первый
//! найденный экземпляр, не удаляя остальные загруженные записи. Состояние
//! строго истекает после `started + keep`, увеличивает только
//! максимальную атаку. Signed-прибавка, `0.01_f32` и полный unsigned-максимум
//! перемножаются в x87, а общее с `CFuryState` тело `__ftol2` усекает результат
//! к нулю. Для игрока прибавка сужается до `WORD` и ограничивается суммой
//! `0xFFFF`; начало и завершение публикуются как `0xBFE03/04`.
//! Клиентский `GetRemainedTime` подтверждён ссылкой vtable на общее тело
//! `CFuryState` по `0x00605E10` и сохраняет два чтения wrapping clock.
//! Общий serializer `0x005E7330` и exact `Unserialize` `0x005FD660`
//! задают 12 байт: `ID + remaining time + attack gain`; spatial login
//! восстанавливает срок до общего пересчёта свойств.
//! Вызов Restart из Fury (vtable `0x006612B4 +0x20`, `0x005FD450`)
//! меняет только время начала, сохраняя прежние срок, усиление и DB-запись.
//! End (slot +0x1C, `0x005FD420`) сначала отправляет эффект, затем удаляет
//! состояние через RemoveState с пересчётом свойств держателя. Замена и AI
//! используют один этот порядок.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; RTTI-ограничения
//! формул игрока не запрещают жизненный цикл региональных держателей.
//! После visual владелец перечитывается; общий virtual UpdateProperty
//! вызывается только при фактическом удалении этой записи.
//! Прямой End, замена и AI используют один exact-key хвост без чтения часов.

//! Restart воспроизводит только Begin(NULL, holder) (0x005FD5C0):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Begin создаёт принадлежащий записи loop=1 visual без немедленного пакета.

//! Unserialize 0x005FD660 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! OnUpdateProperties 0x005FD480 после GetSufferer выполняет существующий
//! visual Update(0), затем RTTI-формулу. Игрок повторно сужает временную
//! прибавку +0x3C до WORD ПОСЛЕ ограничения суммы 0xFFFF; монстр прибавляет
//! полный signed delta к maximum_attack modifier через native +0x1AC.
//! Временное +0x3C не входит в Serialize и используется лишь в этом расчёте;
//! Rust держит его скаляром, не создавая второго persisted-поля.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const RAGE_BREAK_STATE_ID: u32 = 0x6e;
pub(crate) const RAGE_BREAK_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RageBreakState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl RageBreakState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, attack_gain_percent: i32) -> Self {
        Self { started_at_ms, keep_time_ms, attack_gain_percent }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != RAGE_BREAK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }


    pub(crate) fn encoded_for_install(self) -> [u8; RAGE_BREAK_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; RAGE_BREAK_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; RAGE_BREAK_STATE_BYTES] {
        let mut bytes = [0; RAGE_BREAK_STATE_BYTES];
        bytes[..4].copy_from_slice(&RAGE_BREAK_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { RAGE_BREAK_STATE_ID }

    pub(crate) const fn restart_timer(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    fn truncated_gain(self, maximum: u32) -> i32 {
        truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        )
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let mut gain = self.truncated_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain as u16 as u32).min(i32::MAX as u32)
    }
}

pub(crate) fn end_player_rage_break_state(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let Some((region_id, holder, key)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_region_id(), player.shape().identity(),
            player.move_shape().applied_state_key::<RageBreakState>()?))
    }) else { return false };
    end_rage_break_state_key(game, region_id, holder, key)
}

pub(crate) fn end_rage_break_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<RageBreakState>(key, RAGE_BREAK_STATE_BYTES))
        .is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

pub(crate) fn update_rage_break_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<RageBreakState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        let maximum = game.find_region(target_region)
            .and_then(|region| region.base().find_monster_by_id(target.id))
            .and_then(|monster| {
                let property = game.find_monster_property_by_origin_name(monster.original_name())?;
                Some(monster.state_attack_bounds(property.minimum_attack, property.maximum_attack).1)
            });
        if let Some(maximum) = maximum {
            let gain = state.truncated_gain(maximum);
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(gain);
            }
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.maximum_attack = state.apply_to_player_maximum_attack(properties.maximum_attack);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_rage_break_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_rage_break_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_rage_break_state_key(game, region_id, holder, key)
}
