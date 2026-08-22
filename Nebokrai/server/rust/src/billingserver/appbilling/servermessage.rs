//! Две lifecycle-ветви GameServer у BillingServer из
//! `appbilling/servermessage.cpp`.
//!
//! Исходный путь PDB:
//!
//! `0xEF101` не читает payload: socket ID и peer IPv4 берутся из runtime-
//! metadata принятого сообщения. При наличии `CGame::m_pGSServer` первым
//! побочным эффектом ставится общая команда `SetClientMapID(socket, socket)`,
//! затем безусловно публикуется операторская запись connected. Старый MFC
//! `LB_INSERTSTRING` добавлял тот же dotted IPv4 в список GameServer; typed
//! outcome сохраняет адрес один раз для `CGame::ProcessMessage` и не создаёт
//! Windows GUI-аналог.
//!
//! `0x10EF01` последовательно читает `map_id`, C-строку с границей `0x20` и
//! Windows `long` port. Только при наличии server-owner выполняется прямой
//! `IsAllowedAddress(address, port as u16)`; его результат выбирает обычную
//! либо invalid lost-запись. Проверка вызывается независимо от admission-
//! флага allow-list, как исходный отдельный метод. `map_id` и port сохраняются
//! как signed payload-слова, хотя старый operator log форматировал их `%lu` и
//! тем самым показывал тот же 32-битный pattern как unsigned.
//!
//! Найденные журналы точной BillingServer-пары независимо подтверждают формы
//! `GameServer ID<IPv4> Connected OK` и
//! `[Invalid ]GameServer ID<IPv4:port> Lost`; персональные значения в код и
//! документацию не переносятся. Недостаток numeric payload сохраняет общий
//! `CBaseMessage::GetLong == 0`, а ограниченная строка использует уже
//! восстановленный `GetStr`.
//!
//! `nullptr` сообщения заменён обязательной Rust-ссылкой; nullable
//! `GetGame()/m_pGSServer` представлен `Option<&CServerForGS>`. Полный
//! `std::list`, generated field-copy/destructor, SEH, allocator и `$L/$E`
//! cleanup этого compilation unit удалены как library/compiler noise. Их
//! существенный эффект выражен owned полями, `Clone/Drop` и уже подключёнными
//! общими FIFO `CBillingPlayerManager`.

use std::net::Ipv4Addr;

use crate::nets::netbilling::message::CMessage;
use crate::nets::netbilling::serverforgs::CServerForGS;

const GAME_SERVER_CONNECTED_MESSAGE_TYPE: i32 = 0x000E_F101;
const GAME_SERVER_DISCONNECTED_MESSAGE_TYPE: i32 = 0x0010_EF01;
const GAME_SERVER_ADDRESS_LIMIT: usize = 0x20;

/// Наблюдаемый результат одной ветви Billing `OnServerMessage`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ServerMessageOutcome {
    /// GameServer подключился; `map_assignment` отсутствует только без server-owner.
    Connected {
        socket_id: i32,
        address: Ipv4Addr,
        map_assignment: Option<i32>,
    },
    /// GameServer отключился после явной проверки payload-адреса по allow-list.
    Disconnected {
        map_id: i32,
        address: Vec<u8>,
        port: i32,
        allowed: bool,
    },
    /// Неизвестный тип сохраняет исходный no-op.
    Unsupported { message_type: i32 },
}

/// Узкая композиция свободного handler с nullable `CGame::m_pGSServer`.
pub(crate) struct ServerMessageHandler<'a> {
    server: Option<&'a CServerForGS>,
}

impl<'a> ServerMessageHandler<'a> {
    /// Создаёт handler для текущего состояния Billing `CGame`.
    pub(crate) const fn new(server: Option<&'a CServerForGS>) -> Self {
        Self { server }
    }

    /// Выполняет две подтверждённые ветви, сохраняя неизвестный opcode как no-op.
    pub(crate) fn on_server_message(&self, message: &mut CMessage) -> ServerMessageOutcome {
        match message.message_type() {
            GAME_SERVER_CONNECTED_MESSAGE_TYPE => self.on_connected(message),
            GAME_SERVER_DISCONNECTED_MESSAGE_TYPE => self.on_disconnected(message),
            message_type => ServerMessageOutcome::Unsupported { message_type },
        }
    }

    fn on_connected(&self, message: &CMessage) -> ServerMessageOutcome {
        let socket_id = message.socket_id();
        let map_assignment = self.server.map(|server| {
            server
                .command_handle()
                .set_client_map_id(socket_id, socket_id)
        });
        let address = Ipv4Addr::from(message.ip().to_le_bytes());

        ServerMessageOutcome::Connected {
            socket_id,
            address,
            map_assignment,
        }
    }

    fn on_disconnected(&self, message: &mut CMessage) -> ServerMessageOutcome {
        let map_id = message.base_mut().get_long().unwrap_or(0);
        let address = message
            .base_mut()
            .get_str_bytes(GAME_SERVER_ADDRESS_LIMIT)
            .expect("ненулевая GetStr-граница задана константой");
        let port = message.base_mut().get_long().unwrap_or(0);
        let allowed = self
            .server
            .is_some_and(|server| server.is_allowed_address(&address, port as u16));

        ServerMessageOutcome::Disconnected {
            map_id,
            address,
            port,
            allowed,
        }
    }
}
