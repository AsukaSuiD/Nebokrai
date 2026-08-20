//! Владельцы исторического LoginServer.

pub(crate) mod applogin;

#[allow(
    clippy::module_inception,
    reason = "двойной loginserver буквально сохраняет исходный PDB-путь server/loginserver/loginserver"
)]
pub(crate) mod loginserver;
