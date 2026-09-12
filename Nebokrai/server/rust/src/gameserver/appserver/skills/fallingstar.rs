//! Выпуск накопленных стрел CFallingStar (0xD5).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fallingstar.cpp.
//! Begin, проверки лука/MP/пути, AI и Summon совпадают с MeteorArrow: общий
//! исполнитель удерживает исходного U для Check, свежие U/S на каждом AI,
//! frozen свойства через callbacks и необратимый расход MP перед повторным
//! допуском. Первый IDCC расходуется деструктором без End/UpdateProperty;
//! отрицательный signed-запас остаётся ненулевым и сохраняет DWORD-биты.
//!
//! Отличается маска формы: все уровни выбирают полную 1×1 после SetTileXY,
//! сохраняя два вызова random(1) на стрелу. Снимок атаки и сетевой ID формы
//! остаются метеорными 0xCD; visual самого cast использует 0xD5 и target0/0.
//! Отдельного execution payload и второго фонового runtime нет. Общий End
//! сбрасывает фазу, возвращает Move1 свежему U и вызывает Summon End
//! с настоящим аргументом, не откатывая уже израсходованные ресурсы.

use super::meteorarrow::execute_meteor_arrow_family;
use super::meteorarrowphalanx::MeteorArrowScope;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) const FALLING_STAR_SKILL_ID: u32 = 0xd5;

pub(crate) fn execute_player_falling_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_meteor_arrow_family(game, player_id, instance, dispatch, MeteorArrowScope::FallingStar, runtime)
}
