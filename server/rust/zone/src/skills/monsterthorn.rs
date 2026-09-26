//! Владелец шипастой атаки `CMonsterThorn` (ID `0x197`): Check/AI/Attack/
//! Calculate/player-путь буквально; путь, reuse, `BLOCK_UNFLY` после
//! задержки, запрет движения и одиночный удар по живой объектной цели.
//! На время прямого удара настоящий CPlayerAI опубликован в CPlayer:
//! вложенные обработчики смерти видят и изменяют ту же очередь источника.
//!
//! Машинные quirks: `BLOCK_UNFLY` после задержки → End(1) БЕЗ удара; второй
//! RNG crit-roll выполняется всегда, но `vt+0x114` монстра ≡ 0 — крит
//! никогда не срабатывает; провал Begin — End(0) без терминального кадра
//! (отличие от `CMonsterTaming`); мёртвая первая цель клетки не заменяется
//! следующей допустимой. Монстр-кейсы диспетчера остаются у hub старого
//! пакета.
//!
//! Швы: hub-трейты `skills/monsterattack.rs`; общий
//! `approach_attack_range`/`schedule_attack_interval` — зонский
//! `ai::monsterai` через прежний адаптер; часы — fn-параметр
//! `now_milliseconds` делегата старого main loop.
//!
//! Исходный владелец PDB: `appserver/skills/monsterthorn.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#monsterthorn--cmonsterthorn-0x197

use nebokrai_shared::runtime::get_line_direction;

use crate::ai::monsterai::{MonsterSkillCallOutcome, schedule_attack_interval};
use crate::app::game_message::CMessage;
use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::CShape;

use super::baseattackruntime::{
    BaseAttackContact, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER, SKILL_USAGE_CAN_BE_BREAKED,
};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, SkillTermination, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatGame, MonsterCombatOutcome, MonsterCombatPlayer,
    apply_owned_monster_attack_hit, end_owned_monster_skill_without_reuse,
    finish_owned_monster_attack_impact, resolve_owned_monster_attack_target,
};

use super::execution::PlayerMonsterThornExecutionState;

pub const MONSTER_THORN_SKILL_ID: u32 = 0x197;

const CAST_VISUAL_MESSAGE: i32 = 0x000b_fe01;

/// Кадр visual `0x000BFE01` монстрового шипа: action 1 — направление
/// источника, action 2 — identity цели и её клетка (нулевая и fallback
/// исчезнувшей объектной формы — поле навыка владельца).
fn thorn_visual_message(
    skill_level: i32,
    actor_type: i32,
    actor_id: i32,
    action: u8,
    direction: i32,
    target: Option<(ShapeIdentity, i32, i32)>,
) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(action);
    message.add_long(MONSTER_THORN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    if action == 1 {
        message.add_long(direction);
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(0);
        message.add_long(0);
        message.add_long(0);
        message.add_long(0);
    }
    message
}

/// Старт-кадр и поворот Begin-стадии монстрового шипа ровно один раз:
/// поворот к клетке назначения тика (S либо fallback), затем action-1 кадр.
fn begin_monster_cast_visual<Game: MonsterCombatGame>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    skill_level: u16,
    destination: (i32, i32),
) -> bool {
    let Some(source) = game.monster_shape(region, monster_id) else { return false };
    let (Ok(x), Ok(y)) = (source.get_tile_x(), source.get_tile_y()) else { return false };
    game.monster_set_direction(
        region, monster_id, get_line_direction(x, y, destination.0, destination.1),
    );
    let source = game.monster_shape(region, monster_id).unwrap_or(source);
    let visual = thorn_visual_message(
        i32::from(skill_level), MONSTER_TYPE, monster_id, 1, source.get_direction(), None,
    );
    game.send_visual_around(region, &source, &visual);
    game.monster_advance_cast(region, monster_id, MONSTER_THORN_SKILL_ID, SkillStage::Begin, SkillStage::Check);
    true
}

/// Путь `GetTargetPath` монстрового шипа: собственная клетка даёт пустой
/// путь; точка цели — `GetBeAttackedPoint` владельца, не центр footprint.
fn monster_target_path<Game: MonsterCombatGame>(
    game: &Game,
    owner: &Game::RegionOwner,
    source: &CShape,
    identity: ShapeIdentity,
) -> Option<Vec<(i32, i32, u8)>> {
    if identity == source.identity() {
        return Some(Vec::new());
    }
    let (x, y) = (source.get_tile_x().ok()?, source.get_tile_y().ok()?);
    let (target_x, target_y) = game.base_magic_target_point_in(owner, x, y, identity)?;
    Some(game.monster_straight_skill_path(Game::owner_base(owner), x, y, target_x, target_y))
}

/// Отказ Begin монстрового шипа до нового cast: машинный CheckCast отказ
/// вызывает End(0) (0x541751) с SetMoveable(1) даже без новой блокировки.
fn reject_monster_begin<Game: MonsterCombatGame>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
) {
    game.monster_set_moveable(region, monster_id, true);
}

/// Player/monster-путь `CMonsterThorn`: подход к дистанции расписания,
/// attack-speed монстрового AI, reuse/paths гейты до записи cast, фазы
/// визуала и одиночный удар. Caller hub-владельца решает исход через
/// `MonsterSkillCallOutcome` (BeginRejected → OnLoseTarget + SearchEnemy).
#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub fn execute_owned_monster_thorn<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> MonsterSkillCallOutcome
where
    Game: MonsterCombatContact<Runtime> + BaseAttackContact<Runtime>,
{
    let Some(region_owner) = owner.as_mut() else { return MonsterSkillCallOutcome::NotHandled; };
    let Some(facts) = game.monster_combat_facts(
        Game::owner_base(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
    ) else {
        return MonsterSkillCallOutcome::NotHandled;
    };
    let (source_shape, property, cast) = (facts.source, facts.property, facts.cast);
    let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity) else {
        if let Some(cast) = cast {
            if cast.skill_id != MONSTER_THORN_SKILL_ID || cast.target != target_identity {
                return MonsterSkillCallOutcome::NotHandled;
            }
            // Исчезнувшая объектная цель: старт при необходимости с клеткой
            // fallback (0, 0), после delay — fire с нулями и живой End(1).
            if cast.stage == SkillStage::Begin
                && !begin_monster_cast_visual(game, Game::owner_base_mut(region_owner), monster_id, skill_level, (0, 0))
            {
                return MonsterSkillCallOutcome::Handled;
            }
            let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
            if now_milliseconds() < cast.started_at_ms.wrapping_add(delay_ms) {
                return MonsterSkillCallOutcome::Handled;
            }
            let visual = thorn_visual_message(
                i32::from(skill_level), MONSTER_TYPE, monster_id, 2, 0, None,
            );
            game.send_visual_around(Game::owner_base(region_owner), &source_shape, &visual);
            game.monster_advance_cast(
                Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
                SkillStage::Check, SkillStage::Calculate,
            );
            finish_owned_monster_attack_impact(
                game, Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID, runtime,
            );
            return MonsterSkillCallOutcome::Handled;
        }
        game.monster_clear_ai_target(Game::owner_base_mut(region_owner), monster_id);
        return MonsterSkillCallOutcome::Handled;
    };
    // Мёртвая цель mid-cast: updateVE(10) + End(0) без reuse (0x5421FA).
    if target.dead && cast.is_some_and(|cast| cast.skill_id == MONSTER_THORN_SKILL_ID) {
        let _ = end_owned_monster_skill_without_reuse(
            game, Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
        );
        return MonsterSkillCallOutcome::Handled;
    }
    if cast.is_none() && target.dead {
        game.monster_clear_ai_target(Game::owner_base_mut(region_owner), monster_id);
        return MonsterSkillCallOutcome::Handled;
    }
    let (Ok(target_x), Ok(target_y)) = (target.shape.get_tile_x(), target.shape.get_tile_y()) else {
        return MonsterSkillCallOutcome::Handled;
    };

    if cast.is_none() {
        if !game.monster_combat_approach_attack_range(
            Game::owner_base_mut(region_owner),
            monster_id,
            target.view,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return MonsterSkillCallOutcome::Handled;
        }
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let schedule_ready = schedule_attack_interval(facts.ai_kind, facts.attack_interval_ms)
            .is_none_or(|interval| {
                game.monster_begin_attack_attempt(
                    Game::owner_base_mut(region_owner), monster_id, now_ms, interval,
                )
            });
        if !schedule_ready {
            return MonsterSkillCallOutcome::Handled;
        }
        if !skill_is_restored(facts.last_used_ms, reuse_delay_ms, now_milliseconds()) {
            reject_monster_begin(game, Game::owner_base_mut(region_owner), monster_id);
            return MonsterSkillCallOutcome::BeginRejected;
        }
        let Some(path) = monster_target_path(game, region_owner, &source_shape, target_identity) else {
            return MonsterSkillCallOutcome::Handled;
        };
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let rejected = (maximum != 0 && path.len() > maximum as usize)
            || path.iter().any(|cell| cell.2 == 2);
        drop(path);
        if rejected {
            reject_monster_begin(game, Game::owner_base_mut(region_owner), monster_id);
            return MonsterSkillCallOutcome::BeginRejected;
        }
        let target_object = game.resolve_owned_skill_begin_object(
            Game::owner_base(region_owner), target_identity,
        );
        game.monster_set_moveable(Game::owner_base_mut(region_owner), monster_id, false);
        game.monster_install_cast(
            Game::owner_base_mut(region_owner), monster_id, target_identity,
            MONSTER_THORN_SKILL_ID, skill_level, now_ms, target_object,
        );
        return MonsterSkillCallOutcome::Handled;
    }

    let cast = cast.expect("выполнение шипастой атаки проверено выше");
    if cast.skill_id != MONSTER_THORN_SKILL_ID || cast.target != target_identity {
        return MonsterSkillCallOutcome::NotHandled;
    }
    if cast.stage == SkillStage::Begin
        && !begin_monster_cast_visual(
            game, Game::owner_base_mut(region_owner), monster_id, skill_level, (target_x, target_y),
        )
    {
        return MonsterSkillCallOutcome::Handled;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if now_milliseconds() < cast.started_at_ms.wrapping_add(delay_ms) {
        return MonsterSkillCallOutcome::Handled;
    }
    let Some(path) = monster_target_path(game, region_owner, &source_shape, target_identity) else {
        return MonsterSkillCallOutcome::Handled;
    };
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    // Слишком длинный путь: End(0) без reuse (0x542314).
    if maximum != 0 && path.len() > maximum as usize {
        let _ = end_owned_monster_skill_without_reuse(
            game, Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
        );
        return MonsterSkillCallOutcome::Handled;
    }
    // BLOCK_UNFLY после задержки: updateVE(15) + End(1) со штампом reuse,
    // удара нет (0x542386..0x54239B).
    if path.iter().any(|cell| cell.2 == 2) {
        game.monster_finish_cast_clock(
            Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID, runtime,
        );
        return MonsterSkillCallOutcome::Handled;
    }
    game.monster_advance_cast(
        Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
        SkillStage::Check, SkillStage::Calculate,
    );
    let visual = thorn_visual_message(
        i32::from(skill_level), MONSTER_TYPE, monster_id, 2, 0,
        Some((target_identity, target_x, target_y)),
    );
    game.send_visual_around(Game::owner_base(region_owner), &source_shape, &visual);

    if target_identity == source_shape.identity() {
        finish_owned_monster_attack_impact(
            game, Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID, runtime,
        );
        return MonsterSkillCallOutcome::Handled;
    }

    let Some((minimum, maximum)) = game.monster_state_attack_bounds(
        Game::owner_base(region_owner), monster_id, property.minimum_attack, property.maximum_attack,
    ) else {
        return MonsterSkillCallOutcome::Handled;
    };
    let physical_minimum = minimum as i32;
    let physical_maximum = maximum as i32;
    let physical_span = physical_maximum
        .wrapping_sub(physical_minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    // `CMonster::GetCriticalChance` (vt+0x114) равен нулю, но исходный код всё
    // равно выполняет второй RNG-вызов `random(100)`: крит не срабатывает
    // никогда, вызов обязателен (0x541FC7..0x541FD6).
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: MONSTER_THORN_SKILL_ID,
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
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            // Виртуальные `GetAddElementAtk/GetAddSoulAtk` монстра возвращают
            // ноль, обе записи остаются частью исходного порядка защиты.
            AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: 0, mp_damage: 0 },
        ],
    };
    game.monster_advance_cast(
        Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
        SkillStage::Calculate, SkillStage::Attack,
    );
    game.monster_advance_cast(
        Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID,
        SkillStage::Attack, SkillStage::Apply,
    );
    apply_owned_monster_attack_hit(game, owner, runtime, target_identity, attack);
    let Some(region_owner) = owner.as_mut() else { return MonsterSkillCallOutcome::Handled; };
    game.monster_set_action(Game::owner_base_mut(region_owner), monster_id, 1);
    game.monster_finish_cast_clock(
        Game::owner_base_mut(region_owner), monster_id, MONSTER_THORN_SKILL_ID, runtime,
    );
    MonsterSkillCallOutcome::Handled
}

/// Диспетчерская форма player-cast шипа: координатная и объектная формы
/// `0x197` (object без ограничения типов — машинный Check их не проверяет).
pub const fn is_player_monster_thorn_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: MONSTER_THORN_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: MONSTER_THORN_SKILL_ID, .. })
}

fn player_target<Game: MonsterCombatGame>(
    game: &Game,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => game.resolve_identity_sufferer(region_id, target),
        PlayerSkillDispatch::Point { x, y, .. } => game.resolve_coordinate_sufferer(region_id, x, y),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn player_destination<Game: MonsterCombatGame>(
    game: &Game,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
    fallback: Option<(i32, i32)>,
) -> Option<(i32, i32)> {
    if let Some(target) = player_target(game, region_id, dispatch) {
        return game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y));
    }
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { .. } => fallback,
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn player_target_path<Game: MonsterCombatGame>(
    game: &Game,
    region_id: i32,
    player_id: i32,
    source: (i32, i32),
    dispatch: PlayerSkillDispatch,
) -> Option<Vec<(i32, i32, u8)>> {
    let destination = if let Some(target) = player_target(game, region_id, dispatch) {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            return Some(Vec::new());
        }
        game.base_magic_target_point(region_id, source.0, source.1, target)?
    } else {
        match dispatch {
            PlayerSkillDispatch::Point { x, y, .. } if x != 0 || y != 0 => (x, y),
            PlayerSkillDispatch::Point { .. } | PlayerSkillDispatch::Object { .. } => return Some(Vec::new()),
            PlayerSkillDispatch::SelfTarget { .. } => return None,
        }
    };
    if destination == source {
        return Some(Vec::new());
    }
    Some(game.base_magic_path(region_id, source.0, source.1, destination.0, destination.1))
}

/// Движение возвращается всегда (derived End 0x146090 — SetMoveable(U, 1) до
/// CAttackSkill::End); успех дополнительно выполняет `AfterUseSkill` с
/// reuse-штампом прежнего владельца.
fn restore_player<Game: MonsterCombatGame>(game: &mut Game, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player<Game, Runtime>(game: &mut Game, player_id: i32, runtime: &mut Runtime)
where
    Game: MonsterCombatContact<Runtime>,
{
    restore_player(game, player_id);
    game.monster_combat_after_use_player_skill(player_id, MONSTER_THORN_SKILL_ID, runtime);
}

fn end_player_monster_thorn<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    successful: bool,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    let Some(dispatch) = game
        .player_monster_thorn_state(player_id, MONSTER_THORN_SKILL_ID)
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    if successful {
        finish_player(game, player_id, runtime);
    } else {
        restore_player(game, player_id);
    }
    game.finish_monster_player_skill(
        player_id,
        player_ai,
        dispatch,
        if successful { SkillTermination::Completed } else { SkillTermination::Cancelled },
    )
}

pub fn cancel_player_monster_thorn<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    end_player_monster_thorn(game, player_id, player_ai, runtime, false)
}

pub fn complete_player_monster_thorn<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    end_player_monster_thorn(game, player_id, player_ai, runtime, true)
}

/// Player-формула шипа: span `abs(max-min)+1`, исходный порядок physical,
/// element, soul; критический множитель из глобалки 0xEF3E5C с x87-
/// усечением (личный crit-roll `random(100) < GetCCH` dyn-CPlayer).
fn calculate_player_attack<Game: MonsterCombatGame>(
    game: &mut Game,
    player_id: i32,
    level: i32,
    hit: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master) = game
        .find_player(player_id)
        .map(|player| (player.combat_properties(), player.master_info()))?;
    let minimum = combat.minimum_attack as i32;
    let span = (combat.maximum_attack as i32).wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let mut attack = AttackInformation {
        skill_id: MONSTER_THORN_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: hit,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    Some((master, attack))
}

/// Player-вход `0x197`: reuse/paths-гейты с failure 0x0d/0x0b/0x0f без
/// GS-строк, направление и update-стейт в Begin-стадии, delay-absolute,
/// повторная проверка пути с failure по отказу, fire с повторным GetS
/// visual только для кадра и одиночный удар по сохранённой S.
pub fn execute_player_monster_thorn<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> MonsterCombatOutcome
where
    Game: MonsterCombatContact<Runtime> + BaseAttackContact<Runtime>,
{
    if !is_player_monster_thorn_dispatch(dispatch) {
        return MonsterCombatOutcome::Rejected;
    }
    let Some((region_id, source_x, source_y, level)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            game.monster_combat_player_skill_level(player_id, MONSTER_THORN_SKILL_ID)?,
        ))
    }) else {
        return MonsterCombatOutcome::Rejected;
    };
    let Some(properties) = game
        .skill_base_properties(MONSTER_THORN_SKILL_ID, level)
        .cloned()
    else {
        // Player CheckCastCondition без свойств также завершается через End(0).
        restore_player(game, player_id);
        return MonsterCombatOutcome::Rejected;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now = now_milliseconds();
    if game.player_monster_thorn_state(player_id, MONSTER_THORN_SKILL_ID).is_none() {
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, MONSTER_THORN_SKILL_ID),
            reuse,
            now,
        ) {
            game.send_cast_failure(player_id, 0x0d);
            restore_player(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        let Some(destination) = player_destination(game, region_id, dispatch, Some((0, 0))) else {
            return MonsterCombatOutcome::Rejected;
        };
        let Some(path) = player_target_path(game, region_id, player_id, (source_x, source_y), dispatch) else {
            restore_player(game, player_id);
            return MonsterCombatOutcome::Rejected;
        };
        if maximum != 0 && path.len() > maximum as usize {
            game.send_cast_failure(player_id, 0x0b);
            drop(path);
            restore_player(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_cast_failure(player_id, 0x0f);
            drop(path);
            restore_player(game, player_id);
            return MonsterCombatOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(MONSTER_THORN_SKILL_ID));
        }
        game.begin_player_monster_thorn_execution(
            player_id,
            PlayerMonsterThornExecutionState::begin(dispatch, destination, now),
        );
        return MonsterCombatOutcome::Begun;
    }
    let gameplay_target = player_target(game, region_id, dispatch);
    let fallback = game
        .player_monster_thorn_state(player_id, MONSTER_THORN_SKILL_ID)
        .map(|state| state.destination);
    let Some(destination) = player_destination(game, region_id, dispatch, fallback) else {
        restore_player(game, player_id);
        return MonsterCombatOutcome::Rejected;
    };
    if gameplay_target.is_some_and(|target| game.base_magic_target_dead(region_id, target)) {
        game.send_cast_failure(player_id, 10);
        restore_player(game, player_id);
        return MonsterCombatOutcome::Rejected;
    }
    if game
        .player_monster_thorn_state(player_id, MONSTER_THORN_SKILL_ID)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        if let Some(player) = game.find_player_mut(player_id) {
            player.face_cast_direction(get_line_direction(source_x, source_y, destination.0, destination.1));
        }
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) {
            let visual = thorn_visual_message(level, PLAYER_TYPE, player_id, 1, direction, None);
            game.send_player_visual(player_id, &visual);
        }
        if let Some(state) = game.player_monster_thorn_state_mut(player_id, MONSTER_THORN_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game
        .player_monster_thorn_state(player_id, MONSTER_THORN_SKILL_ID)
        .map(|state| state.kernel().started_at_ms())
        .unwrap_or_default();
    if now_milliseconds() < started.wrapping_add(delay) {
        return MonsterCombatOutcome::Pending;
    }
    let Some(path) = player_target_path(game, region_id, player_id, (source_x, source_y), dispatch) else {
        return MonsterCombatOutcome::Pending;
    };
    if maximum != 0 && path.len() > maximum as usize {
        game.send_cast_failure(player_id, 0x0b);
        restore_player(game, player_id);
        return MonsterCombatOutcome::Rejected;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.send_cast_failure(player_id, 0x0f);
        finish_player(game, player_id, runtime);
        return MonsterCombatOutcome::RejectedAfterUse;
    }
    let visual_target = player_target(game, region_id, dispatch);
    let visual_destination = player_destination(game, region_id, dispatch, fallback).unwrap_or(destination);
    let visual = thorn_visual_message(
        level, PLAYER_TYPE, player_id, 2, 0,
        visual_target.map(|target| (target, visual_destination.0, visual_destination.1)),
    );
    game.send_player_visual(player_id, &visual);
    // Attack сохраняет S входа AI; повторный GetS внутри пути и visual не
    // заменяет выбранную цель, даже если состав координатной клетки изменился.
    if let Some(target) = gameplay_target
        && !(target.object_type == PLAYER_TYPE && target.id == player_id)
        && let Some((master, attack)) = calculate_player_attack(game, player_id, level, hit)
    {
        if let Some(target_region) = game.state_move_shape_region_id(region_id, target) {
            game.with_published_player_ai(player_id, player_ai, |game| {
                game.apply_owned_skill_contact(master, target, target_region, attack, runtime);
            });
        }
    }
    if let Some(state) = game.player_monster_thorn_state_mut(player_id, MONSTER_THORN_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player(game, player_id, runtime);
    MonsterCombatOutcome::Completed
}
