//! FIFO и сигнализация исторического `AccLogQueue` LoginServer.
//!
//! Контракт подтверждён точной парой LoginServer EXE/PDB.
//! Сохранена наблюдаемая странность: `push` сначала добавляет запись и
//! игнорирует отказ `ReleaseSemaphore`, поэтому после 10000 накопленных
//! сигналов более новые записи могут остаться в deque без сигнала. `clear`
//! очищает только deque, не счётчик; следующий `pop` способен поглотить старый
//! сигнал, получить пустую очередь и тем самым завершить `AccLogThread`.
//! Typed-запись заменяет промежуточную C-строку, но SQL строится владельцем
//! consumer в исходной producer-позиции времени. Явное stop-пробуждение —
//! безопасная Linux/Rust-замена принудительного `TerminateThread` из `Release`.
//! Локальных неизвестностей нет.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Condvar, Mutex};

use super::game::AccountLogRecord;

const LEGACY_MAX_SIGNALS: usize = 10_000;

#[derive(Default)]
struct AccLogQueueState {
    records: VecDeque<AccountLogRecord>,
    signals: usize,
}

/// Общая FIFO account-журналов с точной семантикой старого semaphore.
pub(crate) struct AccLogQueue {
    state: Mutex<AccLogQueueState>,
    ready: Condvar,
}

impl AccLogQueue {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(AccLogQueueState::default()),
            ready: Condvar::new(),
        }
    }

    /// Добавляет запись даже при исчерпанном исходном лимите сигналов.
    pub(crate) fn push(&self, record: AccountLogRecord) {
        let mut state = self.state.lock();
        state.records.push_back(record);
        if state.signals < LEGACY_MAX_SIGNALS {
            state.signals += 1;
            self.ready.notify_one();
        }
    }

    /// Ждёт один сигнал и извлекает старейшую запись, если она ещё существует.
    pub(crate) fn pop(&self, stop: &AtomicBool) -> Option<AccountLogRecord> {
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

    /// Очищает deque, намеренно сохраняя уже накопленные semaphore-сигналы.
    pub(crate) fn clear(&self) {
        self.state.lock().records.clear();
    }

    /// Будит ожидающий worker после установки безопасного stop-флага.
    pub(crate) fn wake_all(&self) {
        self.ready.notify_all();
    }
}
