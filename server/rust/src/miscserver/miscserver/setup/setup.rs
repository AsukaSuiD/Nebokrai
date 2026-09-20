//! Конфигурация `miscserver/setup/setup.cpp/.h`, подтверждённая
//! `miscserver.exe` и `miscserver.pdb`.
//!
//! `LoadIpPort` читает из `setup.ini` четыре позиционные пары: World IP и порт,
//! local bind IP и listen port. Имена слева игнорируются, строки остаются
//! byte-exact, порты сохраняют тип `unsigned short`. После успешного открытия
//! поздний stream fail оставляет прочитанный префикс и общий успех; ошибка
//! открытия не меняет прежнее состояние.
//! Неинициализированные поля оригинала представлены `None`. Процессный
//! singleton заменён единственным owned `CSetup`; файловый ввод и память
//! переданы стандартной библиотеке.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct IpPortSetup {
    world_ip: Option<Vec<u8>>,
    world_port: Option<u16>,
    local_ip: Option<Vec<u8>>,
    listen_port: Option<u16>,
}

impl IpPortSetup {
    pub(crate) fn world_ip(&self) -> Option<&[u8]> {
        self.world_ip.as_deref()
    }

    pub(crate) const fn world_port(&self) -> Option<u16> {
        self.world_port
    }

    pub(crate) fn local_ip(&self) -> Option<&[u8]> {
        self.local_ip.as_deref()
    }

    pub(crate) const fn listen_port(&self) -> Option<u16> {
        self.listen_port
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetupLoadReport {
    pub(crate) parsed_pairs: usize,
    pub(crate) stopped_at_pair: Option<usize>,
}

#[derive(Debug, Error)]
#[error("не удалось открыть MiscServer setup {}: {source}", path.display())]
pub(crate) struct SetupOpenError {
    pub(crate) path: PathBuf,
    #[source]
    pub(crate) source: io::Error,
}

#[derive(Debug, Default)]
pub(crate) struct CSetup {
    ip_port: IpPortSetup,
}

impl CSetup {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn ip_port(&self) -> &IpPortSetup {
        &self.ip_port
    }

    /// Выполняет `LoadIpPort` над явно переданным runtime-файлом.
    ///
    /// Ошибка открытия оставляет текущие значения неизменными. После успешного
    /// открытия поздний stream fail возвращается в отчёте и сохраняет prefix.
    pub(crate) fn load_ip_port(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<SetupLoadReport, SetupOpenError> {
        let requested = path.as_ref();
        let path = resolve_legacy_ascii_case(requested).unwrap_or_else(|| requested.to_path_buf());
        let bytes = fs::read(&path).map_err(|source| SetupOpenError {
            path: path.clone(),
            source,
        })?;
        Ok(self.parse_ip_port(&bytes))
    }

    pub(crate) fn load_setup(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<SetupLoadReport, SetupOpenError> {
        self.load_ip_port(path)
    }

    fn parse_ip_port(&mut self, bytes: &[u8]) -> SetupLoadReport {
        let mut tokens = SetupTokens::new(bytes);

        let Some(world_ip) = tokens.next_value() else {
            return tokens.report();
        };
        self.ip_port.world_ip = Some(world_ip.to_vec());
        tokens.parsed();

        let Some(world_port) = tokens.next_value().and_then(parse_u16) else {
            return tokens.report();
        };
        self.ip_port.world_port = Some(world_port);
        tokens.parsed();

        let Some(local_ip) = tokens.next_value() else {
            return tokens.report();
        };
        self.ip_port.local_ip = Some(local_ip.to_vec());
        tokens.parsed();

        let Some(listen_port) = tokens.next_value().and_then(parse_u16) else {
            return tokens.report();
        };
        self.ip_port.listen_port = Some(listen_port);
        tokens.parsed();
        tokens.report()
    }
}

struct SetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> SetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    fn report(&self) -> SetupLoadReport {
        SetupLoadReport {
            parsed_pairs: self.parsed_pairs,
            stopped_at_pair: (self.parsed_pairs < self.attempted_pairs)
                .then_some(self.attempted_pairs),
        }
    }
}

fn parse_u16(raw: &[u8]) -> Option<u16> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn resolve_legacy_ascii_case(requested: &Path) -> Option<PathBuf> {
    if requested.is_file() {
        return Some(requested.to_path_buf());
    }
    let parent = requested.parent().unwrap_or_else(|| Path::new("."));
    let name = requested.file_name()?.to_str()?;
    fs::read_dir(parent).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let entry_name = entry.file_name();
        let entry_name = entry_name.to_str()?;
        (entry_name.eq_ignore_ascii_case(name) && entry.file_type().ok()?.is_file())
            .then(|| entry.path())
    })
}
