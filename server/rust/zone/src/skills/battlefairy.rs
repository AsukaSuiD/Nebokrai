//! Правила навыков боевого духа.
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp/.h
//! (проверка уровня VA 0x0042E7A0–0x0042E81C) и
//! appserver/player.cpp/.h (снятие/установка девяти навыков
//! VA 0x00430610–0x00430756 и 0x00430760–0x00430921),
//! constructor `CBattleFairyContainer` (пары несовместимых навыков
//! VA 0x00504052–0x00504149),
//! container/cbattlefairycontainer.cpp/.h (проверка кандидата ResetSkill
//! VA 0x00501815–0x00501A8D),
//! appserver/skills/wangsheng.cpp/.h (стоимость текста MP
//! CWangsheng::AI VA 0x0051E097–0x0051E0E7).

/// Пара, записанная в исходную `m_UnPairSkills` при создании контейнера.
/// Порядок попыток и вызовов RNG остаётся в операции сброса у игрока.
pub const fn unpaired_battle_fairy_skill(skill_id: u32) -> Option<u32> {
    Some(match skill_id {
        530 => 534,
        531 => 535,
        532 => 536,
        533 => 537,
        534 => 530,
        535 => 531,
        536 => 532,
        537 => 533,
        _ => return None,
    })
}

/// Проверка новой попытки для одного из трёх слотов стихии. Ноль после
/// DWORD-сложения принимается до сравнения с текущими навыками.
pub fn battle_fairy_reset_candidate_allowed(
    candidate: u32,
    current_skills: [u32; 3],
    replaced: usize,
) -> bool {
    if candidate == 0 {
        return true;
    }
    if current_skills.contains(&candidate) {
        return false;
    }
    !unpaired_battle_fairy_skill(candidate).is_some_and(|paired| {
        current_skills
            .iter()
            .enumerate()
            .any(|(index, &skill)| index != replaced && skill == paired)
    })
}

/// Общий слот отвергает только прежний ID; ноль принимается раньше сравнения.
pub const fn battle_fairy_all_skill_candidate_allowed(candidate: u32, current: u32) -> bool {
    candidate == 0 || candidate != current
}

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
