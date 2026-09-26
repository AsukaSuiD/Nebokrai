//! Фабрика владельцев ИИ монстра.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец
//! `e:\\svn\\fengyun_russia_dev\\server\\gameserver\\appserver\\ai\\aifactory.cpp`
//! подтверждают отображение `tagMonster::dwAI` на конкретный класс, запись
//! исходного AI type и отдельное создание `CPet`/`CCarriage`. Значения вне
//! специализированных case-ов, включая пропуски внутри диапазона `0..=104`,
//! создают обычный `CMonsterAI`, а не нулевой указатель. Rust не
//! воспроизводит C++-иерархию и ручное владение указателями: тот же выбор
//! хранится как типизированное состояние внутри единственного `CMonster`.
//! `SetOwner` тем самым выражен принадлежностью binding-а монстру, а повторный
//! `InitAI` заменяет все три прежних экземпляра; один лишь новый binding
//! недостаточен. CMonster::InitAI (0x004E6E10) проверяет property до delete,
//! затем создаёт primary, pet и carriage в этом порядке. Без property он
//! сохраняет старые экземпляры; Rust принимает уже разрешённый &property.
//! SetOwner (0x004C7C90) вызывает Clear до записи owner, но fresh constructor
//! обнуляет также WarSoul и dormancy timestamps, которые Clear сохраняет.
//! Донор Luvinia имеет только один m_pAI и обычный factory-type Pet; эта
//! техническая форма не подменяет три независимых владельца Miracle.
//!
//! Два `Catch@...` из EXE относятся только к сгенерированной MSVC очистке
//! временного `std::vector<CGUID>` и не являются семантикой фабрики.

use crate::setup::monsterlist::MonsterProperties;

use super::super::masterinfo::MasterInfo;

const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterAiKind {
    Gladiator,
    PassiveGladiator,
    SmartGladiator,
    StupidGladiator,
    Archer,
    FixedPositionArcher,
    StupidArcher,
    PuninessCreature,
    GuardWithBow,
    GuardWithSword,
    CityGuardWithSword,
    CityGuardWithBow,
    Carriage,
    GuardCountry,
    GuardCountry2,
    VillageCountyGuardWithSword,
    VillageCountyGuardWithBow,
    WarDefendMonster,
    WarAttackMonster,
    NationCountyGuardWithSword,
    NationGladiator,
    GodsBattleGuardWithSword,
    GodsBattleMonster,
    Lord,
    JiuMai,
    BossBlue,
    BossFiend,
    Monster,
}

impl MonsterAiKind {
    /// Общий пост и виртуальные методы семейства CGuardWithSword.
    pub(crate) const fn has_guard_station(self) -> bool {
        matches!(self, Self::GuardWithSword | Self::CityGuardWithSword
            | Self::VillageCountyGuardWithSword | Self::NationCountyGuardWithSword)
    }

    /// Точное отображение машинного реестра `CAIFactory::CreateAI` (RVA
    /// `0x1DC550`): case-отображение byte-map `0x5DCB08` + jump-table
    /// `0x5DCA94`, default-case `0x5DCA32` создаёт обычный `CMonsterAI`
    /// (включая 22, 25..=99, 102 и значения больше 104).
    pub(crate) const fn from_ai_type(ai_type: u32) -> Self {
        match ai_type {
            0 => Self::Gladiator,
            1 => Self::PassiveGladiator,
            2 => Self::SmartGladiator,
            3 => Self::StupidGladiator,
            4 => Self::Archer,
            5 => Self::FixedPositionArcher,
            6 => Self::StupidArcher,
            7 => Self::PuninessCreature,
            8 => Self::GuardWithBow,
            9 => Self::GuardWithSword,
            10 => Self::CityGuardWithSword,
            11 => Self::CityGuardWithBow,
            12 => Self::Carriage,
            13 | 20 => Self::GuardCountry,
            14 => Self::GuardCountry2,
            15 => Self::VillageCountyGuardWithSword,
            16 => Self::VillageCountyGuardWithBow,
            17 => Self::WarDefendMonster,
            18 => Self::WarAttackMonster,
            19 => Self::NationCountyGuardWithSword,
            21 => Self::NationGladiator,
            23 => Self::GodsBattleGuardWithSword,
            24 => Self::GodsBattleMonster,
            100 => Self::Lord,
            101 => Self::JiuMai,
            103 => Self::BossBlue,
            104 => Self::BossFiend,
            _ => Self::Monster,
        }
    }

    /// Соответствует default-ветви точного `CAIFactory::CreateAI`.
    pub(crate) const fn is_generic_ai_type(ai_type: u32) -> bool {
        matches!(Self::from_ai_type(ai_type), Self::Monster)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActiveMonsterAi {
    Primary(MonsterAiKind),
    Pet,
    Carriage,
}

impl ActiveMonsterAi {
    /// Три самостоятельных экземпляра CBaseAI из CMonster::InitAI.
    /// Тип первичного AI меняет виртуальные методы, но не его слот хранения.
    pub(crate) const fn storage_index(self) -> usize {
        match self {
            Self::Primary(_) => 0,
            Self::Pet => 1,
            Self::Carriage => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterAiBinding {
    primary: MonsterAiKind,
    ai_type: u32,
    pet: bool,
    carriage: bool,
}

impl MonsterAiBinding {
    /// Эквивалент `CreateAI`, `CreatePetAI` и `CreateCarriageAI` из одного
    /// `CMonster::InitAI`. Сравнение числа попыток сохраняет DWORD-семантику.
    pub(crate) const fn create(property: &MonsterProperties, tame_attempt_count: u32) -> Self {
        Self {
            primary: MonsterAiKind::from_ai_type(property.ai),
            ai_type: property.ai,
            pet: property.tamable == 1 && tame_attempt_count < property.maximum_tame_attempt_count,
            carriage: property.tamable == 1 && property.maximum_tame_attempt_count == 0,
        }
    }

    pub(crate) const fn primary(self) -> MonsterAiKind {
        self.primary
    }

    pub(crate) const fn ai_type(self) -> u32 {
        self.ai_type
    }

    pub(crate) const fn has_carriage(self) -> bool {
        self.carriage
    }

    pub(crate) const fn has_pet(self) -> bool {
        self.pet
    }

    /// Эквивалент `CMonster::GetAI`: до назначения валидного хозяина-игрока
    /// используется первичный AI, после — carriage либо pet. Отсутствующий
    /// вспомогательный владелец сохраняется как `None`, как нулевой C++ pointer.
    pub(crate) const fn active(self, master: MasterInfo) -> Option<ActiveMonsterAi> {
        if master.master_type != PLAYER_TYPE || master.master_id == 0 {
            return Some(ActiveMonsterAi::Primary(self.primary));
        }
        if self.carriage {
            Some(ActiveMonsterAi::Carriage)
        } else if self.pet {
            Some(ActiveMonsterAi::Pet)
        } else {
            None
        }
    }
}
