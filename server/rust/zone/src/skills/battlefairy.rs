//! Правила навыков боевого духа.
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp/.h
//! (проверка уровня VA 0x0042E7A0–0x0042E81C) и
//! appserver/player.cpp/.h (снятие/установка девяти навыков
//! VA 0x00430610–0x00430756 и 0x00430760–0x00430921),
//! constructor `CBattleFairyContainer` (пары несовместимых навыков
//! VA 0x00504052–0x00504149),
//! container/cbattlefairycontainer.cpp/.h (проверка кандидата ResetSkill
//! VA 0x00501815–0x00501A8D, допуск VA 0x00501599–0x00501645,
//! поиск VA 0x00501663–0x0050178A,
//! чтение VA 0x005017B4–0x00501806,
//! запись VA 0x00501892–0x00501AC7 и стоимость уведомления
//! VA 0x00501B6B–0x00501B8D),
//! appserver/player.cpp/.h (расход одного предмета: количество
//! VA 0x004315E9–0x00431615, удаление VA 0x0043169C–0x00431702),
//! appserver/skills/wangsheng.cpp/.h (стоимость текста MP
//! CWangsheng::AI VA 0x0051E097–0x0051E0E7).

/// Пара, записанная в исходную `m_UnPairSkills` при создании контейнера.
const fn unpaired_battle_fairy_skill(skill_id: u32) -> Option<u32> {
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
fn battle_fairy_reset_candidate_allowed(
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
const fn battle_fairy_all_skill_candidate_allowed(candidate: u32, current: u32) -> bool {
    candidate == 0 || candidate != current
}

/// Выбирает новый ID после снятия прежних навыков. Каждый повтор вызывает
/// переданный общий игровой RNG ровно один раз; изменение предмета — у игрока.
pub fn select_battle_fairy_reset_skill(
    replaced: Option<usize>,
    current_skills: [u32; 3],
    current_all_skill: u32,
    random: &mut dyn FnMut(i32) -> i32,
) -> u32 {
    match replaced {
        Some(replaced) => loop {
            let candidate = 530_u32.wrapping_add(random(13) as u32);
            if battle_fairy_reset_candidate_allowed(candidate, current_skills, replaced) {
                break candidate;
            }
        },
        None => loop {
            let candidate = 543_u32.wrapping_add(random(3) as u32);
            if battle_fairy_all_skill_candidate_allowed(candidate, current_all_skill) {
                break candidate;
            }
        },
    }
}

/// После выбора ID четыре раза меняет выбранное свойство головного предмета.
/// Порядок и промежуточные значения сохраняются даже при отказе одной записи;
/// доступ к живому предмету и числовой ключ предоставляет Game.
pub fn write_battle_fairy_reset_skill(selected_skill: u32, mut write_value: impl FnMut(u32, i32)) {
    write_value(1, 0);
    write_value(2, 0);
    write_value(1, 1);
    write_value(2, selected_skill as i32);
}

/// Проверки ResetSkill до обращения к предмету сброса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyResetPreflight {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    Ready,
}

/// Флаг проверяется до чтения головного предмета; свойство требуется ровно 1.
pub fn battle_fairy_reset_preflight(
    enabled: bool,
    read_headgear_flag: impl FnOnce() -> Option<i32>,
) -> BattleFairyResetPreflight {
    if !enabled {
        return BattleFairyResetPreflight::FeatureDisabled;
    }
    match read_headgear_flag() {
        None => BattleFairyResetPreflight::MissingHeadgear,
        Some(1) => BattleFairyResetPreflight::Ready,
        Some(_) => BattleFairyResetPreflight::InvalidHeadgear,
    }
}

/// При расходе одного предмета количество 0 или 1 ведёт к попытке удаления.
/// Фактическую операцию над рюкзаком выполняет игрок.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyResetItemChange {
    Remove,
    SetAmount(u32),
}

pub const fn battle_fairy_reset_item_change(amount: u32) -> BattleFairyResetItemChange {
    if amount > 1 {
        BattleFairyResetItemChange::SetAmount(amount - 1)
    } else {
        BattleFairyResetItemChange::Remove
    }
}

/// Результат выбора предмета после двух запросов к рюкзаку.
pub enum BattleFairyResetItemLookup<T> {
    InvalidPosition,
    MissingItem,
    Found(T),
}

/// Оригинал запрашивает оба имени до проверки позиции; поиск живых предметов
/// передаёт Game. Для неверной позиции предмет не расходуется.
pub fn battle_fairy_reset_item<T>(
    position: i32,
    mut find_item: impl FnMut(&[u8]) -> Option<T>,
) -> BattleFairyResetItemLookup<T> {
    let elemental = find_item(b"ZHJNS01");
    let all_skill = find_item(b"ZHJNS02");
    let selected = match position {
        3..=5 => elemental,
        6 => all_skill,
        _ => return BattleFairyResetItemLookup::InvalidPosition,
    };
    selected.map_or(
        BattleFairyResetItemLookup::MissingItem,
        BattleFairyResetItemLookup::Found,
    )
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

/// Четыре ID головного предмета, прочитанные перед выбором ветви ResetSkill.
pub struct BattleFairyResetSlot {
    pub property: BattleFairySkillProperty,
    pub replaced: Option<usize>,
    pub current_skills: [u32; 3],
    pub current_all_skill: u32,
}

impl BattleFairyResetSlot {
    pub fn previous_skill(&self) -> u32 {
        self.replaced
            .map_or(self.current_all_skill, |index| self.current_skills[index])
    }
}

/// Четыре getter-а вызываются до проверки позиции, включая неверную позицию.
pub fn battle_fairy_reset_slot(
    position: i32,
    mut read_property: impl FnMut(BattleFairySkillProperty, u32) -> i32,
) -> Option<BattleFairyResetSlot> {
    let current_skills = [
        read_property(BattleFairySkillProperty::SkySkill, 2) as u32,
        read_property(BattleFairySkillProperty::EarthSkill, 2) as u32,
        read_property(BattleFairySkillProperty::ManSkill, 2) as u32,
    ];
    let current_all_skill = read_property(BattleFairySkillProperty::AllSkill, 2) as u32;
    let (property, replaced) = match position {
        3 => (BattleFairySkillProperty::SkySkill, Some(0)),
        4 => (BattleFairySkillProperty::EarthSkill, Some(1)),
        5 => (BattleFairySkillProperty::ManSkill, Some(2)),
        6 => (BattleFairySkillProperty::AllSkill, None),
        _ => return None,
    };
    Some(BattleFairyResetSlot {
        property,
        replaced,
        current_skills,
        current_all_skill,
    })
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

/// Уведомление выбранного ResetSkill трактует DWORD стоимости как знаковый
/// перед double-масштабированием и записью младшего WORD.
pub fn battle_fairy_reset_notice_cost(cost: u32) -> i32 {
    (f64::from(cost as i32) * 0.0001_f64).trunc() as i32
}
