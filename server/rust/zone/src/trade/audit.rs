//! Audit-кадры семьи обмена и наземного перемещения исторического GameServer,
//! изолированные от transport-шва: сборка кадра в исходном порядке полей без
//! знания маршрута доставки. Исходный владелец `appserver/game.cpp` (пара
//! журналов обмена после commit и move-журнал drop/pickup/перевода банка);
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`. Флаги log-system,
//! форматирование текстов и отправка (`Send` на log server) остаются у прежнего
//! owner.
//!
//! Wire-константы `0x60201`/`0x60202` — семья MATCH; состав `0x6020D` машинно не
//! переоткрывался — Rust форма соответствует прежнему владельцу и держит честный
//! статус `UNKNOWN`. C-строковый примитив общий с `trade/session.rs`
//! ([`append_legacy_c_string`]).
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#торговля-и-деньги

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;

use super::session::append_legacy_c_string;

/// Скаляры party-записи журнала обмена `0x60201`: identity, базовые счётчики
/// и IPv4 участника на момент commit (pre-commit снапшот прежнего owner).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeAuditPartyFrame {
    pub owner_id: i32,
    pub pk_count: u32,
    pub money: u32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub client_ip: u32,
}

/// Скаляры actor-записи move-журнала `0x60202`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundMoveAuditActor {
    pub player_id: i32,
    pub pk_count: i16,
    pub money: u32,
    pub depot_money: u32,
    pub region_id: i32,
    pub tile_x: u32,
    pub tile_y: u32,
    pub client_ip: u32,
}

/// Кадр `0x60201` журнала предмета обмена в исходном порядке: запрос
/// receiver идёт первым, затем source, затем товар и оба IPv4. Для денежной
/// строки caller передаёт `CGuid::GUID_INVALID`, name пустым и price/amount
/// равными сумме.
pub fn trade_goods_audit_frame(
    source: &TradeAuditPartyFrame,
    receiver: &TradeAuditPartyFrame,
    goods_id: CGuid,
    price: u32,
    amount: u32,
    goods_name: &[u8],
) -> CMessage {
    let mut audit = CMessage::new(0x0006_0201);
    audit.add_byte(0);
    audit.add_long(receiver.owner_id);
    audit.add_ulong(receiver.pk_count);
    audit.add_ulong(receiver.money);
    audit.add_long(receiver.tile_x);
    audit.add_long(receiver.tile_y);
    audit.add_long(source.owner_id);
    audit.add_ulong(source.pk_count);
    audit.add_ulong(source.money);
    audit.add_long(source.tile_x);
    audit.add_long(source.tile_y);
    audit.base_mut().add_guid(goods_id);
    audit.add_ulong(price);
    audit.add_ulong(amount);
    append_legacy_c_string(audit.base_mut(), goods_name);
    audit.add_ulong(receiver.client_ip);
    audit.add_ulong(source.client_ip);
    audit
}

/// Кадр `0x6020D` журнала валюты обмена в исходном порядке: kind (2 — трата
/// payer, 3 — приход receiver), transaction id, amount, отформатированный
/// caller-ом текст, owner и баланс после commit.
pub fn trade_currency_audit_frame(
    kind: u8,
    transaction: &[u8],
    amount: u32,
    text: &[u8],
    owner_id: i32,
    balance: u32,
) -> CMessage {
    let mut audit = CMessage::new(0x0006_020d);
    audit.add_byte(kind);
    append_legacy_c_string(audit.base_mut(), transaction);
    audit.add_ulong(amount);
    append_legacy_c_string(audit.base_mut(), text);
    audit.add_long(owner_id);
    audit.add_ulong(balance);
    audit
}

/// Кадр `0x60202` move-журнала наземного/банковского перемещения в исходном
/// порядке: reason byte, actor, товар, price либо amount для валюты (правило
/// [`super::ground::ground_move_audit_price`] применяет caller), name, сумма,
/// region/tile/IP актёра.
pub fn ground_goods_move_log_frame(
    reason: u8,
    actor: &GroundMoveAuditActor,
    goods_id: CGuid,
    price: u32,
    name: &[u8],
    amount: u32,
) -> CMessage {
    let mut audit = CMessage::new(0x0006_0202);
    audit.add_byte(reason);
    audit.add_long(actor.player_id);
    audit.base_mut().add_short(actor.pk_count);
    audit.add_ulong(actor.money);
    audit.add_ulong(actor.depot_money);
    audit.base_mut().add_guid(goods_id);
    audit.add_ulong(price);
    append_legacy_c_string(audit.base_mut(), name);
    audit.add_ulong(amount);
    audit.add_long(actor.region_id);
    audit.add_ulong(actor.tile_x);
    audit.add_ulong(actor.tile_y);
    audit.add_ulong(actor.client_ip);
    audit
}
