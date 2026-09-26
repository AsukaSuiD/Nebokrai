//! Тонкий путь к Check/AI передачи ресурсов CHuoxieshu/CLingzhishu в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/{huoxieshu,lingzhishu}.cpp. Тела перенесены буквально в
//! `nebokrai_zone::skills::battlefairytransfer` порцией №6b; доставка BF918
//! стала точечной (решение C, якоря `0x501BDE..0x501C61`/`0x51F249` в шапке
//! Zone `skills/battlefairyskill.rs`). Здесь реэкспорт вида и делегации с
//! прежними сигнатурами; потребители (huoxieshu, lingzhishu, thunder,
//! tianhuo, poisonarrow) не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::BattleFairyTransferKind;

/// Общий кадр `0xBF918` семейства: делегация в Zone (точечный SendToPlayer,
/// решение C; ветки thunder/poisonarrow порции №6c получают ту же доставку,
/// т.к. оригинал делит один container-хелпер).
pub(super) fn send_goods_update(game: &mut CGame, update: &BattleFairyDefaultGoodsUpdate) {
    nebokrai_zone::skills::send_battle_fairy_goods_update(
        game, update.player_id, update.goods.ex_id, &update.old_client_payload,
    );
}

pub(crate) fn execute_battle_fairy_transfer<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, kind: BattleFairyTransferKind, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_transfer(
        game, player_id, instance, dispatch, kind, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}
