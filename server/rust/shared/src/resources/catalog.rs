//! Каталог индекса и пакетов по public/clientresource.cpp/.h, World CClientResource.
//! LoadEx задаёт порядок загрузки; публикация и жизненный цикл принадлежат серверу.
//! Правила и ограничения: docs/architecture/resources-and-configuration.md.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
};

use super::path::resolve_resource_path;
use super::{FileInfo, FilesInfo, FilesInfoParseError, PackageArchive, PackageReadError};

/// Отказ загрузки индекса; это результат Rust, не исходный bool `LoadEx`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceLoadError {
    FileInfoOpen {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    FileInfoParse {
        path: PathBuf,
        source: FilesInfoParseError,
    },
}

/// Результат одного пакета после сведения повторных ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourcePackageLoad {
    Loaded {
        package_type: u32,
        path: PathBuf,
    },
    SkippedEmptyName {
        package_type: u32,
    },
    OpenFailed {
        package_type: u32,
        path: PathBuf,
        kind: io::ErrorKind,
    },
    ReadFailed {
        package_type: u32,
        path: PathBuf,
        kind: io::ErrorKind,
    },
    ParseFailed {
        package_type: u32,
        path: PathBuf,
        source: PackageReadError,
    },
}

/// Каталог с доступными пакетами и отчёт обо всех попытках их загрузки.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceLoadReport {
    pub catalog: ResourceCatalog,
    pub packages: Vec<ResourcePackageLoad>,
}

/// Неизменяемый каталог; клоны разделяют снимки индекса и каждого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCatalog {
    files_info: FilesInfo,
    packages: BTreeMap<u32, PackageArchive>,
}

impl ResourceCatalog {
    /// Читает FilesInfo.ril и Package/<имя>.pak относительно переданного корня.
    /// Отказ пакета попадает в отчёт и не отменяет загрузку остальных.
    pub fn load_directory(root: &Path) -> Result<ResourceLoadReport, ResourceLoadError> {
        let file_info_path = root.join("FilesInfo.ril");
        let file_info_bytes = resolve_resource_path(&file_info_path)
            .and_then(fs::read)
            .map_err(|error| ResourceLoadError::FileInfoOpen {
                path: file_info_path.clone(),
                kind: error.kind(),
            })?;
        let files_info = FilesInfo::from_ril(&file_info_bytes).map_err(|source| {
            ResourceLoadError::FileInfoParse {
                path: file_info_path,
                source,
            }
        })?;

        // Последнее имя на ID побеждает; пакеты открываются по возрастанию ID.
        let package_names = files_info
            .package_infos()
            .iter()
            .map(|info| (info.id, info.file_name.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut packages = BTreeMap::new();
        let mut package_loads = Vec::with_capacity(package_names.len());
        for (package_type, file_name) in package_names {
            if file_name.is_empty() {
                package_loads.push(ResourcePackageLoad::SkippedEmptyName { package_type });
                continue;
            }

            let path = package_path(root, &file_name);
            let bytes = match read_package_snapshot(&path) {
                Ok(bytes) => bytes,
                Err(PackageSnapshotReadError::Open(error)) => {
                    package_loads.push(ResourcePackageLoad::OpenFailed {
                        package_type,
                        path,
                        kind: error.kind(),
                    });
                    continue;
                }
                Err(PackageSnapshotReadError::Read(error)) => {
                    package_loads.push(ResourcePackageLoad::ReadFailed {
                        package_type,
                        path,
                        kind: error.kind(),
                    });
                    continue;
                }
            };
            match PackageArchive::from_bytes(bytes) {
                Ok(package) => {
                    packages.insert(package_type, package);
                    package_loads.push(ResourcePackageLoad::Loaded { package_type, path });
                }
                Err(source) => package_loads.push(ResourcePackageLoad::ParseFailed {
                    package_type,
                    path,
                    source,
                }),
            }
        }

        Ok(ResourceLoadReport {
            catalog: Self {
                files_info,
                packages,
            },
            packages: package_loads,
        })
    }
    pub(super) fn package(&self, package_type: u32) -> Option<&PackageArchive> {
        self.packages.get(&package_type)
    }

    /// Внутренняя граница: путь уже нормализован открывающим его source.
    pub(super) fn file_info_normalized(&self, normalized: &[u8]) -> Option<&FileInfo> {
        self.files_info.file_info_by_text(normalized)
    }

    pub fn is_file_exist(&self, path: &[u8]) -> bool {
        // Сохранена особенность прежнего IsFileExist: при отсутствии начального
        // слеша он дописывается в конец. Открытие ресурса использует иной путь.
        let mut lookup = path
            .split(|byte| *byte == 0)
            .next()
            .unwrap_or_default()
            .to_vec();
        if lookup.first() != Some(&b'\\') {
            lookup.push(b'\\');
        }
        self.files_info.file_info_by_text(&lookup).is_some()
    }

    /// Список из индекса; наличие записи не подтверждает доступность пакета.
    pub fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        self.files_info.find_file_list(root, extension)
    }
}

// Сохранён режим r+b прежнего OpenFileHandle; снимок освобождает дескриптор.
fn read_package_snapshot(path: &Path) -> Result<Vec<u8>, PackageSnapshotReadError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(resolve_resource_path(path).map_err(PackageSnapshotReadError::Open)?)
        .map_err(PackageSnapshotReadError::Open)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(PackageSnapshotReadError::Read)?;
    Ok(bytes)
}

enum PackageSnapshotReadError {
    Open(io::Error),
    Read(io::Error),
}

// Преобразование в host-path остаётся lossy; ключи индексов хранят исходные байты.
fn package_path(root: &Path, file_name: &[u8]) -> PathBuf {
    let mut package_name = String::from_utf8_lossy(file_name).into_owned();
    package_name.push_str(".pak");
    root.join("Package").join(package_name)
}
