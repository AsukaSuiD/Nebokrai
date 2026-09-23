//! Каталог заданий Realm по server/setup/questsystem.cpp/.h.
//! Nworldserver.exe: CQuestSystem::Load RVA 0x67aa0, Initialize RVA 0x68be0;
//! владелец читает ресурсы, а формат и частичные изменения заданы Shared.

use nebokrai_shared::resources::{CQuestSystem, QuestSystemLoadReport};

pub const QUEST_PATH: &[u8] = b"Data/Quest.ini";
pub const QUEST_EX_PATH: &[u8] = b"Data/QuestEx.ini";

/// Опубликованный Realm каталог: загрузка меняет этот экземпляр без отката.
#[derive(Debug, Default)]
pub struct QuestCatalog {
    system: CQuestSystem,
}

impl QuestCatalog {
    pub fn system(&self) -> &CQuestSystem {
        &self.system
    }

    /// Оба входа World вызывают один Load. Второй файл читается после
    /// обработки первого; отсутствие первого завершает загрузку раньше.
    pub fn load<ReadResource, ResolveString>(
        &mut self,
        mut read_resource: ReadResource,
        resolve_string: &mut ResolveString,
    ) -> QuestSystemLoadReport
    where
        ReadResource: FnMut(&[u8]) -> Option<Vec<u8>>,
        ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        let primary = read_resource(QUEST_PATH);
        self.system.load_from_resources(
            primary.as_deref(),
            || read_resource(QUEST_EX_PATH),
            resolve_string,
        )
    }

    pub fn clear(&mut self) {
        self.system = CQuestSystem::default();
    }
}
