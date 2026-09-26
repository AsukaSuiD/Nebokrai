//! Живые Begin/restart/update/End `CDaubPoisonState` (`0xDF`) в переходном
//! Game. Данные, срок и сохраняемая запись — Zone `effects/daubpoison.rs`
//! (ctor `0x5F17B0`, vtable `0x66047C`, кодек и правило срока уже там; здесь
//! только обращения к живой арене, без дублирования данных и записи).
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/daubpoisonstate.cpp/.h`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/daubpoisonstate.rs`; тела Begin/restart/
//! AI/End перенесены буквально (кластер D полосы «трупная/ядовая state-
//! линия», порция D4, карта — запись аудита «Zone skills: карта полосы
//! Monster 0x19x — 5 кластеров волн», 26 сентября 2026).
//!
//! Машинная разведка порции по этой паре (запись `.local/recon-de/notes/
//! D4-daubpoison.md`, тела `.local/recon-de/disasm/CDaubPoisonState.txt`);
//! сопоставление с перенесённым кодом — MATCH по всем пунктам:
//!
//! - Begin JJ/typed `0x5F18A0`/`0x5F1960` — `CState::Begin` (`0x5DBDD0`/
//!   `0x5DBE20`) → GetSufferer → null: End `vcall+0x1C` → ret 0; ok →
//!   новый visual `0x6604C8` → `[+0x34]` → BeginVisualEffect(1) →
//!   UpdateVisualEffect(0) → ret 1. Object Begin `0x5F1A50`: **null U →
//!   тихий ret 0**; base clock при non-NULL U до getters, S не проверяется.
//!   Первичный адаптер ниже сохраняет ту же форму: разрешение U до часов,
//!   часы Begin, публикация `0x000BFE03` (type/id/0xDF/remaining/0) до
//!   append, стороны U/S сохраняются после append отдельно; отдельного
//!   UpdateProperty после Begin нет.
//! - End `0x5FD420` (ICF `CAgilityState::End`): optional Update(1) → свежий
//!   GetSufferer → RemoveState(pointer) (`0x4CDAB0`), без записи state.ended;
//!   чужой или отсутствующий S оставляет запись у держателя.
//! - AI `0x5D5BA0` (ICF `CBlindState::AI`): `now > start + keep` → End,
//!   равенство с границей ещё активно (`DaubPoisonState::expired`). Обход
//!   состояний тика — общий callback-реестр `states/state.rs` прежнего пакета,
//!   вызывающий `update_daub_poison_state` ниже (объявленный шов).
//! - GetRemainedTime `0x5F2CD0` (double-read race часов), Serialize
//!   `0x5F51E0` (ID до вызова GetRemainedTime), Unserialize `0x5EAAC0`
//!   (часы сначала, затем keep) — реализованы в Zone `effects/daubpoison.rs`
//!   (MATCH, здесь не повторяются). Wire: `0x000BFE03` = type/id/DF/
//!   remaining/additional(0), `0x000BFE04` = type/id/DF.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//!/`StateCastMoveShape` — visual (`send_move_shape_around`,
//! `update_property_state_visual`, `update_applied_state_end_visual`), base
//! Begin (`mark_applied_state_begun` + `begin_applied_state_visual(1)`) и
//! `RemoveState(pointer)` (`remove_applied_state_from`). Арена хранит один
//! экземпляр, стороны Begin и serialized span; заимствования заменяют
//! `CState*` без копирования payload. Первичное Begin(U,U) и append
//! принадлежат захваченному CMoveShape, не только игроку; player-only
//! проверка переноса яда принадлежит ударам. Потребление статическое
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).

use crate::app::game_message::CMessage;
use crate::effects::{DAUB_POISON_STATE_BYTES, DaubPoisonState};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPropertyTarget, state_cast_storage_participant,
};

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

/// Первичный Begin(U,U) после ветки применения `CDaubPoison`: U разрешается
/// до часов, часы и стороны фиксируются, visual `0x000BFE03` публикуется до
/// append; возвращённый ключ однозначен внутри одной арены. `None` — отказ
/// разрешения U либо append (Begin false → deleting-dtor нового у native).
pub fn begin_primary_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    keep_time_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = DaubPoisonState::new(keep_time_ms);
    game.resolve_state_move_shape(source.0, source.1)?;
    state.begin_at(now());
    let user = state_cast_storage_participant(game, source)?;
    let sufferer = state_cast_storage_participant(game, source)?;
    if let Some(shape) = game.resolve_state_move_shape(sufferer.0, sufferer.1) {
        let target = (shape.shape().get_region_id(), shape.shape().identity());
        let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
        message.add_long(target.1.object_type);
        message.add_long(target.1.id);
        message.add_ulong(state.skill_id());
        message.add_long(state.client_state_time(&mut *now) as i32);
        message.add_ulong(0);
        game.send_move_shape_around(target.0, target.1, &message);
    }
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(source.0, source.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Повторный вход: NULL-user restart сохраняет timestamp и User; visual
/// loop1 перезапускается, property-visual `0x000BFE03` читает actual S.
pub fn restart_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key)) { return false; }
    if game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1))
    {
        game.update_property_state_visual::<DaubPoisonState>(
            region_id, holder, key, StateCastPropertyTarget::Sufferer,
            now, |state, now| state.client_state_time(now),
        );
    }
    true
}

/// Тик AI `0x5D5BA0`: истечение срока (строгое неравенство) → полный End.
pub fn update_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    end_daub_poison_state(game, region_id, holder, key)
}

/// End `0x5FD420`: optional Update(1) (`0x000BFE04`) → свежий S →
/// RemoveState(pointer), без записи state.ended; отсутствующий S подавляет
/// удаление, но не visual-хвост ресурса.
pub fn end_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, DAUB_POISON_STATE_BYTES)
}
