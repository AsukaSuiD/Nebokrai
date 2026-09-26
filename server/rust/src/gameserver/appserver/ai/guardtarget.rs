//! Делегат общего дистанционного ядра охранников в Zone.
//!
//! Точка поста `GuardStationState`, дистанционное ядро выбора и групповой
//! выбор перенесены буквально в `nebokrai_zone::ai::guardtarget` — машинная
//! база `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS match,
//! RVA-якоря `0x0060E290`/`0x0060E350`/`0x0060E510`) описана в её шапке
//! волной Z-AI. Здесь — только переэкспорт прежних типов и функций:
//! потребители старого пакета (`monster.rs`, `cityguardwithsword`,
//! `guardcountry`, `godsbattleguardwithsword`, `guardwithsword`) не меняются.

pub(crate) use nebokrai_zone::ai::guardtarget::{
    GuardDistanceTarget, GuardStationState, consider_guard_distance_target,
    select_guard_target_groups,
};
