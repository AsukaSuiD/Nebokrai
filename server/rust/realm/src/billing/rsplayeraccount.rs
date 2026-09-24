//! DB-владелец `CRsPlayerAccount` BillingServer из `rsplayeraccount.cpp`.
//! Владелец той же семьи перенесён в Realm `billing/`.
//!
//! Реализованы операции `GetUserPoint`, `PutCashLog`, `BuyPlayerItem` и
//! `BuyItemCode`. Выходной `@TranCode` остаётся `adVarChar(500)`, а принимающий
//! буфер сохраняет исходную ёмкость 512 байт. Контракт подтверждён точной
//! парой BillingServer EXE/PDB.
//!
//! `tiberius` заменяет ADO/COM, но не SQL владельца. Каждая публичная операция
//! по-прежнему открывает отдельное соединение; `PutCashLog` открывает одно
//! соединение на весь переданный snapshot и последовательно переиспользует
//! одну procedure-форму. Выполняются точные имена `GetUserPoint`,
//! `PutCashLog`, `buyPlayerItem` и `buyItemCode` с исходными именованными
//! параметрами. Внешняя транзакция не добавлена: старый ADO-код не вызывал
//! `BeginTrans/CommitTrans/RollbackTrans`, поэтому транзакционные границы
//! остаются внутри найденных baseline-процедур MSSQL.
//!
//! Входные `std::string` были ANSI `char*`, затем ADO преобразовывал BSTR в
//! `adVarChar`. Windows-1251 декодирует ту же русскую ANSI-границу, а SQL
//! явно приводит bind values к `varchar(200)`; для доказанного `@UserID`
//! `buyItemCode` сохранён предел 32. `dwYuanbao` сначала был `VT_UI4`, но
//! параметр имел тип signed `adInteger`: передача через `bigint -> int`
//! сохраняет DB-ошибку при значении выше `INT_MAX`, а не меняет bit pattern.
//! OLE Automation `double` cash-log времени переводится от эпохи
//! `1899-12-30` в штатный TDS `datetime`; ручной MSSQL wire не создаётся.
//!
//! `GetUserPoint` и `BuyPlayerItem` сохраняют DB-fallback `-2`,
//! `BuyItemCode` — `-5`; их output-поля при ошибке остаются нулевыми/пустыми,
//! как заранее очищенные caller-буферы. `PutCashLog` всегда возвращал `0`,
//! даже после ADO-исключения; ошибка останавливает уже вынутый snapshot и
//! теряет его хвост. Rust возвращает тот же код, а техническую ошибку оставляет
//! в отдельной FIFO notices без credential или account values.
//!
//! `GetUserPoint`, `BuyPlayerItem` и `BuyItemCode` используют основную
//! четвёрку setup, тогда как `PutCashLog` использует отдельные поля
//! LogServer/LogDB. Поэтому owner хранит две TDS-конфигурации; смешивать
//! cash-log с Billing DB нельзя.
//! Преобразования входов выполняются в локальных SQL-переменных до EXEC:
//! аргумент процедуры в T-SQL не допускает выражение CONVERT непосредственно.
//!
//! После успешного `buyItemCode` исходная функция при включённом LogServer
//! снимала local time и ставила `tagIncLogNode` в общую очередь. Глобальный
//! `GetGame` заменён явным atomic-флагом `log_server_enabled`, который читается
//! после успешной процедуры в той же исходной позиции; owned запись
//! возвращается вызывающему manager для постановки в ту же FIFO.
//! ADO wrappers, COM smart pointers, `VARIANT`, BSTR, STL/CRT helpers и все
//! `Catch/FUN` cleanup-блоки удалены как library/compiler noise; их
//! существенные cleanup-эффекты выражены владением и `Drop`.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{Local, NaiveDate, NaiveDateTime, TimeDelta};
use encoding_rs::WINDOWS_1251;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::billingplayermanager::{TagIncLogNode, TagIncLogNodeParts, TagTradeNode};

type TdsClient = Client<Compat<TcpStream>>;

const TRANSACTION_CODE_LIMIT: usize = 500;
const MILLIS_PER_DAY: f64 = 86_400_000.0;

/// Две исходные четвёрки connection-полей без публикации credentials.
#[derive(Clone)]
pub struct BillingDatabaseSettings {
    account: BillingDatabaseConnectionSettings,
    cash_log: BillingDatabaseConnectionSettings,
}

#[derive(Clone)]
struct BillingDatabaseConnectionSettings {
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

/// Обе независимые четвёрки connection-полей одного Billing setup snapshot.
pub struct BillingDatabaseSettingsParts {
    pub account_host: Vec<u8>,
    pub account_database: Vec<u8>,
    pub account_user: Vec<u8>,
    pub account_password: Vec<u8>,
    pub cash_log_host: Vec<u8>,
    pub cash_log_database: Vec<u8>,
    pub cash_log_user: Vec<u8>,
    pub cash_log_password: Vec<u8>,
}

impl BillingDatabaseSettings {
    /// Копирует обе независимые четвёрки connection-полей Billing setup.
    pub fn from_parts(parts: BillingDatabaseSettingsParts) -> Self {
        Self {
            account: BillingDatabaseConnectionSettings {
                host: parts.account_host,
                database: parts.account_database,
                user: parts.account_user,
                password: parts.account_password,
            },
            cash_log: BillingDatabaseConnectionSettings {
                host: parts.cash_log_host,
                database: parts.cash_log_database,
                user: parts.cash_log_user,
                password: parts.cash_log_password,
            },
        }
    }
}

/// Исходная DB-операция, для которой применён её доказанный fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RsPlayerAccountOperation {
    GetUserPoint,
    PutCashLog,
    BuyPlayerItem,
    BuyItemCode,
}

/// Структурированная замена старого `AddLogText` с COM description.
#[derive(Debug)]
pub struct RsPlayerAccountNotice {
    pub operation: RsPlayerAccountOperation,
    pub error: RsPlayerAccountDatabaseError,
}

/// Ошибка создания синхронного Linux/TDS-владельца.
#[derive(Debug)]
pub struct RsPlayerAccountInitializationError(io::Error);

impl fmt::Display for RsPlayerAccountInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не создан синхронный runtime Billing DB: {}",
            self.0
        )
    }
}

impl Error for RsPlayerAccountInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Ошибка технической ADO/TDS-границы без credential или runtime-значений.
#[derive(Debug)]
pub enum RsPlayerAccountDatabaseError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
    MissingOutputRow,
    MissingOutput(&'static str),
    InvalidOleAutomationDate,
}

impl fmt::Display for RsPlayerAccountDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(formatter, "не установлено соединение Billing DB: {error}")
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS Billing DB: {error}"),
            Self::MissingOutputRow => {
                formatter.write_str("Billing DB-процедура не вернула output-строку")
            }
            Self::MissingOutput(parameter) => {
                write!(formatter, "Billing DB-процедура вернула NULL в {parameter}")
            }
            Self::InvalidOleAutomationDate => {
                formatter.write_str("cash-log содержит недопустимую OLE Automation date")
            }
        }
    }
}

impl Error for RsPlayerAccountDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
            Self::MissingOutputRow | Self::MissingOutput(_) | Self::InvalidOleAutomationDate => {
                None
            }
        }
    }
}

impl From<tiberius::error::Error> for RsPlayerAccountDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Output `GetUserPoint`; при DB-ошибке равен исходным `-2/0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserPointOutcome {
    pub result: i32,
    pub point: i32,
}

/// Output `buyItemCode` и optional cash-log, создаваемый после успеха.
#[derive(Clone, Debug, PartialEq)]
pub struct BuyItemCodeOutcome {
    pub result: i32,
    pub transaction_code: Vec<u8>,
    pub last_point: i32,
    pub increment_log: Option<TagIncLogNode>,
}

/// Output `buyPlayerItem`; `from` — seller, `to` — buyer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuyPlayerItemOutcome {
    pub result: i32,
    pub transaction_code: Vec<u8>,
    pub buyer_last_point: i32,
    pub seller_last_point: i32,
}

/// Узкая объектная граница четырёх функций исходного `CRsPlayerAccount`.
pub trait RsPlayerAccountOwner {
    fn get_user_point(&mut self, user_id: &[u8]) -> UserPointOutcome;

    fn put_cash_log(&mut self, entries: &[TagIncLogNode]) -> i32;

    fn buy_player_item(&mut self, trade: &TagTradeNode) -> BuyPlayerItemOutcome;

    fn buy_item_code(
        &mut self,
        trade: &TagTradeNode,
        log_server_enabled: &AtomicBool,
    ) -> BuyItemCodeOutcome;

    fn pop_notice(&mut self) -> Option<RsPlayerAccountNotice>;
}

/// Linux/TDS-замена одного исходного `CRsPlayerAccount`.
pub struct TiberiusRsPlayerAccount {
    account_config: Config,
    cash_log_config: Config,
    runtime: Runtime,
    notices: VecDeque<RsPlayerAccountNotice>,
}

impl TiberiusRsPlayerAccount {
    /// Создаёт owner без соединения; ADO также соединялся внутри каждой функции.
    pub fn new(
        settings: BillingDatabaseSettings,
    ) -> Result<Self, RsPlayerAccountInitializationError> {
        let runtime = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(RsPlayerAccountInitializationError)?;
        Ok(Self {
            account_config: create_tds_config(settings.account),
            cash_log_config: create_tds_config(settings.cash_log),
            runtime,
            notices: VecDeque::new(),
        })
    }

    async fn connect(config: Config) -> Result<TdsClient, RsPlayerAccountDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(RsPlayerAccountDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(RsPlayerAccountDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(RsPlayerAccountDatabaseError::Tds)
    }

    async fn query_user_point(
        config: Config,
        user_id: String,
    ) -> Result<UserPointOutcome, RsPlayerAccountDatabaseError> {
        let mut client = Self::connect(config).await?;
        let row = client
            .query(
                "DECLARE @Point int, @Result int, \
                 @UserID varchar(200) = CONVERT(varchar(200), @P1); \
                 EXEC [GetUserPoint] \
                 @UserID = @UserID, \
                 @Point = @Point OUTPUT, @Result = @Result OUTPUT; \
                 SELECT @Result, @Point;",
                &[&user_id],
            )
            .await?
            .into_row()
            .await?
            .ok_or(RsPlayerAccountDatabaseError::MissingOutputRow)?;
        Ok(UserPointOutcome {
            result: required_i32(&row, 0, "@Result")?,
            point: required_i32(&row, 1, "@Point")?,
        })
    }

    async fn write_cash_logs(
        config: Config,
        entries: &[TagIncLogNode],
    ) -> Result<(), RsPlayerAccountDatabaseError> {
        let mut client = Self::connect(config).await?;
        for entry in entries {
            let log_time = ole_automation_datetime(entry.log_time)
                .ok_or(RsPlayerAccountDatabaseError::InvalidOleAutomationDate)?;
            let account = decode_ansi_c_string(&entry.buyer_identity);
            let address = decode_ansi_c_string(&entry.buyer_ip);
            let amount = i64::from(entry.yuanbao);
            client
                .execute(
                    "DECLARE @UserAcc varchar(200) = CONVERT(varchar(200), @P2), \
                     @UserIp varchar(200) = CONVERT(varchar(200), @P3), \
                     @Money int = CONVERT(int, @P4); \
                     EXEC [PutCashLog] \
                     @LogTime = @P1, \
                     @UserAcc = @UserAcc, @UserIp = @UserIp, @Money = @Money, \
                     @ItemIdx = @P5, @itemNum = @P6, @ls = @P7, @ws = @P8;",
                    &[
                        &log_time,
                        &account,
                        &address,
                        &amount,
                        &entry.goods_id,
                        &entry.goods_number,
                        &entry.login_server_id,
                        &entry.world_server_id,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    async fn query_buy_player_item(
        config: Config,
        trade: TagTradeNode,
    ) -> Result<BuyPlayerItemOutcome, RsPlayerAccountDatabaseError> {
        let mut client = Self::connect(config).await?;
        let seller_id = decode_ansi_c_string(&trade.seller_identity);
        let seller_ip = decode_ansi_c_string(&trade.seller_ip);
        let seller_name = decode_ansi_c_string(&trade.seller_name);
        let buyer_id = decode_ansi_c_string(&trade.buyer_identity);
        let buyer_ip = decode_ansi_c_string(&trade.buyer_ip);
        let buyer_name = decode_ansi_c_string(&trade.buyer_name);
        let amount = i64::from(trade.yuanbao);
        let row = client
            .query(
                "DECLARE @Result int, @TranCode varchar(500), \
                 @UIDfromLastPoint int, @UIDtoLastPoint int, \
                 @UIDfrom varchar(200) = CONVERT(varchar(200), @P1), \
                 @Ipfrom varchar(200) = CONVERT(varchar(200), @P2), \
                 @Cnamefrom varchar(200) = CONVERT(varchar(200), @P3), \
                 @UIDto varchar(200) = CONVERT(varchar(200), @P4), \
                 @Ipto varchar(200) = CONVERT(varchar(200), @P5), \
                 @Cnameto varchar(200) = CONVERT(varchar(200), @P6), \
                 @Amount int = CONVERT(int, @P10); \
                 EXEC [buyPlayerItem] \
                 @UIDfrom = @UIDfrom, @Ipfrom = @Ipfrom, @Cnamefrom = @Cnamefrom, \
                 @UIDto = @UIDto, @Ipto = @Ipto, @Cnameto = @Cnameto, \
                 @WorldId = @P7, @ItemIdx = @P8, @ItemNum = @P9, \
                 @Amount = @Amount, \
                 @Result = @Result OUTPUT, @TranCode = @TranCode OUTPUT, \
                 @UIDfromLastPoint = @UIDfromLastPoint OUTPUT, \
                 @UIDtoLastPoint = @UIDtoLastPoint OUTPUT; \
                 SELECT @Result, CONVERT(varbinary(500), @TranCode), \
                 @UIDfromLastPoint, @UIDtoLastPoint;",
                &[
                    &seller_id,
                    &seller_ip,
                    &seller_name,
                    &buyer_id,
                    &buyer_ip,
                    &buyer_name,
                    &trade.world_server_id,
                    &trade.goods_id,
                    &trade.goods_number,
                    &amount,
                ],
            )
            .await?
            .into_row()
            .await?
            .ok_or(RsPlayerAccountDatabaseError::MissingOutputRow)?;
        Ok(BuyPlayerItemOutcome {
            result: required_i32(&row, 0, "@Result")?,
            transaction_code: required_bytes(&row, 1, "@TranCode")?,
            seller_last_point: required_i32(&row, 2, "@UIDfromLastPoint")?,
            buyer_last_point: required_i32(&row, 3, "@UIDtoLastPoint")?,
        })
    }

    async fn query_buy_item_code(
        config: Config,
        trade: TagTradeNode,
    ) -> Result<BuyItemCodeOutcome, RsPlayerAccountDatabaseError> {
        let mut client = Self::connect(config).await?;
        let user_id = decode_ansi_c_string(&trade.buyer_identity);
        let address = decode_ansi_c_string(&trade.buyer_ip);
        let world = trade.world_server_id.to_string();
        let character_name = decode_ansi_c_string(&trade.buyer_name);
        let amount = i64::from(trade.yuanbao);
        let row = client
            .query(
                "DECLARE @Result int, @TranCode varchar(500), @LastPoint int, \
                 @UserID varchar(32) = CONVERT(varchar(32), @P1), \
                 @Amount int = CONVERT(int, @P4), \
                 @IP varchar(200) = CONVERT(varchar(200), @P5), \
                 @World varchar(200) = CONVERT(varchar(200), @P6), \
                 @Character_Name varchar(200) = CONVERT(varchar(200), @P7); \
                 EXEC [buyItemCode] \
                 @UserID = @UserID, \
                 @ItemIndex = @P2, @Qty = @P3, \
                 @Amount = @Amount, @IP = @IP, @World = @World, \
                 @Character_Name = @Character_Name, \
                 @Result = @Result OUTPUT, @TranCode = @TranCode OUTPUT, \
                 @LastPoint = @LastPoint OUTPUT; \
                 SELECT @Result, CONVERT(varbinary(500), @TranCode), @LastPoint;",
                &[
                    &user_id,
                    &trade.goods_id,
                    &trade.goods_number,
                    &amount,
                    &address,
                    &world,
                    &character_name,
                ],
            )
            .await?
            .into_row()
            .await?
            .ok_or(RsPlayerAccountDatabaseError::MissingOutputRow)?;
        Ok(BuyItemCodeOutcome {
            result: required_i32(&row, 0, "@Result")?,
            transaction_code: required_bytes(&row, 1, "@TranCode")?,
            last_point: required_i32(&row, 2, "@LastPoint")?,
            increment_log: None,
        })
    }

    fn push_error(
        &mut self,
        operation: RsPlayerAccountOperation,
        error: RsPlayerAccountDatabaseError,
    ) {
        self.notices
            .push_back(RsPlayerAccountNotice { operation, error });
    }
}

impl RsPlayerAccountOwner for TiberiusRsPlayerAccount {
    fn get_user_point(&mut self, user_id: &[u8]) -> UserPointOutcome {
        let future =
            Self::query_user_point(self.account_config.clone(), decode_ansi_c_string(user_id));
        match self.runtime.block_on(future) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.push_error(RsPlayerAccountOperation::GetUserPoint, error);
                UserPointOutcome {
                    result: -2,
                    point: 0,
                }
            }
        }
    }

    fn put_cash_log(&mut self, entries: &[TagIncLogNode]) -> i32 {
        let future = Self::write_cash_logs(self.cash_log_config.clone(), entries);
        if let Err(error) = self.runtime.block_on(future) {
            self.push_error(RsPlayerAccountOperation::PutCashLog, error);
        }
        0
    }

    fn buy_player_item(&mut self, trade: &TagTradeNode) -> BuyPlayerItemOutcome {
        let future = Self::query_buy_player_item(self.account_config.clone(), trade.clone());
        match self.runtime.block_on(future) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.push_error(RsPlayerAccountOperation::BuyPlayerItem, error);
                BuyPlayerItemOutcome {
                    result: -2,
                    transaction_code: Vec::new(),
                    buyer_last_point: 0,
                    seller_last_point: 0,
                }
            }
        }
    }

    fn buy_item_code(
        &mut self,
        trade: &TagTradeNode,
        log_server_enabled: &AtomicBool,
    ) -> BuyItemCodeOutcome {
        let future = Self::query_buy_item_code(self.account_config.clone(), trade.clone());
        match self.runtime.block_on(future) {
            Ok(mut outcome) => {
                if outcome.result == 0 && log_server_enabled.load(Ordering::Acquire) {
                    outcome.increment_log = Some(TagIncLogNode::from_parts(TagIncLogNodeParts {
                        log_time: current_ole_automation_date(),
                        buyer_identity: trade.buyer_identity.clone(),
                        buyer_ip: trade.buyer_ip.clone(),
                        yuanbao: trade.yuanbao,
                        goods_id: trade.goods_id,
                        goods_number: trade.goods_number,
                        login_server_id: trade.login_server_id,
                        world_server_id: trade.world_server_id,
                    }));
                }
                outcome
            }
            Err(error) => {
                self.push_error(RsPlayerAccountOperation::BuyItemCode, error);
                BuyItemCodeOutcome {
                    result: -5,
                    transaction_code: Vec::new(),
                    last_point: 0,
                    increment_log: None,
                }
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsPlayerAccountNotice> {
        self.notices.pop_front()
    }
}

fn required_i32(
    row: &tiberius::Row,
    index: usize,
    parameter: &'static str,
) -> Result<i32, RsPlayerAccountDatabaseError> {
    row.get::<i32, _>(index)
        .ok_or(RsPlayerAccountDatabaseError::MissingOutput(parameter))
}

fn required_bytes(
    row: &tiberius::Row,
    index: usize,
    parameter: &'static str,
) -> Result<Vec<u8>, RsPlayerAccountDatabaseError> {
    let bytes = row
        .get::<&[u8], _>(index)
        .ok_or(RsPlayerAccountDatabaseError::MissingOutput(parameter))?;
    let visible = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end]);
    debug_assert!(visible.len() <= TRANSACTION_CODE_LIMIT);
    Ok(visible.to_vec())
}

fn ole_epoch() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(1899, 12, 30)
        .expect("фиксированная OLE-эпоха является корректной датой")
        .and_hms_opt(0, 0, 0)
        .expect("полночь является корректным временем")
}

fn ole_automation_datetime(value: f64) -> Option<NaiveDateTime> {
    if !value.is_finite() {
        return None;
    }
    let milliseconds = value * MILLIS_PER_DAY;
    if milliseconds < i64::MIN as f64 || milliseconds > i64::MAX as f64 {
        return None;
    }
    ole_epoch().checked_add_signed(TimeDelta::milliseconds(milliseconds.round() as i64))
}

fn current_ole_automation_date() -> f64 {
    let now = Local::now().naive_local();
    now.signed_duration_since(ole_epoch()).num_milliseconds() as f64 / MILLIS_PER_DAY
}

fn decode_ansi_c_string(bytes: &[u8]) -> String {
    let visible = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end]);
    let (decoded, _, _) = WINDOWS_1251.decode(visible);
    decoded.into_owned()
}

fn create_tds_config(settings: BillingDatabaseConnectionSettings) -> Config {
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
