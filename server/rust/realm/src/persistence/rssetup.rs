//! Инициализация World DB из точной пары `worldserver.exe` и `worldserver.pdb`, перенесённая в Realm `persistence/`.
//!
//! Owner открывает настроенные TDS-соединения и возвращает typed runtime
//! handles. Tiberius заменяет ADO/COM; порядок открытия, обязательность баз и
//! значения конфигурации остаются у `CGame::Init` без скрытых retry/rollback.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;

use encoding_rs::WINDOWS_1251;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub type WorldTdsClient = Client<Compat<TcpStream>>;

const LOAD_PLAYER_ID_SQL: &str = "SELECT TOP 1 playerID FROM csl_setup";
const LOAD_LEAVE_WORLD_ID_SQL: &str = "SELECT TOP 1 LeaveWordID FROM csl_setup";
const SAVE_PLAYER_ID_SQL: &str = "UPDATE csl_setup SET playerID=@P1";
const SAVE_LEAVE_WORLD_ID_SQL: &str = "UPDATE csl_setup SET LeaveWordID=@P1";

#[derive(Clone)]
pub struct WorldDatabaseSettings {
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

pub struct WorldDatabaseSettingsParts {
    pub host: Vec<u8>,
    pub database: Vec<u8>,
    pub user: Vec<u8>,
    pub password: Vec<u8>,
}

impl WorldDatabaseSettings {
    pub fn from_parts(parts: WorldDatabaseSettingsParts) -> Self {
        Self {
            host: parts.host,
            database: parts.database,
            user: parts.user,
            password: parts.password,
        }
    }

    pub fn tds_config(&self) -> Config {
        let mut config = Config::new();
        config.host(decode_ansi_c_string(&self.host));
        config.database(decode_ansi_c_string(&self.database));
        config.authentication(AuthMethod::sql_server(
            decode_ansi_c_string(&self.user),
            decode_ansi_c_string(&self.password),
        ));
        config.encryption(EncryptionLevel::NotSupported);
        config
    }

    pub async fn connect(&self) -> Result<WorldTdsClient, WorldDatabaseConnectionError> {
        let config = self.tds_config();
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(WorldDatabaseConnectionError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(WorldDatabaseConnectionError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(WorldDatabaseConnectionError::Tds)
    }
}

#[derive(Debug)]
pub enum WorldDatabaseConnectionError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for WorldDatabaseConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(formatter, "не установлено соединение World DB: {error}")
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS World DB: {error}"),
        }
    }
}

impl Error for WorldDatabaseConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LoadedSetupIds {
    pub player_id: u32,
    pub leave_world_id: i32,
}

#[derive(Debug)]
pub enum RsSetupNotice {
    PlayerLoad {
        error: RsSetupDatabaseError,
    },
    LeaveWorldLoad {
        error: RsSetupDatabaseError,
    },
    PlayerSave {
        player_id: i32,
        error: RsSetupDatabaseError,
    },
    LeaveWorldSave {
        leave_world_id: i32,
        error: RsSetupDatabaseError,
    },
}

#[derive(Debug)]
pub enum RsSetupDatabaseError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
    MissingRequiredValue(&'static str),
}

impl fmt::Display for RsSetupDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(
                    formatter,
                    "не установлено соединение World Setup DB: {error}"
                )
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS World Setup DB: {error}"),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "World Setup DB вернула NULL в {column}")
            }
        }
    }
}

impl Error for RsSetupDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
            Self::MissingRequiredValue(_) => None,
        }
    }
}

impl From<tiberius::error::Error> for RsSetupDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

pub trait RsSetupOwner {
    fn save_player_id(
        &mut self,
        active_transaction: &mut WorldTdsClient,
        player_id_snapshot: u32,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_leave_world_id(
        &mut self,
        active_transaction: &mut WorldTdsClient,
        leave_world_id_snapshot: i32,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<RsSetupNotice>;
}

pub struct TiberiusRsSetup {
    config: Config,
    notices: VecDeque<RsSetupNotice>,
}

impl TiberiusRsSetup {
    pub fn new_for_save(settings: WorldDatabaseSettings) -> Self {
        Self {
            config: settings.tds_config(),
            notices: VecDeque::new(),
        }
    }

    pub async fn initialize(settings: WorldDatabaseSettings) -> (Self, LoadedSetupIds) {
        let mut owner = Self {
            config: settings.tds_config(),
            notices: VecDeque::new(),
        };
        let player_id = owner.load_player_id().await;
        let leave_world_id = owner.load_leave_world_id().await;
        (
            owner,
            LoadedSetupIds {
                player_id,
                leave_world_id,
            },
        )
    }

    async fn connect(config: Config) -> Result<WorldTdsClient, RsSetupDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(RsSetupDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(RsSetupDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(RsSetupDatabaseError::Tds)
    }

    async fn query_player_id(config: Config) -> Result<u32, RsSetupDatabaseError> {
        let mut client = Self::connect(config).await?;
        let Some(row) = client
            .query(LOAD_PLAYER_ID_SQL, &[])
            .await?
            .into_row()
            .await?
        else {
            return Ok(0);
        };
        let player_id = row
            .try_get::<i32, _>("playerID")?
            .ok_or(RsSetupDatabaseError::MissingRequiredValue("playerID"))?;
        Ok(player_id as u32)
    }

    async fn query_leave_world_id(config: Config) -> Result<i32, RsSetupDatabaseError> {
        let mut client = Self::connect(config).await?;
        let Some(row) = client
            .query(LOAD_LEAVE_WORLD_ID_SQL, &[])
            .await?
            .into_row()
            .await?
        else {
            return Ok(0);
        };
        row.try_get::<i32, _>("LeaveWordID")?
            .ok_or(RsSetupDatabaseError::MissingRequiredValue("LeaveWordID"))
    }
    async fn load_player_id(&mut self) -> u32 {
        match Self::query_player_id(self.config.clone()).await {
            Ok(player_id) => player_id,
            Err(error) => {
                self.notices.push_back(RsSetupNotice::PlayerLoad { error });
                0
            }
        }
    }

    async fn load_leave_world_id(&mut self) -> i32 {
        match Self::query_leave_world_id(self.config.clone()).await {
            Ok(leave_world_id) => leave_world_id,
            Err(error) => {
                self.notices
                    .push_back(RsSetupNotice::LeaveWorldLoad { error });
                0
            }
        }
    }
}

impl RsSetupOwner for TiberiusRsSetup {
    fn save_player_id(
        &mut self,
        active_transaction: &mut WorldTdsClient,
        player_id_snapshot: u32,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            let player_id = player_id_snapshot as i32;
            match active_transaction
                .execute(SAVE_PLAYER_ID_SQL, &[&player_id])
                .await
            {
                Ok(_) => true,
                Err(error) => {
                    self.notices.push_back(RsSetupNotice::PlayerSave {
                        player_id,
                        error: error.into(),
                    });
                    false
                }
            }
        }
    }

    fn save_leave_world_id(
        &mut self,
        active_transaction: &mut WorldTdsClient,
        leave_world_id_snapshot: i32,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            match active_transaction
                .execute(SAVE_LEAVE_WORLD_ID_SQL, &[&leave_world_id_snapshot])
                .await
            {
                Ok(_) => true,
                Err(error) => {
                    self.notices.push_back(RsSetupNotice::LeaveWorldSave {
                        leave_world_id: leave_world_id_snapshot,
                        error: error.into(),
                    });
                    false
                }
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsSetupNotice> {
        self.notices.pop_front()
    }
}

fn decode_ansi_c_string(bytes: &[u8]) -> String {
    let visible = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end]);
    let (decoded, _, _) = WINDOWS_1251.decode(visible);
    decoded.into_owned()
}
