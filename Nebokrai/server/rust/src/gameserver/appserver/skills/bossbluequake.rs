//! Землетрясение синего босса `CBossBlueQuake` (`0x1f8`) для игрока и монстра.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluequake.cpp`. Путь игрока требует меч категории `1`,
//! сохраняет необратимое списание MP перед повторной проверкой RP и оружия,
//! направление, задержку и упорядоченное поражение передней клетки. Каждая цель
//! проходит исходную RNG-формулу, защиту и урон; цель ниже уровнем затем получает
//! `BossBlueQuakeState` и `ForceMove`. `Attack` и `AI` не изнашивают оружие на
//! отдельных целях: унаследованный `AfterUseSkill` делает это один раз при
//! успешном `End`. Успешный `AddBossBlueQuakeState` вместо износа отдельно
//! вызывает атакующий `IncreaseRp(1, 0)`, поэтому цель состояния даёт второе
//! начисление после обычного damage-hit. Для источника-игрока длительность
//! состояния уменьшается на `reank` источника с насыщением до нуля.
//! Критический множитель исходного
//! `CalculateAttackPower` усекается к нулю отдельно для физического,
//! элементального и духовного компонентов, а коэффициент урона вычисляется в
//! расширенной точности x87 из `u32` и `0.01_f32` до единственной записи `f32`.
//! Путь монстра сохраняет собственную формулу и тот же порядок состояния и `ForceMove`;
//! `CGame` только координирует временное владение регионом и доставку.
//! Для monster-цели `time_percent` отдельно сохраняется в `f32`, после чего
//! unsigned duration масштабируется в x87 и усекается к нулю. Обе ветви
//! проверяют восстановление абсолютным сроком `CSkill::IsRestored`, сохраняя
//! elapsed-семантику общей задержки.
//! End (0x00546090) возвращает движение до AfterUseSkill (0x0053CF30),
//! затем освобождает текущий навык. Callback CPlayer +0x158 пуст:
//! дополнительного пересчёта свойств при завершении нет.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    time_reached,
};
use super::bossbluequakestate::BossBlueQuakeState;
use super::fightdefense::truncate_original;
use super::flash::cell_views;
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{
    skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::send_owned_state_visual;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TIME_PERCENT: u32 = 20_004;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
const SKILL_USAGE_TARGET_BACK_STEP: u32 = 30_002;
const SKILL_USAGE_TARGET_MOVE_SPEED: u32 = 30_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_USER_RP_LOSE: u32 = 3;

fn scaled_monster_duration(persist: u32, time_percent: u32) -> u32 {
    let percent = f64::from(time_percent) as f32;
    truncate_original(
        f64::from(persist) * f64::from(percent) * f64::from(0.01_f32),
    ) as u32
}

pub(crate) const BOSS_BLUE_QUAKE_SKILL_ID: u32 = 0x1f8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerBossBlueQuakeExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    direction: i32,
}

impl PlayerBossBlueQuakeExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            direction: -1,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn direction(&self) -> i32 {
        self.direction
    }

    pub(crate) const fn set_direction(&mut self, direction: i32) {
        self.direction = direction;
    }
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) fn is_player_boss_blue_quake_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == BOSS_BLUE_QUAKE_SKILL_ID,
    }
}

fn player_master(player: &CPlayer) -> MasterInfo {
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

fn player_weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8, amount: u32) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
    match action {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0287"),
        _ => {}
    }
}

fn player_destination(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => {
            let face = game.find_player(player_id)?.shape().get_face_position().ok()?;
            Some((face.x, face.y))
        }
    }
}

fn send_player_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    direction: i32,
    begin: bool,
) {
    let Some(shape) = game.find_player(player_id).map(|player| player.shape().clone()) else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if begin { 1 } else { 2 });
    message.add_long(BOSS_BLUE_QUAKE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if begin {
        message.add_long(direction);
    } else {
        let position = ShapeAreaCoordinates {
            x: shape.get_tile_x().unwrap_or_default(),
            y: shape.get_tile_y().unwrap_or_default(),
        };
        let front = CShape::get_direction_position(direction, position).unwrap_or(position);
        message.add_long(0);
        message.add_long(0);
        message.add_long(front.x);
        message.add_long(front.y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_boss_blue_quake<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    successful: bool,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if successful {
        game.damage_player_weapon(player_id, runtime);
    }
    if successful {
        player_ai.mark_skill_used(BOSS_BLUE_QUAKE_SKILL_ID, runtime.now_milliseconds());
    }
}

pub(crate) fn cancel_player_boss_blue_quake<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_boss_blue_quake(game, player_id, player_ai, runtime, false);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn player_target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target.id)?;
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as u8)
        }),
        _ => None,
    }
}

fn calculate_player_attack(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    properties: &CSkillBaseProperties,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = player_master(player);
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let difference = maximum.wrapping_sub(minimum);
    let width = (if difference < 0 {
        difference.wrapping_neg()
    } else {
        difference
    })
    .wrapping_add(1);
    let physical = minimum
        .wrapping_add(game.skill_random_below(width))
        .max(0);
    let mut attack = AttackInformation {
        skill_id: BOSS_BLUE_QUAKE_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: (f64::from(
            properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR),
        ) * f64::from(0.01_f32)) as f32,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: (combat.add_element_attack as i32).max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(combat.add_soul_attack),
                mp_damage: 0,
            },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(rate),
            );
        }
    }
    Some((master, attack))
}

fn player_front_targets(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    direction: i32,
) -> Vec<ShapeIdentity> {
    let Some(position) = game.find_player(player_id).and_then(|player| {
        Some(ShapeAreaCoordinates {
            x: player.shape().get_tile_x().ok()?,
            y: player.shape().get_tile_y().ok()?,
        })
    }) else {
        return Vec::new();
    };
    let Ok(front) = CShape::get_direction_position(direction, position) else {
        return Vec::new();
    };
    cell_views(game, region_id, front.x, front.y)
        .into_iter()
        .map(|view| view.identity)
        .filter(|identity| {
            matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                && !(identity.object_type == PLAYER_TYPE && identity.id == player_id)
        })
        .collect()
}

fn player_knockback_destination(
    game: &CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target: ShapeIdentity,
    back_steps: u32,
) -> Option<(i32, i32, u32)> {
    let target_view = game.base_magic_target_view(region_id, target)?;
    let direction = get_line_direction(source_x, source_y, target_view.tile_x, target_view.tile_y);
    let mut position = ShapeAreaCoordinates {
        x: target_view.tile_x,
        y: target_view.tile_y,
    };
    let region = game.find_region(region_id)?.base();
    let mut moved = 0_u32;
    while moved < back_steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else {
            break;
        };
        if region
            .region
            .get_block(next.x, next.y)
            .map_or(true, |block| block & 7 != 0)
        {
            break;
        }
        position = next;
        moved = moved.wrapping_add(1);
    }
    Some((position.x, position.y, moved))
}

pub(crate) fn execute_player_boss_blue_quake<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_boss_blue_quake_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, level, mana, rp)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(BOSS_BLUE_QUAKE_SKILL_ID, game.skill_factory()),
                player.mana(),
                player.rp(),
            ))
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game
        .skill_base_properties(BOSS_BLUE_QUAKE_SKILL_ID, level)
        .cloned()
    else {
        if player_ai.player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied().is_some() {
            finish_player_boss_blue_quake(game, player_id, player_ai, runtime, false);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let rp_loss = properties.query_property(SKILL_USAGE_USER_RP_LOSE);
    let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let persist = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let time_percent = properties.query_property(SKILL_USAGE_TIME_PERCENT);
    let back_steps = properties.query_property(SKILL_USAGE_TARGET_BACK_STEP);
    let move_speed = properties.query_property(SKILL_USAGE_TARGET_MOVE_SPEED);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();

    if player_ai.player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied().is_none() {
        if !skill_is_restored(
            player_ai.skill_last_used_ms(BOSS_BLUE_QUAKE_SKILL_ID),
            reuse_delay,
            now_ms,
        ) {
            send_player_failure(game, player_id, 0x0d, 0);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        if !player_weapon_is_sword(game, player) {
            send_player_failure(game, player_id, 0x0e, 0);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7, mp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if rp_loss != 0 && (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 8, rp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(BOSS_BLUE_QUAKE_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(PlayerBossBlueQuakeExecutionState::begin(
            dispatch, now_ms,
        ));
        return player_terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7, mp_loss);
            finish_player_boss_blue_quake(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
        }
        let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 8, rp_loss);
            finish_player_boss_blue_quake(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_rp(current_rp.wrapping_sub(rp_loss as u16));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        if game
            .find_player(player_id)
            .is_none_or(|player| !player_weapon_is_sword(game, player))
        {
            send_player_failure(game, player_id, 0x0e, 0);
            finish_player_boss_blue_quake(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let (target_x, target_y) =
            player_destination(game, region_id, player_id, dispatch).unwrap_or((source_x, source_y));
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(direction);
        }
        if let Some(state) = player_ai.player_skill_state_mut::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID) {
            state.set_direction(direction);
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
        send_player_visual(game, player_id, level, direction, true);
    }

    let Some(execution) = player_ai.player_skill_state::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID).copied() else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    if !time_reached(
        runtime.now_milliseconds(),
        execution.kernel().started_at_ms(),
        delay,
    ) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    let direction = execution.direction();
    send_player_visual(game, player_id, level, direction, false);
    if let Some(state) = player_ai.player_skill_state_mut::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
    }

    let owner = game.find_player(player_id).map(player_master).unwrap_or_default();
    for target in player_front_targets(game, region_id, player_id, direction) {
        if !game.owned_player_skill_target_attackable(owner, target, region_id) {
            continue;
        }
        let Some(target_level) = player_target_level(game, region_id, target) else {
            continue;
        };
        let Some((master, attack)) =
            calculate_player_attack(game, player_id, level, &properties)
        else {
            continue;
        };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
                master, target.id, region_id, attack, runtime,
            ),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
                master, target.id, region_id, attack, runtime,
            ),
            _ => continue,
        }
        if game.periodic_state_target_dead(region_id, target)
            || !game.owned_player_skill_target_attackable(owner, target, region_id)
        {
            continue;
        }
        let source_level = game.find_player(player_id).map_or(0, CPlayer::level);
        if target_level >= source_level {
            continue;
        }
        let base_duration = if target.object_type == PLAYER_TYPE {
            persist
        } else {
            scaled_monster_duration(persist, time_percent)
        };
        let source_reank = game
            .find_player(player_id)
            .map_or(0, |player| u32::from(player.combat_properties().reank));
        let reduced_duration = base_duration.wrapping_sub(source_reank);
        let duration = if (reduced_duration as i32) < 0 {
            0
        } else {
            reduced_duration
        };
        let Some((destination_x, destination_y, moved)) = player_knockback_destination(
            game, region_id, source_x, source_y, target, back_steps,
        ) else {
            continue;
        };
        let state_now_ms = runtime.now_milliseconds();
        let _ = game.apply_boss_blue_quake_control(
            region_id,
            player_id,
            target,
            BossBlueQuakeState::new(state_now_ms, duration),
            destination_x,
            destination_y,
            move_speed.wrapping_mul(moved),
            runtime,
        );
    }
    if let Some(state) = player_ai.player_skill_state_mut::<PlayerBossBlueQuakeExecutionState>(BOSS_BLUE_QUAKE_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_boss_blue_quake(game, player_id, player_ai, runtime, true);
    player_terminal(QueuedSkillExecutionState::Completed)
}

fn send_visual(game: &CGame, region: &CServerRegion, source: &CShape, level: u16, begin: bool) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if begin { 1 } else { 2 });
    message.add_long(BOSS_BLUE_QUAKE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if begin {
        message.add_long(source.get_direction());
    } else if let Ok(face) = source.get_face_position() {
        message.add_long(0);
        message.add_long(0);
        message.add_long(face.x);
        message.add_long(face.y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn target_level(game: &CGame, region: &CServerRegion, identity: ShapeIdentity) -> Option<u8> {
    match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).map(|player| player.level()),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as u8)
        }),
        _ => None,
    }
}

pub(crate) fn replace_quake_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    identity: ShapeIdentity,
    state: BossBlueQuakeState,
    now_milliseconds: impl FnMut() -> u32,
) {
    let Some((shape, previous)) = (if identity.object_type == PLAYER_TYPE {
        game.find_player_mut(identity.id).and_then(|target| {
            let shape = target.shape().clone();
            let previous = target.take_boss_blue_quake_state();
            Some((shape, previous))
        })
    } else {
        region.find_monster_by_id_mut(identity.id).and_then(|target| {
            let shape = target.move_shape().shape().clone();
            let previous = target.move_shape_mut().take_boss_blue_quake_state();
            Some((shape, previous))
        })
    }) else { return };
    if let Some(previous) = previous {
        send_owned_state_visual(game, region, &shape, previous.skill_id(), false, 0, 0);
    }
    if identity.object_type == PLAYER_TYPE {
        if let Some(target) = game.find_player_mut(identity.id) {
            if previous.is_some() {
                target.set_skill_moveable(true);
                target.set_skill_fightable(true);
            }
            let replaced = target.replace_boss_blue_quake_state(state);
            debug_assert!(replaced.is_none());
            target.set_skill_moveable(false);
            target.set_skill_fightable(false);
        }
    } else if let Some(target) = region.find_monster_by_id_mut(identity.id) {
        if previous.is_some() {
            target.move_shape_mut().set_moveable(true);
            target.move_shape_mut().set_fightable(true);
        }
        let replaced = target.move_shape_mut().replace_boss_blue_quake_state(state);
        debug_assert!(replaced.is_none());
        target.move_shape_mut().set_moveable(false);
        target.move_shape_mut().set_fightable(false);
    }
    send_owned_state_visual(
        game,
        region,
        &shape,
        state.skill_id(),
        true,
        state.client_time(now_milliseconds),
        0,
    );
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет формулу, состояние и ForceMove одной цели")]
fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &crate::setup::monsterlist::MonsterProperties,
    master: MasterInfo,
    tamed: bool,
    identity: ShapeIdentity,
    source_x: i32,
    source_y: i32,
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else { return };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(
        game, region.id, attacker_property, tamed, master, identity, &target,
    ) { return; }
    let bounds = region.find_monster_by_id(monster_id)
        .map(|monster| monster.state_attack_bounds(attacker_property.minimum_attack, attacker_property.maximum_attack))
        .unwrap_or((attacker_property.minimum_attack, attacker_property.maximum_attack));
    let minimum = bounds.0 as i32;
    let physical_difference = (bounds.1 as i32).wrapping_sub(minimum);
    let physical_width = (if physical_difference < 0 {
        physical_difference.wrapping_neg()
    } else {
        physical_difference
    })
    .wrapping_add(1);
    let physical = minimum
        .wrapping_add(game.skill_random_below(physical_width))
        .max(0);
    // Исходный virtual `GetAddElementAtk` у `CMonster` возвращает ноль;
    // дополнительного RNG-вызова между физическим и критическим бросками нет.
    let element = 0;
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: BOSS_BLUE_QUAKE_SKILL_ID,
        skill_level: level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: (f64::from(
            properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR),
        ) * f64::from(0.01_f32)) as f32,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(CMonster::resource_soul_attack(attacker_property)), mp_damage: 0 },
        ],
    };
    let attack = defend_owned_monster_attack(game, identity, target.mana, target.war_soul_mana,
        target.player_properties, target.monster_properties, attack);
    apply_owned_monster_attack_hit(game, region, runtime, now_ms, monster_id, master, identity,
        &target.shape, target.health, target.mana, target.master, target.monster_property,
        target.tamed, target.carriage, attack, deaths);

    let Some(live_target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return;
    };
    if live_target.dead
        || live_target.god
        || live_target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            attacker_property,
            tamed,
            master,
            identity,
            &live_target,
        )
    {
        return;
    }
    if target_level(game, region, identity).is_some_and(|target_level| target_level < attacker_property.level as u8) {
        let persist = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
        let duration = if identity.object_type == PLAYER_TYPE { persist } else {
            scaled_monster_duration(
                persist,
                properties.query_property(SKILL_USAGE_TIME_PERCENT),
            )
        };
        replace_quake_state(
            game,
            region,
            identity,
            BossBlueQuakeState::new(now_ms, duration),
            || runtime.now_milliseconds(),
        );

        let Ok(target_x) = target.shape.get_tile_x() else { return };
        let Ok(target_y) = target.shape.get_tile_y() else { return };
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        let mut position = ShapeAreaCoordinates { x: target_x, y: target_y };
        let mut moved = 0_u32;
        let back_steps = properties.query_property(SKILL_USAGE_TARGET_BACK_STEP);
        while moved < back_steps {
            let Ok(next) = CShape::get_direction_position(direction, position) else { break };
            if region.region.get_block(next.x, next.y).map_or(true, |block| block & 7 != 0) { break; }
            position = next;
            moved = moved.wrapping_add(1);
        }
        let duration_ms = properties.query_property(SKILL_USAGE_TARGET_MOVE_SPEED).wrapping_mul(moved);
        let _ = game.force_move_owned_shape(region, identity, position.x, position.y, duration_ms);
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и последствия всех целей клетки")]
pub(crate) fn execute_owned_boss_blue_quake<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((mut source, property, master, tamed, cast, last_used_ms)) = region.find_monster_by_id(monster_id).and_then(|monster| Some((
        monster.move_shape().shape().clone(),
        game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
        monster.master_info(), monster.is_tamed(), monster.current_active_attack_cast(game.skill_factory()), monster.skill_last_used_ms(BOSS_BLUE_QUAKE_SKILL_ID),
    ))) else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
        return true;
    };
    let (target_x, target_y) = (target.view.tile_x, target.view.tile_y);
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        let direction = get_line_direction(source.get_tile_x().unwrap_or_default(), source.get_tile_y().unwrap_or_default(), target_x, target_y);
        let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, BOSS_BLUE_QUAKE_SKILL_ID, level, now_ms);
        }
        source.set_direction(direction);
        send_visual(game, region, &source, level, true);
        return true;
    }
    let cast = cast.expect("выполнение землетрясения проверено выше");
    if cast.dispatch().skill_id != BOSS_BLUE_QUAKE_SKILL_ID { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) { return true; }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_visual(game, region, &source, level, false);
    let Ok(face) = source.get_face_position() else { return true };
    let source_x = source.get_tile_x().unwrap_or_default();
    let source_y = source.get_tile_y().unwrap_or_default();
    for identity in monster_attack_cell_candidates(game, region, monster_id, face.x, face.y) {
        attack_target(game, region, runtime, now_ms, monster_id, level, properties, &property,
            master, tamed, identity, source_x, source_y, deaths);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
    }
    true
}
