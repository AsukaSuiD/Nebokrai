//! Рассылка значимых изменений игровым клиентам: выбор получателей и постановка
//! кадра после эффекта по [карте владельцев]. Переносимая часть опирается только
//! на identity и формат получателей; выбор момента callback — у владельца эффекта.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod around; // around-runtime view: кадр рассылки окружения.
pub mod recipients; // spatial/recipient snapshot для around-family рассылок региона.
