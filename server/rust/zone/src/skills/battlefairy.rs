//! Правила навыков боевого духа.
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp/.h
//! (проверка уровня VA 0x0042E7A0–0x0042E81C) и
//! appserver/skills/wangsheng.cpp/.h (стоимость текста MP
//! CWangsheng::AI VA 0x0051E097–0x0051E0E7).

/// Какое свойство предмета запрашивает правило; числовой ключ и чтение
/// живого предмета остаются у Game до переноса его владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillProperty {
    RequestedOffset(i32),
    Huoxieshu,
    Lingzhishu,
}

/// Нулевой offset разрешает уровень 1 без чтения предмета. При несовпадении
/// ID выбранного свойства fallback читает 224, затем при необходимости 225.
pub fn battle_fairy_skill_level(
    property_offset: i32,
    requested_skill: u32,
    mut read_property: impl FnMut(BattleFairySkillProperty, u32) -> i32,
) -> i32 {
    if property_offset == 0 {
        return 1;
    }
    let requested = BattleFairySkillProperty::RequestedOffset(property_offset);
    if read_property(requested, 2) as u32 == requested_skill {
        return read_property(requested, 1);
    }
    if read_property(BattleFairySkillProperty::Huoxieshu, 2) == 0x222
        || read_property(BattleFairySkillProperty::Lingzhishu, 2) == 0x223
    {
        return 1;
    }
    0
}

/// Unsigned DWORD → double 0.0001 → усечение в QWORD → младший DWORD.
pub fn battle_fairy_mana_text_cost(cost: u32) -> u32 {
    (f64::from(cost) * 0.0001_f64).trunc() as i64 as u32
}
