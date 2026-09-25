//! GM-обработчик `applogin/message/gmmessage.cpp`, подтверждённый
//! `loginserver.exe` и `loginserver.pdb`, перенесённый в Realm `access/`.
//!
//! Код `0x20001` читает имя учётной записи до 256 байт и знаковый Windows
//! `long`, затем синхронно вызывает `CRsCDKey::CDKeyBan`. Код `0x20002` читает
//! ID игрока и сценария, получает у того же владельца БД действующие блокировки
//! и возвращает строгий `0x4FD05` исходному сокету WorldServer. Остальные коды
//! не выполняют действий. Короткое числовое поле даёт исходный ноль без сдвига
//! позиции чтения. Имя учётной записи остаётся набором байт, отсутствие
//! владельца БД является ошибкой незавершённого жизненного цикла. Результат
//! вызова БД исходная версия игнорировала; отчёт не меняет эффектов.

use std::error::Error;
use std::fmt;

use super::game::CGame;
use crate::app::login_message::CMessage;

const CD_KEY_BAN_MESSAGE_TYPE: i32 = 0x0002_0001;
const ACTIVE_BAN_LIST_REQUEST: i32 = 0x0002_0002;
const ACTIVE_BAN_LIST_RESPONSE: i32 = 0x0004_fd05;
const ACCOUNT_LIMIT: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmMessageOutcome {
    BanAttempted {
        succeeded: bool,
    },
    ActiveBanListReturned {
        requester_player_id: i32,
        script_id: i32,
        success: bool,
        total_count: i32,
        record_count: usize,
        delivered: bool,
    },
    Unsupported {
        message_type: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmMessageError {
    DatabaseOwnerMissing,
}

impl fmt::Display for GmMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseOwnerMissing => {
                formatter.write_str("DB-владелец CRsCDKey для GM-ban отсутствует")
            }
        }
    }
}

impl Error for GmMessageError {}

pub struct GmMessageHandler<'a> {
    game: &'a mut CGame,
}

impl<'a> GmMessageHandler<'a> {
    pub fn new(game: &'a mut CGame) -> Self {
        Self { game }
    }

    pub fn on_gm_message(
        &mut self,
        message: &mut CMessage,
    ) -> Result<GmMessageOutcome, GmMessageError> {
        if message.message_type() == ACTIVE_BAN_LIST_REQUEST {
            let requester_player_id = message.base_mut().get_long().unwrap_or(0);
            let script_id = message.base_mut().get_long().unwrap_or(0);
            let result = if requester_player_id > 0 && script_id > 0 {
                self.game
                    .rs_cdkey_owner_mut()
                    .ok_or(GmMessageError::DatabaseOwnerMissing)?
                    .list_active_bans()
            } else {
                None
            };
            let success = result.is_some();
            let total_count = result.as_ref().map_or(0, |value| value.total_count);
            let records = result.map_or_else(Vec::new, |value| value.records);
            let mut response = CMessage::new(ACTIVE_BAN_LIST_RESPONSE);
            response.base_mut().add_long(requester_player_id);
            response.base_mut().add_long(script_id);
            response.base_mut().add_byte(u8::from(success));
            response.base_mut().add_long(total_count);
            response
                .base_mut()
                .add_byte(u8::from(total_count > records.len() as i32));
            response.base_mut().add_long(records.len() as i32);
            for record in &records {
                response.base_mut().add(&record.account);
                response.base_mut().add_byte(0);
                response.base_mut().add(&record.ban_until);
                response.base_mut().add_byte(0);
            }
            let delivered = self
                .game
                .send_to_world_socket(&response, message.socket_id())
                .is_ok_and(|sent| sent != 0);
            return Ok(GmMessageOutcome::ActiveBanListReturned {
                requester_player_id,
                script_id,
                success,
                total_count,
                record_count: records.len(),
                delivered,
            });
        }
        if message.message_type() != CD_KEY_BAN_MESSAGE_TYPE {
            return Ok(GmMessageOutcome::Unsupported {
                message_type: message.message_type(),
            });
        }

        let account = message
            .base_mut()
            .get_str_bytes(ACCOUNT_LIMIT)
            .expect("ненулевая GetStr-граница задана константой");
        let duration_minutes = message.base_mut().get_long().unwrap_or(0);
        let owner = self
            .game
            .rs_cdkey_owner_mut()
            .ok_or(GmMessageError::DatabaseOwnerMissing)?;
        let succeeded = owner.cd_key_ban(&account, duration_minutes);
        Ok(GmMessageOutcome::BanAttempted { succeeded })
    }
}
