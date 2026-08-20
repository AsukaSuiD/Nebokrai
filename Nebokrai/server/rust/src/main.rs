//! Прямая Linux-точка входа восстановленного AuthServer.
//!
//! Статус: `IMPLEMENTED` для технической замены `WinMain`, MFC message pump и
//! Windows game-thread. Доменный lifecycle остаётся у
//! `authserver/src/cgame.rs::game_thread_func`; entry point только создаёт Tokio
//! runtime, переводит `SIGINT/SIGTERM` в исходную команду завершения и сообщает
//! итог оператору через стандартные потоки процесса.
//! Provenance технической оболочки: `authserver.cpp::WinMain` RVA `0x00001E70`,
//! `app.cpp::App::init` RVA `0x00001BD0`, `App::mainLoop` RVA `0x00001000` и
//! `cgame.cpp::GameThreadFunc` RVA `0x000068C0`; точная пара
//! `AuthServer/authserver.exe + AuthServer/authserver.pdb`, SHA-256 EXE
//! `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`, PDB
//! `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`.
//!
//! Оригинал искал `setup.ini`, `allowed_ls.ini` и `client_forbid_ip.ini`
//! относительно рабочего каталога AuthServer. Этот путь сохранён: бинарник не
//! вводит новую CLI-грамматику, environment override или fallback ресурсов.

#[allow(
    dead_code,
    reason = "Auth baseline использует только восстановленный путь; остальные владельцы ещё сырой корпус"
)]
mod authserver;
#[allow(
    dead_code,
    reason = "BillingServer подключён до восстановления полного доменного lifecycle"
)]
mod billingserver;
#[allow(
    dead_code,
    reason = "Auth baseline использует только восстановленный DB-путь; остальные владельцы ещё сырой корпус"
)]
mod dbaccess;
#[allow(
    dead_code,
    reason = "GameServer подключён с парных schedule snapshot decoders до отдельной точки запуска"
)]
mod gameserver;
#[allow(
    dead_code,
    reason = "LoginServer подключён по первому Auth lifecycle до отдельной точки запуска"
)]
mod loginserver;
#[allow(
    dead_code,
    reason = "MiscServer подключён с первого setup-owner без выбора отдельной точки запуска"
)]
mod miscserver;
mod nets;
mod public;
#[allow(
    dead_code,
    reason = "Auth baseline использует первый CServer; остальные transport-границы ещё не подключены"
)]
mod transport;
#[allow(
    dead_code,
    reason = "WorldServer подключён с первого setup-owner без выбора отдельной точки запуска"
)]
mod worldserver;

use std::error::Error;
use std::process::ExitCode;

use authserver::src::cgame::{AuthGameThreadReport, AuthRuntimePaths, game_thread_func};
use tokio::signal::unix::{SignalKind, signal};

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("AuthServer: точка запуска не создана: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, Box<dyn Error>> {
    let runtime_directory = std::env::current_dir()?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(run_auth_server(runtime_directory))
}

async fn run_auth_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    let shutdown = async move {
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
    };

    eprintln!(
        "AuthServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );
    let paths = AuthRuntimePaths::from_runtime_directory(runtime_directory);
    let report = game_thread_func(&paths, shutdown).await;
    Ok(report_auth_result(&report))
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
