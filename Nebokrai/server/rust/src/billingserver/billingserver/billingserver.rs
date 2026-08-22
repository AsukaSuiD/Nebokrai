//! Процессная Win32/MFC оболочка исходного BillingServer.
//!
//! Происхождение: `billingserver.exe`/`billingserver.pdb`, `billingserver.cpp`.
//! Доменный init/main-loop/release находится в соседнем `game` owner-е; window,
//! message pump и CRT startup не являются частью серверного протокола. Отдельная
//! Linux-точка запуска в текущем репозитории отсутствует.
