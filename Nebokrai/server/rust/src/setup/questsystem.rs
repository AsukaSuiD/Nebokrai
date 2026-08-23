//! Конфигурация `CQuestSystem` из WorldServer и GameServer, подтверждённая
//! точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/setup/questsystem.cpp/.h`.
//!
//! Wire содержит max count, level difference, три script C-строки и ordered
//! quest records. Внутри record region/x/y идут до effect вопреки C++ layout;
//! ключ map отдельно не передаётся.
//!
//! `Load` очищает map, читает `Quest.ini` и накладывает `QuestEx.ini` по ID.
//! StringTable miss даёт пустую строку, script path проходит `ReplaceLine` и
//! ASCII lowercase. Отсутствие второго ресурса сохраняет основной map и
//! возвращает ошибку вместо прежнего null-dereference.
//!
//! Game decoder присваивает scalars и три script-строки до очистки map, затем
//! публикует только полностью прочитанные records; duplicate ID заменяет
//! прежний record. Process singleton технически хранится непосредственно в
//! `CGame`, без изменения lookup- и wire-семантики.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QuestEntry {
    pub(crate) id: u16,
    pub(crate) old: u32,
    pub(crate) quest_type: u32,
    pub(crate) level: u32,
    pub(crate) difficulty: u32,
    pub(crate) track: u32,
    pub(crate) short_description: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) description: Vec<u8>,
    pub(crate) abandon_script: Vec<u8>,
    pub(crate) complete_script: Vec<u8>,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) effect_id: i32,
    pub(crate) display: bool,
}

impl Default for QuestEntry {
    fn default() -> Self {
        Self {
 // Конструктор `tagQuest` задаёт `dwType = 2`; QuestEx затем
 // перезаписывает его своим parsed value.
            quest_type: 2,
            id: 0,
            old: 0,
            level: 0,
            difficulty: 0,
            track: 0,
            short_description: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            abandon_script: Vec::new(),
            complete_script: Vec::new(),
            region_id: 0,
            tile_x: 0,
            tile_y: 0,
            effect_id: 0,
            display: false,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CQuestSystem {
    pub(crate) max_quest_count: i32,
    pub(crate) level_difference: u32,
    pub(crate) player_login_script: Vec<u8>,
    pub(crate) player_level_up_script: Vec<u8>,
    pub(crate) player_died_script: Vec<u8>,
    quests: BTreeMap<u16, QuestEntry>,
}

impl CQuestSystem {
 /// Оригинал начало `Load`: очищается только map заданий; scalar и script
 /// поля сохраняются, если первый resource `Data/Quest.ini` недоступен.
    pub(crate) fn clear_quests_for_load(&mut self) {
        self.quests.clear();
    }

 /// Выполняет безопасную resource-границу оригинал World `Load`.
 ///
 /// Основной source записывается непосредственно в owner, как EXE. Поэтому
 /// уже распарсенные scalar/records сохраняются и при повреждённом хвосте;
 /// временная копия здесь намеренно не подменяет наблюдаемое частичное
 /// обновление.
    pub(crate) fn load_from_resources<ResolveString>(
        &mut self,
        quest_source: Option<&[u8]>,
        quest_ex_source: Option<&[u8]>,
        resolve_string: &mut ResolveString,
    ) -> QuestSystemLoadReport
    where
        ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear_quests_for_load();
        let Some(quest_source) = quest_source else {
            return QuestSystemLoadReport {
                primary_records: 0,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::QuestFileMissing,
            };
        };

        let mut primary = QuestInput::new(quest_source);
        if !self.load_primary_header(&mut primary) {
            return QuestSystemLoadReport {
                primary_records: 0,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::PrimaryFormatStopped,
            };
        }

        let primary_records = self.load_primary_nodes(&mut primary, resolve_string);
        let Some(quest_ex_source) = quest_ex_source else {
            return QuestSystemLoadReport {
                primary_records,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::QuestExFileMissing,
            };
        };

        let mut extension = QuestInput::new(quest_ex_source);
        let extension_records = self.load_extension_nodes(&mut extension, resolve_string);
        QuestSystemLoadReport {
            primary_records,
            extension_records,
            completion: QuestSystemLoadCompletion::Loaded,
        }
    }

    fn load_primary_header(&mut self, input: &mut QuestInput<'_>) -> bool {
 // Labels здесь в EXE просто извлекаются в буфер и не сравниваются.
        let Some(_) = input.token() else { return false };
        let Some(max_quest_count) = input.i32() else { return false };
        self.max_quest_count = max_quest_count;
        let Some(_) = input.token() else { return false };
        let Some(level_difference) = input.u32() else { return false };
        self.level_difference = level_difference;
        let Some(_) = input.token() else { return false };
        let Some(player_login_script) = input.token() else { return false };
        self.player_login_script = normalize_script_path(player_login_script);
        let Some(_) = input.token() else { return false };
        let Some(player_level_up_script) = input.token() else { return false };
        self.player_level_up_script = normalize_script_path(player_level_up_script);
        let Some(_) = input.token() else { return false };
        let Some(player_died_script) = input.token() else { return false };
        self.player_died_script = normalize_script_path(player_died_script);
        true
    }

    fn load_primary_nodes<ResolveString>(
        &mut self,
        input: &mut QuestInput<'_>,
        resolve_string: &mut ResolveString,
    ) -> usize
    where
        ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        let mut loaded = 0;
        while input.read_to(b"<Node>") {
            let Some(quest) = read_primary_node(input, resolve_string) else {
                break;
            };
            self.quests.insert(quest.id, quest);
            loaded += 1;
        }
        loaded
    }

    fn load_extension_nodes<ResolveString>(
        &mut self,
        input: &mut QuestInput<'_>,
        resolve_string: &mut ResolveString,
    ) -> usize
    where
        ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        let mut loaded = 0;
        while input.read_to(b"<Node>") {
            let Some(quest) = read_extension_node(input, resolve_string) else {
                break;
            };
            self.quests.insert(quest.id, quest);
            loaded += 1;
        }
        loaded
    }

    pub(crate) fn insert(&mut self, quest: QuestEntry) -> Option<QuestEntry> {
        self.quests.insert(quest.id, quest)
    }

    pub(crate) fn quests(&self) -> &BTreeMap<u16, QuestEntry> {
        &self.quests
    }

    pub(crate) fn quest_data_by_id(&self, quest_id: u16) -> Option<&QuestEntry> {
        self.quests.get(&quest_id)
    }

    pub(crate) fn complete_script_by_id(&self, quest_id: u16) -> Option<&[u8]> {
        self.quest_data_by_id(quest_id)
            .map(|quest| quest.complete_script.as_slice())
    }

    pub(crate) fn disband_script_by_id(&self, quest_id: u16) -> Option<&[u8]> {
        self.quest_data_by_id(quest_id)
            .map(|quest| quest.abandon_script.as_slice())
    }

    /// Декодирует GameServer snapshot RVA `0x000627A0` с исходным порядком
    /// частичных side effects.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<QuestSystemDecodeReport, QuestSystemDecodeError> {
        self.max_quest_count = read_quest_i32(source, cursor, QuestWireField::MaxQuestCount)?;
        self.level_difference = read_quest_u32(source, cursor, QuestWireField::LevelDifference)?;
        self.player_login_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerLoginScript)?;
        self.player_level_up_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerLevelUpScript)?;
        self.player_died_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerDiedScript)?;

        self.quests.clear();
        let count = read_quest_i32(source, cursor, QuestWireField::QuestCount)?;
        let mut decoded = 0;
        for _ in 0..count.max(0) {
            let id = read_quest_u16(source, cursor, QuestWireField::QuestId)?;
            let quest = QuestEntry {
                id,
                old: read_quest_u32(source, cursor, QuestWireField::QuestOld)?,
                quest_type: read_quest_u32(source, cursor, QuestWireField::QuestType)?,
                level: read_quest_u32(source, cursor, QuestWireField::QuestLevel)?,
                difficulty: read_quest_u32(source, cursor, QuestWireField::QuestDifficulty)?,
                track: read_quest_u32(source, cursor, QuestWireField::QuestTrack)?,
                short_description: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestShortDescription,
                )?,
                name: read_quest_c_string(source, cursor, QuestWireField::QuestName)?,
                description: read_quest_c_string(source, cursor, QuestWireField::QuestDescription)?,
                abandon_script: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestAbandonScript,
                )?,
                complete_script: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestCompleteScript,
                )?,
                region_id: read_quest_i32(source, cursor, QuestWireField::QuestRegionId)?,
                tile_x: read_quest_i32(source, cursor, QuestWireField::QuestTileX)?,
                tile_y: read_quest_i32(source, cursor, QuestWireField::QuestTileY)?,
                effect_id: read_quest_i32(source, cursor, QuestWireField::QuestEffectId)?,
                display: read_quest_u8(source, cursor, QuestWireField::QuestDisplay)? != 0,
            };
            self.quests.insert(id, quest);
            decoded += 1;
        }
        Ok(QuestSystemDecodeReport {
            declared: count,
            decoded,
            retained: self.quests.len(),
        })
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), QuestSystemSerializationBlock> {
        let count = i32::try_from(self.quests.len()).map_err(|_| {
            QuestSystemSerializationBlock::QuestCountOutOfRange {
                count: self.quests.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.max_quest_count.to_le_bytes());
        payload.extend_from_slice(&self.level_difference.to_le_bytes());
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLoginScript,
            &self.player_login_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLevelUpScript,
            &self.player_level_up_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerDiedScript,
            &self.player_died_script,
        )?;
        payload.extend_from_slice(&count.to_le_bytes());
        for quest in self.quests.values() {
            payload.extend_from_slice(&quest.id.to_le_bytes());
            for value in [
                quest.old,
                quest.quest_type,
                quest.level,
                quest.difficulty,
                quest.track,
            ] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
            for (field, value) in [
                (
                    QuestStringField::ShortDescription,
                    quest.short_description.as_slice(),
                ),
                (QuestStringField::Name, quest.name.as_slice()),
                (QuestStringField::Description, quest.description.as_slice()),
                (
                    QuestStringField::AbandonScript,
                    quest.abandon_script.as_slice(),
                ),
                (
                    QuestStringField::CompleteScript,
                    quest.complete_script.as_slice(),
                ),
            ] {
                write_quest_string(&mut payload, Some(quest.id), field, value)?;
            }
            payload.extend_from_slice(&quest.region_id.to_le_bytes());
            payload.extend_from_slice(&quest.tile_x.to_le_bytes());
            payload.extend_from_slice(&quest.tile_y.to_le_bytes());
            payload.extend_from_slice(&quest.effect_id.to_le_bytes());
            payload.push(u8::from(quest.display));
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QuestSystemDecodeReport {
    pub(crate) declared: i32,
    pub(crate) decoded: usize,
    pub(crate) retained: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestWireField {
    MaxQuestCount,
    LevelDifference,
    PlayerLoginScript,
    PlayerLevelUpScript,
    PlayerDiedScript,
    QuestCount,
    QuestId,
    QuestOld,
    QuestType,
    QuestLevel,
    QuestDifficulty,
    QuestTrack,
    QuestShortDescription,
    QuestName,
    QuestDescription,
    QuestAbandonScript,
    QuestCompleteScript,
    QuestRegionId,
    QuestTileX,
    QuestTileY,
    QuestEffectId,
    QuestDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestSystemDecodeError {
    UnexpectedEnd {
        field: QuestWireField,
        offset: usize,
        needed: usize,
        available: usize,
    },
    UnterminatedString {
        field: QuestWireField,
        offset: usize,
    },
}

impl fmt::Display for QuestSystemDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "QuestSystem snapshot обрывается на {field:?} в {offset}: нужно {needed}, доступно {available}"
            ),
            Self::UnterminatedString { field, offset } => write!(
                formatter,
                "QuestSystem snapshot содержит незавершённую строку {field:?} в {offset}"
            ),
        }
    }
}

impl Error for QuestSystemDecodeError {}

fn read_quest_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    size: usize,
    field: QuestWireField,
) -> Result<&'a [u8], QuestSystemDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(size)) else {
        return Err(QuestSystemDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: size,
            available,
        });
    };
    *cursor += size;
    Ok(bytes)
}

fn read_quest_i32(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<i32, QuestSystemDecodeError> {
    Ok(i32::from_le_bytes(
        read_quest_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("quest signed long уже проверен"),
    ))
}

fn read_quest_u32(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u32, QuestSystemDecodeError> {
    Ok(u32::from_le_bytes(
        read_quest_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("quest unsigned long уже проверен"),
    ))
}

fn read_quest_u16(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u16, QuestSystemDecodeError> {
    Ok(u16::from_le_bytes(
        read_quest_bytes(source, cursor, 2, field)?
            .try_into()
            .expect("quest unsigned short уже проверен"),
    ))
}

fn read_quest_u8(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u8, QuestSystemDecodeError> {
    Ok(read_quest_bytes(source, cursor, 1, field)?[0])
}

fn read_quest_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<Vec<u8>, QuestSystemDecodeError> {
    let offset = *cursor;
    let tail = source
        .get(offset..)
        .ok_or(QuestSystemDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 1,
            available: 0,
        })?;
    let Some(length) = tail.iter().position(|byte| *byte == 0) else {
        return Err(QuestSystemDecodeError::UnterminatedString { field, offset });
    };
    *cursor += length + 1;
    Ok(tail[..length].to_vec())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestSystemLoadCompletion {
    Loaded,
    QuestFileMissing,
    QuestExFileMissing,
    PrimaryFormatStopped,
}

/// Число вставок каждой фазы; дубликат ID считается отдельной обработанной
/// записью, хотя `std::map::operator[]` сохраняет только последнюю.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QuestSystemLoadReport {
    pub(crate) primary_records: usize,
    pub(crate) extension_records: usize,
    pub(crate) completion: QuestSystemLoadCompletion,
}

struct QuestInput<'source> {
    source: &'source [u8],
    cursor: usize,
}

impl<'source> QuestInput<'source> {
    fn new(source: &'source [u8]) -> Self {
        Self { source, cursor: 0 }
    }

    fn token(&mut self) -> Option<&'source [u8]> {
        while self.cursor < self.source.len() && self.source[self.cursor].is_ascii_whitespace() {
            self.cursor += 1;
        }
        let start = self.cursor;
        while self.cursor < self.source.len() && !self.source[self.cursor].is_ascii_whitespace() {
            self.cursor += 1;
        }
        (start != self.cursor).then_some(&self.source[start..self.cursor])
    }

    fn i32(&mut self) -> Option<i32> {
        let token = self.token()?;
        std::str::from_utf8(token).ok()?.parse().ok()
    }

    fn u32(&mut self) -> Option<u32> {
        let token = self.token()?;
        std::str::from_utf8(token).ok()?.parse().ok()
    }

    fn u16(&mut self) -> Option<u16> {
        let token = self.token()?;
        std::str::from_utf8(token).ok()?.parse().ok()
    }

    fn read_to(&mut self, marker: &[u8]) -> bool {
        while let Some(token) = self.token() {
            if token == marker {
                return true;
            }
        }
        false
    }

    fn discard_to_line_end(&mut self) {
        while self.cursor < self.source.len() && self.source[self.cursor] != b'\n' {
            self.cursor += 1;
        }
        if self.cursor < self.source.len() {
            self.cursor += 1;
        }
    }

    fn line(&mut self) -> Option<&'source [u8]> {
        if self.cursor >= self.source.len() {
            return None;
        }
        let start = self.cursor;
        while self.cursor < self.source.len() && self.source[self.cursor] != b'\n' {
            self.cursor += 1;
        }
        let mut end = self.cursor;
        if end > start && self.source[end - 1] == b'\r' {
            end -= 1;
        }
        if self.cursor < self.source.len() {
            self.cursor += 1;
        }
        Some(&self.source[start..end])
    }

    fn marked_text(&mut self, append_lines: bool) -> Option<Vec<u8>> {
        if !self.read_to(b"<Start>") {
            return None;
        }
        self.discard_to_line_end();
        let mut text = Vec::new();
        while let Some(line) = self.line() {
            if line.windows(b"<End>".len()).any(|window| window == b"<End>") {
                return Some(text);
            }
            if append_lines {
                text.extend_from_slice(line);
            } else {
                text.clear();
                text.extend_from_slice(line);
            }
        }
        None
    }
}

fn resolve_or_empty<ResolveString>(resolve_string: &mut ResolveString, string_id: &[u8]) -> Vec<u8>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_string(string_id).unwrap_or_default()
}

fn read_primary_node<ResolveString>(
    input: &mut QuestInput<'_>,
    resolve_string: &mut ResolveString,
) -> Option<QuestEntry>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let mut quest = QuestEntry::default();
    let _id_label = input.token()?;
    quest.id = input.u16()?;
    let _name_label = input.token()?;
    quest.name = resolve_or_empty(resolve_string, input.token()?);
    quest.description = resolve_or_empty(resolve_string, &input.marked_text(true)?);
    let _abandon_label = input.token()?;
    quest.abandon_script = normalize_script_path(input.token()?);
    let _complete_label = input.token()?;
    quest.complete_script = normalize_script_path(input.token()?);
    let _coordinate_label = input.token()?;
    quest.region_id = input.i32()?;
    quest.tile_x = input.i32()?;
    quest.tile_y = input.i32()?;
    let _effect_label = input.token()?;
    quest.effect_id = input.i32()?;
    let _display_label = input.token()?;
    quest.display = input.i32()? != 0;
    Some(quest)
}

fn read_extension_node<ResolveString>(
    input: &mut QuestInput<'_>,
    resolve_string: &mut ResolveString,
) -> Option<QuestEntry>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let mut quest = QuestEntry { old: 1, ..QuestEntry::default() };
    let _id_label = input.token()?;
    quest.id = input.u16()?;
    let _type_label = input.token()?;
    quest.quest_type = input.u32()?;
    let _level_label = input.token()?;
    quest.level = input.u32()?;
    let _difficulty_label = input.token()?;
    quest.difficulty = input.u32()?;
    let _tracking_label = input.token()?;
    quest.track = input.u32()?;
    let _description_label = input.token()?;
    quest.short_description = resolve_or_empty(resolve_string, &input.marked_text(true)?);
    let _name_label = input.token()?;
    quest.name = resolve_or_empty(resolve_string, input.token()?);
    quest.description = resolve_or_empty(resolve_string, &input.marked_text(false)?);
    let _abandon_label = input.token()?;
    quest.abandon_script = normalize_script_path(input.token()?);
    let _complete_label = input.token()?;
    quest.complete_script = normalize_script_path(input.token()?);
    let _coordinate_label = input.token()?;
    quest.region_id = input.i32()?;
    quest.tile_x = input.i32()?;
    quest.tile_y = input.i32()?;
    let _effect_label = input.token()?;
    quest.effect_id = input.i32()?;
    let _display_label = input.token()?;
    quest.display = input.i32()? != 0;
    Some(quest)
}

fn normalize_script_path(value: &[u8]) -> Vec<u8> {
    value
        .iter()
        .map(|&byte| match byte {
            b'\\' => b'/',
            b'A'..=b'Z' => byte + (b'a' - b'A'),
            _ => byte,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestStringField {
    PlayerLoginScript,
    PlayerLevelUpScript,
    PlayerDiedScript,
    ShortDescription,
    Name,
    Description,
    AbandonScript,
    CompleteScript,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QuestSystemSerializationBlock {
    QuestCountOutOfRange { count: usize },
    StringContainsNul {
        quest_id: Option<u16>,
        field: QuestStringField,
    },
}

impl fmt::Display for QuestSystemSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuestCountOutOfRange { count } => write!(
                formatter,
                "QuestSystem содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul { quest_id, field } => {
                let owner = quest_id
                    .map_or_else(|| "setup".to_owned(), |id| format!("quest {id}"));
                write!(
                    formatter,
                    "QuestSystem {owner}, поле {field:?} содержит внутренний NUL"
                )
            }
        }
    }
}

impl Error for QuestSystemSerializationBlock {}

fn write_quest_string(
    destination: &mut Vec<u8>,
    quest_id: Option<u16>,
    field: QuestStringField,
    value: &[u8],
) -> Result<(), QuestSystemSerializationBlock> {
    if value.contains(&0) {
        return Err(QuestSystemSerializationBlock::StringContainsNul {
            quest_id,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}
