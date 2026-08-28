//! Достигнутая часть ИИ синего босса `CBossBlue`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/bossblue.cpp` подтверждают восемь
//! одноразовых HP-порогов ярости. `CMonster` сбрасывает их после общего
//! пробуждения, а реальный путь `monsterbaseattack` выполняет один исходный
//! RNG-бросок перед пороговым или взвешенным выбором навыка. Накопление
//! `odds` намеренно учитывает доли исключённых ID `2` и `0x1f7`; применение
//! выбранного навыка остаётся у его skill-owner-а.
//!
//! `Run`, `Hibernate`, `OnIdle`, `OnSchedule` и `OnSearchEnemy` ниже остаются
//! RAW: их специальные переходы и поиск цели ещё не подключены к runtime.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp

// ============================================================================
// FUNCTION: CBossBlue::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:48
// RVA: 0x00209340
// ADDRESS: 00609340
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlue::Hibernate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:513
// RVA: 0x002093B0
// ADDRESS: 006093b0
// PROTOTYPE: void __thiscall Hibernate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlue::CBossBlue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:19
// RVA: 0x00209CB0
// ADDRESS: 00609cb0
// PROTOTYPE: undefined __thiscall CBossBlue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlue::OnIdle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:302
// RVA: 0x00209E40
// ADDRESS: 00609e40
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlue::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:93
// RVA: 0x00209FE0
// ADDRESS: 00609fe0
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlue::OnSearchEnemy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp:197
// RVA: 0x0020A3E0
// ADDRESS: 0060a3e0
// PROTOTYPE: int __thiscall OnSearchEnemy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

use crate::setup::monsterlist::MonsterSkill;

const BOSS_BLUE_FURY_SKILL_ID: u16 = 0x1f7;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Восемь одноразовых порогов ярости принадлежат конкретному ИИ синего босса.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BossBlueAiState {
    fury_used: [bool; 8],
}

impl BossBlueAiState {
    /// `CBossBlue::WakeUp` вызывается после общего восстановления HP и открывает
    /// только ещё не пройденные пороги текущей фазы жизни.
    pub(crate) fn wake(&mut self, hit_points: u32, maximum_hit_points: u32) {
        self.fury_used.fill(true);
        let health_rate = hit_points as f32 / maximum_hit_points as f32;
        let first_available = if health_rate > 0.83 {
            0
        } else if health_rate > 0.67 {
            1
        } else if health_rate > 0.5 {
            2
        } else if health_rate > 0.4 {
            3
        } else if health_rate > 0.3 {
            4
        } else if health_rate > 0.2 {
            5
        } else if health_rate > 0.15 {
            6
        } else if health_rate > 0.1 {
            7
        } else {
            8
        };
        for used in &mut self.fury_used[first_available..] {
            *used = false;
        }
    }
}

/// Сохраняет один исходный бросок, HP-пороги и накопление `odds`, включая
/// доли исключённых навыков `2` и `0x1f7` перед проверкой следующей записи.
pub(crate) fn select_boss_blue_attack_skill(
    state: &mut BossBlueAiState,
    hit_points: u32,
    maximum_hit_points: u32,
    has_fury_state: bool,
    skills: &[MonsterSkill],
    roll: i32,
    default_skill_id: u16,
) -> u16 {
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
        && !state.fury_used[index]
    {
        state.fury_used[index] = true;
        return BOSS_BLUE_FURY_SKILL_ID;
    }
    if health_rate < 0.08 && !has_fury_state {
        return BOSS_BLUE_FURY_SKILL_ID;
    }

    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if !matches!(skill.id, EXCLUDED_ARCHERY_SKILL_ID | BOSS_BLUE_FURY_SKILL_ID)
            && roll <= cumulative_odds
        {
            return skill.id;
        }
    }
    default_skill_id
}
