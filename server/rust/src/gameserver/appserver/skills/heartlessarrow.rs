//! Заряжаемый выстрел CHeartLessArrow (0xCA).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/heartlessarrow.cpp.
//! Общие Check и visual семейства принадлежат heartlessarrow2; зарегистрированный
//! playercast сохраняет исходный Begin, единственный ключ исполнения и End.
//! Каждый AI удерживает таблицу и полные U/S. NULL заканчивает End(0) молча;
//! смерть/self дают visual10 и разные сообщения. MP→OnChangeStates→лук,
//! CAN→направление→visual0 предшествуют condition. Поздний отказ не возвращает MP.
//!
//! Удержание измеряет wrapping now-start и ограничено двойным ACTION_INTERVAL.
//! Ненулевой End во время удержания только включает attacking, сохраняя фазу,
//! команду и ресурсы. Выпуск округляет hold вверх до половины интервала ДО
//! новых часов и ждёт unsigned start+hold. Свежий путь проверяется повторно;
//! flight записывается до visual1, затем casted и базовый prepared независимо.
//! Контакт ждёт абсолютный start+hold+flight, возвращает движение захваченному U,
//! выполняет сырой Attack без дополнительного допуска, перенос покрытия и End(1).
//! Target не копируется в payload: каждый AI заново получает базовую S.
//!
//! Calculate читает свежую таблицу и лишь один коэффициент по текущему hold.
//! Physical сохраняет MIN→MAX→RNG с первым MIN, а не вторым getter-ом;
//! ELEMENT/SOUL/CCH остаются живыми. Общий оружейный хвост сохраняет RNG,
//! unsigned x87-коэффициент и критическое усечение. RP здесь не начисляется.
//! Отсутствующая таблица сохраняет UNKNOWN/1 и не отменяет сырой контакт.
//! Некорректный нулевой делитель в ресурсах безопасно прекращает соответствующий
//! выпуск/расчёт вместо native деления на ноль; новый коэффициент не выдумывается.
//!
//! AddPoisonState проверяет Cure, покрытие и навык, затем фиксирует свойства
//! и MasterInfo Player со страной 0. End первого SpiderPoison и destructor
//! свежего остатка предшествуют modifier→уровень оружия→CONST→frequency→keep.
//! DWORD-произведение остаётся unsigned до x87 * -0.01 и FISTP dword;
//! новый Begin с живыми часами предшествует append. Покрытие не расходуется.
//! Общие SlotMap/kernel и state-арена заменяют указатели без копирования owners.

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::fightdefense::truncate_original;
use super::heartlessarrow2::check_heartless_cast;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastPathBlock, RangedWeaponKind, check_skill_path, prepare_ranged_weapon_player, terminal,
};
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage, source_master};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const HEARTLESS_ARROW_SKILL_ID: u32 = 0xca;
const DAUB_POISON_SKILL_ID: u32 = 0xdf;
const ACTION_INTERVAL: u32 = 10_009;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTORS: [u32; 4] = [20_003, 20_021, 20_022, 20_023];
const STATE_PERSIST_TIME_MODIFIER: u32 = 10_003;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const POISON_CONSTANT: u32 = 20_010;
const WEAPON_DAMAGE_LEVEL_MODIFIER: u32 = 20_018;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartlessArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    attacking_started: bool,
    skill_casted: bool,
    hold_time_ms: u32,
    missile_flying_time_ms: u32,
}

impl HeartlessArrowExecutionState {
    pub(crate) fn prepare_derived_end(&mut self, argument: i32) -> bool {
        if argument != 0 && self.kernel.stage() == SkillStage::Check && !self.attacking_started {
            self.attacking_started = true;
            return false;
        }
        self.kernel.clear_phase_for_end();
        self.attacking_started = false;
        self.skill_casted = false;
        self.missile_flying_time_ms = 0;
        self.hold_time_ms = 0;
        true
    }

    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            attacking_started: false, skill_casted: false, hold_time_ms: 0, missile_flying_time_ms: 0,
        }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(super) const fn missile_flying_time_ms(&self) -> u32 { self.missile_flying_time_ms }
}

fn progress(game: &CGame, instance: RegisteredSkill) -> Option<&HeartlessArrowExecutionState> {
    game.registered_skill(instance)?.player_state()
}

fn progress_mut(game: &mut CGame, instance: RegisteredSkill) -> Option<&mut HeartlessArrowExecutionState> {
    game.registered_skill_mut(instance)?.player_state_mut()
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let mut attack = AttackInformation::for_master(master);
    if let Some(skill) = game.registered_skill(instance)
        && let Some(properties) = game.skill_base_properties(skill.id(), skill.level())
    {
        attack.skill_id = skill.id();
        attack.skill_level = skill.level() as u8;
        attack.damage_modifier = 0;
        let interval = properties.query_property(ACTION_INTERVAL);
        if interval == 0 { return; }
        let Some(hold) = progress(game, instance).map(|state| state.hold_time_ms) else { return; };
        let tier = hold.wrapping_mul(2) / interval;
        if (1..=4).contains(&tier) {
            let factor = properties.query_property(TARGET_DAMAGE_FACTORS[tier as usize - 1]);
            attack.damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
        }
        attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
        fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::CapturedMinimumAbsoluteRange, &mut attack);
    }
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

pub(crate) fn apply_daub_poison(
    game: &mut CGame, player_id: i32, region_id: i32, target: ShapeIdentity,
    now: &mut dyn FnMut() -> u32,
) {
    let Some(source) = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()))
    else { return; };
    apply_daub_poison_between(game, source, (region_id, target), now);
}

fn apply_daub_poison_between(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    now: &mut dyn FnMut() -> u32,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let source = (user.shape().get_region_id(), user.shape().identity());
    let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
    if sufferer.has_state_by_skill_id(0x131) || !user.has_state_by_skill_id(DAUB_POISON_SKILL_ID) { return; }
    let Some(skill) = user.skill(DAUB_POISON_SKILL_ID, game.skill_factory()) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    if source.1.object_type != 400 { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let previous = sufferer.find_state_position(|state| state.state_id() == SPIDER_POISON_SKILL_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let modifier = properties.query_property(WEAPON_DAMAGE_LEVEL_MODIFIER);
    let Some(weapon_level) = game.find_player(source.1.id)
        .map(|player| player.weapon_damage_level(game.goods_factory())) else { return; };
    let scaled = (weapon_level as u32).wrapping_mul(modifier);
    let constant = properties.query_property(POISON_CONSTANT);
    let negative_bonus = truncate_original(f64::from(scaled) * f64::from(-0.01_f32));
    let hp_loss = constant.wrapping_sub(negative_bonus as u32);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let keep_time_ms = properties.query_property(STATE_PERSIST_TIME_MODIFIER);
    let state = SpiderPoisonState::new(master, keep_time_ms, frequency_ms, hp_loss);
    let _ = begin_primary_spider_poison_state(
        game, target.0, target.1, Some(source), Some(target), state, None, now,
    );
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity);
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let (Some(source), Some(target)) = (source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let targets_self = std::ptr::eq(source, target);
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = (target.shape().get_region_id(), target.shape().identity());
    let player = (source.1.object_type == 400).then_some(source.1.id);
    let target_dead = game.move_shape_health(target.0, target.1) == Some(0);
    if target_dead || targets_self {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player {
            game.send_skill_system_info(player, if target_dead { b"GS0285" } else { b"GS0286" });
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::HeldBow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let interval = properties.query_property(ACTION_INTERVAL);
    let Some(state) = progress(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !state.attacking_started {
        let now = runtime.now_milliseconds();
        let Some(state) = progress_mut(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.hold_time_ms = now.wrapping_sub(state.kernel.started_at_ms());
        if state.hold_time_ms >= interval.wrapping_mul(2) {
            state.hold_time_ms = interval.wrapping_mul(2);
            state.attacking_started = true;
        }
    }
    let Some(state) = progress(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if state.attacking_started && !state.skill_casted {
        let half = interval / 2;
        if half == 0 { return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(state) = progress_mut(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if state.hold_time_ms % half != 0 {
            state.hold_time_ms = (state.hold_time_ms / half).wrapping_add(1).wrapping_mul(half);
        }
        let now = runtime.now_milliseconds();
        let Some(state) = progress(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if now >= state.kernel.started_at_ms().wrapping_add(state.hold_time_ms) {
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let path = game.skill_target_path(skill.lifecycle());
            if !check_skill_path(game, instance, &properties, &path, player,
                CastPathBlock::Named { target, message: b"GS0307" })
            { return terminal(QueuedSkillExecutionState::Rejected); }
            let flight = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
            if let Some(state) = progress_mut(game, instance) { state.missile_flying_time_ms = flight; }
            game.update_registered_skill_visual(instance, 1);
            if let Some(state) = progress_mut(game, instance) {
                state.skill_casted = true;
                state.kernel.lifecycle_mut().mark_prepared();
            }
        }
    }
    if progress(game, instance).is_none_or(|state| !state.skill_casted) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let now = runtime.now_milliseconds();
    let Some(state) = progress(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let deadline = state.kernel.started_at_ms().wrapping_add(state.hold_time_ms).wrapping_add(state.missile_flying_time_ms);
    if now < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    attack(game, instance, source, target, runtime);
    apply_daub_poison_between(game, source, target, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_heartless_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::HeartlessArrow,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            check_heartless_cast(game, instance, original_user, target, RangedWeaponKind::Bow, runtime)
        },
        |dispatch, started| HeartlessArrowExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
