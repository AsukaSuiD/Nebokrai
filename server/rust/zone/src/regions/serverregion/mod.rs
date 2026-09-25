//! Data-типы, позиционное ядро и spatial-запросы `CServerRegion` исторического
//! GameServer, перенесённые в Zone `regions/` волной serverregion (порции
//! 1-2). Исходный владелец — `appserver/serverregion.h/.cpp`. Переходный
//! агрегат `CServerRegion` остаётся в старом пакете, хранит те же хранилища
//! и делегирует им area-grid, war-soul, block-refresh и shape-lookup операции
//! этого компонента без изменения сигнатур методов; доменные классы и
//! lifecycle вернутся в Zone последующими порциями.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2).
//!
//! Разбиение по обязанности: `geometry` — типы объектов, area/drop/city-state
//! константы и чистая арифметика координат, `areagrid` — построение и доступ
//! к area-grid вместе с war-soul картами, `blocks` — динамическая
//! block-разметка клеток и shape-lookup девяти-area окружения, `membership` —
//! typed-отказ пространственного членства и gate area-span, `transitions` —
//! immutable plan смены области, `registry` — identity-регистр фигур и
//! счётчики ID, `queries` — observable порядок старого MSVC hash-обхода
//! NPC-кэша, `weather` и `tax` — data-контракты.

pub mod areagrid; // area-grid: построение, доступ и war-soul карты.
pub mod blocks; // block-refresh клеток и spatial shape-lookup девяти-area окружения.
pub mod geometry; // типы объектов, area/drop/city-state константы и арифметика координат.
pub mod membership; // typed-отказ пространственного членства и gate area-span.
pub mod queries; // observable traversal-контракты запросов старых hash-хранилищ.
pub mod registry; // identity-регистр фигур и монотонные счётчики ID.
pub mod tax; // data-контракты налогового начисления и сбора.
pub mod transitions; // immutable transition-контракт смены области.
pub mod weather; // data-контракты погодных сегментов и tick-результата.
