//! Механический топот `CMachineryStomp` (`0x1a7`) для объектного пути игрока и монстра.
//! На время участка нанесения урона настоящий CPlayerAI опубликован в CPlayer:
//! вложенные обработчики смерти видят и изменяют ту же очередь источника.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/machinerystomp.cpp`. Владелец сохраняет полную маску 5×5,
//! `g_dwBesideCells == 3` и две трёхклеточные дуги вокруг исходной цели.
//! Проверка расстояния и непроходимого блока, задержка, направление, пакеты и
//! порядок X → Y принадлежат этому модулю. Для каждой допустимой цели отдельно
//! выполняются вызовы RNG физического урона и критического удара; формула игрока
//! также сохраняет урон стихией и душой. `CGame` только разрешает независимых
//! владельцев, применяет защиту и последствия смерти и доставляет пакеты.
//! Координатный Begin (VA `0x00531340`) использует общий GetSufferer на каждом
//! такте. Пустая клетка проходит проверки reuse/дальности/пути, затем AI
//! завершает её failure `13` и End(0); ошибка проверки добавляет failure `2`.
//! Player-варианты обоих владельцев не изнашивают оружие в `Attack` или `AI`:
//! унаследованный
//! `AfterUseSkill` делает это один раз через общий `End`, после возврата
//! движения и перед cooldown соответствующего идентификатора. Коэффициент
//! урона вычисляется в расширенной точности x87 из `u32` и `0.01_f32`, а
//! критический float-множитель усекается к нулю перед записью `int`. Player и
//! monster ветви используют абсолютный срок `CSkill::IsRestored`, включая
//! delay в AI (VA `0x005324c3/0x0052fff3`). Исходная объектная цель допускает
//! NPC, постройки и ворота; IsDied проверяет их общий HP, а путь заканчивается
//! в GetBeAttackedPoint. Область поражает только игроков и монстров:
//! AI фильтрует `400/600` и проверяет IsAttackAble до расчёта урона.
//! Attack (VA `0x00532290/0x0052fdc0`) не увеличивает RP атакующему.
//! End(0) возвращает движение без износа оружия и cooldown; общий
//! CSkill::End не пересчитывает свойства игрока (его virtual `+0x158` пуст).
//! Это относится и к отказам Begin до создания kernel. Монстровый Begin
//! использует тот же GetTargetPath: линия идёт к GetBeAttackedPoint цели,
//! но направление, fire и дополнительные дуги сохраняют её центральную клетку.
//! Общий CMonster завершает существующий cast через политику End 0x00546090
//! при успехе, отмене и Stiffen. End(0) без живого cast остаётся здесь:
//! он тоже вызывает SetMoveable(true), но не создаёт kernel и не пишет reuse.
//! CheckCastCondition (0x00531B80/0x0052F6B0) проверяет reuse, signed
//! RealDistance до цели при ненулевом максимуме, затем BLOCK_UNFLY.
//! Эти отказы обоих навыков выполняют End(0) и возвращают BeginRejected;
//! общий schedule-caller вызывает OnLoseTarget → SearchEnemy. Ожидание
//! Tracing/интервала не является отказом Begin. Begin (0x00531520)
//! оставляет +0x50 = 0: поворот и стартовый пакет выполняет первый AI
//! (0x00532440/0x0052FF70) после проверки цели и до свежих часов delay.
//! Смерть цели, отсутствие цели или свойств после Begin ведут в End(0)
//! (0x00532425/0x00532836 и 0x0052FF55/0x00530366), не отменяя AI-цель
//! и Move. До Begin ранние отказы общего schedule ещё требуют согласования.

//! Цепочка попадания передаёт Option владельца региона до синхронной смерти.
//! Заимствование базы не переживает эту границу; продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону.

use crate::gameserver::gameserver::game::ServerRegionOwner;

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    SKILL_USAGE_USER_HIT_MODIFIER,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::{cell_views, master_info};
use super::fightdefense::truncate_original;
use super::monsterattack::{
    apply_owned_monster_attack_hit,
    monster_attack_cell_candidates,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterSkillCallOutcome, MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use nebokrai_shared::values::CGuid;
use crate::gameserver::appserver::monster::{MonsterBaseAttackCast, MonsterBaseAttackDispatch};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::state::resolve_coordinate_sufferer;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const BLOCK_UNFLY: u8 = 2;
const SCOPE_HALF_SIDE: i32 = 2;
const BESIDE_CELLS: usize = 3;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

pub(crate) const MACHINERY_STOMP_SKILL_ID: u32 = 0x1a7;

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_machinery_stomp_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: MACHINERY_STOMP_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: MACHINERY_STOMP_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
            skill_id: MACHINERY_STOMP_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | 500 | MONSTER_TYPE | 1100 | 1200, .. },
        }
    )
}
fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

fn send_player_start(game: &mut CGame, player_id: i32, skill_id: u32, level: i32) {
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

fn send_player_fire(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_wide_arc_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    game.after_use_player_skill(player_id, skill_id, runtime);
}

fn abort_player_wide_arc_attack(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub(crate) fn cancel_player_wide_arc_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32, skill_id: u32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game
        .player_skill_execution(player_id, skill_id)
        .map(|kernel| kernel.dispatch())
    else {
        return false;
    };
    abort_player_wide_arc_attack(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn calculate_player_attack(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    hit_modifier: i32,
    damage_factor: f32,
) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master) = game.find_player(player_id)
        .map(|player| (player.combat_properties(), master_info(player)))?;
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let mut attack = AttackInformation {
        skill_id,
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
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(critical_rate),
            );
        }
    }
    Some((master, attack))
}

fn attack_player_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    skill_id: u32,
    tile_x: i32,
    tile_y: i32,
    level: i32,
    hit_modifier: i32,
    damage_factor: f32,
    runtime: &mut Runtime,
) {
    for view in cell_views(game, region_id, tile_x, tile_y) {
        let target = view.identity;
        if (target.object_type == PLAYER_TYPE && target.id == player_id)
            || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
        {
            continue;
        }
        let Some(master) = game.find_player(player_id).map(master_info) else { return };
        if !game.owned_player_skill_target_attackable(master, target, region_id) {
            continue;
        }
        let Some((master, attack)) = calculate_player_attack(
            game, player_id, skill_id, level, hit_modifier, damage_factor,
        ) else { continue };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn execute_player_wide_arc_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id: dispatch_skill_id, .. }
            if dispatch_skill_id == skill_id => {
                send_player_failure(game, player_id, 2);
                abort_player_wide_arc_attack(game, player_id);
                return player_terminal(QueuedSkillExecutionState::Rejected);
            }
        PlayerSkillDispatch::Object { skill_id: dispatch_skill_id, target }
            if dispatch_skill_id == skill_id
                && matches!(target.object_type, PLAYER_TYPE | 500 | MONSTER_TYPE | 1100 | 1200) => {},
        PlayerSkillDispatch::Point { skill_id: dispatch_skill_id, .. }
            if dispatch_skill_id == skill_id => {},
        _ => return player_terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, level, source_view)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(skill_id, game.skill_factory()),
            player.shape_view()?,
        ))
    }) else {
        abort_player_wide_arc_attack(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let (target, fallback) = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => (Some(target), (0, 0)),
        PlayerSkillDispatch::Point { x, y, .. } => (
            resolve_coordinate_sufferer(game, region_id, x, y), (x, y),
        ),
        PlayerSkillDispatch::SelfTarget { .. } => unreachable!(),
    };
    let target = target.and_then(|identity| {
        game.base_magic_target_view(region_id, identity).map(|view| (identity, view))
    });
    let active = game.player_skill_execution(player_id, skill_id).is_some();
    let Some(properties) = game.skill_base_properties(skill_id, level) else {
        send_player_failure(game, player_id, if active { 0x0d } else { 2 });
        abort_player_wide_arc_attack(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_factor = (f64::from(
        properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR),
    ) * f64::from(0.01_f32)) as f32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_execution(player_id, skill_id).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, skill_id),
            reuse_delay_ms,
            now_ms,
        ) {
            send_player_failure(game, player_id, 0x0d);
            send_player_failure(game, player_id, 2);
            abort_player_wide_arc_attack(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let distance = target.map_or_else(
            || super::baseattack::real_distance(source_view.tile_x, source_view.tile_y, fallback.0, fallback.1),
            |(_, view)| source_view.real_distance(Some(view)),
        );
        if maximum_distance != 0 && distance > maximum_distance as i32 {
            send_player_failure(game, player_id, 0x0b);
            send_player_failure(game, player_id, 2);
            abort_player_wide_arc_attack(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let (path_x, path_y) = if let Some((identity, _)) = target {
            let Some(point) = game.base_magic_target_point(
                region_id, source_view.tile_x, source_view.tile_y, identity,
            ) else {
                send_player_failure(game, player_id, 2);
                abort_player_wide_arc_attack(game, player_id);
                return player_terminal(QueuedSkillExecutionState::Rejected);
            };
            point
        } else {
            fallback
        };
        // GetTargetPath (VA 0x004d85e0) не строит линию к самому источнику
        // либо к отсутствующей цели с нулевыми/совпадающими координатами.
        let no_path = target.is_some_and(|(identity, _)| {
            identity.object_type == PLAYER_TYPE && identity.id == player_id
        }) || (target.is_none() && ((path_x, path_y) == (0, 0)
            || (path_x, path_y) == (source_view.tile_x, source_view.tile_y)));
        let path = if no_path { Vec::new() } else {
            game.base_magic_path(
                region_id, source_view.tile_x, source_view.tile_y,
                path_x, path_y, None,
            )
        };
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_player_failure(game, player_id, 0x0f);
            send_player_failure(game, player_id, 2);
            abort_player_wide_arc_attack(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now_ms));
        return player_terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, skill_id).is_none_or(|kernel| kernel.dispatch() != dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target, target_view)) = target else {
        send_player_failure(game, player_id, 0x0d);
        abort_player_wide_arc_attack(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(region_id, target) {
        send_player_failure(game, player_id, 10);
        abort_player_wide_arc_attack(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, skill_id).is_some_and(|kernel| kernel.stage() == SkillStage::Begin) {
        let Some(source) = game.find_player(player_id).and_then(|player| player.shape_view()) else {
            abort_player_wide_arc_attack(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source.tile_x, source.tile_y, target_view.tile_x, target_view.tile_y,
            ));
        }
        send_player_start(game, player_id, skill_id, level);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, skill_id) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = game.player_skill_execution(player_id, skill_id)
        .map(|kernel| kernel.started_at_ms())
        .expect("выполнение широкой дуговой атаки хранит время начала");
    if !skill_is_restored(started_at_ms, delay_ms, runtime.now_milliseconds()) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(source_view) = game.find_player(player_id).and_then(|player| player.shape_view()) else {
        abort_player_wide_arc_attack(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    send_player_fire(game, player_id, skill_id, level, target_view.tile_x, target_view.tile_y);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, skill_id) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    for offset_x in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
        for offset_y in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
            game.with_published_player_ai(player_id, player_ai, |game| attack_player_cell(
                game, player_id, region_id, skill_id,
                source_view.tile_x.wrapping_add(offset_x),
                source_view.tile_y.wrapping_add(offset_y),
                level, hit_modifier, damage_factor, runtime,
            ));
        }
    }
    for (tile_x, tile_y) in outside_cells(
        source_view.tile_x, source_view.tile_y, target_view.tile_x, target_view.tile_y,
    ) {
        game.with_published_player_ai(player_id, player_ai, |game| attack_player_cell(
            game, player_id, region_id, skill_id, tile_x, tile_y,
            level, hit_modifier, damage_factor, runtime,
        ));
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, skill_id) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_wide_arc_attack(game, player_id, skill_id, runtime);
    player_terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_machinery_stomp<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_wide_arc_attack(
        game,
        player_id,
        dispatch,
        MACHINERY_STOMP_SKILL_ID,
        player_ai,
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
    target: &CShape,
    skill_id: u32,
    skill_level: u16,
) {
    let (Ok(target_x), Ok(target_y)) = (target.get_tile_x(), target.get_tile_y()) else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn turn_arc_step(
    delta_x: i32,
    delta_y: i32,
    radius: i32,
    second: bool,
    step_x: &mut i32,
    step_y: &mut i32,
) {
    if delta_x == radius && delta_y == radius {
        *step_x = if second { 0 } else { -1 };
        *step_y = if second { -1 } else { 0 };
    } else if delta_x == -radius {
        if delta_y == radius {
            *step_x = if second { 1 } else { 0 };
            *step_y = if second { 0 } else { -1 };
        } else if delta_y == -radius {
            *step_x = if second { 0 } else { 1 };
            *step_y = if second { 1 } else { 0 };
        }
    } else if delta_x == radius && delta_y == -radius {
        *step_x = if second { -1 } else { 0 };
        *step_y = if second { 0 } else { 1 };
    }
}

fn append_arc(
    cells: &mut Vec<(i32, i32)>,
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
    radius: i32,
    mut step_x: i32,
    mut step_y: i32,
    second: bool,
) {
    let (mut x, mut y) = (target_x, target_y);
    for _ in 0..BESIDE_CELLS {
        turn_arc_step(
            x.wrapping_sub(source_x),
            y.wrapping_sub(source_y),
            radius,
            second,
            &mut step_x,
            &mut step_y,
        );
        x = x.wrapping_add(step_x);
        y = y.wrapping_add(step_y);
        cells.push((x, y));
    }
}

fn outside_cells(
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
) -> Vec<(i32, i32)> {
    let delta_x = target_x.wrapping_sub(source_x);
    let delta_y = target_y.wrapping_sub(source_y);
    // Исходная проверка намеренно использует signed delta, а не abs.
    if delta_x <= SCOPE_HALF_SIDE && delta_y <= SCOPE_HALF_SIDE {
        return Vec::new();
    }
    let mut cells = vec![(target_x, target_y)];
    let absolute_x = delta_x.wrapping_abs();
    let absolute_y = delta_y.wrapping_abs();
    let radius = absolute_x.max(absolute_y);

    let (first_x, first_y) = if absolute_x == radius {
        if absolute_y == radius { (0, 0) } else { (0, if delta_x < 1 { -1 } else { 1 }) }
    } else {
        (if delta_y < 1 { 1 } else { -1 }, 0)
    };
    append_arc(&mut cells, source_x, source_y, target_x, target_y, radius, first_x, first_y, false);

    let (second_x, second_y) = if absolute_x == radius {
        if absolute_y == radius { (0, 0) } else { (0, if delta_x < 1 { 1 } else { -1 }) }
    } else {
        (if delta_y < 1 { -1 } else { 1 }, 0)
    };
    append_arc(&mut cells, source_x, source_y, target_x, target_y, radius, second_x, second_y, true);
    cells
}

#[derive(Clone, Debug)]
pub(crate) struct WideArcAttackDispatch {
    pub(crate) monster_id: i32,
    pub(crate) cells: Vec<(i32, i32)>,
    pub(crate) skill_id: u32,
    skill_level: u16,
    properties: CSkillBaseProperties,
    property: MonsterProperties,
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn prepare_owned_wide_arc_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_owner: &mut ServerRegionOwner,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    dispatch: &mut Option<WideArcAttackDispatch>,
) -> MonsterSkillCallOutcome {
    let region = region_owner.base_mut();
    let Some((source, property, pet_attack, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                property.clone(),
                monster.is_tamed().then(|| monster.pet_attack_properties(&property)),
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(skill_id, game.skill_factory()),
            ))
        })
    else {
        return MonsterSkillCallOutcome::NotHandled;
    };
    if cast.is_some_and(|cast| cast.dispatch().skill_id != skill_id
        || cast.dispatch().target != target_identity)
    {
        return MonsterSkillCallOutcome::NotHandled;
    }
    let target = resolve_owned_monster_attack_target(game, region_owner, target_identity);
    let region = region_owner.base_mut();
    if cast.is_some() && target.as_ref().is_none_or(|target| target.dead) {
        let _ = super::monsterattack::end_owned_monster_skill_without_reuse(region, monster_id, skill_id, game);
        return MonsterSkillCallOutcome::Handled;
    }
    let Some(target) = target else { return MonsterSkillCallOutcome::NotHandled };
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source.get_tile_x(),
        source.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return MonsterSkillCallOutcome::Handled;
    };

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return MonsterSkillCallOutcome::Handled;
        }
        let attack_interval_ms = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return MonsterSkillCallOutcome::Handled;
            }
        }
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                runtime.now_milliseconds(),
            )
        {
            return abort_monster_wide_arc_begin(region, monster_id);
        }
        let Some(source_view) = region.find_monster_by_id(monster_id)
            .and_then(|monster| monster.shape_view(&property))
        else { return MonsterSkillCallOutcome::Handled };
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        if maximum != 0 && source_view.real_distance(Some(target.view)) > maximum as i32 {
            return abort_monster_wide_arc_begin(region, monster_id);
        }
        let point = if target_identity.object_type == MONSTER_TYPE {
            region.find_monster_by_id(target_identity.id).and_then(|monster| {
                monster.be_attacked_point(target.monster_property.as_ref()?, source_x, source_y)
            })
        } else {
            Some((target_x, target_y))
        };
        let Some((path_x, path_y)) = point else { return MonsterSkillCallOutcome::Handled };
        let path = if target_identity.object_type == MONSTER_TYPE && target_identity.id == monster_id {
            Vec::new()
        } else {
            region.straight_skill_path(source_x, source_y, path_x, path_y, None)
        };
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            drop(path);
            return abort_monster_wide_arc_begin(region, monster_id);
        }
        let target_object = resolve_owned_skill_begin_object(game, region, target_identity);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(false);
            monster.install_base_attack_cast(MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
                target: target_identity,
                skill_id,
                skill_level,
            }, now_ms), target_object, game.skill_factory());
        }
        return MonsterSkillCallOutcome::Handled;
    }

    let cast = cast.expect("выполнение механического топота проверено выше");
    if cast.stage() == SkillStage::Begin {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(get_line_direction(
                source_x, source_y, target_x, target_y,
            ));
        }
        let source = region.find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape()).unwrap_or(&source);
        send_start(game, region, source, skill_id, skill_level);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Begin, SkillStage::Check, game.skill_factory());
        }
    }
    if !skill_is_restored(
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
        runtime.now_milliseconds(),
    ) {
        return MonsterSkillCallOutcome::Handled;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
    }
    send_fire(game, region, &source, &target.shape, skill_id, skill_level);
    let mut cells = Vec::with_capacity(32);
    for offset_x in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
        for offset_y in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
            cells.push((
                source_x.wrapping_add(offset_x),
                source_y.wrapping_add(offset_y),
            ));
        }
    }
    cells.extend(outside_cells(source_x, source_y, target_x, target_y));
    *dispatch = Some(WideArcAttackDispatch {
        monster_id,
        cells,
        skill_id,
        skill_level,
        properties: properties.clone(),
        property,
    });
    MonsterSkillCallOutcome::Handled
}

fn wide_arc_attack(
    game: &mut CGame,
    region: &CServerRegion,
    dispatch: &WideArcAttackDispatch,
) -> Option<AttackInformation> {
    let monster = region.find_monster_by_id(dispatch.monster_id)?;
    let (minimum, maximum) = monster.state_attack_bounds(
        dispatch.property.minimum_attack,
        dispatch.property.maximum_attack,
    );
    let soul_attack = monster.soul_attack(&dispatch.property);
    let minimum = minimum as i32;
    let maximum = maximum as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let _critical_roll = game.skill_random_below(100);
    Some(AttackInformation {
        skill_id: dispatch.skill_id,
        skill_level: dispatch.skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: dispatch.monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: dispatch.properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: (f64::from(
            dispatch
                .properties
                .query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR),
        ) * f64::from(0.01_f32)) as f32,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(soul_attack),
                mp_damage: 0,
            },
        ],
    })
}

pub(crate) fn wide_arc_attack_cell_candidates(
    game: &CGame,
    region_owner: &ServerRegionOwner,
    dispatch: &WideArcAttackDispatch,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    monster_attack_cell_candidates(game, region_owner, dispatch.monster_id, tile_x, tile_y)
}

pub(crate) fn execute_owned_wide_arc_attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    dispatch: &WideArcAttackDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_ref() else { return false; };
    if !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE) { return false; }
    if resolve_owned_monster_attack_target(game, region_owner, identity).is_none() { return false; }
    let source = ShapeIdentity { object_type: MONSTER_TYPE, id: dispatch.monster_id, ex_id: CGuid::GUID_INVALID };
    if !game.live_skill_target_attackable_in(region_owner, source, identity) { return false; }
    let Some(attack) = wide_arc_attack(game, region_owner.base(), dispatch) else { return false; };
    apply_owned_monster_attack_hit(game, owner, runtime, identity, attack);
    true
}

fn abort_monster_wide_arc_begin(region: &mut CServerRegion, monster_id: i32) -> MonsterSkillCallOutcome {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
    }
    MonsterSkillCallOutcome::BeginRejected
}
