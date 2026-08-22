//! Lock-обёртка исходного LoginServer.
//!
//! Происхождение: `loginserver.exe`/`LoginServer.pdb`, `locker.cpp`. Ее critical
//! section и RAII unlock выражены `parking_lot::Mutex` у фактических очередей;
//! отдельный тип не нужен.
