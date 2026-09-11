//! Каноническое состояние семейства `CSwordshipState`.
//!
//! PDB подтверждает одинаковый набор виртуальных операций у четырёх исходных
//! классов. Отличается только ID `0x6f/0xe0/0xe8/0xe9`. Для игрока обе
//! прибавки сначала знаково усекаются до `i16`, затем складываются через
//! `u32` и ограничиваются `i32::MAX`. Для монстра исходный владелец передаёт
//! полные значения `i32` методам `SetMinAtk/SetMaxAtk`, то есть выполняет
//! сложение с переполнением.
//! Состояния не имеют собственного таймера и визуального сообщения. Все четыре
//! vtable используют одну exact-пару `Serialize/Unserialize`
//! `0x005ECE70/0x005F0010`: persisted-запись состоит из ID и двух знаковых
//! DWORD-прибавок. Установка навыка и player-login работают с той же записью
//! `CanonicalStateStorage`, без отдельной raw-модели.

//! End +0x1C таблицы 0x00660D4C/0x006602AC/0x0065FEB4/0x0065FE64 →0x005ECFC0→CState::End0x005DBCE0:
//! ended=1, затем GetUser +0x14 и RemoveState при разрешённом user, без visual.
//! Begin +0x08 0x00601290 передаёт оба аргумента в CState::Begin.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime Begin0x005808BE/0x005588AE/0x0054B99E/0x0054B71E получает self,self.

use super::swordship::is_swordship_skill;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const SWORDSHIP_STATE_BYTES: usize = 12;

pub(crate) fn end_swordship_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, SWORDSHIP_STATE_BYTES)
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwordshipState {
    skill_id: u32,
    minimum_attack_gain: i32,
    maximum_attack_gain: i32,
}

impl SwordshipState {
    pub(crate) const fn new(
        skill_id: u32,
        minimum_attack_gain: i32,
        maximum_attack_gain: i32,
    ) -> Self {
        debug_assert!(is_swordship_skill(skill_id));
        Self {
            skill_id,
            minimum_attack_gain,
            maximum_attack_gain,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !is_swordship_skill(skill_id) {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(skill_id, reader.read_i32()?, reader.read_i32()?))
    }

    pub(crate) fn encoded(self) -> [u8; SWORDSHIP_STATE_BYTES] {
        let mut bytes = [0; SWORDSHIP_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.minimum_attack_gain.to_le_bytes());
        bytes[8..].copy_from_slice(&self.maximum_attack_gain.to_le_bytes());
        bytes
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.minimum_attack = apply_player_gain(
            properties.minimum_attack,
            self.minimum_attack_gain,
        );
        properties.maximum_attack = apply_player_gain(
            properties.maximum_attack,
            self.maximum_attack_gain,
        );
        properties
    }

    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32) -> (u32, u32) {
        (
            minimum.wrapping_add(self.minimum_attack_gain as u32),
            maximum.wrapping_add(self.maximum_attack_gain as u32),
        )
    }
}

fn apply_player_gain(value: u32, gain: i32) -> u32 {
    value
        .wrapping_add((gain as i16 as i32) as u32)
        .min(i32::MAX as u32)
}
