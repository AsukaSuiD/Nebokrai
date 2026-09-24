//! `CGasOperator::GetIP` перенесён в Realm access.
//! Здесь его реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::access::gasoperator::format_ipv4;
