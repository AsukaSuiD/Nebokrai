//! Каноническое состояние божественного благословения `CGodBlessState` (`0x12F`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godblessstate.cpp`. Состояние владеет сроком и тремя
//! прибавками. Игрок сохраняет сужение прибавок до `u16` и ограничение атаки
//! `INT_MAX`; монстр применяет исходное wrapping-сложение полных `u32`.
//! `CGame` только координирует независимых владельцев и around-доставку.
//! Обе идентичности используют общую 20-байтовую persisted-запись: ID,
//! remaining time и три прибавки; загрузка активируется при spatial login.
//! Обе concrete vtable направляют `GetRemainedTime` на точное тело
//! `0x00601480` с отдельным вторым чтением часов для положительного остатка.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! End вариантов различается: vtable 0x00661864 → 0x00601610 выполняет
//! visual → ended → GetSufferer → RemoveState; у GodBless2 vtable 0x00660074
//! ведёт на 0x005D5B80 с тем же хвостом, но БЕЗ visual. Прямой End не читает
//! часы; AI проверяет срок и вызывает это же завершение точного экземпляра.
//! God1 вызывает visual только при существующем ресурсе; Update(1)
//! 0x00601880 учитывает visual.ended и GetSufferer, затем общий visual-tail.
//! NULL sufferer оставляет завершённый payload для внешнего destructor;
//! одноимённая форма в другом регионе не владеет этим поколенческим ключом.

//! Restart воспроизводит только Begin(NULL, holder) (0x00601790/0x005EE380):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! GodBless1 создаёт принадлежащий записи loop=1 visual без немедленного пакета.
//! GodBless2 требует ненулевой User и при таком restart возвращает 0 до базы.

//! Unserialize 0x00601830 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! Общий OnUpdateProperties 0x00601690: GetSufferer → существующий visual
//! Update(0) → type600/400 и RTTI → min/max/element_modify. Monster setters
//! складывают полные raw DWORD с модификаторами; player сохраняет WORD gains.
//! God2 Begin 0x005EE380 создаёт loop=0 БЕЗ initial Update, поэтому первый
//! property visual отправляет BFE03 и завершает ресурс; формула работает далее.
//! DecodeExStates назначает sufferer до Begin: loaded player получает формулу
//! даже при отказе Begin(NULL,holder), но отсутствующий visual не подменяется.
//! Первичный ctor 0x00601370/0x005EE0B0 оставляет timestamp=0, без часов.
//! Begin God1 требует sufferer, God2 — user; общий CState::Begin 0x005DBD70
//! читает один clock только при ненулевом user. Первичная установка делает
//! это после End прежнего экземпляра, до append и property callback.
//! Silent visual и base user/sufferer metadata переходят общему arena-owner
//! в той же границе без промежуточного callback; persisted codec не меняется.

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
    pub(crate) const fn new(skill_id: u32, started_at_ms: u32, keep_time_ms: u32, minimum_attack_gain: u32, maximum_attack_gain: u32, element_gain: u32) -> Self {
        debug_assert!(matches!(skill_id, GOD_BLESS_STATE_ID | super::godblessstate2::GOD_BLESS_STATE_2_ID));
        Self { skill_id, started_at_ms, keep_time_ms, minimum_attack_gain, maximum_attack_gain, element_gain }
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
        Ok(Self::new(skill_id, now_ms, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?))
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
