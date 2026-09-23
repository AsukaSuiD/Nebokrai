//! CBloodLoss (0x21D), gameserver.exe/GameServer.pdb,
//! исходный владелец appserver/skills/bloodloss.cpp.
//!
//! Общий с PoisonArrow зарегистрированный Check/AI сохраняет исходную S,
//! расход MP, BF918, CAN, задержку, поздний путь и PK. При позднем препятствии
//! ZHGS0053 получает имя захваченной цели; mode13 визуала сохраняет bytes 0,13.
//! Единственный End(int) принадлежит общему координатору.
//!
//! После PK заново берётся WarSoul: MAX → MIN → BF_ATTACK, затем factor,
//! frequency и keep из таблицы, захваченной на входе AI. Без предмета min/max
//! и modifier равны нулю, но состояние всё равно создаётся. Коэффициент
//! вычисляется из u32 и 0.01_f32 в x87 с единственным сохранением f32;
//! modifier сохраняет усечение до i64, младший DWORD и unsigned-перевод в f32.
//! Ctor предшествует первому старому state End/fresh-slot destructor;
//! primary Begin регистрирует новый state в прежней позиции либо в конце.
//! Здесь нет RNG, UpdateProperty или второго End при отказе state Begin.

use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use super::bloodlossstate::{BloodLossState, begin_primary_blood_loss_state};
use super::poisonarrow::{
    ArrowEffect, arrow_replacement_slot, execute_periodic_battle_fairy_arrow,
};
use super::stateskill::state_skill_outcome;
use super::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_ATTACK;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) use nebokrai_zone::effects::BLOOD_LOSS_STATE_ID as BLOOD_LOSS_SKILL_ID;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

pub(crate) fn execute_battle_fairy_blood_loss<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != BLOOD_LOSS_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    execute_periodic_battle_fairy_arrow(
        game, player_id, instance, dispatch, begin_target, runtime, b"ZHGS0053", apply_blood_loss,
    )
}

fn apply_blood_loss<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, effect: ArrowEffect, runtime: &mut Runtime,
) {
    let goods = (effect.user.1.object_type == 400).then(|| game.find_player(effect.user.1.id))
        .flatten().and_then(|player| player.war_soul_goods(game.goods_factory()));
    let (minimum, maximum, modifier) = if let Some(goods) = goods {
        let maximum = effect.properties.query_property(SKILL_USAGE_MAX_ATTACK) as u16;
        let minimum = effect.properties.query_property(SKILL_USAGE_MIN_ATTACK) as u16;
        let attack = goods.addon_property_value(game.goods_factory(), GAP_BF_ATTACK, 1);
        let scaled = truncate_original_i64_low(f64::from(attack) * 0.0001);
        (minimum, maximum, f64::from(scaled as u32) as f32)
    } else { (0, 0, 0.0) };
    let factor = (f64::from(effect.properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR))
        * f64::from(0.01_f32)) as f32;
    let frequency = effect.properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let keep = effect.properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state = BloodLossState::new(effect.master, keep, frequency, factor, modifier, minimum, maximum);
    let Some(placement) = arrow_replacement_slot(game, effect.target, BLOOD_LOSS_SKILL_ID) else { return; };
    let _ = begin_primary_blood_loss_state(
        game, effect.target.0, effect.target.1, Some(effect.user), Some(effect.target), state, placement,
        &mut || runtime.now_milliseconds(),
    );
}
