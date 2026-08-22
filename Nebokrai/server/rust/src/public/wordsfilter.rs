//! Владелец запрещённых слов WorldServer.
//!
//! World `CWordsFilter::CWordsFilter/Initial/LoadFilter/ReloadFilter/IsValid`,
//! двухаргументный `Check` и singleton lifetime — `IMPLEMENTED`;
//! трёхаргументный replace-owner, wire serializer и GameServer-вариант ниже
//! остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! исходный owner `e:\svn\fengyun_russia_dev\public\wordsfilter.cpp`.
//!
//! Запрещённые строки сохраняются byte-exact и проверяются как case-sensitive
//! подстроки в исходном list-order. Двухаргументный overload не заменяет слова:
//! он фиксирует первый match, всё равно вызывает `CharCodeFilter::check`, затем
//! возвращает conjunction обоих результатов. `Vec` и owned `CWordsFilter` у
//! `CGame` заменяют singleton/list/STL lifetime. Чтение ресурсов выполняет
//! контекст WorldServer; parser сохраняет Windows text-mode CRLF, `fgets(1024)`
//! и безусловное удаление последнего прочитанного byte каждой порции.

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

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || (needle.len() <= haystack.len()
            && haystack.windows(needle.len()).any(|window| window == needle))
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\wordsfilter.h

// ============================================================================
// FUNCTION: CWordsFilter::Check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:148
// RVA: 0x00029B20
// ADDRESS: 00429b20
// PROTOTYPE: bool __thiscall Check(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::~CWordsFilter
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.h:15
// RVA: 0x0002A000
// ADDRESS: 0042a000
// PROTOTYPE: void __thiscall ~CWordsFilter(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::CWordsFilter
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:18
// RVA: 0x0002A080
// ADDRESS: 0042a080
// PROTOTYPE: undefined __thiscall CWordsFilter(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:85
// RVA: 0x0002A100
// ADDRESS: 0042a100
// PROTOTYPE: CWordsFilter * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:93
// RVA: 0x0002A170
// ADDRESS: 0042a170
// PROTOTYPE: void __cdecl Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::FromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:210
// RVA: 0x0002A1A0
// ADDRESS: 0042a1a0
// PROTOTYPE: long __thiscall FromByteArray(uchar * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\wordsfilter.h

// ============================================================================
// FUNCTION: Catch@0044f902
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// RVA: 0x0004F902
// ADDRESS: 0044f902
// PROTOTYPE: undefined Catch@0044f902()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044f98a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// RVA: 0x0004F98A
// ADDRESS: 0044f98a
// PROTOTYPE: undefined Catch@0044f98a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagFileInfo::~tagFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// RVA: 0x000505B0
// ADDRESS: 004505b0
// PROTOTYPE: void __thiscall ~tagFileInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::IsValid
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:258
// RVA: 0x00050B10
// ADDRESS: 00450b10
// PROTOTYPE: bool __thiscall IsValid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::Check
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:98
// RVA: 0x00050BD0
// ADDRESS: 00450bd0
// PROTOTYPE: bool __thiscall Check(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::Check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:148
// RVA: 0x00050CF0
// ADDRESS: 00450cf0
// PROTOTYPE: bool __thiscall Check(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:187
// RVA: 0x000511B0
// ADDRESS: 004511b0
// PROTOTYPE: void __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::~CWordsFilter
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.h:15
// RVA: 0x00051260
// ADDRESS: 00451260
// PROTOTYPE: void __thiscall ~CWordsFilter(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::CWordsFilter
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:18
// RVA: 0x000512E0
// ADDRESS: 004512e0
// PROTOTYPE: undefined __thiscall CWordsFilter(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::LoadFilter
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:29
// RVA: 0x00051360
// ADDRESS: 00451360
// PROTOTYPE: bool __thiscall LoadFilter(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::ReloadFilter
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:50
// RVA: 0x00051540
// ADDRESS: 00451540
// PROTOTYPE: bool __thiscall ReloadFilter(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::GetInstance
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:85
// RVA: 0x000515B0
// ADDRESS: 004515b0
// PROTOTYPE: CWordsFilter * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::Initial
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:23
// RVA: 0x00051620
// ADDRESS: 00451620
// PROTOTYPE: bool __thiscall Initial(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWordsFilter::Release
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp:93
// RVA: 0x00051680
// ADDRESS: 00451680
// PROTOTYPE: void __cdecl Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052ebe0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// RVA: 0x0012EBE0
// ADDRESS: 0052ebe0
// PROTOTYPE: undefined Unwind@0052ebe0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052ec00
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\wordsfilter.cpp
// RVA: 0x0012EC00
// ADDRESS: 0052ec00
// PROTOTYPE: undefined Unwind@0052ec00()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//













// COMPONENT_VARIANT_END: WorldServer
