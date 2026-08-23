//! `CGasOperator::GetIP` из `gasoperator.cpp/.h`, подтверждённый
//! `loginserver.exe` и `loginserver.pdb`.
//! Singleton и ручное владение не влияли на результат и заменены чистой
//! функцией.

pub(crate) fn format_ipv4(raw: u32) -> Vec<u8> {
    let [first, second, third, fourth] = raw.to_le_bytes();
    format!("{first}.{second}.{third}.{fourth}").into_bytes()
}
