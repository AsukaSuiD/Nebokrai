//! Частично восстановленный владелец `public/filesinfo.cpp`.
//!
//! Точная пара WorldServer: `Nworldserver.exe` / `WorldServer.pdb`, исходные
//! пути PDB: `e:\svn\fengyun_russia_dev\public\filesinfo.cpp/.h`.
//! Материализована read-side проекция подтверждённых полей `tagFileInfo`,
//! grammar корректного `.ril` и lookup
//! `GetFileInfoByText`/`FindChildFileInfoByText` и `FindFileList`. Массивы,
//! деревья и владение C++ заменены `Vec`/`BTreeMap` без изменения порядка
//! package-записей и побайтовых ключей пути.
//!
//! Полный `CFilesInfo::Load` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально): exact RAW теряет его
//! возвращаемое значение в security-cookie epilogue, а архивный C++-донор
//! навязывает строгий error mapping. `FilesInfo::from_ril` выражает только
//! успешный форматный путь и не выдаёт свою ошибку за legacy return value.
//! Сырой C++ ниже остаётся доказательной заготовкой, а не Rust-реализацией.

use std::collections::BTreeMap;

/// Один элемент package-заголовка `.ril` в исходном порядке.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackFileInfo {
    pub(crate) id: u32,
    pub(crate) file_name: Vec<u8>,
    pub(crate) index_num: u32,
    pub(crate) empty_index_num: u32,
}

/// Безопасный владелец полей исторического `tagFileInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileInfo {
    name: Vec<u8>,
    size: u32,
    origin_size: u32,
    valid_size: u32,
    crc32: u32,
    package_type: u32,
    compress_type: u32,
    folder: bool,
    children: BTreeMap<Vec<u8>, FileInfo>,
}

impl FileInfo {
    fn root() -> Self {
        Self {
            name: Vec::new(),
            size: 0,
            origin_size: 0,
            valid_size: 0,
            crc32: 0,
            package_type: 1,
            compress_type: 1,
            folder: true,
            children: BTreeMap::new(),
        }
    }

    fn new(
        name: Vec<u8>,
        size: u32,
        origin_size: u32,
        valid_size: u32,
        crc32: u32,
        package_type: u32,
        compress_type: u32,
        folder: bool,
    ) -> Self {
        Self {
            name,
            size,
            origin_size,
            valid_size,
            crc32,
            package_type,
            compress_type,
            folder,
            children: BTreeMap::new(),
        }
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn origin_size(&self) -> u32 {
        self.origin_size
    }

    pub(crate) fn valid_size(&self) -> u32 {
        self.valid_size
    }

    pub(crate) fn crc32(&self) -> u32 {
        self.crc32
    }

    pub(crate) fn package_type(&self) -> u32 {
        self.package_type
    }

    pub(crate) fn compress_type(&self) -> u32 {
        self.compress_type
    }

    pub(crate) fn is_folder(&self) -> bool {
        self.folder
    }
}

/// Ошибка чтения формата, не являющаяся историческим return/error mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FilesInfoParseError {
    UnexpectedEnd,
    InvalidUnsigned,
    InvalidSigned,
}

/// Read-side владелец индекса файлов World resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FilesInfo {
    version: Vec<u8>,
    package_infos: Vec<PackFileInfo>,
    root: FileInfo,
    file_count: u32,
}

impl FilesInfo {
    /// Точное устойчивое состояние `CFilesInfo::CFilesInfo` без GUI callback-а.
    pub(crate) fn new() -> Self {
        Self {
            version: b"00.00.0000".to_vec(),
            package_infos: Vec::new(),
            root: FileInfo::root(),
            file_count: 0,
        }
    }

    /// Читает успешный текстовый `.ril` layout, использованный World `rfOpen`.
    pub(crate) fn from_ril(input: &[u8]) -> Result<Self, FilesInfoParseError> {
        let mut tokens = RilTokens::new(input);
        let version = tokens.next()?.to_vec();
        let package_count = parse_unsigned(tokens.next()?)?;
        let mut package_infos = Vec::new();
        for _ in 0..package_count {
            package_infos.push(PackFileInfo {
                id: parse_unsigned(tokens.next()?)?,
                // Exact `LoadPackInfo` копирует `%s` без `_strlwr`; это имя
                // позднее образует путь package-файла и не является lookup-key.
                file_name: tokens.next()?.to_vec(),
                index_num: parse_unsigned(tokens.next()?)?,
                empty_index_num: parse_unsigned(tokens.next()?)?,
            });
        }

        let mut result = Self::new();
        result.version = version;
        result.package_infos = package_infos;
        result.root.size = parse_unsigned(tokens.next()?)?;
        result.root.origin_size = parse_unsigned(tokens.next()?)?;
        result.root.valid_size = parse_unsigned(tokens.next()?)?;
        result.root.crc32 = parse_unsigned(tokens.next()?)?;
        result.root.package_type = signed_bits(tokens.next()?)?;
        result.root.compress_type = signed_bits(tokens.next()?)?;
        let _ignored_root_type = signed_bits(tokens.next()?)?;

        read_folder(&mut tokens, &mut result.root, &mut result.file_count)?;
        Ok(result)
    }

    pub(crate) fn version(&self) -> &[u8] {
        &self.version
    }

    pub(crate) fn package_infos(&self) -> &[PackFileInfo] {
        &self.package_infos
    }

    pub(crate) fn root(&self) -> &FileInfo {
        &self.root
    }

    pub(crate) fn file_count(&self) -> u32 {
        self.file_count
    }

    /// Повторяет `GetFileInfoByText` для нормального пути `rfOpen`.
    ///
    /// Как точный общий RAW, пустой путь и пустой segment возвращают root;
    /// путь без начального `\\` также не проходит ни одного сегмента и оставляет
    /// root. `CheckRFileStr` нормализует реальный путь до этого вызова.
    pub(crate) fn file_info_by_text(&self, text: &[u8]) -> Option<&FileInfo> {
        let mut current = &self.root;
        let mut remaining = text;
        while remaining.first() == Some(&b'\\') {
            remaining = &remaining[1..];
            let segment_end = remaining
                .iter()
                .position(|byte| *byte == b'\\')
                .unwrap_or(remaining.len());
            let segment = &remaining[..segment_end];
            remaining = &remaining[segment_end..];
            current = find_child(&self.root, current, segment)?;
        }
        Some(current)
    }

    /// Точный `CFilesInfo::FindFileList(root, extension, output)`.
    ///
    /// Public overload lowercases только `root`; extension передаётся в
    /// leaf-comparison без нормализации. Поэтому `".script"` на lowercased
    /// `.ril` именах находит файлы, а иной регистр extension не получает
    /// неявный compatibility fallback. `BTreeMap` сохраняет `std::map`
    /// key-order рекурсивного обхода, а каждый результат начинается с `\\`.
    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        let root = lowercase_ascii(root);
        let Some(root) = self.file_info_by_text(&root) else {
            return Vec::new();
        };
        let mut files = Vec::new();
        let mut path = Vec::new();
        find_file_list(root, extension, &mut path, &mut files);
        files
    }
}

fn find_file_list(
    file: &FileInfo,
    extension: &[u8],
    parent_path: &mut Vec<u8>,
    files: &mut Vec<Vec<u8>>,
) {
    let path_length = parent_path.len();
    if !file.name.is_empty() {
        parent_path.push(b'\\');
        parent_path.extend_from_slice(&file.name);
    }

    if file.children.is_empty() {
        if !file.folder && file_extension(&file.name) == extension {
            files.push(parent_path.clone());
        }
    } else {
        for child in file.children.values() {
            find_file_list(child, extension, parent_path, files);
        }
    }
    parent_path.truncate(path_length);
}

/// Соответствует `find_last_of(".")` плюс `erase(0, position)`.
fn file_extension(name: &[u8]) -> &[u8] {
    name.iter()
        .rposition(|byte| *byte == b'.')
        .map_or(&[], |position| &name[position..])
}

fn read_folder(
    tokens: &mut RilTokens<'_>,
    parent: &mut FileInfo,
    file_count: &mut u32,
) -> Result<(), FilesInfoParseError> {
    loop {
        let name = lowercase_ascii(tokens.next()?);
        let size = parse_unsigned(tokens.next()?)?;
        let origin_size = parse_unsigned(tokens.next()?)?;
        let valid_size = parse_unsigned(tokens.next()?)?;
        let crc32 = parse_unsigned(tokens.next()?)?;
        let package_type = signed_bits(tokens.next()?)?;
        let compress_type = signed_bits(tokens.next()?)?;
        let record_type = signed_bits(tokens.next()?)? as i32;

        if record_type == 0x14 {
            return Ok(());
        }

        let folder = record_type != 0;
        let mut child = FileInfo::new(
            name.clone(),
            size,
            origin_size,
            valid_size,
            if folder { 0 } else { crc32 },
            package_type,
            compress_type,
            folder,
        );
        if folder {
            read_folder(tokens, &mut child, file_count)?;
        } else {
            *file_count = file_count.wrapping_add(1);
        }
        parent.children.insert(name, child);
    }
}

fn find_child<'a>(root: &'a FileInfo, parent: &'a FileInfo, text: &[u8]) -> Option<&'a FileInfo> {
    if text.is_empty() {
        return Some(root);
    }
    parent.children.get(&lowercase_ascii(text))
}

fn lowercase_ascii(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn parse_unsigned(token: &[u8]) -> Result<u32, FilesInfoParseError> {
    let token = std::str::from_utf8(token).map_err(|_| FilesInfoParseError::InvalidUnsigned)?;
    token
        .parse()
        .map_err(|_| FilesInfoParseError::InvalidUnsigned)
}

fn signed_bits(token: &[u8]) -> Result<u32, FilesInfoParseError> {
    let token = std::str::from_utf8(token).map_err(|_| FilesInfoParseError::InvalidSigned)?;
    token
        .parse::<i32>()
        .map(|value| value as u32)
        .map_err(|_| FilesInfoParseError::InvalidSigned)
}

struct RilTokens<'a> {
    remaining: &'a [u8],
}

impl<'a> RilTokens<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { remaining: input }
    }

    fn next(&mut self) -> Result<&'a [u8], FilesInfoParseError> {
        let start = self
            .remaining
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())
            .ok_or(FilesInfoParseError::UnexpectedEnd)?;
        self.remaining = &self.remaining[start..];
        let end = self
            .remaining
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(self.remaining.len());
        let token = &self.remaining[..end];
        self.remaining = &self.remaining[end..];
        Ok(token)
    }
}

// COMPONENT_VARIANT_BEGIN: ServerUpdate
// Точная пара: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SHA-256 EXE: 21CDB7E22DE1DD9F4530C29ACEEA0E337E251BD082E0DF6674B4FD90050B2252
// SHA-256 PDB: FF4C7D2515C83D99253A9F518D7CAD7FE18BB060CA9BF934C8053024424F2E77
// Исходный владелец PDB: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp
// Исходный владелец PDB: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.h

// ============================================================================
// FUNCTION: CFilesInfo::SetVersion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.h:102
// RVA: 0x000027E0
// ADDRESS: 004027e0
// PROTOTYPE: void __thiscall SetVersion(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::HaveChild
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:929
// RVA: 0x00003F80
// ADDRESS: 00403f80
// PROTOTYPE: bool __thiscall HaveChild(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFileSize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:1014
// RVA: 0x00003FA0
// ADDRESS: 00403fa0
// PROTOTYPE: void __thiscall AddFileSize(tagFileInfo * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::SavePackInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:187
// RVA: 0x00004110
// ADDRESS: 00404110
// PROTOTYPE: int __thiscall SavePackInfo(_iobuf * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Save
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:204
// RVA: 0x000045D0
// ADDRESS: 004045d0
// PROTOTYPE: bool __thiscall Save(_iobuf * param_1, tagFileInfo * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::UpdateParentFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:1028
// RVA: 0x00004740
// ADDRESS: 00404740
// PROTOTYPE: void __thiscall UpdateParentFolderInfo(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Save
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:156
// RVA: 0x00004A30
// ADDRESS: 00404a30
// PROTOTYPE: int __thiscall Save(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::FindChildFileInfoByText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:949
// RVA: 0x00004BF0
// ADDRESS: 00404bf0
// PROTOTYPE: tagFileInfo * __thiscall FindChildFileInfoByText(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileTextWithParent
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:1000
// RVA: 0x00004CA0
// ADDRESS: 00404ca0
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __thiscall GetFileTextWithParent(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagFileInfo::tagFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.h:48
// RVA: 0x00005200
// ADDRESS: 00405200
// PROTOTYPE: void __thiscall tagFileInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagFileInfo::tagFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.h:55
// RVA: 0x000052C0
// ADDRESS: 004052c0
// PROTOTYPE: void __thiscall tagFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7, bool param_8, tagFileInfo * param_9)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::CFilesInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:23
// RVA: 0x000053A0
// ADDRESS: 004053a0
// PROTOTYPE: void __thiscall CFilesInfo(HWND__ * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::LoadPackInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:97
// RVA: 0x00005490
// ADDRESS: 00405490
// PROTOTYPE: int __thiscall LoadPackInfo(_iobuf * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileInfoByText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:968
// RVA: 0x00005630
// ADDRESS: 00405630
// PROTOTYPE: tagFileInfo * __thiscall GetFileInfoByText(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::DelFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:656
// RVA: 0x00005A80
// ADDRESS: 00405a80
// PROTOTYPE: void __thiscall DelFileInfo(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:737
// RVA: 0x00005B30
// ADDRESS: 00405b30
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:779
// RVA: 0x00005BD0
// ADDRESS: 00405bd0
// PROTOTYPE: int __thiscall GetFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong * param_2, ulong * param_3, ulong * param_4, ulong * param_5, ulong * param_6, ulong * param_7, bool * param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::~CFilesInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:36
// RVA: 0x00005DF0
// ADDRESS: 00405df0
// PROTOTYPE: void __thiscall ~CFilesInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:252
// RVA: 0x00005EB0
// ADDRESS: 00405eb0
// PROTOTYPE: tagFileInfo * __thiscall AddFolderInfo(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:272
// RVA: 0x00005FB0
// ADDRESS: 00405fb0
// PROTOTYPE: tagFileInfo * __thiscall AddFileInfo(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7, ulong param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:292
// RVA: 0x000060C0
// ADDRESS: 004060c0
// PROTOTYPE: tagFileInfo * __thiscall AddFolderInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::UpdateFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:330
// RVA: 0x00006130
// ADDRESS: 00406130
// PROTOTYPE: tagFileInfo * __thiscall UpdateFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::UpdateFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:436
// RVA: 0x000066C0
// ADDRESS: 004066c0
// PROTOTYPE: tagFileInfo * __thiscall UpdateFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5, bool param_6, ulong param_7, ulong param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::DelFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:639
// RVA: 0x00006A40
// ADDRESS: 00406a40
// PROTOTYPE: void __thiscall DelFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetDifferenceByPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:811
// RVA: 0x00006B50
// ADDRESS: 00406b50
// PROTOTYPE: bool __thiscall GetDifferenceByPackage(tagFileInfo * param_1, CFilesInfo * param_2, CFilesInfo * param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetDifferenceByCrc32
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:892
// RVA: 0x00006D90
// ADDRESS: 00406d90
// PROTOTYPE: bool __thiscall GetDifferenceByCrc32(CFilesInfo * param_1, CFilesInfo * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::LoadFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:118
// RVA: 0x00006DD0
// ADDRESS: 00406dd0
// PROTOTYPE: int __thiscall LoadFolderInfo(_iobuf * param_1, tagFileInfo * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:46
// RVA: 0x00007000
// ADDRESS: 00407000
// PROTOTYPE: int __thiscall Load(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004071c9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\filesinfo.cpp:89
// RVA: 0x000071C9
// ADDRESS: 004071c9
// PROTOTYPE: undefined4 __stdcall Catch@004071c9(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: ServerUpdate

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\filesinfo.cpp

// ============================================================================
// FUNCTION: CFilesInfo::FindChildFileInfoByText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:949
// RVA: 0x001D2050
// ADDRESS: 005d2050
// PROTOTYPE: tagFileInfo * __thiscall FindChildFileInfoByText(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileInfoByText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:968
// RVA: 0x001D21D0
// ADDRESS: 005d21d0
// PROTOTYPE: tagFileInfo * __thiscall GetFileInfoByText(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\filesinfo.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\filesinfo.h

// ============================================================================
// FUNCTION: Catch@004d13b5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp
// RVA: 0x000D13B5
// ADDRESS: 004d13b5
// PROTOTYPE: undefined Catch@004d13b5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFileSize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:1014
// RVA: 0x000D2080
// ADDRESS: 004d2080
// PROTOTYPE: void __thiscall AddFileSize(tagFileInfo * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::UpdateParentFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:1028
// RVA: 0x000D20E0
// ADDRESS: 004d20e0
// PROTOTYPE: void __thiscall UpdateParentFolderInfo(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::FindChildFileInfoByText
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:949
// RVA: 0x000D2230
// ADDRESS: 004d2230
// PROTOTYPE: tagFileInfo * __thiscall FindChildFileInfoByText(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileTextWithParent
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:1000
// RVA: 0x000D23B0
// ADDRESS: 004d23b0
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __thiscall GetFileTextWithParent(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::FindFileList
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:1122
// RVA: 0x000D27A0
// ADDRESS: 004d27a0
// PROTOTYPE: void __thiscall FindFileList(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::LoadPackInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:97
// RVA: 0x000D2A50
// ADDRESS: 004d2a50
// PROTOTYPE: int __thiscall LoadPackInfo(_iobuf * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::GetFileInfoByText
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:968
// RVA: 0x000D2BF0
// ADDRESS: 004d2bf0
// PROTOTYPE: tagFileInfo * __thiscall GetFileInfoByText(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::FindFileList
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:1102
// RVA: 0x000D2E90
// ADDRESS: 004d2e90
// PROTOTYPE: void __thiscall FindFileList(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagFileInfo::tagFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.h:48
// RVA: 0x000D3250
// ADDRESS: 004d3250
// PROTOTYPE: undefined __thiscall tagFileInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagFileInfo::tagFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.h:55
// RVA: 0x000D3310
// ADDRESS: 004d3310
// PROTOTYPE: undefined __thiscall tagFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7, bool param_8, tagFileInfo * param_9)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::CFilesInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:23
// RVA: 0x000D33F0
// ADDRESS: 004d33f0
// PROTOTYPE: undefined __thiscall CFilesInfo(HWND__ * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFolderInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:252
// RVA: 0x000D34E0
// ADDRESS: 004d34e0
// PROTOTYPE: tagFileInfo * __thiscall AddFolderInfo(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::AddFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:272
// RVA: 0x000D35F0
// ADDRESS: 004d35f0
// PROTOTYPE: tagFileInfo * __thiscall AddFileInfo(tagFileInfo * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7, ulong param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::UpdateFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:330
// RVA: 0x000D3700
// ADDRESS: 004d3700
// PROTOTYPE: tagFileInfo * __thiscall UpdateFileInfo(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5, ulong param_6, ulong param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::LoadFolderInfo
// STATUS: IMPLEMENTED / ERROR_MAPPING_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:118
// RVA: 0x000D3C90
// ADDRESS: 004d3c90
// PROTOTYPE: int __thiscall LoadFolderInfo(_iobuf * param_1, tagFileInfo * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::DelFileInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:656
// RVA: 0x000D3EC0
// ADDRESS: 004d3ec0
// PROTOTYPE: void __thiscall DelFileInfo(tagFileInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:737
// RVA: 0x000D3FA0
// ADDRESS: 004d3fa0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::~CFilesInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:36
// RVA: 0x000D4060
// ADDRESS: 004d4060
// PROTOTYPE: void __thiscall ~CFilesInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFilesInfo::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:46
// RVA: 0x000D4120
// ADDRESS: 004d4120
// PROTOTYPE: int __thiscall Load(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d42e9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp:89
// RVA: 0x000D42E9
// ADDRESS: 004d42e9
// PROTOTYPE: undefined Catch@004d42e9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00534eb0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\filesinfo.cpp
// RVA: 0x00134EB0
// ADDRESS: 00534eb0
// PROTOTYPE: undefined Unwind@00534eb0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
