//! Ресурсы сценариев Realm по worldserver/game.cpp/.h: LoadScriptFileData,
//! LoadOneScript и GetScriptFileData. Nworldserver.exe: RVA 0x14450/0x13440/0x132e0.
//! Сборка, проверенные ветви и ограничения — docs/gameplay/scripting.md.

use std::collections::BTreeMap;

/// Доступ к ресурсам и уведомлениям процесса; порядок загрузки задаёт Realm.
pub trait ScriptLoadContext {
    fn read_resource(&mut self, path: &[u8]) -> Option<Vec<u8>>;
    /// None означает отсутствие индексного корня; Some(empty) не разрешает обход диска.
    fn indexed_files(&mut self, root: &[u8], extension: &[u8]) -> Option<Vec<Vec<u8>>>;
    fn loose_files(&mut self, pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>>;
    fn missing_resource(&mut self, path: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptListSource {
    Index,
    Disk,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptRequiredFile {
    Functions,
    Variables,
}

/// Отдельные отказы сценариев не меняют исходный успешный результат общего loader-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptLoadReport {
    pub source: ScriptListSource,
    pub loaded: usize,
    pub failed: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptReleaseState {
    pub functions: bool,
    pub variables: bool,
    pub scripts: bool,
}

/// Единственный владелец загруженных текстов Realm. Состояния исполнения здесь нет.
#[derive(Debug, Default)]
pub struct ScriptResources {
    functions: Option<Vec<u8>>,
    variables: Option<Vec<u8>>,
    scripts: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl ScriptResources {
    pub fn functions(&self) -> Option<&[u8]> {
        self.functions.as_deref()
    }

    pub fn variables(&self) -> Option<&[u8]> {
        self.variables.as_deref()
    }

    /// GetScriptFileData ищет исходный C-string, не преобразуя слеши или регистр.
    pub fn get(&self, path: &[u8]) -> Option<&[u8]> {
        self.scripts.get(c_string_prefix(path)).map(Vec::as_slice)
    }

    /// Порядок ключей сохранён; усечение текстов для wire выполняет отправитель.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.scripts
            .iter()
            .map(|(path, data)| (path.as_slice(), data.as_slice()))
    }

    pub fn clear(&mut self) -> ScriptReleaseState {
        let state = ScriptReleaseState {
            functions: self.functions.take().is_some(),
            variables: self.variables.take().is_some(),
            scripts: !self.scripts.is_empty(),
        };
        self.scripts.clear();
        state
    }

    pub fn load_one<C: ScriptLoadContext + ?Sized>(&mut self, context: &mut C, path: &[u8]) -> bool {
        let path = c_string_prefix(path);
        let Some(data) = context.read_resource(path) else {
            context.missing_resource(path);
            return false;
        };
        self.scripts.insert(normalize_script_path(path), data);
        true
    }

    /// Сначала освобождает прежнее состояние; отказ не откатывает прочитанное.
    pub fn load<C: ScriptLoadContext + ?Sized>(
        &mut self,
        context: &mut C,
        function_file: &[u8],
        variable_file: &[u8],
    ) -> Result<ScriptLoadReport, ScriptRequiredFile> {
        self.clear();
        let Some(data) = context.read_resource(function_file) else {
            context.missing_resource(function_file);
            return Err(ScriptRequiredFile::Functions);
        };
        self.functions = Some(data);
        let Some(data) = context.read_resource(variable_file) else {
            context.missing_resource(variable_file);
            return Err(ScriptRequiredFile::Variables);
        };
        self.variables = Some(data);

        // В оригинале literal scripts/*.* превращается в индексный корень \\scripts.
        let (source, files) = match context.indexed_files(b"\\scripts", b".script") {
            Some(files) => (ScriptListSource::Index, files),
            None => (
                ScriptListSource::Disk,
                context.loose_files(b"scripts/*.*", b".script"),
            ),
        };
        let mut report = ScriptLoadReport {
            source,
            loaded: 0,
            failed: 0,
        };
        for path in files {
            if self.load_one(context, &path) {
                report.loaded += 1;
            } else {
                report.failed += 1;
            }
        }
        Ok(report)
    }
}

/// LoadOneScript убирает один начальный '\\', затем ReplaceLine заменяет '\\' на '/'.
pub fn normalize_script_path(path: &[u8]) -> Vec<u8> {
    let path = c_string_prefix(path);
    let path = path.strip_prefix(b"\\").unwrap_or(path);
    path.iter()
        .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
        .collect()
}

fn c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}
