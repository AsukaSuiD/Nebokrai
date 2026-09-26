//! Сообщение направления GameServer (`CMessage`): тип и runtime metadata
//! game-направления. Исходник `nets/netserver/message.cpp`; источник
//! контракта — точная пара `.exe/gameserver.exe` SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! ImageBase `0x400000`, PE timestamp `0x53FAFF2D` ↔ `.exe/GameServer.pdb`
//! GUID `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2 (CodeView RSDS, match);
//! PDB-owner `e:\svn\fengyun_russia_dev\nets\netserver\message.cpp`.
//!
//! Адресные свидетельства: ctor `CMessage(long)` RVA `0x000136D0`,
//! `CreateMessage/CreateMessageWithoutRLE` RVA `0x00013700/0x00013850` со
//! статусом `IMPLEMENTED, VERIFIED_DISASSEMBLY` (RLE-путь: capacity
//! `0x800000` для входа до `0x100000` включительно, затем exact
//! `compressed_len * 8`; decode/create/free общего scratch под critical
//! section до возврата). Отдельный send-family — wire/domain — остаётся
//! владельцем процесса стороной доменного расширения этого типа (см.
//! `src/nets/netserver/message.rs`): `SendToSocket/SendToPlayer/SendAll`
//! `0x00013910/0x000139C0/0x00013A70`, `Send/SendToBS` `0x00013B40/0x00013BE0`,
//! region/area/country `0x00014100/0x000142B0/0x00014760`, обе around
//! `0x00014420/0x00014970`, `Run` `0x000149D0` — тот же статус.
//!
//! Машинные точки самого типа, зафиксированные дизассемблером:
//! ctor `CMessage(long)` `0x4136D0` — базовый `0x413520`, vtable `0x64CA48`,
//! `MsgType` в header `+4`, нули пяти runtime dword `+0x18/+0x1C/+0x20/+0x24/+0x28`
//! (region, player, numeric map, socket, IPv4 через этот порядок полей);
//! объект `0x2C` выделяется `new(0x2C)` во всех synthetic-путях (`OnClose`
//! принятого client `0x41C670` — `CMessage(0x6FA01)`, world/billing `OnClose`
//! outgoing `0x41A590` — `0x6F901`/`0x6F903`).
//!
//! Общий `CBaseMessage` уже сохраняет 16-байтовый header, little-endian Add и
//! длину. Этот owner добавляет GameServer type constructor и runtime metadata
//! `region/player/map/socket/IP`; nullable pointers выражены numeric identity
//! через `Option`, а accepted-client context заполняет три scalar-поля после
//! успешной wire-проверки. `Vec`/`Drop` заменяют base destructor и STL.
//! Ненулевой wire короче header и переполнение старой RLE capacity — известные
//! классы повреждённого wire (`HeaderTooShortReactionUnknown` /
//! `RleCapacityOverflowReactionUnknown`), без фиктивной реакции.

use std::fmt;

use nebokrai_shared::network::{
    decode_rle, CBaseMessage, RleDecodeError, ServerClientMessageContext,
};

const MESSAGE_HEADER_LEN: usize = 16;
const SMALL_RLE_INPUT_LIMIT: usize = 0x10_0001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x80_0000;

/// Ошибка восстановления Game-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub enum CreateMessageError {
    /// Нулевой decode-result либо нулевой несжатый вход не создаёт сообщение.
    EmptyInput,
    /// Реакция C++ на ненулевой buffer короче header безопасно не определена.
    HeaderTooShortReactionUnknown,
    /// Размер не представим 32-битным `unsigned long` исходного API.
    InputOutsideLegacyRange,
    /// `compressed_len * 8` переполнял старую 32-битную арифметику.
    RleCapacityOverflowReactionUnknown,
    /// Декодер отклонил поток либо достиг malformed-границы своего owner-а.
    Rle(RleDecodeError),
}

/// Локальная safe-граница GameServer send-family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendMessageError {
    /// Итоговый server envelope не представим положительным Windows `long`.
    LengthOutsideLegacyRange,
    /// BLOCKED_MISSING_FACT: oversized `SendAll` читает неинициализированный
    /// constructor-ом inherited `m_lIndexID` до самой отправки.
    SendAllIndexIdUninitialized,
}

impl fmt::Display for SendMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOutsideLegacyRange => formatter
                .write_str("длина GameServer envelope не представима положительным Windows long"),
            Self::SendAllIndexIdUninitialized => formatter
                .write_str("oversized SendAll требует недоказанное значение CMySocket::m_lIndexID"),
        }
    }
}

impl std::error::Error for SendMessageError {}

/// Владеющее Game-сообщение с runtime metadata принятого server-клиента.
pub struct CMessage {
    base: CBaseMessage,
    region_id: Option<i32>,
    player_id: Option<i32>,
    map_id: i32,
    socket_id: i32,
    ip: u32,
}

impl CMessage {
    pub fn new(message_type: i32) -> Self {
        let mut base = CBaseMessage::new();
        base.set_message_type(message_type);
        Self {
            base,
            region_id: None,
            player_id: None,
            map_id: 0,
            socket_id: 0,
            ip: 0,
        }
    }

    /// Декодирует legacy RLE и создаёт внутреннее Game-сообщение.
    pub fn create(compressed: &[u8]) -> Result<Self, CreateMessageError> {
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
        Self::create_without_rle(&decoded)
    }

    /// Создаёт входящее сообщение из несжатых header + payload.
    pub fn create_without_rle(wire: &[u8]) -> Result<Self, CreateMessageError> {
        if wire.is_empty() {
            return Err(CreateMessageError::EmptyInput);
        }
        if wire.len() > u32::MAX as usize {
            return Err(CreateMessageError::InputOutsideLegacyRange);
        }
        if wire.len() < MESSAGE_HEADER_LEN {
            // BLOCKED_MISSING_FACT: Game RVA 0x00013850 проверяет только
            // ненулевые pointer/len, затем читает header[0..16] и вызывает
            // Add(wire + 16, len - 16). Реакция для 1..15 bytes не доказана.
            return Err(CreateMessageError::HeaderTooShortReactionUnknown);
        }
        let header = wire[..MESSAGE_HEADER_LEN]
            .try_into()
            .expect("длина Game header уже проверена");
        Ok(Self::from_parts(header, &wire[MESSAGE_HEADER_LEN..]))
    }

    pub fn add_byte(&mut self, value: u8) {
        self.base.add_byte(value);
    }

    pub fn add_short(&mut self, value: i16) {
        self.base.add_short(value);
    }

    pub fn add_long(&mut self, value: i32) {
        self.base.add_long(value);
    }

    pub fn add_ulong(&mut self, value: u32) {
        self.base.add_ulong(value);
    }

    pub fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    pub fn unread_bytes(&self) -> &[u8] {
        self.base.unread_bytes()
    }

    pub fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Присваивает metadata принятого игрового client-соединения.
    pub fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.ip = context.peer_ipv4;
    }

    pub const fn map_id(&self) -> i32 {
        self.map_id
    }

    pub const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    pub const fn ip(&self) -> u32 {
        self.ip
    }

    pub const fn player_id(&self) -> Option<i32> {
        self.player_id
    }

    pub const fn region_id(&self) -> Option<i32> {
        self.region_id
    }

    /// Присваивает уже доказанные raw pointer identities локального caller-а.
    pub const fn apply_player_context(&mut self, player_id: i32, region_id: Option<i32>) {
        self.player_id = Some(player_id);
        self.region_id = region_id;
    }

    pub fn message_type(&self) -> i32 {
        self.base.message_type()
    }

    pub fn set_message_type(&mut self, message_type: i32) {
        self.base.set_message_type(message_type);
    }

    fn from_parts(header: [u8; MESSAGE_HEADER_LEN], payload: &[u8]) -> Self {
        Self {
            base: CBaseMessage::from_header_and_payload(header, payload),
            region_id: None,
            player_id: None,
            map_id: 0,
            socket_id: 0,
            ip: 0,
        }
    }
}
