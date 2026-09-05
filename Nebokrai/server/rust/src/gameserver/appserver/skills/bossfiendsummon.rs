//! Призыв CBossFiendSummon (0x1f9) для игрока и монстра.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/bossfiendsummon.cpp.
//! Все три Begin (0x0052c390/0x0052c460/0x0052c550) отличаются от
//! CSummonCorpseCandle только типом visual effect; сетевой обработчик эффекта
//! (0x0052c7e0) сохраняет тот же формат. CheckCastCondition (0x0053e8d0),
//! AI (0x0053f270) и End (0x005ae7a0) — общие функции этих классов.
//! Исполнение, самостоятельный cooldown игрока, движение и публикация
//! существ связаны через summoncreatureskill. Summon (0x0052c610) принимает
//! также игрока и сохраняет его MasterInfo. После разрешения региона, до
//! цикла создания, ровно один random(3) выбирает свойство 30003/30004/30005;
//! бросок выполняется и при нулевом количестве существ. Все позиции затем
//! выбираются обычным региональным владельцем, без дополнительного RNG здесь.

pub(crate) const BOSS_FIEND_SUMMON_SKILL_ID: u32 = 0x1f9;

pub(crate) const fn summoned_creature_usage(random_value: i32) -> u32 {
    match random_value {
        1 => 30_004,
        2 => 30_005,
        _ => 30_003,
    }
}
