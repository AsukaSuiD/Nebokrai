//! Упорядоченная граница отложенных эффектов GameServer.
//!
//! Владелец хранит только те типизированные действия, которые возникают в
//! callback-е без доступа к `CGame` и потому должны быть применены игровым
//! владельцем позже. Уже выполненные сетевые и игровые действия сюда не
//! копируются. `Mutex<VecDeque<_>>` заменяет исходную межвладельческую очередь,
//! сохраняя порядок добавления и атомарное извлечение накопленного прохода.
//! Диагностика публикуется через `tracing` в месте возникновения и журналом не
//! является.

use std::collections::VecDeque;
use std::mem;
use std::sync::Arc;

use super::player::{BattleFairySkillDispatch, PlayerSkillDispatch};
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
    SkillNotification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        message_type: u32,
    },
    ClearPlayerEmotion {
        player_id: i32,
        region_id: Option<i32>,
    },
    SkillSocketReject {
        socket_id: i32,
        message_type: u32,
        reason: u32,
        code: u8,
    },
    QueuePlayerSkill {
        player_id: i32,
        dispatch: PlayerSkillDispatch,
    },
    QueueBattleFairySkill {
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
    },
}

#[derive(Default)]
pub(crate) struct GameEffectJournal {
    effects: VecDeque<GameEffect>,
}

impl GameEffectJournal {
    pub(crate) fn push(&mut self, effect: GameEffect) {
        self.effects.push_back(effect);
    }

    pub(crate) fn take_all(&mut self) -> VecDeque<GameEffect> {
        mem::take(&mut self.effects)
    }

    pub(crate) fn extend(&mut self, effects: impl IntoIterator<Item = GameEffect>) {
        self.effects.extend(effects);
    }
}

pub(crate) type SharedGameEffectJournal = Arc<CMsgQueue<GameEffect>>;

pub(crate) fn shared_game_effect_journal() -> SharedGameEffectJournal {
    Arc::new(CMsgQueue::default())
}
