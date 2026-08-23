//! WinMain-оболочка `authserver.cpp`, подтверждённая `authserver.exe` и
//! `authserver.pdb`. Сигналы, Tokio runtime и владение `CGame` находятся в
//! process entrypoint и `cgame::game_thread_func`; Win32/MFC-слой не переносится.
