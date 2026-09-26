//! Идентичность фигур и data-ядра регионов: shape/moveshape агрегаты,
//! доменные data-профили (npc/monster/build/гейты), spatial и war-семьи
//! `CServerRegion`. Исходные владельцы — `appserver/*` исторического
//! GameServer; переходные агрегаты старого пакета делегируют этим модулям
//! поведение своих колонок без изменения сигнатур.

pub mod area; // CArea: storage-часть ячеек области.
pub mod baseobject; // CBaseObject: базовая идентичность type/ID/GUID и имя объекта.
pub mod build; // CBuild: data-семья и скалярные правила постройки.
pub mod citygate; // CCityGate: data-семья, скалярные правила и региональное гейтовое тело городских ворот.
mod identity; // ShapeIdentity: локальная игровая ссылка type/ID/GUID.
pub mod monster; // CMonster: скалярная база и правила монстра (script/tame/pet, защита первого удара).
pub mod moveshape; // MoveShapeState (CMoveShape): агрегат фигуры — пространство, реестр навыков, арена состояний и скалярные колонки.
pub mod npc; // CNpc: data-профиль и скалярные правила NPC.
pub mod proxyserverregion; // CProxyServerRegion: proxy-регион GameServer.
pub mod region; // CRegion: spatial/persistence поверхность региона.
pub mod regionparam; // tagRegionParam: wire-проекция налогов и владения городом.
pub mod servercityregion; // CServerCityRegion: скалярный state, данные и правила городского war-региона.
pub mod servercountryregion; // ServerCountryRegion: скалярный state, данные и правила country war-региона.
pub mod servergodsbattleregion; // CGodsBattleMgr/CServerGodsBattleRegion: GodsBattle state, top-ten decoder и скалярные правила.
pub mod servernationregion; // ServerNationRegion: скалярный state, данные и правила nation war-региона.
pub mod serverregion; // CServerRegion: data-типы и позиционное ядро региона.
pub mod servervillageregion; // CServerVillageRegion: context-контракты, эффекты и скалярные правила деревенского war-региона.
pub mod serverwarregion; // CServerWarRegion: данные, wire-stream decoder-семья и скалярные операции war-региона.
pub mod shape; // CShape: spatial/membership-часть и геометрия фигуры.
pub mod skillregistry; // CMoveShape: реестр навыков (категории, current/item, скалярная identity записи).
pub mod summonedcreature; // CSummonedCreature: жизненный цикл призванного монстра.

pub use identity::ShapeIdentity; // локальная игровая ссылка на фигуру (type/ID/GUID).
