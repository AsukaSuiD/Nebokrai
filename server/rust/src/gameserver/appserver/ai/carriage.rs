//! Делегат контроллера повозки `CCarriage` (AI12) в Zone.
//!
//! Lifecycle-состояние, master-проверка/таймер и follow/stay-планы движения
//! перенесены буквально в `nebokrai_zone::ai::carriage` — машинная база
//! `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS match),
//! RVA-якоря (`0x00506710` SetCurrentAction, `0x00506720`/`0x005068F0`
//! расписания, `0x00506A20` follow-OnSchedule с хвостом `0x00506CFE`/
//! `0x00506D04`) и граница hub-входа (`CGame` lifecycle, журнал `0x6020E`,
//! пакеты GS, фактический `Evanish`) описаны в её шапке волной Z-AI. Здесь —
//! прежние сигнатуры и переэкспорт типов: состояние повозки остаётся полем
//! переходного `CMonster`, а пространственное перемещение, привязка игрока и
//! фактическое удаление — hub-владением через фасады `MonsterDispatcher*`.
//! Потребители (`monster.rs`, `game.rs`) не меняются.

pub(crate) use nebokrai_zone::ai::carriage::{
    CARRIAGE_FOLLOWING, CARRIAGE_STAYING, CarriageLifecycleState, CarriageMasterFacts,
    CarriageMasterOutcome, CarriageMovementPlan, plan_carriage_movement,
};
