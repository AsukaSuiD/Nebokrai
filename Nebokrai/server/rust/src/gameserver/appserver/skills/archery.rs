//! Базовая стрельба GameServer (`SKILL_BASE_ARCHERY`, ID `2`).
//! Задержка уже первого AI считается от CState::Begin до OnBeginSkill,
//! переданного общим расписанием, а не от поздних проверок оружия и пути.
//! Begin 0x005B1E00 вызывает CheckCastCondition 0x005B2770 и возвращает
//! управление расписанию с kernel в Begin. Только следующий Attack вызывает
//! AI 0x005B2370: проверяет смерть/самоцель, поворачивает источник, публикует
//! начало и затем запрещает движение. Срок сравнивается как unsigned
//! now >= wrapping(start + delay), в том числе при нулевой задержке.
//! Отказные AI-ветви вызывают End(0), общий 0x005AE7A0 возвращает движение,
//! но не вызывает AfterUseSkill и не меняет время восстановления.
//! Luvinia CNewSkill/BaseModule не соответствует этому lifecycle CSkill.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/archery.cpp`. Навык исполняется из обычной очереди
//! `CPlayerAI`: проверяет дальность, непролётные клетки и оружие категории
//! лука либо арбалета, блокирует движение на задержку и передаёт попадание
//! региональному `CArcheryPhalanx`. Тот же owner допускает monster-source без
//! проверки оружия и создаёт видимый снаряд, но phalanx затем безусловно ищет
//! атакующего в player-map: штатная monster-owned стрельба поэтому не наносит
//! урон. Rust сохраняет этот наблюдаемый legacy-контракт без зависимости от
//! случайного совпадения player/monster ID. Формулы и порядок RNG применяются только
//! при достижении цели снарядом. Обычное, отказное и клиентское завершение
//! после `Begin` различают End(1) и отказный End(0), а reuse
//! проверяется exact `CSkill::IsRestored`. Как и
//! исходный `CState::GetSufferer`, owner принимает player/NPC/monster/build/gate;
//! NPC отклоняется как мёртвый, а постройки проходят region-owned defence.

use super::archeryphalanx::CArcheryPhalanx;
use super::baseattack::{finish_delayed_base_attack, real_distance, time_reached};
use super::basemagicphalanx::CBaseMagicPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::monsterattack::{
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use crate::gameserver::appserver::skills::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

pub(crate) const ARCHERY_SKILL_ID: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterBaseProjectileKind {
    Archery,
    Magic,
}

impl MonsterBaseProjectileKind {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Archery => ARCHERY_SKILL_ID,
            Self::Magic => super::basemagic::BASE_MAGIC_SKILL_ID,
        }
    }
}

fn send_monster_base_projectile_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    kind: MonsterBaseProjectileKind,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32, i32)>,
) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(kind.skill_id() as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else if let Some((target, x, y, attack_time)) = target {
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(x);
        message.add_long(y);
        message.add_long(attack_time);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет monster AI, skill и region owners")]
pub(crate) fn execute_owned_monster_base_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    kind: MonsterBaseProjectileKind,
    runtime: &mut Runtime,
) -> bool {
    let skill_id = kind.skill_id();
    let Some(properties) = game
        .skill_base_properties(skill_id, i32::from(skill_level))
        .cloned()
    else {
        return false;
    };
    let Some((source, property, master, tamed, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.skill_last_used_ms(skill_id),
            ))
        })
    else {
        return false;
    };
    if cast.is_some_and(|cast| {
        cast.dispatch().skill_id != skill_id
            || cast.dispatch().target != target_identity
    }) {
        return false;
    }
    let now_ms = runtime.now_milliseconds();
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            if cast.is_some() {
                let _ = monster.finish_base_attack_cast_without_reuse(now_ms);
            }
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead
        || (cast.is_none()
            && (target.god
                || target.city_dead
                || !owned_monster_attackable(
                    game,
                    region.id,
                    &property,
                    tamed,
                    master,
                    target_identity,
                    &target,
                )))
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            if cast.is_some() {
                let _ = monster.finish_base_attack_cast_without_reuse(now_ms);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source.get_tile_x(),
        source.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        if cast.is_some()
            && let Some(monster) = region.find_monster_by_id_mut(monster_id)
        {
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.finish_base_attack_cast_without_reuse(now_ms);
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        let attack_interval = if tamed {
            region
                .find_monster_by_id(monster_id)
                .map(|monster| monster.pet_attack_properties(&property).attack_interval)
                .unwrap_or(property.attack_speed)
        } else {
            property.attack_speed
        };
        if schedule_attack_interval(property.ai, attack_interval).is_some_and(|interval| {
            region
                .find_monster_by_id_mut(monster_id)
                .is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))
        }) {
            return true;
        }
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                reuse_delay_ms,
                now_ms,
            )
        {
            return true;
        }
        let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
        let maximum_distance_allowance = usize::from(matches!(kind, MonsterBaseProjectileKind::Archery));
        if maximum_distance != 0
            && path.len() > maximum_distance as usize + maximum_distance_allowance
        {
            return true;
        }
        if matches!(kind, MonsterBaseProjectileKind::Archery)
            && path.iter().any(|cell| cell.2 == 2)
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                skill_id,
                skill_level,
                now_ms,
            );
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_monster_base_projectile_visual(game, region, source, kind, skill_level, 1, None);
        return true;
    }
    let cast = cast.expect("monster base projectile cast проверен выше");
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
    }
    let attack_time = real_distance(source_x, source_y, target_x, target_y)
        .wrapping_mul(properties.query_property(SKILL_USAGE_SUMMONED_SPEED) as i32);
    send_monster_base_projectile_visual(
        game,
        region,
        &source,
        kind,
        skill_level,
        2,
        Some((target_identity, target_x, target_y, attack_time)),
    );
    let forced_distance = real_distance(source_x, source_y, target_x, target_y) as u32;
    let path = region.straight_skill_path(
        source_x,
        source_y,
        target_x,
        target_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let summon_id = game.allocate_summon_shape_id();
        let started_at_ms = runtime.now_milliseconds();
        let master = MasterInfo {
            master_type: MONSTER_TYPE,
            master_id: monster_id,
            ..MasterInfo::default()
        };
        let (tile_x, tile_y, _) = path[0];
        let (area_width, area_height) = game.area_dimensions();
        match kind {
            MonsterBaseProjectileKind::Archery => {
                let mut phalanx = CArcheryPhalanx::new(
                    summon_id,
                    master,
                    started_at_ms,
                    properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME),
                    i32::from(skill_level),
                    attack_time as u32,
                    target_identity,
                );
                phalanx.shape_mut().set_region_id(region.id);
                let _ = region.add_archery_phalanx(
                    phalanx,
                    tile_x,
                    tile_y,
                    area_width,
                    area_height,
                    started_at_ms,
                    runtime,
                );
            }
            MonsterBaseProjectileKind::Magic => {
                let mut phalanx = CBaseMagicPhalanx::new(
                    summon_id,
                    master,
                    started_at_ms,
                    properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME),
                    i32::from(skill_level),
                    properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32,
                    properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32,
                    properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32,
                    attack_time as u32,
                    target_identity,
                );
                phalanx.shape_mut().set_region_id(region.id);
                let _ = region.add_base_magic_phalanx(
                    phalanx,
                    tile_x,
                    tile_y,
                    area_width,
                    area_height,
                    started_at_ms,
                    runtime,
                );
            }
        }
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
}

impl ArcheryExecutionState {
    pub(crate) const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }
}

fn finish_player_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_delayed_base_attack(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_archery_used(now_ms);
    });
}

pub(crate) fn cancel_player_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.archery().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    finish_player_archery(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let outcome = execute_player_archery_stage(game, player_id, dispatch, player_ai, runtime);
    if outcome.state == QueuedSkillExecutionState::Rejected {
        super::baseattack::finish_failed_base_attack(game, player_id, true);
    }
    outcome
}

fn execute_player_archery_stage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let rejected = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Rejected,
        first_contact: false,
        killing_blow: None,
    };
    let pending = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Pending,
        first_contact: false,
        killing_blow: None,
    };
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    let Some(region_id) = player.server_region_id() else {
        return rejected();
    };
    let skill_level = player.learned_skill_level(ARCHERY_SKILL_ID);
    let Some(properties) = game.skill_base_properties(ARCHERY_SKILL_ID, skill_level)
    else {
        return rejected();
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let summoned_speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let summoned_lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();
    let target = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => target,
        _ => {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        }
    };

    if player_ai.archery().is_none() {
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            player_ai.archery_last_used_ms(),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.send_base_magic_failure(player_id, 0x0d);
            return rejected();
        }
        let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        };
        let (source_x, source_y) = match (
            player.shape().get_tile_x(),
            player.shape().get_tile_y(),
        ) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return rejected(),
        };
        let Some((target_x, target_y)) =
            game.base_magic_target_point(region_id, source_x, source_y, target)
        else {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        };
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            target_x,
            target_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize + 1 {
            game.send_base_magic_failure(player_id, 0x0b);
            let target_name = game.base_magic_target_name(region_id, target).unwrap_or_default();
            game.send_skill_system_info_with_text(player_id, b"GS0280", target_name);
            return rejected();
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_base_magic_failure(player_id, 0x0f);
            game.send_skill_system_info(player_id, b"GS0282");
            return rejected();
        }
        let weapon_category = player
            .equipment()
            .get_goods(2)
            .map(|weapon| {
                weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1)
            });
        match weapon_category {
            None => {
                game.send_base_magic_failure(player_id, 0x0e);
                game.send_skill_system_info(player_id, b"GS0283");
                return rejected();
            }
            Some(3 | 4) => {}
            Some(_) => {
                game.send_base_magic_failure(player_id, 0x0e);
                game.send_skill_system_info(player_id, b"GS0284");
                return rejected();
            }
        }
        player_ai.begin_archery(ArcheryExecutionState::begin(dispatch, target, now_ms));
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(ARCHERY_SKILL_ID));
        }
        return pending();
    }
    let Some(execution) = player_ai.archery() else {
        return rejected();
    };
    if execution.kernel().dispatch() != dispatch {
        return rejected();
    }
    if game.base_magic_target_view(region_id, target).is_none() {
        game.send_base_magic_failure(player_id, 10);
        return rejected();
    }
    if execution.kernel().stage() == SkillStage::Begin {
        let Some(source_view) = player.shape_view() else {
            return rejected();
        };
        let (source_x, source_y) = (source_view.tile_x, source_view.tile_y);
        let Some((target_x, target_y)) =
            game.base_magic_target_point(region_id, source_x, source_y, target)
        else {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        };
        let target_dead = game.base_magic_target_dead(region_id, target);
        if target_dead {
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0285");
            return rejected();
        }
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            game.send_base_magic_failure(player_id, 10);
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target_x,
                target_y,
            ));
            player.set_current_skill_id(Some(ARCHERY_SKILL_ID));
        }
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction())
            .unwrap_or_default();
        let mut start = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        start.add_byte(1);
        start.add_long(ARCHERY_SKILL_ID as i32);
        start.base_mut().add_short(skill_level as i16);
        start.add_long(PLAYER_TYPE);
        start.add_long(player_id);
        start.add_long(direction);
        let _ = game.send_player_shape_around(player_id, None, &start);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
        }
        if let Some(execution) = player_ai.archery_mut() {
            let _ = execution.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = execution.kernel().started_at_ms();
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return pending();
    }

    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
        game.send_base_magic_failure(player_id, 10);
        return rejected();
    };
    let target_dead = game.base_magic_target_dead(region_id, target);
    if target_dead {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        game.send_base_magic_failure(player_id, 10);
        return rejected();
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        return rejected();
    };
    let Some((target_x, target_y)) = game.base_magic_target_point(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target,
    ) else {
        return rejected();
    };
    let attack_time = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
    )
    .wrapping_mul(summoned_speed as i32);
    let mut fire = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    fire.add_byte(2);
    fire.add_long(ARCHERY_SKILL_ID as i32);
    fire.base_mut().add_short(skill_level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target.object_type);
    fire.add_long(target.id);
    fire.add_long(target_x);
    fire.add_long(target_y);
    fire.add_long(attack_time);
    let _ = game.send_player_shape_around(player_id, None, &fire);

    let forced_distance = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
    ) as u32;
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let player = game.find_player(player_id).expect("стрелок сохранён");
        let permissions = player.pk_permissions();
        let master = MasterInfo {
            master_type: PLAYER_TYPE,
            master_id: player_id,
            master_guild_id: player.faction_id(),
            master_team_id: player.team_id(),
            master_union_id: player.union_id(),
            master_country_id: 0,
            permitted_to_kill_player: i32::from(permissions.player),
            permitted_to_kill_teammate: i32::from(permissions.teammate),
            permitted_to_kill_guild_member: i32::from(permissions.guild_member),
            permitted_to_kill_criminal: i32::from(permissions.criminal),
        };
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CArcheryPhalanx::new(
            summon_id,
            master,
            summon_started_at_ms,
            summoned_lifetime,
            skill_level,
            attack_time as u32,
            target,
        );
        phalanx.shape_mut().set_region_id(region_id);
        let (tile_x, tile_y, _) = path[0];
        let result = game.add_archery_phalanx(
            region_id,
            phalanx,
            tile_x,
            tile_y,
            summon_started_at_ms,
            runtime,
        );
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд базовой стрельбы");
    }
    if let Some(state) = player_ai.archery_mut() {
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_archery(game, player_id, player_ai, runtime);
    QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Completed,
        first_contact: false,
        killing_blow: None,
    }
    }

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранена только недостигнутая внутренняя функция `FUN_005b27a3`; skill-путь материализован полностью.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.h

// ============================================================================
// FUNCTION: FUN_005b27a3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:41
// RVA: 0x001B27A3
// ADDRESS: 005b27a3
// PROTOTYPE: undefined FUN_005b27a3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
