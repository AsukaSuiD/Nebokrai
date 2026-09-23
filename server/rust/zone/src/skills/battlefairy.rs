//! Правила навыков боевого духа.
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp/.h
//! (проверка уровня VA 0x0042E7A0–0x0042E81C) и
//! appserver/player.cpp/.h (снятие/установка девяти навыков
//! VA 0x00430610–0x00430756 и 0x00430760–0x00430921),
//! appserver/skills/wangsheng.cpp/.h (стоимость текста MP
//! CWangsheng::AI VA 0x0051E097–0x0051E0E7).

/// Какое свойство предмета запрашивает правило; числовой ключ и чтение
/// живого предмета остаются у Game до переноса его владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillProperty {
    RequestedOffset(i32),
    Sky,
    Earth,
    Man,
    SkySkill,
    EarthSkill,
    ManSkill,
    AllSkill,
    Huoxieshu,
    Lingzhishu,
}

/// Порядок слотов при снятии и установке навыков головного предмета.
pub const EQUIPPED_SKILL_PROPERTIES: [BattleFairySkillProperty; 9] = [
    BattleFairySkillProperty::Sky,
    BattleFairySkillProperty::Earth,
    BattleFairySkillProperty::Man,
    BattleFairySkillProperty::SkySkill,
    BattleFairySkillProperty::EarthSkill,
    BattleFairySkillProperty::ManSkill,
    BattleFairySkillProperty::AllSkill,
    BattleFairySkillProperty::Huoxieshu,
    BattleFairySkillProperty::Lingzhishu,
];

/// Читает ID непосредственно перед снятием одного слота.
pub fn battle_fairy_skill_id(
    property: BattleFairySkillProperty,
    mut read_property: impl FnMut(BattleFairySkillProperty, u32) -> i32,
) -> u32 {
    read_property(property, 2) as u32
}

/// Установка первых семи навыков читает уровень перед ID. Для двух
/// последних читается только ID, уровень передаётся как единица.
pub fn battle_fairy_skill_entry(
    property: BattleFairySkillProperty,
    mut read_property: impl FnMut(BattleFairySkillProperty, u32) -> i32,
) -> (u32, i32) {
    match property {
        BattleFairySkillProperty::Huoxieshu | BattleFairySkillProperty::Lingzhishu => {
            (read_property(property, 2) as u32, 1)
        }
        _ => {
            let level = read_property(property, 1);
            (read_property(property, 2) as u32, level)
        }
    }
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
