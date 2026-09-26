//! Счётчик ID сообщений организации (исходный `CGame::m_lLeaveWordID`) —
//! primary state владельца `organizations` (прежнее transitional-поле `CGame`).
//!
//! `CGame` хранит только composition handle `leave_words` и делегирует прежний
//! pub facade `allocate_leave_word_id`; загрузка значения из БД и его запись в
//! пакет сохранения остаются оркестрацией init/save через pub-поле (прецедент
//! `characters::worldplayers::WorldPlayerRegistry::player_id`).

/// Действующий счётчик ID сообщений организации.
///
/// Constructor-ное значение — `0`; init-оркестрация подставляет загруженное
/// из БД значение через pub-поле до первого allocate.
pub struct WorldLeaveWordIds {
    pub next: i32,
}

impl WorldLeaveWordIds {
    pub const fn new() -> Self {
        Self { next: 0 }
    }

    /// Pre-increment выдача исходного счётчика: signed wrapping.
    pub fn allocate(&mut self) -> i32 {
        self.next = self.next.wrapping_add(1);
        self.next
    }
}
