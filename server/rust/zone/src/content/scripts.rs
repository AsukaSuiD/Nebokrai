//! Ресурсы Zone по server/gameserver/gameserver/game.cpp/.h: SetFunctionFileData, SetVariableFileData,
//! SetScriptFileData и GetScriptFileData. Game RVA 0x2310/0x2340/0xb4b0/0x28c00.
//! Идентификатор сборки и проверенные ветви — docs/gameplay/scripting.md.

use std::collections::BTreeMap;

/// Диагностика Rust: исходные setters не возвращают признак замены.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptResourcePublication {
    Published,
    Replaced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptResourcesReleased {
    pub functions: bool,
    pub variables: bool,
    pub scripts: usize,
}

/// Полученные тексты, без активных сценариев и значений переменных персонажей.
#[derive(Debug, Default)]
pub struct ScriptResources {
    functions: Option<Vec<u8>>,
    variables: Option<Vec<u8>>,
    scripts: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl ScriptResources {
    /// После замены буфера всегда перестраивает реестр по новому C-string.
    /// Результат callback передаёт вызывающему диагностику реестра.
    pub fn set_functions<R>(
        &mut self,
        data: Vec<u8>,
        load_registry: impl FnOnce(&[u8]) -> R,
    ) -> (ScriptResourcePublication, R) {
        let publication = replace_buffer(&mut self.functions, data);
        let report = load_registry(c_string_prefix(
            self.functions.as_deref().expect("буфер установлен"),
        ));
        (publication, report)
    }

    /// Новые объявления применяются потребителем отдельно; живые значения здесь не меняются.
    pub fn set_variables(&mut self, data: Vec<u8>) -> ScriptResourcePublication {
        replace_buffer(&mut self.variables, data)
    }

    /// Ключ — исходные байты до первого NUL, без нормализации слешей или регистра.
    /// Возвращаемый признак замены используется только для диагностики Rust.
    pub fn set_script(&mut self, mut path: Vec<u8>, data: Vec<u8>) -> bool {
        path.truncate(c_string_prefix(&path).len());
        self.scripts.insert(path, data).is_some()
    }

    pub fn functions(&self) -> Option<&[u8]> {
        self.functions.as_deref()
    }

    pub fn variables(&self) -> Option<&[u8]> {
        self.variables.as_deref()
    }

    pub fn script(&self, path: &[u8]) -> Option<&[u8]> {
        self.scripts.get(c_string_prefix(path)).map(Vec::as_slice)
    }

    pub fn clear(&mut self) -> ScriptResourcesReleased {
        let released = ScriptResourcesReleased {
            functions: self.functions.take().is_some(),
            variables: self.variables.take().is_some(),
            scripts: self.scripts.len(),
        };
        self.scripts.clear();
        released
    }
}

fn replace_buffer(slot: &mut Option<Vec<u8>>, data: Vec<u8>) -> ScriptResourcePublication {
    // После delete в оригинале есть продолжение: публикация нового указателя.
    let replaced = slot.take().is_some();
    *slot = Some(data);
    if replaced {
        ScriptResourcePublication::Replaced
    } else {
        ScriptResourcePublication::Published
    }
}

fn c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}
