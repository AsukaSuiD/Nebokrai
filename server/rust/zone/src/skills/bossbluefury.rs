//! Ярость синего босса `CBossBlueFury` (`0x1F7`): Check, AI, порядок
//! состояний и монстровый owned-вход; тела перенесены буквально.
//! Данные и codec состояния — Zone `effects/bossbluefury.rs`, живые callbacks
//! состояния — соседний `skills/bossbluefurystate.rs`.
//!
//! Машинный инвариант порядка состояний: полный продув КАЖДОГО живого слота
//! U с id `0x1F7` (скан до конца вектора, без досрочного выхода) → новый
//! state → Begin(U,U) → UpdateProperty → End(1). Отказы Check wire-кадром —
//! BYTE-парой `[0, mode]` только игроку (отдельное тело, не DWORD-префикс
//! RageBreak). RP-списание первого прохода AI необратимо.
//!
//! Hub-швы: `statecast::StateCastGame` и RP-подготовка `skills/fury.rs`;
//! монстр-вход — hub `monsterattack` старого пакета. Статическое потребление
//! (generic), dyn-совместимость не вводится (ADR-0013).
//!
//! Исходный владелец PDB: `appserver/skills/bossbluefury.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#bossbluefury-cbossbluefury-0x1f7

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::content::CSkillBaseProperties;
use crate::effects::BOSS_BLUE_FURY_STATE_ID;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::MONSTER_TYPE;

use super::baseattackruntime::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::bossbluefurystate::begin_primary_boss_blue_fury_state;
use super::fury::{
    RageRpPolicy, RageSkillEffect, check_rage_skill_cast, prepare_rage_skill_effect,
};
use super::lifecycle::{SkillStage, skill_is_restored};
use super::monsterattack::{MonsterCombatContact, resolve_owned_monster_attack_target};
use super::statecast::{
    RageCastVisualContract, StateCastExecutionOutcome, StateCastGame, StateCastMoveShape,
    StateCastVisualTarget, publish_rage_cast_visual,
};
use super::visualeffect::SkillVisualEffectKind;

pub const BOSS_BLUE_FURY_SKILL_ID: u32 = BOSS_BLUE_FURY_STATE_ID;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER: u32 = 10_003;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

static BOSS_BLUE_FURY_VISUAL: RageCastVisualContract = RageCastVisualContract {
    skill_id: BOSS_BLUE_FURY_SKILL_ID,
    kind: SkillVisualEffectKind::BossBlueFury,
    failures: &[2, 8, 13],
    dword_failures: &[],
    target: StateCastVisualTarget::User,
};

/// Visual навыка `0x000BFE01` `CBossBlueFuryEffect`: отказы 2/8/13 только
/// игроку BYTE-парой `[0, mode]`, modes 0/1 — around-кадр с самим U.
pub fn publish_boss_blue_fury_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    publish_rage_cast_visual(game, skill, mode, &BOSS_BLUE_FURY_VISUAL);
}

pub const fn is_boss_blue_fury_dispatch(dispatch: super::dispatch::PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == BOSS_BLUE_FURY_SKILL_ID
}

/// CheckCastCondition `0x52E8E0`: общая RP-проверка яростной семьи с запретом
/// нулевой стоимости (`RageRpPolicy::RequirePositive`, тихий ret 0).
pub fn check_boss_blue_fury_cast<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    Game: StateCastGame,
    Game::Player: super::fury::RageCastPlayer,
{
    check_rage_skill_cast(game, address, RageRpPolicy::RequirePositive, now)
}

/// AI `0x52EAC0`: подготовка `skills/fury.rs`, затем продув всех прежних
/// состояний `0x1F7`, новый `CBossBlueFuryState(U,U)`, UpdateProperty и End(1).
pub fn run_boss_blue_fury_ai<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> StateCastExecutionOutcome
where
    Game: StateCastGame,
    Game::Player: super::fury::RageCastPlayer,
{
    let effect = match prepare_rage_skill_effect(game, address, now) {
        std::ops::ControlFlow::Continue(effect) => effect,
        std::ops::ControlFlow::Break(outcome) => return outcome,
    };
    match apply_boss_blue_fury_effect(game, effect, now) {
        Some(_argument) => StateCastExecutionOutcome::EndCompleted,
        None => StateCastExecutionOutcome::Pending,
    }
}

/// Машинный продув живого вектора — End + dtor КАЖДОГО слота id
/// `0x1F7` (не только первого). Позиция читается заново после callback,
/// пропуски не уплотняются, новые хвостовые состояния участвуют в проходе.
pub fn sweep_boss_blue_fury_states<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
) {
    let mut position = 0;
    loop {
        let Some(shape) = game.resolve_state_move_shape(region_id, holder) else { return; };
        if position >= shape.state_slot_count() { break; }
        if shape.state_at(position).is_some_and(|(_, state)| state.state_id() == BOSS_BLUE_FURY_STATE_ID) {
            let _ = game.end_and_destroy_state_at(region_id, holder, position);
        }
        position += 1;
    }
}

/// Порядок состояний AI: продув → новый экземпляр в хвост → UpdateProperty,
/// аргумент End всегда 1 (прерванный Begin состояния не отменяет хвоста).
fn apply_boss_blue_fury_effect<Game: StateCastGame>(
    game: &mut Game,
    effect: RageSkillEffect,
    now: &mut dyn FnMut() -> u32,
) -> Option<i32> {
    let RageSkillEffect { source, properties } = effect;
    game.resolve_state_move_shape(source.0, source.1)?;
    sweep_boss_blue_fury_states(game, source.0, source.1);
    let _ = begin_primary_boss_blue_fury_state(
        game,
        source.0,
        source.1,
        Some(source),
        Some(source),
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER),
        now,
    );
    game.update_move_shape_properties(source.0, source.1);
    Some(1)
}

fn self_identity(monster_id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: monster_id,
        ex_id: CGuid::GUID_INVALID,
    }
}

/// Кадр начала `0x000BFE01` (visual mode 0): action 1, навык, уровень,
/// сторона 600 и direction источника.
fn boss_blue_fury_start_message(skill_level: u16, source_identity: ShapeIdentity, direction: i32) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(source_identity.object_type);
    message.add_long(source_identity.id);
    message.add_long(direction);
    message
}

/// Кадр исполнения `0x000BFE01` (visual mode 1): action 2, навык, уровень,
/// дважды сторона источника (target = сам U) и его живая клетка.
fn boss_blue_fury_fire_message(skill_level: u16, source_identity: ShapeIdentity, tile_x: i32, tile_y: i32) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(source_identity.object_type);
    message.add_long(source_identity.id);
    message.add_long(source_identity.object_type);
    message.add_long(source_identity.id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// Монстровый owned-вход `CBossBlueFury`: подход к дистанции, reuse, Begin
/// каста target=self и visual старта; после задержки — visual исполнения,
/// продув прежних `0x1F7` и установка нового состояния над опубликованным
/// регионом, UpdateProperty и живой End(1) с часами. Подпись делегата с
/// внешними `properties`/`now_ms` сохранена (hub `monsterattack`).
#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт исходного навыка")]
pub fn execute_owned_boss_blue_fury<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    mut now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterCombatContact<Runtime> + StateCastGame,
{
    let Some(region) = owner.as_mut().map(Game::owner_base_mut) else { return false; };
    let Some(facts) = game.monster_combat_facts(region, monster_id, BOSS_BLUE_FURY_SKILL_ID)
    else {
        return false;
    };
    let (source, cast, last_used_ms) = (facts.source, facts.cast, facts.last_used_ms);

    let initial_target = if cast.is_none() {
        owner
            .as_ref()
            .and_then(|region_owner| resolve_owned_monster_attack_target(game, region_owner, target_identity))
    } else {
        None
    };
    let Some(region_owner) = owner.as_mut() else { return false; };
    if cast.is_none() {
        let Some(target) = initial_target else {
            game.monster_clear_ai_target(Game::owner_base_mut(region_owner), monster_id);
            return true;
        };
        if !game.monster_combat_approach_attack_range(
            Game::owner_base_mut(region_owner),
            monster_id,
            target.view,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if !skill_is_restored(
            last_used_ms,
            properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            now_ms,
        ) {
            return true;
        }
        let region = Game::owner_base_mut(region_owner);
        let target_object = game.resolve_owned_skill_begin_object(&*region, self_identity(monster_id));
        game.monster_install_cast(
            region,
            monster_id,
            self_identity(monster_id),
            BOSS_BLUE_FURY_SKILL_ID,
            skill_level,
            now_ms,
            target_object,
        );
        let region = Game::owner_base_mut(region_owner);
        let message = boss_blue_fury_start_message(
            skill_level,
            source.identity(),
            source.get_direction(),
        );
        game.send_visual_around(&*region, &source, &message);
        return true;
    }

    let cast = cast.expect("выполнение ярости синего босса проверено выше");
    if cast.skill_id != BOSS_BLUE_FURY_SKILL_ID {
        return false;
    }
    if !skill_is_restored(
        cast.started_at_ms,
        properties.query_property(SKILL_USAGE_DELAY_TIME),
        now_ms,
    ) {
        return true;
    }

    // Кадр исполнения читает живую клетку; отказ чтения гасит только пакет,
    // продолжение каста не меняется.
    if let (Ok(tile_x), Ok(tile_y)) = (source.get_tile_x(), source.get_tile_y()) {
        let message = boss_blue_fury_fire_message(skill_level, source.identity(), tile_x, tile_y);
        game.send_visual_around(Game::owner_base(&*region_owner), &source, &message);
    }
    let region = Game::owner_base_mut(region_owner);
    game.monster_advance_cast(region, monster_id, BOSS_BLUE_FURY_SKILL_ID, SkillStage::Check, SkillStage::Calculate);
    game.monster_advance_cast(region, monster_id, BOSS_BLUE_FURY_SKILL_ID, SkillStage::Calculate, SkillStage::Attack);

    let region_id = Game::owner_region_id(region_owner);
    let _ = game.with_published_region(owner, |game| {
        sweep_boss_blue_fury_states(game, region_id, source.identity());
        let _ = begin_primary_boss_blue_fury_state(
            game,
            region_id,
            source.identity(),
            Some((region_id, source.identity())),
            Some((region_id, source.identity())),
            properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32,
            properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER),
            &mut now_milliseconds,
        );
        game.update_move_shape_properties(region_id, source.identity());
    });
    let Some(region_owner) = owner.as_mut() else { return true };
    let region = Game::owner_base_mut(region_owner);
    game.monster_advance_cast(region, monster_id, BOSS_BLUE_FURY_SKILL_ID, SkillStage::Attack, SkillStage::Apply);
    game.monster_finish_cast_clock(region, monster_id, BOSS_BLUE_FURY_SKILL_ID, runtime);
    true
}
