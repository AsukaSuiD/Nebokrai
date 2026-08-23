//! FIFO команд World write-log, подтверждённая `worldserver.exe` и
//! `worldserver.pdb`.
//!
//! Push добавляет в хвост, worker снимает с головы. `Arc<Mutex<VecDeque<_>>>`
//! заменяет critical section/STL, сохраняя порядок, передачу владения и
//! отсутствие дополнительного duplicate/retry состояния.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::worldserver::appworld::message::writelogmessage::WorldWriteLogCommand;

#[derive(Clone, Default)]
pub(crate) struct WorldWriteLogQueue {
    commands: Arc<Mutex<VecDeque<WorldWriteLogCommand>>>,
}

impl WorldWriteLogQueue {
    pub(crate) fn push(&self, command: WorldWriteLogCommand) -> usize {
        let mut commands = self.lock();
        commands.push_back(command);
        commands.len()
    }

    pub(crate) fn pop(&self) -> Option<WorldWriteLogCommand> {
        self.lock().pop_front()
    }

    pub(crate) fn len(&self) -> usize {
        self.lock().len()
    }

    pub(crate) fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<WorldWriteLogCommand>> {
        self.commands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
