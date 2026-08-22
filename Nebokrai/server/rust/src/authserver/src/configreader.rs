//! Владелец конфигурации AuthServer из `authserver/src/configreader.cpp`.
//!
//! Контракт `ConfigReader::{ConfigReader,reset,load,set_sp_name,getDBSP}`:
//! восстановлено для корректного baseline `setup.ini`. Необычные malformed-
//! границы числового `operator>>` локализованы ниже.
//!
//! Исходный путь PDB:
//!
//! `load` не разбирал настоящий key/value INI: двадцать раз подряд он читал
//! whitespace-token метки в один scratch `std::string`, тут же забывал его и
//! затем извлекал значение фиксированного типа. Поэтому названия меток не
//! проверяются, перестановка пар меняет назначение значений, а лишний хвост
//! после двадцатой пары игнорируется. `std::fs::read` и байтовый scanner
//! заменяют `ifstream`/locale/STL на корректном baseline без требования UTF-8
//! к адресам, именам и учётным данным.
//!
//! Поля изменялись прямо во время цепочки `operator>>`. Ошибка после открытия
//! оставляла уже прочитанный prefix, после чего оригинал всё равно отключал
//! `_enable_ipfilter`, заполнял четыре имени процедур и возвращал `false`.
//! Rust сохраняет эту частичную мутацию. Ошибка открытия не меняет объект.
//! Подтверждённая странность сохранена буквально: успешно прочитанное
//! `EnableIPFilter(NOT_USED)` безусловно заменяется `false`; поэтому Auth IP
//! allow-list этой сборкой фактически не включался через `setup.ini`.
//!
//! `Vec<u8>` заменяет `std::string`, `BTreeMap<Vec<u8>, Vec<u8>>` — старую
//! `std::map`; Rust ownership/Drop удаляют constructor/destructor и весь
//! экспортированный STL/iostream/locale/SEH noise. Отдельная parser-библиотека
//! не выбрана: обычный INI parser проверял бы имена и разделители, которых в
//! наблюдаемом формате нет, а стандартный scanner полностью покрывает
//! корректную поставку. Возвращаемый borrowed slice `get_db_sp` заменяет
//! старую копию `std::string`; отсутствие ключа по-прежнему даёт пустые bytes.
//!
//! Malformed numeric token детерминированно возвращает локальную ошибку поля:
//! это безопасная замена внутренних `num_get` state bits и overflow, которые не
//! имеют необходимого runtime-контракта. Для корректной поставки все двенадцать
//! numeric token независимо проверяются как целые нужного диапазона, а три bool
//! равны `0/1`. Редкая ошибка чтения уже открытого файла также свёрнута в
//! owned-read ошибку без попытки воспроизвести частично доступный filesystem.
//!
//! Старый `CGame::DBProcData` объединял Windows thread handle и `DBCmdProc`.
//! После прохода DB lifecycle его существенный эффект принадлежит
//! `dbaccess/authdb/authproc.rs`: owned `JoinHandle` содержит processor внутри
//! worker closure. Конструктор/destructor, forced `TerminateThread` и ручное
//! выделение отдельного Rust-типа не требуют.

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

/// Ошибка открытия либо последовательного разбора Auth `setup.ini`.
#[derive(Debug)]
pub(crate) enum ConfigLoadError {
    /// Файл не удалось прочитать целиком.
    Io(io::Error),
    /// В фиксированной последовательности отсутствует label либо value.
    MissingToken {
        /// Назначение ожидавшегося значения без раскрытия его содержимого.
        field: &'static str,
    },
    /// Числовое или bool-значение не соответствует доказанному baseline виду.
    InvalidValue {
        /// Назначение ошибочного значения без включения секрета в ошибку.
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

/// Буквальное состояние исходного Auth `ConfigReader`.
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
    /// Создаёт reader и применяет исходный `reset`.
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

    /// Восстанавливает все исходные defaults и очищает имена DB-процедур.
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

    /// Читает двадцать позиционных пар, сохраняя исходную partial-mutation.
    pub(crate) fn load(&mut self, path: impl AsRef<Path>) -> Result<(), ConfigLoadError> {
        let input = fs::read(path).map_err(ConfigLoadError::Io)?;
        let result = self.load_tokens(&input);
        // failbit. Значение из setup намеренно не влияет на runtime.
        self.enable_ip_filter = false;
        self.set_sp_name();
        result
    }

    /// Возвращает имя процедуры либо старую пустую строку для неизвестного ID.
    pub(crate) fn get_db_sp(&self, identifier: &[u8]) -> &[u8] {
        self.db_stored_procedures
            .get(identifier)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Возвращает signed предел исходной `mDBQuestQueue` без нормализации.
    pub(crate) const fn max_auth_queue_size(&self) -> i32 {
        self.max_auth_queue_size
    }

    /// Возвращает signed число Auth DB workers из исходного setup.
    pub(crate) const fn database_thread_count(&self) -> i32 {
        self.db_thread_count
    }

    /// Возвращает адрес основной Auth runtime-базы как исходные ANSI-байты.
    pub(crate) fn auth_database_host(&self) -> &[u8] {
        &self.db_ip
    }

    /// Возвращает физическое имя основной Auth runtime-базы как ANSI-байты.
    pub(crate) fn auth_database_name(&self) -> &[u8] {
        &self.auth_database_name
    }

    /// Возвращает физическое имя Auth log-базы как исходные ANSI-байты.
    pub(crate) fn log_database_name(&self) -> &[u8] {
        &self.log_database_name
    }

    /// Возвращает адрес Auth log-базы как исходные ANSI-байты.
    pub(crate) fn log_database_host(&self) -> &[u8] {
        &self.log_db_ip
    }

    /// Возвращает login Auth log-базы; значение нельзя логировать.
    pub(crate) fn log_database_user(&self) -> &[u8] {
        &self.log_db_user
    }

    /// Возвращает пароль Auth log-базы; значение нельзя логировать.
    pub(crate) fn log_database_password(&self) -> &[u8] {
        &self.log_db_password
    }

    /// Возвращает login основной Auth runtime-базы; значение нельзя логировать.
    pub(crate) fn auth_database_user(&self) -> &[u8] {
        &self.db_user
    }

    /// Возвращает пароль основной Auth runtime-базы; значение нельзя логировать.
    pub(crate) fn auth_database_password(&self) -> &[u8] {
        &self.db_password
    }

    /// Сообщает, применялся ли исходный deny-list клиентских IPv4.
    pub(crate) const fn client_ip_filter_enabled(&self) -> bool {
        self.enable_client_ip_filter
    }

    /// Сообщает, выполнял ли исходный main-loop обновление server-info.
    pub(crate) const fn update_server_info_enabled(&self) -> bool {
        self.enable_update_server_info
    }

    /// Возвращает wrapping-интервал запросов актуального server-info.
    pub(crate) const fn update_server_info_time_ms(&self) -> u32 {
        self.update_server_info_time_ms
    }

    /// Возвращает wrapping-интервал постановки server-info в DB-очередь.
    pub(crate) const fn write_server_info_time_ms(&self) -> u32 {
        self.write_server_info_time_ms
    }

    /// Собирает сетевые параметры для исходной последовательности `CGame`.
    pub(crate) fn auth_network_config(&self) -> AuthNetworkConfig {
        AuthNetworkConfig::new(
            self.host_port,
            self.max_login_server_count,
            self.send_io_number,
            self.max_client_send_buffer_size,
            self.send_inter_time,
        )
    }

    /// Создаёт начальное handler-state из config flag и разобранных шаблонов.
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

    /// Передаёт handler-state доказанный config flag и разобранные IP-шаблоны.
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
