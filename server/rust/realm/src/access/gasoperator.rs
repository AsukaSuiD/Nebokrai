//! `CGasOperator::GetIP` из `gasoperator.cpp/.h`, подтверждённый точной
//! парой `loginserver.exe` и `loginserver.pdb` (RSDS match).
//! Singleton и ручное владение не влияли на результат и заменены чистой
//! функцией; `CGasOperator` как класс в публичных символах этой сборки не
//! находится — зафиксировано без адреса.

pub fn format_ipv4(raw: u32) -> Vec<u8> {
    let [first, second, third, fourth] = raw.to_le_bytes();
    format!("{first}.{second}.{third}.{fourth}").into_bytes()
}
