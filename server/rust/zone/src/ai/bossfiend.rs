//! ИИ демона-босса `CBossFiend` (AI104): восемь одноразовых HP-порогов
//! призыва, строгий таймер повторного призыва ниже 8% HP, пороговый выбор
//! боевого навыка и enemy-проход с минимальной дистанцией текущего навыка.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossfiend.cpp`.
//! Машинная сверка по этой паре:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | ctor записывает `timeGetTime` и открывает все восемь HP-порогов ровно в момент создания AI-owner-а | VA `0x00609310` | [`BossFiendAiState::new`] | `MATCH` |
//! | `SelectAttackSkill`: один исходный RNG-бросок, восемь HP-порогов, повторный призыв ниже 8% HP по строгой проверке `last + persist < now`; накопление `odds` через исключённые ID `1`, `2` и `0x1f9`; исход без совпадения — возврат без назначения текущего навыка (в отличие от default у `CBossBlue`) | VA `0x00609670` | [`select_boss_fiend_attack_skill`], [`BossFiendSkillSelection`] | `MATCH` |
//! | полная фиксация выбранного навыка: проверка таймера и запись момента призыва используют два отдельных чтения часов в исходных местах | тот же VA | [`choose_boss_fiend_attack_skill`] | `MATCH` |
//! | `OnSearchEnemy`: один выбор проходит игроков, затем питомцев и сохраняет особое предпочтение целей не ближе минимальной дистанции текущего навыка | подтверждён прежней шапкой владельца | [`select_boss_fiend_enemy`] | `MATCH` |
//!
//! Остаются hub-владением: общий monster tick hub — `Run`, боевой caller
//! `OnSchedule` без обычного attack-speed gate, `OnIdle` со случайным шагом
//! либо ожиданием, `OnMoving` и усыпление при отсутствии игроков. FIFO caller
//! сохраняет target/current skill, `Tracing`, `CheckCast` и потерю цели.
//! Выполнение выбранного навыка (`bossfiendsummon` и реестр исполнителей)
//! остаётся у своих skill-owner-ов.
//!
//! Швы к hub-владельцам:
//!
//! - [`BossFiendDispatcherMonster`] — доступ к одноразовым порогам призыва и
//!   таймеру последнего принудительного призыва на hub-владельце `CMonster`.
//! - Часы проверки таймера и фиксации призыва читаются отдельными вызовами
//!   `now_milliseconds` (fn-параметр делегата старого main loop); точное
//!   значение равно `game_tick_milliseconds`
//!   (`GameClockContext::now_milliseconds`).
//! - Enemy-проход повторяет общий шов кандидатов
//!   [`super::lord::EnemySearchDispatcherRegion`]/
//!   [`super::lord::EnemySearchDispatcherPlayer`]; ядро отбора
//!   [`consider_distance_target`] ниже повторяет общее дистанционное ядро
//!   `ai/guardtarget.rs` (`consider_guard_distance_target`), но держит тело
//!   собственного RVA владыки демона.

use nebokrai_shared::resources::{MonsterProperties, MonsterSkill};

use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::shape::ShapeView;

use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherPlayer, MonsterDispatcherRegion,
};

const BOSS_FIEND_SUMMON_SKILL_ID: u16 = 0x1f9;
const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Запись-кандидат прохода цели с дистанцией до владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DistanceTarget {
    identity: ShapeIdentity,
    distance: i32,
}

/// Ядро отбора с минимальной дистанцией навыка: внутри отдельной категории
/// предпочитается ближайшая цель не ближе минимальной дистанции, а при
/// отсутствии такой цели остаётся последняя слишком близкая запись.
fn consider_distance_target(
    selected: Option<DistanceTarget>,
    candidate: DistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<DistanceTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    let Some(current) = selected else {
        return Some(candidate);
    };
    if current.distance <= candidate.distance {
        if current.distance < minimum_skill_distance {
            Some(candidate)
        } else {
            Some(current)
        }
    } else if candidate.distance < minimum_skill_distance {
        Some(current)
    } else {
        Some(candidate)
    }
}

/// Выполняет подтверждённый `OnSearchEnemy` демона-босса. Один выбор проходит
/// игроков, затем питомцев и сохраняет особое предпочтение целей не ближе
/// минимальной дистанции текущего навыка.
pub fn select_boss_fiend_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id()) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_distance_target(
            selected,
            DistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        selected = consider_distance_target(
            selected,
            DistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
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
pub struct BossFiendAiState {
    summon_used: [bool; 8],
    last_summon_ms: u32,
}

impl BossFiendAiState {
    /// Исходный конструктор (VA `0x00609310`) фиксирует `timeGetTime` и
    /// открывает все восемь HP-порогов ровно в момент создания AI-owner-а.
    pub const fn new(now_ms: u32) -> Self {
        Self {
            summon_used: [false; 8],
            last_summon_ms: now_ms,
        }
    }

    pub fn record_summon(&mut self, threshold: Option<usize>, now_ms: u32) {
        if let Some(threshold) = threshold {
            self.summon_used[threshold] = true;
        }
        self.last_summon_ms = now_ms;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BossFiendSkillSelection {
    pub skill_id: u16,
    pub summon_threshold: Option<usize>,
    pub records_summon: bool,
}

/// Сохраняет один исходный RNG-бросок, восемь HP-порогов, строгую проверку
/// низкоуровневого таймера и накопление `odds` через исключённые записи.
/// `None` соответствует исходному возврату без назначения текущего навыка.
pub fn select_boss_fiend_attack_skill(
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

/// Монстр демона-босса: переходный фасад прежнего `CMonster`, открывающий
/// состояние конкретного экземпляра ИИ.
pub trait BossFiendDispatcherMonster: MonsterDispatcherMonster {
    fn boss_fiend_ai(&self) -> Option<&BossFiendAiState>;

    fn boss_fiend_ai_mut(&mut self) -> Option<&mut BossFiendAiState>;
}

/// Выполняет полную AI-specific фиксацию выбранного навыка демона-босса.
/// Проверка таймера и запись момента призыва используют два отдельных чтения
/// часов в исходных местах.
pub fn choose_boss_fiend_attack_skill<Game, Region>(
    game: &Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    hit_points: u32,
    roll: i32,
    now_milliseconds: fn() -> u32,
) -> Option<u16>
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: BossFiendDispatcherMonster,
{
    let below_repeat_threshold = hit_points as f32 / (property.maximum_hp as f32) < 0.08;
    let persist_modifier = below_repeat_threshold
        .then(|| {
            game.skill_base_properties(BOSS_FIEND_SUMMON_SKILL_ID.into(), 1)
                .map(|properties| properties.query_property(10_003))
        })
        .flatten();
    let timer_check_ms = (below_repeat_threshold && persist_modifier.is_some())
        .then(now_milliseconds);
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
        let recorded_at_ms = now_milliseconds();
        if let Some(state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| monster.boss_fiend_ai_mut())
        {
            state.record_summon(selection.summon_threshold, recorded_at_ms);
        }
    }
    Some(selection.skill_id)
}
