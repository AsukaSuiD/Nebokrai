//! DB-владелец `CRsCDKey` LoginServer из `rscdkey.cpp`.
//!
//! Owner реализует `CDKeyBan`, IP-фильтры, matrix-card операции, `GetBanTime`,
//! `FixPtAcc`, `ValidateLocalPassord` и GAS-процедуру `getAccInfoEx`.
//! Контракты подтверждены точной парой LoginServer EXE/PDB.
//!
//! Каждая операция по-прежнему открывает отдельное соединение. `CDKeyBan`
//! сохраняет нетранзакционный `SELECT WITH(NOLOCK) -> UPDATE/INSERT`;
//! `matrix_validate` читает три позиции действующей matrix-card, а
//! `matrix_used` проверяет только наличие такой строки. `GetBanTime` возвращает
//! nullable Rust datetime вместо промежуточного OLE Automation `double`:
//! исходный потребитель сразу превращал его обратно в календарные поля и
//! считал ноль отсутствием ban.
//!
//! `IPIsAllowed` и `IPIsForbidded` получают исходные setup-флаги явным
//! аргументом вместо чтения глобального `CGame`. Перед сравнением с `bigint`
//! исходный raw WinSock IPv4 разворачивается тем же `bswap`. Allow-ошибка даёт
//! `false`, forbid-ошибка — также `false`, поэтому только allow остаётся
//! fail-closed. `IsBetweenIP` сводит и EOF, и найденный диапазон к `true`:
//! любой успешно
//! прочитанный result set разрешает вход, а `false` возможен только при ADO-
//! ошибке. Этот наблюдаемый дефект сохранён без придуманного deny-результата.
//!
//! `FixPtAcc` заменяет числовой `originsdid` найденным `userid`, иначе оставляет
//! вход неизменным. Единственный достигнутый caller заранее доказывает ASCII-
//! цифры; Rust повторяет проверку перед построением исходного некавыченного
//! числового литерала. Это сохраняет SQL type precedence даже при неизвестном
//! типе `userinfo.originsdid` и не возвращает старую injection-границу.
//! `ValidateLocalPassord` сам выбирает `originsdid` для полностью числового
//! account либо `userid`, сравнивает `CAST(passwd AS VARCHAR(50))` без учёта
//! ASCII-регистра и при успехе возвращает канонический `userid`. `nullptr` и
//! выходной `char*` заменены slice/owned `Option<Vec<u8>>`.
//! `getAccInfoEx` сохраняет отдельное соединение, три входных `varchar(200)`
//! (`@UserID`, `@UserIP`, `@UserPwd`) и output `int @Result`; output не меняет
//! доказанный безусловный `false` вызывающего `CGame::ExecuteProce`.
//!
//! ADO/COM, `_Connection`, `_Recordset`, `VARIANT`/`SAFEARRAY`, `inet_addr` и
//! compiler cleanup заменены `tiberius`, Tokio TCP и владеющими Rust-
//! значениями. В этом владельце SQL параметризован везде, кроме доказанно
//! цифрового литерала `FixPtAcc`; Windows-1251 обеспечивает старую ANSI-
//! границу. Полный найденный
//! `LoginDB.bak` подтверждает `csl_cdkey`, `ip_allow`, `ip_forbid`, `ip_list` и
//! их типы, но ни он, ни `Account.bak` не содержат `userinfo`. Поэтому контракт
//! двух account-функций подтверждён EXE/PDB, но их DB-schema неизвестна:
//! нельзя утверждать длину/тип `userid`, `originsdid` и `passwd` либо создавать
//! собственную таблицу.
//!
//! Во всех четырёх backup-наборах `LoginDB.bak` строки `matrix_card image`
//! равны `NULL`; `sp_bindCdkey` принимает blob любой ненулевой длины. Оригинал
//! не проверял индекс перед чтением `SAFEARRAY`. Для blob, покрывающего три
//! позиции, сохраняется исходный `bool`; выход за длину остаётся явно
//! неразрешённой границей без `unsafe` и без придуманного `false`. Конструктор,
//! деструктор, ADO wrappers, STL/COM internals и EH cleanup удалены как
//! доказанный compiler/library noise.
//! Техническая функция `connect_login_database` переиспользуется соседним
//! `game.cpp::UpdateOnlineUser2DB`; её SQL и порядок остаются в том owner-файле.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;

use chrono::NaiveDateTime;
use encoding_rs::WINDOWS_1251;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::runtime::{Handle, TryCurrentError};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub(crate) type TdsClient = Client<Compat<TcpStream>>;

/// Четыре используемых SQL connection-поля LoginServer без публикации credentials.
#[derive(Clone)]
pub(crate) struct LoginDatabaseSettings {
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

/// Открывает отдельное Login DB соединение для соседнего исходного DB-owner.
pub(crate) async fn connect_login_database(
    settings: LoginDatabaseSettings,
) -> Result<TdsClient, RsCdKeyDatabaseError> {
    TiberiusRsCdKey::connect(create_tds_config(settings)).await
}

impl LoginDatabaseSettings {
    /// Копирует byte-exact значения `SqlServerIP/DBName/SqlUserName/SqlPassWord`.
    pub(crate) fn new(host: Vec<u8>, database: Vec<u8>, user: Vec<u8>, password: Vec<u8>) -> Self {
        Self {
            host,
            database,
            user,
            password,
        }
    }
}

/// Операция исходного `CRsCDKey`, завершившаяся DB-ошибкой.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RsCdKeyOperation {
    CdKeyBan,
    FixPtAccount,
    GetBanTime,
    ListActiveBans,
    IpIsAllowed,
    IpIsForbidden,
    IsBetweenIp,
    MatrixUsed,
    MatrixValidate,
    ValidateLocalPassword,
    GetAccInfoEx,
}

/// Структурированная замена `PrintErr` и успешного `banplayer` file-log.
#[derive(Debug)]
pub(crate) enum RsCdKeyNotice {
    DatabaseFailure {
        operation: RsCdKeyOperation,
        error: RsCdKeyDatabaseError,
    },
    BanApplied {
        account: Vec<u8>,
        minutes: i32,
        inserted: bool,
    },
}

/// Ошибка создания синхронного Linux/TDS-владельца.
#[derive(Debug)]
pub(crate) struct RsCdKeyInitializationError(TryCurrentError);

impl fmt::Display for RsCdKeyInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не найден runtime Login DB: {}",
            self.0
        )
    }
}

impl Error for RsCdKeyInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Ошибка доказанного ADO/TDS-пути без credential values.
#[derive(Debug)]
pub(crate) enum RsCdKeyDatabaseError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
    MissingRequiredValue(&'static str),
}

impl fmt::Display for RsCdKeyDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(formatter, "не установлено соединение Login DB: {error}")
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS Login DB: {error}"),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "обязательное поле Login DB равно NULL: {column}")
            }
        }
    }
}

impl Error for RsCdKeyDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
            Self::MissingRequiredValue(_) => None,
        }
    }
}

impl From<tiberius::error::Error> for RsCdKeyDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Результат исходного `bool` либо одна безопасно неразрешимая malformed-граница.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MatrixValidation {
    Compared(bool),
    BlockedMatrixCardTooShort {
        actual_len: usize,
        required_len: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveBanRecord {
    pub(crate) account: Vec<u8>,
    pub(crate) ban_until: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveBanList {
    pub(crate) total_count: i32,
    pub(crate) records: Vec<ActiveBanRecord>,
}

/// Узкая объектная граница достигнутых операций исходного `CRsCDKey`.
pub(crate) trait RsCdKeyOwner {
    /// Пытается заблокировать byte-exact account на исходное число минут.
    fn cd_key_ban(&mut self, account: &[u8], minutes: i32) -> bool;

    /// Оставляет account прежним либо возвращает `userid` для цифрового `originsdid`.
    fn fix_pt_account(&mut self, account: &[u8]) -> Vec<u8>;

    /// Возвращает ненулевой исходный `ban_time`; отсутствие/DB-ошибка дают `None`.
    fn get_ban_time(&mut self, account: &[u8]) -> Option<NaiveDateTime>;

    /// Возвращает до 256 действующих блокировок и полный размер выборки.
    fn list_active_bans(&mut self) -> Option<ActiveBanList>;

    /// Проверяет allow-диапазоны, если исходный setup-флаг включён.
    fn ip_is_allowed(&mut self, check_enabled: bool, raw_ipv4: u32) -> bool;

    /// Проверяет forbid-диапазоны, если исходный setup-флаг включён.
    fn ip_is_forbidden(&mut self, check_enabled: bool, raw_ipv4: u32) -> bool;

    /// Сохраняет фактический account/IP-list контракт и его исходный no-op дефект.
    fn is_between_ip(&mut self, check_enabled: bool, account: &[u8], raw_ipv4: u32) -> bool;

    /// Сообщает наличие действующей ненулевой matrix-card для account.
    fn matrix_used(&mut self, account: &[u8]) -> bool;

    /// Сравнивает три client-байта с действующей matrix-card по трём позициям.
    fn matrix_validate(
        &mut self,
        account: &[u8],
        positions: &[u8; 3],
        answer: &[u8; 3],
    ) -> MatrixValidation;

    /// При совпадении локального пароля возвращает канонический `userid`.
    fn validate_local_password(&mut self, account: &[u8], password: &[u8]) -> Option<Vec<u8>>;

    /// Вызывает исходную GAS-процедуру `getAccInfoEx` с output `@Result`.
    fn execute_get_acc_info_ex(
        &mut self,
        user_id: &[u8],
        user_ip: &[u8],
        user_password: &[u8],
    ) -> bool;

    /// Забирает следующее операторское событие в порядке синхронных вызовов.
    fn pop_notice(&mut self) -> Option<RsCdKeyNotice>;
}

/// Linux/TDS-замена одного исходного `CRsCDKey` LoginServer.
pub(crate) struct TiberiusRsCdKey {
    config: Config,
    runtime: Handle,
    notices: VecDeque<RsCdKeyNotice>,
}

impl TiberiusRsCdKey {
    /// Создаёт owner без соединения; как ADO, соединяется внутри каждой операции.
    pub(crate) fn new(settings: LoginDatabaseSettings) -> Result<Self, RsCdKeyInitializationError> {
        let runtime = Handle::try_current().map_err(RsCdKeyInitializationError)?;
        Ok(Self {
            config: create_tds_config(settings),
            runtime,
            notices: VecDeque::new(),
        })
    }

    /// Сохраняет синхронную owner-границу поверх process-level Tokio runtime.
    fn block_on_database<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        tokio::task::block_in_place(|| self.runtime.block_on(future))
    }

    async fn connect(config: Config) -> Result<TdsClient, RsCdKeyDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(RsCdKeyDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(RsCdKeyDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(RsCdKeyDatabaseError::Tds)
    }

    async fn apply_ban(
        config: Config,
        account: String,
        minutes: i32,
    ) -> Result<bool, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let exists = client
            .query(
                "SELECT cdkey FROM csl_cdkey WITH(NOLOCK) WHERE cdkey = @P1",
                &[&account],
            )
            .await?
            .into_row()
            .await?
            .is_some();

        if exists {
            client
                .execute(
                    "UPDATE CSL_CDKEY SET ban_time = DATEADD(minute, @P2, GETDATE()) \
                     WHERE cdkey = @P1",
                    &[&account, &minutes],
                )
                .await?;
        } else {
            client
                .execute(
                    "INSERT INTO csl_cdkey(cdkey, password, ban_time) \
                     VALUES(@P1, '', DATEADD(minute, @P2, GETDATE()))",
                    &[&account, &minutes],
                )
                .await?;
        }
        Ok(!exists)
    }

    async fn read_fixed_account(
        config: Config,
        numeric_account: String,
    ) -> Result<Option<Vec<u8>>, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = format!("SELECT userid FROM userinfo WHERE originsdid={numeric_account}");
        let Some(row) = client.query(sql, &[]).await?.into_row().await? else {
            return Ok(None);
        };
        let user_id = row
            .get::<&str, _>(0)
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("userid"))?;
        Ok(Some(encode_ansi(user_id)))
    }

    async fn read_ban_time(
        config: Config,
        account: String,
    ) -> Result<Option<NaiveDateTime>, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let Some(row) = client
            .query(
                "SELECT cdkey, ban_time FROM CSL_CDKEY \
                 WHERE cdkey = @P1 AND ban_time IS NOT NULL",
                &[&account],
            )
            .await?
            .into_row()
            .await?
        else {
            return Ok(None);
        };
        row.get::<NaiveDateTime, _>(1)
            .map(Some)
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("ban_time"))
    }

    async fn read_active_bans(config: Config) -> Result<ActiveBanList, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let stream = client
            .query(
                "SELECT TOP (256) cdkey, \
                 CONVERT(varchar(19), ban_time, 120) AS ban_until, \
                 COUNT(*) OVER() AS total_count \
                 FROM dbo.CSL_CDKEY \
                 WHERE ban_time IS NOT NULL AND ban_time > GETDATE() \
                 ORDER BY ban_time ASC, cdkey ASC",
                &[],
            )
            .await?;
        let rows = stream.into_first_result().await?;
        let mut total_count = 0_i32;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let account = row
                .get::<&str, _>(0)
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("cdkey"))?;
            let ban_until = row
                .get::<&str, _>(1)
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("ban_until"))?;
            let row_total = row
                .get::<i32, _>(2)
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("total_count"))?;
            let account = encode_ansi(account);
            let ban_until = ban_until.as_bytes().to_vec();
            if account.is_empty()
                || account.len() > 32
                || ban_until.len() != 19
                || row_total < records.len() as i32 + 1
                || (total_count != 0 && total_count != row_total)
            {
                return Err(RsCdKeyDatabaseError::MissingRequiredValue(
                    "контракт действующих блокировок",
                ));
            }
            total_count = row_total;
            records.push(ActiveBanRecord { account, ban_until });
        }
        Ok(ActiveBanList {
            total_count,
            records,
        })
    }

    async fn execute_gas_procedure(
        config: Config,
        user_id: String,
        user_ip: String,
        user_password: String,
    ) -> Result<(), RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        client
            .execute(
                "DECLARE @UserID varchar(200) = CONVERT(varchar(200), @P1); \
                 DECLARE @UserIP varchar(200) = CONVERT(varchar(200), @P2); \
                 DECLARE @UserPwd varchar(200) = CONVERT(varchar(200), @P3); \
                 DECLARE @Result int; \
                 EXEC getAccInfoEx @UserID = @UserID, @UserIP = @UserIP, \
                     @UserPwd = @UserPwd, @Result = @Result OUTPUT",
                &[&user_id, &user_ip, &user_password],
            )
            .await?;
        Ok(())
    }

    async fn count_ip_ranges(
        config: Config,
        sql: &'static str,
        raw_ipv4: u32,
    ) -> Result<i32, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let network_order = i64::from(raw_ipv4.swap_bytes());
        let row = client
            .query(sql, &[&network_order])
            .await?
            .into_row()
            .await?
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("count(*)"))?;
        row.get::<i32, _>(0)
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("count(*)"))
    }

    async fn read_ip_list(
        config: Config,
        account: String,
        _raw_ipv4: u32,
    ) -> Result<(), RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let results = client
            .query("SELECT * FROM ip_list WHERE cdkey = @P1", &[&account])
            .await?
            .into_results()
            .await?;
        for row in results.into_iter().flatten() {
            let _ip_begin = row
                .get::<&str, _>("ip_begin")
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("ip_begin"))?;
            let _ip_end = row
                .get::<&str, _>("ip_end")
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("ip_end"))?;
        }
        Ok(())
    }

    async fn has_current_matrix(
        config: Config,
        account: String,
    ) -> Result<bool, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        Ok(client
            .query(
                "SELECT matrix_card FROM csl_cdkey \
                 WHERE cdkey = @P1 AND matrix_date > GETDATE() \
                 AND matrix_card IS NOT NULL",
                &[&account],
            )
            .await?
            .into_row()
            .await?
            .is_some())
    }

    async fn read_matrix_card(
        config: Config,
        account: String,
    ) -> Result<Option<Vec<u8>>, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let Some(row) = client
            .query(
                "SELECT matrix_card FROM csl_cdkey \
                 WHERE cdkey = @P1 AND matrix_date > GETDATE() \
                 AND matrix_card IS NOT NULL",
                &[&account],
            )
            .await?
            .into_row()
            .await?
        else {
            return Ok(None);
        };
        row.get::<&[u8], _>(0)
            .map(|bytes| Some(bytes.to_vec()))
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("matrix_card"))
    }

    async fn validate_password(
        config: Config,
        account: String,
        original_account: Vec<u8>,
        numeric_account: bool,
        expected_password: String,
    ) -> Result<Option<Vec<u8>>, RsCdKeyDatabaseError> {
        let mut client = Self::connect(config).await?;
        let sql = if numeric_account {
            "SELECT CAST(passwd AS VARCHAR(50)) AS pwd, userid, originsdid, passwd \
             FROM userinfo WITH(NOLOCK) WHERE originsdid = @P1"
        } else {
            "SELECT CAST(passwd AS VARCHAR(50)) AS pwd, userid, passwd \
             FROM userinfo WITH(NOLOCK) WHERE userid = @P1"
        };
        let Some(row) = client.query(sql, &[&account]).await?.into_row().await? else {
            return Ok(None);
        };
        let stored_password = row
            .get::<&str, _>(0)
            .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("pwd"))?;
        if !stored_password.eq_ignore_ascii_case(&expected_password) {
            return Ok(None);
        }
        if numeric_account {
            let user_id = row
                .get::<&str, _>(1)
                .ok_or(RsCdKeyDatabaseError::MissingRequiredValue("userid"))?;
            Ok(Some(encode_ansi(user_id)))
        } else {
            Ok(Some(original_account))
        }
    }

    fn push_database_failure(&mut self, operation: RsCdKeyOperation, error: RsCdKeyDatabaseError) {
        self.notices
            .push_back(RsCdKeyNotice::DatabaseFailure { operation, error });
    }
}

impl RsCdKeyOwner for TiberiusRsCdKey {
    fn cd_key_ban(&mut self, account: &[u8], minutes: i32) -> bool {
        if minutes == 0 {
            return false;
        }
        let account_text = decode_ansi(account);
        match self
            .block_on_database(Self::apply_ban(self.config.clone(), account_text, minutes))
        {
            Ok(inserted) => {
                self.notices.push_back(RsCdKeyNotice::BanApplied {
                    account: account.to_vec(),
                    minutes,
                    inserted,
                });
                true
            }
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::CdKeyBan, error);
                false
            }
        }
    }

    fn fix_pt_account(&mut self, account: &[u8]) -> Vec<u8> {
        if !is_numeric_account(account) {
            return account.to_vec();
        }
        let account_text = decode_ansi(account);
        match self
            .block_on_database(Self::read_fixed_account(self.config.clone(), account_text))
        {
            Ok(Some(user_id)) => user_id,
            Ok(None) => account.to_vec(),
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::FixPtAccount, error);
                account.to_vec()
            }
        }
    }

    fn get_ban_time(&mut self, account: &[u8]) -> Option<NaiveDateTime> {
        let account = decode_ansi(account);
        match self
            .block_on_database(Self::read_ban_time(self.config.clone(), account))
        {
            Ok(ban_time) => ban_time,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::GetBanTime, error);
                None
            }
        }
    }

    fn list_active_bans(&mut self) -> Option<ActiveBanList> {
        match self
            .block_on_database(Self::read_active_bans(self.config.clone()))
        {
            Ok(result) => Some(result),
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::ListActiveBans, error);
                None
            }
        }
    }

    fn ip_is_allowed(&mut self, check_enabled: bool, raw_ipv4: u32) -> bool {
        if !check_enabled {
            return true;
        }
        let result = self.block_on_database(Self::count_ip_ranges(
            self.config.clone(),
            "SELECT count(*) AS exp1 FROM ip_allow \
             WHERE @P1 >= int_begin AND @P1 <= int_end",
            raw_ipv4,
        ));
        match result {
            Ok(count) => count > 0,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::IpIsAllowed, error);
                false
            }
        }
    }

    fn ip_is_forbidden(&mut self, check_enabled: bool, raw_ipv4: u32) -> bool {
        if !check_enabled {
            return false;
        }
        let result = self.block_on_database(Self::count_ip_ranges(
            self.config.clone(),
            "SELECT count(*) AS exp1 FROM ip_forbid \
             WHERE @P1 >= int_begin AND @P1 <= int_end",
            raw_ipv4,
        ));
        match result {
            Ok(count) => count > 0,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::IpIsForbidden, error);
                false
            }
        }
    }

    fn is_between_ip(&mut self, check_enabled: bool, account: &[u8], raw_ipv4: u32) -> bool {
        if !check_enabled {
            return true;
        }
        let account = decode_ansi(account);
        match self
            .block_on_database(Self::read_ip_list(self.config.clone(), account, raw_ipv4))
        {
            // И EOF, и найденный диапазон возвращают true; false остаётся
            // только результатом ошибки запроса.
            Ok(()) => true,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::IsBetweenIp, error);
                false
            }
        }
    }

    fn matrix_used(&mut self, account: &[u8]) -> bool {
        let account = decode_ansi(account);
        match self
            .block_on_database(Self::has_current_matrix(self.config.clone(), account))
        {
            Ok(used) => used,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::MatrixUsed, error);
                false
            }
        }
    }

    fn matrix_validate(
        &mut self,
        account: &[u8],
        positions: &[u8; 3],
        answer: &[u8; 3],
    ) -> MatrixValidation {
        let account = decode_ansi(account);
        let matrix_card = match self
            .block_on_database(Self::read_matrix_card(self.config.clone(), account))
        {
            Ok(Some(matrix_card)) => matrix_card,
            Ok(None) => return MatrixValidation::Compared(false),
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::MatrixValidate, error);
                return MatrixValidation::Compared(false);
            }
        };

        let required_len = positions
            .iter()
            .copied()
            .max()
            .map_or(0, |position| usize::from(position) + 1);
        if matrix_card.len() < required_len {
            // Оригинал индексировал SAFEARRAY без проверки. `LoginDB.bak` не
            // задаёт minimum length у image,
            // поэтому реакция выхода позиции за blob не доказана.
            return MatrixValidation::BlockedMatrixCardTooShort {
                actual_len: matrix_card.len(),
                required_len,
            };
        }
        let expected = positions.map(|position| matrix_card[usize::from(position)]);
        MatrixValidation::Compared(expected == *answer)
    }

    fn validate_local_password(&mut self, account: &[u8], password: &[u8]) -> Option<Vec<u8>> {
        let original_account = account.to_vec();
        let account_text = decode_ansi(account);
        let password_text = decode_ansi(password);
        let numeric_account = is_numeric_account(account);
        match self.block_on_database(Self::validate_password(
            self.config.clone(),
            account_text,
            original_account,
            numeric_account,
            password_text,
        )) {
            Ok(account) => account,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::ValidateLocalPassword, error);
                None
            }
        }
    }

    fn execute_get_acc_info_ex(
        &mut self,
        user_id: &[u8],
        user_ip: &[u8],
        user_password: &[u8],
    ) -> bool {
        let result = self.block_on_database(Self::execute_gas_procedure(
            self.config.clone(),
            decode_ansi(user_id),
            decode_ansi(user_ip),
            decode_ansi(user_password),
        ));
        match result {
            Ok(()) => true,
            Err(error) => {
                self.push_database_failure(RsCdKeyOperation::GetAccInfoEx, error);
                false
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsCdKeyNotice> {
        self.notices.pop_front()
    }
}

fn is_numeric_account(account: &[u8]) -> bool {
    account.iter().all(|byte| byte.is_ascii_digit())
}

fn decode_ansi(bytes: &[u8]) -> String {
    let (decoded, _, _) = WINDOWS_1251.decode(bytes);
    decoded.into_owned()
}

fn encode_ansi(text: &str) -> Vec<u8> {
    let (encoded, _, _) = WINDOWS_1251.encode(text);
    encoded.into_owned()
}

fn create_tds_config(settings: LoginDatabaseSettings) -> Config {
    let mut config = Config::new();
    config.host(decode_ansi(&settings.host));
    config.database(decode_ansi(&settings.database));
    config.authentication(AuthMethod::sql_server(
        decode_ansi(&settings.user),
        decode_ansi(&settings.password),
    ));
    config.encryption(EncryptionLevel::NotSupported);
    config
}
