//! Тонкий путь к смазке оружия ядом CDaubPoison (0xDF) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/daubpoison.cpp`. Тело применения (завершение первого
//! непустого слота ID 0xDF с destructor-ом свежего остатка позиции, ctor
//! keep только из Query(10002), primary Begin(U,U) с append; результат
//! установки не отменяет завершение навыка End(1)) перенесено буквально в
//! `nebokrai_zone::skills::daubpoison` (основание и статусы MATCH — в шапке
//! Zone-файла; кластер D, порция D4). Скелет Begin/Check/AI остаётся общим
//! hub `selfstatecast.rs` (граница D): применение и visual используют только
//! исходного U, запрошенная S не становится получателем смазки. Здесь —
//! делегации с прежними сигнатурами: драйвер `selfstatecast.rs` и
//! потребители ID (`game.rs`, `playercast.rs`) не меняются.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::DAUB_POISON_SKILL_ID;

pub(super) fn apply_daub_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    nebokrai_zone::skills::apply_daub_poison(
        game, source, properties, &mut || runtime.now_milliseconds(),
    );
}
