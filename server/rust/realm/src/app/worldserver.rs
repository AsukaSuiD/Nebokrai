//! Технические функции process-owner-а исторического WorldServer, перенесённые в Realm `app/`.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`. Файл хранит
//! operator-log адаптеры, имя/состояние процесса и узкие lifecycle helpers,
//! используемые `CGame`; доменный `Init/MainLoop/Release` остаётся в `game.rs`.
//!
//! Windows MFC/console side effects заменены структурированными результатами и
//! stderr process-оболочки. Byte-exact format keys, порядок публикации и
//! различие штатной ошибки, retained owner и безопасной остановки сохраняются.
//! Rust не вводит второй singleton либо дополнительный process lifecycle.

use std::sync::Arc;

use parking_lot::Mutex;

const LEGACY_LOG_BUFFER_CAPACITY: usize = 64_000;
const LEGACY_WINDOW_TEXT_CAPACITY: usize = 64_000;
const SAVE_LOG_HEADER: &[u8] =
    b"\r\n=============================== Start Save Log ===============================\r\n";
const SAVE_LOG_FOOTER: &[u8] =
    b"================================ End Save Log ================================\r\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLogLocalTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveLogTextDisposition {
    Retained,
    Flushed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddLogTextBlock {
    MissingNoArgumentValue { percent_offset: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldLogLine {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddLogTextDisposition {
    Written {
        rotation: SaveLogTextDisposition,
        line: WorldLogLine,
    },
    BlockedMissingFact {
        rotation: SaveLogTextDisposition,
        local_time: WorldLogLocalTime,
        block: AddLogTextBlock,
    },
}

#[derive(Clone, Default)]
pub struct WorldLogTextOwner {
    state: Arc<Mutex<WorldLogTextState>>,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct WorldLogTextState {
    initialized: bool,
    last_save_tick_ms: u32,
    info_text: Vec<u8>,
    log_text: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRefreshInfoCurrent {
    pub connections: i32,
    pub map_players: i32,
    pub online_players: u32,
    pub offline_players: u32,
    pub login_players: u32,
    pub creation_players: u32,
    pub deletion_players: u32,
    pub restore_players: u32,
    pub saving_players: i32,
    pub team_sessions: i32,
    pub largess_entries: u32,
    pub write_log_queue: u32,
    pub player_load_queue: u32,
    pub reback_messages: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldRefreshInfoHighWater {
    pub connections: i32,
    pub map_players: i32,
    pub online_players: u32,
    pub offline_players: u32,
    pub login_players: u32,
    pub creation_players: u32,
    pub deletion_players: u32,
    pub restore_players: u32,
    pub saving_players: i32,
    pub team_sessions: i32,
    pub largess_entries: u32,
    pub write_log_queue: u32,
    pub player_load_queue: u32,
    pub reback_messages: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRefreshSaveState {
    pub last_save_time: WorldLogLocalTime,
    pub save_point_time_ms: u32,
    pub last_save_tick_ms: u32,
    pub this_save_start_tick_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRefreshSaveStatus {
    Normal,
    Abnormal,
}

impl WorldRefreshSaveStatus {
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Normal => b"(Normal)",
            Self::Abnormal => b"(Abnormal!!!!)",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRefreshInfoReport {
    pub current: WorldRefreshInfoCurrent,
    pub high_water: WorldRefreshInfoHighWater,
    pub save_status: WorldRefreshSaveStatus,
    pub this_save_seconds: u32,
    pub text: Vec<u8>,
}

pub fn refresh_info_text<GetTick>(
    owner: &mut WorldLogTextOwner,
    current: WorldRefreshInfoCurrent,
    high_water: &mut WorldRefreshInfoHighWater,
    save: WorldRefreshSaveState,
    mut get_tick: GetTick,
) -> WorldRefreshInfoReport
where
    GetTick: FnMut() -> u32,
{
    high_water.connections = high_water.connections.max(current.connections);
    high_water.map_players = high_water.map_players.max(current.map_players);
    update_signed_u32_max(&mut high_water.online_players, current.online_players);
    update_signed_u32_max(&mut high_water.offline_players, current.offline_players);
    update_signed_u32_max(&mut high_water.login_players, current.login_players);
    update_signed_u32_max(&mut high_water.creation_players, current.creation_players);
    update_signed_u32_max(&mut high_water.deletion_players, current.deletion_players);
    update_signed_u32_max(&mut high_water.restore_players, current.restore_players);
    high_water.saving_players = high_water.saving_players.max(current.saving_players);
    high_water.team_sessions = high_water.team_sessions.max(current.team_sessions);
    update_signed_u32_max(&mut high_water.largess_entries, current.largess_entries);
    update_signed_u32_max(&mut high_water.write_log_queue, current.write_log_queue);
    update_signed_u32_max(&mut high_water.player_load_queue, current.player_load_queue);
    high_water.reback_messages = high_water.reback_messages.max(current.reback_messages);

    let status_tick_ms = get_tick();
    let save_status =
        if save.save_point_time_ms < status_tick_ms.wrapping_sub(save.last_save_tick_ms) {
            WorldRefreshSaveStatus::Abnormal
        } else {
            WorldRefreshSaveStatus::Normal
        };
    let this_save_seconds = if save.this_save_start_tick_ms == 0 {
        0
    } else {
        get_tick()
            .wrapping_sub(save.this_save_start_tick_ms)
            .wrapping_div(1_000)
    };

    let text = format!(
        "Last Save Time : {:04}-{:02}-{:02} {:02}:{:02}:{:02} {}\r\nThis Save Time : {} sec  Saving = {}/{}\r\nConnects = {}/{}  Map = {}/{}  Online = {}/{}  Offline = {}/{}  Login = {}/{}  Create = {}/{}  Delete = {}/{}  Resume = {}/{}\r\nTeams = {}/{}  Largess = {}/{}  WriteLog = {}/{}  LoadPlayer = {}/{}  ReMsg = {}/{}\r\n",
        save.last_save_time.year,
        save.last_save_time.month,
        save.last_save_time.day,
        save.last_save_time.hour,
        save.last_save_time.minute,
        save.last_save_time.second,
        String::from_utf8_lossy(save_status.as_bytes()),
        this_save_seconds as i32,
        current.saving_players,
        high_water.saving_players,
        current.connections,
        high_water.connections,
        current.map_players,
        high_water.map_players,
        current.online_players as i32,
        high_water.online_players as i32,
        current.offline_players as i32,
        high_water.offline_players as i32,
        current.login_players as i32,
        high_water.login_players as i32,
        current.creation_players as i32,
        high_water.creation_players as i32,
        current.deletion_players as i32,
        high_water.deletion_players as i32,
        current.restore_players as i32,
        high_water.restore_players as i32,
        current.team_sessions,
        high_water.team_sessions,
        current.largess_entries as i32,
        high_water.largess_entries as i32,
        current.write_log_queue as i32,
        high_water.write_log_queue as i32,
        current.player_load_queue as i32,
        high_water.player_load_queue as i32,
        current.reback_messages,
        high_water.reback_messages,
    )
    .into_bytes();
    owner.set_info_text(&text);
    WorldRefreshInfoReport {
        current,
        high_water: *high_water,
        save_status,
        this_save_seconds,
        text,
    }
}

fn update_signed_u32_max(high_water: &mut u32, current: u32) {
    if (*high_water as i32) < current as i32 {
        *high_water = current;
    }
}

impl WorldLogTextOwner {
    pub fn set_info_text(&self, text: &[u8]) {
        let mut state = self.state.lock();
        state.info_text.clear();
        state.info_text
            .extend_from_slice(legacy_c_string_prefix(text));
    }

    pub fn log_text(&self) -> Vec<u8> {
        self.state.lock().log_text.clone()
    }

    pub fn save_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        force: bool,
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> SaveLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().save_log_text_with(
            force,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        message: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            message,
            false,
            false,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_error_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        message: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            message,
            false,
            true,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_log_text_no_arguments<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        format: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            format,
            true,
            false,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }
}

impl WorldLogTextState {
    fn add_log_text_with<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        message: &[u8],
        no_arguments_format: bool,
        error_marker: bool,
        save_info_time_ms: u32,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let rotation = self.save_log_text_with(
            false,
            save_info_time_ms,
            get_tick,
            get_local_time,
            put_log_info,
        );
        let local_time = get_local_time();
        let message = legacy_c_string_prefix(message);
        let message = if no_arguments_format {
            match format_without_arguments(message) {
                Ok(message) => message,
                Err(block) => {
                    return AddLogTextDisposition::BlockedMissingFact {
                        rotation,
                        local_time,
                        block,
                    };
                }
            }
        } else {
            message.to_vec()
        };
        let prefix = if error_marker {
            format!(
                "[{:02}-{:02} {:02}:{:02}:{:02}] <error> ",
                local_time.month,
                local_time.day,
                local_time.hour,
                local_time.minute,
                local_time.second,
            )
        } else {
            format!(
                "[{:02}-{:02} {:02}:{:02}:{:02}] ",
                local_time.month,
                local_time.day,
                local_time.hour,
                local_time.minute,
                local_time.second,
            )
        }
        .into_bytes();
        let required_bytes_with_nul = prefix.len() + message.len() + 2 + 1;
        let mut bytes = Vec::with_capacity(required_bytes_with_nul - 1);
        bytes.extend_from_slice(&prefix);
        bytes.extend_from_slice(&message);
        bytes.extend_from_slice(b"\r\n");
        put_log_info(&bytes);
        self.log_text.extend_from_slice(&bytes);

        AddLogTextDisposition::Written {
            rotation,
            line: WorldLogLine { bytes },
        }
    }

    fn save_log_text_with<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        force: bool,
        save_info_time_ms: u32,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> SaveLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        if !self.initialized {
            self.initialized = true;
            self.last_save_tick_ms = get_tick();
        }
        if !force {
            let now = get_tick();
            if now.wrapping_sub(self.last_save_tick_ms) <= save_info_time_ms
                && self.log_text.len() < LEGACY_LOG_BUFFER_CAPACITY
            {
                return SaveLogTextDisposition::Retained;
            }
        }

        self.last_save_tick_ms = get_tick();
        put_log_info(SAVE_LOG_HEADER);
        let local_time = get_local_time();
        let dated_separator = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}\r\n\r\n",
            local_time.year,
            local_time.month,
            local_time.day,
            local_time.hour,
            local_time.minute,
            local_time.second,
        );
        put_log_info(dated_separator.as_bytes());
        let info_text = legacy_c_string_prefix(&self.info_text);
        let copied_len = info_text
            .len()
            .min(LEGACY_WINDOW_TEXT_CAPACITY.saturating_sub(1));
        put_log_info(&info_text[..copied_len]);
        put_log_info(b"\r\n");
        self.log_text.clear();
        put_log_info(SAVE_LOG_FOOTER);
        SaveLogTextDisposition::Flushed
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn format_without_arguments(format: &[u8]) -> Result<Vec<u8>, AddLogTextBlock> {
    let mut output = Vec::with_capacity(format.len());
    let mut offset = 0;
    while offset < format.len() {
        if format[offset] != b'%' {
            output.push(format[offset]);
            offset += 1;
            continue;
        }
        if format.get(offset + 1) == Some(&b'%') {
            output.push(b'%');
            offset += 2;
            continue;
        }
        return Err(AddLogTextBlock::MissingNoArgumentValue {
            percent_offset: offset,
        });
    }
    Ok(output)
}
