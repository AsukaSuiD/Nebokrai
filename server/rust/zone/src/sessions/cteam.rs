//! Сеанс команды `CTeam` GameServer в Zone `sessions/`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец —
//! `appserver/session/cteam.cpp`. Материализованы локальные создание и
//! вступление, идентичность команды и главы, общее распределение по умолчанию,
//! точная сериализация сеанса и участников, а также локальные выход, смена
//! главы, исключение, роспуск, распределение, чат и восстановление удалённого
//! снимка с публикациями World и клиенту. Минутный `AI`-контроль удаляет из
//! локального registry проекцию, когда на GameServer не осталось ни одного её
//! игрока. Обход заданий и сценариев команды выполняет `CGame`, потому что он
//! владеет картой локальных игроков, очередью сценариев и маршрутом World.
//! Остальные переходы состояния остаются RAW. Статические
//! `QuestTeamData/CompleteTeamData` заменены принадлежащей
//! `CGame` очередью повторов: вход задаёт ID команды, стадия сеанса отправляет
//! `0x60008`, а успешный `0x7FD08` снимает запрос.
//! GetTeamatesAmount (0x00507590) обслуживает CSessionFactory: каждый ID
//! списка учитывается при успешном QueryPlugByID, без ended/owner-фильтра.

use super::cplug::CPlug;
use super::csession::CSession;
use super::cteamate::CTeamate;
use nebokrai_shared::protocol::LegacyWriter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTeam {
    team_id: u32,
    leader_id: i32,
    allocation_scheme: i32,
    last_checked_time_stamp: u32,
    team_name: Vec<u8>,
    password: Vec<u8>,
}

impl CTeam {
    pub const fn new(team_id: u32) -> Self {
        Self {
            team_id,
            leader_id: 0,
            allocation_scheme: 1,
            last_checked_time_stamp: 0,
            team_name: Vec::new(),
            password: Vec::new(),
        }
    }

    pub fn restored(team_id: u32, team_name: Vec<u8>, password: Vec<u8>, leader_id: i32) -> Self {
        Self {
            team_id,
            leader_id,
            allocation_scheme: 1,
            last_checked_time_stamp: 0,
            team_name,
            password,
        }
    }

    pub const fn team_id(&self) -> u32 {
        self.team_id
    }

    pub const fn leader_id(&self) -> i32 {
        self.leader_id
    }

    pub const fn set_leader(&mut self, leader_id: i32) {
        self.leader_id = leader_id;
    }

    pub fn serialize<'a>(
        &self,
        session: &CSession,
        now_ms: u32,
        teammates: impl IntoIterator<Item = (&'a CPlug, &'a CTeamate)>,
    ) -> Vec<u8> {
        let teammates: Vec<(&CPlug, &CTeamate)> = teammates.into_iter().collect();
        let mut output = Vec::new();
        let mut writer = LegacyWriter::new(&mut output);
        writer.write_i32(1);
        writer.write_u32(session.minimum_plugs());
        writer.write_u32(session.maximum_plugs());
        writer.write_u32(session.remaining_lifetime(now_ms));
        writer.write_u32(self.team_id);
        writer.write_c_string(&self.team_name);
        writer.write_c_string(&self.password);
        writer.write_i32(self.leader_id);
        writer.write_u32(u32::try_from(teammates.len()).unwrap_or(u32::MAX));
        drop(writer);
        for (plug, teammate) in teammates {
            teammate.serialize(&mut output, plug.ended_state());
        }
        output
    }

    pub const fn allocation_scheme(&self) -> i32 {
        self.allocation_scheme
    }

    pub const fn set_allocation_scheme(&mut self, allocation_scheme: i32) {
        self.allocation_scheme = allocation_scheme;
    }

    /// Exact `AI`: первый вызов только запоминает timestamp. После наступления
    /// минутной границы timestamp намеренно не обновляется, поэтому live team
    /// проверяется на каждом следующем проходе session loop.
    pub fn idle_check_due(&mut self, now_ms: u32) -> bool {
        if self.last_checked_time_stamp == 0 {
            self.last_checked_time_stamp = now_ms;
            return false;
        }
        self.last_checked_time_stamp.wrapping_add(60_000) <= now_ms
    }
}
