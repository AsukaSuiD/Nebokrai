//! Расходование накопленных метеорных стрел навыком `CFallingStar` (`0xD5`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fallingstar.cpp`. Начальная проверка сохраняет задержку
//! повторного использования,
//! дальность, непроходимые клетки, лук и ненулевую стоимость MP. В AI MP
//! списывается до повторной проверки лука и накопленного состояния; эта
//! частичная мутация необратима. После задержки первый слот `0xCC` уничтожается
//! напрямую: destructor публикует снятие, но End и UpdateProperty не вызываются.
//! Ненулевой signed-запас разрешает выстрел и область с сетевым ID `0xCD`.
//! Формулы, два вызова RNG на стрелу и пакеты принадлежат владельцам навыка;
//! `CGame` только связывает player, region и доставку.
//! `End(true)` фиксирует cooldown после создания области; `End(false)` только
//! прекращает незавершённый cast и не возвращает уже списанные MP или стрелы.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; cast-delay
//! и lifetime созданной области остаются elapsed.

use super::baseattack::{time_reached, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::fallingstarphalanx::create_falling_star_phalanx;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::meteorarrowstate::{consume_meteor_arrow_count, first_meteor_arrow_count};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const FALLING_STAR_SKILL_ID: u32 = 0xd5;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon|
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let p = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(),
        master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate),
        permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) }
}

fn target_snapshot(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(i32, i32, bool)> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.is_dead()))),
        MONSTER_TYPE => game.find_region(region_id).and_then(|r| { let m = r.base().find_monster_by_id(target.id)?;
            Some((m.move_shape().shape().get_tile_x().ok()?, m.move_shape().shape().get_tile_y().ok()?, m.hit_points() == 0)) }),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FallingStarExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
    target: Option<ShapeIdentity>,
    condition_checked: bool,
}

impl FallingStarExecutionState {
    fn begin(
        dispatch: PlayerSkillDispatch,
        destination: (i32, i32),
        target: Option<ShapeIdentity>,
        now_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            destination,
            target,
            condition_checked: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
    }
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_falling_star<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, FALLING_STAR_SKILL_ID, runtime);
}

fn abort_player_falling_star(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }

pub(crate) fn complete_player_falling_star<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_falling_star(game, player_id, ai, runtime);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_falling_star<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_falling_star(game, player_id);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn send_failure(game: &CGame, player_id: i32, action: u8, mp_loss: u32) {
    game.send_base_magic_failure(player_id, action);
    match action {
        4 => game.send_skill_system_info(player_id, b"GS0300"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0297"),
        0x0f => game.send_skill_system_info(player_id, b"GS0282"),
        _ => {}
    }
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(FALLING_STAR_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    x: i32,
    y: i32,
) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(FALLING_STAR_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(x);
    message.add_long(y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const fn is_falling_star_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget {
            skill_id: FALLING_STAR_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Point {
            skill_id: FALLING_STAR_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: FALLING_STAR_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
        }
    )
}

pub(crate) fn execute_player_falling_star<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_falling_star_dispatch(dispatch) {
        return outcome(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, source_x, source_y)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(FALLING_STAR_SKILL_ID, game.skill_factory()),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    }) else {
        return outcome(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(FALLING_STAR_SKILL_ID, level) else {
        if game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID).is_some() { abort_player_falling_star(game, player_id); }
        return outcome(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID).is_none() {
        let (destination, target) = match dispatch {
            PlayerSkillDispatch::SelfTarget { .. } => ((source_x, source_y), None),
            PlayerSkillDispatch::Point { x, y, .. } => ((x, y), None),
            PlayerSkillDispatch::Object { target, .. } => match target_snapshot(game, region_id, target) {
                Some((x, y, false)) => ((x, y), Some(target)),
                _ => {
                    send_failure(game, player_id, 10, mp_loss);
                    return outcome(QueuedSkillExecutionState::Rejected);
                }
            },
        };
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, FALLING_STAR_SKILL_ID), reuse, now_ms) {
            send_failure(game, player_id, 0x0d, mp_loss);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            destination.0,
            destination.1,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_failure(game, player_id, 0x0f, mp_loss);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return outcome(QueuedSkillExecutionState::Rejected);
        };
        if !weapon_is_valid(game, player) {
            send_failure(game, player_id, 0x0e, mp_loss);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 {
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if (player.mana().wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(FALLING_STAR_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, FallingStarExecutionState::begin(
            dispatch,
            destination,
            target,
            now_ms,
        ));
        return outcome(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return outcome(QueuedSkillExecutionState::Rejected);
    }

    let state = game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID).expect("выполнение падающей звезды создано");
    let (mut destination, target) = (state.destination, state.target);
    if let Some(target) = target {
        match target_snapshot(game, region_id, target) {
            Some((x, y, false)) => destination = (x, y),
            _ => {
                send_failure(game, player_id, 10, mp_loss);
                abort_player_falling_star(game, player_id);
                return outcome(QueuedSkillExecutionState::Rejected);
            }
        }
    }
    if !state.condition_checked {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            abort_player_falling_star(game, player_id);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game
            .find_player(player_id)
            .is_none_or(|player| !weapon_is_valid(game, player))
        {
            send_failure(game, player_id, 0x0e, mp_loss);
            abort_player_falling_star(game, player_id);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if game.find_player(player_id)
            .and_then(|player| first_meteor_arrow_count(game, (region_id, player.shape().identity())))
            .is_none_or(|count| count == 0)
        {
            send_failure(game, player_id, 4, mp_loss);
            abort_player_falling_star(game, player_id);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                destination.0,
                destination.1,
            ));
        }
        send_start(game, player_id, level);
        if let Some(state) = game.player_skill_state_mut::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID) {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID)
        .expect("состояние падающей звезды сохранено")
        .kernel()
        .started_at_ms();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay) {
        return outcome(QueuedSkillExecutionState::Pending);
    }
    let source = game.find_player(player_id).map(|player| player.shape().identity());
    let arrows = source.map_or(0, |source| consume_meteor_arrow_count(game, (region_id, source)));
    if arrows == 0 {
        send_failure(game, player_id, 4, mp_loss);
        abort_player_falling_star(game, player_id);
        return outcome(QueuedSkillExecutionState::Rejected);
    }
    send_fire(game, player_id, level, destination.0, destination.1);

    let Some(player) = game.find_player(player_id) else {
        return outcome(QueuedSkillExecutionState::Rejected);
    };
    let master = master_info(player);
    let combat = player.combat_properties();
    let summon_id = game.allocate_summon_shape_id();
    let now_ms = runtime.now_milliseconds();
    let mut phalanx = create_falling_star_phalanx(
        summon_id,
        master,
        now_ms,
        frequency,
        level,
        combat.minimum_attack as i32,
        combat.maximum_attack as i32,
        combat.add_element_attack as i32,
        i32::from(combat.add_soul_attack),
        i32::from(combat.cch),
        hit_modifier,
        arrows as u32,
        destination.0,
        destination.1,
        |maximum| game.skill_random_below(maximum),
    );
    phalanx.shape_mut().set_pos_xy_base(destination.0 as f32 + 0.5, destination.1 as f32 + 0.5);
    let _ = game.spawn_meteor_arrow_phalanx(region_id, phalanx, now_ms, runtime);
    if let Some(state) = game.player_skill_state_mut::<FallingStarExecutionState>(player_id, FALLING_STAR_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_falling_star(game, player_id, ai, runtime);
    outcome(QueuedSkillExecutionState::Completed)
}
