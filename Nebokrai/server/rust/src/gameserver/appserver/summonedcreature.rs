//! Владелец жизненного цикла призванного монстра `CSummonedCreature`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/summonedcreature.cpp`, подтверждает два поля DWORD: время начала
//! и срока жизни. Нулевое время отключает срок, смерть имеет приоритет, а
//! истечение проверяется строгим сравнением суммы с переполнением с текущим DWORD.
//! Сама сущность остаётся единственным `CMonster` в `MonsterWorld`; этот тип
//! хранит только отличающийся жизненный цикл и не создаёт параллельное хранилище.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SummonedCreatureLifecycle {
    started_at_ms: u32,
    lifetime_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SummonedCreatureTick {
    Continue,
    Vanish,
}

impl SummonedCreatureLifecycle {
    pub(crate) const fn new(started_at_ms: u32, lifetime_ms: u32) -> Self {
        Self {
            started_at_ms,
            lifetime_ms,
        }
    }

    pub(crate) const fn tick(self, now_ms: u32, dead: bool) -> SummonedCreatureTick {
        if dead
            || (self.started_at_ms != 0
                && self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms)
        {
            SummonedCreatureTick::Vanish
        } else {
            SummonedCreatureTick::Continue
        }
    }
}
