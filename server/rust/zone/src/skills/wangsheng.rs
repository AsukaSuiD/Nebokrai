//! Прямое восстановление HP навыком CWangsheng, без создания WangshengState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/wangsheng.cpp/.h;
//! конструктор ID и vtable VA 0x0051D4E0–0x0051D514, AI через слот +0x90
//! VA 0x0051DBD0, участок лечения VA 0x0051E01D–0x0051E043.

pub const WANGSHENG_SKILL_ID: u32 = 0x221;
const TARGET_HP_GAIN: u32 = 31;

/// Текущий HP читается до свойства; сумма передаётся setter-у без ограничения.
pub fn wangsheng_restored_health(
    current: u32, mut query_property: impl FnMut(u32) -> u32,
) -> u32 {
    current.wrapping_add(query_property(TARGET_HP_GAIN))
}
