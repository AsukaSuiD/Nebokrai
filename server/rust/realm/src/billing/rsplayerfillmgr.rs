//! Владелец `CRsPlayerFillMgr` BillingServer.
//!
//! Реализованы `InitConn`, `GetPlayerDeleteSQL`, `DeletePlayerFillLog` и
//! `GetPlayerFillLog`.
//!
//! `InitConn` выбирает основную Billing DB, а не отдельную cash-log DB.
//! Каждый последующий вызов открывает собственное соединение. Fetch выполняет
//! точный `select top 50 * from TBL_NeedUpdate order by ID`, сохраняет порядок
//! строк и читает только `Account` и signed Windows `long` `ID`. Delete после
//! обработки строит один исходный `delete ... where id in (...)` для всего
//! snapshot без транзакции и без повторной выборки.
//! При ошибке recordset уже прочитанный prefix остаётся у caller и затем проходит
//! обычные send/delete, как partial mutation исходного caller-owned vector;
//! `futures-util::TryStreamExt` сохраняет это поверх потокового TDS-result.
//!
//! `GetPlayerDeleteSQL` читает signed поле `ID` строки и передаёт его формату
//! `%d`. Поэтому ID не заменён индексом vector и не параметризован другим
//! значением.
//!
//! ADO/COM, `SQLOLEDB`, recordset, BSTR/VARIANT и SEH заменены Tiberius и
//! current-thread Tokio runtime. Windows ANSI-преобразование DB `varchar`
//! сохраняется через Windows-1251, уже выбранную для русского Billing-корпуса.
//! `Vec` и `Drop` выражают `std::vector`, `std::string` и cleanup; все их raw-
//! внутренности, catch/unwind funclets и process-global destructors удалены.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;

use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type TdsClient = Client<Compat<TcpStream>>;

const GET_PLAYER_FILL_SQL: &str = "select top 50 * from TBL_NeedUpdate order by ID";
const PLAYER_ACCOUNT_CAPACITY: usize = 64;

/// Четыре исходных connection-поля основной Billing DB.
#[derive(Clone)]
pub struct PlayerFillDatabaseSettings {
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

/// Владеющие части основной Billing DB без публикации credentials.
pub struct PlayerFillDatabaseSettingsParts {
    pub host: Vec<u8>,
    pub database: Vec<u8>,
    pub user: Vec<u8>,
    pub password: Vec<u8>,
}

impl PlayerFillDatabaseSettings {
    /// Копирует setup snapshot для последующих отдельных соединений.
    pub fn from_parts(parts: PlayerFillDatabaseSettingsParts) -> Self {
        Self {
            host: parts.host,
            database: parts.database,
            user: parts.user,
            password: parts.password,
        }
    }
}

/// Одна строка `TBL_NeedUpdate` в исходном порядке snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerFillInfo {
    pub player_account: Vec<u8>,
    pub id: i32,
}

/// DB-функция, породившая operator-visible ошибку.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RsPlayerFillOperation {
    GetPlayerFillLog,
    DeletePlayerFillLog,
}

/// Структурированная замена старого `AddLogText` без DB/runtime-значений.
#[derive(Debug)]
pub struct RsPlayerFillNotice {
    pub operation: RsPlayerFillOperation,
    pub error: RsPlayerFillDatabaseError,
}

/// Ошибка создания синхронного Linux/TDS-владельца.
#[derive(Debug)]
pub struct RsPlayerFillInitializationError(io::Error);

impl fmt::Display for RsPlayerFillInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не создан синхронный runtime PlayerFill DB: {}",
            self.0
        )
    }
}

impl Error for RsPlayerFillInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Ошибка технической ADO/TDS-границы без credential и account values.
#[derive(Debug)]
pub enum RsPlayerFillDatabaseError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
    MissingRequiredValue(&'static str),
}

impl fmt::Display for RsPlayerFillDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(
                formatter,
                "не установлено соединение PlayerFill Billing DB: {error}"
            ),
            Self::Tds(error) => write!(formatter, "ошибка TDS PlayerFill Billing DB: {error}"),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "PlayerFill Billing DB вернула NULL в {column}")
            }
        }
    }
}

impl Error for RsPlayerFillDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
            Self::MissingRequiredValue(_) => None,
        }
    }
}

impl From<tiberius::error::Error> for RsPlayerFillDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Узкая объектная граница трёх достигнутых DB-функций после `InitConn`.
pub trait RsPlayerFillOwner {
    fn get_player_fill_log(&mut self) -> Vec<PlayerFillInfo>;

    fn delete_player_fill_log(&mut self, entries: &[PlayerFillInfo]) -> i32;

    fn pop_notice(&mut self) -> Option<RsPlayerFillNotice>;
}

/// Linux/TDS-замена статического исходного `CRsPlayerFillMgr`.
pub struct TiberiusRsPlayerFillMgr {
    config: Config,
    runtime: Runtime,
    notices: VecDeque<RsPlayerFillNotice>,
}

struct PlayerFillFetch {
    entries: Vec<PlayerFillInfo>,
    error: Option<RsPlayerFillDatabaseError>,
}

impl TiberiusRsPlayerFillMgr {
    /// Выполняет техническую часть `InitConn` без раннего сетевого соединения.
    pub fn new(
        settings: PlayerFillDatabaseSettings,
    ) -> Result<Self, RsPlayerFillInitializationError> {
        let runtime = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(RsPlayerFillInitializationError)?;
        Ok(Self {
            config: create_tds_config(settings),
            runtime,
            notices: VecDeque::new(),
        })
    }

    async fn connect(config: Config) -> Result<TdsClient, RsPlayerFillDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(RsPlayerFillDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(RsPlayerFillDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(RsPlayerFillDatabaseError::Tds)
    }

    async fn query_player_fill_log(config: Config) -> PlayerFillFetch {
        let mut entries = Vec::new();
        let mut client = match Self::connect(config).await {
            Ok(client) => client,
            Err(error) => {
                return PlayerFillFetch {
                    entries,
                    error: Some(error),
                };
            }
        };
        let mut stream = match client.query(GET_PLAYER_FILL_SQL, &[]).await {
            Ok(stream) => stream,
            Err(error) => {
                return PlayerFillFetch {
                    entries,
                    error: Some(error.into()),
                };
            }
        };
        loop {
            let item = match stream.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(error) => {
                    return PlayerFillFetch {
                        entries,
                        error: Some(error.into()),
                    };
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let account = match row.try_get::<&str, _>("Account") {
                Ok(Some(account)) => account,
                Ok(None) => {
                    return PlayerFillFetch {
                        entries,
                        error: Some(RsPlayerFillDatabaseError::MissingRequiredValue("Account")),
                    };
                }
                Err(error) => {
                    return PlayerFillFetch {
                        entries,
                        error: Some(error.into()),
                    };
                }
            };
            let id = match row.try_get::<i32, _>("ID") {
                Ok(Some(id)) => id,
                Ok(None) => {
                    return PlayerFillFetch {
                        entries,
                        error: Some(RsPlayerFillDatabaseError::MissingRequiredValue("ID")),
                    };
                }
                Err(error) => {
                    return PlayerFillFetch {
                        entries,
                        error: Some(error.into()),
                    };
                }
            };
            let (account, _, _) = WINDOWS_1251.encode(account);
            let account = account
                .iter()
                .position(|byte| *byte == 0)
                .map_or(account.as_ref(), |end| &account[..end]);

            // `_bstr_t` копировался через `strcpy` в `char[64]`. Rust сохраняет
            // 63-byte payload
            // без stack overflow; oversized DB-значение детерминированно
            // обрезается на внутренней фиксированной границе.
            entries.push(PlayerFillInfo {
                player_account: account[..account.len().min(PLAYER_ACCOUNT_CAPACITY - 1)].to_vec(),
                id,
            });
        }
        PlayerFillFetch {
            entries,
            error: None,
        }
    }

    async fn delete_player_fill_log(
        config: Config,
        entries: &[PlayerFillInfo],
    ) -> Result<(), RsPlayerFillDatabaseError> {
        let sql = player_delete_sql(entries);
        let mut client = Self::connect(config).await?;
        client.execute(sql, &[]).await?;
        Ok(())
    }

    fn push_error(&mut self, operation: RsPlayerFillOperation, error: RsPlayerFillDatabaseError) {
        self.notices
            .push_back(RsPlayerFillNotice { operation, error });
    }
}

impl RsPlayerFillOwner for TiberiusRsPlayerFillMgr {
    fn get_player_fill_log(&mut self) -> Vec<PlayerFillInfo> {
        let future = Self::query_player_fill_log(self.config.clone());
        let fetch = self.runtime.block_on(future);
        if let Some(error) = fetch.error {
            self.push_error(RsPlayerFillOperation::GetPlayerFillLog, error);
        }
        fetch.entries
    }

    fn delete_player_fill_log(&mut self, entries: &[PlayerFillInfo]) -> i32 {
        if entries.is_empty() {
            return 0;
        }
        let future = Self::delete_player_fill_log(self.config.clone(), entries);
        match self.runtime.block_on(future) {
            Ok(()) => 0,
            Err(error) => {
                self.push_error(RsPlayerFillOperation::DeletePlayerFillLog, error);
                -2
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsPlayerFillNotice> {
        self.notices.pop_front()
    }
}

fn player_delete_sql(entries: &[PlayerFillInfo]) -> String {
    let mut ids = String::new();
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            ids.push(',');
        }
        ids.push_str(&entry.id.to_string());
    }
    format!("delete from TBL_NeedUpdate where id in ({ids})")
}

fn decode_ansi_c_string(bytes: &[u8]) -> String {
    let visible = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end]);
    let (decoded, _, _) = WINDOWS_1251.decode(visible);
    decoded.into_owned()
}

fn create_tds_config(settings: PlayerFillDatabaseSettings) -> Config {
    let mut config = Config::new();
    config.host(decode_ansi_c_string(&settings.host));
    config.database(decode_ansi_c_string(&settings.database));
    config.authentication(AuthMethod::sql_server(
        decode_ansi_c_string(&settings.user),
        decode_ansi_c_string(&settings.password),
    ));
    config.encryption(EncryptionLevel::NotSupported);
    config
}
