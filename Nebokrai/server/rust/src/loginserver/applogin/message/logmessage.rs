//! Client/World сообщения LoginServer из `logmessage.cpp`.
//!
//! Статус `OnLogMessage`: `IMPLEMENTED` для synthetic disconnect `0x10001`,
//! World opcode `0x1FF01..0x1FF07` и client opcode `0x2FD01..0x2FD0C`.
//! Неизвестный opcode проходит исходный default без side effects. Точная пара:
//! `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`, SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходный путь PDB:
//! `d:\complite_version\fengyun_russia\trunk\server\loginserver\applogin\message\logmessage.cpp`;
//! `OnLogMessage` RVA `0x0007F3F0`.
//!
//! `0x2FD09` сначала выполняет `CheckMsgInfo` над принятым account и немедленно
//! игнорирует только socket-mismatch. Затем непустой account приводится к
//! lowercase, читается ответ с границей `10` и вызывается `ValidateValidCode`.
//! Ошибка добавляет `ValidErr` до client-кода `L`; успех повторяет исходный
//! `DelValidCode` и продолжает `PrepareEnter -> EnterGame/matrix_register` с
//! сохранёнными world/matrix полями.
//!
//! `0x2FD0A` не меняет регистр account: `CHECK_SUCC` создаёт новую картинку,
//! меняет code/change-time и отправляет `J`; frequent даёт `O`, отсутствующая
//! запись — `P`, а socket-mismatch остаётся без ответа. `0x2FD08` читает
//! lowercase account и ровно три байта, вызывает одноразовый
//! `CLoginQueue::matirx_validate`, после `C` удаляет valid-code и продолжает
//! вход только при существующем `m_LoginCdkeyWorld`; `D` сначала увеличивает
//! `ValidErr`, затем отправляется клиенту.
//!
//! `0x2FD02` читает только byte-exact World-name с границей `0x14`, затем
//! передаёт его вместе с socket ID и metadata CD-key в
//! `AddQuestPlayerList`. `0x2FD03` сначала читает signed Windows `long`
//! player ID, затем передаёт socket ID, metadata CD-key и IPv4 в
//! `AddQuestPlayerData`. Обе исходные ветви игнорировали результат queue-
//! владельца; typed outcome делает тихое принятие/отказ наблюдаемым для
//! будущего runner, не меняя порядок либо side effects. Metadata CD-key во
//! всех этих C++-вызовах передавался как `char*`; единый helper сохраняет
//! byte-prefix только до первого NUL.
//!
//! `0x2FD04..0x2FD06` сначала получают nullable World-name по metadata CD-key;
//! delete/restore только после этого читают один 32-битный player ID; delete
//! сохраняет его как signed `long`, restore — как unsigned `uint`.
//! Create-role передаёт исходное сообщение владельцу, который сохраняет его
//! payload, меняет opcode на `0x4FB04` и дописывает account. Delete строит
//! `0x4FB02 + account + player ID + IPv4`, restore —
//! `0x4FB03 + account + player ID`. Отсутствующий либо отключённый World даёт
//! исходный no-op без нового client-ответа.
//!
//! `0x2FD07` всегда сначала читает signed status и при ненулевом значении
//! завершает ветвь без других эффектов. Ноль получает nullable World-name;
//! известному открытому World ставится `0x4FB06 + account`, но ошибка этой
//! исходно void-отправки не прерывает последующие `ClearCDKey -> OnClientLost
//! -> AccountLeaveLog`. Queue cleanup затрагивает только первые совпадения
//! обычных request FIFO. Typed outcome сохраняет проигнорированную transport-
//! ошибку, не меняя порядок cleanup.
//!
//! Synthetic `0x10001` читает account из payload с границей `0x20`, удаляет
//! его из login-map и ставит client `QUIT` до поиска World CD-key. Найденный
//! World ID вызывает исходный ранний возврат без account/queue cleanup. При
//! отсутствии World CD-key буквально сохраняется дальнейшая nullable цепочка
//! `GetLoginCdkeyWorldServer -> GetWorldIDByName`: из-за уже выполненного
//! `ClearLoginCdkey` она не находит World, но после неё всё равно выполняются
//! `AccountLeaveLog -> ClearCDKey -> OnClientLost`. Ошибки ещё не собранного
//! transport-owner фиксируются typed outcome и не меняют исходный порядок.
//!
//! Пять прямых World-ответов сначала пропускают доказанные status/player-
//! поля, затем читают account с границей `0x20`, меняют opcode того же
//! сообщения на `0xAF502/0xAF505/0xAF506/0xAF504/0xAF508` и отправляют его по
//! строковой client identity. Payload не пересобирается: cursor-чтение не
//! удаляет bytes, а `set_message_type` меняет только header, поэтому клиент
//! получает исходное тело WorldServer с новым типом. Связанный World-owner
//! подтверждает назначения первых четырёх ответов как
//! player-base/delete/restore/create; для `0x1FF07` текущий raw доказывает
//! только account-only relay в `0xAF508`, поэтому доменное имя не назначается.
//!
//! `0x1FF01` всегда читает status и account, затем сохраняет World map ID из
//! metadata. Только status `0x1D` сначала вызывает `AddCdkey`, после чего
//! последовательно читает дополнительную строку, `long`, role name и role
//! level. Первые два поля handler не использует; вторые два вместе с account
//! попадают в `RoleEnterLog`, причём World number исходно усекался до младшего
//! octet через `map_id & 0xFF`. После условной ветви сообщение независимо от
//! status меняет opcode на `0xAF503` и отправляется по account identity.
//! Role-enter сохраняется typed-вариантом общей `_acc_logs` до её будущего
//! consumer; cursor-чтение не меняет пересылаемый payload.
//!
//! `0x1FF06` читает account с границей `0x20`, затем буквально выполняет
//! `ClearCDKey`, потребляет дополнительную строку `0x100` и один `char` и
//! вызывает `LeaveLog(account)`. Два последних payload-поля не используются,
//! а client-send в этой ветви отсутствует. `LeaveLog` ставит typed-запись в
//! общую `_acc_logs` после очистки CD-key.
//!
//! Client login-request `0x2FD01` читает два `long` как marker/version,
//! удаляет из account все пробелы и принимает только длину `1..=31`. Затем
//! обязательный `GetEx(..., 0x10)` даёт digest; marker должен быть `6`, а
//! version — совпадать с setup. Оба несоответствия отправляют client-код `4`,
//! наличие `'`, `=` либо пробела — код `5`; неверная длина account/digest
//! остаётся исходным тихим возвратом. После проверок последовательно читаются
//! `short` client code, `long` encryption key, неиспользуемая строка `0x40` и
//! World `0x14`; только затем account приводится к lowercase и полный owner-
//! набор ставится в `CLoginQueue::AddQuestCdkey` с login type `0`.
//! Неустойчивая decompiler dataflow имеет статус `VERIFIED_DISASSEMBLY`:
//! reads/checks находятся в `0x0047F825..0x0047F92D`, а точный порядок
//! аргументов вызова — в `0x0047FB02..0x0047FB29`.
//!
//! Расширенный login-вариант `0x2FD0B` читает marker/version, World `0x20`,
//! account `0x20` и password-source `0x104`. Пустой password вызывает тихий
//! возврат раньше пустого account; затем из account удаляются пробелы без
//! повторной проверки и без lowercase. Marker/version проверяются уже после
//! сборки digest; отказ отправляет `0xAF50A + long(6) + "" + ""`. Успех
//! ставит тот же queue-owner с login type `1`, нулевыми client code/key и
//! принятым World. Странность digest имеет статус `VERIFIED_DISASSEMBLY`:
//! `0x0047FF70..0x0047FFAF` берёт длину у password без пробелов, но копирует
//! prefix исходного buffer и добавляет NUL только при отсутствии пробелов.
//! Exact вызов `AddQuestCdkey` подтверждён `0x00480053..0x00480082`.
//!
//! Малый запрос списка миров `0x2FD0C` читает account с границей `0x20` и
//! оставляет пустое значение без ответа. Для непустого account он строит
//! `0xAF50B + account`, затем фактический owner `AddWorldInfoToMsg` дописывает
//! число и записи миров с исходной no-queue проверкой и только после этого
//! сообщение отправляется по socket ID принятого запроса.
//!
//! `Vec`, owned message и типизированные outcomes заменяют stack-массивы,
//! `std::string`, SEH и ручные деструкторы. Сырые участки всех двадцати case и
//! их compiler-generated cleanup-блоки удалены после переноса полезного эффекта.

use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::path::Path;

use crate::loginserver::applogin::validcode::{CValidCode, VALID_CODE_BITMAP_LEN, ValidCodeError};
use crate::loginserver::loginserver::game::{CGame, GameRouteError};
use crate::loginserver::loginserver::loginqueue::{
    CheckMessageInfo, ClientLostCleanupReport, MatrixValidationOutcome, PwdCheckedNotice,
    TagPwdChecked, ValidateValidCodeOutcome,
};
use crate::nets::basemessage::CBaseMessage;
use crate::nets::netlogin::message::CMessage;

const LOGIN_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F501;
const SYNTHETIC_DISCONNECT_MESSAGE_TYPE: i32 = 0x0001_0001;
const WORLD_LOGIN_RESULT_MESSAGE_TYPE: i32 = 0x0001_FF01;
const WORLD_PLAYER_BASE_RESPONSE_MESSAGE_TYPE: i32 = 0x0001_FF02;
const WORLD_DELETE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x0001_FF03;
const WORLD_RESTORE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x0001_FF04;
const WORLD_CREATE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x0001_FF05;
const WORLD_LEAVE_RESULT_MESSAGE_TYPE: i32 = 0x0001_FF06;
const WORLD_RESPONSE_1FF07_MESSAGE_TYPE: i32 = 0x0001_FF07;
const CLIENT_LOGIN_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD01;
const EXTENDED_LOGIN_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD0B;
const PLAYER_LIST_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD02;
const PLAYER_DATA_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD03;
const CREATE_ROLE_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD04;
const DELETE_ROLE_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD05;
const RESTORE_ROLE_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD06;
const CLIENT_CLEANUP_MESSAGE_TYPE: i32 = 0x0002_FD07;
const MATRIX_ANSWER_MESSAGE_TYPE: i32 = 0x0002_FD08;
const VALID_CODE_ANSWER_MESSAGE_TYPE: i32 = 0x0002_FD09;
const VALID_CODE_REFRESH_MESSAGE_TYPE: i32 = 0x0002_FD0A;
const WORLD_LIST_REQUEST_MESSAGE_TYPE: i32 = 0x0002_FD0C;
const WORLD_CLIENT_LOST_MESSAGE_TYPE: i32 = 0x0004_FB06;
const CLIENT_LOGIN_RESULT_MESSAGE_TYPE: i32 = 0x000A_F503;
const CLIENT_PLAYER_BASE_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F502;
const CLIENT_CREATE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F504;
const CLIENT_DELETE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F505;
const CLIENT_RESTORE_ROLE_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F506;
const CLIENT_RESPONSE_AF508_MESSAGE_TYPE: i32 = 0x000A_F508;
const EXTENDED_LOGIN_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F50A;
const WORLD_LIST_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F50B;
const ACCOUNT_LIMIT: usize = 0x14;
const EXTENDED_ACCOUNT_LIMIT: usize = 0x20;
const EXTENDED_ROLE_FIELD_LIMIT: usize = 0x100;
const WORLD_SERVER_LIMIT: usize = 0x14;
const VALID_CODE_LIMIT: usize = 10;
const WORLD_LOGIN_SUCCESS_STATUS: i8 = 0x1D;
const CLIENT_LOGIN_MARKER: i32 = 6;
const PASSWORD_DIGEST_LEN: usize = 0x10;
const LOGIN_UNUSED_FIELD_LIMIT: usize = 0x40;
const EXTENDED_LOGIN_WORLD_LIMIT: usize = 0x20;
const EXTENDED_LOGIN_PASSWORD_LIMIT: usize = 0x104;

/// Наблюдаемый результат одной из восстановленных ветвей `OnLogMessage`.
#[derive(Debug)]
pub(crate) enum LogMessageOutcome {
    /// Ветка выполнила все достижимые side effects.
    Handled {
        /// Неуспешные send/RNG side effects продолжения password-check.
        notices: Vec<PwdCheckedNotice>,
    },
    /// Запрос списка персонажей передан queue-owner либо тихо им отклонён.
    PlayerListQuest { accepted: bool },
    /// Запрос данных персонажа передан queue-owner либо тихо им отклонён.
    PlayerDataQuest { accepted: bool, player_id: i32 },
    /// Create-role payload переслан выбранному World либо дал исходный no-op.
    CreateRole { forwarded: bool },
    /// Delete-role запрос переслан выбранному World либо дал исходный no-op.
    DeleteRole { forwarded: bool, player_id: i32 },
    /// Restore-role запрос переслан выбранному World либо дал исходный no-op.
    RestoreRole { forwarded: bool, player_id: u32 },
    /// Ненулевой status завершил `0x2FD07` до любых cleanup-эффектов.
    ClientCleanupSkipped { status: i32 },
    /// Нулевая ветвь выполнила полный cleanup независимо от World send.
    ClientCleanup {
        world_notified: bool,
        world_notice_error: Option<GameRouteError>,
        queue: ClientLostCleanupReport,
    },
    /// Synthetic disconnect нашёл account в World-списке и вернулся раньше cleanup.
    SyntheticDisconnectWorldCdkeyPresent {
        login_mapping_removed: bool,
        client_quit_error: Option<GameRouteError>,
        world_id: i32,
    },
    /// Synthetic disconnect не нашёл World CD-key и выполнил оставшийся cleanup.
    SyntheticDisconnectCleaned {
        login_mapping_removed: bool,
        client_quit_error: Option<GameRouteError>,
        world_notified: bool,
        world_notice_error: Option<GameRouteError>,
        queue: ClientLostCleanupReport,
    },
    /// World payload переслан исходному client identity после in-place opcode.
    WorldClientRelay { response_type: i32 },
    /// World login-result переслан после условных CD-key/role-log side effects.
    WorldLoginResult {
        status: i8,
        cdkey_added: Option<bool>,
        role_logged: bool,
    },
    /// World leave-result выполнил cleanup и поставил leave-запись без send.
    WorldLeaveResult,
    /// Account после удаления пробелов не попал в исходную длину `1..=31`.
    LoginRequestAccountIgnored,
    /// `GetEx(..., 0x10)` доказанно вернул null без client-ответа.
    LoginRequestPasswordIgnored,
    /// Marker/version/account-проверка отправила исходный client-код.
    LoginRequestRejected { response_code: u8 },
    /// Полный login-request поставлен фактическому queue-owner.
    LoginRequestQueued,
    /// `0x2FD0B` получил пустой password-source и вернулся раньше account.
    ExtendedLoginPasswordIgnored,
    /// `0x2FD0B` получил пустой account до удаления пробелов.
    ExtendedLoginAccountIgnored,
    /// Marker либо version вызвали точный ответ `0xAF50A`.
    ExtendedLoginRejected,
    /// Полный `0x2FD0B` поставлен queue-owner с login type `1`.
    ExtendedLoginQueued,
    /// `0x2FD0C` получил пустой account и оставил запрос без ответа.
    WorldListRequestIgnored,
    /// `0x2FD0C` отправил account и фактический список миров по исходному socket.
    WorldListSent,
    /// `CheckMsgInfo` доказанно оставлял socket-mismatch без ответа.
    SocketMismatchIgnored,
    /// Matrix/valid-code handler получил пустой account и только диагностировал его.
    EmptyAccountIgnored,
    /// Malformed payload безопасно отклонён без чтения за его границей.
    MalformedPayloadIgnored,
    /// Успешная matrix-проверка не нашла обязательную login/world запись.
    LoginStateMissing,
    /// Для opcode отсутствует case в исходном `OnLogMessage`.
    Unsupported { message_type: i32 },
}

/// Ошибка безопасной границы восстановленных ветвей `OnLogMessage`.
#[derive(Debug)]
pub(crate) enum LogMessageError {
    /// Фактический Client/World transport-owner не выполнил обязательный send.
    Route(GameRouteError),
    /// Новую valid-code картинку нельзя построить из runtime-ресурсов.
    ValidCode(ValidCodeError),
    /// Positional Login setup не определил сравниваемую client version.
    LoginServerVersionMissing,
    /// Частичный Login lifecycle ещё не присоединил исходный `CRsCDKey`.
    MatrixDatabaseOwnerMissing,
}

impl fmt::Display for LogMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Route(error) => error.fmt(formatter),
            Self::ValidCode(error) => error.fmt(formatter),
            Self::LoginServerVersionMissing => {
                formatter.write_str("Login setup не определил server_version")
            }
            Self::MatrixDatabaseOwnerMissing => {
                formatter.write_str("DB-владелец CRsCDKey для matrix-проверки отсутствует")
            }
        }
    }
}

impl Error for LogMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Route(error) => Some(error),
            Self::ValidCode(error) => Some(error),
            Self::LoginServerVersionMissing
            | Self::MatrixDatabaseOwnerMissing => None,
        }
    }
}

impl From<GameRouteError> for LogMessageError {
    fn from(error: GameRouteError) -> Self {
        Self::Route(error)
    }
}

impl From<ValidCodeError> for LogMessageError {
    fn from(error: ValidCodeError) -> Self {
        Self::ValidCode(error)
    }
}

/// Узкая композиция `OnLogMessage` с фактическим `CGame` LoginServer.
pub(crate) struct LogMessageHandler<'a> {
    game: &'a mut CGame,
}

impl<'a> LogMessageHandler<'a> {
    /// Связывает client-handler с текущим LoginServer owner.
    pub(crate) fn new(game: &'a mut CGame) -> Self {
        Self { game }
    }

    /// Выполняет двадцать восстановленных opcode; неизвестные повторяют default no-op.
    pub(crate) fn on_log_message(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        match message.message_type() {
            SYNTHETIC_DISCONNECT_MESSAGE_TYPE => Ok(self.on_synthetic_disconnect(message)),
            WORLD_LOGIN_RESULT_MESSAGE_TYPE => self.on_world_login_result(message),
            WORLD_PLAYER_BASE_RESPONSE_MESSAGE_TYPE => self.on_player_base_response(message),
            WORLD_DELETE_ROLE_RESPONSE_MESSAGE_TYPE => self.on_delete_role_response(message),
            WORLD_RESTORE_ROLE_RESPONSE_MESSAGE_TYPE => self.on_restore_role_response(message),
            WORLD_CREATE_ROLE_RESPONSE_MESSAGE_TYPE => self.on_create_role_response(message),
            WORLD_LEAVE_RESULT_MESSAGE_TYPE => Ok(self.on_world_leave_result(message)),
            WORLD_RESPONSE_1FF07_MESSAGE_TYPE => self.on_world_response_1ff07(message),
            CLIENT_LOGIN_REQUEST_MESSAGE_TYPE => self.on_client_login_request(message),
            EXTENDED_LOGIN_REQUEST_MESSAGE_TYPE => self.on_extended_login_request(message),
            PLAYER_LIST_REQUEST_MESSAGE_TYPE => Ok(self.on_player_list_request(message)),
            PLAYER_DATA_REQUEST_MESSAGE_TYPE => Ok(self.on_player_data_request(message)),
            CREATE_ROLE_REQUEST_MESSAGE_TYPE => self.on_create_role(message),
            DELETE_ROLE_REQUEST_MESSAGE_TYPE => self.on_delete_role(message),
            RESTORE_ROLE_REQUEST_MESSAGE_TYPE => self.on_restore_role(message),
            CLIENT_CLEANUP_MESSAGE_TYPE => Ok(self.on_client_cleanup(message)),
            MATRIX_ANSWER_MESSAGE_TYPE => self.on_matrix_answer(message),
            VALID_CODE_ANSWER_MESSAGE_TYPE => self.on_valid_code_answer(message),
            VALID_CODE_REFRESH_MESSAGE_TYPE => self.on_valid_code_refresh(message),
            WORLD_LIST_REQUEST_MESSAGE_TYPE => self.on_world_list_request(message),
            message_type => Ok(LogMessageOutcome::Unsupported { message_type }),
        }
    }

    fn on_world_login_result(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let status = message.base_mut().get_char().unwrap_or(0);
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        let world_id = message.map_id();

        let cdkey_added = if status == WORLD_LOGIN_SUCCESS_STATUS {
            let added = self.game.add_cdkey(&account, world_id);
            let _unused_role_field =
                get_bounded_string(message.base_mut(), EXTENDED_ROLE_FIELD_LIMIT);
            let _unused_role_number = message.base_mut().get_long().unwrap_or(0);
            let role_name = get_bounded_string(message.base_mut(), EXTENDED_ROLE_FIELD_LIMIT);
            let role_level = message.base_mut().get_char().unwrap_or(0) as u8;
            self.game.role_enter_log(
                &account,
                &role_name,
                role_level,
                world_id & i32::from(u8::MAX),
            );
            Some(added)
        } else {
            None
        };

        message.set_message_type(CLIENT_LOGIN_RESULT_MESSAGE_TYPE);
        self.game.send_to_client_cdkey(message, &account)?;
        Ok(LogMessageOutcome::WorldLoginResult {
            status,
            cdkey_added,
            role_logged: status == WORLD_LOGIN_SUCCESS_STATUS,
        })
    }

    fn on_player_base_response(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let _status = message.base_mut().get_char().unwrap_or(0);
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.relay_world_response(message, &account, CLIENT_PLAYER_BASE_RESPONSE_MESSAGE_TYPE)
    }

    fn on_world_leave_result(&mut self, message: &mut CMessage) -> LogMessageOutcome {
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.game.clear_cdkey(&account);
        let _unused_role_field = get_bounded_string(message.base_mut(), EXTENDED_ROLE_FIELD_LIMIT);
        let _unused_status = message.base_mut().get_char().unwrap_or(0);
        self.game.leave_log(&account);
        LogMessageOutcome::WorldLeaveResult
    }

    fn on_client_login_request(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let marker = message.base_mut().get_long().unwrap_or(0);
        let client_version = message.base_mut().get_long().unwrap_or(0);
        let mut account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        account.retain(|byte| *byte != b' ');
        if account.is_empty() || account.len() >= EXTENDED_ACCOUNT_LIMIT {
            return Ok(LogMessageOutcome::LoginRequestAccountIgnored);
        }

        let Some(password_digest) = get_password_digest(message.base_mut())? else {
            return Ok(LogMessageOutcome::LoginRequestPasswordIgnored);
        };
        if marker != CLIENT_LOGIN_MARKER {
            send_code(self.game, message.socket_id(), 4)?;
            return Ok(LogMessageOutcome::LoginRequestRejected { response_code: 4 });
        }

        let server_version = self
            .game
            .login_setup()
            .server_version()
            .ok_or(LogMessageError::LoginServerVersionMissing)?;
        if client_version != server_version {
            send_code(self.game, message.socket_id(), 4)?;
            return Ok(LogMessageOutcome::LoginRequestRejected { response_code: 4 });
        }
        if account
            .iter()
            .any(|byte| matches!(byte, b'\'' | b'=' | b' '))
        {
            send_code(self.game, message.socket_id(), 5)?;
            return Ok(LogMessageOutcome::LoginRequestRejected { response_code: 5 });
        }

        let client_code = message.base_mut().get_short().unwrap_or(0);
        let encryption_key = message.base_mut().get_long().unwrap_or(0);
        let _unused_login_field = get_bounded_string(message.base_mut(), LOGIN_UNUSED_FIELD_LIMIT);
        let world_server = get_bounded_string(message.base_mut(), WORLD_SERVER_LIMIT);
        legacy_lower_account(&mut account);

        self.game.login_queue().add_quest_cdkey(
            message.socket_id(),
            message.ip(),
            0,
            client_version,
            account,
            password_digest,
            client_code,
            encryption_key,
            world_server,
        );
        Ok(LogMessageOutcome::LoginRequestQueued)
    }

    fn on_extended_login_request(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let marker = message.base_mut().get_long().unwrap_or(0);
        let client_version = message.base_mut().get_long().unwrap_or(0);
        let world_server = get_bounded_string(message.base_mut(), EXTENDED_LOGIN_WORLD_LIMIT);
        let mut account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        let password_source = get_bounded_string(message.base_mut(), EXTENDED_LOGIN_PASSWORD_LIMIT);
        if password_source.is_empty() {
            return Ok(LogMessageOutcome::ExtendedLoginPasswordIgnored);
        }
        if account.is_empty() {
            return Ok(LogMessageOutcome::ExtendedLoginAccountIgnored);
        }
        account.retain(|byte| *byte != b' ');

        let compacted_len = password_source.iter().filter(|byte| **byte != b' ').count();
        // Login RVA 0x0007F3F0, exact `0x0047FF70..0x0047FFAF`: оригинал
        // использует длину очищенной std::string, но индексирует исходный
        // stack-buffer. Поэтому пробел сокращает digest с хвоста, а не в своей
        // позиции; NUL добавляется только когда длина вообще не изменилась.
        let mut password_digest = password_source[..compacted_len].to_vec();
        if compacted_len == password_source.len() {
            password_digest.push(0);
        }

        if marker != CLIENT_LOGIN_MARKER {
            send_extended_login_rejection(self.game, message.socket_id())?;
            return Ok(LogMessageOutcome::ExtendedLoginRejected);
        }
        let server_version = self
            .game
            .login_setup()
            .server_version()
            .ok_or(LogMessageError::LoginServerVersionMissing)?;
        if client_version != server_version {
            send_extended_login_rejection(self.game, message.socket_id())?;
            return Ok(LogMessageOutcome::ExtendedLoginRejected);
        }

        self.game.login_queue().add_quest_cdkey(
            message.socket_id(),
            message.ip(),
            1,
            client_version,
            account,
            password_digest,
            0,
            0,
            world_server,
        );
        Ok(LogMessageOutcome::ExtendedLoginQueued)
    }

    fn on_delete_role_response(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let _status = message.base_mut().get_char().unwrap_or(0);
        let _player_id = message.base_mut().get_long().unwrap_or(0);
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.relay_world_response(message, &account, CLIENT_DELETE_ROLE_RESPONSE_MESSAGE_TYPE)
    }

    fn on_restore_role_response(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let _status = message.base_mut().get_char().unwrap_or(0);
        let _player_id = message.base_mut().get_long().unwrap_or(0);
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.relay_world_response(message, &account, CLIENT_RESTORE_ROLE_RESPONSE_MESSAGE_TYPE)
    }

    fn on_create_role_response(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let _status = message.base_mut().get_char().unwrap_or(0);
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.relay_world_response(message, &account, CLIENT_CREATE_ROLE_RESPONSE_MESSAGE_TYPE)
    }

    fn on_world_response_1ff07(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        self.relay_world_response(message, &account, CLIENT_RESPONSE_AF508_MESSAGE_TYPE)
    }

    fn relay_world_response(
        &self,
        message: &mut CMessage,
        account: &[u8],
        response_type: i32,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        message.set_message_type(response_type);
        self.game.send_to_client_cdkey(message, account)?;
        Ok(LogMessageOutcome::WorldClientRelay { response_type })
    }

    fn on_synthetic_disconnect(&mut self, message: &mut CMessage) -> LogMessageOutcome {
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        let login_mapping_removed = self.game.clear_login_cdkey(&account);
        let client_quit_error = self.game.quit_client_by_cdkey(&account).err();

        let world_id = self.game.find_cdkey(&account);
        if world_id != -1 {
            return LogMessageOutcome::SyntheticDisconnectWorldCdkeyPresent {
                login_mapping_removed,
                client_quit_error,
                world_id,
            };
        }

        let world_id = self
            .game
            .login_cdkey_world_server(&account)
            .map_or(-1, |world_name| self.game.world_id_by_name(world_name));
        let (world_notified, world_notice_error) = if world_id == -1 {
            (false, None)
        } else {
            let mut notice = CMessage::new(WORLD_CLIENT_LOST_MESSAGE_TYPE);
            add_legacy_string(notice.base_mut(), &account);
            match self.game.send_msg_to_world(&notice, world_id) {
                Ok(_) => (true, None),
                Err(error) => (false, Some(error)),
            }
        };

        self.game.account_leave_log(&account);
        self.game.clear_cdkey(&account);
        let queue = self.game.login_queue().on_client_lost(&account);
        LogMessageOutcome::SyntheticDisconnectCleaned {
            login_mapping_removed,
            client_quit_error,
            world_notified,
            world_notice_error,
            queue,
        }
    }

    fn on_player_list_request(&mut self, message: &mut CMessage) -> LogMessageOutcome {
        let world_server = get_bounded_string(message.base_mut(), WORLD_SERVER_LIMIT);
        let account = legacy_c_string_prefix(message.cdkey());
        let queue = self.game.login_queue();
        let accepted =
            queue.add_quest_player_list(self.game, message.socket_id(), account, &world_server);
        LogMessageOutcome::PlayerListQuest { accepted }
    }

    fn on_player_data_request(&mut self, message: &mut CMessage) -> LogMessageOutcome {
        let player_id = message.base_mut().get_long().unwrap_or(0);
        let account = legacy_c_string_prefix(message.cdkey());
        let queue = self.game.login_queue();
        let accepted = queue.add_quest_player_data(
            self.game,
            message.socket_id(),
            account,
            player_id,
            message.ip(),
        );
        LogMessageOutcome::PlayerDataQuest {
            accepted,
            player_id,
        }
    }

    fn on_create_role(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = legacy_c_string_prefix(message.cdkey()).to_vec();
        let world_server = self
            .game
            .login_cdkey_world_server(&account)
            .map(<[u8]>::to_vec);
        let forwarded =
            self.game
                .l2w_create_role_send(world_server.as_deref(), &account, message)?;
        Ok(LogMessageOutcome::CreateRole { forwarded })
    }

    fn on_delete_role(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = legacy_c_string_prefix(message.cdkey()).to_vec();
        let world_server = self
            .game
            .login_cdkey_world_server(&account)
            .map(<[u8]>::to_vec);
        let player_id = message.base_mut().get_long().unwrap_or(0);
        let forwarded = self.game.l2w_delete_role_send(
            world_server.as_deref(),
            &account,
            player_id,
            message.ip(),
        )?;
        Ok(LogMessageOutcome::DeleteRole {
            forwarded,
            player_id,
        })
    }

    fn on_restore_role(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = legacy_c_string_prefix(message.cdkey()).to_vec();
        let world_server = self
            .game
            .login_cdkey_world_server(&account)
            .map(<[u8]>::to_vec);
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let forwarded =
            self.game
                .l2w_restore_role_send(world_server.as_deref(), &account, player_id)?;
        Ok(LogMessageOutcome::RestoreRole {
            forwarded,
            player_id,
        })
    }

    fn on_client_cleanup(&mut self, message: &mut CMessage) -> LogMessageOutcome {
        let status = message.base_mut().get_long().unwrap_or(0);
        if status != 0 {
            return LogMessageOutcome::ClientCleanupSkipped { status };
        }

        let account = legacy_c_string_prefix(message.cdkey()).to_vec();
        let world_id = self
            .game
            .login_cdkey_world_server(&account)
            .map_or(-1, |world_name| self.game.world_id_by_name(world_name));
        let (world_notified, world_notice_error) = if world_id == -1 {
            (false, None)
        } else {
            let mut notice = CMessage::new(WORLD_CLIENT_LOST_MESSAGE_TYPE);
            add_legacy_string(notice.base_mut(), &account);
            match self.game.send_msg_to_world(&notice, world_id) {
                Ok(_) => (true, None),
                Err(error) => (false, Some(error)),
            }
        };

        self.game.clear_cdkey(&account);
        let queue = self.game.login_queue().on_client_lost(&account);
        self.game.account_leave_log(&account);
        LogMessageOutcome::ClientCleanup {
            world_notified,
            world_notice_error,
            queue,
        }
    }

    fn on_matrix_answer(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let mut account = get_bounded_string(message.base_mut(), ACCOUNT_LIMIT);
        if account.is_empty() {
            return Ok(LogMessageOutcome::EmptyAccountIgnored);
        }
        legacy_lower_account(&mut account);

        let mut answer = [0; 3];
        if !message.base_mut().get(&mut answer) {
            return Ok(LogMessageOutcome::MalformedPayloadIgnored);
        }

        let queue = self.game.login_queue();
        match queue.validate_matrix(
            self.game,
            message.socket_id(),
            message.ip(),
            &account,
            &answer,
        ) {
            MatrixValidationOutcome::Accepted => {
                queue.delete_valid_code(&account);
                let no_queue_account = queue.is_no_queue_account(&account);
                if !self.game.enter_after_matrix(
                    &account,
                    message.ip(),
                    message.socket_id(),
                    no_queue_account,
                )? {
                    return Ok(LogMessageOutcome::LoginStateMissing);
                }
                Ok(handled())
            }
            MatrixValidationOutcome::Rejected => {
                queue.add_valid_error(&account, self.game.valid_error_stay_time_ms());
                send_code(self.game, message.socket_id(), b'D')?;
                Ok(handled())
            }
            MatrixValidationOutcome::DatabaseOwnerMissing => {
                Err(LogMessageError::MatrixDatabaseOwnerMissing)
            }
        }
    }

    fn on_valid_code_answer(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let mut account = get_bounded_string(message.base_mut(), ACCOUNT_LIMIT);
        let queue = self.game.login_queue();
        if queue.check_message_info(&account, message.socket_id())
            == CheckMessageInfo::SocketMismatch
        {
            return Ok(LogMessageOutcome::SocketMismatchIgnored);
        }
        if account.is_empty() {
            return Ok(LogMessageOutcome::EmptyAccountIgnored);
        }
        legacy_lower_account(&mut account);
        let supplied_code = get_bounded_string(message.base_mut(), VALID_CODE_LIMIT);

        match queue.validate_valid_code(message.socket_id(), message.ip(), &supplied_code, &account)
        {
            ValidateValidCodeOutcome::Accepted {
                world_server,
                has_matrix,
            } => {
                queue.delete_valid_code(&account);
                let checked = TagPwdChecked::new(
                    message.socket_id(),
                    message.ip(),
                    account,
                    world_server,
                    has_matrix,
                );
                let notices = queue.continue_validated_login(self.game, &checked);
                Ok(LogMessageOutcome::Handled { notices })
            }
            ValidateValidCodeOutcome::Rejected => {
                queue.add_valid_error(&account, self.game.valid_error_stay_time_ms());
                send_code(self.game, message.socket_id(), b'L')?;
                Ok(handled())
            }
        }
    }

    fn on_valid_code_refresh(
        &mut self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = get_bounded_string(message.base_mut(), ACCOUNT_LIMIT);
        let queue = self.game.login_queue();
        match queue.check_message_info(&account, message.socket_id()) {
            CheckMessageInfo::Success => {
                let valid_code = CValidCode::generate(Path::new("."))?;
                queue.change_valid_code(&account, valid_code.valid_code());

                let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
                response.base_mut().add_char(b'J' as i8);
                add_legacy_string(response.base_mut(), &account);
                response.base_mut().add_long(VALID_CODE_BITMAP_LEN as i32);
                response.base_mut().add_ex(valid_code.bitmap());
                self.game.send_to_client(&response, message.socket_id())?;
                Ok(handled())
            }
            CheckMessageInfo::Frequent => {
                send_code(self.game, message.socket_id(), b'O')?;
                Ok(handled())
            }
            CheckMessageInfo::Missing => {
                send_code(self.game, message.socket_id(), b'P')?;
                Ok(handled())
            }
            CheckMessageInfo::SocketMismatch => Ok(LogMessageOutcome::SocketMismatchIgnored),
        }
    }

    fn on_world_list_request(
        &self,
        message: &mut CMessage,
    ) -> Result<LogMessageOutcome, LogMessageError> {
        let account = get_bounded_string(message.base_mut(), EXTENDED_ACCOUNT_LIMIT);
        if account.is_empty() {
            return Ok(LogMessageOutcome::WorldListRequestIgnored);
        }

        let mut response = CMessage::new(WORLD_LIST_RESPONSE_MESSAGE_TYPE);
        add_legacy_string(response.base_mut(), &account);
        self.game
            .add_world_info_to_message_for_account(&mut response, &account);
        self.game.send_to_client(&response, message.socket_id())?;
        Ok(LogMessageOutcome::WorldListSent)
    }
}

fn handled() -> LogMessageOutcome {
    LogMessageOutcome::Handled {
        notices: Vec::new(),
    }
}

fn send_code(game: &CGame, socket_id: i32, code: u8) -> Result<(), GameRouteError> {
    let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
    response.base_mut().add_char(code as i8);
    game.send_to_client(&response, socket_id)?;
    Ok(())
}

fn send_extended_login_rejection(game: &CGame, socket_id: i32) -> Result<(), GameRouteError> {
    let mut response = CMessage::new(EXTENDED_LOGIN_RESPONSE_MESSAGE_TYPE);
    response.base_mut().add_long(CLIENT_LOGIN_MARKER);
    add_legacy_string(response.base_mut(), b"");
    add_legacy_string(response.base_mut(), b"");
    game.send_to_client(&response, socket_id)?;
    Ok(())
}

fn get_bounded_string(message: &mut CBaseMessage, maximum: usize) -> Vec<u8> {
    message
        .get_str_bytes(maximum)
        .expect("ненулевая GetStr-граница задана константой")
}

fn get_password_digest(message: &mut CBaseMessage) -> Result<Option<Vec<u8>>, LogMessageError> {
    let remaining = message
        .as_wire_bytes()
        .len()
        .saturating_sub(message.cursor());
    if remaining < PASSWORD_DIGEST_LEN {
        return Ok(None);
    }

    let declared_len = message
        .get_long()
        .expect("первая GetEx-проверка гарантирует наличие length-prefix");
    if declared_len != PASSWORD_DIGEST_LEN as i32 {
        return Ok(None);
    }
    let available = remaining - size_of::<i32>();
    if available < PASSWORD_DIGEST_LEN {
        return Ok(None);
    }

    let mut digest = vec![0; PASSWORD_DIGEST_LEN];
    assert!(message.get(&mut digest));
    Ok(Some(digest))
}

fn add_legacy_string(message: &mut CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn legacy_lower_account(account: &mut [u8]) {
    for byte in account {
        // EXE не меняет process locale, поэтому встроенный CRT `tolower`
        // преобразует только ASCII `A..Z`; прочие signed-char значения остаются.
        if byte.is_ascii() {
            byte.make_ascii_lowercase();
        }
    }
}
