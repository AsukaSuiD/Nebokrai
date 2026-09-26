//! Компонент `players`: реестр живых игроков и агрегат персонажа Zone.
//!
//! Родившаяся часть — GameSave codec и клиентские снимки живого игрока
//! (`docs/architecture/realm-and-zone.md` §5: `players/` — реестр живых
//! игроков, агрегат персонажа, подготовка проекций). Hub `CPlayer` старого
//! пакета (`gameserver/appserver/player.rs`) остаётся владельцем агрегата и
//! состояния и бережёт делегаты прежних сигнатур; codec оперирует
//! принадлежащими игроку зонными частями через заёмные проекции
//! (`PlayerGameSaveParts`, организационные `PlayerOrganizing*`,
//! `PlayerClientShape/InitialClientParts`), без ссылок на `CPlayer`
//! (циклическая зависимость) и без dyn/Send-швов.
//!
//! Persistence-коллекции игрока хранят владельцы частей: контейнеры
//! `items/`, свойства (`combat::PlayerCombatProperties` — общий боевой
//! снимок), переменные `scripts/`, прогресс заданий `quests/`,
//! `MoveShapeState` `regions/moveshape`, состояния `skills/state`.
//!
//! Временные generic-швы до ухода кодов-владельцев из старого пакета
//! (закрываются следующими волнами): realm-appellation bonus-предикат и
//! результат требований `CanMountEquip` — см. шапку `gamesave.rs`.

pub mod clientsnapshot; // клиентские снимки игрока AddToByteArray_ForClient: short area/query и полный login 0xBF401.
pub mod gamesave; // GameSave codec игрока: DecordFromByteArray/AddGameSaveToByteArray, wires свойств, организационный блок, LeiTing, persistence-проекции.
