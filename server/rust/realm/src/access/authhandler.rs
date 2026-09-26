//! Embedded `AuthHandler` из `authhandler.cpp`. Первый listener-slot остаётся
//! no-op; второй синхронно передаётся stateless owner-у через адаптер `CGame`.
//!
//! Успех ставит `TagPwdChecked` с пустым world-name и `has_matrix = false`.
//! Ошибка сначала отправляет клиентский код; только default-код `7` затем меняет
//! счётчик паролей. При превышении лимита ban вызывается до безусловного удаления
//! счётчика, независимо от результата DB-owner-а.

use super::authmanager::{AuthQuest, AuthResult};
use super::game::{AuthHandlerNotice, CGame, PasswordFailureOutcome};
use super::loginqueue::TagPwdChecked;
use crate::app::login_message::CMessage;

const AUTH_FAILED_MESSAGE_TYPE: i32 = 0x000A_F501;

pub struct AuthHandler;

impl AuthHandler {
    pub fn on_quest(_quest: &AuthQuest) {}

    pub fn on_response(game: &mut CGame, result: &AuthResult) {
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
