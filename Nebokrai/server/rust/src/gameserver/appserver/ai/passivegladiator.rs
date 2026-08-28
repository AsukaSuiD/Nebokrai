//! Пассивный гладиатор `CPassiveGladiator`, тип ИИ `1`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает упорядоченный список не более десяти уникальных ID нападавших
//! игроков. Недоступные записи удаляются при поиске, ближайшая живая цель
//! выбирается строго внутри `GetChaseRange`, равенство сохраняет более ранний
//! ID. Прирученное существо или повозка назначается сразу только вне боя.
//! `IndexSet` заменяет исходный `vector`, линейное подавление дубликатов и
//! удаление первого элемента, не меняя наблюдаемый порядок.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\passivegladiator.cpp
// COMPONENT_VARIANT_END: GameServer

use indexmap::IndexSet;

use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};

const MAXIMUM_ENEMY_PLAYERS: usize = 10;

/// Каноническое состояние `CPassiveGladiator::m_vEnemy`. `IndexSet` заменяет
/// исходные линейный поиск дубликата и `vector::erase(begin)`, сохраняя порядок.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PassiveGladiatorState {
    enemy_player_ids: IndexSet<i32>,
}

impl PassiveGladiatorState {
    pub(crate) fn record_player_attack(&mut self, player_id: i32) {
        if self.enemy_player_ids.insert(player_id)
            && self.enemy_player_ids.len() > MAXIMUM_ENEMY_PLAYERS
        {
            let _ = self.enemy_player_ids.shift_remove_index(0);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.enemy_player_ids.clear();
    }

    /// `WhenBeenHurted` запоминает игрока, а прирученное существо или повозку
    /// назначает непосредственно только пока гладиатор ещё не сражается.
    pub(crate) fn on_hurt(
        &mut self,
        attacker: ShapeIdentity,
        already_fighting: bool,
        attacker_is_owned_creature: bool,
    ) -> Option<ShapeIdentity> {
        match attacker.object_type {
            400 => {
                self.record_player_attack(attacker.id);
                None
            }
            600 if !already_fighting && attacker_is_owned_creature => Some(attacker),
            _ => None,
        }
    }

    /// Удаляет недоступные записи на месте и выбирает ближайшего живого игрока
    /// строго внутри `GetChaseRange`; равная дистанция сохраняет более ранний ID.
    pub(crate) fn select_target(
        &mut self,
        owner: ShapeView,
        chase_range: i32,
        mut resolve: impl FnMut(i32) -> Option<ShapeView>,
    ) -> Option<ShapeIdentity> {
        let mut selected = None;
        let mut selected_distance = i32::MAX;
        self.enemy_player_ids.retain(|player_id| {
            let Some(candidate) = resolve(*player_id) else {
                return false;
            };
            let distance = crate::gameserver::appserver::skills::baseattack::real_distance(
                owner.tile_x,
                owner.tile_y,
                candidate.tile_x,
                candidate.tile_y,
            );
            if chase_range < distance {
                return false;
            }
            if distance < selected_distance {
                selected = Some(candidate.identity);
                selected_distance = distance;
            }
            true
        });
        selected
    }
}
