//! FIFO команд World write-log, подтверждённая `Nworldserver.exe` и
//! `WorldServer.pdb`.
//!
//! Push добавляет в хвост, worker снимает с головы. `Arc<Mutex<VecDeque<_>>>`
//! заменяет critical section/STL, сохраняя порядок, передачу владения и
//! отсутствие дополнительного duplicate/retry состояния.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::persistence::writelog::WorldWriteLogCommand;

#[derive(Clone, Default)]
pub struct WorldWriteLogQueue {
    commands: Arc<Mutex<VecDeque<WorldWriteLogCommand>>>,
}

impl WorldWriteLogQueue {
    pub fn push(&self, command: WorldWriteLogCommand) -> usize {
        let mut commands = self.lock();
        commands.push_back(command);
        commands.len()
    }

    pub fn pop(&self) -> Option<WorldWriteLogCommand> {
        self.lock().pop_front()
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<WorldWriteLogCommand>> {
        self.commands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
