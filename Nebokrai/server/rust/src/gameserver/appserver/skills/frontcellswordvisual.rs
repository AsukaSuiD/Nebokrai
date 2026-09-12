//! Общий visual JuCut, LightningSword1–4 и InverseChopped.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/jucut.cpp,
//! lightningsword*.cpp и inversechopped.cpp.
//!
//! Подготовка передаёт живое направление U, выпуск заново разрешает S и её
//! X/Y либо базовую точку с нулевыми type/id. Это не цель боевого обхода:
//! frontcellsword независимо получает лицевую клетку после visual1.
//! Inverse поддерживает только ошибки 2/7/13/14; остальные варианты также
//! 10/11/15. Around требует действительной связи U с регионом; общий владелец
//! выполняет базовый visual-tail даже без производного пакета.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_front_cell_sword_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CJuCut | SkillOwner::CLightningSword
        | SkillOwner::CLightningSword2 | SkillOwner::CLightningSword3
        | SkillOwner::CLightningSword4 | SkillOwner::CInverseChopped)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::FrontCellSword || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 13 | 14)
        || (skill.owner() != SkillOwner::CInverseChopped && matches!(mode, 10 | 11 | 15))
    {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if mode == 1 {
        Some(match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
                let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
                let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
                (target.shape().identity().object_type, target.shape().identity().id, x, y)
            }
            None => {
                let (x, y) = skill.lifecycle().destination();
                (0, 0, x, y)
            }
        })
    } else { None };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some((kind, id, x, y)) = target {
        message.add_long(kind);
        message.add_long(id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
