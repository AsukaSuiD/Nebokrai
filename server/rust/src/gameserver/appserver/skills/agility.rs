//! Состояния CAgility, CAgility2, CNatural и CRapture.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые appserver/skills owners.
//! Общий selfstatecast сохраняет Begin, исходного U, MP, delay, visual и End.
//! Создание по таблице свойств и выбор ветки — zone rules `skills/selfstate.rs`;
//! здесь живой обход Game. Постоянные варианты удаляют все встреченные DA/DB/DC
//! живым индексным обходом, Agility2 заменяет только первый ID81. Бонус читается
//! после завершения старых состояний; только Agility2 затем читает persist.
//! Begin(U,U), visual, append и безусловный UpdateProperty принадлежат владельцу
//! состояния. Отдельной публикации после замены нет.

use super::agilitystate::replace_persistent_agility_state;
use super::agilitystate2::replace_agility_state_2;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::{agility_state_2, persistent_agility_family_state};

pub(crate) use nebokrai_zone::effects::AGILITY_2_SKILL_ID;
pub(crate) use nebokrai_zone::effects::AGILITY_SKILL_ID;

pub(super) fn apply_agility_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    if skill_id == AGILITY_2_SKILL_ID {
        let _ = replace_agility_state_2(game, source, || {
            agility_state_2(|usage| properties.query_property(usage))
        }, &mut || runtime.now_milliseconds());
    } else {
        let _ = replace_persistent_agility_state(game, source, || {
            persistent_agility_family_state(skill_id, |usage| properties.query_property(usage))
        }, &mut || runtime.now_milliseconds());
    }
}
