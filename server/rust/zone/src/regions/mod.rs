//! Идентичность фигур, используемая живыми компонентами Zone.

pub mod area; // CArea: storage-часть ячеек области.
pub mod baseobject; // CBaseObject: базовая идентичность type/ID/GUID и имя объекта.
pub mod build; // CBuild: data-семья и скалярные правила постройки.
pub mod citygate; // CCityGate: data-семья и скалярные правила городских ворот.
mod identity; // ShapeIdentity: локальная игровая ссылка type/ID/GUID.
pub mod monster; // CMonster: скалярная база и правила монстра (script/tame/pet, защита первого удара).
pub mod moveshape; // CMoveShape: пространственное ядро и скалярные колонки (запреты, направления, питомцы).
pub mod npc; // CNpc: data-профиль и скалярные правила NPC.
pub mod proxyserverregion; // CProxyServerRegion: proxy-регион GameServer.
pub mod region; // CRegion: spatial/persistence поверхность региона.
pub mod regionparam; // tagRegionParam: wire-проекция налогов и владения городом.
pub mod serverregion; // CServerRegion: data-типы и позиционное ядро региона.
pub mod shape; // CShape: spatial/membership-часть и геометрия фигуры.
pub mod skillregistry; // CMoveShape: реестр навыков (категории, current/item, скалярная identity записи).
pub mod summonedcreature; // CSummonedCreature: жизненный цикл призванного монстра.

pub use identity::ShapeIdentity; // локальная игровая ссылка на фигуру (type/ID/GUID).
