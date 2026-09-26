//! Базовый container-listener GameServer; памятка о владельце перенесена в
//! Zone `items/` вместе с типизированными visitor-ами соседних файлов
//! (волна Z-C2c).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/ccontainerlistener.cpp`. В owner-файле нет собственной
//! игровой семантики: это только polymorphic интерфейс обхода контейнера.
//! Rust заменяет vtable конкретными типизированными visitor-ами соседних
//! модулей; неизвестностей наблюдаемого контракта не осталось.
