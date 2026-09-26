//! GM-обработчик `applogin/message/gmmessage.cpp`.
//!
//! Код `0x20001` читает имя учётной записи до 256 байт и знаковый Windows
//! `long`, затем синхронно вызывает `CRsCDKey::CDKeyBan`. Остальные коды
//! не выполняют действий. Собственная ветвь `0x20002` без машинного основания
//! удалена. Короткое числовое поле даёт исходный ноль без сдвига
//! позиции чтения. Имя учётной записи остаётся набором байт, отсутствие
//! владельца БД является ошибкой незавершённого жизненного цикла. Результат
//! вызова БД исходная версия игнорировала; отчёт не меняет эффектов.

use std::error::Error;
use std::fmt;

use super::game::CGame;
use crate::app::login_message::CMessage;

const CD_KEY_BAN_MESSAGE_TYPE: i32 = 0x0002_0001;
const ACCOUNT_LIMIT: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmMessageOutcome {
    BanAttempted {
        succeeded: bool,
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
