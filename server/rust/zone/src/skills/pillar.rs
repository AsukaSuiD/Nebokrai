//! Параметры создаваемой стойки CPillar.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/pillar.cpp/.h;
//! CPillar::AI, VA 0x0057033C–0x0057039B.

pub const PILLAR_SKILL_ID: u32 = 0x74;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const STATE_PERSIST_TIME: u32 = 10_002;

/// Запросы выполняются только при создании нового состояния: factor, затем срок.
pub fn pillar_state_parameters(mut query_property: impl FnMut(u32) -> u32) -> (u32, f32) {
    let factor =
        (f64::from(query_property(TARGET_DAMAGE_FACTOR)) * f64::from(0.001_f32)) as f32;
    let keep_time_ms = query_property(STATE_PERSIST_TIME);
    (keep_time_ms, factor)
}
