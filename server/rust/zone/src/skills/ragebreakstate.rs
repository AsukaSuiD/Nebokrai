//! Живые Begin×3/End/Restart/AI и пересчёт свойств `CRageBreakState` (0x6E),
//! а также consume-API для `CThunderSlash`. Источник: точная пара
//! `gameserver.exe` (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match),
//! исходные владельцы `appserver/skills/ragebreakstate.cpp/.h` и
//! `appserver/skills/furystate.cpp/.h`. Данные, 12-байтный codec и формулы
//! усиления — Zone `effects/attackgain.rs` (тип `{RageBreak,Fury}State`,
//! `ATTACK_GAIN_STATE_BYTES`); ниже — только живые callbacks.
//!
//! Машинные якоря тела (VA = RVA + 0x400000): ctor `0x5FD1C0`
//! (два аргумента: gain → [+0x38], keep → [+0x40]; [+0x8] и [+0x3C]
//! обнуляются) и `0x5FD240` (default); vtable `0x6612B4`; Begin триадой
//! `0x5FD370` (U, OBJECT_TYPE, j, j) / `0x5FD2C0` (U, j, j) / `0x5FD5C0`
//! (S, S; часы читаются только при U, loop1 создан без Update);
//! AI `0x5EA4C0` (строгий unsigned-срок `[+0x2C] + [+0x40] < now` →
//! vcall End); End `0x5FD420` (ICF `CTeamState::End`: visual End,
//! повторное разрешение фактического S и удаление именно этого экземпляра
//! у него через `0x4CDAB0`, держатель не подставляется); dtor `0x5FD460`;
//! Restart `0x5FD450` (только `[+0x2C] = timeGetTime()`);
//! OnUpdateProperties `0x5FD480`; OnChangeRegion `0x5D9BA0` (только запись
//! региона User); GetRemainedTime `0x605E10` (unsigned-граница и повторное
//! чтение часов); Serialize `0x5E7330` / Unserialize `0x5FD660`;
//! `CRageBreakStateVisualEffect` `0x661300`, Update `0x5FD690`.
//!
//! `CRageBreakState` ICF-разделяет Serialize/OnUpdateProperties/Unserialize/
//! AI с `CFuryState`; поэтому общие живые callbacks AttackGain-семьи
//! (`begin_primary_attack_gain_state`, `update_attack_gain_state_properties`)
//! оформлены здесь один раз и используются обоими владельцами — прежний
//! `appserver/skills/furystate.rs` делегирует им.
//!
//! ThunderSlash расходует первый ID-0x6E слот без проверки RTTI/ended:
//! End и затем destructor свежего остатка той же позиции. Расход не
//! добавляет чтения часов или UpdateProperty сверх callbacks самого
//! завершения (якоря в `skills/thunderslash.rs`).
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! (арена состояний, property-visual, удаление по ключу) реализован у
//! прежнего владельца; монстровый пересчёт OnUpdateProperties открыт
//! `AttackGainStateGame` (реализация у делегата
//! `appserver/skills/ragebreakstate.rs`). Базовый `begin_base_applied_state`
//! — это `mark_applied_state_begun` шва MoveShape, отдельный метод не нужен.

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
