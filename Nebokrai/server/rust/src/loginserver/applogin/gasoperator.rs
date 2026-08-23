//! Наблюдаемый контракт `CGasOperator::GetIP` из
//! `loginserver/applogin/gasoperator.cpp` и `.h`.
//!
//! Контракт подтверждён точной парой LoginServer EXE/PDB.
//! Singleton, deleting-destructor thunk, выделение памяти и compiler cleanup
//! не имели отдельного наблюдаемого контракта и заменены чистой функцией.
//! Локальных неизвестностей нет.

/// Форматирует исторический DWORD IPv4 в том же порядке байтов, что `GetIP`.
pub(crate) fn format_ipv4(raw: u32) -> Vec<u8> {
    let [first, second, third, fourth] = raw.to_le_bytes();
    format!("{first}.{second}.{third}.{fourth}").into_bytes()
}
