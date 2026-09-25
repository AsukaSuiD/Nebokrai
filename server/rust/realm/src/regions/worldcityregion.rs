//! Городской регион `CWorldCityRegion` из `worldcityregion.cpp/.h`,
//! подтверждённый `worldserver.exe` и `worldserver.pdb`, перенесённый в
//! Realm `regions/`.
//!
//! `.city` после успешного открытия очищает gates, читает 0x2C scalar bytes,
//! resolved name/script и первые пять полей defence setup. Missing resource
//! сохраняет прежние lists.
//!
//! Serializer пишет полный 0x20 defence block, count и gates. Точный конструктор
//! не инициализирует последние три defence DWORD, а loader намеренно читает
//! только первые пять; безопасная Rust-замена задаёт этим внутренним полям нули
//! вместо переноса исходного UB. `Load` успешен только при успешных War/City
//! loaders, включённом return setup и совпадении обоих return region IDs.
//!
//! Decoder делегирует no-op War owner и не двигает cursor. `Clone` строк
//! заменяет MSVC SSO, сохраняя одиннадцать scalar fields и две C-строки.

use crate::activities::attackcitysys::CAttackCitySys;
use crate::app::worldserver::WorldRegionResourceContext;
use crate::characters::player::CPlayer;
use crate::content::countryparam::CCountryParam;
use crate::content::organizing::ECityState;
use crate::regions::region::{CRegion, RegionRandomPositionBlock};
use crate::regions::shapetypes::ShapeTileCoordinateBlock;
use crate::regions::worldregion::{
    WorldRegionEnterBlock, WorldRegionLoadedCounts, WorldRegionSetupSerializationBlock,
};
use crate::regions::worldwarregion::{
    CWorldWarRegion, WorldWarRegionLoadError, WorldWarRegionSerializationBlock,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionTextLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionLoadError {
    War(WorldWarRegionLoadError),
    City(WorldCityRegionTextLoadError),
    BaseSetup(WorldRegionSetupSerializationBlock),
    UninitializedDefenceField { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionSerializationBlock {
    War(WorldWarRegionSerializationBlock),
    UninitializedDefenceField { field: &'static str },
    TooManyGates { count: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCityRegionEnterBlock {
    ActiveStateMissingPlayer,
    TileCoordinate(ShapeTileCoordinateBlock),
    UninitializedDefenceField { field: &'static str },
    BaseReturnPoint(WorldRegionEnterBlock),
    CoordinateOverflow { operation: &'static str },
    RandomPosition(RegionRandomPositionBlock),
}

/// Одни городские ворота из `CWorldCityRegion::tagBuild`.
///
/// Порядок полей равен точному 44-байтному scalar-prefix конструктора и
/// сериализатора. Два byte-вектора сохраняют legacy C-строки без MSVC ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldCityBuild {
    id: i32,
    picture_id: i32,
    direction: i32,
    action: i32,
    maximum_hp: i32,
    defence: i32,
    element_resistant: i32,
    title_x: i32,
    title_y: i32,
    width_increment: i32,
    height_increment: i32,
    name: Vec<u8>,
    script: Vec<u8>,
}

impl WorldCityBuild {
    const fn wire_scalars(&self) -> [i32; 11] {
        [
            self.id,
            self.picture_id,
            self.direction,
            self.action,
            self.maximum_hp,
            self.defence,
            self.element_resistant,
            self.title_x,
            self.title_y,
            self.width_increment,
            self.height_increment,
        ]
    }
}

#[derive(Clone, Debug)]
struct WorldCityDefenceSetup {
    values: [Option<i32>; 8],
}

impl WorldCityDefenceSetup {
    const fn with_safe_constructor_state() -> Self {
        Self {
            values: [None, None, None, None, None, Some(0), Some(0), Some(0)],
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldCityRegionLoadOutcome {
    pub counts: WorldRegionLoadedCounts,
    pub loaded: bool,
}

pub struct CWorldCityRegion {
    war: CWorldWarRegion,
    gates: Vec<WorldCityBuild>,
    defence: WorldCityDefenceSetup,
}

impl CWorldCityRegion {
    pub const fn with_constructor_state() -> Self {
        let mut war = CWorldWarRegion::with_constructor_base();
        war.set_constructor_symbols(3, 3, 2);
        Self {
            war,
            gates: Vec::new(),
            defence: WorldCityDefenceSetup::with_safe_constructor_state(),
        }
    }

    pub const fn war(&self) -> &CWorldWarRegion {
        &self.war
    }

    pub const fn war_mut(&mut self) -> &mut CWorldWarRegion {
        &mut self.war
    }

    pub fn load_from_context<Context, ResolveName>(
        &mut self,
        context: &mut Context,
        resolve_name: &mut ResolveName,
    ) -> Result<WorldCityRegionLoadOutcome, WorldCityRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
        ResolveName: FnMut(&[u8]) -> Vec<u8> + ?Sized,
    {
        let counts = self
            .war
            .load_from_context(context, resolve_name)
            .map_err(WorldCityRegionLoadError::War)?;
        if counts.base_failure.is_some() {
            return Ok(WorldCityRegionLoadOutcome {
                counts,
                loaded: false,
            });
        }
        let path = format!("regions/{}.city", self.war.base().get_id()).into_bytes();
        let city = context.read_resource(&path);
        if !self
            .load_city_bytes(city.as_deref(), &mut *resolve_name)
            .map_err(WorldCityRegionLoadError::City)?
        {
            return Ok(WorldCityRegionLoadOutcome {
                counts,
                loaded: false,
            });
        }
        let (use_return_point, base_return_id, own_id) = self
            .war
            .base()
            .city_load_base_guard()
            .map_err(WorldCityRegionLoadError::BaseSetup)?;
        let defence_return_id =
            self.defence.values[0].ok_or(WorldCityRegionLoadError::UninitializedDefenceField {
                field: "m_DefenceSideRS.lReturnRegionID",
            })?;
        let loaded = use_return_point && base_return_id == own_id && defence_return_id == own_id;
        Ok(WorldCityRegionLoadOutcome { counts, loaded })
    }

    pub fn load_city_bytes<ResolveName>(
        &mut self,
        bytes: Option<&[u8]>,
        mut resolve_name: ResolveName,
    ) -> Result<bool, WorldCityRegionTextLoadError>
    where
        ResolveName: FnMut(&[u8]) -> Vec<u8>,
    {
        let Some(bytes) = bytes else {
            return Ok(false);
        };
        let mut tokens = CityTokens::new(bytes);
        let mut gates = Vec::new();
        while let Some(token) = tokens.next_optional() {
            if token == b"<end>" {
                break;
            }
            if token != b"#" {
                continue;
            }
            let id = tokens.next_i32("tagBuild.lID")?;
            let string_id = tokens.next("tagBuild.stringID")?;
            let picture_id = tokens.next_i32("tagBuild.lPicID")?;
            let direction = tokens.next_i32("tagBuild.lDir")?;
            let action = tokens.next_i32("tagBuild.lAction")?;
            let maximum_hp = tokens.next_i32("tagBuild.lMaxHP")?;
            let defence = tokens.next_i32("tagBuild.lDef")?;
            let element_resistant = tokens.next_i32("tagBuild.lElementResistant")?;
            let title_x = tokens.next_i32("tagBuild.lTitleX")?;
            let title_y = tokens.next_i32("tagBuild.lTitleY")?;
            let width_increment = tokens.next_i32("tagBuild.lWidthInc")?;
            let height_increment = tokens.next_i32("tagBuild.lHeightInc")?;
            let script = tokens.next("tagBuild.strScript")?.to_vec();
            gates.push(WorldCityBuild {
                id,
                picture_id,
                direction,
                action,
                maximum_hp,
                defence,
                element_resistant,
                title_x,
                title_y,
                width_increment,
                height_increment,
                name: resolve_name(string_id),
                script,
            });
        }
        self.gates = gates;

        while let Some(token) = tokens.next_optional() {
            if token == b"<end>" {
                break;
            }
            if token != b"#" {
                continue;
            }
            for (index, field) in CITY_DEFENCE_FIELDS.iter().enumerate().take(5) {
                self.defence.values[index] = Some(tokens.next_i32(field)?);
            }
        }
        Ok(true)
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldCityRegionSerializationBlock> {
        let _ = self
            .war
            .add_to_byte_array(destination, include_child)
            .map_err(WorldCityRegionSerializationBlock::War)?;
        for (index, value) in self.defence.values.into_iter().enumerate() {
            let value = value.ok_or(
                WorldCityRegionSerializationBlock::UninitializedDefenceField {
                    field: CITY_DEFENCE_FIELDS[index],
                },
            )?;
            destination.extend_from_slice(&value.to_le_bytes());
        }
        let count = i32::try_from(self.gates.len()).map_err(|_| {
            WorldCityRegionSerializationBlock::TooManyGates {
                count: self.gates.len(),
            }
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for gate in &self.gates {
            for scalar in gate.wire_scalars() {
                destination.extend_from_slice(&scalar.to_le_bytes());
            }
            append_city_c_string(destination, &gate.name);
            append_city_c_string(destination, &gate.script);
        }
        Ok(true)
    }

    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> bool {
        let _ = self
            .war
            .decord_from_byte_array(source, cursor, include_child);
        true
    }

    pub fn set_enter_pos_xy<'a, FindRegion, Random>(
        &self,
        player: Option<&mut CPlayer>,
        country_param: &mut CCountryParam,
        attack_city: &CAttackCitySys,
        mut find_region: FindRegion,
        mut random: Random,
    ) -> Result<(), WorldCityRegionEnterBlock>
    where
        FindRegion: FnMut(i32) -> Option<&'a CRegion>,
        Random: FnMut(i32) -> i32,
    {
        let state = attack_city.get_city_state(self.war.base().get_id());
        if state != ECityState::Fight && state != ECityState::Mass {
            return Ok(());
        }
 // Исходный владелец разыменовывает `pPlayer` без
 // проверки только в активном состоянии; достижимая реакция null
 // неизвестна.
        let player = player.ok_or(WorldCityRegionEnterBlock::ActiveStateMissingPlayer)?;
        let mut tile_x = player
            .get_tile_x()
            .map_err(WorldCityRegionEnterBlock::TileCoordinate)?;
        let mut tile_y = player
            .get_tile_y()
            .map_err(WorldCityRegionEnterBlock::TileCoordinate)?;

        let faction_id = player.faction_id();
        let defender = faction_id != 0 && faction_id == self.war.base().get_owned_city_faction();
        let (region_id, left, top, span_x, span_y) =
            if defender {
                let field = |index: usize, field: &'static str| {
                    self.defence.values[index]
                        .ok_or(WorldCityRegionEnterBlock::UninitializedDefenceField { field })
                };
                let region_id = field(0, "m_DefenceSideRS.lReturnRegionID")?;
                let left = field(1, "m_DefenceSideRS.rtReturnPoint.left")?;
                let top = field(2, "m_DefenceSideRS.rtReturnPoint.top")?;
                let right = field(3, "m_DefenceSideRS.rtReturnPoint.right")?;
                let bottom = field(4, "m_DefenceSideRS.rtReturnPoint.bottom")?;
                let span_x = right.checked_sub(left).ok_or(
                    WorldCityRegionEnterBlock::CoordinateOverflow {
                        operation: "defence right - left",
                    },
                )?;
                let span_y = bottom.checked_sub(top).ok_or(
                    WorldCityRegionEnterBlock::CoordinateOverflow {
                        operation: "defence bottom - top",
                    },
                )?;
                (region_id, left, top, span_x, span_y)
            } else {
                let point = self
                    .war
                    .base()
                    .get_return_point(Some(&*player), country_param)
                    .map_err(WorldCityRegionEnterBlock::BaseReturnPoint)?;
 // оставляет оба span-регистра нулевыми в attacker
 // branch; это наблюдаемая странность, а не сокращение RECT.
                (point.region_id, point.left, point.top, 0, 0)
            };

        if let Some(region) = find_region(region_id) {
            let position = region
                .get_random_pos_in_range(left, top, span_x, span_y, &mut random)
                .map_err(WorldCityRegionEnterBlock::RandomPosition)?;
            tile_x = position.x;
            tile_y = position.y;
        }
        player.set_region_id(region_id);
        player.set_tile_xy(tile_x, tile_y);
        Ok(())
    }
}

const CITY_DEFENCE_FIELDS: [&str; 8] = [
    "m_DefenceSideRS.lReturnRegionID",
    "m_DefenceSideRS.rtReturnPoint.left",
    "m_DefenceSideRS.rtReturnPoint.top",
    "m_DefenceSideRS.rtReturnPoint.right",
    "m_DefenceSideRS.rtReturnPoint.bottom",
    "m_DefenceSideRS.bDoesRecallWhenLost",
    "m_DefenceSideRS.bMoveMonsterWhenRefeash",
    "m_DefenceSideRS.bUse",
];

struct CityTokens<'a> {
    values: Vec<&'a [u8]>,
    next: usize,
}

impl<'a> CityTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            values: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
        }
    }

    fn next_optional(&mut self) -> Option<&'a [u8]> {
        let value = self.values.get(self.next).copied()?;
        self.next += 1;
        Some(value)
    }

    fn next(&mut self, field: &'static str) -> Result<&'a [u8], WorldCityRegionTextLoadError> {
        self.next_optional()
            .ok_or(WorldCityRegionTextLoadError::MissingValue { field })
    }

    fn next_i32(&mut self, field: &'static str) -> Result<i32, WorldCityRegionTextLoadError> {
        let value = self.next(field)?;
        std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or(WorldCityRegionTextLoadError::InvalidValue { field })
    }
}

fn append_city_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..end]);
    destination.push(0);
}
