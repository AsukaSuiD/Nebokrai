//! Очереди и проверки `CLoginQueue` из `loginqueue.cpp` и `.h`.
//!
//! Статус `OnInitial` RVA `0x000147A0`, `CGasQueue::Pop` `0x00015C20`,
//! `AddGasQueue` `0x000199A0`, `OnQuestCdkey` `0x0001A130`,
//! `AddQuestCdkey` `0x0001CAD0`, `tagPwdChecked` RVA `0x00001D80`,
//! `IsValidQuest` `0x000172C0`, `ClearTimeoutList` `0x00017330`,
//! `PushLoginList` `0x0001AAB0`, `OnQuestPlayerData` `0x0001B3F0`,
//! `AddQuestPlayerList` `0x0001E740`, `AddQuestPlayerData` `0x0001E880`,
//! `OnClientLost` `0x0001A800`,
//! `IsInNoQueueList` `0x00016040`, `LoadNoQueueCdkeyList` `0x000195E0`,
//! `PushBackPwdChecked` RVA
//! `0x00019880`, `IsValidErrManyTimes` `0x000163E0`, `CheckValidErr`
//! `0x00018700`, `AddValidErr` `0x0001C100` и baseline-ветви
//! `HandlePwdChecked` `0x0001BB20`, `tagValidCode` `0x00015F70`,
//! `CheckMsgInfo` `0x00016200`, `ChangeValidCode` `0x00016300`,
//! `ValidateValidCode` `0x000184B0`, `ValidCodeOvertime` `0x00018610`,
//! `matrix_get/matrix_del/matirx_validate` `0x00016100/0x00018310/0x00019800`,
//! `matrix_add` `0x0001B870`, `matrix_register` `0x0001B950`,
//! `matrices_timeout` `0x000183D0`, constructor/destructor
//! `0x0001E9F0/0x0001E490` и timeout-хвост `Run` `0x0001D500` —
//! `IMPLEMENTED`; спорные field/call mappings этих функций —
//! `VERIFIED_DISASSEMBLY`. Владелец завершён. Точная пара:
//! `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`, SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходные пути PDB:
//! `d:\complite_version\fengyun_russia\trunk\server\loginserver\loginserver\loginqueue.cpp`
//! и `.h`.
//!
//! `TagPwdChecked` сохраняет signed socket ID, исходный IPv4 `ulong`,
//! byte-exact account/world-name и matrix-флаг. `PushBackPwdChecked` игнорировал
//! `nullptr`; Rust меняет форму API и принимает только owned значение. Под
//! `lockPwdChecked` исходная функция искала первое точное совпадение account,
//! вызывала `CGame::KickOut`, уничтожала прежний объект и узел, затем всегда
//! добавляла новый объект в хвост. `parking_lot::Mutex<VecDeque<_>>` заменяет
//! `Lock + std::list + new/delete`; callback выполняется до удаления и всё ещё
//! под тем же lock, поэтому наблюдаемый порядок не изменён.
//!
//! `HandlePwdChecked` держал этот же lock вокруг полного drain и выполнял
//! client-send, `PrepareEnter` и `EnterGame` до освобождения. Rust сохраняет
//! эту широкую сериализацию. Проверка `p_Var12 == nullptr` относилась к
//! внутреннему storage pointer `std::string`, а не к пустоте account: owned
//! Rust-строка всегда имеет допустимое хранилище, и пустой account не получает
//! придуманного отказа. Нулевые socket ID и IPv4 по-прежнему отбрасываются.
//! `m_mapValidErr` не находился под `lockPwdChecked` и представлен отдельным
//! mutex; `IsValidErrManyTimes` сравнивает `error_times >= setup limit`,
//! `AddValidErr` создаёт `1` либо увеличивает signed счётчик с 32-битным
//! wrapping и продлевает boot-tick deadline, а `CheckValidErr` удаляет запись
//! только при строгом `next_login_time < now`.
//!
//! Включённая ветка legacy valid-code сохраняет замену прежней записи с
//! уведомлением `N` старому socket, `added_time/change_time`, исходные поля
//! world/matrix и ответ `J + account + 0x70B6 + BMP`. `CheckMsgInfo` различает
//! отсутствующий account, другой socket и строгий интервал одной секунды;
//! `ChangeValidCode` меняет только код и boot tick. `ValidateValidCode`
//! удаляет запись при endpoint mismatch либо успехе `K`, но сохраняет её при
//! неверном коде `L`. Периодический timeout использует unsigned wrapping
//! разность и отправляет `M` перед удалением.
//!
//! `matrix_add` отвергает нулевой socket/IP и уже существующий account;
//! иначе он один раз сохраняет endpoint, три позиции и текущий boot tick.
//! Nullable `char*`/`uchar*` заменены обязательными safe Rust-ссылками.
//! `matrix_register` сначала выбирает три позиции `0..80`, затем заменяет
//! старую запись с `F`, вызывает ту же границу `matrix_add` и независимо от
//! её исходно проигнорированного результата отправляет `B + account + 3
//! bytes`.
//! `matrix_get/matrix_del/matirx_validate` RVA `0x00016100/0x00018310/`
//! `0x00019800` сохраняют одноразовую запись: отсутствующая запись даёт `D`,
//! endpoint mismatch удаляет её без DB-вызова, а совпавший endpoint передаёт
//! позиции и ответ единому `CRsCDKey` и затем удаляет запись при любом `C/D`.
//! `matrices_timeout` использует отдельный boot tick для каждой записи,
//! строгую unsigned wrapping-разность, отправляет `E` до удаления и сохраняет
//! ошибку отправки только как наблюдаемое уведомление.
//!
//! Полный `Run` RVA `0x0001D500..0x0001DEAF` сохраняет исходный порядок:
//! немедленные GAS/no-queue drains, один обычный CD-key по cadence,
//! `HandlePwdChecked`, по одному player-list/player-data на World, ответы
//! позиции `0xAF507`, `ClearTimeoutList`, `AuthManager::run` и три timeout-
//! проверки. Один boot tick обслуживает очередь и позиции; timeout-хвост
//! отдельно семплирует tick перед каждой проверкой. Matrix и valid-code
//! таймеры обновляются при строгом `last + 1000 < now` только для непустой
//! map; valid-error таймер — при строгом `last + 3000 < now` независимо от её
//! содержимого. Все сложения остаются 32-битными wrapping, а исходные три
//! нулевых timer-поля собраны в локальное состояние cadence без нового общего
//! scheduler.
//! Точный код `0x0001D52B..0x0001D58A` подтвердил дефект: после обработки GAS
//! очищается no-queue CD-key FIFO, сама GAS FIFO остаётся и повторяется в
//! следующих проходах. Safe Rust семплирует GAS и полностью извлекаемые
//! no-queue maps в начале соответствующей стадии; конкурентное добавление
//! остаётся следующему проходу. Исходный race при конкурентной мутации этих
//! контейнеров не имеет требуемого внешнего контракта и не воспроизводится.
//! `BTreeMap` и owned значения заменяют `std::map` и ручное владение; отдельные
//! mutex не расширяют доменную семантику, а широкая сериализация
//! `lockPwdChecked` по-прежнему охватывает весь password drain и его sends.
//! `OnInitial` переносит три positional setup-значения, исправляет исходный
//! нулевой `m_nWordNum` на `1` и от одного boot-tick выставляет оба wrapping-
//! deadline. Остальные queue-поля не назначаются раньше их владельцев.
//!
//! `AddQuestCdkey` сохраняет signed IDs, IPv4, login/version/code/key, byte-
//! exact account, digest и World, ставит wrapping send-deadline и выбирает
//! обычную либо no-queue FIFO по точному account. `OnQuestCdkey` сначала
//! обслуживает уже выбранный World, затем буквально различает inside/GAS-
//! режимы. Локальная ветка сохраняет порядок numeric fix -> ban -> allow ->
//! forbid -> between -> matrix -> AuthServer/local password. Первые 16 байт
//! digest кодируются в 32 uppercase hex для локальной DB; только AuthServer-
//! ветка применяет исходный lowercase. Короткий digest безопасно отклоняется,
//! а отсутствие `CRsCDKey` не превращается в успех.
//! `Mutex<VecDeque<QuestCdkey>>` заменяет обычные `std::list` и GAS
//! `Locker + list<tagQuestCdkey*>`; clone/Drop заменяют copy constructor и
//! ручное владение без изменения FIFO.
//! Player-очереди сохраняют исходное `map<world, list<...>>` через
//! `Mutex<BTreeMap<Vec<u8>, VecDeque<_>>>`: ключи и FIFO остаются byte-exact,
//! no-queue слой отделён. `m_mLoginList` использует ordered map `player_id ->
//! boot tick`; повтор допустим только после строгого истечения wrapping-
//! интервала. `OnQuestPlayerData` независимо от результата World send сначала
//! фиксирует новый login tick, а ранний повтор получает `0xAF503 + 0x1C +
//! account` по строковой client identity.
//! `OnClientLost` принимает byte-exact C-string account и последовательно
//! удаляет только первое совпадение из обычной CD-key FIFO, затем не более
//! первого совпадения из каждого World FIFO обычных player-list и player-data
//! карт. Пустые World entries остаются в картах. No-queue, GAS, password,
//! valid-code и matrix слои исходная функция не трогала. Три отдельных mutex-
//! секции сохраняют порядок контейнеров без придуманной общей атомарности;
//! `VecDeque::remove` и `Drop` заменяют unlink/destructor/delete.
//!
//! `LoadNoQueueCdkeyList` очищает ordered set до открытия case-insensitive
//! `NoQueueAccounts.conf`, читает whitespace-token, применяет C-locale `_strlwr`
//! и вставляет каждый account немедленно, сохраняя partial mutation и
//! дедупликацию. Размер `char[0x100]` имеет статус `VERIFIED_DISASSEMBLY`:
//! `0x004196F0` передаёт `ESP+0xC8`, верхняя граница локала находится на
//! `ESP+0x1C8`; безопасный предел равен 255 bytes плюс NUL. Найденный fixture
//! непустой, содержит два коротких ASCII-token. Пустой файл, более длинный
//! token безопасно отклоняется до переполнения; пустой файл даёт пустой set.
//! Exact EXE не устанавливает process locale: встроенный CRT `_strlwr` поэтому
//! меняет только ASCII `A..Z`, а high-bit bytes сохраняет.
//! `IsInNoQueueList` представлен byte-exact поиском в том же `BTreeSet`;
//! nullable C-string не переносится во внутренний owned API.
//!
//! Constructor создаёт пустые collections и нулевые cadence-поля, затем в
//! собственной последней позиции выполняет начальный `LoadNoQueueCdkeyList`;
//! его нефатальный результат возвращается только в operator-report `CGame`.
//! `Drop` всех полей заменяет ручной destructor; его внутренний порядок
//! удаления не воспроизводится, поскольку там нет внешнего callback либо
//! иного наблюдаемого эффекта. После классификации удалён
//! весь заменённый raw-корпус: 269 STL/iostream функций, 16 ADO/COM blocks,
//! 36 их `Catch/FUN` continuations, 187 MSVC unwind-funclet `$L`, два
//! process-global cleanup `$E`, deleting thunks и пять чужих `CGame` COMDAT.
//! Их существенные эффекты выражены стандартными collections, mutex guards,
//! Tiberius и `Drop`.

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

use crate::dbaccess::logindb::rscdkey::MatrixValidation;
use crate::loginserver::applogin::validcode::{CValidCode, ValidCodeError};
use crate::loginserver::loginserver::authhandler::AuthHandler;
use crate::loginserver::loginserver::authmanager::{
    AddQuestOutcome, AuthManager, AuthQuest, AuthRunOutcome,
};
use crate::loginserver::loginserver::game::{
    AuthLifecycleError, CGame, GameRouteError, PrepareEnterOutcome, resolve_legacy_ascii_case,
};
use crate::nets::netlogin::message::CMessage;

const AUTH_FAILED_MESSAGE_TYPE: i32 = 0x000A_F501;
const PLAYER_DATA_REJECT_MESSAGE_TYPE: i32 = 0x000A_F503;
const QUEUE_POSITION_MESSAGE_TYPE: i32 = 0x000A_F507;
const NO_QUEUE_ACCOUNT_BUFFER_SIZE: usize = 0x100;

/// Owned-форма исходного `CLoginQueue::tagPwdChecked`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TagPwdChecked {
    client_ip: u32,
    socket_id: i32,
    account: Vec<u8>,
    world_server: Vec<u8>,
    has_matrix: bool,
}

impl TagPwdChecked {
    /// Сохраняет все пять полей без перекодирования и нормализации.
    pub(crate) fn new(
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

    /// Возвращает byte-exact account, по которому очередь ищет duplicate.
    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    /// Возвращает исходный signed client socket ID.
    pub(crate) const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает исходный 32-битный client IPv4.
    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    /// Возвращает byte-exact имя уже выбранного WorldServer.
    pub(crate) fn world_server(&self) -> &[u8] {
        &self.world_server
    }

    /// Возвращает исходный matrix-флаг.
    pub(crate) const fn has_matrix(&self) -> bool {
        self.has_matrix
    }
}

/// Owned-форма исходного `CLoginQueue::tagQuestCdkey`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QuestCdkey {
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

    /// Возвращает byte-exact account исходной заявки.
    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    /// Возвращает исходный signed client socket ID.
    pub(crate) const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает исходный 32-битный client IPv4.
    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    /// Возвращает byte-exact первые байты password digest из заявки.
    pub(crate) fn password_digest(&self) -> &[u8] {
        &self.password_digest
    }

    /// Возвращает byte-exact имя выбранного WorldServer.
    pub(crate) fn world_server(&self) -> &[u8] {
        &self.world_server
    }

    /// Возвращает nickname, полученный от GAS.
    pub(crate) fn nickname(&self) -> &[u8] {
        &self.nickname
    }

    /// Сохраняет успешный GAS nickname одновременно как nickname и account.
    pub(crate) fn replace_account_with_nickname(&mut self, nickname: Vec<u8>) {
        self.account.clone_from(&nickname);
        self.nickname = nickname;
    }

    /// Возвращает исходный deadline повторного queue-сообщения.
    pub(crate) const fn send_message_time(&self) -> u32 {
        self.send_message_time
    }
}

/// Owned-форма исходного `CLoginQueue::tagQuestPlayerList`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QuestPlayerList {
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

/// Owned-форма исходного `CLoginQueue::tagQuestPlayerData`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QuestPlayerData {
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

/// Итог доказанного `OnQuestPlayerData`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestPlayerDataOutcome {
    /// Запрос принят; bool сообщает, существовал ли World route при отправке.
    Forwarded { world_found: bool },
    /// `player_id` ещё находится внутри исходного repeat-интервала.
    Repeated,
}

/// Итог исходного точечного cleanup `CLoginQueue::OnClientLost`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClientLostCleanupReport {
    /// Удалена ли первая обычная CD-key заявка.
    pub(crate) cdkey_removed: bool,
    /// Число World FIFO, из которых удалена первая player-list заявка.
    pub(crate) player_list_removed: usize,
    /// Число World FIFO, из которых удалена первая player-data заявка.
    pub(crate) player_data_removed: usize,
}

/// Итог одного вызова исходного `OnQuestCdkey`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum QuestCdkeyOutcome {
    /// Непустой World прошёл доказанный `PrepareEnter -> EnterGame` путь.
    DirectWorld,
    /// `PrepareEnter` полностью завершил прямую ветвь без `EnterGame`.
    DirectWorldFinished,
    /// Режим `m_lIsInsideUse == 0` передал owned-копию в GAS FIFO.
    QueuedForGas,
    /// Иное доказанное значение `m_lIsInsideUse` является исходным no-op.
    InsideModeIgnored { mode: i32 },
    /// Действующий `ban_time` отправлен клиенту кодом `0x10`.
    ActiveBan,
    /// Allow/forbid IP-проверка отправила общий код `0x12`.
    IpRejected,
    /// Account/IP-list проверка отправила отдельный код `0x11`.
    BetweenIpRejected,
    /// Заявка передана подключённому AuthServer либо оказалась duplicate.
    AuthQuest(AddQuestOutcome),
    /// Локальная проверка пароля отправила исходный код `7`.
    LocalPasswordRejected,
    /// Локальный пароль подтверждён, а запись добавлена в password-check FIFO.
    LocalPasswordAccepted,
}

/// Безопасная неразрешимая граница `OnQuestCdkey` либо ошибка его send.
#[derive(Debug)]
pub(crate) enum QuestCdkeyError {
    /// Частичный lifecycle ещё не присоединил обязательный `CRsCDKey`.
    DatabaseOwnerMissing,
    /// Positional setup не определил все три IP admission-флага.
    IpSetupMissing,
    /// Positional setup не определил `m_lIsInsideUse`.
    InsideModeMissing,
    /// Оригинал без проверки читал первые 16 байт более короткого digest.
    PasswordDigestTooShort { actual_len: usize },
    /// `CharLowerA` зависит от внешней ANSI locale исходной Windows-системы.
    AuthAnsiCaseMappingUnknown,
    /// Фактическая Client/World transport-граница не выполнила send.
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

/// Исходный `eCheckRes` для client valid-code сообщений.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckMessageInfo {
    /// Account и socket совпали, одна секунда после последней замены прошла.
    Success,
    /// Account отсутствует в `m_mapValidCode`.
    Missing,
    /// Сообщение пришло не с сохранённого socket ID.
    SocketMismatch,
    /// Новый запрос картинки пришёл раньше `change_time + 1000`.
    Frequent,
}

/// Итог исходного `ValidateValidCode` с кодами `K/L`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValidateValidCodeOutcome {
    /// Код `K`: запись удалена, а сохранённые поля возвращены caller.
    Accepted {
        /// Выбранный до проверки byte-exact WorldServer.
        world_server: Vec<u8>,
        /// Исходный matrix-флаг password-check записи.
        has_matrix: bool,
    },
    /// Код `L`: значение не совпало; корректная endpoint-запись остаётся.
    Rejected,
}

/// Итог исходного `matirx_validate` с внутренними кодами `C/D`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MatrixValidationOutcome {
    /// Код `C`: endpoint и три matrix-значения подтверждены DB-owner.
    Accepted,
    /// Код `D`: запись отсутствует, endpoint не совпал либо значения неверны.
    Rejected,
    /// Частичный lifecycle ещё не присоединил обязательный `CRsCDKey`.
    DatabaseOwnerMissing,
}

/// Наблюдаемая проблема одной уже извлечённой password-check записи.
#[derive(Debug)]
pub(crate) enum PwdCheckedNotice {
    /// Не удалось отправить клиенту доказанный `0xAF501 + 'Q'`.
    ClientResponse {
        /// Исходный signed socket ID.
        socket_id: i32,
        /// Ошибка фактического client net-owner.
        error: GameRouteError,
    },
    /// Не удалось построить исходный `CValidCode` из runtime-ресурсов.
    ValidCodeGeneration {
        /// Byte-exact account извлечённой записи.
        account: Vec<u8>,
        /// Ошибка фактического owner изображения.
        error: ValidCodeError,
    },
    /// `PrepareEnter` не смог выполнить обязательный client/World side effect.
    PrepareEnter {
        /// Byte-exact account извлечённой записи.
        account: Vec<u8>,
        /// Ошибка фактической сетевой границы.
        error: GameRouteError,
    },
    /// Linux RNG не выдал три позиции исходного `matrix_register`.
    MatrixRandom {
        /// Byte-exact account matrix-регистрации.
        account: Vec<u8>,
        /// Ошибка системного источника случайности.
        error: getrandom::Error,
    },
    /// `EnterGame` не смог выполнить обязательный client/World side effect.
    EnterGame {
        /// Byte-exact account извлечённой записи.
        account: Vec<u8>,
        /// Ошибка фактической сетевой границы.
        error: GameRouteError,
    },
}

/// Итог одного исходного полного drain `HandlePwdChecked`.
#[derive(Debug)]
pub(crate) struct HandlePwdCheckedReport {
    /// Число извлечённых owned записей.
    pub(crate) processed: usize,
    /// Число записей, отброшенных из-за нулевого socket ID либо IPv4.
    pub(crate) dropped_invalid_endpoint: usize,
    /// Число account, отвергнутых текущим `m_mapValidErr`.
    pub(crate) rejected_by_valid_errors: usize,
    /// Число созданных и поставленных client valid-code картинок.
    pub(crate) generated_valid_codes: usize,
    /// Ошибки и локальные недостающие владельцы в порядке обработки.
    pub(crate) notices: Vec<PwdCheckedNotice>,
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

/// Итог одного восстановленного timeout-хвоста `CLoginQueue::Run`.
#[derive(Debug)]
pub(crate) struct LoginQueueTimeoutReport {
    /// Запускался ли `matrices_timeout` после непустой map и строгого интервала.
    pub(crate) matrices_ran: bool,
    /// Запускался ли `ValidCodeOvertime` после непустой map и строгого интервала.
    pub(crate) valid_codes_ran: bool,
    /// Запускался ли `CheckValidErr` после строгого трёхсекундного интервала.
    pub(crate) valid_errors_ran: bool,
    /// Ошибки client-send кодов `E/M` в исходном порядке стадий.
    pub(crate) notices: Vec<PwdCheckedNotice>,
}

/// Стадия полного `CLoginQueue::Run`, на которой transport/owner дал ошибку.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginQueueRunStage {
    /// Повторно обрабатываемая GAS FIFO.
    GasCdkey,
    /// Немедленная no-queue CD-key FIFO.
    NoQueueCdkey,
    /// Немедленная no-queue player-list map.
    NoQueuePlayerList,
    /// Немедленная no-queue player-data map.
    NoQueuePlayerData,
    /// Один обычный CD-key по cadence.
    RegularCdkey,
    /// Один player-list на каждый World по cadence.
    RegularPlayerList,
    /// Один player-data на каждый World по cadence.
    RegularPlayerData,
    /// Периодический client-ответ с текущей позицией.
    QueuePosition,
}

/// Структурированная замена проигнорированной ошибки одного элемента `Run`.
#[derive(Debug)]
pub(crate) enum LoginQueueRunNotice {
    /// Ошибка полного `OnQuestCdkey` с byte-exact account.
    Cdkey {
        /// Стадия, на которой выполнялась заявка.
        stage: LoginQueueRunStage,
        /// Byte-exact account исходной заявки.
        account: Vec<u8>,
        /// Ошибка владельца CD-key проверки.
        error: QuestCdkeyError,
    },
    /// Ошибка World/client маршрута player-заявки либо queue-position ответа.
    Route {
        /// Стадия, на которой выполнялась заявка или отправка позиции.
        stage: LoginQueueRunStage,
        /// Byte-exact account исходной заявки.
        account: Vec<u8>,
        /// Signed socket ID исходной заявки.
        socket_id: i32,
        /// Ошибка маршрута к World либо client.
        error: GameRouteError,
    },
}

/// Наблюдаемый итог одного полного прохода `CLoginQueue::Run`.
#[derive(Debug)]
pub(crate) struct LoginQueueRunReport {
    /// Число заявок из неизвлекаемой GAS FIFO, обработанных в этом проходе.
    pub(crate) gas_cdkeys_processed: usize,
    /// Число no-queue CD-key заявок, уничтоженных подтверждённым GAS-дефектом.
    pub(crate) no_queue_cdkeys_discarded_by_gas_bug: usize,
    /// Число обработанных no-queue CD-key заявок.
    pub(crate) no_queue_cdkeys_processed: usize,
    /// Число обработанных no-queue player-list заявок.
    pub(crate) no_queue_player_lists_processed: usize,
    /// Число обработанных no-queue player-data заявок.
    pub(crate) no_queue_player_data_processed: usize,
    /// Число обычных CD-key заявок; за проход не больше одной.
    pub(crate) regular_cdkeys_processed: usize,
    /// Число обычных player-list заявок; за World не больше одной.
    pub(crate) regular_player_lists_processed: usize,
    /// Число обычных player-data заявок; за World не больше одной.
    pub(crate) regular_player_data_processed: usize,
    /// Число предпринятых client-отправок позиции `0xAF507`.
    pub(crate) queue_position_messages: usize,
    /// Число удалённых просроченных player ID.
    pub(crate) login_timeouts_removed: usize,
    /// Итог password-result drain в исходной позиции прохода.
    pub(crate) pwd_checked: HandlePwdCheckedReport,
    /// Итог `AuthManager::run` либо отсутствие подключённого Auth publisher.
    pub(crate) auth: Result<AuthRunOutcome, AuthLifecycleError>,
    /// Итог matrix/valid-code/valid-error timeout-хвоста.
    pub(crate) timeout_tail: LoginQueueTimeoutReport,
    /// Ошибки проигнорированных оригиналом send/owner-вызовов по порядку.
    pub(crate) notices: Vec<LoginQueueRunNotice>,
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

/// Итог успешного чтения исходного `NoQueueAccounts.conf`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NoQueueAccountsLoadReport {
    /// Число реально извлечённых whitespace-token до `std::set` дедупликации.
    pub(crate) extracted_accounts: usize,
    /// Итоговый размер byte-exact ordered set после lowercase.
    pub(crate) unique_accounts: usize,
}

/// Ошибка чтения либо безопасная граница старого небезопасного `char[0x100]`.
#[derive(Debug)]
pub(crate) enum NoQueueAccountsLoadError {
    /// Файл не найден либо не прочитан; set уже очищен в исходной позиции.
    Io(io::Error),
    /// Token не помещается вместе с NUL в доказанный stack-buffer.
    TokenTooLong {
        token_index: usize,
        actual: usize,
        maximum: usize,
    },
}

impl fmt::Display for NoQueueAccountsLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не прочитан NoQueueAccounts.conf: {error}"),
            Self::TokenTooLong {
                token_index,
                actual,
                maximum,
            } => write!(
                formatter,
                "token {token_index} NoQueueAccounts.conf имеет {actual} байт при пределе {maximum}",
            ),
        }
    }
}

impl Error for NoQueueAccountsLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::TokenTooLong { .. } => None,
        }
    }
}

impl From<io::Error> for NoQueueAccountsLoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Минимальный восстановленный owner очереди подтверждённых паролей.
pub(crate) struct CLoginQueue {
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
    /// Ошибка файла не отменяет создание owner и возвращается отдельно, как
    /// нефатальный operator-visible результат исходного constructor.
    pub(crate) fn new(
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

    /// Применяет исходный `OnInitial` после позиционного `CGame::LoadSetup`.
    pub(crate) fn on_initial(
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

    /// Обновляет исходный `m_nWordNum` после `AddWorld`/`DelWorld`.
    pub(crate) fn set_world_count(&self, world_count: u32) {
        self.setup.lock().world_count = world_count;
    }

    /// Создаёт исходный `tagQuestCdkey` и выбирает обычную/no-queue FIFO.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_quest_cdkey(
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

    /// Извлекает старейшую owned-заявку GAS под исходным queue-lock.
    pub(crate) fn pop_gas_quest(&self) -> Option<QuestCdkey> {
        self.gas_quests.lock().pop_front()
    }

    /// Выполняет полный доказанный `OnQuestCdkey` в исходном порядке.
    pub(crate) fn on_quest_cdkey(
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
    pub(crate) fn add_quest_player_list(
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

    /// Добавляет запрос деталей персонажа в карту текущего World account.
    pub(crate) fn add_quest_player_data(
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

    /// Удаляет первые обычные заявки потерянного client account.
    pub(crate) fn on_client_lost(&self, account: &[u8]) -> ClientLostCleanupReport {
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

    /// Выполняет исходную repeat-проверку и маршрутизацию деталей персонажа.
    pub(crate) fn on_quest_player_data(
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

    /// Разрешает отсутствующий либо строго просроченный `player_id`.
    pub(crate) fn is_valid_quest(&self, player_id: i32, interval_ms: u32) -> bool {
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

    /// Вставляет новый `player_id`; существующая запись не обновляется.
    pub(crate) fn push_login_list(&self, player_id: i32) -> bool {
        let mut login_list = self.login_list.lock();
        if login_list.contains_key(&player_id) {
            return false;
        }
        login_list.insert(player_id, legacy_tick_ms());
        true
    }

    /// Удаляет записи только при строгом `added + interval < now`.
    pub(crate) fn clear_timeout_list(&self, interval_ms: u32) -> usize {
        let mut login_list = self.login_list.lock();
        let previous_len = login_list.len();
        login_list.retain(|_, added_time| {
            let now = legacy_tick_ms();
            added_time.wrapping_add(interval_ms) >= now
        });
        previous_len - login_list.len()
    }

    /// Выполняет полный доказанный порядок исходного `CLoginQueue::Run`.
    ///
    /// Неопределённый исходный race с producers заменён короткими snapshot-
    /// границами: конкурентно добавленные элементы остаются следующему проходу.
    pub(crate) fn run(
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

        // VERIFIED_DISASSEMBLY: Login RVA 0x0001D52B..0x0001D58A читает
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
    pub(crate) fn push_back_pwd_checked(
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

    /// Возвращает текущий размер для будущего исходного queue cadence.
    pub(crate) fn pwd_checked_len(&self) -> usize {
        self.pwd_checked.lock().len()
    }

    /// Полностью дренирует FIFO под исходным `lockPwdChecked`.
    pub(crate) fn handle_pwd_checked(&self, game: &mut CGame) -> HandlePwdCheckedReport {
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

    /// Продолжает исходный `PrepareEnter -> EnterGame/matrix_register` после `K`.
    pub(crate) fn continue_validated_login(
        &self,
        game: &mut CGame,
        checked: &TagPwdChecked,
    ) -> Vec<PwdCheckedNotice> {
        let mut notices = Vec::new();
        self.complete_checked_entry(game, checked, &mut notices);
        notices
    }

    /// Сохраняет исходный `AddValidErr` с wrapping tick и продлением stay-time.
    pub(crate) fn add_valid_error(&self, account: &[u8], stay_time_ms: u32) {
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

    /// Удаляет записи, для которых `next_login_time < now`, как `CheckValidErr`.
    pub(crate) fn clear_expired_valid_errors(&self, now: u32) {
        self.valid_errors
            .lock()
            .retain(|_, error| error.next_login_time >= now);
    }

    /// Выполняет только хвост `Run` после ещё не восстановленных промежуточных стадий.
    ///
    /// Три вызова boot clock намеренно не объединены: исходник вызывал
    /// `timeGetTime` отдельно перед matrix, valid-code и valid-error проверкой.
    pub(crate) fn run_timeout_tail(
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
    /// выполненную partial mutation. Ошибка открытия также оставляет set
    /// пустым, как исходный вызов до `ifstream::open`.
    pub(crate) fn load_no_queue_cdkey_list(
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
                // VERIFIED_DISASSEMBLY: Login 0x004196F0 передаёт в extraction
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

    /// Проверяет account/socket и исходный one-second change interval.
    pub(crate) fn check_message_info(&self, account: &[u8], socket_id: i32) -> CheckMessageInfo {
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

    /// Меняет сохранённый ответ и ставит `change_time = timeGetTime()`.
    pub(crate) fn change_valid_code(&self, account: &[u8], valid_code: &[u8]) {
        if let Some(entry) = self.valid_codes.lock().get_mut(account) {
            entry.change_time = legacy_tick_ms();
            entry.valid_code.clear();
            entry.valid_code.extend_from_slice(valid_code);
        }
    }

    /// Проверяет endpoint и byte-exact ответ, сохраняя удаление `K/L` ветвей.
    pub(crate) fn validate_valid_code(
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

    /// Удаляет точный account из `m_mapValidCode`.
    pub(crate) fn delete_valid_code(&self, account: &[u8]) {
        self.valid_codes.lock().remove(account);
    }

    /// Проверяет одноразовую matrix-запись и удаляет её в исходных `C/D` ветвях.
    pub(crate) fn validate_matrix(
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
            // Без исходного DB-owner продолжить вызов невозможно; запись не
            // выдаётся за проверенную и остаётся для штатного lifecycle.
            return MatrixValidationOutcome::DatabaseOwnerMissing;
        };
        let accepted = match validation {
            MatrixValidation::Compared(accepted) => accepted,
            // Короткий DB blob в оригинале приводил к out-of-bounds чтению.
            // Safe Rust детерминированно считает проверку неуспешной.
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
    pub(crate) fn add_matrix(
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

    /// Проверяет наличие account в исходном `NoQueueCDkeyList`.
    pub(crate) fn is_no_queue_account(&self, account: &[u8]) -> bool {
        self.no_queue_accounts.lock().contains(account)
    }

    /// Отправляет `M` и удаляет все записи со строгим unsigned timeout.
    pub(crate) fn expire_valid_codes(
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

    /// Отправляет `E` и удаляет matrix-записи со строгим unsigned timeout.
    pub(crate) fn expire_matrices(&self, game: &CGame, timeout_ms: u32) -> Vec<PwdCheckedNotice> {
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
        // Login RVA 0x0001A130 читал `strPassWord[0..16]` без size-проверки;
        // safe Rust отклоняет короткий внешний digest до доступа.
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
