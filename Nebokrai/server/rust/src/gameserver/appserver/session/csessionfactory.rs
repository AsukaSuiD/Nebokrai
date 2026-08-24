//! Достигнутая lookup-часть GameServer `CSessionFactory`.
//!
//! `QuerySession` RVA `0x000780C0` и `QueryPlug` RVA `0x00078190` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `server/gameserver/appserver/session/csessionfactory.cpp`. PDB подтверждает
//! две static `hash_map<long, CSession*/CPlug*>`; оба lookup возвращают null
//! при отсутствии ключа.
//!
//! Один owned `CSessionFactory`, подключённый к `CGame`, заменяет две
//! process-static maps, а `BTreeMap`
//! является deterministic replacement: hash iteration этими функциями не
//! наблюдается, только exact-key lookup. `register_*`
//! материализует достигнутый registry storage; `goodsmessage 0x8FC25`
//! выполняет ordered session plug lookup по owner type/ID. Equipment-upgrade
//! close материализует concrete session GC. Script-входы трёх equipment
//! механик также создают normal session и typed plug, связывают owner/session,
//! shadow owner/extend ID и insert-order. Container-message проход разрешает
//! wire `(session, plug << 8)`, записывает и снимает typed upgrade/DaKong/
//! compose shadows с исходным player slot. Terminal `End/Exit` хранится здесь,
//! а ended equipment-session GC сохраняет session/plug order и owner identity
//! для listener detach на MainLoop session-stage. Достигнутый personal-shop
//! lifecycle создаёт normal `(1, 20, 0)` session, typed seller/buyer plugs,
//! exact owner/session relations и personal-shop shadow metadata. Team и
//! остальные polymorphic session-варианты ниже этим не объявляются готовыми.

use std::collections::BTreeMap;

use crate::gameserver::appserver::container::camountlimitgoodsshadowcontainer::AmountShadowAdded;
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeShadowAddBlock;
use crate::gameserver::appserver::container::cequipmentdakongcontainer::{
    DaKongAddBlock, DaKongAddOutcome,
};
use crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeShadowAddBlock;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::{
    PlacedShadowGoods, ShadowRecordBlock, ShadowRemovedReport,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

use super::cequipmentcompose::CEquipmentCompose;
use super::cequipmentdakong::CEquipmentDaKong;
use super::cequipmentupgrade::CEquipmentUpgrade;
use super::cpersonalshopbuyer::CPersonalShopBuyer;
use super::cpersonalshopseller::CPersonalShopSeller;
use super::cplug::CPlug;
use super::csession::CSession;
use super::ctrader::CTrader;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionPlugKind {
    Upgrade,
    DaKong,
    Compose,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionShadowAddBlock {
    MissingSessionOrPlug,
    OwnerMismatch,
    InvalidContainerIndex,
    Upgrade(UpgradeShadowAddBlock),
    DaKong(DaKongAddBlock),
    Compose(ComposeShadowAddBlock),
}

#[must_use = "shadow add содержит actual cell и AddShadow publication data"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionShadowAdded {
    pub(crate) kind: EquipmentSessionPlugKind,
    pub(crate) plug_id: i32,
    pub(crate) position: u32,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "shadow remove сохраняет original source и delete publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionShadowRemoved {
    pub(crate) kind: EquipmentSessionPlugKind,
    pub(crate) plug_id: i32,
    pub(crate) original: PreviousContainer,
    pub(crate) removed: ShadowRemovedReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShopShadowAddBlock {
    MissingSessionOrPlug,
    OwnerMismatch,
    InvalidContainerIndex,
    PositionUnavailable,
    Shadow(ShadowRecordBlock),
}

#[must_use = "personal-shop add сохраняет actual cell и AddShadow publication data"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopShadowAdded {
    pub(crate) plug_id: i32,
    pub(crate) position: u32,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "personal-shop remove сохраняет original source и delete publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopShadowRemoved {
    pub(crate) plug_id: i32,
    pub(crate) original: PreviousContainer,
    pub(crate) removed: ShadowRemovedReport,
}

#[must_use = "session end сохраняет ordered callback targets и terminal state"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SessionEndReport {
    pub(crate) session_id: i32,
    pub(crate) ended: bool,
    pub(crate) remove_requested: bool,
    pub(crate) callback_plug_ids: Vec<i32>,
}

#[must_use = "plug exit фиксирует session dispatch и ended state"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlugExitReport {
    pub(crate) session_id: i32,
    pub(crate) plug_id: i32,
    pub(crate) session_found: bool,
    pub(crate) exited: bool,
}

#[must_use = "terminal GC report сохраняет session и ordered plug identities"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEquipmentSessionCollected {
    pub(crate) session_id: i32,
    pub(crate) plugs: Vec<TerminalEquipmentPlugCollected>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEquipmentPlugCollected {
    pub(crate) plug_id: i32,
    pub(crate) owner_id: i32,
}

#[derive(Debug)]
pub(crate) struct CSessionFactory {
    sessions: BTreeMap<i32, CSession>,
    plugs: BTreeMap<i32, CPlug>,
    equipment_compose_plugs: BTreeMap<i32, CEquipmentCompose>,
    equipment_da_kong_plugs: BTreeMap<i32, CEquipmentDaKong>,
    equipment_upgrade_plugs: BTreeMap<i32, CEquipmentUpgrade>,
    personal_shop_seller_plugs: BTreeMap<i32, CPersonalShopSeller>,
    personal_shop_buyer_plugs: BTreeMap<i32, CPersonalShopBuyer>,
    trader_plugs: BTreeMap<i32, CTrader>,
    next_session_id: i32,
    next_plug_id: i32,
}

impl Default for CSessionFactory {
    fn default() -> Self {
        Self {
            sessions: BTreeMap::new(),
            plugs: BTreeMap::new(),
            equipment_compose_plugs: BTreeMap::new(),
            equipment_da_kong_plugs: BTreeMap::new(),
            equipment_upgrade_plugs: BTreeMap::new(),
            personal_shop_seller_plugs: BTreeMap::new(),
            personal_shop_buyer_plugs: BTreeMap::new(),
            trader_plugs: BTreeMap::new(),
            next_session_id: 1,
            next_plug_id: 1,
        }
    }
}

impl CSessionFactory {
    /// Exact normal `(2, 2, 0)` player trade: первый plug принадлежит
    /// пригласившему, второй — отвечающему, как два последовательных
    /// `CreatePlug/InsertPlug` в `0x8FA07`.
    pub(crate) fn create_player_trade_session(
        &mut self,
        inviter_id: i32,
        answerer_id: i32,
    ) -> Option<(i32, i32, i32)> {
        let session_id = self.next_session_id;
        let mut session = CSession::normal(2, 2, 0);
        if !session.start() {
            return None;
        }
        let inviter_plug_id = self.next_plug_id;
        let answerer_plug_id = self.next_plug_id.wrapping_add(1);
        let mut inviter = CPlug::new();
        inviter.set_id(inviter_plug_id);
        inviter.set_owner(400, inviter_id);
        inviter.set_session(session_id);
        inviter.set_plug_type(1);
        let mut answerer = CPlug::new();
        answerer.set_id(answerer_plug_id);
        answerer.set_owner(400, answerer_id);
        answerer.set_session(session_id);
        answerer.set_plug_type(1);
        if !session.insert_plug(inviter_plug_id) || !session.insert_plug(answerer_plug_id) {
            return None;
        }
        self.next_session_id = self.next_session_id.wrapping_add(1);
        self.next_plug_id = self.next_plug_id.wrapping_add(2);
        self.sessions.insert(session_id, session);
        self.plugs.insert(inviter_plug_id, inviter);
        self.plugs.insert(answerer_plug_id, answerer);
        self.trader_plugs.insert(
            inviter_plug_id,
            CTrader::inserted(inviter_plug_id, session_id, inviter_id),
        );
        self.trader_plugs.insert(
            answerer_plug_id,
            CTrader::inserted(answerer_plug_id, session_id, answerer_id),
        );
        Some((session_id, inviter_plug_id, answerer_plug_id))
    }

    pub(crate) fn query_trader(&self, plug_id: i32) -> Option<&CTrader> {
        self.trader_plugs.get(&plug_id)
    }

    pub(crate) fn query_trader_mut(&mut self, plug_id: i32) -> Option<&mut CTrader> {
        self.trader_plugs.get_mut(&plug_id)
    }

    pub(crate) fn trader_plug_by_owner(&self, session_id: i32, owner_id: i32) -> Option<i32> {
        let plug = self.query_session_plug_by_owner(session_id, 400, owner_id)?;
        self.trader_plugs
            .contains_key(&plug.id())
            .then_some(plug.id())
    }

    pub(crate) fn contrary_trader_id(&self, session_id: i32, plug_id: i32) -> Option<i32> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .copied()
            .find(|candidate| *candidate != plug_id && self.trader_plugs.contains_key(candidate))
    }

    pub(crate) fn trade_session_plug_ids(&self, session_id: i32) -> Option<Vec<i32>> {
        let session = self.sessions.get(&session_id)?;
        let ids: Vec<_> = session
            .plug_ids_storage()
            .iter()
            .copied()
            .filter(|plug_id| self.trader_plugs.contains_key(plug_id))
            .collect();
        (ids.len() == session.plug_ids_storage().len()).then_some(ids)
    }

    pub(crate) fn trade_session_available(
        &self,
        session_id: i32,
        mut owner_available: impl FnMut(i32) -> bool,
    ) -> bool {
        let Some(session) = self.sessions.get(&session_id) else {
            return false;
        };
        if !session.is_available_prefix() {
            return false;
        }
        let available = session
            .plug_ids_storage()
            .iter()
            .filter_map(|plug_id| self.trader_plugs.get(plug_id))
            .filter(|trader| owner_available(trader.owner_id()))
            .count();
        session.minimum_plugs() as usize <= available
    }

    pub(crate) fn abort_session(&mut self, session_id: i32) -> Option<SessionEndReport> {
        let callback_plug_ids = self.sessions.get_mut(&session_id)?.abort();
        let callback_plug_ids = callback_plug_ids
            .into_iter()
            .filter(|plug_id| self.plugs.contains_key(plug_id))
            .collect();
        let session = self.sessions.get(&session_id)?;
        Some(SessionEndReport {
            session_id,
            ended: session.is_ended(),
            remove_requested: session.remove_requested(),
            callback_plug_ids,
        })
    }

    /// Exact normal `(1, 20, 0)` session + personal-shop seller plug `(400,
    /// player)`. Свежая session всегда проходит `Start(0)` и первый insertion;
    /// поэтому safe atomic publication не меняет достижимый legacy outcome.
    pub(crate) fn create_personal_shop_seller_session(
        &mut self,
        player_id: i32,
    ) -> Option<(i32, i32)> {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        let mut session = CSession::normal(1, 20, 0);
        if !session.start() {
            return None;
        }

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, player_id);
        base.set_session(session_id);
        base.set_plug_type(2);
        if !session.insert_plug(plug_id) {
            return None;
        }

        self.sessions.insert(session_id, session);
        self.plugs.insert(plug_id, base);
        self.personal_shop_seller_plugs
            .insert(plug_id, CPersonalShopSeller::inserted(plug_id));
        Some((session_id, plug_id))
    }

    pub(crate) fn personal_shop_seller(&self, plug_id: i32) -> Option<&CPersonalShopSeller> {
        self.personal_shop_seller_plugs.get(&plug_id)
    }

    pub(crate) fn personal_shop_seller_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CPersonalShopSeller> {
        self.personal_shop_seller_plugs.get_mut(&plug_id)
    }

    pub(crate) fn personal_shop_buyer(&self, plug_id: i32) -> Option<&CPersonalShopBuyer> {
        self.personal_shop_buyer_plugs.get(&plug_id)
    }

    pub(crate) fn personal_shop_seller_plug_id(&self, session_id: i32) -> Option<i32> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .copied()
            .find(|plug_id| self.personal_shop_seller_plugs.contains_key(plug_id))
    }

    pub(crate) fn personal_shop_session_available(&self, session_id: i32) -> bool {
        self.sessions
            .get(&session_id)
            .is_some_and(CSession::is_available_prefix)
    }

    pub(crate) fn insert_personal_shop_buyer(
        &mut self,
        session_id: i32,
        owner_id: i32,
    ) -> Option<i32> {
        let session = self.sessions.get_mut(&session_id)?;
        if !session.is_available_prefix() {
            return None;
        }
        let plug_id = self.next_plug_id;
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, owner_id);
        base.set_session(session_id);
        base.set_plug_type(3);
        if !session.insert_plug(plug_id) {
            return None;
        }
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        self.plugs.insert(plug_id, base);
        self.personal_shop_buyer_plugs.insert(
            plug_id,
            CPersonalShopBuyer::inserted(plug_id, session_id, owner_id),
        );
        Some(plug_id)
    }

    pub(crate) fn remove_personal_shop_buyer(
        &mut self,
        session_id: i32,
        plug_id: i32,
    ) -> Option<CPersonalShopBuyer> {
        let buyer = self.personal_shop_buyer_plugs.remove(&plug_id)?;
        if buyer.session_id() != session_id {
            self.personal_shop_buyer_plugs.insert(plug_id, buyer);
            return None;
        }
        let _ = self.sessions.get_mut(&session_id)?.remove_plug(plug_id);
        self.plugs.remove(&plug_id);
        Some(buyer)
    }

    pub(crate) fn personal_shop_participants(
        &self,
        session_id: i32,
    ) -> Option<(i32, i32, Vec<CPersonalShopBuyer>)> {
        let seller_plug_id = self.personal_shop_seller_plug_id(session_id)?;
        let seller_owner_id = self.plugs.get(&seller_plug_id)?.owner_id();
        let buyers = self
            .sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .filter_map(|plug_id| self.personal_shop_buyer_plugs.get(plug_id).copied())
            .collect();
        Some((seller_plug_id, seller_owner_id, buyers))
    }

    fn resolve_personal_shop_seller_plug(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> Result<i32, PersonalShopShadowAddBlock> {
        if extend_id & 0xff != 0 {
            return Err(PersonalShopShadowAddBlock::InvalidContainerIndex);
        }
        let plug_id = extend_id >> 8;
        let plug = self
            .query_session_plug_by_owner(session_id, 400, player_id)
            .ok_or(PersonalShopShadowAddBlock::MissingSessionOrPlug)?;
        if plug.id() != plug_id {
            return Err(PersonalShopShadowAddBlock::OwnerMismatch);
        }
        self.personal_shop_seller_plugs
            .contains_key(&plug_id)
            .then_some(plug_id)
            .ok_or(PersonalShopShadowAddBlock::MissingSessionOrPlug)
    }

    pub(crate) fn is_personal_shop_seller_container(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> bool {
        self.resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .is_ok()
    }

    pub(crate) fn record_personal_shop_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        requested_position: u32,
        goods: &CGoods,
        previous: PreviousContainer,
    ) -> Result<PersonalShopShadowAdded, PersonalShopShadowAddBlock> {
        let plug_id = self.resolve_personal_shop_seller_plug(session_id, extend_id, player_id)?;
        let seller = self
            .personal_shop_seller_plugs
            .get_mut(&plug_id)
            .expect("personal-shop plug проверен по concrete registry");
        let position = if requested_position == u32::MAX {
            seller
                .goods()
                .query_goods_position(goods.identity().ex_id)
                .or_else(|| {
                    (0..seller.goods().size())
                        .find(|position| seller.goods().is_space_enough(*position))
                })
                .ok_or(PersonalShopShadowAddBlock::PositionUnavailable)?
        } else {
            requested_position
        };
        if seller.goods().query_goods_position(goods.identity().ex_id) != Some(position)
            && !seller.goods().is_space_enough(position)
        {
            return Err(PersonalShopShadowAddBlock::PositionUnavailable);
        }
        let placed = PlacedShadowGoods {
            identity: goods.identity().ex_id,
            position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        let shadow = seller
            .goods_mut()
            .base_mut()
            .record_placed_goods(previous, placed)
            .map_err(PersonalShopShadowAddBlock::Shadow)?;
        if !seller
            .goods_mut()
            .occupy_cell(position, goods.identity().ex_id)
            && seller.goods().query_goods_position(goods.identity().ex_id) != Some(position)
        {
            let _ = seller.goods_mut().remove_shadow(goods.identity().ex_id);
            return Err(PersonalShopShadowAddBlock::PositionUnavailable);
        }
        Ok(PersonalShopShadowAdded {
            plug_id,
            position,
            shadow,
        })
    }

    pub(crate) fn personal_shop_shadow_original(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        self.personal_shop_seller_plugs
            .get(&plug_id)?
            .goods()
            .base()
            .base()
            .original_container_information(goods_id)
    }

    pub(crate) fn personal_shop_shadow_position(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<u32> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        self.personal_shop_seller_plugs
            .get(&plug_id)?
            .goods()
            .query_goods_position(goods_id)
    }

    pub(crate) fn remove_personal_shop_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PersonalShopShadowRemoved> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        let original =
            self.personal_shop_shadow_original(session_id, extend_id, player_id, goods_id)?;
        let seller = self.personal_shop_seller_plugs.get_mut(&plug_id)?;
        seller.remove_goods_price(goods_id);
        let removed = seller.goods_mut().remove_shadow(goods_id)?;
        Some(PersonalShopShadowRemoved {
            plug_id,
            original,
            removed,
        })
    }

    fn resolve_equipment_plug(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> Result<(EquipmentSessionPlugKind, i32), EquipmentSessionShadowAddBlock> {
        if extend_id & 0xff != 0 {
            return Err(EquipmentSessionShadowAddBlock::InvalidContainerIndex);
        }
        let plug_id = extend_id >> 8;
        let plug = self
            .query_session_plug_by_owner(session_id, 400, player_id)
            .ok_or(EquipmentSessionShadowAddBlock::MissingSessionOrPlug)?;
        if plug.id() != plug_id {
            return Err(EquipmentSessionShadowAddBlock::OwnerMismatch);
        }
        let kind = if self.equipment_upgrade_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::Upgrade
        } else if self.equipment_da_kong_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::DaKong
        } else if self.equipment_compose_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::Compose
        } else {
            return Err(EquipmentSessionShadowAddBlock::MissingSessionOrPlug);
        };
        Ok((kind, plug_id))
    }

    pub(crate) fn record_equipment_session_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        requested_position: u32,
        goods: &CGoods,
        previous: PreviousContainer,
        factory: &CGoodsFactory,
    ) -> Result<EquipmentSessionShadowAdded, EquipmentSessionShadowAddBlock> {
        let (kind, plug_id) = self.resolve_equipment_plug(session_id, extend_id, player_id)?;
        let goods_id = goods.identity().ex_id;
        let placed_position = previous.goods_position;
        let (position, shadow) = match kind {
            EquipmentSessionPlugKind::Upgrade => {
                let plug = self
                    .equipment_upgrade_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.upgrade_container()
                        .select_cell(goods, factory)
                        .map_err(EquipmentSessionShadowAddBlock::Upgrade)?
                } else {
                    crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeEquipmentCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::Upgrade(
                            UpgradeShadowAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                let added = plug
                    .upgrade_container_mut()
                    .record_placed_goods(cell, goods, previous, goods_id, placed_position, factory)
                    .map_err(EquipmentSessionShadowAddBlock::Upgrade)?;
                (added.cell.position(), added.shadow)
            }
            EquipmentSessionPlugKind::DaKong => {
                let plug = self
                    .equipment_da_kong_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.upgrade_container()
                        .select_cell(goods, factory)
                        .map_err(EquipmentSessionShadowAddBlock::DaKong)?
                } else {
                    crate::gameserver::appserver::container::cequipmentdakongcontainer::DaKongCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::DaKong(
                            DaKongAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                match plug.upgrade_container_mut().record_placed_goods(
                    cell,
                    goods,
                    previous,
                    goods_id,
                    placed_position,
                ) {
                    DaKongAddOutcome::Added { effects, shadow } => {
                        (effects.cell.position(), shadow)
                    }
                    DaKongAddOutcome::Rejected { block, .. } => {
                        return Err(EquipmentSessionShadowAddBlock::DaKong(block));
                    }
                }
            }
            EquipmentSessionPlugKind::Compose => {
                let plug = self
                    .equipment_compose_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.compose_container()
                        .select_cell()
                        .map_err(EquipmentSessionShadowAddBlock::Compose)?
                } else {
                    crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeEquipmentCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::Compose(
                            ComposeShadowAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                let added = plug
                    .compose_container_mut()
                    .record_placed_goods(cell, goods, previous, goods_id, placed_position)
                    .map_err(EquipmentSessionShadowAddBlock::Compose)?;
                (added.cell.position(), added.shadow)
            }
        };
        Ok(EquipmentSessionShadowAdded {
            kind,
            plug_id,
            position,
            shadow,
        })
    }

    pub(crate) fn equipment_session_shadow_original(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get(&plug_id)?
                .upgrade_container()
                .base()
                .base()
                .original_container_information(goods_id),
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get(&plug_id)?
                .upgrade_container()
                .original_container_information(goods_id),
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get(&plug_id)?
                .compose_container()
                .original_container_information(goods_id),
        }
    }

    pub(crate) fn equipment_session_shadow_position(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<u32> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get(&plug_id)?
                .upgrade_container()
                .positions()
                .iter()
                .find_map(|(cell, id)| (*id == goods_id).then_some(cell.position())),
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get(&plug_id)?
                .upgrade_container()
                .positions()
                .iter()
                .find_map(|(cell, id)| (*id == goods_id).then_some(cell.position())),
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get(&plug_id)?
                .compose_container()
                .query_goods_position(goods_id),
        }
    }

    pub(crate) fn remove_equipment_session_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<EquipmentSessionShadowRemoved> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        let original =
            self.equipment_session_shadow_original(session_id, extend_id, player_id, goods_id)?;
        let removed = match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get_mut(&plug_id)?
                .upgrade_container_mut()
                .remove_shadow(goods_id)?,
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get_mut(&plug_id)?
                .upgrade_container_mut()
                .remove_shadow(goods_id)?,
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get_mut(&plug_id)?
                .compose_container_mut()
                .remove_shadow(goods_id)?,
        };
        Some(EquipmentSessionShadowRemoved {
            kind,
            plug_id,
            original,
            removed,
        })
    }
    pub(crate) fn create_equipment_session(
        &mut self,
        kind: EquipmentSessionPlugKind,
        player_id: i32,
    ) -> Option<(i32, i32)> {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        let mut session = CSession::normal(1, 1, 0);
        if !session.start() {
            return None;
        }

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, player_id);
        base.set_session(session_id);
        base.set_plug_type(match kind {
            EquipmentSessionPlugKind::Upgrade => 4,
            EquipmentSessionPlugKind::DaKong => 6,
            EquipmentSessionPlugKind::Compose => 7,
        });
        if !session.insert_plug(plug_id) {
            return None;
        }

        self.sessions.insert(session_id, session);
        self.plugs.insert(plug_id, base);
        match kind {
            EquipmentSessionPlugKind::Upgrade => {
                let mut plug = CEquipmentUpgrade::new();
                let shadow = plug.upgrade_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_upgrade_plugs.insert(plug_id, plug);
            }
            EquipmentSessionPlugKind::DaKong => {
                let mut plug = CEquipmentDaKong::new();
                let shadow = plug.upgrade_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_da_kong_plugs.insert(plug_id, plug);
            }
            EquipmentSessionPlugKind::Compose => {
                let mut plug = CEquipmentCompose::new();
                let shadow = plug.compose_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_compose_plugs.insert(plug_id, plug);
            }
        }
        Some((session_id, plug_id))
    }
    pub(crate) fn register_session(
        &mut self,
        session_id: i32,
        session: CSession,
    ) -> Option<CSession> {
        self.sessions.insert(session_id, session)
    }

    pub(crate) fn register_plug(&mut self, plug_id: i32, mut plug: CPlug) -> Option<CPlug> {
        plug.set_id(plug_id);
        self.plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_session(&self, session_id: i32) -> Option<&CSession> {
        self.sessions.get(&session_id)
    }

    pub(crate) fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        self.plugs.get(&plug_id)
    }

    pub(crate) fn query_session_plug_by_owner(
        &self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> Option<&CPlug> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .find_map(|plug_id| {
                self.plugs
                    .get(plug_id)
                    .filter(|plug| plug.has_owner(owner_type, owner_id))
            })
    }

    pub(crate) fn end_session(&mut self, session_id: i32) -> Option<SessionEndReport> {
        let callback_plug_ids = self.sessions.get_mut(&session_id)?.end();
        let callback_plug_ids = callback_plug_ids
            .into_iter()
            .filter(|plug_id| self.plugs.contains_key(plug_id))
            .collect();
        let session = self
            .sessions
            .get(&session_id)
            .expect("ended session остаётся в registry до factory GC");
        Some(SessionEndReport {
            session_id,
            ended: session.is_ended(),
            remove_requested: session.remove_requested(),
            callback_plug_ids,
        })
    }

    pub(crate) fn exit_plug(&mut self, session_id: i32, plug_id: i32) -> PlugExitReport {
        let session_found = self.sessions.contains_key(&session_id);
        let exited = if session_found {
            self.plugs
                .get_mut(&plug_id)
                .filter(|plug| plug.session_id() == session_id)
                .map(|plug| {
                    plug.mark_ended();
                    true
                })
                .unwrap_or(false)
        } else {
            false
        };
        PlugExitReport {
            session_id,
            plug_id,
            session_found,
            exited,
        }
    }

    pub(crate) fn register_equipment_compose_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentCompose,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_compose_plug(&self, plug_id: i32) -> Option<&CEquipmentCompose> {
        self.equipment_compose_plugs.get(&plug_id)
    }

    pub(crate) fn query_equipment_compose_plug_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CEquipmentCompose> {
        self.equipment_compose_plugs.get_mut(&plug_id)
    }

    pub(crate) fn take_equipment_compose_plug(
        &mut self,
        plug_id: i32,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.remove(&plug_id)
    }

    pub(crate) fn register_equipment_da_kong_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentDaKong,
    ) -> Option<CEquipmentDaKong> {
        self.equipment_da_kong_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_da_kong_plug(&self, plug_id: i32) -> Option<&CEquipmentDaKong> {
        self.equipment_da_kong_plugs.get(&plug_id)
    }

    pub(crate) fn query_equipment_da_kong_plug_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CEquipmentDaKong> {
        self.equipment_da_kong_plugs.get_mut(&plug_id)
    }

    pub(crate) fn take_equipment_da_kong_plug(&mut self, plug_id: i32) -> Option<CEquipmentDaKong> {
        self.equipment_da_kong_plugs.remove(&plug_id)
    }

    pub(crate) fn register_equipment_upgrade_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentUpgrade,
    ) -> Option<CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_upgrade_plug(&self, plug_id: i32) -> Option<&CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.get(&plug_id)
    }

    pub(crate) fn take_equipment_upgrade_plug(
        &mut self,
        plug_id: i32,
    ) -> Option<CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.remove(&plug_id)
    }

    /// Session-GC удаляет саму session и все её plug identities из base и
    /// concrete registries. Возвращаемый порядок совпадает с `m_lPlugs`.
    pub(crate) fn garbage_collect_session(&mut self, session_id: i32) -> Vec<i32> {
        let Some(session) = self.sessions.remove(&session_id) else {
            return Vec::new();
        };
        let plug_ids = session.plug_ids_storage().to_vec();
        for plug_id in &plug_ids {
            self.plugs.remove(plug_id);
            self.equipment_compose_plugs.remove(plug_id);
            self.equipment_da_kong_plugs.remove(plug_id);
            self.equipment_upgrade_plugs.remove(plug_id);
            self.personal_shop_seller_plugs.remove(plug_id);
            self.personal_shop_buyer_plugs.remove(plug_id);
            self.trader_plugs.remove(plug_id);
        }
        plug_ids
    }

    pub(crate) fn garbage_collect_terminal_equipment_sessions(
        &mut self,
    ) -> Vec<TerminalEquipmentSessionCollected> {
        let session_ids: Vec<i32> = self
            .sessions
            .iter()
            .filter_map(|(session_id, session)| {
                let has_equipment_plug = session.plug_ids_storage().iter().any(|plug_id| {
                    self.equipment_compose_plugs.contains_key(plug_id)
                        || self.equipment_da_kong_plugs.contains_key(plug_id)
                        || self.equipment_upgrade_plugs.contains_key(plug_id)
                });
                (session.remove_requested() && has_equipment_plug).then_some(*session_id)
            })
            .collect();
        session_ids
            .into_iter()
            .map(|session_id| {
                let plugs = self
                    .sessions
                    .get(&session_id)
                    .expect("terminal session выбрана из registry")
                    .plug_ids_storage()
                    .iter()
                    .filter_map(|plug_id| {
                        self.plugs
                            .get(plug_id)
                            .map(|plug| TerminalEquipmentPlugCollected {
                                plug_id: *plug_id,
                                owner_id: plug.owner_id(),
                            })
                    })
                    .collect();
                let _plug_ids = self.garbage_collect_session(session_id);
                TerminalEquipmentSessionCollected { session_id, plugs }
            })
            .collect()
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp

// ============================================================================
// FUNCTION: CSessionFactory::query_session_by_owner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:81
// RVA: 0x00078060
// ADDRESS: 00478060
// PROTOTYPE: CSession * __cdecl query_session_by_owner(OBJECT_TYPE param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `QuerySession` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: owned map removal и ordered concrete plug cleanup выполняет
// `garbage_collect_session`; MSVC hash/destructor RAW удалён.
// ============================================================================
// FUNCTION: CSessionFactory::InsertPlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:104
// RVA: 0x000781C0
// ADDRESS: 004781c0
// PROTOTYPE: int __cdecl InsertPlug(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// MATERIALIZED: live equipment-session registry sweep выполняется на обеих
// MainLoop session-stage. Неподключённая `CTeam::s_mQuestedTeams` retry-queue
// (`0x60008`, 60 секунд) остаётся конкретной границей будущего team owner-а.
// ============================================================================
// FUNCTION: CSessionFactory::CreateSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:37
// RVA: 0x00078DA0
// ADDRESS: 00478da0
// PROTOTYPE: long __cdecl CreateSession(ulong param_1, ulong param_2, ulong param_3, SESSION_TYPE param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::CreatePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:244
// RVA: 0x00078EC0
// ADDRESS: 00478ec0
// PROTOTYPE: long __cdecl CreatePlug(PLUG_TYPE param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:297
// RVA: 0x000790D0
// ADDRESS: 004790d0
// PROTOTYPE: long __cdecl UnserializePlug(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializeSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:324
// RVA: 0x000791B0
// ADDRESS: 004791b0
// PROTOTYPE: long __cdecl UnserializeSession(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
