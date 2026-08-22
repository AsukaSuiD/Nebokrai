//! Конфигурация синтеза исторического Miracle.
//!
//! Статус World `CSynthesis::LoadSynthesisList` RVA `0x0008EE50` и
//! `AddToByteArray` RVA `0x0008D560`: `IMPLEMENTED`; static queries и Game
//! decoder ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:283`.
//!
//! Wire сначала содержит ordered broadcast map: signed count, затем
//! `u16 tag + C-string`. После него идут signed recipe count и vector recipes:
//! `u32 index + u16 probability + u16 type + u32 goods + i32 coins +
//! i32 prestige + u16 broadcast_tag + C-string key + signed formula count`,
//! затем пары `u32 goods + u32 amount`.
//!
//! Порядок `probability -> type` намеренно обратен layout исходного
//! `tagSynthesis`, где type лежит раньше probability; точный Game decoder также
//! ждёт wire-порядок. Rust не воспроизводит layout/сырой `char*`, но сохраняет
//! этот compatibility quirk явно. `BTreeMap`, `Vec` и owned bytes заменяют
//! только MSVC containers и ручной lifetime.
//!
//! Loader очищает recipes до открытия, но намеренно не очищает broadcast map:
//! это разные static owners в EXE, и частично прочитанные Broadcast остаются
//! после последующей ошибки Item. XML parsing выполняет `quick-xml`; узкий
//! pre-pass сохраняет TinyXML acceptance historic unquoted attributes.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisFormula {
    pub(crate) goods_index: u32,
    pub(crate) amount: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisRecipe {
    pub(crate) synthesis_index: u32,
    pub(crate) synthesis_type: u16,
    pub(crate) probability: u16,
    pub(crate) goods_index: u32,
    pub(crate) coins: i32,
    pub(crate) prestige: i32,
    pub(crate) broadcast_tag: u16,
    pub(crate) key: Vec<u8>,
    pub(crate) formulas: Vec<SynthesisFormula>,
}

/// Safe owner двух исходных static containers `CSynthesis`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSynthesis {
    broadcasts: BTreeMap<u16, Vec<u8>>,
    recipes: Vec<SynthesisRecipe>,
}

impl CSynthesis {
    /// Сохраняет исходную map assignment семантику duplicate tag-а.
    pub(crate) fn insert_broadcast(&mut self, tag: u16, text: Vec<u8>) -> Option<Vec<u8>> {
        self.broadcasts.insert(tag, text)
    }

    pub(crate) fn push_recipe(&mut self, recipe: SynthesisRecipe) {
        self.recipes.push(recipe);
    }

    pub(crate) fn clear(&mut self) {
        self.broadcasts.clear();
        self.recipes.clear();
    }

    /// Exact `LoadSynthesisList` clear до resource-open затрагивает только recipes.
    pub(crate) fn clear_recipes(&mut self) {
        self.recipes.clear();
    }

    /// Загружает XML `Synthesis` с concrete lookup уже живого `CGoodsFactory`.
    pub(crate) fn load_from_bytes<GoodsLookup>(
        &mut self,
        source: &[u8],
        mut goods_lookup: GoodsLookup,
    ) -> Result<SynthesisLoadReport, SynthesisLoadError>
    where
        GoodsLookup: FnMut(&[u8]) -> (u32, Option<Vec<u8>>),
    {
        self.clear_recipes();
        let result = self.load_from_bytes_after_clear(
            source,
            &mut goods_lookup,
        );
        if result.is_err() {
            self.clear_recipes();
        }
        result
    }

    fn load_from_bytes_after_clear<GoodsLookup>(
        &mut self,
        source: &[u8],
        goods_lookup: &mut GoodsLookup,
    ) -> Result<SynthesisLoadReport, SynthesisLoadError>
    where
        GoodsLookup: FnMut(&[u8]) -> (u32, Option<Vec<u8>>),
    {
        let normalized = normalize_legacy_attributes(source);
        let mut reader = Reader::from_reader(normalized.as_slice());
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut broadcast_list_seen = false;
        let mut broadcast_list_complete = false;
        let mut broadcast_count = 0usize;
        let mut synthesis_list_seen = false;
        let mut active_item: Option<ActiveSynthesisItem> = None;

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(
                        &start,
                        depth,
                        &mut root_seen,
                        &mut broadcast_list_seen,
                        broadcast_list_complete,
                        &mut broadcast_count,
                        &mut synthesis_list_seen,
                        &mut active_item,
                        goods_lookup,
                    )?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(
                        &empty,
                        depth,
                        &mut root_seen,
                        &mut broadcast_list_seen,
                        broadcast_list_complete,
                        &mut broadcast_count,
                        &mut synthesis_list_seen,
                        &mut active_item,
                        goods_lookup,
                    )?;
                    if depth == 2 && empty.name().as_ref() == b"Item" {
                        let item = active_item
                            .take()
                            .expect("Item создаётся перед его empty-завершением");
                        self.push_item_if_has_formula(item);
                    }
                }
                Ok(Event::End(end)) => {
                    if depth == 0 {
                        return Err(SynthesisLoadError::InvalidFormat);
                    }
                    depth -= 1;
                    match (depth, end.name().as_ref()) {
                        (1, b"BroadcastList") => {
                            if broadcast_count == 0 {
                                return Err(SynthesisLoadError::InvalidBroadcastList);
                            }
                            broadcast_list_complete = true;
                        }
                        (2, b"Item") => {
                            let item = active_item
                                .take()
                                .ok_or(SynthesisLoadError::InvalidFormat)?;
                            self.push_item_if_has_formula(item);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(SynthesisLoadError::InvalidFormat),
            }
            buffer.clear();
        }

        if !root_seen || !broadcast_list_seen || !broadcast_list_complete || !synthesis_list_seen {
            return Err(SynthesisLoadError::InvalidFormat);
        }
        if depth != 0 || active_item.is_some() {
            return Err(SynthesisLoadError::InvalidFormat);
        }
        Ok(SynthesisLoadReport {
            broadcasts: broadcast_count,
            recipes: self.recipes.len(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn process_start<GoodsLookup>(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        broadcast_list_seen: &mut bool,
        broadcast_list_complete: bool,
        broadcast_count: &mut usize,
        synthesis_list_seen: &mut bool,
        active_item: &mut Option<ActiveSynthesisItem>,
        goods_lookup: &mut GoodsLookup,
    ) -> Result<(), SynthesisLoadError>
    where
        GoodsLookup: FnMut(&[u8]) -> (u32, Option<Vec<u8>>),
    {
        let qualified_name = start.name();
        let name = qualified_name.as_ref();
        if !*root_seen {
            if name != b"Synthesis" {
                return Err(SynthesisLoadError::InvalidFormat);
            }
            *root_seen = true;
        } else if depth == 1 && name == b"BroadcastList" {
            *broadcast_list_seen = true;
        } else if depth == 2 && name == b"Broadcast" {
            if !*broadcast_list_seen || broadcast_list_complete {
                return Err(SynthesisLoadError::InvalidBroadcastList);
            }
            let tag = legacy_atoi(&required_attribute(
                start,
                b"id",
                SynthesisLoadError::InvalidBroadcast,
            )?) as u16;
            let value = required_attribute(start, b"value", SynthesisLoadError::InvalidBroadcast)?;
            self.broadcasts.insert(tag, value);
            *broadcast_count += 1;
        } else if depth == 1 && name == b"SynthesisList" {
            if !broadcast_list_complete {
                return Err(SynthesisLoadError::InvalidFormat);
            }
            *synthesis_list_seen = true;
        } else if depth == 2 && name == b"Item" {
            if !*synthesis_list_seen {
                return Err(SynthesisLoadError::InvalidFormat);
            }
            let synthesis_type = legacy_atoi(&required_attribute(
                start,
                b"lSType",
                SynthesisLoadError::MissingType,
            )?) as u16;
            if !is_valid_synthesis_type(synthesis_type) {
                return Err(SynthesisLoadError::InvalidType);
            }
            let original_name = required_attribute(
                start,
                b"strSynthesisName",
                SynthesisLoadError::MissingSynthesisName,
            )?;
            let (goods_index, key) = goods_lookup(&original_name);
            if goods_index == 0 {
                return Err(SynthesisLoadError::InvalidSynthesisGoods);
            }
            let key = key.ok_or(SynthesisLoadError::InvalidSynthesisGoods)?;
            let coins = legacy_atol(&required_attribute(
                start,
                b"lCoins",
                SynthesisLoadError::MissingCoins,
            )?);
            let prestige = legacy_atol(&required_attribute(
                start,
                b"lContribute",
                SynthesisLoadError::MissingContribute,
            )?);
            let probability = legacy_atoi(&required_attribute(
                start,
                b"dwProbability",
                SynthesisLoadError::MissingProbability,
            )?) as u16;
            let broadcast_tag = attribute(start, b"wBroadcastTag")
                .map(|value| legacy_atoi(&value) as u16)
                .unwrap_or(0);
            *active_item = Some(ActiveSynthesisItem {
                synthesis_type,
                probability,
                goods_index,
                coins,
                prestige,
                broadcast_tag,
                key,
                formulas: Vec::new(),
            });
        } else if depth == 3 && name == b"Formula" {
            let item = active_item.as_mut().ok_or(SynthesisLoadError::InvalidFormat)?;
            let original_name = required_attribute(
                start,
                b"strFormulaName",
                SynthesisLoadError::MissingFormulaName,
            )?;
            let (goods_index, _) = goods_lookup(&original_name);
            if goods_index == 0 {
                return Err(SynthesisLoadError::InvalidFormulaGoods);
            }
            let amount = legacy_atol(&required_attribute(
                start,
                b"wFormulaNum",
                SynthesisLoadError::MissingFormulaNumber,
            )?) as u32;
            item.formulas.push(SynthesisFormula { goods_index, amount });
        }
        Ok(())
    }

    fn push_item_if_has_formula(&mut self, item: ActiveSynthesisItem) {
        if item.formulas.is_empty() {
            return;
        }
        self.recipes.push(SynthesisRecipe {
            synthesis_index: self.recipes.len() as u32,
            synthesis_type: item.synthesis_type,
            probability: item.probability,
            goods_index: item.goods_index,
            coins: item.coins,
            prestige: item.prestige,
            broadcast_tag: item.broadcast_tag,
            key: item.key,
            formulas: item.formulas,
        });
    }

    /// Дописывает exact broadcast + recipe wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), SynthesisSerializeError> {
        write_count(
            destination,
            self.broadcasts.len(),
            SynthesisCount::Broadcasts,
        )?;
        for (&tag, text) in &self.broadcasts {
            ensure_c_string(text, SynthesisString::Broadcast { tag })?;
            destination.extend_from_slice(&tag.to_le_bytes());
            destination.extend_from_slice(text);
            destination.push(0);
        }

        write_count(
            destination,
            self.recipes.len(),
            SynthesisCount::Recipes,
        )?;
        for recipe in &self.recipes {
            ensure_c_string(
                &recipe.key,
                SynthesisString::RecipeKey {
                    synthesis_index: recipe.synthesis_index,
                },
            )?;
            destination.extend_from_slice(&recipe.synthesis_index.to_le_bytes());
            // Compatibility quirk: EXE отправляет probability раньше type.
            destination.extend_from_slice(&recipe.probability.to_le_bytes());
            destination.extend_from_slice(&recipe.synthesis_type.to_le_bytes());
            destination.extend_from_slice(&recipe.goods_index.to_le_bytes());
            destination.extend_from_slice(&recipe.coins.to_le_bytes());
            destination.extend_from_slice(&recipe.prestige.to_le_bytes());
            destination.extend_from_slice(&recipe.broadcast_tag.to_le_bytes());
            destination.extend_from_slice(&recipe.key);
            destination.push(0);
            write_count(
                destination,
                recipe.formulas.len(),
                SynthesisCount::Formulas {
                    synthesis_index: recipe.synthesis_index,
                },
            )?;
            for formula in &recipe.formulas {
                destination.extend_from_slice(&formula.goods_index.to_le_bytes());
                destination.extend_from_slice(&formula.amount.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveSynthesisItem {
    synthesis_type: u16,
    probability: u16,
    goods_index: u32,
    coins: i32,
    prestige: i32,
    broadcast_tag: u16,
    key: Vec<u8>,
    formulas: Vec<SynthesisFormula>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisLoadReport {
    pub(crate) broadcasts: usize,
    pub(crate) recipes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisLoadError {
    InvalidFormat,
    InvalidBroadcastList,
    InvalidBroadcast,
    MissingType,
    InvalidType,
    MissingSynthesisName,
    InvalidSynthesisGoods,
    MissingCoins,
    MissingContribute,
    MissingProbability,
    MissingFormulaName,
    InvalidFormulaGoods,
    MissingFormulaNumber,
}

impl SynthesisLoadError {
    pub(crate) const fn log_payload(self) -> &'static [u8] {
        match self {
            Self::InvalidFormat => b"error: error format OF compose FILE ...!!",
            Self::InvalidBroadcastList => b"error: error format OF compose (Broadcast) file!!",
            Self::InvalidBroadcast => b"error:error format OF compose (Broadcast) FILE!!",
            Self::MissingType => b"error: there is no lSType configure OF compose file!! ",
            Self::InvalidType => b"error: compose FILE [lSType] ...is wrong.!!!",
            Self::MissingSynthesisName => {
                b"error: the original name of compound(strSynthesisName) is not exist!!!"
            }
            Self::InvalidSynthesisGoods => {
                b"error: original name of compose file is wrong,the sequence number is not exist!"
            }
            Self::MissingCoins => b"error: there is no prop quantity configured!!!",
            Self::MissingContribute => b"error: there is no national contribute configured!!",
            Self::MissingProbability => b"error:there is no composite probability configured!!",
            Self::MissingFormulaName => b"error:there is no strFormulaName configured!!",
            Self::InvalidFormulaGoods => {
                b"error: the original name of compound is wrong,the sequence number is not exist!"
            }
            Self::MissingFormulaNumber => b"error: there is no wFormulaNum configured!!",
        }
    }
}

fn attribute(start: &BytesStart<'_>, expected: &[u8]) -> Option<Vec<u8>> {
    start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == expected)
        .map(|attribute| attribute.value.into_owned())
}

fn required_attribute(
    start: &BytesStart<'_>,
    expected: &[u8],
    error: SynthesisLoadError,
) -> Result<Vec<u8>, SynthesisLoadError> {
    attribute(start, expected).ok_or(error)
}

fn is_valid_synthesis_type(value: u16) -> bool {
    (100..401).contains(&value)
        || (500..510).contains(&value)
        || (600..610).contains(&value)
        || (700..710).contains(&value)
        || matches!(value, 800 | 900)
}

fn legacy_atoi(value: &[u8]) -> i32 {
    legacy_atol(value)
}

/// `_atol` decimal-prefix semantics без signed-overflow UB CRT.
fn legacy_atol(value: &[u8]) -> i32 {
    let mut bytes = value.iter().copied().skip_while(u8::is_ascii_whitespace).peekable();
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

/// TinyXML accepts the historic unquoted ASCII attributes of `synthesis.xml`.
fn normalize_legacy_attributes(source: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(source.len());
    let mut index = 0;
    while index < source.len() {
        if source[index..].starts_with(b"<!--") {
            let end = source[index + 4..]
                .windows(3)
                .position(|window| window == b"-->")
                .map(|offset| index + 4 + offset + 3)
                .unwrap_or(source.len());
            normalized.extend_from_slice(&source[index..end]);
            index = end;
            continue;
        }
        if source[index] != b'<' {
            normalized.push(source[index]);
            index += 1;
            continue;
        }
        normalized.push(b'<');
        index += 1;
        while index < source.len() && source[index] != b'>' {
            if source[index] != b'=' {
                normalized.push(source[index]);
                index += 1;
                continue;
            }
            normalized.push(b'=');
            index += 1;
            while index < source.len() && source[index].is_ascii_whitespace() {
                normalized.push(source[index]);
                index += 1;
            }
            if index == source.len() || matches!(source[index], b'\'' | b'"') {
                continue;
            }
            normalized.push(b'"');
            while index < source.len()
                && !source[index].is_ascii_whitespace()
                && source[index] != b'>'
                && source[index] != b'/'
            {
                normalized.push(source[index]);
                index += 1;
            }
            normalized.push(b'"');
        }
        if index < source.len() {
            normalized.push(b'>');
            index += 1;
        }
    }
    normalized
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisCount {
    Broadcasts,
    Recipes,
    Formulas { synthesis_index: u32 },
}

impl fmt::Display for SynthesisCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broadcasts => formatter.write_str("broadcast-записей синтеза"),
            Self::Recipes => formatter.write_str("рецептов синтеза"),
            Self::Formulas { synthesis_index } => {
                write!(formatter, "формул рецепта {synthesis_index}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisString {
    Broadcast { tag: u16 },
    RecipeKey { synthesis_index: u32 },
}

impl fmt::Display for SynthesisString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broadcast { tag } => write!(formatter, "broadcast-текст с tag {tag}"),
            Self::RecipeKey { synthesis_index } => {
                write!(formatter, "ключ рецепта {synthesis_index}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisSerializeError {
    CountOutOfRange {
        field: SynthesisCount,
        count: usize,
    },
    StringContainsNul {
        field: SynthesisString,
    },
}

impl fmt::Display for SynthesisSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { field, count } => write!(
                formatter,
                "количество {field} ({count}) не помещается в signed 32-битный диапазон"
            ),
            Self::StringContainsNul { field } => write!(formatter, "{field} содержит внутренний NUL"),
        }
    }
}

impl Error for SynthesisSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    field: SynthesisCount,
) -> Result<(), SynthesisSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| SynthesisSerializeError::CountOutOfRange { field, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn ensure_c_string(value: &[u8], field: SynthesisString) -> Result<(), SynthesisSerializeError> {
    if value.contains(&0) {
        return Err(SynthesisSerializeError::StringContainsNul { field });
    }
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация XML loader-а, static
// queries и Game decoder-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp

// ============================================================================
// FUNCTION: Catch@005c0171
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x001C0171
// ADDRESS: 005c0171
// PROTOTYPE: undefined Catch@005c0171()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::CheckType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:367
// RVA: 0x001C1070
// ADDRESS: 005c1070
// PROTOTYPE: int __cdecl CheckType(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:408
// RVA: 0x001C1120
// ADDRESS: 005c1120
// PROTOTYPE: ushort __cdecl GetProbability(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetGoodsIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:418
// RVA: 0x001C1150
// ADDRESS: 005c1150
// PROTOTYPE: ulong __cdecl GetGoodsIndex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetCoins
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:439
// RVA: 0x001C1180
// ADDRESS: 005c1180
// PROTOTYPE: long __cdecl GetCoins(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetPrestige
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:450
// RVA: 0x001C11B0
// ADDRESS: 005c11b0
// PROTOTYPE: long __cdecl GetPrestige(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetKey
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:460
// RVA: 0x001C11E0
// ADDRESS: 005c11e0
// PROTOTYPE: char * __cdecl GetKey(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetBroadcastTag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:469
// RVA: 0x001C1210
// ADDRESS: 005c1210
// PROTOTYPE: ushort __cdecl GetBroadcastTag(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetBroadcast
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:536
// RVA: 0x001C1310
// ADDRESS: 005c1310
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __cdecl GetBroadcast(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetFormula
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:479
// RVA: 0x001C26B0
// ADDRESS: 005c26b0
// PROTOTYPE: long __cdecl GetFormula(ulong param_1, vector<CSynthesis::tagFormula,std::allocator<CSynthesis::tagFormula>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetInitForm
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:389
// RVA: 0x001C2BF0
// ADDRESS: 005c2bf0
// PROTOTYPE: bool __cdecl GetInitForm(ushort param_1, vector<CSynthesis::tagNameAndIndex,std::allocator<CSynthesis::tagNameAndIndex>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:323
// RVA: 0x001C31B0
// ADDRESS: 005c31b0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp

// ============================================================================
// FUNCTION: Catch@00488321
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088321
// ADDRESS: 00488321
// PROTOTYPE: undefined Catch@00488321()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004887a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000887A1
// ADDRESS: 004887a1
// PROTOTYPE: undefined Catch@004887a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488831
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088831
// ADDRESS: 00488831
// PROTOTYPE: undefined Catch@00488831()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000888D0
// ADDRESS: 004888d0
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::~stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088960
// ADDRESS: 00488960
// PROTOTYPE: void __thiscall ~stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088A10
// ADDRESS: 00488a10
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(stTaoZhuangAddItem * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488af1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088AF1
// ADDRESS: 00488af1
// PROTOTYPE: undefined Catch@00488af1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488ba6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088BA6
// ADDRESS: 00488ba6
// PROTOTYPE: undefined Catch@00488ba6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004891f1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000891F1
// ADDRESS: 004891f1
// PROTOTYPE: undefined Catch@004891f1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004893f1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000893F1
// ADDRESS: 004893f1
// PROTOTYPE: undefined Catch@004893f1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00089410
// ADDRESS: 00489410
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::~stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000894C0
// ADDRESS: 004894c0
// PROTOTYPE: void __thiscall ~stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000895D0
// ADDRESS: 004895d0
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(stTaoZhuangItemNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048973f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x0008973F
// ADDRESS: 0048973f
// PROTOTYPE: undefined Catch@0048973f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:283
// RVA: 0x0008D560
// ADDRESS: 0048d560
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::LoadSynthesisList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:34
// RVA: 0x0008EE50
// ADDRESS: 0048ee50
// PROTOTYPE: int __cdecl LoadSynthesisList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@005314a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x001314A0
// ADDRESS: 005314a0
// PROTOTYPE: undefined Unwind@005314a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//









// ============================================================================
// FUNCTION: Unwind@00531560
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00131560
// ADDRESS: 00531560
// PROTOTYPE: undefined Unwind@00531560()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//








// COMPONENT_VARIANT_END: WorldServer
