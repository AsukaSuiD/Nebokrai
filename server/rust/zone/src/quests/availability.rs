//! Флаг доступности и время задания из `CPlayer` Game
//! (`server/gameserver/appserver/player.cpp`).
//! GameServer/gameserver.exe SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E;
//! GameServer.pdb: RSDS 5bee6dd1-bf90-49b8-8be9-eb25c4038d53, age 2.
//! `QuestTimeBegin` VA 0x0042DA20–0x0042DAAF, `QuestTimeClear`
//! VA 0x0042DAC0–0x0042DB32, `SetQuestOn` VA 0x0042DB40–0x0042DBB4
//! записывают поля игрока до создания клиентского сообщения.

/// Три сохраняемых поля вложены в живого игрока, а Game выполняет доставку.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerQuestAvailability {
    time_begin: i32,
    time_limit: i32,
    enabled: bool,
}

impl PlayerQuestAvailability {
    pub const fn from_snapshot(time_begin: i32, time_limit: i32, enabled: bool) -> Self {
        Self {
            time_begin,
            time_limit,
            enabled,
        }
    }

    pub const fn time_begin(self) -> i32 {
        self.time_begin
    }

    pub const fn time_limit(self) -> i32 {
        self.time_limit
    }

    pub const fn enabled(self) -> bool {
        self.enabled
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn begin(&mut self, now_seconds: i32, time_limit: i32) {
        self.time_begin = now_seconds;
        self.time_limit = time_limit;
    }

    pub const fn clear_time(&mut self) {
        self.time_begin = 0;
        self.time_limit = 0;
    }

    /// Прежняя Rust-формула; её исходные ветви пока требуют прямой сверки.
    pub const fn remaining(self, now_seconds: i32) -> i32 {
        if self.time_begin == 0 || self.time_limit == 0 {
            return 0;
        }
        let remaining = self
            .time_begin
            .wrapping_add(self.time_limit)
            .wrapping_sub(now_seconds);
        if remaining < 0 { 0 } else { remaining }
    }
}
