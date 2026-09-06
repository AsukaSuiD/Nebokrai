//! Семейство бегущего удара `CGhostCut` (`0x66/0x79/0x7A`).
//! Reuse проверяется exact `CSkill::IsRestored`; шаги полёта остаются elapsed.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/ghostcut*.cpp`. После двукратной проверки меча и MP
//! навык фиксирует прямой путь вместе с исходной клеткой, затем обрабатывает
//! не более одной клетки за проход ИИ. Живой блок читается заново, а цели
//! сохраняют региональный порядок и могут быть поражены только один раз за
//! полёт. Первый вариант перед обычными фигурами отдельно проводит
//! упорядоченную ветвь боевой феи. Формула каждого попадания выполняет ровно
//! два собственных RNG-вызова; защитные RNG остаются у `CGame`. Ни одна из
//! перегрузок `Attack` не изнашивает оружие на отдельных целях: все три
//! варианта делают это один раз через унаследованный `AfterUseSkill` в общем
//! подтверждённом `End`, после освобождения пути и возврата движения.
//! Все три варианта сохраняют беззнаковый коэффициент в исходной x87-цепочке
//! до единственной записи в `float` и усекают критический множитель к нулю
//! перед `int`. Физический RNG у всех трёх получает исходную DWORD-ширину
//! `maximum - minimum + 1` без нормализации перевёрнутых границ.
//! Выпуск устанавливает общий prepared-флаг после эффекта 1
//! (варианты 1/2/3: 0x0059E348 / 0x00562638 / 0x00560EE8). Последующий AI продолжает тот же
//! экземпляр в фоне; повторный Begin и отдельное хранилище не создаются.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::ghostcut2::GHOST_CUT_2_SKILL_ID;
use super::ghostcut3::GHOST_CUT_3_SKILL_ID;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const GHOST_CUT_SKILL_ID: u32 = 0x66;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GhostCutExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    path: Vec<(i32, i32, u8)>,
    current_position: usize,
    attacked: Vec<ShapeIdentity>,
}

impl GhostCutExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), condition_checked: false, path: Vec::new(), current_position: 0, attacked: Vec::new() }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
fn family_index(skill_id: u32) -> Option<usize> { match skill_id { GHOST_CUT_SKILL_ID => Some(0), GHOST_CUT_2_SKILL_ID => Some(1), GHOST_CUT_3_SKILL_ID => Some(2), _ => None } }
pub(crate) fn is_ghost_cut_dispatch(dispatch: PlayerSkillDispatch) -> bool { family_index(skill_id(dispatch)).is_some() }

fn finish_player_ghost_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, skill_id: u32, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(skill_id, now_ms);
    });
}

pub(crate) fn cancel_player_ghost_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, execution_skill_id: u32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<GhostCutExecutionState>(execution_skill_id).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_ghost_cut(game, player_id, player_ai, skill_id(dispatch), runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1) }
fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0287"), _ => {} }
}

fn send_visual(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, action: u8, endpoint: Option<(i32, i32, u32)>) {
    // В исходном visual-effect значение `3` только завершает нелуповый effect;
    // отдельного wire-пакета для него нет.
    if action == 3 { return }
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action); message.add_long(skill_id as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if action == 1 { message.add_long(player.shape().get_direction()); }
    else if action == 2 { let (x, y, total) = endpoint.unwrap_or_default(); message.add_long(0); message.add_long(0); message.add_long(x); message.add_long(y); message.add_ulong(total); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
    }
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}
fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }),
        _ => None,
    }
}

fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, skill_id: u32, level: i32, hit_modifier: i32, target_damage_factor: u32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player);
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor);
    let minimum = combat.minimum_attack as i32;
    let width = (combat.maximum_attack as i32).wrapping_sub(minimum).wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor = (f64::from(target_damage_factor)
        * f64::from(weapon_factor)
        * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } }
    Some((master, attack))
}

fn cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (area_width, area_height) = game.area_dimensions(); let mut shapes = Vec::new();
    if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { return Vec::new() }
    let mut identities = Vec::new(); for shape in shapes { if !identities.contains(&shape.identity) { identities.push(shape.identity); } } identities
}

fn attack_cell<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, skill_id: u32, level: i32, hit_modifier: i32, target_damage_factor: u32, x: i32, y: i32, attacked: &mut Vec<ShapeIdentity>, runtime: &mut Runtime) {
    let Some(master) = game.find_player(player_id).map(master_info) else { return };
    if skill_id == GHOST_CUT_SKILL_ID {
        let war_souls = game.find_region(region_id).map(|owner| owner.base().war_souls_at(x, y)).unwrap_or_default();
        for (&target_id, _) in &war_souls {
            let target = ShapeIdentity { object_type: PLAYER_TYPE, id: target_id as i32, ex_id: Default::default() };
            if target.id == player_id || !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
            let Some(level_of_target) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, level_of_target, skill_id, level, hit_modifier, target_damage_factor) else { continue };
            game.apply_owned_skill_attack_to_war_soul(master, target.id, region_id, attack, runtime);
        }
    }
    for target in cell_targets(game, region_id, x, y) {
        if (target.id == player_id && target.object_type == PLAYER_TYPE) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || attacked.contains(&target) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
        attacked.push(target); let Some(level_of_target) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, level_of_target, skill_id, level, hit_modifier, target_damage_factor) else { continue };
        match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => continue }
    }
}

pub(crate) fn execute_player_ghost_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let requested_skill = skill_id(dispatch); if family_index(requested_skill).is_none() { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(requested_skill), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(requested_skill, level) else { if player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).is_some() { finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE); let missile_step_ms = properties.query_property(MISSILE_FLYING_TIME); let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let target_damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR); let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(player_ai.skill_last_used_ms(requested_skill), reuse_delay_ms, now_ms) { send_failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_sword(game, player) { send_failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(requested_skill)); }
        player_ai.begin_player_skill_execution(GhostCutExecutionState::begin(dispatch, now_ms));
    } else if player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).is_some_and(|state| !state.condition_checked) {
        if game.find_player(player_id).is_none_or(|player| !weapon_is_sword(game, player)) { send_failure(game, player_id, 0x0e, mp_loss); finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some((target_x, target_y)) = target_position(game, region_id, player_id, dispatch) else { finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_visual(game, player_id, requested_skill, level, 1, None);
        if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).map(|state| state.kernel.started_at_ms()).unwrap_or_default();
    if !player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).is_some_and(|state| state.kernel().is_prepared()) {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
        let Some((target_x, target_y)) = target_position(game, region_id, player_id, dispatch) else { finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
        let path_limit = (maximum_distance != 0).then_some(maximum_distance);
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, path_limit);
        if maximum_distance != 0 && maximum_distance.wrapping_add(1) < path.len() as u32 { send_failure(game, player_id, 0x0b, mp_loss); finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let endpoint_index = path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len()); let endpoint = path.get(endpoint_index).or_else(|| path.last()).copied().unwrap_or((target_x, target_y, 2)); let total = missile_step_ms.wrapping_mul(endpoint_index as u32);
        send_visual(game, player_id, requested_skill, level, 2, Some((endpoint.0, endpoint.1, total)));
        if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { state.path = path; state.current_position = 1; state.kernel_mut().mark_prepared(); let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); }
    }
    let Some((current_position, cell)) = player_ai.player_skill_state::<GhostCutExecutionState>(dispatch.skill_id()).map(|state| (state.current_position, state.path.get(state.current_position).copied())) else { return terminal(QueuedSkillExecutionState::Rejected) };
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms.wrapping_add(missile_step_ms.wrapping_mul(current_position as u32))) { return terminal(QueuedSkillExecutionState::Pending) }
    let Some((x, y, _)) = cell else { send_visual(game, player_id, requested_skill, level, 3, None); if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_ghost_cut(game, player_id, player_ai, requested_skill, runtime); return terminal(QueuedSkillExecutionState::Completed) };
    let live_block = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(x, y));
    if live_block == 3 { let mut attacked = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()).map(|state| std::mem::take(&mut state.attacked)).unwrap_or_default(); attack_cell(game, player_id, region_id, requested_skill, level, hit_modifier, target_damage_factor, x, y, &mut attacked, runtime); if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { state.attacked = attacked; state.current_position = state.current_position.wrapping_add(1); } }
    else if live_block == 2 { send_visual(game, player_id, requested_skill, level, 3, None); if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { state.current_position = state.path.len().wrapping_add(1); } }
    else if let Some(state) = player_ai.player_skill_state_mut::<GhostCutExecutionState>(dispatch.skill_id()) { state.current_position = state.current_position.wrapping_add(1); }
    terminal(QueuedSkillExecutionState::Pending)
}
