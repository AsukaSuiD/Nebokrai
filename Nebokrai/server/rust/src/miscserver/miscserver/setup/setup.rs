//! Конфигурация соединения MiscServer из `miscserver/setup/setup.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED` для `CSetup::GetInstance` RVA `0x00001000`,
//! `CSetup::CSetup` RVA `0x0000BD50`, `LoadIpPort` RVA `0x000105D0` и
//! `LoadSetup` RVA `0x00010760`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\server\miscserver\miscserver\setup\setup.cpp`
//! и `.h`.
//!
//! `LoadIpPort` открывал ровно `setup.ini` и последовательно извлекал четыре
//! пары `label value`: World IP, World port, local bind IP и listen port.
//! Labels не проверялись. Строки остаются byte-exact, оба порта — исходные
//! `unsigned short`. Успешный open давал `true` даже после позднего stream
//! fail, сохраняя уже записанный prefix; Rust возвращает тот же partial
//! snapshot отдельным отчётом. Ошибка открытия не меняет прежнее состояние.
//!
//! Constructor записывал только vtable и не задавал полям `IP_PORT` defaults.
//! Поэтому Rust хранит каждое ещё не прочитанное поле как `None`, а не
//! придумывает ноль либо пустой адрес. Process-global lazy `GetInstance`
//! заменён обычным owned `CSetup`: единственный будущий `CGame` получает тот
//! же единственный экземпляр без global mutable state и ручного `new`.
//!
//! Найденный read-only fixture `MiscServer/setup.ini` имеет SHA-256
//! `9a1bc4280961d49bf785dbb68d37209f7fa2e5defd7fce7b3e0150ecef3de1c5`
//! и содержит все четыре коротких корректных значения. Точный размер старых
//! destination `char[]` не присутствует в текущем raw-export; реакция
//! operator `>>` на слишком длинный внешний token могла быть переполнением и
//! не воспроизводится через `unsafe`. Owned `Vec<u8>` доказывает обычный путь,
//! но не объявляет malformed long-token совместимым поведением.
//!
//! `std::vector<unsigned char>::resize`, два `$L` unwind-funclet, allocator,
//! iostream internals и singleton allocation удалены как library/compiler
//! plumbing. Их существенный эффект выражен owned bytes, `fs::read` и `Drop`.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Четыре значения исходного `IP_PORT`, ещё не прочитанные после constructor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct IpPortSetup {
    world_ip: Option<Vec<u8>>,
    world_port: Option<u16>,
    local_ip: Option<Vec<u8>>,
    listen_port: Option<u16>,
}

impl IpPortSetup {
    /// Возвращает byte-exact адрес WorldServer либо исходное отсутствие value.
    pub(crate) fn world_ip(&self) -> Option<&[u8]> {
        self.world_ip.as_deref()
    }

    /// Возвращает исходный WorldServer port.
    pub(crate) const fn world_port(&self) -> Option<u16> {
        self.world_port
    }

    /// Возвращает byte-exact local bind IPv4 либо исходное отсутствие value.
    pub(crate) fn local_ip(&self) -> Option<&[u8]> {
        self.local_ip.as_deref()
    }

    /// Возвращает port, который MiscServer сообщает WorldServer при connect.
    pub(crate) const fn listen_port(&self) -> Option<u16> {
        self.listen_port
    }
}

/// Итог одного успешного открытия positional Misc `setup.ini`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetupLoadReport {
    /// Число полностью применённых пар в диапазоне `0..=4`.
    pub(crate) parsed_pairs: usize,
    /// Первая недочитанная либо malformed пара; `None` означает полный файл.
    pub(crate) stopped_at_pair: Option<usize>,
}

/// Ошибка открытия исходного Misc `setup.ini` без раскрытия его содержимого.
#[derive(Debug)]
pub(crate) struct SetupOpenError {
    pub(crate) path: PathBuf,
    pub(crate) source: io::Error,
}

impl fmt::Display for SetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не удалось открыть MiscServer setup {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl Error for SetupOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Owned форма единственного исходного singleton `CSetup`.
#[derive(Debug, Default)]
pub(crate) struct CSetup {
    ip_port: IpPortSetup,
}

impl CSetup {
    /// Создаёт состояние без придуманных constructor-defaults.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Возвращает текущий partial либо полный снимок `m_IpPort`.
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

    /// Сохраняет прямое делегирование исходного `LoadSetup` в `LoadIpPort`.
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
