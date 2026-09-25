//! Общая база прицельных снарядов Archery, BaseMagic и FireBolt:
//! снимок полёта, физический контакт Archery, элементный контакт,
//! усилитель душами и общий серверный decoder снимка.
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
//! `basemagicphalanx.cpp/.h` и `fireboltphalanx.cpp/.h`; базовый класс —
//! `appserver/summonshape.cpp/.h` (кто хранит слоты полёта, master и skill id,
//! см. `skills/summonshape`).
//!
//! Pub-адреса (`.local/evidence/symbols.py pubs`): ctor `CArcheryPhalanx`
//! `1:00200aa0`, общий `End` `1:00200c10`,
//! `CArcheryPhalanx::CalculateAttackPower` `1:00200c40`,
//! `CArcheryPhalanx::AI` `1:00201000`,
//! `CBaseMagicPhalanx::CalculateAttackPower` `1:00201240`,
//! `CFireBoltPhalanx::CalculateAttackPower` `1:001fc970`,
//! `CFireBoltPhalanx::AddToByteArray` `1:001fad20`,
//! `CArcheryPhalanx::DecordFromByteArray` `1:001ea070` (RVA `0x1EB070`),
//! базовый `CSummonShape` ctor `1:001e87a0`, `CShape::DecordFromByteArray`
//! pub offset `0x5a280`.
//!
//! Сроки полёта: VERIFIED_DISASSEMBLY. `CArcheryPhalanx::AI` (RVA
//! `0x201000`) дважды вызывает часы через IAT `0x64B264`
//! (`WINMM!timeGetTime`): истечение `now > started + lifetime` и готовность
//! атаки `now > started + attack_delay` считаются dword-сложением по модулю
//! 2^32 с беззнаковыми `ja`/`jbe`; строгость неравенств сохранена дословно.
//! Между проверками стоит NULL-проверка выделенного конструктором `CScope`
//! (`+0xBC` → общий End) — сама область и её время жизни остаются у прежнего
//! владельца AI и не переносятся.
//!
//! Общий End: VERIFIED_DISASSEMBLY. Тело по адресу `1:00200c10` —
//! `mov dword ptr [ecx+0x80], 1; ret`, оно свёрнуто линкером как минимум
//! для Archery и FireBolt: тихая пометка `SHAPE_CHANGE_DELETE` без рассылки.
//!
//! Конструктор `CArcheryPhalanx` (pub `1:00200aa0`): VERIFIED_DISASSEMBLY
//! для хранимого снимка. Базовый `CSummonShape` (pub `1:001e87a0`) принимает
//! master и срок жизни: вложенный `tagMasterInfo` конструируется и
//! присваивается по `+0x84` (ctor/assign VA `0x50A610/0x50A640`, см.
//! `combat/masterinfo`), срок жизни кладётся в `+0xB0`, живой `timeGetTime` —
//! в `+0xB4`, а skill id предустановлен `0x7FFFFFFF` в `+0xB8`. Archery
//! перезаписывает `+0xB8` литералом `2` (= прежнему `ARCHERY_SKILL_ID`
//! адаптера), уровень аргументом в `+0xCC`, неиспользуемые дальше аргументы
//! MIN/MAX/ELEMENT — в `+0xC0/+0xC4/+0xC8` (не дублируются), задержку атаки
//! в `+0xD0`, цель type/id в `+0xD4/+0xD8`; отдельно выделяется не участвующий
//! в расчёте `CScope` (`+0xBC`). Машинный `AI` подтверждает назначение слотов
//! дедлайнами `+0xB4`+`+0xB0` и `+0xB4`+`+0xD0` и поиском цели по
//! `+0xD4/+0xD8`.
//!
//! Физический удар Archery: VERIFIED_DISASSEMBLY по
//! `CArcheryPhalanx::CalculateAttackPower` (pub `1:00200c40`). Сначала
//! `GetGame()` (pub `?GetGame@@YAPAVCGame@@XZ`) и поиск CPlayer в map игроков
//! по `attacker_id` атаки независимо от сохранённого типа, затем NULL-проверка
//! цели и живая таблица через pub
//! `?QuerySkillBaseProperties@CSkillFactory@@SAPAVCSkillBaseProperties@@W4tagSkillID@@J@Z`
//! (skill id из `+0xB8`, уровень из `+0xCC`); отсутствие игрока или таблицы
//! оставляет атаку прежней, записи полей идут после. Из снимка объекта
//! пишутся skill id, байт уровня и нулевой damage modifier; уровень цели
//! читается vtable `+0x110` как байт (movzx), живой weapon modifier — vtable
//! `+0x184` игрока с записью в damage factor, hit — `QueryProperty(20001)`
//! (`push 0x4E21`; чтения 20002 нет). Физический roll: чтения MAX
//! (vtable `+0xE8`) и MIN (`+0xE4`), ширина `max(max - min, 0)` БЕЗ +1
//! (`sub` + `jns`/`xor` — отличие от элементной формулы ниже, сохранено
//! дословно), второе чтение MIN до RNG глобального `random` (pub
//! `?random@@YAHH@Z`), сумма второй MIN со сгенерированным значением и нижняя
//! граница ноль; далее компонент Element (vtable `+0x118`) с той же нижней
//! границей, компонент Soul (vtable `+0x11C`, movzx u16) и критический хвост:
//! CCH (vtable `+0x114`, movzx u16), RNG(100), флаг и float-масштабирование
//! FILD/FMUL/FISTP только компонентов видов 1/3/4 (Physical/Element/Soul) —
//! тот же фильтр и та же последовательность, что у элементной формулы.
//! Выделения `new tagAttackPower` (ctor `0x5D3C80`) и `std::vector::push_back`
//! выражены `Vec::push`; множитель крита читается из изменяемой BSS-глобали
//! (`fmul dword ptr [0xEF3E5C]`) и передаётся параметром `critical_rate`
//! живого чтения владельца, как у элементной формулы.
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
//! Серверный `DecordFromByteArray` (pub `1:001ea070`, RVA `0x1EB070`):
//! VERIFIED_DISASSEMBLY. Линкер сливает в одно тело декодеры
//! `CArcheryPhalanx`, `CBaseMagicPhalanx` и `CBFBaseAttackPhalanx`; hub не
//! используется, поэтому тело перенесено. Пять DWORD читаются подряд с
//! продвижением offset: skill id в `+0xB8`, уровень в `+0xCC`, master type/id
//! вложенного `tagMasterInfo` в `+0x84/+0x88` (остальные восемь полей master
//! не трогаются) и остаток времени в слот срока жизни `+0xB0` — зеркально
//! префиксу `encode_related_phalanx_prefix`; затем одно чтение часов через
//! IAT `0x64B264` перезапускает отсчёт в `+0xB4`, и хвост делегируется
//! `CShape::DecordFromByteArray` (pub offset `0x5a280`; достигнутый
//! `decode_from_byte_array` в `regions::shape`) с проброшенным
//! `include_ex_data`. Неконтролируемые native-чтения выражены проверками
//! `UnexpectedEnd`: уже записанные поля префикса сохраняются, как и у
//! оригинала, возвращавшего FALSE после частичных записей. Достижимого
//! caller-а у оригинала нет; часы приходят параметром, как у клиентского
//! encoder-а.
//!
//! PARTIAL: боевые значения и souls снимков BaseMagic и FireBolt (включая
//! их смещения уровня и souls) унаследованы от прежнего Rust-адаптера без
//! отдельной сверки layout в этой порции; для Archery layout сверен выше
//! только по её собственному ctor, а подтверждённые слоты уровня weak
//! (`+0xC8`) и firebolt (`+0xD4`) из шапки `summonshape` относятся к их
//! собственным телам encoder-ов. Точные имена полей PDB не фиксировались.
//! Семейность формул FireBall и GodPunishment в этой порции не сверялась
//! (их тела `CalculateAttackPower` адресно не совпадают с проверенной парой)
//! — предмет follow-up.

use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeDecodeError};
use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseProjectileFlight {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

/// Прочитанный серверным decoder-ом префикс снимка прицельной фаланги до
/// базового `CShape`: идентификатор и уровень навыка владельца и master
/// type/id вложенного `tagMasterInfo`. Назначение и порядок чтений — в шапке
/// модуля; остаток времени ложится сразу в слот срока жизни полёта.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectileServerSnapshotPrefix {
    pub skill_id: i32,
    pub skill_level: i32,
    pub master_type: i32,
    pub master_id: i32,
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

    /// Общий серверный `DecordFromByteArray` прицельных фаланг (тело слито
    /// линкером у Archery, BaseMagic и BFBaseAttack): пять DWORD префикса —
    /// навык, уровень, master type/id, остаток времени — затем перезапуск
    /// отсчёта от живых часов и хвост базового `CShape`. Порядок записей и
    /// их назначение сверены в шапке модуля; достижимого caller-а у
    /// оригинала нет.
    pub fn decode_server_snapshot(
        &mut self,
        prefix: &mut ProjectileServerSnapshotPrefix,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Result<(), ShapeDecodeError> {
        prefix.skill_id = read_projectile_i32(source, cursor, "прицельный снаряд: skill id")?;
        prefix.skill_level =
            read_projectile_i32(source, cursor, "прицельный снаряд: уровень навыка")?;
        prefix.master_type =
            read_projectile_i32(source, cursor, "прицельный снаряд: master type")?;
        prefix.master_id = read_projectile_i32(source, cursor, "прицельный снаряд: master id")?;
        self.lifetime_ms =
            read_projectile_u32(source, cursor, "прицельный снаряд: остаток времени")?;
        self.started_at_ms = now_milliseconds();
        self.shape.decode_from_byte_array(source, cursor, include_child)
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
        phalanx_attack_master(self.master)
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
        apply_projectile_critical(attack, chance, roll, critical_rate);
    }
}

/// Табличный параметр живого hit модификатора Archery (исходный
/// `QueryProperty(20001)`; прежний `USER_HIT_MODIFIER` оружейного прохода
/// старого пакета — тот же номер usage).
pub const ARCHERY_HIT_MODIFIER_PROPERTY: u32 = 20_001;

/// Живые поля источника и RNG физического контакта Archery прицельного
/// снаряда. Разрешает их старый владелец в порядке исходного Calculate;
/// movzx-u16 чтения Soul и CCH принадлежат этой же подстановке.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcheryProjectileLiveField {
    RandomBelow(i32), MinimumAttack, MaximumAttack,
    AddElementAttack, AddSoulAttack, CriticalChance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcheryProjectileAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
}

impl ArcheryProjectileAttack {
    pub fn attack_master(self) -> MasterInfo {
        phalanx_attack_master(self.master)
    }

    /// Вызывается после разрешения игрока по attacker ID и живой таблицы
    /// навыка (оба остаются у владельца): записывает исходные id навыка,
    /// байт уровня и нулевой damage modifier.
    pub fn begin_calculation(self, attack: &mut AttackInformation) {
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
    }

    /// Вызывается после разрешения уровня цели и живого weapon modifier
    /// (оба остаются у владельца). Hit читается параметром
    /// `ARCHERY_HIT_MODIFIER_PROPERTY`, затем физический roll в исходном
    /// порядке MAX→MIN→второй MIN→RNG с шириной `max(max - min, 0)` без +1
    /// (отличие от элементной формулы), нижняя граница ноль, компоненты
    /// Element и Soul и общий критический хвост. Отсутствие живого поля
    /// обрывает расчёт с уже записанными полями и компонентами.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, weapon_modifier: f32, critical_rate: f32,
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(ArcheryProjectileLiveField) -> Option<i32>,
    ) {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = query_property(ARCHERY_HIT_MODIFIER_PROPERTY) as i32;
        let Some(maximum) = read_live(ArcheryProjectileLiveField::MaximumAttack) else { return; };
        let Some(minimum) = read_live(ArcheryProjectileLiveField::MinimumAttack) else { return; };
        let width = maximum.wrapping_sub(minimum).max(0);
        let Some(minimum) = read_live(ArcheryProjectileLiveField::MinimumAttack) else { return; };
        let Some(random) = read_live(ArcheryProjectileLiveField::RandomBelow(width)) else { return; };
        let physical = minimum.wrapping_add(random).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0,
        });
        let Some(element) = read_live(ArcheryProjectileLiveField::AddElementAttack) else { return; };
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: element.max(0), mp_damage: 0,
        });
        let Some(soul) = read_live(ArcheryProjectileLiveField::AddSoulAttack) else { return; };
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0,
        });
        let Some(chance) = read_live(ArcheryProjectileLiveField::CriticalChance) else { return; };
        let Some(roll) = read_live(ArcheryProjectileLiveField::RandomBelow(100)) else { return; };
        apply_projectile_critical(attack, chance, roll, critical_rate);
    }
}

/// Общее для прицельных фаланг правило владельца атаки: только master типа
/// 400 сохраняет расширенные поля, остальные получают type/id при нулевых
/// разрешениях.
fn phalanx_attack_master(master: MasterInfo) -> MasterInfo {
    if master.master_type == 400 { return master; }
    MasterInfo {
        master_type: master.master_type, master_id: master.master_id,
        ..MasterInfo::default()
    }
}

/// Общий критический хвост: после живого шанса и RNG(100) ставится флаг,
/// а компоненты физического, элементного и душевного видов (исходная
/// фильтрация 1/3/4) масштабируются float-множителем с усечением.
fn apply_projectile_critical(
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

fn read_projectile_i32(
    source: &[u8], cursor: &mut usize, field: &'static str,
) -> Result<i32, ShapeDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: 4, available: block.available,
        })?;
    let value = reader.read_i32().map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: block.needed, available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_projectile_u32(
    source: &[u8], cursor: &mut usize, field: &'static str,
) -> Result<u32, ShapeDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: 4, available: block.available,
        })?;
    let value = reader.read_u32().map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: block.needed, available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}
