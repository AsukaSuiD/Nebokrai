//! Владелец `CWorldVillageRegion` исторического WorldServer — `IMPLEMENTED`.
//!
//! Constructor RVA `0x00079F50` создаёт ровно один `CWorldWarRegion` и задаёт
//! его три DWORD `1/1/1`. Virtual `Load` RVA `0x00079F90` намеренно вызывает
//! непосредственно `CWorldRegion::Load`, не читает `.war`, игнорирует его
//! legacy `0/1` result и всегда возвращает `1`; safe parser-блоки при этом не
//! превращаются в успех. Serializer наследуется от `CWorldWarRegion` и потому
//! дописывает `1/1/1`. Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! source `worldvillageregion.cpp:22,35`. Destructors/compiler cleanup заменены
//! обычным `Drop`; Rust layout не объявляется старым ABI.

use super::worldregion::{
    WorldRegionLoadError, WorldRegionLoadedCounts, WorldRegionResourceContext,
};
use super::worldwarregion::{CWorldWarRegion, WorldWarRegionSerializationBlock};

pub(crate) struct CWorldVillageRegion {
    war: CWorldWarRegion,
}

impl CWorldVillageRegion {
    pub(crate) const fn with_constructor_state() -> Self {
        let mut war = CWorldWarRegion::with_constructor_base();
        war.set_constructor_symbols(1, 1, 1);
        Self { war }
    }

    pub(crate) const fn war(&self) -> &CWorldWarRegion {
        &self.war
    }

    pub(crate) const fn war_mut(&mut self) -> &mut CWorldWarRegion {
        &mut self.war
    }

    /// Выполняет прямой base Load; caller для Village не проверяет
    /// `loaded.base_failure`, воспроизводя unconditional legacy success.
    pub(crate) fn load_from_context<Context>(
        &mut self,
        context: &mut Context,
    ) -> Result<WorldRegionLoadedCounts, WorldRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
    {
        self.war.base_mut().load_from_context(context)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldWarRegionSerializationBlock> {
        self.war.add_to_byte_array(destination, include_child)
    }
}
