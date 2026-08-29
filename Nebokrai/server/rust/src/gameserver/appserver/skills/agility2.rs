//! Временная ловкость `CAgility2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `agility2.cpp` и `agilitystate2.cpp`. Общие проверки, расход MP, задержка и
//! публикация визуального сообщения совпадают с семейством ловкости и остаются
//! в `agility.rs`. Этот владелец сохраняет создание состояния `0x81`, порядок
//! двух вызовов часов для клиентского остатка и передачу состояния каноническому
//! хранилищу. Формула и строгая граница завершения принадлежат
//! `AgilityState2`.

use super::agilitystate2::AgilityState2;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const AGILITY_2_SKILL_ID: u32 = 129;

pub(crate) fn begin_agility_2_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
    runtime: &mut Runtime,
) -> i32 {
    let state = AgilityState2::new(full_miss, started_at_ms, keep_time_ms);
    if let Some(player) = game.find_player_mut(player_id) {
        player.begin_agility_state_2(state);
    }
    let first_now_ms = runtime.now_milliseconds();
    let second_now_ms = if state.client_time_needs_second_clock(first_now_ms) {
        runtime.now_milliseconds()
    } else {
        first_now_ms
    };
    state.client_time(first_now_ms, second_now_ms)
}
