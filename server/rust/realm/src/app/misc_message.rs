//! Сообщение сетевого направления MiscServer из `nets/netmisc/message.cpp`;
//! Realm — владелец Misc-направления объединённого Realm-процесса.
//!
//! Источник контракта — точная пара `.exe/miscserver.exe`
//! SHA-256 `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! ImageBase `0x400000`, PE timestamp `0x53A26D7D` ↔ `.exe/miscserver.pdb`
//! GUID `FE6CDEF1-D110-4C25-ABE9-F66D5E665F3B` age 1 (CodeView RSDS).
//!
//! Машинно подтверждённые точки (первая секция `.text`, VA по дизассемблеру):
//! - ctor `0x4107A0` хранит `MsgType` в header-слово `+4` и обнуляет
//!   runtime-поля `+0x18/+0x1C/+0x20/+0x24` объекта `0x28` байт, vtable
//!   `0x4259F0`;
//! - `CreateMessage` `0x4107D0`: весь decode/create/copy/free закрыт
//!   `CRITICAL_SECTION 0x550564` до возврата (оба пути выхода) — защищаемым
//!   состоянием был общий static scratch (`0x450548` статический, `0x550548`
//!   выделяемый); owned `Vec` исключает это единственное global-состояние,
//!   поэтому create-mutex осознанно не вводится. Порог `cmp len,0x20000; jbe` —
//!   вход короче `0x20001` получает capacity `0x100000`, больший — `len*8`;
//!   failure decode `0x410C00` возвращает null; `timeGetTime` пишется в `+0x24`;
//! - `CreateMessageWithoutRLE` `0x410920`: null/нулевая длина → null; та же
//!   16-байтовая header-копия без отдельной проверки `len < 16`, поэтому
//!   текущий `HeaderTooShort` относится к классу повреждённого wire;
//! - `Send` `0x4109F0`: отправитель `g_Game+0x78`, null → 0; сериализация CS
//!   `0x55054C` вокруг envelope helper `0x410B50` (`total_len = len + 0xC`,
//!   scratch `0x55057C`, message копируется сразу после префикса), два общих
//!   `DataCrc32 0x405730`, виртуальный send `[vtable+0x40]` с `prioritized` и
//!   flags `0`, выход через LeaveCriticalSection;
//! - `Run` `0x410AB0`: маска `type - (type & 0xFF)`, `0x14EC00` → пустой
//!   destructor-стаб `0x4057A0` и возврат `1`, `0x14ED00` → `0x4036B0`
//!   (OnMSG_W2M_AUCTION) и `1`, `0x16EA00` → `0x403C10` (OnMSG_M2M_Fuction)
//!   и `1`, прочие → `0x403CC0` (OnOtherMsg) и строго `0` — в отличие от
//!   World-направления.
//!
//! Конструктор создаёт базовый 16-байтовый header, записывает `MsgType` в слово
//! `+4` и обнуляет `MapID`, `SocketID`, IP и receive tick. Оба create-пути
//! копируют все четыре слова входного header, затем нормализуют длину по реально
//! добавленному payload. Rust получает tick параметром: Linux runtime
//! предоставляет совместимый монотонный wrapping-счётчик, а этот wire-владелец
//! не подменяет `timeGetTime` системным временем.
//!
//! `Send` строит межсерверный envelope `[total_len, crc(total_len),
//! crc(message), message]`, где все три слова little-endian, а CRC — IEEE из
//! Shared protocol. `CMyNetClient::SendToServer` немедленно копирует весь вход
//! в owned команду, поэтому локальный `Vec<u8>` совместим по lifetime.
//! Наблюдаемая сериализация исходной `m_CSTemptBuffer` вокруг build/CRC/send
//! сохранена этим статическим mutex: её нельзя удалить только из-за
//! исчезновения общей scratch-памяти.
//!
//! `MessageHandlers` и `MessageSender` — узкие синхронные selector/send
//! границы доменного владельца и фактического клиента; они не принимают
//! игровых решений и не создают общей административной архитектуры.

use parking_lot::Mutex;

use nebokrai_shared::network::{decode_rle, CBaseMessage, RleDecodeError};
use nebokrai_shared::protocol::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const SMALL_RLE_INPUT_LIMIT: usize = 0x20_001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x10_0000;
const MESSAGE_FAMILY_MASK: u32 = 0xFFFF_FF00;

static SEND_SERIALIZER: Mutex<()> = Mutex::new(());

/// Ошибка восстановления сообщения из входного wire-буфера.
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
    /// Декодер отклонил поток либо встретил локально неизвестную malformed-границу.
    Rle(RleDecodeError),
}

/// Ошибка построения 12-байтового межсерверного envelope.
#[derive(Debug, Eq, PartialEq)]
pub enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
}

/// Фактический сетевой владелец, которому `CMessage::Send` передавал готовый
/// envelope с флагом приоритета и нулевыми socket flags.
pub trait MessageSender {
    /// Копирует или принимает во владение весь buffer до возврата.
    fn send_to_server(&self, buffer: &[u8], prioritized: bool, flags: i32) -> i32;
}

/// Три конкретных доменных обработчика, вызывавшихся исходным `Run`.
pub trait MessageHandlers {
    /// Обрабатывает семейство World-to-Misc auction `0x14ED00`.
    fn on_world_auction(&mut self, message: &mut CMessage);

    /// Обрабатывает семейство Misc-to-Misc function `0x16EA00`.
    fn on_misc_function(&mut self, message: &mut CMessage);

    /// Обрабатывает любое другое семейство, кроме особого no-op `0x14EC00`.
    fn on_other(&mut self, message: &mut CMessage);
}

/// Владеющее сообщение направления MiscServer с исходными runtime-метаданными.
///
/// Четыре поля сохраняют origin-layout объекта `0x28` (`+0x18/+0x1C/+0x20/+0x24`
/// обнулены в ctor `0x4107A0`, `+0x24` получал `timeGetTime`); доказанного
/// assignment-пути контекста у принятого Misc-направления нет, поэтому читателей
/// у них в этом владельце пока нет.
#[allow(
    dead_code,
    reason = "runtime-поля исходного layout без доказанного читателя"
)]
pub struct CMessage {
    base: CBaseMessage,
    map_id: i32,
    socket_id: i32,
    ip: u32,
    recv_time_ms: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного типа с нулевыми метаданными.
    pub fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[], 0)
    }

    /// Декодирует RLE и создаёт входящее сообщение с переданным monotonic tick.
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
            .expect("длина header уже проверена");
        Ok(Self::from_parts(
            header,
            &wire[MESSAGE_HEADER_LEN..],
            recv_time_ms,
        ))
    }

    /// Возвращает точный 32-битный `MsgType` из слова header `+4`.
    pub fn message_type(&self) -> i32 {
        self.base.message_type()
    }

    /// Предоставляет payload-владельцам чтение и добавление полей сообщения.
    pub fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Возвращает всё внутреннее сообщение без внешнего server envelope.
    pub fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Строит исходный 12-байтовый CRC-envelope для межсерверной отправки.
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

    /// Сохраняет исходную последовательность build/CRC/send под одним mutex.
    ///
    /// Отсутствующий client даёт `Ok(0)`, как исходный `Send`. Ошибка длины
    /// остаётся явной новой границей 64-битного Linux runtime.
    pub fn send(
        &self,
        sender: Option<&dyn MessageSender>,
        prioritized: bool,
    ) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };

        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_to_server(&envelope, prioritized, 0))
    }

    /// Выполняет точную маршрутизацию исходного `Run` и возвращает его `long`.
    pub fn run(&mut self, handlers: &mut dyn MessageHandlers) -> i32 {
        match self.message_type() as u32 & MESSAGE_FAMILY_MASK {
            // Пустой `CGUID::~CGUID` не даёт отдельного side effect; selector
            // возвращает исходную единицу.
            0x0014_EC00 => 1,
            0x0014_ED00 => {
                handlers.on_world_auction(self);
                1
            }
            0x0016_EA00 => {
                handlers.on_misc_function(self);
                1
            }
            _ => {
                handlers.on_other(self);
                0
            }
        }
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
