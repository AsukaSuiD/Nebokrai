//! LeafCut3 (0x8F): вариант без расхода RP, с удалением прежнего состояния
//! до боевого снимка и добавлением нового в конец контейнера.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut3.cpp.
//! Общие Begin/Check/AI/visual принадлежат leafcut/leafcutvisual,
//! различия наложения — leafcutapply; периодическое состояние — leafcutstate3.

pub(crate) const LEAF_CUT_3_SKILL_ID: u32 = 0x8f;
