//! Мировые стадии войн и событий Realm.

pub mod attackcitysys; // расписание городских войн CAttackCitySys и его фазовые timers.
pub mod countrywarsys; // расписание войн государств.
pub mod factionwarsys; // войны фракций: реестр war types и enemy pairs.
pub mod fournationwarsys; // война четырёх стран и её wire-setup.
pub mod jjcmaintenanceworker; // detached owner недельного DB-сброса JJC.
pub mod jjcsystem; // арена CJJcSystem: карты, ранги и недельный сброс.
pub mod leiting; // суточное обновление LeiTing.
pub mod leitingreset; // DB-владелец сброса LeiTing.
pub mod leitingresetworker; // fire-and-forget worker сброса LeiTing.
pub mod misc; // process-global номер копии ShengSiShiSu.
pub mod rsgodsbattle; // DB-владелец Gods Battle.
pub mod rsjjcsys; // DB-владелец JJC.
pub mod villagewarsys; // расписание деревенских войн.
