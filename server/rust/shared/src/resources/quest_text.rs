//! Текстовые определения по server/setup/questsystem.cpp/.h, World CQuestSystem::Load.
//! Частичные изменения и порядок Quest.ini → QuestEx.ini сохраняются.
//! Смещения и причины отказов — диагностика Rust; правила: docs/gameplay/quests.md.

use super::quest::{CQuestSystem, QuestEntry};

impl CQuestSystem {
    /// Очищает только записи; при недоступном основном файле общие поля сохраняются.
    pub fn clear_quests_for_load(&mut self) {
        self.quests.clear();
    }

    /// Применяет ресурсы по порядку без отката уже прочитанных полей и записей.
    /// Realm открывает расширение через callback после разбора основного списка.
    pub fn load_from_resources<ReadExtension, ResolveString>(
        &mut self,
        quest_source: Option<&[u8]>,
        read_extension: ReadExtension,
        resolve_string: &mut ResolveString,
    ) -> QuestSystemLoadReport
    where
        ReadExtension: FnOnce() -> Option<Vec<u8>>,
        ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear_quests_for_load();
        let Some(quest_source) = quest_source else {
            return QuestSystemLoadReport {
                primary_records: 0,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::QuestFileMissing,
                primary_error: None,
                extension_error: None,
            };
        };

        let mut primary = QuestInput::new(quest_source);
        if !self.load_primary_header(&mut primary) {
            return QuestSystemLoadReport {
                primary_records: 0,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::PrimaryFormatStopped,
                primary_error: primary.error,
                extension_error: None,
            };
        }

        let primary_records = self.load_primary_nodes(&mut primary, resolve_string);
        let Some(quest_ex_source) = read_extension() else {
            return QuestSystemLoadReport {
                primary_records,
                extension_records: 0,
                completion: QuestSystemLoadCompletion::QuestExFileMissing,
                primary_error: primary.error,
                extension_error: None,
            };
        };

        let mut extension = QuestInput::new(&quest_ex_source);
        let extension_records = self.load_extension_nodes(&mut extension, resolve_string);
        QuestSystemLoadReport {
            primary_records,
            extension_records,
            completion: if primary.error.is_some() || extension.error.is_some() {
                QuestSystemLoadCompletion::Partial
            } else {
                QuestSystemLoadCompletion::Loaded
            },
            primary_error: primary.error,
            extension_error: extension.error,
        }
    }

    fn load_primary_header(&mut self, input: &mut QuestInput<'_>) -> bool {
        // Метки извлекаются, но их текст не проверяется.
        let Some(_) = input.token("max_quest_count label") else {
            return false;
        };
        let Some(max_quest_count) = input.i32("max_quest_count") else {
            return false;
        };
        self.max_quest_count = max_quest_count;
        let Some(_) = input.token("level_difference label") else {
            return false;
        };
        let Some(level_difference) = input.u32("level_difference") else {
            return false;
        };
        self.level_difference = level_difference;
        let Some(_) = input.token("player_login_script label") else {
            return false;
        };
        let Some(player_login_script) = input.token("player_login_script") else {
            return false;
        };
        self.player_login_script = normalize_script_path(player_login_script);
        let Some(_) = input.token("player_level_up_script label") else {
            return false;
        };
        let Some(player_level_up_script) = input.token("player_level_up_script") else {
            return false;
        };
        self.player_level_up_script = normalize_script_path(player_level_up_script);
        let Some(_) = input.token("player_died_script label") else {
            return false;
        };
        let Some(player_died_script) = input.token("player_died_script") else {
            return false;
        };
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
            input.record_offset = input.token_offset;
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
            input.record_offset = input.token_offset;
            let Some(quest) = read_extension_node(input, resolve_string) else {
                break;
            };
            self.quests.insert(quest.id, quest);
            loaded += 1;
        }
        loaded
    }

}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestSystemLoadCompletion {
    Loaded,
    Partial,
    QuestFileMissing,
    QuestExFileMissing,
    PrimaryFormatStopped,
}

/// Число вставок каждой фазы; дубликат ID считается отдельной обработанной
/// записью, хотя `std::map::operator[]` сохраняет только последнюю.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuestSystemLoadReport {
    pub primary_records: usize,
    pub extension_records: usize,
    pub completion: QuestSystemLoadCompletion,
    pub primary_error: Option<QuestTextError>,
    pub extension_error: Option<QuestTextError>,
}

/// Диагностика текстового разбора; смещения отсчитываются в байтах от начала файла.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuestTextError {
    pub field: &'static str,
    pub offset: usize,
    pub record_offset: usize,
    pub kind: QuestTextErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestTextErrorKind {
    UnexpectedEnd,
    InvalidI32,
    InvalidU32,
    InvalidU16,
    MissingStart,
    MissingEnd,
}

struct QuestInput<'source> {
    source: &'source [u8],
    cursor: usize,
    token_offset: usize,
    record_offset: usize,
    error: Option<QuestTextError>,
}

impl<'source> QuestInput<'source> {
    fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            cursor: 0,
            token_offset: 0,
            record_offset: 0,
            error: None,
        }
    }

    fn raw_token(&mut self) -> Option<&'source [u8]> {
        while self.cursor < self.source.len() && self.source[self.cursor].is_ascii_whitespace() {
            self.cursor += 1;
        }
        let start = self.cursor;
        self.token_offset = start;
        while self.cursor < self.source.len() && !self.source[self.cursor].is_ascii_whitespace() {
            self.cursor += 1;
        }
        (start != self.cursor).then_some(&self.source[start..self.cursor])
    }

    fn fail(&mut self, field: &'static str, offset: usize, kind: QuestTextErrorKind) {
        self.error.get_or_insert(QuestTextError {
            field,
            offset,
            record_offset: self.record_offset,
            kind,
        });
    }

    fn token(&mut self, field: &'static str) -> Option<&'source [u8]> {
        let token = self.raw_token();
        if token.is_none() {
            self.fail(field, self.token_offset, QuestTextErrorKind::UnexpectedEnd);
        }
        token
    }

    fn number<T: std::str::FromStr>(
        &mut self,
        field: &'static str,
        kind: QuestTextErrorKind,
    ) -> Option<T> {
        let token = self.token(field)?;
        let value = std::str::from_utf8(token)
            .ok()
            .and_then(|text| text.parse().ok());
        if value.is_none() {
            self.fail(field, self.token_offset, kind);
        }
        value
    }

    fn i32(&mut self, field: &'static str) -> Option<i32> {
        self.number(field, QuestTextErrorKind::InvalidI32)
    }

    fn u32(&mut self, field: &'static str) -> Option<u32> {
        self.number(field, QuestTextErrorKind::InvalidU32)
    }

    fn u16(&mut self, field: &'static str) -> Option<u16> {
        self.number(field, QuestTextErrorKind::InvalidU16)
    }

    fn read_to(&mut self, marker: &[u8]) -> bool {
        while let Some(token) = self.raw_token() {
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

    fn marked_text(&mut self, append_lines: bool, field: &'static str) -> Option<Vec<u8>> {
        if !self.read_to(b"<Start>") {
            self.fail(field, self.cursor, QuestTextErrorKind::MissingStart);
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
        self.fail(field, self.cursor, QuestTextErrorKind::MissingEnd);
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
    let _id_label = input.token("id label")?;
    quest.id = input.u16("id")?;
    let _name_label = input.token("name label")?;
    quest.name = resolve_or_empty(resolve_string, input.token("name")?);
    quest.description = resolve_or_empty(resolve_string, &input.marked_text(true, "description")?);
    let _abandon_label = input.token("abandon label")?;
    quest.abandon_script = normalize_script_path(input.token("abandon_script")?);
    let _complete_label = input.token("complete label")?;
    quest.complete_script = normalize_script_path(input.token("complete_script")?);
    let _coordinate_label = input.token("coordinate label")?;
    quest.region_id = input.i32("region_id")?;
    quest.tile_x = input.i32("tile_x")?;
    quest.tile_y = input.i32("tile_y")?;
    let _effect_label = input.token("effect label")?;
    quest.effect_id = input.i32("effect_id")?;
    let _display_label = input.token("display label")?;
    quest.display = input.i32("display")? != 0;
    Some(quest)
}

fn read_extension_node<ResolveString>(
    input: &mut QuestInput<'_>,
    resolve_string: &mut ResolveString,
) -> Option<QuestEntry>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let mut quest = QuestEntry {
        old: 1,
        ..QuestEntry::default()
    };
    let _id_label = input.token("id label")?;
    quest.id = input.u16("id")?;
    let _type_label = input.token("type label")?;
    quest.quest_type = input.u32("quest_type")?;
    let _level_label = input.token("level label")?;
    quest.level = input.u32("level")?;
    let _difficulty_label = input.token("difficulty label")?;
    quest.difficulty = input.u32("difficulty")?;
    let _tracking_label = input.token("tracking label")?;
    quest.track = input.u32("track")?;
    let _description_label = input.token("description label")?;
    quest.short_description =
        resolve_or_empty(resolve_string, &input.marked_text(true, "short_description")?);
    let _name_label = input.token("name label")?;
    quest.name = resolve_or_empty(resolve_string, input.token("name")?);
    quest.description = resolve_or_empty(resolve_string, &input.marked_text(false, "description")?);
    let _abandon_label = input.token("abandon label")?;
    quest.abandon_script = normalize_script_path(input.token("abandon_script")?);
    let _complete_label = input.token("complete label")?;
    quest.complete_script = normalize_script_path(input.token("complete_script")?);
    let _coordinate_label = input.token("coordinate label")?;
    quest.region_id = input.i32("region_id")?;
    quest.tile_x = input.i32("tile_x")?;
    quest.tile_y = input.i32("tile_y")?;
    let _effect_label = input.token("effect label")?;
    quest.effect_id = input.i32("effect_id")?;
    let _display_label = input.token("display label")?;
    quest.display = input.i32("display")? != 0;
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
