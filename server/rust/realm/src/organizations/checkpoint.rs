//! Статический `CGame::CheckPoint` из `worldserver/game.cpp/.h`,
//! подтверждённый `worldserver.exe` и `worldserver.pdb`: удвоение одинарных
//! кавычек SQL-строкового литерала. Функция не читает состояние `CGame`,
//! поэтому перенесена в Realm `organizations/` как свободная операция над
//! bytes; её единственный сохранённый потребитель — вставка leave word в
//! `rsfaction` (`SaveLeaveWords` передавал `strContent` тому же `CheckPoint`).

/// Удваивает одинарные кавычки как исходный `CheckPoint`.
///
/// Вход уже является исходным видимым C-string prefix; отсутствие NUL в
/// конкретном fixed field проверяет его владелец до этого вызова.
pub fn check_point(input: &[u8]) -> Vec<u8> {
    let escaped_length = input
        .len()
        .saturating_add(input.iter().filter(|byte| **byte == b'\'').count());
    let mut escaped = Vec::with_capacity(escaped_length);
    for byte in input {
        escaped.push(*byte);
        if *byte == b'\'' {
            escaped.push(*byte);
        }
    }
    escaped
}
