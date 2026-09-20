//! Общий token-scan `ReadTo` из `public/readwrite.cpp`.
//! Источники контракта — точные пары Auth/Billing/Login/World EXE/PDB.
//!
//! Все варианты читают whitespace tokens до точного marker-а и возвращают
//! `false` при EOF, stream error или точном `<end>`. Iterator и borrowed bytes
//! заменяют iostream/string, не интерпретируя кодировку и не проходя границу
//! `<end>`. World formatted `Read` остаётся отдельной typed границей.

/// Продвигает token-stream до точного маркера, но не проходит через `<end>`.
pub(crate) fn read_to<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>, expected: &[u8]) -> bool {
    for token in tokens {
        if token == expected {
            return true;
        }
        if token == b"<end>" {
            return false;
        }
    }
    false
}

// typed boundary: World `Read` сначала вызывал
// `ReadTo(stream, marker)`, затем выполнял `stream >> unsigned long`, но
// возвращал true независимо от успеха второго extraction:
// `if (ReadTo(stream, marker)) { stream >> value; return true; }`.
// До живого call site не выбираются Rust-реакция на malformed число и форма
// результата; отдельный generic parser ради неиспользуемой функции не создаётся.
