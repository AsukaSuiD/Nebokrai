//! Сообщение направления GameServer <-> BillingServer из
//! `nets/netbilling/message.cpp`.
//!
//! Owner сохраняет сжатый и несжатый create-пути, 12-байтовый CRC-envelope,
//! `SendToGS`, `SendToAllGS` и selector `Run`. Контракт подтверждён точной
//! парой BillingServer EXE/PDB.
//!
//! Конструктор пишет полный `MsgType` в header `+4`, оставляет пустой CD-key,
//! нулевые socket/map/IP и null player/region. Rust хранит только metadata,
//! которые соседний `CClientForGS::OnReceive` действительно переносит в
//! сообщение; бестиповые аналоги `m_pPlayer/m_pRegion` не вводятся до их
//! доменного владельца.
//!
//! Оба create-пути копируют четыре слова входного header, затем нормализуют
//! первое слово по реально добавленному payload. RLE-путь сохраняет порог
//! `0x20001`, capacity `0x100000` либо `compressed_len * 8`. Исходная critical
//! section покрывала декодирование в общие `pBuf/pBigBuf`, создание сообщения,
//! копирование и освобождение временного буфера; локальные owned `Vec<u8>`
//! исключают общий scratch и не оставляют под lock внешних эффектов или
//! наблюдаемого порядка. Вход 1..15 bytes и переполнение умножения относятся к
//! внутренним memory/arithmetic defects: safe Rust детерминированно отклоняет
//! их до чтения header или выделения буфера.
//!
//! Оба send-метода строят envelope `[total_len, crc(total_len), crc(message),
//! message]`. Billing `CServer` принимает buffer синхронно и общий Linux-owner
//! копирует его в owned socket-команду до возврата. В исходных `SendToGS` и
//! `SendToAllGS` critical section вокруг общей `s_pTemptBuffer` отсутствовала;
//! безопасный Rust использует локальный buffer и не воспроизводит возможную
//! data race через `unsafe`.
//!
//! `Run` маскирует младший byte opcode: семейства `0x0FF000/0x0EF200`
//! передаются `OnBillingMessage`, `0x0EF100/0x10EF00` — `OnServerMessage`,
//! остальные являются no-op; результат всегда `1`. Два handler-метода — узкая
//! граница именно этих свободных функций, а не новый общий dispatch framework.
//! STL string, allocator, SEH, deleting destructor и compiler cleanup удалены
//! как compiler/library noise; их эффект выражен владением и `Drop`.

use crate::nets::basemessage::{CBaseMessage, RleDecodeError, decode_rle};
use crate::nets::serverclient::ServerClientMessageContext;
use crate::nets::servers::ServerCommandHandle;
use crate::public::crc32static::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const SMALL_RLE_INPUT_LIMIT: usize = 0x20_001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x10_0000;
const MESSAGE_FAMILY_MASK: u32 = 0xFFFF_FF00;

/// Ошибка восстановления Billing-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CreateMessageError {
    /// Нулевой результат декодирования или нулевой несжатый вход не создаёт сообщение.
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

/// Ошибка построения внешнего Billing envelope.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
}

/// Две доказанные свободные handler-функции BillingServer.
pub(crate) trait BillingMessageHandlers {
    /// Обрабатывает семейства `0x0FF000` и `0x0EF200`.
    fn on_billing(&mut self, message: &mut CMessage);

    /// Обрабатывает семейства `0x0EF100` и `0x10EF00`.
    fn on_server(&mut self, message: &mut CMessage);
}

/// Владеющее Billing-сообщение с runtime-метаданными GameServer-соединения.
pub(crate) struct CMessage {
    base: CBaseMessage,
    cdkey: Vec<u8>,
    map_id: i32,
    socket_id: i32,
    ip: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного полного типа.
    pub(crate) fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[])
    }

    /// Декодирует legacy RLE и создаёт внутреннее Billing-сообщение.
    pub(crate) fn create(compressed: &[u8]) -> Result<Self, CreateMessageError> {
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
        Self::create_without_rle(&decoded)
    }

    /// Создаёт входящее сообщение из несжатых header + payload.
    pub(crate) fn create_without_rle(wire: &[u8]) -> Result<Self, CreateMessageError> {
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
            .expect("длина Billing header уже проверена");
        Ok(Self::from_parts(header, &wire[MESSAGE_HEADER_LEN..]))
    }

    /// Возвращает точный полный `MsgType` из слова header `+4`.
    pub(crate) fn message_type(&self) -> i32 {
        self.base.message_type()
    }

    /// Предоставляет доменному владельцу доказанные `CBaseMessage::Get/Add`.
    pub(crate) fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Возвращает полное внутреннее сообщение без внешнего server envelope.
    pub(crate) fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Присваивает metadata, которые `CClientForGS::OnReceive` брал из соединения.
    pub(crate) fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.cdkey.clear();
        self.cdkey.extend_from_slice(context.map_name);
        self.ip = context.peer_ipv4;
    }

    /// Возвращает socket ID исходного GameServer-соединения.
    pub(crate) const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает числовую identity исходного GameServer-соединения.
    pub(crate) const fn map_id(&self) -> i32 {
        self.map_id
    }

    /// Возвращает буквальные bytes исходного `m_strCdkey`.
    pub(crate) fn cdkey(&self) -> &[u8] {
        &self.cdkey
    }

    /// Возвращает исходное 32-битное представление peer IPv4.
    pub(crate) const fn ip(&self) -> u32 {
        self.ip
    }

    /// Строит исходный 12-байтовый CRC-envelope для GameServer.
    pub(crate) fn server_envelope(&self) -> Result<Vec<u8>, SendMessageError> {
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

    /// Копирует server envelope одному GameServer по socket ID.
    pub(crate) fn send_to_gs(
        &self,
        sender: &ServerCommandHandle,
        socket_id: i32,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_by_socket_id(socket_id, &envelope))
    }

    /// Копирует server envelope всем подключённым GameServer.
    pub(crate) fn send_to_all_gs(
        &self,
        sender: &ServerCommandHandle,
    ) -> Result<i32, SendMessageError> {
        let envelope = self.server_envelope()?;
        Ok(sender.send_all(&envelope))
    }

    /// Выполняет точную маршрутизацию `Run`; неизвестный тип остаётся no-op.
    pub(crate) fn run(&mut self, handlers: &mut dyn BillingMessageHandlers) -> i32 {
        match self.message_type() as u32 & MESSAGE_FAMILY_MASK {
            0x000F_F000 | 0x000E_F200 => handlers.on_billing(self),
            0x000E_F100 | 0x0010_EF00 => handlers.on_server(self),
            _ => {}
        }
        1
    }

    fn from_parts(header: [u8; MESSAGE_HEADER_LEN], payload: &[u8]) -> Self {
        Self {
            base: CBaseMessage::from_header_and_payload(header, payload),
            cdkey: Vec::new(),
            map_id: 0,
            socket_id: 0,
            ip: 0,
        }
    }
}
