//! Собственный Rust-механизм последовательного чтения бинарных полей.
//!
//! bytes::Buf выполняет чтение little-endian-примитивов. При ошибке курсор
//! остаётся прежним. Count, допустимые хвосты и частичные эффекты составного
//! формата определяет вызывающий декодер.

use bytes::Buf;

use super::LegacyReadBlock;

pub struct LegacyReader<'source> {
    source: &'source [u8],
    cursor: usize,
}

impl<'source> LegacyReader<'source> {
    pub const fn new(source: &'source [u8]) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn at(source: &'source [u8], cursor: usize) -> Result<Self, LegacyReadBlock> {
        if cursor > source.len() {
            return Err(LegacyReadBlock {
                offset: cursor,
                needed: 0,
                available: 0,
            });
        }
        Ok(Self { source, cursor })
    }

    pub const fn position(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.source.len().saturating_sub(self.cursor)
    }

    pub fn read_u8(&mut self) -> Result<u8, LegacyReadBlock> {
        self.read_primitive(1, |source| source.try_get_u8())
    }

    pub fn read_i8(&mut self) -> Result<i8, LegacyReadBlock> {
        self.read_primitive(1, |source| source.try_get_i8())
    }

    pub fn read_u16(&mut self) -> Result<u16, LegacyReadBlock> {
        self.read_primitive(2, |source| source.try_get_u16_le())
    }

    pub fn read_i16(&mut self) -> Result<i16, LegacyReadBlock> {
        self.read_primitive(2, |source| source.try_get_i16_le())
    }

    pub fn read_u32(&mut self) -> Result<u32, LegacyReadBlock> {
        self.read_primitive(4, |source| source.try_get_u32_le())
    }

    pub fn read_i32(&mut self) -> Result<i32, LegacyReadBlock> {
        self.read_primitive(4, |source| source.try_get_i32_le())
    }

    pub fn read_u64(&mut self) -> Result<u64, LegacyReadBlock> {
        self.read_primitive(8, |source| source.try_get_u64_le())
    }

    pub fn read_i64(&mut self) -> Result<i64, LegacyReadBlock> {
        self.read_primitive(8, |source| source.try_get_i64_le())
    }

    pub fn read_u8_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u8, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u8)
    }

    pub fn read_i8_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i8, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i8)
    }

    pub fn read_u16_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u16, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u16)
    }

    pub fn read_i16_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i16, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i16)
    }

    pub fn read_u32_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u32, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u32)
    }

    pub fn read_i32_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i32, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i32)
    }

    pub fn read_u64_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u64, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u64)
    }

    pub fn read_i64_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i64, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i64)
    }

    pub fn read_bytes_from(
        source: &'source [u8],
        cursor: &mut usize,
        length: usize,
    ) -> Result<&'source [u8], LegacyReadBlock> {
        Self::read_from(source, cursor, |reader| reader.read_bytes(length))
    }

    pub fn read_c_string_from(
        source: &'source [u8],
        cursor: &mut usize,
        maximum: usize,
    ) -> Result<&'source [u8], LegacyReadBlock> {
        Self::read_from(source, cursor, |reader| reader.read_c_string(maximum))
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&'source [u8], LegacyReadBlock> {
        self.ensure(length)?;
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.source[start..self.cursor])
    }

    /// Читает C-string не длиннее `maximum` байт вместе с обязательным NUL.
    /// Возвращаемое значение не содержит terminator.
    pub fn read_c_string(
        &mut self,
        maximum: usize,
    ) -> Result<&'source [u8], LegacyReadBlock> {
        let available = self.remaining().min(maximum);
        let tail = &self.source[self.cursor..self.cursor + available];
        let Some(length) = tail.iter().position(|byte| *byte == 0) else {
            return Err(LegacyReadBlock {
                offset: self.cursor,
                needed: available.saturating_add(1),
                available,
            });
        };
        let start = self.cursor;
        self.cursor += length + 1;
        Ok(&self.source[start..start + length])
    }

    fn ensure(&self, needed: usize) -> Result<(), LegacyReadBlock> {
        let available = self.remaining();
        if available < needed {
            return Err(LegacyReadBlock {
                offset: self.cursor,
                needed,
                available,
            });
        }
        Ok(())
    }

    fn read_primitive<T>(
        &mut self,
        width: usize,
        read: impl FnOnce(&mut &'source [u8]) -> Result<T, bytes::TryGetError>,
    ) -> Result<T, LegacyReadBlock> {
        self.ensure(width)?;
        let mut source = &self.source[self.cursor..];
        let value = read(&mut source).map_err(|_| LegacyReadBlock {
            offset: self.cursor,
            needed: width,
            available: self.remaining(),
        })?;
        self.cursor += width;
        Ok(value)
    }

    fn read_from<T>(
        source: &'source [u8],
        cursor: &mut usize,
        read: impl FnOnce(&mut Self) -> Result<T, LegacyReadBlock>,
    ) -> Result<T, LegacyReadBlock> {
        let mut reader = Self::at(source, *cursor)?;
        let value = read(&mut reader)?;
        *cursor = reader.position();
        Ok(value)
    }
}
