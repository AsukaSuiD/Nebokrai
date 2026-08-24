//! Reached buff-skill extension исторического `CScript::RunFunction`.
//!
//! `AddJingJieBuff` присутствует в shipped RU scripts, но отсутствует в их
//! `function.ini`; поздний GameServer регистрирует selector `11131` кодом.
//! Исполнение сохраняет player-before-argument gate и передаёт mutation в
//! concrete realm/property owner, а не в отдельный script shadow.

use crate::gameserver::gameserver::game::{CGame, RealmAppellationScriptContext};

pub(crate) const SCRIPT_FUNCTION_ADD_JING_JIE_BUFF: i32 = 11131;
pub(crate) const SCRIPT_FUNCTION_ADD_JING_JIE_BUFF_NAME: &[u8] = b"AddJingJieBuff";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuffSkillScriptFunctionOutcome {
    DifferentFunction,
    Invalid,
    Handled { legacy_return: i32 },
}

pub(crate) fn run_buff_skill_script_function<Context: RealmAppellationScriptContext>(
    game: &mut CGame,
    context: &mut Context,
    script_player_id: Option<i32>,
    function_id: i32,
    argument_count: usize,
    appellation_id: Option<i32>,
) -> BuffSkillScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_ADD_JING_JIE_BUFF {
        return BuffSkillScriptFunctionOutcome::DifferentFunction;
    }
    if argument_count != 1 {
        return BuffSkillScriptFunctionOutcome::Invalid;
    }
    let (Some(player_id), Some(appellation_id)) = (script_player_id, appellation_id) else {
        return BuffSkillScriptFunctionOutcome::Invalid;
    };
    game.set_script_realm_appellation_bonus(player_id, appellation_id as u32, context)
        .map_or(BuffSkillScriptFunctionOutcome::Invalid, |legacy_return| {
            BuffSkillScriptFunctionOutcome::Handled { legacy_return }
        })
}
