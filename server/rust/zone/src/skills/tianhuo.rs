//! Небесный огонь CTianhuo (0x21A, ID 32-бит) — самостоятельная область
//! боевого духа с общим префиксом допуска громовых облаков
//! (`skills/thunder.rs`).
//!
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), `appserver/skills/tianhuo.cpp`. Адресная
//! конвенция факт-листа волны: истинный RVA (pub off + 0x1000; VA = RVA +
//! 0x400000). Прежний переходный владелец —
//! `src/gameserver/appserver/skills/tianhuo.rs`; тело перенесено буквально
//! порцией T1 «thunder/leiming2/tianhuo — BF-облака призыва».
//!
//! Машинный факт (MATCH по снятой доказательной базе):
//!
//! - Check: часы читаются до свойства reuse (`reuse_clock_first`), препятствий
//!   по клеткам нет, MP проверяется прямым vcall +0x5C по equipment[10]
//!   (не `GetWarSoulGoods`) даже при нулевой цене. CAN этим навыком не
//!   изменяется.
//! - AI `0x122DC0` (VA `0x522DC0`): MP списывается необратимо на первом AI;
//!   `0xBF918` доставляется точечно `SendToPlayer` (`0x12300B`, VA `0x52300B`)
//!   — решение C порции №6b здесь верно и сохраняется; затем поворот U
//!   (GetLineDir + SetDirection +0x60) и повторная проверка длины пути
//!   (visual 0xB + ZHGS0049). По абсолютному сроку сохранённая объектная S
//!   проверяется на смерть и превращается в точку до visual1; visual1 →
//!   Summon → безусловный внешний End(1).
//! - Summon `0x123280` (VA `0x523280`): region-RTTI → свежая таблица →
//!   очистка тройки [+0x18/1C/20] → Master → player-RTTI → мёртвые чтения
//!   GAP 0xC3 и usage 20015 (здесь `let _ =`) → ctor-стек (lifetime, level,
//!   min, max, elem) → SetCenter → GetShape(клетка) + RTTI + skill==0x21A →
//!   vcall [+0xa8] свёртка старой области → Add → `0xBF502`; свёртка старой
//!   области исполняется регистрационным швом прежнего владельца
//!   (`replace_tianhuo_phalanxes_in_cell` до add, здесь не переоткрывается).
//!   Отказ самого Summon не меняет завершающий End(1); область живёт
//!   независимо от навыка. Gameplay ID области — 0x21A, legacy ID режима
//!   применения эффекта — 0x13A; wire и lifetime области принадлежат
//!   CTianhuoPhalanx (ещё у старого владельца).
//!
//! Объявленные швы переноса (не расхождения): hub `thunder::SummonCloudGame`
//! (точечный кадр `0xBF918` собирает и доставляет готовый энкодер порции №6b
//! `send_battle_fairy_goods_update`); поворот U — шов
//! `set_summon_cloud_user_direction`; конструктор `CTianhuoPhalanx` и
//! регистрация области выполняются прежним владельцем через callback
//! `complete_summon` (`TianhuoSummon`). Часы `now` — шов делегата (прежний
//! main-loop runtime). UNKNOWN списком: ctor-стек 9 аргументов
//! CTianhuoPhalanx этой волной не открыт (перенос прежнего вызова буквально);
//! ветка AI с не-player U читает CPlayer после RTTI без проверки — прежняя
//! реконструкция возвращает отказ вместо разыменования (сохраняется).

use nebokrai_shared::runtime::get_line_direction;

use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::content::goods::{GAP_BF_MP, GAP_BF_SPRITE_BASE};
use crate::regions::ShapeIdentity;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    battle_fairy_master_info, execute_registered_battle_fairy_state, send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::SkillStage;
use super::thunder::{
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_SUMMONED_LIFETIME,
    SummonCloudGame, check_battle_fairy_summon_prefix, fail_battle_fairy_summon,
};

pub const TIANHUO_SKILL_ID: u32 = 0x21a;
pub const TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;

/// Параметры призыва для делегата: конструктор `CTianhuoPhalanx` ещё у
/// старого владельца, как и свёртка старой области и входное `0xBF502`.
pub struct TianhuoSummon {
    pub region_id: i32,
    pub id: i32,
    pub master: MasterInfo,
    pub started_at_ms: u32,
    pub lifetime_ms: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
    pub center_x: i32,
    pub center_y: i32,
}

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    let Some(properties) = check_battle_fairy_summon_prefix(
        game, instance, player_id, begin_target, runtime, now, true, false,
    ) else { return false; };
    let _ = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let Some(current) = game.battle_fairy_equipment_addon(player_id, GAP_BF_MP)
    else { return false; };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

pub fn execute_battle_fairy_tianhuo<Game: SummonCloudGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    mut complete_summon: impl FnMut(&mut Game, &mut Runtime, TianhuoSummon),
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != TIANHUO_SKILL_ID { return BattleFairySkillOutcome::Rejected; }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime, now),
        |game, instance, runtime| run_ai(game, instance, runtime, now, &mut complete_summon),
    )
}

fn run_ai<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    complete_summon: &mut dyn FnMut(&mut Game, &mut Runtime, TianhuoSummon),
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (source_region, source_identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(source_region, source_identity)
        .map(|shape| shape.shape().identity())
    else { return BattleFairySkillOutcome::Rejected; };

    if skill.execution_stage() == Some(SkillStage::Begin) {
        // В этой ветви оригинал разыменовывает CPlayer после RTTI без проверки.
        if source.object_type != 400 { return BattleFairySkillOutcome::Rejected; }
        let Some(current) = game.battle_fairy_equipment_addon(source.id, GAP_BF_MP)
        else { return BattleFairySkillOutcome::Rejected; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, source.id, &properties);
            return BattleFairySkillOutcome::Rejected;
        }
        let Some(_stored) = game.set_battle_fairy_equipment_addon(source.id, GAP_BF_MP, remaining)
        else { return BattleFairySkillOutcome::Rejected; };
        if let Some((ex_id, payload)) = game.battle_fairy_equipment_payload(source.id) {
            send_battle_fairy_goods_update(game, source.id, ex_id, &payload);
        }

        let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
        let destination = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, target)| game.resolve_state_move_shape(region, target))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ))
            .unwrap_or_else(|| skill.lifecycle().destination());
        let source_tiles = game.find_player(source.id).map(|player| (
            player.shape().get_tile_y().unwrap_or(i32::MIN),
            player.shape().get_tile_x().unwrap_or(i32::MIN),
        ));
        if let Some((source_y, source_x)) = source_tiles {
            game.set_summon_cloud_user_direction(
                source.id, get_line_direction(source_x, source_y, destination.0, destination.1),
            );
        }
        let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if path.len() > maximum as usize {
                fail_battle_fairy_summon(game, instance, source.id, 11, b"ZHGS0049");
                return BattleFairySkillOutcome::Rejected;
            }
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| {
        skill.execution_stage() != Some(SkillStage::Check)
    }) {
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    let started = skill.lifecycle().started_at_ms();
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    let (_, saved_target) = skill.lifecycle().sufferer();
    if saved_target.object_type != 0 && saved_target.id != 0 {
        let target = game.resolve_skill_sufferer(skill.lifecycle());
        let Some((region, target)) = target.filter(|(region, target)| {
            !game.base_magic_target_dead(*region, *target)
        }) else {
            game.update_registered_skill_visual(instance, 10);
            return BattleFairySkillOutcome::Rejected;
        };
        let Some(destination) = game.resolve_state_move_shape(region, target).map(|target| (
            target.shape().get_tile_x().unwrap_or(i32::MIN),
            target.shape().get_tile_y().unwrap_or(i32::MIN),
        )) else {
            game.update_registered_skill_visual(instance, 10);
            return BattleFairySkillOutcome::Rejected;
        };
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_point_target(destination);
        }
    }
    game.update_registered_skill_visual(instance, 1);
    if let Some(destination) = game.registered_skill(instance)
        .map(|skill| skill.lifecycle().destination())
    {
        if let Some(request) = summon(game, instance, (source_region, source), destination, runtime, now) {
            complete_summon(game, runtime, request);
        }
    }
    BattleFairySkillOutcome::Completed
}

fn summon<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> Option<TianhuoSummon> {
    let region_id = game.resolve_state_move_shape(source.0, source.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .map(|source| source.shape().get_region_id())
        .filter(|region| game.battle_fairy_region_exists(*region))?;
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level()).cloned()?;
    if let Some(skill) = game.registered_skill_mut(instance) {
        let point = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(point);
    }
    if source.1.object_type != 400 { return None; }
    let player = game.find_player(source.1.id)?;
    if !game.battle_fairy_equipment_present(source.1.id) { return None; }
    let master = battle_fairy_master_info(player);
    let _ = game.battle_fairy_equipment_addon(source.1.id, GAP_BF_SPRITE_BASE);
    let _ = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let element_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let skill_level = game.registered_skill(instance).map(|skill| skill.level())?;
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started_at_ms = now(runtime);
    let id = game.allocate_summon_shape_id();
    Some(TianhuoSummon {
        region_id, id, master, started_at_ms, lifetime_ms, skill_level,
        minimum_attack, maximum_attack, element_modifier,
        center_x: destination.0, center_y: destination.1,
    })
}
