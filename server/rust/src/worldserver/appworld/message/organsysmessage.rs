//! Организационные сообщения `OnOrgasysMessage` из `organsysmessage.cpp`,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Ветки диспетчера переведены в `nebokrai_realm::app::organsysmessage`
//! (generic-обработчики поверх статического организационного view,
//! `WorldGameView` и bridge-швов `Organizing*Bridge`). Волной C5-C с переносом
//! `CGame` туда же уехала вся CGame-обвязка: адаптеры
//! `WorldUnionApplicationEffectCallbacks`/`WorldUnionApplicationEffects` и
//! `World*Effects`/`Bridge` типы, одноимённые `dispatch_*` направляющие функции
//! и семейный `reload_attack_city` — по orphan-правилу impl-ы швов для
//! `CGame` допустимы только в crate типа. Их новое место —
//! `nebokrai_realm::app::world_dispatch` поверх той же generic-связки.
//!
//! Здесь остаётся реэкспорт realm-обработчиков и alias session-runtime
//! владельца для process-owner `runtime.rs`.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub use nebokrai_realm::app::organsysmessage::*;

/// Session-runtime владелец слит с realm [`WorldOrganizingSessionRuntimeOwner`]:
/// прежнее имя сохраняет импорты main-loop runtime; очереди едины — это тот
/// же самый `Arc`-state, type-алиас расхождения не создаёт.
#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::app::organsysmessage::WorldOrganizingSessionRuntimeOwner as WorldUnionApplicationRuntimeOwner;
