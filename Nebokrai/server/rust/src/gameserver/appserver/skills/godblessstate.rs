//! Состояния CGodBlessState/CGodBlessState2 с общей формулой и DB-записью.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godblessstate{,2}.cpp.
//! Срок и три прибавки принадлежат payload; независимые U/S, ended и visual —
//! тому же поколенческому экземпляру общей арены. DB-запись из 20 байтов содержит
//! ID, остаток и min/max/element. Decode получает часы перед полями; ctor оставляет время 0.
//! Остаток требует второго чтения времени только до наступления срока,
//! AI завершает точный ключ при строгом unsigned start+keep < now.
//!
//! Объектный Begin God1 требует S, God2 — U; при ненулевом U базовые часы
//! предшествуют чтению регионов U/S. Loop1/0 создаётся без initial Update.
//! Restart(NULL, holder) God1 сохраняет U и время, заменяет S и visual;
//! God2 отказывает до базы, сохраняя запись. SetRegion переносит оба региона.
//! Property удерживает первую S через отдельный fresh-S visual, затем меняет
//! min/max/element: у игрока WORD-прибавки и потолок INT_MAX атаки, у монстра
//! полное wrapping-сложение DWORD. Loop0 God2 заканчивается после первого
//! BFE03; отсутствие visual не подавляет формулу, в том числе после загрузки.
//! End God1 делает visual1 → ended → fresh S → RemoveState, God2 пропускает
//! visual. Часов и fallback к держателю нет; чужая арена не владеет ключом.
//! SlotMap и общий wire-код заменяют native указатели и STL без нового runtime.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time};
use crate::gameserver::gameserver::game::CGame;

pub(crate) const GOD_BLESS_STATE_ID: u32 = 0x12f;
pub(crate) const GOD_BLESS_STATE_BYTES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GodBlessState {
    skill_id: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    minimum_attack_gain: u32,
    maximum_attack_gain: u32,
    element_gain: u32,
}

impl GodBlessState {
    pub(crate) const fn new(skill_id: u32, keep_time_ms: u32, minimum_attack_gain: u32, maximum_attack_gain: u32, element_gain: u32) -> Self {
        debug_assert!(matches!(skill_id, GOD_BLESS_STATE_ID | super::godblessstate2::GOD_BLESS_STATE_2_ID));
        Self { skill_id, started_at_ms: 0, keep_time_ms, minimum_attack_gain, maximum_attack_gain, element_gain }
    }
    pub(crate) fn begin_for_install(
        &mut self,
        user_exists: bool,
        sufferer_exists: bool,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        let can_begin = if self.skill_id == GOD_BLESS_STATE_ID { sufferer_exists } else { user_exists };
        if !can_begin {
            return false;
        }
        if user_exists {
            self.started_at_ms = now();
        }
        true
    }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !matches!(skill_id, GOD_BLESS_STATE_ID | super::godblessstate2::GOD_BLESS_STATE_2_ID) {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let mut state = Self::new(skill_id, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?);
        state.started_at_ms = now_ms;
        Ok(state)
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
    pub(crate) fn encoded_for_install(self) -> [u8; GOD_BLESS_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; GOD_BLESS_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; GOD_BLESS_STATE_BYTES] {
        let mut bytes = [0; GOD_BLESS_STATE_BYTES];
        for (index, value) in [self.skill_id, remaining_time_ms, self.minimum_attack_gain, self.maximum_attack_gain, self.element_gain].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        properties.minimum_attack = properties.minimum_attack.wrapping_add(self.minimum_attack_gain as u16 as u32).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_add(self.maximum_attack_gain as u16 as u32).min(i32::MAX as u32);
        properties.element_modify = properties.element_modify.wrapping_add(self.element_gain as u16 as i32);
        properties
    }

}


pub(crate) fn update_god_bless_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<GodBlessState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(state.minimum_attack_gain as i32);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(state.maximum_attack_gain as i32);
            modifiers.element_modify = modifiers.element_modify.wrapping_add(state.element_gain as i32);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
        else { return false };
    if state.skill_id() != GOD_BLESS_STATE_ID {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_god_bless_state(game, region_id, holder, key)
}

pub(crate) fn end_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
        else { return false };
    if state.skill_id() == GOD_BLESS_STATE_ID {
        crate::gameserver::appserver::states::state::update_applied_state_end_visual(
            game, region_id, holder, key, StatePropertyTarget::Sufferer,
        );
    }
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    shape.mark_applied_state_ended(key);
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false };
    crate::gameserver::appserver::states::state::remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), GOD_BLESS_STATE_BYTES,
    )
}
