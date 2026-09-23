//! Параметры и маски областей CYinYang и CYinYang2.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/yinyang{,2}.cpp и yinyangphalanx{,2}.cpp/.h.
//! Summon VA 0x005A6270 и 0x005682E0; конструкторы областей
//! VA 0x005FE4F0 и 0x005F2410.

pub const YIN_YANG_SKILL_ID: u32 = 0x139;
pub const YIN_YANG_2_SKILL_ID: u32 = 0x146;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const LIFETIME_PROPERTY: u32 = 30_001;
const FULL: [bool; 9] = [true; 9];
const SINGLE: [bool; 1] = [true];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct YinYangSummonParameters {
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub lifetime_ms: u32,
}

impl YinYangSummonParameters {
    /// Текущий экземпляр читается после двух свойств атаки и до срока.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        current_skill: impl FnOnce() -> Option<(u32, i32)>,
    ) -> Option<Self> {
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let (skill_id, skill_level) = current_skill()?;
        let lifetime_ms = query_property(LIFETIME_PROPERTY);
        Some(Self { skill_id, skill_level, minimum_attack, maximum_attack, lifetime_ms })
    }
}

pub fn yin_yang_scope(skill_id: u32) -> (i32, i32, &'static [bool]) {
    if skill_id == YIN_YANG_2_SKILL_ID { (1, 1, &SINGLE) }
    else { (3, 3, &FULL) }
}
