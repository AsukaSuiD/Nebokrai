//! Linux process-owner AuthServer.
//!
//! Техническая оболочка заменяет `WinMain`, MFC message pump и Windows
//! game-thread. Доменный lifecycle остаётся у `CGame::GameThreadFunc`;
//! process-owner только создаёт Tokio runtime, переводит `SIGINT/SIGTERM` в
//! команду завершения и сообщает итог оператору. Оригинальные runtime-файлы
//! разрешаются относительно текущего каталога без новой CLI-грамматики.

use std::error::Error;

use crate::authserver::src::cgame::{
    AuthGameThreadReport, AuthRuntimePaths, AuthRuntimeStep, game_thread_func,
};
use crate::nets::servers::ServerIoCompletion;
use super::{process_shutdown, run_process};

/// Запускает полный AuthServer process lifecycle и возвращает код процесса.
pub fn run_authserver_process() -> std::process::ExitCode {
    run_process("AuthServer", run_auth_server)
}

async fn run_auth_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let shutdown = process_shutdown()?;

    eprintln!(
        "AuthServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );
    let paths = AuthRuntimePaths::from_runtime_directory(runtime_directory);
    let report = game_thread_func(&paths, shutdown, report_auth_step).await;
    Ok(report_auth_result(&report))
}

fn report_auth_step(step: &AuthRuntimeStep) {
    for notice in &step.login_server_notices {
        eprintln!("AuthServer: событие LoginServer: {notice}");
    }
    for notice in &step.database_notices {
        eprintln!(
            "AuthServer: операция DB «{}» завершилась ошибкой: {}",
            notice.operation, notice.error
        );
    }
    for error in &step.accept_errors {
        eprintln!("AuthServer: ошибка accept: {error}");
    }
    for error in &step.network_errors {
        eprintln!("AuthServer: ошибка обработки network snapshot: {error:?}");
    }
    for completion in &step.io_completions {
        match completion {
            ServerIoCompletion::ReceiveEnded {
                socket_id,
                error: Some(error),
            } => eprintln!("AuthServer: ошибка чтения socket {socket_id}: {error}"),
            ServerIoCompletion::SendEnded {
                socket_id,
                result: Err(error),
            } => eprintln!("AuthServer: ошибка отправки socket {socket_id}: {error}"),
            ServerIoCompletion::ReceiveEnded { error: None, .. }
            | ServerIoCompletion::SendEnded { result: Ok(_), .. } => {}
        }
    }
}

fn report_auth_result(report: &AuthGameThreadReport) -> bool {
    let mut clean = true;
    match &report.initialization {
        Ok(initialization) => {
            for notice in &initialization.notices {
                eprintln!("AuthServer: предупреждение инициализации: {notice}");
            }
            eprintln!(
                "AuthServer: обновление server-info {}",
                if initialization.update_server_info_enabled {
                    "включено"
                } else {
                    "отключено"
                }
            );
        }
        Err(error) => {
            clean = false;
            eprintln!("AuthServer: инициализация остановлена: {error}");
        }
    }

    if let Some(error) = &report.runtime_error {
        clean = false;
        eprintln!("AuthServer: main-loop остановлен ошибкой: {error}");
    }
    for error in &report.release.database_worker_errors {
        clean = false;
        eprintln!("AuthServer: ошибка освобождения: {error}");
    }
    if let Some(error) = &report.release.network_error {
        clean = false;
        eprintln!("AuthServer: ошибка освобождения network: {error}");
    }

    if clean {
        eprintln!("AuthServer: штатно завершён");
    }
    clean
}
