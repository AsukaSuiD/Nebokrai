//! Флаг доступности и время задания из `CPlayer` Game
//! (`server/gameserver/appserver/player.cpp`).
//! Точная пара `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`
//! (идентификаторы — docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки).
//! `QuestTimeBegin`, `QuestTimeClear` и `SetQuestOn`
//! записывают поля игрока до создания клиентского сообщения.
//! Game script GetQuestTime проверяет нули и обнуляет отрицательный
//! остаток; client 0x8FA12 отправляет сырую 32-битную разность.
//! Доказательства:
//! docs/reconstruction/gameserver-npc-and-regions.md#квестовые-записи-и-wire-снимки-cplayer

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

    /// Сценарный GetQuestTime обнуляет отсутствующий и отрицательный остаток.
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

    /// Клиентский запрос передаёт разность, включая отрицательное значение.
    pub const fn client_remaining(self, now_seconds: i32) -> i32 {
        self.time_limit
            .wrapping_sub(now_seconds)
            .wrapping_add(self.time_begin)
    }
}
