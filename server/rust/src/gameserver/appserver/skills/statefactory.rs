//! Каталог записей состояний GameSave перенесён в Zone `skills::statefactory`,
//! включая привязку записей к единому enum-каталогу payload арены
//! `StateData` (impl `StateRecordTarget for StateData` живёт рядом с trait в
//! Zone). Здесь реэкспорт для старого пакета; потребители сохраняют путь.

pub(crate) use nebokrai_zone::skills::statefactory::*;
