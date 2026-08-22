//! Частично восстановленный read-side `CClientResource`.
//!
//! `GetPackage` и `IsFileExist` materialized поверх точных `FilesInfo` и
//! `PackageArchive`: `BTreeMap<u32, _>` заменяет только STL-владение.
//! `load_world_server_directory` воспроизводит доказанный успешный путь
//! `LoadEx`: читает `FilesInfo.ril`, сворачивает package ID как `std::map` и
//! затем открывает записи в key-order из `Package`. Неопределённый в EXE
//! bool-эпилог `LoadEx` намеренно не выдан за Rust-result: filesystem и
//! format ошибки выражены отдельным report-API.

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::public::filesinfo::{FileInfo, FilesInfo, FilesInfoParseError};
use crate::public::package::PackageArchive;
use crate::public::package::PackageReadError;
use crate::public::rfile::{CRFile, RFileResource, rf_open};

/// Безусловный `AddLogText` exact `CGame::LoadServerResource`.
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

/// Итог exact замены `g_pDefaultClientResource` из `CGame::LoadServerResource`.
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
        // Exact `CGame::LoadServerResource` сначала удаляет прежний global
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
                    // Exact caller игнорирует bool LoadEx и всегда возвращает true.
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

    /// Exact `GetDefaultClientResource()->IsFileExist` без global raw pointer.
    pub(crate) fn is_file_exist(&self, path: &[u8]) -> bool {
        get_default_client_resource(self).is_some_and(|resource| resource.is_file_exist(path))
    }

    /// Возвращает package-backed file list, если текущий default owner имеет
    /// загруженный индекс. `None` отличает unavailable `LoadEx` от пустого
    /// списка и оставляет caller-у exact loose fallback.
    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Option<Vec<Vec<u8>>> {
        let resource = get_default_client_resource(self)?;
        resource
            .has_loaded_index()
            .then(|| resource.find_file_list(root, extension))
    }

    /// Exact Release удаляет global pointer только если он был установлен.
    pub(crate) fn clear(&mut self) -> bool {
        self.installed.take().is_some()
    }

    pub(crate) fn is_installed(&self) -> bool {
        self.installed.is_some()
    }
}

/// Связанный read-side owner одного World resource набора.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientResource {
    files_info: FilesInfo,
    packages: BTreeMap<u32, PackageArchive>,
}

impl ClientResource {
    /// Принимает результаты exact `.ril` и `.pak` owner-ов в порядке `LoadEx`.
    pub(crate) fn new(files_info: FilesInfo, packages: BTreeMap<u32, PackageArchive>) -> Self {
        Self {
            files_info,
            packages,
        }
    }

    /// Загружает новый World resource owner из exact `cwd` layout.
    ///
    /// `CGame::LoadServerResource` всегда создаёт новый `CClientResource`, так
    /// что здесь нет старых map-записей между вызовами. После успешного `.ril`
    /// пути `LoadEx` сначала строит `std::map` по package ID: дубликат ID
    /// заменяет прежнее имя, а `LoadPackage(false)` идёт в sorted key-order.
    /// Пустое имя пропускает `CPackage::Open`. Для непустого имени точный
    /// `OpenFileHandle` добавляет literal `.pak` к имени из `.ril`, поэтому
    /// `data` открывается как `Package/data.pak`. Неоткрытый/повреждённый
    /// пакет остаётся в report, а остальные пакеты продолжают обрабатываться
    /// — это сохраняет порядок его side effects без unsafe `FILE*` lifetime.
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

        // `map::operator[]` в exact `LoadEx` оставляет по одному последнему
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
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    package_loads.push(ClientResourcePackageLoad::OpenFailed {
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

    /// Повторяет `IsFileExist` после normalizing lookup text.
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
        self.file_info(path).is_some()
    }

    /// Связывает `CClientResource::FindFileList` с owned `.ril` tree.
    pub(crate) fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        self.files_info.find_file_list(root, extension)
    }

    /// Достижимая package-ветвь `rfOpen`: loose-файлы здесь намеренно не
    /// выбираются, потому что их current-folder/open error owner отдельный.
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
            .extract_decoded(&normalized)
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

/// Строит exact `cwd + "\\Package" + package file name + ".pak"`.
///
/// PDB-вариант хранит путь как narrow `char`; текущий World resource-contract
/// подтверждён для ASCII, поэтому lossy adapter ограничен только host-path
/// boundary и не меняет byte keys `.ril`/`.pak` в памяти.
fn package_path(root: &Path, file_name: &[u8]) -> PathBuf {
    let mut package_name = String::from_utf8_lossy(file_name).into_owned();
    package_name.push_str(".pak");
    root.join("Package").join(package_name)
}

// COMPONENT_VARIANT_BEGIN: ServerUpdate
// Точная пара: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SHA-256 EXE: 21CDB7E22DE1DD9F4530C29ACEEA0E337E251BD082E0DF6674B4FD90050B2252
// SHA-256 PDB: FF4C7D2515C83D99253A9F518D7CAD7FE18BB060CA9BF934C8053024424F2E77
// Исходный владелец PDB: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::UpdateSave
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:373
// RVA: 0x00002840
// ADDRESS: 00402840
// PROTOTYPE: bool __thiscall UpdateSave(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::GetPackageForUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:463
// RVA: 0x00002930
// ADDRESS: 00402930
// PROTOTYPE: CPackage * __thiscall GetPackageForUpdate(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::~CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:75
// RVA: 0x00003640
// ADDRESS: 00403640
// PROTOTYPE: void __thiscall ~CClientResource(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:66
// RVA: 0x000038A0
// ADDRESS: 004038a0
// PROTOTYPE: void __thiscall CClientResource(eResourceType param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, HWND__ * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadExForAutoUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:292
// RVA: 0x00003990
// ADDRESS: 00403990
// PROTOTYPE: bool __thiscall LoadExForAutoUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::ResetPackInfosForAutoUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:318
// RVA: 0x00003B90
// ADDRESS: 00403b90
// PROTOTYPE: void __thiscall ResetPackInfosForAutoUpdate(list<tagPackFileInfo,std::allocator<tagPackFileInfo>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: __delayLoadHelper2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp
// RVA: 0x0002EEFA
// ADDRESS: 0042eefa
// PROTOTYPE: _func___cdecl_int * __cdecl __delayLoadHelper2(ImgDelayDescr * param_1, _func___cdecl_int * * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: ServerUpdate

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::GetPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:455
// RVA: 0x001D1EA0
// ADDRESS: 005d1ea0
// PROTOTYPE: CPackage * __thiscall GetPackage(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005d2008
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x001D2008
// ADDRESS: 005d2008
// PROTOTYPE: undefined Catch@005d2008()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::GetPackage
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:455
// RVA: 0x0004EC60
// ADDRESS: 0044ec60
// PROTOTYPE: CPackage * __thiscall GetPackage(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadPackage
// STATUS: PARTIALLY_IMPLEMENTED / RESULT_MAPPING_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:127
// RVA: 0x0004F5C0
// ADDRESS: 0044f5c0
// PROTOTYPE: bool __thiscall LoadPackage(bool param_1)
//
// `Package/<имя>.pak`, empty-name skip, sorted unique package IDs and one
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::IsFileExist
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:792
// RVA: 0x0004F750
// ADDRESS: 0044f750
// PROTOTYPE: bool __thiscall IsFileExist(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::FindFileList
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:1878
// RVA: 0x0004F9E0
// ADDRESS: 0044f9e0
// PROTOTYPE: void __thiscall FindFileList(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::~CClientResource
// STATUS: IMPLEMENTED / RAII_SUBSTITUTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:75
// RVA: 0x0004FFD0
// ADDRESS: 0044ffd0
// PROTOTYPE: void __thiscall ~CClientResource(void)
//
// `DefaultClientResourceOwner::replace_from_world_directory` завершает старый
// `InstalledClientResource` через `take` и `Drop` до публикации нового. В
// достигнутом World пути `ahThread` constructor-а остаётся null, а LoadEx
// синхронен; `FilesInfo`, `BTreeMap` пакетов и архивные bytes освобождаются
// Rust RAII без переноса Win32 wait/FILE*/STL-destruction plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::CClientResource
// STATUS: IMPLEMENTED / RAII_SUBSTITUTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:66
// RVA: 0x00050290
// ADDRESS: 00450290
// PROTOTYPE: undefined __thiscall CClientResource(eResourceType param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, HWND__ * param_4)
//
// cwd и `FilesInfo.ril`; current-folder хранится даже после failed LoadEx.
// `ClientResource` владеет только уже разобранными `FilesInfo` и пакетами,
// а `InstalledClientResource::resource == None` представляет созданный
// `CFilesInfo` с недоступным индексом. Это заменяет allocator/Win32 fields
// safe owner-ом, не меняя nullable query/loose fallback контракт.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadEx
// STATUS: PARTIALLY_IMPLEMENTED / RESULT_MAPPING_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:259
// RVA: 0x00050380
// ADDRESS: 00450380
// PROTOTYPE: bool __thiscall LoadEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d21e8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x000D21E8
// ADDRESS: 004d21e8
// PROTOTYPE: undefined Catch@004d21e8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d25a2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x000D25A2
// ADDRESS: 004d25a2
// PROTOTYPE: undefined Catch@004d25a2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00534fe0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x00134FE0
// ADDRESS: 00534fe0
// PROTOTYPE: undefined Unwind@00534fe0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
