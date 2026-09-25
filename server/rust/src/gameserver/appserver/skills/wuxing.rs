//! Установка постоянных состояний CWuXingMetal/Wood/Water/Fire/Earth.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/wuxing*.cpp.
//! ID-карта и подготовка 24 параметров — zone rules `skills/wuxing.rs`;
//! выбор элемента-записи и player-gate — zone; здесь живой обход Game.
//! После player-gate новый объект получает первичный Begin до поиска первого
//! старого ID. Общая установка сохраняет прежнюю позицию, исполняет End и
//! destructor свежего остатка, затем UpdateProperty. RestoreHpMp следует
//! отдельно и не зависит от результата UpdateProperty. Успешный AI вызывает
//! End1 даже при отказе нового Begin; отсутствие U/S или иной тип даёт End0.

use super::immediatestateinstallation::replace_immediate_state;
use super::wuxingstate::{WuXingState, WuXingStateParameters, kind_for_skill_id};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(super) fn apply_wuxing_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    parameters: WuXingStateParameters, runtime: &mut Runtime,
) -> bool {
    let Some(kind) = kind_for_skill_id(skill_id) else { return false; };
    let state = WuXingState::new(skill_id, kind, parameters);
    let record = state.encoded();
    let installed = replace_immediate_state(game, source, skill_id, state, &record, runtime);
    if installed && source.1.object_type == 400 && game.find_player(source.1.id).is_some() {
        let _ = game.restore_player_hp_mp_states(source.1.id);
    }
    installed
}
