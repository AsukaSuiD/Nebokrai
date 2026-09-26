//! Живые Begin×3/End/Restart/AI и пересчёт свойств `CRageBreakState` (0x6E),
//! а также consume-API для `CThunderSlash`. Данные, 12-байтный codec и
//! формулы усиления — Zone `effects/attackgain.rs` (тип `{RageBreak,Fury}State`,
//! `ATTACK_GAIN_STATE_BYTES`); здесь — только живые callbacks.
//!
//! `CRageBreakState` ICF-разделяет Serialize/OnUpdateProperties/Unserialize/AI
//! с `CFuryState` — общие callbacks AttackGain-семьи
//! (`begin_primary_attack_gain_state`, `update_attack_gain_state_properties`)
//! оформлены здесь один раз; прежний `appserver/skills/furystate.rs` делегирует.
//! Quirks: End удаляет именно этот экземпляр через RemoveState(pointer) без
//! подстановки держателя; Restart — только перевзвод часов.
//!
//! Швы: hub `statecast::StateCastGame`; монстровый пересчёт — фасад
//! `AttackGainStateGame` (делегат `appserver/skills/ragebreakstate.rs`).
//!
//! Исходные владельцы PDB: `appserver/skills/ragebreakstate.cpp/.h`,
//! `appserver/skills/furystate.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#ragebreak--cragebreak-0x6e-и-cragebreakstate

use crate::effects::{
    ATTACK_GAIN_STATE_BYTES, AttackGainState, RAGE_BREAK_STATE_ID, RageBreakState,
};
use crate::regions::ShapeIdentity;

use super::state::{AppliedState, StateKey};
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPlayer, StateCastPropertyTarget,
    state_cast_storage_participant,
};

/// Переходные фасады прежнего владельца `CGame` для AttackGain-семьи:
/// только монстровые property-операции пересчёта максимума атаки
/// (`OnUpdateProperties`); остальное покрывает `StateCastGame`.
pub trait AttackGainStateGame: StateCastGame {
    /// Максимум атаки живой цели-монстра: `find_monster_by_id` →
    /// `find_monster_property_by_origin_name` → `state_attack_bounds(..).1`.
    fn attack_gain_monster_maximum(&self, region_id: i32, monster_id: i32) -> Option<u32>;

    /// Wrapping-прибавка `property_modifiers_mut().maximum_attack` живого
    /// монстра (монстр ищется заново после чтения свойств).
    fn apply_attack_gain_monster_maximum(&mut self, region_id: i32, monster_id: i32, gain: i32);
}

/// Объектный Begin первичного экземпляра AttackGain-семьи: держатель и S
/// обязаны разрешиться; часы ставятся только при U; участники перечитываются
/// живыми (ex_id сброшен); запись арены — `encoded_for_install` прежней
/// кодировки; экземпляр добавляется в хвост.
#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub fn begin_primary_attack_gain_state<Game: StateCastGame, const ID: u32>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: AttackGainState<ID>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey>
where
    AttackGainState<ID>: AppliedState,
{
    if user.is_some() { state.restart_timer(now()); }
    let user = match user {
        Some(user) => Some(state_cast_storage_participant(game, user)?),
        None => None,
    };
    let sufferer = match sufferer {
        Some(sufferer) => Some(state_cast_storage_participant(game, sufferer)?),
        None => None,
    };
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, sufferer);
    // Между base Begin, созданием loop1 и append нет внешнего callback.
    // Ресурс создаёт каталог арены; первый пакет принадлежит UpdateProperty.
    Some(key)
}

/// Property callback семьи: visual по Sufferer, затем пересчёт живой
/// максимальной атаки цели — монстра (origin-name property снимок) или
/// игрока (u16-насыщение `apply_to_player_maximum_attack`).
pub fn update_attack_gain_state_properties<Game: AttackGainStateGame, const ID: u32>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    AttackGainState<ID>: AppliedState,
{
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<AttackGainState<ID>>(
        region_id, holder, key, StateCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<AttackGainState<ID>>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(maximum) = game.attack_gain_monster_maximum(target_region, target.id) {
            let gain = state.truncated_gain(maximum);
            game.apply_attack_gain_monster_maximum(target_region, target.id, gain);
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

/// Объектный Begin первичного `CRageBreakState`: обязателен только S —
/// путь DB-restart (`Begin(NULL, holder)`) идёт тем же телом без U.
#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub fn begin_primary_rage_break_state<Game: StateCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: RageBreakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    sufferer?;
    begin_primary_attack_gain_state(game, holder_region, holder, user, sufferer, state, now)
}

/// Расход первого ID-0x6E живого вектора: End и destructor той же позиции.
/// Результат означает наличие первого ID, а не успешное удаление его End.
pub fn consume_rage_break_state<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
) -> bool {
    let Some((position, _)) = game.resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == RAGE_BREAK_STATE_ID))
    else { return false; };
    let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    true
}

/// Полный End по ключу: visual End по держателю, повторное разрешение
/// фактического Sufferer и удаление именно этого ключа у него.
pub fn end_rage_break_state_key<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some(sufferer) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, sufferer, ATTACK_GAIN_STATE_BYTES)
}

/// OnUpdateProperties конкретного ключа — общий callback AttackGain-семьи.
pub fn update_rage_break_state_properties<Game: AttackGainStateGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    update_attack_gain_state_properties::<Game, RAGE_BREAK_STATE_ID>(game, region_id, holder, key, now)
}

/// DB-restart прежнего экземпляра (не timer-only Restart `0x5FD450`, а
/// путь `Begin(NULL, holder)`): base Begin помечает слот, visual заменяется
/// loop1; часы и User сохраняются.
pub fn restart_rage_break_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).is_none()
    { return false; }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    let _ = game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1));
    true
}

/// AI конкретного ключа: строгое истечение срока завершает состояние —
/// машинный переход `jbe` AI `0x5EA4C0` на vcall End.
pub fn update_rage_break_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_rage_break_state_key(game, region_id, holder, key)
}
