//! Третье рассечение, источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/leafcutstate3.cpp. Общий layout, формула и lifecycle
//! семейства LeafCut сохраняют отдельную идентичность состояния 0x8F.

pub(crate) const LEAF_CUT_3_STATE_ID: u32 = 0x8f;
pub(crate) use super::leafcutstate::LEAF_CUT_STATE_BYTES as LEAF_CUT_3_STATE_BYTES;
pub(crate) type LeafCutState3 = super::leafcutstate::LeafCutState<LEAF_CUT_3_STATE_ID>;

// Для координатного Begin (0x005EBD10) и typed Begin (0x005EBDB0)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
