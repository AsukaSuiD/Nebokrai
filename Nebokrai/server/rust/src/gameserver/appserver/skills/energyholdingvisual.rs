//! Визуальный ресурс CEnergyHoldingEffect.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/energyholding.cpp.
//!
//! Ресурс разрешает только свежего U, даже когда AI использует запасную S.
//! Подготовка пишет направление, выпуск — повторные type/id U и его X/Y.
//! Только ошибки 2/7/13 отправляют BYTE0/BYTEcode игроку; режим 14 и прочие
//! не создают пакет. Общий владелец выполняет базовый visual-tail всегда.

use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_energy_holding_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CEnergyHolding
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::EnergyHolding || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 13) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
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
