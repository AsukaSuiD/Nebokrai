//! Центральный `COrganizingCtrl` из `organizingctrl.cpp/.h` перенесён в Realm
//! `organizations/organizingctrl.rs` целиком (структура, endpoints, bridges и
//! вся impl-семья). Здесь его реэкспорт для переходных потребителей старого
//! пакета. Session endpoint-блоки переиздаются из исходного Realm владельца
//! `organizations/union.rs`; billboard/reinitialization контракты и terminal
//! старый пакет через этот путь больше не потребляет и они не переиздаются.

pub(crate) use nebokrai_realm::organizations::organizingctrl::*;
pub(crate) use nebokrai_realm::organizations::union::{
    CityTransferEndpointBlock, ConfederationCreationEndpointBlock,
};
