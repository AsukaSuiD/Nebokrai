//! Lock-обёртка исходного LoginServer.
//!
//! Происхождение: `loginserver.exe`/`loginserver.pdb`, `locker.cpp`. Её critical
//! section и RAII unlock выражены `parking_lot::Mutex` у фактических очередей;
//! отдельный тип не нужен.
