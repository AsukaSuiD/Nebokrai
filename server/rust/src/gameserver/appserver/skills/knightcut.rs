//! Рыцарский удар KnightCut (0x67).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/knightcut.cpp/.h.
//!
//! Все Begin выполняют Attack-base и создают loop1-visual перед Check.
//! Check требует игрока, абсолютный срок reuse, оружие категории 1/2 и достаточные
//! MP/RP; нулевая цена допустима. Успех блокирует движение, включает фазу
//! с direction=-1 и ещё не расходует ресурсы. Общий playercast связывает
//! Begin, AI, visual и End одним поколенческим ключом.
//!
//! Первый AI сохраняет таблицу свойств и разрешённого U, списывает MP до
//! проверки RP, затем RP, OnChangeStates и повторная проверка оружия.
//! Эти траты не откатываются при последующем отказе. Свежий S либо базовая
//! точка, включая (0,0), определяет направление; SelfTarget не заменяется
//! лицевой клеткой. Запись direction предшествует SetDir, CAN — visual0,
//! а condition включается после visual. Срок выпуска — unsigned start+delay,
//! не разность часов. Visual1 предшествует новому чтению direction/региона.
//!
//! Сканирование 5×5 и полный AddKnightCutState принадлежат knightcutattack.
//! Отсутствующий регион отменяет только сканирование: успешный хвост End(1)
//! сохраняется. Отказ Begin/AI вызывает End(0). End сбрасывает
//! condition/active до нового GetUser и Move(1), затем Attack-base с исходным
//! аргументом; direction не сбрасывается. Отдельного command-tail End нет.

use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::knightcutattack::run_knight_cut_attack;
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) use super::knightcutvisual::publish_knight_cut_visual;

pub(crate) const KNIGHT_CUT_SKILL_ID: u32 = 0x67;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnightCutExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    direction: i32,
}

impl KnightCutExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), direction: -1 }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(super) const fn direction(&self) -> i32 { self.direction }
}

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_compatible(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        matches!(weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1), 1 | 2)
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0308", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, property: u32,
) {
    let (code, text): (u32, &[u8]) = match property {
        USER_MP_LOSE => (7, b"GS0288"),
        USER_RP_LOSE => (8, b"GS0289"),
        _ => return,
    };
    game.update_registered_skill_visual(instance, code);
    let amount = properties.query_property(property);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    if !weapon_is_compatible(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let rp = player.rp();
        let loss = properties.query_property(USER_RP_LOSE);
        if (u32::from(rp).wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_RP_LOSE);
            return false;
        }
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return outcome(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return outcome(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return outcome(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return outcome(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, user.1.id, &properties, USER_MP_LOSE);
                return outcome(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            let Some(player) = game.find_player(user.1.id) else { return outcome(QueuedSkillExecutionState::Rejected); };
            let rp = player.rp();
            let remaining = u32::from(rp).wrapping_sub(properties.query_property(USER_RP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, user.1.id, &properties, USER_RP_LOSE);
                return outcome(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(remaining as u16); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_compatible(game, player)) {
                failure(game, instance, user.1.id, 14);
                return outcome(QueuedSkillExecutionState::Rejected);
            }
        }
        let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map_or_else(|| skill.lifecycle().destination(), |target| {
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            });
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<KnightCutExecutionState>()) else {
            return outcome(QueuedSkillExecutionState::Rejected);
        };
        state.direction = direction;
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return outcome(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return outcome(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    let Some(direction) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<KnightCutExecutionState>()).map(KnightCutExecutionState::direction)
    else { return outcome(QueuedSkillExecutionState::Rejected); };
    run_knight_cut_attack(game, instance, user, direction, &properties, runtime);
    outcome(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_knight_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != KNIGHT_CUT_SKILL_ID { return outcome(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::KnightCut,
        check_cast, |dispatch, started| KnightCutExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
