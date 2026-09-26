//! Data-ядра `CServerRegion` (исходный владелец — `appserver/serverregion.h/.cpp`).
//! Переходный агрегат старого пакета делегирует операции файлам этого
//! компонента без изменения сигнатур методов; entry-effects входа, доменные
//! классы и lifecycle остаются у него.

pub mod areagrid; // area-grid: построение, доступ и war-soul карты.
pub mod blocks; // block-refresh клеток, spatial shape-lookup и skill-cell формула блока.
pub mod geometry; // типы объектов, area/drop/city-state константы и арифметика координат.
pub mod membership; // отказ и gate членства, ядра add/remove и owner-обвязки позиции move-shape.
pub mod queries; // observable traversal старых hash-хранилищ и ids/find/registered запросы regions.
pub mod registry; // identity-регистр фигур и монотонные счётчики ID.
pub mod returnsetup; // return-setup типы и fallback-цепочка точки возврата игрока.
pub mod spawnsetup; // NPC/monster setup типы, ядро инициализации NPC и batch-циклы spawn.
pub mod startup; // startup decode-ядра snapshot/setup/forbid-good/resource и store-шов ServerRegionDecodeStore.
pub mod tax; // data-контракты и скалярные state-owner действия налогов региона.
pub mod transitions; // transition-контракт, staging-очереди и plan/commit смены области.
pub mod war; // war-фазовые и city-ownership действия над скалярами переходного агрегата.
pub mod weather; // data-контракты и tick/change-действия погодных сегментов.
