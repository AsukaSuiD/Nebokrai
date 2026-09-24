//! Общий визуальный ресурс семейства LeafCut.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut*.cpp.
//!
//! Подготовка читает только направление U; выпуск заново разрешает S и пишет
//! её type/id/X/Y, не подставляя сохранённую точку при исчезнувшей цели.
//! Коды ошибок одинаковы у всех трёх вариантов, включая RP-код 8. Around
//! использует действительную связь U с регионом. Базовый visual-tail
//! выполняется общим владельцем даже при отсутствии производного пакета.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;

pub(crate) fn publish_leaf_cut_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CLeafCut | SkillOwner::CLeafCut2 | SkillOwner::CLeafCut3)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::LeafCut || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 8 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if mode == 1 {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
        let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
        Some(target.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
