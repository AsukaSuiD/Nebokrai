//! Прогресс заданий живого игрока Zone.

mod availability; // флаг доступности и время задания игрока.
mod client; // клиентская запись задания.
mod progress; // карта прогресса заданий игрока.

pub use availability::PlayerQuestAvailability; // доступность задания для выдачи.
pub use client::append_client_quest_record; // кодирование клиентской записи задания.
pub use progress::PlayerQuestProgress; // карта прогресса игрока.
