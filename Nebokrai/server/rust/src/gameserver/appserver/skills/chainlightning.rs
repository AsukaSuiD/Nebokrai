//! Цепная молния `CChainLightning` (`0x13E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/chainlightning.cpp`. Первый такт поворачивает владельца,
//! строит и ограничивает путь, списывает MP, публикует начало и последовательно
//! атакует уникальные фигуры до первого блока `2`. Навык завершается только
//! после строгой границы `SKILL_USAGE_ACTION_INTERVAL`; формула сохраняет два
//! вызова генератора MSVCRT на каждую рассчитанную атаку. `CGame` разрешает
//! независимых владельцев и применяет уже рассчитанные результаты. Собственный
//! `End` очищает накопленный путь, возвращает движение, выполняет общий
//! оружейный `AfterUseSkill` и фиксирует время
//! восстановления; тот же ненулевой хвост используется при отказе после
//! `Begin` и клиентской отмене.
//! Reuse использует exact `CSkill::IsRestored`; action interval — elapsed.
//! Element modifier вычисляется в расширенной точности x87 из целых свойств и
//! сохранённой `f32`-константы; он и критический множитель усекаются к нулю
//! перед `int`.
//! End (0x00598EE0) возвращает движение до AfterUseSkill (0x0053CF30).
//! Общий callback игрока +0x158 пуст и не пересчитывает свойства.

use super::baseattack::{SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const CHAIN_LIGHTNING_SKILL_ID: u32 = 0x13e;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const ACTION_INTERVAL: u32 = 10_009;
const TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChainLightningExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    fallback_x: i32,
    fallback_y: i32,
    attacked: bool,
}

impl ChainLightningExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32, x: i32, y: i32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            fallback_x: x,
            fallback_y: y,
            attacked: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) const fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn finish_player_chain_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    super::baseattack::finish_delayed_base_attack(
        game, player_id, player_ai, runtime,
        |ai, now_ms| ai.mark_skill_used(CHAIN_LIGHTNING_SKILL_ID, now_ms),
    );
}

pub(crate) fn cancel_player_chain_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .chain_lightning()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_chain_lightning(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn dispatch_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
                let view = game.base_magic_target_view(region_id, target)?;
                Some((view.tile_x, view.tile_y))
            }
        _ => None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        2 => game.send_skill_system_info(player_id, b"GS0298"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    action: u8,
    path_end: Option<(i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(CHAIN_LIGHTNING_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let (x, y) = path_end.unwrap_or_default();
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .and_then(|monster| monster.base_property_key())
            .and_then(|key| game.find_monster_property_by_origin_name(key))
            .map(|property| property.level as u8),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments, reason = "аргументы соответствуют полям исходной формулы")]
fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
    damage_modifier: i32,
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
    let width_delta = maximum.wrapping_sub(minimum);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let element_bonus = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = element_bonus
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(minimum);
    let mut attack = AttackInformation {
        skill_id: CHAIN_LIGHTNING_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage.max(0), mp_damage: 0 }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    Some((master, attack))
}

pub(crate) const fn is_chain_lightning_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: CHAIN_LIGHTNING_SKILL_ID, .. }
        | PlayerSkillDispatch::Object {
            skill_id: CHAIN_LIGHTNING_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        }
    )
}

pub(crate) fn execute_player_chain_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_chain_lightning_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, level, source_x, source_y)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.learned_skill_level(CHAIN_LIGHTNING_SKILL_ID),
        player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CHAIN_LIGHTNING_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let action_interval_ms = properties.query_property(ACTION_INTERVAL);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_modifier = properties.query_property(TARGET_FINAL_DAMAGE_MODIFIER) as i32;

    if player_ai.chain_lightning().is_none() {
        if !skill_is_restored(
            player_ai.skill_last_used_ms(CHAIN_LIGHTNING_SKILL_ID),
            cooldown_ms,
            runtime.now_milliseconds(),
        ) {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((x, y)) = dispatch_destination(game, region_id, dispatch) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let started_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(CHAIN_LIGHTNING_SKILL_ID));
        }
        player_ai.begin_chain_lightning(ChainLightningExecutionState::begin(dispatch, started_at_ms, x, y));
    } else if player_ai.chain_lightning().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.chain_lightning().is_some_and(|state| state.kernel().stage() == SkillStage::Begin) {
        let state = player_ai.chain_lightning().expect("выполнение цепной молнии создано");
        let (target_x, target_y) = dispatch_destination(game, region_id, dispatch)
            .unwrap_or((state.fallback_x, state.fallback_y));
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
        while maximum_distance < path.len() as u32 { path.pop(); }
        if path.is_empty() {
            send_failure(game, player_id, 2, mp_loss);
            finish_player_chain_lightning(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            finish_player_chain_lightning(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, 1, None);
        send_visual(game, player_id, level, 2, path.last().map(|cell| (cell.0, cell.1)));

        let master = game.find_player(player_id).map(master_info).unwrap_or_default();
        let (area_width, area_height) = game.area_dimensions();
        let mut attacked = Vec::new();
        for (x, y, block) in path {
            if block == 2 { break; }
            let identities = game.find_region(region_id).map(|owner| {
                let mut shapes = Vec::new();
                let _ = owner.base().get_shapes(x, y, area_width, area_height, game, &mut shapes);
                shapes.into_iter().map(|shape| shape.identity).collect::<Vec<_>>()
            }).unwrap_or_default();
            for target in identities {
                if target == (ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: Default::default() })
                    || attacked.contains(&target)
                    || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || !game.owned_player_skill_target_attackable(master, target, region_id)
                { continue; }
                let Some((master, attack)) = calculate_attack(
                    game, player_id, region_id, target, level, minimum, maximum,
                    element_modifier, hit_modifier, damage_modifier,
                ) else { continue };
                match target.object_type {
                    PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                    MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                    _ => {}
                }
                attacked.push(target);
            }
        }
        if let Some(state) = player_ai.chain_lightning_mut() {
            state.attacked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    let state = player_ai.chain_lightning().expect("выполнение цепной молнии сохраняется до интервала");
    if !state.attacked
        || runtime.now_milliseconds().wrapping_sub(state.kernel().started_at_ms()) <= action_interval_ms
    {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    send_visual(game, player_id, level, 3, None);
    if let Some(state) = player_ai.chain_lightning_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_chain_lightning(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
