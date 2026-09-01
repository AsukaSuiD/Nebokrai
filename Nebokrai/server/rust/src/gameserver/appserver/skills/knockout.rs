//! Адаптер диспетчеризации `CKnockOut` (`0x192`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knockout.cpp`. Рабочее исполнение находится в соседнем
//! `knockoutruntime.rs`. Этот адаптер сохраняет выбор перегрузки: координатная
//! и пустая формы доходят до owner-а, где null target даёт только failure `2`
//! и `End(0)`; объектная форма допускает игрока либо монстра.

use super::knockoutruntime::KNOCK_OUT_SKILL_ID;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

pub(crate) const fn is_knock_out_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget {
            skill_id: KNOCK_OUT_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Point {
            skill_id: KNOCK_OUT_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: KNOCK_OUT_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
        }
    )
}
