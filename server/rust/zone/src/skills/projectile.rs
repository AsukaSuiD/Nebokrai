//! Общая база прицельных снарядов Archery, BaseMagic и FireBolt:
//! снимок полёта, элементный контакт и усилитель душами.
//!
//! Размещение в `skills/`: полёт строится на конверте `summonshape`, а
//! боевые формулы принадлежат навыковым владельцам, как у соседних
//! `elementphalanx` и `directelement`.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`
//! age 2, совпадение подтверждено `.local/evidence/symbols.py identity`).
//! Исходные владельцы PDB: `appserver/skills/archeryphalanx.cpp/.h`,
//! `basemagicphalanx.cpp/.h` и `fireboltphalanx.cpp/.h`.
//!
//! Pub-адреса (`.local/evidence/symbols.py pubs`): ctor `CArcheryPhalanx`
//! `1:00200aa0`, общий `End` `1:00200c10`,
//! `CArcheryPhalanx::CalculateAttackPower` `1:00200c40`,
//! `CBaseMagicPhalanx::CalculateAttackPower` `1:00201240`,
//! `CFireBoltPhalanx::CalculateAttackPower` `1:001fc970`,
//! `CFireBoltPhalanx::AddToByteArray` `1:001fad20`.
//!
//! Сроки полёта: VERIFIED_DISASSEMBLY. `CArcheryPhalanx::AI` (RVA
//! `0x201000`) дважды вызывает часы через IAT `0x64B264`
//! (`WINMM!timeGetTime`): истечение `now > started + lifetime` и готовность
//! атаки `now > started + attack_delay` считаются dword-сложением по модулю
//! 2^32 с беззнаковыми `ja`/`jbe`; строгость неравенств сохранена дословно.
//!
//! Общий End: VERIFIED_DISASSEMBLY. Тело по адресу `1:00200c10` —
//! `mov dword ptr [ecx+0x80], 1; ret`, оно свёрнуто линкером как минимум
//! для Archery и FireBolt: тихая пометка `SHAPE_CHANGE_DELETE` без рассылки.
//!
//! Элементная формула: VERIFIED_DISASSEMBLY по двум телам семьи —
//! `CFireBoltPhalanx::CalculateAttackPower` (RVA `0x1FC970`) и
//! `CBaseMagicPhalanx::CalculateAttackPower` (RVA `0x201240`). Живой
//! element_modify игрока читается первым, до трёх записей снимка; затем
//! уровень цели (vtable `+0x110`) и живой weapon modifier (vtable `+0x184`,
//! float в `damage_factor`), `hit_modifier = 100`, знаковое
//! `element_modifier * element_modify / 100`, ширина `abs(max - min) + 1`,
//! RNG, сохранённый MIN, живой AddElementAtk (vtable `+0x118`),
//! необязательное усиление душами и нижняя граница ноль; единственная
//! атака вида Element. Критический хвост общий: живой CCH (vtable `+0x114`,
//! movzx u16), RNG(100), затем флаг `critical` и масштабирование компонентов
//! видов 1/3/4 (Physical/Element/Soul) float-множителем с усечением FISTP.
//! Тела различаются только смещением байта уровня (`+0xCC` у BaseMagic,
//! `+0xD4` у FireBolt) и наличием усилителя у FireBolt, что соответствует
//! снимку `souls`; сумма трёх слагаемых по модулю 2^32 не зависит от
//! порядка сложения.
//!
//! Усилитель душами: VERIFIED_DISASSEMBLY (RVA `0x1FDAC0`..`0x1FDB14`):
//! обе нулевые проверки `count == 0` / `variable == 0`, затем x87-цепочка
//! `fild variable`, `fimul count`, `fmul float(0.01)`, `fadd float(1.0)`,
//! `fimul damage` и одно усечение FISTP без промежуточной float-записи.
//!
//! Клиентский снимок `encode_client_snapshot` делегируется общему конверту
//! `summonshape` (master type/id вложенного `tagMasterInfo`, не сохранённая
//! цель): статус см. там.
//!
//! PARTIAL: смещения полей конструктора (started/lifetime/attack_delay/
//! target и боевые значения владельцев) унаследованы от прежнего
//! Rust-адаптера без отдельной сверки layout; точные имена полей PDB не
//! фиксировались. Физический удар Archery (`0x200C40`) и перенос конкретных
//! владельцев — предмет отдельных порций.

use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseProjectileFlight {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl BaseProjectileFlight {
    pub fn new(
        id: i32, started_at_ms: u32, lifetime_ms: u32,
        attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms, lifetime_ms, attack_delay_ms,
            target: ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..target },
        }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn target(&self) -> ShapeIdentity { self.target }

    pub fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub fn attack_due_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.attack_delay_ms) < now_ms
    }

    pub fn end(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub fn encode_client_snapshot(
        &self, skill_id: u32, skill_level: i32, master: MasterInfo,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, skill_id as i32, skill_level,
            master.master_type, master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}

/// Живые поля источника и RNG элементного контакта прицельного снаряда.
/// Разрешает их старый владелец в порядке исходного Calculate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementProjectileLiveField {
    RandomBelow(i32), AddElementAttack, CriticalChance, ElementModify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoulProjectileAmplification {
    count: i32,
    variable: i32,
}

impl SoulProjectileAmplification {
    pub const fn new(count: i32, variable: i32) -> Self { Self { count, variable } }

    fn apply(self, damage: i32) -> i32 {
        if self.count == 0 || self.variable == 0 { return damage; }
        truncate_original(
            (f64::from(self.variable) * f64::from(self.count) * f64::from(0.01_f32) + 1.0)
                * f64::from(damage),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementProjectileAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
    pub souls: Option<SoulProjectileAmplification>,
}

impl ElementProjectileAttack {
    pub fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    /// Вызывается после разрешения Player по attacker ID: живой
    /// element_modify читается первым, до трёх записей конструктора, и
    /// возвращается для `roll_damage`. Отсутствие поля оставляет исходную
    /// пустую атаку, но не отменяет контакт у владельца.
    pub fn begin_calculation(
        self, attack: &mut AttackInformation,
        mut read_live: impl FnMut(ElementProjectileLiveField) -> Option<i32>,
    ) -> Option<i32> {
        let element_modify = read_live(ElementProjectileLiveField::ElementModify)?;
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        Some(element_modify)
    }

    /// Вызывается после разрешения уровня цели и живого weapon modifier
    /// (оба остаются у владельца). Отсутствие живого AddElement или CCH
    /// обрывает расчёт с уже записанными полями и компонентами.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, element_modify: i32, weapon_modifier: f32,
        critical_rate: f32, mut read_live: impl FnMut(ElementProjectileLiveField) -> Option<i32>,
    ) {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = 100;
        let element = self.element_modifier.wrapping_mul(element_modify).wrapping_div(100);
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let Some(random) = read_live(ElementProjectileLiveField::RandomBelow(width)) else { return; };
        let rolled = random.wrapping_add(self.minimum_attack);
        let Some(addition) = read_live(ElementProjectileLiveField::AddElementAttack) else { return; };
        let damage = element.wrapping_add(addition.wrapping_add(rolled));
        let damage = self.souls.map_or(damage, |souls| souls.apply(damage)).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        let Some(chance) = read_live(ElementProjectileLiveField::CriticalChance) else { return; };
        let Some(roll) = read_live(ElementProjectileLiveField::RandomBelow(100)) else { return; };
        Self::apply_critical(attack, chance, roll, critical_rate);
    }

    /// Общий критический хвост: после живого шанса и RNG(100) ставится флаг,
    /// а компоненты физического, элементного и душевного видов (исходная
    /// фильтрация 1/3/4) масштабируются float-множителем с усечением.
    fn apply_critical(attack: &mut AttackInformation, chance: i32, roll: i32, rate: f32) {
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
}
