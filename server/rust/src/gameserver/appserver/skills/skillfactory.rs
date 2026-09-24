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
//! End-политики всех 209 владельцев подтверждены slot `+0x68` их vtable:
//! найдено 37 общих тел End(int), без подстановки категории runtime-свойств.
//! Только общий хвост: `50E9B0`, `5A30D0`, `5AC2D0`, `5AFA40`, `5B3010`,
//! `5DFBD0`, `601A40`; чтение GetUser в двух из этих тел не меняет движение.
//! Возврат движения источнику: `52B410`, `53BF50`, `540890`, `546090`,
//! `54D760`, `55B2B0`, `56A330`, `577C40`, `57B810`, `582810`, `58C3A0`,
//! `58DE90`, `58F5E0`, `591AD0`, `598EE0`, `59A410`, `59D6B0`, `5AE7A0`.
//! `570B20` освобождает пути после возврата движения, остальные path-owner-ы
//! делают это до него. `5502F0` при null GetUser пробует GetSufferer.
//! `512B50` и `5970C0` также ставят available=true; `544830` — false.
//! `5888D0` ставит available=true и обновляет visual действием 3 только при
//! End(0). Особые visual-хвосты: `516FB0`, `51A700`, `51E370`, `5222A0`,
//! `5355F0`, `5A0790`; их точные условия описаны в SkillEndEffect.
//! End не проверяет ended: constructor/failed Begin не разрешают пропустить
//! source-aware действия. Null source не заменяется держателем реестра.
//! Наличие owned visual независимо от concrete payload; политика не создаёт
//! эффект и не разрешает пакет только по факту регистрации навыка.
//! SpiderMist возвращает движение перед CSummonSkill::End. Его Begin не
//! добавляет cast в m_vStates; End не удаляет самостоятельную phalanx.
//! У Po/Yu и transfer-owner-ов End(bool) в `+0x94` не подменяет End(int)
//! в `+0x68`. Удерживаемый HeartLessArrow при ненулевом End имеет отдельный
//! ранний release-переход; эта карта не заменяет проверку его concrete-флагов.
//! OnChangeRegion (+0x2C) всех concrete CSkill-derived vtable не присваивает
//! регионы базы. Большинство используют ret4 0x00601A70; overrides по
//! 0x0056A370/0x0052AAC0 вызывают End(0)/End(1), а 0x0053CF10 проверяет
//! IsEnded и вызывает End(0) только при false. Эта политика хранится здесь
//! вместе с тем же owner, не в отдельной таблице идентификаторов.
//! Клиентские virtual getters +0x70/+0x74/+0x78: minimum range обычно 1;
//! ChuckStone/SkeletonArchery 0x005387A0 читают usage5004 и принимают только
//! signed-положительный результат, иначе 1. Общие maximum range0x004D81C0
//! и MP cost0x004D81F0 читают свежие properties usage5003/2; отсутствующая
//! запись даёт соответственно 1/0. Это getters, не сырой triplet usages.
//! Attack/State AfterUse (`+0x8C`) и Summon AfterUse (`+0x90`) вызываются
//! только при ненулевом End перед общим сбросом. `53CF30` изнашивает оружие
//! разрешённого CPlayer; пять пустых overrides используют `601A70`.
//! `CItemSkill_2` (`5149E0`) вместо износа записывает время item-группы из
//! текущих properties ID/level (`usage 0xC351`); Defense не имеет AfterUse.
//! Каталог не выполняет End: concrete поля/ресурсы, вызовы visual и общая
//! source-aware граница остаются обязанностью runtime-владельцев. Эта карта
//! не объявляет восстановленными неизвестные concrete layout и полный End.
//! `BTreeMap`, `Vec` и `Drop` заменяют служебный код MSVC map/heap. Старые
//! переполнения и выходы за границу при повреждённой длине остановлены
//! типизированной ошибкой без дополнительных side effects. Неизвестности
//! concrete layout и lifecycle ещё не восстановленных skill-owner-ов этот
//! каталог не закрывает; выбор класса, его категория и операции реестра
//! свойств уже исполняются в Rust.

use std::collections::BTreeMap;

pub(crate) use nebokrai_zone::content::CSkillBaseProperties;

pub(crate) use nebokrai_zone::combat::UNKNOWN_SKILL_ID;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillEndMovement {
    None,
    User,
    UserOrSufferer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillEndPathOrder {
    BeforeMovement,
    AfterMovement,
}

/// Дополнительные действия конкретного End до общего CSkill::End.
/// Любая visual-ветвь требует реально существующего owned visual; отсутствие
/// concrete execution само по себе ничего не говорит о наличии этого ресурса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillEndEffect {
    None,
    /// После возврата движения: OnChangeStates у разрешённого CPlayer,
    /// затем UpdateVisualEffect(skill, 3), если visual существует.
    Rage,
    /// UpdateVisualEffect(skill, 3) только при нулевом аргументе End.
    ScorpionOnlyZero,
    /// После освобождения пути и возврата движения — visual action 3.
    Star,
    /// При разрешённом GetUser: BeginVisualEffect(1), затем visual action 3.
    /// Это не EndVisualEffect: первый вызов идёт через slot 0, не slot 4.
    BattleFairyBaseMagic,
    /// При существующих visual и GetUser: BeginVisualEffect(1), затем action 3.
    BattleFairyState,
    /// Visual action 3 без дополнительного source-gate в самом End.
    BattleFairyFatal,
    /// Visual action 3 без дополнительного source-gate в самом End.
    BattleFairySummon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillEndPolicy {
    pub(crate) movement: SkillEndMovement,
    /// Сбрасывает concrete condition/active через фазу до возврата движения.
    /// Базовая возможность прерывания (CAN_BE_BREAKED) при этом сохраняется.
    pub(crate) reset_phase: bool,
    /// None сохраняет available; Some задаёт безусловную concrete-запись.
    pub(crate) available: Option<bool>,
    /// Concrete CAN меняется до GetUser/Move1, в том числе без payload.
    pub(crate) available_before_movement: bool,
    pub(crate) effect: SkillEndEffect,
    /// Порядок освобождения существующих путей, а не создание пустого payload.
    pub(crate) path_order: SkillEndPathOrder,
}

impl SkillEndPolicy {
    const COMMON: Self = Self {
        movement: SkillEndMovement::None,
        reset_phase: false,
        available: None,
        available_before_movement: false,
        effect: SkillEndEffect::None,
        path_order: SkillEndPathOrder::BeforeMovement,
    };
    const USER: Self = Self { movement: SkillEndMovement::User, ..Self::COMMON };
    const COMMON_RESET_PHASE: Self = Self { reset_phase: true, ..Self::COMMON };
    const USER_RESET_PHASE: Self = Self { reset_phase: true, ..Self::USER };
    const USER_OR_SUFFERER: Self = Self { movement: SkillEndMovement::UserOrSufferer, ..Self::COMMON };
    const USER_OR_SUFFERER_RESET_PHASE: Self = Self { reset_phase: true, ..Self::USER_OR_SUFFERER };
    const USER_AVAILABLE: Self = Self { available: Some(true), ..Self::USER };
    const IGNITION: Self = Self {
        available: Some(false), available_before_movement: true, ..Self::USER_RESET_PHASE
    };
    const USER_PATHS_AFTER_MOVEMENT: Self = Self { path_order: SkillEndPathOrder::AfterMovement, ..Self::USER_RESET_PHASE };
    const RAGE: Self = Self { effect: SkillEndEffect::Rage, ..Self::USER };
    const SCORPION: Self = Self {
        reset_phase: true, available_before_movement: true,
        effect: SkillEndEffect::ScorpionOnlyZero, ..Self::USER_AVAILABLE
    };
    const STAR: Self = Self { effect: SkillEndEffect::Star, ..Self::USER };
    const BATTLE_FAIRY_BASE_MAGIC: Self = Self { effect: SkillEndEffect::BattleFairyBaseMagic, ..Self::COMMON };
    const BATTLE_FAIRY_STATE: Self = Self { effect: SkillEndEffect::BattleFairyState, ..Self::COMMON };
    const BATTLE_FAIRY_FATAL: Self = Self { effect: SkillEndEffect::BattleFairyFatal, ..Self::COMMON };
    const BATTLE_FAIRY_SUMMON: Self = Self { effect: SkillEndEffect::BattleFairySummon, ..Self::COMMON };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillAfterUse {
    Weapon,
    None,
    ItemGroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillRegionChange {
    None,
    EndZero,
    EndOne,
    EndZeroUnlessEnded,
}

macro_rules! skill_owners {
    (@region_policy) => { SkillRegionChange::None };
    (@region_policy $policy:ident) => { SkillRegionChange::$policy };
    (@minimum_range_usage) => { None };
    (@minimum_range_usage $usage:literal) => { Some($usage) };
    ($($(#[$attribute:meta])* $owner:ident: $category:ident, $end_policy:ident, $after_use:ident $(, $region_policy:ident $(, $minimum_range_usage:literal)?)? => $id:pat),+ $(,)?) => {
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

            pub(crate) const fn end_policy(self) -> SkillEndPolicy {
                match self {
                    $(Self::$owner => SkillEndPolicy::$end_policy,)+
                }
            }

            pub(crate) const fn after_use(self) -> SkillAfterUse {
                match self {
                    $(Self::$owner => SkillAfterUse::$after_use,)+
                }
            }

            pub(crate) const fn region_change(self) -> SkillRegionChange {
                match self {
                    $(Self::$owner => skill_owners!(@region_policy $($region_policy)?),)+
                }
            }

            pub(crate) const fn minimum_range_usage(self) -> Option<u32> {
                match self {
                    $(Self::$owner => skill_owners!(@minimum_range_usage $($($minimum_range_usage)?)?),)+
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
    CBaseAttack: Attack, COMMON, Weapon, EndZero => 0x001,
    CArchery: Summon, USER_RESET_PHASE, Weapon => 0x002,
    CBaseMagic: Summon, USER_RESET_PHASE, Weapon => 0x003,
    CFightDefense: Defense, COMMON, None => 0x00a,
    CMosou: Attack, USER_RESET_PHASE, Weapon => 0x065,
    CGhostCut: Attack, USER_RESET_PHASE, Weapon, EndZero => 0x066,
    CKnightCut: State, USER_RESET_PHASE, Weapon => 0x067,
    CArmyBreak: Attack, USER_RESET_PHASE, Weapon => 0x068,
    CFlash: Attack, USER_RESET_PHASE, Weapon => 0x069,
    CSwallow: Attack, USER_RESET_PHASE, Weapon => 0x06a,
    CLeafCut: State, USER_RESET_PHASE, Weapon => 0x06b,
    CJuCut: Attack, USER_RESET_PHASE, Weapon => 0x06c,
    CRage: State, RAGE, Weapon => 0x06d,
    CRageBreak: State, USER_RESET_PHASE, Weapon => 0x06e,
    CSwordship: State, COMMON_RESET_PHASE, Weapon => 0x06f,
    CLightningSword: Attack, USER_RESET_PHASE, Weapon => 0x070,
    CLittleFlash: Attack, USER_RESET_PHASE, Weapon => 0x071,
    CThunderSlash: Summon, USER_RESET_PHASE, Weapon => 0x072,
    CRush: State, USER_PATHS_AFTER_MOVEMENT, Weapon => 0x073,
    CPillar: State, USER_RESET_PHASE, Weapon => 0x074,
    CCallosity: State, USER_RESET_PHASE, Weapon => 0x075,
    CBlind: State, USER, Weapon => 0x076,
    CLightningSword2: Attack, USER_RESET_PHASE, Weapon => 0x077,
    CLightningSword3: Attack, USER_RESET_PHASE, Weapon => 0x078,
    CGhostCut2: Attack, USER_RESET_PHASE, Weapon, EndZero => 0x079,
    CGhostCut3: Attack, USER_RESET_PHASE, Weapon, EndZero => 0x07a,
    CArmyBreak2: Attack, USER_RESET_PHASE, Weapon => 0x07b,
    CRush2: State, USER_PATHS_AFTER_MOVEMENT, Weapon => 0x07c,
    CCallosity2: State, USER_RESET_PHASE, Weapon => 0x07d,
    CLightningSword4: Attack, USER_RESET_PHASE, Weapon => 0x07e,
    CLittleFlash2: Attack, USER_RESET_PHASE, Weapon => 0x07f,
    CLeafCut2: State, USER_RESET_PHASE, Weapon => 0x080,
    CAgility2: State, USER_RESET_PHASE, Weapon => 0x081,
    CRoar: State, USER_RESET_PHASE, Weapon => 0x083,
    CEnergyHolding: State, USER_OR_SUFFERER_RESET_PHASE, Weapon => 0x089,
    CInverseChopped: Attack, USER_RESET_PHASE, Weapon => 0x08a,
    CLeafCut3: State, USER_RESET_PHASE, Weapon => 0x08f,
    CPoisonFog: Summon, USER_RESET_PHASE, Weapon => 0x0c9,
    CHeartLessArrow: Attack, USER, Weapon => 0x0ca,
    CLightingArrow: Attack, USER_RESET_PHASE, Weapon, EndOne => 0x0cb,
    CMeteorArrowMass: State, USER_OR_SUFFERER_RESET_PHASE, Weapon => 0x0cc,
    CMeteorArrow: Summon, USER_RESET_PHASE, Weapon => 0x0cd,
    CRainArrow: Attack, USER_RESET_PHASE, Weapon, EndOne => 0x0ce,
    CPoisonMoth: Attack, USER_RESET_PHASE, Weapon, EndOne => 0x0cf,
    CBloodRose: Attack, USER_RESET_PHASE, Weapon => 0x0d0,
    CScorpion: Attack, SCORPION, Weapon => 0x0d1,
    CBoaLock: State, USER_RESET_PHASE, Weapon => 0x0d2,
    CHeal: State, USER_RESET_PHASE, Weapon, EndZero => 0x0d3,
    CMonsterTaming: Attack, USER, None => 0x0d4,
    CFallingStar: Summon, USER_RESET_PHASE, Weapon => 0x0d5,
    CExplosiveArrow: Attack, USER_RESET_PHASE, Weapon => 0x0d6,
    CPetsControl: Attack, USER, None => 0x0d7,
    CGibe: Attack, COMMON, None => 0x0d8,
    CSuperHeal: State, USER_RESET_PHASE, Weapon, EndZero => 0x0d9,
    CAgility: State, USER_RESET_PHASE, Weapon => 0x0da,
    CRapture: State, USER_RESET_PHASE, Weapon => 0x0db,
    CNatural: State, USER_RESET_PHASE, Weapon => 0x0dc,
    CStrike: Attack, USER_RESET_PHASE, Weapon, EndZero => 0x0dd,
    CMachineShield: State, USER_RESET_PHASE, Weapon => 0x0de,
    CDaubPoison: State, USER_RESET_PHASE, Weapon => 0x0df,
    CSwordship2: State, COMMON_RESET_PHASE, Weapon => 0x0e0,
    CExplosiveArrow2: Attack, USER_RESET_PHASE, Weapon => 0x0e1,
    CExplosiveArrow3: Attack, USER_RESET_PHASE, Weapon => 0x0e2,
    CHeal2: State, USER_RESET_PHASE, Weapon, EndZero => 0x0e3,
    CSuperHeal2: State, USER_RESET_PHASE, Weapon, EndZero => 0x0e4,
    CHeartLessArrow2: Attack, USER_RESET_PHASE, Weapon => 0x0e5,
    CHeartLessArrow3: Attack, USER_RESET_PHASE, Weapon => 0x0e6,
    CLightingArrow2: Attack, USER_RESET_PHASE, Weapon, EndOne => 0x0e7,
    CSwordship3: State, COMMON_RESET_PHASE, Weapon => 0x0e8,
    CSwordship4: State, COMMON_RESET_PHASE, Weapon => 0x0e9,
    CKerosene: State, USER_RESET_PHASE, Weapon => 0x0f1,
    CIgnition: Attack, IGNITION, Weapon => 0x0f2,
    CTaiJi: State, COMMON_RESET_PHASE, Weapon => 0x12d,
    CWeak: Summon, USER_RESET_PHASE, Weapon => 0x12e,
    CGodBless: State, USER_OR_SUFFERER_RESET_PHASE, Weapon => 0x12f,
    COrigin: State, COMMON_RESET_PHASE, Weapon => 0x130,
    CCure: State, USER_RESET_PHASE, Weapon => 0x131,
    CFireBolt: Summon, USER_RESET_PHASE, Weapon => 0x132,
    CLightning: Attack, COMMON_RESET_PHASE, Weapon => 0x133,
    CFireWall: Summon, USER_RESET_PHASE, Weapon => 0x134,
    CInfernol: Attack, USER_RESET_PHASE, Weapon => 0x135,
    CSevenShootingStar: Attack, STAR, Weapon => 0x136,
    CChaosSphere: Summon, USER_RESET_PHASE, Weapon => 0x137,
    CSeal: Attack, USER_RESET_PHASE, Weapon => 0x138,
    CYinYang: Summon, USER_RESET_PHASE, Weapon => 0x139,
    CGodPunishment: Summon, COMMON_RESET_PHASE, Weapon => 0x13a,
    CSoulCollect: State, USER_OR_SUFFERER_RESET_PHASE, Weapon => 0x13b,
    CSoulMirror: Attack, USER_RESET_PHASE, Weapon => 0x13c,
    CFireBall: Summon, USER_RESET_PHASE, Weapon => 0x13d,
    CChainLightning: Attack, USER_RESET_PHASE, Weapon => 0x13e,
    CThunderBlow: Summon, COMMON, Weapon => 0x13f,
    CGodThunder: Summon, USER_RESET_PHASE, Weapon => 0x140,
    CManaShield: State, USER_RESET_PHASE, Weapon => 0x141,
    CPromotion: State, USER_RESET_PHASE, Weapon => 0x142,
    CGodThunder2: Summon, USER_RESET_PHASE, Weapon => 0x143,
    CHearten: State, USER_RESET_PHASE, Weapon => 0x144,
    CGodBless2: State, USER_OR_SUFFERER_RESET_PHASE, Weapon => 0x145,
    CYinYang2: Summon, USER_RESET_PHASE, Weapon => 0x146,
    CThunderBlow2: Attack, COMMON_RESET_PHASE, Weapon => 0x14d,
    CSpiderPoison: State, USER_RESET_PHASE, Weapon => 0x191,
    CKnockOut: State, USER_RESET_PHASE, Weapon => 0x192,
    CSnowStorm: Summon, USER_RESET_PHASE, Weapon => 0x193,
    CCorpseCandleBlasting: Attack, USER, Weapon => 0x194,
    CSporeBlasting: State, USER, Weapon => 0x195,
    CYakshaSlash: Attack, USER_RESET_PHASE, Weapon => 0x196,
    CMonsterThorn: Attack, USER, Weapon => 0x197,
    CSpiderMist: Summon, USER, Weapon => 0x198,
    CSpiderWeb: State, USER_RESET_PHASE, Weapon => 0x199,
    CSummonCorpseCandle: Summon, USER, Weapon => 0x19a,
    CSummonSkeleton: Summon, USER, Weapon => 0x19b,
    CSummonSpore: Summon, USER, Weapon => 0x19c,
    CChuckStone: Attack, USER_RESET_PHASE, Weapon, EndZeroUnlessEnded, 5004 => 0x19d,
    CYunShengLightning: Attack, USER, Weapon => 0x19e,
    CCorpsePtomaine: State, USER, None => 0x19f,
    CEnergyBolt: Attack, USER_RESET_PHASE, Weapon => 0x1a0,
    CSkeletonArchery: Attack, USER_RESET_PHASE, Weapon, EndZeroUnlessEnded, 5004 => 0x1a1,
    CZombieClaw: Attack, USER_RESET_PHASE, Weapon => 0x1a2,
    CFury: State, USER_RESET_PHASE, Weapon => 0x1a3,
    CLittleStar: Attack, STAR, Weapon => 0x1a4,
    CSnakeBolt: Attack, USER_RESET_PHASE, Weapon => 0x1a5,
    CSpriteBurn: State, USER, None => 0x1a6,
    CMachineryStomp: Attack, USER, Weapon => 0x1a7,
    CLordFastAttack: Attack, USER_AVAILABLE, Weapon => 0x1f5,
    CLordWiderangingAttack: Attack, USER, Weapon => 0x1f6,
    CBossBlueFury: State, USER, Weapon => 0x1f7,
    CBossBlueQuake: Attack, USER, Weapon => 0x1f8,
    CBossFiendSummon: Summon, USER, Weapon => 0x1f9,
    CBossFiendPenetrate: Attack, USER, Weapon, EndOne => 0x1fa,
    CPojia: State, COMMON, Weapon => 0x212,
    CPobing: State, COMMON, Weapon => 0x213,
    CPomo: State, COMMON, Weapon => 0x214,
    CPofa: State, COMMON, Weapon => 0x215,
    CYujia: State, COMMON, Weapon => 0x216,
    CYubing: State, COMMON, Weapon => 0x217,
    CYumo: State, COMMON, Weapon => 0x218,
    CYufa: State, COMMON, Weapon => 0x219,
    CTianhuo: Summon, BATTLE_FAIRY_SUMMON, Weapon => 0x21a,
    CLeiming2: Summon, BATTLE_FAIRY_SUMMON, Weapon => 0x21b,
    CFatalBlow: Attack, BATTLE_FAIRY_FATAL, Weapon => 0x21c,
    CBloodLoss: State, BATTLE_FAIRY_STATE, Weapon => 0x21d,
    CPoisonArrow: State, BATTLE_FAIRY_STATE, Weapon => 0x21e,
    CThunder: Summon, BATTLE_FAIRY_SUMMON, Weapon => 0x21f,
    CLifeShield: State, BATTLE_FAIRY_STATE, Weapon => 0x220,
    CWangsheng: State, COMMON, Weapon => 0x221,
    CHuoxieshu: State, COMMON, Weapon => 0x222,
    CLingzhishu: State, COMMON, Weapon => 0x223,
    BFBaseAttack: Summon, BATTLE_FAIRY_BASE_MAGIC, Weapon => 0x224,
    CEnlargeMaxHp: State, COMMON_RESET_PHASE, Weapon => 0x259,
    CEnlargeMaxMp: State, COMMON_RESET_PHASE, Weapon => 0x25a,
    CEnlargeFullMiss: State, COMMON_RESET_PHASE, Weapon => 0x25b,
    CMonsterBaseAttack: Attack, COMMON, Weapon, EndZero => 0x2bd,
    CMonsterFastAttack: Attack, USER_AVAILABLE, Weapon => 0x2d1,
    CMonsterRangeAttack: Attack, USER, Weapon => 0x2ef,
    #[allow(non_camel_case_types)]
    CItemSkill_2: Summon, USER, ItemGroup => 0x322,
    CWuXingMetal: State, COMMON_RESET_PHASE, Weapon => 0x353,
    CWuXingWood: State, COMMON_RESET_PHASE, Weapon => 0x354,
    CWuXingWater: State, COMMON_RESET_PHASE, Weapon => 0x355,
    CWuXingFire: State, COMMON_RESET_PHASE, Weapon => 0x356,
    CWuXingEarth: State, COMMON_RESET_PHASE, Weapon => 0x357,
    CNonFun: Attack, COMMON, Weapon => 0x384,
    CNonFun1: Attack, COMMON, Weapon => 0x385,
    CNonFun2: Attack, COMMON, Weapon => 0x386,
    CNonFun3: Attack, COMMON, Weapon => 0x387,
    CNonFun4: Attack, COMMON, Weapon => 0x388,
    CNonFun5: Attack, COMMON, Weapon => 0x389,
    CNonFun6: Attack, COMMON, Weapon => 0x38a,
    CNonFun7: Attack, COMMON, Weapon => 0x38b,
    CNonFun8: Attack, COMMON, Weapon => 0x38c,
    CNonFun9: Attack, COMMON, Weapon => 0x38d,
    CNonFun10: Attack, COMMON, Weapon => 0x38e,
    CNonFun11: Attack, COMMON, Weapon => 0x38f,
    CNonFun12: Attack, COMMON, Weapon => 0x390,
    CNonFun13: Attack, COMMON, Weapon => 0x391,
    CNonFun14: Attack, COMMON, Weapon => 0x392,
    CNonFun15: Attack, COMMON, Weapon => 0x393,
    CNonFun16: Attack, COMMON, Weapon => 0x394,
    CNonFun17: Attack, COMMON, Weapon => 0x395,
    CNonFun18: Attack, COMMON, Weapon => 0x396,
    CNonFun19: Attack, COMMON, Weapon => 0x397,
    CNonFun20: Attack, COMMON, Weapon => 0x398,
    CNonFun21: Attack, COMMON, Weapon => 0x399,
    CNonFun22: Attack, COMMON, Weapon => 0x39a,
    CNonFun23: Attack, COMMON, Weapon => 0x39b,
    CNonFun24: Attack, COMMON, Weapon => 0x39c,
    CNonFun25: Attack, COMMON, Weapon => 0x39d,
    CNonFun26: Attack, COMMON, Weapon => 0x39e,
    CNonFun27: Attack, COMMON, Weapon => 0x39f,
    CNonFun28: Attack, COMMON, Weapon => 0x3a0,
    CNonFun29: Attack, COMMON, Weapon => 0x3a1,
    CNonFun30: Attack, COMMON, Weapon => 0x3a2,
    CNonFun31: Attack, COMMON, Weapon => 0x3a3,
    CNonFun32: Attack, COMMON, Weapon => 0x3a4,
    CNonFun33: Attack, COMMON, Weapon => 0x3a5,
    CNonFun34: Attack, COMMON, Weapon => 0x3a6,
    CNonFun35: Attack, COMMON, Weapon => 0x3a7,
    CNonFun36: Attack, COMMON, Weapon => 0x3a8,
    CNonFun37: Attack, COMMON, Weapon => 0x3a9,
    CNonFun38: Attack, COMMON, Weapon => 0x3aa,
    CNonFun39: Attack, COMMON, Weapon => 0x3ab,
    CNonFun40: Attack, COMMON, Weapon => 0x3ac,
    CNonFun41: Attack, COMMON, Weapon => 0x3ad,
    CNonFun42: Attack, COMMON, Weapon => 0x3ae,
    CNonFun43: Attack, COMMON, Weapon => 0x3af,
    CNonFun44: Attack, COMMON, Weapon => 0x3b0,
    CNonFun45: Attack, COMMON, Weapon => 0x3b1,
    CNonFun46: Attack, COMMON, Weapon => 0x3b2,
    CNonFun60: Attack, COMMON, Weapon => 0x3c0,
    CNonFun61: Attack, COMMON, Weapon => 0x3c1,
    CNonFun62: Attack, COMMON, Weapon => 0x3c2,
}

pub(crate) use nebokrai_zone::content::SkillPropertiesDecodeError as SkillFactoryDecodeError;


pub(crate) use nebokrai_zone::content::SkillPropertiesCatalog as SkillPropertiesCatalogInternal;

/// Переходная обёртка прежнего имени: добирает остальные API поверх Zone.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSkillFactory {
    catalog: SkillPropertiesCatalogInternal,
}

impl CSkillFactory {
    pub(crate) const fn properties(&self) -> &BTreeMap<u32, CSkillBaseProperties> {
        self.catalog.properties()
    }

    pub(crate) fn clear_skill_cache(&mut self) {
        self.catalog.clear();
    }

    pub(crate) fn rebuild(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), SkillFactoryDecodeError> {
        self.catalog.rebuild(source, cursor)
    }

    pub(crate) fn query_skill_base_properties(
        &self,
        skill_id: u32,
        level: i32,
    ) -> Option<&CSkillBaseProperties> {
        self.catalog.query_skill_base_properties(skill_id, level)
    }

    pub(crate) fn query_skill_type(&self, skill_id: u32, level: i32) -> u32 {
        self.catalog.query_skill_type(skill_id, level)
    }

    /// Точная null/non-null граница `CSkillFactory::QuerySkill`. Уровень
    /// влияет на `SetSourceObjectAmount`, но не на выбор concrete owner-а.
    pub(crate) const fn supports_skill_id(skill_id: u32) -> bool {
        Self::factory_owner(skill_id).is_some()
    }

    pub(crate) fn query_skill_id(&self, name: Option<&[u8]>) -> u32 {
        self.catalog.query_skill_id(name).unwrap_or(UNKNOWN_SKILL_ID)
    }

    pub(crate) fn query_skill_name(&self, skill_id: i32) -> Option<&[u8]> {
        self.catalog.query_skill_name(skill_id)
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

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}
