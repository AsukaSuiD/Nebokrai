//! Typed-отказ пространственного членства `CServerRegion` исторического
//! GameServer (порция 1). Исходный владелец — `appserver/serverregion.h/.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.

use crate::regions::moveshape::MoveShapePositionBlock;
use crate::regions::region::RegionCellAccessBlock;
use crate::regions::shape::{ShapeBlockError, ShapeCoordinateBlock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionMembershipBlock {
    InvalidAreaSpan { width: i32, height: i32 },
    StaleAreaIndex { index: usize, available: usize },
    ShapeCoordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    RegionCell(RegionCellAccessBlock),
    MoveShape(MoveShapePositionBlock),
}

pub fn validate_area_span(width: i32, height: i32) -> Result<(), RegionMembershipBlock> {
    if width <= 0 || height <= 0 {
        // BLOCKED_MISSING_FACT: zero вызывает x86 `idiv` trap, negative
        // GlobeSetup span не имеет доказанного переносимого runtime contract.
        return Err(RegionMembershipBlock::InvalidAreaSpan { width, height });
    }
    Ok(())
}
