//! Ограничения начисления honor за убийство в историческом Miracle.
//!
//! Контракт World `HonorElimilateConfig::LoadConfig` и
//! `AddToByteArray`:; singleton lifecycle и
//! Game decoder не входят в этот owner и остаются. Точная пара:
//! Исходный owner PDB:
//!
//! Оригинал World serializer и Game decoder подтверждают единственный wire:
//! `level_difference`, затем `minimum_level`, оба signed little-endian `long`.
//! Отдельные таблицы `CHonorRanks` сюда не входят и отправляются следующими
//! subtype `0x27/0x28`. Два typed `i32` заменяют process-global singleton без
//! изменения payload; неизвестный legacy return loader-а не выдумывается.
//!
//! После успешного открытия formatted extraction читает два `label + long`
//! поля и всегда возвращает success: обрыв/некорректное число сохраняют уже
//! присвоенный scalar, не становясь новой failure-веткой. Missing resource
//! state не меняет; caller заменяет только `MessageBoxA` operator notice.

/// Восьмибайтовый wire-value вместо singleton `HonorElimilateConfig`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HonorElimilateConfig {
    pub(crate) level_difference: i32,
    pub(crate) minimum_level: i32,
}

impl HonorElimilateConfig {
 /// Оригинал successful `LoadConfig` branch после resource-open.
    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) {
        let mut tokens = source
            .split(|byte: &u8| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        let _ = tokens.next();
        if let Some(value) = tokens.next().and_then(parse_legacy_i32) {
            self.level_difference = value;
        }
        let _ = tokens.next();
        if let Some(value) = tokens.next().and_then(parse_legacy_i32) {
            self.minimum_level = value;
        }
    }

    pub(crate) fn add_to_byte_array(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.level_difference.to_le_bytes());
        destination.extend_from_slice(&self.minimum_level.to_le_bytes());
    }
}

/// `operator>>(long)` принимает десятичный token целиком; overflow/failure
/// не присваивают target, в отличие от `_atol`-семантики других INI owner-ов.
fn parse_legacy_i32(token: &[u8]) -> Option<i32> {
    std::str::from_utf8(token).ok()?.parse().ok()
}
