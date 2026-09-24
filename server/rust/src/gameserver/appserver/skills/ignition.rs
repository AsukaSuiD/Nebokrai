//! Воспламенение: арбалетный контакт и расход горючей смеси на цели.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/ignition.cpp/.h.
//!
//! Общий combustioncast сохраняет исходного U, свежего S после visual1,
//! расход MP до поздней проверки оружия и абсолютный срок одного удара.
//! SECOND_TIME и THIRD_TIME задают три отметки только в visual-пакете,
//! а не дополнительные серверные атаки. End сбрасывает обе фазы и CAN
//! до свежего U/Move1 и общего AttackEnd.
//!
//! Контакт проверяет указательное self и живой допуск, затем сохраняет PK
//! перед свежей таблицей Calculate. Наличие любого ID F1 выбирает коэффициент
//! 20021 вместо 20003; ended и RTTI здесь не фильтруются. Нет weapon modifier,
//! чтения уровня цели или RP. Общий оружейный хвост сохраняет живые getter-ы
//! MAX→MIN→RNG→MIN, ELEMENT, SOUL WORD, CCH WORD→RNG и критическое усечение.
//! NULL таблица оставляет UNKNOWN/1, но не отменяет сырой OnBeenAttacked.
//! После его callbacks заново ищется первый F1: каноническая арена вызывает
//! End и destructor свежего остатка той же позиции без внешнего UpdateProperty.
//!
//! Visual1 читает собственную свежую таблицу и пишет 0, SECOND_TIME,
//! THIRD_TIME+SECOND_TIME. Только отсутствующий S этого режима пропускает
//! базовый visual-хвост; остальные режимы сохраняют его вызов. Некорректная
//! float-координата сохраняет native FISTP sentinel i32::MIN.

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::kerosenestate::KEROSENE_STATE_ID;
use super::skillfactory::SkillOwner;
use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage, source_master};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;

pub(crate) const IGNITION_SKILL_ID: u32 = 0xf2;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const SECOND_TIME: u32 = 15_002;
const THIRD_TIME: u32 = 15_003;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const TARGET_DAMAGE_FACTOR_2: u32 = 20_021;

pub(crate) fn publish_ignition_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) -> bool {
    if skill.owner() != SkillOwner::CIgnition
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Ignition || effect.is_ended()
        })
    { return true; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return true; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 4 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return true;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return true };
    let target = if action == 2 {
        let Some((region, target)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return false; };
        let Some(target) = resolve_state_move_shape(game, region, target) else { return false; };
        Some(target.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
        if let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) {
            message.add_long(0);
            message.add_ulong(properties.query_property(SECOND_TIME));
            let third = properties.query_property(THIRD_TIME);
            let second = properties.query_property(SECOND_TIME);
            message.add_ulong(third.wrapping_add(second));
        }
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
    true
}

fn calculate_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    let soaked = resolve_state_move_shape(game, target.0, target.1)
        .is_some_and(|shape| shape.has_state_by_skill_id(KEROSENE_STATE_ID));
    let factor = properties.query_property(if soaked { TARGET_DAMAGE_FACTOR_2 } else { TARGET_DAMAGE_FACTOR });
    attack.damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::AbsoluteRange, attack);
}

pub(super) fn apply_ignition_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) {
    let Some(target) = target else { return; };
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(user, sufferer)
        || !game.live_skill_target_attackable_between(source, target)
    { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let mut attack = AttackInformation::for_master(master);
    calculate_attack(game, instance, source, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == KEROSENE_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
}
