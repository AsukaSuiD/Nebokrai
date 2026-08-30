//! Базовый container-listener GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/ccontainerlistener.cpp`. В owner-файле нет собственной
//! игровой семантики: это только polymorphic интерфейс обхода контейнера.
//! Rust заменяет vtable конкретными типизированными visitor-ами соседних
//! модулей; неизвестностей наблюдаемого контракта не осталось.
