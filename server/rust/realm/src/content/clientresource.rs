//! Жизненный цикл ресурсов World в Realm content: установка, чтение загрузчиками
//! и освобождение контекста. Привязки оригинала: World CGame::LoadServerResource/Release,
//! public/clientresource.cpp/.h; поиск skill-cache — CSkillFactory. Каталог и выбор
//! источника принадлежат Shared resources; публикация контекста и дисковый fallback
//! остаются у Realm; правила — docs/architecture/resources-and-configuration.md.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use nebokrai_shared::resources::{
    open_resource, resolve_resource_path, CRFile, ResourceCatalog, ResourceLoadError,
    ResourceOpenError, ResourcePackageLoad, ResourceSource,
};

/// Прежнее безусловное сообщение World; само по себе не доказывает успех загрузки.
pub const LOAD_SERVER_RESOURCE_SUCCESS_LOG: &[u8] = b"Load package file OK!";

/// Итог замены контекста из `CGame::LoadServerResource`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefaultClientResourceReplacement {
    Loaded {
        previous_owner_released: bool,
        packages: Vec<ResourcePackageLoad>,
        legacy_result: bool,
    },
    IndexUnavailable {
        previous_owner_released: bool,
        source: ResourceLoadError,
        legacy_result: bool,
    },
}

/// Владелец корня и каталога World. При отказе .ril новый корень остаётся
/// установленным для дискового чтения, а индекс отсутствует.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DefaultClientResourceOwner {
    installed: Option<InstalledClientResource>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstalledClientResource {
    root: PathBuf,
    resource: Option<ResourceCatalog>,
}

impl DefaultClientResourceOwner {
    /// Корень установленного контекста, в том числе после отказа загрузки индекса.
    pub fn root_directory(&self) -> Option<&Path> {
        self.installed
            .as_ref()
            .map(|installed| installed.root.as_path())
    }

    /// Заменяет контекст после получения корня, сохраняя прежний порядок освобождения.
    pub fn replace_from_world_directory(
        &mut self,
        root: &Path,
    ) -> DefaultClientResourceReplacement {
        // Старый контекст освобождается до установки нового, в том числе при отказе .ril.
        let previous = self.installed.take();
        let previous_owner_released = previous.is_some();
        drop(previous);
        self.installed = Some(InstalledClientResource {
            root: root.to_path_buf(),
            resource: None,
        });
        let load = ResourceCatalog::load_directory(root);
        match load {
            Ok(report) => {
                let packages = report.packages;
                self.installed
                    .as_mut()
                    .expect("новый default resource опубликован до LoadEx")
                    .resource = Some(report.catalog);
                log_package_loads(root, &packages);
                DefaultClientResourceReplacement::Loaded {
                    previous_owner_released,
                    packages,
                    // Прежний caller игнорирует bool LoadEx.
                    legacy_result: true,
                }
            }
            Err(source) => {
                tracing::warn!(root = %root.display(), ?source,
                    "Индекс ресурсов не загружен; доступна только дисковая ветвь");
                DefaultClientResourceReplacement::IndexUnavailable {
                    previous_owner_released,
                    source,
                    legacy_result: true,
                }
            }
        }
    }

    /// Открывает через Shared; совместимый интерфейс загрузчиков сохраняет Option.
    pub fn open(&self, path: &[u8]) -> Option<CRFile> {
        let context = self
            .installed
            .as_ref()
            .map(|installed| ResourceSource::new(&installed.root, installed.resource.as_ref()));
        match open_resource(path, context) {
            Ok(file) => Some(file),
            Err(ResourceOpenError::LooseFile { path, source }) => {
                if source.kind() == io::ErrorKind::NotFound {
                    tracing::debug!(path = %path.display(), %source, "Файл ресурса не найден");
                } else {
                    tracing::warn!(path = %path.display(), %source, "Не удалось открыть файл ресурса");
                }
                None
            }
            Err(error) => {
                tracing::warn!(
                    root = ?self.installed.as_ref().map(|installed| &installed.root),
                    path = %String::from_utf8_lossy(path),
                    ?error,
                    "Не удалось открыть ресурс из пакета"
                );
                None
            }
        }
    }

    /// Забирает весь ресурс из нового курсора. Буфер пакета передаётся
    /// загрузчику во владение; ошибка файла сохраняет прежний результат None.
    pub fn read_resource(&self, path: &[u8]) -> Option<Vec<u8>> {
        match self.open(path)?.into_bytes() {
            Ok(data) => Some(data),
            Err(error) => {
                tracing::warn!(
                    path = %String::from_utf8_lossy(path),
                    %error,
                    "Не удалось прочитать ресурс целиком"
                );
                None
            }
        }
    }

    /// None оставляет вызывающему дисковый поиск; пустой индексный список — Some.
    pub fn find_file_list(&self, root: &[u8], extension: &[u8]) -> Option<Vec<Vec<u8>>> {
        let resource = self.installed.as_ref()?.resource.as_ref()?;
        resource
            .is_file_exist(root)
            .then(|| resource.find_file_list(root, extension))
    }

    /// Skill-cache выбирает индекс только при наличии корня; иначе обходит SKILLS.
    pub fn find_cache_file_list(&self, extension: &[u8]) -> Vec<Vec<u8>> {
        const INDEX_ROOT: &[u8] = b"\\skills";
        if let Some(files) = self.find_file_list(INDEX_ROOT, extension) {
            return files;
        }
        let Some(installed) = self.installed.as_ref() else {
            return Vec::new();
        };
        let mut files = Vec::new();
        let directory = match resolve_resource_path(&installed.root.join("SKILLS")) {
            Ok(directory) => directory,
            Err(error) => {
                if error.kind() != io::ErrorKind::NotFound {
                    tracing::warn!(root = %installed.root.display(), %error,
                        "Не удалось найти каталог skill-cache");
                }
                return files;
            }
        };
        find_loose_cache_files(&directory, b".\\skills".to_vec(), extension, &mut files);
        files
    }

    /// Освобождает установленный контекст; сообщает, был ли он установлен.
    pub fn clear(&mut self) -> bool {
        self.installed.take().is_some()
    }
}

fn find_loose_cache_files(
    directory: &Path,
    wire_directory: Vec<u8>,
    extension: &[u8],
    files: &mut Vec<Vec<u8>>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(path = %directory.display(), %error,
                "Не удалось прочитать каталог skill-cache");
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(path = %directory.display(), %error,
                    "Не удалось прочитать запись каталога skill-cache");
                continue;
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                tracing::warn!(path = %entry.path().display(), %error,
                    "Не удалось определить тип записи skill-cache");
                continue;
            }
        };
        let mut name = entry
            .file_name()
            .to_string_lossy()
            .into_owned()
            .into_bytes();
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

fn log_package_loads(root: &Path, packages: &[ResourcePackageLoad]) {
    let mut loaded = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    for package in packages {
        match package {
            ResourcePackageLoad::Loaded { .. } => loaded += 1,
            ResourcePackageLoad::SkippedEmptyName { package_type } => {
                skipped += 1;
                tracing::debug!(root = %root.display(), package_type,
                    "Пропущено пустое имя пакета");
            }
            ResourcePackageLoad::OpenFailed {
                package_type,
                path,
                kind,
            } => {
                failed += 1;
                tracing::warn!(package_type, path = %path.display(), ?kind,
                    "Не удалось открыть пакет ресурсов");
            }
            ResourcePackageLoad::ReadFailed {
                package_type,
                path,
                kind,
            } => {
                failed += 1;
                tracing::warn!(package_type, path = %path.display(), ?kind,
                    "Не удалось прочитать пакет ресурсов");
            }
            ResourcePackageLoad::ParseFailed {
                package_type,
                path,
                source,
            } => {
                failed += 1;
                tracing::warn!(package_type, path = %path.display(), ?source,
                    "Не удалось разобрать пакет ресурсов");
            }
        }
    }
    tracing::info!(root = %root.display(), loaded, skipped, failed,
        "Загрузка каталога пакетов завершена");
}
