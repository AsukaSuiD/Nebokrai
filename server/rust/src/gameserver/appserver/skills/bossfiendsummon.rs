//! Призыв CBossFiendSummon (0x1f9) для игрока и монстра.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/bossfiendsummon.cpp.
//! Константа и правило выбора свойства 30003/30004/30005 по `random(3)`
//! (бросок до нулевой проверки количества) перенесены в
//! `nebokrai_zone::skills::summoncreatureskill` и прочитаны оттуда общим
//! путём `execute_owned_summon_creature`; машинные факты Begin
//! (0x0052c390/0x0052c460/0x0052c550), Summon (0x0052c610) и общие
//! CheckCastCondition/AI записаны в шапке zone-владельца, там же поправка
//! карты полосы (Summon — НЕ 4-классовый фолд). Здесь — тонкий re-export
//! ID для `monsterbaseattack.rs` и диспетчера игрока.

pub(crate) use nebokrai_zone::skills::BOSS_FIEND_SUMMON_SKILL_ID;
