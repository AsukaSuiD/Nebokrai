//! Конфигурация `authserver/src/configreader.cpp`, подтверждённая
//! `authserver.exe` и `authserver.pdb`.
//!
//! `setup.ini` — позиционная последовательность из двадцати пар: labels не
//! проверяются, перестановка меняет назначение, хвост игнорируется. Поля
//! изменяются по мере чтения; ошибка после открытия сохраняет прочитанный prefix,
//! затем отключает IP-фильтр и устанавливает имена процедур. Ошибка открытия
//! объекта не меняет. Прочитанный `EnableIPFilter` эта сборка всегда заменяла
//! на `false`.
//!
//! Байтовый scanner сохраняет не-UTF-8 данные без семантики обычного INI parser.
//! Некорректное число или bool безопасно возвращает ошибку поля вместо внутренних
//! состояний iostream и переполнения, не имевших подтверждённого контракта.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

use crate::authserver::appauth::message::message_func::AuthMessageHandlers;
use crate::authserver::src::cgame::{AuthDbContext, AuthNetworkConfig};

const FIELD_COUNT: usize = 20;

#[derive(Debug)]
pub(crate) enum ConfigLoadError {
    Io(io::Error),
    MissingToken {
        field: &'static str,
    },
    InvalidValue {
        field: &'static str,
    },
}

impl fmt::Display for ConfigLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не удалось прочитать Auth setup.ini: {error}"),
            Self::MissingToken { field } => {
                write!(
                    formatter,
                    "в Auth setup.ini отсутствует пара для поля {field}"
                )
            }
            Self::InvalidValue { field } => {
                write!(
                    formatter,
                    "в Auth setup.ini недопустимое значение поля {field}"
                )
            }
        }
    }
}

impl Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::MissingToken { .. } | Self::InvalidValue { .. } => None,
        }
    }
}

pub(crate) struct ConfigReader {
    host_port: u32,
    db_thread_count: i32,
    max_login_server_count: i32,
    send_io_number: i32,
    max_client_send_buffer_size: i32,
    send_inter_time: i32,
    enable_ip_filter: bool,
    db_ip: Vec<u8>,
    auth_database_name: Vec<u8>,
    db_user: Vec<u8>,
    db_password: Vec<u8>,
    db_sp_config: Vec<u8>,
    log_db_ip: Vec<u8>,
    log_database_name: Vec<u8>,
    log_db_user: Vec<u8>,
    log_db_password: Vec<u8>,
    max_auth_queue_size: i32,
    enable_client_ip_filter: bool,
    update_server_info_time_ms: u32,
    write_server_info_time_ms: u32,
    enable_update_server_info: bool,
    db_stored_procedures: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl ConfigReader {
    pub(crate) fn new() -> Self {
        let mut reader = Self {
            host_port: 0,
            db_thread_count: 0,
            max_login_server_count: 0,
            send_io_number: 0,
            max_client_send_buffer_size: 0,
            send_inter_time: 0,
            enable_ip_filter: false,
            db_ip: Vec::new(),
            auth_database_name: Vec::new(),
            db_user: Vec::new(),
            db_password: Vec::new(),
            db_sp_config: Vec::new(),
            log_db_ip: Vec::new(),
            log_database_name: Vec::new(),
            log_db_user: Vec::new(),
            log_db_password: Vec::new(),
            max_auth_queue_size: 0,
            enable_client_ip_filter: false,
            update_server_info_time_ms: 0,
            write_server_info_time_ms: 0,
            enable_update_server_info: false,
            db_stored_procedures: BTreeMap::new(),
        };
        reader.reset();
        reader
    }

    pub(crate) fn reset(&mut self) {
        self.host_port = 0x1BBC;
        self.db_thread_count = 1;
        self.max_login_server_count = 20;
        self.send_io_number = 100;
        self.max_client_send_buffer_size = 0x0A00_0000;
        self.send_inter_time = 5_000;
        self.enable_ip_filter = false;
        self.enable_client_ip_filter = false;
        self.update_server_info_time_ms = 30_000;
        self.write_server_info_time_ms = 30_000;
        self.enable_update_server_info = false;
        self.db_ip.clear();
        self.auth_database_name = b"DB_gCFY".to_vec();
        self.db_user.clear();
        self.db_password.clear();
        self.db_sp_config = b"dbspcfg.ini".to_vec();
        self.log_db_ip.clear();
        self.log_database_name = b"Unknown".to_vec();
        self.log_db_user.clear();
        self.log_db_password.clear();
        self.max_auth_queue_size = 3_000;
        self.db_stored_procedures.clear();
    }

    pub(crate) fn load(&mut self, path: impl AsRef<Path>) -> Result<(), ConfigLoadError> {
        let input = fs::read(path).map_err(ConfigLoadError::Io)?;
        let result = self.load_tokens(&input);
        // После любого результата разбора значение из setup не влияет на runtime.
        self.enable_ip_filter = false;
        self.set_sp_name();
        result
    }

    pub(crate) fn get_db_sp(&self, identifier: &[u8]) -> &[u8] {
        self.db_stored_procedures
            .get(identifier)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) const fn max_auth_queue_size(&self) -> i32 {
        self.max_auth_queue_size
    }

    pub(crate) const fn database_thread_count(&self) -> i32 {
        self.db_thread_count
    }

    pub(crate) fn auth_database_host(&self) -> &[u8] {
        &self.db_ip
    }

    pub(crate) fn auth_database_name(&self) -> &[u8] {
        &self.auth_database_name
    }

    pub(crate) fn log_database_name(&self) -> &[u8] {
        &self.log_database_name
    }

    pub(crate) fn log_database_host(&self) -> &[u8] {
        &self.log_db_ip
    }

    pub(crate) fn log_database_user(&self) -> &[u8] {
        &self.log_db_user
    }

    pub(crate) fn log_database_password(&self) -> &[u8] {
        &self.log_db_password
    }

    pub(crate) fn auth_database_user(&self) -> &[u8] {
        &self.db_user
    }

    pub(crate) fn auth_database_password(&self) -> &[u8] {
        &self.db_password
    }

    pub(crate) const fn client_ip_filter_enabled(&self) -> bool {
        self.enable_client_ip_filter
    }

    pub(crate) const fn update_server_info_enabled(&self) -> bool {
        self.enable_update_server_info
    }

    pub(crate) const fn update_server_info_time_ms(&self) -> u32 {
        self.update_server_info_time_ms
    }

    pub(crate) const fn write_server_info_time_ms(&self) -> u32 {
        self.write_server_info_time_ms
    }

    pub(crate) fn auth_network_config(&self) -> AuthNetworkConfig {
        AuthNetworkConfig::new(
            self.host_port,
            self.max_login_server_count,
            self.send_io_number,
            self.max_client_send_buffer_size,
            self.send_inter_time,
        )
    }

    pub(crate) fn auth_message_handlers(
        &self,
        allowed_patterns: Vec<[u8; 4]>,
        game: AuthDbContext,
    ) -> AuthMessageHandlers {
        AuthMessageHandlers::new(
            self.enable_ip_filter,
            self.active_login_server_patterns(allowed_patterns),
            game,
        )
    }

    pub(crate) fn apply_login_server_filter(
        &self,
        handlers: &mut AuthMessageHandlers,
        allowed_patterns: Vec<[u8; 4]>,
    ) {
        handlers.replace_ip_filter(
            self.enable_ip_filter,
            self.active_login_server_patterns(allowed_patterns),
        );
    }

    fn set_sp_name(&mut self) {
        for (identifier, procedure) in [
            (b"sp_auth".as_slice(), b"getAccInfo".as_slice()),
            (b"sp_lock".as_slice(), b"suspendAccount".as_slice()),
            (b"sp_authex".as_slice(), b"GetAccount".as_slice()),
            (b"sp_writelog".as_slice(), b"PutOnlineLog".as_slice()),
        ] {
            self.db_stored_procedures
                .insert(identifier.to_vec(), procedure.to_vec());
        }
    }

    fn active_login_server_patterns(&self, patterns: Vec<[u8; 4]>) -> Vec<[u8; 4]> {
        if self.enable_ip_filter {
            patterns
        } else {
            Vec::new()
        }
    }

    fn load_tokens(&mut self, input: &[u8]) -> Result<(), ConfigLoadError> {
        let mut tokens = TokenCursor::new(input);

        self.host_port = tokens.pair_number("AuthServerPort")?;
        self.db_thread_count = tokens.pair_number("DatabaseAuthThreadCount")?;
        self.max_login_server_count = tokens.pair_number("MaxLoginServerCount")?;
        self.send_io_number = tokens.pair_number("SendIONum")?;
        self.max_client_send_buffer_size = tokens.pair_number("MaxClientSendBufSize")?;
        self.send_inter_time = tokens.pair_number("SendInterTime")?;
        self.enable_ip_filter = tokens.pair_bool("EnableIPFilter(NOT_USED)")?;
        self.db_ip = tokens.pair_bytes("DatabaseIP")?;
        self.auth_database_name = tokens.pair_bytes("AuthDatabaseName")?;
        self.db_user = tokens.pair_bytes("DatabaseUser")?;
        self.db_password = tokens.pair_bytes("DatabasePassword")?;
        self.log_db_ip = tokens.pair_bytes("LogDatabaseIP")?;
        self.log_database_name = tokens.pair_bytes("LogDatabaseName")?;
        self.log_db_user = tokens.pair_bytes("LogDatabaseUser")?;
        self.log_db_password = tokens.pair_bytes("LogDatabasePassword")?;
        self.max_auth_queue_size = tokens.pair_number("MaxAuthQueueSize")?;
        self.enable_client_ip_filter = tokens.pair_bool("EnableClientIPFilter")?;
        self.update_server_info_time_ms = tokens.pair_number("UpdateServerInfoTime(ms)")?;
        self.write_server_info_time_ms = tokens.pair_number("WriteServerInfoTime(ms)")?;
        self.enable_update_server_info = tokens.pair_bool("EnableUpdateServerInfo(1or0)")?;

        debug_assert_eq!(tokens.pairs_read, FIELD_COUNT);
        Ok(())
    }
}

struct TokenCursor<'input> {
    input: &'input [u8],
    offset: usize,
    pairs_read: usize,
}

impl<'input> TokenCursor<'input> {
    const fn new(input: &'input [u8]) -> Self {
        Self {
            input,
            offset: 0,
            pairs_read: 0,
        }
    }

    fn pair_bytes(&mut self, field: &'static str) -> Result<Vec<u8>, ConfigLoadError> {
        self.discard_label(field)?;
        let value = self.next().ok_or(ConfigLoadError::MissingToken { field })?;
        self.pairs_read += 1;
        Ok(value.to_vec())
    }

    fn pair_number<Value>(&mut self, field: &'static str) -> Result<Value, ConfigLoadError>
    where
        Value: FromStr,
    {
        self.discard_label(field)?;
        let token = self.next().ok_or(ConfigLoadError::MissingToken { field })?;
        let text =
            std::str::from_utf8(token).map_err(|_| ConfigLoadError::InvalidValue { field })?;
        let value = text
            .parse()
            .map_err(|_| ConfigLoadError::InvalidValue { field })?;
        self.pairs_read += 1;
        Ok(value)
    }

    fn pair_bool(&mut self, field: &'static str) -> Result<bool, ConfigLoadError> {
        let value: u8 = self.pair_number(field)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ConfigLoadError::InvalidValue { field }),
        }
    }

    fn discard_label(&mut self, field: &'static str) -> Result<(), ConfigLoadError> {
        self.next()
            .map(drop)
            .ok_or(ConfigLoadError::MissingToken { field })
    }

    fn next(&mut self) -> Option<&'input [u8]> {
        while self
            .input
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
        let start = self.offset;
        while self
            .input
            .get(self.offset)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            self.offset += 1;
        }
        (start != self.offset).then_some(&self.input[start..self.offset])
    }
}
