//! Летящий по клеткам навык `CPoisonMoth` (`0xCF`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonmoth.cpp`. Навык дважды проверяет оружие, причём
//! поздняя проверка следует после необратимого расхода MP. После задержки он
//! заново строит путь к живой цели, очищает ссылку на неё и обрабатывает не
//! более одной клетки за проход ИИ. Первый `BLOCK_SHAPE` завершает полёт после
//! упорядоченной атаки всех допустимых фигур клетки; end-пакет хранит последнюю
//! такую фигуру. Формула попадания выполняет ровно два собственных RNG-вызова,
//! защитные RNG остаются у `CGame`. Обе перегрузки `Attack` не изнашивают
//! оружие на отдельных целях: унаследованный `AfterUseSkill` делает это один
//! раз из `End(true)`, который прекращает оставшиеся клетки, обновляет свойства
//! и cooldown. `End(false)` не откатывает уже выполненные клеточные атаки, но
//! не допускает новых и не изнашивает оружие.
//! Damage factor сохраняет x87-порядок `u32 factor × f32 weapon × 0.01_f32`
//! до записи в `f32`. Критический множитель не округляет исходный урон в
//! `f32` и усекается к нулю лишь при итоговой записи каждого компонента.
//! Физический RNG получает исходную DWORD-ширину
//! `maximum - minimum + 1` без нормализации перевёрнутых границ. Cooldown
//! использует абсолютный срок `CSkill::IsRestored`; cast и полёт остаются elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const POISON_MOTH_SKILL_ID: u32 = 0xcf;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(super) const PLAYER_TYPE: i32 = 400;
pub(super) const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PoisonMothExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    attacking_started: bool,
    path: Vec<(i32, i32, u8)>,
    current_position: usize,
    destination: (i32, i32),
    end_tile: (i32, i32),
    visual_target: Option<ShapeIdentity>,
    end_sent: bool,
}

impl PoisonMothExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32, destination: (i32, i32)) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), condition_checked: false, attacking_started: false, path: Vec::new(), current_position: 0, destination, end_tile: (0, 0), visual_target: None, end_sent: false }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) fn is_poison_moth_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: POISON_MOTH_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: POISON_MOTH_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: POISON_MOTH_SKILL_ID, .. }) }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_poison_moth<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(POISON_MOTH_SKILL_ID, now_ms));
}
fn abort_player_poison_moth(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }
pub(crate) fn complete_player_poison_moth<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_poison_moth(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_poison_moth<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_poison_moth(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
pub(super) fn weapon_is_crossbow(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 4) }

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss), 10 => game.send_skill_system_info(player_id, b"GS0286"), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0293"), 0x0f => game.send_skill_system_info(player_id, b"GS0282"), _ => {} }
}
fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1); message.add_long(POISON_MOTH_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_fire(game: &mut CGame, player_id: i32, level: i32, destination: (i32, i32), flying_time: u32) {
    let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(POISON_MOTH_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(0); message.add_long(0); message.add_long(destination.0); message.add_long(destination.1); message.add_ulong(flying_time); let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_end(game: &mut CGame, player_id: i32, level: i32, end_tile: (i32, i32), target: Option<ShapeIdentity>) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(3); message.add_long(POISON_MOTH_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); message.add_long(end_tile.0); message.add_long(end_tile.1); message.add_long(target.map_or(0, |value| value.object_type)); message.add_long(target.map_or(0, |value| value.id)); let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(super) fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch { PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)), PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)) }
}
pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}
pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type { PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }), _ => None }
}
fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, level: i32, target_damage_factor: u32, hit_modifier: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player);
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor);
    let minimum = combat.minimum_attack as i32; let width = (combat.maximum_attack as i32).wrapping_sub(minimum).wrapping_add(1); let physical = minimum.wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor = (f64::from(target_damage_factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id: POISON_MOTH_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit_modifier.wrapping_neg(), damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.blast_attack) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } }
    Some((master, attack))
}
pub(super) fn cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (area_width, area_height) = game.area_dimensions(); let mut shapes = Vec::new(); if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { return Vec::new() } shapes.into_iter().map(|shape| shape.identity).collect()
}
fn attack_cell<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, level: i32, target_damage_factor: u32, hit_modifier: i32, x: i32, y: i32, runtime: &mut Runtime) -> Option<ShapeIdentity> {
    if x == 0 && y == 0 { return None } let master = game.find_player(player_id).map(master_info)?; let mut last_target = None;
    for target in cell_targets(game, region_id, x, y) { if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue } let Some(target_level) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, target_level, level, target_damage_factor, hit_modifier) else { continue }; match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => continue } last_target = Some(target); }
    last_target
}

pub(crate) fn execute_player_poison_moth<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_poison_moth_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(POISON_MOTH_SKILL_ID, game.skill_factory()), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(POISON_MOTH_SKILL_ID, level) else { if player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).is_none() { send_failure(game, player_id, 2, 0) } else { abort_player_poison_moth(game, player_id); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE); let missile_step_ms = properties.query_property(MISSILE_FLYING_TIME); let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let target_damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR); let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).is_none() {
        let now_ms = runtime.now_milliseconds();
        let reject = |game: &CGame, code| { send_failure(game, player_id, code, mp_loss); send_failure(game, player_id, 2, mp_loss); };
        let targets_self = match dispatch {
            PlayerSkillDispatch::SelfTarget { .. } => true,
            PlayerSkillDispatch::Object { target, .. } => target.object_type == PLAYER_TYPE && target.id == player_id,
            PlayerSkillDispatch::Point { .. } => false,
        };
        if targets_self { reject(game, 10); return terminal(QueuedSkillExecutionState::Rejected) }
        if !skill_is_restored(player_ai.skill_last_used_ms(POISON_MOTH_SKILL_ID), reuse_delay_ms, now_ms) { reject(game, 0x0d); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(destination) = target_position(game, region_id, player_id, dispatch) else { send_failure(game, player_id, 2, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) };
        let initial_path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None);
        if maximum_distance != 0 && initial_path.len() as u32 > maximum_distance { reject(game, 0x0b); return terminal(QueuedSkillExecutionState::Rejected) }
        if initial_path.iter().any(|cell| cell.2 == 2) { reject(game, 0x0f); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_crossbow(game, player) { reject(game, 0x0e); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { reject(game, 7); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(POISON_MOTH_SKILL_ID)); }
        player_ai.begin_player_skill_execution(PoisonMothExecutionState::begin(dispatch, now_ms, destination));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).is_some_and(|state| !state.condition_checked) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); abort_player_poison_moth(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) { send_failure(game, player_id, 0x0e, mp_loss); abort_player_poison_moth(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
        let destination = player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| state.destination).unwrap_or_default(); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } send_start(game, player_id, level); if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| state.kernel.started_at_ms()).unwrap_or_default();
    if !player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).is_some_and(|state| state.attacking_started) {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true) }
        let Some(destination) = target_position(game, region_id, player_id, dispatch) else { abort_player_poison_moth(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None);
        if maximum_distance != 0 && path.len() as u32 > maximum_distance.wrapping_add(1) { send_failure(game, player_id, 0x0b, mp_loss); abort_player_poison_moth(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
        let missile_index = path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len()); let endpoint = path.get(missile_index).or_else(|| path.last()).copied().unwrap_or((destination.0, destination.1, 2)); let flying_time = missile_step_ms.wrapping_mul(missile_index as u32); send_fire(game, player_id, level, (endpoint.0, endpoint.1), flying_time);
        if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.path = path; state.current_position = 0; state.destination = (endpoint.0, endpoint.1); state.attacking_started = true; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); }
    }
    let Some((position, path_len, cell, end_sent)) = player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| (state.current_position, state.path.len(), state.path.get(state.current_position).copied(), state.end_sent)) else { return terminal(QueuedSkillExecutionState::Rejected) };
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms.wrapping_add(missile_step_ms.wrapping_mul(position as u32))) { return terminal(QueuedSkillExecutionState::Pending) }
    let Some((x, y, _)) = cell else { if !end_sent { send_end(game, player_id, level, player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).map(|state| state.end_tile).unwrap_or_default(), player_ai.player_skill_state::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID).and_then(|state| state.visual_target)); } if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_poison_moth(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Completed) };
    let live_block = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(x, y)); if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.end_tile = (x, y) }
    if live_block == 3 { let target = attack_cell(game, player_id, region_id, level, target_damage_factor, hit_modifier, x, y, runtime); if target.is_some() { send_end(game, player_id, level, (x, y), target); if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.visual_target = target; state.end_sent = true; state.current_position = path_len.wrapping_add(1); } return terminal(QueuedSkillExecutionState::Pending) } }
    else if live_block == 2 { send_end(game, player_id, level, (x, y), None); if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.end_sent = true; state.current_position = path_len; } }
    if let Some(state) = player_ai.player_skill_state_mut::<PoisonMothExecutionState>(POISON_MOTH_SKILL_ID) { state.current_position = state.current_position.wrapping_add(1) }
    terminal(QueuedSkillExecutionState::Pending)
}
