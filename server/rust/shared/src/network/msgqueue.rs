//! Потокобезопасная очередь сообщений (`nets/msgqueue.cpp`, пары EXE/PDB
//! шести служб в `server/rust/src/manifest/`). Живой контракт совпадает во
//! всех вариантах: добавление в хвост, извлечение из головы, размер под тем
//! же lock, уничтожение оставшихся владельцем очереди и атомарная передача
//! всех элементов вызывающей стороне.
//!
//! `parking_lot::Mutex<VecDeque<T>>` заменяет `CRITICAL_SECTION` и старый
//! deque; владение `T` заменяет виртуальное ручное удаление. Конкретная форма
//! полиморфного сообщения намеренно не выбирается: её определит владелец
//! `CBaseMessage/CMessage`. Старый `PushMessage(nullptr) -> false` не
//! переносится: nullable-результат разбирает вызывающий владелец, сохранённый
//! `T` всегда является реальным объектом, а возвращаемый `bool` ни один
//! call site не использовал.
//!
//! Экземпляр очереди хранит владелец процесса/направления; Shared несёт типы
//! записей и атомарность операций.
//! Доказательства: docs/reconstruction/shared-technical.md#очередь-сообщений-cmsgqueue

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

/// Владеющая FIFO-очередь, сохраняющая атомарность операций старого
/// `CMsgQueue`.
///
/// Элемент уничтожается под mutex при [`Self::clear`], как и под исходной
/// critical section. Поэтому `Drop` будущего конкретного типа сообщения не
/// должен повторно входить в тот же экземпляр очереди; проверенные деструкторы
/// `CBaseMessage` и `CMessage` такого вызова не делают.
pub struct CMsgQueue<T> {
    messages: Mutex<VecDeque<T>>,
}

impl<T> CMsgQueue<T> {
    /// Создаёт пустую очередь с готовым примитивом синхронизации.
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::new()),
        }
    }

    /// Возвращает число сообщений как 32-битный Windows `long` исходного
    /// `GetSize`, сохраняя его signedness и младшие биты.
    pub fn get_size(&self) -> i32 {
        self.messages.lock().len() as u32 as i32
    }

    /// Передаёт очереди владение сообщением и помещает его в хвост.
    pub fn push(&self, message: T) {
        self.messages.lock().push_back(message);
    }

    /// Извлекает самое старое сообщение или возвращает `None` для пустой
    /// очереди.
    pub fn pop(&self) -> Option<T> {
        self.messages.lock().pop_front()
    }

    /// Уничтожает все оставшиеся сообщения, удерживая mutex на протяжении
    /// очистки, как исходный `CMsgQueue::Clear`.
    pub fn clear(&self) {
        self.messages.lock().clear();
    }

    /// Атомарно опустошает очередь и передаёт вызывающему владение всеми
    /// сообщениями с сохранением FIFO-порядка.
    pub fn take_all(&self) -> VecDeque<T> {
        mem::take(&mut *self.messages.lock())
    }

    /// Добавляет последовательность в хвост одной атомарной операцией. Это
    /// сохраняет относительный порядок повторно отложенных эффектов и не даёт
    /// другим producer-ам вклиниться внутрь возвращаемой группы.
    pub fn extend(&self, messages: impl IntoIterator<Item = T>) {
        self.messages.lock().extend(messages);
    }
}

impl<T> Default for CMsgQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
