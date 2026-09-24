//! Достигнутая spatial/membership-часть `CShape` старого GameServer перенесена
//! в Zone regions. Здесь реэкспорт типов и адаптер area-lookup к `CServerRegion`.

pub(crate) use nebokrai_zone::regions::shape::*;
pub(crate) use nebokrai_zone::regions::ShapeIdentity;

use super::serverregion::CServerRegion;

impl ShapeAreaLookup for CServerRegion {
    fn area_coordinates(&self, index: i32) -> Option<ShapeAreaCoordinates> {
        let area = self.get_area_by_index(index).ok()?;
        Some(ShapeAreaCoordinates {
            x: area.x(),
            y: area.y(),
        })
    }
}
