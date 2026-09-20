//! Зарегистрированный вход JuCut, LightningSword1–4 и InverseChopped.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/jucut.cpp,
//! lightningsword*.cpp и inversechopped.cpp.
//!
//! Begin сохраняет исходного U, раннее время и loop1 visual; отказ Check вызывает End(0)
//! без дополнительного visual2. Обычные варианты после reuse допускают
//! непользовательский U без Move0, а игроку требуют оружие и ненулевую цену MP.
//! Inverse требует игрока, но разрешает MP0. Категория 2 нужна JuCut,
//! LightningSword3 и Inverse, остальным — категория 1.
//!
//! Первый AI списывает MP до OnChangeStates; обычные варианты затем повторяют
//! проверку оружия, сохраняя уже списанный ресурс при отказе. Inverse вместо
//! этого проверяет игрока и его жизнь при каждом AI. CAN предшествует свежему
//! GetS, повороту и visual0. Таблица свойств заморожена на один AI; Calculate
//! разрешает собственную таблицу. Абсолютный unsigned срок start+delay
//! выпускает visual1 и общий frontcellsword-tail, без раннего отсечения
//! региона. Только общий End возвращает движение и выполняет AfterUse/reuse.
//! Исполнение и AI опубликованы общим зарегистрированным владельцем целиком.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::frontcellsword::run_front_cell_sword_attack;
use super::inversechopped::INVERSE_CHOPPED_SKILL_ID;
use super::jucut::JU_CUT_SKILL_ID;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::lightningsword::LIGHTNING_SWORD_SKILL_ID;
use super::lightningsword2::LIGHTNING_SWORD_2_SKILL_ID;
use super::lightningsword3::LIGHTNING_SWORD_3_SKILL_ID;
use super::lightningsword4::LIGHTNING_SWORD_4_SKILL_ID;
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
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

const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;

#[derive(Clone, Copy)]
struct FrontCellSwordProfile {
    weapon_category: i32,
    inverse: bool,
}

impl FrontCellSwordProfile {
    fn from_skill_id(id: u32) -> Option<Self> {
        let weapon_category = match id {
            JU_CUT_SKILL_ID | LIGHTNING_SWORD_3_SKILL_ID | INVERSE_CHOPPED_SKILL_ID => 2,
            LIGHTNING_SWORD_SKILL_ID | LIGHTNING_SWORD_2_SKILL_ID | LIGHTNING_SWORD_4_SKILL_ID => 1,
            _ => return None,
        };
        Some(Self { weapon_category, inverse: id == INVERSE_CHOPPED_SKILL_ID })
    }

    fn weapon_failure(self) -> &'static [u8] {
        if self.weapon_category == 2 { b"GS0292" } else { b"GS0287" }
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_compatible(game: &CGame, player: &CPlayer, profile: FrontCellSwordProfile) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == profile.weapon_category
    })
}

fn failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>,
    profile: FrontCellSwordProfile, mode: u32,
) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id {
        let text = match mode { 13 => &b"GS0278"[..], 14 => profile.weapon_failure(), _ => return };
        game.send_skill_system_info(player_id, text);
    }
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    profile: FrontCellSwordProfile, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let user = source.shape().identity();
    let player_id = (user.object_type == PLAYER_TYPE).then_some(user.id);
    if profile.inverse && player_id.is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, profile, 13);
        return false;
    }
    let Some(player_id) = player_id else { return true; };
    let Some(player) = game.find_player(player_id) else { return false; };
    if !weapon_is_compatible(game, player, profile) {
        failure(game, instance, Some(player_id), profile, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) == 0 {
        if !profile.inverse { return false; }
    } else {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            mana_failure(game, instance, player_id, &properties);
            return false;
        }
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, profile: FrontCellSwordProfile, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if profile.inverse && (user.1.object_type != PLAYER_TYPE
        || game.find_player(user.1.id).is_none_or(CPlayer::is_dead))
    { return terminal(QueuedSkillExecutionState::Rejected); }
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, user.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
            if !profile.inverse && game.find_player(user.1.id).is_none_or(|player| !weapon_is_compatible(game, player, profile)) {
                failure(game, instance, Some(user.1.id), profile, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
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
    run_front_cell_sword_attack(game, instance, user, profile.inverse, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_front_cell_sword<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(profile) = FrontCellSwordProfile::from_skill_id(dispatch.skill_id()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::FrontCellSword,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, profile, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        |game, instance, runtime| run_ai(game, instance, profile, runtime),
    )
}
