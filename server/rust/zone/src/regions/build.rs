//! Данные и скалярные правила постройки `CBuild` (factory type `0x44C`).
//! Исходный владелец — `appserver/build.h/.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`. Переходный агрегат `CBuild` остаётся в
//! старом пакете, хранит те же колонки и делегирует сюда их поведение без
//! изменения сигнатур; нематериальные accessor-ы полей и hub-публикация
//! `BuildClientPublication` (`game.rs`) остаются у него.
//!
//! Три тела разобраны по машинному коду, но не перенесены из-за недостижимости
//! в этой сборке: `GetAttackerDir`, одноаргументный `OnBeenAttacked` и `OnDied`
//! (у постройки нет `CBaseAI`, а победу country-war обрабатывает `CountryWarSys`
//! отдельно). Машинные основания — в evidence-документе.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

use super::shape::{ShapeCoordinateBlock, ShapeDecodeError, ShapeFigure};
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const BUILD_OBJECT_TYPE: u32 = 0x44C;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildInit {
    pub id: i32,
    pub graphics_id: i32,
    pub region_id: i32,
    pub name: Vec<u8>,
    pub direction: i32,
    pub max_hp: i32,
    pub defence: i32,
    pub width_increment: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub height_increment: i32,
    pub element_resistance: i32,
    pub script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildBlockUpdate {
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub width_increment: u8,
    pub height_increment: u8,
    pub block: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildClientUpdate {
    pub object_type: u32,
    pub object_id: u32,
    pub action: u16,
    pub max_hp: u32,
    pub hp: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildClientPublication {
    pub region_id: i32,
    pub build_id: i32,
    pub update: BuildClientUpdate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildDecodeError {
    Shape(ShapeDecodeError),
    Property(LegacyReadBlock),
    Coordinate(ShapeCoordinateBlock),
}

/// Непрерывный 0x18-byte `m_Property`: шесть little-endian DWORD в исходном
/// порядке (`hp`, `max_hp`, `defence`, `width`, `height`, `element_resistance`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildProperties {
    pub hp: u32,
    pub max_hp: u32,
    pub defence: u32,
    pub width_increment: i32,
    pub height_increment: i32,
    pub element_resistance: u32,
}

/// Direction-guard factory init: значение вне диапазона `0..8` заменяется
/// нулём до записи в общий `CShape`.
pub const fn init_direction(direction: i32) -> i32 {
    if direction >= 0 && direction < 8 {
        direction
    } else {
        0
    }
}

/// Позиция factory init: conversion `SetTileXY` и центр клетки `+0.5`.
pub fn init_center_coordinate(tile: i32) -> f32 {
    legacy_build_title_tile(tile) as f32 + 0.5f32
}

/// Guard применения script-файла factory init: пустое имя и маркер `"0"` не
/// перезаписывают пустой script конструктора.
pub fn init_script_uses_file(script: &[u8]) -> bool {
    !script.is_empty() && script != b"0"
}

/// Базовые навыки constructor `CBuild::CBuild` (`0x001DD570`): обычная атака
/// `1` и базовая защита владельца (`SKILL_BASE_DEFENSE`) уровня `1`,
/// регистрируются в этом порядке.
pub fn register_base_skills(
    base_defense_skill_id: u32,
    mut insert_new_skill: impl FnMut(u32, i32),
) {
    insert_new_skill(1, 1);
    insert_new_skill(base_defense_skill_id, 1);
}

/// Block-эффект action постройки: разрушение (`6`) освобождает footprint
/// (`0`), остальные действия ставят блок `3`.
pub const fn action_block(action: u16) -> u16 {
    if action == 6 { 0 } else { 3 }
}

/// Снимок текущего block-эффекта footprint; регион применяет его напрямую.
pub const fn block_snapshot(
    action: u16,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    width_increment: i32,
    height_increment: i32,
) -> BuildBlockUpdate {
    BuildBlockUpdate {
        region_id,
        tile_x,
        tile_y,
        width_increment: width_increment as u8,
        height_increment: height_increment as u8,
        block: action_block(action),
    }
}

/// Решение `CBuild::SetAction` (`0x001DD1C0`): неизменившийся action остаётся
/// no-op без записи change-state и обновления карты; при смене действия
/// применяется правило `action_block` нового действия.
pub const fn decide_set_action_update(
    current_action: u16,
    next_action: u16,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    width_increment: i32,
    height_increment: i32,
) -> Option<BuildBlockUpdate> {
    if current_action == next_action {
        return None;
    }
    Some(block_snapshot(
        next_action,
        region_id,
        tile_x,
        tile_y,
        width_increment,
        height_increment,
    ))
}

/// Exact `CBuild::IsAttackAble` (`0x001DD520`): локальные action/death guards
/// выполняются до результата virtual region-policy; concrete policy остаётся
/// у owning региона (`BuildIsAttackAble` country, `SymbolIsAttackAble`
/// остальных) и передаётся closure без erased указателей.
pub fn is_attackable_in_region(
    action: u16,
    hp: u32,
    region_allows: impl FnOnce() -> bool,
) -> bool {
    action != 6 && hp != 0 && region_allows()
}

/// Стадия `ApplyFinalDamage` меняет только HP; смертельные callbacks и packet
/// выполняются раньше отдельного virtual `SetAction(6)`.
pub const fn apply_combat_damage(hp: &mut u32, damage: u32) {
    *hp = hp.saturating_sub(damage);
}

/// Refresh постройки — `SetHP(GetMaxHP)` без побочных эффектов.
pub const fn refresh_hp(hp: &mut u32, max_hp: u32) {
    *hp = max_hp;
}

/// Exact `CBuild::GetFigure` (`0x001DD280`): directions `0/1` используют
/// младший byte `height_increment`, `2/3` — `width_increment`.
pub const fn figure_from_increments(width_increment: i32, height_increment: i32) -> ShapeFigure {
    ShapeFigure::from_directions([
        height_increment as u8,
        height_increment as u8,
        width_increment as u8,
        width_increment as u8,
    ])
}

/// Хвост exact `CBuild::AddToByteArray` (`0x001DD140`): после canonical
/// `CMoveShape` prefix дописывается 0x18-byte `m_Property` и текущий WORD action.
pub fn encode_client_property(payload: &mut Vec<u8>, properties: BuildProperties, action: u16) {
    let mut writer = LegacyWriter::new(payload);
    writer.write_u32(properties.hp);
    writer.write_u32(properties.max_hp);
    writer.write_u32(properties.defence);
    writer.write_i32(properties.width_increment);
    writer.write_i32(properties.height_increment);
    writer.write_u32(properties.element_resistance);
    writer.write_u16(action);
}

/// Хвост exact `CBuild::DecordFromByteArray` (`0x001DD180`): копия шести
/// little-endian DWORD `m_Property` с safe-Rust проверкой границ. Возвращает
/// прочитанные свойства и позицию после блока; запись в колонки и commit
/// cursor выполняет владелец после своей coordinate validation.
pub fn decode_client_property(
    source: &[u8],
    cursor: usize,
) -> Result<(BuildProperties, usize), LegacyReadBlock> {
    let mut reader = LegacyReader::at(source, cursor)?;
    let properties = BuildProperties {
        hp: reader.read_u32()?,
        max_hp: reader.read_u32()?,
        defence: reader.read_u32()?,
        width_increment: reader.read_i32()?,
        height_increment: reader.read_i32()?,
        element_resistance: reader.read_u32()?,
    };
    Ok((properties, reader.position()))
}

/// Conversion `CBuild::SetTileXY` в клетку: x87-округление к trunc.
pub fn legacy_build_title_tile(value: i32) -> i32 {
    // VERIFIED_DISASSEMBLY RVA 0x001DD2B0: `fild i32 -> fstp f32 ->
    // fld f32 -> fistp i32` под truncation control word. Округлённое вверх
    // `i32::MAX` становится x87 integer-indefinite `0x80000000`.
    let stored_float = value as f32;
    if stored_float >= 2_147_483_648.0_f32 {
        i32::MIN
    } else {
        stored_float as i32
    }
}
