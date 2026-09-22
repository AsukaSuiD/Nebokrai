//! Ошибки границ чтения и записи общего Rust-кодека.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyReadBlock {
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyWriteBlock {
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}
