//! Мировые сессии взаимодействия Realm: plug/session/team/teamate и фабрика.

pub mod cplug; // CPlug: базовый plug мировой сессии.
pub mod csession; // CSession: базовая session мира.
pub mod csessionfactory; // CSessionFactory: фабрика сессий.
pub mod cteam; // CTeam: командная session мира.
pub mod cteamate; // CTeamate: участник мировой команды.
pub mod teamsessions; // индекс маршрутов команд мира team_id → session_id (typed-операции, missing→0).
