//! восстановлено — наблюдаемый контракт `CGasOperator::GetIP` из
//! `loginserver/applogin/gasoperator.cpp` и `.h`.
//!
//! Источник: точная пара LoginServer.exe/PDB
//! (`1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` /
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`).
//! Singleton, deleting-destructor thunk, выделение памяти и compiler cleanup
//! не имели отдельного наблюдаемого контракта и заменены чистой функцией.
//! Локальных неизвестностей нет.

/// Форматирует исторический DWORD IPv4 в том же порядке байтов, что `GetIP`.
pub(crate) fn format_ipv4(raw: u32) -> Vec<u8> {
    let [first, second, third, fourth] = raw.to_le_bytes();
    format!("{first}.{second}.{third}.{fourth}").into_bytes()
}
