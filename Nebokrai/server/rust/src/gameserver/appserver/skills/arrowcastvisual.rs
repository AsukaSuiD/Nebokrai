//! Общий visual CLightingArrow и CLightingArrow2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightingarrow.cpp
//! и lightingarrow2.cpp. Подготовка передаёт направление свежего U; выпуск
//! заново разрешает S, её X/Y либо базовую точку. Очистка сохранённой identity
//! не запрещает GetS найти фигуру по координатам. Только E7 добавляет время
//! полёта и поддерживает mode3 с направлением U. Ошибки — BYTE0/BYTEcode.
//! Around требует действительной связи U с регионом; общий dispatcher
//! сохраняет базовый visual-tail даже без производного сообщения.

use super::lightingarrow2::LightingArrow2ExecutionState;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_arrow_cast_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CLightingArrow | SkillOwner::CLightingArrow2)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::ArrowCast || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let second = skill.owner() == SkillOwner::CLightingArrow2;
    let action = match mode { 0 => 1, 1 => 2, 3 if second => 3, _ => return };
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
        if second {
            let duration = skill.player_state::<LightingArrow2ExecutionState>()
                .map_or(0, LightingArrow2ExecutionState::missile_flying_time);
            message.add_ulong(duration);
        }
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
