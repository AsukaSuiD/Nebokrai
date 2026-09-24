//! Карта прогресса из `CPlayer` Game (`server/gameserver/appserver/player.cpp`).
//! GameServer/gameserver.exe SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E;
//! GameServer.pdb: RSDS 5bee6dd1-bf90-49b8-8be9-eb25c4038d53, age 2.
//! `AddQuestDataByteArray` VA 0x00433310–0x0043339F: count, затем
//! упорядоченные u16 ID и u8 state. `AddQuestDataByteArray_ForClient`
//! VA 0x0043E204–0x0043E229 пропускает state 1 и неизвестные каталогу ID.

use std::collections::BTreeMap;

/// Состояние задания вложено в живого игрока; каталог определений хранится отдельно.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct PlayerQuestProgress {
    states: BTreeMap<u16, u8>,
}

impl PlayerQuestProgress {
    pub fn clear(&mut self) {
        self.states.clear();
    }

    pub fn insert_snapshot(&mut self, quest_id: u16, state: u8) {
        self.states.insert(quest_id, state);
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u16, &u8)> {
        self.states.iter()
    }

    /// Клиент получает только незавершённые ID, присутствующие в каталоге.
    pub fn client_entries<T>(
        &self,
        mut lookup: impl FnMut(u16) -> Option<T>,
    ) -> impl Iterator<Item = (u16, T)> {
        self.states.iter().filter_map(move |(quest_id, state)| {
            if *state == 1 {
                return None;
            }
            lookup(*quest_id).map(|entry| (*quest_id, entry))
        })
    }

    pub fn raw_state(&self, quest_id: u16) -> Option<u8> {
        self.states.get(&quest_id).copied()
    }

    pub fn state(&self, quest_id: u16) -> i32 {
        self.raw_state(quest_id).map_or(2, i32::from)
    }

    pub fn accept(&mut self, quest_id: u16) -> bool {
        if self.raw_state(quest_id) == Some(0) {
            return false;
        }
        self.states.insert(quest_id, 0);
        true
    }

    pub fn complete(&mut self, quest_id: u16) -> bool {
        let Some(state) = self.states.get_mut(&quest_id) else {
            return false;
        };
        *state = 1;
        true
    }

    pub fn remove(&mut self, quest_id: u16) -> bool {
        self.states.remove(&quest_id).is_some()
    }

    pub fn contains(&self, quest_id: u16) -> bool {
        self.states.contains_key(&quest_id)
    }

    pub fn valid_count(&self, mut is_displayed_quest: impl FnMut(u16) -> bool) -> i32 {
        self.states
            .iter()
            .filter(|(quest_id, state)| **state != 1 && is_displayed_quest(**quest_id))
            .count()
            .try_into()
            .unwrap_or(i32::MAX)
    }
}
