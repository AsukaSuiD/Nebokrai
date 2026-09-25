//! Оружейный roll семейства CalculateAttackPower: три живых вида ширины,
//! компоненты физического, элементного и душевного урона, живые property
//! источника по типу владельца и общий критический хвост. Старый серверный
//! пакет разрешает CGame/find_player/таблицы и передаёт живые поля
//! типизированным обратным вызовом `WeaponDamageLiveField`; RNG-состояние
//! и сам `random(int)` остаются у этого владельца.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2, совпадение подтверждено `.local/evidence/symbols.py identity`).
//! Исходные владельцы PDB — тела `CalculateAttackPower` в
//! `appserver/skills/chuckstone.cpp`, `skeletonarchery.cpp`, `yakshaslash.cpp`,
//! `ignition.cpp`, `scorpion.cpp`, `ghostcut*.cpp`, `strike.cpp`,
//! `lightingarrow2.cpp`, `poisonmoth.cpp`, `bloodrose.cpp`,
//! `explosivearrow*.cpp`, `heartlessarrow.cpp`, `heartlessarrowphalanx2.cpp/.3`,
//! `jucut.cpp`, `lightningsword*.cpp`, `inversechopped.cpp` и др.
//!
//! Живой vtable-канал тел: уровень цели `+0x110`, CCH `+0x114` (movzx WORD),
//! живой AddElementAtk `+0x118`, AddSoulAtk `+0x11C` (movzx WORD),
//! GetMinAttack `+0xE4`, GetMaxAttack `+0xE8`, weapon modifier `+0x184`;
//! живая ловкость CPlayer — `[+0x3B8]` через хелпер разрешения `0x619319`.
//! RNG — глобальный `?random@@YAHH@Z` (VA `0x41CBA0`; его внутренний
//! `rand` на адресе `0x61903F` повторяет msvc-формулу
//! `state = state * 214013 + 2531011`, `(state >> 16) & 0x7FFF`), выделение
//! компонента — ctor `tagAttackPower` `0x5D3C80`, push_back `0x4AF200/0x4AEFF0`,
//! критический множитель — изменяемая BSS-глобаль `fmul dword [0xEF3E5C]`
//! (читается владельцем и передаётся параметром `critical_rate`), константа
//! `0x64DBD0` = 0.01f.
//!
//! RawRange: VERIFIED_DISASSEMBLY по `CChuckStone::CalculateAttackPower`
//! (pub `1:0013c6e0`, истинный RVA `0x13D6E0`): чтения MIN (`[esi]+0xE4`) →
//! MAX (`+0xE8`), сырая DWORD-ширина `1 - min + max` (`mov ebx,1; sub; add`
//! — без abs и без нижней границы ширины), RNG, повторное MIN после RNG,
//! сумма и `jns`-нижняя граница нуля. Затем Element (`+0x118`) с
//! `jge`-нижней границей, Soul (`+0x11C`, movzx WORD) и общий критический
//! хвост. Тот же порядок зафиксирован шапками вызывающих у CStrike
//! (pub `1:00169eb0`) и CGhostCut (pub `1:0019c7e0`).
//!
//! AbsoluteRange: VERIFIED_DISASSEMBLY по `CYakshaSlash::CalculateAttackPower`
//! (pub `1:00141dc0`, RVA `0x142DC0`), контрольно `CJuCut::CalculateAttackPower`
//! (pub `1:00194060`, RVA `0x195060`) и `CInverseChopped::CalculateAttackPower`
//! (pub `1:00148230`, RVA `0x149230`): чтения MAX (`+0xE8`) → MIN (`+0xE4`),
//! ширина `abs(max - min) + 1` (`cdq; xor eax,edx; sub eax,edx` + `add eax,1`),
//! RNG, повторное MIN после RNG, сумма, `jns`-нижняя граница. Отличие от
//! RawRange — порядок первых двух чтений и abs ширины.
//!
//! CapturedMinimumAbsoluteRange: VERIFIED_DISASSEMBLY по
//! `CHeartLessArrow::CalculateAttackPower` (pub `1:001915f0`, RVA `0x1925F0`):
//! чтения MIN → MAX, та же abs-ширина с `+1`, RNG, но сумма с СОХРАНЁННЫМ
//! первым MIN — повторного чтения MIN нет; далее Element/Soul и живой CCH
//! (vtable `+0x114`, movzx WORD), как у остальных.
//!
//! Знаковый CCH снимка: VERIFIED_DISASSEMBLY по
//! `CHeartLessArrowPhalanx2::CalculateAttackPower` (pub `1:001ec910`, RVA
//! `0x1ED910`): hit = 0 и damage_factor = сохранённый factor × 0.01f до
//! компонентов, тот же captured-ролл, затем RNG(100) ПЕРВЫМ и знаковое
//! `jge`-сравнение с сохранённым DWORD `+0xC4` конструктора — повторного
//! чтения живого CCH нет. NULL источника тело допускает до SOUL (physical
//! становится RNG(1) → 0, element нулевым), затем разыменовывает NULL;
//! старый владелец отсекает этот случай до вызова формулы и не выдумывает
//! SOUL/CCH.
//!
//! Фронтальное усиление: VERIFIED_DISASSEMBLY. JuCut и InverseChopped после
//! суммы со вторым MIN добавляют живую ловкость `[player+0x3B8]`;
//! NULL-результат хелпера разрешения пропускает добавку без обрыва тела.
//! В Rust-переносе недостижимый обрыв оставлен намеренно: цепочка вызова
//! уже разрешила того же игрока выше (weapon modifier), поэтому
//! пропуск-и-продолжение и обрыв наблюдаемо совпадают. InverseChopped до
//! roll расходует первое состояние ID 0x89 (End+destroy) и умножает все ТРИ
//! компонента f64-множителем: `fild; fmul qword; fistp qword` с control
//! word `or ah,0xC` и записью МЛАДШЕГО DWORD (`truncate_original_i64_low`).
//!
//! Критический хвост: VERIFIED_DISASSEMBLY по ChuckStone (`0x53D888`),
//! HeartLessArrow (`0x59286B`) и HeartLessArrowPhalanx2 (`0x5EDAF3`):
//! фильтр видов компонентов `cmp [kind],1/3/4; jne skip`, затем
//! `fnstcw; fild; fmul dword [0xEF3E5C]; fldcw (or ah,0xC); fistp; fldcw`
//! только для Physical/Element/Soul. Прежний Rust-проход масштабировал
//! компоненты без фильтра; наблюдаемого различия не было, потому что все
//! вызывающие кладут только виды 1/3/4, — zone-форма фиксирует машинный
//! фильтр, симметричный `skills/projectile.rs::apply_projectile_critical`.
//!
//! source_property по типу владельца: PARTIAL (основание — vtable-контракты
//! тел выше и уже сверенные `CMonster`-формулы `combat/monsterformula`;
//! собственные тела getter-ов CPlayer этой порцией не пересверялись). Тип
//! 400 отдаёт боевые поля игрока; тип 600 — state-границы MIN/MAX и SOUL
//! монстра, а Element и CCH всегда 0: исходный `GetAddElementAtk` монстра
//! умножает pet factor на ноль, и даже нечисловой factor после native FISTP
//! с нижней границей даёт 0. Типы 500/1100/1200 (NPC/Build/CityGate) — 0;
//! прочие типы не участвуют.
//!
//! Мёртвая сырая ветвь `PlayerWeaponRoll::Archery` прежнего пакета сюда не
//! перенесена и удалена: callsites отсутствуют (аудит 2026-09 — «очистка
//! отложена на оружейную волну»), живой Archery-roll (`max(max-min, 0)` БЕЗ
//! +1, два чтения MIN до RNG) уже находится в `skills/projectile.rs` со
//! статусом VERIFIED_DISASSEMBLY. Отношение +1 по видам: RawRange и оба
//! abs-вида прибавляют единицу к ширине, Archery — нет.
//!
//! Контракт RNG при неположительной ширине принадлежит владельцу (старый
//! пакет `game_legacy_random` возвращает 0 без расхода состояния); машинное
//! `random(int)` при bound == 0 вызывает `rand` и возвращает 0 — наблюдение
//! к владельцу RNG, формула ширину передаёт дословно.

use super::{
    AttackInformation, AttackPower, AttackPowerType, truncate_original, truncate_original_i64_low,
};

/// Живые виды оружейного roll. Порядок чтений каждого вида и quirk-ширины —
/// в шапке модуля; мёртвый `Archery` прежнего пакета удалён (живой
/// перенос — `skills/projectile`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerWeaponRoll {
    AbsoluteRange,
    RawRange,
    CapturedMinimumAbsoluteRange,
}

/// Усиление фронтальных ударов: ловкость источника и/или расходованный
/// множитель EnergyHolding; вычисление самого множителя остаётся у владельца.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeaponPowerBoost {
    None,
    Dexterity,
    EnergyHolding(f64),
}

/// Живые поля источника и RNG оружейного roll. Разрешает их старый владелец
/// в порядке исходного CalculateAttackPower; сужения WORD у Soul и CCH и
/// типовой разворот Dexterity принадлежат формуле/подстановке этой схемы,
/// как у `skills/projectile.rs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaponDamageLiveField {
    RandomBelow(i32),
    MinimumAttack,
    MaximumAttack,
    AddElementAttack,
    AddSoulAttack,
    CriticalChance,
    Dexterity,
}

/// Живое property источника оружейного расчёта по типу владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaponSourceProperty {
    Minimum,
    Maximum,
    Element,
    Soul,
    CriticalChance,
}

/// Типизированный снимок боевых property источника. Для монстра MIN/MAX
/// приходят уже после `monsterformula::state_attack_bounds`, а SOUL — после
/// `monsterformula::soul_attack`; нулевые Element/CCH монстра и нули
/// NPC/Build/CityGate разрешает правило `weapon_source_property` без снимка.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WeaponSourceCombat {
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub add_element_attack: u32,
    pub add_soul_attack: u16,
    pub critical_chance: u16,
}

/// Исходное правило `source_property`: боевые поля игрока, state-шкалы и
/// нулевые Element/CCH монстра, нули NPC/Build/CityGate. Отсутствующий
/// снимок отменяет чтение, как исходный NULL найденного объекта.
pub fn weapon_source_property(
    owner_type: i32, property: WeaponSourceProperty, combat: Option<WeaponSourceCombat>,
) -> Option<u32> {
    match owner_type {
        400 => combat.map(|combat| match property {
            WeaponSourceProperty::Minimum => combat.minimum_attack,
            WeaponSourceProperty::Maximum => combat.maximum_attack,
            WeaponSourceProperty::Element => combat.add_element_attack,
            WeaponSourceProperty::Soul => u32::from(combat.add_soul_attack),
            WeaponSourceProperty::CriticalChance => u32::from(combat.critical_chance),
        }),
        600 => match property {
            WeaponSourceProperty::Element | WeaponSourceProperty::CriticalChance => Some(0),
            WeaponSourceProperty::Minimum => combat.map(|combat| combat.minimum_attack),
            WeaponSourceProperty::Maximum => combat.map(|combat| combat.maximum_attack),
            WeaponSourceProperty::Soul => combat.map(|combat| u32::from(combat.add_soul_attack)),
        },
        500 | 1_100 | 1_200 => Some(0),
        _ => None,
    }
}

/// Общий критический хвост: после живого шанса и RNG(100) ставится флаг,
/// а компоненты физического, элементного и душевного видов (исходная
/// фильтрация 1/3/4, VERIFIED_DISASSEMBLY — см. шапку) масштабируются
/// float-множителем с усечением FISTP. Та же машинная форма, что и у
/// `skills/projectile.rs::apply_projectile_critical`.
pub fn apply_weapon_critical(
    attack: &mut AttackInformation, chance: i32, roll: i32, rate: f32,
) {
    if roll >= chance { return; }
    attack.critical = true;
    for power in &mut attack.damages {
        if matches!(power.kind,
            AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul)
        {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

/// Тело CalculateAttackPower семейства: roll по виду `PlayerWeaponRoll`,
/// фронтальная ловкость и i64-усечённый множитель EnergyHolding, компоненты
/// Physical/Element/Soul с живыми чтениями и общий критический хвост.
/// Отсутствие живого поля обрывает расчёт в исходной точке (компоненты и
/// записи до неё сохраняются), RNG владельца вызывается строго в исходном
/// порядке; `captured_critical_chance` заменяет живое чтение CCH знаковым
/// значением конструктора (см. шапку про HeartLessArrowPhalanx2).
pub fn fill_weapon_damage(
    roll: PlayerWeaponRoll, boost: WeaponPowerBoost,
    captured_critical_chance: Option<i32>, critical_rate: f32,
    element_addition: impl FnOnce() -> u32,
    mut read_live: impl FnMut(WeaponDamageLiveField) -> Option<i32>,
    attack: &mut AttackInformation,
) {
    let scale = |damage: i32| match boost {
        WeaponPowerBoost::EnergyHolding(multiplier) => {
            truncate_original_i64_low(f64::from(damage) * multiplier)
        }
        _ => damage,
    };
    let (width, captured_minimum) = match roll {
        PlayerWeaponRoll::AbsoluteRange => {
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1), None)
        }
        PlayerWeaponRoll::RawRange => {
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_add(1), None)
        }
        PlayerWeaponRoll::CapturedMinimumAbsoluteRange => {
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1), Some(minimum))
        }
    };
    let Some(random) = read_live(WeaponDamageLiveField::RandomBelow(width)) else { return; };
    let Some(minimum) =
        captured_minimum.or_else(|| read_live(WeaponDamageLiveField::MinimumAttack))
    else { return; };
    let mut physical = minimum.wrapping_add(random);
    if !matches!(boost, WeaponPowerBoost::None) {
        let Some(dexterity) = read_live(WeaponDamageLiveField::Dexterity) else { return; };
        physical = physical.wrapping_add(dexterity);
    }
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Physical, hp_damage: scale(physical.max(0)), mp_damage: 0,
    });
    let element_addition = element_addition();
    let Some(element) = read_live(WeaponDamageLiveField::AddElementAttack) else { return; };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Element,
        hp_damage: scale(element.wrapping_add(element_addition as i32).max(0)),
        mp_damage: 0,
    });
    let Some(soul) = read_live(WeaponDamageLiveField::AddSoulAttack) else { return; };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Soul, hp_damage: scale(i32::from(soul as u16)), mp_damage: 0,
    });
    let critical_chance = match captured_critical_chance {
        Some(chance) => chance,
        None => {
            let Some(chance) = read_live(WeaponDamageLiveField::CriticalChance) else { return; };
            i32::from(chance as u16)
        }
    };
    let Some(critical_roll) = read_live(WeaponDamageLiveField::RandomBelow(100)) else { return; };
    apply_weapon_critical(attack, critical_chance, critical_roll, critical_rate);
}
