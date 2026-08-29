//! Достигнутая часть ИИ демона-босса `CBossFiend`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/bossfiend.cpp` подтверждают восемь
//! одноразовых HP-порогов призыва и повторный призыв ниже 8% HP по строгой
//! проверке исходного таймера. Этот владелец принимает один RNG-бросок runtime
//! до порогового выбора, отдельно читает время для проверки и фиксации призыва
//! и накапливает `odds` через исключённые ID `1`, `2` и
//! `0x1f9`. Выполнение выбранного навыка остаётся у skill-owner-а.
//!
//! `OnSearchEnemy` подключён к реальному ходу монстра и сохраняет зависимость
//! выбора от минимальной дистанции текущего навыка. `OnIdle` подключён
//! целиком: общий владелец бездействия выполняет исходный случайный шаг либо ожидание
//! перед поиском, а отсутствие игроков переводит владельца в сон.
//! `OnSchedule` достигнут через общий боевой caller без обычного
//! attack-speed gate. `OnMoving` замкнут общим владельцем бездействия: поиск
//! начинается после завершения поставленного перед ним шага. Не достигнутые
//! части `OnSchedule` ниже остаются RAW.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp

// ============================================================================
// FUNCTION: CBossFiend::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_monster_base_attack` и
// `bossidle::schedule_attack_interval` сохраняют достигнутые target/current
// skill, `Tracing`, `CheckCast` и отсутствие обычного attack-speed gate.
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

// COMPONENT_VARIANT_END: GameServer

use super::guardtarget::{GuardDistanceTarget, consider_guard_distance_target};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::{MonsterProperties, MonsterSkill};

const BOSS_FIEND_SUMMON_SKILL_ID: u16 = 0x1f9;
const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Выполняет подтверждённый `OnSearchEnemy` демона-босса. Один выбор проходит
/// игроков, затем питомцев и сохраняет особое предпочтение целей не ближе
/// минимальной дистанции текущего навыка.
pub(crate) fn select_boss_fiend_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity> {
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_guard_distance_target(
            selected,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        selected = consider_guard_distance_target(
            selected,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    selected.map(|selected| selected.identity)
}

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

/// Выполняет полную AI-specific фиксацию выбранного навыка демона-босса.
/// Проверка таймера и запись момента призыва используют два отдельных чтения
/// runtime-часов в исходных местах.
pub(crate) fn choose_boss_fiend_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    hit_points: u32,
    roll: i32,
    runtime: &mut Runtime,
) -> Option<u16> {
    let below_repeat_threshold = hit_points as f32 / (property.maximum_hp as f32) < 0.08;
    let persist_modifier = below_repeat_threshold
        .then(|| {
            game.skill_base_properties(BOSS_FIEND_SUMMON_SKILL_ID.into(), 1)
                .map(|properties| properties.query_property(10_003))
        })
        .flatten();
    let timer_check_ms = (below_repeat_threshold && persist_modifier.is_some())
        .then(|| runtime.now_milliseconds());
    let selection = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.boss_fiend_ai())
        .and_then(|state| {
            select_boss_fiend_attack_skill(
                state,
                hit_points,
                property.maximum_hp,
                &property.skills,
                roll,
                timer_check_ms,
                persist_modifier,
            )
        })?;
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .set_current_skill_id(Some(u32::from(selection.skill_id)));
    }
    if selection.records_summon {
        let recorded_at_ms = runtime.now_milliseconds();
        if let Some(state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| monster.boss_fiend_ai_mut())
        {
            state.record_summon(selection.summon_threshold, recorded_at_ms);
        }
    }
    Some(selection.skill_id)
}
