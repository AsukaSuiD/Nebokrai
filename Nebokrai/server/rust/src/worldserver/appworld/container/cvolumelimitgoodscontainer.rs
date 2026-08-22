//! Владелец volume-limited goods-container исторического `WorldServer`.
//!
//! Статус `CSerializeContainer::OnTraversingContainer` RVA `0x000DA6B0`,
//! `Remove` wrapper-ы RVA `0x000DA650/0x000DA660`,
//! три inherited `Find` wrapper-а RVA `0x000DA670/0x000DA680/0x000DA690`,
//! `QueryGoodsPosition(CGoods*)/Serialize/GetGoods` RVA
//! `0x000DA7B0/0x000DA7D0/0x000DA8A0`,
//! GUID-`Remove` RVA `0x000DA910`,
//! `IsFull/QueryGoodsPosition(CGUID)/FindPositionForGoods/Add/GetGoodsAmount`
//! RVA `0x000DA9A0/0x000DA9F0/0x000DAA50/0x000DAB30/0x000DAB90`,
//! `AddFromDB` RVA `0x000DBAE0`,
//! empty-cell branch positional `Add` RVA
//! `0x000DB980`, а также его occupied-cell stacking-ветка через base Add RVA
//! `0x000E07F0`,
//! `Release/Clear` RVA `0x000DAE10/0x000DB5B0`, constructor/destructor RVA
//! `0x000DB4A0/0x000DAF40` и оба `SetContainerVolume` RVA
//! `0x000DB500/0x000DB560`, `Clone` RVA `0x000DB600` — `IMPLEMENTED`;
//! остальные операции ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). `Unserialize` RVA `0x000D8DA0` находится у точного PDB-
//! владельца `cequipmentcontainer.cpp` и реализуется в соседнем экспортированном
//! `.rs`. Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp`.
//!
//! Exact PDB задаёт размер `0x74`: готовый `CAmountLimitGoodsContainer` prefix
//! `0x60`, unsigned `_size` по `+0x60` и `std::vector<CGUID> m_vCells` по
//! `+0x64`. Constructor ставит size `0` и пустой vector. Оба volume-setter-а
//! сначала вызывают virtual `Release`, затем задают size (двухаргументная форма
//! использует 32-битное wrapping multiplication), заполняют cells нулевым GUID
//! и приравнивают amount-limit к size. `Clear` сохраняет size, очищает amount-
//! owner и заново создаёт столько же пустых cells; `Release` сбрасывает всё и
//! size `0`.
//!
//! Positional `Add` сначала проверяет amount-full и существующий товар в cell.
//! Для пустой cell восстановлены границы, non-null base-properties, перенос
//! единственного `Box<CGoods>` в amount-owner и запись GUID. Занятая cell
//! выполняет exact base stacking: base index, particular attribute, stacking
//! limit и unsigned capacity проверяются в исходном порядке. False возвращает
//! rejected `Box` Rust-вызывающему; decoder, который исходно игнорировал bool,
//! безопасно уничтожает его обычным `Drop`. Успех меняет amount существующего
//! товара и уничтожает incoming. Автоматический
//! `Add(CBaseObject*)` теперь использует достигнутый поиск stack/empty позиции.
//!
//! Serialize helper пишет для каждого valid товара его первый cell index и
//! полный `CGoods` с literal `include_child=true`; входной `param_2` не
//! используется. Count включает только записи с non-null base-properties и
//! найденным cell. Старый temporary `CSerializeContainer` и traversal callback
//! выражены прямым read-only обходом: callbacks добавления/удаления amount-
//! owner-а уже доказанно no-op, а `&self` исключает mutation между count и
//! records. Map traversal технически заменён готовым `BTreeMap`, cell-order и
//! все wire scalar остаются unsigned little-endian.
//!
//! Cell-aware `GetGoods` читает GUID только в пределах vector, отбрасывает
//! `GUID_INVALID` и делегирует inherited `Find`, поэтому locked товар остаётся
//! занятым в cell, но не выдаётся вызывающему. Object-overload position-query
//! не ищет identity: exact EXE берёт GUID из `CGoods + 0x0c` и вызывает GUID-
//! overload. Автоматический `Add` сохраняет исходный двухфазный выбор: сначала
//! первый map-order стек с тем же base-index и unsigned вместимостью по
//! stacking-limit входящего товара, затем первый пустой GUID cell. Lock и
//! particular-attribute на этой стадии намеренно не фильтруются; окончательная
//! stacking-проверка остаётся в positional base Add. Process-global factory
//! технически заменена явным read-only `GoodsBasePropertiesRegistry`.
//! Текущий C++ reference сводил automatic Add к `FindFreePosition`; архивный
//! Linux-донор сохранил stack-first форму, а перечисленный порядок и отсутствие
//! дополнительных фильтров подтверждены exact `Nworldserver.exe`.
//! При нескольких подходящих стеках Rust сохраняет уже принятую `BTreeMap`-
//! модель owner-а; bucket-order старого `stdext::_Hash` отдельно не восстановлен
//! и остаётся явным неизвестным порядка выбора, а не скрытой гарантией exact.
//!
//! GUID-`Remove` сначала выполняет locked-aware removal amount-owner-а и лишь
//! затем проверяет base-properties, ищет первый GUID cell и обнуляет его. Это
//! отличается и от нового C++ reference, очищающего cell до base removal, и от
//! Linux-донора, ищущего cell первым. При post-remove отказе legacy pointer уже
//! терялся; Rust исправляет этот внутренний lifetime-дефект обычным `Drop`, не
//! меняя уже совершённое удаление или возвращаемый результат. Контекст и RTTI
//! удалены только потому, что единственные достигнутые callbacks no-op, а owner
//! typed как `CGoods`.
//!
//! `AddFromDB` намеренно не делегирует обычному Add: сначала cell-`GetGoods`
//! отличает unlocked collision для legacy debug-log, затем проверяются non-null
//! incoming, `IsSpaceEnough` и factory lookup; amount-full и stacking здесь не
//! участвуют. Успех напрямую вставляет GUID owner и заполняет cell без listener-
//! callback. Rust возвращает rejected `Box` вместо сырого false, сохраняет
//! duplicate overwrite с безопасным уничтожением вытесненного товара и не
//! материализует технический `debug-DB` file sink.
//!
//! `Clone` сначала выполняет достигнутый amount deep-copy с сохранением target
//! owner/locked, затем присваивает size и cells. Так исправляется только
//! внутренний shallow-pointer lifetime-дефект; World-набор полей и порядок
//! side effects остаются exact. `AI` наследует недостигнутый child-graph
//! `CBaseObject` и остаётся RAW.
//!
//! Короткий source в соседнем decoder-е сохраняет ранний `Clear`, cursor и уже
//! добавленные записи, затем возвращает типизированную ошибку вместо legacy
//! overread.
//!
//! `CPlayer::CheckGoodsInPacket` вызывает у packet-а унаследованные vtable
//! slots `Find(long, GUID)` и `TraversingContainer`. Узкие Rust wrappers ниже
//! только делегируют достигнутому `CAmountLimitGoodsContainer`: собственную
//! volume-политику или новый порядок обхода они не вводят.

use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::listener::ccontainerlistener::CContainerListener;

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, query_goods_base_properties,
};
use super::camountlimitgoodscontainer::{AmountContainerCodecError, CAmountLimitGoodsContainer};
use super::cgoodscontainer::add_to_occupied_position;

/// Ошибка безопасной границы volume-container codec-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VolumeContainerCodecError {
    Amount(AmountContainerCodecError),
    Goods(GoodsCodecError),
    ValidGoodsCountOutsideLegacyRange { count: usize },
}

impl fmt::Display for VolumeContainerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Amount(error) => error.fmt(formatter),
            Self::Goods(error) => error.fmt(formatter),
            Self::ValidGoodsCountOutsideLegacyRange { count } => write!(
                formatter,
                "volume-container содержит {count} валидных cells вне 32-битного legacy-диапазона"
            ),
        }
    }
}

impl Error for VolumeContainerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Amount(error) => Some(error),
            Self::Goods(error) => Some(error),
            Self::ValidGoodsCountOutsideLegacyRange { .. } => None,
        }
    }
}

impl From<AmountContainerCodecError> for VolumeContainerCodecError {
    fn from(error: AmountContainerCodecError) -> Self {
        Self::Amount(error)
    }
}

impl From<GoodsCodecError> for VolumeContainerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

/// Достигнутая owning-часть `CVolumeLimitGoodsContainer`.
pub(crate) struct CVolumeLimitGoodsContainer {
    amount_base: CAmountLimitGoodsContainer,
    size: u32,
    cells: Vec<CGuid>,
}

impl CVolumeLimitGoodsContainer {
    /// Создаёт exact constructor defaults поверх готового amount-owner-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            amount_base: CAmountLimitGoodsContainer::with_constructor_defaults(),
            size: 0,
            cells: Vec::new(),
        }
    }

    /// Сбрасывает owner и задаёт volume как 32-битное произведение сторон.
    pub(crate) fn set_container_volume_2d(&mut self, width: u32, height: u32) {
        self.set_container_volume(width.wrapping_mul(height));
    }

    /// Сбрасывает owner и задаёт точное unsigned число cells.
    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.release();
        self.size = size;
        self.cells.resize(size as usize, CGuid::GUID_INVALID);
        self.amount_base.set_goods_amount_limit(size);
    }

    /// Возвращает exact inherited amount-limit, которым largess обходит cells.
    pub(crate) const fn get_goods_amount_limit(&self) -> u32 {
        self.amount_base.get_goods_amount_limit()
    }

    /// Очищает logical contents, сохраняя size и заново обнуляя cells.
    pub(crate) fn clear(&mut self) {
        self.amount_base.clear();
        self.cells.clear();
        self.cells.resize(self.size as usize, CGuid::GUID_INVALID);
    }

    /// Сбрасывает base-owner, size и cell storage.
    pub(crate) fn release(&mut self) {
        self.amount_base.release();
        self.size = 0;
        self.cells.clear();
    }

    /// Exact `AI` является только inherited dispatch amount-owner-а.
    pub(crate) fn ai(&mut self, on_goods_ai: impl FnMut(&mut CGoods)) {
        self.amount_base.ai(on_goods_ai);
    }

    /// Возвращает первый cell с exact GUID, как исходный linear scan.
    pub(crate) fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| cell == ex_id)
            .and_then(|position| u32::try_from(position).ok())
    }

    /// Делегирует exact inherited `Find(long, GUID)` typed amount-owner-у.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.amount_base.find(ex_id)
    }

    /// Возвращает товар cell-а через inherited locked-aware `Find`.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        let ex_id = self.cells.get(position as usize)?;
        if ex_id.is_invalid() {
            return None;
        }
        self.find(ex_id)
    }

    /// Возвращает mutable DB-view товара exact cell-а.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let ex_id = *self.get_goods(position)?.get_ex_id();
        self.amount_base.goods_mut(&ex_id)
    }

    /// Берёт GUID non-null объекта и делегирует GUID-overload position-query.
    pub(crate) fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        self.query_goods_position(goods?.get_ex_id())
    }

    /// Делегирует exact inherited traversal без собственной volume-фильтрации.
    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        self.amount_base.traversing_container(listener);
    }

    /// Замораживает concrete traversal и младший байт его cell-position.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.amount_base
            .goods()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: self
                        .query_goods_position_by_object(Some(goods))
                        .unwrap_or(0) as u8,
                })
            })
            .collect()
    }

    /// Проверяет обе исходные unsigned границы и нулевой GUID cell-а.
    pub(crate) fn is_space_enough(&self, position: u32) -> bool {
        position < self.size
            && self
                .cells
                .get(position as usize)
                .is_some_and(|cell| cell.is_invalid())
    }

    /// Считает valid base-properties товары, реально связанные с cell.
    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, VolumeContainerCodecError> {
        let mut count = 0usize;
        for goods in self.amount_base.goods() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some()
                && self.query_goods_position_by_object(Some(goods)).is_some()
            {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| VolumeContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    /// Сохраняет двухступенчатую amount/cell проверку `IsFull`.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        if self.amount_base.is_full(registry)? {
            return Ok(true);
        }
        Ok(!self.cells.iter().any(|cell| cell.is_invalid()))
    }

    /// Ищет первый stack-candidate в map-order, затем первый пустой cell.
    pub(crate) fn find_position_for_goods(
        &self,
        goods: Option<&CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<u32>, GoodsCodecError> {
        let Some(goods) = goods else {
            return Ok(None);
        };

        if goods.get_max_stack_number(registry)? > 1 {
            for existing in self.amount_base.goods() {
                if existing.get_base_properties_index() == goods.get_base_properties_index()
                    && goods.get_amount().wrapping_add(existing.get_amount())
                        <= goods.get_max_stack_number(registry)?
                    && let Some(position) = self.query_goods_position_by_object(Some(existing))
                {
                    return Ok(Some(position));
                }
            }
        }

        Ok(self
            .cells
            .iter()
            .position(|cell| cell.is_invalid())
            .and_then(|position| u32::try_from(position).ok()))
    }

    /// Выбирает exact stack/empty позицию и делегирует positional `Add`.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        let Some(position) = self.find_position_for_goods(Some(goods.as_ref()), registry)? else {
            return Ok(Some(goods));
        };
        self.add_at(position, goods, registry)
    }

    /// Вынимает unlocked товар и очищает cell только после factory-validation.
    pub(crate) fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        let Some(goods) = self.amount_base.remove(ex_id) else {
            return Ok(None);
        };

        let Some(index) = goods.get_base_properties_index() else {
            return Err(GoodsCodecError::MissingBasePropertiesIndex.into());
        };
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(None);
        }

        let Some(position) = self.query_goods_position(ex_id) else {
            return Ok(None);
        };
        self.cells[position as usize] = CGuid::GUID_INVALID;
        Ok(Some(goods))
    }

    /// Вставляет DB-товар напрямую в доказанно пустую cell без full/stacking.
    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.get_goods(position).is_some() || !self.is_space_enough(position) {
            return Ok(Some(goods));
        }

        let Some(index) = goods.get_base_properties_index() else {
            return Err(GoodsCodecError::MissingBasePropertiesIndex.into());
        };
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(Some(goods));
        }

        let ex_id = *goods.get_ex_id();
        self.amount_base.insert_unchecked(goods);
        self.cells[position as usize] = ex_id;
        Ok(None)
    }

    /// Вставляет товар в пустую cell; `Some` возвращает ownership при false.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.amount_base.is_full(registry)? || position >= self.size {
            return Ok(Some(goods));
        }
        let Some(&cell) = self.cells.get(position as usize) else {
            return Ok(Some(goods));
        };
        if !cell.is_invalid() {
            let Some(existing) = self.amount_base.goods_mut(&cell) else {
                return Ok(Some(goods));
            };
            return add_to_occupied_position(existing, goods, registry).map_err(Into::into);
        }
        let index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(Some(goods));
        }
        let ex_id = *goods.get_ex_id();
        if let Some(rejected) = self.amount_base.add(goods, registry)? {
            return Ok(Some(rejected));
        }
        self.cells[position as usize] = ex_id;
        Ok(None)
    }

    /// Клонирует amount-owner первым, затем exact volume/cell state.
    pub(crate) fn clone_into(
        &self,
        target: &mut CVolumeLimitGoodsContainer,
    ) -> Result<bool, GoodsCodecError> {
        let _ = self.amount_base.clone_into(&mut target.amount_base)?;
        target.size = self.size;
        target.cells.clone_from(&self.cells);
        Ok(true)
    }

    /// Кодирует count, cell index и полный goods record.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let count = self.get_goods_amount(registry)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for goods in self.amount_base.goods() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some()
                && let Some(position) = self.query_goods_position_by_object(Some(goods))
            {
                destination.extend_from_slice(&position.to_le_bytes());
                let _ = goods.serialize(destination, true)?;
            }
        }
        Ok(true)
    }
}
