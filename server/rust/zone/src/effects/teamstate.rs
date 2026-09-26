//! Данные и запись `CTeamState` (ID `0x186A6`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/teamstate.cpp/.h.
//! Конструктор сохраняет C-prefix имени и пароля без часов;
//! stamp проверки нулевой до Begin. Serialize/Unserialize
//! пишут ID и две C-строки без часов; bounded decode допускает
//! до 255 байт на строку. GetAdditionalData ставит бит 16
//! по ненулевой длине строки пароля и берёт живое число разрешённых team
//! plugs либо 1. AI проверяет unsigned `last+5000 <= now`
//! и записывает stamp; без цели/Player — End. Живые Begin/restart/AI/End
//! и пакеты остаются у переходного Game.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const TEAM_STATE_ID: i32 = 0x0001_86a6;
pub const TEAM_STATE_STRING_CAPACITY: usize = 256;
pub const TEAM_STATE_CHECK_INTERVAL_MS: u32 = 5_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTeamState {
    team_name: Vec<u8>,
    team_password: Vec<u8>,
    last_check_timestamp_ms: u32,
}

impl CTeamState {
    pub fn new(mut team_name: Vec<u8>, mut team_password: Vec<u8>) -> Self {
        if let Some(end) = team_name.iter().position(|byte| *byte == 0) {
            team_name.truncate(end);
        }
        if let Some(end) = team_password.iter().position(|byte| *byte == 0) {
            team_password.truncate(end);
        }
        Self {
            team_name,
            team_password,
            last_check_timestamp_ms: 0,
        }
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        let team_name = reader.read_c_string(TEAM_STATE_STRING_CAPACITY)?.to_vec();
        let team_password = reader.read_c_string(TEAM_STATE_STRING_CAPACITY)?.to_vec();
        Ok(Self::new(team_name, team_password))
    }

    pub fn serialized_size(payload: &[u8], offset: usize) -> Option<usize> {
        let mut reader = LegacyReader::at(payload, offset).ok()?;
        let _state_id = reader.read_i32().ok()?;
        let _team_name = reader.read_c_string(TEAM_STATE_STRING_CAPACITY).ok()?;
        let _team_password = reader.read_c_string(TEAM_STATE_STRING_CAPACITY).ok()?;
        reader.position().checked_sub(offset)
    }

    pub fn encoded_for_install(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6 + self.team_name.len() + self.team_password.len());
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(TEAM_STATE_ID);
        writer.write_c_string(&self.team_name);
        writer.write_c_string(&self.team_password);
        bytes
    }

    pub const fn state_id(&self) -> i32 {
        TEAM_STATE_ID
    }

    /// Базовый `CState::GetClientStateTime` для этого бессрочного state.
    pub const fn client_state_time(&self) -> i32 {
        0
    }

    /// Бит 16 зависит от ненулевой длины строки пароля.
    pub fn additional_data(&self, teammates: usize) -> u32 {
        (u32::from(!self.team_password.is_empty()) << 16) | teammates as u32
    }

    pub fn team_name(&self) -> &[u8] {
        &self.team_name
    }

    pub fn team_password(&self) -> &[u8] {
        &self.team_password
    }

    pub const fn check_due(&self, sampled_at_ms: u32) -> bool {
        self.last_check_timestamp_ms
            .wrapping_add(TEAM_STATE_CHECK_INTERVAL_MS)
            <= sampled_at_ms
    }

    pub const fn record_check(&mut self, sampled_at_ms: u32) {
        self.last_check_timestamp_ms = sampled_at_ms;
    }

    /// Restart обнуляет stamp без часов.
    pub const fn reset_check(&mut self) {
        self.last_check_timestamp_ms = 0;
    }

    /// Конец наборного состояния: команда есть, а текущий лидер — не игрок.
    pub const fn ends_for_team(player_id: i32, team_id: i32, team_leader_id: Option<i32>) -> bool {
        team_id != 0 && matches!(team_leader_id, Some(leader_id) if leader_id != player_id)
    }
}
