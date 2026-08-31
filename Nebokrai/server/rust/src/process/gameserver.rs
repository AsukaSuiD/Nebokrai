//! Владелец процесса GameServer под Linux.
//!
//! Источник жизненного цикла — `gameserver.exe` и `GameServer.pdb`, исходный
//! owner `gameserver/gameserver.cpp`: оболочка заменяет WinMain и Win32
//! game-thread, переводит SIGINT/SIGTERM в process-owned флаг выхода и
//! публикует итог единственной цепочки `CreateGame -> Init -> MainLoop ->
//! Release -> DeleteGame`. Игровое состояние остаётся у локального `CGame`.

use std::error::Error;

use crate::gameserver::gameserver::game::{
    GameThreadReport, game_thread_func,
};
use crate::gameserver::gameserver::runtime::GameProcessRuntime;

use super::{process_shutdown, run_process};

pub fn run_gameserver_process() -> std::process::ExitCode {
    run_process("GameServer", run_game_server)
}

async fn run_game_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let shutdown = process_shutdown()?;
    let (mut runtime, control) = GameProcessRuntime::new(
        tokio::runtime::Handle::current(),
        runtime_directory.clone(),
    );
    eprintln!(
        "GameServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );

    let game_thread = game_thread_func(&mut runtime);
    tokio::pin!(game_thread);
    let report = tokio::select! {
        report = &mut game_thread => report,
        _ = shutdown => {
            control.request_exit();
            game_thread.await
        }
    };
    Ok(report_game_result(&report))
}

fn report_game_result(report: &GameThreadReport) -> bool {
    match &report.initialization {
        Ok(()) => {
            eprintln!(
                "GameServer: штатно завершён после {} turn",
                report.main_loop_calls
            );
            true
        }
        Err(error) => {
            eprintln!("GameServer: инициализация остановлена: {error}");
            false
        }
    }
}
