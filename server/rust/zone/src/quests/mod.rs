//! Прогресс заданий живого игрока Zone.

mod availability; // флаг доступности и время задания игрока.
mod client; // клиентская запись задания.
mod progress; // карта прогресса заданий игрока.

pub use availability::PlayerQuestAvailability;
pub use client::append_client_quest_record;
pub use progress::PlayerQuestProgress;
