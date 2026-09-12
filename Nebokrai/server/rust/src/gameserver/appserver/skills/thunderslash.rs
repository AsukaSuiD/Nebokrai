//! Зарегистрированный вход громового рассечения CThunderSlash (0x72).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderslash.cpp.
//! Begin сохраняет исходного U, ранний отсчёт и loop1 visual; отказ Check
//! вызывает End(0) без дополнительного visual2. После reuse непользовательский
//! U допускается без Move0; игроку нужны топор категории 1 и достаточные
//! ресурсы при ненулевой цене MP/RP. RageBreak проверяется только первым AI.
//!
//! Первый AI повторяет оружие, расходует первый ID6E без RTTI/ended-фильтра,
//! затем MP и RP по знаку DWORD-разности и вызывает OnChangeStates. Отказ
//! сохраняет предшествующие частичные эффекты. CAN предшествует свежему S,
//! направлению, visual0 и condition. Таблица свойств заморожена на один AI;
//! Summon получает отдельную свежую таблицу после visual1 и лицевой клетки.
//! Срок start+delay сравнивается как unsigned; AI не проверяет смерть U и не
//! отсекает регион заранее. Безрезультатный GetShapes/RTTI заменён прямым
//! вызовом: найденный объект не использовался и игровых callback-ов там нет.
//!
//! Снимок MasterInfo оставляет country нулевым. Боевые getter-ы читаются
//! Dex→SOUL→CCH→ELEMENT→MIN→MAX, затем frequency, clock и новый ID. У native
//! Summon после необязательного RTTI CPlayer есть разыменование NULL; безопасная
//! замена ограничивает создание формы игроком, не меняя общий Check/AI.
//! Производный End сбрасывает phase/active, возвращает движение
//! свежему U и передаёт настоящий аргумент в CSummonSkill::End. Общий владелец
//! публикует всё исполнение AI; форма, её wire и runtime живут отдельно.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME};
use super::flash::master_info;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::ragebreakstate::consume_rage_break_state;
use super::skillbaseproperties::CSkillBaseProperties;
use super::thunderslashphalanx::CThunderSlashPhalanx;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const THUNDER_SLASH_SKILL_ID: u32 = 0x72;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>, mode: u32) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id {
        let text = match mode { 4 => &b"GS0304"[..], 13 => b"GS0278", 14 => b"GS0287", _ => return };
        game.send_skill_system_info(player_id, text);
    }
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (mode, text) = if usage == USER_MP_LOSE { (7, &b"GS0288"[..]) } else { (8, &b"GS0289"[..]) };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn handle_resources(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, spend: bool,
) -> bool {
    if properties.query_property(USER_MP_LOSE) != 0 {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return false; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, player_id, properties, USER_MP_LOSE);
            return false;
        }
        if spend {
            let Some(player) = game.find_player_mut(player_id) else { return false; };
            player.set_mana(remaining);
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let Some(rp) = game.find_player(player_id).map(CPlayer::rp) else { return false; };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, player_id, properties, USER_RP_LOSE);
            return false;
        }
        if spend {
            let Some(player) = game.find_player_mut(player_id) else { return false; };
            player.set_rp(remaining as u16);
        }
    }
    true
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity), runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let identity = source.shape().identity();
    let player_id = (identity.object_type == PLAYER_TYPE).then_some(identity.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let Some(player_id) = player_id else { return true; };
    if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
        failure(game, instance, Some(player_id), 14);
        return false;
    }
    if !handle_resources(game, instance, player_id, &properties, false) { return false; }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let identity = user.shape().identity();
    let mut master = if identity.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(identity.id) else { return; };
        master_info(player)
    } else {
        crate::gameserver::appserver::masterinfo::MasterInfo {
            master_type: identity.object_type, master_id: identity.id, ..Default::default()
        }
    };
    master.master_country_id = 0;
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    if identity.object_type != PLAYER_TYPE { return; }
    let Some(player) = game.find_player(identity.id) else { return; };
    let dexterity = player.combat_properties().dexterity as i32;
    let soul = i32::from(player.combat_properties().add_soul_attack);
    let cch = i32::from(player.combat_properties().cch);
    let element = player.combat_properties().add_element_attack as i32;
    let minimum = player.combat_properties().minimum_attack as i32;
    let maximum = player.combat_properties().maximum_attack as i32;
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CThunderSlashPhalanx::new(
        id, master, started, lifetime, level, frequency, maximum, minimum,
        element, dexterity, cch, soul, x, y,
    );
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.spawn_thunder_slash_phalanx(region, phalanx, started, runtime);
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
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
                failure(game, instance, Some(user.1.id), 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if !consume_rage_break_state(game, user) {
                failure(game, instance, Some(user.1.id), 4);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if !handle_resources(game, instance, user.1.id, &properties, true) {
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            game.publish_player_states(user.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    if let Some(source) = resolve_state_move_shape(game, user.0, user.1) {
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let direction = source.shape().get_direction();
        if let Ok(front) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x, y })
            && source.shape().is_assigned_to_server_region()
            && game.find_region(source.shape().get_region_id()).is_some()
        { summon(game, instance, user, front.x, front.y, runtime); }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_thunder_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != THUNDER_SLASH_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ThunderSlash,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_ai,
    )
}
