//! Семейство малых рывков `CLittleFlash` (`0x71`) и `CLittleFlash2` (`0x7f`).
//! Успешный Begin возвращает Begun до первого AI. Расход ресурсов,
//! перемещение и атака остаются у AI после постановки Attack в том же Run;
//! раннее время Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/littleflash.cpp` и `littleflash2.cpp`. Первый навык
//! требует живую object-цель, второй допускает сохранённые координаты. Оба
//! подрезают прямой путь до первого непрерывного ряда занятых клеток,
//! публикует destination до relocation и поражает каждую уникальную цель на
//! пройденных клетках. Списание MP необратимо предшествует повторной проверке
//! оружия. Общая с `CFlash` damage-формула вызывается из этого owner-а;
//! различия второго навыка задаёт его собственный owner.
//! Общий `End` очищает execution-state, возвращает движение и завершает
//! `CAttackSkill::End(1)` с единичным оружейным `AfterUseSkill` и отдельной
//! cooldown-ячейкой варианта; `Attack` оружие по числу целей не изнашивает.
//! Ячейки проверяются абсолютным сроком `CSkill::IsRestored`; рывок остаётся elapsed.
//! End (0x0055B2B0, PDB layout) сбрасывает attacked/attacking и освобождает
//! path перед attacked-list. Derived condition/available оригинала не
//! подменяются стадией kernel или базовым available зарегистрированного навыка.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::flash::{
    calculate_dash_attack, cell_views, master_info, target_level, weapon_is_valid,
};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::littleflash2::{
    EMPTY_PATH_MESSAGE_ID as LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID, LITTLE_FLASH_2_SKILL_ID,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::citygate::CITY_GATE_OBJECT_TYPE;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LITTLE_FLASH_SKILL_ID: u32 = 0x71;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const PILLAR_SKILL_ID: u32 = 0x74;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LittleFlashVariant {
    Original,
    Second,
}

impl LittleFlashVariant {
    const fn from_dispatch(dispatch: PlayerSkillDispatch) -> Option<Self> {
        if matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: LITTLE_FLASH_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: LITTLE_FLASH_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: LITTLE_FLASH_SKILL_ID, .. })
        {
            Some(Self::Original)
        } else if super::littleflash2::is_dispatch(dispatch) {
            Some(Self::Second)
        } else {
            None
        }
    }

    const fn skill_id(self) -> u32 {
        match self {
            Self::Original => LITTLE_FLASH_SKILL_ID,
            Self::Second => LITTLE_FLASH_2_SKILL_ID,
        }
    }

    const fn empty_path_message_id(self) -> &'static [u8] {
        match self {
            Self::Original => b"GS0290",
            Self::Second => LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LittleFlashExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Vec<(i32, i32, u8)>,
    attacked_creatures: Vec<ShapeIdentity>,
    visual_destination: (i32, i32),
    attacking_started: bool,
    attacked: bool,
}

impl LittleFlashExecutionState {
    pub(crate) fn clear_end_paths(&mut self) {
        self.attacked = false;
        self.attacking_started = false;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }

    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            path: Vec::new(),
            attacked_creatures: Vec::new(),
            visual_destination: (0, 0),
            attacking_started: false,
            attacked: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn skill_id(&self) -> u32 {
        match self.kernel.dispatch() {
            PlayerSkillDispatch::SelfTarget { skill_id, .. }
            | PlayerSkillDispatch::Point { skill_id, .. }
            | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
        }
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_little_flash_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    LittleFlashVariant::from_dispatch(dispatch).is_some()
}

fn finish_player_little_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, skill_id, runtime);
}

pub(crate) fn cancel_player_little_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some((dispatch, skill_id)) = game.player_skill_state::<LittleFlashExecutionState>(player_id, execution_skill_id)
        .map(|state| (state.kernel().dispatch(), state.skill_id()))
    else {
        return false;
    };
    finish_player_little_flash(game, player_id, skill_id, player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32, string_id: Option<&[u8]>) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match string_id {
        Some(b"GS0288") => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        Some(string_id) => game.send_skill_system_info(player_id, string_id),
        None => {}
    }
}

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    action: u8,
    destination: (i32, i32),
) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 2 {
        message.add_long(0);
        message.add_long(0);
        message.add_long(destination.0);
        message.add_long(destination.1);
    } else {
        message.add_long(direction);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_position(
    game: &CGame,
    region_id: i32,
    variant: LittleFlashVariant,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32)> {
    if variant == LittleFlashVariant::Second {
        if let Some(destination) = super::littleflash2::point_destination(dispatch) {
            return Some(destination);
        }
    }
    let PlayerSkillDispatch::Object { target, .. } = dispatch else {
        return None;
    };
    game.base_magic_target_view(region_id, target)
        .map(|view| (view.tile_x, view.tile_y))
}

fn build_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
    maximum: u32,
    clear_single_blocked_cell: bool,
    runtime: &mut Runtime,
) -> Vec<(i32, i32, u8)> {
    let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
    while maximum < path.len() as u32 { path.pop(); }
    let Some(last) = path.last().copied() else { return path };
    if let Ok(next) = CShape::get_direction_position(
        get_line_direction(source_x, source_y, last.0, last.1),
        ShapeAreaCoordinates { x: last.0, y: last.1 },
    ) {
        let block = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(next.x, next.y));
        path.push((next.x, next.y, block));
    }

    let mut saw_shape_block = false;
    let mut trim_index = path.len();
    for (index, cell) in path.iter_mut().enumerate() {
        cell.2 = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(cell.0, cell.1)) & 7;
        let city_gate = cell_views(game, region_id, cell.0, cell.1)
            .first()
            .is_some_and(|shape| shape.identity.object_type == CITY_GATE_OBJECT_TYPE as i32);
        if city_gate || matches!(cell.2, 1 | 2) {
            trim_index = index.saturating_sub(1);
            break;
        }
        if !saw_shape_block {
            saw_shape_block = cell.2 == 3;
        } else if cell.2 != 3 {
            trim_index = index;
            break;
        }
    }
    if !saw_shape_block { return Vec::new() }
    if trim_index >= path.len() { trim_index = path.len().saturating_sub(1); }
    path.truncate(trim_index.saturating_add(1));
    let Some(anchor) = path.last().copied() else { return path };
    if anchor.2 != 0 {
        if clear_single_blocked_cell
            && super::littleflash2::clears_single_blocked_cell(path.len(), true)
        {
            path.clear();
            return path;
        }
        for direction in 0..8 {
            let Ok(candidate) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: anchor.0, y: anchor.1 }) else { continue };
            let open = game.find_region(region_id).is_some_and(|owner| {
                candidate.x >= 0 && candidate.y >= 0
                    && candidate.x < owner.base().region.width
                    && candidate.y < owner.base().region.height
                    && owner.base().skill_cell_block(candidate.x, candidate.y) & 7 == 0
            });
            if open { path.push((candidate.x, candidate.y, 0)); return path; }
        }
        if let Some(owner) = game.take_region_owner(region_id) {
            if let Ok(candidate) = owner.base().region.get_random_pos_in_range(
                anchor.0.wrapping_sub(2), anchor.1.wrapping_sub(2), 5, 5, runtime,
            ) {
                path.push((candidate.x, candidate.y, 0));
            }
            game.restore_region_owner(owner);
        }
    }
    path
}

fn attack_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    level: i32,
    skill_id: u32,
    hit_modifier: i32,
    damage_factor: u32,
    maximum: u32,
    path: &[(i32, i32, u8)],
    attacked: &mut Vec<ShapeIdentity>,
    runtime: &mut Runtime,
) {
    let Some(owner) = game.find_player(player_id).map(master_info) else { return };
    for &(x, y, _) in path.iter().take(path.len().saturating_sub(1)).take(maximum as usize) {
        for view in cell_views(game, region_id, x, y) {
            let target = view.identity;
            if (target.object_type == PLAYER_TYPE && target.id == player_id)
                || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
                || attacked.contains(&target)
                || !game.owned_player_skill_target_attackable(owner, target, region_id)
            { continue }
            attacked.push(target);
            let Some(target_level) = target_level(game, region_id, target) else { continue };
            let Some((master, attack)) = calculate_dash_attack(
                game, player_id, skill_id, target_level, level, hit_modifier, damage_factor,
            ) else { continue };
            match target.object_type {
                PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                _ => unreachable!("тип цели проверен перед расчётом"),
            }
        }
    }
}

pub(crate) fn execute_player_little_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(variant) = LittleFlashVariant::from_dispatch(dispatch) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let skill_id = variant.skill_id();
    let Some((region_id, source_x, source_y, level)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(skill_id, game.skill_factory()),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(skill_id, level) else {
        if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_some() {
            finish_player_little_flash(game, player_id, skill_id, ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let interval = properties.query_property(ACTION_INTERVAL);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, skill_id), reuse, now) {
            failure(game, player_id, 0x0d, mp_loss, Some(b"GS0278"));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if game.find_player(player_id).is_some_and(|player| player.has_state_by_skill_id(PILLAR_SKILL_ID)) {
            failure(game, player_id, 2, mp_loss, Some(b"GS0302"));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, LittleFlashExecutionState::begin(dispatch, now));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_none_or(|state| state.kernel.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_some_and(|state| state.kernel.stage() == SkillStage::Begin) {
        let Some((target_x, target_y)) = target_position(game, region_id, variant, dispatch) else {
            failure(game, player_id, 10, mp_loss, None);
            finish_player_little_flash(game, player_id, skill_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let path = build_path(
            game,
            region_id,
            source_x,
            source_y,
            target_x,
            target_y,
            maximum,
            variant == LittleFlashVariant::Second,
            runtime,
        );
        if path.is_empty() {
            failure(
                game,
                player_id,
                2,
                mp_loss,
                Some(variant.empty_path_message_id()),
            );
            finish_player_little_flash(game, player_id, skill_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            failure(game, player_id, 7, mp_loss, Some(b"GS0288"));
            finish_player_little_flash(game, player_id, skill_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
            failure(game, player_id, 0x0e, mp_loss, Some(b"GS0292"));
            finish_player_little_flash(game, player_id, skill_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(state) = game.player_skill_state_mut::<LittleFlashExecutionState>(player_id, dispatch.skill_id()) {
            state.path = path;
            state.visual_destination = (target_x, target_y);
            let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started = game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).map(|state| state.kernel.started_at_ms()).unwrap_or_default();
    if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_some_and(|state| !state.attacking_started) {
        if !time_reached(runtime.now_milliseconds(), started, delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        let (destination, final_cell) = game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).and_then(|state| Some((state.visual_destination, *state.path.last()?))).expect("непустой путь проверен");
        send_visual(game, player_id, skill_id, level, 2, destination);
        let _ = game.relocate_player_shape(player_id, region_id, final_cell.0, final_cell.1);
        if let Some(state) = game.player_skill_state_mut::<LittleFlashExecutionState>(player_id, dispatch.skill_id()) { state.attacking_started = true; }
    }

    if game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).is_some_and(|state| !state.attacked) {
        let path = game.player_skill_state::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).map(|state| state.path.clone()).unwrap_or_default();
        let mut attacked = game.player_skill_state_mut::<LittleFlashExecutionState>(player_id, dispatch.skill_id()).map(|state| std::mem::take(&mut state.attacked_creatures)).unwrap_or_default();
        attack_path(
            game,
            player_id,
            region_id,
            level,
            skill_id,
            hit_modifier,
            damage_factor,
            maximum,
            &path,
            &mut attacked,
            runtime,
        );
        if let Some(state) = game.player_skill_state_mut::<LittleFlashExecutionState>(player_id, dispatch.skill_id()) {
            state.attacked_creatures = attacked;
            state.attacked = true;
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(interval)) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    send_visual(game, player_id, skill_id, level, 3, (0, 0));
    if let Some(state) = game.player_skill_state_mut::<LittleFlashExecutionState>(player_id, dispatch.skill_id()) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_little_flash(game, player_id, skill_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
