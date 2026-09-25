//! Data-типы и позиционное ядро `CServerRegion` исторического GameServer,
//! перенесённые в Zone `regions/` первой порцией волны serverregion.
//! Исходный владелец — `appserver/serverregion.h/.cpp`. Переходный агрегат
//! `CServerRegion` остаётся в старом пакете, хранит те же хранилища и
//! делегирует им area-grid и war-soul операции этого компонента без
//! изменения сигнатур методов; доменные классы и lifecycle вернутся в Zone
//! последующими порциями.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2).
//!
//! Разбиение по обязанности: `geometry` — типы объектов, area/drop/city-state
//! константы и чистая арифметика координат, `areagrid` — построение и доступ
//! к area-grid вместе с war-soul картами, `membership` — typed-отказ
//! пространственного членства и gate area-span, `transitions` — immutable
//! plan смены области, `registry` — identity-регистр фигур и счётчики ID,
//! `queries` — observable порядок старого MSVC hash-обхода NPC-кэша,
//! `weather` и `tax` — data-контракты.

pub mod areagrid;
pub mod geometry;
pub mod membership;
pub mod queries;
pub mod registry;
pub mod tax;
pub mod transitions;
pub mod weather;
