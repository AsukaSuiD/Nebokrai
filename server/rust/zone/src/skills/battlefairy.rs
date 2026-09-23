//! Число стоимости MP для сообщения навыков боевого духа.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/wangsheng.cpp/.h;
//! CWangsheng::AI VA 0x0051E097–0x0051E0E7.

/// Unsigned DWORD → double 0.0001 → усечение в QWORD → младший DWORD.
pub fn battle_fairy_mana_text_cost(cost: u32) -> u32 {
    (f64::from(cost) * 0.0001_f64).trunc() as i64 as u32
}
