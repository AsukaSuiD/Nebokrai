//! Мировые фигуры и регионы Realm: пространственные типы и их DB-владелец.

pub mod baseobject; // CBaseObject: базовая type/ID identity мирового объекта.
pub mod moveshape; // CMoveShape: подвижная форма мира.
pub mod region; // CRegion: регион мира.
pub mod rsregion; // CRsRegion: World DB-владелец регионов.
pub mod shape; // CShape: форма мира.
pub mod shapetypes; // codec error-типы мирового CShape и его CBaseObject.
pub mod worldcityregion; // CWorldCityRegion: городской регион мира.
pub mod worldcountrywarregion; // WorldCountryWarRegion: country-war регион мира.
pub mod worldregion; // CWorldRegion: мировой регион.
pub mod worldvillageregion; // CWorldVillageRegion: владелец деревенского региона.
pub mod worldwarregion; // CWorldWarRegion: промежуточный war-регион мира.
pub mod worldzones; // реестр обслуживающих Zone мира: назначения регионов, записи GameServer и ping-индекс.
