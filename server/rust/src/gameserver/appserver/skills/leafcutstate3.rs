//! Третье рассечение, источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/leafcutstate3.cpp. Общий layout, формула и lifecycle
//! семейства LeafCut сохраняют отдельную идентичность состояния 0x8F.

pub(crate) use nebokrai_zone::effects::LEAF_CUT_3_STATE_ID;
pub(crate) type LeafCutState3 = super::leafcutstate::LeafCutState<LEAF_CUT_3_STATE_ID>;

// Для координатного Begin (0x005EBD10) и typed Begin (0x005EBDB0)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
