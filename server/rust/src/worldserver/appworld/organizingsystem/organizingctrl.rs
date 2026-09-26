//! Центральный `COrganizingCtrl` из `organizingctrl.cpp/.h` перенесён в Realm
//! `organizations/organizingctrl.rs` целиком (структура, endpoints, bridges и
//! вся impl-семья). Здесь его реэкспорт для переходных потребителей старого
//! пакета. Session endpoint-блоки union-владельца с переездом terminal/AI
//! семейства сообщений и hub-данных (волна C5-A) старый пакет через этот
//! путь больше не потребляет и они не переиздаются.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::organizations::organizingctrl::*;
