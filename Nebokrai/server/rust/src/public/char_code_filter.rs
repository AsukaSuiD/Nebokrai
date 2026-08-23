//! Фильтр допустимых однобайтовых кодов WorldServer.
//!
//! World-варианты `load` RVA `0x000D47A0`, `check` RVA `0x000D4590` и
//! `IsAllNumbers` RVA `0x000D4330` — `IMPLEMENTED`; GameServer-вариант ниже
//! остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CharRange {
    pub(crate) first: u8,
    pub(crate) last: u8,
}

#[derive(Default)]
pub(crate) struct CharCodeFilter {
    ranges: Vec<CharRange>,
}

impl CharCodeFilter {
    pub(crate) const fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    pub(crate) fn ranges(&self) -> &[CharRange] {
        &self.ranges
    }

    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
    }

    /// Дописывает один wire range для Game `CWordsFilter::FromByteArray`.
    pub(super) fn push_range(&mut self, first: u8, last: u8) {
        self.ranges.push(CharRange { first, last });
    }

    /// Дописывает пары точно в порядке formatted extraction исходного stream.
    pub(crate) fn load(&mut self, source: Option<&[u8]>) -> bool {
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

    pub(crate) fn check(
        &self,
        value: &mut Vec<u8>,
        _replace: bool,
        reject_all_numbers: bool,
    ) -> bool {
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp

// ============================================================================
// FUNCTION: CharCodeFilter::IsAllNumbers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:162
// RVA: 0x000C5410
// ADDRESS: 004c5410
// PROTOTYPE: bool __thiscall IsAllNumbers(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1, ulong param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CharCodeFilter::check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:95
// RVA: 0x000C5670
// ADDRESS: 004c5670
// PROTOTYPE: bool __thiscall check(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e4bd6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E4BD6
// ADDRESS: 004e4bd6
// PROTOTYPE: undefined Catch@004e4bd6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e4e31
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E4E31
// ADDRESS: 004e4e31
// PROTOTYPE: undefined Catch@004e4e31()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e50db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E50DB
// ADDRESS: 004e50db
// PROTOTYPE: undefined Catch@004e50db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5392
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5392
// ADDRESS: 004e5392
// PROTOTYPE: undefined Catch@004e5392()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5536
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5536
// ADDRESS: 004e5536
// PROTOTYPE: undefined Catch@004e5536()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5a6d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5A6D
// ADDRESS: 004e5a6d
// PROTOTYPE: undefined Catch@004e5a6d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5b0d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5B0D
// ADDRESS: 004e5b0d
// PROTOTYPE: undefined Catch@004e5b0d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5df4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5DF4
// ADDRESS: 004e5df4
// PROTOTYPE: undefined Catch@004e5df4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e5ea7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp
// RVA: 0x000E5EA7
// ADDRESS: 004e5ea7
// PROTOTYPE: undefined Catch@004e5ea7()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp

// ============================================================================
// FUNCTION: CharCodeFilter::IsAllNumbers
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:162
// RVA: 0x000D4330
// ADDRESS: 004d4330
// PROTOTYPE: bool __thiscall IsAllNumbers(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1, ulong param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CharCodeFilter::check
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:95
// RVA: 0x000D4590
// ADDRESS: 004d4590
// PROTOTYPE: bool __thiscall check(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CharCodeFilter::load
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\char_code_filter.cpp:11
// RVA: 0x000D47A0
// ADDRESS: 004d47a0
// PROTOTYPE: bool __thiscall load(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
