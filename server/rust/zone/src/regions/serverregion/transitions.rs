//! Immutable transition-контракт смены области `CServerRegion` исторического
//! GameServer (порция 1). Исходный владелец — `appserver/serverregion.h/.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.

use crate::regions::ShapeIdentity;
use crate::regions::shape::ShapeView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaTransitionBlock {
    StaleAreaIndex { index: usize, available: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaTransitionAudience {
    pub area_x: i32,
    pub area_y: i32,
    pub shapes_for_moving_player: Vec<ShapeView>,
}

/// Immutable effect-plan исходного `OnShapeChangeArea`. Регион вычисляет
/// exclusive areas и их ordered shape snapshots до membership mutation;
/// `CGame` исполняет клиентский wire, затем регион применяет target index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaTransitionPlan {
    pub moving: ShapeIdentity,
    pub current_index: usize,
    pub target_index: Option<usize>,
    pub audience: Vec<AreaTransitionAudience>,
}
