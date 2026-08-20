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
//! empty-cell branch positional `Add` RVA
//! `0x000DB980`, а также его occupied-cell stacking-ветка через base Add RVA
//! `0x000E07F0`,
//! `Release/Clear` RVA `0x000DAE10/0x000DB5B0`, constructor/destructor RVA
//! `0x000DB4A0/0x000DAF40` и оба `SetContainerVolume` RVA
//! `0x000DB500/0x000DB560` — `IMPLEMENTED`; остальные операции ниже остаются
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
//! переносит его в уже готовый lifetime-quarantine base-owner-а. Успех меняет
//! amount существующего товара и уничтожает incoming. Автоматический
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
//! терялся; Rust оставляет такой `Box<CGoods>` в lifetime-quarantine, сохраняя
//! невидимость для вызывающего без преждевременного destructor-а. Контекст и
//! RTTI удалены только потому, что единственные достигнутые callbacks no-op, а
//! owner typed как `CGoods`.
//!
//! Короткий source в соседнем decoder-е сохраняет ранний `Clear`, cursor и уже
//! добавленные записи, затем возвращает typed `BLOCKED_MISSING_FACT` вместо
//! legacy overread.
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
            self.amount_base.retain_detached_goods(goods);
            return Err(GoodsCodecError::MissingBasePropertiesIndex.into());
        };
        if query_goods_base_properties(registry, index).is_none() {
            self.amount_base.retain_detached_goods(goods);
            return Ok(None);
        }

        let Some(position) = self.query_goods_position(ex_id) else {
            self.amount_base.retain_detached_goods(goods);
            return Ok(None);
        };
        self.cells[position as usize] = CGuid::GUID_INVALID;
        Ok(Some(goods))
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

    /// Удерживает rejected factory-result как потерянный legacy pointer.
    pub(super) fn retain_rejected_goods(&mut self, goods: Box<CGoods>) {
        self.amount_base.retain_detached_goods(goods);
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:339
// RVA: 0x000DA640
// ADDRESS: 004da640
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:419
// RVA: 0x000DA650
// ADDRESS: 004da650
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Exact inherited tail-chain проверяет null, берёт GUID по `+0x0c` и вызывает
// GUID-slot; typed Rust boundary передаёт GUID напрямую в `remove`.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:424
// RVA: 0x000DA660
// ADDRESS: 004da660
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// Exact inherited tail-chain игнорирует type scalar и делегирует GUID-slot;
// Rust не носит рядом с typed GUID неиспользуемый `long`.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:429
// RVA: 0x000DA670
// ADDRESS: 004da670
// PROTOTYPE: CBaseObject * __thiscall Find(CBaseObject * param_1)
//
// Exact tail-chain `0x004DA670 -> 0x004DBD40 -> 0x004E0650 -> 0x004E0A40`
// только проверяет null, берёт GUID объекта по `+0x0c` и вызывает GUID-slot.
// Typed Rust call sites передают уже извлечённый `CGuid` в `find`.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:434
// RVA: 0x000DA680
// ADDRESS: 004da680
// PROTOTYPE: CBaseObject * __thiscall Find(long param_1, CGUID * param_2)
//
// Exact tail-chain `0x004DA680 -> 0x004D5F50` игнорирует type scalar и
// достигает того же GUID lookup; `find` выражает его без лишнего параметра.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:439
// RVA: 0x000DA690
// ADDRESS: 004da690
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Exact `0x004DA690` tail-jump-ит в достигнутый amount-owner GUID lookup
// `0x004DC180`; Rust `find` делегирует ему и сохраняет locked-фильтр.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CSerializeContainer::~CSerializeContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:498
// RVA: 0x000DA6A0
// ADDRESS: 004da6a0
// PROTOTYPE: void __thiscall ~CSerializeContainer(void)
//
// Temporary listener заменён lexical borrow внутри `serialize`; отдельного
// destructor-а или stream pointer нет.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CSerializeContainer::OnTraversingContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:503
// RVA: 0x000DA6B0
// ADDRESS: 004da6b0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// IMPLEMENTED внутри прямого typed traversal `serialize`: factory lookup,
// first cell, `include_child=true` и valid-count сохранены.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:157
// RVA: 0x000DA7B0
// ADDRESS: 004da7b0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// IMPLEMENTED выше как `query_goods_position_by_object`; null возвращает
// `None`, а non-null путь берёт exact GUID и делегирует GUID-overload.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:445
// RVA: 0x000DA7D0
// ADDRESS: 004da7d0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; immutable Rust borrow делает исходную count/traversal
// equality обязательной без отдельного mutable callback-object.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::IsSpaceEnough
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:95
// RVA: 0x000DA850
// ADDRESS: 004da850
// PROTOTYPE: int __thiscall IsSpaceEnough(ulong param_1)
//
// IMPLEMENTED выше; проверяются vector length, `_size` и точный нулевой GUID.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:140
// RVA: 0x000DA8A0
// ADDRESS: 004da8a0
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// IMPLEMENTED выше; vector-bound и `GUID_INVALID` проверяются до inherited
// locked-aware `Find`. Typed owner устраняет доказанно избыточный RTTI cast.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:227
// RVA: 0x000DA910
// ADDRESS: 004da910
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// IMPLEMENTED выше; exact порядок amount removal -> typed result -> factory
// lookup -> first cell query -> `GUID_INVALID` сохранён. Post-remove false
// удерживает потерянное ownership в quarantine и не очищает cell.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::IsFull
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:347
// RVA: 0x000DA9A0
// ADDRESS: 004da9a0
// PROTOTYPE: int __thiscall IsFull(void)
//
// IMPLEMENTED выше; amount-full проверяется до linear empty-cell scan.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:368
// RVA: 0x000DA9F0
// ADDRESS: 004da9f0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// IMPLEMENTED выше; Rust `Option<u32>` заменяет bool плюс out-параметр.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::FindPositionForGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:55
// RVA: 0x000DAA50
// ADDRESS: 004daa50
// PROTOTYPE: int __thiscall FindPositionForGoods(CGoods * param_1, ulong * param_2)
//
// IMPLEMENTED выше; exact map-order stack scan использует base-index,
// wrapping unsigned amount-sum и stacking-limit входящего товара, затем
// выполняется linear поиск первого `GUID_INVALID` во всём cell-vector.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:167
// RVA: 0x000DAB30
// ADDRESS: 004dab30
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// IMPLEMENTED выше; typed `Box<CGoods>` заменяет RTTI boundary, ownership при
// false возвращается как `Some`, а найденная позиция передаётся `add_at`.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetGoodsAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:383
// RVA: 0x000DAB90
// ADDRESS: 004dab90
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// IMPLEMENTED выше; factory-valid товар считается только при найденном cell.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:197
// RVA: 0x000DAE10
// ADDRESS: 004dae10
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; base-owner очищается до сброса size и cell storage.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::~CVolumeLimitGoodsContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:27
// RVA: 0x000DAF40
// ADDRESS: 004daf40
// PROTOTYPE: void __thiscall ~CVolumeLimitGoodsContainer(void)
//
// Rust field drop освобождает cells и готовый amount-owner; vtable/EH cleanup
// не имеет отдельного наблюдаемого контракта.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CVolumeLimitGoodsContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:20
// RVA: 0x000DB4A0
// ADDRESS: 004db4a0
// PROTOTYPE: undefined __thiscall CVolumeLimitGoodsContainer(void)
//
// IMPLEMENTED выше как `with_constructor_defaults`: готовый amount-prefix,
// size `0` и пустой cells vector; vtable/EH bookkeeping отброшен как noise.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetContainerVolume
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:34
// RVA: 0x000DB500
// ADDRESS: 004db500
// PROTOTYPE: void __thiscall SetContainerVolume(ulong param_1, ulong param_2)
//
// IMPLEMENTED выше: virtual Release предшествует wrapping `width * height`,
// заполнению invalid GUID и синхронизации amount-limit.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetContainerVolume
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:44
// RVA: 0x000DB560
// ADDRESS: 004db560
// PROTOTYPE: void __thiscall SetContainerVolume(ulong param_1)
//
// IMPLEMENTED выше: Release, exact unsigned size, invalid GUID cells и
// одинаковый amount-limit сохраняют исходный порядок эффектов.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:187
// RVA: 0x000DB5B0
// ADDRESS: 004db5b0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; base Clear выполняется первым, size сохраняется, cells
// полностью пересоздаются с invalid GUID.

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:207
// RVA: 0x000DB600
// ADDRESS: 004db600
// PROTOTYPE: int __thiscall Clone(CGoodsContainer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:255
// RVA: 0x000DB980
// ADDRESS: 004db980
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// IMPLEMENTED выше как `add_at`: пустая cell вставляет новый owner, занятая
// вызывает восстановленный base stacking-owner.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp:297
// RVA: 0x000DBAE0
// ADDRESS: 004dbae0
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535390
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp
// RVA: 0x00135390
// ADDRESS: 00535390
// PROTOTYPE: undefined Unwind@00535390()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053539b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cvolumelimitgoodscontainer.cpp
// RVA: 0x0013539B
// ADDRESS: 0053539b
// PROTOTYPE: undefined Unwind@0053539b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
