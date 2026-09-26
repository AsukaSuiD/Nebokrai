//! Правила зарегистрированной self-state семьи: Agility `0xDA`, Agility2
//! `0x81`, Natural `0xDC`, Rapture `0xDB`, DaubPoison и щиты ManaShield/
//! MachineShield: ID-карта семьи, выбор ветки состояния, текст MP-отказа и
//! создание Agility-состояний после завершения старых.
//!
//! Quirks: ManaShield не проверяет смерть U и удаляет первый встреченный ID
//! `0x141`; Agility2 заменяет только первый `0x81`; постоянное семейство
//! завершает все `0xDA/0xDB/0xDC`; visual object Begin — loop1 у постоянных,
//! loop0 у временной Agility2; BFE03/BFE04 несут time=0 и additional=0.
//!
//! Живой обход Game (арена, участники, очередь, часы) — за переходным
//! `appserver/skills/{selfstatecast,selfshield,agility,agilitystate,
//! agilitystate2}.rs`; данные состояний и формулы — zone/effects.
//! VERIFIED (дизассемблинг, `.local/verify-selfstate/REPORT.md`): тела AI
//! CNatural `0x16C0B0`, CRapture `0x16CBB0` и CMachineShield `0x1671C0` —
//! точный клон-шаблон эталона CManaShield::AI; Natural/Rapture проверяют
//! смерть U (visual2/End(1)), щиты — нет; скан состояний — все
//! `0xDA/0xDB/0xDC` у пары, первый `0xDE` у MachineShield; ctor-маппинг
//! MachineShield `10002/10010/20024/20025` подтверждён (`CDaubPoison::AI` —
//! VERIFIED ранее по recon-de). Клиентское чтение кадров — UNKNOWN.
//!
//! Исходные владельцы PDB: `appserver/skills/{agility,agility2,natural,
//! rapture,daubpoison,manashield,machineshield}.cpp` и owners состояний.
//! Доказательства: docs/reconstruction/gameserver-skills.md#selfstate--правила-self-state-семьи

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
