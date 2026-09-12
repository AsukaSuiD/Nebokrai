//! Общие callbacks, доступ и клиентское представление состояний GameServer.
//! Источник: gameserver.exe/GameServer.pdb, appserver/states/state.h/.cpp
//! и операции состояний appserver/moveshape.cpp.
//!
//! SlotMap сохраняет идентичность экземпляра, но не продлевает жизнь удалённого
//! payload. После callback позиция разрешается заново; удаление не уплотняет
//! контейнер. Часы и игровые правила принадлежат конкретному состоянию.
//! User/Sufferer разрешаются независимо от держателя: игрок — глобально,
//! остальные формы — через регион. Ключ нельзя применять к другой арене.
//! Identity-поиск проверяет реестр и живой объект, а не пространственный view:
//! координаты и таблица figure не определяют существование CMoveShape.
//! В ещё не перенесённых первичных установках сохраняется прежняя привязка
//! к держателю; общий restart не заменяет эти конкретные Begin.
//!
//! NULL-user restart сохраняет источник и timestamp; отказ Begin не удаляет
//! запись. Visual имеет собственный ended и принадлежит экземпляру состояния.
//! Клиентские getters читаются по порядку без Serialize и поиска записи по ID;
//! их параметры могут отличаться от BFE03 конкретного визуального эффекта.
//! Базовые getters возвращают ноль, а неизвестный AI не становится no-op.
//! Destructor — отдельный переход, а не повтор End: его visual вызывается
//! перед освобождением того же ключа. Прямой расход не меняет ended и не
//! вызывает UpdateProperty; RemoveState добавляет этот callback после удаления.
//! Неперенесённые контракты сохранены адресно в RAW ниже.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::{AppliedState, CMoveShape, StateData, StateKey};
use crate::gameserver::appserver::skills;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillLifecycle;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, RegionShapeResolver};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const STATE_IDENTITY_BYTES: usize = 16;

type StateAi<Runtime> = fn(&mut CGame, i32, ShapeIdentity, StateKey, &mut Runtime);

type StateRestart = fn(&mut CGame, i32, ShapeIdentity, StateKey, bool, &mut dyn FnMut() -> u32) -> bool;

type StateProperty = fn(&mut CGame, i32, ShapeIdentity, StateKey, &mut dyn FnMut() -> u32) -> bool;

// 0x005D9BA0 меняет только user-region; все восемь BF vtable
// (0x0065F5F4..0x0065F8CC), а также Fury/Wangsheng используют этот слот.
fn set_state_user_region(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) {
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_user_region(key, region_id);
    }
}

// У блокировок Blind/Rush, тумана и RestoreHp/RestoreMp переход держателя
// меняет регион S перед Begin, но не переносит регион источника вместе с ним.
fn set_state_sufferer_region(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) {
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer_region(key, region_id);
    }
}

fn set_defense_shield_region(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.defense_shield(key)) else { return; };
    if matches!(state, skills::shieldstate::DefenseShieldState::Promotion(_)) {
        set_state_user_region(game, region_id, holder, key);
    } else {
        set_state_sufferer_region(game, region_id, holder, key);
    }
}

// Оба GodBless: vtable0x00661864/0x00660074 +0x2C=0x00601660,
// записывающий user-region и sufferer-region независимо от варианта Begin.
fn set_god_bless_regions(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) {
    set_state_user_region(game, region_id, holder, key);
    set_state_sufferer_region(game, region_id, holder, key);
}

// ImproveExp наследует S-only SetRegion, остальные шесть Script — U-only.
fn set_script_state_region(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) {
    let Some(improve_exp) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<crate::gameserver::appserver::scriptstate::ScriptMoveState>(key))
        .map(|state| state.is_improve_exp())
    else { return; };
    if improve_exp { set_state_sufferer_region(game, region_id, holder, key); }
    else { set_state_user_region(game, region_id, holder, key); }
}

/// CMoveShape::UpdateProperty (0x004CFB60): обнуление 25 LONG, затем
/// virtual +0x24 в исходном порядке. Граница фиксируется один раз, слот
/// читается заново перед callback. IsEnded и return callback не фильтруют
/// проход; удалённые слоты не уплотняются, новые за границей не посещаются.
pub(crate) fn update_move_shape_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    now: &mut dyn FnMut() -> u32,
) -> Option<()> {
    let shape = resolve_state_move_shape_mut(game, region_id, holder)?;
    shape.reset_property_modifiers();
    let initial_len = shape.state_slot_count();
    for index in 0..initial_len {
        let shape = resolve_state_move_shape(game, region_id, holder)?;
        if index >= shape.state_slot_count() { break; }
        let callback = shape.state_at(index).map(|(key, state)| (key, state_property(state)));
        if let Some((key, update)) = callback {
            update(game, region_id, holder, key, now);
        }
    }
    Some(())
}

#[derive(Clone, Copy)]
pub(crate) enum StatePropertyTarget {
    User,
    Sufferer,
}

/// DecodeExStates (0x004D1B18) назначает sufferer type/id до Begin,
/// сохраняя нулевой region. GetSufferer (0x005DBFD0) разрешает player
/// глобально по ID, остальных — через сохранённый region. Поэтому ни
/// state.ended, ни отсутствие Begin сами по себе не запрещают lookup.
pub(crate) fn resolve_applied_state_sufferer(
    game: &CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, region_id, holder)?;
    shape.applied_state_data(key)?;
    let (region, target) = shape.applied_state_sufferer(key)?;
    let target_shape = resolve_state_move_shape(game, region, target)?;
    Some((target_shape.shape().get_region_id(), target))
}

/// GetUser использует сохранённый источник конкретного Begin. Unserialize
/// сохраняет NULL user и не заменяет его sufferer даже после регистрации.
pub(crate) fn resolve_applied_state_user(
    game: &CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, region_id, holder)?;
    shape.applied_state_data(key)?;
    let (region, target) = shape.applied_state_user(key)?;
    let target_shape = resolve_state_move_shape(game, region, target)?;
    Some((target_shape.shape().get_region_id(), target))
}

/// Общая player-only часть concrete property owners без внешних callbacks
/// внутри формулы. Копируется только Copy-параметр, не владелец состояния,
/// и только после lookup живого sufferer; неподдерживаемый type даёт true.
pub(crate) fn update_player_state_properties<T: AppliedState + Copy>(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    update: impl FnOnce(T, &mut crate::gameserver::appserver::player::CPlayer),
) -> bool {
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    if target.object_type == 400 {
        let Some(state) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<T>(key)).copied()
        else { return false; };
        if let Some(player) = game.find_player_mut(target.id) {
            update(state, player);
        }
    }
    true
}

/// Общий wire/base-tail для стандартных property visuals: их concrete
/// Update(0) проверяет visual.ended, заново разрешает адресата и лишь затем
/// читает remaining-time. Отсутствующий или завершённый target не мешает
/// базовому tail; отсутствующий visual вообще не получает вызова.
pub(crate) fn update_property_state_visual<T: AppliedState>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    target: StatePropertyTarget,
    now: &mut dyn FnMut() -> u32,
    client_time: impl FnOnce(&T, &mut dyn FnMut() -> u32) -> u32,
) -> bool {
    let Some(ended) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_visual_ended(key))
    else { return false; };
    if !ended {
        let target = match target {
            StatePropertyTarget::User => resolve_applied_state_user(game, region_id, holder, key),
            StatePropertyTarget::Sufferer => resolve_applied_state_sufferer(game, region_id, holder, key),
        };
        if let Some((target_region, identity)) = target {
            let snapshot = resolve_state_move_shape(game, region_id, holder).and_then(|shape| {
                let data = shape.applied_state_data(key)?;
                Some((data.state_id(), client_time(shape.applied_state::<T>(key)?, now)))
            });
            if let Some((state_id, remaining)) = snapshot {
                let mut message = CMessage::new(0x000b_fe03);
                message.add_long(identity.object_type);
                message.add_long(identity.id);
                message.add_ulong(state_id);
                message.add_ulong(remaining);
                message.add_ulong(0);
                let _ = game.send_move_shape_around(target_region, identity, &message);
            }
        }
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    true
}

/// Стандартный visual Update(1), подтверждённый God/Fog (0x00601880/
/// 0x00608660). Нет ресурса — нет вызова; ended/NULL target подавляют только
/// пакет, но не base tail. Это не state End и не запись state.ended.
pub(crate) fn update_applied_state_end_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    target: StatePropertyTarget,
) -> bool {
    let Some(ended) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_visual_ended(key))
    else { return false };
    if !ended {
        let target = match target {
            StatePropertyTarget::User => resolve_applied_state_user(game, region_id, holder, key),
            StatePropertyTarget::Sufferer => resolve_applied_state_sufferer(game, region_id, holder, key),
        };
        if let Some((target_region, target)) = target {
            if let Some(state_id) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state_data(key)).map(StateData::state_id) {
                let mut message = CMessage::new(0x000b_fe04);
                message.add_long(target.object_type);
                message.add_long(target.id);
                message.add_ulong(state_id);
                let _ = game.send_move_shape_around(target_region, target, &message);
            }
        }
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    true
}

/// CHBY/CExState GetRemainedTime (0x005DA030), включая самостоятельные
/// чтения часов и специальное значение 1 после ненулевого срока.
pub(crate) fn change_body_client_time(start: u32, keep: u32, now: &mut dyn FnMut() -> u32) -> u32 {
    let deadline = start.wrapping_add(keep);
    if keep != 0 && deadline <= now() { return 1; }
    if deadline <= now() { return 0; }
    deadline.wrapping_sub(now())
}

/// CExStateNew/CNotDisappearAfterDead (0x005D6320): нулевой срок
/// бессрочен и не читает часы, положительный читает одно либо два значения.
pub(crate) fn extended_client_time(start: u32, keep: u32, now: &mut dyn FnMut() -> u32) -> u32 {
    if keep == 0 { return 0; }
    let deadline = start.wrapping_add(keep);
    if deadline <= now() { return 0; }
    deadline.wrapping_sub(now())
}

/// Объектный CState::Begin (0x005DBD70) с NULL user. Часы и прежний user
/// сохраняются; concrete owner обновляет своё представление sufferer.
/// Возвращаемое значение сообщает наличие экземпляра, не native return 0.
pub(crate) fn begin_base_applied_state(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) -> bool {
    resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
}

pub(crate) fn begin_applied_state_visual(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, loop_value: i32) -> bool {
    resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, loop_value))
}

pub(crate) fn update_applied_state_visual_base(game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey) -> bool {
    resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.update_applied_state_visual_base(key))
}

/// CMoveShape::StartAllStates (0x004CE050). SetRegion может выполнить End;
/// Begin получает новое чтение той же позиции, а не прежний ключ. Длина
/// живая, результат Begin игнорируется: отказ не означает удаление состояния.
/// Простые SetRegion 0x005D9BA0/0x005E3B30/0x00601660 меняют только регион
/// сохранённого User, Sufferer либо обоих, не подменяя их identity держателем.
/// Отдельный source в payload периодических состояний здесь не изменяется.
pub(crate) fn start_move_shape_states(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    after_death: bool,
    now: &mut dyn FnMut() -> u32,
) -> Option<()> {
    let actual_region = resolve_state_move_shape(game, region_id, holder)?.shape().get_region_id();
    game.find_region(actual_region)?;
    let mut index = 0;
    loop {
        let shape = resolve_state_move_shape(game, region_id, holder)?;
        if index >= shape.state_slot_count() { break; }
        let selected = shape.state_at(index).filter(|(_, state)| {
            !after_death || !matches!(state.state_id(), 100_007..=100_012 | 0x32 | 0x33 | 0x38)
        }).map(|(key, state)| (key, state_set_region(state)));
        if let Some((key, set_region)) = selected {
            set_region(game, actual_region, holder, key);
            let next = resolve_state_move_shape(game, region_id, holder)?
                .state_at(index).map(|(key, state)| (key, state_restart(state)));
            if let Some((key, restart)) = next {
                restart(game, actual_region, holder, key, after_death, now);
            }
        }
        index += 1;
    }
    Some(())
}

/// Базовый End 0x005DBCE0: ended → GetUser → RemoveState(pointer).
/// Sufferer отдельных BFAttribute отличается и не подставляется вместо user.
/// При NULL user запись лишь помечается ended; внешнее удаление остатка
/// принадлежит ClearAllStates либо конкретному replacement caller-у.
pub(crate) fn end_base_applied_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    bytes: usize,
) -> bool {
    if !resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_ended(key)) { return false; }
    let Some((user_region, user)) = resolve_applied_state_user(game, region_id, holder, key)
    else { return false };
    remove_applied_state_from(game, region_id, holder, key, (user_region, user), bytes)
}

/// CMoveShape::RemoveState(pointer) 0x004CDAB0: поиск того же объекта,
/// освобождение/NULL-слот и затем UpdateProperty владельца найденной записи.
/// В Rust уникальный StateKey действует только внутри одной арены.
pub(crate) fn remove_applied_state_from(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    (target_region, target): (i32, ShapeIdentity), _bytes: usize,
) -> bool {
    // Ключ локален арене. RemoveState(pointer) другого User не должен удалить
    // совпавший численно ключ чужого контейнера: такого указателя там нет.
    let same_holder = resolve_state_move_shape(game, region_id, holder)
        .zip(resolve_state_move_shape(game, target_region, target))
        .is_some_and(|(arena, target)| std::ptr::eq(arena, target));
    if !same_holder { return false; }
    let removed = destroy_move_shape_state(game, region_id, holder, key);
    if removed {
        let _ = game.update_move_shape_properties(target_region, target);
    }
    removed
}

/// Прямое уничтожение выбранного объекта: concrete destructor до удаления
/// payload и DB-проекции. Состояние остаётся доступным его visual; ended,
/// User::RemoveState и пересчёт свойств этим переходом не вызываются.
pub(crate) fn destroy_move_shape_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(callback) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).map(state_destructor)
    else { return false; };
    if let Some(callback) = callback { callback(game, region_id, holder, key); }
    resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.remove_applied_state(key).is_some())
}

/// Прямой virtual +0x1C, без искусственного deadline и без удержания payload
/// на стеке во время callback. bool сообщает удаление, не значение IsEnded.
pub(crate) fn end_move_shape_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(end) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).map(state_end)
    else { return false };
    end(game, region_id, holder, key)
}

/// Общий хвост ClearAllStates/CastCure: вызов выбранного End и затем
/// destructor оставшегося в этой же позиции объекта. Внешний UpdateProperty
/// сюда не входит; RemoveState внутри конкретного End выполняет свой сам.
pub(crate) fn end_and_destroy_state_at(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    index: usize,
) -> Option<()> {
    let key = resolve_state_move_shape(game, region_id, holder)?
        .state_at(index).map(|(key, _)| key);
    if let Some(key) = key {
        end_move_shape_state(game, region_id, holder, key);
        let remaining = resolve_state_move_shape(game, region_id, holder)?
            .state_at(index).map(|(remaining, _)| remaining);
        if let Some(remaining) = remaining {
            destroy_move_shape_state(game, region_id, holder, remaining);
        }
    }
    Some(())
}

/// CMoveShape::RemoveState(tagSkillID), moveshape.cpp:744, 0x004CDB20.
/// Каждый выбранный ID получает End (0x004CDB62), затем destructor свежего
/// остатка той же позиции и самостоятельный UpdateProperty (0x004CDB93).
/// Даже если End уже удалил запись и обновил свойства, внешний Update остаётся.
/// Длина перечитывается после callback: новые хвостовые состояния участвуют
/// в этом проходе; пропуски не уплотняются, End-result и ended не фильтруются.
pub(crate) fn remove_move_shape_states_by_id(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state_id: u32,
) -> Option<()> {
    let mut index = 0;
    loop {
        let shape = resolve_state_move_shape(game, region_id, holder)?;
        if index >= shape.state_slot_count() { break; }
        let selected = shape.state_at(index)
            .is_some_and(|(_, state)| state.state_id() == state_id);
        if selected {
            end_and_destroy_state_at(game, region_id, holder, index)?;
            let _ = game.update_move_shape_properties(region_id, holder);
        }
        index += 1;
    }
    Some(())
}

/// CMoveShape::ClearAllStates (0x004CF090). Три death-прохода сохраняют
/// native исключения и самостоятельные UpdateProperty первых двух проходов.
/// После End перечитывается именно позиция, не прежний ключ: callback мог
/// удалить или заменить её. Destructor-only хвост не публикует второй End.
/// Без death-фильтра длина тоже живая; в конце безопасно освобождается массив
/// вместо оставленных native operator_delete висячих границ vector.
pub(crate) fn clear_move_shape_states(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    after_death: bool,
) -> Option<()> {
    let passes = if after_death { 3 } else { 1 };
    for pass in 0..passes {
        let mut index = 0;
        loop {
            let shape = resolve_state_move_shape(game, region_id, holder)?;
            if index >= shape.state_slot_count() { break; }
            let selected = if let Some((key, state)) = shape.state_at(index) {
                let should_end = if !after_death {
                    true
                } else if pass == 0 {
                    matches!(state, StateData::ChangeBody(state) if !state.continue_after_death)
                } else if pass == 1 {
                    matches!(state, StateData::Undead(state) if state.disappear_after_dead == 1)
                } else {
                    match state.state_id() {
                        100_007..=100_012 | 0x37 | 0x38 => false,
                        0x32 | 0x33 => {
                            if let Some(region) = game.find_region(shape.shape().get_region_id()) {
                                // Невиртуальный CRegion::GetCell, не war GetSecurity.
                                let x = shape.shape().get_tile_x().ok()?;
                                let y = shape.shape().get_tile_y().ok()?;
                                region.base().region.get_cell(x, y).ok()?
                                    .is_some_and(|cell| cell.security().value() == 0)
                            } else {
                                true
                            }
                        }
                        _ => true,
                    }
                };
                should_end.then_some(key)
            } else { None };
            if selected.is_some() {
                end_and_destroy_state_at(game, region_id, holder, index)?;
                if after_death && pass < 2 {
                    let _ = game.update_move_shape_properties(region_id, holder);
                }
            }
            index += 1;
        }
    }
    if !after_death {
        resolve_state_move_shape_mut(game, region_id, holder)?.release_state_slots();
    }
    Some(())
}

/// Исходный массив перечитывается после каждого callback: добавленное состояние
/// не расширяет границу этого прохода, а удалённое не оживает из снимка ключей.
pub(crate) fn update_move_shape_states<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
) -> Option<()> {
    let changed = resolve_state_move_shape_mut(game, region_id, identity)?.compact_state_slots();
    if changed {
        let _ = game.update_move_shape_properties(region_id, identity);
    }
    let initial_len = resolve_state_move_shape(game, region_id, identity)?.state_slot_count();
    for index in 0..initial_len {
        let shape = resolve_state_move_shape(game, region_id, identity)?;
        let current_len = shape.state_slot_count();
        if index >= current_len {
            log_state_array_change(game, shape.shape(), b"GS0121", None, initial_len, current_len, index);
            return Some(());
        }
        let Some((key, state)) = shape.state_at(index) else { continue };
        let state_id = state.state_id();
        let ai = state_ai::<Runtime>(state);
        ai(game, region_id, identity, key, runtime);
        let shape = resolve_state_move_shape(game, region_id, identity)?;
        let current_len = shape.state_slot_count();
        if current_len != initial_len {
            log_state_array_change(game, shape.shape(), b"GS0122", Some(state_id), initial_len, current_len, index);
        }
    }
    Some(())
}

fn log_state_array_change(
    game: &CGame,
    shape: &CShape,
    string_id: &[u8],
    state_id: Option<u32>,
    initial_len: usize,
    current_len: usize,
    index: usize,
) {
    use crate::gameserver::gameserver::game::{format_legacy_mixed, LegacyFormatArgument};
    let mut args = vec![LegacyFormatArgument::Bytes(shape.base_object().get_name())];
    if let Some(state_id) = state_id {
        args.push(LegacyFormatArgument::Word32(state_id));
    }
    args.extend([initial_len, current_len, index].map(|value| LegacyFormatArgument::Word32(value as u32)));
    let text = format_legacy_mixed(game.get_string_by_id(string_id), &args, 0xff);
    crate::public::tools::put_debug_string(&text);
}

/// Заимствованная клиентская проекция одного живого экземпляра, без DB Serialize.
/// Только Team дописывает имя после общей тройки ID/time/additional.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StateClientRecord<'a> {
    pub(crate) time: i32,
    pub(crate) additional: u32,
    pub(crate) team_name: Option<&'a [u8]>,
}

impl StateClientRecord<'_> {
    fn timed(time: i32) -> Self {
        Self { time, ..Self::default() }
    }
}

// Один список задаёт AI/End/Begin/destructor/OnUpdateProperties/SetRegion, клиентскую
// проекцию и остаточное состояние visual после runtime Begin. Внутри общего
// lifecycle-семейства client-clause относится к конкретному typed payload.
macro_rules! state_callbacks {
    (@destructor) => { None };
    (@destructor $destructor:path) => { Some($destructor) };
    (@region) => { set_state_user_region };
    (@region $set_region:path) => { $set_region };
    (@visual) => { |_state: &StateData| Some((1, false)) };
    (@visual $visual:expr) => { $visual };
    (@client $payload:expr, $team_count:expr, $clock:expr) => { StateClientRecord::default() };
    (@client $payload:expr, $team_count:expr, $clock:expr;
        |$state:ident, $team:ident, $now:ident| $client:expr) => {{
        let $state = $payload;
        let $team = $team_count;
        let $now = $clock;
        $client
    }};
    ($($(
        StateData::$variant:ident(_)
        $(; client = |$state:ident, $team:ident, $now:ident| $client:block)?
    )|+ => ($ai:expr, $end:path, $restart:path, $property:expr $(, $set_region:path)?) $(; visual = $visual:expr)? $(; destructor = $destructor:path)?),+ $(,)?) => {
        fn state_ai<Runtime: GameMainLoopRuntime>(state: &StateData) -> StateAi<Runtime> {
            match state { $($(StateData::$variant(_))|+ => $ai),+ }
        }

        fn state_end(state: &StateData) -> fn(&mut CGame, i32, ShapeIdentity, StateKey) -> bool {
            match state { $($(StateData::$variant(_))|+ => $end),+ }
        }

        fn state_destructor(state: &StateData) -> Option<fn(&mut CGame, i32, ShapeIdentity, StateKey)> {
            match state { $($(StateData::$variant(_))|+ => state_callbacks!(@destructor $($destructor)?)),+ }
        }

        fn state_restart(state: &StateData) -> StateRestart {
            match state { $($(StateData::$variant(_))|+ => $restart),+ }
        }

        fn state_property(state: &StateData) -> StateProperty {
            match state { $($(StateData::$variant(_))|+ => $property),+ }
        }

        pub(crate) fn state_client_record<'a>(
            state: &'a StateData,
            team_member_count: usize,
            now: &mut dyn FnMut() -> u32,
        ) -> StateClientRecord<'a> {
            match state {
                $($(StateData::$variant(_payload) => state_callbacks!(@client
                    _payload, team_member_count, &mut *now
                    $(; |$state, $team, $now| $client)?
                )),+),+
            }
        }

        pub(crate) fn registered_runtime_state_visual(state: &StateData) -> Option<super::visualeffect::CVisualEffect> {
            let plan: fn(&StateData) -> Option<(i32, bool)> = match state {
                $($(StateData::$variant(_))|+ => state_callbacks!(@visual $($visual)?)),+
            };
            let (loop_value, updated) = plan(state)?;
            let mut visual = super::visualeffect::CVisualEffect::new();
            visual.begin_visual_effect(loop_value);
            if updated { visual.update_visual_effect(); }
            Some(visual)
        }

        fn state_set_region(state: &StateData) -> fn(&mut CGame, i32, ShapeIdentity, StateKey) {
            match state { $($(StateData::$variant(_))|+ => state_callbacks!(@region $($set_region)?)),+ }
        }
    };
}

state_callbacks! {
    StateData::PersistentAgility(_) => (
        |_, _, _, _, _| {},
        skills::agilitystate::end_persistent_agility_state,
        skills::agilitystate::restart_persistent_agility_state,
        skills::agilitystate::update_persistent_agility_state_properties
    ),
    StateData::TaiJi(_) => (
        |_, _, _, _, _| {},
        skills::taijistate::end_tai_ji_state,
        skills::taijistate::restart_tai_ji_state,
        skills::taijistate::update_tai_ji_state_properties
    ); visual = |_state| None,
    StateData::EnlargeFullMiss(_) => (
        |_, _, _, _, _| {},
        skills::enlargefullmissstate::end_enlarge_full_miss_state,
        skills::enlargefullmissstate::restart_enlarge_full_miss_state,
        skills::enlargefullmissstate::update_enlarge_full_miss_state_properties
    ); visual = |_state| None,
    StateData::EnlargeMaxHp(_) => (
        |_, _, _, _, _| {},
        skills::enlargemaxhpstate::end_enlarge_max_hp_state,
        skills::enlargemaxhpstate::restart_enlarge_max_hp_state,
        skills::enlargemaxhpstate::update_enlarge_max_hp_state_properties
    ); visual = |_state| None,
    StateData::EnlargeMaxMp(_) => (
        |_, _, _, _, _| {},
        skills::enlargemaxmpstate::end_enlarge_max_mp_state,
        skills::enlargemaxmpstate::restart_enlarge_max_mp_state,
        skills::enlargemaxmpstate::update_enlarge_max_mp_state_properties
    ); visual = |_state| None,
    StateData::Origin(_) => (
        |_, _, _, _, _| {},
        skills::originstate::end_origin_state,
        skills::originstate::restart_origin_state,
        skills::originstate::update_origin_state_properties
    ); visual = |_state| None,
    StateData::MeteorArrow(_); client = |state, _team, _now| { StateClientRecord { additional: state.additional_data() as u32, ..StateClientRecord::default() } } => (
        |_, _, _, _, _| {},
        skills::meteorarrowstate::end_meteor_arrow_state,
        skills::meteorarrowstate::restart_meteor_arrow_state,
        |_, _, _, _, _| true
    ); destructor = skills::meteorarrowstate::destroy_meteor_arrow_state_visual,
    StateData::EnergyHolding(_) => (
        |_, _, _, _, _| {},
        skills::energyholdingstate::end_energy_holding_state,
        skills::energyholdingstate::restart_energy_holding_state,
        |_, _, _, _, _| true
    ),
    StateData::SoulCollect(_); client = |state, _team, _now| { StateClientRecord { additional: state.souls() as u32, ..StateClientRecord::default() } } => (
        |_, _, _, _, _| {},
        skills::soulcollectstate::end_soul_collect_state,
        skills::soulcollectstate::restart_soul_collect_state,
        |_, _, _, _, _| true
    ),
    StateData::Swordship(_) => (
        |_, _, _, _, _| {},
        skills::swordshipstate::end_swordship_state,
        skills::swordshipstate::restart_swordship_state,
        skills::swordshipstate::update_swordship_state_properties
    ); visual = |_state| None,
    StateData::WuXing(_) => (
        |_, _, _, _, _| {},
        skills::wuxingstate::end_wuxing_state,
        skills::wuxingstate::restart_wuxing_state,
        skills::wuxingstate::update_wuxing_state_properties
    ); visual = |_state| None,
    StateData::Agility2(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::agilitystate2::update_agility_state_2(game, region, target, key, runtime.now_milliseconds());
        },
        skills::agilitystate2::end_agility_state_2,
        skills::agilitystate2::restart_agility_state_2,
        skills::agilitystate2::update_agility_state_2_properties
    ); visual = |_state| Some((0, true)),
    StateData::Callosity(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::callositystate::update_callosity_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::callositystate::end_callosity_state_key,
        skills::callositystate::restart_callosity_state,
        skills::callositystate::update_callosity_state_properties
    ),
    StateData::Hearten(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::heartenstate::update_hearten_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::heartenstate::end_hearten_state,
        skills::heartenstate::restart_hearten_state,
        skills::heartenstate::update_hearten_state_properties
    ),
    StateData::RageBreak(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::ragebreakstate::update_rage_break_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::ragebreakstate::end_rage_break_state_key,
        skills::ragebreakstate::restart_rage_break_state,
        skills::ragebreakstate::update_rage_break_state_properties
    ),
    StateData::Pillar(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::pillarstate::update_pillar_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::pillarstate::end_pillar_state,
        skills::pillarstate::restart_pillar_state,
        |_, _, _, _, _| true
    ),
    StateData::TianShenXiaFan(_) => (
        |game, region, target, key, runtime| {
            skills::tianshenxiafanstate::update_tian_shen_xia_fan_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::tianshenxiafanstate::end_tian_shen_xia_fan_state,
        skills::tianshenxiafanstate::restart_tian_shen_xia_fan_state,
        skills::tianshenxiafanstate::update_tian_shen_xia_fan_state_properties
    ),
    StateData::Wangsheng(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::wangshengstate::update_wangsheng_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::wangshengstate::end_wangsheng_state,
        skills::wangshengstate::restart_wangsheng_state,
        skills::wangshengstate::update_wangsheng_state_properties
    ),
    StateData::Blind(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) }
    | StateData::Rush(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) }
    | StateData::Rush2(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) }
    | StateData::KnockOut(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) }
    | StateData::KnightCut(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) }
    | StateData::SpiderWeb(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) }
    | StateData::Seal(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) }
    | StateData::Strike(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::blindstate::update_blind_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::blindstate::end_blind_state,
        skills::blindstate::restart_blind_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ); visual = |state| Some((1, matches!(state, StateData::Blind(_) | StateData::Rush(_) | StateData::Rush2(_)))),
    StateData::Heal(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::healstate::update_stored_heal_state(game, region, target, key, || runtime.now_milliseconds());
        },
        skills::healstate::end_heal_state,
        skills::healstate::restart_heal_state,
        |_, _, _, _, _| true
    ),
    StateData::PoisonArrow(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            super::poison::update_poison_state::<0x21e, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::poisonarrowstate::PoisonArrowState>,
        super::periodicattack::restart_periodic_attack_state::<skills::poisonarrowstate::PoisonArrowState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::SpiderPoison(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            super::poison::update_poison_state::<0x191, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::spiderpoisonstate::SpiderPoisonState>,
        super::periodicattack::restart_periodic_attack_state::<skills::spiderpoisonstate::SpiderPoisonState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::SpriteBurn(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            super::poison::update_poison_state::<0x1a6, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::spriteburnstate::SpriteBurnState>,
        super::periodicattack::restart_periodic_attack_state::<skills::spriteburnstate::SpriteBurnState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::BloodLoss(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::bloodlossstate::update_blood_loss_state(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::bloodlossstate::BloodLossState>,
        super::periodicattack::restart_periodic_attack_state::<skills::bloodlossstate::BloodLossState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::LeafCut(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::leafcutstate::update_leaf_cut_state::<0x6b, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::leafcutstate::LeafCutState>,
        super::periodicattack::restart_periodic_attack_state::<skills::leafcutstate::LeafCutState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::LeafCut2(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::leafcutstate::update_leaf_cut_state::<0x80, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::leafcutstate2::LeafCutState2>,
        super::periodicattack::restart_periodic_attack_state::<skills::leafcutstate2::LeafCutState2>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::LeafCut3(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::leafcutstate::update_leaf_cut_state::<0x8f, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::leafcutstate3::LeafCutState3>,
        super::periodicattack::restart_periodic_attack_state::<skills::leafcutstate3::LeafCutState3>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::Kerosene(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            super::poison::update_poison_state::<0xf1, _>(game, region, target, key, runtime);
        },
        super::periodicattack::end_periodic_attack_state::<skills::kerosenestate::KeroseneState>,
        super::periodicattack::restart_periodic_attack_state::<skills::kerosenestate::KeroseneState>,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::Cure(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::curestate::update_cure_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::curestate::end_cure_state_key,
        skills::curestate::restart_cure_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::BossBlueQuake(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::bossbluequakestate::update_boss_blue_quake_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::bossbluequakestate::end_boss_blue_quake_state,
        skills::bossbluequakestate::restart_boss_blue_quake_state,
        |_, _, _, _, _| true
    ),
    StateData::BoaLock(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::boalockstate::update_boa_lock_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::boalockstate::end_boa_lock_state,
        skills::boalockstate::restart_boa_lock_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::GodBless(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::godblessstate::update_god_bless_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::godblessstate::end_god_bless_state,
        skills::godblessstate::restart_god_bless_state,
        skills::godblessstate::update_god_bless_state_properties,
        set_god_bless_regions
    ); visual = |state| Some((if matches!(state, StateData::GodBless(state) if state.skill_id() == skills::godblessstate2::GOD_BLESS_STATE_2_ID) { 0 } else { 1 }, false)),
    StateData::Roar(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::roarstate::update_roar_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::roarstate::end_roar_state,
        skills::roarstate::restart_roar_state,
        skills::roarstate::update_roar_state_properties,
        set_state_sufferer_region
    ),
    StateData::Weak(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::weakstate::update_weak_state(game, region, target, key, &mut || runtime.now_milliseconds());
        },
        skills::weakstate::end_weak_state,
        skills::weakstate::restart_weak_state,
        skills::weakstate::update_weak_state_properties,
        skills::weakstate::set_weak_state_region
    ),
    StateData::Fury(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::furystate::update_fury_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::furystate::end_fury_state,
        skills::furystate::restart_fury_state,
        skills::furystate::update_fury_state_properties
    ),
    StateData::BossBlueFury(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::bossbluefurystate::update_boss_blue_fury_state(game, region, target, key, || runtime.now_milliseconds());
        },
        skills::bossbluefurystate::end_boss_blue_fury_state,
        skills::bossbluefurystate::restart_boss_blue_fury_state,
        skills::bossbluefurystate::update_boss_blue_fury_state_properties
    ),
    StateData::PoisonFog(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::poisonfogstate::update_poison_fog_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::poisonfogstate::end_poison_fog_state,
        skills::poisonfogstate::restart_poison_fog_state,
        skills::poisonfogstate::update_poison_fog_state_properties,
        set_state_sufferer_region
    ),
    StateData::BattleFairyAttribute(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::battlefairyattributestate::update_battle_fairy_attribute_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::battlefairyattributestate::end_battle_fairy_attribute_state,
        skills::battlefairyattributestate::restart_battle_fairy_attribute_state,
        skills::battlefairyattributestate::update_battle_fairy_attribute_state_properties
    ),
    StateData::DefenseShield(_); client = |state, _team, now| {
        use skills::shieldstate::DefenseShieldState;
        let (time, additional) = match state {
            DefenseShieldState::Life(state) => (state.client_time(now), state.life() as u32),
            DefenseShieldState::Machine(state) => (state.client_time(now), state.life() as u32),
            DefenseShieldState::Mana(state) => (state.client_time(now), state.life() as u32),
            DefenseShieldState::Promotion(state) => (state.client_time(now), 0),
        };
        StateClientRecord { time, additional, team_name: None }
    } => (
        |game, region, target, key, runtime| {
            skills::shieldstate::update_defense_shield(game, region, target, key, runtime.now_milliseconds());
        },
        skills::shieldstate::end_defense_shield,
        skills::shieldstate::restart_defense_shield_state,
        |_, _, _, _, _| true,
        set_defense_shield_region
    ); visual = |state| { let once = matches!(state, StateData::DefenseShield(skills::shieldstate::DefenseShieldState::Promotion(_))); Some((if once { 0 } else { 1 }, once)) },
    StateData::DaubPoison(_); client = |state, _team, now| { StateClientRecord::timed(state.client_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            skills::daubpoisonstate::update_daub_poison_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::daubpoisonstate::end_daub_poison_state,
        skills::daubpoisonstate::restart_daub_poison_state,
        |_, _, _, _, _| true
    ),
    StateData::AutomaticRestore(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_automatic_restore_state(region, target, key, runtime);
        },
        CGame::end_move_shape_automatic_restore_state,
        super::automaticrestore::restart_automatic_restore_state,
        |_, _, _, _, _| true
    ),
    StateData::ConsumableRestore(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            game.update_move_shape_consumable_restore_state(region, target, key, runtime);
        },
        CGame::end_move_shape_consumable_restore_state,
        crate::gameserver::appserver::restorestate::restart_consumable_restore_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ),
    StateData::Particular(_); client = |state, _team, _now| { StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None } } => (
        |game, region, target, key, runtime| {
            crate::gameserver::appserver::particularstate::update_particular_state(game, region, target, key, runtime);
        },
        crate::gameserver::appserver::particularstate::end_particular_state,
        crate::gameserver::appserver::particularstate::restart_particular_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ); visual = |_state| Some((1, true)),
    StateData::Team(_); client = |state, team, _now| { StateClientRecord { time: state.client_state_time(), additional: state.additional_data(team), team_name: Some(state.team_name()) } } => (
        |game, region, target, key, runtime| {
            crate::gameserver::appserver::teamstate::update_team_recruitment_state(game, region, target, key, runtime);
        },
        crate::gameserver::appserver::teamstate::end_team_recruitment_state,
        crate::gameserver::appserver::teamstate::restart_team_recruitment_state,
        |_, _, _, _, _| true,
        set_state_sufferer_region
    ); visual = |_state| Some((1, true)),
    StateData::Script(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            crate::gameserver::appserver::scriptstate::update_script_move_state(game, region, target, key, runtime);
        },
        crate::gameserver::appserver::scriptstate::end_script_move_state,
        crate::gameserver::appserver::scriptstate::restart_script_move_state,
        crate::gameserver::appserver::scriptstate::update_script_move_state_properties,
        set_script_state_region
    ); visual = |state| match state {
        StateData::Script(state) => Some((if state.is_auto_protect() { 1 } else { 0 }, false)),
        _ => None,
    },
    StateData::ChangeBody(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            game.update_move_shape_change_body_state(region, target, key, runtime);
        },
        CGame::end_move_shape_change_body_state,
        crate::gameserver::appserver::chbystate::restart_change_body_state,
        crate::gameserver::appserver::chbystate::update_change_body_state_properties,
        CGame::set_change_body_state_region
    ),
    StateData::Extended(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            game.update_move_shape_extended_state(region, target, key, runtime);
        },
        CGame::end_move_shape_extended_state,
        CGame::restart_move_shape_extended_state,
        CGame::update_move_shape_extended_state_properties
    ),
    StateData::Undead(_); client = |state, _team, now| { StateClientRecord::timed(state.client_state_time(now) as i32) } => (
        |game, region, target, key, runtime| {
            game.update_move_shape_appellation_state(region, target, key, runtime);
        },
        CGame::end_move_shape_appellation_state,
        CGame::restart_move_shape_appellation_state,
        CGame::update_move_shape_appellation_state_properties
    ),
    StateData::Ride(_); client = |state, _team, _now| { StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None } } => (
        |game, region, target, key, runtime| {
            game.update_move_shape_ride_state(region, target, key, runtime);
        },
        CGame::end_move_shape_ride_state,
        CGame::restart_move_shape_ride_state,
        CGame::update_move_shape_ride_state_properties
    ),
}



/// Факты объектного CState::Begin при временно извлечённом регионе.
/// Запоминаются только region/type/id, без нового GUID-фильтра. Для
/// неподвижных CMoveShape реестр региона подтверждает регистрацию;
/// этот снимок не заменяет последующее разрешение GetSufferer в живой объект.
pub(crate) fn resolve_owned_skill_begin_object(
    game: &CGame,
    region: &CServerRegion,
    identity: ShapeIdentity,
) -> Option<(i32, ShapeIdentity)> {
    let identity = ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..identity };
    let region_id = match identity.object_type {
        400 => game.find_player(identity.id)?.shape().get_region_id(),
        500 => region.find_npc_by_id(identity.id)?.move_shape().shape().get_region_id(),
        600 => region.find_monster_by_id(identity.id)?.move_shape().shape().get_region_id(),
        1_100 | 1_200 if region.has_registered_shape(identity) => region.id,
        _ => return None,
    };
    Some((region_id, identity))
}

pub(crate) fn send_owned_state_visual(
    game: &CGame,
    region: &CServerRegion,
    shape: &CShape,
    state_id: u32,
    begin: bool,
    client_time: i32,
    additional_data: u32,
) {
    let identity = shape.identity();
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state_id as i32);
    if begin {
        message.add_long(client_time);
        message.add_long(additional_data as i32);
    }
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

/// Координатная ветвь точного `CState::GetSufferer`: `(0, 0)` означает
/// отсутствие цели, иначе выбирается первый `CMoveShape` в порядке региона.
pub(crate) fn resolve_coordinate_sufferer(
    game: &CGame,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Option<ShapeIdentity> {
    if tile_x == 0 && tile_y == 0 {
        return None;
    }
    let owner = game.find_region(region_id)?;
    let region = owner.base();
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    region
        .get_shapes(
            tile_x,
            tile_y,
            area_width,
            area_height,
            &RegionShapeResolver { game, owner },
            &mut shapes,
        )
        .ok()?;
    shapes
        .into_iter()
        .map(|shape| shape.identity)
        .find(|identity| matches!(identity.object_type, 400 | 500 | 600 | 1_100 | 1_200))
}

/// Identity-ветвь точного `CState::GetSufferer`: игрок разрешается глобально,
/// остальные поддержанные `CMoveShape` — через сохранённый регион состояния.
pub(crate) fn resolve_identity_sufferer(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<ShapeIdentity> {
    match identity.object_type {
        400 => game.find_player(identity.id).map(|_| identity),
        500 | 600 | 1_100 | 1_200 => {
            let owner = game.find_region(region_id)?;
            let lookup = ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..identity };
            if !owner.base().has_registered_shape(lookup) { return None; }
            // GetObject возвращает живой объект, не пространственный ShapeView:
            // некорректная координата и отсутствие figure-таблицы не дают NULL.
            let shape = match identity.object_type {
                500 => owner.base().find_npc_by_id(identity.id)?.move_shape().shape(),
                600 => owner.base().find_monster_by_id(identity.id)?.move_shape().shape(),
                _ => owner.stationary_build(lookup)?.move_shape().shape(),
            };
            (shape.identity().object_type == identity.object_type && shape.identity().id == identity.id)
                .then_some(identity)
        }
        _ => None,
    }
}

/// Точный `CState::GetUser`: player хранится в глобальном реестре, остальные
/// достигнутые `CMoveShape` разрешаются через регион владельца состояния.
pub(crate) fn resolve_state_user(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<ShapeIdentity> {
    resolve_identity_sufferer(game, region_id, identity)
}

/// GetSufferer сначала пытается разрешить сохранённую identity, затем клетку
/// в сохранённом регионе источника. Наличие живого GetUser не является gate.
pub(crate) fn resolve_skill_sufferer(
    game: &CGame,
    lifecycle: &SkillLifecycle,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = lifecycle.sufferer();
    if let Some(identity) = resolve_identity_sufferer(game, region, identity) {
        return Some((region, identity));
    }
    let (region, _) = lifecycle.user();
    let (x, y) = lifecycle.destination();
    Some((region, resolve_coordinate_sufferer(game, region, x, y)?))
}

/// Живой GetUser по сохранённым region/type/id, без подстановки держателя
/// состояния. Региональные объекты здесь принадлежат опубликованному CGame.
pub(crate) fn resolve_state_move_shape(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<&CMoveShape> {
    let identity = resolve_state_user(game, region_id, identity)?;
    match identity.object_type {
        400 => Some(game.find_player(identity.id)?.move_shape()),
        500 => Some(game.find_region(region_id)?.base().find_npc_by_id(identity.id)?.move_shape()),
        600 => Some(game.find_region(region_id)?.base().find_monster_by_id(identity.id)?.move_shape()),
        1_100 | 1_200 => Some(game.find_region(region_id)?.stationary_build(identity)?.move_shape()),
        _ => None,
    }
}

pub(crate) fn resolve_state_move_shape_mut(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<&mut CMoveShape> {
    let identity = resolve_state_user(game, region_id, identity)?;
    match identity.object_type {
        400 => Some(game.find_player_mut(identity.id)?.move_shape_mut()),
        500 => Some(game.find_region_mut(region_id)?.base_mut().find_npc_by_id_mut(identity.id)?.move_shape_mut()),
        600 => Some(game.find_region_mut(region_id)?.base_mut().find_monster_by_id_mut(identity.id)?.move_shape_mut()),
        1_100 | 1_200 => Some(game.find_region_mut(region_id)?.stationary_build_mut(identity)?.move_shape_mut()),
        _ => None,
    }
}

/// Byte-exact базовый `CState::Serialize`: user type/ID, затем sufferer type/ID.
pub(crate) fn encode_state_identities(
    user: ShapeIdentity,
    sufferer: ShapeIdentity,
) -> [u8; STATE_IDENTITY_BYTES] {
    let mut bytes = Vec::with_capacity(STATE_IDENTITY_BYTES);
    let mut writer = LegacyWriter::new(&mut bytes);
    writer.write_i32(user.object_type);
    writer.write_i32(user.id);
    writer.write_i32(sufferer.object_type);
    writer.write_i32(sufferer.id);
    bytes.try_into().expect("размер identity-префикса CState фиксирован")
}

/// Byte-exact базовый `CState::Unserialize` без C++ cursor side effects.
pub(crate) fn decode_state_identities(
    payload: &[u8],
    offset: usize,
) -> Result<(ShapeIdentity, ShapeIdentity), LegacyReadBlock> {
    let mut reader = LegacyReader::at(payload, offset)?;
    let read_identity = |reader: &mut LegacyReader<'_>| -> Result<ShapeIdentity, LegacyReadBlock> {
        Ok(ShapeIdentity {
            object_type: reader.read_i32()?,
            id: reader.read_i32()?,
            ex_id: CGuid::GUID_INVALID,
        })
    };
    Ok((read_identity(&mut reader)?, read_identity(&mut reader)?))
}

/// Exact базовый `CState::GetClientStateTime` для классов без override-а.
pub(crate) const fn default_client_state_time() -> i32 {
    0
}

/// Exact базовый `CState::GetAdditionalData` для классов без override-а.
pub(crate) const fn default_additional_data() -> u32 {
    0
}

/// Exact общее тело `CBlindState::GetRemainedTime` по адресу `0x005F2CD0`.
/// Проверка deadline и вычисление положительного остатка независимо читают
/// wrapping clock; второе чтение не выполняется на уже истёкшем состоянии.
pub(crate) fn timed_client_state_time(
    started_at_ms: u32,
    keep_time_ms: u32,
    mut now_milliseconds: impl FnMut() -> u32,
) -> u32 {
    let deadline = started_at_ms.wrapping_add(keep_time_ms);
    if deadline <= now_milliseconds() {
        0
    } else {
        deadline.wrapping_sub(now_milliseconds())
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h

// ============================================================================
// FUNCTION: CState::IsEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h:61
// RVA: 0x000D8380
// ADDRESS: 004d8380
// PROTOTYPE: int __thiscall IsEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:21
// RVA: 0x001DBC90
// ADDRESS: 005dbc90
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:29
// RVA: 0x001DBCA0
// ADDRESS: 005dbca0
// PROTOTYPE: undefined __thiscall CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:149
// RVA: 0x001DBCE0
// ADDRESS: 005dbce0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::~CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:48
// RVA: 0x001DBD40
// ADDRESS: 005dbd40
// PROTOTYPE: void __thiscall ~CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:175
// RVA: 0x001DBD70
// ADDRESS: 005dbd70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:208
// RVA: 0x001DBDD0
// ADDRESS: 005dbdd0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:227
// RVA: 0x001DBE20
// ADDRESS: 005dbe20
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
