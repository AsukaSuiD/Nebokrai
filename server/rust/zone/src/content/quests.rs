//! Полученный каталог заданий Zone по server/setup/questsystem.cpp/.h.
//! Game gameserver.exe: CQuestSystem::DecordFromByteArray RVA 0x627a0;
//! формат и порядок частичного декодирования находятся в Shared.

use nebokrai_shared::resources::{
    CQuestSystem, QuestSystemDecodeError, QuestSystemDecodeOutcome,
};

/// Опубликованные определения Zone; прогресс каждого персонажа остаётся у игрока.
#[derive(Debug, Default)]
pub struct QuestCatalog {
    system: CQuestSystem,
}

impl QuestCatalog {
    pub const fn system(&self) -> &CQuestSystem {
        &self.system
    }

    /// Устанавливает полученный World snapshot с исходными частичными эффектами.
    pub fn decode(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<QuestSystemDecodeOutcome, QuestSystemDecodeError> {
        self.system.decord_from_byte_array(source, cursor)
    }

    pub fn clear(&mut self) {
        self.system = CQuestSystem::default();
    }
}
