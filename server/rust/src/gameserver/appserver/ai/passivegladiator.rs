//! Состояние пассивного гладиатора перенесено в Zone (`ai/passivegladiator.rs`)
//! зелёной AI-порцией; здесь переходный реэкспорт типов для hub-владельца
//! `CMonster` и его caller-ов.

pub(crate) use nebokrai_zone::ai::PassiveGladiatorState;
