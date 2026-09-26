//! Owner `CFairyContainer` старого GameServer перенесён в Zone items.
//! Здесь его реэкспорт для старого пакета; report-типы путей опыта
//! параметризованы generic-швом `FairyGrowEffectSink` — подстановка прежнего
//! `GameEffectJournal` выводится в точке доставки эффектов (волна Z-C2c,
//! потребители без правок).

pub(crate) use nebokrai_zone::items::cfairycontainer::*;
