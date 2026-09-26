//! Маска, параметры клетки и живой обход области CSoulMirror (0x13C).
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
//! (точная пара, RSDS match). GetScope VA 0x005A40D0, GetLength/GetHeight VA
//! 0x005A4120/0x005A4150, AI VA 0x005A4D10, CalculateAttackPower `0x1A4A30`,
//! Attack `0x1A4BF0`; End(H) 13-fold `0x146090` — общий CStateSkill tail,
//! здесь не дублируется (порядок clear+End исполняет прежний kernel);
//! исходный владелец `appserver/skills/soulmirror.cpp/.h`.
//!
//! Общий ZonalCast хранит зарегистрированный Attack, его U/S, Check с
//! Player-only MP/Move0, unsigned срок start+delay и общий End. После visual1
//! этот owner захватывает текущие регион и центр U. Начало области остаётся
//! от этого момента, но перед каждой клеткой заново читаются level и
//! direction: GetScope задаёт фронтальную линию ширины `2 * level - 1` в
//! таблицах 3×3/5×5/7×7. Клетки идут X→Y и не собираются заранее, поэтому
//! синхронный контакт меняет следующий снимок. Любой разрешённый CMoveShape
//! делает клетку занятой; допуск, дедупликация и raw Attack относятся только
//! к подходящим целям. Пустая проходимая клетка создаёт CSummonedCreature с
//! fresh Master(country0) и параметрами Zone. Формулу и raw
//! контакт сохраняет directelementattack: weapon factor, Player-only EM и
//! единственный RNG без damage modifier, RP, CCH и второго RNG.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::{SelfCastGame,
//! SelfCastContact}` реализован у прежнего владельца; снимок фигур клетки —
//! шов `area_cell_views` (прежний `cell_views` семейства Flash), PK-допуск —
//! `live_skill_target_attackable_between`, элементный контакт —
//! `apply_direct_element_contact` (`directelementattack` прежнего пакета),
//! мастер источника — `soul_mirror_source_master` (`weaponattack` прежнего
//! пакета), lifecycle призванного существа свёрнут в шов
//! `add_soul_mirror_summoned_creature` (property по picture id → owner →
//! Add → возврат owner, в порядке прежнего тела).
//!
//! Общий Begin/Check/AI/End скелет зеркала и кадр его visual живут в
//! `skills/zonalcast.rs`; вызов этого обхода оттуда — шов
//! `ZonalCastContact::apply_soul_mirror_area`. Маска и scope области
//! остаются этому файлу (соседям не делегируются). По визуалам: hub
//! публикует только кадр `0xBFE01` скелета, а входные снимки порождённых
//! зеркалом существ — wire-конверт `skills/summonshape.rs`.

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;

use super::execution::ArrowTargetIdentity;
use super::selfcast::{SelfCastContact, SelfCastGame, SelfCastMoveShape};

pub const SOUL_MIRROR_SKILL_ID: u32 = 0x13c;
const SUMMONED_LIFETIME: u32 = 30_001;
const SUMMONED_CREATURE_ID: u32 = 30_003;

const SCOPE_DIRECTIONS: [((i32, i32), (i32, i32)); 8] = [
    ((0, -1), (1, 0)), ((1, -1), (1, 1)), ((1, 0), (0, 1)), ((1, 1), (-1, 1)),
    ((0, 1), (1, 0)), ((-1, 1), (1, 1)), ((-1, 0), (0, 1)), ((-1, -1), (-1, 1)),
];

pub fn soul_mirror_scope_size(level: i32) -> Option<i32> {
    match level {
        1 => Some(3),
        2 => Some(5),
        3 => Some(7),
        _ => None,
    }
}

/// Совпадает с байтовыми таблицами направлений 0x006A33D0/3420/34F0.
pub fn soul_mirror_scope_cell(level: i32, direction: i32, x: i32, y: i32) -> bool {
    let Some(size) = soul_mirror_scope_size(level) else { return false; };
    if !(0..size).contains(&x) || !(0..size).contains(&y) { return false; }
    let Some(&(forward, tangent)) = SCOPE_DIRECTIONS.get(direction as usize) else {
        return false;
    };
    let radius = level.wrapping_sub(1);
    (-radius..=radius).any(|offset| {
        x == level.wrapping_add(forward.0).wrapping_add(tangent.0.wrapping_mul(offset))
            && y == level.wrapping_add(forward.1).wrapping_add(tangent.1.wrapping_mul(offset))
    })
}

/// Обходит маску по столбцам и строкам, не сохраняя живые level/direction.
/// После выданной клетки следующий вызов заново читает состояние владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoulMirrorArea {
    origin_x: i32,
    origin_y: i32,
    column: i32,
    row: i32,
    new_column: bool,
    advance_row: bool,
}

impl SoulMirrorArea {
    pub fn new(center: (i32, i32), initial_level: i32) -> Option<Self> {
        let size = soul_mirror_scope_size(initial_level)?;
        Some(Self { origin_x: center.0.wrapping_sub(size >> 1),
            origin_y: center.1.wrapping_sub(size >> 1),
            column: 0, row: 0, new_column: true, advance_row: false })
    }

    pub fn next_cell(
        &mut self,
        mut current_level: impl FnMut() -> Option<i32>,
        mut current_direction: impl FnMut() -> Option<i32>,
    ) -> Option<(i32, i32)> {
        loop {
            if self.advance_row {
                self.row = self.row.wrapping_add(1);
                self.advance_row = false;
            }
            if self.new_column {
                let width = soul_mirror_scope_size(current_level()?)?;
                if self.column >= width { return None; }
                self.new_column = false;
            }
            let height = soul_mirror_scope_size(current_level()?)?;
            if self.row >= height {
                self.column = self.column.wrapping_add(1);
                self.row = 0;
                self.new_column = true;
                continue;
            }
            let level = current_level()?;
            let direction = current_direction()?;
            let column = self.column;
            let row = self.row;
            if soul_mirror_scope_cell(level, direction, column, row) {
                self.advance_row = true;
                return Some((self.origin_x.wrapping_add(column),
                    self.origin_y.wrapping_add(row)));
            }
            self.row = self.row.wrapping_add(1);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoulMirrorSummonParameters {
    pub lifetime_ms: u32,
    pub direction: i32,
    pub creature_picture_id: u32,
}

impl SoulMirrorSummonParameters {
    /// После проверки пустой проходимой клетки и взятия Master.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        read_direction: impl FnOnce() -> Option<i32>,
    ) -> Option<Self> {
        let lifetime_ms = query_property(SUMMONED_LIFETIME);
        let direction = read_direction()?;
        let creature_picture_id = query_property(SUMMONED_CREATURE_ID);
        Some(Self { lifetime_ms, direction, creature_picture_id })
    }
}

fn summon_empty_cell<Game: SelfCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    region_id: i32,
    x: i32,
    y: i32,
    properties: &CSkillBaseProperties,
) {
    if !game.region_cell_walkable(region_id, x, y) { return; }

    let Some(mut master) = game.soul_mirror_source_master(source) else { return; };
    master.master_country_id = 0;
    let Some(parameters) = SoulMirrorSummonParameters::read(
        |property| properties.query_property(property),
        || game.resolve_state_move_shape(source.0, source.1)
            .map(|source| source.shape().get_direction()),
    ) else { return; };
    game.add_soul_mirror_summoned_creature(
        region_id, parameters.creature_picture_id, master, x, y,
        parameters.direction, parameters.lifetime_ms,
    );
}

pub fn apply_soul_mirror_area<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
)
where
    Game: SelfCastContact<Runtime>,
{
    // visual1 уже мог вызвать произвольный код: здесь берутся live region и
    // центр U, но source для Attack остаётся результатом GetUser этого AI.
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
        return;
    };
    let user = user.shape();
    if !user.is_assigned_to_server_region() { return; }
    let region_id = user.get_region_id();
    let Some(initial_level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let center_x = user.get_tile_x().unwrap_or(i32::MIN);
    let center_y = user.get_tile_y().unwrap_or(i32::MIN);
    let Some(mut area) = SoulMirrorArea::new((center_x, center_y), initial_level) else { return; };
    let mut attacked = Vec::<ArrowTargetIdentity>::new();

    while let Some((cell_x, cell_y)) = area.next_cell(
        || game.registered_skill(instance).map(|skill| skill.level()),
        || game.resolve_state_move_shape(source.0, source.1)
            .map(|source| source.shape().get_direction()),
    ) {
        // Один resolver-снимок на клетку; следующий создаётся только
        // после всех callbacks текущей клетки.
        let mut occupied = false;
        for view in game.area_cell_views(region_id, cell_x, cell_y) {
            let Some(target) = game.resolve_state_move_shape(region_id, view.identity) else {
                continue;
            };
            occupied = true;
            let target = (target.shape().get_region_id(), target.shape().identity());
            if !game.live_skill_target_attackable_between(source, target) { continue; }
            let target_key = ArrowTargetIdentity::new(target.0, target.1);
            if attacked.contains(&target_key) { continue; }
            game.apply_direct_element_contact(instance, source, target, runtime);
            // Raw Attack может сам пропустить U==S; список всё равно
            // получает достигнутую цель только после этого вызова.
            attacked.push(target_key);
        }
        if !occupied {
            summon_empty_cell(game, source, region_id, cell_x, cell_y, properties);
        }
    }
}
