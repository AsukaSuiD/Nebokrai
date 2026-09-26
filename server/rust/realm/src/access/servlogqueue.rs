//! FIFO `ServLogQueue` LoginServer.
//!
//! `push`, `pop`, `size` и `clear` работали с одним `std::deque` под общей
//! critical section. `Mutex<VecDeque>` сохраняет порядок и единицу блокировки;
//! `Vec<u8>` — глубокую копию C-строки без терминатора. IPv4 хранится как `u32`,
//! сохраняя 32-битный bit pattern исходного `long`.

use std::collections::VecDeque;

use parking_lot::Mutex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServLog {
    pub source_ip: u32,
    pub server_type: i32,
    pub server_number: i32,
    pub description: Vec<u8>,
}

impl ServLog {
    pub fn new(source_ip: u32, server_type: i32, server_number: i32, description: Vec<u8>) -> Self {
        Self {
            source_ip,
            server_type,
            server_number,
            description,
        }
    }
}

pub struct ServLogQueue {
    records: Mutex<VecDeque<ServLog>>,
}

impl ServLogQueue {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, record: ServLog) {
        self.records.lock().push_back(record);
    }

    pub fn pop(&self) -> Option<ServLog> {
        self.records.lock().pop_front()
    }

    pub fn size(&self) -> i32 {
        self.records.lock().len() as i32
    }

    pub fn clear(&self) {
        self.records.lock().clear();
    }
}
