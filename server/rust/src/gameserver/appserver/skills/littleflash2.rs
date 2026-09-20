//! Вариант малого рывка к текущей S либо сохранённой точке.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/littleflash2.cpp.
//! В отличие от LittleFlash он не требует занятую клетку на пути, но очищает
//! одиночную заблокированную клетку. Общие Begin, AI, visual и End находятся
//! в littleflash.rs; регистрация, путь и список поражённых самостоятельны.

pub(crate) const LITTLE_FLASH_2_SKILL_ID: u32 = 0x7f;
pub(super) const EMPTY_PATH_MESSAGE_ID: &[u8] = b"GS0309";
