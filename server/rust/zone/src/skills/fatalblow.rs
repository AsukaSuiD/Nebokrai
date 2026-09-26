//! CFatalBlow (0x21C): Check/AI и Summon снаряда боевого духа.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! `appserver/skills/fatalblow.cpp` (ctor `0x11E130`, Summon(shape,shape)
//! `0x51F640`: машинная цепочка summon-хелперов — MasterInfo → cast→CPlayer →
//! GetWarSoulGoods → пермишены → prop 156 sprite → CCH WORD → EM 20015 → new
//! → level WORD (+0x114) → 20010/20009/20008/6001/30001 → set center →
//! initialize → region → `0xBF502` входное сообщение фаланги; порядок
//! cch/region — машинно новый по CThunder, отличается от godthunder-аудитного).
//! Тела перенесены буквально.
//!
//! Общий вход сохраняет зарегистрированный экземпляр, исходные аргументы
//! объектного Begin и visual loop1. Check читает таблицу до проверки S;
//! отсутствие S даёт 10/ZHGS0045, собственная цель проверяется только в AI.
//! Первый конфликт состояния выбирается по позиции. MP0 и отсутствие WarSoul
//! дают тихий отказ; стоимость вычитается как signed wrapping DWORD.
//!
//! AI сохраняет U/S и таблицу, проверяет смерть S, затем равенство U/S.
//! Исчезнувший предмет оставляет ожидание; запись MP и BF918 (даже при false
//! Serialize — точечно, решение C шапки координатора) предшествуют CAN/visual0/condition
//! и абсолютной задержке. Поздний GetTargetPath разрешает свежую S, но
//! диагностика и Summon используют прежнюю цель. Flying-time — поле единственного
//! BF-payload, не копия в effect.
//!
//! Summon повторно проверяет регион U и таблицу, очищает S навыка, сохраняет
//! Master без country и читает параметры конструктора в исходном порядке.
//! Clock предшествует ID; центр берётся у прежней S после конструктора.
//! Отказ AddShape не подавляет сериализацию; после любой попытки Summon
//! координатор выполняет End(1). Попадание и lifetime принадлежат отдельному
//! снаряду (`skills/fatalblowphalanx.rs`): этот owner не подменяет его фоновой
//! AI синхронной атакой. End обнуляет flying-time до visual3; его единственный
//! общий хвост внешний.
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; фактическая регистрация снаряда и входное сообщение
//! `0xBF502` требуют прежнего main-loop runtime и выполняются делегатом через
//! callback `complete_summon` (`FatalBlowSummon`).

use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::content::goods::{GAP_BF_MP, GAP_BF_SPRITE};
use crate::regions::ShapeIdentity;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    check_battle_fairy_target_states, execute_registered_battle_fairy_skill,
    send_battle_fairy_goods_update, summon_user_cch, summon_user_region,
};
use super::dispatch::BattleFairySkillDispatch;
use super::execution::{BattleFairyExecution, FatalBlowExecutionState};
use super::fatalblowphalanx::{CFatalBlowPhalanx, FATAL_BLOW_SKILL_ID};
use super::lifecycle::{SkillStage, skill_is_restored};

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

/// Построенный снаряд для делегата: регистрация региона и входное сообщение
/// `0xBF502` выполняются прежним `CGame` с его main-loop runtime.
pub struct FatalBlowSummon {
    pub region_id: i32,
    pub phalanx: CFatalBlowPhalanx,
    pub started_at_ms: u32,
}

fn fail<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: Option<i32>, mode: u32, text: &[u8],
) -> BattleFairySkillOutcome {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id { game.send_skill_system_info(player_id, text); }
    BattleFairySkillOutcome::Rejected
}

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost));
}

fn fail_obstacle<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: Option<i32>,
    target: (i32, ShapeIdentity), text: &[u8],
) {
    game.update_registered_skill_visual(instance, 15);
    if let Some(player_id) = player_id
        && let Some(name) = game.base_magic_target_name(target.0, target.1).map(<[u8]>::to_vec)
    {
        game.send_skill_system_info_with_text(player_id, text, &name);
    }
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let Some(target) = begin_target else {
        let _ = fail(game, instance, Some(player_id), 10, b"ZHGS0045");
        return false;
    };
    if !check_battle_fairy_target_states(game, player_id, target) { return false; }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else { return false; };
    if !skill_is_restored(last_used, reuse, now(runtime)) {
        let _ = fail(game, instance, Some(player_id), 13, b"ZHGS0048");
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        let _ = fail(game, instance, Some(player_id), 11, b"ZHGS0049");
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        fail_obstacle(game, instance, Some(player_id), target, b"ZHGS0051");
        return false;
    }
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

pub fn execute_battle_fairy_fatal_blow<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    mut complete_summon: impl FnMut(&mut Game, &mut Runtime, FatalBlowSummon),
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != FATAL_BLOW_SKILL_ID { return BattleFairySkillOutcome::Rejected; }
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime, now),
        |dispatch, started| FatalBlowExecutionState::begin(dispatch, started).into(),
        |game, instance, runtime| run_ai(game, instance, runtime, now, &mut complete_summon),
    )
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    complete_summon: &mut dyn FnMut(&mut Game, &mut Runtime, FatalBlowSummon),
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return BattleFairySkillOutcome::Rejected; };
    let user = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(user.0, user.1)
        .map(|shape| (user.0, shape.shape().identity()));
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    let (Some(source), Some(target)) = (source, target) else {
        return BattleFairySkillOutcome::Rejected;
    };
    let player_id = (source.1.object_type == 400).then_some(source.1.id);
    if game.base_magic_target_dead(target.0, target.1) {
        return fail(game, instance, player_id, 10, b"ZHGS0050");
    }
    if game.resolve_state_move_shape(source.0, source.1)
        .zip(game.resolve_state_move_shape(target.0, target.1))
        .is_some_and(|(source, target)| std::ptr::eq(source, target))
    {
        return fail(game, instance, player_id, 10, b"ZHGS0045");
    }
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if let Some(player_id) = player_id {
            let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP)
            else { return BattleFairySkillOutcome::Pending; };
            let remaining = current.wrapping_sub(properties.query_property(SKILL_USAGE_USER_MP_LOSE) as i32);
            if remaining < 0 {
                fail_mana(game, instance, player_id, &properties);
                return BattleFairySkillOutcome::Rejected;
            }
            let Some(_stored) = game.set_battle_fairy_equipment_addon(player_id, GAP_BF_MP, remaining)
            else { return BattleFairySkillOutcome::Pending; };
            let Some((ex_id, payload)) = game.battle_fairy_equipment_payload(player_id)
            else { return BattleFairySkillOutcome::Pending; };
            send_battle_fairy_goods_update(game, player_id, ex_id, &payload);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    let started = skill.lifecycle().started_at_ms();
    if now(runtime) < started.wrapping_add(delay) { return BattleFairySkillOutcome::Pending; }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        return fail(game, instance, player_id, 11, b"ZHGS0049");
    }
    if path.iter().any(|cell| cell.2 == 2) {
        fail_obstacle(game, instance, player_id, target, b"ZHGS0053");
        return BattleFairySkillOutcome::Rejected;
    }
    let flight = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
    let Some(BattleFairyExecution::FatalBlow(state)) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.battle_fairy_execution_state_mut())
    else { return BattleFairySkillOutcome::Rejected; };
    state.set_missile_flying_time(flight);
    game.update_registered_skill_visual(instance, 1);
    if let Some(request) = summon(game, instance, source, target, runtime, now) {
        complete_summon(game, runtime, request);
    }
    BattleFairySkillOutcome::Completed
}

fn summon<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> Option<FatalBlowSummon> {
    let user = game.resolve_state_move_shape(source.0, source.1).map(|shape| shape.shape().identity())?;
    game.resolve_state_move_shape(target.0, target.1)?;
    let region = summon_user_region(game, source)?;
    let skill = game.registered_skill(instance)?;
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return None; };
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    let mut master = MasterInfo { master_type: user.object_type, master_id: user.id, ..MasterInfo::default() };
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
        let _ = game.battle_fairy_war_soul_addon(user.id, GAP_BF_SPRITE);
    }
    let _ = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let target_identity = game.resolve_state_move_shape(target.0, target.1)
        .map(|shape| shape.shape().identity())?;
    let cch = summon_user_cch(game, user);
    let factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32;
    let level = game.registered_skill(instance).map(|skill| skill.level())?;
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = now(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CFatalBlowPhalanx::new(id, master, started, lifetime, level, factor, cch, target_identity);
    let target_shape = game.resolve_state_move_shape(target.0, target.1)?;
    let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    phalanx.set_center(x, y);
    Some(FatalBlowSummon { region_id: region, phalanx, started_at_ms: started })
}
