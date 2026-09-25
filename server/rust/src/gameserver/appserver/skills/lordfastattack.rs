//! Быстрая атака владыки `CLordFastAttack` (`0x1f5`) для игрока и монстра.
//! На время применения удара настоящий AI источника опубликован в CPlayer;
//! изменения синхронных callback возвращаются в тот же проход навыка.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordfastattack.cpp`. Объектный путь сохраняет проверку
//! перезарядки, расстояния и непроходимых клеток, затем начало, задержку и два
//! последовательных удара на границах `15001/15002`. Каждый удар игрока
//! выполняет исходные вызовы RNG для физического разброса и критического удара,
//! после которого каждый компонент урона усекается к нулю;
//! путь монстра использует тот же узкий двухударный механизм с отдельной формулой.
//! `CGame` только разрешает владельцев, применяет рассчитанную атаку и доставляет
//! пакеты. Координатный `Begin` выбирает первый `CMoveShape` клетки через
//! общий `CState::GetSufferer` (VA `0x005dbfd0`); пустая клетка и пустой
//! `Begin` делают `End(0)` до properties/cooldown без failure-пакета.
//! Player `End` сбрасывает execution-флаги, возвращает движение и завершает
//! `CAttackSkill::End(1)` после второго удара с единичным оружейным
//! `AfterUseSkill`; сами два `Attack` оружие не изнашивают. Отмена использует
//! `End(0)` без износа, обновления свойств и cooldown. Player и monster ветви
//! используют абсолютный срок `CSkill::IsRestored`; сроки двух ударов остаются elapsed.
//! Player-вариант `CMonsterFastAttack` (`0x2d1`, owner `monsterfastattack.cpp`)
//! использует тот же тип исполнения, но отдельный ID и cooldown: ненулевой MP-cost
//! проверяется в `CheckCastCondition` (VA `0x005133c2`), списывается перед
//! направлением/визуализацией в AI; нулевой cost означает отказ. Его сроки
//! абсолютные, разброс `max(max-min, 0)+1`, критический множитель принадлежит
//! игроку. Ошибки reuse/MP дополнены `GS1143/GS1144`; `Begin` не добавляет
//! failure 2. Общий kernel и существующие владельцы MP/урона заменяют только
//! техническое хранение и доставку, не порядок игровых эффектов.
//! `End` (Monster VA `0x00512b50`) вызывает общий `CAttackSkill::End`:
//! только успешный исход изнашивает оружие; player virtual `+0x158` пуст,
//! поэтому здесь нет дополнительного пересчёта свойств. MP virtual `+0x164`
//! — `CPlayer::OnChangeStates` (VA `0x00433080`), а не combat tick.
//! Расчёт player-урона MonsterBaseAttack использует здесь ту же формулу,
//! что MonsterFastAttack; его одноударное исполнение остаётся у своего owner-а.
//! Объектный и координатный входы включают NPC, постройки и ворота. Общий
//! GetSufferer/IsDied сохраняет нулевой HP NPC; путь к крупной цели заканчивается
//! в GetBeAttackedPoint её footprint. Постройки проходят общий OnBeenAttacked
//! формы с защитой и смертью, без отдельного сценария постройки.
//! Attack (VA `0x00513700/0x00530fb0`) рассчитывает урон
//! без предварительного IsAttackAble и не вызывает IncreaseRp атакующего;
//! оба RNG-вызова поэтому предшествуют возможному отказу защиты цели.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillStage, SkillTermination};
use super::monsterfastattack::{MONSTER_FAST_ATTACK_SKILL_ID, SKILL_USAGE_FIRST_TIME, SKILL_USAGE_SECOND_TIME};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::state::resolve_coordinate_sufferer;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{LordFastAttackExecutionState};

pub(crate) const LORD_FAST_ATTACK_SKILL_ID: u32 = 0x1f5;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const NPC_TYPE: i32 = 500;
const BUILD_TYPE: i32 = 1100;
const CITY_GATE_TYPE: i32 = 1200;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
    }
}

fn send_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

pub(super) fn send_start(game: &mut CGame, player_id: i32, skill_id: u32, level: i32) {
    let Some(direction) = game
        .find_player(player_id)
        .map(|player| player.shape().get_direction())
    else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(
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

fn finish_player_lord_fast_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    let Some(skill_id) = game.player_skill_state::<LordFastAttackExecutionState>(player_id, execution_skill_id).map(|state| state.kernel().dispatch().skill_id()) else { return };
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    game.after_use_player_skill(player_id, skill_id, runtime);
}

fn abort_player_lord_fast_attack(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub(crate) fn complete_player_lord_fast_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<LordFastAttackExecutionState>(player_id, execution_skill_id).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_lord_fast_attack(game, player_id, dispatch.skill_id(), player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_lord_fast_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<LordFastAttackExecutionState>(player_id, execution_skill_id)
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    abort_player_lord_fast_attack(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
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

pub(super) fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master) = game
        .find_player(player_id)
        .map(|player| (player.combat_properties(), master_info(player)))?;
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let difference = maximum.wrapping_sub(minimum);
    let monster_formula = matches!(skill_id, MONSTER_FAST_ATTACK_SKILL_ID | super::monsterbaseattack::MONSTER_BASE_ATTACK_SKILL_ID);
    let span = if monster_formula {
        difference.max(0).wrapping_add(1)
    } else {
        difference.unsigned_abs().wrapping_add(1) as i32
    };
    let physical = minimum
        .wrapping_add(game.skill_random_below(span))
        .max(0);
    let mut attack = AttackInformation {
        skill_id,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: 1.0,
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
        let critical_rate = if monster_formula {
            combat.critical_rate()
        } else {
            game.globe_setup().critical_rate()
        };
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(critical_rate),
            );
        }
    }
    Some((master, attack))
}

fn apply_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    hit_modifier: i32,
    runtime: &mut Runtime,
) {
    let Some((master, attack)) = calculate_attack(game, player_id, skill_id, level, hit_modifier) else {
        return;
    };
    match target.object_type {
        PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        BUILD_TYPE | CITY_GATE_TYPE => game.receive_stationary_build_skill_attack(
            region_id, target, attack, runtime,
        ),
        _ => return,
    }
}

pub(crate) const fn is_lord_fast_attack_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Point {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | BUILD_TYPE | CITY_GATE_TYPE,
                ..
            },
        }
    )
}

pub(crate) fn execute_player_lord_fast_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = dispatch.skill_id();
    let monster_variant = skill_id == MONSTER_FAST_ATTACK_SKILL_ID;
    let begin_failed = |game: &mut CGame| {
        if monster_variant {
            abort_player_lord_fast_attack(game, player_id);
        } else {
            send_failure(game, player_id, 2);
        }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let target = match dispatch {
        PlayerSkillDispatch::SelfTarget {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            ..
        } => {
            abort_player_lord_fast_attack(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        PlayerSkillDispatch::Point {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            x, y,
        } => {
            let target = game.find_player(player_id)
                .and_then(CPlayer::server_region_id)
                .and_then(|region_id| resolve_coordinate_sufferer(game, region_id, x, y));
            let Some(target) = target.filter(|target| matches!(target.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | BUILD_TYPE | CITY_GATE_TYPE)) else {
                abort_player_lord_fast_attack(game, player_id);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            target
        }
        PlayerSkillDispatch::Object {
            skill_id: LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | BUILD_TYPE | CITY_GATE_TYPE) => target,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, level, source_view)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(skill_id, game.skill_factory()),
            player.shape_view()?,
        ))
    }) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(skill_id, level) else {
        if monster_variant || game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()).is_some() {
            abort_player_lord_fast_attack(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let first_time_ms = properties.query_property(SKILL_USAGE_FIRST_TIME);
    let second_time_ms = properties.query_property(SKILL_USAGE_SECOND_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);

    if game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()).is_none() {
        let now_ms = runtime.now_milliseconds();
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            return begin_failed(game);
        };
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, skill_id),
            reuse_delay_ms,
            now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            if monster_variant {
                game.send_skill_system_info(player_id, b"GS1143");
            }
            return begin_failed(game);
        }
        if maximum_distance != 0
            && source_view.real_distance(Some(target_view)) > maximum_distance as i32
        {
            send_failure(game, player_id, 0x0b);
            return begin_failed(game);
        }
        let Some((path_x, path_y)) = game.base_magic_target_point(
            region_id, source_view.tile_x, source_view.tile_y, target,
        ) else { return begin_failed(game) };
        let path = game.base_magic_path(
            region_id,
            source_view.tile_x,
            source_view.tile_y,
            path_x,
            path_y,
            None,
        );
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_failure(game, player_id, 0x0f);
            return begin_failed(game);
        }
        if monster_variant {
            if mp_loss == 0 {
                return begin_failed(game);
            }
            let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
            if (mana.wrapping_sub(mp_loss) as i32) < 0 {
                send_failure(game, player_id, 7);
                game.send_skill_system_info_with_unsigned(player_id, b"GS1144", mp_loss);
                return begin_failed(game);
            }
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, LordFastAttackExecutionState::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id())
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        if !monster_variant {
            send_failure(game, player_id, 10);
        }
        abort_player_lord_fast_attack(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(region_id, target)
        || (target.object_type == PLAYER_TYPE && target.id == player_id)
    {
        send_failure(game, player_id, 10);
        abort_player_lord_fast_attack(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id())
        .is_some_and(|state| !state.condition_checked)
    {
        if monster_variant {
            let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
            let remaining = mana.wrapping_sub(mp_loss);
            if (remaining as i32) < 0 {
                send_failure(game, player_id, 7);
                game.send_skill_system_info_with_unsigned(player_id, b"GS1144", mp_loss);
                abort_player_lord_fast_attack(game, player_id);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(player_id) {
                player.set_mana(remaining);
            }
            let _ = game.publish_player_states(player_id);
        }
        let Some(source_view) = game
            .find_player(player_id)
            .and_then(|player| player.shape_view())
        else {
            abort_player_lord_fast_attack(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_view.tile_x,
                source_view.tile_y,
                target_view.tile_x,
                target_view.tile_y,
            ));
        }
        send_start(game, player_id, skill_id, level);
        if let Some(state) = game.player_skill_state_mut::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()) {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id())
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение быстрой атаки владыки хранит время начала");
    let reached = |now, delay| {
        if monster_variant {
            skill_is_restored(started_at_ms, delay, now)
        } else {
            time_reached(now, started_at_ms, delay)
        }
    };
    if game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id())
        .is_some_and(|state| !state.fire_started)
    {
        if !reached(runtime.now_milliseconds(), delay_ms) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        send_fire(
            game,
            player_id,
            skill_id,
            level,
            target_view.tile_x,
            target_view.tile_y,
        );
        if let Some(state) = game.player_skill_state_mut::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()) {
            state.fire_started = true;
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    if game.player_skill_state::<LordFastAttackExecutionState>(player_id, dispatch.skill_id())
        .is_some_and(|state| !state.first_attack_done)
    {
        if !reached(
            runtime.now_milliseconds(),
            delay_ms.wrapping_add(first_time_ms),
        ) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        game.with_published_player_ai(player_id, player_ai, |game| apply_attack(
            game,
            player_id,
            skill_id,
            region_id,
            target,
            level,
            hit_modifier,
            runtime,
        ));
        if let Some(state) = game.player_skill_state_mut::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()) {
            state.first_attack_done = true;
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    if !reached(
        runtime.now_milliseconds(),
        delay_ms
            .wrapping_add(first_time_ms)
            .wrapping_add(second_time_ms),
    ) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.with_published_player_ai(player_id, player_ai, |game| apply_attack(
        game,
        player_id,
        skill_id,
        region_id,
        target,
        level,
        hit_modifier,
        runtime,
    ));
    if let Some(state) = game.player_skill_state_mut::<LordFastAttackExecutionState>(player_id, dispatch.skill_id()) {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_lord_fast_attack(game, player_id, dispatch.skill_id(), player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
