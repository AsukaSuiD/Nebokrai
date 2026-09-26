//! Потокобезопасная очередь сокетных команд (`nets/socketcommands.cpp`,
//! пары EXE/PDB шести служб в `server/rust/src/manifest/`). Контракт всех
//! полных вариантов совпадает: push в оба конца, извлечение из головы, размер
//! под тем же lock, очистка, атомарная передача всех элементов и вставка
//! перед ожидающими с сохранением порядка обеих очередей. Auth/Billing не
//! emitted неиспользованных `Pop_Front`/`Push_Front`/
//! `AddCommandsQueueToFront`; остальные методы совпадают с полными вариантами
//! того же source-файла.
//!
//! `parking_lot::Mutex<VecDeque<T>>` заменяет `CRITICAL_SECTION` и старый
//! deque: lock без добавленного poisoning, порядок сохранён, передача `T` —
//! владение. Generic `T` оставляет layout команды владельцам `clients.rs`/
//! `servers.rs`; владеющий конкретный тип заменяет ручные ветви `Clear` и
//! `operator_delete` обычным `Drop`. Старый nullable `Push_* -> false` не
//! переносится: сохранённый `T` всегда существует, а возвращаемый `bool` ни
//! один call site не использовал.
//!
//! Экземпляр очереди хранит владелец процесса/направления; Shared несёт типы
//! записей и атомарность операций.
//! Доказательства: docs/reconstruction/shared-technical.md#очередь-socket-команд-csocketcommands

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

/// Владеющая двусторонняя очередь команд с атомарностью исходного
/// `CSocketCommands`.
///
/// `Drop` элемента может выполняться под mutex при [`Self::clear`]. Конкретный
/// тип команды не должен повторно входить в тот же экземпляр очереди из своего
/// деструктора; исходный `tagSocketOper` такого поведения не имел.
pub struct CSocketCommands<T> {
    commands: Mutex<VecDeque<T>>,
}

impl<T> CSocketCommands<T> {
    /// Создаёт пустую очередь с готовым примитивом синхронизации.
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(VecDeque::new()),
        }
    }

    /// Возвращает размер как 32-битный Windows `long`, сохраняя его младшие
    /// биты при теоретическом переполнении.
    pub fn get_size(&self) -> i32 {
        self.commands.lock().len() as u32 as i32
    }

    /// Передаёт владение командой и ставит её перед всеми ожидающими.
    pub fn push_front(&self, command: T) {
        self.commands.lock().push_front(command);
    }

    /// Передаёт владение командой и ставит её после всех ожидающих.
    pub fn push_back(&self, command: T) {
        self.commands.lock().push_back(command);
    }

    /// Извлекает первую команду либо возвращает `None` для пустой очереди.
    pub fn pop_front(&self) -> Option<T> {
        self.commands.lock().pop_front()
    }

    /// Уничтожает все ожидающие команды под тем же mutex.
    pub fn clear(&self) {
        self.commands.lock().clear();
    }

    /// Атомарно опустошает очередь и передаёт все команды в исходном порядке.
    pub fn take_all(&self) -> VecDeque<T> {
        mem::take(&mut *self.commands.lock())
    }

    /// Ставит всю переданную очередь перед уже ожидающими командами.
    ///
    /// Порядок внутри `front` и внутри текущей очереди сохраняется.
    pub fn prepend(&self, mut front: VecDeque<T>) {
        let mut commands = self.commands.lock();
        front.append(&mut commands);
        *commands = front;
    }
}

impl<T> Default for CSocketCommands<T> {
    fn default() -> Self {
        Self::new()
    }
}
