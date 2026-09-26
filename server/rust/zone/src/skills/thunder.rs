//! CThunder (0x21F, ID 32-бит) и общий путь семьи громовых облаков призыва:
//! Check/AI, которые CLeiming2 (0x21B) разделяет буквально, и Summon обеих
//! ветвей. CTianhuo использует только общий префикс допуска с собственными
//! правилами часов и препятствий (`skills/tianhuo.rs`).
//!
//! Машинные quirks: BF918 расхода MP рассылается кругом (thunder/leiming2)
//! или точечно (tianhuo) одним энкодером у своих швов; сериализация `0xBF502`
//! безусловна даже при отказе Add области; отказ самого Summon не меняет
//! завершающий End(1) — область живёт независимо от навыка.
//!
//! Швы: hub-трейт `SummonCloudGame` — фасад прежнего `CGame` (делегат
//! `appserver/skills/thunder.rs`); конструктор фаланги и регистрация области с
//! входным `0xBF502` — прежний владелец через callback `complete_summon`
//! (порядок SetCenter → Initialize → допуск региона U → Add → `0xBF502`).
//!
//! UNKNOWN: второй аргумент исходного `SendToAround` (exclude-player) —
//! делегат доставляет без исключений; RVA тела Summon CThunder — тело
//! перенесено буквально без новых утверждений.
//!
//! Исходные владельцы PDB: `appserver/skills/{thunder,thunder2}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#thunder--cthunder-0x21f-общие-checkai-семьи

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::{MasterInfo, truncate_original, truncate_original_i64_low};
use crate::content::CSkillBaseProperties;
use crate::content::goods::{GAP_BF_MP, GAP_BF_SPRITE};
use crate::regions::ShapeIdentity;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    check_battle_fairy_target_states, execute_registered_battle_fairy_state, summon_user_cch,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};

pub const THUNDER_SKILL_ID: u32 = 0x21f;
pub const THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub(crate) const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
pub(crate) const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

const SUMMON_CLOUD_GOODS_UPDATE_MESSAGE: i32 = 0x0b_f918;

/// Hub-фасады прежнего владельца `CGame` для BF-облаков призыва
/// (CThunder/CLeiming2/CTianhuo); открывают только прежние
/// обращения, имена сохраняют исходную операцию. Реализация остаётся у
/// делегата старого пакета `appserver/skills/thunder.rs`.
pub trait SummonCloudGame: BattleFairyGame {
    /// Around-доставка кадра `0xBF918` от формы игрока-U (якоря
    /// `0x121BAB`/`0x12080B`). Значение второго аргумента исходного
    /// `SendToAround` (exclude-player) — UNKNOWN; делегат не исключает никого.
    fn send_summon_cloud_goods_update_around(&mut self, player_id: i32, message: &CMessage);

    /// Поворот формы движения игрока-U (`SetDirection`, +0x60) — CTianhuo::AI.
    fn set_summon_cloud_user_direction(&mut self, player_id: i32, direction: i32);
}

/// Параметры призыва для делегата: конструктор `CThunderPhalanx` вызывается
/// фасадом прежнего владельца (тип — zone `skills/thunderphalanx`), как и
/// Initialize(RNG) и регистрация области.
pub struct ThunderSummon {
    pub source: (i32, ShapeIdentity),
    pub id: i32,
    pub master: MasterInfo,
    pub started_at_ms: u32,
    pub lifetime_ms: u32,
    pub skill_level: i32,
    pub frequency_ms: u32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
    pub target_count: u32,
    pub cch: i32,
    pub center_x: i32,
    pub center_y: i32,
}

pub fn scaled_battle_fairy_sprite(sprite: i32) -> i32 {
    truncate_original_i64_low(f64::from(sprite) * 0.0001)
}

pub(super) fn thunder_element_modifier(em_modifier: u32, scaled_sprite: i32) -> i32 {
    truncate_original(f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(scaled_sprite))
}

pub fn thunder_base_damage(target_damage_factor: u32, sprite: i32) -> i32 {
    truncate_original_i64_low(f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6)
}

/// Кадр `0xBF918` с круговой доставкой от U: форма кадра идентична точечному
/// `send_battle_fairy_goods_update` — id, GUID, len, blob
/// `SerializeForOldClient`; отличается только получатель.
fn send_summon_cloud_goods_update<Game: SummonCloudGame>(
    game: &mut Game, player_id: i32, ex_id: CGuid, old_client_payload: &[u8],
) {
    let mut message = CMessage::new(SUMMON_CLOUD_GOODS_UPDATE_MESSAGE);
    message.add_long(player_id);
    message.base_mut().add_guid(ex_id);
    message.add_ulong(old_client_payload.len() as u32);
    message.base_mut().add(old_client_payload);
    game.send_summon_cloud_goods_update_around(player_id, &message);
}

pub(crate) fn fail_battle_fairy_summon<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, mode: u32, string_id: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    game.send_skill_system_info(player_id, string_id);
}

/// Исходная S принадлежит аргументу Begin, а GetTargetPath повторно разрешает
/// текущую S. Tianhuo читает clock раньше reuse и не проверяет figure2.
pub(crate) fn check_battle_fairy_summon_prefix<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    reuse_clock_first: bool, check_obstacles: bool,
) -> Option<CSkillBaseProperties> {
    game.find_player(player_id)?;
    let target = begin_target?;
    if !check_battle_fairy_target_states(game, player_id, target) { return None; }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let (reuse, last_used, now_ms) = if reuse_clock_first {
        let now_ms = now(runtime);
        (properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME), skill.last_used_ms(), now_ms)
    } else {
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let last_used = skill.last_used_ms();
        (reuse, last_used, now(runtime))
    };
    if !skill_is_restored(last_used, reuse, now_ms) {
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

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost));
}

fn check_cast<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    let Some(properties) = check_battle_fairy_summon_prefix(
        game, instance, player_id, begin_target, runtime, now, false, true,
    ) else { return false; };
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) == 0 { return false; }
    let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP)
    else { return false; };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

#[allow(clippy::too_many_arguments, reason = "форма повторяет исходный общий путь семьи")]
pub(crate) fn execute_thunder_family<Game: SummonCloudGame, Runtime, Summon>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    summon: impl FnOnce(&mut Game, Game::SkillAddress, (i32, ShapeIdentity), (i32, i32), &mut Runtime) -> Option<Summon>,
    mut complete_summon: impl FnMut(&mut Game, &mut Runtime, Summon),
) -> BattleFairySkillOutcome {
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime, now),
        |game, instance, runtime| run_ai(game, instance, runtime, now, summon, &mut complete_summon),
    )
}

pub fn execute_battle_fairy_thunder<Game: SummonCloudGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    complete_summon: impl FnMut(&mut Game, &mut Runtime, ThunderSummon),
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != THUNDER_SKILL_ID { return BattleFairySkillOutcome::Rejected; }
    execute_thunder_family(game, player_id, instance, dispatch, begin_target, runtime, now,
        |game, instance, source, position, runtime| summon_thunder(game, instance, source, position, runtime, now),
        complete_summon)
}

fn run_ai<Game: SummonCloudGame, Runtime, Summon>(
    game: &mut Game, instance: Game::SkillAddress, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    summon: impl FnOnce(&mut Game, Game::SkillAddress, (i32, ShapeIdentity), (i32, i32), &mut Runtime) -> Option<Summon>,
    complete_summon: &mut dyn FnMut(&mut Game, &mut Runtime, Summon),
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(region, identity)
        .map(|shape| (region, shape.shape().identity()));
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    let position = if let Some(target) = target {
        if game.base_magic_target_dead(target.0, target.1) {
            game.update_registered_skill_visual(instance, 10);
            if let Some((_, user)) = source.filter(|(_, user)| user.object_type == 400) {
                game.send_skill_system_info(user.id, b"ZHGS0050");
            }
            return BattleFairySkillOutcome::Rejected;
        }
        let Some(shape) = game.resolve_state_move_shape(target.0, target.1) else {
            return BattleFairySkillOutcome::Rejected;
        };
        (shape.shape().get_tile_x().unwrap_or(i32::MIN), shape.shape().get_tile_y().unwrap_or(i32::MIN))
    } else { skill.lifecycle().destination() };
    let Some(source) = source else { return BattleFairySkillOutcome::Rejected; };
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if source.1.object_type == 400 {
            let Some(current) = game.battle_fairy_war_soul_addon(source.1.id, GAP_BF_MP)
            else { return BattleFairySkillOutcome::Pending; };
            let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
            let remaining = current.wrapping_sub(cost as i32);
            if remaining < 0 {
                fail_mana(game, instance, source.1.id, &properties);
                return BattleFairySkillOutcome::Rejected;
            }
            let Some(_stored) = game.set_battle_fairy_equipment_addon(source.1.id, GAP_BF_MP, remaining)
            else { return BattleFairySkillOutcome::Pending; };
            // Setter не заменяет equipment: сериализуется тот же предмет без
            // повторной проверки WarSoul и без подавления частичного payload.
            let Some((ex_id, old_client_payload)) = game.battle_fairy_equipment_payload(source.1.id)
            else { return BattleFairySkillOutcome::Pending; };
            send_summon_cloud_goods_update(game, source.1.id, ex_id, &old_client_payload);
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
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if now(runtime) < started.wrapping_add(delay) { return BattleFairySkillOutcome::Pending; }
    game.update_registered_skill_visual(instance, 1);
    if let Some(request) = summon(game, instance, source, position, runtime) {
        complete_summon(game, runtime, request);
    }
    BattleFairySkillOutcome::Completed
}

pub(crate) fn thunder_summon_properties<Game: BattleFairyGame>(
    game: &Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
) -> Option<(MasterInfo, CSkillBaseProperties, i32)> {
    let user = game.resolve_state_move_shape(source.0, source.1)?.shape().identity();
    let mut master = MasterInfo { master_type: user.object_type, master_id: user.id, ..MasterInfo::default() };
    let mut sprite = 0;
    if user.object_type == 400 {
        let player = game.find_player(user.id)?;
        if !game.battle_fairy_war_soul_goods_present(user.id) { return None; }
        let permissions = player.battle_fairy_pk_permissions();
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
        sprite = scaled_battle_fairy_sprite(game.battle_fairy_war_soul_addon(user.id, GAP_BF_SPRITE)?);
    }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let element = thunder_element_modifier(properties.query_property(SKILL_USAGE_EM_MODIFIER), sprite);
    Some((master, properties, element))
}

fn summon_thunder<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    position: (i32, i32), runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> Option<ThunderSummon> {
    let (master, properties, element) = thunder_summon_properties(game, instance, source)?;
    let cch = summon_user_cch(game, source.1);
    let target_count = properties.query_property(SKILL_USAGE_CONST);
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let frequency = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let skill_level = game.registered_skill(instance).map(|skill| skill.level())?;
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started_at_ms = now(runtime);
    let id = game.allocate_summon_shape_id();
    Some(ThunderSummon {
        source, id, master, started_at_ms, lifetime_ms, skill_level,
        frequency_ms: frequency, minimum_attack: minimum, maximum_attack: maximum,
        element_modifier: element, target_count, cch,
        center_x: position.0, center_y: position.1,
    })
}
