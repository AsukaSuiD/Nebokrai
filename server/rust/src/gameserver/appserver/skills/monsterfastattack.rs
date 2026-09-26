//! Тонкий путь к двухударной быстрой атаке `CMonsterFastAttack` (ID `0x2d1`)
//! в Zone. Источник: gameserver.exe/GameServer.pdb, исходный owner
//! `appserver/skills/monsterfastattack.cpp`. Константы сроков, wire-кадр
//! выпуска и машинная база (фазы +0x50/+0x54/+0x58, кумулятивные сроки
//! 15001/15002, MP только player, двойной Attack с End(1), calc
//! `max(max-min,0)+1`, dyn-CPlayer crit) перенесены в
//! `nebokrai_zone::skills::monsterfastattack` (статусы VERIFIED/MATCH см.
//! там). Здесь — только re-export прежних имён; внешние потребители
//! (hub `monsterbaseattack.rs`, `monster.rs`, диспетчер `game.rs`) не
//! меняются. Связка FIRST/SECOND_TIME сохранена для будущего lord-владельца
//! split player-ветви `lordfastattack` (не этой волны).

pub(crate) use nebokrai_zone::skills::execution::MonsterFastAttackProgress;
pub(crate) use nebokrai_zone::skills::{
    MONSTER_FAST_ATTACK_SKILL_ID, SKILL_USAGE_FIRST_TIME, SKILL_USAGE_SECOND_TIME,
    fast_attack_fire_message,
};
