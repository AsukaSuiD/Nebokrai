//! Двойной направленный удар Swallow (0x6A).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/swallow.cpp.
//!
//! Зарегистрированный Begin сохраняет исходного U, раннее время и loop1 visual.
//! Check требует reuse, меч категории 2 и ненулевую цену MP; нулевая цена
//! отклоняется молча. Первый AI списывает MP до OnChangeStates и повторной
//! проверки меча. Живой S либо базовая точка определяет направление: оно
//! сохраняется до SetDir, затем CAN, visual0 и condition. Таблица свойств
//! остаётся прежней через callbacks этого AI; каждый Calculate читает свою.
//!
//! Два прохода ждут абсолютные unsigned сроки start+delay и start+delay+interval.
//! Первый публикует visual1, выключает available и лишь после прохода включает
//! firstAttack. Второй заново читает start и не повторяет visual1. Оба могут
//! пройти за один AI; отсутствие региона не отменяет успешный End(1).
//! Swallowattack владеет живым направлением каждой клетки, отдельной локальной
//! дедупликацией проходов и общим оружейным расчётом. Сохранённое направление
//! нужно только visual1. End сбрасывает фазу и firstAttack, включает available
//! и сбрасывает направление до движения, AfterUse, reuse и удаления visual. Каноническое
//! исполнение остаётся опубликованным через весь AI и его вложенные callbacks.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use super::swallowattack::run_swallow_attack;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const SWALLOW_SKILL_ID: u32 = 0x6a;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const ACTION_INTERVAL: u32 = 10_009;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwallowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    first_attack_done: bool,
    direction: i32,
}

impl SwallowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            first_attack_done: false,
            direction: -1,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(super) const fn direction(&self) -> i32 { self.direction }

    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.first_attack_done = false;
        self.kernel.lifecycle_mut().set_available(true);
        self.direction = -1;
        true
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0292", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    if !weapon_is_sword(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        mana_failure(game, instance, player_id, &properties);
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, user.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_sword(game, player)) {
                failure(game, instance, user.1.id, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<SwallowExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.direction = direction;
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let Some(first_attack_done) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<SwallowExecutionState>())
        .map(|state| state.first_attack_done)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !first_attack_done {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
        game.update_registered_skill_visual(instance, 1);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(false);
        run_swallow_attack(game, instance, user, runtime);
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<SwallowExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.first_attack_done = true;
        let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let interval = properties.query_property(ACTION_INTERVAL);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < interval.wrapping_add(delay).wrapping_add(started) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    run_swallow_attack(game, instance, user, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_swallow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != SWALLOW_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Swallow,
        |game, instance, player_id, runtime| {
            if !check_cast(game, instance, player_id, runtime) { return false; }
            let Some(skill) = game.registered_skill_mut(instance) else { return false; };
            skill.lifecycle_mut().set_available(true);
            true
        },
        |dispatch, started| SwallowExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
