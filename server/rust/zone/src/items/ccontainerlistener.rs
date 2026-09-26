//! Памятка о базовом container-listener GameServer: polymorphic интерфейс
//! обхода контейнера без собственных типов и игровой семантики.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/ccontainerlistener.cpp`. Rust заменяет vtable
//! конкретными типизированными visitor-ами соседних модулей; неизвестностей
//! наблюдаемого контракта не осталось.
