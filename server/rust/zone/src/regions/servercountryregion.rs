//! Данные и скалярные правила country war-региона `ServerCountryRegion`.
//! Исходный владелец — `appserver/servercountryregion.h/.cpp`; сверка по
//! точной паре `gameserver.exe` + `GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`). Переходный
//! агрегат `CServerCountryRegion` остаётся в старом пакете: хранит hub
//! `CServerRegion`, contender-список (element `ContendState` — data-тип Zone
//! `regions/serverwarregion`, re-export через hub) и карты concrete
//! gates/flags поверх hub-типов, а этому агрегату делегирует чистый state
//! (symbol ownership, area-maps, guard sets, стороны и фазовые флаги) и все
//! скалярные операции без изменения сигнатур; wire-stream readers с
//! `RegionDecodeInputBlock` живут в Zone `regions/serverwarregion`, а
//! entry-effects для contenders и их вызовы остаются у старого пакета.
//!
//! Статус `IMPLEMENTED, VERIFIED_DISASSEMBLY` (унаследован от шапки старого
//! владельца, без повышения): subtype decoder `0x001CD3F0`, gate runtime
//! `0x001CAC80/0x001CADD0/0x001CB1E0..0x001CB310`, refresh
//! `0x001CB750/0x001CB880`, `ClearRegion` `0x001CBA50`, phase callbacks
//! `0x001CABC0..0x001CAC60`, spatial `0x001CA9F0/0x001CE010`, virtual
//! security `0x001CAD70`, guard ownership/refresh
//! `0x001CB370/0x001CC960..0x001CCA30`, `GetCamp` `0x001CE540`, attackability
//! `0x001CE830..0x001CE970`, contender damage/time
//! `0x001CA980/0x001CAFF0/0x001CB0D0`, enter/list/symbol
//! `0x001CB710/0x001CCAC0/0x001CCB50/0x001CE190/0x001CE2B0`, AI/victory
//! `0x001CE380/0x001CEA10` и `IsPlayerContendSymbol` `0x001D24E0`;
//! `VERIFIED_DISASSEMBLY`: gate/camp/flag layout, map-key, overload return,
//! area selection и refresh/clear order.
//!
//! `BTreeMap`/`BTreeSet`/Vec сохраняют STL order. Flags являются обычными
//! `CBuild` type `0x44C`: wire `field_24` не применяется, initial action
//! остаётся `0`, а refresh меняет только HP; scalar decode wire blocks — это
//! семья этого модуля, а hub-публикация `0xBF60F` и обходы карт остаются у
//! вызывающего aggregate. `random(map.size())` и исходный mutating
//! `operator[]` заданы return-point семейством. `INT_MIN / -1` и недоказанные
//! invalid x87 conversions остаются локальными typed-границами арифметики
//! этого модуля. Exact EXE подтвердил исходную странность
//! `OnPrepareBegin/End`: оба проверяют `_state_prepare`, но меняют
//! `_state_declare`; безопасный исходный gate закрыт.
//! Victory callback `OnFlagDestroy` `0x001CAD50` независимо от исходного
//! war-byte оставляет его false при совпавшем region ID; переданный country
//! long не читает.
//! `CancelContendByPlayer` содержит исходный дефект инвертированного условия:
//! любой реальный player немедленно получает `false`, а null-ветка читает
//! absolute `0x8` и вызывает метод с null; safe Rust не придумывает ей
//! результат.

use std::collections::{BTreeMap, BTreeSet};

use nebokrai_shared::protocol::LegacyReader;

use super::build::BuildClientPublication;
use super::region::{RegionCellAccessBlock, RegionRandomContext, RegionReturnPoint};
use super::serverregion::returnsetup::ServerReturnSetupBlock;

/// Camp-коды `WCDefend/WCAttack` исходного wire layout и runtime lookups.
pub const COUNTRY_CAMP_DEFEND: i32 = 0;
pub const COUNTRY_CAMP_ATTACK: i32 = 1;

/// Скалярный state country war-региона без hub-хранилищ: symbol ownership,
/// defend/attack area-maps и guard sets, выбранные стороны и фазовые флаги.
/// Contender-список и карты gates/flags остаются у переходного aggregate
/// старого пакета (element `ContendState` — Zone data-тип, gate-типы — hub).
#[derive(Debug, Eq, PartialEq)]
pub struct ServerCountryRegionState {
    symbol_hold: BTreeMap<i32, i32>,
    defend_areas: BTreeMap<i32, CountryAreaState>,
    attack_areas: BTreeMap<i32, CountryAreaState>,
    defend_guards: BTreeSet<i32>,
    defend_guard_indices: BTreeSet<i32>,
    attack_guards: BTreeSet<i32>,
    attack_guard_indices: BTreeSet<i32>,
    defend_country: i32,
    attack_country: i32,
    declare_active: bool,
    prepare_active: bool,
    war_active: bool,
}

impl Default for ServerCountryRegionState {
    fn default() -> Self {
        Self {
            symbol_hold: BTreeMap::new(),
            defend_areas: BTreeMap::new(),
            attack_areas: BTreeMap::new(),
            defend_guards: BTreeSet::new(),
            defend_guard_indices: BTreeSet::new(),
            attack_guards: BTreeSet::new(),
            attack_guard_indices: BTreeSet::new(),
            // Exact ctor не инициализирует эти primitive-поля. Нулевые стороны
            // и закрытые фазы — безопасное нейтральное состояние до первых
            // доказанных CountryWarSys writer/callback-ов, без чтения UB.
            defend_country: 0,
            attack_country: 0,
            declare_active: false,
            prepare_active: false,
            war_active: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryGateBuild {
    pub picture_id: i32,
    pub direction: i32,
    pub action: u16,
    pub max_hp: i32,
    pub defence: i32,
    pub width_increment: i32,
    pub title_x: i32,
    pub title_y: i32,
    pub height_increment: i32,
    pub element_resistance: i32,
    pub name: Vec<u8>,
    pub script: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryFlagBuild {
    pub picture_id: i32,
    pub direction: i32,
    pub max_hp: i32,
    pub defence: i32,
    pub width_increment: i32,
    pub title_x: i32,
    pub title_y: i32,
    pub height_increment: i32,
    pub element_resistance: i32,
    pub name: Vec<u8>,
    pub script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountryAreaState {
    pub id: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountrySecurityError {
    Cell(RegionCellAccessBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryReturnPointError {
    Base(ServerReturnSetupBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryEntryError {
    ReturnPoint(CountryReturnPointError),
    Cell(RegionCellAccessBlock),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CountryGuardRefreshTargets {
    pub monster_ids: Vec<i32>,
    pub spawn_indices: Vec<CountryGuardSpawnTarget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryGuardSpawnTarget {
    pub spawn_index: i32,
    pub camp: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CountryClearRefreshEffects {
    pub guard_targets: CountryGuardRefreshTargets,
    pub build_updates: Vec<BuildClientPublication>,
}

pub trait CountryReturnPointContext {
    /// Вызывает исходный `random(map.size())`; реакция на нулевой count
    /// остаётся у достигнутого RNG-owner-а, а returned DWORD используется
    /// map key.
    fn random_country_area_key(&mut self, area_count: u32) -> i32;
}

pub trait CountryEntryContext: CountryReturnPointContext + RegionRandomContext {
    /// Выполняет virtual player slot `+0x88` с `(x, y)`.
    fn set_country_player_position(&mut self, player_id: i32, x: i32, y: i32);
}

pub trait CountryCampContext {
    /// Возвращает `CPlayer::m_btCountry` только для существующего player ID.
    fn country_player_country(&mut self, player_id: i32) -> Option<u8>;
}

pub trait CountryContendEntryContext {
    /// Возвращает младшие 32 бита монотонного миллисекундного счётчика.
    fn now_millis(&mut self) -> u32;

    /// Шлёт player-у `0xBFF29` с одним signed значением времени.
    fn send_contend_time(&mut self, player_id: i32, time: i32);

    /// Меняет contend-state у уже известного non-null player pointer.
    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Шлёт player-localized `GS0228/GS0229` с исходным красным цветом.
    fn notify_player(&mut self, player_id: i32, string_id: &'static str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryContendPlayer {
    pub player_id: i32,
    pub faction_id: i32,
    pub country: u8,
    pub shape_type: i32,
    pub is_dead: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountryDamagePlayer {
    pub player_id: i32,
    pub max_hp: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountryMoveShape {
    pub object_type: i32,
    pub id: i32,
}

#[derive(Clone, Copy)]
pub enum CountryTargetKind {
    Gate,
    Flag,
    Guard,
}

/// BLOCKED_MISSING_FACT: для NaN/inf/out-of-range x87 `fistp i32` точная
/// реакция процесса не доказана; safe Rust не назначает ей saturating cast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryDamageArithmeticBlock {
    pub max_time: i32,
    pub damage: i32,
    pub max_hp: u32,
    pub dec_time_param_bits: u32,
}

/// BLOCKED_MISSING_FACT: RVA `0x001CCAC0` в null-ветке читает absolute
/// address `0x00000008`, а затем вызывает `CPlayer::SetContendState` с null.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryNullPlayerCancelBlock;

/// VERIFIED_DISASSEMBLY RVA `0x001CAFF0`: damage и unsigned max HP сначала
/// становятся f32; отношение с global factor сохраняется как f32, затем
/// `fild max_time`, умножение и `fistp i32` идут с truncation RC.
pub fn legacy_country_damage_decrement(
    max_time: i32,
    damage: i32,
    max_hp: u32,
    dec_time_param: f32,
) -> Result<i32, CountryDamageArithmeticBlock> {
    let block = || CountryDamageArithmeticBlock {
        max_time,
        damage,
        max_hp,
        dec_time_param_bits: dec_time_param.to_bits(),
    };

    let damage_as_float = damage as f32;
    let max_hp_as_float = max_hp as f32;
    let ratio = ((f64::from(damage_as_float) / f64::from(max_hp_as_float))
        * f64::from(dec_time_param)) as f32;
    let scaled = f64::from(max_time) * f64::from(ratio);
    if !scaled.is_finite() || scaled < f64::from(i32::MIN) || scaled >= 2_147_483_648.0_f64 {
        return Err(block());
    }
    Ok(scaled.trunc() as i32)
}

/// VERIFIED_DISASSEMBLY RVA `0x001CCAC0`: условие оригинала инвертировано;
/// любой реальный player немедленно получает `false` без side effects.
pub const fn decide_cancel_contend_by_player(
    player_present: bool,
) -> Result<bool, CountryNullPlayerCancelBlock> {
    if player_present {
        return Ok(false);
    }
    Err(CountryNullPlayerCancelBlock)
}

pub const fn country_gate_count_field(camp: i32) -> &'static str {
    match camp {
        COUNTRY_CAMP_DEFEND => "m_DefendGates count",
        COUNTRY_CAMP_ATTACK => "m_AttackGates count",
        _ => "invalid country gate camp count",
    }
}

pub const fn country_flag_count_field(camp: i32) -> &'static str {
    match camp {
        COUNTRY_CAMP_DEFEND => "m_DefendFlags count",
        COUNTRY_CAMP_ATTACK => "m_AttackFlags count",
        _ => "invalid country flag camp count",
    }
}

pub const fn country_area_count_field(camp: i32) -> &'static str {
    match camp {
        COUNTRY_CAMP_DEFEND => "m_DefendArea count",
        COUNTRY_CAMP_ATTACK => "m_AttackArea count",
        _ => "invalid country area camp count",
    }
}

/// VERIFIED_DISASSEMBLY RVA `0x001CD3F0`: country `tagGate` начинается с
/// picture ID; city `tagBuild` имеет дополнительный logical ID в field_00.
/// Последний DWORD `field_28` остаётся сознательно прочитанной частью
/// `bytes`, но исходная функция не переносит его в созданный gate.
pub fn country_gate_build_from_block(
    bytes: &[u8; 0x2C],
    name: Vec<u8>,
    script: Vec<u8>,
) -> CountryGateBuild {
    CountryGateBuild {
        picture_id: country_i32_at(bytes, 0x00),
        direction: country_i32_at(bytes, 0x04),
        action: LegacyReader::at(bytes, 0x08)
            .and_then(|mut reader| reader.read_u16())
            .expect("фиксированный country block содержит action"),
        max_hp: country_i32_at(bytes, 0x0C),
        defence: country_i32_at(bytes, 0x10),
        width_increment: country_i32_at(bytes, 0x14),
        title_x: country_i32_at(bytes, 0x18),
        title_y: country_i32_at(bytes, 0x1C),
        height_increment: country_i32_at(bytes, 0x20),
        element_resistance: country_i32_at(bytes, 0x24),
        name,
        script,
    }
}

/// VERIFIED_DISASSEMBLY RVA `0x001CD3F0`: field_00 — graphics ID;
/// field_04 — direction; fields_08..20 — шесть CBuild properties/position.
/// field_24 входит в wire, но не переносится в factory-объект.
pub fn country_flag_build_from_block(
    bytes: &[u8; 0x28],
    name: Vec<u8>,
    script: Vec<u8>,
) -> CountryFlagBuild {
    CountryFlagBuild {
        picture_id: country_i32_at(bytes, 0x00),
        direction: country_i32_at(bytes, 0x04),
        max_hp: country_i32_at(bytes, 0x08),
        defence: country_i32_at(bytes, 0x0C),
        width_increment: country_i32_at(bytes, 0x10),
        title_x: country_i32_at(bytes, 0x14),
        title_y: country_i32_at(bytes, 0x18),
        height_increment: country_i32_at(bytes, 0x1C),
        element_resistance: country_i32_at(bytes, 0x20),
        name,
        script,
    }
}

/// Byte-exact `tagArea` scalar block `0x14`.
pub fn country_area_state_from_block(bytes: &[u8; 0x14]) -> CountryAreaState {
    CountryAreaState {
        id: country_i32_at(bytes, 0x00),
        left: country_i32_at(bytes, 0x04),
        top: country_i32_at(bytes, 0x08),
        right: country_i32_at(bytes, 0x0C),
        bottom: country_i32_at(bytes, 0x10),
    }
}

impl ServerCountryRegionState {
    /// Writer-side region projection: defend записывается раньше attack.
    pub fn set_country_sides(&mut self, defend_country: i32, attack_country: i32) {
        self.defend_country = defend_country;
        self.attack_country = attack_country;
    }

    /// Country vtable `0x65D464`, slot `+0x104`, указывает на единственный
    /// `ret` по `0x485540`: после записи сторон эта разновидность региона не
    /// фильтрует contender-ов, в отличие от city/village war owners.
    pub const fn update_contend_player(&mut self) {}

    pub const fn country_side_bytes(&self) -> (u8, u8) {
        (self.defend_country as u8, self.attack_country as u8)
    }

    pub fn on_declare_begin(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id == region_id {
            self.declare_active = true;
        }
    }

    pub fn on_declare_end(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id == region_id {
            self.declare_active = false;
        }
    }

    pub fn on_prepare_begin(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id != region_id {
            return;
        }
        if !self.prepare_active {
            self.declare_active = true;
        }
    }

    pub fn on_prepare_end(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id != region_id {
            return;
        }
        if self.prepare_active {
            self.declare_active = false;
        }
    }

    pub fn on_war_start(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id == region_id {
            self.war_active = true;
        }
    }

    pub fn on_war_timeout(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id == region_id {
            self.war_active = false;
        }
    }

    /// VERIFIED_DISASSEMBLY: vtable `+0x150` совпадает с `OnTimeOut`
    /// `+0x14C` и указывает на одно RVA `0x001CAC60`.
    pub fn on_war_end(&mut self, region_id: i32, base_region_id: i32) {
        self.on_war_timeout(region_id, base_region_id);
    }

    pub fn on_flag_destroy(&mut self, region_id: i32, base_region_id: i32) {
        if base_region_id == region_id {
            self.war_active = false;
        }
    }

    pub const fn war_active(&self) -> bool {
        self.war_active
    }

    pub fn add_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        if let Some(guards) = self.guards_mut(camp) {
            guards.insert(monster_id);
        }
    }

    pub fn del_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        if let Some(guards) = self.guards_mut(camp) {
            guards.remove(&monster_id);
        }
    }

    pub fn add_guard_index(&mut self, spawn_index: i32, camp: i32) {
        if let Some(indices) = self.guard_indices_mut(camp) {
            indices.insert(spawn_index);
        }
    }

    /// Guard membership-collection для attackability lookups: только guard
    /// camp-колонки живут здесь, gates/flags остаются у aggregate.
    pub fn guard_collection_contains(&self, camp: i32, id: i32) -> bool {
        match camp {
            COUNTRY_CAMP_DEFEND => self.defend_guards.contains(&id),
            COUNTRY_CAMP_ATTACK => self.attack_guards.contains(&id),
            _ => false,
        }
    }

    pub fn guard_refresh_targets(&self) -> CountryGuardRefreshTargets {
        CountryGuardRefreshTargets {
            monster_ids: self
                .defend_guards
                .iter()
                .chain(&self.attack_guards)
                .copied()
                .collect(),
            spawn_indices: self
                .defend_guard_indices
                .iter()
                .map(|&spawn_index| CountryGuardSpawnTarget {
                    spawn_index,
                    camp: COUNTRY_CAMP_DEFEND,
                })
                .chain(self.attack_guard_indices.iter().map(|&spawn_index| {
                    CountryGuardSpawnTarget {
                        spawn_index,
                        camp: COUNTRY_CAMP_ATTACK,
                    }
                }))
                .collect(),
        }
    }

    /// Exact `GetCamp` RVA `0x001CE540`: нулевой player и отсутствующая
    /// country дают `-1`; defend-камп проверяется первым, иначе attack.
    pub fn get_camp<Context: CountryCampContext>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> i32 {
        if player_id == 0 {
            return -1;
        }
        let Some(country) = context.country_player_country(player_id) else {
            return -1;
        };
        if i32::from(country) == self.defend_country {
            return COUNTRY_CAMP_DEFEND;
        }
        if i32::from(country) == self.attack_country {
            COUNTRY_CAMP_ATTACK
        } else {
            -1
        }
    }

    pub fn is_win_symbol(&self, country: i32, symbol_id: i32) -> bool {
        self.symbol_hold.get(&symbol_id) == Some(&country)
    }

    pub fn capture_symbol(&mut self, symbol_id: i32, country: i32) {
        self.symbol_hold.insert(symbol_id, country);
    }

    /// Fight-phase return point: сторона выбирает owner area-map, исходный
    /// `random(map.size())` возвращает map key, а missing key — mutating
    /// `operator[]` zeroed/default `tagArea`. Оба два созвучных решения —
    /// нулевые стороны safe constructor-а до первого
    /// `CountryWarSys::update_apply_war` (exact ctor оставлял здесь UB) и
    /// fallback при чужой country — сохранены.
    pub fn war_return_point<Context: CountryReturnPointContext>(
        &mut self,
        region_id: i32,
        player_country: u8,
        context: &mut Context,
    ) -> Option<RegionReturnPoint> {
        let areas = if i32::from(player_country) == self.defend_country {
            &mut self.defend_areas
        } else if i32::from(player_country) == self.attack_country {
            &mut self.attack_areas
        } else {
            return None;
        };
        let area_key = context.random_country_area_key(areas.len() as u32);
        // Original `std::map::operator[]` вставлял zeroed/default `tagArea`,
        // если RNG key отсутствовал среди загруженных IDs.
        let area = areas.entry(area_key).or_default();
        Some(RegionReturnPoint {
            region_id,
            left: area.left,
            top: area.top,
            right: area.right,
            bottom: area.bottom,
            direction: -1,
        })
    }

    /// Entry path клиентского `0x9050B`: camp выбирает ordered area-map,
    /// `random(size)` является порядковым индексом iterator-а, а не map key.
    /// Пустая карта и недостигнутый индекс сохраняют исходный silent no-op.
    pub fn war_entry_area<Context: RegionRandomContext>(
        &self,
        camp: i32,
        context: &mut Context,
    ) -> Option<CountryAreaState> {
        let areas = match camp {
            COUNTRY_CAMP_DEFEND => &self.defend_areas,
            COUNTRY_CAMP_ATTACK => &self.attack_areas,
            _ => return None,
        };
        let index = context.random_below(areas.len() as i32);
        usize::try_from(index)
            .ok()
            .and_then(|index| areas.values().nth(index))
            .copied()
    }

    pub fn insert_decoded_area(&mut self, camp: i32, area: CountryAreaState) {
        self.areas_mut(camp)
            .expect("decoder передаёт только доказанный camp")
            .insert(area.id, area);
    }

    fn guards_mut(&mut self, camp: i32) -> Option<&mut BTreeSet<i32>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&mut self.defend_guards),
            COUNTRY_CAMP_ATTACK => Some(&mut self.attack_guards),
            _ => None,
        }
    }

    fn guard_indices_mut(&mut self, camp: i32) -> Option<&mut BTreeSet<i32>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&mut self.defend_guard_indices),
            COUNTRY_CAMP_ATTACK => Some(&mut self.attack_guard_indices),
            _ => None,
        }
    }

    fn areas_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CountryAreaState>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&mut self.defend_areas),
            COUNTRY_CAMP_ATTACK => Some(&mut self.attack_areas),
            _ => None,
        }
    }
}

fn country_i32_at<const N: usize>(bytes: &[u8; N], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("фиксированный country gate block содержит поле")
}
