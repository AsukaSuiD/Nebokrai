//! Оглушение рывка CKnightCutState (0x67), переходный адаптер Game.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/knightcutstate.cpp.
//! Данные и 8-байтная запись перенесены в Zone `effects/blind.rs`
//! (конструкторы VA 0x005FCCF0/0x005FCD60, vtable 0x00661254).
//!
//! Объектный Begin, AI, End, региональная смена и visual используют общий
//! blindstate: S обязательна, NULL U сохраняет timestamp. Visual предшествует
//! запретам движения и боя, публикация занимает выбранный caller-ом слот.
//! End снимает запреты живой S и удаляет тот же экземпляр, а не первый ID.
//! Защитное действие завершает состояние; Cure использует общий direct End.
//! Вход в регион восстанавливает visual и запреты без нового отсчёта.

pub(crate) use nebokrai_zone::effects::{
    KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID, KnightCutState,
};
