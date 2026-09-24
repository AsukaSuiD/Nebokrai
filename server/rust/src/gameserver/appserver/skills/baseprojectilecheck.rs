//! Проверки и визуальные сообщения стрельбы, базовой/огненной магии и GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archery.cpp,
//! basemagic.cpp, firebolt.cpp, fireball.cpp, godpunishment.cpp и GetTargetPath
//! из appserver/states/skill.cpp.
//! Check сохраняет исходного U и необязательного S, но строит свежий базовый
//! путь. MAX0 не ограничивает дальность; ненулевой MAX читается повторно
//! и у Archery допускает ещё одну клетку. Магия отклоняет указательную
//! самоцель до свойств, но допускает NULL S; Archery и GodPunishment не
//! проверяют самоцель. GodPunishment вообще не использует Check-параметр S.
//! Только стрельба проверяет BLOCK2 и лук/арбалет игрока. Текст BLOCK2 зависит
//! от наличия visual; оружейный режим 14 не имеет собственного пакета.
//! У магии отдельные сообщения самоцели, reuse и дальности. FireBolt/FireBall
//! и GodPunishment проверяют MP игрока: нулевая цена — тихий отказ, недостаток
//! даёт visual7 и GS0288. Только FireBall запрещает движение при успехе;
//! остальные CMoveShape проходят без проверки MP и изменения движения.
//!
//! Visual1 сохраняет базовую точку и нулевые type/id при отсутствующем S;
//! время берётся из постоянного progress игрока/монстра. У FireBall и GodPunishment DWORD
//! времени полёта отсутствует в сообщении.
//! Базовый visual-хвост вызывается при любом режиме и отсутствии участников.
//! Некорректная float-координата сохраняет native FISTP sentinel i32::MIN.
//! Путь с заданной длиной использует общий региональный механизм, без второй
//! геометрии или снимка участников из предыдущего AI.
//! Общий профиль объединяет только Check/пакеты: самостоятельный AI
//! GodPunishment повторяет тот же distance-only gate после расхода MP и поворота.

use super::basemagic::{SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::baseprojectilecast::BaseProjectileKind;
use super::godpunishment::GOD_PUNISHMENT_SKILL_ID;
use super::kernel::skill_is_restored;
use super::rangedweaponcast::{CastPathBlock, check_cast_mana, check_cast_mana_without_movement, check_skill_path};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;

const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ProjectileCheckProfile { Base(BaseProjectileKind), GodPunishment }

pub(super) fn check_base_projectile_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, kind: BaseProjectileKind,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    check_projectile_cast(game, instance, ProjectileCheckProfile::Base(kind), original_user, original_target, runtime)
}

pub(super) fn check_god_punishment_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    check_projectile_cast(game, instance, ProjectileCheckProfile::GodPunishment, original_user, None, runtime)
}

fn check_projectile_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, profile: ProjectileCheckProfile,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let archery = profile == ProjectileCheckProfile::Base(BaseProjectileKind::Archery);
    let Some(source) = original_user.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
    else { return false; };
    let player = (source.shape().identity().object_type == PLAYER_TYPE).then_some(source.shape().identity().id);
    if !archery && profile != ProjectileCheckProfile::GodPunishment && original_target
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .is_some_and(|target| std::ptr::eq(source, target))
    {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0286"); }
        return false;
    }
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if !archery && let Some(player) = player {
            game.send_skill_system_info(player, b"GS0278");
        }
        return false;
    }
    let Some(path) = checked_projectile_path(game, instance, archery, player, original_target, &properties)
    else { return false; };
    match profile {
        ProjectileCheckProfile::Base(BaseProjectileKind::Magic) => return true,
        ProjectileCheckProfile::Base(BaseProjectileKind::FireBolt) | ProjectileCheckProfile::GodPunishment =>
            return check_cast_mana_without_movement(game, instance, source, &properties),
        ProjectileCheckProfile::Base(BaseProjectileKind::FireBall) => return check_cast_mana(game, instance, source, &properties),
        ProjectileCheckProfile::Base(BaseProjectileKind::Archery) => {}
    }
    if path.iter().any(|cell| cell.2 == 2) {
        if game.registered_skill(instance).is_some_and(|skill| skill.visual_effect().is_some()) {
            game.update_registered_skill_visual(instance, 15);
            if let Some(player) = player { game.send_skill_system_info(player, b"GS0282"); }
        }
        return false;
    }
    let Some(player) = player else { return true; };
    let failure: Option<&[u8]> = match game.find_player(player).and_then(|player| player.equipment().get_goods(2)) {
        None => Some(b"GS0283"),
        Some(weapon) => (!matches!(weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1), 3 | 4))
            .then_some(b"GS0284"),
    };
    if let Some(text) = failure {
        game.update_registered_skill_visual(instance, 14);
        game.send_skill_system_info(player, text);
        return false;
    }
    true
}

pub(super) fn check_god_punishment_distance(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
) -> bool {
    checked_projectile_path(game, instance, false, player, None, properties).is_some()
}

fn checked_projectile_path(
    game: &mut CGame, instance: RegisteredSkill, archery: bool, player: Option<i32>,
    original_target: Option<(i32, ShapeIdentity)>, properties: &CSkillBaseProperties,
) -> Option<Vec<(i32, i32, u8)>> {
    let skill = game.registered_skill(instance)?;
    let path = game.skill_target_path(skill.lifecycle());
    if !archery {
        return check_skill_path(game, instance, properties, &path, player, CastPathBlock::Ignore).then_some(path);
    }
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() as u32 > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE)
            .wrapping_add(1)
    {
        game.update_registered_skill_visual(instance, 11);
        if let Some(player) = player {
            if let Some(target) = original_target {
                if let Some(target) = resolve_state_move_shape(game, target.0, target.1) {
                    game.send_skill_system_info_with_text(player, b"GS0280", target.shape().base_object().get_name());
                }
            } else { game.send_skill_system_info(player, b"GS0281"); }
        }
        return None;
    }
    Some(path)
}

pub(super) fn base_projectile_attack_path(game: &CGame, instance: RegisteredSkill, length: u32) -> Vec<(i32, i32, u8)> {
    game.registered_skill(instance)
        .map(|skill| game.skill_target_path_with_length(skill.lifecycle(), length))
        .unwrap_or_default()
}

pub(crate) fn publish_base_projectile_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    let (owner, has_flight_time) = match BaseProjectileKind::from_skill_id(skill.id()) {
        Some(kind) => (kind.owner(), kind != BaseProjectileKind::FireBall),
        None if skill.id() == GOD_PUNISHMENT_SKILL_ID => (SkillOwner::CGodPunishment, false),
        None => return,
    };
    if skill.owner() != owner
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::BaseProjectile || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let destination = if action == 2 {
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
        Some(match target {
            Some(target) => {
                let shape = target.shape();
                let x = shape.get_tile_x().unwrap_or(i32::MIN);
                let y = shape.get_tile_y().unwrap_or(i32::MIN);
                (shape.identity().object_type, shape.identity().id, x, y)
            }
            None => {
                let (x, y) = skill.lifecycle().destination();
                (0, 0, x, y)
            }
        })
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if let Some((target_type, target_id, x, y)) = destination {
        message.add_long(target_type);
        message.add_long(target_id);
        message.add_long(x);
        message.add_long(y);
        if has_flight_time {
            message.add_long(skill.base_projectile_progress().map_or(0, |progress| progress.attack_time_ms()));
        }
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
