//! Визуальный ресурс фронтального удара Мо-шоу.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/mosou.cpp.
//!
//! Подготовка читает только направление U, удар отдельно получает его живую
//! лицевую клетку до записи сообщения. Ошибки отправляются самому игроку;
//! Around требует действительной связи U с регионом. Безусловный базовый
//! хвост visual принадлежит общему владельцу и не зависит от отправки пакета.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_mosou_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CMosou
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::Mosou || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let shape = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if shape.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), shape.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let face = if mode == 1 {
        let position = ShapeAreaCoordinates {
            x: shape.get_tile_x().unwrap_or(i32::MIN),
            y: shape.get_tile_y().unwrap_or(i32::MIN),
        };
        let Ok(face) = CShape::get_direction_position(shape.get_direction(), position) else { return; };
        Some(face)
    } else { None };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    if let Some(face) = face {
        message.add_long(0);
        message.add_long(0);
        message.add_long(face.x);
        message.add_long(face.y);
    } else {
        message.add_long(shape.get_direction());
    }
    if shape.is_assigned_to_server_region()
        && let Some(owner) = game.find_region(shape.get_region_id())
    {
        let _ = game.send_game_shape_around(owner.base(), shape, None, &message);
    }
}
