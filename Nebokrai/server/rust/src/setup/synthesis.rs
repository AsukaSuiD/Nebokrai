//! Синтез `CSynthesis` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/synthesis.cpp`.
//!
//! Wire сначала передаёт ordered broadcast map, затем recipes и их formula
//! records. В recipe probability идёт раньше type, хотя C++ layout обратный;
//! парный decoder ожидает именно этот порядок.
//!
//! Loader очищает recipes, но сохраняет прежний broadcast map: это разные
//! static owners, и ошибка позднего Item не откатывает ранние Broadcast.
//! `quick-xml` с узкой нормализацией unquoted attributes заменяет TinyXML.
//! Game decoder, напротив, очищает оба owners. Recipe публикуется лишь после
//! полного временного formula-vector; safe NUL reader заменяет старый
//! безразмерный `_GetStringFromByteArray` и не воспроизводит его overflow.
//! Gameplay query-family `Get*` связана с goods-message lifecycle через
//! byte-exact recipe lookup, type validation и insertion-order forms/search.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSynthesis {
    broadcasts: BTreeMap<u16, Vec<u8>>,
    recipes: Vec<SynthesisRecipe>,
}

impl CSynthesis {
    pub(crate) fn recipe(&self, synthesis_index: u32) -> Option<&SynthesisRecipe> {
        self.recipes
            .iter()
            .find(|recipe| recipe.synthesis_index == synthesis_index)
    }

    pub(crate) fn forms(&self, synthesis_type: u16) -> Option<Vec<(&[u8], u32)>> {
        is_valid_synthesis_type(synthesis_type).then(|| {
            self.recipes
                .iter()
                .filter(|recipe| recipe.synthesis_type == synthesis_type)
                .map(|recipe| (recipe.key.as_slice(), recipe.synthesis_index))
                .collect()
        })
    }

    pub(crate) fn search_forms(&self, synthesis_type: u16, keyword: &[u8]) -> Vec<(&[u8], u32)> {
        if keyword.is_empty() {
            return Vec::new();
        }
        self.recipes
            .iter()
            .filter(|recipe| {
                recipe.synthesis_type == synthesis_type
                    && recipe
                        .key
                        .windows(keyword.len())
                        .any(|candidate| candidate == keyword)
            })
            .map(|recipe| (recipe.key.as_slice(), recipe.synthesis_index))
            .collect()
    }

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

    pub(crate) fn clear_recipes(&mut self) {
        self.recipes.clear();
    }

    pub(crate) fn broadcasts(&self) -> &BTreeMap<u16, Vec<u8>> {
        &self.broadcasts
    }

    pub(crate) fn recipes(&self) -> &[SynthesisRecipe] {
        &self.recipes
    }

    pub(crate) fn load_from_bytes<GoodsLookup>(
        &mut self,
        source: &[u8],
        mut goods_lookup: GoodsLookup,
    ) -> Result<SynthesisLoadReport, SynthesisLoadError>
    where
        GoodsLookup: FnMut(&[u8]) -> (u32, Option<Vec<u8>>),
    {
        self.clear_recipes();
        let result = self.load_from_bytes_after_clear(source, &mut goods_lookup);
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
            let item = active_item
                .as_mut()
                .ok_or(SynthesisLoadError::InvalidFormat)?;
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
            item.formulas.push(SynthesisFormula {
                goods_index,
                amount,
            });
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

        write_count(destination, self.recipes.len(), SynthesisCount::Recipes)?;
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

    /// Воспроизводит `CSynthesis::DecordFromByteArray` GameServer.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<SynthesisDecodeReport, SynthesisDecodeError> {
        self.clear();

        let broadcast_count = read_wire_i32(source, cursor)?;
        for _ in 0..broadcast_count.max(0) {
            let tag = read_wire_u16(source, cursor)?;
            let text = read_wire_c_string(source, cursor)?;
            self.broadcasts.insert(tag, text);
        }

        let recipe_count = read_wire_i32(source, cursor)?;
        self.recipes
            .try_reserve(recipe_count.max(0) as usize)
            .map_err(|source| SynthesisDecodeError::Allocation {
                field: SynthesisDecodeField::Recipes,
                source,
            })?;
        for recipe_index in 0..recipe_count.max(0) {
            let synthesis_index = read_wire_u32(source, cursor)?;
            // Compatibility quirk: wire передаёт probability раньше type.
            let probability = read_wire_u16(source, cursor)?;
            let synthesis_type = read_wire_u16(source, cursor)?;
            let goods_index = read_wire_u32(source, cursor)?;
            let coins = read_wire_i32(source, cursor)?;
            let prestige = read_wire_i32(source, cursor)?;
            let broadcast_tag = read_wire_u16(source, cursor)?;
            let key = read_wire_c_string(source, cursor)?;
            let formula_count = read_wire_i32(source, cursor)?;
            let mut formulas = Vec::new();
            formulas
                .try_reserve(formula_count.max(0) as usize)
                .map_err(|source| SynthesisDecodeError::Allocation {
                    field: SynthesisDecodeField::Formulas {
                        recipe_index: recipe_index as usize,
                    },
                    source,
                })?;
            for _ in 0..formula_count.max(0) {
                formulas.push(SynthesisFormula {
                    goods_index: read_wire_u32(source, cursor)?,
                    amount: read_wire_u32(source, cursor)?,
                });
            }
            self.recipes.push(SynthesisRecipe {
                synthesis_index,
                synthesis_type,
                probability,
                goods_index,
                coins,
                prestige,
                broadcast_tag,
                key,
                formulas,
            });
        }

        Ok(SynthesisDecodeReport {
            broadcasts: self.broadcasts.len(),
            recipes: self.recipes.len(),
        })
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
    CountOutOfRange { field: SynthesisCount, count: usize },
    StringContainsNul { field: SynthesisString },
}

impl fmt::Display for SynthesisSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { field, count } => write!(
                formatter,
                "количество {field} ({count}) не помещается в signed 32-битный диапазон"
            ),
            Self::StringContainsNul { field } => {
                write!(formatter, "{field} содержит внутренний NUL")
            }
        }
    }
}

impl Error for SynthesisSerializeError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SynthesisDecodeReport {
    pub(crate) broadcasts: usize,
    pub(crate) recipes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisDecodeField {
    Recipes,
    Formulas { recipe_index: usize },
}

impl fmt::Display for SynthesisDecodeField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recipes => formatter.write_str("recipes синтеза"),
            Self::Formulas { recipe_index } => {
                write!(formatter, "formulas recipe #{recipe_index}")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum SynthesisDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    UnterminatedString {
        offset: usize,
        available: usize,
    },
    Allocation {
        field: SynthesisDecodeField,
        source: std::collections::TryReserveError,
    },
}

impl fmt::Display for SynthesisDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "Synthesis snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::UnterminatedString { offset, available } => write!(
                formatter,
                "Synthesis string с позиции {offset} не имеет NUL в {available} доступных байтах"
            ),
            Self::Allocation { field, .. } => {
                write!(formatter, "не удалось выделить память для {field}")
            }
        }
    }
}

impl Error for SynthesisDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            _ => None,
        }
    }
}

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

fn read_wire_u16(source: &[u8], cursor: &mut usize) -> Result<u16, SynthesisDecodeError> {
    Ok(u16::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, SynthesisDecodeError> {
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, SynthesisDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], SynthesisDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(SynthesisDecodeError::UnexpectedEnd {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер Synthesis scalar уже проверен"))
}

fn read_wire_c_string(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, SynthesisDecodeError> {
    let offset = *cursor;
    let tail = source
        .get(offset..)
        .ok_or(SynthesisDecodeError::UnexpectedEnd {
            offset,
            needed: 1,
            available: 0,
        })?;
    let Some(length) = tail.iter().position(|byte| *byte == 0) else {
        return Err(SynthesisDecodeError::UnterminatedString {
            offset,
            available: tail.len(),
        });
    };
    *cursor = offset + length + 1;
    Ok(tail[..length].to_vec())
}
