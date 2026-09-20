//! Владелец `CWorldVillageRegion` исторического WorldServer — часть контракта owner-а.
//!
//! Constructor создаёт ровно один `CWorldWarRegion` и задаёт
//! его три DWORD `1/1/1`. Virtual `Load` намеренно вызывает
//! непосредственно `CWorldRegion::Load`, не читает `.war`, игнорирует его
//! legacy `0/1` result и всегда возвращает `1`; safe parser-блоки при этом не
//! превращаются в успех. Serializer наследуется от `CWorldWarRegion` и потому
//! дописывает `1/1/1`. Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
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
    pub(crate) fn load_from_context<Context, ResolveName>(
        &mut self,
        context: &mut Context,
        resolve_name: &mut ResolveName,
    ) -> Result<WorldRegionLoadedCounts, WorldRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
        ResolveName: FnMut(&[u8]) -> Vec<u8> + ?Sized,
    {
        self.war.base_mut().load_from_context(context, resolve_name)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldWarRegionSerializationBlock> {
        self.war.add_to_byte_array(destination, include_child)
    }
}
