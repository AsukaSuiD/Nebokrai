//! Состояние и правила игровых клиентских сессий Zone.

pub mod cpersonalshopbuyer; // buyer plug личной лавки.
pub mod cplug; // CPlug: owner-часть plug GameServer.
pub mod csession; // CSession: plug-list storage-часть сессии.
pub mod cteam; // CTeam: сеанс команды.
pub mod cteamate; // CTeamate: командный разъём, принадлежащий игроку.

mod sequence; // проверочная последовательность сессии.
mod validation; // состояние проверок игрового входа по player ID.

pub use sequence::{
    CSequenceRegistry, CSequenceString, SequenceRegistryInitializationError, SequenceSerializeError,
}; // реестр и строка проверочной последовательности.
pub use validation::{
    LoginValidationRelease, LoginValidationState, PreparedSequence, SequencePreparationError,
}; // подготовка последовательности и состояние входа.
