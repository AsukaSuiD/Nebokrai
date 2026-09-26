//! Элементы FIFO-очередей объявленных действий фигуры (`AI_EVENT`
//! исторического GameServer) и числовые коды `AI_SHAPE_ACTION`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (идентификаторы —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки).
//! Публичные
//! символы семейства: `CBaseAI::AddAIEvent` (`1:0x0c7f90` → RVA `0x0c8f90`)
//! и `CBaseAI::ProcessPassiveAction` (`1:0x0c74f0` → RVA `0x0c84f0`).
//! PDB подтверждает numeric `AI_SHAPE_ACTION` `0..8`,
//! `ASA_FORCE_DWROD = 0xFF` и layout `AI_EVENT`
//! `action/beginning/delay/handling = +0/+4/+8/+C`.
//! Исходный владелец PDB: `server/gameserver/appserver/ai/baseai.cpp`.
//!
//! Это совместимые данные, а не владелец очереди: сами `VecDeque`-поля,
//! `add_ai_event` и active-фаза `CBaseAI` остаются hub-владением и хранят эти
//! элементы дословно. Deadline событий сравнивается после
//! wrapping DWORD-сложения `begin + delay`, включая раннее завершение рядом с
//! переполнением часов `timeGetTime`.

/// Числовые коды `AI_SHAPE_ACTION`; PDB-значения сохранены литералами,
/// включая служебный `0xFF`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiShapeAction {
    Stand = 0,
    Move = 1,
    Attack = 2,
    Defense = 3,
    Stiffen = 4,
    SearchEnemy = 5,
    ChangeSkill = 6,
    Died = 7,
    Open = 8,
    ForceDwrod = 0xff,
}

/// Одна запись `AI_EVENT`; часы и `handling` мутирует только владелец очереди
/// либо порядковая реакция через узкую сварку `PassiveReactionQueues`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AiEvent {
    pub action: AiShapeAction,
    pub beginning_time_ms: u32,
    pub delay_ms: u32,
    pub handling: i32,
}

/// Exact unsigned deadline из `ProcessActiveAction/ProcessPassiveAction`:
/// исходник сначала складывает два DWORD, затем сравнивает результат с
/// текущими часами. Это намеренно не эквивалентно elapsed-сравнению в момент
/// переполнения `timeGetTime`.
pub const fn ai_event_deadline_reached(event: &AiEvent, now_ms: u32) -> bool {
    event.beginning_time_ms.wrapping_add(event.delay_ms) <= now_ms
}
