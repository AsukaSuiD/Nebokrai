//! Каноническое временное состояние `CAgilityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `agilitystate2.cpp`. Состояние `0x81` добавляет `full_miss` сложением с
//! переполнением, AI завершает его только при строгом `started + keep < now` и при
//! вычислении положительного клиентского остатка второй раз читает часы.
//! Жизненный цикл принадлежит `CanonicalStateStorage`; DB-запись хранит
//! остаток срока и WORD-прибавку полного уклонения. Истечение сразу запускает
//! полный пересчёт свойств, чтобы снятая прибавка не оставалась в combat snapshot.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; правила свойств
//! игрока не запрещают жизненный цикл региональных держателей.
//! Exact vtable 0x006602FC: AI 0x005D60B0, End 0x005EEBA0.
//! End только разрешает sufferer и удаляет запись: визуала и отдельной
//! публикации HP/MP/RP/YP в этом пути нет.
//! UpdateProperty вызывается только для игрока при фактическом удалении.
//! Прямой End и AI используют один exact-key хвост; проверка срока остаётся только в AI.

//! Restart воспроизводит только Begin(NULL, holder) (0x005F0350):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Visual принадлежит экземпляру общей арены: BeginVisualEffect(0) →
//! concrete Update(0) → базовый visual-хвост; только getter пакета читает часы.
//! У loop=0 базовый visual-хвост завершает ресурс; End этого состояния visual не вызывает.

//! Unserialize 0x005F48E0 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::nets::netserver::message::CMessage;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use super::agility2::AGILITY_2_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const AGILITY_STATE_2_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityState2 {
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
}

impl AgilityState2 {
    pub(crate) const fn new(full_miss: u16, started_at_ms: u32, keep_time_ms: i32) -> Self {
        Self {
            full_miss,
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { AGILITY_2_SKILL_ID }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.full_miss = properties.full_miss.wrapping_add(self.full_miss);
        properties
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms as u32) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.keep_time_ms as u32,
            now_milliseconds,
        ) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != AGILITY_2_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let keep_time_ms = reader.read_i32()?;
        Ok(Self::new(reader.read_u16()?, now_ms, keep_time_ms))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = Vec::with_capacity(AGILITY_STATE_2_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(AGILITY_2_SKILL_ID);
        writer.write_i32(self.client_time(|| now_ms));
        writer.write_u16(self.full_miss);
        bytes.try_into().expect("размер временного состояния ловкости фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; AGILITY_STATE_2_BYTES] {
        self.encoded(self.started_at_ms)
    }


}

pub(crate) fn restart_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 0) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_long(state.client_time(now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn update_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_agility_state_2(game, region_id, holder, key)
}

pub(crate) fn end_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<AgilityState2>(key, AGILITY_STATE_2_BYTES))
        .is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}
