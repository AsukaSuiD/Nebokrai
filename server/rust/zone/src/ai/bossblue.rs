//! ИИ синего босса `CBossBlue` (AI103): восемь одноразовых HP-порогов ярости,
//! пороговый выбор боевого навыка и общий enemy-проход ближайшего игрока или
//! питомца.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\bossblue.cpp`.
//! Свидетельства машинной базы зафиксированы разведкой линий D/E и прежней
//! шапкой `appserver/ai/bossblue.rs`; тела перенесены буквально:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | ctor: восемь одноразовых порогов ярости созданы сброшенными | VA `0x00609CB0` | [`BossBlueAiState`] (`Default`) | `MATCH` |
//! | `WakeUp` после общего восстановления HP открывает только ещё не пройденные пороги текущей фазы жизни | VA `0x00609CD0`; `CBossFiend::WakeUp` слинкован тем же RVA (ICF) | [`BossBlueAiState::wake`] | `MATCH` |
//! | `SelectAttackSkill`: один исходный RNG-бросок runtime, пороговая ярость по фазе, повтор ниже 8% HP при отсутствии записанного fury-состояния; накопление `odds` намеренно учитывает доли исключённых ID `2` и `0x1f7`; исход — default-навык (в отличие от fallthrough без назначения у `CBossFiend`) | VA `0x0060A0E0` | [`select_boss_blue_attack_skill`], [`choose_boss_blue_attack_skill`] | `MATCH` |
//! | `OnSearchEnemy`: общий проход игроков, затем питомцев с заменой цели при равной дистанции | подтверждён прежней шапкой владельца | [`select_boss_blue_enemy`] через [`super::lord::select_nearest_player_or_pet`] | `MATCH` |
//!
//! Граница порции E1 (не расхождения): общий monster tick hub — `Run`,
//! `OnSchedule` (VA `0x00609FE0`), `OnIdle`, `OnMoving`, а также решение сна
//! при отсутствии игроков (`Hibernate`, VA `0x006093B0`, слинковано ICF с
//! общими телами) — остаются hub-владением и этой волной не затрагиваются.
//! `CMonster` создаёт и сбрасывает пороги после общего пробуждения, а
//! применение выбранного навыка (`bossbluefury` и реестр исполнителей)
//! остаётся у своих skill-owner-ов.
//!
//! Объявленные швы (не расхождения):
//!
//! - [`BossBlueDispatcherMonster`]/[`BossBlueDispatcherMoveShape`] — доступ к
//!   состоянию восьми порогов на hub-владельце `CMonster` и признаку
//!   записанного fury-состояния формы; сами пороги и тело выбора перенесены.
//! - Enemy-проход повторяет общий шов [`super::lord::EnemySearchDispatcherRegion`]/
//!   [`super::lord::EnemySearchDispatcherPlayer`] и тот же nearest-проход
//!   [`super::lord::select_nearest_player_or_pet`], что и владыка: разведка
//!   линии E подтверждает поведенчески одинаковый проход (игроки перед
//!   питомцами, равенство заменяет запись), единый дом — у `CLord` этой
//!   порции до переноса общего `guardtarget` его волной.

use nebokrai_shared::resources::{MonsterProperties, MonsterSkill};

use crate::regions::ShapeIdentity;
use crate::regions::shape::ShapeView;

use super::lord::{
    EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion, select_nearest_player_or_pet,
};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherRegion,
};

const BOSS_BLUE_FURY_SKILL_ID: u16 = 0x1f7;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Выполняет подтверждённый `OnSearchEnemy` синего босса: ближайшая живая
/// цель выбирается общим проходом игроков, затем питомцев; равенство заменяет
/// предыдущую запись.
pub fn select_boss_blue_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    select_nearest_player_or_pet(game, region, owner, area_index, guard_range)
}

/// Подвижная форма синего босса: переходный фасад прежнего `CMoveShape`.
/// Хранилище записанных состояний остаётся hub-владением.
pub trait BossBlueDispatcherMoveShape: MonsterDispatcherMoveShape {
    /// Признак записанного `CBossBlueFuryState` в хранилище состояний формы.
    fn has_boss_blue_fury_state(&self) -> bool;
}

/// Монстр синего босса: переходный фасад прежнего `CMonster`, открывающий
/// состояние восьми порогов конкретного экземпляра ИИ.
pub trait BossBlueDispatcherMonster:
    MonsterDispatcherMonster<MoveShape: BossBlueDispatcherMoveShape>
{
    fn boss_blue_ai_mut(&mut self) -> &mut BossBlueAiState;
}

/// Восемь одноразовых порогов ярости принадлежат конкретному ИИ синего босса.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BossBlueAiState {
    fury_used: [bool; 8],
}

impl BossBlueAiState {
    /// `CBossBlue::WakeUp` вызывается после общего восстановления HP и открывает
    /// только ещё не пройденные пороги текущей фазы жизни.
    pub fn wake(&mut self, hit_points: u32, maximum_hit_points: u32) {
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
pub fn select_boss_blue_attack_skill(
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

/// Разрешает состояние конкретного синего босса и выполняет его пороговый
/// выбор после единственного RNG-броска, полученного вызывающим runtime.
pub fn choose_boss_blue_attack_skill<Region>(
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    hit_points: u32,
    roll: i32,
    default_skill_id: u16,
) -> Option<u16>
where
    Region: MonsterDispatcherRegion,
    Region::Monster: BossBlueDispatcherMonster,
{
    region.find_monster_by_id_mut(monster_id).map(|monster| {
        let has_fury_state = monster.move_shape().has_boss_blue_fury_state();
        select_boss_blue_attack_skill(
            monster.boss_blue_ai_mut(),
            hit_points,
            property.maximum_hp,
            has_fury_state,
            &property.skills,
            roll,
            default_skill_id,
        )
    })
}
