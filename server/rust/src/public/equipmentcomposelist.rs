//! Таблицы преобразования экипировки перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    EquipmentComposeDecodeError, EquipmentComposeList, EquipmentComposeSerializeError,
};
