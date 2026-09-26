//! Ярость синего босса `CBossBlueFury` (`0x1F7`): Check, AI, порядок
//! состояний и монстровый owned-вход. Источник: точная пара `gameserver.exe`
//! (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/bossbluefury.cpp/.h`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/bossbluefury.rs`; тела перенесены
//! кластером E3 полосы D/E (сверка — разведка `.local/recon-de/notes/
//! E4-bossbluefury.md`, тела `.local/recon-de/disasm/CBossBlueFury.txt`).
//! Данные и codec состояния — Zone `effects/bossbluefury.rs`, живые callbacks
//! состояния — соседний `skills/bossbluefurystate.rs`.
//!
//! Машинные якоря (VA = RVA + 0x400000): vtable класса `0x6578AC`,
//! `CBossBlueFuryEffect` vtable `0x657940`, UpdateVisualEffect `0x52E560`
//! (отдельный адрес — не ICF с `0x59FB90` RageBreak); Begin триадой
//! `0x52D4A0`/`0x52D2E0`/`0x52D3B0` (базовый Begin, `new` loop=1 visual,
//! CheckCast `vcall+0x64`, отказ — полный End(0), успех — `[+0x4C]=1`,
//! `[+0x50]=0`); End — общий ICF-хвост `0x546090` (`[+0x50]/[+0x4C]` в 0,
//! свежему U возвращается движение `0x4CCEE0(1)`, аргумент пробрасывается
//! базовому End; тот же хвост у Fury/BossBlueQuake).
//!
//! Check (`0x52E8E0`): `QuerySkillBaseProperties` обязательна (null → ret 0);
//! абсолютный срок reuse `0x2715` + `[+0x40]` против `timeGetTime` — отказ
//! visual(13) и GS0278 только игроку; дальше только RTTI-CPlayer: стоимость
//! RP `Query(3) == 0` — тихий ret 0 без visual; movzx RP `[U+0x288]` −
//! `Query(3)` signed < 0 → visual(8) + GS0289 с ценой → ret 0; успех —
//! SetMoveable(0). Монстр проходит без RP-блока и без SetMoveable(0).
//!
//! AI (`0x52EAC0`): `[+0x4C] == 0` — чистый возврат; таблица и U обязаны
//! (отсутствие → End(0)); `IsDied(U)` → visual(2) → End(1); первый проход
//! `[+0x50] == 0` списывает RP игрока до дефицита (необратимо; дефицит —
//! visual(8) + GS0289 + End(0)), затем `vcall+0x164` = OnChangeStates
//! (`publish_player_states`), CAN `0x2716` → `[+0x3C]`, visual(0), `[+0x50]=1`;
//! задержка `0x2711` — абсолютная `start + delay` unsigned, затем visual(1).
//! После неё порядок состояний: **продув каждого слота U с id `0x1F7`**
//! (`0x52EC10`–`0x52EC72`: End `vcall+0x1C` → deleting-dtor(1) свежего остатка
//! позиции → слот в 0, скан до конца вектора без досрочного выхода) →
//! `new(0x44)` `CBossBlueFuryState::ctor` `0x5E8A60`
//! (factor = `Query(20003)`, keep = `Query(10002)`, weak = `Query(10003)`;
//! вычисление запросов 10003 → 10002 → 20003 по push-порядку дампа) →
//! Begin(U,U) `vcall+8` → append в хвост | dtor; затем UpdateProperty
//! `vcall+0x9C` на U (call site `0x52EDAC`) и End(1). Атрибуция разведки
//! «`new(0x44)` CRageBreakState, gain = `Query(105)`» относится к соседнему
//! `CRageBreak::AI` (`0x5A0348`: new(0x44) → ctor `0x5FD1C0`, ключи
//! `Query(0x69)`/`Query(0x2712)`): у CBossBlueFury свой ctor и свои ключи.
//!
//! **FIX F3 (продув; основание — тело AI `0x52EAC0`):** прежняя реконструкция
//! завершала только первый типизированный ключ `applied_state_key::<
//! BossBlueFuryState>()`; машина завершает и уничтожает КАЖДЫЙ живой слот id
//! `0x1F7`, продолжая скан (позиция всегда +1, длина перечитывается, пропуски
//! не уплотняются). Узкая достижимость: игровой путь держит не более одной
//! записи `0x1F7` (каждый cast чистит), расхождение видимо при дублированных
//! записях БД id `0x1F7`, загруженных Restart-ом без продува. Класс:
//! statement-order/scope удаления; исправлено машинным продувом общим для
//! обеих ветвей (`sweep_boss_blue_fury_states`), `PublishStateCastGame`-шов
//! совпадает с конфликт-свипом семьи Fury.
//!
//! Wire `0x000BFE01`: кадры mode 0/1 — action 1/2 around с U (target =
//! сам U, тайлы живого источника); отказы 13/8/2 — только игроку BYTE-парой
//! `[byte 0][byte mode]` (форма отдельного тела `0x52E560`, не DWORD-префикс
//! RageBreak). После CheckCast-отказов Begin — терминальный `[0, 2]`; GS0278
//! и GS0289 — без/с суммой стоимости. Отдельное расхождение прежнего тела:
//! vcall+0x164 оно отображало на `update_player_current_state(MoveShapeAi)`;
//! тот же vcall CFury/CDaubPoison/CRageBreak hub-сверен как OnChangeStates —
//! перенос на общий hub (`prepare_rage_skill_effect`) приводит вызов к
//! `publish_player_states`.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! и RP-подготовка `skills/fury.rs` (check/фаза 0 AI, свип-примитив
//! `end_and_destroy_state_at`) реализованы у прежнего владельца; hub
//! `monsterattack` кластера A2 — подход/регион/cast-машина монстрового входа
//! (подпись делегата с `properties`/`now_ms` сохранена, часы — `fn()` от
//! делегата). MasterInfo конца (движение USER) и AfterUse/reuse — реестр
//! исполнения зарегистрированного навыка (`skillfactory`: State, USER,
//! Weapon). Потребление статическое (generic), dyn-совместимость не вводится
//! (ADR-0013).

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

/// FIX F3: машинный продув живого вектора — End + dtor КАЖДОГО слота id
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
/// внешними `properties`/`now_ms` сохранена (hub `monsterattack` кластера A2).
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
