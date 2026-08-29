//! Владелец общего пошагового снаряда и конкретной семантики энергетического снаряда.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` и исходный владелец
//! `GameServer/appserver/skills/energybolt.cpp` подтверждают общий жизненный цикл
//! игрока и монстра: повторную проверку и расход MP игрока, задержку до выстрела,
//! принудительный путь длиной `SKILL_USAGE_TARGET_MAX_DISTANCE`, один шаг за
//! единицу времени полёта, живое перечитывание блока клетки и обход области
//! сначала по X, затем по Y. Первый успешный удар либо `BLOCK_UNFLY` посылает
//! промежуточное завершение; на следующем такте владелец посылает его повторно.
//! Для игрока первая допустимая цель потребляет `CSoulCollectState`, а формула
//! сохраняет единственный вызов RNG, добавку стихии и коэффициент оружия. Для
//! монстра недостигнутые добавки игрока не выдумываются. `CGame` только разрешает
//! владельцев, применяет защиту и смерть и доставляет уже построенные пакеты.
//! Три player-варианта используют общий `End`: очистку progress, возврат
//! движения и `CAttackSkill::End(1)` с cooldown конкретного идентификатора.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_ELEMENT_MODIFIER};
use super::flash::{cell_views, master_info, target_level};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::soulcollectstate::send_soul_collect_state_visual;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const ENERGY_BOLT_SKILL_ID: u32 = 0x1a0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PathProjectileSpec {
    skill_id: u32,
    wide_scope_level: u16,
    wide_scope_from_third: bool,
    initial_position: usize,
}

impl PathProjectileSpec {
    pub(crate) const fn new(
        skill_id: u32,
        wide_scope_level: u16,
        wide_scope_from_third: bool,
        initial_position: usize,
    ) -> Self {
        Self {
            skill_id,
            wide_scope_level,
            wide_scope_from_third,
            initial_position,
        }
    }

    const fn uses_weapon_factor(self) -> bool {
        self.skill_id != super::zombieclaw::ZOMBIE_CLAW_SKILL_ID
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathProjectileProgress {
    destination_x: i32,
    destination_y: i32,
    path: Vec<(i32, i32, u8)>,
    current_position: usize,
    end_x: i32,
    end_y: i32,
    visual_target: Option<ShapeIdentity>,
    missile_flying_time_ms: u32,
    fired: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPathProjectileExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    progress: PathProjectileProgress,
}

impl PlayerPathProjectileExecutionState {
    fn begin(
        dispatch: PlayerSkillDispatch,
        now_ms: u32,
        destination: (i32, i32),
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            progress: PathProjectileProgress::new(destination.0, destination.1),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn skill_id(&self) -> u32 {
        match self.kernel.dispatch() {
            PlayerSkillDispatch::SelfTarget { skill_id, .. }
            | PlayerSkillDispatch::Point { skill_id, .. }
            | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
        }
    }
}

impl PathProjectileProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            path: Vec::new(),
            current_position: 0,
            end_x: 0,
            end_y: 0,
            visual_target: None,
            missile_flying_time_ms: 0,
            fired: false,
        }
    }

    const fn destination(&self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    const fn fired(&self) -> bool {
        self.fired
    }

    fn fire(
        &mut self,
        path: Vec<(i32, i32, u8)>,
        missile_unit_ms: u32,
        initial_position: usize,
    ) {
        let unfly = path.iter().position(|cell| cell.2 == BLOCK_UNFLY);
        let end_index = unfly.unwrap_or(path.len());
        if let Some(&(x, y, _)) = unfly
            .and_then(|index| path.get(index))
            .or_else(|| path.last())
        {
            self.end_x = x;
            self.end_y = y;
        }
        self.missile_flying_time_ms = missile_unit_ms.wrapping_mul(end_index as u32);
        self.path = path;
        self.current_position = initial_position;
        self.visual_target = None;
        self.fired = true;
    }

    fn current_cell(&self) -> Option<(i32, i32)> {
        self.path
            .get(self.current_position)
            .map(|&(x, y, _)| (x, y))
    }

    fn advance(&mut self) {
        self.current_position = self.current_position.wrapping_add(1);
    }

    fn finish_after_collision(&mut self) {
        self.current_position = self.path.len().wrapping_add(1);
    }
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_path_projectile_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    matches!(
        skill_id,
        ENERGY_BOLT_SKILL_ID
            | super::zombieclaw::ZOMBIE_CLAW_SKILL_ID
            | super::snakebolt::SNAKE_BOLT_SKILL_ID
    )
}
fn player_destination(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn player_object_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        _ => None,
    }
}

fn send_player_projectile_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

fn send_player_projectile_start(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_projectile_fire(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    target: Option<ShapeIdentity>,
    progress: &PathProjectileProgress,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(progress.end_x);
    message.add_long(progress.end_y);
    message.add_ulong(progress.missile_flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_projectile_end(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    progress: &PathProjectileProgress,
) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(3);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    message.add_long(progress.end_x);
    message.add_long(progress.end_y);
    message.add_long(progress.visual_target.map_or(0, |identity| identity.object_type));
    message.add_long(progress.visual_target.map_or(0, |identity| identity.id));
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_path_projectile_used(skill_id, now_ms);
    });
}

pub(crate) fn cancel_player_path_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some((dispatch, skill_id)) = player_ai
        .path_projectile()
        .map(|state| (state.kernel().dispatch(), state.skill_id()))
    else {
        return false;
    };
    finish_player_projectile(game, player_id, skill_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют формулу конкретного projectile-owner-а")]
fn calculate_player_projectile_attack(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    spec: PathProjectileSpec,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
    souls: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master, weapon_level) = {
        let player = game.find_player(player_id)?;
        (
            player.combat_properties(),
            master_info(player),
            player.weapon_damage_level(game.goods_factory()),
        )
    };
    let damage_factor = if spec.uses_weapon_factor() {
        let target_level = target_level(game, region_id, target)?;
        let (divisor, floor) = game.globe_setup().weapon_damage_factors();
        let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
        (if divisor == 0.0 { 1.0 } else { delta as f32 / divisor }).min(1.0).max(floor)
    } else {
        1.0
    };
    let span = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random_damage = game.skill_random_below(span);
    let element_bonus = (element_modifier as f32 * 0.01 * combat.element_modify as f32) as i32;
    let base_damage = (combat.add_element_attack as i32)
        .wrapping_add(minimum)
        .wrapping_add(random_damage)
        .wrapping_add(element_bonus);
    let damage = (base_damage as f32 * (1.0 + 0.7 * souls as f32)) as i32;
    Some((master, AttackInformation {
        skill_id: spec.skill_id,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage.max(0), mp_damage: 0 }],
    }))
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок scope, целей и потребления душ")]
fn attack_player_projectile_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    spec: PathProjectileSpec,
    level: i32,
    center_x: i32,
    center_y: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
    progress: &mut PathProjectileProgress,
    runtime: &mut Runtime,
) -> bool {
    let wide_scope = if spec.wide_scope_from_third { level > 2 } else { level == i32::from(spec.wide_scope_level) };
    let radius = if wide_scope { 1 } else { 0 };
    let mut attacked = Vec::new();
    let mut did_attack = false;
    for offset_x in -radius..=radius {
        for offset_y in -radius..=radius {
            let cell_x = center_x.wrapping_add(offset_x);
            let cell_y = center_y.wrapping_add(offset_y);
            for view in cell_views(game, region_id, cell_x, cell_y) {
                let target = view.identity;
                if (target.object_type == PLAYER_TYPE && target.id == player_id)
                    || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || attacked.contains(&target)
                {
                    continue;
                }
                if cell_x == center_x && cell_y != 0 && progress.visual_target.is_none() {
                    progress.visual_target = Some(target);
                }
                let Some(owner) = game.find_player(player_id).map(master_info) else { return did_attack };
                if !game.owned_player_skill_target_attackable(owner, target, region_id) { continue }
                let soul = game.find_player_mut(player_id).and_then(CPlayer::take_soul_collect_state);
                let souls = soul.map_or(0, |state| state.souls());
                if let Some(state) = soul {
                    let Some((tile_x, tile_y)) = game.find_player(player_id).and_then(|player| {
                        Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
                    }) else { return did_attack };
                    send_soul_collect_state_visual(
                        game,
                        region_id,
                        ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: Default::default() },
                        tile_x,
                        tile_y,
                        state,
                        false,
                    );
                }
                let Some((master, attack)) = calculate_player_projectile_attack(
                    game, player_id, region_id, target, spec, level, minimum, maximum,
                    element_modifier, hit_modifier, souls,
                ) else { continue };
                match target.object_type {
                    PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                    MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                    _ => unreachable!(),
                }
                attacked.push(target);
                did_attack = true;
            }
        }
    }
    did_attack
}

pub(crate) fn execute_player_path_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    spec: PathProjectileSpec,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_path_projectile_dispatch(dispatch) || spec.skill_id != match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    } {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, level, mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(spec.skill_id), player.mana(),
    ))) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(spec.skill_id, level) else {
        if ai.path_projectile().is_some() {
            finish_player_projectile(game, player_id, spec.skill_id, ai, runtime);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let missile_unit = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.path_projectile().is_none() {
        let now_ms = runtime.now_milliseconds();
        let self_target = player_object_target(dispatch)
            .is_some_and(|target| target.object_type == PLAYER_TYPE && target.id == player_id);
        if self_target {
            send_player_projectile_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if ai.path_projectile_last_used_ms(spec.skill_id) != 0
            && !time_reached(now_ms, ai.path_projectile_last_used_ms(spec.skill_id), reuse)
        {
            send_player_projectile_failure(game, player_id, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(destination) = player_destination(game, region_id, dispatch) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let initial_path = game.base_magic_path(
            region_id, source_x, source_y, destination.0, destination.1, None,
        );
        if maximum_distance != 0 && initial_path.len() > maximum_distance as usize {
            send_player_projectile_failure(game, player_id, 0x0b);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_projectile_failure(game, player_id, 7);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(spec.skill_id));
        }
        ai.begin_path_projectile(PlayerPathProjectileExecutionState::begin(
            dispatch, now_ms, destination,
        ));
    } else if ai.path_projectile().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    if ai.path_projectile().is_some_and(|state| state.kernel().stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_projectile_failure(game, player_id, 7);
            finish_player_projectile(game, player_id, spec.skill_id, ai, runtime);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let destination = ai.path_projectile()
            .map(|state| state.progress.destination())
            .expect("projectile execution хранит координаты назначения");
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, destination.0, destination.1,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_projectile_start(game, player_id, spec.skill_id, level);
        if let Some(state) = ai.path_projectile_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started = ai.path_projectile()
        .map(|state| state.kernel().started_at_ms())
        .unwrap_or_default();
    if ai.path_projectile().is_some_and(|state| !state.progress.fired()) {
        if !time_reached(runtime.now_milliseconds(), started, delay) {
            return player_terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
        let destination = ai.path_projectile()
            .map(|state| state.progress.destination())
            .expect("projectile execution хранит координаты назначения");
        let forced_length = (maximum_distance != 0).then_some(maximum_distance);
        let path = game.base_magic_path(
            region_id, source_x, source_y, destination.0, destination.1, forced_length,
        );
        if maximum_distance != 0 && path.len() > maximum_distance.wrapping_add(1) as usize {
            send_player_projectile_failure(game, player_id, 0x0b);
            finish_player_projectile(game, player_id, spec.skill_id, ai, runtime);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let target = player_object_target(dispatch);
        if let Some(state) = ai.path_projectile_mut() {
            state.progress.fire(path, missile_unit, spec.initial_position);
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
        let progress = ai.path_projectile().map(|state| state.progress.clone())
            .expect("projectile progress создан перед wire-эффектом");
        send_player_projectile_fire(game, player_id, spec.skill_id, level, target, &progress);
    }

    let mut progress = ai.path_projectile().map(|state| state.progress.clone())
        .expect("projectile execution сохраняется до завершения");
    let due = delay.wrapping_add(missile_unit.wrapping_mul(progress.current_position as u32));
    if !time_reached(runtime.now_milliseconds(), started, due) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    if progress.current_position >= progress.path.len() {
        send_player_projectile_end(game, player_id, spec.skill_id, level, &progress);
        if let Some(state) = ai.path_projectile_mut() {
            if state.kernel().stage() == SkillStage::Calculate {
                let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
            }
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_projectile(game, player_id, spec.skill_id, ai, runtime);
        return player_terminal(QueuedSkillExecutionState::Completed);
    }

    let Some((cell_x, cell_y)) = progress.current_cell() else {
        return player_terminal(QueuedSkillExecutionState::Pending);
    };
    progress.end_x = cell_x;
    progress.end_y = cell_y;
    match game.find_region(region_id).map_or(BLOCK_UNFLY, |owner| owner.base().skill_cell_block(cell_x, cell_y)) {
        BLOCK_SHAPE => {
            if attack_player_projectile_scope(
                game, player_id, region_id, spec, level, cell_x, cell_y, minimum, maximum,
                element_modifier, hit_modifier, &mut progress, runtime,
            ) {
                send_player_projectile_end(game, player_id, spec.skill_id, level, &progress);
                progress.finish_after_collision();
                if let Some(state) = ai.path_projectile_mut() { state.progress = progress; }
                return player_terminal(QueuedSkillExecutionState::Pending);
            }
        }
        BLOCK_UNFLY => {
            send_player_projectile_end(game, player_id, spec.skill_id, level, &progress);
            progress.current_position = progress.path.len();
        }
        _ => {}
    }
    progress.advance();
    if let Some(state) = ai.path_projectile_mut() {
        state.progress = progress;
        if state.kernel().stage() == SkillStage::Calculate {
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    player_terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_energy_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_path_projectile(
        game,
        player_id,
        dispatch,
        PathProjectileSpec::new(ENERGY_BOLT_SKILL_ID, 1, false, 1),
        ai,
        runtime,
    )
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_id: u32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_id: u32,
    skill_level: u16,
    target: Option<ShapeIdentity>,
    destination: (i32, i32),
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(destination.0);
    message.add_long(destination.1);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_end(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_id: u32,
    skill_level: u16,
    progress: &PathProjectileProgress,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    message.add_long(progress.end_x);
    message.add_long(progress.end_y);
    message.add_long(
        progress
            .visual_target
            .map_or(0, |identity| identity.object_type),
    );
    message.add_long(progress.visual_target.map_or(0, |identity| identity.id));
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок scope, формулу и последствия каждого удара")]
fn attack_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    spec: PathProjectileSpec,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    center_x: i32,
    center_y: i32,
    progress: &mut PathProjectileProgress,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let wide_scope = if spec.wide_scope_from_third {
        !matches!(skill_level, 1 | 2)
    } else {
        skill_level == spec.wide_scope_level
    };
    let scope_radius = wide_scope
        .then_some(1)
        .unwrap_or(0);
    let mut attacked = Vec::new();
    let mut did_attack = false;
    for offset_x in -scope_radius..=scope_radius {
        for offset_y in -scope_radius..=scope_radius {
            let cell_x = center_x.wrapping_add(offset_x);
            let cell_y = center_y.wrapping_add(offset_y);
            for identity in monster_attack_cell_candidates(
                game, region, monster_id, cell_x, cell_y,
            ) {
                if attacked.contains(&identity) {
                    continue;
                }
                let Some(target) = resolve_owned_monster_attack_target(game, region, identity)
                else {
                    continue;
                };
                if target.dead {
                    continue;
                }
                if cell_x == center_x && cell_y != 0 && progress.visual_target.is_none() {
                    progress.visual_target = Some(identity);
                }
                if target.god
                    || target.city_dead
                    || !owned_monster_attackable(
                        game,
                        region.id,
                        attacker_property,
                        attacker_tamed,
                        attacker_master,
                        identity,
                        &target,
                    )
                {
                    continue;
                }

                // Для monster-owner-а `GetAddElementAtk` и `ElementModify`
                // равны нулю; недостигнутый `CSoulCollectState` не выдумывается.
                let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
                let span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
                    .wrapping_sub(minimum)
                    .unsigned_abs()
                    .wrapping_add(1) as i32;
                let damage = minimum.wrapping_add(game.skill_random_below(span)).max(0);
                let attack = AttackInformation {
                    skill_id: spec.skill_id,
                    skill_level: skill_level as u8,
                    attacker_type: MONSTER_TYPE,
                    attacker_id: monster_id,
                    attacker_team_id: 0,
                    attacker_faction_id: 0,
                    attacker_union_id: 0,
                    hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
                    damage_factor: 1.0,
                    damage_modifier: 0,
                    critical: false,
                    blast_attack: false,
                    full_miss: 0,
                    damages: vec![AttackPower {
                        kind: AttackPowerType::Element,
                        hp_damage: damage,
                        mp_damage: 0,
                    }],
                };
                let attack = defend_owned_monster_attack(
                    game,
                    identity,
                    target.mana,
                    target.war_soul_mana,
                    target.player_properties,
                    target.monster_properties,
                    attack,
                );
                apply_owned_monster_attack_hit(
                    game,
                    region,
                    runtime,
                    now_ms,
                    monster_id,
                    attacker_master,
                    identity,
                    &target.shape,
                    target.health,
                    target.mana,
                    target.master,
                    target.monster_property,
                    target.tamed,
                    target.carriage,
                    attack,
                    deaths,
                );
                attacked.push(identity);
                did_attack = true;
            }
        }
    }
    did_attack
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет owner, путь и текущий такт многоцелевого полёта")]
pub(crate) fn execute_owned_path_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    spec: PathProjectileSpec,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((
        source,
        property,
        master,
        tamed,
        attack_interval_ms,
        cast,
        progress,
        last_used_ms,
    )) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.master_info(),
                monster.is_tamed(),
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.path_projectile_progress().cloned(),
                monster.skill_last_used_ms(spec.skill_id),
            ))
        })
    else {
        return false;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let live_destination = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some(destination) = live_destination
        .or_else(|| progress.as_ref().map(PathProjectileProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            destination.0,
            destination.1,
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return true;
            }
        }
        if last_used_ms != 0
            && !time_reached(
                now_ms,
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            )
        {
            return true;
        }
        let initial_path = region.straight_skill_path(
            source_x, source_y, destination.0, destination.1, None,
        );
        if maximum_distance != 0 && initial_path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(target_identity, spec.skill_id, skill_level, now_ms);
            monster.set_path_projectile_progress(PathProjectileProgress::new(
                destination.0,
                destination.1,
            ));
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_start(game, region, source, spec.skill_id, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение пошагового снаряда проверено выше");
    if cast.dispatch().skill_id != spec.skill_id {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let missile_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        let forced_length = (maximum_distance != 0).then_some(maximum_distance);
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            forced_length,
        );
        if maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        progress.fire(path, missile_unit_ms, spec.initial_position);
        send_fire(
            game,
            region,
            &source,
            spec.skill_id,
            skill_level,
            target.as_ref().map(|_| target_identity),
            destination,
            progress.missile_flying_time_ms,
        );
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_path_projectile_progress(progress.clone());
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let due_ms = delay_ms.wrapping_add(
        missile_unit_ms.wrapping_mul(progress.current_position as u32),
    );
    if !time_reached(now_ms, cast.started_at_ms(), due_ms) {
        return true;
    }
    if progress.current_position >= progress.path.len() {
        send_end(game, region, &source, spec.skill_id, skill_level, &progress);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    let Some((cell_x, cell_y)) = progress.current_cell() else {
        return true;
    };
    progress.end_x = cell_x;
    progress.end_y = cell_y;
    match region.skill_cell_block(cell_x, cell_y) {
        BLOCK_SHAPE => {
            if attack_scope(
                game,
                region,
                runtime,
                now_ms,
                monster_id,
                spec,
                skill_level,
                properties,
                &property,
                master,
                tamed,
                cell_x,
                cell_y,
                &mut progress,
                deaths,
            ) {
                send_end(game, region, &source, spec.skill_id, skill_level, &progress);
                progress.finish_after_collision();
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    monster.set_path_projectile_progress(progress);
                }
                return true;
            }
        }
        BLOCK_UNFLY => {
            send_end(game, region, &source, spec.skill_id, skill_level, &progress);
            progress.current_position = progress.path.len();
        }
        _ => {}
    }
    progress.advance();
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_path_projectile_progress(progress);
    }
    true
}

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn execute_owned_energy_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    execute_owned_path_projectile(
        game,
        region,
        monster_id,
        target_identity,
        PathProjectileSpec::new(ENERGY_BOLT_SKILL_ID, 1, false, 1),
        skill_level,
        properties,
        now_ms,
        runtime,
        deaths,
    )
}
