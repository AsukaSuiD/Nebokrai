//! Общая база прицельных снарядов Archery, BaseMagic и FireBolt и площадных
//! областей FireBall и GodPunishment: снимок полёта, физический и элементный
//! контакт, усилитель душами, серверный decoder и живые композиты FireBall и
//! GodPunishment.
//!
//! Размещение в `skills/`: полёт строится на конверте `summonshape`, боевые
//! формулы принадлежат навыковым владельцам; live-разрешение полей источника
//! и доставка контакта остаются у владельца `CGame` (обёртка
//! `elementprojectileattack` старого пакета). Выделенный ctor-ом `CScope`
//! объектов семьи остаётся у прежнего владельца AI и не переносится.
//!
//! Машинные quirks, сохранённые дословно: физический roll Archery берёт
//! ширину `max(max - min, 0)` БЕЗ +1 (элементная формула — `abs(max - min) + 1`);
//! Archery хранит неиспользуемые аргументы MIN/MAX/ELEMENT из ctor.
//!
//! Исходные владельцы PDB: `appserver/skills/archeryphalanx.cpp/.h`,
//! `basemagicphalanx.cpp/.h`, `fireboltphalanx.cpp/.h`,
//! `fireballphalanx.cpp/.h`, `godpunishmentphalanx.cpp/.h`; базовый класс —
//! `appserver/summonshape.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#projectile-прицельные-снаряды-и-композиты-fireballgodpunishment

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

    /// Снимок полёта областей без прицела (FireBall, GodPunishment): общими
    /// с прицельными снарядами остаются форма, часы и конверт `CSummonShape`;
    /// их конструкторы слотов задержки атаки и цели не содержат, поэтому они
    /// и не задаются.
    pub fn new_untargeted(id: i32, started_at_ms: u32, lifetime_ms: u32) -> Self {
        Self::new(
            id, started_at_ms, lifetime_ms, 0,
            ShapeIdentity { object_type: 0, id: 0, ex_id: CGuid::GUID_INVALID },
        )
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn target(&self) -> ShapeIdentity { self.target }

    /// Общая стартовая отметка полёта (слот `+0xB4`); движение FireBall
    /// считает от неё дедлайн очередной клетки.
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }

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

/// Движение области FireBall по пути клеток: независимый снимок пути,
/// курсор и однократный ForceMove. Слоты и назначение — по ctor
/// `1:001f6cc0` и AI `1:001f75f0` (шапка модуля): speed лежит в `+0xD8`
/// между souls и уровнем профиля, вектор клеток — в `+0xDC`, текущая
/// позиция, endpoint и флаг обнуляются конструктором; срок жизни читает
/// свои часы отдельно — у полёта композита.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FireBallPath {
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    current_position: u32,
    destination: (i32, i32),
    force_moved: bool,
}

impl FireBallPath {
    /// Курсор, endpoint и флаг начинаются нулевым состоянием конструктора;
    /// путь сохраняется независимо от владельца cast-а.
    pub fn new(path: Vec<(i32, i32)>, speed_ms: u32) -> Self {
        Self { path, speed_ms, current_position: 0, destination: (0, 0), force_moved: false }
    }

    pub fn is_empty(&self) -> bool { self.path.is_empty() }

    /// Готовность очередной клетки: дедлайн считается dword-сложением
    /// `started + позиция · speed` от общей стартовой отметки полёта.
    pub fn cell_due_at(&self, started_at_ms: u32, now_ms: u32) -> bool {
        started_at_ms.wrapping_add(
            self.current_position.wrapping_mul(self.speed_ms)
        ) <= now_ms
    }

    pub fn current_cell(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_position as usize).copied()
    }

    /// Endpoint допущенной клетки; клиентский конверт его не содержит.
    pub fn set_destination(&mut self, cell: (i32, i32)) { self.destination = cell; }

    pub fn advance(&mut self) { self.current_position = self.current_position.wrapping_add(1); }

    /// Один раз за жизнь области: последняя клетка пути и длительность
    /// `len · speed`; флаг владелец ставит после callback.
    pub fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }

    pub fn mark_force_moved(&mut self) { self.force_moved = true; }

    /// Статическая маска области 3×3 X→Y, заменяющая CScope конструктора.
    pub fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        (0..3).flat_map(move |x| (0..3).map(move |y| {
            (center_x.wrapping_add(x - 1), center_y.wrapping_add(y - 1))
        }))
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

/// Литерал навыка огненного шара для слота `+0xB8` (ctor pub `1:001f6cc0`,
/// статус в шапке модуля).
pub const FIRE_BALL_SKILL_ID: u32 = 0x13d;

/// Литерал навыка божественной кары для слота `+0xB8` (ctor pub `1:001fcd80`,
/// статус в шапке модуля).
pub const GOD_PUNISHMENT_SKILL_ID: u32 = 0x13a;

/// Живая форма движущейся площадной области огненного шара: общий полёт,
/// элементный контакт с усилителем душами и движение пути. Души приходят
/// снимком из cast-а, как у FireBolt. Срок жизни и дедлайн очередной клетки
/// читают разные часы.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CFireBallPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
    movement: FireBallPath,
}

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBallPhalanx")]
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack {
                master,
                skill_id: FIRE_BALL_SKILL_ID,
                skill_level,
                minimum_attack,
                maximum_attack,
                element_modifier,
                souls: Some(SoulProjectileAmplification::new(soul_count, soul_variable as i32)),
            },
            movement: FireBallPath::new(path, speed_ms),
        }
    }

    pub const fn shape(&self) -> &CShape { self.flight.shape() }
    pub const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }
    pub fn path_is_empty(&self) -> bool { self.movement.is_empty() }

    pub fn expired_at(&self, now_ms: u32) -> bool { self.flight.expired_at(now_ms) }

    pub fn cell_due_at(&self, now_ms: u32) -> bool {
        self.movement.cell_due_at(self.flight.started_at_ms(), now_ms)
    }

    pub fn current_cell(&self) -> Option<(i32, i32)> { self.movement.current_cell() }
    pub fn set_cell_destination(&mut self, cell: (i32, i32)) {
        self.movement.set_destination(cell);
    }
    pub fn advance(&mut self) { self.movement.advance(); }
    pub fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        self.movement.pending_force_move()
    }
    pub fn mark_force_moved(&mut self) { self.movement.mark_force_moved(); }

    pub fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        FireBallPath::scope_cells(center_x, center_y)
    }

    pub fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            FIRE_BALL_SKILL_ID, self.attack.skill_level, self.attack.master,
            now_milliseconds,
        )
    }
}

/// Живая форма одноклеточной области божественной кары: тот же общий полёт
/// и элементный контакт без усиления душами и без CScope конструктора.
/// Замена игнорирует уровень и завершает форму при совпадении живой клетки.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CGodPunishmentPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
}

impl CGodPunishmentPhalanx {
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack {
                master,
                skill_id: GOD_PUNISHMENT_SKILL_ID,
                skill_level,
                minimum_attack,
                maximum_attack,
                element_modifier,
                souls: None,
            },
        }
    }

    pub const fn shape(&self) -> &CShape { self.flight.shape() }
    pub const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }

    pub fn expired_at(&self, now: u32) -> bool { self.flight.expired_at(now) }

    pub fn replacement_matches(&self, _level: i32, x: i32, y: i32) -> bool {
        self.flight.shape().get_tile_x().unwrap_or(i32::MIN) == x
            && self.flight.shape().get_tile_y().unwrap_or(i32::MIN) == y
    }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            GOD_PUNISHMENT_SKILL_ID, self.attack.skill_level, self.attack.master, now,
        )
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
