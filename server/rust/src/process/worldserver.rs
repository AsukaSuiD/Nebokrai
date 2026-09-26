//! Владелец процесса WorldServer под Linux.
//!
//! Оболочка связывает SIGINT/SIGTERM с единственным процессным runtime и
//! публикует итог `CreateGame -> Init -> MainLoop -> Release`, не открывая
//! второй module graph и не повторяя process boilerplate в бинарнике.
//! Процессные owners и драйвер потока игры живут в Realm `app/world_process*`
//! и `app/world_runtime` (волна C5-D); эта оболочка пользуется ими напрямую.

use std::error::Error;

use nebokrai_realm::app::world_game::CGame;
use nebokrai_realm::app::world_process::{
    WorldProcessMainLoopBlock, WorldProcessRuntime,
};
use nebokrai_realm::app::world_runtime::{WorldGameThreadReport, game_thread_func};

use super::{process_shutdown, run_process};

pub fn run_worldserver_process() -> std::process::ExitCode {
    run_process("WorldServer", run_world_server)
}

async fn run_world_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let shutdown = process_shutdown()?;
    let (mut runtime, control) =
        WorldProcessRuntime::new(tokio::runtime::Handle::current(), runtime_directory.clone());
    eprintln!(
        "WorldServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );

    let game_thread = game_thread_func(&mut runtime, new_world_game);
    tokio::pin!(game_thread);
    let report = tokio::select! {
        report = &mut game_thread => report,
        _ = shutdown => {
            control.request_exit();
            game_thread.await
        }
    };
    Ok(report_world_result(report))
}

fn new_world_game() -> Box<CGame> {
    Box::new(CGame::new())
}

fn report_world_result(
    report: WorldGameThreadReport<CGame, std::convert::Infallible, WorldProcessMainLoopBlock>,
) -> bool {
    match report {
        WorldGameThreadReport::Complete {
            main_loop_calls,
            legacy_exit_code,
            ..
        } => {
            eprintln!(
                "WorldServer: штатно завершён после {main_loop_calls} turn; код {legacy_exit_code}"
            );
            true
        }
        WorldGameThreadReport::FatalInitialization { block, .. } => {
            eprintln!("WorldServer: Init достиг подтверждённой аварийной ветки: {:?}", block.reason);
            false
        }
        WorldGameThreadReport::BlockedInitialization { block, .. } => {
            eprintln!("WorldServer: Init остановлен на явно представленной границе: {:?}", block.reason);
            false
        }
        WorldGameThreadReport::BlockedMainLoop {
            main_loop_calls,
            block,
            ..
        } => {
            eprintln!(
                "WorldServer: MainLoop остановлен после {main_loop_calls} turn: {block}"
            );
            false
        }
        WorldGameThreadReport::BlockedRelease {
            main_loop_calls,
            block,
            ..
        } => {
            eprintln!(
                "WorldServer: Release остановлен после {main_loop_calls} turn и {} выполненных шагов",
                block.events.len()
            );
            false
        }
    }
}
