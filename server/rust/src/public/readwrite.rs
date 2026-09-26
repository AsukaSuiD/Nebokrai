//! Общий token-scan `ReadTo` из `public/readwrite.cpp`.
//! Источники контракта — точные пары Auth/Billing/Login/World EXE/PDB.
//!
//! Все варианты читают whitespace tokens до точного marker-а и возвращают
//! `false` при EOF, stream error или точном `<end>`. Iterator и borrowed bytes
//! заменяют iostream/string, не интерпретируя кодировку и не проходя границу
//! `<end>`. World formatted `Read` остаётся отдельной typed границей.
//!
//! Последним потребителем был positional-парсинг World setup; с его переездом
//! в Realm `app/world_setup` (волна C5-A) старый пакет `read_to_marker`
//! больше не переиздаёт — владелец формы в Shared остаётся единственным.

// typed boundary: World `Read` сначала вызывал
// `ReadTo(stream, marker)`, затем выполнял `stream >> unsigned long`, но
// возвращал true независимо от успеха второго extraction:
// `if (ReadTo(stream, marker)) { stream >> value; return true; }`.
// До живого call site не выбираются Rust-реакция на malformed число и форма
// результата; отдельный generic parser ради неиспользуемой функции не создаётся.
