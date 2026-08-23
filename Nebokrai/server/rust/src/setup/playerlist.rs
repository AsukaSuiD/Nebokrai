//! Player properties и progression `CPlayerList` из WorldServer,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire состоит из player map, level-exp и трёх upgrade maps для Fighter,
//! Hunter и Taoist. Player record сохраняет 0x58-байтный значимый layout, но
//! неопределённый padding обнулён; upgrade record пишет level и scalars перед
//! NUL notification.
//!
//! Create-role equipment хранит occupation, slot и byte-name в list order.
//! Отсутствующий `sex + occupation*2` по-прежнему вставляет нулевую запись.
//!
//! Loaders очищают каждый свой owner до открытия. Ошибка первого player файла
//! сохраняет прежний equipment list; ошибка второго оставляет новый player map
//! и пустой equipment list. Malformed input сохраняет только полный префикс.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) hot_hit: u32,
    pub(crate) remain_point: u16,
    pub(crate) yp: u16,
    pub(crate) hp: u32,
    pub(crate) mp: u32,
    pub(crate) rp: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_maximum_yp: u16,
    pub(crate) base_maximum_rp: u16,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) base_minimum_attack: u32,
    pub(crate) base_maximum_attack: u32,
    pub(crate) base_hit: u16,
    pub(crate) base_burden: u16,
    pub(crate) base_cch: u16,
    pub(crate) base_defence: u32,
    pub(crate) base_dodge: u16,
    pub(crate) base_attack_speed: u16,
    pub(crate) base_element_resistant: u32,
    pub(crate) base_hp_recover_speed: u16,
    pub(crate) base_mp_recover_speed: u16,
    pub(crate) constitution_to_maximum_hp: u16,
    pub(crate) intelligence_to_maximum_mp: u16,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPropertiesUpgrade {
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) base_burden: u16,
    pub(crate) notification: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerOriginEquipment {
    pub(crate) occupation: u8,
    pub(crate) place_position: u16,
    pub(crate) original_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCreationPropertiesLookup {
    pub(crate) key: u32,
    pub(crate) inserted: bool,
    pub(crate) properties: PlayerBaseProperties,
}

pub(crate) type PlayerBasePropertiesMap = BTreeMap<u32, PlayerBaseProperties>;
pub(crate) type PlayerPropertiesUpgradeMap = BTreeMap<u32, PlayerPropertiesUpgrade>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerList {
    player_properties: PlayerBasePropertiesMap,
    origin_equipment: Vec<PlayerOriginEquipment>,
    player_experience: Vec<u32>,
    fighter_upgrades: PlayerPropertiesUpgradeMap,
    hunter_upgrades: PlayerPropertiesUpgradeMap,
    taoist_upgrades: PlayerPropertiesUpgradeMap,
}

impl CPlayerList {
    pub(crate) fn from_parts(
        player_properties: PlayerBasePropertiesMap,
        player_experience: Vec<u32>,
        fighter_upgrades: PlayerPropertiesUpgradeMap,
        hunter_upgrades: PlayerPropertiesUpgradeMap,
        taoist_upgrades: PlayerPropertiesUpgradeMap,
    ) -> Self {
        Self {
            player_properties,
            origin_equipment: Vec::new(),
            player_experience,
            fighter_upgrades,
            hunter_upgrades,
            taoist_upgrades,
        }
    }

    pub(crate) fn set_origin_equipment(&mut self, equipment: Vec<PlayerOriginEquipment>) {
        self.origin_equipment = equipment;
    }

    pub(crate) fn origin_equipment(&self) -> &[PlayerOriginEquipment] {
        &self.origin_equipment
    }

    pub(crate) fn clear_player_properties(&mut self) {
        self.player_properties.clear();
    }

    pub(crate) fn clear_origin_equipment(&mut self) {
        self.origin_equipment.clear();
    }

    pub(crate) fn clear_player_experience(&mut self) {
        self.player_experience.clear();
    }

    pub(crate) fn clear_properties_upgrades(&mut self) {
        self.fighter_upgrades.clear();
        self.hunter_upgrades.clear();
        self.taoist_upgrades.clear();
    }

    pub(crate) fn load_player_properties_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, PlayerListFormatError> {
        self.clear_player_properties();
        let mut tokens = tokens(source);
        let mut loaded = 0;
        while read_to(&mut tokens, b"*") {
            let occupation_token = read_i32(&mut tokens, "occupation")?;
            let sex_token = read_i32(&mut tokens, "sex")?;
            let occupation = u8::try_from(occupation_token)
                .ok()
                .filter(|value| *value < 3)
                .ok_or(PlayerListFormatError::InvalidOccupation(occupation_token))?;
            let sex = u8::try_from(sex_token)
                .ok()
                .filter(|value| *value <= 1)
                .ok_or(PlayerListFormatError::InvalidSex(sex_token))?;

            let properties = PlayerBaseProperties {
                occupation,
                sex,
                hot_hit: read_u32(&mut tokens, "hot hit")?,
                remain_point: read_u16(&mut tokens, "remain point")?,
                yp: 0,
                hp: 0,
                mp: 0,
                rp: 0,
                base_maximum_hp: read_u32(&mut tokens, "base maximum HP")?,
                base_maximum_mp: read_u32(&mut tokens, "base maximum MP")?,
                base_maximum_yp: read_u16(&mut tokens, "base maximum YP")?,
                base_maximum_rp: read_u16(&mut tokens, "base maximum RP")?,
                base_strength: read_u32(&mut tokens, "base strength")?,
                base_dexterity: read_u32(&mut tokens, "base dexterity")?,
                base_constitution: read_u32(&mut tokens, "base constitution")?,
                base_intelligence: read_u32(&mut tokens, "base intelligence")?,
                base_minimum_attack: read_u32(&mut tokens, "base minimum attack")?,
                base_maximum_attack: read_u32(&mut tokens, "base maximum attack")?,
                base_hit: read_u16(&mut tokens, "base hit")?,
                base_burden: read_u16(&mut tokens, "base burden")?,
                base_cch: read_u16(&mut tokens, "base CCH")?,
                base_defence: read_u32(&mut tokens, "base defence")?,
                base_dodge: read_u16(&mut tokens, "base dodge")?,
                base_attack_speed: read_u16(&mut tokens, "base attack speed")?,
                base_element_resistant: read_u32(&mut tokens, "base element resistance")?,
                base_hp_recover_speed: read_u16(&mut tokens, "base HP recovery speed")?,
                base_mp_recover_speed: read_u16(&mut tokens, "base MP recovery speed")?,
                constitution_to_maximum_hp: read_u16(&mut tokens, "constitution to HP")?,
                intelligence_to_maximum_mp: read_u16(&mut tokens, "intelligence to MP")?,
            };
            let properties = PlayerBaseProperties {
                hp: properties.base_maximum_hp,
                mp: properties.base_maximum_mp,
                ..properties
            };
            let key = u32::from(sex) + u32::from(occupation) * 2;
            self.player_properties.insert(key, properties);
            loaded += 1;
        }
        Ok(loaded)
    }

    pub(crate) fn load_origin_equipment_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, PlayerListFormatError> {
        self.clear_origin_equipment();
        let mut tokens = tokens(source);
        while read_to(&mut tokens, b"*") {
            let occupation = read_i32(&mut tokens, "origin equipment occupation")?;
            let place_position = read_u16(&mut tokens, "origin equipment position")?;
            let original_name = next_token(&mut tokens, "origin equipment original name")?.to_vec();
            self.origin_equipment.push(PlayerOriginEquipment {
                occupation: occupation as u8,
                place_position,
                original_name,
            });
        }
        Ok(self.origin_equipment.len())
    }

    pub(crate) fn load_player_list_from_bytes(
        &mut self,
        player_list_source: &[u8],
        origin_equipment_source: &[u8],
    ) -> Result<PlayerListLoadReport, PlayerListFormatError> {
        let player_properties = self.load_player_properties_from_bytes(player_list_source)?;
        let origin_equipment = self.load_origin_equipment_from_bytes(origin_equipment_source)?;
        Ok(PlayerListLoadReport {
            player_properties,
            origin_equipment,
        })
    }

    pub(crate) fn load_player_list_from_files(
        &mut self,
        player_list_path: impl AsRef<Path>,
        origin_equipment_path: impl AsRef<Path>,
    ) -> Result<PlayerListLoadReport, PlayerListFileLoadError> {
        self.clear_player_properties();
        let player_list_source = std::fs::read(player_list_path).map_err(PlayerListFileLoadError::Io)?;
        let player_properties = self
            .load_player_properties_from_bytes(&player_list_source)
            .map_err(PlayerListFileLoadError::Format)?;
        self.clear_origin_equipment();
        let origin_equipment_source =
            std::fs::read(origin_equipment_path).map_err(PlayerListFileLoadError::Io)?;
        let origin_equipment = self
            .load_origin_equipment_from_bytes(&origin_equipment_source)
            .map_err(PlayerListFileLoadError::Format)?;
        Ok(PlayerListLoadReport {
            player_properties,
            origin_equipment,
        })
    }

    pub(crate) fn load_player_experience_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, PlayerListFormatError> {
        self.clear_player_experience();
        let mut tokens = tokens(source);
        while read_to(&mut tokens, b"#") {
            let _level = read_u32(&mut tokens, "experience level")?;
            self.player_experience
                .push(read_u32(&mut tokens, "experience value")?);
        }
        Ok(self.player_experience.len())
    }

    pub(crate) fn load_player_experience_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, PlayerListFileLoadError> {
        self.clear_player_experience();
        let source = std::fs::read(path).map_err(PlayerListFileLoadError::Io)?;
        self.load_player_experience_from_bytes(&source)
            .map_err(PlayerListFileLoadError::Format)
    }

 /// Выполняет три последовательных блока `LoadPlayerProperitiesUpgrade`.
 ///
 /// `StringTable::getStringByID` в EXE подменял отсутствующий key пустой
 /// строкой. Closure получает byte-оригинал key и возвращает локализованный
 /// текст либо `None` для того же результата.
    pub(crate) fn load_properties_upgrades_from_bytes<ResolveNotification>(
        &mut self,
        source: &[u8],
        resolve_notification: &mut ResolveNotification,
    ) -> Result<PlayerPropertiesUpgradeLoadReport, PlayerListFormatError>
    where
        ResolveNotification: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear_properties_upgrades();
        let mut tokens = tokens(source);
        let fighter = load_upgrade_block(&mut tokens, &mut self.fighter_upgrades, resolve_notification)?;
        let hunter = load_upgrade_block(&mut tokens, &mut self.hunter_upgrades, resolve_notification)?;
        let taoist = load_upgrade_block(&mut tokens, &mut self.taoist_upgrades, resolve_notification)?;
        Ok(PlayerPropertiesUpgradeLoadReport {
            fighter,
            hunter,
            taoist,
        })
    }

    pub(crate) fn load_properties_upgrades_from_file<ResolveNotification>(
        &mut self,
        path: impl AsRef<Path>,
        resolve_notification: &mut ResolveNotification,
    ) -> Result<PlayerPropertiesUpgradeLoadReport, PlayerListFileLoadError>
    where
        ResolveNotification: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear_properties_upgrades();
        let source = std::fs::read(path).map_err(PlayerListFileLoadError::Io)?;
        self.load_properties_upgrades_from_bytes(&source, resolve_notification)
            .map_err(PlayerListFileLoadError::Format)
    }

    pub(crate) fn creation_properties(
        &mut self,
        sex: u8,
        occupation: u8,
    ) -> PlayerCreationPropertiesLookup {
        let key = u32::from(sex).wrapping_add(u32::from(occupation).wrapping_mul(2));
        let inserted = !self.player_properties.contains_key(&key);
        let properties = *self.player_properties.entry(key).or_default();
        PlayerCreationPropertiesLookup {
            key,
            inserted,
            properties,
        }
    }

    pub(crate) fn properties_upgrade(
        &self,
        occupation: u8,
        level: u8,
    ) -> Option<&PlayerPropertiesUpgrade> {
        let upgrades = match occupation {
            0 => &self.fighter_upgrades,
            1 => &self.hunter_upgrades,
            2 => &self.taoist_upgrades,
            _ => return None,
        };
        upgrades.get(&u32::from(level))
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerListSerializeError> {
        append_count(
            destination,
            "player property map",
            self.player_properties.len(),
        )?;
        for properties in self.player_properties.values() {
            append_player_properties(destination, properties);
        }

        append_count(
            destination,
            "player experience vector",
            self.player_experience.len(),
        )?;
        for experience in &self.player_experience {
            destination.extend_from_slice(&experience.to_le_bytes());
        }

        append_upgrade_map(destination, "fighter upgrade map", &self.fighter_upgrades)?;
        append_upgrade_map(destination, "hunter upgrade map", &self.hunter_upgrades)?;
        append_upgrade_map(destination, "taoist upgrade map", &self.taoist_upgrades)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerListLoadReport {
    pub(crate) player_properties: usize,
    pub(crate) origin_equipment: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPropertiesUpgradeLoadReport {
    pub(crate) fighter: usize,
    pub(crate) hunter: usize,
    pub(crate) taoist: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerListFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidUnsignedLong { field: &'static str, token: Vec<u8> },
    InvalidOccupation(i32),
    InvalidSex(i32),
    MissingUpgradeBlock { block: &'static str },
}

impl fmt::Display for PlayerListFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidUnsignedLong { field, token } => write!(
                formatter,
                "поле {field} не является подходящим unsigned long: {}",
                String::from_utf8_lossy(token)
            ),
            Self::InvalidOccupation(value) => write!(formatter, "недопустимая occupation {value}"),
            Self::InvalidSex(value) => write!(formatter, "недопустимый sex {value}"),
            Self::MissingUpgradeBlock { block } => {
                write!(formatter, "отсутствует маркер блока upgrade {block}")
            }
        }
    }
}

impl Error for PlayerListFormatError {}

#[derive(Debug)]
pub(crate) enum PlayerListFileLoadError {
    Io(io::Error),
    Format(PlayerListFormatError),
}

impl fmt::Display for PlayerListFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for PlayerListFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

fn tokens(source: &[u8]) -> impl Iterator<Item = &[u8]> {
    source
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty())
}

fn load_upgrade_block<'source, ResolveNotification>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    upgrades: &mut PlayerPropertiesUpgradeMap,
    resolve_notification: &mut ResolveNotification,
) -> Result<usize, PlayerListFormatError>
where
    ResolveNotification: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    if !read_to(tokens, b"*") {
        return Err(PlayerListFormatError::MissingUpgradeBlock { block: "properties" });
    }
    let count = read_u32(tokens, "upgrade count")?;
    let mut applied = 0;
    for _ in 0..count {
        let level = read_u32(tokens, "upgrade level")?;
        let properties = PlayerPropertiesUpgrade {
            base_maximum_hp: read_u32(tokens, "upgrade base maximum HP")?,
            base_maximum_mp: read_u32(tokens, "upgrade base maximum MP")?,
            base_strength: read_u32(tokens, "upgrade base strength")?,
            base_dexterity: read_u32(tokens, "upgrade base dexterity")?,
            base_constitution: read_u32(tokens, "upgrade base constitution")?,
            base_intelligence: read_u32(tokens, "upgrade base intelligence")?,
            base_burden: read_u16(tokens, "upgrade base burden")?,
            notification: resolve_notification(next_token(tokens, "upgrade notification key")?)
                .unwrap_or_default(),
        };
        upgrades.insert(level, properties);
        applied += 1;
    }
    Ok(applied)
}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], PlayerListFormatError> {
    tokens
        .next()
        .ok_or(PlayerListFormatError::UnexpectedEnd { field })
}

fn read_i32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i32, PlayerListFormatError> {
    let token = next_token(tokens, field)?;
    let text = std::str::from_utf8(token).map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<i32>()
        .map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
            field,
            token: token.to_vec(),
        })
}

fn read_u32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<u32, PlayerListFormatError> {
    let token = next_token(tokens, field)?;
    let text = std::str::from_utf8(token).map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<u32>()
        .map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
            field,
            token: token.to_vec(),
        })
}

fn read_u16<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<u16, PlayerListFormatError> {
    let token = next_token(tokens, field)?;
    let value = parse_u32(token, field)?;
    u16::try_from(value).map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
        field,
        token: token.to_vec(),
    })
}

fn parse_u32(token: &[u8], field: &'static str) -> Result<u32, PlayerListFormatError> {
    let text = std::str::from_utf8(token).map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<u32>()
        .map_err(|_| PlayerListFormatError::InvalidUnsignedLong {
            field,
            token: token.to_vec(),
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerListSerializeError {
    pub(crate) owner: &'static str,
    pub(crate) count: usize,
}

impl fmt::Display for PlayerListSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} элементов вне signed 32-битного диапазона",
            self.owner, self.count
        )
    }
}

impl Error for PlayerListSerializeError {}

fn append_player_properties(destination: &mut Vec<u8>, value: &PlayerBaseProperties) {
    destination.extend_from_slice(&[value.occupation, value.sex, 0, 0]);
    destination.extend_from_slice(&value.hot_hit.to_le_bytes());
    destination.extend_from_slice(&value.remain_point.to_le_bytes());
    destination.extend_from_slice(&value.yp.to_le_bytes());
    destination.extend_from_slice(&value.hp.to_le_bytes());
    destination.extend_from_slice(&value.mp.to_le_bytes());
    destination.extend_from_slice(&value.rp.to_le_bytes());
    destination.extend_from_slice(&0_u16.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_hp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_mp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_yp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_rp.to_le_bytes());
    for scalar in [
        value.base_strength,
        value.base_dexterity,
        value.base_constitution,
        value.base_intelligence,
        value.base_minimum_attack,
        value.base_maximum_attack,
    ] {
        destination.extend_from_slice(&scalar.to_le_bytes());
    }
    destination.extend_from_slice(&value.base_hit.to_le_bytes());
    destination.extend_from_slice(&value.base_burden.to_le_bytes());
    destination.extend_from_slice(&value.base_cch.to_le_bytes());
    destination.extend_from_slice(&0_u16.to_le_bytes());
    destination.extend_from_slice(&value.base_defence.to_le_bytes());
    destination.extend_from_slice(&value.base_dodge.to_le_bytes());
    destination.extend_from_slice(&value.base_attack_speed.to_le_bytes());
    destination.extend_from_slice(&value.base_element_resistant.to_le_bytes());
    destination.extend_from_slice(&value.base_hp_recover_speed.to_le_bytes());
    destination.extend_from_slice(&value.base_mp_recover_speed.to_le_bytes());
    destination.extend_from_slice(&value.constitution_to_maximum_hp.to_le_bytes());
    destination.extend_from_slice(&value.intelligence_to_maximum_mp.to_le_bytes());
}

fn append_upgrade_map(
    destination: &mut Vec<u8>,
    owner: &'static str,
    upgrades: &PlayerPropertiesUpgradeMap,
) -> Result<(), PlayerListSerializeError> {
    append_count(destination, owner, upgrades.len())?;
    for (&level, properties) in upgrades {
        destination.extend_from_slice(&level.to_le_bytes());
        for scalar in [
            properties.base_maximum_hp,
            properties.base_maximum_mp,
            properties.base_strength,
            properties.base_dexterity,
            properties.base_constitution,
            properties.base_intelligence,
        ] {
            destination.extend_from_slice(&scalar.to_le_bytes());
        }
        destination.extend_from_slice(&properties.base_burden.to_le_bytes());
        append_legacy_string(destination, &properties.notification);
    }
    Ok(())
}

fn append_count(
    destination: &mut Vec<u8>,
    owner: &'static str,
    count: usize,
) -> Result<(), PlayerListSerializeError> {
    let count = i32::try_from(count).map_err(|_| PlayerListSerializeError { owner, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_legacy_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(value.split(|byte| *byte == 0).next().unwrap_or_default());
    destination.push(0);
}
