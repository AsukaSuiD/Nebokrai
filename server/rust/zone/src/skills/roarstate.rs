//! Живая установка, visual и End подавления атаки CRoarState в Zone.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/roarstate.cpp/.h`. Машинные якоря:
//! конструктор `0x1EC7E0`, Serialize 5-fold `0x1F65F0` (с heal-квартетом),
//! Unserialize `0x1ECC60`, AI `0x1EC9A0`, OnUpdateProperties `0x1ECAA0`;
//! данные и числовое подавление — `effects/roar.rs`. Прежний переходный
//! владелец — `src/gameserver/appserver/skills/roarstate.rs`; тела замены,
//! restart, AI и End перенесены буквально порцией №6c «self/zone-касты»
//! (разведка — запись аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)», 26 сентября 2026).
//!
//! Замена завершает прежний ID до чтения параметров нового. Restart не
//! обновляет часы, а End ищет фактического Sufferer перед удалением записи.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::SelfCastGame`
//! реализован у прежнего владельца; обвязка арены (`end_and_destroy_state_at`,
//! `begin_base_applied_state`, `begin_applied_state_visual`,
//! `update_property_state_visual`, `update_applied_state_end_visual`,
//! `resolve_applied_state_sufferer`, `remove_applied_state_from`) остаётся у
//! прежнего `states/state.rs` и объявлена одноимёнными методами трейта;
//! ветка монстра OnUpdateProperties (lookup property → свежие границы →
//! wrapping-модификаторы) свёрнута в шов `roar_monster_apply_losses`.

use crate::app::game_message::CMessage;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::selfcast::{
    SelfCastGame, SelfCastMoveShape, SelfCastPlayer, SelfCastPropertyTarget,
    selfcast_storage_participant,
};
use super::state::StateKey;

pub use crate::effects::{ROAR_STATE_BYTES, ROAR_STATE_ID, RoarState};

const ELEMENT_ATTACK_LOSS: u32 = 215;
const ATTACK_LOSS: u32 = 205;
const PERSIST_TIME: u32 = 10_002;

pub fn replace_roar_state<Game: SelfCastGame>(
    game: &mut Game, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = game.resolve_state_move_shape(target.0, target.1) else { return false; };
    let previous = shape.find_state_position(|state| state.state_id() == ROAR_STATE_ID);
    if let Some((position, _)) = previous {
        let _ = game.end_and_destroy_state_at(target.0, target.1, position);
    }
    let element_loss = properties.query_property(ELEMENT_ATTACK_LOSS) as i32;
    let attack_loss = properties.query_property(ATTACK_LOSS) as i32;
    let keep_time = properties.query_property(PERSIST_TIME);
    let mut state = RoarState::new(keep_time, attack_loss, element_loss);
    state.begin_at(now());
    let Some(user) = selfcast_storage_participant(game, source) else { return false; };
    let Some(sufferer) = selfcast_storage_participant(game, target) else { return false; };
    if game.resolve_state_move_shape(sufferer.0, sufferer.1).is_some() {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(sufferer.1.object_type);
        message.add_long(sufferer.1.id);
        message.add_ulong(ROAR_STATE_ID);
        message.add_long(state.client_time(&mut *now));
        message.add_ulong(0);
        game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    }
    let record = state.encoded_for_install();
    let Some(shape) = game.resolve_state_move_shape_mut(target.0, target.1) else { return false; };
    let key = shape.append_applied_state_record(state, &record);
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    game.update_move_shape_properties(target.0, target.1);
    true
}

pub fn update_roar_state_properties<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<RoarState>(
        region_id, holder, key, SelfCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).copied()
    else { return false; };
    if target.object_type == PLAYER_TYPE {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                (properties.minimum_attack, properties.maximum_attack, properties.element_modify) =
                    state.player_attack(
                        properties.minimum_attack, properties.maximum_attack, properties.element_modify,
                    );
                properties
            });
        }
    } else if matches!(target.object_type, 600 | 602) {
        game.roar_monster_apply_losses(target_region, target.id, state);
    }
    true
}

pub fn restart_roar_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).is_none()
    { return false; }
    let Some(sufferer) = selfcast_storage_participant(game, (region_id, holder)) else { return false; };
    if !game.begin_base_applied_state(region_id, holder, key) { return false; }
    if let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if game.begin_applied_state_visual(region_id, holder, key, 1) {
        game.update_property_state_visual::<RoarState>(
            region_id, holder, key, SelfCastPropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    true
}

pub fn update_roar_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key))
    else { return false; };
    if state.expired(now_ms) { return end_roar_state(game, region_id, holder, key); }
    let dead = game.resolve_applied_state_sufferer(region_id, holder, key)
        .and_then(|target| game.move_shape_health(target.0, target.1))
        .is_some_and(|health| health == 0);
    dead && end_roar_state(game, region_id, holder, key)
}

pub fn end_roar_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, SelfCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key) else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, ROAR_STATE_BYTES)
}
