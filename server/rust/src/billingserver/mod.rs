//! Владельцы исторического BillingServer.

pub(crate) mod appbilling;

#[allow(
    clippy::module_inception,
    reason = "двойной billingserver буквально сохраняет исходный PDB-путь server/billingserver/billingserver"
)]
pub(crate) mod billingserver;
