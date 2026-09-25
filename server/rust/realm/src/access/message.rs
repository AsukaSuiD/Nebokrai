//! Свободные обработчики сообщений LoginServer из `applogin/message`, перенесённые в Realm `access/`.
//!
//! Исторический `CMessage::Run`, подтверждённый точной парой LoginServer
//! EXE/PDB, выбирает
//! ровно одного владельца Auth/GMA, GM, Log либо Server. `LoginComponentRunner`
//! не повторяет его numeric switch: узкий selector принимает callback от
//! `Run`, после чего вызывает соответствующий фактический handler. Auth-путь
//! остаётся awaitable ради упорядоченного завершения прежней reconnect-задачи;
//! остальные владельцы выполняются синхронно в той же позиции сообщения.
//!
//! `process_login_messages` соединяет runner с
//! `CGame::ProcessMessage` и сохраняет результат каждого сообщения в
//! порядке World -> Client -> Auth. Неизвестный opcode получает только
//! no-op `CMessage::Run`; общий protocol framework, runtime-loop и обработчик
//! неподтверждённого `0x10F101` здесь не создаются.

use std::error::Error;
use std::fmt;

use super::asmessage::{AsMessageError, AsMessageHandlers, AsMessageOutcome};
use super::gmmessage::{GmMessageError, GmMessageHandler, GmMessageOutcome};
use super::logmessage::{LogMessageError, LogMessageHandler, LogMessageOutcome};
use super::servermessage::{ServerMessageHandler, ServerMessageOutcome};
use super::authmanager::AuthManager;
use super::game::{CGame, GameRouteError, ProcessMessageError};
use crate::app::login_message::{CMessage, LoginMessageHandlers};

#[derive(Debug)]
pub enum LoginComponentMessageOutcome {
    Auth(AsMessageOutcome),
    Gm(GmMessageOutcome),
    Gma(AsMessageOutcome),
    Log(LogMessageOutcome),
    Server(ServerMessageOutcome),
    Ignored {
        message_type: i32,
    },
}

#[derive(Debug)]
pub struct LoginProcessMessageOutcome {
    pub legacy_result: i32,
    pub messages: Vec<LoginComponentMessageOutcome>,
}

#[derive(Debug)]
pub enum LoginComponentMessageError {
    Auth(AsMessageError),
    Gm(GmMessageError),
    Log(LogMessageError),
    Server(GameRouteError),
}

impl fmt::Display for LoginComponentMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(error) => error.fmt(formatter),
            Self::Gm(error) => error.fmt(formatter),
            Self::Log(error) => error.fmt(formatter),
            Self::Server(error) => error.fmt(formatter),
        }
    }
}

impl Error for LoginComponentMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Auth(error) => Some(error),
            Self::Gm(error) => Some(error),
            Self::Log(error) => Some(error),
            Self::Server(error) => Some(error),
        }
    }
}

pub struct LoginComponentRunner<'a> {
    auth_manager: &'a mut AuthManager,
}

impl<'a> LoginComponentRunner<'a> {
    pub fn new(auth_manager: &'a mut AuthManager) -> Self {
        Self { auth_manager }
    }

    pub async fn run(
        &mut self,
        game: &mut CGame,
        message: &mut CMessage,
    ) -> Result<LoginComponentMessageOutcome, LoginComponentMessageError> {
        let mut selector = LoginOwnerSelector::default();
        let legacy_result = message.run(&mut selector);
        debug_assert_eq!(legacy_result, 1);

        match selector.owner {
            Some(LoginMessageOwner::Auth) => AsMessageHandlers::new(game, self.auth_manager)
                .on_as_message(message)
                .await
                .map(LoginComponentMessageOutcome::Auth)
                .map_err(LoginComponentMessageError::Auth),
            Some(LoginMessageOwner::Gm) => GmMessageHandler::new(game)
                .on_gm_message(message)
                .map(LoginComponentMessageOutcome::Gm)
                .map_err(LoginComponentMessageError::Gm),
            Some(LoginMessageOwner::Gma) => AsMessageHandlers::new(game, self.auth_manager)
                .on_gma_message(message)
                .map(LoginComponentMessageOutcome::Gma)
                .map_err(LoginComponentMessageError::Auth),
            Some(LoginMessageOwner::Log) => LogMessageHandler::new(game)
                .on_log_message(message)
                .map(LoginComponentMessageOutcome::Log)
                .map_err(LoginComponentMessageError::Log),
            Some(LoginMessageOwner::Server) => ServerMessageHandler::new(game)
                .on_server_message(message)
                .map(LoginComponentMessageOutcome::Server)
                .map_err(LoginComponentMessageError::Server),
            None => Ok(LoginComponentMessageOutcome::Ignored {
                message_type: message.message_type(),
            }),
        }
    }
}

pub async fn process_login_messages(
    game: &mut CGame,
    auth_manager: &mut AuthManager,
) -> Result<LoginProcessMessageOutcome, ProcessMessageError<LoginComponentMessageError>> {
    let mut runner = LoginComponentRunner::new(auth_manager);
    let mut messages = Vec::new();
    let legacy_result = game
        .process_message(async |game, message| {
            messages.push(runner.run(game, message).await?);
            Ok(())
        })
        .await?;
    Ok(LoginProcessMessageOutcome {
        legacy_result,
        messages,
    })
}

#[derive(Clone, Copy, Debug)]
enum LoginMessageOwner {
    Auth,
    Gm,
    Gma,
    Log,
    Server,
}

#[derive(Default)]
struct LoginOwnerSelector {
    owner: Option<LoginMessageOwner>,
}

impl LoginMessageHandlers for LoginOwnerSelector {
    fn on_auth(&mut self, _message: &mut CMessage) {
        self.owner = Some(LoginMessageOwner::Auth);
    }

    fn on_gm(&mut self, _message: &mut CMessage) {
        self.owner = Some(LoginMessageOwner::Gm);
    }

    fn on_gma(&mut self, _message: &mut CMessage) {
        self.owner = Some(LoginMessageOwner::Gma);
    }

    fn on_log(&mut self, _message: &mut CMessage) {
        self.owner = Some(LoginMessageOwner::Log);
    }

    fn on_server(&mut self, _message: &mut CMessage) {
        self.owner = Some(LoginMessageOwner::Server);
    }
}
