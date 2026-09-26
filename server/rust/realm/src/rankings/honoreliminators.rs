//! Honor-eliminator индекс мира (исходный `CGame::m_mapHonorEliminate`) —
//! primary state владельца `rankings` (прежнее transitional-поле `CGame`).
//! Тип исхода регистрации перенесён из `app/worldothermessage`; прежний путь
//! сохранён re-export-ом для прежних consumers.
//!
//! `CGame` хранит только composition handle `honor_eliminators` и делегирует
//! прежний pub facade; онлайн-предусловие регистрации остаётся у facade
//! (прецедент route-параметра bai_tan), а reset-обход игроков и load-очереди —
//! оркестрацией app. Ветви сообщений — `app/worldothermessage`
//! (`0x5FD0C` reset, `0x5FD0D` register).

use std::collections::{BTreeMap, VecDeque};

/// Исход регистрации убийцы ветки `0x5FD0D`.
///
/// Проверку присутствия игрока в online выполняет facade игры; typed
/// register владельца различает только повтор и принятие.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldHonorEliminatorRegistration {
    MissingOnlinePlayer,
    Duplicate,
    Accepted,
}

/// Действующий honor-eliminator индекс мира.
///
/// Constructor-ное состояние — пустая карта; reset-стадия app очищает её
/// целиком typed clear-ом.
pub struct WorldHonorEliminateIndex {
    eliminators: BTreeMap<u32, VecDeque<u32>>,
}

impl WorldHonorEliminateIndex {
    pub fn new() -> Self {
        Self {
            eliminators: BTreeMap::new(),
        }
    }

    /// Регистрирует убийцу игрока; повторный ID не добавляется.
    pub fn register(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration {
        let eliminators = self.eliminators.entry(player_id).or_default();
        if eliminators
            .iter()
            .any(|tracked_id| *tracked_id == eliminator_id)
        {
            return WorldHonorEliminatorRegistration::Duplicate;
        }
        eliminators.push_back(eliminator_id);
        WorldHonorEliminatorRegistration::Accepted
    }

    pub fn clear(&mut self) {
        self.eliminators.clear();
    }
}
