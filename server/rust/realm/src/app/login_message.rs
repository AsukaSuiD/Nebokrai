//! Сообщение трёх сетевых направлений LoginServer из `nets/netlogin/message.cpp`;
//! Realm — владелец Login-направления объединённого Realm-процесса; исходные
//! идентификаторы сборки — в evidence (см. ниже).
//!
//! Client send кодирует полное внутреннее сообщение legacy RLE и добавляет
//! четырёхбайтовую little-endian длину; World/Auth send строит envelope
//! `[total_len, crc(total_len), crc(message), message]`. Различие сериализации
//! значимо: `SendToAS` — единственный send-владелец с CRITICAL_SECTION вокруг
//! build/CRC/send, World/client send-методы lock машинно не имеют. Общие
//! `CServer`/`CClient` send-владельцы копируют вход до возврата, поэтому
//! локальные `Vec<u8>` сохраняют lifetime общей scratch-памяти. Client send
//! достижим также из отдельного `CGasThread`; исходная общая RLE-память могла
//! пересекаться с game-thread без синхронизации — safe Rust не воспроизводит
//! data race через `unsafe` и не объявляет недетерминированное повреждение
//! старого scratch совместимым контрактом.
//!
//! RLE create сохраняет порог `0x20001`, capacity `0x100000` либо
//! `compressed_len * 8`; несжатый create копирует четыре header-слова и
//! нормализует длину. Вход 1..15 bytes и overflow умножения безопасно
//! отклоняются до чтения header или выделения; malformed trailing marker
//! принадлежит shared basemessage.
//!
//! Конструктор записывает полный `MsgType` в header `+4`, оставляет пустой
//! CD-key, нулевые socket/map/IP и null player/region. Rust хранит только
//! реально передаваемые receive-owner’ами metadata; `m_pPlayer/m_pRegion` не
//! получают бестиповых аналогов до появления их доменного владельца. Traits
//! ниже соответствуют только этим историческим владельцам и не создают общий
//! protocol framework.
//!
//! Доказательства: docs/reconstruction/realm-services.md#login-сообщение-и-направления

use std::fmt;

use parking_lot::Mutex;

use nebokrai_shared::network::{
    decode_rle, encode_rle, CBaseMessage, ClientSendQueue, RleDecodeError, RleEncodeError,
    ServerClientMessageContext, ServerCommandHandle,
};
use nebokrai_shared::protocol::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const CLIENT_ENVELOPE_LEN: usize = 4;
const SERVER_ENVELOPE_LEN: usize = 12;
const SMALL_RLE_INPUT_LIMIT: usize = 0x20_001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x10_0000;
const MESSAGE_FAMILY_MASK: u32 = 0xFFFF_FF00;

static AUTH_SEND_SERIALIZER: Mutex<()> = Mutex::new(());

/// Ошибка восстановления Login-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub enum CreateMessageError {
    /// Исходный nullable/нулевой вход доказанно не создавал сообщение.
    EmptyInput,
    /// Внутренний wire-буфер короче обязательного header.
    HeaderTooShort { actual: usize },
    /// Размер не представим 32-битным `unsigned long` исходного API.
    InputOutsideLegacyRange,
    /// Требуемая RLE-capacity не представима в 32-битном исходном диапазоне.
    RleCapacityOutsideLegacyRange,
    /// Декодер отклонил поток либо достиг локально неизвестной malformed-границы.
    Rle(RleDecodeError),
}

/// Ошибка построения внешнего Login wire-envelope.
#[derive(Debug, Eq, PartialEq)]
pub enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
    /// RLE encoder достиг доказанной неизвестной границы пустого ввода.
    Rle(RleEncodeError),
}

impl fmt::Display for SendMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOutsideLegacyRange => formatter
                .write_str("длина Login envelope не представима положительным Windows long"),
            Self::Rle(_) => formatter.write_str("Login client-wire не удалось закодировать RLE"),
        }
    }
}

impl std::error::Error for SendMessageError {}

/// Пять конкретных семейств обработчиков `CMessage::Run` LoginServer.
pub trait LoginMessageHandlers {
    /// Обрабатывает полный AuthServer-диапазон `0xCF301..0xDF1FE`.
    fn on_auth(&mut self, message: &mut CMessage);
    /// Обрабатывает семейство GM `0x20000`.
    fn on_gm(&mut self, message: &mut CMessage);
    /// Обрабатывает семейство GMA `0x20100`.
    fn on_gma(&mut self, message: &mut CMessage);
    /// Обрабатывает три доказанных log-семейства.
    fn on_log(&mut self, message: &mut CMessage);
    /// Обрабатывает два доказанных server-семейства.
    fn on_server(&mut self, message: &mut CMessage);
}

/// Владеющее Login-сообщение с runtime-метаданными receive-направлений.
pub struct CMessage {
    base: CBaseMessage,
    cdkey: Vec<u8>,
    socket_id: i32,
    map_id: i32,
    ip: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного полного типа.
    pub fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[])
    }

    /// Декодирует client RLE и создаёт внутреннее Login-сообщение.
    pub fn create(compressed: &[u8]) -> Result<Self, CreateMessageError> {
        if compressed.is_empty() {
            return Err(CreateMessageError::EmptyInput);
        }
        if compressed.len() > u32::MAX as usize {
            return Err(CreateMessageError::InputOutsideLegacyRange);
        }
        let capacity = if compressed.len() < SMALL_RLE_INPUT_LIMIT {
            SMALL_RLE_OUTPUT_CAPACITY
        } else {
            compressed
                .len()
                .checked_mul(8)
                .filter(|capacity| *capacity <= u32::MAX as usize)
                .ok_or(CreateMessageError::RleCapacityOutsideLegacyRange)?
        };
        let decoded = decode_rle(compressed, capacity).map_err(CreateMessageError::Rle)?;
        Self::create_without_rle(&decoded)
    }

    /// Создаёт сообщение из несжатых header + payload.
    pub fn create_without_rle(wire: &[u8]) -> Result<Self, CreateMessageError> {
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
            .expect("длина Login header уже проверена");
        Ok(Self::from_parts(header, &wire[MESSAGE_HEADER_LEN..]))
    }

    /// Возвращает точный полный `MsgType` из header `+4`.
    pub fn message_type(&self) -> i32 {
        self.base.message_type()
    }

    /// Меняет только полный `MsgType`, сохраняя payload и runtime metadata.
    ///
    /// Нужен для доказанных GMA- и create-role маршрутов LoginServer, где
    /// оригинал пересылал тот же объект после записи нового opcode
    /// непосредственно в header.
    pub fn set_message_type(&mut self, message_type: i32) {
        self.base.set_message_type(message_type);
    }

    /// Предоставляет доменному владельцу доказанные `CBaseMessage::Get/Add`.
    pub fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Читает байтовую C-строку по контракту Login `GetString`.
    pub fn get_string(&mut self) -> Vec<u8> {
        self.base.get_c_string_bytes()
    }

    /// Возвращает полное внутреннее сообщение без внешнего envelope.
    pub fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Присваивает metadata accepted client/WorldServer соединения.
    pub fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.cdkey.clear();
        self.cdkey.extend_from_slice(context.map_name);
        self.ip = context.peer_ipv4;
    }

    /// Присваивает только numeric map metadata synthetic World disconnect.
    pub fn apply_map_id(&mut self, map_id: i32) {
        self.map_id = map_id;
    }

    /// Возвращает socket ID источника.
    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает числовую map identity источника.
    pub const fn map_id(&self) -> i32 {
        self.map_id
    }

    /// Возвращает буквальные bytes исходного CD-key/map-name.
    pub fn cdkey(&self) -> &[u8] {
        &self.cdkey
    }

    /// Возвращает исходное 32-битное представление peer IPv4.
    pub const fn ip(&self) -> u32 {
        self.ip
    }

    /// Строит client frame `[rle_len + 4, rle(message)]`.
    pub fn client_envelope(&self) -> Result<Vec<u8>, SendMessageError> {
        let compressed = encode_rle(self.base.as_wire_bytes()).map_err(SendMessageError::Rle)?;
        let total_length = compressed
            .len()
            .checked_add(CLIENT_ENVELOPE_LEN)
            .filter(|length| *length <= i32::MAX as usize)
            .ok_or(SendMessageError::LengthOutsideLegacyRange)?;
        let mut envelope = Vec::with_capacity(total_length);
        envelope.extend_from_slice(&(total_length as i32).to_le_bytes());
        envelope.extend_from_slice(&compressed);
        Ok(envelope)
    }

    /// Строит межсерверный 12-байтовый CRC-envelope.
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

    /// Копирует RLE frame клиенту по socket ID.
    pub fn send_to_client_socket(
        &self,
        sender: &ServerCommandHandle,
        socket_id: i32,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.client_envelope()?;
        Ok(sender.send_by_socket_id(socket_id, &envelope))
    }

    /// Копирует RLE frame клиенту по строковой map identity/CD-key.
    pub fn send_to_client_cdkey(
        &self,
        sender: &ServerCommandHandle,
        cdkey: &[u8],
    ) -> Result<i32, SendMessageError> {
        let envelope = self.client_envelope()?;
        Ok(sender.send_by_map_name(cdkey, &envelope))
    }

    /// Копирует server envelope WorldServer по socket ID.
    pub fn send_to_world_socket(
        &self,
        sender: &ServerCommandHandle,
        socket_id: i32,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_socket_id(socket_id, &envelope))
    }

    /// Копирует server envelope WorldServer по numeric map identity.
    pub fn send_to_world_map(
        &self,
        sender: &ServerCommandHandle,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_map_id(map_id, &envelope))
    }

    /// Копирует server envelope всем подключённым WorldServer.
    pub fn send_all_world(&self, sender: &ServerCommandHandle) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_all(&envelope))
    }

    /// Сохраняет сериализацию исходного `SendToAS` вокруг build/CRC/send.
    pub fn send_to_auth(&self, sender: &ClientSendQueue) -> Result<i32, SendMessageError> {
        let _guard = AUTH_SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_to_server(&envelope, false, 0))
    }

    /// Выполняет точную маршрутизацию `Run`; неизвестный тип остаётся no-op.
    pub fn run(&mut self, handlers: &mut dyn LoginMessageHandlers) -> i32 {
        let opcode = self.message_type() as u32;
        if 0x000C_F300 < opcode && opcode < 0x000D_F1FF {
            handlers.on_auth(self);
            return 1;
        }
        match opcode & MESSAGE_FAMILY_MASK {
            0x0002_0000 => handlers.on_gm(self),
            0x0002_0100 => handlers.on_gma(self),
            0x0001_FF00 | 0x0002_FD00 | 0x0001_0000 => handlers.on_log(self),
            0x0000_FF00 | 0x0001_FE00 => handlers.on_server(self),
            _ => {}
        }
        1
    }

    fn from_parts(header: [u8; MESSAGE_HEADER_LEN], payload: &[u8]) -> Self {
        Self {
            base: CBaseMessage::from_header_and_payload(header, payload),
            cdkey: Vec::new(),
            socket_id: 0,
            map_id: 0,
            ip: 0,
        }
    }
}
