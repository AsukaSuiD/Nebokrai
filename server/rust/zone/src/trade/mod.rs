//! Правила локальных сделок и резервов Zone.

pub mod audit; // audit-кадры 0x60201/0x60202/0x6020D, собранные вне transport-шва.
pub mod auction; // player-side состояние и правила аукциона живого игрока.
pub mod ctrader; // агрегат и правила рамки двустороннего обмена CTrader.
pub mod currency; // денежное ядро обмена и маршруты bank/ground валюты.
pub mod ground; // скалярные правила наземного перемещения предметов.
pub mod session; // скалярное ядро оркестрации CGame: условия, Billing-вилка, уведомления.
