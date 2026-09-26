//! Скалярное ядро оркестрации обмена `CGame`, перенесённое в Zone `trade/`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/game.cpp`: двухфазная `CheckTradeCondition`,
//! YuanBao-вилка Billing и её запрос, таблица уведомлений отказа. Мгновенный
//! владелец `game.rs` исполняет регистрацию, packet-simulation ёмкостей,
//! detach/add последствия и весь transport; здесь — проверяемый порядок
//! скалярных условий, billing-решение и сам кадр запроса `0xEF203`.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинный статус — `MATCH` по подсемейству trade разведки «Zone
//! player: машинная разведка trade/auction/bank/ground currency» от 26
//! сентября 2026:
//!
//! | правило | машинный факт | здесь | статус |
//! |---|---|---|---|
//! | порядок `CheckTradeCondition` | contrary trader + owners → ёмкости/весы/деньги → IsSpaceEnough; 8 уведомлений | [`PlayerTradeConditionBlock`], [`trade_condition_notice`] | семья `MATCH` |
//! | ёмкость валют | `maximum < balance − outgoing + incoming` (wrapping DWORD) | [`trade_currency_capacity_exceeded`] | семья `MATCH` |
//! | вес | burden после замены `own → incoming` строго превышает max | [`trade_resulting_burden_exceeded`] | семья `MATCH` |
//! | Billing-вилка | signed разность YuanBao → payer/receiver/amount | [`trade_yuan_billing_decision`] | семья `MATCH` |
//! | запрос `0xEF203` | кадр type 2 + SendToBS (вызов `0x001BB036` → `0x00013BE0`) | [`TradeBillingRequest`], [`build_trade_billing_request_frame`] | семья `MATCH`, кадр по caller-у прежнего owner |
//!
//! Риск-нота Billing-complete (разведка порции T): завершение обмена по
//! ответу Billing (`OnUniBillMessage`) исполняет commit БЕЗ повторной
//! дистанционной проверки участников — оригинал доверяет уже зафиксированной
//! рамке. Гейты сверх исходного («улучшающие») в этот путь сознательно не
//! добавляются; существующий owner достаточно сохраняет свои прежние отказы.
//!
//! Швы переноса: скалярные предикаты и таблица уведомлений перенесены
//! буквально; simulation packet-а, веса `CGoods`, журналирование и отправка
//! остаются у прежнего owner, который вызывает правила ниже в исходном
//! порядке. Кадр `0xEF203` собирается здесь, а его транспортный выбор
//! (`SendToBS`, не World) — у caller-а вместе с комментарием-маршрутом.
//! C-строковый wire-примитив [`append_legacy_c_string`] — основное место
//! соглашения «prefix байт + NUL» семьи; его использует и соседний
//! `trade/audit.rs`.

use nebokrai_shared::network::CBaseMessage;
use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;

use super::auction::legacy_ipv4_text;

/// Порядок отказов `CheckTradeCondition`: каждому блоку соответствует одно
/// из восьми уведомлений обеим сторонам исходного обмена.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerTradeConditionBlock {
    SessionUnavailable,
    MissingPlayerOrPlug,
    MissingOfferedGoods,
    BurdenExceeded,
    PacketSpace,
    InsufficientGold,
    GoldCapacity,
    InsufficientYuanBao,
    YuanBaoCapacity,
}

/// Таблица string-id уведомлений `CheckTradeCondition` в порядке исходных
/// отказов (`GS0267..GS0274`).
pub const fn trade_condition_notice(block: PlayerTradeConditionBlock) -> &'static [u8] {
    match block {
        PlayerTradeConditionBlock::PacketSpace => b"GS0267",
        PlayerTradeConditionBlock::BurdenExceeded => b"GS0268",
        PlayerTradeConditionBlock::MissingOfferedGoods => b"GS0269",
        PlayerTradeConditionBlock::MissingPlayerOrPlug
        | PlayerTradeConditionBlock::SessionUnavailable => b"GS0270",
        PlayerTradeConditionBlock::InsufficientGold => b"GS0271",
        PlayerTradeConditionBlock::GoldCapacity => b"GS0272",
        PlayerTradeConditionBlock::InsufficientYuanBao => b"GS0273",
        PlayerTradeConditionBlock::YuanBaoCapacity => b"GS0274",
    }
}

/// Вес после замены собственных предложенных на входящие: итоговый burden
/// `current − own + incoming` (wrapping) сравнивается строгим превышением max
/// исходной проверки.
pub const fn trade_resulting_burden_exceeded(
    current_burden: u32,
    own_weight: u32,
    incoming_weight: u32,
    max_burden: u32,
) -> bool {
    max_burden < current_burden
        .wrapping_sub(own_weight)
        .wrapping_add(incoming_weight)
}

/// Исходная недостаточность баланса стороны для её собственного предложения.
pub const fn trade_currency_balance_insufficient(balance: u32, offered: u32) -> bool {
    balance < offered
}

/// Исходная проверка ёмкости результата `balance − outgoing + incoming`
/// (wrapping DWORD) против max stack соответствующей валюты.
pub const fn trade_currency_capacity_exceeded(
    balance: u32,
    outgoing: u32,
    incoming: u32,
    maximum: u32,
) -> bool {
    maximum < balance.wrapping_sub(outgoing).wrapping_add(incoming)
}

/// Выбор плательщика YuanBao-вилки исходного `CGame`: signed разность
/// предложенных YuanBao, отрицательная — зеркально.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeYuanBillingPayers {
    pub payer_index: usize,
    pub receiver_index: usize,
    pub amount: u32,
}

/// Billing-вилка по разности YuanBao: при нулевой разности обмен завершается
/// без Billing; иначе плательщик — сторона большего предложения.
pub fn trade_yuan_billing_decision(
    first_yuan_bao: u32,
    second_yuan_bao: u32,
) -> Option<TradeYuanBillingPayers> {
    let yuan_difference = i64::from(first_yuan_bao) - i64::from(second_yuan_bao);
    if yuan_difference == 0 {
        return None;
    }
    let (payer_index, receiver_index) = if yuan_difference > 0 { (0, 1) } else { (1, 0) };
    Some(TradeYuanBillingPayers {
        payer_index,
        receiver_index,
        amount: yuan_difference.unsigned_abs() as u32,
    })
}

/// C-строковый примитив wire-семьи обмена: prefix байт до первого NUL, затем
/// завершающий NUL. Поведение `Add(str)` исходного `CBaseMessage`.
pub(crate) fn append_legacy_c_string(message: &mut CBaseMessage, value: &[u8]) {
    let prefix_end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.add(&value[..prefix_end]);
    message.add_byte(0);
}

/// Скаляры кадра YuanBao-запроса обмена `0xEF203`: type `2`, оба account,
/// оба IPv4 dotted-quad текста, оба имени, amount, два литеральных `long 1`,
/// session, plug плательщика, login/world server ids и пустой GUID хвоста.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeBillingRequest<'a> {
    pub payer_owner_id: i32,
    pub receiver_owner_id: i32,
    pub payer_account: &'a [u8],
    pub receiver_account: &'a [u8],
    pub payer_ip: u32,
    pub receiver_ip: u32,
    pub payer_name: &'a [u8],
    pub receiver_name: &'a [u8],
    pub amount: u32,
    pub session_id: i32,
    pub payer_plug_id: i32,
    pub login_server_id: i32,
    pub world_server_id: i32,
}

/// Кадр `0xEF203` YuanBao-запроса обмена в исходном порядке полей. Маршрут
/// `SendToBS` исполняет caller (см. его комментарий о вызове `0x001BB036`).
pub fn build_trade_billing_request_frame(request: &TradeBillingRequest<'_>) -> CMessage {
    let mut message = CMessage::new(0x000e_f203);
    message.add_long(2);
    message.add_long(request.payer_owner_id);
    message.add_long(request.receiver_owner_id);
    append_legacy_c_string(message.base_mut(), request.payer_account);
    append_legacy_c_string(message.base_mut(), request.receiver_account);
    append_legacy_c_string(message.base_mut(), &legacy_ipv4_text(request.payer_ip));
    append_legacy_c_string(message.base_mut(), &legacy_ipv4_text(request.receiver_ip));
    append_legacy_c_string(message.base_mut(), request.payer_name);
    append_legacy_c_string(message.base_mut(), request.receiver_name);
    message.add_ulong(request.amount);
    message.add_long(1);
    message.add_long(1);
    message.add_long(request.session_id);
    message.add_long(request.payer_plug_id);
    message.add_long(request.login_server_id);
    message.add_long(request.world_server_id);
    message.base_mut().add_guid(CGuid::GUID_INVALID);
    message
}
