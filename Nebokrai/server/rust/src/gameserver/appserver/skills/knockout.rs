//! Адаптер диспетчеризации `CKnockOut` (`0x192`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knockout.cpp`. Рабочее исполнение находится в соседнем
//! `knockoutruntime.rs`. Все формы доходят до общего Begin. Поиск первой
//! движущейся цели принадлежит CState; ограничения типов здесь отсутствуют.

use super::knockoutruntime::KNOCK_OUT_SKILL_ID;
use crate::gameserver::appserver::player::PlayerSkillDispatch;

pub(crate) const fn is_knock_out_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == KNOCK_OUT_SKILL_ID
}
