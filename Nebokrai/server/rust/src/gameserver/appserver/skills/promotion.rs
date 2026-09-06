//! Усиление `CPromotion` (`0x142`) для игрока и монстра.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotion.cpp`. Подключённые пути `SelfTarget` и `Object` сохраняют
//! двойную проверку MP, время восстановления, расстояние, задержку, направление, пакеты
//! `0xBFE01` и подтверждённую особенность повторного применения: прежнее
//! состояние перезапускается без замены коэффициентов и длительности. `CGame`
//! предоставляет владельцев и доставку, а проверки, формулы и жизненный цикл
//! остаются в этом модуле. Monster-ветвь использует объектную цель обычного
//! боевого ИИ и сохраняет delay/reuse/range. После первого наложения исходный
//! `AI` (0x00569110) вызывает UpdateProperty источника, но после Restart — нет.
//! Для монстра модификаторы читаются живой проекцией состояний, без лишнего
//! пакета OnChangeStates; для игрока используется полный пересчёт свойств.
//! Координатная перегрузка по точному EXE сохраняет точку и через
//! `CState::GetSufferer` выбирает первый `CMoveShape` клетки; Rust-разрешение
//! повторяет этот порядок через региональный spatial owner. Player и monster
//! ветви используют абсолютный срок `CSkill::IsRestored`; задержка каста
//! также сравнивает unsigned now с wrapping(start + delay), как cmp/jb 0x0056930F.
//! Расход MP в AI проверяется по знаку DWORD-разности и сразу публикуется.

use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::monsterattack::resolve_owned_monster_attack_target;
use super::promotionstate::{PromotionState, send_promotion_state_begin};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::state::{
    resolve_coordinate_sufferer, send_owned_state_visual,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

pub(crate) const PROMOTION_SKILL_ID: u32 = 0x142;
pub(crate) const PROMOTION_EFFECT_MESSAGE: i32 = 0x000b_fe01;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy)]
struct TargetSnapshot {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    dead: bool,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_promotion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    finish_movement(game, player_id);
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_skill_used(PROMOTION_SKILL_ID, now_ms));
}

fn abort_player_promotion(game: &mut CGame, player_id: i32) {
    finish_movement(game, player_id);
}

pub(crate) fn complete_player_promotion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(PROMOTION_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_promotion(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_promotion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(PROMOTION_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_promotion(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn is_promotion_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: PROMOTION_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: PROMOTION_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: PROMOTION_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

fn requested_target(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. } if skill_id == PROMOTION_SKILL_ID => {
            Some(ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: player_id,
                ex_id: CGuid::GUID_INVALID,
            })
        }
        PlayerSkillDispatch::Object { skill_id, target }
            if skill_id == PROMOTION_SKILL_ID
                && matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) =>
        {
            Some(target)
        }
        PlayerSkillDispatch::Point { skill_id, x, y } if skill_id == PROMOTION_SKILL_ID => {
            let target = resolve_coordinate_sufferer(game, region_id, x, y)?;
            matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE).then_some(target)
        }
        _ => None,
    }
}

fn target_snapshot(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<TargetSnapshot> {
    let (tile_x, tile_y) = game.move_shape_target_tile(Some(region_id), identity)?;
    let dead = match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(identity.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => return None,
    };
    Some(TargetSnapshot {
        identity,
        tile_x,
        tile_y,
        dead,
    })
}

fn send_failure(game: &mut CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(PROMOTION_EFFECT_MESSAGE, player_id, action);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: TargetSnapshot,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let source = player.shape().identity();
    let mut message = CMessage::new(PROMOTION_EFFECT_MESSAGE);
    match action {
        0 => {
            message.add_byte(1);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(player.shape().get_direction());
        }
        1 => {
            message.add_byte(2);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(target.identity.object_type);
            message.add_long(target.identity.id);
            message.add_long(target.tile_x);
            message.add_long(target.tile_y);
        }
        _ => return,
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_monster_cast(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    skill_level: u16,
    action: u8,
) {
    let Some(source) = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.move_shape().shape())
    else {
        return;
    };
    let mut message = CMessage::new(PROMOTION_EFFECT_MESSAGE);
    match action {
        0 => {
            message.add_byte(1);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(MONSTER_TYPE);
            message.add_long(monster_id);
            message.add_long(source.get_direction());
        }
        1 => {
            message.add_byte(2);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(MONSTER_TYPE);
            message.add_long(monster_id);
            message.add_long(target.object_type);
            message.add_long(target.id);
            message.add_long(target_x);
            message.add_long(target_y);
        }
        _ => return,
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn install_monster_promotion_target(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    state: PromotionState,
) -> Option<(bool, CShape)> {
    match target.object_type {
        PLAYER_TYPE => {
            let player = game.find_player_mut(target.id)?;
            (player.server_region_id() == Some(region.id))
                .then(|| {
                    let shape = player.shape().clone();
                    (player.begin_promotion_state(state), shape)
                })
        }
        MONSTER_TYPE => region
            .find_monster_by_id_mut(target.id)
            .map(|monster| {
                let shape = monster.move_shape().shape().clone();
                (monster.move_shape_mut().begin_promotion_state(state), shape)
            }),
        _ => None,
    }
}

pub(crate) fn execute_owned_monster_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source_x, source_y, ai_type, attack_interval_ms, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().get_tile_x().ok()?,
                monster.move_shape().shape().get_tile_y().ok()?,
                property.ai,
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.skill_last_used_ms(PROMOTION_SKILL_ID),
            ))
        })
    else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    let (Ok(target_x), Ok(target_y)) = (target.shape.get_tile_x(), target.shape.get_tile_y())
    else {
        return true;
    };
    if target.dead {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            now_ms,
        ) {
            return true;
        }
        if schedule_attack_interval(ai_type, attack_interval_ms).is_some_and(|interval| {
            region
                .find_monster_by_id_mut(monster_id)
                .is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))
        }) {
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
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(
                target_identity,
                PROMOTION_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_monster_cast(
            game,
            region,
            monster_id,
            target_identity,
            target_x,
            target_y,
            skill_level,
            0,
        );
        return true;
    }

    let cast = cast.expect("выполнение усиления проверено выше");
    if cast.dispatch().skill_id != PROMOTION_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    if now_ms < cast.started_at_ms().wrapping_add(properties.query_property(SKILL_USAGE_DELAY_TIME)) {
        return true;
    }

    send_monster_cast(
        game,
        region,
        monster_id,
        target_identity,
        target_x,
        target_y,
        skill_level,
        1,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
    }
    let state = PromotionState::new(
        now_ms,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_EM_MODIFIER) as u16,
        properties.query_property(SKILL_USAGE_HEAL_RECOVER_COEFFICIENT) as u16,
    );
    if let Some((true, shape)) = install_monster_promotion_target(
        game,
        region,
        target_identity,
        state,
    ) {
        send_owned_state_visual(
            game,
            region,
            &shape,
            state.skill_id(),
            true,
            state.client_time(|| now_ms),
            0,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
    }
    true
}

fn install_state(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    state: PromotionState,
) -> Option<bool> {
    match target.object_type {
        PLAYER_TYPE => {
            let player = game.find_player_mut(target.id)?;
            (player.server_region_id() == Some(region_id))
                .then(|| player.begin_promotion_state(state))
        }
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id)?;
            let installed = owner
                .base_mut()
                .find_monster_by_id_mut(target.id)
                .map(|monster| monster.move_shape_mut().begin_promotion_state(state));
            game.restore_region_owner(owner);
            installed
        }
        _ => None,
    }
}

pub(crate) fn execute_player_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_promotion_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, skill_level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(PROMOTION_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target_identity = requested_target(game, region_id, player_id, dispatch);
    let target = target_identity.and_then(|identity| target_snapshot(game, region_id, identity));
    let Some(properties) = game.skill_base_properties(PROMOTION_SKILL_ID, skill_level) else {
        send_failure(game, player_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let magic_attack_factor = properties.query_property(SKILL_USAGE_EM_MODIFIER) as u16;
    let heal_recover_factor =
        properties.query_property(SKILL_USAGE_HEAL_RECOVER_COEFFICIENT) as u16;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(PROMOTION_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let Some(target) = target else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            player_ai.skill_last_used_ms(PROMOTION_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player_id != target.identity.id || target.identity.object_type != PLAYER_TYPE {
            if maximum_distance != 0
                && game
                    .base_magic_path(
                        region_id,
                        source_x,
                        source_y,
                        target.tile_x,
                        target.tile_y,
                        None,
                    )
                    .len()
                    > maximum_distance as usize
            {
                send_failure(game, player_id, 0x0b);
                game.send_skill_system_info(player_id, b"GS0290");
                send_failure(game, player_id, 2);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(PROMOTION_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .player_skill_execution(PROMOTION_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(target) = requested_target(game, region_id, player_id, dispatch)
        .and_then(|identity| target_snapshot(game, region_id, identity))
    else {
        abort_player_promotion(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_promotion(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .player_skill_execution(PROMOTION_SKILL_ID)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        let remaining_mana = current_mana.wrapping_sub(mp_loss);
        if (remaining_mana as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_promotion(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(remaining_mana);
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target.tile_x,
                target.tile_y,
            ));
        }
        let _ = game.publish_player_states(player_id);
        send_cast(game, player_id, target, skill_level, 0);
        if let Some(execution) = player_ai.player_skill_execution_mut(PROMOTION_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .player_skill_execution(PROMOTION_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение усиления создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast(game, player_id, target, skill_level, 1);
    let state_now_ms = runtime.now_milliseconds();
    let state = PromotionState::new(
        state_now_ms,
        keep_time_ms,
        magic_attack_factor,
        heal_recover_factor,
    );
    if install_state(game, region_id, target.identity, state) == Some(true) {
        send_promotion_state_begin(
            game,
            region_id,
            target.identity,
            target.tile_x,
            target.tile_y,
            state,
            || runtime.now_milliseconds(),
        );
        let _ = game.update_player_properties(player_id);
    }
    if let Some(execution) = player_ai.player_skill_execution_mut(PROMOTION_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_promotion(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
