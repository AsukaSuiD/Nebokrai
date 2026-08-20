//! Достигнутые wire, send-family и dispatch GameServer `CMessage`.
//!
//! Constructor RVA `0x000136D0` и его использование movement-командами
//! `0xBF603/0xBF604/0xBF605` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `nets/netserver/message.cpp` и `server/gameserver/appserver/moveshape.cpp`.
//!
//! Общий `CBaseMessage` уже сохраняет 16-байтовый header, little-endian Add и
//! длину. Этот owner добавляет GameServer type constructor и runtime metadata
//! `region/player/map/socket/IP`; nullable pointers выражены numeric identity
//! через `Option`, а accepted-client context заполняет три scalar-поля после
//! успешной wire-проверки. `Vec`/`Drop` заменяют base destructor и STL.
//!
//! `CreateMessage/CreateMessageWithoutRLE` RVA `0x00013700/0x00013850` имеют
//! статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`. RLE-путь использует fixed
//! capacity `0x800000` до входной длины `0x100000` включительно, затем exact
//! `compressed_len * 8`; оба пути копируют четыре header-слова и нормализуют
//! первое по реально добавленному payload. Локальный `Vec` устраняет общий
//! decode scratch; машинный код подтвердил, что critical section охватывал
//! только decode/create/copy/free и всегда освобождался до возврата.
//!
//! `SendToSocket/SendToPlayer/SendAll` RVA `0x00013910/0x000139C0/0x00013A70`,
//! `Send/SendToBS` `0x00013B40/0x00013BE0`, region/area/country
//! `0x00014100/0x000142B0/0x00014760` и обе перегрузки `SendToAround`
//! `0x00014420/0x00014970` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`. Wrapper around получает father-region и
//! текущие tile X/Y; deep overload один раз строит `[compressed_len][RLE]`,
//! обходит девять areas в исходном порядке, выбирает только `CPlayer`, исключает
//! pointer-аргумент и синхронно передаёт owned copy в numeric map-ID queue. Для
//! main player с ненулевой командой дополнительно обходится team session:
//! игроки вне `IsInAround` получают тот же frame, кроме сообщения `0xBF502`.
//!
//! Socket send использует `CServer::SendBySocketID`; все player/region/area
//! пути — `SendByMapID`; `SendAll` сохраняет nullable server и общий broadcast.
//! Region проходит физический row-major массив areas, area использует
//! восстановленный owning-region вместо удалённого father pointer, country
//! сравнивает signed argument с zero-extended `m_btCountry`. Межсерверные WS/BS
//! пути строят exact `[total_len, crc(total_len), crc(message), message]` и
//! передают priority без critical section: отсутствие lock подтверждено обоими
//! машинными телами.
//!
//! Старый общий `s_pRLEBuffer` заменён локальным `Vec`: `ServerCommandHandle`
//! копирует байты до возврата. Exact send-family тела вообще не захватывают
//! `m_CSTemptBuffer`; межпоточную сериализацию, которой здесь не было, Rust не
//! вводит.
//! Исходный father raw pointer caller передаёт как typed
//! `Option<&CServerRegion>`, а nullable exception pointer — как уникальный
//! player/map ID; это сознательная смена формы без изменения recipient set.
//! Положительные глобальные `AREA_WIDTH/HEIGHT` выражены проверяемой concrete
//! runtime-границей. `SendAll` oversized-log читает неинициализированное
//! constructor-ом `CMySocket::m_lIndexID`; Rust не подставляет значение и
//! оставляет только эту branch локальным `BLOCKED_MISSING_FACT`. Ненулевой
//! wire короче header и переполнение старого RLE capacity также остаются
//! локальными safe-границами, а не получают придуманную реакцию.
//!
//! `Run` RVA `0x000149D0` имеет статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`: он сначала лениво разрешает player по numeric map ID,
//! затем берёт region из inherited player father и обнуляет младший byte opcode
//! для выбора конкретного handler-owner-а. Пять player/region routes требуют
//! обе ссылки; неизвестные семейства остаются no-op, а return всегда `1`.
//! `GameMessageRoute` — узкая typed-граница ещё сырых domain handlers, не их
//! реализация и не новый общий protocol framework.

use std::fmt;

use crate::gameserver::appserver::area::CArea;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::session::csessionfactory::CSessionFactory;
use crate::gameserver::appserver::shape::{CShape, ShapeCoordinateBlock};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::basemessage::{CBaseMessage, RleDecodeError, decode_rle, encode_rle};
use crate::nets::netserver::mynetclient::CMyNetClient;
use crate::nets::netserver::mynetserver::CMyNetServer;
use crate::nets::serverclient::ServerClientMessageContext;
use crate::public::crc32static::data_crc32;
use crate::public::tools::put_string_to_file;

const PLAYER_TYPE: i32 = 400;
const TEAM_LOCAL_ONLY_MESSAGE: i32 = 0xBF502;
const MESSAGE_HEADER_LEN: usize = 16;
const SERVER_ENVELOPE_LEN: usize = 12;
const OVERSIZED_MESSAGE_LENGTH: usize = 0x80000;
const SMALL_RLE_INPUT_LIMIT: usize = 0x10_0001;
const SMALL_RLE_OUTPUT_CAPACITY: usize = 0x80_0000;
const AROUND_SEND_AREA_OFFSETS: [(i32, i32); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Ошибка восстановления Game-сообщения из wire-буфера.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CreateMessageError {
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

/// Локальная safe-граница reached Game send-family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SendMessageError {
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

/// Конкретный handler-owner, выбранный exact Game `CMessage::Run`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMessageRoute {
    Server,
    Log,
    Other,
    Depot,
    Gm,
    Team,
    OrganizingSystem,
    Country,
    Gma,
    WorldAuction,
    JjcSystem,
    PlayerObject { player_id: i32, region_id: i32 },
    RegionObject { player_id: i32, region_id: i32 },
    MoveShape { player_id: i32, region_id: i32 },
    Goods { player_id: i32, region_id: i32 },
    Skill { player_id: i32, region_id: i32 },
    Shop,
    PlayerShop,
    Container,
    Pet,
    IncrementShop,
    ClientAuction,
    UniBill,
}

/// Синхронная граница доменного owner-а после numeric route selection.
pub(crate) trait GameMessageHandlers {
    fn handle(&mut self, route: GameMessageRoute, message: &mut CMessage);

    /// Typed-граница исходной server-message ветви `0x6F902`.
    fn handle_world_client_reconnected(&mut self, game: &mut CGame, client: CMyNetClient);
}

pub(crate) struct GameServerAroundRuntime<'a> {
    game: &'a CGame,
    sessions: &'a CSessionFactory,
    area_width: i32,
    area_height: i32,
}

impl<'a> GameServerAroundRuntime<'a> {
    /// Создаёт runtime-view только для доказанных положительных area spans.
    pub(crate) fn new(
        game: &'a CGame,
        sessions: &'a CSessionFactory,
        area_width: i32,
        area_height: i32,
    ) -> Option<Self> {
        (area_width > 0 && area_height > 0).then_some(Self {
            game,
            sessions,
            area_width,
            area_height,
        })
    }
}

pub(crate) struct CMessage {
    base: CBaseMessage,
    region_id: Option<i32>,
    player_id: Option<i32>,
    map_id: i32,
    socket_id: i32,
    ip: u32,
}

impl CMessage {
    pub(crate) fn new(message_type: i32) -> Self {
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
                .ok_or(CreateMessageError::RleCapacityOverflowReactionUnknown)?
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

    pub(crate) fn add_byte(&mut self, value: u8) {
        self.base.add_byte(value);
    }

    pub(crate) fn add_long(&mut self, value: i32) {
        self.base.add_long(value);
    }

    pub(crate) fn add_ulong(&mut self, value: u32) {
        self.base.add_ulong(value);
    }

    pub(crate) fn as_wire_bytes(&self) -> &[u8] {
        self.base.as_wire_bytes()
    }

    pub(crate) fn base_mut(&mut self) -> &mut CBaseMessage {
        &mut self.base
    }

    /// Присваивает metadata принятого игрового client-соединения.
    pub(crate) fn apply_client_context(&mut self, context: ServerClientMessageContext<'_>) {
        self.socket_id = context.socket_id;
        self.map_id = context.map_id;
        self.ip = context.peer_ipv4;
    }

    pub(crate) const fn map_id(&self) -> i32 {
        self.map_id
    }

    pub(crate) const fn socket_id(&self) -> i32 {
        self.socket_id
    }

    pub(crate) const fn ip(&self) -> u32 {
        self.ip
    }

    pub(crate) const fn player_id(&self) -> Option<i32> {
        self.player_id
    }

    pub(crate) const fn region_id(&self) -> Option<i32> {
        self.region_id
    }

    /// Присваивает уже доказанные raw pointer identities локального caller-а.
    pub(crate) const fn apply_player_context(&mut self, player_id: i32, region_id: Option<i32>) {
        self.player_id = Some(player_id);
        self.region_id = region_id;
    }

    /// Выполняет exact selector, не материализуя тела выбранных handlers.
    pub(crate) fn run(&mut self, game: &CGame, handlers: &mut dyn GameMessageHandlers) -> i32 {
        if self.player_id.is_none() && self.map_id != 0 {
            self.player_id = game
                .find_player(self.map_id)
                .map(|player| player.player_id());
        }
        if self.region_id.is_none() {
            self.region_id = self
                .player_id
                .and_then(|player_id| game.find_player(player_id))
                .and_then(|player| player.server_region_id());
        }

        let family = self.message_type() as u32 & 0xFFFF_FF00;
        let player_region = self.player_id.zip(self.region_id);
        let route = match family {
            0x0006_F900 | 0x0007_F800 => Some(GameMessageRoute::Server),
            0x0006_FA00 | 0x0007_F900 | 0x0008_F700 => Some(GameMessageRoute::Log),
            0x0007_FA00 | 0x0008_FB00 => Some(GameMessageRoute::Other),
            0x0007_FB00 | 0x0008_FE00 => Some(GameMessageRoute::Depot),
            0x0007_FC00 => Some(GameMessageRoute::Gm),
            0x0006_FB00 | 0x0007_FD00 | 0x0008_FF00 => Some(GameMessageRoute::Team),
            0x0007_FE00 | 0x0009_0100 => Some(GameMessageRoute::OrganizingSystem),
            0x0007_FF00 | 0x0009_0500 => Some(GameMessageRoute::Country),
            0x0008_0000 => Some(GameMessageRoute::Gma),
            0x0008_0400 => Some(GameMessageRoute::WorldAuction),
            0x0008_0500 => Some(GameMessageRoute::JjcSystem),
            0x0008_FA00 => {
                player_region.map(|(player_id, region_id)| GameMessageRoute::PlayerObject {
                    player_id,
                    region_id,
                })
            }
            0x0008_F800 => {
                player_region.map(|(player_id, region_id)| GameMessageRoute::RegionObject {
                    player_id,
                    region_id,
                })
            }
            0x0008_F900 => {
                player_region.map(|(player_id, region_id)| GameMessageRoute::MoveShape {
                    player_id,
                    region_id,
                })
            }
            0x0008_FC00 => player_region.map(|(player_id, region_id)| GameMessageRoute::Goods {
                player_id,
                region_id,
            }),
            0x0009_0000 => player_region.map(|(player_id, region_id)| GameMessageRoute::Skill {
                player_id,
                region_id,
            }),
            0x0008_FD00 => Some(GameMessageRoute::Shop),
            0x0009_0200 => Some(GameMessageRoute::PlayerShop),
            0x0009_0300 | 0x000C_0100 => Some(GameMessageRoute::Container),
            0x0009_0400 | 0x000C_0200 => Some(GameMessageRoute::Pet),
            0x0009_0600 => Some(GameMessageRoute::IncrementShop),
            0x0009_0A00 => Some(GameMessageRoute::ClientAuction),
            0x000F_F000 => Some(GameMessageRoute::UniBill),
            _ => None,
        };
        if let Some(route) = route {
            handlers.handle(route, self);
        }
        1
    }

    pub(crate) fn message_type(&self) -> i32 {
        i32::from_le_bytes(
            self.base.as_wire_bytes()[4..8]
                .try_into()
                .expect("CBaseMessage всегда содержит 16-байтовый header"),
        )
    }

    /// RLE-отправка одному transport socket; возвращает exact queue result.
    pub(crate) fn send_to_socket(&self, net_server: &CMyNetServer, socket_id: i32) -> i32 {
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToSocket", socket_id, frame.len());
        net_server.send_to_socket(socket_id, &frame)
    }

    /// RLE-отправка по numeric player/map identity.
    pub(crate) fn send_to_player(&self, net_server: &CMyNetServer, player_id: i32) -> i32 {
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToPlayer", player_id, frame.len());
        net_server.send_to_player(player_id, &frame)
    }

    /// RLE-broadcast с исходным nullable `s_pNetServer`.
    pub(crate) fn send_all(
        &self,
        net_server: Option<&CMyNetServer>,
    ) -> Result<i32, SendMessageError> {
        let Some(net_server) = net_server else {
            return Ok(0);
        };
        let frame = self.rle_send_frame();
        if frame.len() > OVERSIZED_MESSAGE_LENGTH {
            let index_id = net_server
                .legacy_index_id()
                .ok_or(SendMessageError::SendAllIndexIdUninitialized)?;
            self.log_oversized_rle("SendToAll", index_id, frame.len());
        }
        Ok(net_server.send_all(&frame))
    }

    /// Строит server CRC-envelope и ставит его WorldServer-клиенту.
    pub(crate) fn send(&self, game: &CGame, prioritized: bool) -> Result<i32, SendMessageError> {
        let Some(client) = game.world_client() else {
            return Ok(0);
        };
        let envelope = self.server_envelope()?;
        let _ = client
            .send_queue()
            .send_to_server(&envelope, prioritized, 0);
        Ok(1)
    }

    /// Строит тот же server CRC-envelope и ставит его BillingServer-клиенту.
    pub(crate) fn send_to_bs(
        &self,
        game: &CGame,
        prioritized: bool,
    ) -> Result<i32, SendMessageError> {
        let Some(client) = game.billing_client() else {
            return Ok(0);
        };
        let envelope = self.server_envelope()?;
        let _ = client
            .send_queue()
            .send_to_server(&envelope, prioritized, 0);
        Ok(1)
    }

    /// Обходит все areas region-а в исходном storage order.
    pub(crate) fn send_to_region(
        &self,
        server_region: Option<&CServerRegion>,
        excluded_player_id: Option<i32>,
        game: &CGame,
    ) -> i32 {
        let Some(server_region) = server_region else {
            return 0;
        };
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToRegion", server_region.id, frame.len());
        let mut player_ids = Vec::new();
        server_region.find_all_player_ids(&mut player_ids);
        self.send_player_ids(&player_ids, excluded_player_id, game, &frame);
        1
    }

    /// Отправляет игрокам одной area; owning region заменяет старый father ptr.
    pub(crate) fn send_to_area(
        &self,
        area: Option<(&CServerRegion, &CArea)>,
        excluded_player_id: Option<i32>,
        game: &CGame,
    ) -> i32 {
        let Some((server_region, area)) = area else {
            return 0;
        };
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToArea", area.id(), frame.len());
        let mut player_ids = Vec::new();
        let _ = server_region.find_player_ids_in_area_object(area, &mut player_ids);
        self.send_player_ids(&player_ids, excluded_player_id, game, &frame);
        1
    }

    /// Сохраняет исходную опечатку имени и сравнение `int` с unsigned country.
    pub(crate) fn send_to_region_contry_player(
        &self,
        server_region: Option<&CServerRegion>,
        country: i32,
        game: &CGame,
    ) -> i32 {
        let Some(server_region) = server_region else {
            return 0;
        };
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToRegion", server_region.id, frame.len());
        let mut player_ids = Vec::new();
        server_region.find_all_player_ids(&mut player_ids);
        for player_id in player_ids {
            let Some(player) = game.find_player(player_id) else {
                continue;
            };
            if i32::from(player.country()) == country {
                let _ = game.net_server().send_to_player(player_id, &frame);
            }
        }
        1
    }

    pub(crate) fn send_to_around(
        &self,
        server_region: Option<&CServerRegion>,
        origin: &CShape,
        excluded_player_id: Option<i32>,
        runtime: &GameServerAroundRuntime<'_>,
    ) -> Result<i32, ShapeCoordinateBlock> {
        let tile_x = origin.get_tile_x()?;
        let tile_y = origin.get_tile_y()?;
        Ok(self.send_to_around_at(
            server_region,
            tile_x,
            tile_y,
            Some(origin),
            excluded_player_id,
            runtime,
        ))
    }

    fn send_to_around_at(
        &self,
        server_region: Option<&CServerRegion>,
        tile_x: i32,
        tile_y: i32,
        main_shape: Option<&CShape>,
        excluded_player_id: Option<i32>,
        runtime: &GameServerAroundRuntime<'_>,
    ) -> i32 {
        let Some(server_region) = server_region else {
            return 0;
        };
        let frame = self.rle_send_frame();
        self.log_oversized_rle("SendToAround", server_region.id, frame.len());
        let mut around_player_ids = Vec::new();
        let center_x = tile_x / runtime.area_width;
        let center_y = tile_y / runtime.area_height;
        for (offset_x, offset_y) in AROUND_SEND_AREA_OFFSETS {
            server_region.find_player_ids_in_area(
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
                &mut around_player_ids,
            );
        }

        for player_id in around_player_ids {
            let Some(player) = runtime.game.find_player(player_id) else {
                continue;
            };
            if excluded_player_id == Some(player.player_id()) {
                continue;
            }
            let _ = runtime
                .game
                .net_server()
                .send_to_player(player.player_id(), &frame);
        }

        let Some(main_player) = main_shape
            .filter(|shape| shape.identity().object_type == PLAYER_TYPE)
            .and_then(|shape| runtime.game.find_player(shape.identity().id))
        else {
            return 1;
        };
        if main_player.team_id() == 0 {
            return 1;
        }
        let session_id = runtime
            .game
            .get_team_session_id(main_player.team_id() as u32);
        if session_id == 0 {
            return 1;
        }
        let Some(session) = runtime.sessions.query_session(session_id) else {
            return 1;
        };
        for plug_id in session.get_plug_list() {
            let Some(player) = runtime
                .sessions
                .query_plug(*plug_id)
                .and_then(|plug| plug.get_owner(runtime.game))
            else {
                continue;
            };
            if excluded_player_id == Some(player.player_id()) {
                continue;
            }
            if player
                .shape()
                .is_in_around(main_player.shape(), server_region)
            {
                continue;
            }
            if self.message_type() == TEAM_LOCAL_ONLY_MESSAGE {
                continue;
            }
            let _ = runtime
                .game
                .net_server()
                .send_to_player(player.player_id(), &frame);
        }
        1
    }

    fn rle_send_frame(&self) -> Vec<u8> {
        let compressed = encode_rle(self.base.as_wire_bytes())
            .expect("CMessage всегда содержит непустой 16-байтовый header");
        let total_len = compressed.len().wrapping_add(4) as i32;
        let mut frame = Vec::with_capacity(compressed.len().saturating_add(4));
        frame.extend_from_slice(&total_len.to_le_bytes());
        frame.extend_from_slice(&compressed);
        frame
    }

    fn server_envelope(&self) -> Result<Vec<u8>, SendMessageError> {
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

    fn log_oversized_rle(&self, route: &str, subject: i32, frame_len: usize) {
        if frame_len <= OVERSIZED_MESSAGE_LENGTH {
            return;
        }
        let line = format!(
            "MsgType {:>10} ; {route} {:>10} ; OriginSize {:>10} ; CompressedSize {:>10}.",
            self.message_type() as u32,
            subject,
            self.base.as_wire_bytes().len() as u32,
            frame_len as i32,
        );
        put_string_to_file("MsgLen.log", line.as_bytes());
    }

    fn send_player_ids(
        &self,
        player_ids: &[i32],
        excluded_player_id: Option<i32>,
        game: &CGame,
        frame: &[u8],
    ) {
        for player_id in player_ids {
            let Some(player) = game.find_player(*player_id) else {
                continue;
            };
            if excluded_player_id == Some(player.player_id()) {
                continue;
            }
            let _ = game.net_server().send_to_player(player.player_id(), frame);
        }
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\nets\netserver\message.cpp

// IMPLEMENTED: `CMessage::~CMessage` выражен `Drop` Rust и деструктором
// `CBaseMessage`; покрытый raw-блок удалён.

// IMPLEMENTED: `CMessage::CMessage(long)` материализован выше; покрытый
// raw-блок удалён.

// IMPLEMENTED: обе входящие фабрики и `Run` материализованы выше; покрытый
// raw-псевдокод удалён после сверки точного selector-а и metadata writers.

// CLASSIFIED_TECHNICAL_NOISE: `CMyNetClient` deleting thunk и homogeneous
// constructor/destructor unwind покрыты `Drop`/RAII соответствующего owner-а.

// COMPONENT_VARIANT_END: GameServer
