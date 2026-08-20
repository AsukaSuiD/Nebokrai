//! Владелец fairy-container исторического `WorldServer`.
//!
//! Статус constructor/destructor-state RVA `0x000D7A70/0x000D7B20`,
//! `Clear/Release` folded RVA `0x000D7AA0/0x000D7AB0` и собственного
//! `Serialize/Unserialize` RVA `0x000D7AD0/0x000D7B80` — `IMPLEMENTED`;
//! игровые `Add/Find/Remove/AddFromDB` остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:14,22,50,89,102`.
//!
//! Exact PDB задаёт размер `0x88`: base `CVolumeLimitGoodsContainer` по `+0x0`
//! и массив пяти `unsigned long m_dwHatchTime` общей длиной `0x14` по `+0x74`.
//! Constructor создаёт base с volume `0` и пять нулей. `Clear`, `Release` и
//! `SetContainerVolume` меняют только base-state: hatch-time сохраняются.
//! Destructor освобождает base ownership; обычный Rust `Drop` заменяет
//! deleting-destructor, vtable/EH и STL cleanup.
//!
//! Fairy-wire сначала содержит полный унаследованный volume-wire. Encoder
//! затем в index-order дописывает пять little-endian `u32` и сразу после
//! каждой записи обнуляет соответствующий live hatch-time; возвращаемый bool
//! остаётся результатом base `Serialize`. Decoder сначала выполняет полный
//! base decode с его ранним virtual `Clear`, затем последовательно читает и
//! присваивает пять `u32`, возвращая base-result без изменения.
//!
//! Поэтому короткий source после успешного base-wire сохраняет очищенный и
//! частично декодированный container, cursor, уже назначенные ранние hatch-
//! time и старые значения ещё не достигнутого хвоста. Безразмерное legacy-
//! чтение выражено локальной typed `BLOCKED_MISSING_FACT`, а не дополнением
//! нулями. Volume `0x0E` задаёт будущий `CPlayer::DecordFromByteArray` отдельно
//! между `Release` и этим decoder-ом; constructor не получает его заранее.
//! Folded `Add/Find/Remove` имеют общие RVA с `CBattleFairyContainer`, но их
//! игровая семантика не нужна clone-codec-у и здесь не дублируется.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::camountlimitgoodscontainer::AmountContainerCodecError;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;

const HATCH_TIME_COUNT: usize = 5;

/// Достигнутое состояние исходного `CFairyContainer`, не копия его x86 ABI.
pub(crate) struct CFairyContainer {
    volume_state: CVolumeLimitGoodsContainer,
    hatch_times: [u32; HATCH_TIME_COUNT],
}

impl CFairyContainer {
    /// Создаёт base с нулевым volume и пять нулевых hatch-time.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
            hatch_times: [0; HATCH_TIME_COUNT],
        }
    }

    /// Сбрасывает только base-state и задаёт точное unsigned число cells.
    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.volume_state.set_container_volume(size);
    }

    /// Очищает товары и cells, сохраняя volume и все hatch-time.
    pub(crate) fn clear(&mut self) {
        self.volume_state.clear();
    }

    /// Сбрасывает inherited owner, товары и volume, сохраняя hatch-time.
    pub(crate) fn release(&mut self) {
        self.volume_state.release();
    }

    /// Замораживает inherited volume traversal, не сбрасывая hatch-time.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    /// Кодирует base volume-wire, затем пять hatch-time и обнуляет их по одному.
    pub(crate) fn serialize(
        &mut self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let base_result = self
            .volume_state
            .serialize(destination, include_child, registry)?;
        for hatch_time in &mut self.hatch_times {
            destination.extend_from_slice(&hatch_time.to_le_bytes());
            *hatch_time = 0;
        }
        Ok(base_result)
    }

    /// Декодирует base volume-wire, затем последовательно назначает hatch-time.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let base_result = self.volume_state.unserialize(source, cursor, registry)?;
        for index in 0..HATCH_TIME_COUNT {
            self.hatch_times[index] = read_hatch_time(source, cursor)?;
        }
        Ok(base_result)
    }
}

fn read_hatch_time(source: &[u8], cursor: &mut usize) -> Result<u32, VolumeContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field: "CFairyContainer hatch time",
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: legacy owner не получал длину source.
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field: "CFairyContainer hatch time",
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    *cursor = end;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("проверенный четырёхбайтовый диапазон hatch-time"),
    ))
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp

// ============================================================================
// FUNCTION: CFairyContainer::CFairyContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:14
// RVA: 0x000D7A70
// ADDRESS: 004d7a70
// PROTOTYPE: undefined __thiscall CFairyContainer(void)
//
// IMPLEMENTED выше; base volume равен нулю, пять hatch-time обнулены.

// ============================================================================
// FUNCTION: CFairyContainer::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:89
// RVA: 0x000D7AD0
// ADDRESS: 004d7ad0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; каждый hatch-time обнуляется сразу после записи.

// ============================================================================
// FUNCTION: CFairyContainer::~CFairyContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:22
// RVA: 0x000D7B20
// ADDRESS: 004d7b20
// PROTOTYPE: void __thiscall ~CFairyContainer(void)
//
// IMPLEMENTED обычным Rust ownership/Drop; vtable/EH cleanup удалён.

// ============================================================================
// FUNCTION: CFairyContainer::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:102
// RVA: 0x000D7B80
// ADDRESS: 004d7b80
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше; base-result сохраняется, пять DWORD читаются после base-wire.

// ============================================================================
// FUNCTION: CFairyContainer::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cfairycontainer.cpp:50
// RVA: 0x000D7C10
// ADDRESS: 004d7c10
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
