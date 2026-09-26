//! Призыв области ослабления CWeak.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/weak.cpp.
//! Тело Summon, расчёт срока CONST×EM+100 и живая форма области перенесены
//! буквально в `nebokrai_zone::skills::weak` порцией T5 «zonalcast-хаб»
//! (основание и статусы — в шапке `zone/skills/zonalcast.rs` и owner-файла).
//! Здесь — делегация с прежней сигнатурой; потребитель (hub zonalcast)
//! не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::WEAK_SKILL_ID;

pub(super) fn summon_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_weak(
        game, instance, source, destination, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
