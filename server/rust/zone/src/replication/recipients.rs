//! Spatial/recipient snapshot данных для around-family рассылок одного region
//! владельца. Игровые объекты, ресурсы и живой region registry остаются у
//! owner-а regions; этот snapshot — то самое доказанное представление одного
//! точного момента по [карте владельцев]. Применяется владельцем
//! `destinations` send-family (`deep SendToAround` RVA `0x14970`), и его
//! собственный контракт совпадает с живой проверкой: индекс области —
//! `area_x * y + x` (row-major) с проверкой положительной границы, а
//! `is_in_around` — `abs_diff < 2` по обеим осям сохранённых area index
//! (`CShape::IsInAround` RVA `0x5BCE0` в той же пары `gameserver.exe` +
//! `GameServer.pdb`).
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

/// Одна область получателей spatial snapshot: координаты и ordered identities.
pub struct ServerRegionRecipientArea {
    /// Индекс области по `x` (`CArea::x`).
    pub x: i32,
    /// Индекс области по `y` (`CArea::y`).
    pub y: i32,
    /// Ordered identities игроков области на момент снимка.
    pub player_ids: Vec<i32>,
}

/// Spatial/recipient snapshot одного region владельца: координаты grid и
/// списки ordered identities игроков на момент снимка. Игровые объекты,
/// ресурсы и живой registry остаются у владельца regions.
pub struct ServerRegionRecipientsSnapshot {
    region_id: i32,
    area_x: i32,
    area_y: i32,
    areas: Vec<ServerRegionRecipientArea>,
}

impl ServerRegionRecipientsSnapshot {
    /// Создаёт snapshot из выбранного region owner-а с выложенными areas.
    pub const fn from_parts(
        region_id: i32,
        area_x: i32,
        area_y: i32,
        areas: Vec<ServerRegionRecipientArea>,
    ) -> Self {
        Self {
            region_id,
            area_x,
            area_y,
            areas,
        }
    }

    /// ID региона-владельца списка (spatial index).
    pub const fn region_id(&self) -> i32 {
        self.region_id
    }

    /// Доводит ordered identities игроков области в получатель; игнорирует
    /// координаты вне собственного grid.
    pub fn find_player_ids_in_area(&self, x: i32, y: i32, destination: &mut Vec<i32>) {
        if x < 0 || x >= self.area_x || y < 0 || y >= self.area_y {
            return;
        }
        let index = usize::try_from(self.area_x * y + x).expect("положительный grid index");
        if let Some(area) = self.areas.get(index) {
            destination.extend_from_slice(&area.player_ids);
        }
    }

    /// Доводит ordered identities всех игроков region в получатель.
    pub fn find_all_player_ids(&self, destination: &mut Vec<i32>) {
        for area in &self.areas {
            destination.extend_from_slice(&area.player_ids);
        }
    }

    /// Та же проверка `IsInAround` по сохранённым area-index: истинна, когда
    /// обе abs-разности меньше 2. Текущие region ID и area index самих игроков
    /// остаются за их текущими shape-владельцами.
    pub fn is_in_around(&self, shape_index: Option<usize>, other_index: Option<usize>) -> bool {
        let (Some(shape_index), Some(other_index)) = (shape_index, other_index) else {
            return false;
        };
        let (Some(shape_area), Some(other_area)) =
            (self.areas.get(shape_index), self.areas.get(other_index))
        else {
            return false;
        };
        shape_area.x.abs_diff(other_area.x) < 2 && shape_area.y.abs_diff(other_area.y) < 2
    }
}
