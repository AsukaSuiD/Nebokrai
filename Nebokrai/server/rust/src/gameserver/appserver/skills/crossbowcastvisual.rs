//! Visual арбалетных PoisonMoth и BloodRose.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/poisonmoth.cpp
//! и bloodrose.cpp. Общие пакеты содержат идентификатор навыка, уровень и
//! свежего U. Mode0 передаёт направление; mode3 добавляет сохранённую конечную
//! клетку и type/id последней либо выбранной цели без её повторного поиска.
//! Выпуск CF не вызывает GetS: пишет нулевые type/id и базовую точку. D0
//! разрешает S заново и передаёт её type/id/X/Y либо базовую точку. Оба
//! добавляют время полёта. Ошибки — BYTE0/BYTEcode; CF не посылает mode4/14.
//! Общий dispatcher сохраняет базовый visual-tail независимо от пакета.

use super::bloodrose::BloodRoseExecutionState;
use super::poisonmoth::PoisonMothExecutionState;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) fn publish_crossbow_cast_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    let poison_moth = match skill.owner() {
        SkillOwner::CPoisonMoth => true,
        SkillOwner::CBloodRose => false,
        _ => return,
    };
    if skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::CrossbowCast || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) || (!poison_moth && matches!(mode, 4 | 14)) {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    let target = if mode == 1 {
        let resolved = if poison_moth { None } else { resolve_skill_sufferer(game, skill.lifecycle()) };
        Some(match resolved {
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
        let duration = if poison_moth {
            let Some(state) = skill.player_state::<PoisonMothExecutionState>() else { return; };
            state.missile_flying_time()
        } else {
            let Some(state) = skill.player_state::<BloodRoseExecutionState>() else { return; };
            state.missile_flying_time()
        };
        message.add_ulong(duration);
    } else {
        message.add_long(source.get_direction());
        if mode == 3 {
            let (end, target) = if poison_moth {
                let Some(state) = skill.player_state::<PoisonMothExecutionState>() else { return; };
                (state.end_tile(), state.visual_target())
            } else {
                let Some(state) = skill.player_state::<BloodRoseExecutionState>() else { return; };
                (state.end_tile(), state.visual_target())
            };
            message.add_long(end.0);
            message.add_long(end.1);
            message.add_long(target.0);
            message.add_long(target.1);
        }
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
