//! Призыв областей CYinYang и CYinYang2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/yinyang.cpp и
//! yinyang2.cpp. Тело Summon перенесено буквально в
//! `nebokrai_zone::skills::yinyang` порцией T5 «zonalcast-хаб» (порядок
//! запросов, CCH WORD, SetTile до actual region и статусы — там). Здесь —
//! делегация с прежней сигнатурой; потребитель (hub zonalcast) не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::YIN_YANG_SKILL_ID;

pub(super) fn summon_yin_yang<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_yin_yang(
        game, instance, source, destination, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
