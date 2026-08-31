//! Сценарный диспетчер арены `CScript::JJcFunction` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/script/jjcfunc.cpp`, IDs `10000..10009`. Сохранены selector-ы
//! восьми `u16` полей `tagJJcData`, wire `0x60902..0x60907`, использование
//! script player/region context и порядок перехода в `CJJcSystem`. Технический
//! `Catch@004281d2` был только MSVC cleanup для `CMessage`; RAII Rust заменяет
//! его без игровой семантики. Значения EAX после исходных void-функций были
//! неопределёнными и здесь намеренно нормализованы в ноль; фактический return
//! `CMessage::Send` сохранён для трёх прямых World-команд.

use super::function::SCRIPT_FUNCTION_ARGUMENT_CAPACITY;
use crate::gameserver::gameserver::game::{CGame, game_wall_time_seconds};
use crate::nets::netserver::message::CMessage;

const FIRST_JJC_SCRIPT_FUNCTION: i32 = 10_000;
const LAST_JJC_SCRIPT_FUNCTION: i32 = 10_009;
const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JjcScriptFunctionOutcome {
    DifferentFunction,
    Handled { legacy_return: i32 },
}

pub(crate) fn dispatch_jjc_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    script_region_id: Option<i32>,
    function_id: i32,
    integer_arguments: [Option<i32>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
) -> JjcScriptFunctionOutcome {
    if !(FIRST_JJC_SCRIPT_FUNCTION..=LAST_JJC_SCRIPT_FUNCTION).contains(&function_id) {
        return JjcScriptFunctionOutcome::DifferentFunction;
    }
    let integer = |index: usize| {
        integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR)
    };
    let legacy_return = match function_id - FIRST_JJC_SCRIPT_FUNCTION {
        0 => i32::from(
            script_player_id.is_some_and(|player_id| game.apply_player_jjc(player_id)),
        ),
        1 => {
            if let Some(player_id) = script_player_id {
                let _ = game.quit_player_jjc(player_id);
            }
            0
        }
        2 => {
            let selector = integer(0);
            let value = integer(1);
            if value == SCRIPT_INT_PARAMETER_ERROR {
                0
            } else {
                if let Some(player_id) = script_player_id {
                    let _ = game
                        .find_player_mut(player_id)
                        .is_some_and(|player| player.set_jjc_counter(selector, value as u16));
                }
                value
            }
        }
        3 => script_player_id
            .and_then(|player_id| game.find_player(player_id))
            .and_then(|player| player.jjc_counter(integer(0)))
            .map(i32::from)
            .unwrap_or_default(),
        4 => {
            if let (Some(region_id), Some(player_id)) = (script_region_id, script_player_id) {
                let _ = game.end_player_jjc(region_id, player_id);
            }
            0
        }
        5 => match (script_region_id, script_player_id) {
            (Some(region_id), Some(player_id)) => {
                game.jjc_opponent_info(integer(0) as u32, region_id, player_id)
            }
            _ => 0,
        },
        6 => 0,
        7 => {
            let mut message = CMessage::new(0x0006_0905);
            message.add_long(script_player_id.unwrap_or_default());
            message.add_ulong(integer(0) as u32);
            message.send(game, false).unwrap_or_default()
        }
        8 => {
            let mut message = CMessage::new(0x0006_0906);
            message.add_ulong(game_wall_time_seconds() as u32);
            message.send(game, false).unwrap_or_default()
        }
        9 => {
            let mut message = CMessage::new(0x0006_0907);
            message.add_ulong(game_wall_time_seconds() as u32);
            message.send(game, false).unwrap_or_default()
        }
        _ => unreachable!("диапазон JJC-функций проверен до dispatch"),
    };
    JjcScriptFunctionOutcome::Handled { legacy_return }
}
