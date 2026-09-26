//! Данные области CSnowStormPhalanx, окна выбранных клеток, живая форма и
//! тело Summon CSnowStorm.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/snowstorm.cpp и snowstormphalanx.cpp/.h.
//! Summon VA 0x00584060, конструктор VA 0x005F9040,
//! Initialize 0x005F8D70,
//! EncodeToByteArray 0x005F8ED0, CalculateAttackPower 0x005F92D0,
//! Attack 0x005F93B0, AI 0x005F94B0.
//! Композит `CSnowStormPhalanx` (CShape + область) и тела
//! `summon_snow_storm`/`apply_snow_storm_attack` следуют старым адаптеру и
//! клею без новых машинных оснований. Summon сохраняет Master(country0) и
//! Player EM либо 0 до свежей таблицы; порядок запросов — в
//! `SnowStormSummonParameters::read`; после свойств ctor получает часы и ID.
//! SetTile→Initialize с RNG выполняются до повторного чтения actual
//! region U; Add→encode/BF502 не зависят от результата регистрации области
//! (швы `ZonalCastContact` в `skills/zonalcast.rs`). Некорректные параметры,
//! вызывающие native деление на ноль или выход из массива, явно
//! диагностируются конструктором и не создают успешную область
//! (безопасная граница, не изменение валидной формулы или RNG-порядка).

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::values::CGuid;
use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo};
use crate::effects::timed_client_state_time;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;
use super::summonshape::SUMMON_SHAPE_TYPE;
use super::zonalcast::{ZonalCastContact, ZonalCastMoveShape, ZonalCastPlayer};

pub const SNOW_STORM_SKILL_ID: u32 = 0x193;
pub const SNOW_STORM_SCOPE_AREA: u32 = 25;
const SCOPE_SIDE: i32 = 5;
const TARGET_COUNT_PROPERTY: u32 = 20_010;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const FREQUENCY_PROPERTY: u32 = 6_001;
const LIFETIME_PROPERTY: u32 = 30_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnowStormSummonParameters {
    pub target_count: u32,
    pub maximum_attack: i32,
    pub minimum_attack: i32,
    pub frequency_ms: u32,
    pub skill_level: i32,
    pub lifetime_ms: u32,
}

impl SnowStormSummonParameters {
    /// Порядок запросов в CSnowStorm::Summon сохраняет чтение живого уровня
    /// между частотой и временем жизни.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        current_level: impl FnOnce() -> Option<i32>,
    ) -> Option<Self> {
        let target_count = query_property(TARGET_COUNT_PROPERTY);
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let frequency_ms = query_property(FREQUENCY_PROPERTY);
        let skill_level = current_level()?;
        let lifetime_ms = query_property(LIFETIME_PROPERTY);
        Some(Self {
            target_count, maximum_attack, minimum_attack, frequency_ms,
            skill_level, lifetime_ms,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnowStormParametersError { ZeroFrequency, TooManyTargets, CellArrayTooLarge }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnowStormAttack {
    pub master: MasterInfo,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
}

impl SnowStormAttack {
    pub fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    fn element_damage(self, random_below: impl FnOnce(i32) -> i32) -> i32 {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack)
            .wrapping_abs().wrapping_add(1);
        random_below(width).wrapping_add(self.minimum_attack)
            .wrapping_add(self.element_modifier).max(0)
    }

    pub fn attack_information(self, random_below: impl FnOnce(i32) -> i32) -> AttackInformation {
        let mut attack = AttackInformation::for_master(self.attack_master());
        attack.skill_id = SNOW_STORM_SKILL_ID;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        attack.damage_factor = 1.0;
        attack.hit_modifier = 100;
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: self.element_damage(random_below),
            mp_damage: 0,
        });
        attack
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnowStormPhalanx {
    attack: SnowStormAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    target_count: u32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl SnowStormPhalanx {
    pub fn new(
        attack: SnowStormAttack, started_at_ms: u32, lifetime_ms: u32,
        frequency_ms: u32, target_count: u32,
    ) -> Result<Self, SnowStormParametersError> {
        let windows = lifetime_ms.checked_div(frequency_ms)
            .ok_or(SnowStormParametersError::ZeroFrequency)?;
        if target_count > SNOW_STORM_SCOPE_AREA {
            return Err(SnowStormParametersError::TooManyTargets);
        }
        let count = windows.checked_mul(SNOW_STORM_SCOPE_AREA)
            .filter(|count| count.checked_mul(8).is_some())
            .ok_or(SnowStormParametersError::CellArrayTooLarge)?;
        let mut cells = Vec::new();
        cells.try_reserve_exact(count as usize)
            .map_err(|_| SnowStormParametersError::CellArrayTooLarge)?;
        cells.resize(count as usize, (0, 0));
        Ok(Self { attack, started_at_ms, lifetime_ms, frequency_ms, target_count,
            last_attack_ms: 0, attack_count: 0, cells })
    }

    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> SnowStormAttack { self.attack }
    pub const fn skill_level(&self) -> i32 { self.attack.skill_level }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn frequency_ms(&self) -> u32 { self.frequency_ms }
    pub fn cells(&self) -> &[(i32, i32)] { &self.cells }

    /// Пишет поля области до базового CShape, который добавляет адаптер Game.
    pub fn write_client_snapshot_fields(
        &self, payload: &mut Vec<u8>, now_milliseconds: impl FnMut() -> u32,
    ) {
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(SNOW_STORM_SKILL_ID);
        writer.write_i32(self.attack.skill_level);
        writer.write_i32(self.attack.master.master_type);
        writer.write_i32(self.attack.master.master_id);
        writer.write_u32(timed_client_state_time(
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        ));
        writer.write_u32(self.lifetime_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.cells.len() as u32);
        for &(x, y) in &self.cells {
            writer.write_i32(x);
            writer.write_i32(y);
        }
    }

    pub fn initialize(
        &mut self, center_x: i32, center_y: i32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) {
        let origin_x = center_x.wrapping_sub(SCOPE_SIDE >> 1);
        let origin_y = center_y.wrapping_sub(SCOPE_SIDE >> 1);
        self.cells.fill((0, 0));
        for window in self.cells.chunks_exact_mut(SNOW_STORM_SCOPE_AREA as usize) {
            for cell in window.iter_mut().take(self.target_count as usize) {
                let x = random_below(SCOPE_SIDE);
                let y = random_below(SCOPE_SIDE);
                *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
            }
        }
    }

    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }

    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }

    pub fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub fn advance_attack_window(&mut self) { self.attack_count = self.attack_count.wrapping_add(1); }

    pub fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        let index = self.attack_count.wrapping_mul(SNOW_STORM_SCOPE_AREA).wrapping_add(index);
        // Native читает за массивом после его исчерпания; не создаём фиктивную цель.
        self.cells.get(index as usize).copied().filter(|cell| *cell != (0, 0))
    }
}

/// Живая форма снежной бури: связка CShape и области окон. Для уровней 1–3
/// размеры маски равны 5×5; Initialize после SetTile расходует X/Y RNG для
/// каждой цели каждого окна. Состав и порядок соответствуют адаптеру
/// старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSnowStormPhalanx {
    shape: CShape,
    area: SnowStormPhalanx,
}

impl CSnowStormPhalanx {
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, element_modifier: i32,
        parameters: SnowStormSummonParameters,
    ) -> Result<Self, SnowStormParametersError> {
        let attack = SnowStormAttack {
            master, skill_level: parameters.skill_level,
            minimum_attack: parameters.minimum_attack,
            maximum_attack: parameters.maximum_attack, element_modifier,
        };
        let area = SnowStormPhalanx::new(
            attack, started_at_ms, parameters.lifetime_ms,
            parameters.frequency_ms, parameters.target_count,
        )?;
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self { shape, area })
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.area.master() }
    pub const fn attack_snapshot(&self) -> SnowStormAttack { self.area.attack_snapshot() }

    pub fn initialize(&mut self, random_below: &mut dyn FnMut(i32) -> i32) {
        let center_x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let center_y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        self.area.initialize(center_x, center_y, random_below);
    }

    pub const fn expired_at(&self, now: u32) -> bool { self.area.expired_at(now) }
    pub const fn attack_due_at(&self, now: u32) -> bool { self.area.attack_due_at(now) }
    pub fn mark_attack_at(&mut self, now: u32) { self.area.mark_attack_at(now); }
    pub fn advance_attack_window(&mut self) { self.area.advance_attack_window(); }

    pub fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        self.area.current_cell(index)
    }

    /// Пакет содержит skill/level/master type/id, срок, частоту и весь
    /// массив клеток, затем передаёт запись базовой форме.
    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.area.write_client_snapshot_fields(&mut payload, now);
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

/// Тело `CSnowStorm::Summon` (VA 0x00584060): Master(country0)/Player EM до
/// свежей таблицы; SetTile→Initialize(RNG) до повторного чтения actual
/// region U; Add→encode/BF502 — швами владельца.
pub fn summon_snow_storm<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    destination: (i32, i32),
    runtime: &mut Runtime,
    now_milliseconds: &mut dyn FnMut(&mut Runtime) -> u32,
)
where
    Game: ZonalCastContact<Runtime>,
{
    let Some(mut master) = game.zonal_source_master(source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == PLAYER_TYPE {
        game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify)
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let Some(parameters) = SnowStormSummonParameters::read(
        |property| properties.query_property(property),
        || game.registered_skill(instance).map(|skill| skill.level()),
    ) else { return; };
    let started = now_milliseconds(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = match CSnowStormPhalanx::new(
        id, master, started, element, parameters,
    ) {
        Ok(phalanx) => phalanx,
        Err(error) => {
            tracing::error!(skill_id = SNOW_STORM_SKILL_ID, ?error, "некорректные параметры снежной бури");
            return;
        }
    };
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    phalanx.initialize(&mut |maximum| game.skill_random_below(maximum));
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    let _ = game.add_snow_storm_phalanx(region_id, phalanx, started, runtime);
}

/// Применение окна к живой цели: отсутствующая или мёртвая цель
/// пропускается, атака собирается с RNG владельца и применяется общим
/// боевым контактом.
pub fn apply_snow_storm_attack<Game, Runtime>(
    game: &mut Game,
    snapshot: SnowStormAttack,
    region: i32,
    target: ShapeIdentity,
    runtime: &mut Runtime,
)
where
    Game: ZonalCastContact<Runtime>,
{
    if game.move_shape_health(region, target).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let attack = snapshot.attack_information(|width| game.skill_random_below(width));
    game.apply_owned_skill_contact(master, target, region, attack, runtime);
}
