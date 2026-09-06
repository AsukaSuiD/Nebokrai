//! Малая звезда `CLittleStar` (`0x1a4`) для игроков и монстров.
//! Успешный Begin возвращает Begun до первого AI. Расход ресурсов,
//! перемещение и атака остаются у AI после постановки Attack в том же Run;
//! раннее время Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/littlestar.cpp`. Навык после задержки один раз строит
//! прямой путь предельной длины, публикует его конечную клетку и до строгой
//! границы длительности периодически обходит клетки до первой `BLOCK_UNFLY`.
//! Игрок проходит две проверки MP и расходует ману до стартового эффекта;
//! каждая допустимая цель получает ровно один вызов legacy RNG. Монстр
//! использует ту же геометрию и частоту без расхода, относящегося к игроку.
//! Формулы, порядок клеток, применение атак и wire-эффекты принадлежат этому
//! owner-у; `CGame` только разрешает владельцев и доставляет результат.
//! Player `End` возвращает движение, публикует action `3` и завершает
//! `CAttackSkill::End(1)` с единичным оружейным `AfterUseSkill`; этот порядок
//! общий для завершения и отмены, а периодический `Attack` оружие не изнашивает.
//! Стихийная прибавка вычисляется в расширенной точности x87 из целых свойств
//! и сохранённой `f32`-константы, затем усекается к нулю. Player и monster
//! ветви используют абсолютный срок `CSkill::IsRestored`; периодические тики
//! и общая длительность остаются elapsed.
//! Monster End (0x005355F0) освобождает путь до SetMoveable(true), затем
//! публикует action 3 и вызывает CAttackSkill::End; часы reuse читаются после
//! доставки, отдельно от проверки длительности. Обычное завершение сохраняет
//! этот порядок, как и End(4) достигнутого Stiffen через monsterai runtime.
//! Прочие расширенные отмены cast пока не доставляют action 3; они требуют
//! такой же runtime-границы и не покрываются простой политикой снятия движения.
//! Потеря объектной цели не отменяет уже начатый cast: AI (0x00535E34)
//! использует резервные координаты +0x24/+0x28. Объектный Begin обнуляет
//! их (0x005DBDBA), поэтому до построения пути fallback равен (0, 0),
//! а не позиции источника. После выпуска сохранённый путь независим от цели.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER,
    time_reached,
};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_ELEMENT_MODIFIER};
use super::fightdefense::truncate_original;
use super::flash::{cell_views, master_info, target_level};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{
    skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination,
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

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_SKILL_PERSIST_TIME: u32 = 10_007;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const LITTLE_STAR_SKILL_ID: u32 = 0x1a4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLittleStarExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Option<Vec<(i32, i32, u8)>>,
    last_attack_ms: u32,
}
impl PlayerLittleStarExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), path: None, last_attack_ms: 0 }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_little_star_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: LITTLE_STAR_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: LITTLE_STAR_SKILL_ID, .. })
}

fn player_target_position(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: LITTLE_STAR_SKILL_ID, x, y } => Some((x, y)),
        PlayerSkillDispatch::Object { skill_id: LITTLE_STAR_SKILL_ID, target } => {
            game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y))
        }
        _ => None,
    }
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

fn send_player_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    action: u8,
    destination: Option<(i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let Some((x, y)) = destination else { return };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_little_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    send_player_visual(game, player_id, level, 3, None);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(LITTLE_STAR_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_little_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID)
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    let level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(LITTLE_STAR_SKILL_ID));
    finish_player_little_star(game, player_id, level, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

#[allow(clippy::too_many_arguments, reason = "параметры соответствуют подтверждённой формуле навыка")]
fn calculate_player_attack(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = target_level(game, region_id, target)?;
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(
        game.goods_factory(),
        i32::from(target_level),
        divisor,
        floor,
    );
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let modifier = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = (combat.add_element_attack as i32).wrapping_add(random_damage)
        .wrapping_add(minimum).wrapping_add(modifier).max(0);
    Some((master, AttackInformation {
        skill_id: LITTLE_STAR_SKILL_ID,
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
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    }))
}

pub(crate) fn execute_player_little_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_little_star_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(LITTLE_STAR_SKILL_ID), player.mana(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(LITTLE_STAR_SKILL_ID, level) else {
        if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_some() {
            finish_player_little_star(game, player_id, level, ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let frequency = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let persist = properties.query_property(SKILL_USAGE_SKILL_PERSIST_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(LITTLE_STAR_SKILL_ID), reuse, now_ms) {
            send_player_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player_target_position(game, region_id, dispatch).is_none() { return terminal(QueuedSkillExecutionState::Rejected) }
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(LITTLE_STAR_SKILL_ID));
        }
        ai.begin_player_skill_execution(PlayerLittleStarExecutionState::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_some_and(|state| state.kernel().stage() == SkillStage::Begin) {
        let Some((target_x, target_y)) = player_target_position(game, region_id, dispatch) else {
            finish_player_little_star(game, player_id, level, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_player_little_star(game, player_id, level, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_visual(game, player_id, level, 1, None);
        if let Some(state) = ai.player_skill_state_mut::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started = ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).map(|state| state.kernel().started_at_ms()).unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) }
    if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_some_and(|state| state.path.is_none()) {
        let Some((target_x, target_y)) = player_target_position(game, region_id, dispatch) else {
            finish_player_little_star(game, player_id, level, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, Some(maximum_distance));
        if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
        path.truncate(maximum_distance as usize);
        let endpoint = path.last().map(|cell| (cell.0, cell.1)).unwrap_or((target_x, target_y));
        send_player_visual(game, player_id, level, 2, Some(endpoint));
        if let Some(state) = ai.player_skill_state_mut::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID) {
            state.path = Some(path);
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let attack_now = runtime.now_milliseconds();
    if ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).is_some_and(|state| state.attack_due(attack_now, frequency)) {
        let path = ai.player_skill_state::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID).and_then(|state| state.path.as_deref()).unwrap_or_default().to_vec();
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        'cells: for &(x, y, block) in &path {
            if block == BLOCK_UNFLY { break 'cells }
            for view in cell_views(game, region_id, x, y) {
                let target = view.identity;
                if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) { continue }
                let Some(owner) = game.find_player(player_id).map(master_info) else { break 'cells };
                if !game.owned_player_skill_target_attackable(owner, target, region_id) { continue }
                let Some((master, attack)) = calculate_player_attack(
                    game, player_id, region_id, target, level, minimum, maximum, element_modifier, hit_modifier,
                ) else { continue };
                match target.object_type {
                    PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                    MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                    _ => unreachable!(),
                }
            }
        }
        let recorded = runtime.now_milliseconds();
        if let Some(state) = ai.player_skill_state_mut::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID) {
            state.last_attack_ms = recorded;
            if state.kernel().stage() == SkillStage::Calculate { let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); }
        }
    }

    let expiration_now = runtime.now_milliseconds();
    if started.wrapping_add(delay).wrapping_add(persist) < expiration_now {
        if let Some(state) = ai.player_skill_state_mut::<PlayerLittleStarExecutionState>(LITTLE_STAR_SKILL_ID) {
            if state.kernel().stage() == SkillStage::Calculate { let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); }
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_little_star(game, player_id, level, ai, runtime);
        terminal(QueuedSkillExecutionState::Completed)
    } else {
        terminal(QueuedSkillExecutionState::Pending)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LittleStarProgress {
    path: Vec<(i32, i32, u8)>,
    last_attack_ms: u32,
}

impl LittleStarProgress {
    fn new(path: Vec<(i32, i32, u8)>) -> Self {
        Self {
            path,
            last_attack_ms: 0,
        }
    }

    fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }

    fn record_attack(&mut self, now_ms: u32) {
        self.last_attack_ms = now_ms;
    }
}

fn send_start(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
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
    skill_level: u16,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

pub(crate) fn send_end(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок клеток, целей и RNG каждого удара")]
fn attack_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &crate::setup::monsterlist::MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    path: &[(i32, i32, u8)],
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    for &(cell_x, cell_y, block) in path {
        if block == BLOCK_UNFLY {
            break;
        }
        for identity in monster_attack_cell_candidates(game, region, monster_id, cell_x, cell_y) {
            let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
                continue;
            };
            if target.dead
                || target.god
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
            let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
            let span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
                .wrapping_sub(minimum)
                .unsigned_abs()
                .wrapping_add(1) as i32;
            let damage = minimum.wrapping_add(game.skill_random_below(span)).max(0);
            let attack = AttackInformation {
                skill_id: LITTLE_STAR_SKILL_ID,
                skill_level: skill_level as u8,
                attacker_type: MONSTER_TYPE,
                attacker_id: monster_id,
                attacker_team_id: 0,
                attacker_faction_id: 0,
                attacker_union_id: 0,
                hit_modifier: properties
                    .query_property(super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER)
                    as i32,
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
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт длительного навыка")]
pub(crate) fn execute_owned_little_star<Runtime: GameMainLoopRuntime>(
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
                monster.little_star_progress().cloned(),
                monster.skill_last_used_ms(LITTLE_STAR_SKILL_ID),
            ))
        })
    else {
        return false;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    if cast.is_none()
        && !target.as_ref().is_some_and(|target| {
            !target.dead
                && !target.god
                && !target.city_dead
                && owned_monster_attackable(
                    game, region.id, &property, tamed, master, target_identity, target,
                )
        })
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    let Ok(source_x) = source.get_tile_x() else {
        return true;
    };
    let Ok(source_y) = source.get_tile_y() else {
        return true;
    };
    let target_position = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let (target_x, target_y) = target_position.unwrap_or((0, 0));

    if cast.is_none() {
        let trace_target = target.as_ref().map_or_else(
            || MonsterTraceTarget::point(target_x, target_y),
            |target| MonsterTraceTarget::Shape(target.view),
        );
        if !approach_attack_range(
            game,
            region,
            monster_id,
            trace_target,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
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
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(target_identity, LITTLE_STAR_SKILL_ID, skill_level, now_ms);
        }
        let current_source = region.find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape()).unwrap_or(&source);
        send_start(game, region, current_source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение малой звезды проверено выше");
    if cast.dispatch().skill_id != LITTLE_STAR_SKILL_ID || cast.dispatch().target != target_identity {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
        return true;
    }

    let mut progress = if let Some(progress) = progress {
        progress
    } else {
        let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize;
        let mut path = region.straight_skill_path(
            source_x,
            source_y,
            target_x,
            target_y,
            Some(maximum_distance as u32),
        );
        path.truncate(maximum_distance);
        let endpoint = path.last().map(|cell| (cell.0, cell.1)).unwrap_or((target_x, target_y));
        send_fire(game, region, &source, skill_level, endpoint.0, endpoint.1);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        LittleStarProgress::new(path)
    };

    if progress.attack_due(now_ms, properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY)) {
        attack_path(
            game, region, runtime, now_ms, monster_id, skill_level, properties, &property,
            master, tamed, &progress.path, deaths,
        );
        progress.record_attack(runtime.now_milliseconds());
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    let expiration_now_ms = runtime.now_milliseconds();
    let expired = cast.started_at_ms()
        .wrapping_add(delay_ms)
        .wrapping_add(properties.query_property(SKILL_USAGE_SKILL_PERSIST_TIME))
        < expiration_now_ms;
    if expired {
        drop(progress);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.prepare_little_star_end();
        }
        send_end(game, region, &source, skill_level);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
        }
    } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_little_star_progress(Some(progress));
    }
    true
}
