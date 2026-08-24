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
//! compose shadows с исходным player slot. Общие team/trader/shop варианты
//! `CreateSession/CreatePlug/InsertPlug` и их polymorphic lifecycle ниже этим
//! не объявляются реализованными.

use std::collections::BTreeMap;

use crate::gameserver::appserver::container::camountlimitgoodsshadowcontainer::AmountShadowAdded;
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeShadowAddBlock;
use crate::gameserver::appserver::container::cequipmentdakongcontainer::{
    DaKongAddBlock, DaKongAddOutcome,
};
use crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeShadowAddBlock;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::ShadowRemovedReport;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

use super::cequipmentcompose::CEquipmentCompose;
use super::cequipmentdakong::CEquipmentDaKong;
use super::cequipmentupgrade::CEquipmentUpgrade;
use super::cplug::CPlug;
use super::csession::CSession;

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

#[derive(Debug)]
pub(crate) struct CSessionFactory {
    sessions: BTreeMap<i32, CSession>,
    plugs: BTreeMap<i32, CPlug>,
    equipment_compose_plugs: BTreeMap<i32, CEquipmentCompose>,
    equipment_da_kong_plugs: BTreeMap<i32, CEquipmentDaKong>,
    equipment_upgrade_plugs: BTreeMap<i32, CEquipmentUpgrade>,
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
            next_session_id: 1,
            next_plug_id: 1,
        }
    }
}

impl CSessionFactory {
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
        }
        plug_ids
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

// ============================================================================
// FUNCTION: CSessionFactory::GarbageCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:200
// RVA: 0x000780F0
// ADDRESS: 004780f0
// PROTOTYPE: void __cdecl GarbageCollect(OBJECT_TYPE param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `QueryPlug` материализован выше; покрытый raw-блок удалён.

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

// ============================================================================
// FUNCTION: CSessionFactory::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:119
// RVA: 0x000785D0
// ADDRESS: 004785d0
// PROTOTYPE: void __cdecl AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
