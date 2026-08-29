//! Достигнутая часть ИИ владыки `CLord`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/lord.cpp` подтверждают один RNG-бросок и
//! зависимое от доли HP сжатие его шкалы перед упорядоченным выбором навыка.
//! Этот владелец выбирает навык и ближайшую живую цель, реальный путь
//! `monsterbaseattack` назначает результат, а `lordfastattack` и
//! `lordwiderangingattack` исполняют конкретные стадии и эффекты.
//!
//! `WhenBeenHurted` ниже остаётся RAW: специальное уклонение от summon-shape
//! ещё не подключено.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\lord.cpp

// ============================================================================
// FUNCTION: CLord::WhenBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\lord.cpp:32
// RVA: 0x0020B0B0
// ADDRESS: 0060b0b0
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use super::guardtarget::select_nearest_player_or_pet;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterSkill;

const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Выполняет подтверждённый `OnSearchEnemy` AI100 через общий nearest-проход,
/// сохраняя игроков перед питомцами и замену при равной дистанции.
pub(crate) fn select_lord_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    select_nearest_player_or_pet(game, region, owner, area_index, guard_range)
        .map(|selected| selected.identity)
}

/// Сохраняет единственный исходный бросок и зависимое от HP сжатие его шкалы:
/// в диапазоне `[20%, 50%)` применяется `ROUND(roll * 0.6666667)`, ниже 20% —
/// целочисленное деление на два. Исключённые ID продолжают накапливать `odds`.
pub(crate) fn select_lord_attack_skill(
    hit_points: u32,
    maximum_hit_points: u32,
    skills: &[MonsterSkill],
    roll: i32,
    default_skill_id: u16,
) -> u16 {
    let health_rate = hit_points as f32 / maximum_hit_points as f32;
    let adjusted_roll = if health_rate >= 0.2 {
        if health_rate < 0.5 {
            (roll as f32 * 0.666_666_7).round_ties_even() as i32
        } else {
            roll
        }
    } else {
        roll / 2
    };

    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if !matches!(
            skill.id,
            EXCLUDED_BASE_ATTACK_SKILL_ID | EXCLUDED_ARCHERY_SKILL_ID
        ) && adjusted_roll <= cumulative_odds
        {
            return skill.id;
        }
    }
    default_skill_id
}
