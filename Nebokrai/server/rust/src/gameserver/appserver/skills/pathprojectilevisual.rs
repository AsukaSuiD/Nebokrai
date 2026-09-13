//! Общий visual пошагового снаряда EnergyBolt, SnakeBolt и ZombieClaw.
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/energybolt.cpp`, `snakebolt.cpp`, `zombieclaw.cpp`.
//! Режимы 0/1/3 публикуются вокруг живого U; режимы отказа 2/7/10/11/13/15
//! уходят только Player-U. Выпуск читает живого S через GetSufferer и при его
//! отсутствии передаёт сохранённый конец полёта; конец читает только снимок.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;

fn is_path_projectile_owner(owner: SkillOwner) -> bool {
    matches!(
        owner,
        SkillOwner::CEnergyBolt | SkillOwner::CSnakeBolt | SkillOwner::CZombieClaw
    )
}

fn send_around(game: &CGame, source: &crate::gameserver::appserver::shape::CShape, message: &CMessage) {
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, message);
    }
}

pub(crate) fn publish_path_projectile_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !is_path_projectile_owner(skill.owner())
        || skill.visual_effect().is_none_or(|effect|
            effect.kind() != SkillVisualEffectKind::PathProjectile || effect.is_ended())
    {
        return;
    }
    let (user_region, user) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, user_region, user).map(|shape| shape.shape()) else {
        return;
    };

    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            let mut message = CMessage::new(EFFECT_MESSAGE);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }

    match mode {
        0 => {
            let mut message = CMessage::new(EFFECT_MESSAGE);
            message.add_byte(1);
            message.add_long(skill.id() as i32);
            message.add_short(skill.level() as i16);
            message.add_long(source.identity().object_type);
            message.add_long(source.identity().id);
            message.add_long(source.get_direction());
            send_around(game, source, &message);
        }
        1 => {
            let Some(progress) = skill.path_projectile_progress() else { return; };
            let target = resolve_skill_sufferer(game, skill.lifecycle())
                .and_then(|(region, target)| resolve_state_move_shape(game, region, target))
                .map(|target| target.shape());
            let (target_type, target_id, target_x, target_y) = target.map_or_else(
                || {
                    let (end_x, end_y) = progress.end_position();
                    (0, 0, end_x, end_y)
                },
                |target| (
                    target.identity().object_type,
                    target.identity().id,
                    target.get_tile_x().unwrap_or(i32::MIN),
                    target.get_tile_y().unwrap_or(i32::MIN),
                ),
            );
            let mut message = CMessage::new(EFFECT_MESSAGE);
            message.add_byte(2);
            message.add_long(skill.id() as i32);
            message.add_short(skill.level() as i16);
            message.add_long(source.identity().object_type);
            message.add_long(source.identity().id);
            message.add_long(target_type);
            message.add_long(target_id);
            message.add_long(target_x);
            message.add_long(target_y);
            message.add_ulong(progress.missile_flying_time_ms());
            send_around(game, source, &message);
        }
        3 => {
            let Some(progress) = skill.path_projectile_progress() else { return; };
            let (end_x, end_y) = progress.end_position();
            let target = progress.visual_target();
            let mut message = CMessage::new(EFFECT_MESSAGE);
            message.add_byte(3);
            message.add_long(skill.id() as i32);
            message.add_short(skill.level() as i16);
            message.add_long(source.identity().object_type);
            message.add_long(source.identity().id);
            message.add_long(source.get_direction());
            message.add_long(end_x);
            message.add_long(end_y);
            message.add_long(target.map_or(0, |target| target.object_type));
            message.add_long(target.map_or(0, |target| target.id));
            send_around(game, source, &message);
        }
        _ => {}
    }
}
