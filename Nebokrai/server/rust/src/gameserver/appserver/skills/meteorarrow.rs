//! Выпуск накопленных метеорных стрел CMeteorArrow (0xCD).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrow.cpp.
//! Общий Begin сохраняет исходного U и ранние часы. Check проверяет reuse,
//! свежий путь, дальность и block2; источник не типа Player допускается без Move0.
//! Игроку нужны лук категории 3 и ненулевая цена MP. Signed DWORD-разность
//! разрешает Move0; нулевая цена означает тихий отказ. Запас стрел в Check
//! не проверяется. Общие с MeteorArrowMass проверки ресурсов остаются здесь.
//!
//! Каждый AI заново получает свойства, U и S. Смерть U не проверяется;
//! отсутствующая S использует базовую точку, мёртвая S даёт End0. Первый AI
//! удерживает координаты S через MP→OnChangeStates→повторную проверку лука;
//! затем CAN, направление, первый IDCC с ненулевым signed запасом и visual0.
//! Поздний отказ не возвращает MP. Срок выпуска — unsigned start+delay.
//!
//! Выпуск повторно выбирает первый IDCC, сохраняет количество и вызывает
//! только его деструктор, даже при чужом RTTI. Ни End состояния, ни отдельного
//! обновления свойств здесь нет. Ненулевой запас, включая отрицательный,
//! разрешает visual1 и Summon; ошибки завершаются End0, выпуск — End1.
//! Общий End сбрасывает фазу, возвращает движение свежему U и вызывает
//! Summon End с настоящим аргументом. Поколенческий ключ заменяет указатель.
//!
//! Summon отдельно читает MasterInfo с country0, компоненты игрока и свежую
//! таблицу; затем HIT→CCH→MAX→MIN→частоту, часы конструктора и ID. SetTileXY
//! предшествует Initialize/RNG; фактический регион U проверяется после них.
//! Региональный owner выполняет Add и публикацию независимо от результата Add.
//! Отсутствующие native объекты безопасно отклоняются без разыменования NULL.

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::meteorarrowphalanx::CMeteorArrowPhalanx;
use super::meteorarrowstate::{consume_meteor_arrow_count, first_meteor_arrow_count};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::{SourceProperty, source_property};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const METEOR_ARROW_SKILL_ID: u32 = 0xcd;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;

pub(super) fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, mode: u32) {
    game.update_registered_skill_visual(instance, mode);
    let Some(player) = player else { return; };
    let text: &[u8] = match mode {
        4 => b"GS0300", 10 => b"GS0285", 11 => b"GS0290", 13 => b"GS0278",
        14 => b"GS0297", 15 => b"GS0282", _ => return,
    };
    game.send_skill_system_info(player, text);
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player, b"GS0288", amount);
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon|
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3)
}

pub(super) fn check_arrow_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    check_path: bool, runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player, 13);
        return false;
    }
    if check_path {
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if path.len() as u32 > maximum {
                failure(game, instance, player, 11);
                return false;
            }
        }
        if path.iter().any(|cell| cell.2 == 2) {
            failure(game, instance, player, 15);
            return false;
        }
    }
    let Some(player) = player else { return true; };
    if game.find_player(player).is_none_or(|source| !weapon_is_valid(game, source)) {
        failure(game, instance, Some(player), 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
    if (mana.wrapping_sub(properties.query_property(USER_MP_LOSE)) as i32) < 0 {
        mana_failure(game, instance, player, &properties);
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

pub(super) fn prepare_arrow_player(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
) -> bool {
    let Some(player) = player else { return true; };
    let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        mana_failure(game, instance, player, properties);
        return false;
    }
    let Some(user) = game.find_player_mut(player) else { return false; };
    user.set_mana(remaining);
    game.publish_player_states(player);
    if game.find_player(player).is_none_or(|user| !weapon_is_valid(game, user)) {
        failure(game, instance, Some(player), 14);
        return false;
    }
    true
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), arrows: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let identity = user.shape().identity();
    let mut master = MasterInfo { master_type: identity.object_type, master_id: identity.id, ..Default::default() };
    let (element, soul) = if identity.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(identity.id) else { return; };
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        let permissions = player.pk_permissions();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
        let element = player.combat_properties().add_element_attack as i32;
        let soul = i32::from(player.combat_properties().add_soul_attack);
        (element, soul)
    } else { (0, 0) };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some(cch) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    let cch = i32::from(cch as u16);
    let Some(maximum) = source_property(game, source, SourceProperty::Maximum) else { return; };
    let Some(minimum) = source_property(game, source, SourceProperty::Minimum) else { return; };
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CMeteorArrowPhalanx::new(
        id, master, started, frequency, level, minimum as i32, maximum as i32,
        element, soul, cch, hit, arrows as u32,
    );
    phalanx.shape_mut().set_pos_xy_base(destination.0 as f32 + 0.5, destination.1 as f32 + 0.5);
    phalanx.initialize_cells(|maximum| game.skill_random_below(maximum));
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.spawn_meteor_arrow_phalanx(region, phalanx, started, runtime);
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let player = source.filter(|source| source.1.object_type == PLAYER_TYPE).map(|source| source.1.id);
    let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
        Some((region, identity)) => {
            if game.move_shape_health(region, identity) == Some(0) {
                failure(game, instance, player, 10);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
            (x, y)
        }
        None => skill.lifecycle().destination(),
    };
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    if stage == SkillStage::Begin {
        if !prepare_arrow_player(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        if first_meteor_arrow_count(game, source).unwrap_or(0) == 0 {
            failure(game, instance, player, 4);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    let arrows = consume_meteor_arrow_count(game, source);
    if arrows == 0 {
        failure(game, instance, player, 4);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, destination, arrows, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_meteor_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ArrowCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_arrow_cast(game, instance, source, true, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
