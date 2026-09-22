//! Индекс .ril по public/filesinfo.cpp/.h: World LoadFolderInfo RVA 0x000D3C90.
//! Формат и привязки — docs/architecture/resources-and-configuration.md,
//! раздел «Индекс FilesInfo». Исходный return/error mapping Load остаётся
//! UNKNOWN; ошибки со смещением и совместное владение — интерфейсы Rust.

use std::{collections::BTreeMap, sync::Arc};

/// Один элемент package-заголовка `.ril` в исходном порядке.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackFileInfo {
    pub id: u32,
    pub file_name: Vec<u8>,
    pub index_num: u32,
    pub empty_index_num: u32,
}

/// Безопасный владелец полей исторического `tagFileInfo`.
#[derive(Debug, Eq, PartialEq)]
pub struct FileInfo {
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

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn origin_size(&self) -> u32 {
        self.origin_size
    }

    pub fn valid_size(&self) -> u32 {
        self.valid_size
    }

    pub fn crc32(&self) -> u32 {
        self.crc32
    }

    pub fn package_type(&self) -> u32 {
        self.package_type
    }

    pub fn compress_type(&self) -> u32 {
        self.compress_type
    }

    pub fn is_folder(&self) -> bool {
        self.folder
    }
}

/// Ошибка чтения формата, не являющаяся историческим return/error mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesInfoParseError {
    UnexpectedEnd { offset: usize },
    InvalidUnsigned { offset: usize },
    InvalidSigned { offset: usize },
}

/// Неизменяемый индекс .ril; клоны разделяют записи пакетов и дерево.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesInfo {
    snapshot: Arc<FilesInfoSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct FilesInfoSnapshot {
    version: Vec<u8>,
    package_infos: Vec<PackFileInfo>,
    root: FileInfo,
    file_count: u32,
}

impl FilesInfo {
    /// Точное устойчивое состояние `CFilesInfo::CFilesInfo` без GUI callback-а.
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(FilesInfoSnapshot {
                version: b"00.00.0000".to_vec(),
                package_infos: Vec::new(),
                root: FileInfo::root(),
                file_count: 0,
            }),
        }
    }

    /// Читает успешный текстовый `.ril` layout, использованный World `rfOpen`.
    pub fn from_ril(input: &[u8]) -> Result<Self, FilesInfoParseError> {
        let mut tokens = RilTokens::new(input);
        let version = tokens.next()?.bytes.to_vec();
        let package_count = parse_unsigned(tokens.next()?)?;
        let mut package_infos = Vec::new();
        for _ in 0..package_count {
            package_infos.push(PackFileInfo {
                id: parse_unsigned(tokens.next()?)?,
                // Exact `LoadPackInfo` копирует `%s` без `_strlwr`; это имя
                // позднее образует путь package-файла и не является lookup-key.
                file_name: tokens.next()?.bytes.to_vec(),
                index_num: parse_unsigned(tokens.next()?)?,
                empty_index_num: parse_unsigned(tokens.next()?)?,
            });
        }

        let mut root = FileInfo::root();
        let mut file_count = 0;
        root.size = parse_unsigned(tokens.next()?)?;
        root.origin_size = parse_unsigned(tokens.next()?)?;
        root.valid_size = parse_unsigned(tokens.next()?)?;
        root.crc32 = parse_unsigned(tokens.next()?)?;
        root.package_type = signed_bits(tokens.next()?)?;
        root.compress_type = signed_bits(tokens.next()?)?;
        let _ignored_root_type = signed_bits(tokens.next()?)?;

        read_folder(&mut tokens, &mut root, &mut file_count)?;
        Ok(Self {
            snapshot: Arc::new(FilesInfoSnapshot {
                version,
                package_infos,
                root,
                file_count,
            }),
        })
    }

    pub fn version(&self) -> &[u8] {
        &self.snapshot.version
    }

    pub fn package_infos(&self) -> &[PackFileInfo] {
        &self.snapshot.package_infos
    }

    pub fn root(&self) -> &FileInfo {
        &self.snapshot.root
    }

    pub fn file_count(&self) -> u32 {
        self.snapshot.file_count
    }

    /// Повторяет `GetFileInfoByText` для нормального пути `rfOpen`.
    ///
    /// Как точный общий RAW, пустой путь и пустой segment возвращают root;
    /// путь без начального `\\` также не проходит ни одного сегмента и оставляет
    /// root. `CheckRFileStr` нормализует реальный путь до этого вызова.
    pub fn file_info_by_text(&self, text: &[u8]) -> Option<&FileInfo> {
        let mut current = &self.snapshot.root;
        let mut remaining = text;
        while remaining.first() == Some(&b'\\') {
            remaining = &remaining[1..];
            let segment_end = remaining
                .iter()
                .position(|byte| *byte == b'\\')
                .unwrap_or(remaining.len());
            let segment = &remaining[..segment_end];
            remaining = &remaining[segment_end..];
            current = find_child(&self.snapshot.root, current, segment)?;
        }
        Some(current)
    }

    /// `CFilesInfo::FindFileList(root, extension, output)` приводит к нижнему
    /// регистру и root, и extension. World RVA 0xd2ecb–0xd2f33: второй `_strlwr`
    /// расположен после `_free`, где прежний анализ Ghidra обрывал функцию.
    /// В Rust преобразование ограничено ASCII. Порядок обхода задаёт BTreeMap;
    /// каждый результат начинается с обратного слеша.
    pub fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        let root = lowercase_ascii(root);
        let extension = lowercase_ascii(extension);
        let Some(root) = self.file_info_by_text(&root) else {
            return Vec::new();
        };
        let mut files = Vec::new();
        let mut path = Vec::new();
        find_file_list(root, &extension, &mut path, &mut files);
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
        let name = lowercase_ascii(tokens.next()?.bytes);
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
    // Нормализованный rfOpen-путь уже в нижнем регистре: BTreeMap
    // принимает заимствованный ключ без отдельного Vec для каждого сегмента.
    if text.iter().any(u8::is_ascii_uppercase) {
        parent.children.get(&lowercase_ascii(text))
    } else {
        parent.children.get(text)
    }
}

fn lowercase_ascii(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn parse_unsigned(token: RilToken<'_>) -> Result<u32, FilesInfoParseError> {
    let error = FilesInfoParseError::InvalidUnsigned { offset: token.offset };
    std::str::from_utf8(token.bytes)
        .map_err(|_| error)?
        .parse()
        .map_err(|_| error)
}

fn signed_bits(token: RilToken<'_>) -> Result<u32, FilesInfoParseError> {
    let error = FilesInfoParseError::InvalidSigned { offset: token.offset };
    std::str::from_utf8(token.bytes)
        .map_err(|_| error)?
        .parse::<i32>()
        .map(|value| value as u32)
        .map_err(|_| error)
}

struct RilToken<'a> {
    bytes: &'a [u8],
    offset: usize,
}

struct RilTokens<'a> {
    remaining: &'a [u8],
    offset: usize,
}

impl<'a> RilTokens<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            remaining: input,
            offset: 0,
        }
    }

    fn next(&mut self) -> Result<RilToken<'a>, FilesInfoParseError> {
        let start = self
            .remaining
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())
            .ok_or(FilesInfoParseError::UnexpectedEnd {
                offset: self.offset + self.remaining.len(),
            })?;
        self.remaining = &self.remaining[start..];
        self.offset += start;
        let end = self
            .remaining
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(self.remaining.len());
        let token = RilToken {
            bytes: &self.remaining[..end],
            offset: self.offset,
        };
        self.remaining = &self.remaining[end..];
        self.offset += end;
        Ok(token)
    }
}
