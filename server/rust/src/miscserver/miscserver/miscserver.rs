//! Процессная Win32/MFC оболочка исходного MiscServer.
//!
//! Происхождение: `miscserver.exe`/`miscserver.pdb`, `miscserver.cpp`. Доменный
//! connect/main-loop/reconnect/release находится в соседнем `game` owner-е;
//! window и message pump не определяют серверный контракт. Отдельная process-
//! точка запуска MiscServer в текущем репозитории отсутствует.
