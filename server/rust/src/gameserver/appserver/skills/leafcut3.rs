//! LeafCut3 (0x8F): вариант без расхода RP, с удалением прежнего состояния
//! до боевого снимка и добавлением нового в конец контейнера.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut3.cpp.
//! Общие Begin/Check/AI/visual принадлежат leafcut/leafcutvisual,
//! различия наложения — leafcutapply; периодическое состояние — leafcutstate3.

pub(crate) use nebokrai_zone::effects::LEAF_CUT_3_STATE_ID as LEAF_CUT_3_SKILL_ID;
