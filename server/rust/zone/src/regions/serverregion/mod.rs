//! Data-типы, позиционное ядро, spatial-запросы, membership-ядро,
//! weather/return-setup/war действия и spawn setup `CServerRegion`
//! исторического GameServer, перенесённые в Zone `regions/` волной
//! serverregion (порции 1-5 и spawn setup порции 1-2). Исходный владелец —
//! `appserver/serverregion.h/.cpp`.
//! Переходный агрегат `CServerRegion` остаётся в старом пакете, хранит те же
//! хранилища и делегирует им area-grid, war-soul, block-refresh, shape-lookup,
//! ids/find/registered запросы, add/remove, позиционную регистрацию,
//! staging/plan/commit смены области, налоговые, погодные, return-point
//! и war-фазовые действия этого компонента, а также batch spawn NPC/monster
//! (setup-контракты, инициализация созданного NPC и batch-циклы
//! `AddNpc`/`AddMonsterRect` с телом `AddMonster`) без изменения сигнатур
//! методов; entry-effects входа, доменные классы и lifecycle вернутся в Zone
//! последующими порциями.
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
//! typed-отказ пространственного членства, gate area-span и ядра входа/выхода
//! с owner-обвязками позиционной регистрации, `transitions` — transition-
//! контракт, staging-очереди и plan/commit смены области, `registry` —
//! identity-регистр фигур и счётчики ID, `queries` — observable порядок
//! старого MSVC hash-обхода NPC-кэша и ids/find/registered запросы над
//! area-grid и registry, `weather` — data-контракты и
//! tick/change-действия погоды, `returnsetup` — return-setup типы и
//! fallback-цепочка точки возврата, `war` — war-фаза и city-ownership
//! действия, `tax` — data-контракты и скалярные state-owner действия налогов,
//! `spawnsetup` — NPC/monster setup data-контракты, spawn outcome/block типы,
//! ядро инициализации создаваемого NPC и batch-циклы `AddNpc`/`AddMonsterRect`
//! с телом `AddMonster` над trait-швами доменных `CNpc`/`CMonster` и
//! store-швами переходных хранилищ.

pub mod areagrid; // area-grid: построение, доступ и war-soul карты.
pub mod blocks; // block-refresh клеток, spatial shape-lookup и skill-cell формула блока.
pub mod geometry; // типы объектов, area/drop/city-state константы и арифметика координат.
pub mod membership; // отказ и gate членства, ядра add/remove и owner-обвязки позиции move-shape.
pub mod queries; // observable traversal старых hash-хранилищ и ids/find/registered запросы regions.
pub mod registry; // identity-регистр фигур и монотонные счётчики ID.
pub mod returnsetup; // return-setup типы и fallback-цепочка точки возврата игрока.
pub mod spawnsetup; // NPC/monster setup типы, ядро инициализации NPC и batch-циклы spawn.
pub mod tax; // data-контракты и скалярные state-owner действия налогов региона.
pub mod transitions; // transition-контракт, staging-очереди и plan/commit смены области.
pub mod war; // war-фазовые и city-ownership действия над скалярами переходного агрегата.
pub mod weather; // data-контракты и tick/change-действия погодных сегментов.
