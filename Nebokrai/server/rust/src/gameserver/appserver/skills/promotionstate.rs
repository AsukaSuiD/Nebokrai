//! Каноническое состояние усиления `CPromotionState` (`0x142`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotionstate.cpp`. Состояние остаётся в упорядоченной
//! ветви `CFightDefense::PreDefense`: для стихийной части удара множитель
//! применяется в точном месте исходного `m_vStates`. Второй коэффициент
//! принадлежит лечению. Начальный пакет состояния имеет исходный формат
//! `0xBFE03`; отдельного пакета завершения этот владелец не создаёт. DB-запись
//! сохраняет остаток срока и оба WORD-коэффициента. Vtable exact EXE
//! подтверждает общий с `CBlindState` `GetRemainedTime` по `0x005F2CD0`.
//! Restart (vtable `0x006605F4 +0x20`, тело `0x005FD450`) меняет только
//! timestamp: прежние длительность и коэффициенты сохраняются без нового пакета.
//! `AI` получает ключ достигнутого экземпляра из общего обхода состояний:
//! истечение удаляет только этот Promotion, не соседний одноимённый щит.
//! Primary Begin проверяет S, читает часы базы при U и отправляет одноразовый
//! visual до append. Запись хранит действительные U/S и собственный DB-span.

use super::promotion::PROMOTION_SKILL_ID;
use super::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const PROMOTION_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const PROMOTION_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PromotionState {
    started_at_ms: u32,
    keep_time_ms: u32,
    magic_attack_factor: u16,
    heal_recover_factor: u16,
}

impl PromotionState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        magic_attack_factor: u16,
        heal_recover_factor: u16,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            magic_attack_factor,
            heal_recover_factor,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        PROMOTION_SKILL_ID
    }

    pub(crate) const fn started_at_ms(self) -> u32 {
        self.started_at_ms
    }

    pub(crate) fn restart(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn magic_attack_factor(self) -> u16 {
        self.magic_attack_factor
    }

    pub(crate) const fn heal_recover_factor(self) -> u16 {
        self.heal_recover_factor
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(
        payload: &[u8],
        offset: usize,
        now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != PROMOTION_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(
            now_ms,
            reader.read_u32()?,
            reader.read_u16()?,
            reader.read_u16()?,
        ))
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; PROMOTION_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; PROMOTION_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(PROMOTION_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(PROMOTION_SKILL_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_u16(self.magic_attack_factor);
        writer.write_u16(self.heal_recover_factor);
        bytes
            .try_into()
            .expect("размер состояния усиления фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; PROMOTION_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

}

/// Повторный AI вызывает только Restart; новый primary проходит
/// CState::Begin(U,S) → visual loop0/Update0 → append в общую арену.
pub(crate) fn begin_or_restart_promotion_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    new_parameters: impl FnOnce() -> (u32, u16, u16),
    now: &mut dyn FnMut() -> u32,
) -> Option<bool> {
    let shape = resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    let sufferer = (shape.shape().get_region_id(), shape.shape().identity());
    if let Some(key) = shape.defense_shield_key(PROMOTION_SKILL_ID) {
        let state = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?
            .applied_state_mut::<DefenseShieldState>(key)?;
        let DefenseShieldState::Promotion(state) = state else { return None; };
        state.restart(now());
        return Some(false);
    }
    let (keep_time_ms, magic_attack_factor, heal_recover_factor) = new_parameters();
    let mut state = PromotionState::new(
        0, keep_time_ms, magic_attack_factor, heal_recover_factor,
    );
    if user.is_some() { state.restart(now()); }
    let user = match user {
        Some((region, identity)) => {
            let shape = resolve_state_move_shape(game, region, identity)?.shape();
            Some((shape.get_region_id(), shape.identity()))
        }
        None => None,
    };
    let mut message = CMessage::new(PROMOTION_STATE_BEGIN_MESSAGE);
    message.add_long(sufferer.1.object_type);
    message.add_long(sufferer.1.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_time(&mut *now));
    message.add_long(0);
    let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?;
    let key = shape.append_applied_state_record(DefenseShieldState::Promotion(state), &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    // Общий каталог хранит уже завершённый loop0 visual после Update(0).
    Some(true)
}
