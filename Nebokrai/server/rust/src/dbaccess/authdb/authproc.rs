//! Владелец Auth MSSQL-команд из `dbaccess/authdb/authproc.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для `do_auth`, `do_auth_ex`, `do_lock`,
//! `do_write_log`, трёх `push_*_result`, обработки команд и DB worker
//! lifecycle.
//!
//! Точная пара: `AuthServer/authserver.exe + AuthServer/authserver.pdb`;
//! SHA-256 EXE:
//! `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`,
//! SHA-256 PDB:
//! `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`;
//! исходный владелец PDB:
//! `h:\fengyun\fy_russia\src\dbaccess\authdb\authproc.cpp`.
//! Существенные RVA: constructor `0x00016E10`, `init` `0x00016E20`,
//! `do_auth` `0x00017530`, `do_auth_ex` `0x00017CC0`, `do_lock` `0x00018730`,
//! result builders `0x00018DD0`, `0x00018F90`, `0x00019160`,
//! `do_write_log` `0x00019270`, `process_quest` `0x0001A360`, worker entry
//! `0x0001A910`.
//!
//! `tiberius` заменяет ADO/COM и выполняет те же именованные MSSQL-процедуры.
//! Каждый вызов по-прежнему открывает отдельное соединение; параметры account,
//! password и IPv4 передаются как bind values, а output-параметры читаются
//! через узкий `DECLARE/EXEC/SELECT` batch. Procedure name берётся только из
//! исходной карты `ConfigReader` и экранируется как SQL identifier. Драйвер
//! собран без TLS feature: исходная ODBC-строка не запрашивала шифрование, а
//! разрешённая цель этой фазы — специально поднятая локальная baseline MSSQL.
//! Auth и log соединения используют собственные host/database/user/password
//! поля исходного setup; секреты не входят ни в SQL-текст, ни в ошибки этого
//! владельца. Auth settings обновляются перед каждым queue snapshot, log
//! settings — перед каждым `do_write_log`, сохраняя config reload.
//!
//! `do_write_log` один раз получает local time Auth-хоста, затем атомарно
//! забирает весь coalesced FIFO `ServerInfo` и открывает отдельное соединение с
//! log-базой. `PutOnlineLog` выполняется последовательно для каждого tuple с
//! одной меткой времени и параметрами `@LogTime`, `@ls`, `@ws`, `@gs`,
//! `@Amount`. `chrono::Local` и Tiberius datetime заменяют `GetLocalTime` и OLE
//! DATE без ручного platform-кода. Если connection или отдельный вызов падает,
//! весь уже вынутый snapshot, включая ещё не записанный хвост, теряется как в
//! оригинале; событие ошибки сохраняет эту странность явно.
//! Неизменяемый `server/database/mssql-source/Account.bak` независимо
//! подтверждает `dbo.PutOnlineLog`, тип `datetime`, четыре signed `int` и
//! `INSERT INTO OnlineLog(LogTime,ls,ws,gs,Amount)` в том же порядке.
//!
//! `encoding_rs::WINDOWS_1251` заменяет преобразование ANSI `char*` в ADO BSTR
//! русской поставки. Перед varchar-параметрами SQL явно делает
//! `CONVERT(varchar(200), ...)`, сохраняя не Unicode-тип старого `adVarChar`.
//! IPv4 форматируется из младших host-endian octet, как `inet_ntoa` на исходном
//! 32-битном значении.
//!
//! Ошибка ADO оставляла `do_auth/do_auth_ex` с начальным `-2`, а `do_lock` — с
//! false; те же fallback возвращаются после записи структурированного события.
//! Пустой account или пустое имя процедуры по-прежнему не создают результата.
//! Старые утечки на этих ветвях исчезают только как ненаблюдаемый ownership-
//! шум Rust.
//!
//! Каждый DB worker сначала проверяет stop, затем отдельно читает начальный
//! 32-битный размер общей очереди, делает ровно столько ожидающих FIFO-pop и
//! после прохода спит `1 ms`. `std::thread::JoinHandle` заменяет
//! `_beginthreadex`/handle, `AtomicBool` — private Windows stop message, а
//! `Arc` сохраняет общее владение очередями без глобального `GetGame`.
//! `DBProcData` схлопнут в owned closure: отдельный Rust-аналог ручного
//! allocation, vtable и destructor не нужен. Shutdown публикует stop, будит
//! только пустое condvar-ожидание и присоединяет workers в порядке создания;
//! forced `TerminateThread` не переносится.
//!
//! При отказе `_beginthreadex` оригинальный `CGame::Init` сохранял запись с null
//! handle и продолжал. Это внутренний lifecycle-дефект без полезного внешнего
//! эффекта: Rust останавливает уже созданные workers и возвращает start-ошибку.
//!
//! ADO-код копировал ровно 80 байт `@Assure` и читал
//! `SYSTEMTIME` без проверки nullable/длины. Для корректного результата база
//! обязана вернуть 80 байт при DB-коде `1` и дату при `-3`. Нарушение сейчас
//! детерминированно даёт обычный DB fallback `-2`, не воспроизводя out-of-bounds.
//!
//! `SystemTimeToVariantTime` для некорректных полей
//! блокировки возвращал BOOL, который старый код игнорировал. Корректная дата
//! передаётся как ISO-строка в `CONVERT(datetime, ..., 126)`; реакция на
//! некорректную дату безопасно остаётся локальной DB-ошибкой.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use encoding_rs::WINDOWS_1251;
use parking_lot::Mutex;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::authserver::src::cgame::AuthDbContext;
use crate::authserver::src::configreader::ConfigReader;
use crate::authserver::src::dbqueue::{
    AuthExResultData, AuthQuestData, AuthResultData, DbQuest, DbResult, LockQuestData,
    LockResultData, LockUntil, ServerInfo,
};
type TdsClient = Client<Compat<TcpStream>>;

const DB_WORKER_DELAY: Duration = Duration::from_millis(1);

/// Операция, на которой ADO/TDS-владелец применил исходный fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthDatabaseOperation {
    /// Создание технического состояния DB worker’а.
    Initialize,
    /// Обычная авторизация `sp_auth`.
    Authenticate,
    /// Расширенная авторизация `sp_authex`.
    AuthenticateExtended,
    /// Блокировка аккаунта `sp_lock`.
    Lock,
    /// Coalesced server-info snapshot через `sp_writelog`.
    WriteServerInfo,
}

/// Структурированное событие вместо старого `AddLogText` с COM description.
pub(crate) struct AuthDatabaseNotice {
    /// Операция, завершившаяся ошибкой.
    pub(crate) operation: AuthDatabaseOperation,
    /// Ошибка без connection string и credential values.
    pub(crate) error: AuthDatabaseError,
}

/// Ошибка технической MSSQL-границы AuthServer.
#[derive(Debug)]
pub(crate) enum AuthDatabaseError {
    /// Не удалось создать локальный Tokio runtime DB worker’а.
    Runtime(io::Error),
    /// TCP-соединение с настроенным MSSQL endpoint не установлено.
    Connect(io::Error),
    /// TDS login, batch либо чтение результата завершились ошибкой.
    Tds(tiberius::error::Error),
    /// Имя процедуры вне доказанного identifier-формата.
    InvalidProcedureName,
    /// Процедура не вернула единственную ожидаемую строку output-значений.
    MissingOutputRow,
    /// Обязательный output-параметр оказался `NULL`.
    MissingOutput(&'static str),
    /// `@Assure` нарушил доказанный 80-байтовый baseline layout.
    UnexpectedAssureLength(usize),
}

impl fmt::Display for AuthDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "не создан runtime Auth DB: {error}"),
            Self::Connect(error) => write!(formatter, "не установлено соединение Auth DB: {error}"),
            Self::Tds(error) => write!(formatter, "ошибка протокола или команды Auth DB: {error}"),
            Self::InvalidProcedureName => {
                formatter.write_str("имя Auth DB-процедуры не является SQL identifier")
            }
            Self::MissingOutputRow => {
                formatter.write_str("Auth DB-процедура не вернула output-строку")
            }
            Self::MissingOutput(parameter) => {
                write!(formatter, "Auth DB-процедура вернула NULL в {parameter}")
            }
            Self::UnexpectedAssureLength(length) => write!(
                formatter,
                "Auth DB-процедура вернула @Assure длиной {length} вместо 80 байт"
            ),
        }
    }
}

impl Error for AuthDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) | Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
            Self::InvalidProcedureName
            | Self::MissingOutputRow
            | Self::MissingOutput(_)
            | Self::UnexpectedAssureLength(_) => None,
        }
    }
}

impl From<tiberius::error::Error> for AuthDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Выходные значения исходной процедуры `sp_authex` до protocol mapping.
pub(crate) struct AuthExtendedDatabaseResult {
    /// DB-код `@Res`, либо принудительный `1` при непустом `@Assure`.
    pub(crate) result: i32,
    /// Ровно 80 байт `@Assure`, если процедура их вернула.
    pub(crate) assure: Option<[u8; 80]>,
    /// Дата `@suspended`, если процедура её вернула.
    pub(crate) suspended: Option<LockUntil>,
}

/// Минимальный контракт трёх account-процедур AuthServer.
pub(crate) trait AuthDatabase {
    /// Обновляет Auth connection settings из одного config snapshot.
    fn refresh_auth_config(&mut self, config: &ConfigReader);

    /// Обновляет log connection settings перед исходным `do_write_log`.
    fn refresh_log_config(&mut self, config: &ConfigReader);

    /// Выполняет `sp_auth` и возвращает исходный signed `@Result`.
    fn authenticate(
        &mut self,
        procedure: &[u8],
        request: &AuthQuestData,
    ) -> Result<i32, AuthDatabaseError>;

    /// Выполняет `sp_authex` и сохраняет nullable output-поля.
    fn authenticate_extended(
        &mut self,
        procedure: &[u8],
        request: &AuthQuestData,
    ) -> Result<AuthExtendedDatabaseResult, AuthDatabaseError>;

    /// Выполняет `sp_lock`; нулевой `@Result` означает успех.
    fn lock(
        &mut self,
        procedure: &[u8],
        request: &LockQuestData,
    ) -> Result<bool, AuthDatabaseError>;

    /// Последовательно записывает весь уже вынутый server-info snapshot.
    fn write_server_info(
        &mut self,
        procedure: &[u8],
        logged_at: NaiveDateTime,
        entries: VecDeque<ServerInfo>,
    ) -> Result<(), AuthDatabaseError>;
}

/// Linux/TDS-замена ADO connection+command одного исходного DB worker’а.
pub(crate) struct TiberiusAuthDatabase {
    auth_config: Config,
    log_config: Config,
    runtime: Runtime,
}

impl TiberiusAuthDatabase {
    /// Создаёт per-worker runtime и копирует исходные connection settings без
    /// публикации credential values.
    pub(crate) fn new(config_reader: &ConfigReader) -> Result<Self, AuthDatabaseError> {
        let auth_config = create_tds_config(
            config_reader.auth_database_host(),
            config_reader.auth_database_name(),
            config_reader.auth_database_user(),
            config_reader.auth_database_password(),
        );
        let log_config = create_tds_config(
            config_reader.log_database_host(),
            config_reader.log_database_name(),
            config_reader.log_database_user(),
            config_reader.log_database_password(),
        );

        let runtime = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(AuthDatabaseError::Runtime)?;
        Ok(Self {
            auth_config,
            log_config,
            runtime,
        })
    }

    async fn connect(config: Config) -> Result<TdsClient, AuthDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(AuthDatabaseError::Connect)?;
        tcp.set_nodelay(true).map_err(AuthDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(AuthDatabaseError::Tds)
    }

    async fn query_auth(
        config: Config,
        procedure: String,
        account: String,
        password: String,
        address: String,
    ) -> Result<i32, AuthDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = format!(
            "DECLARE @Result int; \
             EXEC {procedure} \
             @UserID = CONVERT(varchar(200), @P1), \
             @UserPWDb = CONVERT(varchar(200), @P2), \
             @UserIP = CONVERT(varchar(200), @P3), \
             @Result = @Result OUTPUT; \
             SELECT @Result;"
        );
        let row = client
            .query(sql, &[&account, &password, &address])
            .await?
            .into_row()
            .await?
            .ok_or(AuthDatabaseError::MissingOutputRow)?;
        row.get::<i32, _>(0)
            .ok_or(AuthDatabaseError::MissingOutput("@Result"))
    }

    async fn query_auth_extended(
        config: Config,
        procedure: String,
        account: String,
        address: String,
    ) -> Result<AuthExtendedDatabaseResult, AuthDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = format!(
            "DECLARE @suspended datetime, @Assure varbinary(80), @Res int; \
             EXEC {procedure} \
             @acc = CONVERT(varchar(200), @P1), \
             @ip = CONVERT(varchar(200), @P2), \
             @suspended = @suspended OUTPUT, \
             @Assure = @Assure OUTPUT, \
             @Res = @Res OUTPUT; \
             SELECT @Res, @Assure, \
             DATEPART(year, @suspended), DATEPART(month, @suspended), \
             DATEPART(day, @suspended), DATEPART(hour, @suspended), \
             DATEPART(minute, @suspended), DATEPART(second, @suspended);"
        );
        let row = client
            .query(sql, &[&account, &address])
            .await?
            .into_row()
            .await?
            .ok_or(AuthDatabaseError::MissingOutputRow)?;
        let mut result = row
            .get::<i32, _>(0)
            .ok_or(AuthDatabaseError::MissingOutput("@Res"))?;
        let assure = row
            .get::<&[u8], _>(1)
            .map(|bytes| {
                bytes
                    .try_into()
                    .map_err(|_: std::array::TryFromSliceError| {
                        AuthDatabaseError::UnexpectedAssureLength(bytes.len())
                    })
            })
            .transpose()?;
        if assure.is_some() {
            result = 1;
        }

        let date_parts = [
            row.get::<i32, _>(2),
            row.get::<i32, _>(3),
            row.get::<i32, _>(4),
            row.get::<i32, _>(5),
            row.get::<i32, _>(6),
            row.get::<i32, _>(7),
        ];
        let suspended = date_parts
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .map(|parts| LockUntil {
                year: parts[0] as u16,
                month: parts[1] as u16,
                day: parts[2] as u16,
                hour: parts[3] as u16,
                minute: parts[4] as u16,
                second: parts[5] as u16,
            });

        if result == 1 && assure.is_none() {
            return Err(AuthDatabaseError::MissingOutput("@Assure"));
        }
        if result == -3 && suspended.is_none() {
            return Err(AuthDatabaseError::MissingOutput("@suspended"));
        }
        Ok(AuthExtendedDatabaseResult {
            result,
            assure,
            suspended,
        })
    }

    async fn query_lock(
        config: Config,
        procedure: String,
        account: String,
        suspend_time: String,
    ) -> Result<bool, AuthDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = format!(
            "DECLARE @Result int; \
             EXEC {procedure} \
             @Account = CONVERT(varchar(200), @P1), \
             @SuspendTime = CONVERT(datetime, @P2, 126), \
             @Result = @Result OUTPUT; \
             SELECT @Result;"
        );
        let row = client
            .query(sql, &[&account, &suspend_time])
            .await?
            .into_row()
            .await?
            .ok_or(AuthDatabaseError::MissingOutputRow)?;
        Ok(row
            .get::<i32, _>(0)
            .ok_or(AuthDatabaseError::MissingOutput("@Result"))?
            == 0)
    }

    async fn execute_server_info(
        config: Config,
        procedure: String,
        logged_at: NaiveDateTime,
        entries: VecDeque<ServerInfo>,
    ) -> Result<(), AuthDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = format!(
            "EXEC {procedure} \
             @LogTime = @P1, @ls = @P2, @ws = @P3, \
             @gs = @P4, @Amount = @P5;"
        );
        for entry in entries {
            client
                .execute(
                    &sql,
                    &[
                        &logged_at,
                        &entry.login_server_id,
                        &entry.world_server_id,
                        &entry.game_server_id,
                        &entry.player_count,
                    ],
                )
                .await?;
        }
        Ok(())
    }
}

impl AuthDatabase for TiberiusAuthDatabase {
    fn refresh_auth_config(&mut self, config: &ConfigReader) {
        self.auth_config = create_tds_config(
            config.auth_database_host(),
            config.auth_database_name(),
            config.auth_database_user(),
            config.auth_database_password(),
        );
    }

    fn refresh_log_config(&mut self, config: &ConfigReader) {
        self.log_config = create_tds_config(
            config.log_database_host(),
            config.log_database_name(),
            config.log_database_user(),
            config.log_database_password(),
        );
    }

    fn authenticate(
        &mut self,
        procedure: &[u8],
        request: &AuthQuestData,
    ) -> Result<i32, AuthDatabaseError> {
        let procedure = quote_procedure(procedure)?;
        let account = decode_ansi(&request.account);
        let password = decode_ansi(&request.password);
        let address = legacy_ipv4_text(request.client_ip);
        self.runtime.block_on(Self::query_auth(
            self.auth_config.clone(),
            procedure,
            account,
            password,
            address,
        ))
    }

    fn authenticate_extended(
        &mut self,
        procedure: &[u8],
        request: &AuthQuestData,
    ) -> Result<AuthExtendedDatabaseResult, AuthDatabaseError> {
        let procedure = quote_procedure(procedure)?;
        let account = decode_ansi(&request.account);
        let address = legacy_ipv4_text(request.client_ip);
        self.runtime.block_on(Self::query_auth_extended(
            self.auth_config.clone(),
            procedure,
            account,
            address,
        ))
    }

    fn lock(
        &mut self,
        procedure: &[u8],
        request: &LockQuestData,
    ) -> Result<bool, AuthDatabaseError> {
        let procedure = quote_procedure(procedure)?;
        let account = decode_ansi(&request.account);
        let until = &request.until;
        let suspend_time = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            until.year, until.month, until.day, until.hour, until.minute, until.second
        );
        self.runtime.block_on(Self::query_lock(
            self.auth_config.clone(),
            procedure,
            account,
            suspend_time,
        ))
    }

    fn write_server_info(
        &mut self,
        procedure: &[u8],
        logged_at: NaiveDateTime,
        entries: VecDeque<ServerInfo>,
    ) -> Result<(), AuthDatabaseError> {
        let procedure = quote_procedure(procedure)?;
        self.runtime.block_on(Self::execute_server_info(
            self.log_config.clone(),
            procedure,
            logged_at,
            entries,
        ))
    }
}

/// Обработчик доказанных Auth DB-команд одного worker’а.
pub(crate) struct DBCmdProc<Database> {
    database: Database,
    notices: VecDeque<AuthDatabaseNotice>,
}

impl<Database> DBCmdProc<Database>
where
    Database: AuthDatabase,
{
    /// Создаёт пустое состояние processor; отдельный COM init больше не нужен.
    pub(crate) fn new(database: Database) -> Self {
        Self {
            database,
            notices: VecDeque::new(),
        }
    }

    /// Забирает следующее DB-событие в порядке обработанных команд.
    pub(crate) fn pop_notice(&mut self) -> Option<AuthDatabaseNotice> {
        self.notices.pop_front()
    }

    /// Обрабатывает одну доказанную DB-команду и публикует её результат.
    pub(crate) fn process_quest(&mut self, game: &AuthDbContext, quest: DbQuest) {
        match quest {
            DbQuest::Authenticate {
                return_socket_id,
                request,
            } => {
                self.process_authenticate(game, return_socket_id, request, false);
            }
            DbQuest::AuthenticateExtended {
                return_socket_id,
                request,
            } => {
                self.process_authenticate(game, return_socket_id, request, true);
            }
            DbQuest::Lock {
                return_socket_id,
                request,
            } => {
                self.process_lock(game, return_socket_id, request);
            }
            DbQuest::WriteServerInfo => self.process_write_server_info(game),
        }
    }

    fn process_write_server_info(&mut self, game: &AuthDbContext) {
        let config = game.config();
        self.database.refresh_log_config(&config);
        let procedure = config.get_db_sp(b"sp_writelog").to_vec();
        drop(config);
        // Auth RVA 0x00019270 фиксирует время до атомарного pop_all.
        let logged_at = Local::now().naive_local();
        let entries = game.pop_all_server_info();
        if let Err(error) = self
            .database
            .write_server_info(&procedure, logged_at, entries)
        {
            self.notices.push_back(AuthDatabaseNotice {
                operation: AuthDatabaseOperation::WriteServerInfo,
                error,
            });
        }
    }

    fn process_authenticate(
        &mut self,
        game: &AuthDbContext,
        return_socket_id: i32,
        request: AuthQuestData,
        extended: bool,
    ) {
        if request.account.is_empty() {
            return;
        }
        let identifier: &[u8] = if extended { b"sp_authex" } else { b"sp_auth" };
        let procedure = game.config().get_db_sp(identifier).to_vec();
        if procedure.is_empty() {
            return;
        }
        if !game.is_client_ip_allowed(request.client_ip) {
            if extended {
                game.push_result(DbResult::AuthenticateExtended {
                    return_socket_id,
                    result: build_auth_ex_result(
                        request,
                        AuthExtendedDatabaseResult {
                            result: -7,
                            assure: None,
                            suspended: None,
                        },
                    ),
                });
            } else {
                game.push_result(DbResult::Authenticate {
                    return_socket_id,
                    result: build_auth_result(request, -7),
                });
            }
            return;
        }

        if extended {
            let result = match self.database.authenticate_extended(&procedure, &request) {
                Ok(result) => result,
                Err(error) => {
                    self.notices.push_back(AuthDatabaseNotice {
                        operation: AuthDatabaseOperation::AuthenticateExtended,
                        error,
                    });
                    AuthExtendedDatabaseResult {
                        result: -2,
                        assure: None,
                        suspended: None,
                    }
                }
            };
            game.push_result(DbResult::AuthenticateExtended {
                return_socket_id,
                result: build_auth_ex_result(request, result),
            });
        } else {
            let result = match self.database.authenticate(&procedure, &request) {
                Ok(result) => result,
                Err(error) => {
                    self.notices.push_back(AuthDatabaseNotice {
                        operation: AuthDatabaseOperation::Authenticate,
                        error,
                    });
                    -2
                }
            };
            game.push_result(DbResult::Authenticate {
                return_socket_id,
                result: build_auth_result(request, result),
            });
        }
    }

    fn process_lock(
        &mut self,
        game: &AuthDbContext,
        return_socket_id: i32,
        request: LockQuestData,
    ) {
        if request.account.is_empty() {
            return;
        }
        let procedure = game.config().get_db_sp(b"sp_lock").to_vec();
        if procedure.is_empty() {
            return;
        }
        let succeeded = match self.database.lock(&procedure, &request) {
            Ok(succeeded) => succeeded,
            Err(error) => {
                self.notices.push_back(AuthDatabaseNotice {
                    operation: AuthDatabaseOperation::Lock,
                    error,
                });
                false
            }
        };
        game.push_result(DbResult::Lock {
            return_socket_id,
            result: LockResultData {
                account: request.account,
                succeeded,
            },
        });
    }
}

/// Ошибка создания owned Auth DB worker’а.
#[derive(Debug)]
pub(crate) enum AuthDatabaseWorkerStartError {
    /// Повторный запуск запрошен до присоединения прежних workers.
    AlreadyRunning,
    /// ОС не смогла создать один из настроенных потоков.
    Spawn {
        /// Номер worker’а в исходном порядке создания.
        worker_index: usize,
        /// Системная причина отказа `std::thread::Builder`.
        source: io::Error,
    },
}

impl fmt::Display for AuthDatabaseWorkerStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => formatter.write_str("Auth DB workers уже запущены"),
            Self::Spawn {
                worker_index,
                source,
            } => write!(
                formatter,
                "не удалось создать Auth DB worker {worker_index}: {source}"
            ),
        }
    }
}

impl Error for AuthDatabaseWorkerStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::AlreadyRunning => None,
        }
    }
}

/// Аварийный результат присоединения Auth DB worker’а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuthDatabaseWorkerJoinError {
    /// Номер worker’а в исходном порядке создания.
    pub(crate) worker_index: usize,
}

impl fmt::Display for AuthDatabaseWorkerJoinError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Auth DB worker {} завершился panic",
            self.worker_index
        )
    }
}

impl Error for AuthDatabaseWorkerJoinError {}

struct AuthDatabaseWorkerHandle {
    index: usize,
    join: JoinHandle<()>,
}

/// Owned-замена списка `CGame::DBProcData` и Windows thread messages.
pub(crate) struct AuthDatabaseWorkers {
    stopped: Arc<AtomicBool>,
    context: Option<AuthDbContext>,
    workers: Vec<AuthDatabaseWorkerHandle>,
    notices: Arc<Mutex<VecDeque<AuthDatabaseNotice>>>,
}

impl AuthDatabaseWorkers {
    /// Создаёт остановленный владелец без фоновых потоков.
    pub(crate) fn new() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
            context: None,
            workers: Vec::new(),
            notices: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Запускает положительное signed число workers в исходном порядке.
    pub(crate) fn start(
        &mut self,
        context: AuthDbContext,
        worker_count: i32,
    ) -> Result<(), AuthDatabaseWorkerStartError> {
        if !self.workers.is_empty() {
            return Err(AuthDatabaseWorkerStartError::AlreadyRunning);
        }

        self.stopped = Arc::new(AtomicBool::new(false));
        self.context = Some(context.clone());
        for worker_index in 0..worker_count.max(0) as usize {
            let worker_context = context.clone();
            let stopped = Arc::clone(&self.stopped);
            let notices = Arc::clone(&self.notices);
            let spawn = thread::Builder::new()
                .name(format!("auth-db-{worker_index}"))
                .spawn(move || {
                    run_database_worker(worker_context, stopped, notices);
                });
            match spawn {
                Ok(join) => self.workers.push(AuthDatabaseWorkerHandle {
                    index: worker_index,
                    join,
                }),
                Err(source) => {
                    let _join_errors = self.stop_and_join();
                    return Err(AuthDatabaseWorkerStartError::Spawn {
                        worker_index,
                        source,
                    });
                }
            }
        }
        Ok(())
    }

    /// Публикует stop, будит пустое ожидание и присоединяет workers по порядку.
    pub(crate) fn stop_and_join(&mut self) -> Vec<AuthDatabaseWorkerJoinError> {
        self.stopped.store(true, Ordering::Release);
        if let Some(context) = &self.context {
            context.wake_quest_waiters();
        }

        let mut errors = Vec::new();
        for worker in self.workers.drain(..) {
            if worker.join.join().is_err() {
                errors.push(AuthDatabaseWorkerJoinError {
                    worker_index: worker.index,
                });
            }
        }
        self.context = None;
        errors
    }

    /// Забирает следующее DB-событие в фактическом порядке публикации workers.
    pub(crate) fn pop_notice(&self) -> Option<AuthDatabaseNotice> {
        self.notices.lock().pop_front()
    }
}

impl Default for AuthDatabaseWorkers {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AuthDatabaseWorkers {
    fn drop(&mut self) {
        let _join_errors = self.stop_and_join();
    }
}

fn run_database_worker(
    context: AuthDbContext,
    stopped: Arc<AtomicBool>,
    notices: Arc<Mutex<VecDeque<AuthDatabaseNotice>>>,
) {
    let database = {
        let config = context.config();
        TiberiusAuthDatabase::new(&config)
    };
    let mut processor = match database {
        Ok(database) => DBCmdProc::new(database),
        Err(error) => {
            notices.lock().push_back(AuthDatabaseNotice {
                operation: AuthDatabaseOperation::Initialize,
                error,
            });
            return;
        }
    };

    while !stopped.load(Ordering::Acquire) {
        {
            let config = context.config();
            processor.database.refresh_auth_config(&config);
        }
        let initial_size = context.quest_count();
        for _ in 0..initial_size {
            let Some(quest) = context.pop_quest_until_stopped(&stopped) else {
                break;
            };
            processor.process_quest(&context, quest);
            while let Some(notice) = processor.pop_notice() {
                notices.lock().push_back(notice);
            }
        }
        thread::sleep(DB_WORKER_DELAY);
    }
}

fn build_auth_result(request: AuthQuestData, database_result: i32) -> AuthResultData {
    AuthResultData::new(
        map_auth_result(database_result),
        request.account,
        request.client_ip,
        request.client_socket_id,
    )
}

fn build_auth_ex_result(
    request: AuthQuestData,
    database_result: AuthExtendedDatabaseResult,
) -> AuthExResultData {
    let mut result = AuthExResultData::new(
        map_auth_ex_result(database_result.result),
        request.account,
        request.client_ip,
        request.client_socket_id,
    );
    if let Some(assure) = database_result.assure {
        result.extra = assure;
    } else if let Some(suspended) = database_result.suspended {
        write_system_time(&mut result.extra, &suspended);
    }
    result
}

fn map_auth_result(result: i32) -> i32 {
    match result {
        0 => 0,
        -10 => 12,
        -9 => 11,
        -8 => 10,
        -7 => 5,
        -6 => 8,
        -5 => 9,
        -4 => 6,
        -3 => 3,
        -1 => 2,
        _ => 1,
    }
}

fn map_auth_ex_result(result: i32) -> i32 {
    match result {
        0 => 0,
        1 => 7,
        -7 => 5,
        -3 => 3,
        -1 => 2,
        _ => 1,
    }
}

fn write_system_time(output: &mut [u8; 80], time: &LockUntil) {
    for (offset, value) in [
        (0, time.year),
        (2, time.month),
        (6, time.day),
        (8, time.hour),
        (10, time.minute),
        (12, time.second),
    ] {
        output[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn legacy_ipv4_text(address: u32) -> String {
    Ipv4Addr::from(address.to_le_bytes()).to_string()
}

fn decode_ansi(bytes: &[u8]) -> String {
    let (decoded, _, _) = WINDOWS_1251.decode(bytes);
    decoded.into_owned()
}

fn create_tds_config(host: &[u8], database: &[u8], user: &[u8], password: &[u8]) -> Config {
    let mut config = Config::new();
    config.host(decode_ansi(host));
    config.database(decode_ansi(database));
    config.authentication(AuthMethod::sql_server(
        decode_ansi(user),
        decode_ansi(password),
    ));
    config.encryption(EncryptionLevel::NotSupported);
    config
}

fn quote_procedure(bytes: &[u8]) -> Result<String, AuthDatabaseError> {
    if bytes.is_empty() {
        return Err(AuthDatabaseError::InvalidProcedureName);
    }
    bytes
        .split(|byte| *byte == b'.')
        .map(|part| {
            if part.is_empty()
                || !part
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                return Err(AuthDatabaseError::InvalidProcedureName);
            }
            Ok(format!("[{}]", String::from_utf8_lossy(part)))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("."))
}
