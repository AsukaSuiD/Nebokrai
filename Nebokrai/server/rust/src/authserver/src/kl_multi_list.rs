//! Потокобезопасная очередь Auth DB из `authserver/src/kl_multi_list.h`.
//!
//! Контракт общей очереди и специализации `ServerInfo` подтверждён точной
//! парой AuthServer EXE/PDB.
//!
//! Доказанный контракт — FIFO под одним lock. Ожидающее извлечение спит,
//! пока очередь пуста; добавление будит одного ожидающего только при переходе
//! из пустого состояния. `size` является отдельным snapshot: исходные DB-
//! потоки сначала читали размер, а затем каждый раз отдельно ожидали элемент.
//! Эта необычная межпоточная семантика намеренно не заменена атомарным drain.
//!
//! `parking_lot::{Mutex, Condvar}` и `VecDeque<T>` заменяют Windows critical
//! section/condition/semaphore и внутренности `std::list`. Mutex не вводит
//! отсутствующее в оригинале poisoning, `VecDeque` сохраняет FIFO, а Rust-
//! владение удаляет ручной allocator/destructor noise.
//! Owned DB shutdown расширяет только пустое ожидание: после атомарной
//! публикации stop тот же mutex и `notify_all` не дают потерять пробуждение.
//! Уже доступный элемент по-прежнему передаётся worker’у до проверки stop.
//!
//! Исходный `pop_front(false)` обращался к `front()` даже для пустого списка;
//! наблюдаемого результата для такого вызова нет. Rust API разделён на
//! ожидающий [`MultiList::pop_front_wait`] и безопасный
//! [`MultiList::try_pop_front`]; неизвестное malformed-поведение не выдаётся
//! за контракт.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Condvar, Mutex};

/// Владеющая FIFO-очередь с исходной семантикой ожидания Auth DB.
pub(crate) struct MultiList<T> {
    elements: Mutex<VecDeque<T>>,
    available: Condvar,
}

impl<T> MultiList<T> {
    /// Создаёт пустую очередь и готовый примитив ожидания.
    pub(crate) fn new() -> Self {
        Self {
            elements: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
        }
    }

    /// Возвращает отдельный 32-битный snapshot размера исходного списка.
    pub(crate) fn size(&self) -> u32 {
        self.elements.lock().len() as u32
    }

    /// Передаёт очереди элемент и добавляет его в хвост.
    pub(crate) fn push_back(&self, element: T) {
        let mut elements = self.elements.lock();
        let was_empty = elements.is_empty();
        elements.push_back(element);
        if was_empty {
            self.available.notify_one();
        }
    }

    /// Ждёт непустую очередь и передаёт самый старый элемент вызывающему.
    pub(crate) fn pop_front_wait(&self) -> T {
        let mut elements = self.elements.lock();
        while elements.is_empty() {
            self.available.wait(&mut elements);
        }
        elements
            .pop_front()
            .expect("очередь проверена под тем же mutex")
    }

    /// Ждёт старейший элемент, но разрешает owned worker shutdown прервать
    /// только пустое ожидание.
    pub(crate) fn pop_front_wait_until_stopped(&self, stopped: &AtomicBool) -> Option<T> {
        let mut elements = self.elements.lock();
        while elements.is_empty() {
            if stopped.load(Ordering::Acquire) {
                return None;
            }
            self.available.wait(&mut elements);
        }
        elements.pop_front()
    }

    /// Будит всех ожидающих после публикации stop-флага.
    ///
    /// Короткий захват того же mutex не позволяет потерять wake между
    /// проверкой stop-флага и атомарным переходом `Condvar::wait` ко сну.
    pub(crate) fn wake_all(&self) {
        let _elements = self.elements.lock();
        self.available.notify_all();
    }

    /// Извлекает самый старый элемент без ожидания либо возвращает `None`.
    pub(crate) fn try_pop_front(&self) -> Option<T> {
        self.elements.lock().pop_front()
    }
}

impl<T> Default for MultiList<T> {
    fn default() -> Self {
        Self::new()
    }
}
