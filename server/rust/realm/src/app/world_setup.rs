//! Позиционная конфигурация `tagSetup` WorldServer и её парсинг (hub-data
//! уровень). Источник контракта — та же точная пара, что у
//! [`crate::app::world_runtime`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`, RSDS совпадает).
//!
//! Две формы входа повторяют исходные прочтения: plain `setup.txt` идёт в
//! порядке MSVC `operator>>` поверх whitespace-токенов, encoded DAT держит
//! точный старый порядок без назначения `log_system_provider` и с повторным
//! чтением `name`; лексически неверный числовой/логический token останавливает
//! пару без выдуманной мутации destination. Расширение имени runtime-файла по
//! регистру повторяет поиск без изменения выбранного пути.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nebokrai_shared::resources::read_to_marker;

use crate::app::world_hub_data::WorldNetworkConfig;
use crate::app::world_runtime::WorldNetworkInitializationError;

#[derive(Clone, Debug)]
pub struct WorldSetup {
    pub world_number: Option<u32>,
    pub name: Vec<u8>,
    pub login_ip: Vec<u8>,
    pub login_port: Option<u32>,
    pub listen_port: Option<u32>,
    pub sql_connection_type: Vec<u8>,
    pub sql_server_ip: Vec<u8>,
    pub sql_user_name: Vec<u8>,
    pub sql_password: Vec<u8>,
    pub database_name: Vec<u8>,
    pub check_net: Option<bool>,
    pub maximum_byte_count: Option<u32>,
    pub maximum_message_length: Option<u32>,
    pub ban_ip_time_ms: Option<u32>,
    pub check_message_content: Option<bool>,
    pub maximum_connections: Option<i32>,
    pub maximum_io_sends: Option<i32>,
    pub maximum_client_send_buffer: Option<i32>,
    pub refresh_info_time_ms: u32,
    pub save_info_time_ms: u32,
    pub release_login_player_time_ms: Option<u32>,
    pub use_log_system: bool,
    pub log_system_provider: Vec<u8>,
    pub log_system_server: Vec<u8>,
    pub log_system_database: Vec<u8>,
    pub log_system_user: Vec<u8>,
    pub log_system_password: Vec<u8>,
    pub cost_database_provider: Vec<u8>,
    pub cost_database_ip: Vec<u8>,
    pub cost_database_name: Vec<u8>,
    pub cost_database_user: Vec<u8>,
    pub cost_database_password: Vec<u8>,
    pub load_largess_time_ms: Option<u32>,
    pub login_cost_database_provider: Vec<u8>,
    pub login_cost_database_ip: Vec<u8>,
    pub login_cost_database_name: Vec<u8>,
    pub login_cost_database_user: Vec<u8>,
    pub login_cost_database_password: Vec<u8>,
    pub player_load_thread_count: Option<u32>,
    pub language_package: Vec<u8>,
    pub use_old_save_largess_way: bool,
}

impl Default for WorldSetup {
    fn default() -> Self {
        Self {
            world_number: None,
            name: Vec::new(),
            login_ip: Vec::new(),
            login_port: None,
            listen_port: None,
            sql_connection_type: Vec::new(),
            sql_server_ip: Vec::new(),
            sql_user_name: Vec::new(),
            sql_password: Vec::new(),
            database_name: Vec::new(),
            check_net: None,
            maximum_byte_count: None,
            maximum_message_length: None,
            ban_ip_time_ms: None,
            check_message_content: None,
            maximum_connections: None,
            maximum_io_sends: None,
            maximum_client_send_buffer: None,
            refresh_info_time_ms: 1_000,
            save_info_time_ms: 60_000,
            release_login_player_time_ms: None,
            use_log_system: false,
            log_system_provider: Vec::new(),
            log_system_server: Vec::new(),
            log_system_database: Vec::new(),
            log_system_user: Vec::new(),
            log_system_password: Vec::new(),
            cost_database_provider: Vec::new(),
            cost_database_ip: Vec::new(),
            cost_database_name: Vec::new(),
            cost_database_user: Vec::new(),
            cost_database_password: Vec::new(),
            load_largess_time_ms: None,
            login_cost_database_provider: Vec::new(),
            login_cost_database_ip: Vec::new(),
            login_cost_database_name: Vec::new(),
            login_cost_database_user: Vec::new(),
            login_cost_database_password: Vec::new(),
            player_load_thread_count: None,
            language_package: Vec::new(),
            use_old_save_largess_way: true,
        }
    }
}

impl WorldSetup {
    #[allow(
        clippy::field_reassign_with_default,
        reason = "две стадии буквально сохраняют tagSetup::tagSetup и последующие записи CGame::CGame"
    )]
    pub fn for_game() -> Self {
        let mut setup = Self::default();
        setup.name = b"WorldServer".to_vec();
        setup.login_ip = b"127.0.0.1".to_vec();
        setup.login_port = Some(2_345);
        setup.listen_port = Some(8_100);
        setup
    }

    pub fn network_config_after_host(
        &self,
    ) -> Result<WorldNetworkConfig, WorldNetworkInitializationError> {
        Ok(WorldNetworkConfig {
            // Порядок чтения повторяет локальные значения.
            ban_ip_time_ms: self.ban_ip_time_ms.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwBanIPTime"),
            )?,
            maximum_client_send_buffer: self.maximum_client_send_buffer.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxClientSendBuf"),
            )?,
            maximum_message_length: self.maximum_message_length.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwMaxMsgLen"),
            )?,
            maximum_byte_count: self.maximum_byte_count.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwMaxByteNum"),
            )?,
            check_message_content: self.check_message_content.ok_or(
                WorldNetworkInitializationError::MissingSetupField("bCheckMsgCon"),
            )?,
            maximum_connections: self.maximum_connections.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxConnectNum"),
            )?,
            maximum_io_sends: self.maximum_io_sends.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxIOSendNum"),
            )?,
            check_net: self
                .check_net
                .ok_or(WorldNetworkInitializationError::MissingSetupField(
                    "bCheckNet",
                ))?,
        })
    }

    pub fn parse_plain(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.outcome();
                };
                let Some(value) = $parser(raw) else {
                    // Для лексически неверного числового или логического token
                    // не определена мутация destination старым MSVC iostream.
                    // Найденный setup содержит только корректные такие значения.
                    return tokens.outcome();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }
        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }
        macro_rules! read_number_with_default {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw));
            };
        }
        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_legacy_bool);
            };
        }
        macro_rules! read_optional_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }
        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_number!(world_number, u32);
        read_bytes!(name);
        read_bytes!(login_ip);
        read_number!(login_port, u32);
        read_number!(listen_port, u32);
        read_bytes!(sql_connection_type);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_optional_bool!(check_net);
        read_number!(maximum_byte_count, u32);
        read_number!(maximum_message_length, u32);
        read_number!(ban_ip_time_ms, u32);
        read_optional_bool!(check_message_content);
        read_number!(maximum_connections, i32);
        read_number!(maximum_io_sends, i32);
        read_number!(maximum_client_send_buffer, i32);
        read_number_with_default!(refresh_info_time_ms, u32);
        read_number_with_default!(save_info_time_ms, u32);
        read_number!(release_login_player_time_ms, u32);
        read_bool!(use_log_system);
        read_bytes!(log_system_provider);
        read_bytes!(log_system_server);
        read_bytes!(log_system_database);
        read_bytes!(log_system_user);
        read_bytes!(log_system_password);
        read_bytes!(cost_database_provider);
        read_bytes!(cost_database_ip);
        read_bytes!(cost_database_name);
        read_bytes!(cost_database_user);
        read_bytes!(cost_database_password);
        read_number!(load_largess_time_ms, u32);
        read_bytes!(login_cost_database_provider);
        read_bytes!(login_cost_database_ip);
        read_bytes!(login_cost_database_name);
        read_bytes!(login_cost_database_user);
        read_bytes!(login_cost_database_password);
        read_number!(player_load_thread_count, u32);
        read_bytes!(language_package);
        read_bool!(use_old_save_largess_way);

        tokens.outcome()
    }

    pub fn parse_encoded(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.outcome();
                };
                let Some(value) = $parser(raw) else {
                    // Некорректный числовой или логический token не встречается
                    // в найденном oracle; MSVC destination не угадываем.
                    return tokens.outcome();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }
        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }
        macro_rules! read_number_with_default {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw));
            };
        }
        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_legacy_bool);
            };
        }
        macro_rules! read_optional_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }
        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_number!(world_number, u32);
        read_bytes!(name);
        read_bytes!(login_ip);
        read_number!(login_port, u32);
        read_number!(listen_port, u32);
        read_bytes!(sql_connection_type);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_optional_bool!(check_net);
        read_number!(maximum_byte_count, u32);
        read_number!(maximum_message_length, u32);
        read_number!(ban_ip_time_ms, u32);
        read_optional_bool!(check_message_content);
        read_number!(maximum_connections, i32);
        read_number!(maximum_io_sends, i32);
        read_number!(maximum_client_send_buffer, i32);
        read_number_with_default!(refresh_info_time_ms, u32);
        read_number_with_default!(save_info_time_ms, u32);
        read_number!(release_login_player_time_ms, u32);
        read_bool!(use_log_system);

        // Точный старый DAT-порядок: provider не назначается.
        read_bytes!(log_system_server);
        read_bytes!(log_system_database);
        read_bytes!(log_system_user);
        read_bytes!(log_system_password);
        read_bytes!(cost_database_provider);
        read_bytes!(cost_database_ip);
        read_bytes!(cost_database_name);
        read_bytes!(cost_database_user);
        read_bytes!(cost_database_password);
        read_bytes!(name);
        read_number!(load_largess_time_ms, u32);
        read_bytes!(login_cost_database_provider);
        read_bytes!(login_cost_database_ip);
        read_bytes!(login_cost_database_name);
        read_bytes!(login_cost_database_user);
        read_bytes!(login_cost_database_password);
        read_number!(player_load_thread_count, u32);
        read_bytes!(language_package);
        read_bool!(use_old_save_largess_way);

        tokens.outcome()
    }
}

fn parse_ascii<T: std::str::FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_legacy_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

struct SetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> SetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    const fn outcome(&self) -> (usize, Option<usize>) {
        let stopped_at_pair = if self.parsed_pairs < self.attempted_pairs {
            Some(self.attempted_pairs)
        } else {
            None
        };
        (self.parsed_pairs, stopped_at_pair)
    }
}

pub struct WorldServerSetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    failed: bool,
}

impl<'a> WorldServerSetupTokens<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            failed: false,
        }
    }

    pub fn seek_to(&mut self, expected: &[u8]) -> bool {
        if self.failed {
            return false;
        }
        let remaining = &self.tokens[self.next..];
        let mut iterator = remaining.iter().copied();
        let found = read_to_marker(&mut iterator, expected);
        let consumed = remaining.len() - iterator.len();
        // Общий ReadTo возвращал false на успешно прочитанном `<end>`, не
        // переводя сам formatted stream в fail-state.
        let stopped_at_end =
            !found && consumed != 0 && self.tokens[self.next + consumed - 1] == b"<end>";
        self.next += consumed;
        if !found && !stopped_at_end {
            self.failed = true;
        }
        found
    }

    pub fn next_bytes(&mut self) -> Option<&'a [u8]> {
        if self.failed {
            return None;
        }
        let Some(token) = self.tokens.get(self.next).copied() else {
            self.failed = true;
            return None;
        };
        self.next += 1;
        Some(token)
    }

    pub fn next_ascii<T: std::str::FromStr>(&mut self) -> Option<T> {
        let raw = self.next_bytes()?;
        let parsed = parse_ascii(raw);
        if parsed.is_none() {
            self.failed = true;
        }
        parsed
    }

    pub const fn failed(&self) -> bool {
        self.failed
    }
}

pub fn resolve_world_runtime_file(
    runtime_directory: &Path,
    requested_name: &str,
) -> Result<PathBuf, io::Error> {
    let requested_path = runtime_directory.join(requested_name);
    match fs::metadata(&requested_path) {
        Ok(metadata) if metadata.is_file() => return Ok(requested_path),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    for entry in fs::read_dir(runtime_directory)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.eq_ignore_ascii_case(requested_name) && entry.file_type()?.is_file() {
            return Ok(entry.path());
        }
    }
    Ok(requested_path)
}
