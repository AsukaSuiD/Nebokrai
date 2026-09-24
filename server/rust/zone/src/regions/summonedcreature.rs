//! Владелец жизненного цикла призванного монстра `CSummonedCreature`,
//! перенесённый в Zone `regions/` (spawn и удаление принадлежащих
//! региону монстров).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/summonedcreature.cpp`, подтверждает два поля DWORD: время начала
//! и срока жизни. Нулевое время отключает срок, смерть имеет приоритет, а
//! истечение проверяется строгим сравнением суммы с переполнением с текущим DWORD.
//! Сама сущность остаётся единственным `CMonster` в `MonsterWorld`; этот тип
//! хранит только отличающийся жизненный цикл и не создаёт параллельное хранилище.
//! Constructor `CSummonedCreature` `1:1BA1B0` (VA `0x005BB1B0`) повторно
//! дизассемблирован: base `CMonster` ctor `0x4E7E70`, оба DWORD по
//! `+0x2A8/+0x2AC` обнуляются, vtable `0x65CFE4`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SummonedCreatureLifecycle {
    started_at_ms: u32,
    lifetime_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SummonedCreatureTick {
    Continue,
    Vanish,
}

impl SummonedCreatureLifecycle {
    pub const fn new(started_at_ms: u32, lifetime_ms: u32) -> Self {
        Self {
            started_at_ms,
            lifetime_ms,
        }
    }

    pub const fn tick(self, now_ms: u32, dead: bool) -> SummonedCreatureTick {
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
