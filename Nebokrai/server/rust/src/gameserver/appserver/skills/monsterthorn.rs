//! Владелец шипастой атаки `CMonsterThorn` (`0x197`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/monsterthorn.cpp`. Player и monster/pet пути сохраняют
//! cooldown, повторную проверку длины и `BLOCK_UNFLY` после задержки, запрет
//! движения и одиночный удар по живой объектной цели. Player-формула выполняет
//! RNG физического урона и critical-roll, затем в исходном порядке передаёт
//! physical, element и soul записи; monster-формула сохраняет нулевые element
//! и soul записи и обязательный второй RNG. `CGame` только разрешает владельцев,
//! применяет защиту/смерть и доставляет готовые visual packets.
//! Player-критический множитель вычисляется в расширенной точности x87 и
//! усекается к нулю перед записью `int`. Player и monster ветви используют
//! абсолютный срок `CSkill::IsRestored`; delay также сравнивается с
//! wrapping(start + delay), как unsigned cmp/jb по 0x005422AF.
//! AI различает отказы: мёртвая цель — End(0) (0x005421FA), слишком
//! длинный путь — End(0) (0x00542314), BLOCK_UNFLY после задержки — End(1)
//! без удара (0x00542386..0x0054239B). Последний сохраняет reuse и player
//! AfterUseSkill. End живого cast не сбрасывает AI-цель и очередь движения.
//! Attack (0x00542060) пропускает только null/self до расчёта и Defend;
//! начатый cast не повторяет schedule-проверку IsAttackAble/god. Исчезнувшая
//! объектная цель оставляет пустой GetTargetPath (0x004D85E0): объектный
//! CState::Begin обнуляет fallback. Message-owner (0x0054189B..0x00541973)
//! пишет нулевые type/id и fallback x/y, затем Attack пропускает null,
//! а AI выполняет End(1). Цель AI и Move при этом не отменяются.
//! Begin (0x005416E0) оставляет первый AI невыполненным (+0x50 = 0).
//! Поворот и стартовое сообщение находятся в AI (0x00542232..0x00542283),
//! после проверки смерти цели, до delay; фазу хранит общий kernel, без
//! дополнительного флага. CheckCastCondition (0x00541C40) проверяет reuse,
//! длину GetTargetPath и BLOCK_UNFLY до SetMoveable(false) (0x00541DDD).
//! Отказ вызывает End(0) (0x00541751), включая SetMoveable(true) даже
//! без новой блокировки; Attack при отказе не ставится. Begin и AI
//! используют один выбор точки GetBeAttackedPoint, не центр footprint.
//! Отсутствие свойств после Begin вызывает End(0) (0x005423E2); caller
//! передаёт этот отказ owner-у, не оставляя живой cast в очереди. Player
//! CheckCastCondition без свойств также завершается через End(0).
//! Объектный Begin(type/id) (0x005415F0 → 0x005DBE20) хранит заданную
//! identity и нулевой fallback независимо от разрешения объекта. Null-цель
//! не запрещена CheckCastCondition: Begin использует пустой путь, а AI
//! повторно разрешает сохранённую identity. До monster Begin отсутствие
//! свойств всё ещё останавливает schedule, требующий их для дистанции;
//! точный отказ этого раннего caller-а остаётся несогласованным.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::master_info;
use super::fightdefense::truncate_original;
use super::monsterattack::{
    MonsterAttackDeath, OwnedMonsterAttackTarget, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::{MonsterBaseAttackCast, MonsterBaseAttackDispatch};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
pub(crate) const MONSTER_THORN_SKILL_ID: u32 = 0x197;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMonsterThornExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, destination: (i32, i32) }
impl PlayerMonsterThornExecutionState { fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), destination: if matches!(dispatch, PlayerSkillDispatch::Object { .. }) { (0, 0) } else { destination } } } pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel } pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel } }

fn send_thorn_visual(
    game: &CGame,
    region: &CServerRegion,
    source_shape: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(MONSTER_THORN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    if action == 1 {
        message.add_long(source_shape.get_direction());
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
    let _ = game.send_game_shape_around(region, source_shape, None, &message);
}

fn begin_monster_ai(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    skill_level: u16,
    destination: (i32, i32),
) -> bool {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false };
    if !monster.base_attack_cast().is_some_and(|cast| cast.stage() == SkillStage::Begin) {
        return true;
    }
    let (Ok(x), Ok(y)) = (
        monster.move_shape().shape().get_tile_x(),
        monster.move_shape().shape().get_tile_y(),
    ) else { return false };
    monster.move_shape_mut().shape_mut().set_direction(get_line_direction(
        x, y, destination.0, destination.1,
    ));
    let source = monster.move_shape().shape().clone();
    send_thorn_visual(game, region, &source, monster_id, skill_level, 1, None);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Begin, SkillStage::Check);
    }
    true
}

fn monster_target_path(
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    identity: ShapeIdentity,
    target: &OwnedMonsterAttackTarget,
) -> Option<Vec<(i32, i32, u8)>> {
    if identity == source.identity() {
        return Some(Vec::new());
    }
    let (x, y) = (source.get_tile_x().ok()?, source.get_tile_y().ok()?);
    let (target_x, target_y) = if identity.object_type == MONSTER_TYPE {
        region.find_monster_by_id(identity.id)?
            .be_attacked_point(target.monster_property.as_ref()?, x, y)?
    } else {
        (target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?)
    };
    Some(region.straight_skill_path(x, y, target_x, target_y, None))
}

fn reject_monster_begin(region: &mut CServerRegion, monster_id: i32) {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
    }
}

pub(crate) fn end_monster_thorn_without_reuse(region: &mut CServerRegion, monster_id: i32) -> bool {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false };
    if !monster.base_attack_cast().is_some_and(|cast| cast.dispatch().skill_id == MONSTER_THORN_SKILL_ID) {
        return false;
    }
    let _ = monster.finish_base_attack_cast_without_reuse();
    true
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_monster_thorn<Runtime: GameMainLoopRuntime>(
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
        source_shape,
        property,
        attacker_master,
        attacker_tamed,
        pet_attack,
        cast,
        last_used_ms,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        Some((
            monster.move_shape().shape().clone(),
            property.clone(),
            monster.master_info(),
            monster.is_tamed(),
            monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property)),
            monster.base_attack_cast(),
            monster.skill_last_used_ms(MONSTER_THORN_SKILL_ID),
        ))
    }) else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(cast) = cast {
            if cast.dispatch().skill_id != MONSTER_THORN_SKILL_ID
                || cast.dispatch().target != target_identity
            {
                return false;
            }
            if !begin_monster_ai(game, region, monster_id, skill_level, (0, 0)) {
                return true;
            }
            let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
            if runtime.now_milliseconds() < cast.started_at_ms().wrapping_add(delay_ms) {
                return true;
            }
            send_thorn_visual(
                game, region, &source_shape, monster_id, skill_level, 2, None,
            );
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
            }
            super::monsterattack::finish_owned_monster_attack_impact(region, monster_id, runtime);
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead && cast.is_some_and(|cast| cast.dispatch().skill_id == MONSTER_THORN_SKILL_ID) {
        let _ = end_monster_thorn_without_reuse(region, monster_id);
        return true;
    }
    if cast.is_none()
        && (target.dead
            || target.god
            || target.city_dead
            || !owned_monster_attackable(
                game,
                region.id,
                &property,
                attacker_tamed,
                attacker_master,
                target_identity,
                &target,
            ))
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(target_x), Ok(target_y)) = (
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };

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
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let attack_interval = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        let schedule_ready = schedule_attack_interval(property.ai, attack_interval)
            .is_none_or(|interval| {
                region
                    .find_monster_by_id_mut(monster_id)
                    .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
            });
        if !schedule_ready {
            return true;
        }
        if !skill_is_restored(last_used_ms, reuse_delay_ms, runtime.now_milliseconds()) {
            reject_monster_begin(region, monster_id);
            return true;
        }
        let Some(path) = monster_target_path(region, &source_shape, target_identity, &target) else {
            return true;
        };
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let rejected = (maximum != 0 && path.len() > maximum as usize)
            || path.iter().any(|cell| cell.2 == 2);
        drop(path);
        if rejected {
            reject_monster_begin(region, monster_id);
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(false);
            monster.install_base_attack_cast(MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
                target: target_identity,
                skill_id: MONSTER_THORN_SKILL_ID,
                skill_level,
            }, now_ms));
        }
        return true;
    }

    let cast = cast.expect("выполнение шипастой атаки проверено выше");
    if cast.dispatch().skill_id != MONSTER_THORN_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    if !begin_monster_ai(game, region, monster_id, skill_level, (target_x, target_y)) {
        return true;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if runtime.now_milliseconds() < cast.started_at_ms().wrapping_add(delay_ms) {
        return true;
    }
    let Some(path) = monster_target_path(region, &source_shape, target_identity, &target) else {
        return true;
    };
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if maximum != 0 && path.len() > maximum as usize {
        let _ = end_monster_thorn_without_reuse(region, monster_id);
        return true;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
        }
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_thorn_visual(
        game,
        region,
        &source_shape,
        monster_id,
        skill_level,
        2,
        Some((target_identity, target_x, target_y)),
    );

    if target_identity == source_shape.identity() {
        super::monsterattack::finish_owned_monster_attack_impact(region, monster_id, runtime);
        return true;
    }

    let ordinary_attack = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            monster.state_attack_bounds(property.minimum_attack, property.maximum_attack)
        })
        .unwrap_or((property.minimum_attack, property.maximum_attack));
    let physical_minimum = pet_attack.map_or(ordinary_attack.0, |pet| pet.minimum_attack) as i32;
    let physical_maximum = pet_attack.map_or(ordinary_attack.1, |pet| pet.maximum_attack) as i32;
    let physical_span = physical_maximum
        .wrapping_sub(physical_minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    // `CMonster::GetCriticalChance` равен нулю, но исходный код всё равно
    // выполняет второй RNG-вызов `random(100)`.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: MONSTER_THORN_SKILL_ID,
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
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical.max(0),
                mp_damage: 0,
            },
            // Виртуальные `GetAddElementAtk/GetAddSoulAtk` монстра возвращают
            // ноль, но обе записи остаются частью исходного порядка защиты.
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: 0,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: 0,
                mp_damage: 0,
            },
        ],
    };
    let attack = defend_owned_monster_attack(
        game,
        target_identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    apply_owned_monster_attack_hit(
        game,
        region,
        runtime,
        now_ms,
        monster_id,
        attacker_master,
        target_identity,
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
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
    }
    true
}
fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn is_player_monster_thorn_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Point { skill_id: MONSTER_THORN_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: MONSTER_THORN_SKILL_ID, .. }) }
fn player_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }
fn player_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch, fallback: Option<(i32, i32)>) -> Option<(i32, i32)> { match dispatch { PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)).or(fallback), PlayerSkillDispatch::SelfTarget { .. } => None } }
fn send_player_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(0x000b_fe01, player_id, action); }
fn send_player_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, target: Option<ShapeIdentity>, destination: (i32, i32)) { let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return }; let mut message = CMessage::new(0x000b_fe01); message.add_byte(action); message.add_long(MONSTER_THORN_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); if action == 1 { message.add_long(direction); } else { message.add_long(target.map_or(0, |target| target.object_type)); message.add_long(target.map_or(0, |target| target.id)); message.add_long(destination.0); message.add_long(destination.1); } let _ = game.send_player_shape_around(player_id, None, &message); }
fn restore_player(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) { restore_player(game, player_id); finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(MONSTER_THORN_SKILL_ID, now_ms)); }
pub(crate) fn cancel_player_monster_thorn<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false }; finish_player(game, player_id, ai, runtime); ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }
fn calculate_player_attack(game: &mut CGame, player_id: i32, level: i32, hit: i32) -> Option<(MasterInfo, AttackInformation)> { let (combat, master) = game.find_player(player_id).map(|player| (player.combat_properties(), master_info(player)))?; let minimum = combat.minimum_attack as i32; let span = (combat.maximum_attack as i32).wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32; let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0); let mut attack = AttackInformation { skill_id: MONSTER_THORN_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor: 1.0, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] }; if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } } Some((master, attack)) }

fn player_target_path(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    source: (i32, i32),
    dispatch: PlayerSkillDispatch,
) -> Option<Vec<(i32, i32, u8)>> {
    let destination = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => {
            if (target.object_type == PLAYER_TYPE && target.id == player_id)
                || game.base_magic_target_view(region_id, target).is_none()
            {
                return Some(Vec::new());
            }
            game.base_magic_target_point(region_id, source.0, source.1, target)?
        }
        PlayerSkillDispatch::Point { x, y, .. } => {
            if x == 0 && y == 0 { return Some(Vec::new()) }
            (x, y)
        }
        PlayerSkillDispatch::SelfTarget { .. } => return None,
    };
    if destination == source { return Some(Vec::new()) }
    Some(game.base_magic_path(region_id, source.0, source.1, destination.0, destination.1, None))
}

pub(crate) fn execute_player_monster_thorn<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_monster_thorn_dispatch(dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(MONSTER_THORN_SKILL_ID)))) else { return player_terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(MONSTER_THORN_SKILL_ID, level).cloned() else { restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }; let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let hit = properties.query_property(super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED); let now = runtime.now_milliseconds();
    if ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).is_none() { if !skill_is_restored(ai.skill_last_used_ms(MONSTER_THORN_SKILL_ID), reuse, now) { send_player_failure(game, player_id, 0x0d); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } let Some(destination) = player_destination(game, region_id, dispatch, Some((0, 0))) else { return player_terminal(QueuedSkillExecutionState::Rejected) }; let Some(path) = player_target_path(game, region_id, player_id, (source_x, source_y), dispatch) else { restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }; if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); drop(path); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == 2) { send_player_failure(game, player_id, 0x0f); drop(path); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(MONSTER_THORN_SKILL_ID)); } ai.begin_player_skill_execution(PlayerMonsterThornExecutionState::begin(dispatch, destination, now)); return player_terminal(QueuedSkillExecutionState::Begun); }
    let fallback = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.destination); let Some(destination) = player_destination(game, region_id, dispatch, fallback) else { restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }; if player_target(dispatch).is_some_and(|target| game.base_magic_target_view(region_id, target).is_some() && game.periodic_state_target_dead(region_id, target)) { send_player_failure(game, player_id, 10); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }
    if ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).is_some_and(|state| state.kernel().stage() == SkillStage::Begin) { if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_player_visual(game, player_id, level, 1, None, destination); if let Some(state) = ai.player_skill_state_mut::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.kernel().started_at_ms()).unwrap_or_default(); if runtime.now_milliseconds() < started.wrapping_add(delay) { return player_terminal(QueuedSkillExecutionState::Pending) } let Some(path) = player_target_path(game, region_id, player_id, (source_x, source_y), dispatch) else { return player_terminal(QueuedSkillExecutionState::Pending) }; if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == 2) { send_player_failure(game, player_id, 0x0f); finish_player(game, player_id, ai, runtime); return player_terminal(QueuedSkillExecutionState::RejectedAfterUse) }
    let target = player_target(dispatch).filter(|target| game.base_magic_target_view(region_id, *target).is_some()); send_player_visual(game, player_id, level, 2, target, destination); if let Some(target) = target && !(target.object_type == PLAYER_TYPE && target.id == player_id) { if let Some((master, attack)) = calculate_player_attack(game, player_id, level, hit) { match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} } } } if let Some(state) = ai.player_skill_state_mut::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); } finish_player(game, player_id, ai, runtime); player_terminal(QueuedSkillExecutionState::Completed)
}
