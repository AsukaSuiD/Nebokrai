//! Параметры однократных областей инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, yinyangphalanx{,2}.cpp.
//! Первый вариант имеет полную маску 3×3 на всех уровнях, второй — 1×1.
//! Общий масочный владелец сохраняет живой X→Y обход и End после попаданий.
//! При создании инь-ян не заменяет маски соседей; отклонённые допуском цели
//! не входят в дедупликацию, в отличие от периодической огненной стены.

use super::elementphalanxattack::ElementPhalanxAttack;
use super::maskedelementphalanx::{MaskedAreaPulse, MaskedElementPhalanx};
use crate::gameserver::appserver::masterinfo::MasterInfo;

pub(super) fn scope_for_skill(skill_id: u32) -> (i32, i32, &'static [bool]) {
    if skill_id == super::yinyang2::YIN_YANG_2_SKILL_ID {
        super::yinyangphalanx2::YIN_YANG_2_SCOPE
    } else { (3, 3, &[true; 9]) }
}

#[allow(clippy::too_many_arguments, reason = "параметры исходного конструктора инь-ян")]
pub(super) fn new_yin_yang_phalanx(
    skill_id: u32, id: i32, master: MasterInfo, started: u32, lifetime: u32,
    skill_level: i32, minimum: i32, maximum: i32, element: i32, critical_chance: i32,
) -> MaskedElementPhalanx {
    MaskedElementPhalanx::new(
        id, ElementPhalanxAttack { master, skill_id, skill_level, minimum, maximum, element, critical_chance },
        started, lifetime, MaskedAreaPulse::Once, scope_for_skill(skill_id),
    )
}
