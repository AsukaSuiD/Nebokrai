//! Общие механизмы Realm и Zone без зависимости от серверных владельцев.
//! Существующий пакет nebokrai-server использует их на время разделения ролей.

pub mod network;
pub mod protocol;
pub mod resources;
pub mod runtime;
pub mod scripting;
pub mod values;
