//! Общая byte-exact таблица строк Miracle.
//!
//! WorldServer `StringTable::StringTable/load/free/getStringByID` RVA
//! `0x0004E570/0x0004E5E0/0x0004E020/0x0004D8F0` — `IMPLEMENTED`;
//! файловая перегрузка `load` RVA `0x0004EA30` разделена на универсальный
//! resource-reader конкретного процесса и здешний parser. Остальные варианты
//! и метаданные ниже сохраняют происхождение исследования; полный декомпилят хранится локально.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner
//! `e:\svn\fengyun_russia_dev\public\stringtable.cpp:27,40,157,195,200`.
//! `BTreeMap<Vec<u8>, Vec<u8>>` заменяет только MSVC `std::map/std::string` и
//! сохраняет unsigned byte-лексикографический порядок ASCII ID. `load` не
//! очищает прежнюю таблицу: успешно прочитанный prefix остаётся даже после
//! последующей syntax-error, а повторный ID заменяет value через `operator[]`.
//! Комментарий начинается с `;` после whitespace и пропускается до `\n`;
//! ID содержит только ASCII alphanumeric, value — любые байты между прямыми
//! кавычками без escape-обработки.
//!
//! Exact первая error-ветвь форматировала pointer на ID через `%d`, выдавая
//! нестабильный адрес вместо текста. Это внутренний диагностический баг без
//! совместимого результата; Rust сохраняет момент отказа и prefix-мутации, но
//! пишет сам ID. Last-error не очищается при следующем успехе, как исходный
//! `mErrorDesc`. Rust ownership/Drop заменяют destructor и allocator plumbing.

use std::collections::BTreeMap;

/// Достигнутый World-owner `StringTable` без зависимости от resource backend.
#[derive(Default)]
pub(crate) struct StringTable {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    last_error: Vec<u8>,
}

impl StringTable {
    pub(crate) const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            last_error: Vec::new(),
        }
    }

    /// Возвращает value exact key либо nullable-result исходного map lookup.
    pub(crate) fn get_string_by_id(&self, id: &[u8]) -> Option<&[u8]> {
        self.entries.get(id).map(Vec::as_slice)
    }

    /// Очищает только map; прежний `mErrorDesc` намеренно сохраняется.
    pub(crate) fn free(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn last_error(&self) -> &[u8] {
        &self.last_error
    }

    pub(crate) fn entries(&self) -> &BTreeMap<Vec<u8>, Vec<u8>> {
        &self.entries
    }

    /// Публикует одну уже декодированную wire-пару; duplicate ID — last-wins.
    pub(crate) fn insert_owned(&mut self, id: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.entries.insert(id, value)
    }

    /// Разбирает один resource-buffer с точными prefix/overwrite эффектами.
    pub(crate) fn load_bytes(&mut self, source: &[u8]) -> bool {
        let mut offset = 0usize;

        loop {
            while offset < source.len() && is_legacy_space(source[offset]) {
                offset += 1;
            }
            if offset >= source.len() {
                return true;
            }

            if source[offset] == b';' {
                while offset < source.len() && source[offset] != b'\n' {
                    offset += 1;
                }
                offset = offset.saturating_add(1);
                continue;
            }

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
                return false;
            }

            let value_start = offset;
            while offset < source.len() && source[offset] != b'"' {
                offset += 1;
            }
            if offset >= source.len() {
                self.set_missing_closing_quote_error(&id);
                return false;
            }

            if !id.is_empty() {
                self.entries.insert(id, source[value_start..offset].to_vec());
            }
            offset += 1;
        }
    }

    /// Материализует file/resource-overload failure для пустого имени.
    pub(crate) fn reject_empty_resource_name(&mut self) {
        self.last_error = b"String::load : invalid file name : [] !".to_vec();
    }

    /// Материализует file/resource-overload failure открытия.
    pub(crate) fn reject_missing_resource(&mut self, name: &[u8]) {
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

fn is_legacy_space(byte: u8) -> bool {
    byte.is_ascii_whitespace()
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\stringtable.cpp

// ============================================================================
// FUNCTION: StringTable::getStringByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:200
// RVA: 0x00022BE0
// ADDRESS: 00422be0
// PROTOTYPE: char * __thiscall getStringByID(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::free
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:195
// RVA: 0x00023290
// ADDRESS: 00423290
// PROTOTYPE: void __thiscall free(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::~StringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:31
// RVA: 0x00023610
// ADDRESS: 00423610
// PROTOTYPE: void __thiscall ~StringTable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::StringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:27
// RVA: 0x000237E0
// ADDRESS: 004237e0
// PROTOTYPE: undefined __thiscall StringTable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e2d71
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E2D71
// ADDRESS: 004e2d71
// PROTOTYPE: undefined Catch@004e2d71()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E2E10
// ADDRESS: 004e2e10
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::~stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E2EA0
// ADDRESS: 004e2ea0
// PROTOTYPE: void __thiscall ~stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E2F50
// ADDRESS: 004e2f50
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(stTaoZhuangAddItem * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e3031
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3031
// ADDRESS: 004e3031
// PROTOTYPE: undefined Catch@004e3031()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e30e6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E30E6
// ADDRESS: 004e30e6
// PROTOTYPE: undefined Catch@004e30e6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e3731
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3731
// ADDRESS: 004e3731
// PROTOTYPE: undefined Catch@004e3731()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e3931
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3931
// ADDRESS: 004e3931
// PROTOTYPE: undefined Catch@004e3931()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3950
// ADDRESS: 004e3950
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::~stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3A00
// ADDRESS: 004e3a00
// PROTOTYPE: void __thiscall ~stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3B10
// ADDRESS: 004e3b10
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(stTaoZhuangItemNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e3c7f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp
// RVA: 0x000E3C7F
// ADDRESS: 004e3c7f
// PROTOTYPE: undefined Catch@004e3c7f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




















// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\stringtable.cpp

// ============================================================================
// FUNCTION: StringTable::getStringByID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:200
// RVA: 0x0004D8F0
// ADDRESS: 0044d8f0
// PROTOTYPE: char * __thiscall getStringByID(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// IMPLEMENTED_OWNER: `StringTable::get_string_by_id`; `BTreeMap::get` заменяет
// только MSVC tree traversal и сохраняет nullable miss.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::_setLastErrorDesc
// STATUS: IMPLEMENTED / CORRECTED_INTERNAL_DEFECT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:231
// RVA: 0x0004D930
// ADDRESS: 0044d930
// PROTOTYPE: void __thiscall _setLastErrorDesc(char * param_1, ...)
//
// IMPLEMENTED_OWNER: три typed error-builder-а `StringTable`; нестабильный
// pointer-as-`%d` первой parser-ветви исправлен, остальные значимые bytes
// сохранены без воспроизведения stack buffer/varargs plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::free
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:195
// RVA: 0x0004E020
// ADDRESS: 0044e020
// PROTOTYPE: void __thiscall free(void)
//
// IMPLEMENTED_OWNER: `StringTable::free`; очищается map, но не `last_error`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::~StringTable
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:31
// RVA: 0x0004E3A0
// ADDRESS: 0044e3a0
// PROTOTYPE: void __thiscall ~StringTable(void)
//
// IMPLEMENTED_OWNER: обычный Rust `Drop` полей; ручной allocator/tree/string
// cleanup не является Miracle-семантикой.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::StringTable
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:27
// RVA: 0x0004E570
// ADDRESS: 0044e570
// PROTOTYPE: undefined __thiscall StringTable(void)
//
// IMPLEMENTED_OWNER: `StringTable::new/Default` создают пустые map/error.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::load
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:40
// RVA: 0x0004E5E0
// ADDRESS: 0044e5e0
// PROTOTYPE: bool __thiscall load(char * param_1, uint param_2)
//
// IMPLEMENTED_OWNER: `StringTable::load_bytes`; parser сохраняет точный
// `0x0044E5E0..0x0044EA25` order, partial map mutation и duplicate overwrite.
// ASCII classification заменяет process-locale CRT только для ID/whitespace;
// quoted value остаётся byte-exact.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: StringTable::load
// STATUS: IMPLEMENTED / INFRASTRUCTURE_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\stringtable.cpp:157
// RVA: 0x0004EA30
// ADDRESS: 0044ea30
// PROTOTYPE: bool __thiscall load(char * param_1)
//
// IMPLEMENTED_OWNER: пустое/missing имя выражают `reject_*`, а чтение bytes
// остаётся callback-ом конкретного resource backend; parser — `load_bytes`.
// `Vec` и Rust owner устраняют raw allocation и исходную утечку `CRFile`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
