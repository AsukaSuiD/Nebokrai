//! Владелец сопоставления IPv4-шаблонов `kl_net::ip_filter` AuthServer.
//!
//! Сопоставление IPv4 и чтение whitespace-списка canonical адресов подтверждены
//! точной парой AuthServer EXE/PDB.
//!
//! В `Pred::operator()` правило сравнивается с проверяемым адресом по четырём
//! octet: нулевой octet именно правила является wildcard, остальные должны
//! совпасть. `to_ip_struct` получает элемент списка, второй операнд —
//! сохранённый адрес из
//! predicate, а `test cl, cl` пропускает сравнение нулевого octet правила.
//! `Vec<[u8; 4]>` заменяет `std::list<std::string>` после разбора и сохраняет
//! исходный линейный поиск. Const-параметр сохраняет две старые политики:
//! allow-list разрешает найденное совпадение, deny-list — его отсутствие.
//!
//! `load` читал whitespace-token до EOF и добавлял их в исходном порядке.
//! Token разбирается узким байтовым compatibility-layer: ровно четыре части,
//! каждая принимает знак и десятичный prefix как `atoi`, затем безопасно
//! сужается до младшего byte. Переполнение насыщается в signed `i32` до
//! сужения: это детерминированная замена внутренней CRT/UB-границы, не имеющей
//! необходимого сетевого эффекта. Ошибка чтения после успешного открытия также
//! не воспроизводит частично заполненный `std::list`.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Ошибка чтения либо структурно некорректной записи списка IP-шаблонов.
#[derive(Debug)]
pub(crate) enum IpFilterLoadError {
    /// Файл не удалось прочитать целиком.
    Io(io::Error),
    /// Token содержит не четыре разделённые точками части.
    InvalidPattern {
        /// Номер token с нуля без раскрытия его байтов в ошибке.
        token_index: usize,
    },
}

impl fmt::Display for IpFilterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не удалось прочитать список IP: {error}"),
            Self::InvalidPattern { token_index } => write!(
                formatter,
                "IP-token {token_index} не содержит ровно четыре части"
            ),
        }
    }
}

impl Error for IpFilterLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidPattern { .. } => None,
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
            parse_legacy_pattern(token).ok_or(IpFilterLoadError::InvalidPattern { token_index })
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

fn parse_legacy_pattern(token: &[u8]) -> Option<[u8; 4]> {
    let mut result = [0; 4];
    let mut parts = token.split(|byte| *byte == b'.');
    for octet in &mut result {
        let part = parts.next()?;
        *octet = legacy_atoi(part) as u8;
    }
    parts.next().is_none().then_some(result)
}

fn legacy_atoi(input: &[u8]) -> i32 {
    let mut cursor = 0;
    while input.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    let negative = match input.get(cursor) {
        Some(b'-') => {
            cursor += 1;
            true
        }
        Some(b'+') => {
            cursor += 1;
            false
        }
        _ => false,
    };
    let mut magnitude = 0_i64;
    let mut has_digit = false;
    while let Some(digit) = input.get(cursor).filter(|byte| byte.is_ascii_digit()) {
        has_digit = true;
        magnitude = magnitude
            .saturating_mul(10)
            .saturating_add(i64::from(*digit - b'0'));
        cursor += 1;
    }
    if !has_digit {
        return 0;
    }
    let signed = if negative { -magnitude } else { magnitude };
    signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
