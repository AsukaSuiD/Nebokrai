//! Потокобезопасная очередь сообщений, восстановленная из `nets/msgqueue.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`.
//!
//! Точные варианты и адреса исходных методов:
//! - `AuthServer/authserver.exe + AuthServer/authserver.pdb`: `GetSize` RVA
//!   `0x0000D5E0`, `PopMessage` `0x0000D600`, конструктор `0x0000D680`,
//!   `Clear` `0x0000D6B0`, деструктор `0x0000D720`, `PushMessage` `0x0000D9F0`;
//! - `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`:
//!   соответственно `0x0000C840`, `0x0000C8D0`, `0x0000C950`, `0x0000C980`,
//!   `0x0000C9F0`, `0x0000CCC0`;
//! - `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`:
//!   соответственно `0x0006AA70`, `0x0006AA90`, `0x0006AB20`, `0x0006AB50`,
//!   `0x0006ABC0`, `0x0006AE90`;
//! - `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`: конструктор RVA
//!   `0x00013230`, `Clear` `0x00013260`, деструктор `0x000132D0`,
//!   `PushMessage` `0x000139D0`, `GetAllMessage` `0x00013AD0`;
//! - `GameServer/gameserver.exe + GameServer/GameServer.pdb`: соответственно
//!   `0x00011F30`, `0x00011F60`, `0x00011FD0`, `0x000126D0`, `0x000127D0`;
//! - `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`: `GetSize` RVA
//!   `0x00028310`, `PopMessage` `0x00028330`, конструктор `0x000283B0`,
//!   `Clear` `0x000283E0`, деструктор `0x00028450`, `PushMessage` `0x00028710`.
//!
//! Исходные владельцы PDB: `h:\fengyun\fy_russia\src\nets\msgqueue.cpp`,
//! `d:\complite_version\fengyun_russia\trunk\nets\msgqueue.cpp` и
//! `e:\svn\fengyun_russia_dev\nets\msgqueue.cpp`.
//!
//! Во всех вариантах живой контракт совпадает: добавление в хвост, извлечение
//! из головы, размер под тем же lock, уничтожение оставшихся владельцем очереди
//! и атомарная передача всех элементов вызывающей стороне. Различающиеся имена
//! специализаций `std::deque` в декомпиляции являются ошибками восстановления
//! типов: прототипы и вызовы последовательно используют `CBaseMessage*`.
//!
//! `parking_lot::Mutex<VecDeque<T>>` заменяет `CRITICAL_SECTION` и внутренности
//! старого `std::deque`. Mutex не вводит отсутствующее в оригинале poisoning,
//! `VecDeque` сохраняет FIFO, а владение `T` заменяет виртуальное ручное
//! удаление. Конкретная форма полиморфного сообщения здесь намеренно не
//! выбирается: её определит владелец `CBaseMessage/CMessage`.
//!
//! Старый `PushMessage(nullptr) -> false` не переносится внутрь очереди:
//! nullable-результат создания сообщения должен быть разобран будущим
//! вызывающим владельцем, тогда как сохранённый `T` всегда является реальным
//! объектом. Ни один найденный call site не использует возвращаемый `bool`.
//!
//! Остальные экспортированные из этого source-файла тела классифицированы как
//! не принадлежащие очереди: закрытие сокета остаётся у деструктора
//! `CMySocket` в `nets/mysocket.rs`, удаление `CBaseMessage` — у его владельца
//! в `nets/basemessage.rs`, а `std::vector`, allocator и `$E/$L` cleanup-блоки
//! полностью заменяются коллекциями, владением и `Drop` Rust. Отдельных Rust-тел
//! для них здесь нет.

//! Экземпляр очереди хранит владелец процесса/направления;
//! Shared несёт только типы записей и атомарность операций.

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
