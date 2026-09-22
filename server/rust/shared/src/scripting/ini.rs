//! Чтение списков по public/ini.cpp/.h, memory-ветвь CIni Game.
//! RVA 0xc4d50–0xc53c7; CRT atoi 0x21bacb.
//! Общие правила FunctionList/VariableList — docs/gameplay/scripting.md.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IniListErrorKind {
    MissingSection,
    UnexpectedEnd,
    CaptionTooLong,
    MissingValueSeparator,
}

/// Смещение в полученном тексте; ошибки ограничивают небезопасные ветви оригинала.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IniListError {
    pub offset: usize,
    pub kind: IniListErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IniRecord<'a> {
    pub offset: usize,
    pub caption: &'a [u8],
    pub value: Option<&'a [u8]>,
}

/// Последовательный проход вместо повторного поиска секции для каждой записи CIni.
pub(super) struct IniRecords<'a> {
    source: &'a [u8],
    offset: usize,
    finished: bool,
}

impl<'a> IniRecords<'a> {
    pub fn new(source: &'a [u8], section: &[u8]) -> Result<Self, IniListError> {
        let source = source.split(|b| *b == 0).next().unwrap_or_default();
        for (offset, byte) in source.iter().enumerate() {
            if *byte == b'['
                && (offset == 0 || source[offset - 1] == b'\n')
                && read_text(&source[offset + 1..]) == section
            {
                let after_header = source[offset..]
                    .iter()
                    .position(|b| *b == b'\n')
                    .map(|end| offset + end + 1)
                    .ok_or(IniListError {
                        offset,
                        kind: IniListErrorKind::UnexpectedEnd,
                    })?;
                if after_header == source.len() {
                    return Err(IniListError {
                        offset: after_header,
                        kind: IniListErrorKind::UnexpectedEnd,
                    });
                }
                return Ok(Self {
                    source,
                    offset: after_header,
                    finished: false,
                });
            }
        }
        Err(IniListError {
            offset: 0,
            kind: IniListErrorKind::MissingSection,
        })
    }

    fn record(&self) -> Result<IniRecord<'a>, IniListError> {
        let tail = &self.source[self.offset..];
        let caption_end = tail
            .iter()
            .position(|b| matches!(*b, b'\r' | b'=' | b';'))
            .unwrap_or(tail.len());
        // ReadDataName выделяет 64 байта и не проверяет выход за границу.
        if caption_end >= 64 {
            return Err(IniListError {
                offset: self.offset,
                kind: IniListErrorKind::CaptionTooLong,
            });
        }
        let value = tail
            .iter()
            .position(|b| matches!(*b, b'=' | b'\r'))
            .filter(|at| tail[*at] == b'=')
            .map(|at| read_text(&tail[at + 1..]));
        Ok(IniRecord {
            offset: self.offset,
            caption: &tail[..caption_end],
            value,
        })
    }
}

impl<'a> Iterator for IniRecords<'a> {
    type Item = Result<IniRecord<'a>, IniListError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.offset >= self.source.len() {
            return None;
        }
        if matches!(
            self.source[self.offset],
            b'\r' | b'\n' | b' ' | b'\t' | b'/' | 0xff | 0xfd
        ) {
            self.finished = true;
            return None;
        }
        let record = self.record();
        self.finished = record.is_err();
        self.offset = self.source[self.offset..]
            .iter()
            .position(|b| *b == b'\n')
            .map_or(self.source.len(), |end| self.offset + end + 1);
        Some(record)
    }
}

fn read_text(source: &[u8]) -> &[u8] {
    let end = source
        .iter()
        .position(|b| matches!(*b, b';' | b'\n' | b'\r' | b'\t' | b']'))
        .unwrap_or(source.len());
    &source[..end]
}

pub(super) fn decimal_i32(caption: &[u8]) -> i32 {
    let start = caption
        .iter()
        .position(|b| !matches!(*b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c))
        .unwrap_or(caption.len());
    let (negative, digits) = match &caption[start..] {
        [b'-', digits @ ..] => (true, digits),
        [b'+', digits @ ..] => (false, digits),
        digits => (false, digits),
    };
    let value = digits
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .fold(0_i32, |value, b| {
            value.wrapping_mul(10).wrapping_add(i32::from(*b - b'0'))
        });
    if negative {
        value.wrapping_neg()
    } else {
        value
    }
}
