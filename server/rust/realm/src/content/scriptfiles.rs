//! Поиск файлов сценариев для Realm content. Основа — FindScriptFile
//! из worldserver/game.cpp/.h; обход ОС и явный корень — инфраструктура Rust.
//! Правила путей и ограничения: docs/gameplay/scripting.md.

use std::{
    io,
    path::{Path, PathBuf},
};

use nebokrai_shared::resources::resolve_resource_path;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct ScriptFileScan {
    pub files: Vec<Vec<u8>>,
    pub errors: Vec<ScriptFileScanError>,
}

#[derive(Debug)]
pub enum ScriptFileScanError {
    Root {
        path: PathBuf,
        source: io::Error,
    },
    Walk(walkdir::Error),
    OutsideRoot {
        path: PathBuf,
    },
}

/// Обходит диск относительно корня владельца. Возвращает логические пути,
/// пригодные для read_resource и передачи Game, без добавленного host-префикса.
pub fn find_script_files(
    resource_root: &Path,
    pattern: &[u8],
    extension: &[u8],
) -> ScriptFileScan {
    let pattern = c_string_prefix(pattern)
        .iter()
        .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
        .collect::<Vec<_>>();
    let logical_root = legacy_path_from_bytes(script_search_root(&pattern));
    let requested_root = resource_root.join(&logical_root);
    let mut report = ScriptFileScan::default();
    let disk_root = match resolve_resource_path(&requested_root) {
        Ok(root) => root,
        Err(source) => {
            report.errors.push(ScriptFileScanError::Root {
                path: requested_root,
                source,
            });
            return report;
        }
    };
    let requested_extension = c_string_prefix(extension)
        .strip_prefix(b".")
        .unwrap_or_else(|| c_string_prefix(extension));

    for entry in WalkDir::new(&disk_root)
        .min_depth(1)
        .follow_links(false)
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report.errors.push(ScriptFileScanError::Walk(error));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(extension) = entry.path().extension() else {
            continue;
        };
        if !legacy_path_component_bytes(extension).eq_ignore_ascii_case(requested_extension) {
            continue;
        }
        let relative = match entry.path().strip_prefix(&disk_root) {
            Ok(relative) => relative,
            Err(_) => {
                report.errors.push(ScriptFileScanError::OutsideRoot {
                    path: entry.path().to_path_buf(),
                });
                continue;
            }
        };
        // Регистр дискового корня не должен менять ключ scripts/... у Game.
        let mut path = legacy_path_bytes(&logical_root.join(relative));
        for byte in &mut path {
            if *byte == b'\\' {
                *byte = b'/';
            } else {
                byte.make_ascii_lowercase();
            }
        }
        report.files.push(path);
    }
    report.files.sort_unstable();
    report
}

fn c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn script_search_root(pattern: &[u8]) -> &[u8] {
    let wildcard = pattern.iter().position(|byte| matches!(*byte, b'*' | b'?'));
    let parent_end = wildcard
        .and_then(|position| pattern[..position].iter().rposition(|byte| *byte == b'/'))
        .or_else(|| pattern.iter().rposition(|byte| *byte == b'/'));
    match parent_end {
        Some(0) => b"/",
        Some(end) => &pattern[..end],
        None => b".",
    }
}

#[cfg(unix)]
fn legacy_path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn legacy_path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(unix)]
fn legacy_path_component_bytes(value: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn legacy_path_component_bytes(value: &std::ffi::OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

#[cfg(unix)]
fn legacy_path_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn legacy_path_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}
