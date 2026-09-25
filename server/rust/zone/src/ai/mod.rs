//! Поведение AI фигур живого региона: хранилища врагов конкретных
//! производных AI и общая порядковая механика passive-реакций исторического
//! GameServer. Компонент выделен зелёной порцией PassiveGladiator из старого
//! пакета (`appserver/ai/passivegladiator.cpp`, `appserver/ai/baseai.cpp`);
//! поля, постановка событий, три FIFO и active-фаза `CBaseAI` остаются
//! hub-владением до своих порций.

mod events; // элементы `AI_EVENT` и коды `AI_SHAPE_ACTION`; чистый wrapping-deadline.
mod passivegladiator; // `CPassiveGladiator`: список врагов, hurt-разбор и typed-выбор цели.
mod reactions; // `CBaseAI`: буквальный порядок Defense/Stiffen/Died-реакций над очередями hub-владельца.

pub use events::{ai_event_deadline_reached, AiEvent, AiShapeAction}; // совместимые данные FIFO обоих AI-владельцев.
pub use passivegladiator::{
    PassiveGladiatorAttackFacts, PassiveGladiatorAttackOutcome, PassiveGladiatorCandidate,
    PassiveGladiatorEnemyNotice, PassiveGladiatorSelection, PassiveGladiatorState,
}; // состояние и typed-контракты пассивного гладиатора.
pub use reactions::{
    begin_reached_death_action, begin_reached_stiffen_action, discard_active_prefix,
    finish_reached_death_action, finish_reached_stiffen_action, process_reached_defense_actions,
    reached_death_action_state, PassiveDeathAction, PassiveReactionQueues, PassiveStiffenAction,
}; // семья действий-порядков реакций и узкая generic-сварка очередей.
