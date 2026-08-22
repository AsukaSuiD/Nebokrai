//! Linux process-owner BillingServer.
//!
//! Владелец сохраняет исходный runtime-каталог для `Setup.ini` и
//! `GSInfoSetup.ini`, передаёт `SIGINT/SIGTERM` единственному
//! `GameThreadFunc` и публикует наблюдаемые ошибки без Windows/MFC оболочки.

use std::error::Error;

use crate::billingserver::appbilling::billingmessage::BillingMessageOutcome;
use crate::billingserver::appbilling::servermessage::ServerMessageOutcome;
use crate::billingserver::billingserver::game::{
    BillingGameMessageOutcome, BillingGameThreadReport, BillingRuntimePaths, BillingRuntimeStep,
    game_thread_func,
};
use crate::nets::servers::{AdmissionOutcome, ServerIoCompletion};
use super::{process_shutdown, run_process};

/// Запускает полный BillingServer process lifecycle и возвращает код процесса.
pub fn run_billingserver_process() -> std::process::ExitCode {
    run_process("BillingServer", run_billing_server)
}

async fn run_billing_server(
    runtime_directory: std::path::PathBuf,
) -> Result<bool, Box<dyn Error>> {
    let shutdown = process_shutdown()?;

    eprintln!(
        "BillingServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );
    let paths = BillingRuntimePaths::from_runtime_directory(runtime_directory);
    let report = game_thread_func(&paths, shutdown, report_billing_step).await;
    Ok(report_billing_result(&report))
}

fn report_billing_step(step: &BillingRuntimeStep) {
    for admission in &step.network.admissions {
        match admission {
            AdmissionOutcome::AtCapacity => {
                eprintln!("BillingServer: соединение отклонено по лимиту клиентов")
            }
            AdmissionOutcome::AddressRejected => {
                eprintln!("BillingServer: соединение отклонено списком разрешённых адресов")
            }
            AdmissionOutcome::TemporarilyForbidden => {
                eprintln!("BillingServer: соединение временно запрещено")
            }
            AdmissionOutcome::Queued { .. } => {}
        }
    }
    for outcome in &step.message_outcomes {
        match outcome {
            BillingGameMessageOutcome::Billing(BillingMessageOutcome::Unsupported {
                message_type,
            }) => eprintln!("BillingServer: неизвестный Billing opcode {message_type:#x}"),
            BillingGameMessageOutcome::Billing(
                BillingMessageOutcome::AccountRequestIgnoredEmpty,
            ) => eprintln!("BillingServer: отклонён запрос с пустой учётной записью"),
            BillingGameMessageOutcome::Billing(
                BillingMessageOutcome::AccountRequestQueued { queued: false, .. }
                | BillingMessageOutcome::IncrementPurchaseQueued { queued: false, .. }
                | BillingMessageOutcome::PlayerTradeQueued { queued: false, .. },
            ) => eprintln!("BillingServer: рабочая очередь отклонила запрос"),
            BillingGameMessageOutcome::Server(ServerMessageOutcome::Connected {
                socket_id,
                address,
                ..
            }) => eprintln!(
                "BillingServer: GameServer {address} подключён, socket {socket_id}"
            ),
            BillingGameMessageOutcome::Server(ServerMessageOutcome::Disconnected {
                map_id,
                address,
                port,
                allowed,
            }) => eprintln!(
                "BillingServer: GameServer map {map_id} отключён: {}:{port}, адрес {}",
                String::from_utf8_lossy(address),
                if *allowed { "разрешён" } else { "не разрешён" }
            ),
            BillingGameMessageOutcome::Server(ServerMessageOutcome::Unsupported {
                message_type,
            }) => eprintln!("BillingServer: неизвестный server opcode {message_type:#x}"),
            BillingGameMessageOutcome::Billing(
                BillingMessageOutcome::AccountRequestQueued { queued: true, .. }
                | BillingMessageOutcome::IncrementPurchaseQueued { queued: true, .. }
                | BillingMessageOutcome::PlayerTradeQueued { queued: true, .. },
            ) => {}
        }
    }
    for event in &step.events {
        eprintln!("BillingServer: runtime-событие {event:?}");
    }
    for notice in &step.billing_player_notices {
        eprintln!("BillingServer: событие DB worker: {notice:?}");
    }
    for notice in &step.player_fill_notices {
        eprintln!("BillingServer: событие PlayerFill: {notice:?}");
    }
    for error in &step.network.accept_errors {
        eprintln!("BillingServer: ошибка accept: {error}");
    }
    for error in &step.network.snapshot_errors {
        eprintln!("BillingServer: ошибка обработки network snapshot: {error:?}");
    }
    for completion in &step.network.io_completions {
        match completion {
            ServerIoCompletion::ReceiveEnded {
                socket_id,
                error: Some(error),
            } => eprintln!("BillingServer: ошибка чтения socket {socket_id}: {error}"),
            ServerIoCompletion::SendEnded {
                socket_id,
                result: Err(error),
            } => eprintln!("BillingServer: ошибка отправки socket {socket_id}: {error}"),
            ServerIoCompletion::ReceiveEnded { error: None, .. }
            | ServerIoCompletion::SendEnded { result: Ok(_), .. } => {}
        }
    }
}

fn report_billing_result(report: &BillingGameThreadReport) -> bool {
    let mut clean = true;
    match &report.initialization {
        Ok(initialization) => {
            eprintln!(
                "BillingServer: Setup.ini — разобрано {} пар",
                initialization.setup.parsed_pairs
            );
            for notice in &initialization.notices {
                eprintln!("BillingServer: предупреждение инициализации: {notice:?}");
            }
        }
        Err(error) => {
            clean = false;
            eprintln!("BillingServer: инициализация остановлена: {error:?}");
        }
    }
    if let Some(error) = &report.runtime_error {
        clean = false;
        eprintln!("BillingServer: main-loop остановлен ошибкой: {error:?}");
    }
    if report.release.billing_player_manager_error.is_some()
        || report.release.player_fill_error.is_some()
        || !report.release.network_task_errors.is_empty()
    {
        clean = false;
        eprintln!(
            "BillingServer: освобождение завершилось с ошибками: {:?}",
            report.release
        );
    }
    if clean {
        eprintln!("BillingServer: штатно завершён");
    }
    clean
}
