//! CLeiming2 (0x21B), gameserver.exe/GameServer.pdb, appserver/skills/thunder2.cpp.
//!
//! Общие Check/AI и граница зарегистрированного исполнения находятся в thunder.
//! Собственный Summon сохраняет порядок CCH → AddElementAtk → max → min →
//! текущий level → lifetime. В отличие от Thunder, к вычисленному стихийному
//! коэффициенту прибавляется AddElementAtk, а частота/число целей не читаются.
//! Clock конструктора предшествует ID, центр устанавливается до допуска
//! региона. Отказ регистрации обрабатывает общий publisher области; AI в любом
//! случае заканчивает попытку с внешним End(1), без повторного visual или Begin.

use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use super::thunder::{
    SKILL_USAGE_SUMMONED_LIFETIME, execute_thunder_family, summon_user_add_element,
    summon_user_cch, summon_user_region, terminal, thunder_summon_properties,
};
use super::thunder2phalanx::CLeimingPhalanx2;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const LEIMING2_SKILL_ID: u32 = 0x21b;
pub(crate) const LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;

pub(crate) fn execute_battle_fairy_leiming2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != LEIMING2_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_thunder_family(game, player_id, instance, dispatch, begin_target, runtime, summon_leiming2)
}

fn summon_leiming2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    position: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, element)) = thunder_summon_properties(game, instance, source) else { return; };
    let cch = summon_user_cch(game, source.1);
    let element = summon_user_add_element(game, source.1).wrapping_add(element);
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CLeimingPhalanx2::new(id, master, started, lifetime, level, minimum, maximum, element, cch);
    phalanx.set_center(position.0, position.1);
    let Some(region) = summon_user_region(game, source) else { return; };
    if game.add_leiming2_phalanx(region, phalanx, position.0, position.1, started, runtime)
        .is_some_and(|result| result.is_ok())
    {
        let _ = game.send_leiming2_phalanx_entry(region, id, runtime);
    }
}
