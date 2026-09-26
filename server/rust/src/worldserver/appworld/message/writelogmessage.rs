//! Журналы `OnWriteLogMessage` из `writelogmessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`; охвачены opcode `0x60201..0x60218`.
//!
//! Все ветви диспетчера перенесены в `realm/app/writelogmessage.rs`; здесь
//! остаются dispatcher-адаптер к `CGame` и re-export типов исходов и команд.

use crate::nets::networld::message::CMessage;
use crate::public::auctionlog::CAuctionLog;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::worldserver::game::CGame;
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

pub(crate) use nebokrai_realm::app::writelogmessage::WorldWriteLogMessageDispatch;
pub(crate) use nebokrai_realm::persistence::writelog::{
    WorldFactionLogWrite, WorldWriteLogCommand,
};

/// Исполняет действующие write-log ветки, reserved no-op IDs и default
/// внешнего switch без side effects.
pub(crate) fn on_write_log_message(
    game: &mut CGame,
    increment_log: &mut CIncrementLog,
    auction_log: &mut CAuctionLog,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    message: CMessage,
) -> WorldWriteLogMessageDispatch {
    nebokrai_realm::app::writelogmessage::on_write_log_message(
        game,
        increment_log,
        auction_log,
        add_log_text,
        message,
    )
}
