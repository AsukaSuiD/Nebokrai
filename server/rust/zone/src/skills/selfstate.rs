//! Правила зарегистрированной self-state семьи: Agility `0xDA`, Agility2
//! `0x81`, Natural `0xDC`, Rapture `0xDB`, DaubPoison и щиты ManaShield/
//! MachineShield.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb` (пара
//! `4F5C98E0…`, RSDS match), `appserver/skills/{agility,agility2,natural,
//! rapture,daubpoison,manashield,machineshield}.cpp` и owners состояний.
//!
//! Машинно установлено: Check после reuse (usage 10005 общего cast) не даёт
//! источнику не типа Player Move0, а у игрока MP0 означает тихий отказ;
//! щиты требуют
//! Player и допускают Move0 даже при MP0. Строки ошибок GS0278/GS0279/GS0288
//! байт-сверены. AI: первая фаза читает MP, заново спрашивает цену, выполняет
//! signed DWORD-проверку, затем SetMP → OnChangeStates → CAN → visual0 →
//! condition; выпуск ждёт unsigned start+delay и публикует visual1. Смерть U
//! у пяти усилений даёт visual2/End(1), ManaShield её не проверяет и удаляет
//! первый встреченный ID `0x141`; Agility2 заменяет только первый `0x81`, а
//! постоянное семейство обходит и завершает все `0xDA/0xDB/0xDC`. Бонус и
//! persist читаются после завершения старых состояний; Agility2 затем
//! дополнительно читает persist. Visual object Begin — loop1 у постоянных,
//! loop0 у временной Agility2; BFE03 и BFE04 несут time=0 (клиентский
//! остаток — у Agility2) и additional=0.
//! UNKNOWN честные: тела `CNatural::AI`, `CRapture::AI`, `CDaubPoison::AI`,
//! `CMachineShield::AI` и ctor-маппинг MachineShield дизассемблом не сняты;
//! разделяемые ими строки/цепочки подтверждены по ManaShield
//! (ctor-мэппинг keep `0x2712`, life `0x271A`, phys `0x271B`, elem `0x271C`,
//! `0x4E38`/`0x4E39` и MP-отказ visual7+GS0288 — 1:1).
//!
//! Живой обход Game (арена состояний, участники, очередь исполнения, часы)
//! остаётся за переходным `appserver/skills/{selfstatecast,selfshield,
//! agility,agilitystate,agilitystate2}.rs`; данные состояний и формулы —
//! zone/effects. Здесь ID-карта семьи, выбор ветки состояния, текст MP-отказа
//! и создание Agility-состояний по таблице свойств после завершения старых.

use crate::effects::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AgilityState2, MACHINE_SHIELD_SKILL_ID,
    MANA_SHIELD_SKILL_ID, NATURAL_SKILL_ID, PersistentAgilityFamilyState, RAPTURE_SKILL_ID,
};

use super::daubpoison::DAUB_POISON_SKILL_ID;

const FULL_MISS_GAIN: u32 = 127;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;
const TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;

/// Visual object Begin: loop1 у постоянных Agility/Natural/Rapture.
pub const PERSISTENT_AGILITY_FAMILY_VISUAL_LOOP: i32 = 1;
/// Visual object Begin: loop0 у временной Agility2.
pub const AGILITY_2_VISUAL_LOOP: i32 = 0;

/// Два щита семьи; общий Check требует для них источник типа Player.
pub const fn is_self_shield_skill(skill_id: u32) -> bool {
    matches!(skill_id, MANA_SHIELD_SKILL_ID | MACHINE_SHIELD_SKILL_ID)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfStateBranch {
    AgilityFamily,
    DaubPoison,
    SelfShield,
}

/// Ветка наложения состояния после visual1 того же AI.
pub const fn self_state_branch(skill_id: u32) -> Option<SelfStateBranch> {
    if matches!(skill_id, AGILITY_SKILL_ID | AGILITY_2_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID) {
        Some(SelfStateBranch::AgilityFamily)
    } else if skill_id == DAUB_POISON_SKILL_ID {
        Some(SelfStateBranch::DaubPoison)
    } else if is_self_shield_skill(skill_id) {
        Some(SelfStateBranch::SelfShield)
    } else {
        None
    }
}

/// Строка MP-отказа Check и первого AI (visual7): GS0288 у Natural, DaubPoison
/// и щитов; GS0279 у остальных усилений семьи. Строки — данные, байт-сверены.
pub const fn self_state_mana_failure_text(skill_id: u32) -> &'static [u8] {
    if matches!(skill_id, NATURAL_SKILL_ID | DAUB_POISON_SKILL_ID) || is_self_shield_skill(skill_id) {
        b"GS0288"
    } else {
        b"GS0279"
    }
}

/// Временная Agility2: вызывается после End первого прежнего ID, full-miss
/// читается перед persist; часы старта заполняет установщик Game.
pub fn agility_state_2(mut query_property: impl FnMut(u32) -> u32) -> AgilityState2 {
    let full_miss = query_property(FULL_MISS_GAIN) as u16;
    let keep = query_property(STATE_PERSIST_TIME) as i32;
    AgilityState2::new(full_miss, 0, keep)
}

/// Постоянные варианты: вызывается после живого обхода, завершившего все
/// прежние состояния семейства `0xDA/0xDB/0xDC`.
pub fn persistent_agility_family_state(
    skill_id: u32, mut query_property: impl FnMut(u32) -> u32,
) -> PersistentAgilityFamilyState {
    match skill_id {
        NATURAL_SKILL_ID => PersistentAgilityFamilyState::Natural {
            element_resistance_gain: query_property(TARGET_ELEMENT_RESISTANT_GAIN) as u16,
        },
        RAPTURE_SKILL_ID => PersistentAgilityFamilyState::Rapture {
            blast_attack_gain: query_property(TARGET_BLAST_COEFFICIENT_GAIN) as u16,
        },
        AGILITY_SKILL_ID => PersistentAgilityFamilyState::Agility {
            full_miss: query_property(FULL_MISS_GAIN) as u16,
        },
        _ => unreachable!("общий selfstatecast выбирает только семейство Agility"),
    }
}
