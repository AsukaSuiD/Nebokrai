//! Межвладельческая spatial-координация `CSpiderMistPhalanx`.
//!
//! Маска и правила воздействия принадлежат skill-owner-у. Здесь остаётся
//! только разрешение ordered `CServerRegion::GetShape` для каждой активной
//! клетки при живом region owner-е; повторы намеренно сохраняются, поскольку
//! первое установленное состояние блокирует последующие совпадения.

use super::*;

impl CGame {
    pub(super) fn spider_mist_targets(
        &self,
        region_id: i32,
        phalanx: &crate::gameserver::appserver::skills::spidermistphalanx::CSpiderMistPhalanx,
    ) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else {
            return Vec::new();
        };
        let mut targets = Vec::new();
        for (tile_x, tile_y) in phalanx.active_cells() {
            let mut shapes = Vec::new();
            if region
                .get_shapes(
                    tile_x,
                    tile_y,
                    self.area_width,
                    self.area_height,
                    self,
                    &mut shapes,
                )
                .is_err()
            {
                break;
            }
            targets.extend(shapes.into_iter().map(|shape| shape.identity));
        }
        targets
    }
}
