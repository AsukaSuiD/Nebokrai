//! Потокобезопасный FIFO-владелец `ServLogQueue` LoginServer.
//!
//! восстановлено. Точная пара:
//! `push` копировал четыре поля в отдельный heap-object и добавлял указатель в
//! конец `std::deque` под `CRITICAL_SECTION`; `pop` снимал начало, `size`
//! читал длину, а `clear` удалял все записи под тем же lock. `Mutex<VecDeque>`
//! сохраняет FIFO, единицу синхронизации и owned lifetime без `new/delete`,
//! raw pointers и Windows API. `Vec<u8>` заменяет глубокую копию NUL-
//! terminated `char*`; вызывающий handler уже передаёт bytes до терминатора.
//! Raw `long` IPv4 хранится как `u32`, сохраняя тот же 32-битный bit pattern.
//! STL allocator/deque internals и compiler cleanup удалены как заменённый
//! library noise; локальных неизвестностей у достигнутого контракта нет.

use std::collections::VecDeque;

use parking_lot::Mutex;

/// Одна owned-запись исходного `ServLogQueue::ServLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServLog {
    pub(crate) source_ip: u32,
    pub(crate) server_type: i32,
    pub(crate) server_number: i32,
    pub(crate) description: Vec<u8>,
}

impl ServLog {
    /// Копирует четыре исходных поля без изменения numeric bit pattern.
    pub(crate) fn new(
        source_ip: u32,
        server_type: i32,
        server_number: i32,
        description: Vec<u8>,
    ) -> Self {
        Self {
            source_ip,
            server_type,
            server_number,
            description,
        }
    }
}

/// Owned-замена `CRITICAL_SECTION + deque<ServLog*>`.
pub(crate) struct ServLogQueue {
    records: Mutex<VecDeque<ServLog>>,
}

impl ServLogQueue {
    /// Создаёт исходно пустую очередь.
    pub(crate) fn new() -> Self {
        Self {
            records: Mutex::new(VecDeque::new()),
        }
    }

    /// Добавляет запись в FIFO с глубоко скопированным описанием.
    pub(crate) fn push(&self, record: ServLog) {
        self.records.lock().push_back(record);
    }

    /// Снимает старейшую запись либо возвращает отсутствие.
    pub(crate) fn pop(&self) -> Option<ServLog> {
        self.records.lock().pop_front()
    }

    /// Возвращает размер как исходный signed Windows `long` bit pattern.
    pub(crate) fn size(&self) -> i32 {
        self.records.lock().len() as i32
    }

    /// Удаляет все записи под одной исходной critical section.
    pub(crate) fn clear(&self) {
        self.records.lock().clear();
    }
}
