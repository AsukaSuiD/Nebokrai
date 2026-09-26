//! Жизненный цикл призванного монстра `CSummonedCreature` (spawn и удаление
//! принадлежащих региону монстров). Исходный owner
//! `appserver/summonedcreature.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`.
//!
//! Два поля DWORD: время начала и срока жизни. Нулевое время отключает срок,
//! смерть имеет приоритет, а истечение проверяется строгим сравнением суммы с
//! переполнением с текущим DWORD.
//! Сама сущность остаётся единственным `CMonster` в `MonsterWorld`; этот тип
//! хранит только отличающийся жизненный цикл и не создаёт параллельное хранилище.
//! Constructor разобран по дизассемблу точной пары; оба DWORD по
//! `+0x2A8/+0x2AC` обнуляются. Опорные адреса — раздел «NPC и базовые
//! фигуры» в docs/reconstruction/gameserver-npc-and-regions.md.

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
