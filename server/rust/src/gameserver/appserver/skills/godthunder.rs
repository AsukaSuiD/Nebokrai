//! Призыв областей CGodThunder и CGodThunder2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godthunder.cpp и
//! godthunder2.cpp. Тело Summon перенесено буквально в
//! `nebokrai_zone::skills::godthunder` порцией T5 «zonalcast-хаб» (порядок
//! живых чтений, выбор исходного ID у диспетчера и статусы — там). Здесь —
//! делегация с прежней сигнатурой; потребитель (hub zonalcast) не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::GOD_THUNDER_SKILL_ID;

pub(super) fn summon_god_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, skill_id: u32,
    source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_god_thunder(
        game, instance, skill_id, source, destination, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
