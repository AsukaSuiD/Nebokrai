//! Таблица текстов по public/stringtable.cpp/.h: World StringTable::load,
//! RVA 0x0004E5E0. Поздний отказ сохраняет применённые пары; очистка явная.
//! Происхождение и формат: docs/architecture/resources-and-configuration.md.

use std::collections::BTreeMap;

/// Таблица байтовых строк; жизненным циклом экземпляра управляет вызывающая роль.
#[derive(Default)]
pub struct StringTable {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    last_error: Vec<u8>,
}

impl StringTable {
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            last_error: Vec::new(),
        }
    }

    /// Возвращает значение по байтовому ключу без преобразования кодировки.
    pub fn get_string_by_id(&self, id: &[u8]) -> Option<&[u8]> {
        self.entries.get(id).map(Vec::as_slice)
    }

    /// Очищает записи; сохранённое описание последнего отказа остаётся прежним.
    pub fn free(&mut self) {
        self.entries.clear();
    }

    pub fn last_error(&self) -> &[u8] {
        &self.last_error
    }

    pub fn entries(&self) -> &BTreeMap<Vec<u8>, Vec<u8>> {
        &self.entries
    }

    /// Применяет декодированную пару; повторный ID заменяет значение.
    pub fn insert_owned(&mut self, id: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.entries.insert(id, value)
    }

    /// Дополняет таблицу из ресурса; поздний отказ сохраняет уже применённые записи.
    pub fn load_bytes(&mut self, source: &[u8]) -> Result<(), StringTableParseError> {
        let mut offset = 0usize;

        loop {
            while offset < source.len() && is_legacy_space(source[offset]) {
                offset += 1;
            }
            if offset >= source.len() {
                return Ok(());
            }

            if source[offset] == b';' {
                while offset < source.len() && source[offset] != b'\n' {
                    offset += 1;
                }
                offset = offset.saturating_add(1);
                continue;
            }

            let record_offset = offset;
            let mut id = Vec::new();
            while offset < source.len() {
                let byte = source[offset];
                if byte == b'"' || !byte.is_ascii_alphanumeric() {
                    break;
                }
                id.push(byte);
                offset += 1;
            }

            while offset < source.len() && source[offset] != b'"' {
                offset += 1;
            }
            offset = offset.saturating_add(1);
            if offset >= source.len() {
                self.set_missing_value_error(&id);
                return Err(StringTableParseError {
                    kind: StringTableParseErrorKind::MissingValue,
                    offset: source.len(),
                    record_offset,
                    id,
                });
            }

            let value_start = offset;
            while offset < source.len() && source[offset] != b'"' {
                offset += 1;
            }
            if offset >= source.len() {
                self.set_missing_closing_quote_error(&id);
                return Err(StringTableParseError {
                    kind: StringTableParseErrorKind::MissingClosingQuote,
                    offset: source.len(),
                    record_offset,
                    id,
                });
            }

            if !id.is_empty() {
                self.entries.insert(id, source[value_start..offset].to_vec());
            }
            offset += 1;
        }
    }

    /// Сохраняет прежнее сообщение загрузчика о пустом имени ресурса.
    pub fn reject_empty_resource_name(&mut self) {
        self.last_error = b"String::load : invalid file name : [] !".to_vec();
    }

    /// Сохраняет прежнее сообщение загрузчика о недоступном ресурсе.
    pub fn reject_missing_resource(&mut self, name: &[u8]) {
        let mut error = b"Can not open this file : [".to_vec();
        error.extend_from_slice(name);
        error.extend_from_slice(b"] !");
        self.last_error = error;
    }

    fn set_missing_value_error(&mut self, id: &[u8]) {
        let mut error = b"Syntax error : no string match to the id : ".to_vec();
        error.extend_from_slice(id);
        self.last_error = error;
    }

    fn set_missing_closing_quote_error(&mut self, id: &[u8]) {
        let mut error = b"Syntax error : no '\"' match '\"' of id : ".to_vec();
        error.extend_from_slice(id);
        error.push(b'.');
        self.last_error = error;
    }
}

/// Отказ текстового разбора: смещение в байтах от начала входа, начиная с нуля.
/// Уже применённые пары остаются в таблице; ошибочная пара не добавляется.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringTableParseError {
    pub kind: StringTableParseErrorKind,
    pub offset: usize,
    pub record_offset: usize,
    pub id: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringTableParseErrorKind {
    MissingValue,
    MissingClosingQuote,
}

fn is_legacy_space(byte: u8) -> bool {
    byte.is_ascii_whitespace()
}
