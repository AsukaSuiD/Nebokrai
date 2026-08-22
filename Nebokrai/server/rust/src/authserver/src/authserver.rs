//! WinMain-оболочка и process-global состояние исходного AuthServer.
//!
//! Происхождение: `authserver.exe`/`authserver.pdb`, `authserver.cpp`. Сигналы
//! процесса, Tokio runtime и владение `CGame` реализованы в `main.rs` и
//! `cgame::game_thread_func`; Win32/MFC plumbing отдельного аналога не требует.
