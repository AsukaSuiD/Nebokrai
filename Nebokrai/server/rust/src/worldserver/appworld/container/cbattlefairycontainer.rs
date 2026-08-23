//! Владелец battle-fairy-container исторического `WorldServer`.
//!
//! `Serialize/Unserialize` и folded
//! `Clear/Release`,
//! `Add/Add(position)/Find/Remove/AddFromDB` входят в контракт owner-а.
//! Источник контракта — WorldServer EXE/PDB.
//!
//! Layout сохраняет размер `0x74` и единственный base
//! `CVolumeLimitGoodsContainer` по `+0x0`; собственных data-полей нет.
//! Constructor создаёт base с volume `0`, destructor только уничтожает base.
//! Rust хранит отдельный nominal owner, а обычные ownership/`Drop` заменяют
//! vtable, secondary listener, STL и EH cleanup без копирования x86 layout.
//!
//! Собственные codec-функции являются точными тонкими вызовами volume-codec-а:
//! wire, `include_child`, bool-result, ранний `Clear`, cursor, безопасное
//! уничтожение rejected goods и типизированные границы короткого источника не меняются.
//! `Clear` всегда передаёт base literal `nullptr`, а `Release` прямо делегирует
//! base. Короткий source поэтому сохраняет очищенный container, прежний volume,
//! cursor и уже добавленные records по контракту volume-owner-а.
//!
//! Volume `0x11` задаёт связанный `CPlayer::DecordFromByteArray` отдельно между
//! virtual `Release` и decoder-ом; constructor не получает его заранее.
//! Совпадающие с `CFairyContainer` folded `Clear/Release` и игровые
//! `Add/Find/Remove` не означают объединения двух nominal классов. Все они
//! остаются thin volume-tail-calls без дополнительной type-policy.
//! `AddFromDB` выполняет derived collision lookup, затем base повторяет его и
//! делает direct insert; различие battle-owner-а ограничено string-table
//! `ZHGS0044` для технического `debug-DB` log, не меняющего state/return.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoods::CGoods;

/// Действующее состояние исходного `CBattleFairyContainer` без собственного payload.
pub(crate) struct CBattleFairyContainer {
    volume_state: CVolumeLimitGoodsContainer,
}

impl CBattleFairyContainer {
 /// Создаёт точный base-state с нулевым volume.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
        }
    }

 /// Сбрасывает owner и задаёт точное unsigned число cells.
    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.volume_state.set_container_volume(size);
    }

 /// Очищает товары и заново создаёт cells, сохраняя volume.
    pub(crate) fn clear(&mut self) {
        self.volume_state.clear();
    }

 /// Сбрасывает inherited owner, товары, volume и cells.
    pub(crate) fn release(&mut self) {
        self.volume_state.release();
    }

 /// Делегирует folded automatic volume Add.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add(goods, registry)
    }

 /// Делегирует folded positional volume Add.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add_at(position, goods, registry)
    }

 /// Делегирует folded locked-aware GUID lookup.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.volume_state.find(ex_id)
    }

 /// Возвращает mutable DB-view battle-fairy cell-а.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.volume_state.get_goods_mut(position)
    }

 /// Делегирует folded volume removal с post-remove checks.
    pub(crate) fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.remove(ex_id, registry)
    }

 /// Выполняет derived collision check до повторной base DB-проверки.
    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.volume_state.get_goods(position).is_some() {
            return Ok(Some(goods));
        }
        self.volume_state.add_from_db(position, goods, registry)
    }

 /// Замораживает inherited volume traversal без собственного payload.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

 /// Делегирует точный inherited volume-wire без собственного suffix-а.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state
            .serialize(destination, include_child, registry)
    }

 /// Делегирует volume-decoder с его ранним `Clear` и partial effects.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state.unserialize(source, cursor, registry)
    }
}
