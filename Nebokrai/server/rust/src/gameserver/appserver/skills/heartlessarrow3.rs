//! Вариант `CHeartLessArrow3` (`0xE6`) семейства региональных стрел.
//!
//! Проверки, wire и phalanx совпадают с `CHeartLessArrow2`; единственное
//! подтверждённое отличие порядка разблокировки движения выражено общим
//! owner-ом `heartlessarrow2` через идентификатор варианта.

pub(crate) const HEARTLESS_ARROW_3_SKILL_ID: u32 =
    super::heartlessarrowphalanx3::HEARTLESS_ARROW_PHALANX_3_SKILL_ID;
