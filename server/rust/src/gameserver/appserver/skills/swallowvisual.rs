//! Визуальный ресурс двойного удара Swallow.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/swallow.cpp.
//!
//! Подготовка читает только живое направление U. Единственный ударный пакет
//! использует живую позицию U и сохранённое направление исполнения, а не
//! направление обхода боевых клеток. Ошибки отправляются самому игроку;
//! Around требует действительной связи U с регионом. Безусловный базовый
//! хвост visual принадлежит общему владельцу независимо от отправки пакета.

use super::skillfactory::SkillOwner;
use super::swallow::SwallowExecutionState;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_swallow_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CSwallow
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::Swallow || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let shape = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 4 | 7 | 10 | 11 | 13 | 14 | 15) {
        if shape.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), shape.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    if mode == 0 {
        message.add_long(shape.get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
        let x = shape.get_tile_x().unwrap_or(i32::MIN);
        let y = shape.get_tile_y().unwrap_or(i32::MIN);
        let Some(state) = skill.player_state::<SwallowExecutionState>() else { return; };
        // Native индексирует таблицу без проверки: ошибочное направление
        // не подменяется произвольной клеткой визуального эффекта.
        let Ok(front) = CShape::get_direction_position(state.direction(), ShapeAreaCoordinates { x, y }) else { return; };
        message.add_long(front.x);
        message.add_long(front.y);
    }
    if shape.is_assigned_to_server_region()
        && let Some(owner) = game.find_region(shape.get_region_id())
    {
        let _ = game.send_game_shape_around(owner.base(), shape, None, &message);
    }
}
