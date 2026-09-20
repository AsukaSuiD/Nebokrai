//! Оглушающий снаряд Strike (0xDD).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/strike.cpp.
//! Check удерживает исходные U/S, отклоняет NULL/self до запроса свойств,
//! затем проверяет reuse, дальность, BLOCK_UNFLY и signed MP. MP0 — тихий
//! отказ; источник не типа Player проходит без Move0. Оружие не проверяется.
//! Общий targetedprojectile сохраняет выпуск, полёт, visual и End.
//!
//! Attack выполняет сырой контакт без допуска и RP. Свежий Calculate
//! использует общий RawRange, weapon factor и отрицательный hit modifier;
//! NULL таблица оставляет пустой UNKNOWN/1 контакт. После удара живой S
//! получает сокращённый по уровням срок из прежней таблицы AI. Повторные
//! уровни читаются S→U, только если первая пара U→S превышает U+5.
//! AddState отдельно повторяет допуск и запрос таблицы, требует текущий
//! регион U, создаёт Rush2State до End/destructor первого ID 0x7C и добавляет
//! новый объект в конец. Общий primary lifecycle сохраняет wire/DB и снятие
//! блокировок; Vec/арена заменяют native указатели без второго хранилища.

use super::basemagic::SKILL_USAGE_REUSE_DELAY_TIME;
use super::kernel::skill_is_restored;
use super::rangedweaponcast::{CastPathBlock, check_cast_mana, check_skill_path};
use super::rush::scaled_state_time;
use super::rushstate2::{RUSH_2_STATE_ID, Rush2State, begin_primary_rush_2_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::{PlayerWeaponRoll, calculate_penalized_weapon_attack};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const STRIKE_SKILL_ID: u32 = 0xDD;
const STATE_TIME: u32 = 10_002;
const DAMAGE_FACTOR: u32 = 20_003;

pub(super) fn check_strike_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(user) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return false; };
    let target = original_target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let player = (source.shape().identity().object_type == 400).then_some(source.shape().identity().id);
    if target.is_none_or(|target| std::ptr::eq(source, target)) {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0286"); }
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path(game, instance, &properties, &path, player, CastPathBlock::Generic) { return false; }
    check_cast_mana(game, instance, user, &properties)
}

fn add_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    if keep == 0 || resolve_state_move_shape(game, source.0, source.1).is_none()
        || resolve_state_move_shape(game, target.0, target.1).is_none()
        || !game.live_skill_target_attackable_between(source, target)
    { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    if game.skill_base_properties(skill.id(), skill.level()).is_none() { return; }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region()
        || game.find_region(user.shape().get_region_id()).is_none()
    { return; }
    let state = Rush2State::new(keep);
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|old| old.state_id() == RUSH_2_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let _ = begin_primary_rush_2_state(
        game, target.0, target.1, Some(source), Some(target), state, &mut || runtime.now_milliseconds(),
    );
}

pub(super) fn apply_strike_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), ai_properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || resolve_state_move_shape(game, target.0, target.1).is_none()
    { return; }
    if let Some((master, attack)) = calculate_penalized_weapon_attack(
        game, instance, source, target, DAMAGE_FACTOR, PlayerWeaponRoll::RawRange,
    ) {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    }
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let keep = if u32::from(source_level) + 5 < u32::from(target_level) {
        let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
        let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
        scaled_state_time(source_level, target_level, ai_properties.query_property(STATE_TIME))
    } else { ai_properties.query_property(STATE_TIME) };
    if keep != 0 { add_state(game, instance, source, target, keep, runtime); }
}
