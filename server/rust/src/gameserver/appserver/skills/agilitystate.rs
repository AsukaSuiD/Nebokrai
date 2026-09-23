//! Живой путь постоянных состояний Agility/Natural/Rapture (0xda/0xdc/0xdb).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/agilitystate.cpp`, `naturalstate.cpp`, `rapturestate .cpp` и `.h`.
//! Данные, запись и три формулы принадлежат `zone/effects/agility.rs`.
//!
//! Наложение обходит живые слоты и удаляет все ID этого постоянного семейства,
//! не затрагивая временную Agility2. После каждого End уничтожается свежий
//! остаток той же позиции; пустые слоты не уплотняются. Только после обхода
//! caller читает новый коэффициент. Объектный Begin(U,U) читает часы,
//! сохраняет участников и публикует visual до append; UpdateProperty следует
//! после попытки Begin независимо от результата. Ненужный постоянному
//! состоянию timestamp не дублируется в payload.
//!
//! AI пустой; клиентские время и дополнительные данные нулевые.
//! End не пишет ended: существующий
//! visual получает Update(1) с базовым tail, затем свежий S удаляет именно
//! этот экземпляр. Чужой/NULL S не подменяется держателем арены. SetRegion
//! меняет только регион U; restart Begin(NULL, holder) сохраняет U и меняет S.
//! Общая арена заменяет исходные указатели.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    PERSISTENT_AGILITY_FAMILY_STATE_BYTES, PersistentAgilityFamilyState,
    PersistentAgilityProperties,
};

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_persistent_agility_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> PersistentAgilityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let mut index = 0;
    loop {
        let Some(shape) = resolve_state_move_shape(game, source.0, source.1) else { break; };
        if index >= shape.state_slot_count() { break; }
        if shape.state_at(index).is_some_and(|(_, state)| {
            PersistentAgilityFamilyState::is_known_skill(state.state_id())
        }) {
            let _ = end_and_destroy_state_at(game, source.0, source.1, index);
        }
        index += 1;
    }
    let state = create();
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        let _ = now();
        let user = participant(game, source)?;
        let sufferer = participant(game, source)?;
        if resolve_state_move_shape(game, sufferer.0, sufferer.1).is_some() {
            let mut message = CMessage::new(0x000b_fe03);
            message.add_long(sufferer.1.object_type);
            message.add_long(sufferer.1.id);
            message.add_ulong(state.skill_id());
            message.add_long(0);
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded();
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, 1);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn update_persistent_agility_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    update_player_state_properties::<PersistentAgilityFamilyState>(
        game, region_id, holder, key, |state, player| {
            player.update_state_combat_properties(|properties| {
                let projection = PersistentAgilityProperties {
                    full_miss: properties.full_miss,
                    element_resistance: properties.element_resistance,
                    blast_attack: properties.blast_attack,
                };
                let updated = state.apply_to_properties(projection);
                PlayerCombatProperties {
                    full_miss: updated.full_miss,
                    element_resistance: updated.element_resistance,
                    blast_attack: updated.blast_attack,
                    ..properties
                }
            });
        },
    )
}

pub(crate) fn restart_persistent_agility_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PersistentAgilityFamilyState>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<PersistentAgilityFamilyState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |_, _| 0,
        );
    }
    true
}

pub(crate) fn end_persistent_agility_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PersistentAgilityFamilyState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(
        game, region_id, holder, key, target, PERSISTENT_AGILITY_FAMILY_STATE_BYTES,
    )
}
