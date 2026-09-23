//! Состояния CAgility, CAgility2, CNatural и CRapture.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые appserver/skills owners.
//! Общий selfstatecast сохраняет Begin, исходного U, MP, delay, visual и End.
//! Здесь остаются различия наложения после visual1 и из таблицы начала AI:
//! постоянные варианты удаляют все встреченные DA/DB/DC живым индексным
//! обходом, Agility2 заменяет только первый ID81. Бонус читается после
//! завершения старых состояний; только Agility2 затем читает persist.
//! Begin(U,U), visual, append и безусловный UpdateProperty принадлежат владельцу
//! состояния. Отдельной публикации после замены нет.

pub(crate) use super::agility2::AGILITY_2_SKILL_ID;
use super::agilitystate::{PersistentAgilityFamilyState, replace_persistent_agility_state};
use super::agilitystate2::{AgilityState2, replace_agility_state_2};
use super::natural::{NATURAL_SKILL_ID, SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN};
use super::rapture::{RAPTURE_SKILL_ID, SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::effects::AGILITY_SKILL_ID;
const TARGET_FULL_MISS_GAIN: u32 = 127;
const STATE_PERSIST_TIME: u32 = 10_002;

pub(super) fn apply_agility_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    if skill_id == AGILITY_2_SKILL_ID {
        let _ = replace_agility_state_2(game, source, || {
            let full_miss = properties.query_property(TARGET_FULL_MISS_GAIN) as u16;
            let keep = properties.query_property(STATE_PERSIST_TIME) as i32;
            AgilityState2::new(full_miss, 0, keep)
        }, &mut || runtime.now_milliseconds());
    } else {
        let _ = replace_persistent_agility_state(game, source, || match skill_id {
            NATURAL_SKILL_ID => PersistentAgilityFamilyState::Natural {
                element_resistance_gain: properties.query_property(SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN) as u16,
            },
            RAPTURE_SKILL_ID => PersistentAgilityFamilyState::Rapture {
                blast_attack_gain: properties.query_property(SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN) as u16,
            },
            AGILITY_SKILL_ID => PersistentAgilityFamilyState::Agility {
                full_miss: properties.query_property(TARGET_FULL_MISS_GAIN) as u16,
            },
            _ => unreachable!("общий selfstatecast выбирает только семейство Agility"),
        }, &mut || runtime.now_milliseconds());
    }
}
