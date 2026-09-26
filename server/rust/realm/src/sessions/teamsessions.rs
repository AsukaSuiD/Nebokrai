//! Индекс маршрутов команд мира `team_id → session_id` (исходный
//! `CGame::m_mapTeamSession`) — primary state владельца `sessions` (прежнее
//! transitional-поле `CGame`).
//!
//! `CGame` хранит только composition handle `team_sessions` и делегирует
//! прежний pub facade построчно; release-clear остаётся оркестрацией app
//! через typed-операцию владельца. Читатели (team/log/server-message, MainLoop,
//! process-отчёт) идут через прежние facade-методы игры.

use std::collections::BTreeMap;

/// Действующий индекс маршрутов команд мира.
///
/// Constructor-ное состояние — пустая карта; отсутствующий маршрут читается
/// как session ID `0` (legacy-семантика `find == end`).
pub struct WorldTeamSessionIndex {
    sessions: BTreeMap<u32, i32>,
}

impl WorldTeamSessionIndex {
    pub fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
        }
    }

    /// Session ID команды; отсутствующий маршрут даёт `0`.
    pub fn get_team_session_id(&self, team_id: u32) -> i32 {
        self.sessions.get(&team_id).copied().unwrap_or(0)
    }

    pub fn team_session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn publish_team_session(&mut self, team_id: u32, session_id: i32) {
        self.sessions.insert(team_id, session_id);
    }

    pub fn remove_team_session(&mut self, team_id: u32) {
        self.sessions.remove(&team_id);
    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }
}
