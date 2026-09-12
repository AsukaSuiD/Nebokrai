//! Визуальный ресурс семейства GhostCut.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/ghostcut*.cpp.
//!
//! Три варианта имеют один wire-формат. Выпуск заново разрешает S либо читает
//! базовую точку; продолжительность берётся из канонического исполнения,
//! а не рассчитывается при отправке. Mode 3 не создаёт пакета. Безусловный
//! базовый хвост, включая завершённый ресурс и отсутствующего U, принадлежит
//! общему владельцу visual.

use super::ghostcut::GhostCutExecutionState;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_ghost_cut_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CGhostCut | SkillOwner::CGhostCut2 | SkillOwner::CGhostCut3)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::GhostCut || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let shape = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if shape.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), shape.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if mode == 1 {
        resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map(|target| target.shape())
    } else { None };
    let destination = target.map_or_else(|| skill.lifecycle().destination(), |target| {
        (target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN))
    });
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    if mode == 0 {
        message.add_long(shape.get_direction());
    } else {
        message.add_long(target.map_or(0, |target| target.identity().object_type));
        message.add_long(target.map_or(0, |target| target.identity().id));
        message.add_long(destination.0);
        message.add_long(destination.1);
        let Some(state) = skill.player_state::<GhostCutExecutionState>() else { return; };
        message.add_ulong(state.missile_flying_time());
    }
    if shape.is_assigned_to_server_region()
        && let Some(owner) = game.find_region(shape.get_region_id())
    {
        let _ = game.send_game_shape_around(owner.base(), shape, None, &message);
    }
}
