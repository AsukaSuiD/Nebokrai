//! Скалярные правила наземного перемещения предметов GameServer,
//! перенесённые в Zone `trade/`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/game.cpp` (drop в регион и pickup из региона
//! одной container-message семьи). Мгновенный владелец `game.rs` исполняет
//! detach/add по маршрутам `(1 packet, 2 equipment, 3 hand, 4|5 currency,
//! 9 depot, 11 fairy)`, владельцев регионов, random-stream и весь transport;
//! здесь — приёмка сумм, particular-запрет, дистанция подбора, маршрут
//! currency-назначения и правило цены move-журнала.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинный статус — `MATCH` по подсемейству ground currency
//! разведки «Zone player: машинная разведка trade/auction/bank/ground
//! currency» от 26 сентября 2026:
//!
//! | правило | машинный факт | здесь | статус |
//! |---|---|---|---|
//! | drop-запрет | particular flag `0x100` отклоняет drop | [`ground_drop_forbidden`] | семья `MATCH` |
//! | сумма drop | `0 < amount ≤ source`, equipment — строго целиком | [`ground_drop_amount_valid`] | семья `MATCH` |
//! | частичный drop | busy progress блокирует частичный drop НЕ-валюты | [`ground_partial_drop_busy_blocked`] | семья `MATCH` |
//! | дистанция pickup | отказ при `|dx| ≥ 2` или `|dy| ≥ 2` | [`ground_pickup_out_of_range`] | семья `MATCH` |
//! | currency назначение | Gold → `(4, 0)`, YuanBao → `(5, 0)` | [`ground_currency_pickup_destination`] | семья `MATCH` |
//! | цена журнала | валюта пишет amount, остальные — price | [`ground_move_audit_price`] | семья `MATCH` |
//!
//! Обход team-share наземной валюты (доля по alive участникам с per-player
//! apply из разведки) живёт по соседнему владельцу drop-генерации и сюда не
//! входит; этот файл покрывает только маршруты drop/pickup одного игрока.
//!
//! Швы переноса: предикаты перенесены буквально (pub/пути нормализованы);
//! container mutations, region owner и rollback-цепочки остаются у прежнего
//! owner, который вызывает правила ниже в исходном порядке.

/// Particular-флаг запрета drop предмета на землю (`GAP 0x13`, бит `0x100`).
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
