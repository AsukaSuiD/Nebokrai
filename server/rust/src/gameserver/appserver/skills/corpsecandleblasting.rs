//! Тонкий путь к взрыву трупной свечи `CCorpseCandleBlasting` (`0x194`) в
//! Zone. Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/corpsecandleblasting.cpp`. Тело execute_owned, wire-
//! кадры и формула перенесены буквально в
//! `nebokrai_zone::skills::corpsecandleblasting` (основание и статусы MATCH
//! см. там; кластер D полосы Monster 0x19x); **FIX F1** — MIN/MAX урона и
//! hit-модификатор приведены к машинным ключам 20008/20009/20001 Calculate
//! `0x582EA0` (push 0x4E28/0x4E29/0x4E21), прежние 20001/20002/3 были
//! ошибкой реконструкции.
//! Здесь — объявленные швы переноса: hub-фасады `CorpseCandleGame`
//! (`script_file` живого монстра и stage-for-delete `[U+0x80]=1`) и
//! `CorpseCandleContact` (`RunScript` прежнего скриптового owner-а) над
//! прежними `CGame`/`CServerRegion`; остальное обслуживает hub
//! `monsterattack` кластера A2. Делегация сохраняет прежнюю сигнатуру —
//! единственный потребитель (`monsterbaseattack.rs`) не меняется.

use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};
use nebokrai_zone::skills::corpsecandleblasting::{
    self as zone, CorpseCandleContact, CorpseCandleGame,
};

pub(crate) use nebokrai_zone::skills::corpsecandleblasting::CORPSE_CANDLE_BLASTING_SKILL_ID;

impl CorpseCandleGame for CGame {
    fn monster_script_file(&self, region: &CServerRegion, monster_id: i32) -> Option<Vec<u8>> {
        region.find_monster_by_id(monster_id).map(|monster| monster.script_file().to_vec())
    }

    fn monster_stage_for_delete(&mut self, region: &mut CServerRegion, monster_id: i32) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.stage_for_delete();
        }
    }
}

impl<Runtime: GameMainLoopRuntime> CorpseCandleContact<Runtime> for CGame {
    fn run_corpse_candle_script(
        &mut self,
        script_file: &[u8],
        player_id: Option<i32>,
        region_id: i32,
        runtime: &mut Runtime,
    ) -> bool {
        self.run_script_file(
            script_file,
            ScriptExecutionContext {
                player_id,
                region_id: Some(region_id),
                ..ScriptExecutionContext::default()
            },
            runtime,
        )
        .is_some()
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_corpse_candle_blasting<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_corpse_candle_blasting(
        game, owner, monster_id, target_identity, skill_level, properties, now_ms, runtime,
    )
}
