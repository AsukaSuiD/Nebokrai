//! Семейство региональных стрел `CHeartLessArrow2/3` (`0xE5/0xE6`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/heartlessarrow2.cpp` и `heartlessarrow3.cpp`. Общий owner
//! сохраняет одинаковые проверки оружия категории `4`, расход MP, задержку,
//! повторную проверку пути и создание региональной phalanx в конечной клетке.
//! Вариант `0xE5` разблокирует движение перед повторной проверкой пути, тогда
//! как `0xE6` делает это только в общем `End`; эта наблюдаемая разница не
//! сглаживается. Общий `End` возвращает движение при любом исходе, но только
//! `End(true)` один раз выполняет оружейный `AfterUseSkill`, обновляет свойства
//! и фиксирует cooldown. Cooldown использует абсолютный срок
//! `CSkill::IsRestored`, а задержка исполнения остаётся elapsed-проверкой.
//! `CGame` остаётся владельцем региона и доставки.

use super::baseattack::time_reached;
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::heartlessarrowphalanx2::CHeartlessArrowPhalanx;
use super::heartlessarrow3::HEARTLESS_ARROW_3_SKILL_ID;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const HEARTLESS_ARROW_2_SKILL_ID: u32 = 0xe5;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartlessArrowAreaExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination_x: i32,
    destination_y: i32,
    condition_checked: bool,
}

impl HeartlessArrowAreaExecutionState {
    fn begin(
        dispatch: PlayerSkillDispatch,
        destination_x: i32,
        destination_y: i32,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            destination_x,
            destination_y,
            condition_checked: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn skill_id(dispatch: PlayerSkillDispatch) -> Option<u32> {
    let id = match dispatch {
        PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
        PlayerSkillDispatch::SelfTarget { .. } => return None,
    };
    matches!(id, HEARTLESS_ARROW_2_SKILL_ID | HEARTLESS_ARROW_3_SKILL_ID).then_some(id)
}

fn target_identity(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        _ => None,
    }
}

fn target_position(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => match target.object_type {
            PLAYER_TYPE => game.find_player(target.id).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))),
            MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
                let monster = owner.base().find_monster_by_id(target.id)?;
                Some((monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?))
            }),
            _ => None,
        },
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn target_is_dead(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> bool {
    match target_identity(dispatch) {
        Some(target) if target.object_type == PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        Some(target) if target.object_type == MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).is_none_or(|monster| monster.hit_points() == 0),
        Some(_) => true,
        None => false,
    }
}

fn target_name<'a>(game: &'a CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> &'a [u8] {
    match target_identity(dispatch) {
        Some(target) if target.object_type == PLAYER_TYPE => game.find_player(target.id).map(CPlayer::player_name).unwrap_or_default(),
        Some(target) if target.object_type == MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).map(CMonster::display_name).unwrap_or_default(),
        _ => &[],
    }
}

fn weapon_failure(game: &CGame, player: &CPlayer) -> Option<&'static [u8]> {
    match player.equipment().get_goods(2) {
        None => Some(b"GS0297"),
        Some(weapon) if weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) != 4 => Some(b"GS0293"),
        Some(_) => None,
    }
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_heartless_arrow_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    let Some(id) = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, execution_skill_id).copied()
        .and_then(|state| skill_id(state.kernel().dispatch()))
    else {
        return;
    };
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, id, runtime);
}

fn abort_player_heartless_arrow_area(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_player_heartless_arrow_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, execution_skill_id).copied()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_heartless_arrow_area(game, player_id, dispatch.skill_id(), player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_heartless_arrow_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, execution_skill_id).copied()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    abort_player_heartless_arrow_area(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn send_start(game: &mut CGame, player_id: i32, id: u32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(id as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "поля следуют точному wire-порядку исходного сообщения")]
fn send_fire(
    game: &mut CGame,
    player_id: i32,
    id: u32,
    level: i32,
    target: Option<ShapeIdentity>,
    x: i32,
    y: i32,
    flying_time_ms: u32,
) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(id as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.map_or(0, |target| target.object_type));
    message.add_long(target.map_or(0, |target| target.id));
    message.add_long(x);
    message.add_long(y);
    message.add_ulong(flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const fn is_heartless_arrow_area_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: HEARTLESS_ARROW_2_SKILL_ID | HEARTLESS_ARROW_3_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: HEARTLESS_ARROW_2_SKILL_ID | HEARTLESS_ARROW_3_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

pub(crate) fn execute_player_heartless_arrow_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_heartless_arrow_area_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let id = skill_id(dispatch).expect("dispatcher проверил вариант стрелы");
    let Some((region_id, level, source_x, source_y)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(id, game.skill_factory()), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(id, level) else {
        if game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().is_some() {
            abort_player_heartless_arrow_area(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0286");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let missile_step_ms = properties.query_property(MISSILE_FLYING_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().is_none() {
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, id),
            reuse_delay_ms,
            runtime.now_milliseconds(),
        ) {
            game.send_base_magic_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((destination_x, destination_y)) = target_position(game, region_id, dispatch) else {
            game.send_base_magic_failure(player_id, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(region_id, source_x, source_y, destination_x, destination_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.send_base_magic_failure(player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_base_magic_failure(player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0291", target_name(game, region_id, dispatch));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if let Some(error) = weapon_failure(game, player) {
            game.send_base_magic_failure(player_id, 0x0e);
            game.send_skill_system_info(player_id, error);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (player.mana().wrapping_sub(mp_loss) as i32) < 0 {
            game.send_base_magic_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let started_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(id));
        }
        game.begin_player_skill_execution(player_id, HeartlessArrowAreaExecutionState::begin(dispatch, destination_x, destination_y, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if target_is_dead(game, region_id, dispatch) {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_heartless_arrow_area(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if target_identity(dispatch).is_some_and(|target| target.object_type == PLAYER_TYPE && target.id == player_id) {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0286");
        abort_player_heartless_arrow_area(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().is_some_and(|state| !state.condition_checked) {
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        let mana = player.mana();
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_base_magic_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_heartless_arrow_area(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if let Some(error) = weapon_failure(game, player) {
            game.send_base_magic_failure(player_id, 0x0e);
            game.send_skill_system_info(player_id, error);
            abort_player_heartless_arrow_area(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let (destination_x, destination_y) = target_position(game, region_id, dispatch).unwrap_or_else(|| {
            let state = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().expect("выполнение стрелы существует");
            (state.destination_x, state.destination_y)
        });
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination_x, destination_y));
        }
        send_start(game, player_id, id, level);
        if let Some(state) = game.player_skill_state_mut::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()) {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().expect("выполнение стрелы существует").kernel().started_at_ms();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if id == HEARTLESS_ARROW_2_SKILL_ID
        && let Some(player) = game.find_player_mut(player_id)
    {
        player.set_skill_moveable(true);
    }
    let (destination_x, destination_y) = target_position(game, region_id, dispatch).unwrap_or_else(|| {
        let state = game.player_skill_state::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()).copied().expect("выполнение стрелы существует");
        (state.destination_x, state.destination_y)
    });
    let path = game.base_magic_path(region_id, source_x, source_y, destination_x, destination_y, None);
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        game.send_base_magic_failure(player_id, 0x0b);
        game.send_skill_system_info(player_id, b"GS0290");
        abort_player_heartless_arrow_area(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.send_base_magic_failure(player_id, 0x0f);
        game.send_skill_system_info_with_text(player_id, b"GS0307", target_name(game, region_id, dispatch));
        abort_player_heartless_arrow_area(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let flying_time_ms = missile_step_ms.wrapping_mul(path.len() as u32);
    send_fire(game, player_id, id, level, target_identity(dispatch), destination_x, destination_y, flying_time_ms);
    let Some((master, critical_chance)) = game.find_player(player_id).map(|player| (master_info(player), i32::from(player.combat_properties().cch))) else {
        abort_player_heartless_arrow_area(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CHeartlessArrowPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        id,
        level,
        damage_factor,
        critical_chance,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let result = game.add_heartless_arrow_phalanx(region_id, phalanx, destination_x, destination_y, summon_started_at_ms, runtime);
    tracing::trace!(region_id, player_id, summon_id, skill_id = id, ?result, "создана региональная ловушка стрел");
    if let Some(state) = game.player_skill_state_mut::<HeartlessArrowAreaExecutionState>(player_id, dispatch.skill_id()) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_heartless_arrow_area(game, player_id, dispatch.skill_id(), player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
