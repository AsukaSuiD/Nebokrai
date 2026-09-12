//! Визуальный ресурс рыцарского удара.
//! Источник: gameserver.exe/GameServer.pdb, CKnightCutEffect::UpdateVisualEffect,
//! appserver/skills/knightcut.cpp.
//!
//! Режим 0 передаёт живое направление U, режим 1 — его лицевую клетку по
//! сохранённому направлению навыка, без S. Ошибки 2/7/8/13/14 отправляются
//! только игроку U. Остальные режимы, включая 3, не создают пакет.
//! Общий visual-owner всегда выполняет базовый хвост, также для завершённого
//! ресурса и отсутствующего U.

use super::knightcut::KnightCutExecutionState;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_knight_cut_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CKnightCut
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::KnightCut || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let shape = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 8 | 13 | 14) {
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
    if mode == 1 {
        message.add_long(0);
        message.add_long(0);
        let x = shape.get_tile_x().unwrap_or(i32::MIN);
        let y = shape.get_tile_y().unwrap_or(i32::MIN);
        let direction = skill.player_state::<KnightCutExecutionState>()
            .map_or(-1, KnightCutExecutionState::direction);
        let position = ShapeAreaCoordinates { x, y };
        let front = CShape::get_direction_position(direction, position).unwrap_or(position);
        message.add_long(front.x);
        message.add_long(front.y);
    } else {
        message.add_long(shape.get_direction());
    }
    if shape.is_assigned_to_server_region()
        && let Some(owner) = game.find_region(shape.get_region_id())
    {
        let _ = game.send_game_shape_around(owner.base(), shape, None, &message);
    }
}
