//! Чтение индекса и распаковка .pak по public/package.cpp.
//! Основная read-side привязка: World CPackage::Open, RVA 0x000D1D00.
//! Варианты EXE/PDB и границы реконструкции:
//! docs/architecture/resources-and-configuration.md, раздел «Снимок пакета».
//! Владение снимком и типизированные ошибки — инфраструктура Rust.

use std::{collections::BTreeMap, sync::Arc};

use flate2::{Decompress, FlushDecompress, Status};

use crate::protocol::LegacyReader;

const PACKAGE_HEADER_LEN: usize = 12;
const FILE_INDEX_LEN: usize = 0x118;
const FILE_INDEX_NAME_LEN: usize = 256;

/// Поля одной подтверждённой `tagFileIndex` записи.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageFileIndex {
    name: Vec<u8>,
    offset: u32,
    size: u32,
    origin_size: u32,
    valid_size: u32,
    crc32: u32,
    compress_type: u32,
}

impl PackageFileIndex {
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
    pub fn compress_type(&self) -> u32 {
        self.compress_type
    }
}

/// Ошибка safe read-side, не являющаяся историческим bool `CPackage::Open`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageReadError {
    TruncatedHeader,
    IndexSizeOutsideFile,
    IndexCountOutsideHeader,
    EmptyIndexHeaderOutsideFile,
    IndexNameWithoutNul,
    DataOutsideFile,
    BufferTooSmall { required: u32, available: u32 },
    LzoDecoder,
    ZlibDecoder,
    ZlibOutputIncomplete,
    AllocationFailed { requested: u32 },
}

/// Неизменяемый снимок .pak: клоны разделяют байты и индекс.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageArchive {
    snapshot: Arc<PackageSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct PackageSnapshot {
    bytes: Vec<u8>,
    indexes: BTreeMap<Vec<u8>, PackageFileIndex>,
}

impl PackageArchive {
    /// Читает подтверждённую read-side часть `CPackage::Open`.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, PackageReadError> {
        if bytes.len() < PACKAGE_HEADER_LEN {
            return Err(PackageReadError::TruncatedHeader);
        }
        let index_head_size = read_u32(&bytes, 4).ok_or(PackageReadError::TruncatedHeader)?;
        let index_count = read_u32(&bytes, 8).ok_or(PackageReadError::TruncatedHeader)?;
        let capacity = index_head_size as usize / FILE_INDEX_LEN;
        if index_count as usize > capacity {
            return Err(PackageReadError::IndexCountOutsideHeader);
        }
        let index_end = PACKAGE_HEADER_LEN
            .checked_add(index_count as usize * FILE_INDEX_LEN)
            .ok_or(PackageReadError::IndexSizeOutsideFile)?;
        if index_end > bytes.len() {
            return Err(PackageReadError::IndexSizeOutsideFile);
        }
        // `CPackage::Open` после file-index записей делает `fseek(12 +
        // dwIndexHeadSize)` и два `fread(u32)` empty-index заголовка. Эти
        // update-side записи не нужны read-only owner-у, но сам header
        // обязателен в layout; иначе исходник читает неинициализированные
        // данные. Rust отклоняет такой повреждённый snapshot детерминированно.
        let empty_index_header = PACKAGE_HEADER_LEN
            .checked_add(index_head_size as usize)
            .and_then(|offset| offset.checked_add(8))
            .ok_or(PackageReadError::EmptyIndexHeaderOutsideFile)?;
        if empty_index_header > bytes.len() {
            return Err(PackageReadError::EmptyIndexHeaderOutsideFile);
        }

        let mut indexes = BTreeMap::new();
        for number in 0..index_count as usize {
            let start = PACKAGE_HEADER_LEN + number * FILE_INDEX_LEN;
            let mut raw = [0_u8; FILE_INDEX_LEN];
            raw.copy_from_slice(&bytes[start..start + FILE_INDEX_LEN]);
            for byte in &mut raw {
                *byte = !*byte;
            }
            let nul = raw[..FILE_INDEX_NAME_LEN]
                .iter()
                .position(|byte| *byte == 0)
                .ok_or(PackageReadError::IndexNameWithoutNul)?;
            let name = lowercase_ascii(&raw[..nul]);
            indexes.insert(
                name.clone(),
                PackageFileIndex {
                    name,
                    offset: read_u32(&raw, 256).expect("полная tagFileIndex"),
                    size: read_u32(&raw, 260).expect("полная tagFileIndex"),
                    origin_size: read_u32(&raw, 264).expect("полная tagFileIndex"),
                    valid_size: read_u32(&raw, 268).expect("полная tagFileIndex"),
                    crc32: read_u32(&raw, 272).expect("полная tagFileIndex"),
                    compress_type: read_u32(&raw, 276).expect("полная tagFileIndex"),
                },
            );
        }
        Ok(Self {
            snapshot: Arc::new(PackageSnapshot { bytes, indexes }),
        })
    }

    pub fn file_size(&self, name: &[u8]) -> u32 {
        self.snapshot
            .indexes
            .get(&lowercase_ascii(name))
            .map_or(0, PackageFileIndex::size)
    }

    /// Проверяет границы ExtractToBuf и заимствует запись с её байтами.
    /// Срез привязан к снимку: распаковка не создаёт копию сжатого блока.
    pub fn extract_compressed(
        &self,
        name: &[u8],
        capacity: u32,
    ) -> Result<Option<(&PackageFileIndex, &[u8])>, PackageReadError> {
        let Some(index) = self.snapshot.indexes.get(&lowercase_ascii(name)) else {
            return Ok(None);
        };
        if index.size > capacity {
            return Err(PackageReadError::BufferTooSmall {
                required: index.size,
                available: capacity,
            });
        }
        let start = index.offset as usize;
        let end = start
            .checked_add(index.size as usize)
            .ok_or(PackageReadError::DataOutsideFile)?;
        let payload = self
            .snapshot
            .bytes
            .get(start..end)
            .ok_or(PackageReadError::DataOutsideFile)?;
        Ok(Some((index, payload)))
    }

    /// Восстанавливает LZO/zlib ветви `rfOpen`.
    ///
    /// Тип сжатия принадлежит записи пакета, а верхняя граница результата —
    /// связанной записи `FilesInfo::tagFileInfo::dwOrginSize`. Эти два owner-а
    /// намеренно не подменяют друг друга: именно так `rfOpen` передаёт их в
    /// `DeCompressData`/`DeCompress`.
    pub fn extract_decoded(
        &self,
        name: &[u8],
        output_capacity: u32,
    ) -> Result<Option<Vec<u8>>, PackageReadError> {
        let Some((index, compressed)) = self.extract_compressed(name, u32::MAX)? else {
            return Ok(None);
        };
        // `rfOpen` передаёт blob прямо в `CRFile`, когда установлен bit 0.
        // Только cleared bit 0 означает сжатое содержимое и вызывает LZO/zlib.
        if index.compress_type & 1 != 0 {
            let mut output = reserve_output(index.size)?;
            output.extend_from_slice(compressed);
            return Ok(Some(output));
        }
        let mut output = reserve_output(output_capacity)?;
        output.resize(output_capacity as usize, 0);
        if index.compress_type & 4 == 0 {
            let written = lzo::decompress_into(compressed, &mut output)
                .map_err(|_| PackageReadError::LzoDecoder)?;
            output.truncate(written);
            return Ok(Some(output));
        }
        let mut decoder = Decompress::new(true);
        let status = decoder
            .decompress(compressed, &mut output, FlushDecompress::Finish)
            .map_err(|_| PackageReadError::ZlibDecoder)?;
        let written = usize::try_from(decoder.total_out())
            .map_err(|_| PackageReadError::ZlibOutputIncomplete)?;
        if status != Status::StreamEnd || written > output.len() {
            return Err(PackageReadError::ZlibOutputIncomplete);
        }
        output.truncate(written);
        Ok(Some(output))
    }
}

// Размер задан записью ресурса. Отказ выделения возвращается через тот же
// путь ошибок распаковки; частично подготовленные байты не публикуются.
fn reserve_output(size: u32) -> Result<Vec<u8>, PackageReadError> {
    let capacity = usize::try_from(size)
        .map_err(|_| PackageReadError::AllocationFailed { requested: size })?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| PackageReadError::AllocationFailed { requested: size })?;
    Ok(output)
}

fn lowercase_ascii(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u32())
        .ok()
}
