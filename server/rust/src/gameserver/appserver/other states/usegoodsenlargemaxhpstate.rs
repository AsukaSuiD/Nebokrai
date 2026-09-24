//! Состояние предмета `CUseGoodsEnlargeMaxHpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargemaxhpstate.cpp`. Достигнутый путь
//! создаётся фабрикой `CMoveShape::AddState`; общий ScriptMoveState хранит время
//! и коэффициент, а эта формула увеличивает максимум HP с FISTP-усечением,
//! wrapping-сложением и верхней границей `i32::MAX`. Недостигнутые перегрузки
//! сохранены ниже.
//! Default ctor 0x005D59A0 задаёт keep=0/coefficient=1; промежуточный объект
//! заменён прямой загрузкой полей общего payload, без наблюдаемых callbacks.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;


pub(crate) fn apply(coefficient: u32, properties: &mut PlayerCombatProperties) {
    let delta = truncate_original(f64::from(coefficient)
        * f64::from(0.01_f32)
        * f64::from(properties.maximum_hp)) as u32;
    properties.maximum_hp = properties
        .maximum_hp
        .wrapping_add(delta)
        .min(i32::MAX as u32);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxhpstate.cpp

// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxhpstate.cpp:78
// RVA: 0x001D5A30
// ADDRESS: 005d5a30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxhpstate.cpp:89
// RVA: 0x001D5AD0
// ADDRESS: 005d5ad0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
