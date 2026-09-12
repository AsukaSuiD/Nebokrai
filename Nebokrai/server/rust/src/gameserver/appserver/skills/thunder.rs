//! CThunder (0x21F), gameserver.exe/GameServer.pdb, appserver/skills/thunder.cpp.
//!
//! Thunder/Leiming2 разделяют Check и AI; Tianhuo использует только общий
//! префикс допуска с собственными правилами часов и препятствий. Check получает
//! объектную S, захваченную до callback базового Begin, но путь — через текущий
//! GetS. Конфликт выбирается по первой позиции состояния. MP0 и отсутствие
//! WarSoul в Check дают тихий отказ; разность DWORD проверяется как signed.
//!
//! AI сохраняет таблицу свойств и U/S, при отсутствии S использует сохранённую
//! точку. Смерть S даёт End(0); отсутствие предмета оставляет ожидание. Расход
//! MP → Serialize/BF918 (включая false) → CAN → visual0 → condition предшествуют
//! абсолютному wrapping-сроку. Общий зарегистрированный вход и внешний End(int)
//! сохраняют тот же экземпляр; здесь нет второго Begin, End или хранилища цели.
//!
//! После visual1 Summon заново читает Master/предмет/таблицу, затем параметры
//! конструктора. Его clock предшествует ID, SetCenter/Initialize — проверке
//! региона U. Отказ AddShape не подавляет сериализацию; отказ самого Summon
//! не меняет последующий End(1). Область живёт независимо от навыка. Численные
//! адаптеры сохраняют x87-усечение и младший DWORD результата i64.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
};
use super::battlefairyskill::execute_registered_battle_fairy_state;
use super::battlefairytransfer::send_goods_update;
use super::fightdefense::truncate_original;
use super::kernel::{SkillStage, battle_fairy_mana_text_cost, skill_is_restored};
use super::skillbaseproperties::CSkillBaseProperties;
use super::thunderphalanx::CThunderPhalanx;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_BF_MP, GAP_BF_SPRITE};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const THUNDER_SKILL_ID: u32 = 0x21f;
pub(crate) const THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
pub(super) const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

pub(super) fn truncate_original_i64_low(value: f64) -> i32 {
    if !value.is_finite()
        || value < -9_223_372_036_854_775_808.0
        || value >= 9_223_372_036_854_775_808.0
    {
        i64::MIN as i32
    } else {
        (value as i64) as i32
    }
}

pub(super) fn scaled_battle_fairy_sprite(sprite: i32) -> i32 {
    truncate_original_i64_low(f64::from(sprite) * 0.0001)
}

pub(super) fn thunder_element_modifier(em_modifier: u32, scaled_sprite: i32) -> i32 {
    truncate_original(f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(scaled_sprite))
}

pub(super) fn thunder_base_damage(target_damage_factor: u32, sprite: i32) -> i32 {
    truncate_original_i64_low(f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6)
}

pub(super) fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: 400, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(super) fn fail_battle_fairy_summon(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, mode: u32, string_id: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    game.send_skill_system_info(player_id, string_id);
}

/// Исходная S принадлежит аргументу Begin, а GetTargetPath повторно разрешает
/// текущую S. Tianhuo читает clock раньше reuse и не проверяет figure2.
pub(super) fn check_battle_fairy_summon_prefix<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    reuse_clock_first: bool, check_obstacles: bool,
) -> Option<CSkillBaseProperties> {
    game.find_player(player_id)?;
    let target = begin_target?;
    let holder = resolve_state_move_shape(game, target.0, target.1)?;
    let conflict = holder.find_state_position(|state| matches!(state.state_id(), 0x192 | 0xd2 | 0x67))
        .and_then(|(position, _)| holder.state_at(position))
        .map(|(_, state)| state.state_id());
    if let Some(state_id) = conflict {
        let text = game.get_string_by_id(if state_id == 0xd2 { b"ZHGS0047" } else { b"ZHGS0046" });
        let mut message = CMessage::new(0x0b_f807);
        message.add_ulong(0xffff_ffff);
        let length = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
        message.base_mut().add(&text[..length]);
        message.base_mut().add_byte(0);
        let _ = message.send_to_player(game.net_server(), player_id);
        return None;
    }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let (reuse, last_used, now) = if reuse_clock_first {
        let now = runtime.now_milliseconds();
        (properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME), skill.last_used_ms(), now)
    } else {
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let last_used = skill.last_used_ms();
        (reuse, last_used, runtime.now_milliseconds())
    };
    if !skill_is_restored(last_used, reuse, now) {
        fail_battle_fairy_summon(game, instance, player_id, 13, b"ZHGS0048");
        return None;
    }
    let path = game.skill_target_path(game.registered_skill(instance)?.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        fail_battle_fairy_summon(game, instance, player_id, 11, b"ZHGS0049");
        return None;
    }
    if check_obstacles && path.iter().any(|cell| cell.2 == 2) {
        fail_battle_fairy_summon(game, instance, player_id, 15, b"ZHGS0051");
        return None;
    }
    Some(properties)
}

fn fail_mana(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost));
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(properties) = check_battle_fairy_summon_prefix(
        game, instance, player_id, begin_target, runtime, false, true,
    ) else { return false; };
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) == 0 { return false; }
    let Some(current) = game.find_player(player_id)
        .and_then(|player| player.war_soul_mana(game.goods_factory()))
    else { return false; };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

pub(super) fn execute_thunder_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    summon: impl FnOnce(&mut CGame, RegisteredSkill, (i32, ShapeIdentity), (i32, i32), &mut Runtime),
) -> QueuedSkillExecutionOutcome {
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime),
        |game, instance, runtime| run_ai(game, instance, runtime, summon),
    )
}

pub(crate) fn execute_battle_fairy_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != THUNDER_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_thunder_family(game, player_id, instance, dispatch, begin_target, runtime, summon_thunder)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    summon: impl FnOnce(&mut CGame, RegisteredSkill, (i32, ShapeIdentity), (i32, i32), &mut Runtime),
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)
        .map(|shape| (region, shape.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let position = if let Some(target) = target {
        if game.base_magic_target_dead(target.0, target.1) {
            game.update_registered_skill_visual(instance, 10);
            if let Some((_, user)) = source.filter(|(_, user)| user.object_type == 400) {
                game.send_skill_system_info(user.id, b"ZHGS0050");
            }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        (shape.shape().get_tile_x().unwrap_or(i32::MIN), shape.shape().get_tile_y().unwrap_or(i32::MIN))
    } else { skill.lifecycle().destination() };
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if source.1.object_type == 400 {
            let Some(current) = game.find_player(source.1.id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else { return terminal(QueuedSkillExecutionState::Pending); };
            let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
            let remaining = current.wrapping_sub(cost as i32);
            if remaining < 0 {
                fail_mana(game, instance, source.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(_stored) = game.set_player_equipment_addon_property(source.1.id, 10, GAP_BF_MP, 1, remaining)
            else { return terminal(QueuedSkillExecutionState::Pending); };
            // Setter не заменяет equipment: сериализуется тот же предмет без
            // повторной проверки WarSoul и без подавления частичного payload.
            let Some(goods) = game.find_player(source.1.id).and_then(|player| player.equipment().get_goods(10))
            else { return terminal(QueuedSkillExecutionState::Pending); };
            let mut old_client_payload = Vec::new();
            let _ = goods.serialize_for_old_client(
                &mut old_client_payload, game.goods_factory(), game.globe_setup().da_kong_key(),
            );
            let update = BattleFairyDefaultGoodsUpdate {
                message_type: 0x0b_f918, player_id: source.1.id,
                goods: goods.identity(), old_client_payload,
            };
            send_goods_update(game, &update);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, position, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn thunder_summon_properties(
    game: &CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
) -> Option<(MasterInfo, CSkillBaseProperties, i32)> {
    let user = resolve_state_move_shape(game, source.0, source.1)?.shape().identity();
    let mut master = MasterInfo { master_type: user.object_type, master_id: user.id, ..MasterInfo::default() };
    let mut sprite = 0;
    if user.object_type == 400 {
        let player = game.find_player(user.id)?;
        let goods = player.war_soul_goods(game.goods_factory())?;
        let permissions = player.pk_permissions();
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
        sprite = scaled_battle_fairy_sprite(goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1));
    }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let element = thunder_element_modifier(properties.query_property(SKILL_USAGE_EM_MODIFIER), sprite);
    Some((master, properties, element))
}

pub(super) fn summon_user_cch(game: &CGame, source: ShapeIdentity) -> i32 {
    if source.object_type == 400 {
        game.find_player(source.id).map_or(0, |player| i32::from(player.combat_properties().cch))
    } else { 0 }
}

pub(super) fn summon_user_add_element(game: &CGame, source: ShapeIdentity) -> i32 {
    if source.object_type == 400 {
        game.find_player(source.id).map_or(0, |player| player.combat_properties().add_element_attack as i32)
    } else { 0 }
}

pub(super) fn summon_user_region(game: &CGame, source: (i32, ShapeIdentity)) -> Option<i32> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    if !shape.is_assigned_to_server_region() { return None; }
    let region = shape.get_region_id();
    game.find_region(region)?;
    Some(region)
}

fn summon_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    position: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, element)) = thunder_summon_properties(game, instance, source) else { return; };
    let cch = summon_user_cch(game, source.1);
    let target_count = properties.query_property(SKILL_USAGE_CONST);
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let frequency = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CThunderPhalanx::new(id, master, started, lifetime, level, frequency, minimum, maximum, element, target_count, cch);
    phalanx.set_center(position.0, position.1);
    phalanx.initialize(&mut |maximum| game.skill_random_below(maximum));
    let Some(region) = summon_user_region(game, source) else { return; };
    if game.add_thunder_phalanx(region, phalanx, position.0, position.1, started, runtime)
        .is_some_and(|result| result.is_ok())
    {
        let _ = game.send_thunder_phalanx_entry(region, id, runtime);
    }
}
