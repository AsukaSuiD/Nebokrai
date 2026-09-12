//! Общий visual навыков на себя: EnergyHolding, Pillar, Roar, Callosity1/2
//! и семейства Agility/Agility2/Natural/Rapture.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые owners appserver/skills.
//!
//! Подготовка передаёт направление свежего U, выпуск повторяет его type/id
//! и X/Y. S и точка Begin не используются, даже когда AI допускает запасную S.
//! Ошибки 2/7/13 общие; Roar добавляет 14, Pillar и Callosity — 8.
//! Ошибка 8 имеет LONG0/BYTE8 вместо BYTE0/BYTEcode. Другие режимы не создают пакет.
//! Around требует действительной связи U с регионом; общий dispatcher
//! сохраняет базовый visual-tail независимо от производной публикации.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_self_cast_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CEnergyHolding | SkillOwner::CPillar | SkillOwner::CRoar
        | SkillOwner::CCallosity | SkillOwner::CCallosity2 | SkillOwner::CAgility
        | SkillOwner::CAgility2 | SkillOwner::CNatural | SkillOwner::CRapture)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::SelfCast || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    let long_failure = matches!(skill.owner(), SkillOwner::CPillar | SkillOwner::CCallosity | SkillOwner::CCallosity2) && mode == 8;
    if matches!(mode, 2 | 7 | 13) || long_failure
        || (skill.owner() == SkillOwner::CRoar && mode == 14)
    {
        if source.identity().object_type == 400 {
            if long_failure { message.add_long(0); } else { message.add_byte(0); }
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 1 {
        message.add_long(source.identity().object_type);
        message.add_long(source.identity().id);
        message.add_long(source.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(source.get_tile_y().unwrap_or(i32::MIN));
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
