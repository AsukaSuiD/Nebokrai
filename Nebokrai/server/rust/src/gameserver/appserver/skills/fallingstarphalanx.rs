//! Область `CFallingStarPhalanx`, совместимая с сетевым ID метеорной стрелы `0xCD`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fallingstarphalanx.cpp`. PDB-глобалы всех трёх уровней
//! подтверждают маску `1×1`: каждая стрела выбирает центральную клетку, но
//! обязательно выполняет два вызова исходного RNG через `random(1)`. Дальнейшие
//! упорядоченные проходы, снимок атаки и двоичная сериализация совпадают с
//! `CMeteorArrowPhalanx`, поэтому отдельное параллельное исполнение не хранится.

use super::meteorarrowphalanx::CMeteorArrowPhalanx;
use crate::gameserver::appserver::masterinfo::MasterInfo;

#[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
pub(crate) fn create_falling_star_phalanx(
    id: i32,
    master: MasterInfo,
    started_at_ms: u32,
    frequency_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_attack: i32,
    soul_attack: i32,
    critical_chance: i32,
    hit_modifier: i32,
    arrow_count: u32,
    center_x: i32,
    center_y: i32,
    mut random_below: impl FnMut(i32) -> i32,
) -> CMeteorArrowPhalanx {
    let mut cells = Vec::with_capacity(arrow_count as usize);
    for _ in 0..arrow_count {
        let _ = random_below(1);
        let _ = random_below(1);
        cells.push((center_x, center_y));
    }
    CMeteorArrowPhalanx::from_cells(
        id,
        master,
        started_at_ms,
        frequency_ms,
        skill_level,
        minimum_attack,
        maximum_attack,
        element_attack,
        soul_attack,
        critical_chance,
        hit_modifier,
        cells,
    )
}
