//! Малая звезда `CLittleStar` (`0x1A4`) для игроков и монстров: числовые
//! правила, формулы, геометрия пути и wire-кадры visual `0x000BFE01` обеих
//! ветвей. Payload исполнений — `skills/execution/payload.rs`.
//!
//! Quirks: объектный Begin обнуляет fallback (+0x24/+0x28), поэтому до
//! построения пути он равен (0,0), а не позиции источника; один путь
//! предельной длины с публикацией конечной клетки, периодический обход до
//! первой `BLOCK_UNFLY`, один вызов legacy RNG на цель, x87-прибавка из целых
//! свойств с сохранённой f32-константой и усечением к нулю. Player и monster
//! ветви — абсолютный срок `CSkill::IsRestored`; тики и длительность —
//! elapsed.
//!
//! Швы: обход клеток, допуск целей 400/600 перед Calculate, применение атак,
//! публикация `CPlayerAI`, `finish_summon_skill`, зарегистрированный цикл и
//! доставка кадров — делегат старого пакета; player/monster `End` сохраняют
//! свой порядок у его владельца.
//!
//! Исходный владелец PDB: `appserver/skills/littlestar.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#снаряды-монстров-direct-path-littlestar-yunshenglightning

use crate::app::game_message::CMessage;
use crate::combat::truncate_original;

use super::PlayerSkillDispatch;

pub const LITTLE_STAR_SKILL_ID: u32 = 0x1a4;

pub const BLOCK_UNFLY: u8 = 2;
pub const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
pub const SKILL_USAGE_SKILL_PERSIST_TIME: u32 = 10_007;
pub const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;

/// Диспетчерская форма player-cast: только Point/Object этого навыка.
pub const fn is_player_little_star_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: LITTLE_STAR_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: LITTLE_STAR_SKILL_ID, .. })
}

/// Точка назначения player-cast: Point/Object именно этого навыка несут
/// клетку или живой вид цели (резолв — hub `base_magic_target_view`); прочие
/// формы назначения не имеют.
pub const fn player_target_position(
    dispatch: PlayerSkillDispatch,
    object_view: Option<(i32, i32)>,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: LITTLE_STAR_SKILL_ID, x, y } => Some((x, y)),
        PlayerSkillDispatch::Object { skill_id: LITTLE_STAR_SKILL_ID, .. } => object_view,
        _ => None,
    }
}

/// Кадр direction `0x000BFE01` (action 1 начала и action 3 завершения):
/// навык, уровень, сторона, direction.
pub fn little_star_action_message(
    action: u8,
    skill_level: i32,
    object_type: i32,
    object_id: i32,
    direction: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(object_type);
    message.add_long(object_id);
    message.add_long(direction);
    message
}

/// Кадр выпуска `0x000BFE01` (action 2): навык, уровень, сторона, нулевая
/// целевая пара, конечная клетка пути.
pub fn little_star_fire_message(
    skill_level: i32,
    object_type: i32,
    object_id: i32,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(object_type);
    message.add_long(object_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    message
}

/// Подготовка пути player-ветви после общего поиска: ведущая клетка источника
/// снимается, путь урезается пределом MAX.
pub fn prune_little_star_path(
    path: &mut Vec<(i32, i32, u8)>,
    source_x: i32,
    source_y: i32,
    maximum: u32,
) {
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
    path.truncate(maximum as usize);
}

/// Конечная клетка публикуемого пути; без пути остаётся живое назначение.
pub fn little_star_endpoint(path: &[(i32, i32, u8)], target_x: i32, target_y: i32) -> (i32, i32) {
    path.last().map(|cell| (cell.0, cell.1)).unwrap_or((target_x, target_y))
}

/// Строгая граница длительности: `start + delay + persist < now` (unsigned).
pub fn little_star_expired(started_ms: u32, delay_ms: u32, persist_ms: u32, now_ms: u32) -> bool {
    started_ms.wrapping_add(delay_ms).wrapping_add(persist_ms) < now_ms
}

/// Ширина elemental-диапазона player-ветви: `|max − min| + 1` со знаковым
/// abs (`wrapping_abs`).
pub fn little_star_span(minimum: i32, maximum: i32) -> i32 {
    maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1)
}

/// Ширина elemental-диапазона monster-ветви: беззнаковый abs разности
/// (`unsigned_abs` + 1), приведённая обратно со знаком.
pub fn little_star_monster_span(minimum: i32, maximum: i32) -> i32 {
    maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32
}

/// Стихийный урон игрока: прибавка в x87 до усечения к нулю; порядок
/// сложения — add_element, RNG, MIN, modifier.
pub fn little_star_player_element(
    add_element_attack: i32,
    minimum: i32,
    random_damage: i32,
    element_modifier: u32,
    element_modify: f32,
) -> i32 {
    let modifier = truncate_original(
        f64::from(element_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    add_element_attack
        .wrapping_add(random_damage)
        .wrapping_add(minimum)
        .wrapping_add(modifier)
        .max(0)
}

/// Стихийный урон монстра: MIN + один RNG, итог не опускается ниже нуля.
pub fn little_star_monster_element(minimum: i32, random: i32) -> i32 {
    minimum.wrapping_add(random).max(0)
}
