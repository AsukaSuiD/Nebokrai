//! Опыт fairy `CFairyExpConf` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`. Game-пара использует наследованный
//! decoder `CBattleFairyExpConfig`; исходные owners `setup/fairyexpconf.*` и
//! `setup/cbattlefairyexpconfig.*`.
//!
//! Owner использует тот же ordered map и wire, что `CBattleFairyExpConfig`,
//! но загружает отдельный `fairyexp.xml`. Direct-child traversal, duplicate
//! checks и минимум `maxdengji - 1` exp values сохранены; `quick-xml` заменяет
//! TinyXML.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::resources::cbattlefairyexpconfig::{
    BattleFairyExpDecodeError, BattleFairyExpSerializeError, CBattleFairyExpConfig,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CFairyExpConf {
    base: CBattleFairyExpConfig,
}

impl CFairyExpConf {
    pub fn clear(&mut self) {
        self.base.clear();
    }

    pub fn load_from_bytes(&mut self, source: &[u8]) -> Result<(), FairyExpLoadError> {
        self.clear();
        let result = self.load_from_bytes_after_clear(source);
        if result.is_err() {
            self.clear();
        }
        result
    }

    fn load_from_bytes_after_clear(&mut self, source: &[u8]) -> Result<(), FairyExpLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut groups = 0usize;
        let mut pending: Option<(u32, u32)> = None;
        let mut values: Vec<u32> = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(
                        &start,
                        depth,
                        &mut root_seen,
                        &mut groups,
                        &mut pending,
                        &mut values,
                    )?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(
                        &empty,
                        depth,
                        &mut root_seen,
                        &mut groups,
                        &mut pending,
                        &mut values,
                    )?;
                    if depth == 1 && empty.name().as_ref() == b"wuhunpinzhong" {
                        self.finish_group(&mut pending, &mut values)?;
                    }
                }
                Ok(Event::End(end)) => {
                    if depth == 0 {
                        return Err(FairyExpLoadError::InvalidFormat);
                    }
                    depth -= 1;
                    if depth == 1 && end.name().as_ref() == b"wuhunpinzhong" {
                        self.finish_group(&mut pending, &mut values)?;
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(FairyExpLoadError::InvalidFormat),
            }
            buffer.clear();
        }
        if !root_seen {
            return Err(FairyExpLoadError::MissingRoot);
        }
        if groups == 0 {
            return Err(FairyExpLoadError::MissingGroups);
        }
        if depth != 0 || pending.is_some() {
            return Err(FairyExpLoadError::InvalidFormat);
        }
        Ok(())
    }

    fn process_start(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        groups: &mut usize,
        pending: &mut Option<(u32, u32)>,
        values: &mut Vec<u32>,
    ) -> Result<(), FairyExpLoadError> {
        let qualified_name = start.name();
        let name = qualified_name.as_ref();
        if !*root_seen {
            if name != b"wuhunupjingyanliebiao" {
                return Err(FairyExpLoadError::MissingRoot);
            }
            *root_seen = true;
        } else if depth == 1 && name == b"wuhunpinzhong" {
            let owner_level =
                required_u32(start, b"renzhudengji", FairyExpLoadError::MissingOwnerLevel)?;
            if self.base.contains_exp_list(owner_level) {
                return Err(FairyExpLoadError::DuplicateOwnerLevel);
            }
            let max_level = required_u32(start, b"maxdengji", FairyExpLoadError::MissingMaxLevel)?;
            *pending = Some((owner_level, max_level));
            values.clear();
            *groups += 1;
        } else if depth == 2 && name == b"wuhun" {
            let value = required_u32(start, b"upjingyan", FairyExpLoadError::MissingExperience)?;
            if pending.is_none() {
                return Err(FairyExpLoadError::InvalidFormat);
            }
            values.push(value);
        }
        Ok(())
    }

    fn finish_group(
        &mut self,
        pending: &mut Option<(u32, u32)>,
        values: &mut Vec<u32>,
    ) -> Result<(), FairyExpLoadError> {
        let Some((owner_level, max_level)) = pending.take() else {
            return Err(FairyExpLoadError::InvalidFormat);
        };
        // Map-запись создаётся только после первого `wuhun`; пустая группа пропускается.
        if values.is_empty() {
            return Ok(());
        }
        if values.len().saturating_add(1) < max_level as usize {
            return Err(FairyExpLoadError::InsufficientExperience);
        }
        self.base
            .insert_exp_list(owner_level, std::mem::take(values));
        Ok(())
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), BattleFairyExpSerializeError> {
        self.base.add_to_byte_array(destination)
    }

    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), BattleFairyExpDecodeError> {
        self.base.decord_from_byte_array(source, cursor)
    }

    pub fn dw_exp_up(&self, equip_level: u32, level: u32) -> u32 {
        self.base.dw_exp_up(equip_level, level)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FairyExpLoadError {
    MissingRoot,
    MissingGroups,
    InvalidFormat,
    MissingOwnerLevel,
    DuplicateOwnerLevel,
    MissingMaxLevel,
    MissingExperience,
    InsufficientExperience,
}

impl FairyExpLoadError {
    pub const fn log_payload(self) -> &'static [u8] {
        match self {
            Self::MissingRoot | Self::InvalidFormat => b"wrong foramt of  FairyExp.....failed!",
            Self::MissingGroups => b"wrong foramt of  FairyExp..again..failed!",
            Self::MissingOwnerLevel => b"the fairy has not configure its owner`s  step! ",
            Self::DuplicateOwnerLevel => b"it`s the second time to configure fairy  owner`s step! ",
            Self::MissingMaxLevel => b"this fairy has not configure the max step!  ",
            Self::MissingExperience => b"this fairy have not configure the Exp value!",
            Self::InsufficientExperience => {
                b"the number of offairy `s max step is not match the number of Exp value! "
            }
        }
    }
}

fn required_u32(
    start: &BytesStart<'_>,
    name: &[u8],
    error: FairyExpLoadError,
) -> Result<u32, FairyExpLoadError> {
    let value = start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == name)
        .map(|attribute| attribute.value.into_owned())
        .ok_or(error)?;
    Ok(legacy_atol(&value) as u32)
}

fn legacy_atol(value: &[u8]) -> i32 {
    let mut bytes = value
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut parsed = false;
    let mut result = 0_i32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    if !parsed {
        0
    } else if negative {
        result.saturating_neg()
    } else {
        result
    }
}
