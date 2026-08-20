//! Владелец сопоставления IPv4-шаблонов `kl_net::ip_filter` из
//! `authserver/src/kl_ipfilter.h`.
//!
//! Статус сопоставления одного уже разобранного IPv4: `VERIFIED_DISASSEMBLY`;
//! чтение корректного whitespace-списка canonical IPv4: `IMPLEMENTED`.
//!
//! Точная пара: `AuthServer/authserver.exe + AuthServer/authserver.pdb`;
//! SHA-256 EXE
//! `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`,
//! SHA-256 PDB
//! `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\server\authserver\src\kl_ipfilter.h`.
//!
//! Существенные RVA: `to_ip_struct` `0x00004180`,
//! `ip_filter<1>::Pred::operator()` `0x00004830`, allow-list `is_allowed`
//! `0x000056C0`, deny-list `is_allowed` `0x00005700`, `insert` `0x00005BE0`,
//! `load` `0x000061F0`, `load_ip_list` `0x00006420`.
//!
//! В `Pred::operator()` правило сравнивается с проверяемым адресом по четырём
//! octet: нулевой octet именно правила является wildcard, остальные должны
//! совпасть. Роли двух операндов, перепутанные сырым декомпилятом, точечно
//! подтверждены инструкциями Auth `0x0040487C..0x004048AC`: первый вызов
//! `to_ip_struct` получает элемент списка, второй — сохранённый адрес из
//! predicate, а `test cl, cl` пропускает сравнение нулевого octet правила.
//! `Vec<[u8; 4]>` заменяет `std::list<std::string>` после разбора и сохраняет
//! исходный линейный поиск. Const-параметр сохраняет две старые политики:
//! allow-list разрешает найденное совпадение, deny-list — его отсутствие.
//!
//! `load` читал whitespace-token до EOF и добавлял их в исходном порядке.
//! Canonical token разбирается узким байтовым compatibility-layer: ровно четыре
//! десятичных octet `0..255`, включая ведущие нули. `Ipv4Addr::from_str` здесь
//! не подходит, потому что его грамматика ведущих нулей не является старым
//! `atoi`-контрактом; отдельная parser-библиотека для четырёх bounded octet не
//! даёт совместимой гарантии лучше этого локального слоя.
//!
//! BLOCKED_MISSING_FACT: исходный `atoi` также принимал знак и числовой prefix,
//! а затем сужал результат до `unsigned char`; overflow является CRT-границей.
//! Неканонический token поэтому возвращает отдельную неизвестность, а не
//! автоматически выбранную реакцию оригинала. Ошибка чтения после успешного
//! открытия также не воспроизводит частично заполненный `std::list`.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Ошибка чтения либо неразрешённой CRT-границы списка IP-шаблонов.
#[derive(Debug)]
pub(crate) enum IpFilterLoadError {
    /// Файл не удалось прочитать целиком.
    Io(io::Error),
    /// Token не принадлежит доказанной canonical-грамматике.
    LegacyAtoiBoundaryUnknown {
        /// Номер token с нуля без раскрытия его байтов в ошибке.
        token_index: usize,
    },
}

impl fmt::Display for IpFilterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не удалось прочитать список IP: {error}"),
            Self::LegacyAtoiBoundaryUnknown { token_index } => write!(
                formatter,
                "IP-token {token_index} требует не восстановленной семантики MSVC atoi"
            ),
        }
    }
}

impl Error for IpFilterLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::LegacyAtoiBoundaryUnknown { .. } => None,
        }
    }
}

/// Читает canonical whitespace-список IP-шаблонов в исходном порядке.
pub(crate) fn load_ip_patterns(path: impl AsRef<Path>) -> Result<Vec<[u8; 4]>, IpFilterLoadError> {
    let input = fs::read(path).map_err(IpFilterLoadError::Io)?;
    input
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .enumerate()
        .map(|(token_index, token)| {
            parse_canonical_pattern(token)
                .ok_or(IpFilterLoadError::LegacyAtoiBoundaryUnknown { token_index })
        })
        .collect()
}

/// IPv4-фильтр над уже разобранными шаблонами старого `ip_filter`.
pub(crate) struct IpFilter<const ALLOW_MATCH: bool> {
    patterns: Vec<[u8; 4]>,
}

impl<const ALLOW_MATCH: bool> IpFilter<ALLOW_MATCH> {
    /// Создаёт пустой список шаблонов без неявного чтения файла.
    pub(crate) const fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Полностью заменяет список уже разобранными octet в исходном порядке.
    pub(crate) fn replace_patterns(&mut self, patterns: Vec<[u8; 4]>) {
        self.patterns = patterns;
    }

    /// Применяет старую wildcard-семантику и политику allow/deny варианта.
    pub(crate) fn is_allowed(&self, address: [u8; 4]) -> bool {
        let matched = self.patterns.iter().any(|pattern| {
            pattern
                .iter()
                .zip(address)
                .all(|(expected, actual)| *expected == 0 || *expected == actual)
        });
        matched == ALLOW_MATCH
    }
}

fn parse_canonical_pattern(token: &[u8]) -> Option<[u8; 4]> {
    let mut result = [0; 4];
    let mut parts = token.split(|byte| *byte == b'.');
    for octet in &mut result {
        let part = parts.next()?;
        if part.is_empty() || !part.iter().all(u8::is_ascii_digit) {
            return None;
        }
        *octet = part.iter().try_fold(0_u8, |value, digit| {
            value.checked_mul(10)?.checked_add(*digit - b'0')
        })?;
    }
    parts.next().is_none().then_some(result)
}
