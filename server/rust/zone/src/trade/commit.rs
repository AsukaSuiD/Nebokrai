//! Двухфазный commit/rollback обмена и packet-симуляция проверки условий
//! исторического `CGame`: снимки сторон рамки `CTrader`, порядок
//! detach/deliver/undo/restore над ними, денежное слияние через
//! [`merge_trade_money`] и очередь goods-audit кадров `0x60201`. Исходный
//! владелец `appserver/game.cpp` (commit/RollBack `0x1B9470` и packet-часть
//! `CheckTradeCondition`); сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Contract: docs/gameplay/trade.md.
//!
//! Живые шаги (remove/insert реестра игроков hub-а, контейнерные
//! эффекты, клиентские и log-server сообщения, equipment property/skill
//! проход) исполняет hub через узкий [`PlayerTradeHost`]; Zone-owner держит
//! наблюдаемый порядок фаз и отказов. Quirk-и прежних owner-ов сохранены:
//! rollback обращает только ПОЛНЫЙ merge рамки (release-форма оригинала,
//! Zone `trade/ctrader`), деньги сливаются машинной `sub/sbb` парой
//! [`merge_trade_money`] (`trade/currency`), Billing-завершение доверяет
//! рамке без повторной проверки (`trade/session`).
//! Осознанный quirk перехода формы: при несуществовании игрока стороны с
//! пустым списком goods между snapshot и самим commit прежний hub отказывал до
//! detach, драйвер узнаёт об этом в первом `detach` — ветка практически
//! недостижима (snapshot и commit идут одним синхронным проходом, оба игрока
//! online), а различие сводится к лишней паре клиентских attach/detach-кадров
//! второй стороне; гейт присутствия сознательно не добавлен.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#торговля-и-деньги

use std::collections::BTreeMap;

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::cgoods::CGoods;
use crate::items::cgoodsshadowcontainer::GoodsShadow;
use crate::items::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome,
};

use super::audit::{TradeAuditPartyFrame, trade_goods_audit_frame};
use super::ctrader::trade_offer_amount_satisfies;
use super::currency::{TradeMoneyMerge, merge_trade_money};
use super::session::{
    PlayerTradeConditionBlock, trade_currency_balance_insufficient,
    trade_currency_capacity_exceeded, trade_resulting_burden_exceeded,
};

/// Снимок одной стороны рамки обмена на момент commit: plug и owner живого
/// игрока, зафиксированные предложения и валютные суммы его `CTrader`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerTradePartySnapshot {
    pub plug_id: i32,
    pub owner_id: i32,
    pub goods: Vec<GoodsShadow>,
    pub gold: u32,
    pub yuan_bao: u32,
}

/// Pre-commit снимок audit-скаляров участника: журналы обмена пишут
/// значения ДО эффектов сделки (прежние pk/money/позицию/IP и имя).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerTradeAuditParty {
    pub owner_id: i32,
    pub pk_count: u32,
    pub money: u32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub client_ip: u32,
    pub name: Vec<u8>,
}

impl PlayerTradeAuditParty {
    /// Скаляры party-записи audit-кадра `0x60201` (`trade/audit`).
    pub fn frame(&self) -> TradeAuditPartyFrame {
        TradeAuditPartyFrame {
            owner_id: self.owner_id,
            pk_count: self.pk_count,
            money: self.money,
            tile_x: self.tile_x,
            tile_y: self.tile_y,
            client_ip: self.client_ip,
        }
    }
}

/// Одна принятая packet-ом получателя добавка обмена; `token` непрозрачен
/// для Zone (hub `CiQingPacketAddition`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerTradeAddition<Delivery> {
    /// Итоговое количество позиции; `None` — добавка не материализовалась.
    pub resulting_amount: Option<u32>,
    pub token: Delivery,
}

/// Результат packet-доставки части обмена: добавки в исходном порядке
/// следования goods и непринятые позиции.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerTradeDelivery<Delivery> {
    pub additions: Vec<PlayerTradeAddition<Delivery>>,
    pub rejected: Vec<CGoods>,
}

/// Исход двухфазного commit: успех и audit-проекция снятых со сторон goods.
/// При отказе журнал обмена не пишется и проекция пуста — поведение прежнего
/// owner-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerTradeCommitReport {
    pub completed: bool,
    pub audit_goods_by_plug: BTreeMap<i32, Vec<CGoods>>,
}

impl PlayerTradeCommitReport {
    fn failed() -> Self {
        Self {
            completed: false,
            audit_goods_by_plug: BTreeMap::new(),
        }
    }
}

/// Одна доставленная позиция rollback-трека commit-а (внутреннее звено
/// драйвера между фазами).
#[derive(Clone, Debug, Eq, PartialEq)]
struct PlayerTradeDelivered<Delivery> {
    source_plug_id: i32,
    receiver_id: i32,
    original: CGoods,
    addition: Delivery,
}

/// Живые шаги commit обмена, исполняемые хозяином игроков (hub `CGame`):
/// каждый метод — один наблюдаемый шаг прежнего owner-а; Zone-драйвер
/// [`commit_player_trade`] вызывает их в машинном порядке `0x1B9470`.
pub trait PlayerTradeHost {
    /// Непрозрачный для Zone токен доставки (hub `CiQingPacketAddition`).
    type Delivery;

    /// Pre-detach снимок источника предложения: (количество, base index).
    fn query_trade_offer_source(&mut self, owner_id: i32, offer: &GoodsShadow)
    -> Option<(u32, u32)>;

    /// Создание split-экземпляра packet-части предложения (Game RNG factory).
    fn create_trade_split_goods(&mut self, base_properties_index: u32) -> Option<CGoods>;

    /// Снятие packet-предложения живого игрока с клиентским удалением;
    /// `split` — заранее созданный экземпляр для частичного снятия.
    fn detach_trade_packet_offer(
        &mut self,
        owner_id: i32,
        offer: &GoodsShadow,
        split: Option<CGoods>,
    ) -> Option<CGoods>;

    /// Снятие equipment-предложения живого игрока с полным property/skill
    /// проходом и клиентским удалением.
    fn detach_trade_equipment_offer(&mut self, owner_id: i32, offer: &GoodsShadow)
    -> Option<CGoods>;

    /// Доставка отделённых goods в packet получателя с клиентскими сообщениями;
    /// `Err(goods)` — получатель уже offline, товары возвращаются драйверу.
    fn deliver_trade_goods(
        &mut self,
        receiver_id: i32,
        goods: Vec<CGoods>,
    ) -> Result<PlayerTradeDelivery<Self::Delivery>, Vec<CGoods>>;

    /// Отмена одной уже выполненной доставки у получателя; `true`, если
    /// добавка снята. Исходный rollback обращает только ПОЛНЫЙ merge рамки —
    /// необращённое владелец не возвращает (`false`).
    fn undo_trade_delivery(
        &mut self,
        receiver_id: i32,
        addition: Self::Delivery,
        original_amount: u32,
    ) -> bool;

    /// Возврат отделённых goods владельцу с клиентскими сообщениями; `None` —
    /// владелец offline и товары утрачены, как у прежнего owner-а.
    fn restore_trade_goods(
        &mut self,
        owner_id: i32,
        goods: Vec<CGoods>,
    ) -> Option<PlayerTradeDelivery<()>>;

    /// Уведомление `GS0277` о невозвращённых позициях rollback-а.
    fn notify_trade_restore_loss(&mut self, owner_id: i32, unrecoverable: usize);

    /// Текущий баланс денег участника; `None` — игрок offline.
    fn trade_party_money(&mut self, owner_id: i32) -> Option<u32>;

    /// Применение денежного слияния стороны с клиентским сообщением.
    fn apply_trade_money_merge(&mut self, owner_id: i32, merge: TradeMoneyMerge);
}

/// Двухсторонняя сверка условий обмена `CheckTradeCondition`: для каждой
/// стороны на КОПИИ её packet снимаются собственные предложения и добавляются
/// входящие, затем проверяются итоговый burden и валютные баланс/ёмкость.
/// Порядок отказов — исходный; чтения живых игроков поставляет caller,
/// симуляция игроков не меняет.
pub fn validate_player_trade(
    factory: &CGoodsFactory,
    parties: &[PlayerTradePartySnapshot; 2],
    mut source_goods: impl FnMut(i32, i32, u32, CGuid) -> Option<CGoods>,
    mut party_packet: impl FnMut(i32) -> Option<CVolumeLimitGoodsContainer>,
    mut party_burden: impl FnMut(i32) -> Option<(u32, u32)>,
    mut party_balance: impl FnMut(i32) -> Option<(u32, u32)>,
    mut party_present: impl FnMut(i32) -> bool,
) -> Result<(), PlayerTradeConditionBlock> {
    let maximum_gold = factory.query_goods_max_stack_number(factory.get_gold_coin_index());
    let maximum_yuan_bao = factory.query_goods_max_stack_number(factory.get_yuan_bao_index());
    for index in 0..2 {
        let party = &parties[index];
        let contrary = &parties[1 - index];
        let mut packet =
            party_packet(party.owner_id).ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
        let mut own_weight = 0u32;
        for offer in &party.goods {
            let goods = source_goods(
                party.owner_id,
                offer.original_container_extend_id,
                offer.original_goods_position,
                offer.goods_id,
            )
            .filter(|goods| {
                trade_offer_amount_satisfies(
                    offer.original_container_extend_id,
                    goods.amount(),
                    offer.goods_amount,
                )
            })
            .ok_or(PlayerTradeConditionBlock::MissingOfferedGoods)?;
            let mut offered_goods = goods.clone();
            offered_goods.set_amount(offer.goods_amount);
            own_weight = own_weight.wrapping_add(offered_goods.weight(factory));
            if offer.original_container_extend_id == 1 {
                let mut split = Some(goods);
                if packet
                    .take_goods(
                        offer.original_goods_position,
                        offer.goods_amount,
                        factory,
                        |_| split.take(),
                    )
                    .is_none()
                {
                    return Err(PlayerTradeConditionBlock::MissingOfferedGoods);
                }
            }
        }
        if !party_present(contrary.owner_id) {
            return Err(PlayerTradeConditionBlock::MissingPlayerOrPlug);
        }
        let mut incoming_weight = 0u32;
        for offer in &contrary.goods {
            let goods = source_goods(
                contrary.owner_id,
                offer.original_container_extend_id,
                offer.original_goods_position,
                offer.goods_id,
            )
            .filter(|goods| {
                trade_offer_amount_satisfies(
                    offer.original_container_extend_id,
                    goods.amount(),
                    offer.goods_amount,
                )
            })
            .ok_or(PlayerTradeConditionBlock::MissingOfferedGoods)?;
            let mut offered_goods = goods.clone();
            offered_goods.set_amount(offer.goods_amount);
            incoming_weight = incoming_weight.wrapping_add(offered_goods.weight(factory));
            let mut incoming = Some(offered_goods);
            let outcome = packet.add_goods(&mut incoming, factory, true);
            if incoming.is_some() || matches!(outcome, VolumeGoodsAddOutcome::Rejected(_)) {
                return Err(PlayerTradeConditionBlock::PacketSpace);
            }
        }
        let (current_burden, max_burden) =
            party_burden(party.owner_id).ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
        if trade_resulting_burden_exceeded(current_burden, own_weight, incoming_weight, max_burden)
        {
            return Err(PlayerTradeConditionBlock::BurdenExceeded);
        }
        let (money, yuan_bao) =
            party_balance(party.owner_id).ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
        if trade_currency_balance_insufficient(money, party.gold) {
            return Err(PlayerTradeConditionBlock::InsufficientGold);
        }
        if trade_currency_capacity_exceeded(money, party.gold, contrary.gold, maximum_gold) {
            return Err(PlayerTradeConditionBlock::GoldCapacity);
        }
        if trade_currency_balance_insufficient(yuan_bao, party.yuan_bao) {
            return Err(PlayerTradeConditionBlock::InsufficientYuanBao);
        }
        if trade_currency_capacity_exceeded(
            yuan_bao,
            party.yuan_bao,
            contrary.yuan_bao,
            maximum_yuan_bao,
        ) {
            return Err(PlayerTradeConditionBlock::YuanBaoCapacity);
        }
    }
    Ok(())
}

/// Двухфазный ownership transaction обмена `0x1B9470`: detach предложений у
/// обеих сторон (splits packet-частей создаются до снятия), доставка
/// получателям с rollback-треком, денежное слияние. При любом отказе уже
/// доставленное снимается у получателей в обратном порядке, а отделённое
/// возвращается владельцам — исходный `RollBack`.
pub fn commit_player_trade<Host: PlayerTradeHost>(
    host: &mut Host,
    parties: &[PlayerTradePartySnapshot; 2],
) -> PlayerTradeCommitReport {
    let mut removed_by_plug = BTreeMap::<i32, Vec<CGoods>>::new();
    let mut delivered = Vec::<PlayerTradeDelivered<Host::Delivery>>::new();
    for party in parties {
        let mut packet_splits = BTreeMap::<CGuid, CGoods>::new();
        for offer in &party.goods {
            if offer.original_container_extend_id != 1 {
                continue;
            }
            let Some((source_amount, base_properties_index)) =
                host.query_trade_offer_source(party.owner_id, offer)
            else {
                continue;
            };
            if offer.goods_amount < source_amount {
                let Some(split) = host.create_trade_split_goods(base_properties_index) else {
                    undo_delivered_trade_goods(host, &mut removed_by_plug, delivered);
                    rollback_detached_trade_goods(host, parties, removed_by_plug);
                    return PlayerTradeCommitReport::failed();
                };
                packet_splits.insert(offer.goods_id, split);
            }
        }
        let mut removed_goods = Vec::new();
        let mut failed = false;
        for offer in &party.goods {
            let detached = match offer.original_container_extend_id {
                1 => host.detach_trade_packet_offer(
                    party.owner_id,
                    offer,
                    packet_splits.remove(&offer.goods_id),
                ),
                2 => host.detach_trade_equipment_offer(party.owner_id, offer),
                _ => None,
            };
            let Some(detached) = detached else {
                failed = true;
                break;
            };
            removed_goods.push(detached);
        }
        removed_by_plug.insert(party.plug_id, removed_goods);
        if failed {
            undo_delivered_trade_goods(host, &mut removed_by_plug, delivered);
            rollback_detached_trade_goods(host, parties, removed_by_plug);
            return PlayerTradeCommitReport::failed();
        }
    }
    let audit_goods_by_plug = removed_by_plug.clone();
    for index in 0..2 {
        let source = &parties[index];
        let receiver = &parties[1 - index];
        let goods = removed_by_plug.remove(&source.plug_id).unwrap_or_default();
        let originals = goods.clone();
        let PlayerTradeDelivery { additions, rejected } =
            match host.deliver_trade_goods(receiver.owner_id, goods) {
                Ok(delivery) => delivery,
                Err(goods) => {
                    removed_by_plug.insert(source.plug_id, goods);
                    undo_delivered_trade_goods(host, &mut removed_by_plug, delivered);
                    rollback_detached_trade_goods(host, parties, removed_by_plug);
                    return PlayerTradeCommitReport::failed();
                }
            };
        let failed = !rejected.is_empty()
            || additions
                .iter()
                .any(|addition| addition.resulting_amount.is_none());
        for (original, addition) in originals.into_iter().zip(additions.into_iter()) {
            if addition.resulting_amount.is_some() {
                delivered.push(PlayerTradeDelivered {
                    source_plug_id: source.plug_id,
                    receiver_id: receiver.owner_id,
                    original,
                    addition: addition.token,
                });
            }
        }
        if failed {
            removed_by_plug.insert(source.plug_id, rejected);
            undo_delivered_trade_goods(host, &mut removed_by_plug, delivered);
            rollback_detached_trade_goods(host, parties, removed_by_plug);
            return PlayerTradeCommitReport::failed();
        }
    }
    for index in 0..2 {
        let party = &parties[index];
        let contrary = &parties[1 - index];
        let current = host.trade_party_money(party.owner_id).unwrap_or(0);
        host.apply_trade_money_merge(
            party.owner_id,
            merge_trade_money(current, party.gold, contrary.gold),
        );
    }
    PlayerTradeCommitReport {
        completed: true,
        audit_goods_by_plug,
    }
}

/// Отмена уже доставленного у получателей в обратном порядке исходного
/// `RollBack`; необращённые добавки (неполный merge) владельцам не
/// возвращаются — как у release-формы оригинала.
fn undo_delivered_trade_goods<Host: PlayerTradeHost>(
    host: &mut Host,
    detached_by_plug: &mut BTreeMap<i32, Vec<CGoods>>,
    delivered: Vec<PlayerTradeDelivered<Host::Delivery>>,
) {
    for delivered in delivered.into_iter().rev() {
        if host.undo_trade_delivery(
            delivered.receiver_id,
            delivered.addition,
            delivered.original.amount(),
        ) {
            detached_by_plug
                .entry(delivered.source_plug_id)
                .or_default()
                .push(delivered.original);
        }
    }
}

/// Возврат отделённых goods владельцам тем же packet add исходного отката;
/// о невозвращённых позициях Zone решает, hub уведомляет (`GS0277`).
fn rollback_detached_trade_goods<Host: PlayerTradeHost>(
    host: &mut Host,
    parties: &[PlayerTradePartySnapshot; 2],
    mut goods_by_plug: BTreeMap<i32, Vec<CGoods>>,
) {
    for party in parties {
        let goods = goods_by_plug.remove(&party.plug_id).unwrap_or_default();
        if goods.is_empty() {
            continue;
        }
        let Some(delivery) = host.restore_trade_goods(party.owner_id, goods) else {
            continue;
        };
        if !delivery.rejected.is_empty()
            || delivery
                .additions
                .iter()
                .any(|addition| addition.resulting_amount.is_none())
        {
            host.notify_trade_restore_loss(party.owner_id, delivery.rejected.len());
        }
    }
}

/// Одна запись goods-audit очереди обмена: trace-скаляры и готовый кадр
/// `0x60201` (`trade/audit`).
pub struct PlayerTradeGoodsAudit {
    pub source_id: i32,
    pub receiver_id: i32,
    pub goods_id: CGuid,
    pub amount: u32,
    pub frame: CMessage,
}

/// Очередь кадров журнала предметов обмена после commit в исходном порядке:
/// по каждой стороне её снятые goods, затем денежная псевдо-запись
/// (`GUID_INVALID`, пустое имя, price == amount == сумме). Флаги log-system и
/// transport (`Send` на log server) остаются у caller-а.
pub fn player_trade_goods_audit_frames(
    parties: &[PlayerTradePartySnapshot; 2],
    audit_parties: &[PlayerTradeAuditParty; 2],
    goods_by_plug: &BTreeMap<i32, Vec<CGoods>>,
) -> Vec<PlayerTradeGoodsAudit> {
    let mut audits = Vec::new();
    for source_index in 0..2 {
        let receiver_index = 1 - source_index;
        let source = &audit_parties[source_index];
        let receiver = &audit_parties[receiver_index];
        let source_frame = source.frame();
        let receiver_frame = receiver.frame();
        for goods in goods_by_plug
            .get(&parties[source_index].plug_id)
            .into_iter()
            .flatten()
        {
            audits.push(PlayerTradeGoodsAudit {
                source_id: source.owner_id,
                receiver_id: receiver.owner_id,
                goods_id: goods.identity().ex_id,
                amount: goods.amount(),
                frame: trade_goods_audit_frame(
                    &source_frame,
                    &receiver_frame,
                    goods.identity().ex_id,
                    goods.price(),
                    goods.amount(),
                    goods.name(),
                ),
            });
        }
        let gold = parties[source_index].gold;
        if gold != 0 {
            audits.push(PlayerTradeGoodsAudit {
                source_id: source.owner_id,
                receiver_id: receiver.owner_id,
                goods_id: CGuid::GUID_INVALID,
                amount: gold,
                frame: trade_goods_audit_frame(
                    &source_frame,
                    &receiver_frame,
                    CGuid::GUID_INVALID,
                    gold,
                    gold,
                    b"",
                ),
            });
        }
    }
    audits
}
