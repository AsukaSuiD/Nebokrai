//! Фабричные владельцы и реестр runtime-свойств навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/skills/skillfactory.cpp`. `Rebuild` очищает
//! прежний реестр до чтения `count`, проходит все ячейки знакового количества, пропускает
//! нулевую длину и публикует декодированную запись по ключу
//! `skill_id << 16 | level & 0xffff`; повторный ключ заменяется. Имя остаётся
//! точный по байтам префикс C-string, а карта usage использует последнее значение ключа.
//!
//! Точный цикл и malformed-record skip подтверждены дизассемблировкой RVA
//! `0x0006CE10`; WorldServer serializer является парной стороной wire-контракта.
//! Все 209 соответствий ID → конкретный класс подтверждены переходами и
//! вызовами конструкторов `QuerySkill` (VA `0x00469870`), включая специальные
//! ID и отдельные классы `CNonFun*`. Категория экземпляра подтверждена записью
//! `[this+0x48]` базовыми конструкторами: `CAttackSkill` (`0x005DFA80`) — 0,
//! `CDefenseSkill` (`0x006019C0`) — 1, `CStateSkill` (`0x005DFB50`) — 2,
//! `CSummonSkill` (`0x005E0EC0`) — 3. Проверены именно вызовы для `this`:
//! некоторые concrete-конструкторы дополнительно создают на стеке временный
//! объект другой категории; он не меняет категорию экземпляра. Поэтому,
//! например, `CArchery` относится к Summon, а `CSeal` и `CThunderBlow2` — к Attack.
//! Единственный каталог порождает `SkillOwner`, категорию и `factory_owner`;
//! `supports_skill_id` использует его же null/non-null границу. Перезагружаемый
//! `QuerySkillType` остаётся отдельным lookup runtime-свойств и не подменяется
//! категорией конструктора. QuerySkillType (0x0046C3D0) не проверяет admission:
//! даже ID без concrete factory-owner возвращает категорию имеющейся записи.
//! Этот каталог не реализует создание состояния или
//! `End` конкретных навыков: их lifecycle остаётся в соответствующих owner-ах.
//! `BTreeMap`, `Vec` и `Drop` заменяют служебный код MSVC map/heap. Старые
//! переполнения и выходы за границу при повреждённой длине остановлены
//! типизированной ошибкой без дополнительных side effects. Неизвестности
//! concrete layout и lifecycle ещё не восстановленных skill-owner-ов этот
//! каталог не закрывает; выбор класса, его категория и операции реестра
//! свойств уже исполняются в Rust.

use std::collections::BTreeMap;
use thiserror::Error;

use super::skillbaseproperties::{CSkillBaseProperties, UNKNOWN_SKILL_TYPE};
use super::super::legacycodec::LegacyReader;

pub(crate) const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
const MAX_SKILL_NAME_LENGTH: usize = 255;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum SkillCategory {
    Attack = 0,
    Defense = 1,
    State = 2,
    Summon = 3,
}

impl SkillCategory {
    pub(crate) const fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Attack),
            1 => Some(Self::Defense),
            2 => Some(Self::State),
            3 => Some(Self::Summon),
            _ => None,
        }
    }
}

macro_rules! skill_owners {
    ($($(#[$attribute:meta])* $owner:ident: $category:ident => $id:pat),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum SkillOwner {
            $($(#[$attribute])* $owner,)+
        }

        impl SkillOwner {
            pub(crate) const fn category(self) -> SkillCategory {
                match self {
                    $(Self::$owner => SkillCategory::$category,)+
                }
            }
        }

        impl CSkillFactory {
            pub(crate) const fn factory_owner(skill_id: u32) -> Option<SkillOwner> {
                match skill_id {
                    $($id => Some(SkillOwner::$owner),)+
                    _ => None,
                }
            }
        }
    };
}

// Имена классов сохранены по PDB. Это единственный каталог фабричного выбора:
// категория принадлежит конструктору экземпляра, а не записи runtime-свойств.
skill_owners! {
    CBaseAttack: Attack => 0x001,
    CArchery: Summon => 0x002,
    CBaseMagic: Summon => 0x003,
    CFightDefense: Defense => 0x00a,
    CMosou: Attack => 0x065,
    CGhostCut: Attack => 0x066,
    CKnightCut: State => 0x067,
    CArmyBreak: Attack => 0x068,
    CFlash: Attack => 0x069,
    CSwallow: Attack => 0x06a,
    CLeafCut: State => 0x06b,
    CJuCut: Attack => 0x06c,
    CRage: State => 0x06d,
    CRageBreak: State => 0x06e,
    CSwordship: State => 0x06f,
    CLightningSword: Attack => 0x070,
    CLittleFlash: Attack => 0x071,
    CThunderSlash: Summon => 0x072,
    CRush: State => 0x073,
    CPillar: State => 0x074,
    CCallosity: State => 0x075,
    CBlind: State => 0x076,
    CLightningSword2: Attack => 0x077,
    CLightningSword3: Attack => 0x078,
    CGhostCut2: Attack => 0x079,
    CGhostCut3: Attack => 0x07a,
    CArmyBreak2: Attack => 0x07b,
    CRush2: State => 0x07c,
    CCallosity2: State => 0x07d,
    CLightningSword4: Attack => 0x07e,
    CLittleFlash2: Attack => 0x07f,
    CLeafCut2: State => 0x080,
    CAgility2: State => 0x081,
    CRoar: State => 0x083,
    CEnergyHolding: State => 0x089,
    CInverseChopped: Attack => 0x08a,
    CLeafCut3: State => 0x08f,
    CPoisonFog: Summon => 0x0c9,
    CHeartLessArrow: Attack => 0x0ca,
    CLightingArrow: Attack => 0x0cb,
    CMeteorArrowMass: State => 0x0cc,
    CMeteorArrow: Summon => 0x0cd,
    CRainArrow: Attack => 0x0ce,
    CPoisonMoth: Attack => 0x0cf,
    CBloodRose: Attack => 0x0d0,
    CScorpion: Attack => 0x0d1,
    CBoaLock: State => 0x0d2,
    CHeal: State => 0x0d3,
    CMonsterTaming: Attack => 0x0d4,
    CFallingStar: Summon => 0x0d5,
    CExplosiveArrow: Attack => 0x0d6,
    CPetsControl: Attack => 0x0d7,
    CGibe: Attack => 0x0d8,
    CSuperHeal: State => 0x0d9,
    CAgility: State => 0x0da,
    CRapture: State => 0x0db,
    CNatural: State => 0x0dc,
    CStrike: Attack => 0x0dd,
    CMachineShield: State => 0x0de,
    CDaubPoison: State => 0x0df,
    CSwordship2: State => 0x0e0,
    CExplosiveArrow2: Attack => 0x0e1,
    CExplosiveArrow3: Attack => 0x0e2,
    CHeal2: State => 0x0e3,
    CSuperHeal2: State => 0x0e4,
    CHeartLessArrow2: Attack => 0x0e5,
    CHeartLessArrow3: Attack => 0x0e6,
    CLightingArrow2: Attack => 0x0e7,
    CSwordship3: State => 0x0e8,
    CSwordship4: State => 0x0e9,
    CKerosene: State => 0x0f1,
    CIgnition: Attack => 0x0f2,
    CTaiJi: State => 0x12d,
    CWeak: Summon => 0x12e,
    CGodBless: State => 0x12f,
    COrigin: State => 0x130,
    CCure: State => 0x131,
    CFireBolt: Summon => 0x132,
    CLightning: Attack => 0x133,
    CFireWall: Summon => 0x134,
    CInfernol: Attack => 0x135,
    CSevenShootingStar: Attack => 0x136,
    CChaosSphere: Summon => 0x137,
    CSeal: Attack => 0x138,
    CYinYang: Summon => 0x139,
    CGodPunishment: Summon => 0x13a,
    CSoulCollect: State => 0x13b,
    CSoulMirror: Attack => 0x13c,
    CFireBall: Summon => 0x13d,
    CChainLightning: Attack => 0x13e,
    CThunderBlow: Summon => 0x13f,
    CGodThunder: Summon => 0x140,
    CManaShield: State => 0x141,
    CPromotion: State => 0x142,
    CGodThunder2: Summon => 0x143,
    CHearten: State => 0x144,
    CGodBless2: State => 0x145,
    CYinYang2: Summon => 0x146,
    CThunderBlow2: Attack => 0x14d,
    CSpiderPoison: State => 0x191,
    CKnockOut: State => 0x192,
    CSnowStorm: Summon => 0x193,
    CCorpseCandleBlasting: Attack => 0x194,
    CSporeBlasting: State => 0x195,
    CYakshaSlash: Attack => 0x196,
    CMonsterThorn: Attack => 0x197,
    CSpiderMist: Summon => 0x198,
    CSpiderWeb: State => 0x199,
    CSummonCorpseCandle: Summon => 0x19a,
    CSummonSkeleton: Summon => 0x19b,
    CSummonSpore: Summon => 0x19c,
    CChuckStone: Attack => 0x19d,
    CYunShengLightning: Attack => 0x19e,
    CCorpsePtomaine: State => 0x19f,
    CEnergyBolt: Attack => 0x1a0,
    CSkeletonArchery: Attack => 0x1a1,
    CZombieClaw: Attack => 0x1a2,
    CFury: State => 0x1a3,
    CLittleStar: Attack => 0x1a4,
    CSnakeBolt: Attack => 0x1a5,
    CSpriteBurn: State => 0x1a6,
    CMachineryStomp: Attack => 0x1a7,
    CLordFastAttack: Attack => 0x1f5,
    CLordWiderangingAttack: Attack => 0x1f6,
    CBossBlueFury: State => 0x1f7,
    CBossBlueQuake: Attack => 0x1f8,
    CBossFiendSummon: Summon => 0x1f9,
    CBossFiendPenetrate: Attack => 0x1fa,
    CPojia: State => 0x212,
    CPobing: State => 0x213,
    CPomo: State => 0x214,
    CPofa: State => 0x215,
    CYujia: State => 0x216,
    CYubing: State => 0x217,
    CYumo: State => 0x218,
    CYufa: State => 0x219,
    CTianhuo: Summon => 0x21a,
    CLeiming2: Summon => 0x21b,
    CFatalBlow: Attack => 0x21c,
    CBloodLoss: State => 0x21d,
    CPoisonArrow: State => 0x21e,
    CThunder: Summon => 0x21f,
    CLifeShield: State => 0x220,
    CWangsheng: State => 0x221,
    CHuoxieshu: State => 0x222,
    CLingzhishu: State => 0x223,
    BFBaseAttack: Summon => 0x224,
    CEnlargeMaxHp: State => 0x259,
    CEnlargeMaxMp: State => 0x25a,
    CEnlargeFullMiss: State => 0x25b,
    CMonsterBaseAttack: Attack => 0x2bd,
    CMonsterFastAttack: Attack => 0x2d1,
    CMonsterRangeAttack: Attack => 0x2ef,
    #[allow(non_camel_case_types)]
    CItemSkill_2: Summon => 0x322,
    CWuXingMetal: State => 0x353,
    CWuXingWood: State => 0x354,
    CWuXingWater: State => 0x355,
    CWuXingFire: State => 0x356,
    CWuXingEarth: State => 0x357,
    CNonFun: Attack => 0x384,
    CNonFun1: Attack => 0x385,
    CNonFun2: Attack => 0x386,
    CNonFun3: Attack => 0x387,
    CNonFun4: Attack => 0x388,
    CNonFun5: Attack => 0x389,
    CNonFun6: Attack => 0x38a,
    CNonFun7: Attack => 0x38b,
    CNonFun8: Attack => 0x38c,
    CNonFun9: Attack => 0x38d,
    CNonFun10: Attack => 0x38e,
    CNonFun11: Attack => 0x38f,
    CNonFun12: Attack => 0x390,
    CNonFun13: Attack => 0x391,
    CNonFun14: Attack => 0x392,
    CNonFun15: Attack => 0x393,
    CNonFun16: Attack => 0x394,
    CNonFun17: Attack => 0x395,
    CNonFun18: Attack => 0x396,
    CNonFun19: Attack => 0x397,
    CNonFun20: Attack => 0x398,
    CNonFun21: Attack => 0x399,
    CNonFun22: Attack => 0x39a,
    CNonFun23: Attack => 0x39b,
    CNonFun24: Attack => 0x39c,
    CNonFun25: Attack => 0x39d,
    CNonFun26: Attack => 0x39e,
    CNonFun27: Attack => 0x39f,
    CNonFun28: Attack => 0x3a0,
    CNonFun29: Attack => 0x3a1,
    CNonFun30: Attack => 0x3a2,
    CNonFun31: Attack => 0x3a3,
    CNonFun32: Attack => 0x3a4,
    CNonFun33: Attack => 0x3a5,
    CNonFun34: Attack => 0x3a6,
    CNonFun35: Attack => 0x3a7,
    CNonFun36: Attack => 0x3a8,
    CNonFun37: Attack => 0x3a9,
    CNonFun38: Attack => 0x3aa,
    CNonFun39: Attack => 0x3ab,
    CNonFun40: Attack => 0x3ac,
    CNonFun41: Attack => 0x3ad,
    CNonFun42: Attack => 0x3ae,
    CNonFun43: Attack => 0x3af,
    CNonFun44: Attack => 0x3b0,
    CNonFun45: Attack => 0x3b1,
    CNonFun46: Attack => 0x3b2,
    CNonFun60: Attack => 0x3c0,
    CNonFun61: Attack => 0x3c1,
    CNonFun62: Attack => 0x3c2,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum SkillFactoryDecodeError {
    #[error("skill snapshot обрывается на {field} в {offset}: нужно {required}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    #[error("skill snapshot содержит отрицательное число slots {count}")]
    NegativeSlotCount {
        count: i32,
    },
    #[error("skill slot {slot} содержит имя длиной {length} при максимуме {MAX_SKILL_NAME_LENGTH}")]
    NameTooLong {
        slot: usize,
        length: usize,
    },
    #[error("skill slot {slot} содержит {length} байт, требуется {required}")]
    RecordTooShort {
        slot: usize,
        length: usize,
        required: usize,
    },
    #[error("skill slot {slot} содержит непредставимое число usage-записей {count}")]
    UsageTableTooLarge {
        slot: usize,
        count: u32,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSkillFactory {
    properties: BTreeMap<u32, CSkillBaseProperties>,
}

impl CSkillFactory {
    pub(crate) const fn properties(&self) -> &BTreeMap<u32, CSkillBaseProperties> {
        &self.properties
    }

    pub(crate) fn clear_skill_cache(&mut self) {
        self.properties.clear();
    }

    pub(crate) fn rebuild(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), SkillFactoryDecodeError> {
        self.clear_skill_cache();
        let count = read_i32(source, cursor, "slot count")?;
        if count < 0 {
            return Err(SkillFactoryDecodeError::NegativeSlotCount { count });
        }

        let declared_slots = count as usize;
        let mut empty_slots = 0;
        let mut skipped_records = 0;
        for slot in 0..declared_slots {
            let length = read_u32(source, cursor, "record length")? as usize;
            if length == 0 {
                empty_slots += 1;
                continue;
            }
            let record = take_bytes(source, cursor, length, "skill record")?;
            let Some((key, properties)) = decode_record(record, slot)? else {
                skipped_records += 1;
                continue;
            };
            self.properties.insert(key, properties);
        }

        tracing::trace!(declared_slots, published_records = self.properties.len(), empty_slots, skipped_records, "реестр навыков декодирован");
        Ok(())
    }

    pub(crate) fn query_skill_base_properties(
        &self,
        skill_id: u32,
        level: i32,
    ) -> Option<&CSkillBaseProperties> {
        self.properties.get(&skill_key(skill_id, level as u32))
    }

    pub(crate) fn query_skill_type(&self, skill_id: u32, level: i32) -> u32 {
        self.query_skill_base_properties(skill_id, level)
            .map(CSkillBaseProperties::skill_type)
            .unwrap_or(UNKNOWN_SKILL_TYPE)
    }

    /// Точная null/non-null граница `CSkillFactory::QuerySkill`. Уровень
    /// влияет на `SetSourceObjectAmount`, но не на выбор concrete owner-а.
    pub(crate) const fn supports_skill_id(skill_id: u32) -> bool {
        Self::factory_owner(skill_id).is_some()
    }

    pub(crate) fn query_skill_id(&self, name: Option<&[u8]>) -> u32 {
        let Some(name) = name.map(visible_c_string) else {
            return UNKNOWN_SKILL_ID;
        };
        self.properties
            .iter()
            .find_map(|(&key, properties)| (properties.skill_name() == name).then_some(key >> 16))
            .unwrap_or(UNKNOWN_SKILL_ID)
    }

    pub(crate) fn query_skill_name(&self, skill_id: i32) -> Option<&[u8]> {
        self.properties.iter().find_map(|(&key, properties)| {
            ((key >> 16) == skill_id as u32).then_some(properties.skill_name())
        })
    }

    pub(crate) const fn get_skill_failed_message_color() -> u32 {
        0xffff_0000
    }

    pub(crate) const fn is_war_soul_skill(skill_id: u32) -> bool {
        skill_id > 0x211 && skill_id < 0x225
    }

    pub(crate) const fn is_need_float(skill_id: u32) -> bool {
        (skill_id > 0x211 && skill_id < 0x222) || skill_id == 0x224
    }
}

fn decode_record(
    record: &[u8],
    slot: usize,
) -> Result<Option<(u32, CSkillBaseProperties)>, SkillFactoryDecodeError> {
    const FIXED_PREFIX: usize = 20;
    if record.len() < FIXED_PREFIX {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required: FIXED_PREFIX,
        });
    }

    let mut reader = LegacyReader::new(record);
    let skill_type = reader.read_u32().expect("проверен префикс записи");
    let skill_id = reader.read_u32().expect("проверен префикс записи");
    let level = reader.read_u32().expect("проверен префикс записи");
    let is_target_self = reader.read_i32().expect("проверен префикс записи");
    let name_length = reader.read_u32().expect("проверен префикс записи") as usize;
    if name_length > MAX_SKILL_NAME_LENGTH {
        return Err(SkillFactoryDecodeError::NameTooLong {
            slot,
            length: name_length,
        });
    }
    let usage_count_offset =
        FIXED_PREFIX
            .checked_add(name_length)
            .ok_or(SkillFactoryDecodeError::RecordTooShort {
                slot,
                length: record.len(),
                required: usize::MAX,
            })?;
    let usage_data_offset =
        usage_count_offset
            .checked_add(4)
            .ok_or(SkillFactoryDecodeError::RecordTooShort {
                slot,
                length: record.len(),
                required: usize::MAX,
            })?;
    if record.len() < usage_data_offset {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required: usage_data_offset,
        });
    }

    let name = visible_c_string(
        reader
            .read_bytes(name_length)
            .expect("проверена длина имени"),
    )
    .to_vec();
    let usage_count = reader.read_u32().expect("проверено число usage-записей");
    if usage_count != 0 && usage_data_offset >= record.len() {
        return Ok(None);
    }
    let usage_bytes = usize::try_from(usage_count)
        .ok()
        .and_then(|count| count.checked_mul(8))
        .ok_or(SkillFactoryDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        })?;
    let required = usage_data_offset.checked_add(usage_bytes).ok_or(
        SkillFactoryDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        },
    )?;
    if record.len() < required {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required,
        });
    }

    let mut properties = CSkillBaseProperties::new(skill_type, is_target_self, name);
    for _ in 0..usage_count {
        let usage = reader.read_u32().expect("проверена usage-пара");
        let value = reader.read_u32().expect("проверена usage-пара");
        properties.set_property(usage, value);
    }
    Ok(Some((skill_key(skill_id, level), properties)))
}

const fn skill_key(skill_id: u32, level: u32) -> u32 {
    skill_id.wrapping_shl(16) | (level & 0xffff)
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, SkillFactoryDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, SkillFactoryDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader.read_u32().map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn take_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], SkillFactoryDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, required)?;
    let bytes = reader
        .read_bytes(required)
        .map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn skill_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    required: usize,
) -> Result<LegacyReader<'source>, SkillFactoryDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| SkillFactoryDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required,
        available: block.available,
    })
}

fn skill_read_error(
    field: &'static str,
    block: super::super::legacycodec::LegacyReadBlock,
) -> SkillFactoryDecodeError {
    SkillFactoryDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required: block.needed,
        available: block.available,
    }
}
