//! Общая блокировка движения и боя для Blind и состояний рывка;
//! BoaLock использует тот же lifecycle, но запрещает только движение.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/blindstate.cpp
//! и boalockstate.cpp. Payload-адаптер и живой lifecycle перенесены буквально
//! в Zone `skills/blindstate.rs` (порция №6c «self/zone-касты»; основание и
//! машинные статусы см. там). Здесь — реэкспорт прежних имён; потребители
//! не меняются. Данные и 8-байтная запись семейства — Zone `effects/blind.rs`.

pub(crate) use nebokrai_zone::effects::{BLIND_STATE_BYTES, BLIND_STATE_ID, BlindState};
pub(crate) use nebokrai_zone::skills::blindstate::{
    begin_primary_blind_state, begin_primary_blind_state_at,
    end_blind_state, replace_primary_blind_state, restart_blind_state, update_blind_state,
};
