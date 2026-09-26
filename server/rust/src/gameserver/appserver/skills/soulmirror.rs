//! Зеркало душ `CSoulMirror` (`0x13C`).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/soulmirror.cpp.
//! Маска и параметры клетки находятся в Zone `skills/soulmirror.rs`; туда же
//! буквально перенесены тела обхода области и призыва пустой клетки
//! (порция №6c «self/zone-касты»; основание и машинные статусы см. там).
//! Здесь — тонкие делегации с прежними сигнатурами: швы Zone (`area_cell_views`,
//! PK-допуск, элементный контакт `directelementattack`, мастер `weaponattack`,
//! lifecycle призванного существа) реализованы над `CGame` в
//! `skills/selfcast.rs`; потребители (zonalcast) не меняются.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::soulmirror;

pub(crate) use nebokrai_zone::skills::SOUL_MIRROR_SKILL_ID;

pub(super) fn apply_soul_mirror_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    soulmirror::apply_soul_mirror_area(game, instance, source, properties, runtime);
}
