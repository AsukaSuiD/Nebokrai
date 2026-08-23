//! Владелец процесса LoginServer под Linux.
//!
//! Оболочка читает обязательный `area_id` из исходного `setupex.ini`, передаёт
//! сигналы завершения единственному `CGame` и потребляет runtime/DB-наблюдения,
//! не публикуя account, password и SQL credentials.

use std::error::Error;

use crate::dbaccess::logindb::rscdkey::RsCdKeyNotice;
use crate::loginserver::applogin::acclogthread::AccLogThreadNotice;
use crate::loginserver::applogin::message::LoginComponentMessageOutcome;
use crate::loginserver::applogin::message::asmessage::AsMessageOutcome;
use crate::loginserver::applogin::message::gmmessage::GmMessageOutcome;
use crate::loginserver::applogin::message::logmessage::LogMessageOutcome;
use crate::loginserver::applogin::message::servermessage::ServerMessageOutcome;
use crate::loginserver::loginserver::game::{
    AuthHandlerNotice, LoginGameThreadReport, LoginMainLoopOutcome, LoginRuntimeStep,
    LoginServerInfoTurn, WorldOperatorLogRecord, game_thread_func, load_runtime_area_id,
};
use crate::nets::servers::{AdmissionOutcome, ServerIoCompletion};

use super::{process_shutdown, run_process};

pub fn run_loginserver_process() -> std::process::ExitCode {
    run_process("LoginServer", run_login_server)
}

async fn run_login_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let area_id = load_runtime_area_id(&runtime_directory)?;
    let shutdown = process_shutdown()?;
    eprintln!(
        "LoginServer: запуск area {area_id}; runtime-каталог {}",
        runtime_directory.display()
    );
    let report = game_thread_func(&runtime_directory, area_id, shutdown, report_login_step).await;
    Ok(report_login_result(&report))
}

fn report_login_step(step: &LoginRuntimeStep) {
    report_network_direction(
        "World",
        &step.network.world.admissions,
        &step.network.world.accept_errors,
        &step.network.world.io_completions,
    );
    report_network_direction(
        "Client",
        &step.network.client.admissions,
        &step.network.client.accept_errors,
        &step.network.client.io_completions,
    );
    if let Some(Err(error)) = &step.network.auth {
        eprintln!("LoginServer: ошибка Auth transport: {error}");
    }

    if let Some(LoginMainLoopOutcome::Continue {
        gas,
        account_logs,
        messages,
        login_queue,
        server_info,
        ..
    }) = &step.main_loop
    {
        for outcome in gas {
            eprintln!("LoginServer: результат GAS: {outcome:?}");
        }
        for notice in account_logs {
            if let AccLogThreadNotice::DatabaseFailure {
                kind,
                operation,
                error,
            } = notice
            {
                eprintln!("LoginServer: ошибка AccountLog DB ({kind:?}, {operation:?}): {error}");
            }
        }
        for outcome in &messages.messages {
            let unsupported = match outcome {
                LoginComponentMessageOutcome::Auth(AsMessageOutcome::Unknown {
                    message_type,
                    ..
                })
                | LoginComponentMessageOutcome::Gma(AsMessageOutcome::Unknown {
                    message_type,
                    ..
                })
                | LoginComponentMessageOutcome::Gm(GmMessageOutcome::Unsupported {
                    message_type,
                })
                | LoginComponentMessageOutcome::Log(LogMessageOutcome::Unsupported {
                    message_type,
                })
                | LoginComponentMessageOutcome::Server(ServerMessageOutcome::Unsupported {
                    message_type,
                })
                | LoginComponentMessageOutcome::Ignored { message_type } => Some(message_type),
                _ => None,
            };
            if let Some(message_type) = unsupported {
                eprintln!("LoginServer: неподдерживаемый opcode {message_type:#x}");
            }
        }
        if !login_queue.notices.is_empty() {
            eprintln!(
                "LoginServer: LoginQueue завершил turn с {} ошибками маршрута/данных",
                login_queue.notices.len()
            );
        }
        if let Err(error) = &login_queue.auth {
            eprintln!("LoginServer: ошибка AuthManager: {error}");
        }
        if let LoginServerInfoTurn::WorkerStartFailed { error, .. } = server_info {
            eprintln!("LoginServer: не запущен worker ServerInfoLog: {error}");
        }
    }

    for notice in &step.auth_handler_notices {
        match notice {
            AuthHandlerNotice::ClientResponseFailed { socket_id, error } => {
                eprintln!("LoginServer: ответ Auth не отправлен client socket {socket_id}: {error}")
            }
            AuthHandlerNotice::KickOutFailed { error, .. } => {
                eprintln!("LoginServer: не выполнен KickOut прежней сессии: {error}")
            }
            AuthHandlerNotice::CdKeyBanFailed { .. } => {
                eprintln!("LoginServer: CDKeyBan вернул отказ")
            }
            AuthHandlerNotice::CdKeyBanOwnerMissing { .. } => {
                eprintln!("LoginServer: CDKeyBan вызван без DB-owner")
            }
        }
    }
    for record in &step.world_operator_log_records {
        match record {
            WorldOperatorLogRecord::InvalidConnection => {
                eprintln!("LoginServer: отклонено неизвестное World-соединение")
            }
            WorldOperatorLogRecord::Connected { world_name } => eprintln!(
                "LoginServer: WorldServer {} подключён",
                String::from_utf8_lossy(world_name)
            ),
            WorldOperatorLogRecord::Lost { world_name } => eprintln!(
                "LoginServer: WorldServer {} отключён",
                String::from_utf8_lossy(world_name)
            ),
        }
    }
    for report in &step.online_user_database_reports {
        if !report.failures.is_empty() {
            eprintln!(
                "LoginServer: UpdateOnlineUser world {} — {} DB-ошибок из {} записей",
                report.world_id,
                report.failures.len(),
                report.requested_accounts
            );
        }
    }
    for report in &step.server_info_database_reports {
        if let Some(failure) = &report.failure {
            eprintln!("LoginServer: ошибка ServerInfoLog DB: {failure:?}");
        }
    }
    for notice in &step.rs_cdkey_notices {
        match notice {
            RsCdKeyNotice::DatabaseFailure { operation, error } => {
                eprintln!("LoginServer: ошибка CD-key DB ({operation:?}): {error}")
            }
            RsCdKeyNotice::BanApplied {
                minutes, inserted, ..
            } => eprintln!(
                "LoginServer: применён CD-key ban на {minutes} мин.; новая запись: {inserted}"
            ),
        }
    }
}

fn report_network_direction(
    direction: &str,
    admissions: &[AdmissionOutcome],
    accept_errors: &[std::io::Error],
    completions: &[ServerIoCompletion],
) {
    for admission in admissions {
        match admission {
            AdmissionOutcome::AtCapacity => {
                eprintln!("LoginServer: {direction}-соединение отклонено по лимиту клиентов")
            }
            AdmissionOutcome::AddressRejected => {
                eprintln!("LoginServer: {direction}-соединение отклонено списком адресов")
            }
            AdmissionOutcome::TemporarilyForbidden => {
                eprintln!("LoginServer: {direction}-соединение временно запрещено")
            }
            AdmissionOutcome::Queued { .. } => {}
        }
    }
    for error in accept_errors {
        eprintln!("LoginServer: ошибка {direction} accept: {error}");
    }
    for completion in completions {
        match completion {
            ServerIoCompletion::ReceiveEnded {
                socket_id,
                error: Some(error),
            } => eprintln!("LoginServer: ошибка чтения {direction} socket {socket_id}: {error}"),
            ServerIoCompletion::SendEnded {
                socket_id,
                result: Err(error),
            } => eprintln!("LoginServer: ошибка отправки {direction} socket {socket_id}: {error}"),
            ServerIoCompletion::ReceiveEnded { error: None, .. }
            | ServerIoCompletion::SendEnded { result: Ok(_), .. } => {}
        }
    }
}

fn report_login_result(report: &LoginGameThreadReport) -> bool {
    let mut clean = true;
    match &report.initialization {
        Ok(initialization) => {
            let prefix = &initialization.through_gas.prefix;
            eprintln!(
                "LoginServer: setup загружен, World routes: {}, Auth endpoints: {}",
                prefix.world_routes, prefix.auth_servers.loaded
            );
            if let Err(error) = &prefix.setup_ex {
                clean = false;
                eprintln!("LoginServer: setupex.ini не перечитан в Init: {error}");
            }
            if let Err(error) = &prefix.online_user_clear {
                clean = false;
                eprintln!("LoginServer: стартовая очистка online_user не выполнена: {error}");
            }
            for attempt in &prefix.auth.attempts {
                if let Some(error) = &attempt.failure {
                    eprintln!("LoginServer: попытка подключения к Auth не удалась: {error}");
                }
            }
            if prefix.auth.connected.is_none() {
                clean = false;
                eprintln!("LoginServer: запуск продолжен без соединения с AuthServer");
            }
            if let Err(error) = &initialization.through_gas.gas_thread_start {
                clean = false;
                eprintln!("LoginServer: не запущен GAS worker: {error}");
            }
            if let Err(error) = &initialization.account_log_thread_start {
                clean = false;
                eprintln!("LoginServer: не запущен AccountLog worker: {error}");
            }
        }
        Err(error) => {
            clean = false;
            eprintln!("LoginServer: инициализация остановлена: {error}");
        }
    }
    if let Some(error) = &report.runtime_error {
        clean = false;
        eprintln!("LoginServer: main-loop остановлен ошибкой: {error}");
    }
    if !report.release.world_network_task_errors.is_empty()
        || !report.release.client_network_task_errors.is_empty()
        || report
            .release
            .online_user_clear
            .as_ref()
            .is_some_and(Result::is_err)
    {
        clean = false;
        eprintln!("LoginServer: освобождение завершилось с ошибками");
    }
    if clean {
        eprintln!(
            "LoginServer: штатно завершён после {} turn",
            report.completed_turns
        );
    }
    clean
}
