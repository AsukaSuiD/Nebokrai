//! Заряжаемый выстрел `CHeartLessArrow` (`0xCA`).
//! На время применения удара настоящий AI источника опубликован в CPlayer;
//! изменения синхронных callback возвращаются в тот же проход навыка.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/heartlessarrow.cpp`. Модуль-владелец хранит цель, время удержания и
//! полёта, проверяет лук категории `3`, списывает MP до повторной проверки
//! оружия и рассчитывает три типа урона с двумя вызовами генератора MSVCRT. После
//! попадания активный `CDaubPoisonState` (`0xDF`) может заменить канонический
//! `SpiderPoisonState` цели. `CGame` разрешает независимых владельцев,
//! применяет рассчитанную атаку и выполняет доставку.
//! Ненулевой End (клиентский 1 или Stiffen 4) во время удержания только
//! выпускает стрелу: 0x00591AD9 проверяет аргумент на ноль, а 0x00591AEA
//! возвращается после установки attacking_started без CSkill::End. После
//! выпуска он завершает `CAttackSkill::End(1)` с единичным оружейным
//! `AfterUseSkill`. Отказная отмена использует `End(0)` без износа,
//! cooldown и применения отложенной атаки. Cooldown использует абсолютный
//! срок `CSkill::IsRestored`; удержание и полёт сохраняют elapsed-семантику.
//! Обычный хвост End (0x00591AF7) обнуляет condition/attacking/skill-casted,
//! missile/hold, но не уничтожает registered payload. PDB m_bAvailable
//! +0x50 — derived поле, не базовое CSkill::m_bAvailable +0x3C.
//! Процентный damage factor сохраняется в `f32` только после расширенного
//! x87-умножения; критический урон усекается к нулю при записи в `i32`.
//! При переносе яда DWORD-произведение уровня оружия и модификатора остаётся
//! точным unsigned-значением в x87 вплоть до умножения на `-0.01f` и прямого
//! `FISTP dword`; промежуточного сохранения в `f32` нет.
//! После эффекта выпуска (0x005931BE) устанавливается общий prepared-флаг.
//! Следующий OnFighting переносит исполнение в фон без End; полёт продолжает
//! тот же owner с исходной целью и временем. Это не attacking_started,
//! который лишь прекращает удержание, и не разрешение движения при попадании.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.
//! Explicit End возвращает отдельный Released для выпуска удерживаемой стрелы:
//! coordinator не снимает команду и не освобождает payload этой ветви. Ended
//! разрешает общий терминальный сброс исходного экземпляра после callbacks.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::spiderpoison::{install_spider_poison_state, target_has_cure};
use super::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkillEnd;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const HEARTLESS_ARROW_SKILL_ID: u32 = 0xca;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const DAUB_POISON_SKILL_ID: u32 = 0xdf;
const USER_MP_LOSE: u32 = 2;
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
    target: ShapeIdentity,
    condition_checked: bool,
    attacking_started: bool,
    hold_time_ms: u32,
    missile_flying_time_ms: u32,
}

impl HeartlessArrowExecutionState {
    pub(crate) fn prepare_derived_end(&mut self, argument: i32) -> bool {
        if argument != 0 && self.condition_checked && !self.attacking_started {
            self.attacking_started = true;
            return false;
        }
        true
    }

    pub(crate) fn clear_end_paths(&mut self) {
        self.condition_checked = false;
        self.attacking_started = false;
        self.missile_flying_time_ms = 0;
        self.hold_time_ms = 0;
    }

    fn begin(dispatch: PlayerSkillDispatch, target: ShapeIdentity, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), target, condition_checked: false, attacking_started: false, hold_time_ms: 0, missile_flying_time_ms: 0 }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_heartless_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, HEARTLESS_ARROW_SKILL_ID, runtime);
}

fn abort_player_heartless_arrow(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_or_release_player_heartless_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> Option<RegisteredSkillEnd> {
    let Some((dispatch, completes)) = game.player_skill_state_mut::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).map(|state| (
        state.kernel().dispatch(), state.prepare_derived_end(1),
    )) else { return None };
    if !completes {
        tracing::trace!(player_id, "ненулевой End выпустил удерживаемую стрелу");
        return Some(RegisteredSkillEnd::Released);
    }
    finish_player_heartless_arrow(game, player_id, player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed);
    Some(RegisteredSkillEnd::Ended)
}

pub(crate) fn cancel_player_heartless_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_heartless_arrow(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: 0, permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}

fn target_snapshot(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(i32, i32, bool)> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.is_dead()))),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; Some((monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?, monster.hit_points() == 0)) }),
        _ => None,
    }
}

fn target_name<'a>(game: &'a CGame, region_id: i32, target: ShapeIdentity) -> &'a [u8] {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::player_name).unwrap_or_default(),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).map(CMonster::display_name).unwrap_or_default(),
        _ => &[],
    }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3)
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(HEARTLESS_ARROW_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(game: &mut CGame, player_id: i32, level: i32, target: ShapeIdentity, target_x: i32, target_y: i32, flying_time_ms: u32) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(HEARTLESS_ARROW_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn damage_factor(hold_time_ms: u32, interval_ms: u32, factors: [u32; 4]) -> f32 {
    if interval_ms == 0 { return 1.0; }
    let value = match hold_time_ms.wrapping_mul(2) / interval_ms { 1 => factors[0], 2 => factors[1], 3 => factors[2], 4 => factors[3], _ => return 1.0 };
    (f64::from(value) * f64::from(0.01_f32)) as f32
}

fn calculate_attack(game: &mut CGame, player_id: i32, level: i32, hit_modifier: i32, factor: f32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let master = master_info(player);
    let combat = player.combat_properties();
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let delta = maximum.wrapping_sub(minimum);
    let width = if delta < 0 { delta.wrapping_neg() } else { delta }.wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(width));
    let mut attack = AttackInformation {
        skill_id: HEARTLESS_ARROW_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier, damage_factor: factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); }
    }
    Some((master, attack))
}

pub(crate) fn apply_daub_poison(game: &mut CGame, player_id: i32, region_id: i32, target: ShapeIdentity, now_ms: u32) {
    let Some(master) = game.find_player(player_id).map(master_info) else { return };
    apply_daub_poison_with_master(game, player_id, master, region_id, target, now_ms);
}

pub(crate) fn apply_daub_poison_with_master(game: &mut CGame, player_id: i32, master: MasterInfo, region_id: i32, target: ShapeIdentity, now_ms: u32) {
    let Some((level, weapon_level)) = game.find_player(player_id).and_then(|player| {
        if !player.has_state_by_skill_id(DAUB_POISON_SKILL_ID) { return None; }
        let level = player.learned_skill_level(DAUB_POISON_SKILL_ID, game.skill_factory());
        (level != 0).then(|| (level, player.weapon_damage_level(game.goods_factory())))
    }) else { return };
    let Some(properties) = game.skill_base_properties(DAUB_POISON_SKILL_ID, level) else { return };
    let keep_time_ms = properties.query_property(STATE_PERSIST_TIME_MODIFIER);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let constant = properties.query_property(POISON_CONSTANT);
    let modifier = properties.query_property(WEAPON_DAMAGE_LEVEL_MODIFIER);
    let scaled = (weapon_level as u32).wrapping_mul(modifier);
    let negative_bonus = truncate_original(
        f64::from(scaled) * f64::from(-0.01_f32),
    );
    let hp_loss = constant.wrapping_sub(negative_bonus as u32);
    let Some(mut owner) = game.take_region_owner(region_id) else { return };
    if !target_has_cure(game, owner.base(), target) {
        install_spider_poison_state(game, owner.base_mut(), target, SpiderPoisonState::new(master, now_ms, keep_time_ms, frequency_ms, hp_loss), now_ms);
    }
    game.restore_region_owner(owner);
}

pub(crate) const fn is_heartless_arrow_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::Object { skill_id: HEARTLESS_ARROW_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } })
}

pub(crate) fn execute_player_heartless_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_heartless_arrow_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let PlayerSkillDispatch::Object { target, .. } = dispatch else { unreachable!() };
    let Some((region_id, level, source_x, source_y)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(HEARTLESS_ARROW_SKILL_ID, game.skill_factory()), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(HEARTLESS_ARROW_SKILL_ID, level) else {
        if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_some() { abort_player_heartless_arrow(game, player_id); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let action_interval_ms = properties.query_property(ACTION_INTERVAL);
    let missile_step_ms = properties.query_property(MISSILE_FLYING_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let factors = TARGET_DAMAGE_FACTORS.map(|id| properties.query_property(id));
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_none() {
        let Some((target_x, target_y, _)) = target_snapshot(game, region_id, target) else { game.send_base_magic_failure(player_id, 10); game.send_skill_system_info(player_id, b"GS0286"); return terminal(QueuedSkillExecutionState::Rejected) };
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, HEARTLESS_ARROW_SKILL_ID),
            reuse_delay_ms,
            runtime.now_milliseconds(),
        ) {
            game.send_base_magic_failure(player_id, 0x0d); game.send_skill_system_info(player_id, b"GS0278"); return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize { game.send_base_magic_failure(player_id, 0x0b); game.send_skill_system_info(player_id, b"GS0290"); return terminal(QueuedSkillExecutionState::Rejected); }
        if path.iter().any(|cell| cell.2 == 2) { game.send_base_magic_failure(player_id, 0x0f); game.send_skill_system_info_with_text(player_id, b"GS0291", target_name(game, region_id, target)); return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0297"); return terminal(QueuedSkillExecutionState::Rejected); }
        if mp_loss != 0 && (player.mana().wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); return terminal(QueuedSkillExecutionState::Rejected); }
        let started_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(HEARTLESS_ARROW_SKILL_ID)); }
        game.begin_player_skill_execution(player_id, HeartlessArrowExecutionState::begin(dispatch, target, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_none_or(|state| state.kernel().dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }

    let target = game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().expect("выполнение выстрела создано").target;
    let Some((target_x, target_y, dead)) = target_snapshot(game, region_id, target) else { game.send_base_magic_failure(player_id, 10); game.send_skill_system_info(player_id, b"GS0286"); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    if dead || (target.object_type == PLAYER_TYPE && target.id == player_id) { game.send_base_magic_failure(player_id, 10); game.send_skill_system_info(player_id, if dead { b"GS0285" } else { b"GS0286" }); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }

    if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_some_and(|state| !state.condition_checked) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0292"); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); }
        send_start(game, player_id, level);
        if let Some(state) = game.player_skill_state_mut::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID) { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }

    let now_ms = runtime.now_milliseconds();
    if let Some(state) = game.player_skill_state_mut::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID) {
        if !state.attacking_started {
            state.hold_time_ms = now_ms.wrapping_sub(state.kernel().started_at_ms());
            if state.hold_time_ms >= action_interval_ms.wrapping_mul(2) { state.hold_time_ms = action_interval_ms.wrapping_mul(2); state.attacking_started = true; }
        }
    }
    if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_some_and(|state| !state.attacking_started) { return terminal(QueuedSkillExecutionState::Pending); }

    if game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().is_some_and(|state| !state.kernel().is_prepared()) {
        let state = game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().expect("удержание выстрела существует");
        let unit = action_interval_ms / 2;
        let hold_time_ms = if unit != 0 && state.hold_time_ms % unit != 0 { state.hold_time_ms.wrapping_div(unit).wrapping_add(1).wrapping_mul(unit) } else { state.hold_time_ms };
        if !time_reached(now_ms, state.kernel().started_at_ms(), hold_time_ms) { return terminal(QueuedSkillExecutionState::Pending); }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize { game.send_base_magic_failure(player_id, 0x0b); game.send_skill_system_info(player_id, b"GS0290"); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if path.iter().any(|cell| cell.2 == 2) { game.send_base_magic_failure(player_id, 0x0f); game.send_skill_system_info_with_text(player_id, b"GS0307", target_name(game, region_id, target)); abort_player_heartless_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        let flying_time_ms = missile_step_ms.wrapping_mul(path.len() as u32);
        send_fire(game, player_id, level, target, target_x, target_y, flying_time_ms);
        if let Some(state) = game.player_skill_state_mut::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID) { state.hold_time_ms = hold_time_ms; state.missile_flying_time_ms = flying_time_ms; state.kernel_mut().mark_prepared(); let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); }
    }

    let state = game.player_skill_state::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID).copied().expect("снаряд выстрела сохраняется до попадания");
    if !time_reached(runtime.now_milliseconds(), state.kernel().started_at_ms(), state.hold_time_ms.wrapping_add(state.missile_flying_time_ms)) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    let factor = damage_factor(state.hold_time_ms, action_interval_ms, factors);
    if let Some((master, attack)) = calculate_attack(game, player_id, level, hit_modifier, factor) {
        match target.object_type { PLAYER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime)), MONSTER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime)), _ => {} }
    }
    apply_daub_poison(game, player_id, region_id, target, runtime.now_milliseconds());
    if let Some(state) = game.player_skill_state_mut::<HeartlessArrowExecutionState>(player_id, HEARTLESS_ARROW_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_heartless_arrow(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
