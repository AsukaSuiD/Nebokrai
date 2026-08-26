//! Упорядоченная граница отложенных эффектов GameServer.
//!
//! Владелец хранит только те типизированные действия, которые возникают в
//! callback-е без доступа к `CGame` и потому должны быть применены игровым
//! владельцем позже. Уже выполненные сетевые и игровые действия сюда не
//! копируются. `Mutex<VecDeque<_>>` заменяет исходную межвладельческую очередь,
//! сохраняя порядок добавления и атомарное извлечение накопленного прохода.
//! Диагностика публикуется через `tracing` в месте возникновения и журналом не
//! является.

use std::sync::Arc;

use super::serverregion::RegionTaxSessionKind;
use crate::nets::msgqueue::CMsgQueue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameEffect {
    RegionTaxPrompt {
        kind: RegionTaxSessionKind,
        player_id: i32,
        session_id: i64,
        password: i32,
        first_value: u32,
        second_value: Option<u32>,
    },
    RegionTaxResult {
        kind: RegionTaxSessionKind,
        player_id: i32,
        region_id: i32,
        value: i32,
    },
}

#[derive(Default)]
pub(crate) struct GameEffectJournal {
    effects: CMsgQueue<GameEffect>,
}

impl GameEffectJournal {
    pub(crate) fn push(&self, effect: GameEffect) {
        self.effects.push(effect);
    }

    pub(crate) fn take_all(&self) -> std::collections::VecDeque<GameEffect> {
        self.effects.take_all()
    }

    pub(crate) fn extend(&self, effects: impl IntoIterator<Item = GameEffect>) {
        self.effects.extend(effects);
    }
}

pub(crate) type SharedGameEffectJournal = Arc<GameEffectJournal>;
