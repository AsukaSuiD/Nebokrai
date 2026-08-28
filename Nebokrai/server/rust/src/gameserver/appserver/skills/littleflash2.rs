//! Второй малый рывок `CLittleFlash2` (`0x7f`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/littleflash2.cpp`. Навык использует общий с
//! `CLittleFlash` pipeline, wire-эффект и damage-формулу, но допускает
//! координатную цель, не требует сохранённого sufferer во время `AI`, очищает
//! путь из единственной заблокированной клетки и сообщает `GS0309` при пустом
//! пути. Эти различия остаются в данном owner-е, а общая семейная механика
//! исполняется из `littleflash.rs`.

use crate::gameserver::appserver::player::PlayerSkillDispatch;

pub(crate) const LITTLE_FLASH_2_SKILL_ID: u32 = 0x7f;
pub(super) const EMPTY_PATH_MESSAGE_ID: &[u8] = b"GS0309";

pub(super) const fn is_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget {
            skill_id: LITTLE_FLASH_2_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Point {
            skill_id: LITTLE_FLASH_2_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: LITTLE_FLASH_2_SKILL_ID,
            ..
        }
    )
}

pub(super) const fn point_destination(dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point {
            skill_id: LITTLE_FLASH_2_SKILL_ID,
            x,
            y,
        } => Some((x, y)),
        _ => None,
    }
}

pub(super) const fn clears_single_blocked_cell(path_length: usize, blocked: bool) -> bool {
    blocked && path_length == 1
}
