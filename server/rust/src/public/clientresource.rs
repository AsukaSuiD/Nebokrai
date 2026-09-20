//! Read-side `CClientResource` WorldServer из точной пары EXE/PDB.
//!
//! Owner читает `FilesInfo.ril`, сворачивает package ID как ordered map и
//! открывает package records в key-order с исходным доступом `r+b`. Filesystem
//! и format failures выражены typed report-ом, потому что bool-epilogue `LoadEx`
//! не определён. `FilesInfo`/`PackageArchive` остаются внешними shared owners;
//! этот файл не копирует их ServerUpdate/GameServer семантику.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::public::filesinfo::{FileInfo, FilesInfo, FilesInfoParseError};
use crate::public::package::PackageArchive;
use crate::public::package::PackageReadError;
use crate::public::rfile::{CRFile, RFileResource, rf_open};

/// Безусловный `AddLogText` оригинал `CGame::LoadServerResource`.
pub(crate) const LOAD_SERVER_RESOURCE_SUCCESS_LOG: &[u8] = b"Load package file OK!";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClientResourceReadError {
    MissingPackage { package_type: u32 },
    Package(PackageReadError),
}

/// Ошибка чтения `FilesInfo.ril` на safe disk-boundary, не legacy bool `LoadEx`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClientResourceLoadError {
    FileInfoOpen { path: PathBuf, kind: io::ErrorKind },
    FileInfoParse(FilesInfoParseError),
}

/// Результат одной package-записи после того, как `std::map` уже свёл ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClientResourcePackageLoad {
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

/// Наблюдаемый ход read-side `LoadEx` без ложного отображения его bool-epilogue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientResourceLoadReport {
    pub(crate) resource: ClientResource,
    pub(crate) packages: Vec<ClientResourcePackageLoad>,
}

/// Итог оригинал замены `g_pDefaultClientResource` из `CGame::LoadServerResource`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DefaultClientResourceReplacement {
    Loaded {
        previous_owner_released: bool,
        packages: Vec<ClientResourcePackageLoad>,
        legacy_result: bool,
    },
    IndexUnavailable {
        previous_owner_released: bool,
        source: ClientResourceLoadError,
        legacy_result: bool,
    },
}

/// Safe owner process-global resource pointer WorldServer-а.
///
/// Оригинал освобождает прежний `g_pDefaultClientResource`, немедленно
/// публикует новый `CClientResource`, игнорирует bool `LoadEx` и возвращает
/// `true`. `Installed` сохраняет эту промежуточную/ошибочную доступность: даже
/// при неудачном `.ril` runtime видит новый current-folder и может выполнить
/// loose fallback, но package index отсутствует.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DefaultClientResourceOwner {
    installed: Option<InstalledClientResource>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstalledClientResource {
    root: PathBuf,
    resource: Option<ClientResource>,
}

/// Заимствованная безопасная форма nullable `g_pDefaultClientResource`.
///
/// `Some` означает, что `CGame::LoadServerResource` уже опубликовал новый
/// owner, даже если последующий `LoadEx` не смог прочитать индекс. Последний
/// случай сохраняет C++-поведение запросов: `IsFileExist` возвращает `false`,
/// а `FindFileList` выдаёт пустой список; `rfOpen` остаётся loose fallback.
pub(crate) struct DefaultClientResourceRef<'a> {
    installed: &'a InstalledClientResource,
}

impl<'a> DefaultClientResourceRef<'a> {
    pub(crate) fn is_file_exist(&self, path: &[u8]) -> bool {
        self.installed
            .resource
            .as_ref()
            .is_some_and(|resource| resource.is_file_exist(path))
    }

    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        self.installed
            .resource
            .as_ref()
            .map_or_else(Vec::new, |resource| {
                resource.find_file_list(root, extension)
            })
    }

    fn has_loaded_index(&self) -> bool {
        self.installed.resource.is_some()
    }

    fn rfile_resource(&self) -> RFileResource<'a> {
        match self.installed.resource.as_ref() {
            Some(resource) => RFileResource::loaded(resource, &self.installed.root),
            None => RFileResource::without_index(&self.installed.root),
        }
    }
}

/// Безопасная граница process-global `GetDefaultClientResource`.
///
/// В отличие от старого указателя результат живёт только пока заимствован
/// `DefaultClientResourceOwner`; это исключает dangling pointer между
/// `LoadServerResource` и `Release`, сохраняя nullable результат вызова.
pub(crate) fn get_default_client_resource(
    owner: &DefaultClientResourceOwner,
) -> Option<DefaultClientResourceRef<'_>> {
    owner
        .installed
        .as_ref()
        .map(|installed| DefaultClientResourceRef { installed })
}

impl DefaultClientResourceOwner {
 /// Точный owner-порядок `LoadServerResource` после уже полученного cwd.
    pub(crate) fn replace_from_world_directory(
        &mut self,
        root: &Path,
    ) -> DefaultClientResourceReplacement {
 // Оригинал `CGame::LoadServerResource` сначала удаляет прежний global
 // `CClientResource`, и лишь затем создаёт/публикует новый. `replace`
 // менял бы этот lifecycle-order местами, потому что старый owner
 // доживает до конца выражения уже после записи нового значения.
        let previous = self.installed.take();
        let previous_owner_released = previous.is_some();
        drop(previous);
        self.installed = Some(InstalledClientResource {
            root: root.to_path_buf(),
            resource: None,
        });
        let load = ClientResource::load_world_server_directory(root);
        match load {
            Ok(report) => {
                let packages = report.packages;
                self.installed
                    .as_mut()
                    .expect("новый default resource опубликован до LoadEx")
                    .resource = Some(report.resource);
                DefaultClientResourceReplacement::Loaded {
                    previous_owner_released,
                    packages,
 // Оригинал caller игнорирует bool LoadEx и всегда возвращает true.
                    legacy_result: true,
                }
            }
            Err(source) => DefaultClientResourceReplacement::IndexUnavailable {
                previous_owner_released,
                source,
                legacy_result: true,
            },
        }
    }

 /// Replaces nullable `GetDefaultClientResource` plus immediate `rfOpen`.
    pub(crate) fn open(&self, path: &[u8]) -> Option<CRFile> {
        let context = get_default_client_resource(self).map(|resource| resource.rfile_resource());
        rf_open(path, context)
    }

 /// Читает полный newly-opened `CRFile` для World resource-context.
 ///
 /// `rfOpen` всегда создаёт cursor с позицией ноль. Неудача открытия или
 /// `ReadData` повторяет nullable file-result и преобразуется в `None`,
 /// как того ждут существующие resource consumers.
    pub(crate) fn read_resource(&self, path: &[u8]) -> Option<Vec<u8>> {
        let mut file = self.open(path)?;
        let mut data = vec![0; usize::try_from(file.size()).ok()?];
        file.read_data(&mut data).then_some(data)
    }

 /// Оригинал `GetDefaultClientResource()->IsFileExist` без global raw pointer.
    pub(crate) fn is_file_exist(&self, path: &[u8]) -> bool {
        get_default_client_resource(self).is_some_and(|resource| resource.is_file_exist(path))
    }

 /// Возвращает package-backed file list, если текущий default owner имеет
 /// загруженный индекс. `None` отличает unavailable `LoadEx` от пустого
 /// списка и оставляет caller-у оригинал loose fallback.
    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Option<Vec<Vec<u8>>> {
        let resource = get_default_client_resource(self)?;
        (resource.has_loaded_index() && resource.is_file_exist(root))
            .then(|| resource.find_file_list(root, extension))
    }

 /// Оригинал выбор `CSkillFactory::{LoadSkillCache,LoadUsageCache}` между
 /// package-index и рекурсивным loose `FindFile`.
 ///
 /// Наличие самого `.ril` недостаточно: исходный caller сначала проверял
 /// `IsFileExist("\\skills")` и лишь для существующего root вызывал
 /// `FindFileList`. Иначе он обходил loose `SKILLS`; `read_dir` уже не
 /// выдаёт служебную первую запись `.`, которую старый `FindFile` пропускал
 /// своим предварительным `FindNextFile`.
    pub(crate) fn find_cache_file_list(&self, extension: &[u8]) -> Vec<Vec<u8>> {
        const INDEX_ROOT: &[u8] = b"\\skills";
        if let Some(files) = self.find_file_list(INDEX_ROOT, extension) {
            return files;
        }
        let Some(installed) = self.installed.as_ref() else {
            return Vec::new();
        };
        let mut files = Vec::new();
        find_loose_cache_files(
            &installed.root.join("SKILLS"),
            b".\\skills".to_vec(),
            extension,
            &mut files,
        );
        files
    }

 /// Оригинал Release удаляет global pointer только если он был установлен.
    pub(crate) fn clear(&mut self) -> bool {
        self.installed.take().is_some()
    }

    pub(crate) fn is_installed(&self) -> bool {
        self.installed.is_some()
    }
}

fn find_loose_cache_files(
    directory: &Path,
    wire_directory: Vec<u8>,
    extension: &[u8],
    files: &mut Vec<Vec<u8>>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let mut name = entry.file_name().to_string_lossy().into_owned().into_bytes();
        name.make_ascii_lowercase();
        let mut wire_path = wire_directory.clone();
        wire_path.push(b'\\');
        wire_path.extend_from_slice(&name);
        if file_type.is_dir() {
            if name != b".." {
                find_loose_cache_files(&entry.path(), wire_path, extension, files);
            }
            continue;
        }
        let extension = extension.strip_prefix(b".").unwrap_or(extension);
        let matches_extension = name
            .rsplit(|byte| *byte == b'.')
            .next()
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension));
        if matches_extension {
            files.push(wire_path);
        }
    }
}

/// Связанный read-side owner одного World resource набора.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientResource {
    files_info: FilesInfo,
    packages: BTreeMap<u32, PackageArchive>,
}

impl ClientResource {
 /// Принимает результаты оригинал `.ril` и `.pak` owner-ов в порядке `LoadEx`.
    pub(crate) fn new(files_info: FilesInfo, packages: BTreeMap<u32, PackageArchive>) -> Self {
        Self {
            files_info,
            packages,
        }
    }

 /// Загружает новый World resource owner из оригинал `cwd` layout.
 ///
 /// `CGame::LoadServerResource` всегда создаёт новый `CClientResource`, так
 /// что здесь нет старых map-записей между вызовами. После успешного `.ril`
 /// пути `LoadEx` сначала строит `std::map` по package ID: дубликат ID
 /// заменяет прежнее имя, а `LoadPackage(false)` идёт в sorted key-order.
 /// Пустое имя пропускает `CPackage::Open`. Для непустого имени точный
 /// `OpenFileHandle` добавляет literal `.pak` к имени из `.ril`, поэтому
 /// `data` открывается как `Package/data.pak` в режиме `r+b`. Неоткрытый/
 /// повреждённый пакет остаётся в report, а остальные пакеты продолжают
 /// обрабатываться — это сохраняет порядок его side effects без unsafe
 /// `FILE*` lifetime.
    pub(crate) fn load_world_server_directory(
        root: &Path,
    ) -> Result<ClientResourceLoadReport, ClientResourceLoadError> {
        let file_info_path = root.join("FilesInfo.ril");
        let file_info_bytes =
            fs::read(&file_info_path).map_err(|error| ClientResourceLoadError::FileInfoOpen {
                path: file_info_path.clone(),
                kind: error.kind(),
            })?;
        let files_info = FilesInfo::from_ril(&file_info_bytes)
            .map_err(ClientResourceLoadError::FileInfoParse)?;

 // `map::operator[]` в оригинал `LoadEx` оставляет по одному последнему
 // имени на ID. Последующий `LoadPackage` обходит именно этот map.
        let package_names = files_info
            .package_infos()
            .iter()
            .map(|info| (info.id, info.file_name.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut packages = BTreeMap::new();
        let mut package_loads = Vec::with_capacity(package_names.len());
        for (package_type, file_name) in package_names {
            if file_name.is_empty() {
                package_loads.push(ClientResourcePackageLoad::SkippedEmptyName { package_type });
                continue;
            }

            let path = package_path(root, &file_name);
            let bytes = match read_package_snapshot(&path) {
                Ok(bytes) => bytes,
                Err(PackageSnapshotReadError::Open(error)) => {
                    package_loads.push(ClientResourcePackageLoad::OpenFailed {
                        package_type,
                        path,
                        kind: error.kind(),
                    });
                    continue;
                }
                Err(PackageSnapshotReadError::Read(error)) => {
                    package_loads.push(ClientResourcePackageLoad::ReadFailed {
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
                    package_loads.push(ClientResourcePackageLoad::Loaded { package_type, path });
                }
                Err(source) => package_loads.push(ClientResourcePackageLoad::ParseFailed {
                    package_type,
                    path,
                    source,
                }),
            }
        }

        Ok(ClientResourceLoadReport {
            resource: Self::new(files_info, packages),
            packages: package_loads,
        })
    }

 /// Повторяет nullable `CClientResource::GetPackage`.
    pub(crate) fn package(&self, package_type: u32) -> Option<&PackageArchive> {
        self.packages.get(&package_type)
    }

 /// Нормализованный lookup `rfOpen` после оригинал `CheckRFileStr`.
    pub(crate) fn file_info(&self, path: &[u8]) -> Option<&FileInfo> {
        let mut normalized = path.to_vec();
        for byte in &mut normalized {
            byte.make_ascii_lowercase();
            if *byte == b'/' {
                *byte = b'\\';
            }
        }
        if normalized.first() != Some(&b'\\') {
            normalized.insert(0, b'\\');
        }
        self.files_info.file_info_by_text(&normalized)
    }

    pub(crate) fn is_file_exist(&self, path: &[u8]) -> bool {
 // `IsFileExist` не вызывает `CheckRFileStr`: оригинал код ищет первый
 // `\\` и, когда он не стоит в позиции ноль (включая отсутствие),
 // дописывает `\\` *в конец*. `GetFileInfoByText` тогда оставляет
 // root, поэтому путь без initial `\\` наблюдаемо даёт `true`.
 // Это не перенос внутренней ошибки, а публичный bool-quirk; rfOpen
 // продолжает использовать отдельный нормализованный `file_info`.
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

 /// Связывает `CClientResource::FindFileList` с owned `.ril` tree.
    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        self.files_info.find_file_list(root, extension)
    }

 /// Достижимая package-ветвь `rfOpen`: loose-файлы здесь намеренно не
 /// выбираются, потому что их current-folder/open error owner отдельный.
 /// `compress_type` берёт `PackageArchive`, но выходной предел распаковки
 /// передаётся из связанной записи `FilesInfo`, как в исходном `rfOpen`.
    pub(crate) fn read_packaged(
        &self,
        path: &[u8],
    ) -> Result<Option<Vec<u8>>, ClientResourceReadError> {
        let mut normalized = path.to_vec();
        for byte in &mut normalized {
            byte.make_ascii_lowercase();
            if *byte == b'/' {
                *byte = b'\\';
            }
        }
        if normalized.first() != Some(&b'\\') {
            normalized.insert(0, b'\\');
        }
        let Some(info) = self.files_info.file_info_by_text(&normalized) else {
            return Ok(None);
        };
        if info.package_type() & 1 != 0 {
            return Ok(None);
        }
        let package =
            self.package(info.package_type())
                .ok_or(ClientResourceReadError::MissingPackage {
                    package_type: info.package_type(),
                })?;
        package
            .extract_decoded(&normalized, info.origin_size())
            .map_err(ClientResourceReadError::Package)
    }

 /// Loose fallback `rfOpen` для absent index либо установленного bit 0.
    pub(crate) fn read_loose(&self, root: &Path, path: &[u8]) -> io::Result<Option<Vec<u8>>> {
        let mut normalized = path.to_vec();
        for byte in &mut normalized {
            byte.make_ascii_lowercase();
            if *byte == b'/' {
                *byte = b'\\';
            }
        }
        if normalized.first() != Some(&b'\\') {
            normalized.insert(0, b'\\');
        }
        if self
            .files_info
            .file_info_by_text(&normalized)
            .is_some_and(|info| info.package_type() & 1 == 0)
        {
            return Ok(None);
        }
        let mut file = root.to_path_buf();
        for part in normalized[1..].split(|byte| *byte == b'\\') {
            if !part.is_empty() {
                file.push(String::from_utf8_lossy(part).as_ref());
            }
        }
        fs::read(file).map(Some)
    }
}

/// Снимок `.pak` после literal `fopen(path, "r+b")` оригинал `OpenFileHandle`.
///
/// Read-only доступ намеренно недостаточен: оригинальный read-side требует
/// одновременно права записи. После успешного открытия содержимое берётся в
/// owned buffer, так что дальнейшая работа не наследует `FILE*` lifetime.
fn read_package_snapshot(path: &Path) -> Result<Vec<u8>, PackageSnapshotReadError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
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

/// Строит оригинал `cwd + "\\Package" + package file name + ".pak"`.
///
/// PDB-вариант хранит путь как narrow `char`; текущий World resource-contract
/// подтверждён для ASCII, поэтому lossy adapter ограничен только host-path
/// boundary и не меняет byte keys `.ril`/`.pak` в памяти.
fn package_path(root: &Path, file_name: &[u8]) -> PathBuf {
    let mut package_name = String::from_utf8_lossy(file_name).into_owned();
    package_name.push_str(".pak");
    root.join("Package").join(package_name)
}
