//! Оконный setup-callback исходного MiscServer.
//!
//! Происхождение: `miscserver.exe`/`miscserver.pdb`, `onserversetup.cpp`.
//! Настройка сервиса читается владельцем `setup`; Win32 dialog callback не
//! участвует в runtime-контракте Linux-сервера.
