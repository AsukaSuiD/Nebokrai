//! Общий последовательный codec бинарных форматов GameServer.
//!
//! `bytes::Buf` и `BufMut` отвечают только за безопасное продвижение курсора,
//! проверку границ и little-endian primitives. Правила конкретных ресурсов —
//! signed count, допустимые хвосты, обязательность NUL, wrapping и собственные
//! ошибки — остаются в owner-декодерах. При ошибке чтения позиция не меняется.

use bytes::{Buf, BufMut};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LegacyReadBlock {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LegacyWriteBlock {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

pub(crate) struct LegacyReader<'source> {
    source: &'source [u8],
    cursor: usize,
}

impl<'source> LegacyReader<'source> {
    pub(crate) const fn new(source: &'source [u8]) -> Self {
        Self { source, cursor: 0 }
    }

    pub(crate) fn at(source: &'source [u8], cursor: usize) -> Result<Self, LegacyReadBlock> {
        if cursor > source.len() {
            return Err(LegacyReadBlock {
                offset: cursor,
                needed: 0,
                available: 0,
            });
        }
        Ok(Self { source, cursor })
    }

    pub(crate) const fn position(&self) -> usize {
        self.cursor
    }

    pub(crate) fn remaining(&self) -> usize {
        self.source.len().saturating_sub(self.cursor)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, LegacyReadBlock> {
        self.read_primitive(1, |source| source.try_get_u8())
    }

    pub(crate) fn read_i8(&mut self) -> Result<i8, LegacyReadBlock> {
        self.read_primitive(1, |source| source.try_get_i8())
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, LegacyReadBlock> {
        self.read_primitive(2, |source| source.try_get_u16_le())
    }

    pub(crate) fn read_i16(&mut self) -> Result<i16, LegacyReadBlock> {
        self.read_primitive(2, |source| source.try_get_i16_le())
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, LegacyReadBlock> {
        self.read_primitive(4, |source| source.try_get_u32_le())
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32, LegacyReadBlock> {
        self.read_primitive(4, |source| source.try_get_i32_le())
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, LegacyReadBlock> {
        self.read_primitive(8, |source| source.try_get_u64_le())
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64, LegacyReadBlock> {
        self.read_primitive(8, |source| source.try_get_i64_le())
    }

    pub(crate) fn read_u8_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u8, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u8)
    }

    pub(crate) fn read_i8_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i8, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i8)
    }

    pub(crate) fn read_u16_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u16, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u16)
    }

    pub(crate) fn read_i16_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i16, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i16)
    }

    pub(crate) fn read_u32_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u32, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u32)
    }

    pub(crate) fn read_i32_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i32, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i32)
    }

    pub(crate) fn read_u64_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<u64, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_u64)
    }

    pub(crate) fn read_i64_from(
        source: &'source [u8],
        cursor: &mut usize,
    ) -> Result<i64, LegacyReadBlock> {
        Self::read_from(source, cursor, Self::read_i64)
    }

    pub(crate) fn read_bytes(&mut self, length: usize) -> Result<&'source [u8], LegacyReadBlock> {
        self.ensure(length)?;
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.source[start..self.cursor])
    }

    /// Читает C-string не длиннее `maximum` байт вместе с обязательным NUL.
    /// Возвращаемое значение не содержит terminator.
    pub(crate) fn read_c_string(
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

pub(crate) struct LegacyWriter<'destination> {
    destination: &'destination mut Vec<u8>,
}

impl<'destination> LegacyWriter<'destination> {
    pub(crate) fn new(destination: &'destination mut Vec<u8>) -> Self {
        Self { destination }
    }

    pub(crate) fn position(&self) -> usize {
        self.destination.len()
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.destination.put_u8(value);
    }

    pub(crate) fn write_i8(&mut self, value: i8) {
        self.destination.put_i8(value);
    }

    pub(crate) fn write_u16(&mut self, value: u16) {
        self.destination.put_u16_le(value);
    }

    pub(crate) fn write_i16(&mut self, value: i16) {
        self.destination.put_i16_le(value);
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        self.destination.put_u32_le(value);
    }

    pub(crate) fn write_i32(&mut self, value: i32) {
        self.destination.put_i32_le(value);
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        self.destination.put_u64_le(value);
    }

    pub(crate) fn write_i64(&mut self, value: i64) {
        self.destination.put_i64_le(value);
    }

    pub(crate) fn write_bytes(&mut self, value: &[u8]) {
        self.destination.put_slice(value);
    }

    pub(crate) fn write_c_string(&mut self, value: &[u8]) {
        let visible = value.split(|byte| *byte == 0).next().unwrap_or_default();
        self.destination.put_slice(visible);
        self.destination.put_u8(0);
    }

    pub(crate) fn write_u16_at(
        destination: &mut [u8],
        offset: usize,
        value: u16,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 2)?;
        target.put_u16_le(value);
        Ok(())
    }

    pub(crate) fn write_i16_at(
        destination: &mut [u8],
        offset: usize,
        value: i16,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 2)?;
        target.put_i16_le(value);
        Ok(())
    }

    pub(crate) fn write_u32_at(
        destination: &mut [u8],
        offset: usize,
        value: u32,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 4)?;
        target.put_u32_le(value);
        Ok(())
    }

    pub(crate) fn write_i32_at(
        destination: &mut [u8],
        offset: usize,
        value: i32,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 4)?;
        target.put_i32_le(value);
        Ok(())
    }
}

fn fixed_target(
    destination: &mut [u8],
    offset: usize,
    needed: usize,
) -> Result<&mut [u8], LegacyWriteBlock> {
    let available = destination.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(needed) else {
        return Err(LegacyWriteBlock {
            offset,
            needed,
            available,
        });
    };
    destination.get_mut(offset..end).ok_or(LegacyWriteBlock {
        offset,
        needed,
        available,
    })
}
