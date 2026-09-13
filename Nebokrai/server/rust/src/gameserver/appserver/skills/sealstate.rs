//! Печать CSealState (0x138): запрет движения и боя до срока или защиты.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/sealstate.cpp.
//!
//! Общий blindstate сохраняет Begin, visual, таймер, End и восьмибайтный
//! codec ID/remaining. Begin требует S, читает время при непустом U,
//! отправляет BFE03 до запретов движения и боя. End отправляет BFE04,
//! снимает запреты текущей S и удаляет тот же экземпляр. Вход в регион
//! восстанавливает блокировки без нового отсчёта; Defense завершает состояние.
//! Seal создаёт новый payload до End/destructor первого одноимённого
//! состояния, затем публикует его в прежней позиции либо в конце списка.
//! Поколенческий ключ общей арены заменяет владение сырым указателем.

pub(crate) const SEAL_STATE_ID: u32 = 0x138;
pub(crate) const SEAL_STATE_BYTES: usize = super::blindstate::BLIND_STATE_BYTES;

pub(crate) type SealState = super::blindstate::BlindState<SEAL_STATE_ID>;
