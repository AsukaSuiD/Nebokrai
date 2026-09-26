//! Достигнутый object/addon core `CGoods` старого GameServer перенесён в Zone items.
//! Здесь его реэкспорт для старого пакета; реализация шва lookup реестра базовых
//! свойств переехала в Zone вместе с `CGoodsFactory` (волна Z-G0b).

pub(crate) use nebokrai_zone::items::cgoods::*;
