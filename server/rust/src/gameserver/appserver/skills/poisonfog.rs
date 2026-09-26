//! Призыв ядовитого тумана CPoisonFog.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/poisonfog.cpp.
//! Тело Summon и живая форма области перенесены буквально в
//! `nebokrai_zone::skills::poisonfog` порцией T5 «zonalcast-хаб» (порядок
//! запросов ER_COEFF→ER_LOSS→DODGE_LOSS→DEF_COEFF→DEF_LOSS→PERSIST→уровень→
//! LIFETIME и статусы — там). Здесь — делегация с прежней сигнатурой;
//! потребитель (hub zonalcast) не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::effects::POISON_FOG_STATE_ID as POISON_FOG_SKILL_ID;

pub(super) fn summon_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_poison_fog(
        game, instance, source, destination, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
