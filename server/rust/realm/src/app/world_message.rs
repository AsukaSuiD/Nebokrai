//! Сообщение направления Login/GameServer ↔ WorldServer из
//! `nets/networld/message.cpp`; Realm — сетевой край и диспетчеризация
//! World-направления объединённого Realm-процесса. Формат base-буфера и RLE
//! держит Shared network; процессный scratch-конверт и `CRITICAL_SECTION`
//! заменены owned `Vec` и статическим `Mutex`, не внося второго объекта.
//!
//! Create-пути копируют 16-байтовый header и нормализуют первое слово по
//! реально добавленному payload (порог `0x20001` — capacity `0x100000` либо
//! `len*8`; RLE create пишет recv-tick из `timeGetTime`); `HeaderTooShort`
//! относится к классу повреждённого wire-входа. Все четыре send-ветки
//! сериализуются статическим CS и строят envelope `[total_len, crc(total_len),
//! crc(message), message]`; null-отправитель возвращает 0.
//!
//! `Run` маскирует младший byte opcode и знает тринадцать handler-целей
//! (server, log, gma, player, other, gm, team, orgasys, write-log с
//! gate-byte, country, server-auction, jjc, misc-auction); любая ветвь,
//! включая неизвестный тип, возвращает `1`.
//!
//! `ApplyClientContext` у исходника нет: runtime-поля заполнял принимающий
//! server-путь; здесь это отдельный метод перед публикацией сообщения.
//!
//! Доказательства: docs/reconstruction/realm-services.md#world-wire-сообщение

use parking_lot::Mutex;

use nebokrai_shared::network::{
    decode_rle, CBaseMessage, ClientSendQueue, RleDecodeError, ServerClientMessageContext,
    ServerCommandHandle,
};
use nebokrai_shared::protocol::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const SMALL_RLE_INPUT_LIMIT: usize = 0x20_001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x10_0000;
const MESSAGE_FAMILY_MASK: u32 = 0xFFFF_FF00;

static SEND_SERIALIZER: Mutex<()> = Mutex::new(());

/// Ошибка восстановления World-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub enum CreateMessageError {
    /// Нулевой decode-result либо нулевой несжатый вход не создаёт сообщение.
    EmptyInput,
    /// Внутренний wire-буфер короче обязательного header.
    HeaderTooShort { actual: usize },
    /// Размер не представим 32-битным `unsigned long` исходного API.
    InputOutsideLegacyRange,
    /// Требуемая RLE-capacity не представима в 32-битном исходном диапазоне.
    RleCapacityOutsideLegacyRange,
    /// Декодер отклонил поток либо достиг malformed-границы своего владельца.
    Rle(RleDecodeError),
}

/// Ошибка построения внешнего WorldServer envelope.
#[derive(Debug, Eq, PartialEq)]
pub enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
}

/// Отказ постановки локального World-сообщения при отсутствующем net-server:
/// такое сообщение не ставится, но уже совершённые эффекты caller-а
/// не откатываются.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLocalMessageQueueBlock {
    pub message_type: i32,
}

impl std::fmt::Display for WorldLocalMessageQueueBlock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "локальное World-сообщение {:#08X} не поставлено: s_pNetServer отсутствует",
            self.message_type
        )
    }
}

impl std::error::Error for WorldLocalMessageQueueBlock {}

/// Тринадцать доказанных свободных handler-владельцев WorldServer.
pub trait WorldMessageHandlers {
    /// Возвращает достигнутый `CGame::m_Setup.bUseLogSys` для `0x60200`.
    fn write_log_enabled(&self) -> bool;

    /// Обрабатывает server-семейства `0x3FC00`, `0x4FC00` и `0x5FA00`.
    fn on_server(&mut self, message: &mut CMessage);
    /// Обрабатывает log-семейства `0x4FB00` и `0x5FB00`.
    fn on_log(&mut self, message: &mut CMessage);
    /// Обрабатывает GMA-семейства `0x4FD00` и `0x60400`.
    fn on_gma(&mut self, message: &mut CMessage);
    /// Обрабатывает player-семейство `0x5FC00`.
    fn on_player(&mut self, message: &mut CMessage);
    /// Обрабатывает other-семейство `0x5FD00`.
    fn on_other(&mut self, message: &mut CMessage);
    /// Обрабатывает GM-семейство `0x5FF00`.
    fn on_gm(&mut self, message: &mut CMessage);
    /// Обрабатывает team-семейство `0x60000`.
    fn on_team(&mut self, message: &mut CMessage);
    /// Обрабатывает organizing-system семейство `0x60100`.
    fn on_orgasys(&mut self, message: &mut CMessage);
    /// Обрабатывает разрешённое setup-ом write-log семейство `0x60200`.
    fn on_write_log(&mut self, message: &mut CMessage);
    /// Обрабатывает country-семейства `0x60300` и `0x7FF00`.
    fn on_country(&mut self, message: &mut CMessage);
    /// Обрабатывает GameServer-to-World auction семейство `0x60800`.
    fn on_server_auction(&mut self, message: &mut CMessage);
    /// Обрабатывает JJC-system семейство `0x60900`.
    fn on_jjc_system(&mut self, message: &mut CMessage);
    /// Обрабатывает MiscServer-to-World auction семейство `0x15EB00`.
    fn on_misc_auction(&mut self, message: &mut CMessage);
}

/// Владеющее World-сообщение с runtime-метаданными соединения-источника.
pub struct CMessage {
    base: CBaseMessage,
    map_id: i32,
    socket_id: i32,
    ip: u32,
    recv_time_ms: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного полного типа.
    pub fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[], 0)
    }

    /// Декодирует legacy RLE и создаёт внутреннее World-сообщение.
    pub fn create(compressed: &[u8], recv_time_ms: u32) -> Result<Self, CreateMessageError> {
        if compressed.is_empty() {
            return Err(CreateMessageError::EmptyInput);
        }
        if compressed.len() > u32::MAX as usize {
            return Err(CreateMessageError::InputOutsideLegacyRange);
        }

        let output_capacity = if compressed.len() < SMALL_RLE_INPUT_LIMIT {
            SMALL_RLE_OUTPUT_CAPACITY
        } else {
            compressed
                .len()
                .checked_mul(8)
                .filter(|capacity| *capacity <= u32::MAX as usize)
                .ok_or(CreateMessageError::RleCapacityOutsideLegacyRange)?
        };
        let decoded = decode_rle(compressed, output_capacity).map_err(CreateMessageError::Rle)?;
        Self::create_without_rle(&decoded, recv_time_ms)
    }

    /// Создаёт входящее сообщение из несжатых header + payload.
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
            .expect("длина World header уже проверена");
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

    /// Меняет только полный `MsgType`, сохраняя payload и runtime metadata.
    pub fn set_message_type(&mut self, message_type: i32) {
        self.base.set_message_type(message_type);
    }

    /// Предоставляет payload-владельцам доказанные `CBaseMessage::Get/Add`.
    pub fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Возвращает полное внутреннее сообщение без внешнего server envelope.
    pub fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Присваивает metadata accepted GameServer-соединения.
    pub fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.ip = context.peer_ipv4;
    }

    /// Возвращает numeric map identity источника.
    pub const fn map_id(&self) -> i32 {
        self.map_id
    }

    /// Возвращает socket ID источника.
    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает исходное 32-битное представление peer IPv4.
    pub const fn ip(&self) -> u32 {
        self.ip
    }

    /// Возвращает wrapping millisecond tick приёма.
    pub const fn recv_time_ms(&self) -> u32 {
        self.recv_time_ms
    }

    /// Строит исходный 12-байтовый CRC-envelope.
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

    /// Ставит server envelope одному GameServer по socket ID.
    pub fn send_to_socket(
        &self,
        sender: Option<&ServerCommandHandle>,
        socket_id: i32,
    ) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };
        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_socket_id(socket_id, &envelope))
    }

    /// Ставит server envelope одному GameServer по numeric map identity.
    pub fn send_to_map_id(
        &self,
        sender: Option<&ServerCommandHandle>,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };
        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_map_id(map_id, &envelope))
    }

    /// Ставит server envelope всем подключённым GameServer.
    pub fn send_all(&self, sender: Option<&ServerCommandHandle>) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };
        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_all(&envelope))
    }

    /// Ставит server envelope исходящему LoginServer client.
    pub fn send(
        &self,
        sender: Option<&ClientSendQueue>,
        prioritized: bool,
    ) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };
        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_to_server(&envelope, prioritized, 0))
    }

    /// Выполняет точную маршрутизацию `Run`; неизвестный тип остаётся no-op.
    pub fn run(&mut self, handlers: &mut dyn WorldMessageHandlers) -> i32 {
        match self.message_type() as u32 & MESSAGE_FAMILY_MASK {
            0x0003_FC00 | 0x0004_FC00 | 0x0005_FA00 => handlers.on_server(self),
            0x0004_FB00 | 0x0005_FB00 => handlers.on_log(self),
            0x0004_FD00 | 0x0006_0400 => handlers.on_gma(self),
            0x0005_FC00 => handlers.on_player(self),
            0x0005_FD00 => handlers.on_other(self),
            0x0005_FF00 => handlers.on_gm(self),
            0x0006_0000 => handlers.on_team(self),
            0x0006_0100 => handlers.on_orgasys(self),
            0x0006_0200 if handlers.write_log_enabled() => handlers.on_write_log(self),
            0x0006_0300 | 0x0007_FF00 => handlers.on_country(self),
            0x0006_0800 => handlers.on_server_auction(self),
            0x0006_0900 => handlers.on_jjc_system(self),
            0x0015_EB00 => handlers.on_misc_auction(self),
            _ => {}
        }
        1
    }

    fn from_parts(header: [u8; MESSAGE_HEADER_LEN], payload: &[u8], recv_time_ms: u32) -> Self {
        Self {
            base: CBaseMessage::from_header_and_payload(header, payload),
            map_id: 0,
            socket_id: 0,
            ip: 0,
            recv_time_ms,
        }
    }
}
