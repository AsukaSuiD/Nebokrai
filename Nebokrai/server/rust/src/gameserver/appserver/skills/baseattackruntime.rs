//! Исполнение базовой атаки игрока рядом с владельцем навыка.
//!
//! Модуль подключён как дочерний модуль `CGame` только для доступа к узким
//! межвладельческим примитивам. Проверки, стадии, формула, RNG и пакеты
//! принадлежат базовой атаке; `CGame` остаётся владельцем игроков, регионов
//! и фактического применения урона.
//! Общая формула для player/monster/build усекает критический float-множитель
//! к нулю перед записью каждого компонента в `int`: exact
//! `CBaseAttack::CalculateAttackPower` `0x005B380B..0x005B3835` держит
//! произведение в x87 до `FISTP`, не округляя его предварительно до `float`.
//! Физический RNG получает `max(maximum - minimum, 0)` без `+1`, поэтому
//! верхняя граница исходной базовой атаки остаётся исключённой; player,
//! monster и stationary build-ветви используют один и тот же контракт.
//! Maximum-distance gate использует `RealDistance(CShape*)` для разрешённой
//! объектной цели и координатный overload только для point-target без формы.
//! CheckCastCondition (0x005B2E40) требует только источник и свойства,
//! без дополнительного reuse-gate. Kernel материализуется до проверок AI:
//! отказ дальности — End(0), не false из Begin. OnBeginSkill и ранний отсчёт
//! поступают из общего расписания. После начальной визуализации AI читает
//! часы заново и сравнивает unsigned now с wrapping(start + delay)
//! (0x005B3B0E..0x005B3B19), а не с elapsed и не с нулевым интервалом.
//! Проверка погибшей цели предшествует этой задержке на каждом AI и вызывает
//! End(1) сразу; ожидание конца каста не должно откладывать отказ и cooldown.
//! Begin 0x005B3040 возвращает управление до проверок первого AI 0x005B39B0:
//! kernel остаётся в Begin, а общий AI ставит Attack для обработки в том же Run.
//! На первом AI не повторяются ride/level/DoesTargetEffective-гейты расписания.
//! При исчезнувшем object-target исходные нулевые point-поля участвуют в
//! дальности/повороте; отсутствующая форма не подменяется самим источником.
//! Отказный End(0) очищает active ID без износа оружия и нового cooldown.

use super::{
    AttackInformation, AttackPower, AttackPowerType, BASE_ATTACK_SKILL_ID,
    BUILD_OBJECT_TYPE, CITY_GATE_OBJECT_TYPE,
    BaseAttackExecutionState, CGame, CGuid, CMessage, CMonster, CPlayer, CPlayerAI,
    GameMainLoopRuntime, MONSTER_TYPE,
    MonsterKillingAttack, PLAYER_TYPE, PlayerKillingBlow, PlayerSkillDispatch,
    QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    SKILL_USAGE_USER_HIT_MODIFIER, ShapeIdentity, SkillStage,
    defend_build_base_attack, defend_monster_base_attack, defend_player_base_attack,
    finish_blind_states_on_defense, finish_player_base_attack,
    finish_player_blind_states_on_defense, game_legacy_random, get_line_direction, real_distance,
    retarget_jiumai_after_hurt,
    truncate_original,
};
use crate::gameserver::appserver::states::state::resolve_coordinate_sufferer;

pub(super) fn execute_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let outcome = execute_player_base_attack_stage(game, player_id, dispatch, player_ai, runtime);
    if outcome.state == QueuedSkillExecutionState::Rejected {
        crate::gameserver::appserver::skills::baseattack::finish_failed_base_attack(game, player_id, false);
    }
    outcome
}

fn execute_player_base_attack_stage<Runtime: GameMainLoopRuntime>(
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
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    let skill_level = player.learned_skill_level(BASE_ATTACK_SKILL_ID, game.skill_factory());
    let Some(properties) = game
        .skill_factory
        .query_skill_base_properties(BASE_ATTACK_SKILL_ID, skill_level)
    else {
        return rejected();
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let now_ms = runtime.now_milliseconds();
    let region_id = player.server_region_id();
    if game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).is_none() {
        game.begin_player_skill_execution(player_id, BaseAttackExecutionState::begin(dispatch, now_ms));
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(BASE_ATTACK_SKILL_ID));
        }
        return QueuedSkillExecutionOutcome {
            state: QueuedSkillExecutionState::Begun,
            first_contact: false,
            killing_blow: None,
        };
    }
    let requested_target = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        PlayerSkillDispatch::Point { x, y, .. } => {
            region_id.and_then(|region_id| resolve_coordinate_sufferer(game, region_id, x, y))
        }
        PlayerSkillDispatch::SelfTarget { .. } => None,
    };
    let target = match requested_target {
        Some(target) if target.object_type == PLAYER_TYPE => game
            .find_player(target.id)
            .and_then(|player| player.shape_view())
            .map(|view| (target, view)),
        Some(target) if target.object_type == 600 => region_id
            .and_then(|region_id| game.find_region(region_id))
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .and_then(|monster| {
                let property = monster
                    .base_property_key()
                    .and_then(|key| game.find_monster_property_by_origin_name(key))?;
                monster.shape_view(property)
            })
            .map(|view| (target, view)),
        Some(target)
            if target.object_type == BUILD_OBJECT_TYPE as i32
                || target.object_type == CITY_GATE_OBJECT_TYPE as i32 => region_id
                .and_then(|region_id| game.find_shape_in_region(region_id, target))
                .map(|view| (target, view)),
        _ => None,
    };

    let Some(execution) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID) else {
        return rejected();
    };
    if execution.dispatch() != dispatch {
        return rejected();
    }
    let target_dead = target.is_some_and(|(identity, _)| {
        if identity.object_type == PLAYER_TYPE {
            game.find_player(identity.id).is_some_and(CPlayer::is_dead)
        } else {
            region_id.is_some_and(|region_id| game.base_magic_target_dead(region_id, identity))
        }
    });
    if target_dead {
        let _ = game.send_base_attack_failure(player_id, 2);
        finish_player_base_attack(game, player_id, player_ai, runtime);
        return rejected();
    }

    if execution.stage() == SkillStage::Begin {
        let Some(source_view) = player.shape_view() else {
            return rejected();
        };
        let (source_x, source_y) = (source_view.tile_x, source_view.tile_y);
        let (target_x, target_y) = match (dispatch, target) {
            (_, Some((_, view))) => (view.tile_x, view.tile_y),
            (PlayerSkillDispatch::Point { x, y, .. }, None) => (x, y),
            _ => (0, 0),
        };
        let target_distance = target.map_or_else(
            || real_distance(source_x, source_y, target_x, target_y),
            |(_, target_view)| source_view.real_distance(Some(target_view)),
        );
        if maximum_distance != 0 && maximum_distance < target_distance as u32 {
            let _ = game.send_base_attack_failure(player_id, 0x0b);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player
                .movement_shape_mut()
                .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
            player.set_current_skill_id(Some(BASE_ATTACK_SKILL_ID));
        }
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction())
            .unwrap_or_default();
        let mut start = CMessage::new(0x000b_fe01);
        start.add_byte(1);
        start.add_long(BASE_ATTACK_SKILL_ID as i32);
        start.base_mut().add_short(skill_level as i16);
        start.add_long(PLAYER_TYPE);
        start.add_long(player_id);
        start.add_long(direction);
        let _ = game.send_player_shape_around(player_id, None, &start);
        let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Begin, SkillStage::Check));
    }
    let started_at_ms = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID)
        .map_or(now_ms, BaseAttackExecutionState::started_at_ms);
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return QueuedSkillExecutionOutcome {
            state: QueuedSkillExecutionState::Pending,
            first_contact: false,
            killing_blow: None,
        };
    }
    if let Some((target_identity, _)) = target
        && (target_identity.object_type == BUILD_OBJECT_TYPE as i32
            || target_identity.object_type == CITY_GATE_OBJECT_TYPE as i32)
    {
        let available = game
            .find_player(player_id)
            .and_then(CPlayer::server_region_id)
            .is_some_and(|region_id| {
                game.stationary_build_attackable_by_player(player_id, region_id, target_identity)
            });
        if !available {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
    }
    let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Check, SkillStage::Calculate));
    let (target_type, target_id, target_x, target_y) = match target {
        Some((target, view)) => (target.object_type, target.id, view.tile_x, view.tile_y),
        None => match dispatch {
            PlayerSkillDispatch::Point { x, y, .. } => (0, 0, x, y),
            _ => (0, 0, 0, 0),
        },
    };
    let mut fire = CMessage::new(0x000b_fe01);
    fire.add_byte(2);
    fire.add_long(BASE_ATTACK_SKILL_ID as i32);
    fire.base_mut().add_short(skill_level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target_type);
    fire.add_long(target_id);
    fire.add_long(target_x);
    fire.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &fire);

    let mut killing_blow = None;
    let mut first_contact = false;
    if target_type == PLAYER_TYPE && game.player_base_attackable(player_id, target_id) {
        let (
            mut attacker_properties,
            attacker_occupation,
            attacker_team,
            attacker_faction,
            attacker_union,
            target_properties,
            target_level,
            target_health,
            target_mana,
            target_war_soul_mana,
        ) = {
            let attacker = game
                .find_player(player_id)
                .expect("base-attack attacker сохранён");
            let target = game
                .find_player(target_id)
                .expect("base-attack target сохранён");
            (
                attacker.combat_properties(),
                attacker.occupation(),
                attacker.team_id(),
                attacker.faction_id(),
                attacker.union_id(),
                target.combat_properties(),
                target.level(),
                target.health(),
                target.mana(),
                target.war_soul_mana(&game.goods_factory),
            )
        };
        let [
            blast_attack,
            blast_defense,
            element_blast_attack,
            element_blast_defense,
            full_miss,
        ] = game.globe_setup.base_combat_scales();
        if attacker_properties.blast_attack_scale() < 1.0 {
            attacker_properties.blast_attack_scale_bits = blast_attack.max(1.0).to_bits();
        }
        if attacker_properties.blast_defense_scale() < 0.01 {
            attacker_properties.blast_defense_scale_bits = blast_defense.max(0.01).to_bits();
        }
        if attacker_properties.element_blast_attack_scale() < 1.0 {
            attacker_properties.element_blast_attack_scale_bits =
                element_blast_attack.max(1.0).to_bits();
        }
        if attacker_properties.element_blast_defense_scale() < 0.01 {
            attacker_properties.element_blast_defense_scale_bits =
                element_blast_defense.max(0.01).to_bits();
        }
        if attacker_properties.full_miss_scale() < 0.01 {
            attacker_properties.full_miss_scale_bits = full_miss.max(0.01).to_bits();
        }
        if attacker_properties.critical_rate() < 1.0 {
            attacker_properties.critical_rate_bits =
                game.globe_setup.critical_rate().max(1.0).to_bits();
        }
        let (weapon_divisor, weapon_minimum) = game.globe_setup.weapon_damage_factors();
        let damage_factor = game
            .find_player(player_id)
            .expect("базовая атака сохраняет player owner до расчёта")
            .weapon_modifier(
                &game.goods_factory,
                i32::from(target_level),
                weapon_divisor,
                weapon_minimum,
            );
        let minimum = attacker_properties.minimum_attack as i32;
        let maximum = attacker_properties.maximum_attack as i32;
        let span = maximum.wrapping_sub(minimum).max(0);
        let physical = minimum.wrapping_add(game_legacy_random(&mut game.random_state, span));
        let mut attack = AttackInformation {
            skill_id: BASE_ATTACK_SKILL_ID,
            skill_level: skill_level as u8,
            attacker_type: PLAYER_TYPE,
            attacker_id: player_id,
            attacker_team_id: attacker_team,
            attacker_faction_id: attacker_faction,
            attacker_union_id: attacker_union,
            hit_modifier,
            damage_factor,
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
                AttackPower {
                    kind: AttackPowerType::Element,
                    hp_damage: attacker_properties.add_element_attack as i32,
                    mp_damage: 0,
                },
                AttackPower {
                    kind: AttackPowerType::Soul,
                    hp_damage: i32::from(attacker_properties.add_soul_attack),
                    mp_damage: 0,
                },
            ],
        };
        if game_legacy_random(&mut game.random_state, 100) < i32::from(attacker_properties.cch)
        {
            attack.critical = true;
            let critical_rate = game.globe_setup.critical_rate();
            for power in &mut attack.damages {
                power.hp_damage = truncate_original(
                    f64::from(power.hp_damage) * f64::from(critical_rate),
                );
            }
        }
        let pillar_damage_factor = game.find_player(target_id)
            .and_then(CPlayer::pillar_state).map(|state| state.damage_factor());
        let mut defense_shields = game
            .find_player_mut(target_id)
            .map(CPlayer::take_defense_shields)
            .unwrap_or_default();
        let mut random = |maximum| game_legacy_random(&mut game.random_state, maximum);
        defend_player_base_attack(
            &mut attack,
            attacker_properties,
            attacker_occupation,
            target_properties,
            target_mana,
            target_war_soul_mana,
            &game.globe_setup,
            &mut random,
            &mut defense_shields,
            pillar_damage_factor,
        );
        if let Some(target) = game.find_player_mut(target_id) {
            target.restore_defense_shields(defense_shields);
        }
        let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Calculate, SkillStage::Attack));
        first_contact = true;
        let (damage, mana_damage) =
            CGame::applied_attack_damage(&attack, target_health, target_mana);
        if damage != 0 || mana_damage != 0 {
            let current_health = target_health - damage;
            if let Some(target) = game.find_player_mut(target_id) {
                target.set_health(current_health);
                target.set_mana(target_mana - mana_damage);
                if current_health == 0 || attack.full_miss == 0 {
                    target
                        .movement_shape_mut()
                        .set_action(if current_health == 0 { 6 } else { 5 });
                }
            }
            if damage != 0 {
                game.increase_owned_player_rp(target_id, false, damage as u16);
            }
            if current_health != 0 && attack.full_miss == 0 {
                let _ = game.queue_player_hurt_ai(target_id, damage, runtime);
                let _ = game.retarget_passive_pets_after_player_hurt(
                    target_id,
                    ShapeIdentity {
                        object_type: PLAYER_TYPE,
                        id: player_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                );
                let _ = game.notify_country_after_player_hurt(
                    target_id,
                    ShapeIdentity {
                        object_type: PLAYER_TYPE,
                        id: player_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                    runtime,
                );
                let _ = finish_player_blind_states_on_defense(game, target_id, now_ms);
            }
            if current_health == 0 {
                let mut died = CMessage::new(0x000b_f60b);
                died.add_long(PLAYER_TYPE);
                died.add_long(player_id);
                died.add_long(PLAYER_TYPE);
                died.add_long(target_id);
                died.add_ulong(damage);
                died.base_mut().add_char(1);
                CGame::append_base_attack_tail(&mut died, &attack);
                let _ = game.send_player_shape_around(target_id, None, &died);
                killing_blow = Some(PlayerKillingBlow {
                    victim_id: target_id,
                    attacker_type: PLAYER_TYPE,
                    attacker_id: player_id,
                    attacker_faction_id: attacker_faction,
                });
            } else if attack.full_miss != 0 {
                let mut missed = CMessage::new(0x000b_f612);
                missed.add_byte(attack.full_miss);
                missed.add_long(PLAYER_TYPE);
                missed.add_long(target_id);
                let _ = game.send_player_shape_around(target_id, None, &missed);
            } else {
                let mut hurt = CMessage::new(0x000b_f60a);
                hurt.add_long(PLAYER_TYPE);
                hurt.add_long(player_id);
                hurt.add_long(PLAYER_TYPE);
                hurt.add_long(target_id);
                CGame::append_hurt_damage_records(&mut hurt, damage, mana_damage);
                hurt.add_ulong(current_health);
                CGame::append_base_attack_tail(&mut hurt, &attack);
                let _ = game.send_player_shape_around(target_id, None, &hurt);
                game.damage_player_armor(target_id, runtime);
            }
        } else if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(PLAYER_TYPE);
            missed.add_long(target_id);
            let _ = game.send_player_shape_around(target_id, None, &missed);
        }
        game.increase_owned_player_rp(player_id, true, 0);
        if let Some(attacker) = game.find_player_mut(player_id) {
            attacker.movement_shape_mut().set_action(1);
        }
    } else if target_type == MONSTER_TYPE {
        let Some(region_id) = game
            .find_player(player_id)
            .and_then(CPlayer::server_region_id)
        else {
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        };
        let monster_property = game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target_id))
            .and_then(CMonster::base_property_key)
            .and_then(|key| game.find_monster_property_by_origin_name(key))
            .cloned();
        let Some(monster_property) = monster_property else {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        };
        let monster_snapshot = game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target_id)?;
            Some((
                monster.combat_properties(&monster_property),
                monster.hit_points(),
                monster.is_tamed(),
                monster.is_carriage(&monster_property),
                monster.move_shape().is_god(),
                monster.master_info(),
            ))
        });
        let Some((
            monster_properties,
            monster_health,
            monster_tamed,
            monster_carriage,
            monster_god,
            monster_master,
        )) = monster_snapshot
        else {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        };
        if monster_health == 0 || monster_god {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
        if !game.monster_attackable_by_player(player_id, region_id, &monster_property) {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
        let owned_target_player = ((monster_tamed || monster_carriage)
            && monster_master.master_type == PLAYER_TYPE
            && monster_master.master_id != 0)
            .then_some(monster_master.master_id);
        if let Some(owner_id) = owned_target_player
            && owner_id != player_id
            && let Some((string_id, limit)) =
                game.player_base_attack_level_block(player_id, owner_id)
        {
            game.send_base_attack_level_block(player_id, string_id, limit);
            game.enter_player_combat_state(player_id);
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
        let owned_target_attackable = owned_target_player.is_none_or(|owner_id| {
            if owner_id == player_id {
                game.find_player(player_id)
                    .is_some_and(|player| player.pk_permissions().criminal)
            } else {
                game.player_base_attackable(player_id, owner_id)
            }
        });
        if !owned_target_attackable {
            game.enter_player_combat_state(player_id);
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
        if let Some(owner_id) = owned_target_player {
            let _ = game.player_on_first_skill(player_id, owner_id, Some(region_id), runtime);
        }
        game.apply_guard_monster_first_attack(player_id, region_id, &monster_property, now_ms);

        let (
            mut attacker_properties,
            attacker_occupation,
            attacker_level,
            attacker_team,
            attacker_faction,
            attacker_union,
        ) = {
            let attacker = game
                .find_player(player_id)
                .expect("base-attack attacker сохранён");
            (
                attacker.combat_properties(),
                attacker.occupation(),
                attacker.level(),
                attacker.team_id(),
                attacker.faction_id(),
                attacker.union_id(),
            )
        };
        let [
            blast_attack,
            blast_defense,
            element_blast_attack,
            element_blast_defense,
            full_miss,
        ] = game.globe_setup.base_combat_scales();
        if attacker_properties.blast_attack_scale() < 1.0 {
            attacker_properties.blast_attack_scale_bits = blast_attack.max(1.0).to_bits();
        }
        if attacker_properties.blast_defense_scale() < 0.01 {
            attacker_properties.blast_defense_scale_bits = blast_defense.max(0.01).to_bits();
        }
        if attacker_properties.element_blast_attack_scale() < 1.0 {
            attacker_properties.element_blast_attack_scale_bits =
                element_blast_attack.max(1.0).to_bits();
        }
        if attacker_properties.element_blast_defense_scale() < 0.01 {
            attacker_properties.element_blast_defense_scale_bits =
                element_blast_defense.max(0.01).to_bits();
        }
        if attacker_properties.full_miss_scale() < 0.01 {
            attacker_properties.full_miss_scale_bits = full_miss.max(0.01).to_bits();
        }
        if attacker_properties.critical_rate() < 1.0 {
            attacker_properties.critical_rate_bits =
                game.globe_setup.critical_rate().max(1.0).to_bits();
        }
        let minimum = attacker_properties.minimum_attack as i32;
        let maximum = attacker_properties.maximum_attack as i32;
        let span = maximum.wrapping_sub(minimum).max(0);
        let physical = minimum.wrapping_add(game_legacy_random(&mut game.random_state, span));
        let (weapon_divisor, weapon_minimum) = game.globe_setup.weapon_damage_factors();
        let damage_factor = game
            .find_player(player_id)
            .expect("базовая атака сохраняет player owner до расчёта")
            .weapon_modifier(
                &game.goods_factory,
                i32::from(monster_properties.level),
                weapon_divisor,
                weapon_minimum,
            );
        let mut attack = AttackInformation {
            skill_id: BASE_ATTACK_SKILL_ID,
            skill_level: skill_level as u8,
            attacker_type: PLAYER_TYPE,
            attacker_id: player_id,
            attacker_team_id: attacker_team,
            attacker_faction_id: attacker_faction,
            attacker_union_id: attacker_union,
            hit_modifier,
            damage_factor,
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
                AttackPower {
                    kind: AttackPowerType::Element,
                    hp_damage: attacker_properties.add_element_attack as i32,
                    mp_damage: 0,
                },
                AttackPower {
                    kind: AttackPowerType::Soul,
                    hp_damage: i32::from(attacker_properties.add_soul_attack),
                    mp_damage: 0,
                },
            ],
        };
        if game_legacy_random(&mut game.random_state, 100) < i32::from(attacker_properties.cch)
        {
            attack.critical = true;
            let critical_rate = game.globe_setup.critical_rate();
            for power in &mut attack.damages {
                power.hp_damage = truncate_original(
                    f64::from(power.hp_damage) * f64::from(critical_rate),
                );
            }
        }
        let mut random = |maximum| game_legacy_random(&mut game.random_state, maximum);
        defend_monster_base_attack(
            &mut attack,
            attacker_properties,
            attacker_occupation,
            attacker_level,
            monster_properties,
            &game.globe_setup,
            &mut random,
        );
        let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Calculate, SkillStage::Attack));
        first_contact = true;
        let damage = attack.hp_damage().min(monster_health);
        let current_health = monster_health - damage;
        let lord_hurt_plan = (monster_property.ai == 19
            && attack.full_miss == 0
            && damage != 0
            && current_health != 0)
            .then(|| {
                crate::gameserver::appserver::ai::lord::plan_lord_hurt_response(
                    game,
                    region_id,
                    target_id,
                    &monster_property,
                )
            });
        let mut owner = game
            .take_region_owner(region_id)
            .expect("monster target region сохранён");
        let stiffen_setup = game.globe_setup.stiffen_setup();
        let mut stiffen_delay = 0;
        if let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id) {
            if attack.full_miss == 0 && damage != 0 && current_health != 0 {
                stiffen_delay = monster.roll_stiffen(
                    damage,
                    &monster_property,
                    stiffen_setup,
                    || runtime.now_milliseconds(),
                    |maximum| game_legacy_random(&mut game.random_state, maximum),
                );
            }
            monster.set_hit_points(current_health);
            if damage != 0 && (attack.full_miss == 0 || current_health == 0) {
                monster
                    .move_shape_mut()
                    .shape_mut()
                    .set_action(if current_health == 0 { 6 } else { 5 });
                if current_health == 0 {
                    monster.when_been_killed(now_ms);
                } else if monster_property.ai == 1 {
                    monster.when_passive_gladiator_hurted_by(
                        ShapeIdentity {
                            object_type: PLAYER_TYPE,
                            id: player_id,
                            ex_id: CGuid::GUID_INVALID,
                        },
                        now_ms,
                        false,
                    );
                } else if monster_property.ai == 2 {
                    // Владелец AI2 применит реакцию после освобождения
                    // изменяемого заимствования монстра.
                } else if monster_property.ai == 13 {
                    // Поиск AI13 выполняется после освобождения изменяемого
                    // заимствования монстра.
                } else if monster_property.ai == 11 {
                    // Поиск AI11 выполняется после освобождения изменяемого
                    // заимствования монстра.
                } else if monster_property.ai == 20 {
                    // AI20 разрешает игрока и связывает близнеца после
                    // освобождения изменяемого заимствования монстра.
                } else if monster_property.ai == 19 {
                    // AI19 применяет Defense, spatial-step и выбор цели
                    // после освобождения заимствования монстра.
                } else if matches!(monster_property.ai, 8 | 17 | 100 | 101) {
                    monster.when_been_hurted(now_ms);
                } else {
                    monster.when_been_hurted_by(
                        ShapeIdentity {
                            object_type: PLAYER_TYPE,
                            id: player_id,
                            ex_id: CGuid::GUID_INVALID,
                        },
                        false,
                        now_ms,
                    );
                }
            }
            if current_health == 0 {
                monster.set_killed_by(MonsterKillingAttack {
                    attacker_type: PLAYER_TYPE,
                    attacker_id: player_id,
                    skill_id: attack.skill_id,
                    skill_level: attack.skill_level,
                    critical: attack.critical,
                    blast_attack: attack.blast_attack,
                });
            }
        }
        if attack.full_miss == 0
            && damage != 0
            && current_health != 0
            && monster_property.ai == 2
        {
            crate::gameserver::appserver::ai::smartgladiator::apply_player_hurt_response(
                game,
                owner.base_mut(),
                target_id,
                &monster_property,
                player_id,
                runtime,
            );
        }
        if attack.full_miss == 0
            && damage != 0
            && current_health != 0
            && monster_property.ai == 11
        {
            crate::gameserver::appserver::ai::cityguardwithbow::retarget_city_bow_guard_after_hurt(
                game,
                owner.base_mut(),
                target_id,
                &monster_property,
                now_ms,
            );
        }
        if attack.full_miss == 0
            && damage != 0
            && current_health != 0
            && monster_property.ai == 13
        {
            crate::gameserver::appserver::ai::vilcouguardwithbow::retarget_village_bow_guard_after_hurt(
                game,
                owner.base_mut(),
                target_id,
                &monster_property,
                now_ms,
            );
        }
        if attack.full_miss == 0
            && damage != 0
            && current_health != 0
            && monster_property.ai == 20
        {
            let _ = retarget_jiumai_after_hurt(
                game,
                owner.base_mut(),
                target_id,
                ShapeIdentity {
                    object_type: PLAYER_TYPE,
                    id: player_id,
                    ex_id: CGuid::GUID_INVALID,
                },
                runtime,
            );
        }
        if let Some(plan) = lord_hurt_plan {
            let _ = crate::gameserver::appserver::ai::lord::apply_lord_hurt_response(
                game,
                owner.base_mut(),
                target_id,
                ShapeIdentity {
                    object_type: PLAYER_TYPE,
                    id: player_id,
                    ex_id: CGuid::GUID_INVALID,
                },
                || runtime.now_milliseconds(),
                plan,
            );
        }
        if attack.full_miss == 0
            && damage != 0
            && current_health != 0
            && matches!(monster_property.ai, 8 | 17 | 100 | 101)
        {
            crate::gameserver::appserver::ai::guardcountry::retarget_special_guard_after_hurt(
                game,
                owner.base_mut(),
                target_id,
                &monster_property,
            );
        }
        if stiffen_delay != 0
            && let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id)
        {
            monster.when_been_stiffened(stiffen_delay, runtime.now_milliseconds());
        }
        if attack.full_miss == 0 && damage != 0 && current_health != 0 {
            let _ = finish_blind_states_on_defense(
                game,
                owner.base_mut(),
                ShapeIdentity {
                    object_type: MONSTER_TYPE,
                    id: target_id,
                    ex_id: CGuid::GUID_INVALID,
                },
                now_ms,
            );
        }
        game.restore_region_owner(owner);

        if attack.full_miss != 0 && current_health != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(MONSTER_TYPE);
            missed.add_long(target_id);
            let _ = game.send_shape_position_around(region_id, target_x, target_y, &missed);
        } else if damage != 0 {
            if current_health == 0 {
                let mut died = CMessage::new(0x000b_f60b);
                died.add_long(PLAYER_TYPE);
                died.add_long(player_id);
                died.add_long(MONSTER_TYPE);
                died.add_long(target_id);
                died.add_ulong(damage);
                died.base_mut().add_char(1);
                CGame::append_base_attack_tail(&mut died, &attack);
                let _ = game.send_shape_position_around(region_id, target_x, target_y, &died);
            } else {
                let mut hurt = CMessage::new(0x000b_f60a);
                hurt.add_long(PLAYER_TYPE);
                hurt.add_long(player_id);
                hurt.add_long(MONSTER_TYPE);
                hurt.add_long(target_id);
                hurt.add_byte(1);
                hurt.add_byte(0);
                hurt.add_ulong(damage);
                hurt.add_ulong(current_health);
                CGame::append_base_attack_tail(&mut hurt, &attack);
                let _ = game.send_shape_position_around(region_id, target_x, target_y, &hurt);
                let _ =
                    game.monster_on_been_hurted(region_id, target_id, PLAYER_TYPE, player_id);
                if let Some(mut owner) = game.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id) {
                        monster.register_attacking_player(
                            player_id,
                            now_ms,
                            game.globe_setup.attack_monster_protection_ms(),
                        );
                    }
                    game.restore_region_owner(owner);
                }
            }
        }
        game.increase_owned_player_rp(player_id, true, 0);
        if let Some(attacker) = game.find_player_mut(player_id) {
            attacker.movement_shape_mut().set_action(1);
        }
    } else if target_type == BUILD_OBJECT_TYPE as i32
        || target_type == CITY_GATE_OBJECT_TYPE as i32
    {
        let Some(region_id) = game
            .find_player(player_id)
            .and_then(CPlayer::server_region_id)
        else {
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        };
        let identity = ShapeIdentity {
            object_type: target_type,
            id: target_id,
            ex_id: CGuid::GUID_INVALID,
        };
        if !game.stationary_build_attackable_by_player(player_id, region_id, identity)
            || !execute_player_stationary_attack(
                game,
                player_id,
                region_id,
                identity,
                skill_level,
                hit_modifier,
                target_x,
                target_y,
                runtime,
            )
        {
            let _ = game.send_base_attack_failure(player_id, 2);
            finish_player_base_attack(game, player_id, player_ai, runtime);
            return rejected();
        }
        first_contact = true;
        game.increase_owned_player_rp(player_id, true, 0);
        if let Some(attacker) = game.find_player_mut(player_id) {
            attacker.movement_shape_mut().set_action(1);
        }
    }
    let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Calculate, SkillStage::Attack));
    let _ = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID).is_some_and(|state| state.advance(SkillStage::Attack, SkillStage::Apply));
    finish_player_base_attack(game, player_id, player_ai, runtime);
    QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Completed,
        first_contact,
        killing_blow,
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_player_stationary_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    identity: ShapeIdentity,
    skill_level: i32,
    hit_modifier: i32,
    target_x: i32,
    target_y: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(target) = game.stationary_build_combat_snapshot(region_id, identity) else {
        return false;
    };
    if target.hp == 0 {
        return false;
    }
    let (
        mut attacker_properties,
        attacker_occupation,
        attacker_team,
        attacker_faction,
        attacker_union,
    ) = {
        let Some(attacker) = game.find_player(player_id) else {
            return false;
        };
        (
            attacker.combat_properties(),
            attacker.occupation(),
            attacker.team_id(),
            attacker.faction_id(),
            attacker.union_id(),
        )
    };
    let [
        blast_attack,
        blast_defense,
        element_blast_attack,
        element_blast_defense,
        full_miss,
    ] = game.globe_setup.base_combat_scales();
    if attacker_properties.blast_attack_scale() < 1.0 {
        attacker_properties.blast_attack_scale_bits = blast_attack.max(1.0).to_bits();
    }
    if attacker_properties.blast_defense_scale() < 0.01 {
        attacker_properties.blast_defense_scale_bits = blast_defense.max(0.01).to_bits();
    }
    if attacker_properties.element_blast_attack_scale() < 1.0 {
        attacker_properties.element_blast_attack_scale_bits =
            element_blast_attack.max(1.0).to_bits();
    }
    if attacker_properties.element_blast_defense_scale() < 0.01 {
        attacker_properties.element_blast_defense_scale_bits =
            element_blast_defense.max(0.01).to_bits();
    }
    if attacker_properties.full_miss_scale() < 0.01 {
        attacker_properties.full_miss_scale_bits = full_miss.max(0.01).to_bits();
    }
    if attacker_properties.critical_rate() < 1.0 {
        attacker_properties.critical_rate_bits =
            game.globe_setup.critical_rate().max(1.0).to_bits();
    }

    let (weapon_divisor, weapon_minimum) = game.globe_setup.weapon_damage_factors();
    let damage_factor = game
        .find_player(player_id)
        .expect("стационарная атака сохраняет player owner до расчёта")
        .weapon_modifier(
            &game.goods_factory,
            0,
            weapon_divisor,
            weapon_minimum,
        );
    let minimum = attacker_properties.minimum_attack as i32;
    let maximum = attacker_properties.maximum_attack as i32;
    let span = maximum.wrapping_sub(minimum).max(0);
    let physical = minimum.wrapping_add(game_legacy_random(&mut game.random_state, span));
    let mut attack = AttackInformation {
        skill_id: BASE_ATTACK_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: attacker_team,
        attacker_faction_id: attacker_faction,
        attacker_union_id: attacker_union,
        hit_modifier,
        damage_factor,
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
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: attacker_properties.add_element_attack as i32,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(attacker_properties.add_soul_attack),
                mp_damage: 0,
            },
        ],
    };
    if game_legacy_random(&mut game.random_state, 100) < i32::from(attacker_properties.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup.critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(critical_rate),
            );
        }
    }
    let mut random = |maximum| game_legacy_random(&mut game.random_state, maximum);
    defend_build_base_attack(
        &mut attack,
        attacker_properties,
        attacker_occupation,
        target.defense,
        target.element_resistance,
        &game.globe_setup,
        &mut random,
    );

    game.apply_defended_player_attack_to_stationary_build(
        player_id,
        region_id,
        identity,
        target_x,
        target_y,
        &attack,
        runtime,
    )
}
