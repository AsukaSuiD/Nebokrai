//! GM-обработчик LoginServer из `applogin/message/gmmessage.cpp`.
//!
//!
//! Единственный известный opcode `0x20001` читает ограниченный 256-байтовый
//! account, затем signed Windows `long` duration и синхронно вызывает
//! `CRsCDKey::CDKeyBan`. Неизвестный opcode остаётся исходным no-op. Обычная
//! нехватка числового payload у общего `CBaseMessage` даёт доказанный legacy-
//! ноль без сдвига курсора; поэтому короткий duration передаётся как `0`, а
//! фактический DB-owner возвращает `false` без SQL side effect.
//!
//! Borrowed `CGame` заменяет глобальный `GetGame`, а `Vec<u8>` — stack-массив
//! `char[256]`; bytes не требуют UTF-8 и передаются единственному уже
//! восстановленному `CRsCDKey`. Отсутствующий DB-owner выражен ошибкой
//! незавершённого Rust lifecycle, а не временным успехом. Исходный handler
//! игнорировал `bool` DB-вызова; typed outcome только делает его доступным
//! component-runner’у и не добавляет побочного эффекта.
//!
//! Попавшие в экспорт `std::transform`, внутренности STL и `$L...` cleanup
//! относятся к заменённым библиотечным/compiler-механизмам большого соседнего
//! handler и удалены. Их память и деструкторы уже выражены владением и `Drop`.

use std::error::Error;
use std::fmt;

use crate::loginserver::loginserver::game::CGame;
use crate::nets::netlogin::message::CMessage;

const CD_KEY_BAN_MESSAGE_TYPE: i32 = 0x0002_0001;
const ACCOUNT_LIMIT: usize = 0x100;

/// Наблюдаемый результат `OnGMMessage` без добавления новой реакции.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageOutcome {
    /// `CRsCDKey::CDKeyBan` вызван, а исходно проигнорированный bool сохранён.
    BanAttempted { succeeded: bool },
    /// Opcode не принадлежит единственной ветви этого владельца.
    Unsupported { message_type: i32 },
}

/// Ошибка обязательной lifecycle-границы GM-handler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageError {
    /// `CGame::Init` ещё не присоединил исходный `CRsCDKey`.
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

/// Узкая композиция `OnGMMessage` с фактическим `CGame` LoginServer.
pub(crate) struct GmMessageHandler<'a> {
    game: &'a mut CGame,
}

impl<'a> GmMessageHandler<'a> {
    /// Связывает GM-handler с текущим LoginServer owner.
    pub(crate) fn new(game: &'a mut CGame) -> Self {
        Self { game }
    }

    /// Выполняет единственный известный GM opcode; остальные оставляет no-op.
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
