//! Параметры однократных областей инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, yinyangphalanx{,2}.cpp.
//! Выбор маски по исходному skill ID принадлежит zone/skills/yinyang.rs.
//! Общий масочный владелец сохраняет живой X→Y обход и End после попаданий.
//! При создании инь-ян не заменяет маски соседей; отклонённые допуском цели
//! не входят в дедупликацию, в отличие от периодической огненной стены.

use nebokrai_zone::skills::ElementPhalanxAttack;
use super::maskedelementphalanx::{MaskedAreaPulse, MaskedElementPhalanx};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use nebokrai_zone::skills::{YinYangSummonParameters, yin_yang_scope};

pub(super) fn new_yin_yang_phalanx(
    id: i32, master: MasterInfo, started: u32, element: i32,
    critical_chance: i32, parameters: YinYangSummonParameters,
) -> MaskedElementPhalanx {
    MaskedElementPhalanx::new(
        id, ElementPhalanxAttack {
            master, skill_id: parameters.skill_id, skill_level: parameters.skill_level,
            minimum: parameters.minimum_attack, maximum: parameters.maximum_attack,
            element, critical_chance,
        },
        started, parameters.lifetime_ms, MaskedAreaPulse::Once,
        yin_yang_scope(parameters.skill_id),
    )
}
