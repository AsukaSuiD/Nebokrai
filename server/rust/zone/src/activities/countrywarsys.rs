//! GameServer-владелец country-war side state `CountryWarSys` в Zone `activities/` (локальное исполнение войн).
//!
//! Восстановлены constructor, snapshot decoder, side queries, region
//! lookup/init, `update_apply_war`, phase и victory chains; исходник
//! `country/countrywarsys.cpp`, точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Layout
//! `CountryWarRegion { defend:i32, attack:i32, clear:bool+padding }`, map-order,
//! signed IDs и writer-before-region-callback подтверждены PDB и точечным
//! дизассемблированием.
//!
//! `BTreeMap` заменяет `std::map` с тем же key-order. Decoder очищает map до
//! чтения count и сохраняет частичные эффекты/cursor при коротком payload.
//! `init_country_region_state` вопреки имени только делает lookup каждого
//! region: exact EXE после успешного `find` вызывает не меняющий существующий
//! entry `operator[]` и не переносит side state. `update_apply_war` сначала
//! меняет запись, затем найденный region и только после двух country-полей
//! вызывает `UpdateContendPlayer()` slot `+0x104`.
//!
//! Phase chain сохраняет точный map-order и порядок global-state мутаций:
//! declare сначала обнуляет country result `1..4`, start/timeout и prepare
//! работают только для записей с двумя ненулевыми сторонами, end сначала
//! сбрасывает три global-флага и стороны каждой записи, clear вызывает готовый
//! region owner для всех записей. Timeout после callback читает low byte обеих
//! сторон именно из region и обнуляет соответствующие country results.
//! Vtable slots `+0x138..+0x150`, включая общий callback `OnTimeOut/OnEnd`,
//! подтверждены точным EXE.
//!
//! Victory chain `on_flag_destroy` сначала ищет war-region по
//! входной стране и без найденного non-null region не делает ничего. Затем
//! region callback гасит war-state, входная страна получает result `2`, а
//! найденная через `GetOtherCountry` — result `1`; обе country-map keys
//! усекаются до low byte. Потерянные декомпилятором stack-присваивания и точный
//! порядок двух writes подтверждены дизассемблированием. Process singleton
//! allocation заменён owned-полем `CGame`.

use std::collections::BTreeMap;
use thiserror::Error;

use nebokrai_shared::protocol::LegacyReader;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CountryWarDecodeError {
    #[error("поле {field} с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountryWarRegion {
    pub defend_country: i32,
    pub attack_country: i32,
    pub state_clear: bool,
    state_clear_padding: [u8; 3],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CountryWarSys {
    pub state_declare: bool,
    pub state_war: bool,
    pub state_prepare: bool,
    pub war_regions: BTreeMap<i32, CountryWarRegion>,
}

pub trait CountryWarRegionContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Последовательно пишет `_defend_country`, затем `_attack_country`.
    fn set_country_sides(&mut self, region: Self::Region, defend_country: i32, attack_country: i32);

    /// Вызывает virtual `CServerRegion::UpdateContendPlayer()` slot `+0x104`.
    fn update_contend_player(&mut self, region: Self::Region);
}

pub trait CountryWarStartupContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;
}

pub trait CountryWarPhaseContext {
    type Region: Copy;
    type SideError;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_declare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_war_start(&mut self, region: Self::Region, region_id: i32);
    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32);
    fn on_war_end(&mut self, region: Self::Region, region_id: i32);
    fn clear_country_region(&mut self, region: Self::Region);

    /// Читает после timeout low byte `_defend_country`, затем `_attack_country`.
    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError>;

    /// Для существующей страны ставит `m_lCountryWarRes = 0`; miss/null — no-op.
    fn reset_country_war_result(&mut self, country: u8);
}

pub trait CountryWarVictoryContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Передаёт region ID и исходный country long в concrete region owner.
    fn on_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32);

    /// Пишет `m_lCountryWarRes`; отсутствующая/null country — no-op.
    fn set_country_war_result(&mut self, country: u8, result: i32);
}

impl CountryWarSys {
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), CountryWarDecodeError> {
        self.war_regions.clear();
        let count = read_country_war_i32(source, cursor, "m_CountryWarRegion count")?;
        for _ in 0..count.max(0) {
            let region_id = read_country_war_i32(source, cursor, "CountryWarRegion key")?;
            let bytes =
                read_country_war_array::<0x0C>(source, cursor, "CountryWarRegion scalar block")?;
            let mut reader = LegacyReader::new(&bytes);
            self.war_regions.insert(
                region_id,
                CountryWarRegion {
                    defend_country: reader.read_i32().expect("проверен блок CountryWarRegion"),
                    attack_country: reader.read_i32().expect("проверен блок CountryWarRegion"),
                    state_clear: bytes[8] != 0,
                    state_clear_padding: bytes[9..12].try_into().unwrap(),
                },
            );
        }
        tracing::trace!(
            scheduled_regions = self.war_regions.len(),
            "расписание войны стран декодировано"
        );
        Ok(())
    }

    /// Возвращает первый по map-order region с обеими нулевыми сторонами.
    pub fn get_idle_war_region(&self) -> i32 {
        self.war_regions
            .iter()
            .find_map(|(&region_id, state)| {
                (state.defend_country == 0 && state.attack_country == 0).then_some(region_id)
            })
            .unwrap_or(0)
    }

    pub fn is_already_declar(&self, country: i32) -> bool {
        self.war_regions
            .values()
            .any(|state| state.defend_country == country || state.attack_country == country)
    }

    pub fn get_war_region_for_country(&self, country: i32) -> i32 {
        self.war_regions
            .iter()
            .find_map(|(&region_id, state)| {
                (state.defend_country == country || state.attack_country == country)
                    .then_some(region_id)
            })
            .unwrap_or(0)
    }

    pub fn get_other_country(&self, country: i32) -> i32 {
        self.war_regions
            .values()
            .find_map(|state| {
                if state.defend_country == country {
                    Some(state.attack_country)
                } else if state.attack_country == country {
                    Some(state.defend_country)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn get_war_camp(&self, country: i32) -> i32 {
        self.war_regions
            .values()
            .find_map(|state| {
                if state.defend_country == country {
                    Some(0)
                } else if state.attack_country == country {
                    Some(1)
                } else {
                    None
                }
            })
            .unwrap_or(-1)
    }

    pub fn init_country_region_state<Context: CountryWarStartupContext>(
        &self,
        context: &mut Context,
    ) {
        let mut resolved_regions = 0usize;
        for &region_id in self.war_regions.keys() {
            // Успешный `find` сопровождает `operator[]`, но result
            // отбрасывается и region не мутируется.
            if context.find_country_region(region_id).is_some() {
                resolved_regions += 1;
            }
        }
        tracing::trace!(
            scheduled_regions = self.war_regions.len(),
            resolved_regions,
            "состояние регионов войны стран инициализировано"
        );
    }

    pub fn update_apply_war<Context: CountryWarRegionContext>(
        &mut self,
        region_id: i32,
        defend_country: i32,
        attack_country: i32,
        context: &mut Context,
    ) -> bool {
        let Some(state) = self.war_regions.get_mut(&region_id) else {
            return false;
        };
        state.defend_country = defend_country;
        state.attack_country = attack_country;

        if let Some(region) = context.find_country_region(region_id) {
            context.set_country_sides(region, defend_country, attack_country);
            context.update_contend_player(region);
        }
        true
    }

    pub fn on_declare_begin<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        for country in 1..5 {
            context.reset_country_war_result(country);
        }
        for &region_id in self.war_regions.keys() {
            if let Some(region) = context.find_country_region(region_id) {
                context.on_declare_begin(region, region_id);
            }
        }
        self.state_declare = true;
    }

    pub fn on_declare_end<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        for (&region_id, state) in &self.war_regions {
            if !country_war_is_active(state) {
                continue;
            }
            if let Some(region) = context.find_country_region(region_id) {
                context.on_declare_end(region, region_id);
            }
        }
        self.state_declare = false;
    }

    pub fn on_prepare_begin<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        for (&region_id, state) in &self.war_regions {
            if !country_war_is_active(state) {
                continue;
            }
            if let Some(region) = context.find_country_region(region_id) {
                context.on_prepare_begin(region, region_id);
            }
        }
        self.state_prepare = true;
    }

    pub fn on_prepare_end<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        for (&region_id, state) in &self.war_regions {
            if !country_war_is_active(state) {
                continue;
            }
            if let Some(region) = context.find_country_region(region_id) {
                context.on_prepare_end(region, region_id);
            }
        }
        self.state_prepare = false;
    }

    pub fn on_war_start<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        for (&region_id, state) in &self.war_regions {
            if !country_war_is_active(state) {
                continue;
            }
            if let Some(region) = context.find_country_region(region_id) {
                context.on_war_start(region, region_id);
            }
        }
        self.state_war = true;
    }

    pub fn on_war_timeout<Context: CountryWarPhaseContext>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), Context::SideError> {
        for (&region_id, state) in &self.war_regions {
            if !country_war_is_active(state) {
                continue;
            }
            let Some(region) = context.find_country_region(region_id) else {
                continue;
            };
            context.on_war_timeout(region, region_id);
            let (defend_country, attack_country) = context.country_region_side_bytes(region)?;
            context.reset_country_war_result(defend_country);
            context.reset_country_war_result(attack_country);
        }
        self.state_war = false;
        Ok(())
    }

    pub fn on_war_end<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        self.state_war = false;
        self.state_prepare = false;
        self.state_declare = false;
        for (&region_id, state) in &mut self.war_regions {
            state.defend_country = 0;
            state.attack_country = 0;
            if let Some(region) = context.find_country_region(region_id) {
                // Vtable slots `+0x14C` и `+0x150` у ServerCountryRegion
                // указывают на один callback.
                context.on_war_end(region, region_id);
            }
        }
    }

    pub fn on_war_clear<Context: CountryWarPhaseContext>(&self, context: &mut Context) {
        for &region_id in self.war_regions.keys() {
            if let Some(region) = context.find_country_region(region_id) {
                context.clear_country_region(region);
            }
        }
    }

    pub fn on_flag_destroy<Context: CountryWarVictoryContext>(
        &self,
        country: i32,
        context: &mut Context,
    ) {
        let region_id = self.get_war_region_for_country(country);
        let Some(region) = context.find_country_region(region_id) else {
            return;
        };

        context.on_flag_destroy(region, region_id, country);
        let other_country = self.get_other_country(country);
        // Сначала low byte входного country получает `2`, затем low byte
        // GetOtherCountry получает `1`.
        context.set_country_war_result(country as u8, 2);
        context.set_country_war_result(other_country as u8, 1);
    }
}

fn country_war_is_active(state: &CountryWarRegion) -> bool {
    state.defend_country != 0 && state.attack_country != 0
}

fn read_country_war_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CountryWarDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        CountryWarDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader
        .read_i32()
        .map_err(|block| country_war_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_country_war_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], CountryWarDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        CountryWarDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: N,
            available: block.available,
        }
    })?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| country_war_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn country_war_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> CountryWarDecodeError {
    CountryWarDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
