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
//! Begin +0x08 0x00601290 передаёт оба аргумента в CState::Begin и возвращает 1.
//! Restart Begin(NULL, holder) сохраняет user и timestamp, снимает IsEnded;
//! собственных guards, visual, часов и сброса payload нет.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime Begin0x005808BE/0x005588AE/0x0054B99E/0x0054B71E получает self,self.

//! OnUpdateProperties (точный vtable +0x24 SwordshipState) сначала
//! разрешает GetSufferer; NULL возвращает 0. Type600/400 и RTTI выбирают
//! живые monster modifiers либо player tagProperty. Визуала, таймера,
//! повторного пересчёта и чтения итогового monster getter в этом callback нет.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/swordshipstate.cpp.


use super::swordship::is_swordship_skill;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const SWORDSHIP_STATE_BYTES: usize = 12;

pub(crate) fn update_swordship_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(state.minimum_attack_gain);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(state.maximum_attack_gain);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_swordship_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

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


}

fn apply_player_gain(value: u32, gain: i32) -> u32 {
    value
        .wrapping_add((gain as i16 as i32) as u32)
        .min(i32::MAX as u32)
}
