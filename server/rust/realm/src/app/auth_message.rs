//! Сообщение направления LoginServer -> AuthServer из `nets/netauth/message.cpp`;
//! Realm — владелец Auth-направления объединённого Realm-процесса; исходные
//! идентификаторы сборки — в evidence (см. ниже).
//!
//! Конструктор пишет `MsgType` в слово header `+4`, оставляет пустой CD-key и
//! обнуляет socket ID, map ID, IPv4 и receive tick. Create-путь буквально
//! копирует четыре слова входного header, затем нормализует длину по реально
//! добавленному payload. `GetStr` очищает результат, читает bytes до первого
//! NUL и включает NUL в движение курсора: без NUL результат снова пустой, но
//! уже прочитанные bytes не откатываются (quirk). `Vec<u8>` сохраняет исходную
//! байтовую строку без предположения UTF-8.
//!
//! Диспетчер ищет ПОЛНЫЙ 32-битный opcode в статической таблице: найденный
//! вызывается, неизвестный — no-op, возврат всегда `1`. `0x10F101` имеет
//! гарантированный no-op handler (машинное тело из одного `ret`), поэтому
//! остаётся особой опцией, а не отдельным доменным вызовом. RLE-вариант
//! `CreateMessage` у класса собран, но живой receive-path вызывает только
//! несжатый create (машинный факт). Поле recv-tick, в отличие от World,
//! нигде в направлении не переписывается и живого читателя не имеет;
//! хранимое значение — совместимое представление без наблюдаемой разницы.
//!
//! `SendToLogin` строит envelope `[total_len, crc(total_len), crc(message),
//! message]` и машинно не имеет critical section вокруг build/CRC/send — чужая
//! сериализация из других направлений сюда не переносится. Auth
//! `CServer::SendBySocketID` копирует весь вход в owned socket-команду до
//! возврата, поэтому локальный `Vec<u8>` имеет правильный lifetime.
//!
//! Доказательства: docs/reconstruction/realm-services.md#auth-сообщение-и-диспетчер

use std::fmt;

use nebokrai_shared::network::{CBaseMessage, ServerClientMessageContext, ServerCommandHandle};
use nebokrai_shared::protocol::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const NOOP_HANDLER_OPCODE: u32 = 0x0010_F101;

/// Ошибка восстановления Auth-сообщения из внутреннего wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub enum CreateMessageError {
    /// Внутренний wire-буфер пуст и не содержит обязательного header.
    EmptyInput,
    /// Внутренний wire-буфер короче обязательного header.
    HeaderTooShort { actual: usize },
    /// Размер не представим 32-битным `unsigned long` исходного API.
    InputOutsideLegacyRange,
}

/// Ошибка построения внешнего server envelope.
#[derive(Debug, Eq, PartialEq)]
pub enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
}

impl fmt::Display for SendMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOutsideLegacyRange => {
                formatter.write_str("длина Auth envelope не представима положительным Windows long")
            }
        }
    }
}

impl std::error::Error for SendMessageError {}

/// Ошибка достигнутого Auth-dispatch.
#[derive(Debug, Eq, PartialEq)]
pub enum DispatchError {
    /// Исходящий ответ handler’а не представим в legacy wire-диапазоне.
    OutgoingMessage(SendMessageError),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutgoingMessage(error) => {
                write!(
                    formatter,
                    "не удалось построить исходящий Auth-ответ: {error}"
                )
            }
        }
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OutgoingMessage(error) => Some(error),
        }
    }
}

/// Девять доказанных обработчиков `AuthServer` из `InitMsgFuncPool`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMessageKind {
    /// LoginServer установил соединение (`0x0CF401`).
    LoginServerConnected,
    /// LoginServer разорвал соединение (`0x0CF402`).
    LoginServerDisconnected,
    /// Обычная проверка учётной записи (`0x0CF501`).
    AuthenticateAccount,
    /// Расширенная проверка учётной записи (`0x0CF502`).
    AuthenticateAccountExtended,
    /// Запрос сведений LoginServer (`0x0CF503`).
    GetLoginServerInfo,
    /// Ответ обновления server-info (`0x0CF802`).
    UpdateServerInfoResponse,
    /// GM-запрос отключения игрока (`0x10F102`).
    GmKickPlayer,
    /// Ответ отключения игрока (`0x0CF801`).
    KickPlayerResponse,
    /// GM-запрос блокировки учётной записи (`0x10F103`).
    GmLockAccount,
}

impl AuthMessageKind {
    fn from_opcode(opcode: u32) -> Option<Self> {
        match opcode {
            0x000C_F401 => Some(Self::LoginServerConnected),
            0x000C_F402 => Some(Self::LoginServerDisconnected),
            0x000C_F501 => Some(Self::AuthenticateAccount),
            0x000C_F502 => Some(Self::AuthenticateAccountExtended),
            0x000C_F503 => Some(Self::GetLoginServerInfo),
            0x000C_F802 => Some(Self::UpdateServerInfoResponse),
            0x0010_F102 => Some(Self::GmKickPlayer),
            0x000C_F801 => Some(Self::KickPlayerResponse),
            0x0010_F103 => Some(Self::GmLockAccount),
            _ => None,
        }
    }
}

/// Узкая замена девяти конкретных свободных handler-функций AuthServer.
pub trait AuthMessageHandler {
    /// Обрабатывает уже классифицированное сообщение; wire-чтение остаётся у
    /// фактического доменного обработчика, а не у общего `nets`. Producer
    /// handle разрешает конкретному handler поставить доказанную исходящую
    /// команду, не передавая ему внутреннее состояние `CServer`.
    fn handle(
        &mut self,
        kind: AuthMessageKind,
        message: &mut CMessage,
        sender: &ServerCommandHandle,
    ) -> Result<(), DispatchError>;
}

/// Владеющее Auth-сообщение с исходными runtime-метаданными.
pub struct CMessage {
    base: CBaseMessage,
    cdkey: Vec<u8>,
    socket_id: i32,
    map_id: i32,
    recv_time_ms: u32,
    ip: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного полного типа.
    pub fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[], 0)
    }

    /// Создаёт входящее сообщение из несжатого header + payload.
    pub fn create_without_rle(wire: &[u8], recv_time_ms: u32) -> Result<Self, CreateMessageError> {
        if wire.is_empty() {
            return Err(CreateMessageError::EmptyInput);
        }
        if wire.len() > u32::MAX as usize {
            return Err(CreateMessageError::InputOutsideLegacyRange);
        }
        if wire.len() < MESSAGE_HEADER_LEN {
            return Err(CreateMessageError::HeaderTooShort { actual: wire.len() });
        }

        let header = wire[..MESSAGE_HEADER_LEN]
            .try_into()
            .expect("длина Auth header уже проверена");
        Ok(Self::from_parts(
            header,
            &wire[MESSAGE_HEADER_LEN..],
            recv_time_ms,
        ))
    }

    /// Возвращает точный полный `MsgType` из слова header `+4`.
    pub fn message_type(&self) -> i32 {
        self.base.message_type()
    }

    /// Предоставляет payload-владельцу доказанные `CBaseMessage::Get/Add`.
    pub fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Читает байтовую C-строку по точному контракту Auth `CMessage::GetStr`.
    pub fn get_str(&mut self) -> Vec<u8> {
        self.base.get_c_string_bytes()
    }

    /// Возвращает полное внутреннее сообщение без внешнего envelope.
    pub fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Присваивает metadata, которые Auth receive-owner брал из соединения.
    pub fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.cdkey.clear();
        self.cdkey.extend_from_slice(context.map_name);
        self.ip = context.peer_ipv4;
    }

    /// Присваивает только два metadata-поля synthetic accept/close сообщений.
    pub fn apply_socket_context(&mut self, socket_id: i32, peer_ipv4: u32) {
        self.socket_id = socket_id;
        self.ip = peer_ipv4;
    }

    /// Возвращает socket ID исходного LoginServer-соединения.
    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает числовую identity соединения.
    pub const fn map_id(&self) -> i32 {
        self.map_id
    }

    /// Возвращает буквальные bytes исходного `m_strCdkey`.
    pub fn cdkey(&self) -> &[u8] {
        &self.cdkey
    }

    /// Возвращает исходное 32-битное представление peer IPv4.
    pub const fn ip(&self) -> u32 {
        self.ip
    }

    /// Возвращает wrapping millisecond tick приёма.
    pub const fn recv_time_ms(&self) -> u32 {
        self.recv_time_ms
    }

    /// Строит исходный 12-байтовый CRC-envelope для LoginServer.
    pub fn server_envelope(&self) -> Result<Vec<u8>, SendMessageError> {
        let total_length = self
            .base
            .as_wire_bytes()
            .len()
            .checked_add(SERVER_ENVELOPE_LEN)
            .filter(|length| *length <= i32::MAX as usize)
            .ok_or(SendMessageError::LengthOutsideLegacyRange)?;
        let total_length_bytes = (total_length as i32).to_le_bytes();

        let mut envelope = Vec::with_capacity(total_length);
        envelope.extend_from_slice(&total_length_bytes);
        envelope.extend_from_slice(&data_crc32(&total_length_bytes).to_le_bytes());
        envelope.extend_from_slice(&data_crc32(self.base.as_wire_bytes()).to_le_bytes());
        envelope.extend_from_slice(self.base.as_wire_bytes());
        Ok(envelope)
    }

    /// Копирует envelope в команду `SendBySocketID` конкретного Auth-server.
    pub fn send_to_login(
        &self,
        sender: &ServerCommandHandle,
        socket_id: i32,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_socket_id(socket_id, &envelope))
    }

    /// Выполняет точную маршрутизацию `Run`; неизвестный opcode остаётся no-op.
    pub fn run(
        &mut self,
        handler: &mut dyn AuthMessageHandler,
        sender: &ServerCommandHandle,
    ) -> Result<i32, DispatchError> {
        let opcode = self.message_type() as u32;
        if opcode == NOOP_HANDLER_OPCODE {
            // Auth `InitMsgFuncPool` связывает opcode с no-op handler-ом,
            // тело которого состоит из одного `ret`.
            return Ok(1);
        }
        if let Some(kind) = AuthMessageKind::from_opcode(opcode) {
            handler.handle(kind, self, sender)?;
        }
        Ok(1)
    }

    fn from_parts(header: [u8; MESSAGE_HEADER_LEN], payload: &[u8], recv_time_ms: u32) -> Self {
        Self {
            base: CBaseMessage::from_header_and_payload(header, payload),
            cdkey: Vec::new(),
            socket_id: 0,
            map_id: 0,
            recv_time_ms,
            ip: 0,
        }
    }
}
