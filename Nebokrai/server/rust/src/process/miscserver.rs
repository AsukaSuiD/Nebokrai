//! Linux process-owner MiscServer.
//!
//! Оболочка сохраняет исходный runtime-каталог, передаёт SIGINT/SIGTERM
//! единственному `CGame` и публикует transport/auction diagnostics без
//! дублирования общего module graph.

use std::error::Error;

use crate::miscserver::miscserver::game::{
    MiscClientConnectOutcome, MiscComponentHandlerOutcome, MiscGameThreadReport,
    MiscGameThreadTurn, MiscInitializationEnd, MiscProcessMemoryQuery, game_thread_func,
};
use crate::miscserver::miscserver::miscservermessage::WorldAuctionOutcome;
use crate::miscserver::miscserver::onbillserver::MiscFunctionOutcome;
use crate::miscserver::miscserver::othermessage::OtherMessageOutcome;
use crate::public::aucitionroom::TerminalGoodsDelivery;

use super::{process_shutdown, run_process};

/// Запускает полный MiscServer process lifecycle и возвращает код процесса.
pub fn run_miscserver_process() -> std::process::ExitCode {
    run_process("MiscServer", run_misc_server)
}

async fn run_misc_server(runtime_directory: std::path::PathBuf) -> Result<bool, Box<dyn Error>> {
    let shutdown = process_shutdown()?;
    eprintln!(
        "MiscServer: запуск; runtime-каталог {}",
        runtime_directory.display()
    );
    let report = game_thread_func(
        &runtime_directory,
        shutdown,
        report_connect_attempt,
        report_misc_turn,
    )
    .await;
    Ok(report_misc_result(&report))
}

fn report_connect_attempt(attempt: &MiscClientConnectOutcome) {
    match attempt {
        MiscClientConnectOutcome::Connected { endpoint, sends } => {
            eprintln!("MiscServer: подключён к WorldServer {endpoint}");
            if let Err(error) = &sends.registration {
                eprintln!("MiscServer: не построена регистрация WorldServer: {error:?}");
            }
            if let Some(Err(error)) = &sends.initial_sync {
                eprintln!("MiscServer: не построен initial auction sync: {error:?}");
            }
        }
        MiscClientConnectOutcome::Failed(error) => {
            eprintln!("MiscServer: подключение к WorldServer не удалось: {error:?}");
        }
    }
}

fn report_misc_turn(turn: &MiscGameThreadTurn) {
    if let Some(Err(error)) = &turn.network {
        eprintln!("MiscServer: ошибка World transport: {error}");
    }
    if let Some(refresh) = &turn.refresh {
        if let MiscProcessMemoryQuery::Unavailable(error) = &refresh.memory.query {
            eprintln!("MiscServer: не прочитана process-memory статистика: {error}");
        }
        if refresh.memory.auction_sync_reset {
            eprintln!("MiscServer: auction sync сброшен из-за resident memory");
        }
    }

    if turn.auction.deletion.is_err() {
        eprintln!("MiscServer: auction deletion остановлен безопасной границей");
    }
    for dispatch in turn.auction.sucessed.iter().chain(turn.auction.back.iter()) {
        match &dispatch.delivery {
            TerminalGoodsDelivery::SerializeBlocked(_) => {
                eprintln!("MiscServer: terminal auction item не сериализован")
            }
            TerminalGoodsDelivery::Sent(Err(error)) => {
                eprintln!("MiscServer: terminal auction response не построен: {error:?}")
            }
            TerminalGoodsDelivery::Sent(Ok(_)) => {}
        }
    }

    for message in &turn.messages.messages {
        match &message.handler {
            MiscComponentHandlerOutcome::GuidFamilyNoOp => {}
            MiscComponentHandlerOutcome::WorldAuction(outcome) => {
                report_world_auction(message.message_type, outcome)
            }
            MiscComponentHandlerOutcome::MiscFunction(MiscFunctionOutcome::Unhandled)
            | MiscComponentHandlerOutcome::Other(OtherMessageOutcome::Unhandled) => eprintln!(
                "MiscServer: неподдерживаемый opcode {:#x}",
                message.message_type
            ),
            MiscComponentHandlerOutcome::MiscFunction(MiscFunctionOutcome::Reconnect(outcome)) => {
                report_connect_attempt(outcome)
            }
            MiscComponentHandlerOutcome::MiscFunction(
                MiscFunctionOutcome::AuctionRoomCleared { send: Err(error) },
            )
            | MiscComponentHandlerOutcome::Other(OtherMessageOutcome::Response {
                send: Err(error),
                ..
            }) => eprintln!("MiscServer: response не построен: {error:?}"),
            MiscComponentHandlerOutcome::Other(OtherMessageOutcome::MissingClient) => {
                eprintln!("MiscServer: World status request получен без client-owner")
            }
            MiscComponentHandlerOutcome::MiscFunction(
                MiscFunctionOutcome::AuctionRoomCleared { send: Ok(_) },
            )
            | MiscComponentHandlerOutcome::Other(OtherMessageOutcome::Response {
                send: Ok(_),
                ..
            }) => {}
        }
    }

    if let Some(reconnect) = &turn.reconnect {
        report_connect_attempt(&reconnect.connection);
        if let Err(error) = &reconnect.confirmation {
            eprintln!("MiscServer: reconnect confirmation не построен: {error:?}");
        }
    }
}

fn report_world_auction(message_type: i32, outcome: &WorldAuctionOutcome) {
    match outcome {
        WorldAuctionOutcome::Unhandled => {
            eprintln!("MiscServer: неподдерживаемый auction opcode {message_type:#x}")
        }
        WorldAuctionOutcome::UnserializeRejected(_) => {
            eprintln!("MiscServer: auction item отклонён на malformed payload")
        }
        WorldAuctionOutcome::MissingGoodsType(_) => {
            eprintln!("MiscServer: auction item не содержит доказанного goods type")
        }
        WorldAuctionOutcome::UnitySerializeBlocked { count_warning, .. } => eprintln!(
            "MiscServer: auction unity остановлен сериализацией; превышен лимит: {count_warning}"
        ),
        WorldAuctionOutcome::SelfAuctionSerializeBlocked(_)
        | WorldAuctionOutcome::AuctionPageBuildBlocked(_) => {
            eprintln!("MiscServer: auction response остановлен безопасной границей")
        }
        WorldAuctionOutcome::OperationResponse { send: Err(error) }
        | WorldAuctionOutcome::SelfAuctionResponse { send: Err(error) }
        | WorldAuctionOutcome::AuctionPageResponse { send: Err(error) } => {
            eprintln!("MiscServer: auction response не построен: {error:?}")
        }
        WorldAuctionOutcome::UnityCompleted { sends, .. } if sends.iter().any(Result::is_err) => {
            eprintln!("MiscServer: часть auction unity responses не построена")
        }
        _ => {}
    }
}

fn report_misc_result(report: &MiscGameThreadReport) -> bool {
    let mut clean = true;
    match &report.initialization.setup {
        Some(Ok(setup)) => eprintln!(
            "MiscServer: setup.ini — разобрано {} пар",
            setup.parsed_pairs
        ),
        Some(Err(error)) => {
            clean = false;
            eprintln!("MiscServer: setup не загружен: {error}");
        }
        None => {}
    }
    match &report.initialization.end {
        MiscInitializationEnd::Connected => {}
        MiscInitializationEnd::Cancelled => {
            eprintln!("MiscServer: инициализация отменена сигналом завершения")
        }
        MiscInitializationEnd::DebugFileUnavailable { path, source } => {
            clean = false;
            eprintln!(
                "MiscServer: не создан обязательный {}: {source}",
                path.display()
            );
        }
    }
    if clean {
        eprintln!(
            "MiscServer: штатно завершён после {} connect-попыток и {} turn",
            report.initialization.attempt_count, report.completed_turns
        );
    }
    clean
}
