//! Собственный Rust-механизм записи бинарных полей.
//!
//! Последовательная запись дополняет Vec; запись по смещению проверяет
//! границы заданного среза. Правила конкретного формата остаются у вызывающего.

use bytes::BufMut;

use super::LegacyWriteBlock;

pub struct LegacyWriter<'destination> {
    destination: &'destination mut Vec<u8>,
}

impl<'destination> LegacyWriter<'destination> {
    pub fn new(destination: &'destination mut Vec<u8>) -> Self {
        Self { destination }
    }

    pub fn position(&self) -> usize {
        self.destination.len()
    }

    pub fn destination_mut(&mut self) -> &mut Vec<u8> {
        self.destination
    }

    pub fn write_u8(&mut self, value: u8) {
        self.destination.put_u8(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.destination.put_i8(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.destination.put_u16_le(value);
    }

    pub fn write_i16(&mut self, value: i16) {
        self.destination.put_i16_le(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.destination.put_u32_le(value);
    }

    pub fn write_i32(&mut self, value: i32) {
        self.destination.put_i32_le(value);
    }

    pub fn write_u64(&mut self, value: u64) {
        self.destination.put_u64_le(value);
    }

    pub fn write_i64(&mut self, value: i64) {
        self.destination.put_i64_le(value);
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.destination.put_slice(value);
    }

    pub fn write_c_string(&mut self, value: &[u8]) {
        let visible = value.split(|byte| *byte == 0).next().unwrap_or_default();
        self.destination.put_slice(visible);
        self.destination.put_u8(0);
    }

    pub fn write_u16_at(
        destination: &mut [u8],
        offset: usize,
        value: u16,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 2)?;
        target.put_u16_le(value);
        Ok(())
    }

    pub fn write_i16_at(
        destination: &mut [u8],
        offset: usize,
        value: i16,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 2)?;
        target.put_i16_le(value);
        Ok(())
    }

    pub fn write_u32_at(
        destination: &mut [u8],
        offset: usize,
        value: u32,
    ) -> Result<(), LegacyWriteBlock> {
        let mut target = fixed_target(destination, offset, 4)?;
        target.put_u32_le(value);
        Ok(())
    }

    pub fn write_i32_at(
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
