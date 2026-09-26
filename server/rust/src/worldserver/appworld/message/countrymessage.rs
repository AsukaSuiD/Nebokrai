//! Country-сообщения `OnCountryMessage` из `countrymessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Обработчики ветвей, data-контракты и исходы перенесены в
//! `nebokrai_realm::app::countrymessage` (generic-обработчики поверх gate/view
//! швов). Волной C5-C с переносом `CGame` сюда же, в
//! `nebokrai_realm::app::world_dispatch`, переехала CGame-обвязка этого
//! файла: адаптеры `WorldCountryMutGate`/`WorldCountryGovernanceGate`/
//! `CountryHandlerExistsView`, impl [`WorldCountryPlayerChangeView`] для
//! `CGame` (orphan-правило: impl шва для типа допустим только в crate типа)
//! и одноимённые `dispatch_*` направляющие функции.
//!
//! Здесь остаётся glob-реэкспорт realm-владельца для переходных потребителей
//! старого пакета.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::countrymessage::*;
