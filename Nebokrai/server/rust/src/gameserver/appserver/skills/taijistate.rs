//! Каноническая достигнутая часть `CTaiJiState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0x12d` без собственного визуального эффекта. Для игрока
//! `OnUpdateProperties` использует младшие 16 бит параметра и насыщает
//! сопротивление стихиям до `i32::MAX`. Монстр передаёт полный signed gain в
//! wrapping-additive `CMonster::SetElementResistant`; minimum/factor остаются
//! у итогового monster property getter-а.
//! Exact-пара `Serialize/Unserialize` `0x005E23D0/0x00601350` сохраняет
//! восьмибайтную запись `ID + signed gain`.

//! End +0x1C таблицы 0x006617C4 →0x005ECFC0→CState::End0x005DBCE0:
//! ended=1, затем GetUser +0x14 и RemoveState при разрешённом user, без visual.
//! Begin +0x08 0x00601050: null user завершает Begin с отказом до базы,
//! без изменения IsEnded, timer и payload. Restart сохраняет именно этот отказ.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime CTaiJi::AI0x005AF770 вызывает state Begin(self,self).

//! OnUpdateProperties (точный vtable +0x24 TaiJiState) сначала
//! разрешает GetSufferer; NULL возвращает 0. Type600/400 и RTTI выбирают
//! живые monster modifiers либо player tagProperty. Визуала, таймера,
//! повторного пересчёта и чтения итогового monster getter в этом callback нет.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/taijistate.cpp.


use super::taiji::TAIJI_SKILL_ID;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const TAIJI_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TaiJiState {
    element_resistance_gain: i32,
}

impl TaiJiState {
    pub(crate) const fn new(element_resistance_gain: i32) -> Self {
        Self { element_resistance_gain }
    }

    pub(crate) const fn skill_id(self) -> u32 { TAIJI_SKILL_ID }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != TAIJI_SKILL_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        Ok(Self::new(reader.read_i32()?))
    }
    pub(crate) fn encoded(self) -> [u8; TAIJI_STATE_BYTES] {
        let mut bytes = [0; TAIJI_STATE_BYTES]; bytes[..4].copy_from_slice(&TAIJI_SKILL_ID.to_le_bytes()); bytes[4..].copy_from_slice(&self.element_resistance_gain.to_le_bytes()); bytes
    }
    pub(crate) const fn player_element_resistance_gain(self) -> u16 {
        self.element_resistance_gain as u16
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.element_resistance = properties
            .element_resistance
            .wrapping_add(u32::from(self.player_element_resistance_gain()))
            .min(i32::MAX as u32);
        properties
    }


}

pub(crate) fn update_tai_ji_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.element_resistance = modifiers.element_resistance.wrapping_add(state.element_resistance_gain);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_tai_ji_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn end_tai_ji_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, TAIJI_STATE_BYTES)
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён недостигнутый конструктор по умолчанию; общий callback свойств реализован.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.h

// ============================================================================
// FUNCTION: CTaiJiState::CTaiJiState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.cpp:24
// RVA: 0x00200FD0
// ADDRESS: 00600fd0
// PROTOTYPE: undefined __thiscall CTaiJiState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// CTaiJiState::OnUpdateProperties (0x006010F0) реализован
// в update_tai_ji_state_properties; monster modifier меняется напрямую.

// COMPONENT_VARIANT_END: GameServer
