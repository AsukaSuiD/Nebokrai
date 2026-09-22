//! Нормализация путей по public/rfile.cpp, World CheckRFileStr RVA 0x0005AA20.
//! Привязка — в docs/architecture/resources-and-configuration.md.

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

/// Разрешает различие ASCII-регистра на диске. Точное имя имеет приоритет;
/// несколько подходящих имён вместо отсутствующего точного дают ошибку.
/// Это адаптер Linux, не изменение ключей индекса ресурсов.
pub fn resolve_resource_path(requested: &Path) -> io::Result<PathBuf> {
    match fs::metadata(requested) {
        Ok(_) => return Ok(requested.to_path_buf()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let mut resolved = PathBuf::new();
    for component in requested.components() {
        let Component::Normal(name) = component else {
            resolved.push(component.as_os_str());
            continue;
        };
        let candidate = resolved.join(name);
        match fs::metadata(&candidate) {
            Ok(_) => {
                resolved = candidate;
                continue;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let parent = if resolved.as_os_str().is_empty() {
            Path::new(".")
        } else {
            &resolved
        };
        let mut matched = None;
        for entry in fs::read_dir(parent)? {
            let entry = entry?;
            let entry_name = entry.file_name();
            if !entry_name
                .to_str()
                .zip(name.to_str())
                .is_some_and(|(found, expected)| found.eq_ignore_ascii_case(expected))
            {
                continue;
            }
            if matched.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Неоднозначный регистр пути ресурса: {}", candidate.display()),
                ));
            }
            matched = Some(entry.path());
        }
        resolved = matched.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Путь ресурса не найден: {}", candidate.display()),
            )
        })?;
    }
    Ok(resolved)
}

/// Повторяет побайтовую часть `CheckRFileStr`.
///
/// Оригинал приводил байты к нижнему регистру активной locale CRT. Внешний контракт
/// путей WorldServer подтверждён для ASCII; байты вне ASCII сохраняются, чтобы
/// не навязывать Rust Unicode-normalization. После замены `/` на `\\` функция
/// добавляет начальный `\\`, если первый обратный слеш не стоит на нулевой позиции.
pub fn normalize_resource_path(path: &mut Vec<u8>) {
    for byte in path.iter_mut() {
        byte.make_ascii_lowercase();
        if *byte == b'/' {
            *byte = b'\\';
        }
    }

    if path.first() != Some(&b'\\') {
        path.insert(0, b'\\');
    }
}
