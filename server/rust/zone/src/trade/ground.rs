//! Скалярные правила наземного перемещения предметов исторического GameServer:
//! приёмка сумм, particular-запрет, дистанция подбора, маршрут
//! currency-назначения и правило цены move-журнала. Исходный владелец
//! `appserver/game.cpp` (drop в регион и pickup из региона одной
//! container-message семьи); сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Мгновенный владелец `game.rs` исполняет detach/add по
//! маршрутам `(1 packet, 2 equipment, 3 hand, 4|5 currency, 9 depot,
//! 11 fairy)`, владельцев регионов, random-stream и весь transport.
//!
//! Обход team-share наземной валюты (доля по alive участникам с per-player
//! apply) живёт по соседнему владельцу drop-генерации и сюда не входит; файл
//! покрывает только маршруты drop/pickup одного игрока. Container mutations,
//! region owner и rollback-цепочки остаются у прежнего owner.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#торговля-и-деньги

pub const GROUND_DROP_FORBIDDEN_FLAG: u32 = 0x100;

/// Исходный запрет drop: particular attribute содержит флаг `0x100`.
pub const fn ground_drop_forbidden(particular_attribute: u32) -> bool {
    particular_attribute & GROUND_DROP_FORBIDDEN_FLAG != 0
}

/// Приёмка суммы drop до detach: ненулевая и не превышающая источник sumма,
/// а equipment снимается строго целиком (`==`).
pub const fn ground_drop_amount_valid(
    source_amount: u32,
    amount: u32,
    source_is_equipment: bool,
) -> bool {
    amount != 0 && source_amount >= amount && !(source_is_equipment && source_amount != amount)
}

/// Блок busy progress при частичном drop: валюта частично снимается даже в
/// open-stall/trading/upgrade progress, остальные предметы — нет.
pub const fn ground_partial_drop_busy_blocked(
    amounts_differ: bool,
    source_is_currency: bool,
    busy_progress: bool,
) -> bool {
    amounts_differ && !source_is_currency && busy_progress
}

/// Исходная дистанция pickup: отказ, если по любой оси расстояние клеток не
/// меньше двух.
pub const fn ground_pickup_out_of_range(
    ground_x: i32,
    ground_y: i32,
    player_x: i32,
    player_y: i32,
) -> bool {
    ground_x.abs_diff(player_x) >= 2 || ground_y.abs_diff(player_y) >= 2
}

/// Автоматическое назначение подобранной валюты: Gold coin — wallet `(4, 0)`,
/// YuanBao — increment `(5, 0)`; не-валюта назначение не меняет.
pub const fn ground_currency_pickup_destination(
    base_properties_index: u32,
    gold_coin_index: u32,
    yuan_bao_index: u32,
) -> Option<(i32, u32)> {
    if base_properties_index == gold_coin_index {
        Some((4, 0))
    } else if base_properties_index == yuan_bao_index {
        Some((5, 0))
    } else {
        None
    }
}

/// Цена move-журнала наземного перемещения: для валюты записывается сама
/// сумма, для остальных — цена предмета исходной записи.
pub const fn ground_move_audit_price(source_is_currency: bool, amount: u32, price: u32) -> u32 {
    if source_is_currency { amount } else { price }
}
