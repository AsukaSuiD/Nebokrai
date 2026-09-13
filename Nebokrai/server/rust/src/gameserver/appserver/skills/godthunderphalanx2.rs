//! Маска второго божественного грома.
//! Источник: gameserver.exe/GameServer.pdb, godthunderphalanx2.cpp.
//! Все три уровня используют одну округлую маску 7×7, совпадающую с Thunder.
//! Payload, оконный lifecycle и wire двух GodThunder принадлежат godthunderphalanx.

pub(super) use super::thunderphalanx::{
    THUNDER_SCOPE as GOD_THUNDER_2_SCOPE, THUNDER_SCOPE_SIDE as GOD_THUNDER_2_SCOPE_SIDE,
};
