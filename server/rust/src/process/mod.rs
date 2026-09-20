//! Process-level владельцы точек запуска серверных бинарников.
//!
//! Общая оболочка Rust: единый tracing subscriber устанавливается до создания
//! Tokio runtime, чтобы события всех потоков попадали в stderr без ANSI.

use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::process::ExitCode;

use tokio::signal::unix::{SignalKind, signal};
use tracing_subscriber::EnvFilter;

mod authserver;
mod billingserver;
mod gameserver;
mod loginserver;
mod miscserver;
mod worldserver;

pub use authserver::run_authserver_process;
pub use billingserver::run_billingserver_process;
pub use gameserver::run_gameserver_process;
pub use loginserver::run_loginserver_process;
pub use miscserver::run_miscserver_process;
pub use worldserver::run_worldserver_process;

fn run_process<Run, RunFuture>(service: &str, run: Run) -> ExitCode
where
    Run: FnOnce(PathBuf) -> RunFuture,
    RunFuture: Future<Output = Result<bool, Box<dyn Error>>>,
{
    let result = (|| -> Result<bool, Box<dyn Error>> {
        let filter = match std::env::var("RUST_LOG") {
            Ok(value) => EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into())
                .parse(value)
                .ok(),
            Err(std::env::VarError::NotPresent) => Some(EnvFilter::new("warn")),
            Err(std::env::VarError::NotUnicode(_)) => None,
        }
        .unwrap_or_else(|| {
            eprintln!(
                "{service}: предупреждение: некорректный RUST_LOG; используется фильтр warn"
            );
            EnvFilter::new("warn")
        });
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;

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
