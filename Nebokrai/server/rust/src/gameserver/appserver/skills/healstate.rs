//! Периодическое лечение CHealState/2 и CSuperHealState/2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/{healstate,
//! healstate2,superhealstate,superhealstate2}.cpp и первичные heal*.cpp.
//! Четыре варианта имеют один payload и отличаются ID. Сначала завершается
//! первый прежний ID без RTTI/ended-фильтра, затем создаётся новый экземпляр.
//! SuperHeal удаляет D3, остальные — собственный ID. Begin читает часы,
//! сохраняет U/S и запускает loop=1 visual до append; счётчик сбрасывается
//! после BeginVisual. Begin и End не отправляют BFE03/BFE04.
//!
//! Запись SuperHeal2 остаётся у выбранной цели, но её U/S указывают на
//! заклинателя. Эти привязки живут только в общей арене, не в копии payload.
//! End удаляет собственный указатель через свежего S без записи ended;
//! при чужом или отсутствующем S запись остаётся у держателя. SetRegion
//! меняет только регион S. Restart сохраняет U/старт, назначает S=holder,
//! заново начинает visual и обнуляет счётчик.
//!
//! AI разрешает S и проверяет смерть до часов. Promotion читается перед
//! первым clock; строгий unsigned wrapping срок даёт не более одного тика.
//! Count++ предшествует свежим HP/MAX, setter и OnChangeStates; MAX читается
//! повторно при ограничении. После callbacks отдельно читаются часы срока.
//! Полный unsigned gain умножается на сохранённый f32-множитель и усекается
//! FISTP без промежуточного f32. HP складывается с DWORD-переполнением.
//!
//! DB содержит ID/remaining/frequency/gain (16 байт). Decode читает собственные
//! часы до трёх полей; restart не меняет этот старт. Общий клиентский getter
//! использует одно либо два чтения времени. OnUpdateProperties возвращает 1
//! без побочных эффектов. SlotMap хранит независимые записи и их поколения.

use super::fightdefense::truncate_original;
use super::heal::HEAL_SKILL_ID;
use super::shieldstate::DefenseShieldState;
use super::superheal::SUPER_HEAL_SKILL_ID;
use super::superheal2::SUPER_HEAL_2_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const HEAL_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HealState {
    skill_id: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_gain: u32,
    heal_count: u32,
}

impl HealState {
    pub(crate) const fn new(
        skill_id: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_gain: u32,
    ) -> Self {
        Self { skill_id, started_at_ms: 0, keep_time_ms, frequency_ms, hp_gain, heal_count: 0 }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let keep_time_ms = reader.read_u32()?;
        let frequency_ms = reader.read_u32()?;
        let hp_gain = reader.read_u32()?;
        let mut state = Self::new(skill_id, keep_time_ms, frequency_ms, hp_gain);
        state.started_at_ms = now_ms;
        Ok(state)
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now) as u32)
    }

    fn encoded_for_install(self) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; HEAL_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEAL_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.skill_id);
        writer.write_u32(remaining_time_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.hp_gain);
        bytes.try_into().expect("размер состояния лечения фиксирован")
    }

    pub(crate) fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    fn take_due_gain(&mut self, now_ms: u32) -> Option<u32> {
        if self.started_at_ms.wrapping_add(self.frequency_ms.wrapping_mul(self.heal_count)) >= now_ms {
            return None;
        }
        self.heal_count = self.heal_count.wrapping_add(1);
        Some(self.hp_gain)
    }

    const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
}

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
}

pub(super) fn replace_heal_state(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    storage_target: (i32, ShapeIdentity),
    skill_id: u32,
    create: impl FnOnce() -> HealState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let removed_id = if skill_id == SUPER_HEAL_SKILL_ID { HEAL_SKILL_ID } else { skill_id };
    let Some(storage) = resolve_state_move_shape(game, storage_target.0, storage_target.1) else { return false; };
    if let Some((index, _)) = storage.find_state_position(|state| state.state_id() == removed_id)
        && end_and_destroy_state_at(game, storage_target.0, storage_target.1, index).is_none()
    { return false; }
    let mut state = create();
    state.started_at_ms = now();
    let Some(user) = participant(game, source) else { return false; };
    let effect_target = if skill_id == SUPER_HEAL_2_SKILL_ID { source } else { storage_target };
    let Some(sufferer) = participant(game, effect_target) else { return false; };

    // Loop=1 не вызывает внешний callback; арена публикует готовый visual вместе с записью.
    state.heal_count = 0;
    let record = state.encoded_for_install();
    let Some(storage) = resolve_state_move_shape_mut(game, storage_target.0, storage_target.1) else { return false; };
    let key = storage.append_applied_state_record(state, &record);
    storage.set_applied_state_user(key, Some(user));
    storage.set_applied_state_sufferer(key, Some(sufferer));
    true
}

pub(crate) fn restart_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key)) {
        state.heal_count = 0;
    }
    true
}

pub(crate) fn update_stored_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    mut now: impl FnMut() -> u32,
) {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else {
        let _ = end_heal_state(game, region_id, holder, key);
        return;
    };
    let Some(health) = game.move_shape_health(target.0, target.1) else { return; };
    if health == 0 {
        let _ = end_heal_state(game, region_id, holder, key);
        return;
    }
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let promotion = target_shape.find_state_position(|state| state.state_id() == 0x142);
    let multiplier = match promotion {
        None => 1.0,
        Some((_, promotion_key)) => {
            // Native читает WORD первого ID без RTTI; чужой layout не имитируем.
            let Some(DefenseShieldState::Promotion(promotion)) =
                target_shape.applied_state::<DefenseShieldState>(promotion_key)
            else { return; };
            f32::from(promotion.heal_recover_factor()) * 0.001
        }
    };
    let checked_at_ms = now();
    let gain = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key))
        .and_then(|state| state.take_due_gain(checked_at_ms));
    if let Some(gain) = gain {
        let Some(health) = game.move_shape_health(target.0, target.1) else { return; };
        let mut next = health.wrapping_add(truncate_original(f64::from(gain) * f64::from(multiplier)) as u32);
        let Some(maximum) = game.move_shape_maximum_health(target.0, target.1) else { return; };
        if maximum < next {
            let Some(maximum) = game.move_shape_maximum_health(target.0, target.1) else { return; };
            next = maximum;
        }
        if game.set_move_shape_health(target.0, target.1, next).is_none() { return; }
        let _ = game.publish_move_shape_states(target.0, target.1);
    }
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return; }
    let checked_at_ms = now();
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key))
        .is_some_and(|state| state.expired(checked_at_ms)) {
        let _ = end_heal_state(game, region_id, holder, key);
    }
}

pub(crate) fn end_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, HEAL_STATE_BYTES)
}
