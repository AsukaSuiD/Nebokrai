//! Паучий туман `CSpiderMist` (`0x198`) и его живая область
//! `CSpiderMistPhalanx`.
//!
//! Машинные quirks: поворот `SetDir` исполняется в выпуске AI, а не в
//! Begin-стадии (общая особенность семьи `summoncreatureskill`);
//! `SetMoveable(1)` стоит перед выпуском; `Summon` — единственный живой
//! JJ-вариант мира (tile X/Y точки через SetTileXY области); wire — кадры
//! `0x000BFE01` и входной снимок `0x000BF502` конверта `summonshape`.
//!
//! PARTIAL/UNKNOWN: состав аргументов ctor области сверен только формально;
//! конфиг-зависимость 5×5 таблиц области в этой сборке константна; имя слота
//! и owned-visual — INFERRED и не достраиваются догадкой; см. раздел по
//! ссылке ниже.
//!
//! Швы: hub-трейты семьи `summoncreatureskill`; адаптер области к `CShape`,
//! регистрация, публикация входного снимка и снятие `CSpiderPoisonState` —
//! швы прежнего владельца (statefactory Zone).
//!
//! Исходные владельцы PDB: `appserver/skills/spidermist.cpp`,
//! `appserver/skills/spidermistphalanx.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#spidermist--cspidermist-0x198-и-cspidermistphalanx

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::baseattackruntime::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::dispatch::PlayerSkillDispatch;
use super::execution::{PlayerSpiderMistExecutionState, SpiderMistProgress};
use super::lifecycle::{SkillStage, SkillTermination, skill_is_restored};
use super::statefactory::{SPIDER_POISON_SKILL_ID, SpiderPoisonState};
use super::summoncreatureskill::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_CONST, SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME,
    SummonSkillContact, SummonSkillGame, SummonSkillOutcome, SummonSkillPlayer,
    summon_face_direction, summon_visual_fire_message, summon_visual_start_message,
};

pub const SPIDER_MIST_SKILL_ID: u32 = 0x198;
pub const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const BLOCK_UNFLY: u8 = 2;
const SCOPE_SIDE: usize = 5;
const SCOPE: [[u8; SCOPE_SIDE]; SCOPE_SIDE] = [[1; SCOPE_SIDE]; SCOPE_SIDE];
const SPIDER_MIST_ENTRY_MESSAGE: i32 = 0x000b_f502;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpiderMistPhalanxTick { Scan, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpiderMistPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    state_lifetime_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    scope: [[u8; SCOPE_SIDE]; SCOPE_SIDE],
}

impl SpiderMistPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля соответствуют конструктору EXE")]
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, state_lifetime_ms: u32, frequency_ms: u32, hp_loss: u32,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, skill_level, state_lifetime_ms,
            frequency_ms, hp_loss, scope: SCOPE }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn skill_level(&self) -> i32 { self.skill_level }
    pub const fn state_lifetime_ms(&self) -> u32 { self.state_lifetime_ms }
    pub const fn frequency_ms(&self) -> u32 { self.frequency_ms }
    pub const fn hp_loss(&self) -> u32 { self.hp_loss }

    pub const fn tick(&self, now_ms: u32) -> SpiderMistPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            SpiderMistPhalanxTick::Expired
        } else {
            SpiderMistPhalanxTick::Scan
        }
    }

    pub fn active_cells(&self, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
        let start_x = center_x.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let start_y = center_y.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let mut cells = Vec::new();
        for x in 0..SCOPE_SIDE {
            for y in 0..SCOPE_SIDE {
                if self.scope[y][x] != 0 {
                    cells.push((start_x.wrapping_add(x as i32), start_y.wrapping_add(y as i32)));
                }
            }
        }
        cells
    }

    pub fn replace_affect_region(
        &mut self, center_x: i32, center_y: i32,
        incoming_tile_x: i32, incoming_tile_y: i32,
    ) {
        let half = (SCOPE_SIDE / 2) as i32;
        let existing_left = center_x.wrapping_sub(half);
        let existing_top = center_y.wrapping_sub(half);
        let incoming_left = incoming_tile_x.wrapping_sub(half);
        let incoming_top = incoming_tile_y.wrapping_sub(half);
        let existing_right = existing_left.wrapping_add(SCOPE_SIDE as i32);
        let existing_bottom = existing_top.wrapping_add(SCOPE_SIDE as i32);
        let incoming_right = incoming_left.wrapping_add(SCOPE_SIDE as i32);
        let incoming_bottom = incoming_top.wrapping_add(SCOPE_SIDE as i32);

        let overlap_left = existing_left.max(incoming_left);
        let overlap_top = existing_top.max(incoming_top);
        let overlap_right = existing_right.min(incoming_right);
        let overlap_bottom = existing_bottom.min(incoming_bottom);
        if overlap_left >= overlap_right || overlap_top >= overlap_bottom { return; }

        for world_y in overlap_top..overlap_bottom {
            for world_x in overlap_left..overlap_right {
                let incoming_x = world_x.wrapping_sub(incoming_left) as usize;
                let incoming_y = world_y.wrapping_sub(incoming_top) as usize;
                if SCOPE[incoming_y][incoming_x] != 0 {
                    let existing_x = world_x.wrapping_sub(existing_left) as usize;
                    let existing_y = world_y.wrapping_sub(existing_top) as usize;
                    self.scope[existing_y][existing_x] = 0;
                }
            }
        }
    }
}

/// Входной снимок области `0x000BF502`: identity с GUID, длина и payload
/// wire-конверта `summonshape` (`include_child = true`), хвост char 0.
pub fn spider_mist_entry_message(identity: ShapeIdentity, payload: &[u8]) -> CMessage {
    let mut message = CMessage::new(SPIDER_MIST_ENTRY_MESSAGE);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.base_mut().add_guid(identity.ex_id);
    message.add_long(payload.len() as i32);
    message.base_mut().add(payload);
    message.base_mut().add_char(0);
    message
}

/// Точечная/объектная форма цели player-cast паучьего тумана.
pub const fn is_player_spider_mist_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: SPIDER_MIST_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: SPIDER_MIST_SKILL_ID, .. }
    )
}

fn restore_player_movement<Game: SummonSkillGame>(game: &mut Game, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub fn cancel_player_spider_mist<Game: SummonSkillGame>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game
        .player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID)
        .map(|state| state.kernel().dispatch())
    else { return false };
    restore_player_movement(game, player_id);
    game.finish_summon_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub fn execute_player_spider_mist<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> SummonSkillOutcome
where
    Game: SummonSkillContact<Runtime>,
{
    if !is_player_spider_mist_dispatch(dispatch) {
        return SummonSkillOutcome::Rejected;
    }
    let Some((region_id, source_x, source_y, skill_level, master)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                game.summon_player_skill_level(player_id, SPIDER_MIST_SKILL_ID)?,
                player.master_info(),
            ))
        })
    else {
        return SummonSkillOutcome::Rejected;
    };
    let Some(properties) = game.skill_base_properties(SPIDER_MIST_SKILL_ID, skill_level).cloned()
    else {
        if game.player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID).is_some() {
            restore_player_movement(game, player_id);
        }
        return SummonSkillOutcome::Rejected;
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
    let state_lifetime_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = now_milliseconds();
    if game.player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID).is_none() {
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, SPIDER_MIST_SKILL_ID),
            reuse_delay_ms,
            now_ms,
        ) {
            game.send_summon_cast_failure(player_id, 0x0d);
            return SummonSkillOutcome::Rejected;
        }
        let Some(destination) = (match dispatch {
            PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
            PlayerSkillDispatch::Object { target, .. } => game
                .base_magic_target_view(region_id, target)
                .map(|view| (view.tile_x, view.tile_y)),
            PlayerSkillDispatch::SelfTarget { .. } => None,
        }) else {
            return SummonSkillOutcome::Rejected;
        };
        let path = game.summon_base_magic_path(
            region_id, source_x, source_y, destination.0, destination.1,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.send_summon_cast_failure(player_id, 0x0b);
            return SummonSkillOutcome::Rejected;
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            game.send_summon_cast_failure(player_id, 0x0f);
            return SummonSkillOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SPIDER_MIST_SKILL_ID));
        }
        game.begin_player_execution(
            player_id,
            PlayerSpiderMistExecutionState::begin(dispatch, destination, now_ms).into(),
        );
        return SummonSkillOutcome::Begun;
    }
    if game
        .player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return SummonSkillOutcome::Rejected;
    }
    let destination = game
        .player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID)
        .map(|state| state.destination)
        .expect("выполнение паучьего тумана хранит координаты призыва");
    if game
        .player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        // Поворота в Begin нет: визуал читает GetDir «как есть».
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(player) = game.find_player(player_id) {
            let direction = player.shape().get_direction();
            game.send_player_summon_visual(
                player_id,
                &summon_visual_start_message(
                    SPIDER_MIST_SKILL_ID, skill_level, PLAYER_TYPE, player_id, direction,
                ),
            );
        }
        if let Some(state) = game.player_spider_mist_state_mut(player_id, SPIDER_MIST_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = game
        .player_spider_mist_state(player_id, SPIDER_MIST_SKILL_ID)
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение паучьего тумана хранит время начала");
    if !skill_is_restored(started_at_ms, delay_ms, now_milliseconds()) {
        return SummonSkillOutcome::Pending;
    }
    // Машинный AI 0x5409B0: SetDir к клетке, затем SetMoveable(1) перед выпуском.
    if let Some(player) = game.find_player_mut(player_id) {
        player.cast_face_direction(summon_face_direction(
            source_x, source_y, destination.0, destination.1,
        ));
    }
    restore_player_movement(game, player_id);
    if game.find_player(player_id).is_some() {
        game.send_player_summon_visual(
            player_id,
            &summon_visual_fire_message(
                SPIDER_MIST_SKILL_ID, skill_level, PLAYER_TYPE, player_id,
                destination.0, destination.1,
            ),
        );
    }
    let phalanx_id = game.summon_allocate_shape_id();
    let phalanx_started_at_ms = now_milliseconds();
    let rule = SpiderMistPhalanx::new(
        master,
        phalanx_started_at_ms,
        lifetime_ms,
        skill_level,
        state_lifetime_ms,
        frequency_ms,
        hp_loss,
    );
    let summoned = game
        .with_summon_region(region_id, |game, region| {
            game.summon_add_spider_mist_phalanx(
                region,
                phalanx_id,
                &rule,
                destination.0,
                destination.1,
                phalanx_started_at_ms,
                runtime,
            )
            .is_some()
        })
        .unwrap_or(false);
    if let Some(state) = game.player_spider_mist_state_mut(player_id, SPIDER_MIST_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    restore_player_movement(game, player_id);
    game.summon_after_use_player_skill(player_id, SPIDER_MIST_SKILL_ID, runtime);
    if summoned {
        SummonSkillOutcome::Completed
    } else {
        SummonSkillOutcome::Rejected
    }
}

/// Объектный путь монстра/питомца паучьего тумана: подход к дистанции,
/// прямой путь и BLOCK_UNFLY, независимое attack-speed расписание, release
/// из живого каста (машинный AI 0x5409B0). Владелец региона сохранён
/// параметром: разрешение цели покрывает постройки и дочерние формы.
pub fn execute_owned_spider_mist<Game, Runtime>(
    game: &mut Game,
    owner: &mut Game::RegionOwner,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: SummonSkillContact<Runtime>,
{
    let region = Game::summon_region_base_mut(owner);
    let Some(facts) = game.summon_monster_facts(region, monster_id, SPIDER_MIST_SKILL_ID) else {
        return false;
    };
    let progress = game.summon_spider_mist_progress(region, monster_id);

    if let Some(cast) = facts.cast {
        if cast.skill_id != SPIDER_MIST_SKILL_ID {
            return false;
        }
        let Some(progress) = progress else {
            game.summon_monster_cancel_cast(region, monster_id);
            return true;
        };
        if !skill_is_restored(
            cast.started_at_ms,
            properties.query_property(SKILL_USAGE_DELAY_TIME),
            now_ms,
        ) {
            return true;
        }
        // Машинный AI 0x5409B0: SetDir направлением к клетке, затем
        // SetMoveable(1) перед выпуском.
        let source_x = facts.source.get_tile_x().unwrap_or_default();
        let source_y = facts.source.get_tile_y().unwrap_or_default();
        game.summon_monster_face_direction(
            region,
            monster_id,
            summon_face_direction(source_x, source_y, progress.destination_x, progress.destination_y),
        );
        game.summon_monster_set_moveable(region, monster_id, true);
        game.summon_monster_advance_cast(
            region, monster_id, SPIDER_MIST_SKILL_ID, SkillStage::Check, SkillStage::Calculate,
        );
        let message = summon_visual_fire_message(
            SPIDER_MIST_SKILL_ID,
            i32::from(skill_level),
            MONSTER_TYPE,
            monster_id,
            progress.destination_x,
            progress.destination_y,
        );
        game.send_summon_visual_around(region, &facts.source, &message);
        let phalanx_started_at_ms = now_milliseconds();
        let rule = SpiderMistPhalanx::new(
            MasterInfo {
                master_type: MONSTER_TYPE,
                master_id: monster_id,
                ..MasterInfo::default()
            },
            phalanx_started_at_ms,
            properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME),
            i32::from(skill_level),
            properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
            properties.query_property(SKILL_USAGE_CONST),
        );
        let phalanx_id = game.summon_allocate_shape_id();
        let _ = game.summon_add_spider_mist_phalanx(
            region,
            phalanx_id,
            &rule,
            progress.destination_x,
            progress.destination_y,
            phalanx_started_at_ms,
            runtime,
        );
        game.summon_monster_advance_cast(
            region, monster_id, SPIDER_MIST_SKILL_ID, SkillStage::Calculate, SkillStage::Attack,
        );
        game.summon_monster_advance_cast(
            region, monster_id, SPIDER_MIST_SKILL_ID, SkillStage::Attack, SkillStage::Apply,
        );
        game.summon_monster_finish_cast_clock(region, monster_id, SPIDER_MIST_SKILL_ID, runtime);
        return true;
    }

    let Some((target_shape, target_view)) = game.summon_monster_attack_target(&*owner, target)
    else {
        return true;
    };
    let region = Game::summon_region_base_mut(owner);
    let (Ok(source_x), Ok(source_y), Ok(destination_x), Ok(destination_y)) = (
        facts.source.get_tile_x(),
        facts.source.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else { return true };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if !game.summon_approach_attack_range(
        region, monster_id, target_view, maximum_distance, runtime,
    ) {
        return true;
    }
    let path = game.summon_straight_skill_path(
        region, source_x, source_y, destination_x, destination_y,
    );
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == BLOCK_UNFLY)
    {
        return true;
    }
    if let Some(attack_interval_ms) = game.summon_schedule_attack_interval(
        facts.ai_kind, facts.attack_interval_ms,
    )
        && !game.summon_monster_begin_attack_attempt(
            region, monster_id, now_ms, attack_interval_ms,
        )
    {
        return true;
    }
    if !skill_is_restored(
        facts.last_used_ms,
        properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        now_ms,
    ) {
        return true;
    }
    // Поворота в Begin нет: визуал ниже читает GetDir «как есть».
    let target_object = game.summon_skill_begin_object(region, target);
    game.summon_monster_set_moveable(region, monster_id, false);
    game.summon_monster_begin_cast(
        region, monster_id, target, SPIDER_MIST_SKILL_ID, skill_level, now_ms, target_object,
    );
    game.summon_set_spider_mist_progress(
        region,
        monster_id,
        SpiderMistProgress {
            destination_x,
            destination_y,
        },
    );
    let source = game.summon_monster_shape(region, monster_id);
    let source = source.as_ref().unwrap_or(&facts.source);
    let message = summon_visual_start_message(
        SPIDER_MIST_SKILL_ID,
        i32::from(skill_level),
        MONSTER_TYPE,
        monster_id,
        source.get_direction(),
    );
    game.send_summon_visual_around(region, source, &message);
    true
}

/// Проход живой области по активным клеткам: `GetShapes` каждой клетки,
/// отказ чтения прерывает обход, как `break` прежнего тела.
pub fn spider_mist_cell_targets<Game: SummonSkillGame>(
    game: &Game,
    region_id: i32,
    cells: &[(i32, i32)],
) -> Vec<ShapeIdentity> {
    let mut targets = Vec::new();
    for &(tile_x, tile_y) in cells {
        let Some(shapes) = game.summon_region_cell_identities(region_id, tile_x, tile_y)
        else { break };
        targets.extend(shapes);
    }
    targets
}

/// AI области `0x5EB110`: пропуски мастера, мёртвых, обладателей
/// 0x131/0x191, неатакуемых; создание CSpiderPoisonState из
/// `(info& master, +0xC0 state_lifetime, +0xC4 frequency, +0xC8 hp_loss)`;
/// Begin общего механизма — шов прежнего владельца.
pub fn apply_spider_mist_targets<Game: SummonSkillGame>(
    game: &mut Game,
    region_id: i32,
    rule: &SpiderMistPhalanx,
    candidates: &[ShapeIdentity],
    now: &mut dyn FnMut() -> u32,
) -> usize {
    let master = rule.master();
    let source = ShapeIdentity {
        object_type: master.master_type,
        id: master.master_id,
        ex_id: CGuid::GUID_INVALID,
    };
    if !game.summon_shape_present_in_region(region_id, source)
        || game.summon_shape_region_of(region_id, source).is_none()
    {
        return 0;
    }
    let mut applied = 0usize;
    for &candidate in candidates {
        if candidate.object_type == master.master_type && candidate.id == master.master_id {
            continue;
        }
        if game.summon_move_shape_health(region_id, candidate).is_none_or(|health| health == 0) {
            continue;
        }
        if game.summon_shape_has_state(region_id, candidate, 0x131) != Some(false)
            || game.summon_shape_has_state(region_id, candidate, SPIDER_POISON_SKILL_ID) != Some(false)
        {
            continue;
        }
        if !game.summon_skill_target_attackable(region_id, source, candidate) {
            continue;
        }
        if !game.summon_shape_present_in_region(region_id, source) {
            continue;
        }
        let Some(source_region) = game.summon_shape_region_of(region_id, source) else { continue; };
        let state = SpiderPoisonState::new(
            master,
            rule.state_lifetime_ms(),
            rule.frequency_ms(),
            rule.hp_loss(),
        );
        if game.summon_begin_spider_poison_state(
            region_id, candidate, (source_region, source), state, now,
        ) {
            applied = applied.wrapping_add(1);
        }
    }
    applied
}
