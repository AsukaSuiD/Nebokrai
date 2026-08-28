//! Достигнутая часть ИИ демона-босса `CBossFiend`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/bossfiend.cpp` подтверждают восемь
//! одноразовых HP-порогов призыва и повторный призыв ниже 8% HP по строгой
//! проверке исходного таймера. Реальный путь `monsterbaseattack` сохраняет
//! один RNG-бросок до порогового выбора, отдельные чтения времени для проверки
//! и фиксации призыва и накопление `odds` через исключённые ID `1`, `2` и
//! `0x1f9`. Выполнение выбранного навыка остаётся у skill-owner-а.
//!
//! `OnIdle`, `OnSchedule`, `OnSearchEnemy` и `OnMoving` ниже остаются RAW: их
//! специальные переходы и поиск цели ещё не подключены к runtime.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp

// ============================================================================
// FUNCTION: CBossFiend::OnIdle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp:326
// RVA: 0x002093C0
// ADDRESS: 006093c0
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiend::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp:98
// RVA: 0x00209560
// ADDRESS: 00609560
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiend::OnSearchEnemy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp:168
// RVA: 0x002099F0
// ADDRESS: 006099f0
// PROTOTYPE: int __thiscall OnSearchEnemy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiend::OnMoving
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp:35
// RVA: 0x0020C4F0
// ADDRESS: 0060c4f0
// PROTOTYPE: int __thiscall OnMoving(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

use crate::setup::monsterlist::MonsterSkill;

const BOSS_FIEND_SUMMON_SKILL_ID: u16 = 0x1f9;
const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Одноразовые пороги призыва и время последнего принудительного призыва
/// принадлежат конкретному экземпляру ИИ демона-босса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossFiendAiState {
    summon_used: [bool; 8],
    last_summon_ms: u32,
}

impl BossFiendAiState {
    /// Исходный конструктор фиксирует `timeGetTime` и открывает все восемь
    /// HP-порогов ровно в момент создания AI-owner-а.
    pub(crate) const fn new(now_ms: u32) -> Self {
        Self {
            summon_used: [false; 8],
            last_summon_ms: now_ms,
        }
    }

    pub(crate) fn record_summon(&mut self, threshold: Option<usize>, now_ms: u32) {
        if let Some(threshold) = threshold {
            self.summon_used[threshold] = true;
        }
        self.last_summon_ms = now_ms;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossFiendSkillSelection {
    pub(crate) skill_id: u16,
    pub(crate) summon_threshold: Option<usize>,
    pub(crate) records_summon: bool,
}

/// Сохраняет один исходный RNG-бросок, восемь HP-порогов, строгую проверку
/// низкоуровневого таймера и накопление `odds` через исключённые записи.
/// `None` соответствует исходному возврату без назначения текущего навыка.
pub(crate) fn select_boss_fiend_attack_skill(
    state: &BossFiendAiState,
    hit_points: u32,
    maximum_hit_points: u32,
    skills: &[MonsterSkill],
    roll: i32,
    timer_check_ms: Option<u32>,
    summon_persist_modifier: Option<u32>,
) -> Option<BossFiendSkillSelection> {
    let health_rate = hit_points as f32 / maximum_hit_points as f32;
    let threshold = if (0.67..0.83).contains(&health_rate) {
        Some(0)
    } else if (0.5..0.67).contains(&health_rate) {
        Some(1)
    } else if (0.4..0.5).contains(&health_rate) {
        Some(2)
    } else if (0.3..0.4).contains(&health_rate) {
        Some(3)
    } else if (0.2..0.3).contains(&health_rate) {
        Some(4)
    } else if (0.15..0.2).contains(&health_rate) {
        Some(5)
    } else if (0.1..0.15).contains(&health_rate) {
        Some(6)
    } else if (0.08..0.1).contains(&health_rate) {
        Some(7)
    } else {
        None
    };
    if let Some(index) = threshold
        && !state.summon_used[index]
    {
        return Some(BossFiendSkillSelection {
            skill_id: BOSS_FIEND_SUMMON_SKILL_ID,
            summon_threshold: Some(index),
            records_summon: true,
        });
    }
    if health_rate < 0.08 {
        let persist_modifier = summon_persist_modifier?;
        let timer_check_ms = timer_check_ms?;
        if state.last_summon_ms.wrapping_add(persist_modifier) < timer_check_ms {
            return Some(BossFiendSkillSelection {
                skill_id: BOSS_FIEND_SUMMON_SKILL_ID,
                summon_threshold: None,
                records_summon: true,
            });
        }
    }

    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if !matches!(
            skill.id,
            EXCLUDED_BASE_ATTACK_SKILL_ID
                | EXCLUDED_ARCHERY_SKILL_ID
                | BOSS_FIEND_SUMMON_SKILL_ID
        ) && roll <= cumulative_odds
        {
            return Some(BossFiendSkillSelection {
                skill_id: skill.id,
                summon_threshold: None,
                records_summon: false,
            });
        }
    }
    None
}
