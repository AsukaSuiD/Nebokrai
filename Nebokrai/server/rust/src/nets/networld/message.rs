//! Сообщение направления Login/GameServer <-> WorldServer из
//! `nets/networld/message.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для layout/runtime metadata, RLE и
//! несжатого create-путей, четырёх server-envelope send-направлений и numeric
//! selector `Run`.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\nets\networld\message.cpp`.
//!
//! Существенные RVA: деструктор `0x00022D70`, конструктор `0x00022DA0`,
//! `CreateMessage` `0x00022DD0`, `CreateMessageWithoutRLE` `0x00022F20`,
//! `SendToSocket` `0x00022FF0`, `SendToMapID` `0x000230B0`, `SendAll`
//! `0x00023170`, `Send` `0x00023220` и `Run` `0x000232E0`.
//!
//! Конструктор пишет полный signed `long MsgType` в header `+4` и обнуляет
//! numeric map/socket ID, IPv4 и receive tick. Accepted GameServer получает
//! map/socket/IP из соседнего `CMyServerClient::OnReceive`; сообщения от
//! исходящего LoginServer client сохраняют нулевые metadata. Rust передаёт
//! `timeGetTime`-совместимый tick create-функциям явно, чтобы будущий transport
//! снял его в той же позиции после полного копирования сообщения.
//!
//! Оба create-пути копируют четыре слова входного header, затем нормализуют
//! первое слово по реально добавленному payload. RLE-путь сохраняет порог
//! `0x20001`, capacity `0x100000` либо `compressed_len * 8`. Исходная critical
//! section покрывала декодирование в process-global scratch, создание объекта,
//! копирование и освобождение временного буфера; локальный owned `Vec<u8>`
//! устраняет общую память и не оставляет под этим create-lock внешнего эффекта.
//! Ненулевой вход короче 16-байтового header и переполнение старого умножения
//! остаются локальными safe-границами, а не объявляются поведением оригинала.
//!
//! Все четыре send-функции строят один envelope
//! `[total_len, crc(total_len), crc(message), message]`. `SendToSocket`,
//! `SendToMapID` и `SendAll` адресуют входящие GameServer через server-owner;
//! `Send(bool)` ставит packet исходящему LoginServer client с исходным
//! приоритетом и flags `0`. Старый nullable `g_pGame->s_pNet*` выражен
//! `Option`: его отсутствие возвращает `0` до build, как EXE. Общий
//! `m_CSTemptBuffer` lock охватывал не только scratch, но всю последовательность
//! build/CRC/downstream-send во всех четырёх методах. Поэтому локальный buffer
//! не отменяет наблюдаемую межпоточную сериализацию: один Rust mutex сохраняет
//! её между направлениями. `ServerCommandHandle` и `ClientSendQueue`
//! синхронно копируют bytes в owned-команду до возврата, так что lifetime
//! локального envelope не меняет wire.
//!
//! `Run` обнуляет младший byte полного opcode и выбирает ровно тринадцать
//! исторических handler-владельцев. Семейство `0x60200` вызывает
//! `OnWriteLogMessage` только при `CGame::m_Setup.bUseLogSys`; остальные
//! неизвестные семейства являются no-op. Исходный return всегда равен `1`, в
//! том числе для `OnJJcSystemMessage` и неизвестного opcode. Узкий trait
//! выражает только эти свободные функции и setup-проверку, не создавая общего
//! protocol framework.
//!
//! `CBaseMessage`/`CMyNetClient` deleting destructor, SEH allocation cleanup,
//! ручные new/delete и общий scratch классифицированы как compiler/library
//! noise. Их эффект выражен `CBaseMessage`, локальными коллекциями и `Drop`;
//! не относящийся к сообщению runtime client остаётся соседнему
//! `nets/networld/mynetclient.rs`.

use parking_lot::Mutex;

use crate::nets::basemessage::{CBaseMessage, RleDecodeError, decode_rle};
use crate::nets::clients::ClientSendQueue;
use crate::nets::serverclient::ServerClientMessageContext;
use crate::nets::servers::ServerCommandHandle;
use crate::public::crc32static::data_crc32;

const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const SMALL_RLE_INPUT_LIMIT: usize = 0x20_001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x10_0000;
const MESSAGE_FAMILY_MASK: u32 = 0xFFFF_FF00;

static SEND_SERIALIZER: Mutex<()> = Mutex::new(());

/// Ошибка восстановления World-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CreateMessageError {
    /// Нулевой decode-result либо нулевой несжатый вход не создаёт сообщение.
    EmptyInput,
    /// Реакция C++ на ненулевой буфер короче header безопасно не определена.
    HeaderTooShortReactionUnknown,
    /// Размер не представим 32-битным `unsigned long` исходного API.
    InputOutsideLegacyRange,
    /// `compressed_len * 8` переполнял старую 32-битную арифметику.
    RleCapacityOverflowReactionUnknown,
    /// Декодер отклонил поток либо достиг malformed-границы своего владельца.
    Rle(RleDecodeError),
}

/// Ошибка построения внешнего WorldServer envelope.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SendMessageError {
    /// Итоговая длина не представима положительным Windows `long`.
    LengthOutsideLegacyRange,
}

/// Тринадцать доказанных свободных handler-владельцев WorldServer.
pub(crate) trait WorldMessageHandlers {
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
pub(crate) struct CMessage {
    base: CBaseMessage,
    map_id: i32,
    socket_id: i32,
    ip: u32,
    recv_time_ms: u32,
}

impl CMessage {
    /// Создаёт исходящее сообщение указанного полного типа.
    pub(crate) fn new(message_type: i32) -> Self {
        let mut header = [0; MESSAGE_HEADER_LEN];
        header[4..8].copy_from_slice(&message_type.to_le_bytes());
        Self::from_parts(header, &[], 0)
    }

    /// Декодирует legacy RLE и создаёт внутреннее World-сообщение.
    pub(crate) fn create(compressed: &[u8], recv_time_ms: u32) -> Result<Self, CreateMessageError> {
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
                .ok_or(CreateMessageError::RleCapacityOverflowReactionUnknown)?
        };
        let decoded = decode_rle(compressed, output_capacity).map_err(CreateMessageError::Rle)?;
        Self::create_without_rle(&decoded, recv_time_ms)
    }

    /// Создаёт входящее сообщение из несжатых header + payload.
    pub(crate) fn create_without_rle(
        wire: &[u8],
        recv_time_ms: u32,
    ) -> Result<Self, CreateMessageError> {
        if wire.is_empty() {
            return Err(CreateMessageError::EmptyInput);
        }
        if wire.len() > u32::MAX as usize {
            return Err(CreateMessageError::InputOutsideLegacyRange);
        }
        if wire.len() < MESSAGE_HEADER_LEN {
            // BLOCKED_MISSING_FACT: World RVA 0x00022F20 проверяет только
            // ненулевые pointer/len, затем читает header[0..16] и вызывает
            // Add(wire + 16, len - 16). Наблюдаемая реакция для 1..15 bytes
            // не доказана и не заменяется придуманным fail-closed результатом.
            return Err(CreateMessageError::HeaderTooShortReactionUnknown);
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
    pub(crate) fn message_type(&self) -> i32 {
        i32::from_le_bytes(
            self.base.as_wire_bytes()[4..8]
                .try_into()
                .expect("CBaseMessage всегда содержит 16-байтовый header"),
        )
    }

    /// Меняет только полный `MsgType`, сохраняя payload и runtime metadata.
    pub(crate) fn set_message_type(&mut self, message_type: i32) {
        self.base.set_message_type(message_type);
    }

    /// Предоставляет payload-владельцам доказанные `CBaseMessage::Get/Add`.
    pub(crate) fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Возвращает полное внутреннее сообщение без внешнего server envelope.
    pub(crate) fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    /// Присваивает metadata accepted GameServer-соединения.
    pub(crate) fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.ip = context.peer_ipv4;
    }

    /// Возвращает numeric map identity источника.
    pub(crate) const fn map_id(&self) -> i32 {
        self.map_id
    }

    /// Возвращает socket ID источника.
    pub(crate) const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    /// Возвращает исходное 32-битное представление peer IPv4.
    pub(crate) const fn ip(&self) -> u32 {
        self.ip
    }

    /// Возвращает wrapping millisecond tick приёма.
    pub(crate) const fn recv_time_ms(&self) -> u32 {
        self.recv_time_ms
    }

    /// Строит исходный 12-байтовый CRC-envelope.
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

    /// Ставит server envelope одному GameServer по socket ID.
    pub(crate) fn send_to_socket(
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
    pub(crate) fn send_to_map_id(
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
    pub(crate) fn send_all(
        &self,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<i32, SendMessageError> {
        let Some(sender) = sender else {
            return Ok(0);
        };
        let _guard = SEND_SERIALIZER.lock();
        let envelope = self.server_envelope()?;
        Ok(sender.send_all(&envelope))
    }

    /// Ставит server envelope исходящему LoginServer client.
    pub(crate) fn send(
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
    pub(crate) fn run(&mut self, handlers: &mut dyn WorldMessageHandlers) -> i32 {
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
