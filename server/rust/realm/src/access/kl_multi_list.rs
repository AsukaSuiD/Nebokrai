//! Очередь `kl_multi_list.h` AuthServer. `parking_lot::{Mutex,
//! Condvar}` и `VecDeque` заменяют Win32-синхронизацию и `std::list` без
//! изменения FIFO.
//!
//! Размер читается отдельным snapshot, после чего worker ожидает каждый элемент;
//! это намеренно не заменено атомарным drain. Stop может прервать только пустое
//! ожидание, а уже доступный элемент передаётся worker-у. Недоопределённый вызов
//! исходного `pop_front(false)` на пустом списке заменён безопасным `None`.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Condvar, Mutex};

pub struct MultiList<T> {
    elements: Mutex<VecDeque<T>>,
    available: Condvar,
}

impl<T> MultiList<T> {
    pub fn new() -> Self {
        Self {
            elements: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
        }
    }

    pub fn size(&self) -> u32 {
        self.elements.lock().len() as u32
    }

    pub fn push_back(&self, element: T) {
        let mut elements = self.elements.lock();
        let was_empty = elements.is_empty();
        elements.push_back(element);
        if was_empty {
            self.available.notify_one();
        }
    }

    pub fn pop_front_wait(&self) -> T {
        let mut elements = self.elements.lock();
        while elements.is_empty() {
            self.available.wait(&mut elements);
        }
        elements
            .pop_front()
            .expect("очередь проверена под тем же mutex")
    }

    pub fn pop_front_wait_until_stopped(&self, stopped: &AtomicBool) -> Option<T> {
        let mut elements = self.elements.lock();
        while elements.is_empty() {
            if stopped.load(Ordering::Acquire) {
                return None;
            }
            self.available.wait(&mut elements);
        }
        elements.pop_front()
    }

    pub fn wake_all(&self) {
        // Тот же mutex закрывает окно между проверкой stop и `Condvar::wait`.
        let _elements = self.elements.lock();
        self.available.notify_all();
    }

    pub fn try_pop_front(&self) -> Option<T> {
        self.elements.lock().pop_front()
    }
}

impl<T> Default for MultiList<T> {
    fn default() -> Self {
        Self::new()
    }
}
