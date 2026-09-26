//! Очереди `loginqueue.cpp/.h`. Они связывают CD-key/player/GAS FIFO, проверку
//! пароля,
//! valid-code и matrix с их wrapping-таймерами.
//!
//! Замена `TagPwdChecked`, полная выгрузка очереди и callbacks выполняются под
//! одним прежним lock. Нулевые socket/IP отбрасываются. Valid-code удаляется при
//! несовпадении endpoint или успехе, но остаётся после неверного кода;
//! matrix-запись удаляется после любой проверки либо несовпадения endpoint.
//!
//! Один проход сохраняет порядок GAS/no-queue, обычных FIFO, паролей, player-
//! очередей, `AuthManager` и трёх timeout-проверок. GAS повторяется, no-queue
//! очищается, а конкурентно добавленные элементы остаются следующему проходу.
//! Все сроки используют исходные строгие wrapping-сравнения.
//!
//! CD-key проходит локальные проверки до uppercase digest для БД; lowercase
//! применяется только для AuthServer. Потеря client-а удаляет лишь первые
//! совпадения обычных player-очередей и не затрагивает остальные слои.
//! `NoQueueAccounts.conf` очищает set до открытия и вставляет ASCII-lowercase
//! токены по мере чтения. Токен длиннее 255 байт отклоняется вместо переполнения
//! `char[0x100]`. `Mutex`, `BTreeMap` и `VecDeque` сохраняют прежние lock-границы
//! и FIFO без ручного Win32/STL-владения.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;

use chrono::{Datelike, Local, Timelike};
use parking_lot::Mutex;
use rustix::time::{ClockId, clock_gettime};
use thiserror::Error as ThisError;

use crate::access::rscdkey::MatrixValidation;
use crate::access::validcode::{CValidCode, ValidCodeError};
use super::authhandler::AuthHandler;
use super::authmanager::{
    AddQuestOutcome, AuthManager, AuthQuest, AuthRunOutcome,
};
use super::game::{
    AuthLifecycleError, CGame, GameRouteError, PrepareEnterOutcome, resolve_legacy_ascii_case,
};
use crate::app::login_message::CMessage;

const AUTH_FAILED_MESSAGE_TYPE: i32 = 0x000A_F501;
const PLAYER_DATA_REJECT_MESSAGE_TYPE: i32 = 0x000A_F503;
const QUEUE_POSITION_MESSAGE_TYPE: i32 = 0x000A_F507;
const NO_QUEUE_ACCOUNT_BUFFER_SIZE: usize = 0x100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagPwdChecked {
    client_ip: u32,
    socket_id: i32,
    account: Vec<u8>,
    world_server: Vec<u8>,
    has_matrix: bool,
}

impl TagPwdChecked {
    pub fn new(
        socket_id: i32,
        client_ip: u32,
        account: Vec<u8>,
        world_server: Vec<u8>,
        has_matrix: bool,
    ) -> Self {
        Self {
            client_ip,
            socket_id,
            account,
            world_server,
            has_matrix,
        }
    }

    pub fn account(&self) -> &[u8] {
        &self.account
    }

    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    pub const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub fn world_server(&self) -> &[u8] {
        &self.world_server
    }

    pub const fn has_matrix(&self) -> bool {
        self.has_matrix
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestCdkey {
    socket_id: i32,
    client_ip: u32,
    login_type: i8,
    version: i32,
    account: Vec<u8>,
    password_digest: Vec<u8>,
    client_code: i16,
    encryption_key: i32,
    world_server: Vec<u8>,
    send_message_time: u32,
    first_gas_value: Vec<u8>,
    second_gas_value: Vec<u8>,
    nickname: Vec<u8>,
}

impl QuestCdkey {
    #[allow(clippy::too_many_arguments)]
    fn new(
        socket_id: i32,
        client_ip: u32,
        login_type: i32,
        version: i32,
        account: Vec<u8>,
        password_digest: Vec<u8>,
        client_code: i16,
        encryption_key: i32,
        world_server: Vec<u8>,
        send_message_time: u32,
    ) -> Self {
        Self {
            socket_id,
            client_ip,
            login_type: login_type as i8,
            version,
            account,
            password_digest,
            client_code,
            encryption_key,
            world_server,
            send_message_time,
            first_gas_value: Vec::new(),
            second_gas_value: Vec::new(),
            nickname: Vec::new(),
        }
    }

    pub fn account(&self) -> &[u8] {
        &self.account
    }

    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    pub const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub fn password_digest(&self) -> &[u8] {
        &self.password_digest
    }

    pub fn world_server(&self) -> &[u8] {
        &self.world_server
    }

    pub fn nickname(&self) -> &[u8] {
        &self.nickname
    }

    pub fn replace_account_with_nickname(&mut self, nickname: Vec<u8>) {
        self.account.clone_from(&nickname);
        self.nickname = nickname;
    }

    pub const fn send_message_time(&self) -> u32 {
        self.send_message_time
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestPlayerList {
    socket_id: i32,
    account: Vec<u8>,
    world_server: Vec<u8>,
    send_message_time: u32,
}

impl QuestPlayerList {
    fn new(
        socket_id: i32,
        account: Vec<u8>,
        world_server: Vec<u8>,
        send_message_time: u32,
    ) -> Self {
        Self {
            socket_id,
            account,
            world_server,
            send_message_time,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestPlayerData {
    socket_id: i32,
    account: Vec<u8>,
    player_id: i32,
    client_ip: u32,
    send_message_time: u32,
}

impl QuestPlayerData {
    fn new(
        socket_id: i32,
        account: Vec<u8>,
        player_id: i32,
        client_ip: u32,
        send_message_time: u32,
    ) -> Self {
        Self {
            socket_id,
            account,
            player_id,
            client_ip,
            send_message_time,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestPlayerDataOutcome {
    Forwarded { world_found: bool },
    Repeated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientLostCleanupReport {
    pub cdkey_removed: bool,
    pub player_list_removed: usize,
    pub player_data_removed: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum QuestCdkeyOutcome {
    DirectWorld,
    DirectWorldFinished,
    QueuedForGas,
    InsideModeIgnored { mode: i32 },
    ActiveBan,
    IpRejected,
    BetweenIpRejected,
    AuthQuest(AddQuestOutcome),
    LocalPasswordRejected,
    LocalPasswordAccepted,
}

#[derive(Debug)]
pub enum QuestCdkeyError {
    DatabaseOwnerMissing,
    IpSetupMissing,
    InsideModeMissing,
    PasswordDigestTooShort { actual_len: usize },
    AuthAnsiCaseMappingUnknown,
    Route(GameRouteError),
}

impl fmt::Display for QuestCdkeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseOwnerMissing => {
                formatter.write_str("DB-владелец CRsCDKey для OnQuestCdkey отсутствует")
            }
            Self::IpSetupMissing => formatter.write_str("три IP-флага Login setup не определены"),
            Self::InsideModeMissing => {
                formatter.write_str("режим m_lIsInsideUse Login setup не определён")
            }
            Self::PasswordDigestTooShort { actual_len } => write!(
                formatter,
                "password digest короче исходных 16 байт: {actual_len}"
            ),
            Self::AuthAnsiCaseMappingUnknown => formatter.write_str(
                "для Auth account с high-bit байтами неизвестна системная ANSI lowercase-карта",
            ),
            Self::Route(error) => error.fmt(formatter),
        }
    }
}

impl Error for QuestCdkeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Route(error) => Some(error),
            Self::DatabaseOwnerMissing
            | Self::IpSetupMissing
            | Self::InsideModeMissing
            | Self::AuthAnsiCaseMappingUnknown
            | Self::PasswordDigestTooShort { .. } => None,
        }
    }
}

impl From<GameRouteError> for QuestCdkeyError {
    fn from(error: GameRouteError) -> Self {
        Self::Route(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TagValidErr {
    error_times: i32,
    next_login_time: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TagValidCode {
    socket_id: i32,
    client_ip: u32,
    added_time: u32,
    valid_code: Vec<u8>,
    world_server: Vec<u8>,
    has_matrix: bool,
    change_time: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MatrixEntry {
    socket_id: i32,
    client_ip: u32,
    positions: [u8; 3],
    added_time: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckMessageInfo {
    Success,
    Missing,
    SocketMismatch,
    Frequent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidateValidCodeOutcome {
    Accepted {
        world_server: Vec<u8>,
        has_matrix: bool,
    },
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatrixValidationOutcome {
    Accepted,
    Rejected,
    DatabaseOwnerMissing,
}

#[derive(Debug)]
pub enum PwdCheckedNotice {
    ClientResponse {
        socket_id: i32,
        error: GameRouteError,
    },
    ValidCodeGeneration {
        account: Vec<u8>,
        error: ValidCodeError,
    },
    PrepareEnter {
        account: Vec<u8>,
        error: GameRouteError,
    },
    MatrixRandom {
        account: Vec<u8>,
        error: getrandom::Error,
    },
    EnterGame {
        account: Vec<u8>,
        error: GameRouteError,
    },
}

#[derive(Debug)]
pub struct HandlePwdCheckedReport {
    pub processed: usize,
    pub dropped_invalid_endpoint: usize,
    pub rejected_by_valid_errors: usize,
    pub generated_valid_codes: usize,
    pub notices: Vec<PwdCheckedNotice>,
}

impl HandlePwdCheckedReport {
    fn new() -> Self {
        Self {
            processed: 0,
            dropped_invalid_endpoint: 0,
            rejected_by_valid_errors: 0,
            generated_valid_codes: 0,
            notices: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct LoginQueueTimeoutReport {
    pub matrices_ran: bool,
    pub valid_codes_ran: bool,
    pub valid_errors_ran: bool,
    pub notices: Vec<PwdCheckedNotice>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginQueueRunStage {
    GasCdkey,
    NoQueueCdkey,
    NoQueuePlayerList,
    NoQueuePlayerData,
    RegularCdkey,
    RegularPlayerList,
    RegularPlayerData,
    QueuePosition,
}

#[derive(Debug)]
pub enum LoginQueueRunNotice {
    Cdkey {
        stage: LoginQueueRunStage,
        account: Vec<u8>,
        error: QuestCdkeyError,
    },
    Route {
        stage: LoginQueueRunStage,
        account: Vec<u8>,
        socket_id: i32,
        error: GameRouteError,
    },
}

#[derive(Debug)]
pub struct LoginQueueRunReport {
    pub gas_cdkeys_processed: usize,
    pub no_queue_cdkeys_discarded_by_gas_bug: usize,
    pub no_queue_cdkeys_processed: usize,
    pub no_queue_player_lists_processed: usize,
    pub no_queue_player_data_processed: usize,
    pub regular_cdkeys_processed: usize,
    pub regular_player_lists_processed: usize,
    pub regular_player_data_processed: usize,
    pub queue_position_messages: usize,
    pub login_timeouts_removed: usize,
    pub pwd_checked: HandlePwdCheckedReport,
    pub auth: Result<AuthRunOutcome, AuthLifecycleError>,
    pub timeout_tail: LoginQueueTimeoutReport,
    pub notices: Vec<LoginQueueRunNotice>,
}

#[derive(Debug, Default)]
struct LoginQueueCadence {
    matrices_last_timeout: u32,
    valid_code_last_overtime: u32,
    valid_error_last_check: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct LoginQueueSetup {
    interval_ms: u32,
    send_message_interval_ms: u32,
    world_max_players: u32,
    world_count: u32,
    world_queue_time: u32,
    log_queue_time: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoQueueAccountsLoadReport {
    pub extracted_accounts: usize,
    pub unique_accounts: usize,
}

#[derive(Debug, ThisError)]
pub enum NoQueueAccountsLoadError {
    #[error("не прочитан NoQueueAccounts.conf: {0}")]
    Io(#[from] io::Error),
    #[error(
        "token {token_index} NoQueueAccounts.conf имеет {actual} байт при пределе {maximum}"
    )]
    TokenTooLong {
        token_index: usize,
        actual: usize,
        maximum: usize,
    },
}

pub struct CLoginQueue {
    cdkey_quests: Mutex<VecDeque<QuestCdkey>>,
    no_queue_cdkey_quests: Mutex<VecDeque<QuestCdkey>>,
    gas_quests: Mutex<VecDeque<QuestCdkey>>,
    player_list_quests: Mutex<BTreeMap<Vec<u8>, VecDeque<QuestPlayerList>>>,
    no_queue_player_list_quests: Mutex<BTreeMap<Vec<u8>, VecDeque<QuestPlayerList>>>,
    player_data_quests: Mutex<BTreeMap<Vec<u8>, VecDeque<QuestPlayerData>>>,
    no_queue_player_data_quests: Mutex<BTreeMap<Vec<u8>, VecDeque<QuestPlayerData>>>,
    login_list: Mutex<BTreeMap<i32, u32>>,
    pwd_checked: Mutex<VecDeque<TagPwdChecked>>,
    valid_errors: Mutex<BTreeMap<Vec<u8>, TagValidErr>>,
    valid_codes: Mutex<BTreeMap<Vec<u8>, TagValidCode>>,
    matrices: Mutex<BTreeMap<Vec<u8>, MatrixEntry>>,
    no_queue_accounts: Mutex<BTreeSet<Vec<u8>>>,
    cadence: Mutex<LoginQueueCadence>,
    setup: Mutex<LoginQueueSetup>,
}

impl CLoginQueue {
    /// Создаёт все исходные контейнеры и загружает no-queue accounts.
    ///
    /// Ошибка файла не отменяет создание owner и возвращается отдельно, не
    /// меняя нефатальное поведение исходного конструктора.
    pub fn new(
        runtime_directory: &Path,
    ) -> (
        Self,
        Result<NoQueueAccountsLoadReport, NoQueueAccountsLoadError>,
    ) {
        let queue = Self {
            cdkey_quests: Mutex::new(VecDeque::new()),
            no_queue_cdkey_quests: Mutex::new(VecDeque::new()),
            gas_quests: Mutex::new(VecDeque::new()),
            player_list_quests: Mutex::new(BTreeMap::new()),
            no_queue_player_list_quests: Mutex::new(BTreeMap::new()),
            player_data_quests: Mutex::new(BTreeMap::new()),
            no_queue_player_data_quests: Mutex::new(BTreeMap::new()),
            login_list: Mutex::new(BTreeMap::new()),
            pwd_checked: Mutex::new(VecDeque::new()),
            valid_errors: Mutex::new(BTreeMap::new()),
            valid_codes: Mutex::new(BTreeMap::new()),
            matrices: Mutex::new(BTreeMap::new()),
            no_queue_accounts: Mutex::new(BTreeSet::new()),
            cadence: Mutex::new(LoginQueueCadence::default()),
            setup: Mutex::new(LoginQueueSetup::default()),
        };
        let no_queue_accounts = queue.load_no_queue_cdkey_list(runtime_directory);
        (queue, no_queue_accounts)
    }

    pub fn on_initial(
        &self,
        interval_ms: u32,
        send_message_interval_ms: u32,
        world_max_players: u32,
    ) {
        let mut setup = self.setup.lock();
        setup.interval_ms = interval_ms;
        setup.send_message_interval_ms = send_message_interval_ms;
        setup.world_max_players = world_max_players;
        if setup.world_count == 0 {
            setup.world_count = 1;
        }
        let now = legacy_tick_ms();
        setup.world_queue_time = interval_ms.wrapping_add(now);
        setup.log_queue_time = (interval_ms / setup.world_count).wrapping_add(now);
    }

    pub fn set_world_count(&self, world_count: u32) {
        self.setup.lock().world_count = world_count;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_quest_cdkey(
        &self,
        socket_id: i32,
        client_ip: u32,
        login_type: i32,
        version: i32,
        account: Vec<u8>,
        password_digest: Vec<u8>,
        client_code: i16,
        encryption_key: i32,
        world_server: Vec<u8>,
    ) {
        let send_interval = self.setup.lock().send_message_interval_ms;
        let send_message_time = legacy_tick_ms().wrapping_add(send_interval);
        let no_queue = self.is_no_queue_account(&account);
        let quest = QuestCdkey::new(
            socket_id,
            client_ip,
            login_type,
            version,
            account,
            password_digest,
            client_code,
            encryption_key,
            world_server,
            send_message_time,
        );
        if no_queue {
            self.no_queue_cdkey_quests.lock().push_back(quest);
        } else {
            self.cdkey_quests.lock().push_back(quest);
        }
    }

    pub fn pop_gas_quest(&self) -> Option<QuestCdkey> {
        self.gas_quests.lock().pop_front()
    }

    pub fn on_quest_cdkey(
        &self,
        game: &mut CGame,
        auth_manager: &mut AuthManager,
        quest: &QuestCdkey,
    ) -> Result<QuestCdkeyOutcome, QuestCdkeyError> {
        if !quest.world_server.is_empty() {
            let checked = TagPwdChecked::new(
                quest.socket_id,
                quest.client_ip,
                quest.account.clone(),
                quest.world_server.clone(),
                false,
            );
            return match game.prepare_enter(&checked)? {
                PrepareEnterOutcome::Continue => {
                    game.enter_game(&checked, self.is_no_queue_account(checked.account()))?;
                    Ok(QuestCdkeyOutcome::DirectWorld)
                }
                PrepareEnterOutcome::Finished => Ok(QuestCdkeyOutcome::DirectWorldFinished),
                PrepareEnterOutcome::MatrixRegistrationRequired => {
                    // `has_matrix` выше буквально false, поэтому этот вариант
                    // недостижим без нарушения контракта `PrepareEnter`.
                    Ok(QuestCdkeyOutcome::DirectWorldFinished)
                }
            };
        }

        let inside_mode = game
            .login_setup()
            .inside_use()
            .ok_or(QuestCdkeyError::InsideModeMissing)?;
        if inside_mode != 1 {
            if inside_mode == 0 {
                self.gas_quests.lock().push_back(quest.clone());
                return Ok(QuestCdkeyOutcome::QueuedForGas);
            }
            return Ok(QuestCdkeyOutcome::InsideModeIgnored { mode: inside_mode });
        }

        let mut account = quest.account.clone();
        if account.iter().all(u8::is_ascii_digit) {
            account = game
                .rs_cdkey_owner_mut()
                .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
                .fix_pt_account(&account);
        }

        let ban_time = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .get_ban_time(&account);
        if ban_time.is_some_and(|ban_time| Local::now().naive_local() < ban_time) {
            let ban_time = ban_time.expect("активный ban_time только что проверен");
            let mut response = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
            response.base_mut().add_char(0x10);
            response.base_mut().add_char(0);
            response.base_mut().add_short(ban_time.year() as i16);
            response.base_mut().add_short(ban_time.month() as i16);
            response.base_mut().add_short(ban_time.day() as i16);
            response.base_mut().add_short(ban_time.hour() as i16);
            response.base_mut().add_short(ban_time.minute() as i16);
            game.send_to_client(&response, quest.socket_id)?;
            return Ok(QuestCdkeyOutcome::ActiveBan);
        }

        let (check_allowed, check_forbidden, check_between) = game
            .login_setup()
            .ip_checks()
            .ok_or(QuestCdkeyError::IpSetupMissing)?;
        let allowed = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .ip_is_allowed(check_allowed, quest.client_ip);
        if !allowed {
            send_login_code(game, quest.socket_id, 0x12)?;
            return Ok(QuestCdkeyOutcome::IpRejected);
        }
        let forbidden = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .ip_is_forbidden(check_forbidden, quest.client_ip);
        if forbidden {
            send_login_code(game, quest.socket_id, 0x12)?;
            return Ok(QuestCdkeyOutcome::IpRejected);
        }
        let between = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .is_between_ip(check_between, &account, quest.client_ip);
        if !between {
            send_login_code(game, quest.socket_id, 0x11)?;
            return Ok(QuestCdkeyOutcome::BetweenIpRejected);
        }

        let has_matrix = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .matrix_used(&account);
        let password = password_digest_hex(&quest.password_digest)?;

        if game.is_connect_as() {
            let mut auth_account = account;
            let mut auth_password = password;
            legacy_lower_auth(&mut auth_account)?;
            legacy_lower_auth(&mut auth_password)?;
            let sender = game
                .auth_send_queue()
                .expect("IsConnectAS подтверждает существующий Auth client");
            let outcome = auth_manager.add_quest(
                AuthQuest::new(
                    quest.client_ip,
                    quest.socket_id,
                    auth_account,
                    auth_password,
                ),
                sender,
                AuthHandler::on_quest,
            );
            return Ok(QuestCdkeyOutcome::AuthQuest(outcome));
        }

        let Some(canonical_account) = game
            .rs_cdkey_owner_mut()
            .ok_or(QuestCdkeyError::DatabaseOwnerMissing)?
            .validate_local_password(&account, &password)
        else {
            send_login_code(game, quest.socket_id, 7)?;
            return Ok(QuestCdkeyOutcome::LocalPasswordRejected);
        };
        game.push_back_pwd_checked(TagPwdChecked::new(
            quest.socket_id,
            quest.client_ip,
            canonical_account,
            quest.world_server.clone(),
            has_matrix,
        ));
        Ok(QuestCdkeyOutcome::LocalPasswordAccepted)
    }

    /// Добавляет запрос списка персонажей в карту выбранного WorldServer.
    ///
    /// `false` означает исходный тихий отказ одной из трёх предварительных
    /// проверок: World отсутствует, account ещё не выбрал World либо список
    /// подключённых account этого World ещё не создан.
    pub fn add_quest_player_list(
        &self,
        game: &CGame,
        socket_id: i32,
        account: &[u8],
        world_server: &[u8],
    ) -> bool {
        if !game.is_exit_world(world_server)
            || game.login_cdkey_world_server(account).is_none()
            || game.login_world_player_num_by_name(world_server) == -1
        {
            return false;
        }

        let send_interval = self.setup.lock().send_message_interval_ms;
        let quest = QuestPlayerList::new(
            socket_id,
            account.to_vec(),
            world_server.to_vec(),
            legacy_tick_ms().wrapping_add(send_interval),
        );
        let queues = if self.is_no_queue_account(account) {
            &self.no_queue_player_list_quests
        } else {
            &self.player_list_quests
        };
        queues
            .lock()
            .entry(world_server.to_vec())
            .or_default()
            .push_back(quest);
        true
    }

    pub fn add_quest_player_data(
        &self,
        game: &CGame,
        socket_id: i32,
        account: &[u8],
        player_id: i32,
        client_ip: u32,
    ) -> bool {
        let Some(world_server) = game.login_cdkey_world_server(account).map(<[u8]>::to_vec) else {
            return false;
        };

        let send_interval = self.setup.lock().send_message_interval_ms;
        let quest = QuestPlayerData::new(
            socket_id,
            account.to_vec(),
            player_id,
            client_ip,
            legacy_tick_ms().wrapping_add(send_interval),
        );
        let queues = if self.is_no_queue_account(account) {
            &self.no_queue_player_data_quests
        } else {
            &self.player_data_quests
        };
        queues
            .lock()
            .entry(world_server)
            .or_default()
            .push_back(quest);
        true
    }

    pub fn on_client_lost(&self, account: &[u8]) -> ClientLostCleanupReport {
        let end = account
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(account.len());
        let account = &account[..end];

        let cdkey_removed = {
            let mut quests = self.cdkey_quests.lock();
            let position = quests.iter().position(|quest| quest.account == account);
            position
                .and_then(|position| quests.remove(position))
                .is_some()
        };

        let player_list_removed = {
            let mut queues = self.player_list_quests.lock();
            let mut removed = 0;
            for quests in queues.values_mut() {
                let position = quests.iter().position(|quest| quest.account == account);
                if position
                    .and_then(|position| quests.remove(position))
                    .is_some()
                {
                    removed += 1;
                }
            }
            removed
        };

        let player_data_removed = {
            let mut queues = self.player_data_quests.lock();
            let mut removed = 0;
            for quests in queues.values_mut() {
                let position = quests.iter().position(|quest| quest.account == account);
                if position
                    .and_then(|position| quests.remove(position))
                    .is_some()
                {
                    removed += 1;
                }
            }
            removed
        };

        ClientLostCleanupReport {
            cdkey_removed,
            player_list_removed,
            player_data_removed,
        }
    }

    pub fn on_quest_player_data(
        &self,
        game: &CGame,
        quest: &QuestPlayerData,
    ) -> Result<QuestPlayerDataOutcome, GameRouteError> {
        let world_server = game
            .login_cdkey_world_server(&quest.account)
            .map(<[u8]>::to_vec);
        if self.is_valid_quest(quest.player_id, game.quest_player_data_interval_ms()) {
            let send_result = game.l2w_quest_detail_send(
                world_server.as_deref(),
                &quest.account,
                quest.player_id,
                quest.client_ip,
            );
            // Исходный `PushLoginList` выполнялся после void-send независимо
            // от его внутреннего результата.
            self.push_login_list(quest.player_id);
            return send_result
                .map(|world_found| QuestPlayerDataOutcome::Forwarded { world_found });
        }

        let mut response = CMessage::new(PLAYER_DATA_REJECT_MESSAGE_TYPE);
        response.base_mut().add_char(0x1c);
        add_legacy_string(&mut response, &quest.account);
        game.send_to_client_cdkey(&response, &quest.account)?;
        Ok(QuestPlayerDataOutcome::Repeated)
    }

    pub fn is_valid_quest(&self, player_id: i32, interval_ms: u32) -> bool {
        let mut login_list = self.login_list.lock();
        let Some(added_time) = login_list.get(&player_id).copied() else {
            return true;
        };
        let now = legacy_tick_ms();
        if now <= added_time.wrapping_add(interval_ms) {
            return false;
        }
        login_list.remove(&player_id);
        true
    }

    pub fn push_login_list(&self, player_id: i32) -> bool {
        let mut login_list = self.login_list.lock();
        if login_list.contains_key(&player_id) {
            return false;
        }
        login_list.insert(player_id, legacy_tick_ms());
        true
    }

    pub fn clear_timeout_list(&self, interval_ms: u32) -> usize {
        let mut login_list = self.login_list.lock();
        let previous_len = login_list.len();
        login_list.retain(|_, added_time| {
            let now = legacy_tick_ms();
            added_time.wrapping_add(interval_ms) >= now
        });
        previous_len - login_list.len()
    }

    /// Выполняет полный порядок исходного `CLoginQueue::Run`.
    ///
    /// Неопределённый исходный race с producers заменён короткими snapshot-
    /// границами: конкурентно добавленные элементы остаются следующему проходу.
    pub fn run(
        &self,
        game: &mut CGame,
        auth_manager: &mut AuthManager,
        matrix_timeout_ms: u32,
        valid_code_overtime_ms: u32,
    ) -> LoginQueueRunReport {
        let mut gas_cdkeys_processed = 0;
        let mut no_queue_cdkeys_discarded_by_gas_bug = 0;
        let mut no_queue_cdkeys_processed = 0;
        let mut no_queue_player_lists_processed = 0;
        let mut no_queue_player_data_processed = 0;
        let mut regular_cdkeys_processed = 0;
        let mut regular_player_lists_processed = 0;
        let mut regular_player_data_processed = 0;
        let mut queue_position_messages = 0;
        let mut notices = Vec::new();
        // GAS по `this+0x64/0x68`, но очищает `this+0x34/0x38` — именно
        // no-queue CD-key FIFO. GAS намеренно остаётся и повторяется далее.
        let gas_snapshot: Vec<_> = self.gas_quests.lock().iter().cloned().collect();
        if !gas_snapshot.is_empty() {
            for quest in &gas_snapshot {
                gas_cdkeys_processed += 1;
                if let Err(error) = self.on_quest_cdkey(game, auth_manager, quest) {
                    notices.push(LoginQueueRunNotice::Cdkey {
                        stage: LoginQueueRunStage::GasCdkey,
                        account: quest.account.clone(),
                        error,
                    });
                }
            }
            let mut no_queue = self.no_queue_cdkey_quests.lock();
            no_queue_cdkeys_discarded_by_gas_bug = no_queue.len();
            no_queue.clear();
        }

        let no_queue_cdkeys = mem::take(&mut *self.no_queue_cdkey_quests.lock());
        for quest in no_queue_cdkeys {
            no_queue_cdkeys_processed += 1;
            if let Err(error) = self.on_quest_cdkey(game, auth_manager, &quest) {
                notices.push(LoginQueueRunNotice::Cdkey {
                    stage: LoginQueueRunStage::NoQueueCdkey,
                    account: quest.account,
                    error,
                });
            }
        }

        let no_queue_player_lists = mem::take(&mut *self.no_queue_player_list_quests.lock());
        for quests in no_queue_player_lists.into_values() {
            for quest in quests {
                no_queue_player_lists_processed += 1;
                let route = game.l2w_player_base_send(
                    &quest.world_server,
                    &quest.account,
                    self.is_no_queue_account(&quest.account),
                );
                match route {
                    Ok(true) => {
                        game.set_login_cdkey_world_server(&quest.account, &quest.world_server)
                    }
                    Ok(false) => {}
                    Err(error) => notices.push(LoginQueueRunNotice::Route {
                        stage: LoginQueueRunStage::NoQueuePlayerList,
                        account: quest.account,
                        socket_id: quest.socket_id,
                        error,
                    }),
                }
            }
        }

        let no_queue_player_data = mem::take(&mut *self.no_queue_player_data_quests.lock());
        for quests in no_queue_player_data.into_values() {
            for quest in quests {
                no_queue_player_data_processed += 1;
                if let Err(error) = self.on_quest_player_data(game, &quest) {
                    notices.push(LoginQueueRunNotice::Route {
                        stage: LoginQueueRunStage::NoQueuePlayerData,
                        account: quest.account,
                        socket_id: quest.socket_id,
                        error,
                    });
                }
            }
        }

        let now = legacy_tick_ms();
        let run_regular_cdkey = {
            let mut setup = self.setup.lock();
            if setup.log_queue_time <= now {
                let interval = setup
                    .interval_ms
                    .checked_div(setup.world_count)
                    .unwrap_or(1_000);
                setup.log_queue_time = now.wrapping_add(interval);
                true
            } else {
                false
            }
        };
        if run_regular_cdkey {
            let quest = self.cdkey_quests.lock().front().cloned();
            if let Some(quest) = quest {
                regular_cdkeys_processed = 1;
                if let Err(error) = self.on_quest_cdkey(game, auth_manager, &quest) {
                    notices.push(LoginQueueRunNotice::Cdkey {
                        stage: LoginQueueRunStage::RegularCdkey,
                        account: quest.account.clone(),
                        error,
                    });
                }
                self.cdkey_quests.lock().pop_front();
            }
        }

        let pwd_checked = self.handle_pwd_checked(game);

        let (run_world_queues, world_max_players) = {
            let mut setup = self.setup.lock();
            let due = setup.world_queue_time <= now;
            if due {
                setup.world_queue_time = now.wrapping_add(setup.interval_ms);
            }
            (due, setup.world_max_players as i32)
        };
        if run_world_queues {
            let worlds: Vec<_> = self.player_list_quests.lock().keys().cloned().collect();
            for world in worlds {
                let player_count = game.login_world_player_num_by_name(&world);
                if player_count >= world_max_players {
                    continue;
                }
                let quest = self
                    .player_list_quests
                    .lock()
                    .get(&world)
                    .and_then(VecDeque::front)
                    .cloned();
                let Some(quest) = quest else {
                    continue;
                };
                regular_player_lists_processed += 1;
                let route = game.l2w_player_base_send(
                    &quest.world_server,
                    &quest.account,
                    self.is_no_queue_account(&quest.account),
                );
                match route {
                    Ok(true) => {
                        game.set_login_cdkey_world_server(&quest.account, &quest.world_server)
                    }
                    Ok(false) => {}
                    Err(error) => notices.push(LoginQueueRunNotice::Route {
                        stage: LoginQueueRunStage::RegularPlayerList,
                        account: quest.account,
                        socket_id: quest.socket_id,
                        error,
                    }),
                }
                if let Some(quests) = self.player_list_quests.lock().get_mut(&world) {
                    quests.pop_front();
                }
            }

            let worlds: Vec<_> = self.player_data_quests.lock().keys().cloned().collect();
            for world in worlds {
                let quest = self
                    .player_data_quests
                    .lock()
                    .get(&world)
                    .and_then(VecDeque::front)
                    .cloned();
                let Some(quest) = quest else {
                    continue;
                };
                regular_player_data_processed += 1;
                if let Err(error) = self.on_quest_player_data(game, &quest) {
                    notices.push(LoginQueueRunNotice::Route {
                        stage: LoginQueueRunStage::RegularPlayerData,
                        account: quest.account,
                        socket_id: quest.socket_id,
                        error,
                    });
                }
                if let Some(quests) = self.player_data_quests.lock().get_mut(&world) {
                    quests.pop_front();
                }
            }
        }

        let send_interval = self.setup.lock().send_message_interval_ms;
        let cdkey_positions: Vec<_> = {
            let mut quests = self.cdkey_quests.lock();
            quests
                .iter_mut()
                .zip(std::iter::successors(Some(1_i32), |position| {
                    Some(position.wrapping_add(1))
                }))
                .filter_map(|(quest, position)| {
                    (quest.send_message_time <= now).then(|| {
                        quest.send_message_time = now.wrapping_add(send_interval);
                        (quest.account.clone(), quest.socket_id, position)
                    })
                })
                .collect()
        };
        for (account, socket_id, position) in cdkey_positions {
            queue_position_messages += 1;
            if let Err(error) = send_queue_position(game, socket_id, position) {
                notices.push(LoginQueueRunNotice::Route {
                    stage: LoginQueueRunStage::QueuePosition,
                    account,
                    socket_id,
                    error,
                });
            }
        }

        let player_list_positions: Vec<_> = {
            let mut queues = self.player_list_quests.lock();
            queues
                .values_mut()
                .flat_map(|quests| {
                    quests
                        .iter_mut()
                        .zip(std::iter::successors(Some(1_i32), |position| {
                            Some(position.wrapping_add(1))
                        }))
                        .filter_map(|(quest, position)| {
                            (quest.send_message_time <= now).then(|| {
                                quest.send_message_time = now.wrapping_add(send_interval);
                                (quest.account.clone(), quest.socket_id, position)
                            })
                        })
                })
                .collect()
        };
        for (account, socket_id, position) in player_list_positions {
            queue_position_messages += 1;
            if let Err(error) = send_queue_position(game, socket_id, position) {
                notices.push(LoginQueueRunNotice::Route {
                    stage: LoginQueueRunStage::QueuePosition,
                    account,
                    socket_id,
                    error,
                });
            }
        }

        let player_data_positions: Vec<_> = {
            let mut queues = self.player_data_quests.lock();
            queues
                .values_mut()
                .flat_map(|quests| {
                    quests
                        .iter_mut()
                        .zip(std::iter::successors(Some(1_i32), |position| {
                            Some(position.wrapping_add(1))
                        }))
                        .filter_map(|(quest, position)| {
                            (quest.send_message_time <= now).then(|| {
                                quest.send_message_time = now.wrapping_add(send_interval);
                                (quest.account.clone(), quest.socket_id, position)
                            })
                        })
                })
                .collect()
        };
        for (account, socket_id, position) in player_data_positions {
            queue_position_messages += 1;
            if let Err(error) = send_queue_position(game, socket_id, position) {
                notices.push(LoginQueueRunNotice::Route {
                    stage: LoginQueueRunStage::QueuePosition,
                    account,
                    socket_id,
                    error,
                });
            }
        }

        let login_timeouts_removed = self.clear_timeout_list(game.quest_player_data_interval_ms());
        let auth = game
            .auth_event_publisher()
            .map(|publisher| auth_manager.run(&publisher));
        let timeout_tail = self.run_timeout_tail(game, matrix_timeout_ms, valid_code_overtime_ms);

        LoginQueueRunReport {
            gas_cdkeys_processed,
            no_queue_cdkeys_discarded_by_gas_bug,
            no_queue_cdkeys_processed,
            no_queue_player_lists_processed,
            no_queue_player_data_processed,
            regular_cdkeys_processed,
            regular_player_lists_processed,
            regular_player_data_processed,
            queue_position_messages,
            login_timeouts_removed,
            pwd_checked,
            auth,
            timeout_tail,
            notices,
        }
    }

    /// Заменяет первое совпадение account и добавляет новый объект в хвост.
    ///
    /// `kick_out` вызывается только при duplicate, до удаления прежней записи
    /// и под queue-lock — ровно в исходной позиции `CGame::KickOut`.
    pub fn push_back_pwd_checked(
        &self,
        checked: TagPwdChecked,
        mut kick_out: impl FnMut(&[u8]),
    ) {
        let mut queue = self.pwd_checked.lock();
        if let Some(index) = queue
            .iter()
            .position(|pending| pending.account == checked.account)
        {
            kick_out(queue[index].account());
            queue.remove(index);
        }
        queue.push_back(checked);
    }

    pub fn pwd_checked_len(&self) -> usize {
        self.pwd_checked.lock().len()
    }

    pub fn handle_pwd_checked(&self, game: &mut CGame) -> HandlePwdCheckedReport {
        let mut report = HandlePwdCheckedReport::new();
        let mut queue = self.pwd_checked.lock();
        if queue.is_empty() {
            return report;
        }
        while let Some(checked) = queue.pop_front() {
            report.processed += 1;
            if checked.socket_id == 0 || checked.client_ip == 0 {
                report.dropped_invalid_endpoint += 1;
                continue;
            }

            if self
                .valid_errors
                .lock()
                .get(checked.account())
                .is_some_and(|error| error.error_times >= game.valid_error_limit())
            {
                let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
                message.base_mut().add_char(b'Q' as i8);
                if let Err(error) = game.send_to_client(&message, checked.socket_id) {
                    report.notices.push(PwdCheckedNotice::ClientResponse {
                        socket_id: checked.socket_id,
                        error,
                    });
                }
                report.rejected_by_valid_errors += 1;
                continue;
            }

            if game.valid_code_enabled() {
                if let Some(previous_socket) = self
                    .valid_codes
                    .lock()
                    .get(checked.account())
                    .map(|entry| entry.socket_id)
                {
                    self.send_client_code(game, previous_socket, b'N', &mut report.notices);
                }

                let valid_code = match CValidCode::generate(std::path::Path::new(".")) {
                    Ok(valid_code) => valid_code,
                    Err(error) => {
                        report.notices.push(PwdCheckedNotice::ValidCodeGeneration {
                            account: checked.account,
                            error,
                        });
                        continue;
                    }
                };
                let now = legacy_tick_ms();
                self.valid_codes.lock().insert(
                    checked.account.clone(),
                    TagValidCode {
                        socket_id: checked.socket_id,
                        client_ip: checked.client_ip,
                        added_time: now,
                        valid_code: valid_code.valid_code().to_vec(),
                        world_server: checked.world_server.clone(),
                        has_matrix: checked.has_matrix,
                        change_time: 0,
                    },
                );

                let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
                message.base_mut().add_char(b'J' as i8);
                add_legacy_string(&mut message, checked.account());
                message.base_mut().add_long(0x70B6);
                message.base_mut().add_ex(valid_code.bitmap());
                if let Err(error) = game.send_to_client(&message, checked.socket_id) {
                    report.notices.push(PwdCheckedNotice::ClientResponse {
                        socket_id: checked.socket_id,
                        error,
                    });
                }
                report.generated_valid_codes += 1;
                continue;
            }

            self.complete_checked_entry(game, &checked, &mut report.notices);
        }
        report
    }

    pub fn continue_validated_login(
        &self,
        game: &mut CGame,
        checked: &TagPwdChecked,
    ) -> Vec<PwdCheckedNotice> {
        let mut notices = Vec::new();
        self.complete_checked_entry(game, checked, &mut notices);
        notices
    }

    pub fn add_valid_error(&self, account: &[u8], stay_time_ms: u32) {
        let now = legacy_tick_ms();
        let next_login_time = now.wrapping_add(stay_time_ms);
        let mut valid_errors = self.valid_errors.lock();
        match valid_errors.get_mut(account) {
            Some(error) => {
                error.error_times = error.error_times.wrapping_add(1);
                error.next_login_time = next_login_time;
            }
            None => {
                valid_errors.insert(
                    account.to_vec(),
                    TagValidErr {
                        error_times: 1,
                        next_login_time,
                    },
                );
            }
        }
    }

    pub fn clear_expired_valid_errors(&self, now: u32) {
        self.valid_errors
            .lock()
            .retain(|_, error| error.next_login_time >= now);
    }

    /// Выполняет timeout-хвост `Run` после промежуточных стадий caller-а.
    ///
    /// Три вызова boot clock намеренно не объединены: исходник вызывал
    /// `timeGetTime` отдельно перед matrix, valid-code и valid-error проверкой.
    pub fn run_timeout_tail(
        &self,
        game: &CGame,
        matrix_timeout_ms: u32,
        valid_code_overtime_ms: u32,
    ) -> LoginQueueTimeoutReport {
        let mut report = LoginQueueTimeoutReport {
            matrices_ran: false,
            valid_codes_ran: false,
            valid_errors_ran: false,
            notices: Vec::new(),
        };

        let now = legacy_tick_ms();
        let run_matrices = {
            let mut cadence = self.cadence.lock();
            if cadence.matrices_last_timeout.wrapping_add(1_000) < now
                && !self.matrices.lock().is_empty()
            {
                cadence.matrices_last_timeout = now;
                true
            } else {
                false
            }
        };
        if run_matrices {
            report.matrices_ran = true;
            report
                .notices
                .extend(self.expire_matrices(game, matrix_timeout_ms));
        }

        let now = legacy_tick_ms();
        let run_valid_codes = {
            let mut cadence = self.cadence.lock();
            if cadence.valid_code_last_overtime.wrapping_add(1_000) < now
                && !self.valid_codes.lock().is_empty()
            {
                cadence.valid_code_last_overtime = now;
                true
            } else {
                false
            }
        };
        if run_valid_codes {
            report.valid_codes_ran = true;
            report
                .notices
                .extend(self.expire_valid_codes(game, valid_code_overtime_ms));
        }

        let now = legacy_tick_ms();
        let run_valid_errors = {
            let mut cadence = self.cadence.lock();
            if cadence.valid_error_last_check.wrapping_add(3_000) < now {
                cadence.valid_error_last_check = now;
                true
            } else {
                false
            }
        };
        if run_valid_errors {
            report.valid_errors_ran = true;
            self.clear_expired_valid_errors(now);
        }

        report
    }

    /// Очищает и перечитывает `NoQueueAccounts.conf` как whitespace-token set.
    ///
    /// Каждый account приводится к ASCII lowercase и вставляется сразу после
    /// extraction, поэтому безопасная ошибка позднего token сохраняет уже
    /// выполненное частичное изменение. Ошибка открытия также оставляет set
    /// пустым, как исходный вызов до `ifstream::open`.
    pub fn load_no_queue_cdkey_list(
        &self,
        runtime_directory: &Path,
    ) -> Result<NoQueueAccountsLoadReport, NoQueueAccountsLoadError> {
        self.no_queue_accounts.lock().clear();
        let path = resolve_legacy_ascii_case(runtime_directory, "NoQueueAccounts.conf")?;
        let bytes = fs::read(path)?;
        let tokens = bytes
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .peekable();
        let mut extracted_accounts = 0;
        for (index, token) in tokens.enumerate() {
            let token_index = index + 1;
            if token.len() >= NO_QUEUE_ACCOUNT_BUFFER_SIZE {
                // `ESP+0xC8`; верхняя граница локала находится на `ESP+0x1C8`.
                // Старый `char[0x100]` не имел width и переполнялся вместе с NUL.
                return Err(NoQueueAccountsLoadError::TokenTooLong {
                    token_index,
                    actual: token.len(),
                    maximum: NO_QUEUE_ACCOUNT_BUFFER_SIZE - 1,
                });
            }
            let end = token
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(token.len());
            let mut account = token[..end].to_vec();
            // В C locale встроенный CRT меняет только ASCII `A..Z`.
            account.make_ascii_lowercase();
            self.no_queue_accounts.lock().insert(account);
            extracted_accounts += 1;
        }

        Ok(NoQueueAccountsLoadReport {
            extracted_accounts,
            unique_accounts: self.no_queue_accounts.lock().len(),
        })
    }

    pub fn check_message_info(&self, account: &[u8], socket_id: i32) -> CheckMessageInfo {
        let valid_codes = self.valid_codes.lock();
        let Some(entry) = valid_codes.get(account) else {
            return CheckMessageInfo::Missing;
        };
        if entry.socket_id != socket_id {
            return CheckMessageInfo::SocketMismatch;
        }
        if legacy_tick_ms() < entry.change_time.wrapping_add(1000) {
            CheckMessageInfo::Frequent
        } else {
            CheckMessageInfo::Success
        }
    }

    pub fn change_valid_code(&self, account: &[u8], valid_code: &[u8]) {
        if let Some(entry) = self.valid_codes.lock().get_mut(account) {
            entry.change_time = legacy_tick_ms();
            entry.valid_code.clear();
            entry.valid_code.extend_from_slice(valid_code);
        }
    }

    pub fn validate_valid_code(
        &self,
        socket_id: i32,
        client_ip: u32,
        supplied_code: &[u8],
        account: &[u8],
    ) -> ValidateValidCodeOutcome {
        let mut valid_codes = self.valid_codes.lock();
        let Some(entry) = valid_codes.get(account) else {
            return ValidateValidCodeOutcome::Rejected;
        };
        if entry.client_ip != client_ip || entry.socket_id != socket_id {
            valid_codes.remove(account);
            return ValidateValidCodeOutcome::Rejected;
        }
        if entry.valid_code != supplied_code {
            return ValidateValidCodeOutcome::Rejected;
        }
        let entry = valid_codes
            .remove(account)
            .expect("запись проверена под тем же valid-code lock");
        ValidateValidCodeOutcome::Accepted {
            world_server: entry.world_server,
            has_matrix: entry.has_matrix,
        }
    }

    pub fn delete_valid_code(&self, account: &[u8]) {
        self.valid_codes.lock().remove(account);
    }

    pub fn validate_matrix(
        &self,
        game: &mut CGame,
        socket_id: i32,
        client_ip: u32,
        account: &[u8],
        answer: &[u8; 3],
    ) -> MatrixValidationOutcome {
        let mut matrices = self.matrices.lock();
        let Some(entry) = matrices.get(account).copied() else {
            return MatrixValidationOutcome::Rejected;
        };
        if entry.client_ip != client_ip || entry.socket_id != socket_id {
            matrices.remove(account);
            return MatrixValidationOutcome::Rejected;
        }
        let Some(validation) = game.validate_matrix_card(account, &entry.positions, answer) else {
            // Без DB-owner продолжить вызов невозможно; запись не считается
            // проверенной и остаётся для штатного lifecycle.
            return MatrixValidationOutcome::DatabaseOwnerMissing;
        };
        let accepted = match validation {
            MatrixValidation::Compared(accepted) => accepted,
            // Короткий DB blob в оригинале приводил к out-of-bounds чтению.
            // Короткий blob считается неуспешной проверкой без чтения за границей.
            MatrixValidation::BlockedMatrixCardTooShort { .. } => false,
        };
        matrices.remove(account);
        if accepted {
            MatrixValidationOutcome::Accepted
        } else {
            MatrixValidationOutcome::Rejected
        }
    }

    /// Добавляет одну matrix-запись, только если endpoint и account допустимы.
    ///
    /// Нулевые socket/IP и существующий account возвращают `false` без
    /// мутации. Safe API исключает исходные nullable account/positions.
    pub fn add_matrix(
        &self,
        socket_id: i32,
        client_ip: u32,
        account: &[u8],
        positions: [u8; 3],
    ) -> bool {
        if socket_id == 0 || client_ip == 0 {
            return false;
        }
        let mut matrices = self.matrices.lock();
        if matrices.contains_key(account) {
            return false;
        }
        let added_time = legacy_tick_ms();
        matrices.insert(
            account.to_vec(),
            MatrixEntry {
                socket_id,
                client_ip,
                positions,
                added_time,
            },
        );
        true
    }

    pub fn is_no_queue_account(&self, account: &[u8]) -> bool {
        self.no_queue_accounts.lock().contains(account)
    }

    pub fn expire_valid_codes(
        &self,
        game: &CGame,
        overtime_ms: u32,
    ) -> Vec<PwdCheckedNotice> {
        let mut notices = Vec::new();
        let mut valid_codes = self.valid_codes.lock();
        valid_codes.retain(|_, entry| {
            if overtime_ms >= legacy_tick_ms().wrapping_sub(entry.added_time) {
                return true;
            }
            let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
            message.base_mut().add_char(b'M' as i8);
            if let Err(error) = game.send_to_client(&message, entry.socket_id) {
                notices.push(PwdCheckedNotice::ClientResponse {
                    socket_id: entry.socket_id,
                    error,
                });
            }
            false
        });
        notices
    }

    pub fn expire_matrices(&self, game: &CGame, timeout_ms: u32) -> Vec<PwdCheckedNotice> {
        let mut notices = Vec::new();
        let mut matrices = self.matrices.lock();
        matrices.retain(|_, entry| {
            if timeout_ms >= legacy_tick_ms().wrapping_sub(entry.added_time) {
                return true;
            }
            self.send_client_code(game, entry.socket_id, b'E', &mut notices);
            false
        });
        notices
    }

    fn send_client_code(
        &self,
        game: &CGame,
        socket_id: i32,
        code: u8,
        notices: &mut Vec<PwdCheckedNotice>,
    ) {
        let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
        message.base_mut().add_char(code as i8);
        if let Err(error) = game.send_to_client(&message, socket_id) {
            notices.push(PwdCheckedNotice::ClientResponse { socket_id, error });
        }
    }

    fn complete_checked_entry(
        &self,
        game: &mut CGame,
        checked: &TagPwdChecked,
        notices: &mut Vec<PwdCheckedNotice>,
    ) {
        let no_queue_account = self.is_no_queue_account(checked.account());
        match game.prepare_enter(checked) {
            Ok(PrepareEnterOutcome::Continue) => {
                if let Err(error) = game.enter_game(checked, no_queue_account) {
                    notices.push(PwdCheckedNotice::EnterGame {
                        account: checked.account().to_vec(),
                        error,
                    });
                }
            }
            Ok(PrepareEnterOutcome::Finished) => {}
            Ok(PrepareEnterOutcome::MatrixRegistrationRequired) => {
                self.matrix_register(game, checked, notices);
            }
            Err(error) => {
                notices.push(PwdCheckedNotice::PrepareEnter {
                    account: checked.account().to_vec(),
                    error,
                });
            }
        }
    }

    fn matrix_register(
        &self,
        game: &CGame,
        checked: &TagPwdChecked,
        notices: &mut Vec<PwdCheckedNotice>,
    ) {
        if checked.socket_id() == 0 || checked.client_ip() == 0 {
            return;
        }
        let positions = match random_matrix_positions() {
            Ok(positions) => positions,
            Err(error) => {
                notices.push(PwdCheckedNotice::MatrixRandom {
                    account: checked.account().to_vec(),
                    error,
                });
                return;
            }
        };

        if let Some(previous_socket) = self
            .matrices
            .lock()
            .get(checked.account())
            .map(|entry| entry.socket_id)
        {
            self.send_client_code(game, previous_socket, b'F', notices);
            self.matrices.lock().remove(checked.account());
        }
        let _legacy_add_result = self.add_matrix(
            checked.socket_id(),
            checked.client_ip(),
            checked.account(),
            positions,
        );

        let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
        message.base_mut().add_char(b'B' as i8);
        add_legacy_string(&mut message, checked.account());
        message.base_mut().add_ex(&positions);
        if let Err(error) = game.send_to_client(&message, checked.socket_id()) {
            notices.push(PwdCheckedNotice::ClientResponse {
                socket_id: checked.socket_id(),
                error,
            });
        }
    }
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u128).wrapping_mul(1000);
    let nanoseconds_ms = (now.tv_nsec as u128) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

fn random_matrix_positions() -> Result<[u8; 3], getrandom::Error> {
    Ok([
        (getrandom::u32()? % 0x50) as u8,
        (getrandom::u32()? % 0x50) as u8,
        (getrandom::u32()? % 0x50) as u8,
    ])
}

fn add_legacy_string(message: &mut CMessage, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.base_mut().add_ex(&value[..end]);
    message.base_mut().add_byte(0);
}

fn send_login_code(game: &CGame, socket_id: i32, code: i8) -> Result<(), GameRouteError> {
    let mut response = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
    response.base_mut().add_char(code);
    game.send_to_client(&response, socket_id)?;
    Ok(())
}

fn send_queue_position(game: &CGame, socket_id: i32, position: i32) -> Result<(), GameRouteError> {
    let mut response = CMessage::new(QUEUE_POSITION_MESSAGE_TYPE);
    response.base_mut().add_long(position);
    game.send_to_client(&response, socket_id)?;
    Ok(())
}

fn password_digest_hex(digest: &[u8]) -> Result<Vec<u8>, QuestCdkeyError> {
    if digest.len() < 16 {
        // Короткий внешний digest отклоняется до индексирования.
        return Err(QuestCdkeyError::PasswordDigestTooShort {
            actual_len: digest.len(),
        });
    }
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = Vec::with_capacity(32);
    for byte in &digest[..16] {
        encoded.push(HEX[usize::from(byte >> 4)]);
        encoded.push(HEX[usize::from(byte & 0x0f)]);
    }
    Ok(encoded)
}

fn legacy_lower_auth(value: &mut [u8]) -> Result<(), QuestCdkeyError> {
    if value.iter().any(|byte| !byte.is_ascii()) {
        // `CharLowerA` использовал внешнюю user-default ANSI locale Windows;
        // EXE/PDB не могут определить её таблицу для high-bit account bytes.
        return Err(QuestCdkeyError::AuthAnsiCaseMappingUnknown);
    }
    value.make_ascii_lowercase();
    Ok(())
}
