//! Порог honor за убийство `HonorElimilateConfig` из WorldServer,
//! подтверждённый `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire содержит два signed `i32`: level difference и minimum level.
//! После успешного открытия loader позиционно читает две пары label/value и
//! сохраняет уже присвоенное поле при повреждённом хвосте; missing resource
//! не меняет state. Honor rank tables отправляются отдельными subtypes.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HonorElimilateConfig {
    pub(crate) level_difference: i32,
    pub(crate) minimum_level: i32,
}

impl HonorElimilateConfig {
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
