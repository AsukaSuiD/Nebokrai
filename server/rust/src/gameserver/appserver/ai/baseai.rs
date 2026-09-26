//! Делегат базовой части `CBaseAI` в Zone.
//!
//! Три FIFO-очереди объявленных действий, object-цель, dormancy, фоновый
//! список back-stage навыков, active-фаза `ProcessActiveAction` и
//! пространственная механика `MoveTo` (`Slip` + задержка одного шага)
//! перенесены буквально в `nebokrai_zone::ai::baseai` — машинная база
//! VERIFIED по точной паре `4F5C98E0…` + GameServer.pdb (RSDS match),
//! RVA-якоря, свежая точечная сверка волны Z-AI-player (таблица
//! `_slip_order` 8×8 dword, формула задержки с `g_ms = 80` → `0.68`,
//! прологи Run/passive/active/reaction-входов) и честные UNKNOWN —
//! в её шапке (`zone/src/ai/baseai.rs`). Элементы `AI_EVENT` и порядок
//! реакций Defense/Stiffen/Died остаются в `nebokrai_zone::ai::events` и
//! `nebokrai_zone::ai::reactions` (кластер A1). Здесь:
//!
//! - переэкспорт типов и чистых функций с прежними именами — потребители
//!   (`monster.rs`, `moveshape.rs`, `playerai`, guard refresh в `game.rs`,
//!   dispatcher-делегат `monsterai.rs`, `shapemessage.rs`) не меняются;
//! - прежняя сигнатура `find_slip_step_in_direction` с hub-обёрткой
//!   `CServerRegion`: Zone-версия принимает `&CRegion`, перевод
//!   `&region.region` выполняется здесь. Региональный реестр, around-
//!   доставка и час-тик owner-а остаются hub-владением через фасады
//!   `monsterai`.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{MoveCheckCellRegistry, ShapeAreaCoordinates};

pub(crate) use nebokrai_zone::ai::{AiShapeAction, PassiveDeathAction, PassiveStiffenAction};
pub(crate) use nebokrai_zone::ai::baseai::{
    one_step_move_delay_ms, AiPhaseState, CBaseAI,
};

/// Один одноклеточный `Slip` из `CBaseAI::Slip` (RVA `0x0C7FF0`)
/// (прежняя сигнатура с hub-обёрткой `CServerRegion`).
pub(crate) fn find_slip_step_in_direction(
    move_check_cells: &MoveCheckCellRegistry,
    region: &CServerRegion,
    origin: ShapeAreaCoordinates,
    desired_direction: i32,
    figure_index: usize,
) -> Option<(i32, ShapeAreaCoordinates)> {
    nebokrai_zone::ai::baseai::find_slip_step_in_direction(
        move_check_cells, &region.region, origin, desired_direction, figure_index,
    )
}
