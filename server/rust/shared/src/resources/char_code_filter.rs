//! Фильтр допустимых однобайтовых кодов WorldServer.
//!
//! World-варианты `load` RVA `0x000D47A0`, `check` RVA `0x000D4590` и
//! `IsAllNumbers` RVA `0x000D4330` — `IMPLEMENTED`; GameServer-вариант ниже
//! остаётся `UNKNOWN`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходный owner
//! `e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:11,95,162`.
//!
//! `load` читает whitespace-разделённые signed decimal пары, приводит каждую
//! границу к младшему byte и дописывает диапазоны без предварительной очистки.
//! Rust получает уже прочитанный resource от caller-а: это заменяет только
//! `rfOpen/CRFile/stringstream`, не формат. `check` отклоняет пустую строку,
//! байты вне диапазонов (кроме `ё/Ё/_` в CP1251), полностью цифровое имя при
//! включённом gate и смесь ASCII Latin с CP1251 Cyrillic. Исходный второй bool
//! не читается. Одно завершающее пространство сначала удаляется, но проход всё
//! равно использует прежнюю длину и видит NUL, поэтому итог остаётся false;
//! эта странная мутация сохранена безопасно.
//! Установленный экземпляр и его потребители остаются у владельца роли.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharRange {
    pub first: u8,
    pub last: u8,
}

#[derive(Default)]
pub struct CharCodeFilter {
    ranges: Vec<CharRange>,
}

impl CharCodeFilter {
    pub const fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    pub fn ranges(&self) -> &[CharRange] {
        &self.ranges
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    /// Дописывает один wire range для Game `CWordsFilter::FromByteArray`.
    pub(super) fn push_range(&mut self, first: u8, last: u8) {
        self.ranges.push(CharRange { first, last });
    }

    /// Дописывает пары точно в порядке formatted extraction исходного stream.
    pub fn load(&mut self, source: Option<&[u8]>) -> bool {
        let Some(source) = source else {
            return false;
        };
        let mut values = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty())
            .map(|token| {
                std::str::from_utf8(token)
                    .ok()
                    .and_then(|value| value.parse::<i32>().ok())
            });
        loop {
            let Some(Some(first)) = values.next() else {
                break;
            };
            let Some(Some(last)) = values.next() else {
                break;
            };
            self.ranges.push(CharRange {
                first: first as u8,
                last: last as u8,
            });
        }
        true
    }

    pub fn check(&self, value: &mut Vec<u8>, _replace: bool, reject_all_numbers: bool) -> bool {
        let original_len = value.len();
        if original_len == 0 {
            return false;
        }

        match value.iter().position(|byte| *byte == b' ') {
            Some(0) => {
                // Exact owner присваивает `substr(find(" "), old_len)`, то
                // есть фактически оставляет leading-space строку как есть.
            }
            _ if value.last() == Some(&b' ') => {
                value.truncate(original_len - 1);
            }
            _ if Self::is_all_numbers(value, original_len, reject_all_numbers) => {
                return false;
            }
            _ => {}
        }

        let mut has_latin = false;
        let mut has_cyrillic = false;
        for index in 0..original_len {
            let byte = value.get(index).copied().unwrap_or(0);
            let in_range = self
                .ranges
                .iter()
                .any(|range| range.first <= byte && byte <= range.last);
            if !in_range && !matches!(byte, 0xB8 | 0xA8 | b'_') {
                return false;
            }
            if byte.is_ascii_alphabetic() {
                has_latin = true;
            }
            if 0xC0 <= byte {
                has_cyrillic = true;
            }
        }
        !(has_latin && has_cyrillic)
    }

    fn is_all_numbers(value: &[u8], length: usize, enabled: bool) -> bool {
        enabled
            && length != 0
            && (0..length).all(|index| {
                value
                    .get(index)
                    .copied()
                    .is_some_and(|byte| byte.is_ascii_digit())
            })
    }
}
