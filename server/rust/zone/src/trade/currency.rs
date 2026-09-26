//! Денежное ядро обмена исторического GameServer: таблицы маршрутов extend-id
//! bank/ground валюты, приёмка сумм перевода и правила слияния/установки
//! баланса.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `server/gameserver/appserver/game.cpp` (переводы wallet ↔ bank ↔ auction
//! wallet и commit-пересчёт денег обмена) и
//! `server/gameserver/appserver/player.cpp` (state-ответы `SetYuanBao`/
//! `SetAuctionMoney` и маршруты живых контейнеров валюты). Мгновенный
//! владелец `CPlayer`/контейнеры (`CWallet`, `CYuanBao`, `CBank`, auction
//! wallet) исполняет собственно изменение балансов и listener-эффекты; здесь
//! — таблицы маршрутов extend-id валюты, приёмка сумм перевода и правила
//! слияния/установки баланса.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинный статус — `MATCH` по подсемействам trade и bank/ground
//! currency дизассембла тел точной пары:
//!
//! | правило | машинный факт | здесь | статус |
//! |---|---|---|---|
//! | merge денег commit `0x1B9470` | 64-бит `sub eax,edx; sbb esi,0` с clamp к 0 и `add/adc` (НЕ i64-saturating); валидированное предусловие делает wrap/clamp недостижимым | [`merge_trade_money`] | семья `MATCH`, обоснование ниже |
//! | маршруты банка | `4→8`, `8→4`, `15→4`; позиции строго `0` | [`bank_currency_transfer_route_allowed`], [`bank_currency_transfer_positions_valid`] | семья `MATCH` |
//! | auction-return `15→4` | `CheckAuctionMoneyMove` ДО remove, повторно после remove → RollBack + `GPM015`/`GPM019` у caller-а | `check_auction_money_move` в [`super::auction`], порядок у прежнего owner | семья `MATCH` |
//! | установка баланса | `SetYuanBao`/`SetAuctionMoney`: increase на `current − previous`, decrease на `previous − current`, иначе без изменения | [`balance_set_direction`] | семья `MATCH` |
//! | ground-источник | wallet `4` / increment `5` | [`GroundCurrencyContainer`] | семья `MATCH` |
//! | bank-источник | wallet `4` / bank `8` / auction wallet `15` | [`BankCurrencyContainer`] | семья `MATCH` |
//!
//! Риск-нота merge: машинный пересчёт commit выполняет
//! `sub/sbb` с нижним clamp к 0 и `add/adc`, а НЕ i64-saturating арифметику.
//! Формула ниже исполняет тот же числовой результат wrapping-парами
//! `− outgoing + incoming` и выбором направления сравнением: при
//! предусловии валидации (`outgoing ≤ balance` и ёмкость противоположной
//! стороны проверена) отрицательный и переполняющий итоги недостижимы, clamp
//! и wrap вырождаются в одно значение. Поведение вне предусловия сознательно
//! не «улучшается».
//!
//! Живые контейнеры wire достаёт прежний owner, который вызывает правила
//! ниже в исходном порядке. Свидетельства `CBank` lock-gate (отдельно от
//! `CDepot`) и позиций `<0x60/0x30` остаются у контейнерных owner-ов
//! `items/`.

/// Player-контейнер наземной валюты drop/pickup: wallet `4` и
/// increment-контейнер `5` исходных маршрутов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroundCurrencyContainer {
    Wallet,
    YuanBao,
}

impl GroundCurrencyContainer {
    pub const fn from_extend_id(extend_id: i32) -> Option<Self> {
        match extend_id {
            4 => Some(Self::Wallet),
            5 => Some(Self::YuanBao),
            _ => None,
        }
    }

    pub const fn extend_id(self) -> i32 {
        match self {
            Self::Wallet => 4,
            Self::YuanBao => 5,
        }
    }
}

/// Player-контейнер перевода банковской валюты: wallet `4`, bank `8`,
/// auction wallet `15` исходных маршрутов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCurrencyContainer {
    Wallet,
    Bank,
    AuctionWallet,
}

impl BankCurrencyContainer {
    pub const fn from_extend_id(extend_id: i32) -> Option<Self> {
        match extend_id {
            4 => Some(Self::Wallet),
            8 => Some(Self::Bank),
            15 => Some(Self::AuctionWallet),
            _ => None,
        }
    }

    pub const fn extend_id(self) -> i32 {
        match self {
            Self::Wallet => 4,
            Self::Bank => 8,
            Self::AuctionWallet => 15,
        }
    }
}

/// Исходная таблица допустимых маршрутов перевода: wallet→bank, bank→wallet,
/// auction wallet→wallet. Обратный из auction wallet маршрут `4→15` отсутствует.
pub const fn bank_currency_transfer_route_allowed(
    source_extend_id: i32,
    destination_extend_id: i32,
) -> bool {
    matches!(
        (source_extend_id, destination_extend_id),
        (4, 8) | (8, 4) | (15, 4)
    )
}

/// Обе позиции перевода банковской валюты строго нулевые: однослотовые
/// currency-контейнеры не адресуют ячеек.
pub const fn bank_currency_transfer_positions_valid(
    source_position: u32,
    destination_position: u32,
) -> bool {
    source_position == 0 && destination_position == 0
}

/// Исходная приёмка суммы перевода: нулевая и превышающая источник сумма
/// отклоняется до detach.
pub const fn bank_currency_amount_valid(source_amount: u32, amount: u32) -> bool {
    amount != 0 && source_amount >= amount
}

/// Причина move-журнала перевода банковской валюты: `9` при поступлении в
/// bank, иначе `10` при снятии из bank.
pub const fn bank_transfer_audit_reason(destination_extend_id: i32) -> u8 {
    if destination_extend_id == 8 { 9 } else { 10 }
}

/// Исходный детектор валюты по base index: Gold coin либо YuanBao фабрики.
pub const fn ground_currency_index_matches(
    base_properties_index: u32,
    gold_coin_index: u32,
    yuan_bao_index: u32,
) -> bool {
    base_properties_index == gold_coin_index || base_properties_index == yuan_bao_index
}

/// Направление установки абсолютного баланса `SetYuanBao`/`SetAuctionMoney`:
/// новое значение сравнивается с прежним, delta беззнаковая wrapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BalanceSetDirection {
    Increase(u32),
    Decrease(u32),
    Unchanged,
}

/// Исходная развилка `SetYuanBao`/`SetAuctionMoney`: при равенстве нет ни
/// create/increase/decrease, ни delete-ответа.
pub const fn balance_set_direction(previous: u32, current: u32) -> BalanceSetDirection {
    if previous < current {
        BalanceSetDirection::Increase(current.wrapping_sub(previous))
    } else if current < previous {
        BalanceSetDirection::Decrease(previous.wrapping_sub(current))
    } else {
        BalanceSetDirection::Unchanged
    }
}

/// Результат слияния денег стороны в commit обмена: delta списания либо
/// начисления; равенство не трогает кошелёк.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeMoneyMerge {
    Decrease(u32),
    Increase(u32),
    Unchanged,
}

/// Merge денег commit `0x1B9470`: итог `current − outgoing + incoming`
/// (машинная 64-бит `sub/sbb` + `add/adc` пара, clamp к 0 внизу — см.
/// risk-note шапки) и направление строгим сравнением с текущим балансом.
pub const fn merge_trade_money(current: u32, outgoing: u32, incoming: u32) -> TradeMoneyMerge {
    let resulting = current.wrapping_sub(outgoing).wrapping_add(incoming);
    if resulting < current {
        TradeMoneyMerge::Decrease(current.wrapping_sub(resulting))
    } else if current < resulting {
        TradeMoneyMerge::Increase(resulting.wrapping_sub(current))
    } else {
        TradeMoneyMerge::Unchanged
    }
}
