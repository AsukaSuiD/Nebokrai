//! Каноническое состояние `COriginState`.
//!
//! Игрок получает знаково расширенные младшие 16 бит параметра через
//! wrapping-сложение с `element_modify`. Состояние `304` не имеет собственного
//! таймера или визуального сообщения. Monster-ветвь передаёт полный signed gain
//! в wrapping-additive `SetElementModify`; clamp/factor применяет итоговый
//! monster property getter.
//! Constructor RVA `0x00201210` задаёт ID `0x130` и нулевой gain.
//! Exact persisted-запись общей пары `0x005E23D0/0x00601350` — `ID + i32 gain`.

//! End +0x1C таблицы 0x00661814 →0x005ECFC0→CState::End0x005DBCE0:
//! ended=1, затем GetUser +0x14 и RemoveState при разрешённом user, без visual.
//! Begin +0x08 0x00601290 передаёт оба аргумента в CState::Begin и возвращает 1.
//! Restart Begin(NULL, holder) сохраняет user и timestamp, снимает IsEnded;
//! собственных guards, visual, часов и сброса payload нет.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime COrigin::AI0x005AFA70 вызывает state Begin(self,self).

//! OnUpdateProperties (точный vtable +0x24 OriginState) сначала
//! разрешает GetSufferer; NULL возвращает 0. Type600/400 и RTTI выбирают
//! живые monster modifiers либо player tagProperty. Визуала, таймера,
//! повторного пересчёта и чтения итогового monster getter в этом callback нет.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/originstate.cpp.


use super::origin::ORIGIN_SKILL_ID;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};

pub(crate) const ORIGIN_STATE_BYTES: usize = 8;

pub(crate) fn update_origin_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<OriginState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.element_modify = modifiers.element_modify.wrapping_add(state.element_modify_gain);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.element_modify = state.apply_to_player(properties.element_modify);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_origin_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<OriginState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_origin_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<OriginState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ORIGIN_STATE_BYTES)
}


#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OriginState {
    element_modify_gain: i32,
}

impl OriginState {
    pub(crate) const fn new(element_modify_gain: i32) -> Self {
        Self {
            element_modify_gain,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ORIGIN_SKILL_ID
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != ORIGIN_SKILL_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } Ok(Self::new(reader.read_i32()?)) }
    pub(crate) fn encoded(self) -> [u8; ORIGIN_STATE_BYTES] { let mut bytes = [0; ORIGIN_STATE_BYTES]; bytes[..4].copy_from_slice(&ORIGIN_SKILL_ID.to_le_bytes()); bytes[4..].copy_from_slice(&self.element_modify_gain.to_le_bytes()); bytes }

    pub(crate) const fn apply_to_player(self, value: i32) -> i32 {
        value.wrapping_add((self.element_modify_gain as i16) as i32)
    }


}
