//! Второе рассечение, источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/leafcutstate2.cpp. Общий layout, формула и lifecycle
//! семейства LeafCut сохраняют отдельную идентичность состояния 0x80.

pub(crate) use nebokrai_zone::effects::LEAF_CUT_2_STATE_ID;
pub(crate) use super::leafcutstate::LEAF_CUT_STATE_BYTES as LEAF_CUT_2_STATE_BYTES;
pub(crate) type LeafCutState2 = super::leafcutstate::LeafCutState<LEAF_CUT_2_STATE_ID>;

// Для координатного Begin (0x005F06D0) и typed Begin (0x005F0770)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
