//! GameServer-владелец country-war side state `CountryWarSys`.
//!
//! Constructor `0x000EBF60`, snapshot decoder `0x000EBD60`, side queries
//! `0x000EB700..0x000EB870`, region lookup/init `0x000EBE10/0x000EC010` и
//! writer `update_apply_war` `0x000EC0B0` и phase chain
//! `0x000EC120..0x000EC770` имеют статус `IMPLEMENTED`; исходник
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
//! Vtable slots `+0x138..+0x150`, включая совпадающие `OnTimeOut/OnEnd` RVA,
//! подтверждены точным EXE.
//!
//! Victory chain `on_flag_destroy` RVA `0x000EBE60` сначала ищет war-region по
//! входной стране и без найденного non-null region не делает ничего. Затем
//! region callback гасит war-state, входная страна получает result `2`, а
//! найденная через `GetOtherCountry` — result `1`; обе country-map keys
//! усекаются до low byte. Потерянные декомпилятором stack-присваивания и точный
//! порядок двух writes подтверждены дизассемблированием. Singleton allocation
//! остаётся сырой технической поверхностью.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for CountryWarDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for CountryWarDecodeError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarRegion {
    pub(crate) defend_country: i32,
    pub(crate) attack_country: i32,
    pub(crate) state_clear: bool,
    state_clear_padding: [u8; 3],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarSys {
    pub(crate) state_declare: bool,
    pub(crate) state_war: bool,
    pub(crate) state_prepare: bool,
    pub(crate) war_regions: BTreeMap<i32, CountryWarRegion>,
}

pub(crate) trait CountryWarRegionContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Последовательно пишет `_defend_country`, затем `_attack_country`.
    fn set_country_sides(&mut self, region: Self::Region, defend_country: i32, attack_country: i32);

    /// Вызывает virtual `CServerRegion::UpdateContendPlayer()` slot `+0x104`.
    fn update_contend_player(&mut self, region: Self::Region);
}

pub(crate) trait CountryWarPhaseContext {
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

pub(crate) trait CountryWarVictoryContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Передаёт region ID и исходный country long в concrete region owner.
    fn on_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32);

    /// Пишет `m_lCountryWarRes`; отсутствующая/null country — no-op.
    fn set_country_war_result(&mut self, country: u8, result: i32);
}

impl CountryWarSys {
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, CountryWarDecodeError> {
        self.war_regions.clear();
        let count = read_country_war_i32(source, cursor, "m_CountryWarRegion count")?;
        for _ in 0..count.max(0) {
            let region_id = read_country_war_i32(source, cursor, "CountryWarRegion key")?;
            let bytes =
                read_country_war_array::<0x0C>(source, cursor, "CountryWarRegion scalar block")?;
            self.war_regions.insert(
                region_id,
                CountryWarRegion {
                    defend_country: i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    attack_country: i32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                    state_clear: bytes[8] != 0,
                    state_clear_padding: bytes[9..12].try_into().unwrap(),
                },
            );
        }
        Ok(true)
    }

    /// Возвращает первый по map-order region с обеими нулевыми сторонами.
    pub(crate) fn get_idle_war_region(&self) -> i32 {
        self.war_regions
            .iter()
            .find_map(|(&region_id, state)| {
                (state.defend_country == 0 && state.attack_country == 0).then_some(region_id)
            })
            .unwrap_or(0)
    }

    pub(crate) fn is_already_declar(&self, country: i32) -> bool {
        self.war_regions
            .values()
            .any(|state| state.defend_country == country || state.attack_country == country)
    }

    pub(crate) fn get_war_region_for_country(&self, country: i32) -> i32 {
        self.war_regions
            .iter()
            .find_map(|(&region_id, state)| {
                (state.defend_country == country || state.attack_country == country)
                    .then_some(region_id)
            })
            .unwrap_or(0)
    }

    pub(crate) fn get_other_country(&self, country: i32) -> i32 {
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

    pub(crate) fn get_war_camp(&self, country: i32) -> i32 {
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

    pub(crate) fn init_country_region_state<Context: CountryWarRegionContext>(
        &self,
        context: &mut Context,
    ) {
        for &region_id in self.war_regions.keys() {
            // VERIFIED_DISASSEMBLY RVA 0x000EC010: успешный `find` сопровождает
            // `operator[]`, но result отбрасывается и region не мутируется.
            let _ = context.find_country_region(region_id);
        }
    }

    pub(crate) fn update_apply_war<Context: CountryWarRegionContext>(
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

    pub(crate) fn on_declare_begin<Context: CountryWarPhaseContext>(
        &mut self,
        context: &mut Context,
    ) {
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

    pub(crate) fn on_declare_end<Context: CountryWarPhaseContext>(
        &mut self,
        context: &mut Context,
    ) {
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

    pub(crate) fn on_prepare_begin<Context: CountryWarPhaseContext>(
        &mut self,
        context: &mut Context,
    ) {
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

    pub(crate) fn on_prepare_end<Context: CountryWarPhaseContext>(
        &mut self,
        context: &mut Context,
    ) {
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

    pub(crate) fn on_war_start<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
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

    pub(crate) fn on_war_timeout<Context: CountryWarPhaseContext>(
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

    pub(crate) fn on_war_end<Context: CountryWarPhaseContext>(&mut self, context: &mut Context) {
        self.state_war = false;
        self.state_prepare = false;
        self.state_declare = false;
        for (&region_id, state) in &mut self.war_regions {
            state.defend_country = 0;
            state.attack_country = 0;
            if let Some(region) = context.find_country_region(region_id) {
                // VERIFIED_DISASSEMBLY: vtable slots `+0x14C` и `+0x150`
                // у ServerCountryRegion оба указывают на RVA `0x001CAC60`.
                context.on_war_end(region, region_id);
            }
        }
    }

    pub(crate) fn on_war_clear<Context: CountryWarPhaseContext>(&self, context: &mut Context) {
        for &region_id in self.war_regions.keys() {
            if let Some(region) = context.find_country_region(region_id) {
                context.clear_country_region(region);
            }
        }
    }

    pub(crate) fn on_flag_destroy<Context: CountryWarVictoryContext>(
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
        // VERIFIED_DISASSEMBLY RVA 0x000EBE60: сначала low byte входного
        // country получает `2`, затем low byte GetOtherCountry получает `1`.
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
    Ok(i32::from_le_bytes(read_country_war_array(
        source, cursor, field,
    )?))
}

fn read_country_war_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], CountryWarDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    if available < N {
        return Err(CountryWarDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    }
    *cursor = offset + N;
    Ok(source[offset..offset + N]
        .try_into()
        .expect("длина проверена до копирования country-war блока"))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp

// ============================================================================
// FUNCTION: tagAttackCityTime::~tagAttackCityTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp
// RVA: 0x0005FAC0
// ADDRESS: 0045fac0
// PROTOTYPE: void __thiscall ~tagAttackCityTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAttackCityTime::tagAttackCityTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp
// RVA: 0x0005FB50
// ADDRESS: 0045fb50
// PROTOTYPE: undefined __thiscall tagAttackCityTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAttackCityTime::tagAttackCityTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp
// RVA: 0x00060520
// ADDRESS: 00460520
// PROTOTYPE: undefined __thiscall tagAttackCityTime(tagAttackCityTime * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAttackCityTime::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp
// RVA: 0x000606C0
// ADDRESS: 004606c0
// PROTOTYPE: tagAttackCityTime * __thiscall operator=(tagAttackCityTime * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004608ff
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp
// RVA: 0x000608FF
// ADDRESS: 004608ff
// PROTOTYPE: undefined Catch@004608ff()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::get_war_region
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:316
// RVA: 0x000EB700
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::is_already_declar
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:216
// RVA: 0x000EB770
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::get_war_region
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:345
// RVA: 0x000EB7C0
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::GetOtherCountry
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:356
// RVA: 0x000EB810
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::get_war_camp
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:372
// RVA: 0x000EB870
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:41
// RVA: 0x000EBD60
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::get_region
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:339
// RVA: 0x000EBE10
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_flag_destroy
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:388
// RVA: 0x000EBE60
//
// Реализовано выше в связной country victory chain; потерянные stack-locals
// и порядок result writes имеют статус VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: CountryWarSys::CountryWarSys
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:20
// RVA: 0x000EBF60
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::get_instance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:30
// RVA: 0x000EBFA0
// ADDRESS: 004ebfa0
// PROTOTYPE: CountryWarSys * __cdecl get_instance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::init_country_region_state
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:58
// RVA: 0x000EC010
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::update_apply_war
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:71
// RVA: 0x000EC0B0
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_war_start
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:94
// RVA: 0x000EC120
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_war_timeout
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:108
// RVA: 0x000EC1E0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_war_end
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:129
// RVA: 0x000EC350
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_war_clear
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:144
// RVA: 0x000EC410
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_declare_begin
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:157
// RVA: 0x000EC4D0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_declare_end
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:175
// RVA: 0x000EC5F0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_begin
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:189
// RVA: 0x000EC6B0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_end
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:201
// RVA: 0x000EC770
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: get_country_war_sys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countrywarsys.cpp:404
// RVA: 0x000EC830
// ADDRESS: 004ec830
// PROTOTYPE: CountryWarSys * __cdecl get_country_war_sys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: GameServer
