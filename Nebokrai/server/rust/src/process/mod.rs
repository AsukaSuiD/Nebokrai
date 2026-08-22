//! Process-level владельцы точек запуска серверных бинарников.

use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::process::ExitCode;

use tokio::signal::unix::{SignalKind, signal};

mod authserver;
mod billingserver;

pub use authserver::run_authserver_process;
pub use billingserver::run_billingserver_process;

fn run_process<Run, RunFuture>(service: &str, run: Run) -> ExitCode
where
    Run: FnOnce(PathBuf) -> RunFuture,
    RunFuture: Future<Output = Result<bool, Box<dyn Error>>>,
{
    let result = (|| -> Result<bool, Box<dyn Error>> {
        let runtime_directory = std::env::current_dir()?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        runtime.block_on(run(runtime_directory))
    })();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("{service}: точка запуска не создана: {error}");
            ExitCode::FAILURE
        }
    }
}

fn process_shutdown() -> std::io::Result<impl Future<Output = ()>> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    Ok(async move {
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
    })
}
