//! Командный разъём `CTeamate`, принадлежащий игроку в GameServer, перенесённый в Zone `sessions/`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/session/cteamate.cpp`. Материализована достигнутая часть
//! приглашения и входа: идентификатор разъёма, владелец-игрок, снимок региона
//! и имени, а также `Serialize`, который `OnPlugInserted` вкладывает в
//! клиентское сообщение `0xBFD03`. Base ended-флаг передаётся owner-ом
//! реестра, чтобы wire не расходился с `CPlug::Serialize`. Локальный выход
//! доведён до членства игрока
//! и сообщения `0xBFD05`. Достигнутые обработчики распределения и чата
//! создают `0xBFD08/09` из типизированных владельцев сессии. Регион, состояние
//! участника и удалённое восстановление используют тот же типизированный
//! разъём. `IsPlugAvailable` сохраняет точное пятиминутное окно remote-owner-а
//! и reconnect-сигнал для повторной публикации team snapshot; остальные
//! недостигнутые ветви сохранены только в локальном исследовательском корпусе.

use nebokrai_shared::protocol::LegacyWriter;

const PLAYER_LOSE_TIMEOUT_MS: u32 = 300_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TeamMateAvailability {
    Available,
    Recovered,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTeamate {
    plug_id: i32,
    owner_type: i32,
    owner_id: i32,
    owner_region_id: i32,
    owner_name: Vec<u8>,
    player_lose_time_stamp: u32,
}

impl CTeamate {
    pub fn new(plug_id: i32, owner_id: i32, owner_region_id: i32, owner_name: &[u8]) -> Self {
        Self::new_owned(plug_id, 400, owner_id, owner_region_id, owner_name)
    }

    pub fn new_owned(
        plug_id: i32,
        owner_type: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Self {
        Self {
            plug_id,
            owner_type,
            owner_id,
            owner_region_id,
            owner_name: owner_name
                .split(|byte| *byte == 0)
                .next()
                .unwrap_or_default()
                .to_vec(),
            player_lose_time_stamp: 0,
        }
    }

    pub const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub const fn owner_region_id(&self) -> i32 {
        self.owner_region_id
    }

    pub const fn set_owner_region_id(&mut self, owner_region_id: i32) {
        self.owner_region_id = owner_region_id;
    }

    /// Exact `IsPlugAvailable` reconnect window. Отсутствующий локальный
    /// player запускает пятиминутный срок только если его сохранённый регион
    /// принадлежит этому GameServer; появление owner-а очищает срок и требует
    /// повторной публикации полного team snapshot.
    pub const fn availability(
        &mut self,
        now_ms: u32,
        owner_is_local: bool,
        owner_region_is_local: bool,
    ) -> TeamMateAvailability {
        if owner_is_local {
            if self.player_lose_time_stamp != 0 {
                self.player_lose_time_stamp = 0;
                return TeamMateAvailability::Recovered;
            }
            return TeamMateAvailability::Available;
        }
        if self.player_lose_time_stamp == 0 {
            if owner_region_is_local {
                self.player_lose_time_stamp = now_ms;
            }
            return TeamMateAvailability::Available;
        }
        if self
            .player_lose_time_stamp
            .wrapping_add(PLAYER_LOSE_TIMEOUT_MS)
            <= now_ms
        {
            TeamMateAvailability::Expired
        } else {
            TeamMateAvailability::Available
        }
    }

    pub fn serialize(&self, output: &mut Vec<u8>, plug_ended: i32) {
        let mut writer = LegacyWriter::new(output);
        writer.write_i32(5);
        writer.write_i32(self.owner_type);
        writer.write_i32(self.owner_id);
        writer.write_i32(plug_ended);
        writer.write_i32(self.owner_region_id);
        writer.write_c_string(&self.owner_name);
    }
}
