//! Состояние предмета `CUseGoodsEnlargeFullMissState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargefullmissstate.cpp`. Достигнутый путь
//! хранит время и коэффициент в ScriptMoveState; эта формула прибавляет коэффициент
//! к 16-битному `full_miss` с тем же wrapping-сужением.
//! Default ctor 0x005D46F0 задаёт keep=0/coefficient=0; промежуточный объект
//! заменён прямой загрузкой полей общего payload, без наблюдаемых callbacks.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const USE_GOODS_ENLARGE_FULL_MISS_STATE_ID: i32 = 100_012;

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
// FUNCTION: CUseGoodsEnlargeFullMissState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargefullmissstate.cpp:75
// RVA: 0x001D4770
// ADDRESS: 005d4770
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeFullMissState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargefullmissstate.cpp:86
// RVA: 0x001D4810
// ADDRESS: 005d4810
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
