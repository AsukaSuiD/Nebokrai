//! Процессная оболочка исходного AuthServer.
//!
//! Происхождение: `authserver.exe`/`authserver.pdb`, `app.cpp`. MFC window и message
//! pump заменены Linux entrypoint в `main.rs`; доменный lifecycle принадлежит
//! `cgame::game_thread_func`. В этом модуле отдельного runtime-состояния нет.
