//! Живое накопление, расход и visual метеорных стрел в переходном Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/meteorarrowstate.cpp` и `meteorarrowstate.h`,
//! `meteorarrowmass.cpp` и `meteorarrowmass.h`, `meteorarrow.cpp` и
//! `meteorarrow.h`. Данные и запись состояния находятся в Zone.
//! Поиск первого ID `0xCC`, проверка живого S и публикация visual требуют
//! доступа к арене; расход удаляет найденное состояние через деструктор.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    destroy_move_shape_state, end_base_applied_state, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::visualeffect::CVisualEffect;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MeteorArrowState,
};
const AMOUNT: u32 = 20_016;
const AMOUNT_LIMIT: u32 = 20_017;

fn first_meteor_arrow_slot(game: &CGame, source: (i32, ShapeIdentity)) -> Option<StateKey> {
    resolve_state_move_shape(game, source.0, source.1)?
        .find_state_position(|state| state.state_id() == METEOR_ARROW_MASS_SKILL_ID)
        .map(|(_, key)| key)
}

pub(crate) fn first_meteor_arrow_count(game: &CGame, source: (i32, ShapeIdentity)) -> Option<i32> {
    let key = first_meteor_arrow_slot(game, source)?;
    resolve_state_move_shape(game, source.0, source.1)?
        .applied_state::<MeteorArrowState>(key).map(|state| state.arrows())
}

fn publish_added_arrows(game: &mut CGame, target: (i32, ShapeIdentity), count: i32) {
    let Some(target) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let shape = target.shape();
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    message.add_ulong(METEOR_ARROW_MASS_SKILL_ID);
    message.add_ulong(0);
    message.add_long(count);
    let _ = game.send_move_shape_around(shape.get_region_id(), shape.identity(), &message);
}

fn update_meteor_arrow_add_visual(
    game: &mut CGame, source: (i32, ShapeIdentity), key: StateKey,
) {
    let Some(ended) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.applied_state_visual_ended(key)) else { return; };
    if !ended
        && let Some(target) = resolve_applied_state_sufferer(game, source.0, source.1, key)
        && let Some(count) = resolve_state_move_shape(game, source.0, source.1)
            .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).map(|state| state.arrows())
    {
        publish_added_arrows(game, target, count);
    }
    update_applied_state_visual_base(game, source.0, source.1, key);
}

/// Таблица AI уже захвачена caller-ом. LIMIT нужен только новому ctor,
/// AMOUNT читается после его Begin либо непосредственно перед старым Add.
pub(crate) fn add_meteor_arrows(
    game: &mut CGame, source: (i32, ShapeIdentity), properties: &CSkillBaseProperties,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if let Some(key) = first_meteor_arrow_slot(game, source)
        && resolve_state_move_shape(game, source.0, source.1)
            .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).is_some()
    {
        let amount = properties.query_property(AMOUNT);
        if resolve_applied_state_sufferer(game, source.0, source.1, key).is_some() {
            let added = resolve_state_move_shape_mut(game, source.0, source.1)
                .and_then(|shape| shape.applied_state_mut::<MeteorArrowState>(key))
                .is_some_and(|state| state.add_arrows(amount));
            if added { update_meteor_arrow_add_visual(game, source, key); }
        }
        return true;
    }
    let mut state = MeteorArrowState::new(properties.query_property(AMOUNT_LIMIT));
    let _ = now();
    let Some(source_shape) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    let participant = (source_shape.shape().get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..source_shape.shape().identity()
    });
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(1);
    visual.update_visual_effect();
    let amount = properties.query_property(AMOUNT);
    if resolve_state_move_shape(game, participant.0, participant.1).is_some() && state.add_arrows(amount) {
        publish_added_arrows(game, participant, state.arrows());
        visual.update_visual_effect();
    }
    let record = state.encoded();
    let Some(shape) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    shape.begin_applied_state_visual(key, visual.loop_value());
    shape.update_applied_state_visual_base(key);
    true
}

/// Destructor-only расход: тип определяет возвращаемое количество, но
/// освобождается первый IDCC даже при несовпадении типа или нулевом запасе.
pub(crate) fn consume_meteor_arrow_count(game: &mut CGame, source: (i32, ShapeIdentity)) -> i32 {
    let Some(key) = first_meteor_arrow_slot(game, source) else { return 0; };
    let count = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).map_or(0, |state| state.arrows());
    let _ = destroy_move_shape_state(game, source.0, source.1, key);
    count
}

pub(crate) fn restart_meteor_arrow_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).is_none() { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false; };
    let participant = (shape.shape().get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.shape().identity()
    });
    shape.set_applied_state_sufferer(key, Some(participant));
    begin_applied_state_visual(game, region_id, holder, key, 1)
        && update_applied_state_visual_base(game, region_id, holder, key)
}

pub(crate) fn destroy_meteor_arrow_state_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) {
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
}

pub(crate) fn end_meteor_arrow_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).is_none() { return false; }
    destroy_meteor_arrow_state_visual(game, region_id, holder, key);
    end_base_applied_state(game, region_id, holder, key, METEOR_ARROW_STATE_BYTES)
}
