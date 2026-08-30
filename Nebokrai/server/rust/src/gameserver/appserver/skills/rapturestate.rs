//! Каноническое состояние `CRaptureState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0xdb`, нулевые клиентские время и дополнительные данные,
//! сообщения начала/окончания `0xBFE03/0xBFE04` и сложение `blast_attack`
//! как `u16` с переполнением. Владение состоянием остаётся у
//! `CanonicalStateStorage`. Constructor RVA `0x001F3C30` задаёт ID `0xDB` и
//! нулевой gain; это буквально выражено `Default`.

use super::rapture::RAPTURE_SKILL_ID;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RaptureState {
    blast_attack_gain: u16,
}

impl RaptureState {
    pub(crate) const fn new(blast_attack_gain: u16) -> Self {
        Self { blast_attack_gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        RAPTURE_SKILL_ID
    }
    pub(crate) const fn blast_attack_gain(self) -> u16 {
        self.blast_attack_gain
    }
}
