//! GM-обработчик `applogin/message/gmmessage.cpp`, подтверждённый
//! `loginserver.exe` и `loginserver.pdb`.
//!
//! Opcode `0x20001` читает account до 256 байт и signed Windows `long`, затем
//! синхронно вызывает `CRsCDKey::CDKeyBan`; остальные opcode — no-op. Короткое
//! числовое поле даёт legacy-ноль без сдвига курсора. Account остаётся набором
//! байт, отсутствие DB-owner-а является ошибкой незавершённого lifecycle.
//! Результат DB-вызова оригинал игнорировал; отчёт не меняет эффектов.

use std::error::Error;
use std::fmt;

use crate::loginserver::loginserver::game::CGame;
use crate::nets::netlogin::message::CMessage;

const CD_KEY_BAN_MESSAGE_TYPE: i32 = 0x0002_0001;
const ACCOUNT_LIMIT: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageOutcome {
    BanAttempted { succeeded: bool },
    Unsupported { message_type: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageError {
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

pub(crate) struct GmMessageHandler<'a> {
    game: &'a mut CGame,
}

impl<'a> GmMessageHandler<'a> {
    pub(crate) fn new(game: &'a mut CGame) -> Self {
        Self { game }
    }

    pub(crate) fn on_gm_message(
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
