//! Реестр команд по appserver/script/script.cpp/.h, CScript::LoadFunction.
//! Game RVA 0x29100; общий разбор CIni находится в ini.rs.

pub use super::ini::{
    IniListError as FunctionListError, IniListErrorKind as FunctionListErrorKind,
};
use super::ini::{IniRecords, decimal_i32};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionDefinition<'a> {
    pub id: i32,
    pub name: &'a [u8],
}

pub struct FunctionListRecords<'a> {
    records: IniRecords<'a>,
    finished: bool,
}

impl<'a> FunctionListRecords<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self, FunctionListError> {
        Ok(Self {
            records: IniRecords::new(source, b"FunctionList")?,
            finished: false,
        })
    }
}

impl<'a> Iterator for FunctionListRecords<'a> {
    type Item = Result<FunctionDefinition<'a>, FunctionListError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let record = self.records.next()?.and_then(|record| {
            Ok(FunctionDefinition {
                id: decimal_i32(record.caption),
                name: record.value.ok_or(FunctionListError {
                    offset: record.offset,
                    kind: FunctionListErrorKind::MissingValueSeparator,
                })?,
            })
        });
        self.finished = record.is_err();
        Some(record)
    }
}
