//! Процессная Win32/MFC оболочка исходного LoginServer.
//!
//! Происхождение: `loginserver.exe`/`LoginServer.pdb`, `loginserver.cpp`.
//! Доменный init/main-loop/reconnect/release восстановлен в `game`; оконный
//! message pump и CRT startup не требуют Linux-аналога. Отдельная process-точка
//! запуска LoginServer в текущем репозитории отсутствует.
