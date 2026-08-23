//! Ограничения смены тела `CChangeBodyConf` из WorldServer, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Owner очищает vector до открытия XML, принимает direct `Goods` children
//! `RestrictionsGoodsList` и пишет signed count с `u32` items. Missing `index`
//! очищает результат; diagnostics сохраняют StringTable IDs `GS1148..1151`.
//! `quick-xml` заменяет TinyXML.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CChangeBodyConf {
    restrictions_goods: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChangeBodyLoadError {
    MissingRootOrGoods,
    MissingIndex,
}

impl ChangeBodyLoadError {
    pub(crate) const fn string_id(self) -> &'static [u8] {
        match self {
            Self::MissingRootOrGoods => b"GS1150",
            Self::MissingIndex => b"GS1151",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChangeBodySerializeError {
    CountOverflow,
}

impl CChangeBodyConf {
    pub(crate) fn clear(&mut self) {
        self.restrictions_goods.clear();
    }

    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        self.clear();
        let result = self.load_from_bytes_after_clear(source);
        if matches!(result, Err(ChangeBodyLoadError::MissingIndex)) {
            self.clear();
        }
        result
    }

    fn load_from_bytes_after_clear(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut goods_seen = false;

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(&start, depth, &mut root_seen, &mut goods_seen)?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(&empty, depth, &mut root_seen, &mut goods_seen)?;
                }
                Ok(Event::End(_)) => {
                    if depth == 0 {
                        return Err(ChangeBodyLoadError::MissingRootOrGoods);
                    }
                    depth -= 1;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(ChangeBodyLoadError::MissingRootOrGoods),
            }
            buffer.clear();
        }
        if root_seen && goods_seen && depth == 0 {
            Ok(())
        } else {
            Err(ChangeBodyLoadError::MissingRootOrGoods)
        }
    }

    fn process_start(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        goods_seen: &mut bool,
    ) -> Result<(), ChangeBodyLoadError> {
        let name = start.name();
        if !*root_seen {
            if name.as_ref() != b"RestrictionsGoodsList" {
                return Err(ChangeBodyLoadError::MissingRootOrGoods);
            }
            *root_seen = true;
        } else if depth == 1 && name.as_ref() == b"Goods" {
            let index = required_index(start)?;
            self.restrictions_goods.push(index);
            *goods_seen = true;
        }
        Ok(())
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ChangeBodySerializeError> {
        let count = i32::try_from(self.restrictions_goods.len())
            .map_err(|_| ChangeBodySerializeError::CountOverflow)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for &goods_id in &self.restrictions_goods {
            destination.extend_from_slice(&goods_id.to_le_bytes());
        }
        Ok(())
    }
}

fn required_index(start: &BytesStart<'_>) -> Result<u32, ChangeBodyLoadError> {
    let value = start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == b"index")
        .map(|attribute| attribute.value.into_owned())
        .ok_or(ChangeBodyLoadError::MissingIndex)?;
    Ok(legacy_atol(&value) as u32)
}

fn legacy_atol(value: &[u8]) -> i32 {
    let mut bytes = value.iter().copied().skip_while(u8::is_ascii_whitespace).peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut parsed = false;
    let mut result = 0_i32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    if parsed {
        if negative { result.saturating_neg() } else { result }
    } else {
        0
    }
}
