//! ID состояний, которые Fury снимает перед созданием Cure.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/fury.cpp/.h;
//! CFury::AI, VA 0x00536C43–0x00536CCE.

pub fn is_fury_conflicting_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        0x138 | 0xd2 | 0xc9 | 0x67 | 0x192 | 0x191 | 0x198 | 0x199 | 0x1a6
    )
}
