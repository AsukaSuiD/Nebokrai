//! Исполнение CBaseAttack игроком.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/baseattack.cpp.
//! Этот дочерний модуль CGame хранит только caller: Begin, подготовку, формулу
//! и visual. Общий OnBeenAttacked цели владеет защитами, уроном, AI и смертью;
//! после его возврата Attack отдельно начисляет RP источнику. End и износ
//! оружия выполняет зарегистрированный владелец навыка.

use super::{
    AttackInformation, AttackPower, AttackPowerType, BASE_ATTACK_SKILL_ID,
    BaseAttackExecutionState, CGame, CMessage, CPlayer, CPlayerAI, GameMainLoopRuntime,
    PLAYER_TYPE, PlayerSkillDispatch, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    ShapeIdentity, SkillStage, get_line_direction, truncate_original,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::skills::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use crate::gameserver::appserver::skills::kernel::SkillTermination;
use crate::gameserver::appserver::skills::skillfactory::SkillOwner;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn skill_level(game: &CGame, player_id: i32) -> Option<i32> {
    game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
}

fn target(game: &CGame, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)?;
    let (region, identity) = resolve_skill_sufferer(game, lifecycle)?;
    let shape = resolve_state_move_shape(game, region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_base_attack_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CBaseAttack
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::BaseAttack || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
        let (x, y) = if let Some(target) = target {
            let shape = target.shape();
            let (Ok(x), Ok(y)) = (shape.get_tile_x(), shape.get_tile_y()) else { return; };
            (x, y)
        } else {
            skill.lifecycle().destination()
        };
        message.add_long(target.map_or(0, |target| target.shape().identity().object_type));
        message.add_long(target.map_or(0, |target| target.shape().identity().id));
        message.add_long(x);
        message.add_long(y);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn calculate_attack(game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
    let Some(level) = skill_level(game, player_id) else { return; };
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else { return; };
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let Some(player) = game.find_player(player_id) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    attack.damage_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let maximum = player.combat_properties().maximum_attack as i32;
    let minimum = player.combat_properties().minimum_attack as i32;
    let span = maximum.wrapping_sub(minimum).max(0);
    let minimum = player.combat_properties().minimum_attack as i32;
    // В отличие от Strike, верхняя граница исключена и отрицательная ширина
    // обнуляется. Повторный GetMinAttack предшествует RNG, а не следует за ним.
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let Some(player) = game.find_player(player_id) else { return; };
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let critical = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            // Между x87 multiply и усечением к int нет промежуточной записи float.
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) {
    let Some(target) = target else { return; };
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return; };
    if (target.1.object_type == user.object_type && target.1.id == user.id)
        || target.1.object_type == 500
        || !game.live_skill_target_attackable(target.0, user, target.1)
    { return; }
    let Some(player) = game.find_player(player_id) else { return; };
    let permissions = player.pk_permissions();
    let master = MasterInfo {
        master_type: user.object_type,
        master_id: user.id,
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    };
    // CBaseAttack не записывает skill ID/level в tagAttackInformation:
    // даже успешный Calculate сохраняет конструкторские UNKNOWN и 1.
    let mut attack = AttackInformation::for_master(master);
    calculate_attack(game, player_id, target, &mut attack);
    // Вызван именно общий virtual +15C (..., false), включая постройки.
    // Его внутренний отказ не отменяет последующий caller IncreaseRp.
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    game.increase_owned_player_rp(player_id, true, 0);
}

pub(super) fn execute_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let instance = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID);
    let result = execute_stage(game, player_id, dispatch, ai, runtime);
    let end = match result.state {
        QueuedSkillExecutionState::Begun | QueuedSkillExecutionState::Pending => None,
        QueuedSkillExecutionState::Completed => Some((1, SkillTermination::Completed)),
        QueuedSkillExecutionState::Rejected => Some((0, SkillTermination::Rejected)),
        QueuedSkillExecutionState::RejectedAfterUse => Some((1, SkillTermination::Rejected)),
    };
    if let Some((instance, (argument, termination))) = instance.zip(end)
        && game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended())
    {
        game.with_published_player_ai(player_id, ai, |game| {
            let _ = game.end_registered_instance(instance, argument, termination, runtime);
        });
    }
    result
}

fn execute_stage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).is_none() {
        game.replace_player_skill_visual_effect(
            player_id, BASE_ATTACK_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::BaseAttack, 1),
        );
        if game.find_player(player_id).is_none() { return outcome(QueuedSkillExecutionState::Rejected); }
        let Some(level) = skill_level(game, player_id) else { return outcome(QueuedSkillExecutionState::Rejected); };
        if game.skill_base_properties(BASE_ATTACK_SKILL_ID, level).is_none() {
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(started) = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)
            .map(|lifecycle| lifecycle.started_at_ms())
        else { return outcome(QueuedSkillExecutionState::Rejected); };
        game.begin_player_skill_execution(player_id, BaseAttackExecutionState::begin(dispatch, started));
        return outcome(QueuedSkillExecutionState::Begun);
    }
    let Some(level) = skill_level(game, player_id) else { return outcome(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else {
        return outcome(QueuedSkillExecutionState::Rejected);
    };
    let Some(player) = game.find_player(player_id) else { return outcome(QueuedSkillExecutionState::Rejected); };
    let target = target(game, player_id);
    if target.is_some_and(|(region, target)| game.base_magic_target_dead(region, target)) {
        game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 2);
        return outcome(QueuedSkillExecutionState::Completed);
    }
    if game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(source_view) = player.shape_view() else { return outcome(QueuedSkillExecutionState::Rejected); };
        let Some(destination) = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)
            .map(|lifecycle| lifecycle.destination())
        else { return outcome(QueuedSkillExecutionState::Rejected); };
        let target_view = target.and_then(|(region, target)| {
            if target.object_type == PLAYER_TYPE {
                game.find_player(target.id).and_then(CPlayer::shape_view)
            } else {
                game.base_magic_target_view(region, target)
            }
        });
        let distance = target_view.map_or_else(
            || player.shape().real_distance_to_point(destination.0, destination.1),
            |target| source_view.real_distance(Some(target)),
        );
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < distance as u32
        {
            game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 11);
            return outcome(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        let (target_x, target_y) = if let Some((region, identity)) = target {
            let Some(target) = resolve_state_move_shape(game, region, identity) else {
                return outcome(QueuedSkillExecutionState::Rejected);
            };
            let (Ok(x), Ok(y)) = (target.shape().get_tile_x(), target.shape().get_tile_y()) else {
                return outcome(QueuedSkillExecutionState::Rejected);
            };
            (x, y)
        } else { destination };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return outcome(QueuedSkillExecutionState::Rejected); };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else {
        return outcome(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID)
        .map(BaseAttackExecutionState::started_at_ms)
    else { return outcome(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return outcome(QueuedSkillExecutionState::Pending);
    }
    game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 1);
    // Attack заново разрешает S после visual; первоначальная цель AI нужна
    // только для ранней смерти, дальности и поворота.
    let target = self::target(game, player_id);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    game.with_published_player_ai(player_id, ai, |game| {
        attack(game, player_id, target, runtime)
    });
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    // OnFirstAttack принадлежит receiver; общий legacy first_contact вызвал бы
    // здесь лишний OnFirstSkill уже после попадания.
    outcome(QueuedSkillExecutionState::Completed)
}
