//! LeafCut2 (0x80): вариант без расхода RP, с удалением прежнего состояния
//! до боевого снимка и добавлением нового в конец контейнера.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut2.cpp.
//! Общие Begin/Check/AI/visual принадлежат leafcut/leafcutvisual,
//! различия наложения — leafcutapply; периодическое состояние — leafcutstate2.

pub(crate) use nebokrai_zone::effects::LEAF_CUT_2_STATE_ID as LEAF_CUT_2_SKILL_ID;
