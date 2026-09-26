//! Базовый derived lifecycle `CGoodsContainer` исторического GameServer,
//! перенесённый в Zone `items/` — владельца типов контейнеров и операций над
//! ними.
//!
//! Тела перенесены буквально из прежнего
//! `src/gameserver/appserver/container/cgoodscontainer.rs` (волна Z-C1); отличия —
//! нормализация `pub(crate)`→`pub` на границе crate и швы переноса (не
//! расхождения): `CContainer` — Zone `items/ccontainer.rs`, `CGoods` — Zone
//! `items/cgoods.rs`, GAP-константа — Zone `content/goods.rs`, реестр
//! `CGoodsFactory` — Zone `content/goodsfactory.rs` (волна Z-G0b), `ShapeIdentity` —
//! Zone `regions/` (re-export `identity`), `CGuid` и wire-кодеки — Shared.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/container/cgoodscontainer.cpp`. Constructor,
//! destructor и `Release` RVA `0x001DB840/0x001DB860/0x001DB8C0` сохраняют
//! owner type/id и mode в нуле, причём `Release` также освобождает listener
//! vector base `CContainer`. `Set/GetOwner` и `Set/GetContainerMode` перенесены
//! буквально. PDB подтверждает, что ошибочно названный декомпилятором адрес
//! `0x001DB8F0` — именно `CGoodsContainer::SetContainerMode`.
//!
//! Default `Add(CBaseObject*)` и `Clear` являются намеренными no-op virtual
//! slots. Четыре `Find/Remove` overload-а были переходами в `CContainer` и
//! поглощены typed API concrete container-ов. Общая stack-ветка `Add(position,
//! CGoods*)` живёт здесь; storage lookup и listener reports принадлежат derived
//! owners. `Remove(position, amount)` материализован в amount-limit и wallet
//! owners: exact full/split правила, addon copy, mutation order и callbacks
//! сохранены их typed reports. Случайно попавший сюда `tagGoodsShadow::operator=`
//! заменён `Copy`/присваиванием у его настоящего owner-а.

use super::ccontainer::CContainer;
use super::cgoods::CGoods;
use crate::content::goods::GAP_PARTICULAR_ATTRIBUTE;
use crate::content::goodsfactory::CGoodsFactory;
use crate::regions::ShapeIdentity;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GoodsContainerMode {
    #[default]
    Normal,
    Test,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CGoodsContainer {
    base: CContainer,
    owner_type: i32,
    owner_id: i32,
    mode: GoodsContainerMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsStackMergeOutcome {
    BlockedByOwnerProgress,
    Incompatible,
    Merged { target: ShapeIdentity, amount: u32 },
}

impl CGoodsContainer {
    pub const fn new() -> Self {
        Self {
            base: CContainer::new(),
            owner_type: 0,
            owner_id: 0,
            mode: GoodsContainerMode::Normal,
        }
    }

    pub const fn base(&self) -> &CContainer {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CContainer {
        &mut self.base
    }

    pub const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub const fn container_mode(&self) -> GoodsContainerMode {
        self.mode
    }

    pub const fn set_container_mode(&mut self, mode: GoodsContainerMode) {
        self.mode = mode;
    }

    pub fn release(&mut self) {
        self.owner_type = 0;
        self.owner_id = 0;
        self.mode = GoodsContainerMode::Normal;
        self.base.release();
    }

    /// Общая stack-ветка exact `Add(position, CGoods*)` RVA `0x001DB970`.
    /// Проверка player progress выполнялась перед stack compatibility и
    /// передаётся уже вычисленным owner policy. Derived owner после `Merged`
    /// публикует snapshot своих listener-ов с target и присоединённым amount.
    pub fn merge_stack(
        &self,
        target: &mut CGoods,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> GoodsStackMergeOutcome {
        let Some(source) = incoming.as_ref() else {
            return GoodsStackMergeOutcome::Incompatible;
        };
        if !owner_progress_allows {
            return GoodsStackMergeOutcome::BlockedByOwnerProgress;
        }
        if target.base_properties_index() != source.base_properties_index() {
            return GoodsStackMergeOutcome::Incompatible;
        }
        let maximum = target.max_stack_number(factory);
        if maximum <= 1
            || target.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
                != source.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
        {
            return GoodsStackMergeOutcome::Incompatible;
        }
        let source_amount = source.amount();
        if maximum.wrapping_sub(target.amount()) < source_amount {
            return GoodsStackMergeOutcome::Incompatible;
        }
        if self.mode == GoodsContainerMode::Normal {
            target.set_amount(target.amount().wrapping_add(source_amount));
            let _garbage_collected = incoming.take();
        }
        GoodsStackMergeOutcome::Merged {
            target: target.identity(),
            amount: source_amount,
        }
    }
}
