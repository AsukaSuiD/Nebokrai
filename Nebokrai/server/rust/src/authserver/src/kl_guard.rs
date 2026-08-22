//! Технический lock-guard исходного AuthServer.
//!
//! Происхождение: `authserver.exe`/`authserver.pdb`, `kl_guard.cpp`. Его RAII-
//! семантика выражена стандартными Rust guard-ами у фактических очередей и owner-ов;
//! отдельная обёртка не нужна.
