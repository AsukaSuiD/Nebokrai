//! Владелец ответа AuthServer `AuthHandler` из `authhandler.cpp`.
//!
//! Контракт обеих virtual-функций подтверждён точной парой LoginServer EXE/PDB.
//! Первый slot, вызываемый `AuthManager::addQuest`, является no-op. `OnResponse` при
//! результате `0` создаёт owned `TagPwdChecked` с пустым world-name и
//! `has_matrix = false`, затем передаёт его в `CLoginQueue`. При совпадении
//! account очередь вызывает `CGame::KickOut` под исходным queue-lock, удаляет
//! прежний объект и добавляет новый в хвост.
//!
//! Любой ненулевой результат сначала отправляет клиенту `0xAF501` с одним
//! байтом. Коды `2..=6` и `8..=12` имеют отдельное отображение;
//! `1`, `7` и остальные значения идут через исходный default `7`. Только
//! default-ветвь после client-send изменяет счётчик неверного пароля. Первая
//! ошибка записывает `1`, последующие увеличивают значение, пока старое
//! значение меньше лимита; следующая ошибка вызывает `CRsCDKey::CDKeyBan` и
//! удаляет счётчик независимо от результата вызова.
//!
//! В оригинале `mAuthHandler` был встроен в `CGame`, а callbacks находили тот
//! же глобальный `CGame`. Самоссылочная Rust-структура для этого не создаётся:
//! `CGame` реализует технический `AuthListener`-адаптер и немедленно передаёт
//! оба вызова этому stateless owner. Такая форма API меняет только владение,
//! но сохраняет синхронный порядок callbacks. `std::string`, `std::map`, SEH,
//! ручные `new/delete` и compiler cleanup удалены как технический шум; их
//! существенные эффекты выражены owned bytes, владельцами и `Drop`.

use crate::loginserver::loginserver::authmanager::{AuthQuest, AuthResult};
use crate::loginserver::loginserver::game::{AuthHandlerNotice, CGame, PasswordFailureOutcome};
use crate::loginserver::loginserver::loginqueue::TagPwdChecked;
use crate::nets::netlogin::message::CMessage;

const AUTH_FAILED_MESSAGE_TYPE: i32 = 0x000A_F501;

/// Stateless-владелец двух virtual slot исходного embedded `AuthHandler`.
pub(crate) struct AuthHandler;

impl AuthHandler {
    /// Сохраняет доказанный no-op первого virtual slot.
    pub(crate) fn on_quest(_quest: &AuthQuest) {}

    /// Выполняет полный доказанный switch исходного `OnResponse`.
    pub(crate) fn on_response(game: &mut CGame, result: &AuthResult) {
        if result.result == 0 {
            let checked = TagPwdChecked::new(
                result.client_socket_id,
                result.client_ip,
                result.account.clone(),
                Vec::new(),
                false,
            );
            game.push_back_pwd_checked(checked);
            return;
        }

        let response_code = response_code(result.result);
        let mut message = CMessage::new(AUTH_FAILED_MESSAGE_TYPE);
        message.base_mut().add_char(response_code);
        if let Err(error) = game.send_to_client(&message, result.client_socket_id) {
            game.push_auth_handler_notice(AuthHandlerNotice::ClientResponseFailed {
                socket_id: result.client_socket_id,
                error,
            });
        }

        if response_code != 7 {
            return;
        }

        match game.register_password_failure(&result.account) {
            PasswordFailureOutcome::BanAttempted { succeeded: false } => {
                game.push_auth_handler_notice(AuthHandlerNotice::CdKeyBanFailed {
                    account: result.account.clone(),
                });
            }
            PasswordFailureOutcome::BanOwnerMissing => {
                game.push_auth_handler_notice(AuthHandlerNotice::CdKeyBanOwnerMissing {
                    account: result.account.clone(),
                });
            }
            PasswordFailureOutcome::Disabled
            | PasswordFailureOutcome::Counted { .. }
            | PasswordFailureOutcome::BanAttempted { succeeded: true } => {}
        }
    }
}

const fn response_code(result: i32) -> i8 {
    match result {
        2 => 5,
        3 => 6,
        4 => 73,
        5 => 18,
        6 => 63,
        8 => 82,
        9 => 83,
        10 => 84,
        11 => 87,
        12 => 88,
        _ => 7,
    }
}
