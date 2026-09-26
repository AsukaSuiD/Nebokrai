//! Живые Begin/restart/AI/End `CTianShenXiaFanState` (`0x335`) в переходном Game.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/tianshenxiafanstate.cpp`. Данные, асимметричный кодек и
//! формулы свойств перенесены в Zone `effects/tianshenxiafan.rs` (там же
//! адреса конструктора, vtable и обеих сторон кодека).
//! Нативный input занимает 10 байт с ID; общий factory сохраняет это
//! продвижение, а отдельный tracked cache-span содержит 12 байт Serialize.
//! Неизвестный исходный tail не перечитывается с +12 и не перезаписывается
//! расширением. Этот legacy defect сохранён. Типизированный owner участвует
//! в login, пересчёте свойств, строгом `CBlindState::AI`, визуалах и
//! обратном DB-кодеке. Достигнутый AI получает один поколенческий ключ
//! общей арены; порядок вызовов и границу прохода задаёт общий
//! CMoveShape::UpdateAbnormality. Любое удаление адресует тот же экземпляр,
//! а не первый дубль. AI/End разрешают общий CMoveShape по region/type/id;
//! RTTI-ограничения формул игрока не запрещают жизненный цикл региональных
//! держателей. Exact vtable 0x0066202C: End 0x006059A0 отправляет visual и
//! вызывает базовый End 0x005DBCE0. После эффекта holder перечитывается;
//! общий virtual UpdateProperty пересчитывает свойства только при фактическом
//! удалении точной записи. Прямой End и AI используют один exact-key хвост
//! без чтения часов. StartAllStates 0x004CE050 вызывает Begin(0, self);
//! Begin 0x00605B70 создаёт visual, но User остаётся NULL. Для загруженной
//! записи базовый End лишь отмечает ended; удаление из контейнера User
//! отсутствует. Restart воспроизводит только Begin(NULL, holder): базовый
//! Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Visual принадлежит экземпляру общей арены: BeginVisualEffect(1) →
//! concrete Update(0) → базовый visual-хвост. OnUpdateProperties 0x00605A10:
//! GetSufferer → RTTI CPlayer → семь запросов skill properties → формулы.
//! NULL/неигрок возвращает 0, отсутствующая запись skill — 1 без изменений.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) use nebokrai_zone::effects::{
    TIAN_SHEN_XIA_FAN_STATE_BYTES, TianShenXiaFanPlayerView,
    TianShenXiaFanState,
};

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

fn apply_to_player(
    game: &CGame,
    state: TianShenXiaFanState,
    properties: PlayerCombatProperties,
) -> PlayerCombatProperties {
    let view = TianShenXiaFanPlayerView {
        element_modify: properties.element_modify,
        minimum_attack: properties.minimum_attack,
        maximum_attack: properties.maximum_attack,
        defense: properties.defense,
        element_resistance: properties.element_resistance,
        attack_avoid: properties.attack_avoid,
        element_avoid: properties.element_avoid,
    };
    let query = game
        .skill_factory()
        .query_skill_base_properties(state.state_id(), state.level());
    let view = state.apply_to_player_view(view, query.map(|skill| {
        move |key: u32| skill.query_property(key)
    }));
    PlayerCombatProperties {
        element_modify: view.element_modify,
        minimum_attack: view.minimum_attack,
        maximum_attack: view.maximum_attack,
        defense: view.defense,
        element_resistance: view.element_resistance,
        attack_avoid: view.attack_avoid,
        element_avoid: view.element_avoid,
        ..properties
    }
}

pub(crate) fn send_tian_shen_xia_fan_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: TianShenXiaFanState,
    begin: bool,
    _now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.state_id() as i32);
    if begin {
        message.add_long(state.client_time() as i32);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn update_tian_shen_xia_fan_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    if target.object_type != 400 {
        return false;
    }
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key)).copied()
    else { return false; };
    let Some(player) = game.find_player(target.id) else { return false; };
    let properties = apply_to_player(game, state, player.combat_properties());
    if let Some(player) = game.find_player_mut(target.id) {
        player.update_state_combat_properties(|_| properties);
    }
    true
}

pub(crate) fn restart_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.state_id() as i32);
        message.add_long(0);
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn update_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_tian_shen_xia_fan_state(game, region_id, holder, key)
}

pub(crate) fn end_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key))
        .copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.state_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    end_base_applied_state(game, region_id, holder, key, TIAN_SHEN_XIA_FAN_STATE_BYTES)
}
