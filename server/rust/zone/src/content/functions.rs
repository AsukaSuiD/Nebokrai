//! Реестр имён команд по appserver/script/script.cpp/.h, CScript::LoadFunction.
//! Game RVA 0x29100: очистка, чтение FunctionList, последняя запись имени побеждает.
//! Формат разбирает Shared; правила и сборка — docs/gameplay/scripting.md.

use nebokrai_shared::scripting::{FunctionListError, FunctionListRecords};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FunctionRegistryLoadReport {
    pub declared_functions: usize,
    pub replaced_names: usize,
    pub registered_functions: usize,
    pub error: Option<FunctionListError>,
}

#[derive(Debug, Default)]
pub struct ScriptFunctionRegistry {
    functions: BTreeMap<Vec<u8>, i32>,
}

impl ScriptFunctionRegistry {
    pub fn load(&mut self, source: &[u8]) -> FunctionRegistryLoadReport {
        self.functions.clear();
        let mut report = FunctionRegistryLoadReport::default();
        let records = match FunctionListRecords::new(source) {
            Ok(records) => records,
            Err(error) => {
                report.error = Some(error);
                return report;
            }
        };
        for record in records {
            let record = match record {
                Ok(record) => record,
                Err(error) => {
                    report.error = Some(error);
                    break;
                }
            };
            if self
                .functions
                .insert(record.name.to_vec(), record.id)
                .is_some()
            {
                report.replaced_names += 1;
            }
            report.declared_functions += 1;
        }
        report.registered_functions = self.functions.len();
        report
    }

    pub fn query(&self, name: &[u8]) -> Option<i32> {
        let name = name.split(|b| *b == 0).next().unwrap_or_default();
        self.functions.get(name).copied()
    }

    pub fn release(&mut self) -> usize {
        let count = self.functions.len();
        self.functions.clear();
        count
    }
}
