//! Маска и параметры клетки CSoulMirror.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! GetScope VA 0x005A40D0, GetLength/GetHeight VA 0x005A4120/0x005A4150,
//! AI VA 0x005A4D10 (appserver/skills/soulmirror.cpp/.h).

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
