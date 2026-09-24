//! FIFO и сигнализация `AccLogQueue`, подтверждённые `loginserver.exe` и
//! `loginserver.pdb`.
//!
//! Сохранена наблюдаемая странность: `push` сначала добавляет запись и
//! игнорирует отказ `ReleaseSemaphore`, поэтому после 10000 накопленных
//! сигналов более новые записи могут остаться в deque без сигнала. `clear`
//! очищает только deque, не счётчик; следующий `pop` способен поглотить старый
//! сигнал, получить пустую очередь и тем самым завершить `AccLogThread`.
//! Typed-запись [`AccountLogRecord`] заменяет промежуточную C-строку, но SQL
//! строится consumer-ом в исходной producer-позиции времени. Явное
//! stop-пробуждение — безопасная замена принудительного `TerminateThread` из
//! `Release`.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Condvar, Mutex};

use crate::access::acclog::AccountLogRecord;

const LEGACY_MAX_SIGNALS: usize = 10_000;

#[derive(Default)]
struct AccLogQueueState {
    records: VecDeque<AccountLogRecord>,
    signals: usize,
}

pub struct AccLogQueue {
    state: Mutex<AccLogQueueState>,
    ready: Condvar,
}

impl AccLogQueue {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(AccLogQueueState::default()),
            ready: Condvar::new(),
        }
    }

    pub fn push(&self, record: AccountLogRecord) {
        let mut state = self.state.lock();
        state.records.push_back(record);
        if state.signals < LEGACY_MAX_SIGNALS {
            state.signals += 1;
            self.ready.notify_one();
        }
    }

    pub fn pop(&self, stop: &AtomicBool) -> Option<AccountLogRecord> {
        let mut state = self.state.lock();
        while state.signals == 0 && !stop.load(Ordering::Acquire) {
            self.ready.wait(&mut state);
        }
        if stop.load(Ordering::Acquire) {
            return None;
        }
        state.signals -= 1;
        state.records.pop_front()
    }

    pub fn clear(&self) {
        self.state.lock().records.clear();
    }

    pub fn wake_all(&self) {
        self.ready.notify_all();
    }
}
