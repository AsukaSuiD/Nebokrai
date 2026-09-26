//! Состояние предмета `CUseGoodsEnlargeFullMissState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargefullmissstate.cpp`. Достигнутый путь
//! хранит время и коэффициент в ScriptMoveState; эта формула прибавляет коэффициент
//! к 16-битному `full_miss` с тем же wrapping-сужением.
//! Default ctor 0x005D46F0 задаёт keep=0/coefficient=0; промежуточный объект
//! заменён прямой загрузкой полей общего payload, без наблюдаемых callbacks.

use crate::gameserver::appserver::player::PlayerCombatProperties;


pub(crate) fn apply(coefficient: u32, properties: &mut PlayerCombatProperties) {
    properties.full_miss = properties.full_miss.wrapping_add(coefficient as u16);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargefullmissstate.cpp

// ============================================================================
// ============================================================================

// COMPONENT_VARIANT_END: GameServer
