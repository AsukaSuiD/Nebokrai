//! Горючая смесь CKeroseneState (0xF1).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/kerosenestate.cpp.
//!
//! Наносит фиксированный signed DWORD урона Poison без RNG и ограничения
//! снизу нулём. Срок проверяется до попытки удара. Payload и lifecycle
//! принадлежат общему механизму poison; состояние остаётся отдельным объектом
//! арены. Load читает timestamp после всех полей 56-байтной записи, не сразу
//! после MasterInfo; общий кодек сохраняет этот порядок.

pub(crate) const KEROSENE_STATE_ID: u32 = 0xf1;
pub(crate) use crate::gameserver::appserver::states::poison::{
    POISON_STATE_BYTES as KEROSENE_STATE_BYTES,
    begin_primary_poison_state as begin_primary_kerosene_state,
};
pub(crate) type KeroseneState =
    crate::gameserver::appserver::states::poison::PoisonState<KEROSENE_STATE_ID>;

// Для координатного Begin (0x005EB5F0) и typed Begin (0x005EB690)
// вызывающие цепочки не установлены; общий Begin обслуживает объектную перегрузку.
