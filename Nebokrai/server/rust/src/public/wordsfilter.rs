//! Фильтр запрещённых слов WorldServer из точной пары EXE/PDB.
//!
//! Строки сохраняются byte-exact и проверяются case-sensitive в list-order.
//! Двухаргументный `Check` фиксирует первый match, всё равно вызывает
//! `CharCodeFilter::check` и возвращает conjunction результатов. Resource
//! parser сохраняет text-mode CRLF, chunk `fgets(1024)` и удаление последнего
//! byte каждой порции. `Vec` и owned owner заменяют singleton/STL lifetime
//! без изменения replace/serialization semantics.

use std::error::Error;
use std::fmt;

use super::char_code_filter::CharCodeFilter;

pub(crate) struct CWordsFilter {
    filter_file_name: Vec<u8>,
    char_code_file_name: Vec<u8>,
    filters: Vec<Vec<u8>>,
    char_code_filter: CharCodeFilter,
}

impl CWordsFilter {
    pub(crate) const fn new() -> Self {
        Self {
            filter_file_name: Vec::new(),
            char_code_file_name: Vec::new(),
            filters: Vec::new(),
            char_code_filter: CharCodeFilter::new(),
        }
    }

    pub(crate) fn initial(
        &mut self,
        filter_file_name: &[u8],
        char_code_file_name: &[u8],
        filter_source: Option<&[u8]>,
        char_code_source: Option<&[u8]>,
    ) -> bool {
        self.filter_file_name.clear();
        self.filter_file_name.extend_from_slice(filter_file_name);
        self.char_code_file_name.clear();
        self.char_code_file_name
            .extend_from_slice(char_code_file_name);
        self.load_filter(filter_source, char_code_source)
    }

    pub(crate) fn reload(
        &mut self,
        filter_source: Option<&[u8]>,
        char_code_source: Option<&[u8]>,
    ) -> bool {
        self.filters.clear();
        self.char_code_filter.clear();
        self.load_filter(filter_source, char_code_source)
    }

    pub(crate) fn clear(&mut self) {
        self.filters.clear();
        self.char_code_filter.clear();
        self.filter_file_name.clear();
        self.char_code_file_name.clear();
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.filters.is_empty() || !self.char_code_filter.ranges().is_empty()
    }

    pub(crate) fn check(&self, value: &mut Vec<u8>, replace: bool) -> bool {
        let words_valid = !self.filters.iter().any(|filter| contains(value, filter));
        let codes_valid = self.char_code_filter.check(value, replace, false);
        words_valid && codes_valid
    }

 /// Трёхаргументный overload с оригинал replace- и DBCS-поведением.
    pub(crate) fn check_with_numeric_gate(
        &self,
        value: &mut Vec<u8>,
        replace: bool,
        reject_all_numbers: bool,
    ) -> bool {
        if !self
            .char_code_filter
            .check(value, replace, reject_all_numbers)
        {
            return false;
        }
        for filter in &self.filters {
            let Some(mut position) = find_from_start(value, filter) else {
                continue;
            };
            loop {
                if filter.len() == 1
                    && position >= 1
                    && value[position - 1] & 0x80 != 0
                {
 // Оригинал owner считает совпадение вторым DBCS-byte и сразу
 // переходит к следующему filter, не ищет поздние вхождения.
                    break;
                }
                if !replace {
                    return false;
                }
                value.splice(
                    position..position + filter.len(),
                    std::iter::repeat_n(b'*', filter.len()),
                );
                let Some(next) = find_from_start(value, filter) else {
                    break;
                };
                position = next;
            }
        }
        true
    }

 /// Дописывает ranges и запрещённые C-строки в оригинал World wire-order.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), WordsFilterSerializeError> {
        let range_count = i32::try_from(self.char_code_filter.ranges().len())
            .map_err(|_| WordsFilterSerializeError {
                section: WordsFilterSerializeSection::CharacterRanges,
                count: self.char_code_filter.ranges().len(),
            })?;
        destination.extend_from_slice(&range_count.to_le_bytes());
        for range in self.char_code_filter.ranges() {
            destination.push(range.first);
            destination.push(range.last);
        }
        let filter_count =
            i32::try_from(self.filters.len()).map_err(|_| WordsFilterSerializeError {
                section: WordsFilterSerializeSection::BannedWords,
                count: self.filters.len(),
            })?;
        destination.extend_from_slice(&filter_count.to_le_bytes());
        for filter in &self.filters {
            destination.extend_from_slice(filter);
            destination.push(0);
        }
        Ok(())
    }

    pub(crate) fn filter_file_name(&self) -> &[u8] {
        &self.filter_file_name
    }

    pub(crate) fn char_code_file_name(&self) -> &[u8] {
        &self.char_code_file_name
    }

    fn load_filter(
        &mut self,
        filter_source: Option<&[u8]>,
        char_code_source: Option<&[u8]>,
    ) -> bool {
        let Some(filter_source) = filter_source else {
            return false;
        };
        append_fgets_lines(filter_source, &mut self.filters);
        self.char_code_filter.load(char_code_source)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WordsFilterSerializeSection {
    CharacterRanges,
    BannedWords,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WordsFilterSerializeError {
    pub(crate) section: WordsFilterSerializeSection,
    pub(crate) count: usize,
}

impl fmt::Display for WordsFilterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let section = match self.section {
            WordsFilterSerializeSection::CharacterRanges => "диапазонов символов",
            WordsFilterSerializeSection::BannedWords => "запрещённых строк",
        };
        write!(
            formatter,
            "WordsFilter содержит {} {} вне signed 32-битного диапазона",
            self.count, section
        )
    }
}

impl Error for WordsFilterSerializeError {}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || (needle.len() <= haystack.len()
            && haystack.windows(needle.len()).any(|window| window == needle))
}

fn find_from_start(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        Some(0)
    } else if needle.len() <= haystack.len() {
        haystack.windows(needle.len()).position(|window| window == needle)
    } else {
        None
    }
}

fn append_fgets_lines(source: &[u8], destination: &mut Vec<Vec<u8>>) {
    let mut translated = Vec::with_capacity(source.len());
    let mut cursor = 0;
    while cursor < source.len() {
        if source[cursor..].starts_with(b"\r\n") {
            translated.push(b'\n');
            cursor += 2;
        } else {
            translated.push(source[cursor]);
            cursor += 1;
        }
    }

    let mut cursor = 0;
    while cursor < translated.len() {
        let remaining = &translated[cursor..];
        let take = remaining
            .iter()
            .take(1023)
            .position(|byte| *byte == b'\n')
            .map_or(remaining.len().min(1023), |index| index + 1);
        let mut line = remaining[..take].to_vec();
        cursor += take;
        let _ = line.pop();
        if !line.is_empty() {
            destination.push(line);
        }
    }
}
