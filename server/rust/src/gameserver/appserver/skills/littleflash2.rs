//! Вариант малого рывка к текущей S либо сохранённой точке.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/littleflash2.cpp.
//! ID навыка и текст пустого пути перенесены буквально в Zone
//! `skills/littleflash.rs` (порция №5 «player melee»; основание см. там);
//! общие Begin, AI, visual и End находятся там же. Здесь — прежний переходный
//! путь реэкспорта для старого пакета.

pub(crate) use nebokrai_zone::skills::littleflash::LITTLE_FLASH_2_SKILL_ID;

#[allow(unused_imports, reason = "прежний переходный путь старого пакета")]
pub(super) use nebokrai_zone::skills::littleflash::LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID as EMPTY_PATH_MESSAGE_ID;
