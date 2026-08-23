//! Мировой регион `CWorldRegion` из `worldregion.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Владелец объединяет базовый `CRegion`, девять полей `tagRegionParam`, setup,
//! NPC, monsters, weather и tax. Save snapshot копирует только параметры;
//! имя и прочее runtime-состояние в запись БД не входят.
//!
//! Полный serializer и proxy serializer намеренно различаются. Proxy пишет
//! базовый объект, country, war type и параметры; selective decoder обновляет
//! только tax и ownership поля. Общий `DecordFromByteArray` остаётся no-op.
//!
//! Setup читается позиционно до маркеров `<end>` и `#`; непрочитанные scalars
//! остаются `None`. NPC, monster и weather loaders сохраняют исходную token-
//! грамматику, порядок записей и byte-exact строки. Особый monster prefix
//! воспроизводит значимую часть старого MSVC string-layout; неопределённый хвост
//! обнуляется, а неподдерживаемый heap-вариант отклоняется.
//!
//! `InitOwnerRelation` последовательно проверяет faction и union, очищая только
//! несуществующие связи. Owned snapshots и стандартные коллекции заменяют
//! object slicing, STL и ручное владение без изменения wire или порядка загрузки.

use super::country::countryparam::CCountryParam;
use super::organizingsystem::faction::{
    FactionInitialPropertyBlock, OwnedCityAddOutcome, OwnedCityMutationBuildError,
};
use super::organizingsystem::organizingctrl::COrganizingCtrl;
use super::organizingsystem::villagewarsys::CVillageWarSys;
use super::player::CPlayer;
use super::region::{
    CRegion, RegionLoadError, RegionRandomPositionBlock, RegionSerializationBlock,
};
use std::cell::Cell;
use crate::dbaccess::worlddb::rsregion::RegionSaveSnapshot;
use crate::public::clientresource::DefaultClientResourceOwner;
use crate::worldserver::worldserver::game::CGame;

#[derive(Clone, Copy, Debug)]
struct RegionParamState {
    region_id: i32,
    max_tax_rate: i32,
    current_tax_rate: i32,
    total_tax: u32,
    today_total_tax: u32,
    superior_region_id: i32,
    turn_in_tax_rate: i32,
    owned_faction_id: i32,
    owned_union_id: i32,
}

#[derive(Clone, Debug)]
struct RegionSetupState {
    return_region_id: Option<i32>,
    return_left: Option<i32>,
    return_top: Option<i32>,
    return_right: Option<i32>,
    return_bottom: Option<i32>,
    recall_when_lost: Option<i32>,
    move_monster_when_refresh: Option<i32>,
    use_return_point: Option<i32>,
}

impl RegionSetupState {
    const fn uninitialized() -> Self {
        Self {
            return_region_id: None,
            return_left: None,
            return_top: None,
            return_right: None,
            return_bottom: None,
            recall_when_lost: None,
            move_monster_when_refresh: None,
            use_return_point: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionSetupSerializationBlock {
    pub(crate) field: &'static str,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldReturnPoint {
    pub(crate) region_id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) direction: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionEnterBlock {
    Setup(WorldRegionSetupSerializationBlock),
    UninitializedPlayerCountry,
    CoordinateOverflow { operation: &'static str },
    RandomPosition(RegionRandomPositionBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionOwnerRelationReport {
    pub(crate) initial_faction_id: i32,
    pub(crate) faction_cleared: bool,
    pub(crate) union_cleared: bool,
    pub(crate) owned_city: Option<OwnedCityAddOutcome>,
    pub(crate) country: Option<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionOwnerRelationBlock {
    OwnedCity {
        report: WorldRegionOwnerRelationReport,
        source: OwnedCityMutationBuildError,
    },
    Country {
        report: WorldRegionOwnerRelationReport,
        source: FactionInitialPropertyBlock,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionTextLoadError {
    MissingValue {
        field: &'static str,
    },
    InvalidValue {
        field: &'static str,
    },
    MonsterNameRequiresLegacyHeapLayout {
        length: usize,
    },
    ShortWeatherColorCode {
        length: usize,
    },
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
}

/// Resource/string граница, которую `CWorldRegion::Load` вызывает
/// последовательно и потому не разрешает caller-у заранее читать весь набор.
pub(crate) trait WorldRegionResourceContext {
 /// Единственный опубликованный World resource-owner, общий для reload и
 /// всех последующих `rfOpen`-эквивалентов этого context-а.
    fn default_client_resource(&mut self) -> &mut DefaultClientResourceOwner;

    fn read_resource(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        self.default_client_resource().read_resource(path)
    }
    fn region_monster_num_scale(&mut self) -> f32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionLoadedCounts {
    pub(crate) monsters: i32,
    pub(crate) npcs: i32,
    pub(crate) base_failure: Option<WorldRegionBaseLoadFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionBaseLoadFailure {
    MissingRegionResource,
    Region(RegionLoadError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionLoadError {
    Text {
        owner: &'static str,
        source: WorldRegionTextLoadError,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionSerializationBlock {
    Region(RegionSerializationBlock),
    Setup(WorldRegionSetupSerializationBlock),
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
    MonsterVariant {
        source: WorldRegionTextLoadError,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionParamDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldRegionNpc {
    show_list: bool,
    picture_id: i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    count: i32,
    direction: i32,
    time: i32,
    name: Vec<u8>,
    script: Vec<u8>,
}

impl WorldRegionNpc {
 /// Сохраняет исходные три padding bytes после old `bool` нулевыми,
 /// как прежняя safe Rust-реконструкция.
    fn wire_header(&self) -> [u8; 0x24] {
        let mut bytes = [0; 0x24];
        bytes[0] = u8::from(self.show_list);
        for (offset, value) in [
            (4, self.picture_id),
            (8, self.left),
            (0x0C, self.top),
            (0x10, self.right),
            (0x14, self.bottom),
            (0x18, self.count),
            (0x1C, self.direction),
            (0x20, self.time),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldRegionMonsterVariant {
    cumulative_odds: u16,
    sign: u16,
    leader_sign: u16,
    leader_distance: u16,
    name: Vec<u8>,
    script: Vec<u8>,
}

impl WorldRegionMonsterVariant {
    fn wire_prefix(&self) -> Result<[u8; 0x22], WorldRegionSerializationBlock> {
        legacy_monster_variant_prefix(
            self.cumulative_odds,
            self.sign,
            self.leader_sign,
            self.leader_distance,
            &self.name,
        )
        .map_err(|source| WorldRegionSerializationBlock::MonsterVariant { source })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldRegionMonster {
    index: i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    count: i32,
    reset_time: i32,
    start_time: i32,
    direction: i32,
    variants: Vec<WorldRegionMonsterVariant>,
}

impl WorldRegionMonster {
    const fn wire_scalars(&self) -> [i32; 9] {
        [
            self.index,
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.count,
            self.reset_time,
            self.start_time,
            self.direction,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
struct WorldRegionWeather {
    weather_index: i32,
    fog_color: u32,
}

#[derive(Clone, Debug)]
struct WorldRegionWeatherOption {
    cumulative_odds: i32,
    weather: Vec<WorldRegionWeather>,
}

#[derive(Clone, Debug)]
struct WorldRegionWeatherTime {
    time: i32,
    options: Vec<WorldRegionWeatherOption>,
}

impl RegionParamState {
    const fn zeroed() -> Self {
        Self {
            region_id: 0,
            max_tax_rate: 0,
            current_tax_rate: 0,
            total_tax: 0,
            today_total_tax: 0,
            superior_region_id: 0,
            turn_in_tax_rate: 0,
            owned_faction_id: 0,
            owned_union_id: 0,
        }
    }

    const fn save_snapshot(self) -> RegionSaveSnapshot {
        RegionSaveSnapshot {
            region_id: self.region_id,
            max_tax_rate: self.max_tax_rate,
            current_tax_rate: self.current_tax_rate,
            total_tax: self.total_tax,
            today_total_tax: self.today_total_tax,
            superior_region_id: self.superior_region_id,
            turn_in_tax_rate: self.turn_in_tax_rate,
            owned_faction_id: self.owned_faction_id,
            owned_union_id: self.owned_union_id,
        }
    }
}

pub(crate) struct CWorldRegion {
    region: CRegion,
    war_region_type: i32,
    no_pk: bool,
    no_contribute: bool,
    monsters: Vec<WorldRegionMonster>,
    npcs: Vec<WorldRegionNpc>,
    weather: Vec<WorldRegionWeatherTime>,
    param: RegionParamState,
 /// Organizing callbacks меняют только эту owner relation; `Cell`
 /// позволяет выполнить их немедленно через shared game-view, не создавая
 /// второго mutable alias всего `CGame`.
    owned_city_organizing: Cell<(i32, i32)>,
    setup: RegionSetupState,
    forbidden_make_goods: Vec<Vec<u8>>,
}

impl CWorldRegion {
    pub(crate) const fn with_constructor_region_base() -> Self {
        Self {
            region: CRegion::with_constructor_base_and_type(),
            war_region_type: 0,
            no_pk: false,
            no_contribute: false,
            monsters: Vec::new(),
            npcs: Vec::new(),
            weather: Vec::new(),
            param: RegionParamState::zeroed(),
            owned_city_organizing: Cell::new((0, 0)),
            setup: RegionSetupState::uninitialized(),
            forbidden_make_goods: Vec::new(),
        }
    }

    pub(crate) const fn creation_region_base(&self) -> &CRegion {
        &self.region
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.region.get_name()
    }

    pub(crate) fn generate_save_data(&self) -> RegionSaveSnapshot {
        self.effective_param().save_snapshot()
    }

    fn effective_param(&self) -> RegionParamState {
        let mut param = self.param;
        (param.owned_faction_id, param.owned_union_id) = self.owned_city_organizing.get();
        param
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.region.get_id()
    }

    pub(crate) const fn region_base(&self) -> &CRegion {
        &self.region
    }

    pub(crate) const fn region_base_mut(&mut self) -> &mut CRegion {
        &mut self.region
    }

 /// Возвращает три значения City Load guard без выбора реакции для
 /// constructor-uninitialized setup.
    pub(crate) fn city_load_base_guard(
        &self,
    ) -> Result<(bool, i32, i32), WorldRegionSetupSerializationBlock> {
        let use_return_point = self
            .setup
            .use_return_point
            .ok_or(WorldRegionSetupSerializationBlock { field: "bUse" })?;
        let return_region_id =
            self.setup
                .return_region_id
                .ok_or(WorldRegionSetupSerializationBlock {
                    field: "lReturnRegionID",
                })?;
        Ok((use_return_point != 0, return_region_id, self.get_id()))
    }

    pub(crate) fn get_return_point(
        &self,
        player: Option<&CPlayer>,
        country_param: &mut CCountryParam,
    ) -> Result<WorldReturnPoint, WorldRegionEnterBlock> {
        let Some(player) = player else {
            return Ok(WorldReturnPoint::default());
        };
        let use_return_point = self
            .setup
            .use_return_point
            .ok_or(WorldRegionEnterBlock::Setup(
                WorldRegionSetupSerializationBlock { field: "bUse" },
            ))?;
        if use_return_point != 0 {
            let field = |value: Option<i32>, field| {
                value.ok_or(WorldRegionEnterBlock::Setup(
                    WorldRegionSetupSerializationBlock { field },
                ))
            };
            return Ok(WorldReturnPoint {
                region_id: field(self.setup.return_region_id, "lReturnRegionID")?,
                left: field(self.setup.return_left, "rtReturnPoint.left")?,
                top: field(self.setup.return_top, "rtReturnPoint.top")?,
                right: field(self.setup.return_right, "rtReturnPoint.right")?,
                bottom: field(self.setup.return_bottom, "rtReturnPoint.bottom")?,
                direction: -1,
            });
        }

        let country = player
            .country()
            .ok_or(WorldRegionEnterBlock::UninitializedPlayerCountry)?;
        let point = country_param.main_return_point(country);
        Ok(WorldReturnPoint {
            region_id: point.region_id,
            left: point.rect.left,
            top: point.rect.top,
            right: point.rect.right,
            bottom: point.rect.bottom,
            direction: point.direction,
        })
    }

    pub(crate) fn set_enter_pos_xy<'a, FindRegion, Random>(
        &self,
        player: Option<&mut CPlayer>,
        country_param: &mut CCountryParam,
        village_war: &CVillageWarSys,
        mut find_region: FindRegion,
        mut random: Random,
    ) -> Result<(), WorldRegionEnterBlock>
    where
        FindRegion: FnMut(i32) -> Option<&'a CRegion>,
        Random: FnMut(i32) -> i32,
    {
        let Some(player) = player else {
            return Ok(());
        };
        if !village_war.is_village_region_left_clear_time(self.get_id()) {
            return Ok(());
        }

        let point = self.get_return_point(Some(&*player), country_param)?;
        let mut tile_x = -1;
        let mut tile_y = -1;
        if let Some(region) = find_region(point.region_id) {
            let span_x = point.right.checked_sub(point.left).ok_or(
                WorldRegionEnterBlock::CoordinateOverflow {
                    operation: "return right - left",
                },
            )?;
            let span_y = point.bottom.checked_sub(point.top).ok_or(
                WorldRegionEnterBlock::CoordinateOverflow {
                    operation: "return bottom - top",
                },
            )?;
            let position = region
                .get_random_pos_in_range(point.left, point.top, span_x, span_y, &mut random)
                .map_err(WorldRegionEnterBlock::RandomPosition)?;
            tile_x = position.x;
            tile_y = position.y;
        }
        player.set_region_id(point.region_id);
        player.set_tile_xy(tile_x, tile_y);
        Ok(())
    }

    pub(crate) fn set_region_identity(&mut self, region_id: i32, name: &[u8]) {
        self.region.set_id(region_id);
        self.region.set_name(name);
    }

    pub(crate) fn set_region_list_base_fields(
        &mut self,
        resource_id: u32,
        exp_scale: f32,
        country: u8,
        notify: i32,
    ) {
        self.region
            .set_region_list_fields(resource_id, exp_scale, country, notify);
    }

    pub(crate) const fn set_world_region_list_fields(
        &mut self,
        war_region_type: i32,
        no_pk: bool,
        no_contribute: bool,
    ) {
        self.war_region_type = war_region_type;
        self.no_pk = no_pk;
        self.no_contribute = no_contribute;
    }

    pub(crate) fn load_region_resource(
        &mut self,
        path: &[u8],
        bytes: &[u8],
    ) -> Result<bool, RegionLoadError> {
        self.region.load_from_resource(path, bytes)
    }

    pub(crate) fn load_npc_bytes<ResolveName>(
        &mut self,
        bytes: &[u8],
        mut resolve_name: ResolveName,
    ) -> Result<i32, WorldRegionTextLoadError>
    where
        ResolveName: FnMut(&[u8]) -> Vec<u8>,
    {
        let mut tokens = RegionSetupTokens::new(bytes);
        let mut loaded = Vec::new();
        while tokens.seek_to(b"#") {
            let visible = tokens.next_i32_field("tagNpc.bShowList")? != 0;
            let left = tokens.next_i32_field("tagNpc.left")?;
            let top = tokens.next_i32_field("tagNpc.top")?;
            let right = tokens.next_i32_field("tagNpc.right")?;
            let bottom = tokens.next_i32_field("tagNpc.bottom")?;
            let string_id = tokens.next_bytes_field("tagNpc.stringID")?;
            let graphics_id = tokens.next_i32_field("tagNpc.lPicID")?;
            let count = tokens.next_i32_field("tagNpc.lNum")?;
            let direction = tokens.next_i32_field("tagNpc.lDir")?;
            let script = normalize_legacy_script(tokens.next_bytes_field("tagNpc.strScript")?);

            loaded.push(WorldRegionNpc {
                show_list: visible,
                picture_id: graphics_id,
                left,
                top,
                right,
                bottom,
                count,
                direction,
                time: 0,
                name: legacy_c_string_prefix(&resolve_name(string_id)).to_vec(),
                script,
            });
        }
        let count =
            i32::try_from(loaded.len()).map_err(|_| WorldRegionTextLoadError::TooManyEntries {
                collection: "m_listNpc",
                count: loaded.len(),
            })?;
        self.npcs = loaded;
        Ok(count)
    }

    pub(crate) fn load_monster_bytes(
        &mut self,
        bytes: &[u8],
        monster_num_scale: f32,
    ) -> Result<i32, WorldRegionTextLoadError> {
        let mut tokens = RegionSetupTokens::new(bytes);
        let mut monsters = Vec::<WorldRegionMonster>::new();
        while let Some(token) = tokens.next_bytes_optional() {
            if token == b"<end>" {
                break;
            }
            if token != b"#" {
                continue;
            }
            let mut fields = [0i32; 9];
            for (index, field) in fields.iter_mut().enumerate() {
                *field = tokens.next_i32_field(MONSTER_GROUP_FIELDS[index])?;
            }
            if fields[5] > 1 {
                let scaled = (fields[5] as f32 * monster_num_scale).trunc();
                fields[5] = scaled as i32;
                if fields[5] == 0 {
                    fields[5] = 1;
                }
            }
            fields[6] = fields[6].wrapping_mul(1000);
            fields[7] = fields[7].wrapping_mul(1000);
            monsters.push(WorldRegionMonster {
                index: fields[0],
                left: fields[1],
                top: fields[2],
                right: fields[3],
                bottom: fields[4],
                count: fields[5],
                reset_time: fields[6],
                start_time: fields[7],
                direction: fields[8],
                variants: Vec::new(),
            });
        }

        let mut current = None;
        let mut cumulative_odds = 0u16;
        while let Some(token) = tokens.next_bytes_optional() {
            if token == b"id" {
                let id = tokens.next_i32_field("tagMonsterList.id")?;
                current = monsters.iter().position(|monster| monster.index == id);
                if current.is_none() {
 // Исходная странность поставки: если `id` не найден,
 // остаётся во внешнем scanner-е, пропускает
 // эту секцию до `<end>` и завершает весь variant-phase.
                    while let Some(token) = tokens.next_bytes_optional() {
                        if token == b"<end>" {
                            break;
                        }
                    }
                    break;
                }
                cumulative_odds = 0;
                continue;
            }
            if token == b"<end>" {
                current = None;
                continue;
            }
            if token != b"#" {
                continue;
            }
            let name = tokens.next_bytes_field("tagMonsterList.strName")?;
            let odds = tokens.next_u16_field("tagMonsterList.wOdds")?;
            let sign = tokens.next_u16_field("tagMonsterList.wSign")?;
            let leader_sign = tokens.next_u16_field("tagMonsterList.wLeaderSign")?;
            let leader_distance = tokens.next_u16_field("tagMonsterList.wLeaderDistance")?;
            let script =
                normalize_legacy_script(tokens.next_bytes_field("tagMonsterList.strScript")?);
            let Some(index) = current else {
                continue;
            };
            cumulative_odds = cumulative_odds.wrapping_add(odds);
            legacy_monster_variant_prefix(
                cumulative_odds,
                sign,
                leader_sign,
                leader_distance,
                name,
            )?;
            monsters[index].variants.push(WorldRegionMonsterVariant {
                cumulative_odds,
                sign,
                leader_sign,
                leader_distance,
                name: name.to_vec(),
                script,
            });
        }

        let total = monsters
            .iter()
            .fold(0i32, |sum, monster| sum.wrapping_add(monster.count));
        self.monsters = monsters;
        Ok(total)
    }

 /// Заменяет weather-vector; caller вызывает метод даже при missing resource,
 /// передавая `None`, потому что оригинал очищал vector до `rfOpen`.
    pub(crate) fn load_weather_bytes(
        &mut self,
        bytes: Option<&[u8]>,
    ) -> Result<(), WorldRegionTextLoadError> {
        self.weather.clear();
        let Some(bytes) = bytes else {
            return Ok(());
        };
        let mut tokens = RegionSetupTokens::new(bytes);
        while tokens.seek_to(b"time") {
            let time = tokens.next_i32_field("tagWeatherTime.lTime")?;
            let _num_marker = tokens.next_bytes_field("weather num marker")?;
            let option_count = tokens.next_i32_field("weather option count")?;
            let mut options = Vec::new();
            let mut cumulative_odds = 0i32;
            for _ in 0..option_count.max(0) {
                let odds = tokens.next_i32_field("tagOption.lOdds")?;
                let weather_count = tokens.next_i32_field("weather count")?;
                cumulative_odds = cumulative_odds.wrapping_add(odds);
                let mut weather = Vec::new();
                for _ in 0..weather_count.max(0) {
                    let weather_index = tokens.next_i32_field("tagWeather.lWeatherIndex")?;
                    let fog_color = if weather_index / 100 == 4 {
                        let color = tokens.next_bytes_field("tagWeather.dwFogColor")?;
                        translate_color_code(color)? | 0xFF00_0000
                    } else {
                        0
                    };
                    weather.push(WorldRegionWeather {
                        weather_index,
                        fog_color,
                    });
                }
                options.push(WorldRegionWeatherOption {
                    cumulative_odds,
                    weather,
                });
            }
            self.weather.push(WorldRegionWeatherTime { time, options });
        }
        Ok(())
    }

    pub(crate) fn load_tax_bytes(
        &mut self,
        bytes: Option<&[u8]>,
    ) -> Result<bool, WorldRegionTextLoadError> {
        if let Some(bytes) = bytes {
            let mut tokens = RegionSetupTokens::new(bytes);
            if tokens.seek_to(b"*") {
                self.param.max_tax_rate = tokens.next_i32_field("m_Param.lMaxTaxRate")?;
                self.param.superior_region_id = tokens.next_i32_field("m_Param.lSupRegionID")?;
                self.param.turn_in_tax_rate = tokens.next_i32_field("m_Param.lTurnInTaxRate")?;
            }
        }
        self.param.region_id = self.region.get_id();
        Ok(true)
    }

 /// Выполняет точный ordered `CWorldRegion::Load` и возвращает приращения
 /// двух исходных process-global counters.
    pub(crate) fn load_from_context<Context, ResolveName>(
        &mut self,
        context: &mut Context,
        resolve_name: &mut ResolveName,
    ) -> Result<WorldRegionLoadedCounts, WorldRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
        ResolveName: FnMut(&[u8]) -> Vec<u8> + ?Sized,
    {
        let region_id = self.get_id();
        let region_path = format!("regions/{region_id}.rgn").into_bytes();
        let base_failure = match context.read_resource(&region_path) {
            Some(bytes) => self
                .load_region_resource(&region_path, &bytes)
                .err()
                .map(WorldRegionBaseLoadFailure::Region),
            None => Some(WorldRegionBaseLoadFailure::MissingRegionResource),
        };
        let monster_path = format!("regions/{region_id}.Monster").into_bytes();
        let monsters = match context.read_resource(&monster_path) {
            Some(bytes) => {
                let monster_num_scale = context.region_monster_num_scale();
                self.load_monster_bytes(&bytes, monster_num_scale)
                    .map_err(|source| WorldRegionLoadError::Text {
                        owner: "CWorldRegion::LoadMonsterList",
                        source,
                    })?
            }
            None => 0,
        };
        let npc_path = format!("regions/{region_id}.npc").into_bytes();
        let npcs = match context.read_resource(&npc_path) {
            Some(bytes) => self
                .load_npc_bytes(&bytes, &mut *resolve_name)
                .map_err(|source| WorldRegionLoadError::Text {
                    owner: "CWorldRegion::LoadNpcList",
                    source,
                })?,
            None => 0,
        };
        let weather_path = format!("regions/{region_id}.weather").into_bytes();
        let weather = context.read_resource(&weather_path);
        self.load_weather_bytes(weather.as_deref())
            .map_err(|source| WorldRegionLoadError::Text {
                owner: "CWorldRegion::LoadWeatherSetup",
                source,
            })?;
        let setup_path = format!("regions/{region_id}.rs").into_bytes();
        if let Some(bytes) = context.read_resource(&setup_path) {
            self.load_setup_bytes(&bytes);
        }
        if base_failure.is_none() {
            let tax_path = format!("regions/{region_id}.Tax").into_bytes();
            let tax = context.read_resource(&tax_path);
            let _ = self.load_tax_bytes(tax.as_deref()).map_err(|source| {
                WorldRegionLoadError::Text {
                    owner: "CWorldRegion::LoadTaxParam",
                    source,
                }
            })?;
        }
        Ok(WorldRegionLoadedCounts {
            monsters,
            npcs,
            base_failure,
        })
    }

    pub(crate) fn add_to_byte_array_for_proxy(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, RegionSerializationBlock> {
        let country =
            self.region
                .country()
                .ok_or(RegionSerializationBlock::UninitializedField {
                    field: "m_btCountry",
                })?;
        let _ = self
            .region
            .add_base_object_to_byte_array(destination, include_child);
        destination.push(country);
        destination.extend_from_slice(&self.war_region_type.to_le_bytes());
        append_region_param(destination, self.effective_param());
        Ok(true)
    }

 /// Читает полный `0x24`-байтовый снимок, но меняет только четыре поля,
 /// которые присваивал исходный `DecordRegionParamFromByteArray`.
    pub(crate) fn decord_region_param_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, WorldRegionParamDecodeError> {
        let offset = *cursor;
        let available = source.len().saturating_sub(offset);
        let Some(end) = offset.checked_add(0x24) else {
            return Err(WorldRegionParamDecodeError::UnexpectedEnd {
                offset,
                needed: 0x24,
                available,
            });
        };
        let Some(param) = source.get(offset..end) else {
 // Старый безразмерный helper читал 0x24
 // байта за caller-pointer; реакция на короткий источник неизвестна.
            return Err(WorldRegionParamDecodeError::UnexpectedEnd {
                offset,
                needed: 0x24,
                available,
            });
        };
        *cursor = end;
        self.param.current_tax_rate = read_param_i32(param, 0x08);
        self.param.total_tax = read_param_u32(param, 0x0C);
        self.param.today_total_tax = read_param_u32(param, 0x10);
        let owned_faction_id = read_param_i32(param, 0x1C);
        self.param.owned_faction_id = owned_faction_id;
        let (_, owned_union_id) = self.owned_city_organizing.get();
        self.owned_city_organizing
            .set((owned_faction_id, owned_union_id));
        Ok(true)
    }

    pub(crate) fn set_param_from_db(
        &mut self,
        owned_faction_id: i32,
        owned_union_id: i32,
        current_tax_rate: i32,
        today_total_tax: i32,
        total_tax: i32,
    ) {
        self.param.owned_faction_id = owned_faction_id;
        self.param.owned_union_id = owned_union_id;
        self.owned_city_organizing
            .set((owned_faction_id, owned_union_id));
        self.param.current_tax_rate = if current_tax_rate < self.param.max_tax_rate {
            current_tax_rate
        } else {
            self.param.max_tax_rate
        };
        self.param.today_total_tax = today_total_tax as u32;
        self.param.total_tax = total_tax as u32;
    }

    pub(crate) const fn set_param_from_gs(
        &mut self,
        current_tax_rate: i32,
        today_total_tax: u32,
        total_tax: u32,
    ) {
        self.param.current_tax_rate = current_tax_rate;
        self.param.today_total_tax = today_total_tax;
        self.param.total_tax = total_tax;
    }

    pub(crate) fn set_owned_city_org(&self, owned_faction_id: i32, owned_union_id: i32) {
        self.owned_city_organizing
            .set((owned_faction_id, owned_union_id));
    }

    pub(crate) fn get_owned_city_faction(&self) -> i32 {
        self.owned_city_organizing.get().0
    }

    pub(crate) fn get_owned_city_union(&self) -> i32 {
        self.owned_city_organizing.get().1
    }

    pub(crate) fn init_owner_relation(
        &mut self,
        organizing: &mut COrganizingCtrl,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<WorldRegionOwnerRelationReport, WorldRegionOwnerRelationBlock> {
        let initial_faction_id = self.get_owned_city_faction();
        let mut report = WorldRegionOwnerRelationReport {
            initial_faction_id,
            faction_cleared: false,
            union_cleared: false,
            owned_city: None,
            country: None,
        };
        if initial_faction_id <= 0 {
            return Ok(report);
        }

        if organizing.faction_by_id(initial_faction_id).is_none() {
            self.owned_city_organizing.set((0, 0));
            report.faction_cleared = true;
        }
        let union_id = self.get_owned_city_union();
        if organizing
            .confederation_by_id(union_id)
            .is_none()
        {
            report.union_cleared = union_id != 0;
            self.owned_city_organizing
                .set((self.get_owned_city_faction(), 0));
        }

        let faction_id = self.get_owned_city_faction();
        if faction_id <= 0 {
            return Ok(report);
        }
        report.owned_city = match organizing.add_owned_city_to_faction(
            game,
            faction_id,
            self.get_id(),
            update_player,
        ) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(WorldRegionOwnerRelationBlock::OwnedCity { report, source });
            }
        };
        let country = match organizing.country_by_faction(faction_id) {
            Ok(country) => country.unwrap_or(0),
            Err(source) => {
                return Err(WorldRegionOwnerRelationBlock::Country { report, source });
            }
        };
        self.region.set_country(country);
        report.country = Some(country);
        Ok(report)
    }

    pub(crate) fn new_region(&mut self) -> Result<i32, RegionLoadError> {
        self.region.new_region()
    }

    pub(crate) const fn decord_from_byte_array(
        &mut self,
        _source: &[u8],
        _cursor: &mut usize,
        _include_child: bool,
    ) -> bool {
        true
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldRegionSerializationBlock> {
        let _ = self
            .region
            .add_to_byte_array(destination, include_child)
            .map_err(WorldRegionSerializationBlock::Region)?;
        destination.extend_from_slice(&self.war_region_type.to_le_bytes());
        destination.push(u8::from(self.no_pk));
        destination.push(u8::from(self.no_contribute));

        append_count(destination, "m_listNpc", self.npcs.len())?;
        for npc in &self.npcs {
            destination.extend_from_slice(&npc.wire_header());
            append_c_string(destination, &npc.name);
            append_c_string(destination, &npc.script);
        }
        append_count(destination, "m_listMonster", self.monsters.len())?;
        for monster in &self.monsters {
            for scalar in monster.wire_scalars() {
                destination.extend_from_slice(&scalar.to_le_bytes());
            }
            append_count(
                destination,
                "tagMonster.vectorMonsterList",
                monster.variants.len(),
            )?;
            for variant in &monster.variants {
                destination.extend_from_slice(&variant.wire_prefix()?);
                append_c_string(destination, &variant.name);
                append_c_string(destination, &variant.script);
            }
        }
        append_count(destination, "m_vectorWeatherSetup", self.weather.len())?;
        for time in &self.weather {
            destination.extend_from_slice(&time.time.to_le_bytes());
            append_count(
                destination,
                "tagWeatherTime.vectorOption",
                time.options.len(),
            )?;
            for option in &time.options {
                destination.extend_from_slice(&option.cumulative_odds.to_le_bytes());
                append_count(destination, "tagOption.vectorWeather", option.weather.len())?;
                for weather in &option.weather {
                    destination.extend_from_slice(&weather.weather_index.to_le_bytes());
                    destination.extend_from_slice(&weather.fog_color.to_le_bytes());
                }
            }
        }
        destination.extend_from_slice(
            &self
                .add_setup_to_byte_array()
                .map_err(WorldRegionSerializationBlock::Setup)?,
        );
        append_region_param(destination, self.effective_param());
        Ok(true)
    }

 /// Перечитывает содержимое открытого `regions/{id}.rs`.
 ///
 /// Отсутствие ресурса обрабатывает caller до вызова и не меняет состояние.
    pub(crate) fn load_setup_bytes(&mut self, bytes: &[u8]) {
        let mut tokens = RegionSetupTokens::new(bytes);
        while let Some(token) = tokens.next_bytes() {
            if token == b"<end>" {
                break;
            }
            if token == b"*" {
                assign_next_i32(&mut tokens, &mut self.setup.return_region_id);
                assign_next_i32(&mut tokens, &mut self.setup.return_left);
                assign_next_i32(&mut tokens, &mut self.setup.return_top);
                assign_next_i32(&mut tokens, &mut self.setup.return_right);
                assign_next_i32(&mut tokens, &mut self.setup.return_bottom);
                assign_next_i32(&mut tokens, &mut self.setup.recall_when_lost);
                assign_next_i32(&mut tokens, &mut self.setup.move_monster_when_refresh);
                assign_next_i32(&mut tokens, &mut self.setup.use_return_point);
            }
        }

        self.forbidden_make_goods.clear();
        while let Some(token) = tokens.next_bytes() {
            if token == b"<end>" {
                break;
            }
            if token == b"#" {
                let Some(value) = tokens.next_bytes() else {
                    break;
                };
                self.forbidden_make_goods.push(value.to_vec());
            }
        }
    }

    pub(crate) fn add_setup_to_byte_array(
        &self,
    ) -> Result<Vec<u8>, WorldRegionSetupSerializationBlock> {
        let mut bytes = Vec::new();
        for (field, value) in [
            ("lReturnRegionID", self.setup.return_region_id),
            ("rtReturnPoint.left", self.setup.return_left),
            ("rtReturnPoint.top", self.setup.return_top),
            ("rtReturnPoint.right", self.setup.return_right),
            ("rtReturnPoint.bottom", self.setup.return_bottom),
            ("bDoesRecallWhenLost", self.setup.recall_when_lost),
            (
                "bMoveMonsterWhenRefeash",
                self.setup.move_monster_when_refresh,
            ),
            ("bUse", self.setup.use_return_point),
        ] {
            let value = value.ok_or(WorldRegionSetupSerializationBlock { field })?;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&(self.forbidden_make_goods.len() as u32 as i32).to_le_bytes());
        for value in &self.forbidden_make_goods {
            let value = value
                .get(
                    ..value
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(value.len()),
                )
                .expect("граница вычислена из текущего slice");
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
        Ok(bytes)
    }
}

const MONSTER_GROUP_FIELDS: [&str; 9] = [
    "tagMonster.lIndex",
    "tagMonster.left",
    "tagMonster.top",
    "tagMonster.right",
    "tagMonster.bottom",
    "tagMonster.lCount",
    "tagMonster.lInterval",
    "tagMonster.lStartTime",
    "tagMonster.lDir",
];

fn legacy_monster_variant_prefix(
    cumulative_odds: u16,
    flag: u16,
    leader: u16,
    distance: u16,
    name: &[u8],
) -> Result<[u8; 0x22], WorldRegionTextLoadError> {
    let name = legacy_c_string_prefix(name);
    if name.len() >= 0x10 {
        return Err(
            WorldRegionTextLoadError::MonsterNameRequiresLegacyHeapLayout { length: name.len() },
        );
    }
    let mut bytes = [0; 0x22];
    for (offset, value) in [cumulative_odds, flag, leader, distance]
        .into_iter()
        .enumerate()
    {
        bytes[offset * 2..offset * 2 + 2].copy_from_slice(&value.to_le_bytes());
    }
 // Старый basic_string занимает 0x1C: 4 bytes allocator/padding, 16-byte
 // union `_Bx`, DWORD `_Mysize`, DWORD `_Myres`. Prefix обрывается после
 // младших двух bytes `_Myres`; GameServer заранее выставляет старшие нули.
    bytes[0x0C..0x0C + name.len()].copy_from_slice(name);
    bytes[0x0C + name.len()] = 0;
    bytes[0x1C..0x20].copy_from_slice(&(name.len() as u32).to_le_bytes());
    bytes[0x20..0x22].copy_from_slice(&0x0Fu16.to_le_bytes());
    Ok(bytes)
}

fn normalize_legacy_script(value: &[u8]) -> Vec<u8> {
    legacy_c_string_prefix(value)
        .iter()
        .map(|byte| {
            let byte = if *byte == b'\\' { b'/' } else { *byte };
            byte.to_ascii_lowercase()
        })
        .collect()
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

fn append_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(legacy_c_string_prefix(value));
    destination.push(0);
}

fn append_count(
    destination: &mut Vec<u8>,
    collection: &'static str,
    count: usize,
) -> Result<(), WorldRegionSerializationBlock> {
    let count = i32::try_from(count)
        .map_err(|_| WorldRegionSerializationBlock::TooManyEntries { collection, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_region_param(destination: &mut Vec<u8>, param: RegionParamState) {
    for value in [
        param.region_id,
        param.max_tax_rate,
        param.current_tax_rate,
        param.total_tax as i32,
        param.today_total_tax as i32,
        param.superior_region_id,
        param.turn_in_tax_rate,
        param.owned_faction_id,
        param.owned_union_id,
    ] {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}

fn translate_color_code(value: &[u8]) -> Result<u32, WorldRegionTextLoadError> {
    let Some(value) = value.get(..6) else {
 // В 12 поставочных `.weather` после index 400
 // отсутствует color-token. читает следующий `time` в
 // char-buffer, а затем читает ещё два байта за его NUL;
 // их прежнее stack-содержимое неизвестно и безопасно не имитируется.
        return Err(WorldRegionTextLoadError::ShortWeatherColorCode {
            length: value.len(),
        });
    };
    Ok(value.iter().fold(0u32, |result, byte| {
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'A'..=b'F' => u32::from(byte - b'A' + 10),
            _ => 0,
        };
        result.wrapping_mul(0x10).wrapping_add(digit)
    }))
}

struct RegionSetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    failed: bool,
}

impl<'a> RegionSetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            failed: false,
        }
    }

    fn next_bytes(&mut self) -> Option<&'a [u8]> {
        if self.failed {
            return None;
        }
        let Some(value) = self.tokens.get(self.next).copied() else {
            self.failed = true;
            return None;
        };
        self.next += 1;
        Some(value)
    }

    fn next_bytes_optional(&mut self) -> Option<&'a [u8]> {
        self.next_bytes()
    }

    fn next_bytes_field(
        &mut self,
        field: &'static str,
    ) -> Result<&'a [u8], WorldRegionTextLoadError> {
        self.next_bytes()
            .ok_or(WorldRegionTextLoadError::MissingValue { field })
    }

    fn seek_to(&mut self, marker: &[u8]) -> bool {
        while let Some(token) = self.next_bytes() {
            if token == marker {
                return true;
            }
        }
        false
    }

    fn next_i32_field(&mut self, field: &'static str) -> Result<i32, WorldRegionTextLoadError> {
        parse_ascii_field(self.next_bytes_field(field)?, field)
    }

    fn next_u16_field(&mut self, field: &'static str) -> Result<u16, WorldRegionTextLoadError> {
        parse_ascii_field(self.next_bytes_field(field)?, field)
    }

    fn next_i32(&mut self) -> Option<i32> {
        let value = self.next_bytes()?;
        let parsed = std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.parse().ok());
        if parsed.is_none() {
            self.failed = true;
        }
        parsed
    }
}

fn parse_ascii_field<T>(value: &[u8], field: &'static str) -> Result<T, WorldRegionTextLoadError>
where
    T: std::str::FromStr,
{
    std::str::from_utf8(value)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(WorldRegionTextLoadError::InvalidValue { field })
}

fn read_param_i32(source: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        source[offset..offset + 4]
            .try_into()
            .expect("m_Param slice содержит ровно четыре байта"),
    )
}

fn read_param_u32(source: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        source[offset..offset + 4]
            .try_into()
            .expect("m_Param slice содержит ровно четыре байта"),
    )
}

fn assign_next_i32(tokens: &mut RegionSetupTokens<'_>, field: &mut Option<i32>) {
    if let Some(value) = tokens.next_i32() {
        *field = Some(value);
    }
}
