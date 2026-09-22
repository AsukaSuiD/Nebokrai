//! Поиск маркера в потоке текстовых токенов ресурсов.
//! Исходный helper: `public/readwrite.cpp/.h`, `ReadTo`.

/// Продвигает поток до точного маркера, но не проходит через `<end>`.
pub fn read_to_marker<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>, expected: &[u8]) -> bool {
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
