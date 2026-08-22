//! Ordered cache и initial-config serializer навыков WorldServer.
//!
//! Статус `CSkillFactory::Serialize` RVA `0x00060E90` и безопасной замены
//! `ClearSkillCache` RVA `0x00060DC0`, `StringToUsage` `0x00060F80`,
//! `ClearUsageCache` `0x00061E00`, `LoadConfigration` `0x00062050`,
//! `LoadUsage` `0x000628E0`, `LoadSkillCache` `0x00062510` и
//! `LoadUsageCache` `0x00062A70`: `IMPLEMENTED`. Resource-owner перечисляет
//! и открывает файлы снаружи; factory принимает полученный список в его
//! исходном порядке.
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp`.
//!
//! EXE пишет signed count map-а, затем в unsigned ascending-key порядке для
//! каждого slot-а `u32 length + record`. Null skill и skill с unknown type или
//! нулевым ID не удаляются из framing: им соответствует нулевая длина. Вопреки
//! сырому псевдокоду, точные инструкции после освобождения временного record-а
//! продолжают итерацию, а не выходят из функции.
//!
//! `BTreeMap` заменяет MSVC tree и сохраняет порядок. `Option<CSkill>` оставляет
//! выразимым доказанный null-slot, обычная вставка строит composite key
//! `id << 16 | level & 0xffff`. Замена duplicate key корректно освобождает
//! прежний owner вместо внутренней утечки старого `operator[]` call-site.
//! Отдельная byte-keyed карта usage сохраняет `operator[]`-перезапись в
//! `LoadUsage`, а `StringToUsage` возвращает `SKILL_USAGE_UNKNOW`
//! `0x7fff_ffff`.
//! `LoadConfigration`/`LoadUsage` получают уже прочитанные байты от внешнего
//! resource-owner-а: это заменяет только `CRFile`, `stringstream` и MessageBox.
//! Cache loaders очищают карту до загрузки и после первой ошибки. У
//! `LoadSkillCache` есть подтверждённый EXE-quirk: любой path с byte-substring
//! `rhg_314` пропускается до открытия. Остальной порядок списка и normal
//! legacy-result сохранены.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::skill::{CSkill, SkillSerializeError, SkillTextError, SkillUsage};

/// `SKILL_USAGE_UNKNOW` из общего `SkillRelated.h` World/Game.
///
/// У PDB/EXE `StringToUsage` возвращает именно этот именованный sentinel;
/// public enum фиксирует его численное значение `0x7fff_ffff`. Он отличен от
/// нуля, поэтому неизвестная строка не должна превращаться в допустимый usage
/// в сериализуемой `(usage, cost)`-паре.
pub(crate) const UNKNOWN_SKILL_USAGE: u32 = 0x7fff_ffff;

/// Safe owner исходного process-global `g_mSkillMap`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSkillFactory {
    skills: BTreeMap<u32, Option<CSkill>>,
    usage_names: BTreeMap<Vec<u8>, u32>,
}

impl CSkillFactory {
    /// Вставляет нормальный skill под exact composite key.
    pub(crate) fn insert(&mut self, skill: CSkill) -> Option<CSkill> {
        self.skills.insert(skill.cache_key(), Some(skill)).flatten()
    }

    /// Сохраняет выразимым legacy null-slot и его позицию в ordered framing.
    pub(crate) fn insert_slot(
        &mut self,
        key: u32,
        skill: Option<CSkill>,
    ) -> Option<Option<CSkill>> {
        self.skills.insert(key, skill)
    }

    pub(crate) fn get(&self, skill_id: u32, level: i32) -> Option<&CSkill> {
        let key = skill_id.wrapping_shl(16) | ((level as u32) & 0xffff);
        self.skills.get(&key).and_then(Option::as_ref)
    }

    pub(crate) fn clear_skill_cache(&mut self) {
        self.skills.clear();
    }

    /// Точная запись имени из `.usage`: `std::map::operator[]` заменял прежнее
    /// значение при том же byte-sensitive наборе байтов.
    pub(crate) fn set_usage_name(&mut self, name: &[u8], usage: u32) -> Option<u32> {
        self.usage_names.insert(name.to_vec(), usage)
    }

    /// Exact `StringToUsage`; null указатель и несуществующее имя возвращают
    /// `SKILL_USAGE_UNKNOW`, а не создают новую map-запись.
    pub(crate) fn string_to_usage(&self, name: Option<&[u8]>) -> u32 {
        name.and_then(|name| self.usage_names.get(visible_c_string(name)).copied())
            .unwrap_or(UNKNOWN_SKILL_USAGE)
    }

    /// Safe replacement `ClearUsageCache`; Rust Drop освобождает ключи вместе
    /// с map вместо ручного `_Tree::_Erase`.
    pub(crate) fn clear_usage_cache(&mut self) {
        self.usage_names.clear();
    }

    /// Content-половина exact `LoadUsage` после открытия `CRFile`.
    ///
    /// Каждая пара whitespace-токенов сразу заменяет существующий key. При
    /// оборванной/переполненной паре старый код мог читать неинициализированные
    /// stack-поля; Rust останавливает именно эту недоказанную UB-границу, не
    /// придумывая очередное имя либо usage.
    pub(crate) fn load_usage(&mut self, source: &[u8]) -> Result<usize, SkillFactoryLoadError> {
        let mut cursor = 0;
        let mut loaded = 0;
        while let Some(name) = next_whitespace_token(source, &mut cursor) {
            let value = next_whitespace_token(source, &mut cursor).ok_or(
                SkillFactoryLoadError::MissingUsageValue {
                    name: name.to_vec(),
                },
            )?;
            let usage = parse_legacy_u32(value, "usage")?;
            self.set_usage_name(name, usage);
            loaded += 1;
        }
        Ok(loaded)
    }

    /// Content-половина exact `LoadConfigration` после открытия `CRFile`.
    ///
    /// `Read`/`ReadTo` последовательно ищут четыре ASCII marker-а. До первого
    /// `Level` одиночные токены игнорируются; далее каждый не-`Level` токен
    /// потребляет одно unsigned cost-слово. Неизвестный usage не попадает в
    /// skill record, а повтор composite key освобождает прежний owner.
    pub(crate) fn load_configuration(
        &mut self,
        source: &[u8],
    ) -> Result<SkillConfigurationLoadReport, SkillFactoryLoadError> {
        let mut cursor = 0;
        let skill_type = read_named_u32(source, &mut cursor, b"Type", "Type")?;
        let skill_id = read_named_u32(source, &mut cursor, b"ID", "ID")?;
        seek_marker(source, &mut cursor, b"Name", "Name")?;
        let skill_name = next_whitespace_token(source, &mut cursor)
            .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: "Name" })?;
        let target_self = read_named_u32(source, &mut cursor, b"Target", "Target")?;

        let mut active_key = None;
        let mut loaded_levels = 0;
        while let Some(token) = next_whitespace_token(source, &mut cursor) {
            if token == b"Level" {
                let level = next_whitespace_token(source, &mut cursor)
                    .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: "Level" })?;
                let level = parse_legacy_i32(level, "Level")?;
                let mut skill = CSkill::new(skill_id as i32);
                skill.set_skill_type(skill_type as i32);
                skill.set_level(level);
                skill.set_skill_name(skill_name)
                    .map_err(SkillFactoryLoadError::SkillText)?;
                skill.set_description(&[])
                    .map_err(SkillFactoryLoadError::SkillText)?;
                if target_self != 0 {
                    skill.set_target_type(1);
                }
                let key = skill.cache_key();
                self.insert(skill);
                active_key = Some(key);
                loaded_levels += 1;
                continue;
            }

            let Some(key) = active_key else {
                continue;
            };
            let cost = next_whitespace_token(source, &mut cursor).ok_or(
                SkillFactoryLoadError::MissingUsageValue {
                    name: token.to_vec(),
                },
            )?;
            let usage = self.string_to_usage(Some(token));
            let cost = parse_legacy_u32(cost, "cost")?;
            if usage != UNKNOWN_SKILL_USAGE {
                self.skills
                    .get_mut(&key)
                    .and_then(Option::as_mut)
                    .expect("активный skill slot публикуется перед его usage")
                    .add_usage(SkillUsage { usage, cost });
            }
        }

        Ok(SkillConfigurationLoadReport {
            skill_id,
            loaded_levels,
        })
    }

    /// Safe owner exact `LoadSkillCache` после перечисления файлов resource
    /// subsystem. Входной порядок намеренно не сортируется: им определялась
    /// итоговая замена равных composite key. Любой path с `rhg_314` EXE
    /// пропускал до `LoadConfigration`, включая отсутствующий resource.
    pub(crate) fn load_skill_cache<'a, I>(&mut self, resources: I) -> SkillFactoryCacheLoadReport
    where
        I: IntoIterator<Item = SkillFactoryCacheResource<'a>>,
    {
        self.clear_skill_cache();
        let mut report = SkillFactoryCacheLoadReport::default();

        for resource in resources {
            if contains_bytes(resource.path, b"rhg_314") {
                report.skipped_resources += 1;
                continue;
            }
            let result = resource
                .contents
                .ok_or_else(|| SkillFactoryCacheLoadError::ResourceOpen {
                    path: resource.path.to_vec(),
                })
                .and_then(|source| {
                    self.load_configuration(source)
                        .map(|loaded| loaded.loaded_levels)
                        .map_err(SkillFactoryCacheLoadError::Content)
                });
            match result {
                Ok(loaded_levels) => {
                    report.loaded_resources += 1;
                    report.loaded_records += loaded_levels;
                }
                Err(error) => {
                    self.clear_skill_cache();
                    report.failure = Some(error);
                    return report;
                }
            }
        }
        report
    }

    /// Safe owner exact `LoadUsageCache` после перечисления файлов resource
    /// subsystem. В отличие от skill cache, каждый переданный `.usage` файл
    /// открывается; на первой ошибке EXE стирал уже загруженный prefix.
    pub(crate) fn load_usage_cache<'a, I>(&mut self, resources: I) -> SkillFactoryCacheLoadReport
    where
        I: IntoIterator<Item = SkillFactoryCacheResource<'a>>,
    {
        self.clear_usage_cache();
        let mut report = SkillFactoryCacheLoadReport::default();

        for resource in resources {
            let result = resource
                .contents
                .ok_or_else(|| SkillFactoryCacheLoadError::ResourceOpen {
                    path: resource.path.to_vec(),
                })
                .and_then(|source| {
                    self.load_usage(source)
                        .map_err(SkillFactoryCacheLoadError::Content)
                });
            match result {
                Ok(loaded_entries) => {
                    report.loaded_resources += 1;
                    report.loaded_records += loaded_entries;
                }
                Err(error) => {
                    self.clear_usage_cache();
                    report.failure = Some(error);
                    return report;
                }
            }
        }
        report
    }

    /// Дописывает exact `count + ordered (length, record)` wire.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), SkillFactorySerializeError> {
        let count = i32::try_from(self.skills.len()).map_err(|_| {
            SkillFactorySerializeError::EntryCount {
                count: self.skills.len(),
            }
        })?;
        destination.extend_from_slice(&count.to_le_bytes());

        for (&key, skill) in &self.skills {
            let record = match skill {
                None => None,
                Some(skill) => skill
                    .serialize()
                    .map_err(|source| SkillFactorySerializeError::Skill { key, source })?,
            };
            let Some(record) = record else {
                destination.extend_from_slice(&0_u32.to_le_bytes());
                continue;
            };
            let length = u32::try_from(record.len()).map_err(|_| {
                SkillFactorySerializeError::RecordLength {
                    key,
                    length: record.len(),
                }
            })?;
            destination.extend_from_slice(&length.to_le_bytes());
            destination.extend_from_slice(&record);
        }
        Ok(())
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|candidate| candidate == needle)
}

/// Один уже перечисленный cache-resource. `None` представляет неудачу
/// `rfOpen`; поиск package/fallback-файлов остаётся у resource-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillFactoryCacheResource<'a> {
    pub(crate) path: &'a [u8],
    pub(crate) contents: Option<&'a [u8]>,
}

/// Итог exact cache-load. `legacy_result` повторяет исходный BOOL: список без
/// файлов успешен, а первая ошибка даёт `false` и оставляет целевую карту пустой.
#[derive(Debug)]
pub(crate) struct SkillFactoryCacheLoadReport {
    pub(crate) loaded_resources: usize,
    pub(crate) loaded_records: usize,
    pub(crate) skipped_resources: usize,
    failure: Option<SkillFactoryCacheLoadError>,
}

impl SkillFactoryCacheLoadReport {
    pub(crate) fn legacy_result(&self) -> bool {
        self.failure.is_none()
    }

    pub(crate) fn failure(&self) -> Option<&SkillFactoryCacheLoadError> {
        self.failure.as_ref()
    }
}

impl Default for SkillFactoryCacheLoadReport {
    fn default() -> Self {
        Self {
            loaded_resources: 0,
            loaded_records: 0,
            skipped_resources: 0,
            failure: None,
        }
    }
}

/// Причина безопасного прекращения cache-loader-а вместо исходного
/// неинициализированного/CRT error path.
#[derive(Debug)]
pub(crate) enum SkillFactoryCacheLoadError {
    ResourceOpen { path: Vec<u8> },
    Content(SkillFactoryLoadError),
}

impl fmt::Display for SkillFactoryCacheLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceOpen { path } => write!(
                formatter,
                "skill resource {:?} не удалось открыть",
                String::from_utf8_lossy(path)
            ),
            Self::Content(source) => source.fmt(formatter),
        }
    }
}

impl Error for SkillFactoryCacheLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Content(source) => Some(source),
            Self::ResourceOpen { .. } => None,
        }
    }
}

/// Наблюдаемый итог одного `.skill` content-loader-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillConfigurationLoadReport {
    pub(crate) skill_id: u32,
    pub(crate) loaded_levels: usize,
}

/// Typed safe boundary для некорректного legacy token stream.
#[derive(Debug)]
pub(crate) enum SkillFactoryLoadError {
    MissingMarker { marker: &'static str },
    MissingMarkerValue { marker: &'static str },
    MissingUsageValue { name: Vec<u8> },
    InvalidNumber { field: &'static str, value: Vec<u8> },
    SkillText(SkillTextError),
}

impl fmt::Display for SkillFactoryLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMarker { marker } => write!(formatter, "skill source: не найден marker {marker}"),
            Self::MissingMarkerValue { marker } => {
                write!(formatter, "skill source: marker {marker} не имеет значения")
            }
            Self::MissingUsageValue { name } => write!(
                formatter,
                "skill source: usage {:?} не имеет cost-значения",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidNumber { field, value } => write!(
                formatter,
                "skill source: {field} содержит недопустимое число {:?}",
                String::from_utf8_lossy(value)
            ),
            Self::SkillText(source) => source.fmt(formatter),
        }
    }
}

impl Error for SkillFactoryLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SkillText(source) => Some(source),
            Self::MissingMarker { .. }
            | Self::MissingMarkerValue { .. }
            | Self::MissingUsageValue { .. }
            | Self::InvalidNumber { .. } => None,
        }
    }
}

fn next_whitespace_token<'a>(source: &'a [u8], cursor: &mut usize) -> Option<&'a [u8]> {
    *cursor += source[*cursor..]
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    let start = *cursor;
    *cursor += source[*cursor..]
        .iter()
        .take_while(|byte| !byte.is_ascii_whitespace())
        .count();
    (start != *cursor).then_some(&source[start..*cursor])
}

fn seek_marker(
    source: &[u8],
    cursor: &mut usize,
    marker: &[u8],
    marker_name: &'static str,
) -> Result<(), SkillFactoryLoadError> {
    while let Some(token) = next_whitespace_token(source, cursor) {
        if token == marker {
            return Ok(());
        }
    }
    Err(SkillFactoryLoadError::MissingMarker {
        marker: marker_name,
    })
}

fn read_named_u32(
    source: &[u8],
    cursor: &mut usize,
    marker: &[u8],
    marker_name: &'static str,
) -> Result<u32, SkillFactoryLoadError> {
    seek_marker(source, cursor, marker, marker_name)?;
    let value = next_whitespace_token(source, cursor)
        .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: marker_name })?;
    parse_legacy_u32(value, marker_name)
}

fn parse_legacy_u32(value: &[u8], field: &'static str) -> Result<u32, SkillFactoryLoadError> {
    let (negative, digits) = match value.first() {
        Some(b'+') => (false, &value[1..]),
        Some(b'-') => (true, &value[1..]),
        _ => (false, value),
    };
    let magnitude = parse_decimal_magnitude(digits, field, value)?;
    if magnitude > u64::from(u32::MAX) {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: value.to_vec(),
        });
    }
    let value = magnitude as u32;
    Ok(if negative { value.wrapping_neg() } else { value })
}

fn parse_legacy_i32(value: &[u8], field: &'static str) -> Result<i32, SkillFactoryLoadError> {
    let (negative, digits) = match value.first() {
        Some(b'+') => (false, &value[1..]),
        Some(b'-') => (true, &value[1..]),
        _ => (false, value),
    };
    let magnitude = parse_decimal_magnitude(digits, field, value)?;
    let limit = if negative {
        i64::from(i32::MAX) + 1
    } else {
        i64::from(i32::MAX)
    } as u64;
    if magnitude > limit {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: value.to_vec(),
        });
    }
    if negative && magnitude == (i64::from(i32::MAX) + 1) as u64 {
        Ok(i32::MIN)
    } else if negative {
        Ok(-(magnitude as i32))
    } else {
        Ok(magnitude as i32)
    }
}

fn parse_decimal_magnitude(
    digits: &[u8],
    field: &'static str,
    original: &[u8],
) -> Result<u64, SkillFactoryLoadError> {
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: original.to_vec(),
        });
    }
    digits.iter().try_fold(0_u64, |value, digit| {
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(*digit - b'0')))
            .ok_or_else(|| SkillFactoryLoadError::InvalidNumber {
                field,
                value: original.to_vec(),
            })
    })
}

/// Safe serialization boundary для невозможного legacy registry-state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SkillFactorySerializeError {
    EntryCount {
        count: usize,
    },
    Skill {
        key: u32,
        source: SkillSerializeError,
    },
    RecordLength {
        key: u32,
        length: usize,
    },
}

impl fmt::Display for SkillFactorySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCount { count } => write!(
                formatter,
                "skill cache содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::Skill { key, source } => {
                write!(formatter, "skill slot 0x{key:08X}: {source}")
            }
            Self::RecordLength { key, length } => write!(
                formatter,
                "skill slot 0x{key:08X} имеет record длиной {length} байт вне u32"
            ),
        }
    }
}

impl Error for SkillFactorySerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Skill { source, .. } => Some(source),
            Self::EntryCount { .. } | Self::RecordLength { .. } => None,
        }
    }
}

// Сырой C++ ниже остаётся документацией ещё не восстановленных loaders и
// usage-name cache, а не Rust-реализацией.
