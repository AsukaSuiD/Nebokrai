//! IPv4-фильтр `kl_net::ip_filter` из `kl_ipfilter.cpp`.
//!
//! Нулевой октет правила является wildcard; allow-list принимает найденное
//! совпадение, deny-list — его отсутствие. Правила читаются как whitespace-
//! токены в исходном порядке, а `Vec<[u8; 4]>` сохраняет линейный поиск списка.
//! Разбор сохраняет `atoi`-подобный десятичный prefix и сужение до младшего
//! байта; переполнение безопасно насыщается в `i32`, поскольку исходная CRT/UB-
//! граница не имела подтверждённого сетевого результата.

use std::fs;
use std::io;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpFilterLoadError {
    #[error("не удалось прочитать список IP: {0}")]
    Io(#[source] io::Error),
    #[error("IP-token {token_index} не содержит ровно четыре части")]
    InvalidPattern { token_index: usize },
}

pub fn load_ip_patterns(path: impl AsRef<Path>) -> Result<Vec<[u8; 4]>, IpFilterLoadError> {
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

pub struct IpFilter<const ALLOW_MATCH: bool> {
    patterns: Vec<[u8; 4]>,
}

impl<const ALLOW_MATCH: bool> IpFilter<ALLOW_MATCH> {
    pub const fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn replace_patterns(&mut self, patterns: Vec<[u8; 4]>) {
        self.patterns = patterns;
    }

    pub fn is_allowed(&self, address: [u8; 4]) -> bool {
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
