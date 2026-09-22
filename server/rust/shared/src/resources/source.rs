//! Выбор источника по public/rfile.cpp, World rfOpen RVA 0x0005AB20.
//! Явный контекст заменяет глобальный указатель; ошибки возвращаются владельцу.
//! Правила и привязки: docs/architecture/resources-and-configuration.md.

use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use super::path::{normalize_resource_path, resolve_resource_path};
use super::{CRFile, PackageReadError, ResourceCatalog};

/// Корень уже установленного контекста; каталог может отсутствовать после отказа .ril.
pub struct ResourceSource<'a> {
    root: &'a Path,
    catalog: Option<&'a ResourceCatalog>,
}

impl<'a> ResourceSource<'a> {
    pub fn new(root: &'a Path, catalog: Option<&'a ResourceCatalog>) -> Self {
        Self { root, catalog }
    }
}

/// Причина отказа выбранного источника. Ошибка пакета не разрешает fallback на диск.
#[derive(Debug)]
pub enum ResourceOpenError {
    LooseFile {
        path: PathBuf,
        source: io::Error,
    },
    MissingPackage {
        package_type: u32,
    },
    MissingPackageEntry {
        package_type: u32,
    },
    Package {
        package_type: u32,
        source: PackageReadError,
    },
    MemoryCursor {
        package_type: u32,
        source: io::Error,
    },
}

/// Открывает ресурс; без контекста использует входной дисковый путь без нормализации.
pub fn open_resource(
    path: &[u8],
    context: Option<ResourceSource<'_>>,
) -> Result<CRFile, ResourceOpenError> {
    let path = path.split(|byte| *byte == 0).next().unwrap_or_default();
    let Some(context) = context else {
        return open_loose_file(Path::new(String::from_utf8_lossy(path).as_ref()));
    };

    let mut normalized = path.to_vec();
    normalize_resource_path(&mut normalized);
    if let Some(catalog) = context.catalog {
        // Один lookup выбирает источник и даёт размер для распаковки.
        if let Some(info) = catalog.file_info_normalized(&normalized) {
            let package_type = info.package_type();
            if package_type & 1 == 0 {
                let package = catalog
                    .package(package_type)
                    .ok_or(ResourceOpenError::MissingPackage { package_type })?;
                let data = package
                    .extract_decoded(&normalized, info.origin_size())
                    .map_err(|source| ResourceOpenError::Package { package_type, source })?
                    .ok_or(ResourceOpenError::MissingPackageEntry { package_type })?;
                return CRFile::from_memory(data).map_err(|source| {
                    ResourceOpenError::MemoryCursor { package_type, source }
                });
            }
        }
    }
    open_loose_file(&resource_loose_path(context.root, &normalized))
}

fn resource_loose_path(root: &Path, normalized: &[u8]) -> PathBuf {
    let mut path = root.to_path_buf();
    for part in normalized[1..].split(|byte| *byte == b'\\') {
        if !part.is_empty() {
            path.push(String::from_utf8_lossy(part).as_ref());
        }
    }
    path
}

fn open_loose_file(path: &Path) -> Result<CRFile, ResourceOpenError> {
    let opened = (|| -> io::Result<CRFile> {
        let file = File::open(resolve_resource_path(path)?)?;
        let size = u32::try_from(file.metadata()?.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Ресурс больше u32"))?;
        Ok(CRFile::from_file(file, size))
    })();
    opened.map_err(|source| ResourceOpenError::LooseFile {
        path: path.to_path_buf(),
        source,
    })
}
