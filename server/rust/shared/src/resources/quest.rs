//! Каталог определений по server/setup/questsystem.cpp/.h, World и Game CQuestSystem.
//! Состояние заданий персонажей и выполнение сценариев принадлежат серверам.
//! Происхождение и границы: docs/gameplay/quests.md.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestEntry {
    pub id: u16,
    pub old: u32,
    pub quest_type: u32,
    pub level: u32,
    pub difficulty: u32,
    pub track: u32,
    pub short_description: Vec<u8>,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub abandon_script: Vec<u8>,
    pub complete_script: Vec<u8>,
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub effect_id: i32,
    pub display: bool,
}

impl Default for QuestEntry {
    fn default() -> Self {
        Self {
            // Значение tagQuest по умолчанию; расширение задаёт свой тип.
            quest_type: 2,
            id: 0,
            old: 0,
            level: 0,
            difficulty: 0,
            track: 0,
            short_description: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            abandon_script: Vec::new(),
            complete_script: Vec::new(),
            region_id: 0,
            tile_x: 0,
            tile_y: 0,
            effect_id: 0,
            display: false,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CQuestSystem {
    pub max_quest_count: i32,
    pub level_difference: u32,
    pub player_login_script: Vec<u8>,
    pub player_level_up_script: Vec<u8>,
    pub player_died_script: Vec<u8>,
    pub(super) quests: BTreeMap<u16, QuestEntry>,
}

impl CQuestSystem {
    pub fn insert(&mut self, quest: QuestEntry) -> Option<QuestEntry> {
        self.quests.insert(quest.id, quest)
    }

    pub fn quests(&self) -> &BTreeMap<u16, QuestEntry> {
        &self.quests
    }

    pub fn quest_data_by_id(&self, quest_id: u16) -> Option<&QuestEntry> {
        self.quests.get(&quest_id)
    }

    pub fn complete_script_by_id(&self, quest_id: u16) -> Option<&[u8]> {
        self.quest_data_by_id(quest_id)
            .map(|quest| quest.complete_script.as_slice())
    }

    pub fn disband_script_by_id(&self, quest_id: u16) -> Option<&[u8]> {
        self.quest_data_by_id(quest_id)
            .map(|quest| quest.abandon_script.as_slice())
    }

}
