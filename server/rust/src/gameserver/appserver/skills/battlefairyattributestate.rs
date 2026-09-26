//! Живые Begin/restart/AI/End состояний Po/Yu (0x212..0x219) в переходном
//! Game. Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa}state.cpp.
//! Данные, вид, 12-байтная запись и числовые формулы перенесены в Zone
//! `effects/battlefairy.rs` (там же адреса конструкторов и общего кодека);
//! skill-сторона октета (Check/AI, диспетчер определений) — в Zone
//! `skills/battlefairyattribute.rs` порцией №6b. Этот hub-lifecycle (state —
//! 9-fold AI/End/Serialize/Unserialize/GetRemainedTime/OnChangeRegion)
//! остаётся здесь до переноса общей арены состояний; координатор вызывает его
//! через шов `BattleFairyGame::begin_battle_fairy_attribute_state`.
//!
//! User всегда держатель арены; Sufferer у Po — caster, у Yu — сам держатель.
//! Object Begin требует User, читает собственные часы и создаёт silent loop1.
//! OnUpdateProperties требует User: Po публикует ему, Yu — Sufferer,
//! а формула всегда меняет User. Повторные состояния применяются в живом
//! порядке общей арены, без второго агрегата. Замена завершает прежний
//! экземпляр и добавляет новый в хвост. Прямой End — базовый
//! ended→RemoveState у User без visual. Таймер после строгого unsigned
//! deadline сначала обновляет существующий visual, затем выполняет тот же
//! End. NULL цели подавляет пакет, но не visual tail. Техническая запись
//! первичного наложения часов не читает. SetRegion меняет только User.
//! Begin(NULL, holder) после загрузки отказывает до базы: сохраняет
//! timestamp, ended и visual. Пустой User не заменяется holder; общий Clear
//! удаляет остаток.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_base_applied_state, resolve_applied_state_user, resolve_state_move_shape,
    resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{
    BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BattleFairyAttributeKind,
    BattleFairyAttributePlayerView, BattleFairyAttributeState,
};

fn apply_to_player(
    state: BattleFairyAttributeState,
    properties: PlayerCombatProperties,
) -> PlayerCombatProperties {
    let view = BattleFairyAttributePlayerView {
        attack_avoid: properties.attack_avoid,
        element_avoid: properties.element_avoid,
        minimum_attack: properties.minimum_attack,
        maximum_attack: properties.maximum_attack,
        element_modify: properties.element_modify,
    };
    let view = state.apply_to_player_view(view);
    PlayerCombatProperties {
        attack_avoid: view.attack_avoid,
        element_avoid: view.element_avoid,
        minimum_attack: view.minimum_attack,
        maximum_attack: view.maximum_attack,
        element_modify: view.element_modify,
        ..properties
    }
}

pub(crate) fn begin_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    sufferer: ShapeIdentity,
    mut state: BattleFairyAttributeState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(user_region) = resolve_state_move_shape(game, region_id, holder)
        .map(|shape| shape.shape().get_region_id())
    else { return false };
    state.begin_at(now());
    let sufferer = resolve_state_move_shape(game, region_id, sufferer)
        .map(|shape| (shape.shape().get_region_id(), sufferer));
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    let key = shape.append_applied_state_record(state, &state.encoded_for_install());
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some((user_region, holder)));
    shape.set_applied_state_sufferer(key, sufferer);
    true
}

pub(crate) fn update_battle_fairy_attribute_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key))
    else { return false };
    let visual_target = if state.kind().targets_self() {
        StatePropertyTarget::Sufferer
    } else {
        StatePropertyTarget::User
    };
    if resolve_applied_state_user(game, region_id, holder, key).is_none() {
        return false;
    }
    let _ = update_property_state_visual::<BattleFairyAttributeState>(
        game, region_id, holder, key, visual_target, now,
        |state, now| state.client_state_time(now),
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key)).copied()
    else { return false };
    let Some((target_region, target)) = resolve_applied_state_user(
        game, region_id, holder, key,
    ) else { return false };
    match target.object_type {
        400 => {
            let Some(player) = game.find_player_mut(target.id) else { return false };
            player.update_state_combat_properties(|properties| apply_to_player(state, properties));
        }
        600 => match state.kind() {
            BattleFairyAttributeKind::AttackLoss => {
                for minimum in [true, false] {
                    let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                    else { return false };
                    let modifiers = shape.property_modifiers_mut();
                    let value = if minimum {
                        &mut modifiers.minimum_attack
                    } else {
                        &mut modifiers.maximum_attack
                    };
                    *value = value.wrapping_sub(state.value());
                    let Some(monster) = game.find_region(target_region)
                        .and_then(|region| region.base().find_monster_by_id(target.id))
                    else { return false };
                    let Some(properties) = monster.base_property_key()
                        .and_then(|name| game.find_monster_property_by_origin_name(name))
                    else { return false };
                    let (low, high) = monster.state_attack_bounds(
                        properties.minimum_attack, properties.maximum_attack,
                    );
                    let current = (if minimum { low } else { high }) as i32;
                    if current < 0 {
                        let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                        else { return false };
                        let modifiers = shape.property_modifiers_mut();
                        let value = if minimum {
                            &mut modifiers.minimum_attack
                        } else {
                            &mut modifiers.maximum_attack
                        };
                        *value = value.wrapping_sub(current);
                    }
                }
            }
            BattleFairyAttributeKind::ElementModifyLoss => {
                let Some(monster) = game.find_region(target_region)
                    .and_then(|region| region.base().find_monster_by_id(target.id))
                else { return false };
                let current = monster.element_modifier() as i32;
                let value = state.apply_to_monster_element(current);
                let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                else { return false };
                shape.property_modifiers_mut().element_modify = value;
            }
            _ => {}
        },
        _ => {}
    }
    true
}

pub(crate) fn restart_battle_fairy_attribute_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: crate::gameserver::appserver::moveshape::StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    // Все восемь object Begin проверяют user до base Begin и visual.
    false
}

pub(crate) fn update_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key))
        .copied()
    else { return false };
    if !state.expired(now_ms) { return false }
    let visual_target = if state.kind().targets_self() {
        StatePropertyTarget::Sufferer
    } else {
        StatePropertyTarget::User
    };
    update_applied_state_end_visual(game, region_id, holder, key, visual_target);
    end_battle_fairy_attribute_state(game, region_id, holder, key)
}

pub(crate) fn end_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES)
}
