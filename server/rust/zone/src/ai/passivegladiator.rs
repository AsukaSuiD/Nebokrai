//! Пассивный гладиатор `CPassiveGladiator`, тип ИИ `1`: упорядоченный список не
//! более десяти уникальных ID нападавших игроков и ближайший выбор внутри
//! `GetChaseRange`. Исходный владелец PDB:
//! `appserver/ai/passivegladiator.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`.
//!
//! `IndexSet` заменяет исходный vector, линейное подавление дубликатов и
//! вытеснение первого элемента `memmove` при счёте `> 10`, не меняя наблюдаемый
//! порядок: новичок всегда в конец, дубликат сохраняет позицию; равенство
//! дистанций держит более раннюю запись. Хранилище принадлежит Zone; экземпляр
//! живёт на переходном `CMonster` старого пакета. Критерий спасения
//! vulnerable-товарищей по доле HP (`< 0.4`) относится к SmartGladiator и в этом
//! типе не встречается.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use indexmap::IndexSet;

use crate::regions::shape::ShapeView;
use crate::regions::ShapeIdentity;

const MAXIMUM_ENEMY_PLAYERS: usize = 10;

/// Замечание записи нападавшего игрока из порядка `WhenBeenHurted`: новичок
/// вставлен в конец, дубликат сохраняет прежнюю позицию, а при переполнении
/// за десять врагов вытесняется самая ранняя запись.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassiveGladiatorEnemyNotice {
    Inserted,
    InsertedEvictingOldest,
    DuplicateKept,
}

/// Факты одного события урона по гладиатору (аргументы внешнего caller-а).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveGladiatorAttackFacts {
    pub attacker: ShapeIdentity,
    pub already_fighting: bool,
    pub attacker_is_owned_creature: bool,
}

/// Разбор hurt-реакции: игрок записывается врагом, прирученное существо или
/// повозка назначается непосредственной целью только вне боя, прочие типы
/// игнорируются.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassiveGladiatorAttackOutcome {
    PlayerAttackRecorded(PassiveGladiatorEnemyNotice),
    AttackerAssigned(ShapeIdentity),
    Ignored,
}

/// Запись врага, успешно разрешённая и оценённая в текущем поиске: identity
/// живого игрока и его дистанция до гладиатора.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveGladiatorCandidate {
    pub identity: ShapeIdentity,
    pub distance: i32,
}

/// Итог поиска цели: ближайший из кандидатов в пределах `GetChaseRange`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveGladiatorSelection {
    pub target: ShapeIdentity,
    pub distance: i32,
}

/// Каноническое состояние `CPassiveGladiator::m_vEnemy`. `IndexSet` заменяет
/// исходные линейный поиск дубликата и `vector::erase(begin)`, сохраняя порядок.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PassiveGladiatorState {
    enemy_player_ids: IndexSet<i32>,
}

impl PassiveGladiatorState {
    pub fn has_enemy_players(&self) -> bool {
        !self.enemy_player_ids.is_empty()
    }

    /// Порядок вставки и вытеснения исходного вектора: линейный дубликат-фильтр
    /// заменён `insert`, а удаление первого элемента при счёте `> 0xa` —
    /// `shift_remove_index(0)`.
    pub fn record_player_attack(&mut self, player_id: i32) -> PassiveGladiatorEnemyNotice {
        if !self.enemy_player_ids.insert(player_id) {
            return PassiveGladiatorEnemyNotice::DuplicateKept;
        }
        if self.enemy_player_ids.len() > MAXIMUM_ENEMY_PLAYERS {
            let _ = self.enemy_player_ids.shift_remove_index(0);
            return PassiveGladiatorEnemyNotice::InsertedEvictingOldest;
        }
        PassiveGladiatorEnemyNotice::Inserted
    }

    pub fn clear(&mut self) {
        self.enemy_player_ids.clear();
    }

    /// Типизированный разбор `WhenBeenHurted`: запоминает игрока, а
    /// прирученное существо или повозку назначает непосредственно только пока
    /// гладиатор ещё не сражается.
    pub fn assess_attack(
        &mut self,
        facts: PassiveGladiatorAttackFacts,
    ) -> PassiveGladiatorAttackOutcome {
        match facts.attacker.object_type {
            400 => {
                let notice = self.record_player_attack(facts.attacker.id);
                PassiveGladiatorAttackOutcome::PlayerAttackRecorded(notice)
            }
            600 if !facts.already_fighting && facts.attacker_is_owned_creature =>
                PassiveGladiatorAttackOutcome::AttackerAssigned(facts.attacker),
            _ => PassiveGladiatorAttackOutcome::Ignored,
        }
    }

    /// Совместимая hub-форма hurt-реакции: возвращает только назначенную
    /// непосредственную цель, запись игрока остаётся внутри состояния.
    pub fn on_hurt(
        &mut self,
        attacker: ShapeIdentity,
        already_fighting: bool,
        attacker_is_owned_creature: bool,
    ) -> Option<ShapeIdentity> {
        match self.assess_attack(PassiveGladiatorAttackFacts {
            attacker,
            already_fighting,
            attacker_is_owned_creature,
        }) {
            PassiveGladiatorAttackOutcome::AttackerAssigned(target) => Some(target),
            _ => None,
        }
    }

    /// Удаляет недоступные записи на месте и выбирает ближайшего живого игрока
    /// строго внутри `GetChaseRange`; равная дистанция сохраняет более ранний ID.
    pub fn select_candidate(
        &mut self,
        owner: ShapeView,
        chase_range: i32,
        mut resolve: impl FnMut(i32) -> Option<ShapeView>,
    ) -> Option<PassiveGladiatorSelection> {
        let mut selected: Option<PassiveGladiatorCandidate> = None;
        let mut selected_distance = i32::MAX;
        self.enemy_player_ids.retain(|player_id| {
            let candidate = resolve(*player_id).map(|view| PassiveGladiatorCandidate {
                identity: view.identity,
                distance: owner.real_distance(Some(view)),
            });
            let Some(candidate) = candidate else {
                return false;
            };
            if chase_range < candidate.distance {
                return false;
            }
            if candidate.distance < selected_distance {
                selected = Some(candidate);
                selected_distance = candidate.distance;
            }
            true
        });
        selected.map(|candidate| PassiveGladiatorSelection {
            target: candidate.identity,
            distance: candidate.distance,
        })
    }

    /// Совместимая hub-форма поиска цели: возвращает только назначаемый
    /// identity, дистанция остаётся внутри typed-итога.
    pub fn select_target(
        &mut self,
        owner: ShapeView,
        chase_range: i32,
        resolve: impl FnMut(i32) -> Option<ShapeView>,
    ) -> Option<ShapeIdentity> {
        self.select_candidate(owner, chase_range, resolve)
            .map(|selection| selection.target)
    }
}
