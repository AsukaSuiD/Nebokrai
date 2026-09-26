//! Призыв движущейся сферы CChaosSphere.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/chaossphere.cpp.
//! Тело Summon перенесено буквально в `nebokrai_zone::skills::chaossphere`
//! порцией T5 «zonalcast-хаб» (ветви пути, порядок запросов и статусы —
//! там и в `docs/gameplay/skills.md`). Здесь — делегация с прежней
//! сигнатурой; потребитель (hub zonalcast) не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::CHAOS_SPHERE_SKILL_ID;

pub(super) fn summon_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_chaos_sphere(
        game, instance, source, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
