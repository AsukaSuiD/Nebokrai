//! Правила вставки больших отверстий перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CDaKongXiangQian, DaKongDecodeError, DaKongSerializeError,
};
