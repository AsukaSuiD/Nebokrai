//! Состояние предмета `CUseGoodsEnlargeElmDefState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargeelmdefstate.cpp`. Достигнутый путь
//! хранит время и коэффициент в ScriptMoveState; формула сопротивления выполняет
//! исходное FISTP-усечение, wrapping-сложение и сужение к младшим 16 битам.
//! Default ctor 0x005D4B30 задаёт keep=0/coefficient=1; промежуточный объект
//! заменён прямой загрузкой полей общего payload, без наблюдаемых callbacks.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;


pub(crate) fn apply(coefficient: u32, properties: &mut PlayerCombatProperties) {
    let delta = truncate_original(f64::from(coefficient)
        * f64::from(0.01_f32)
        * f64::from(properties.element_resistance)) as u32;
    properties.element_resistance = properties.element_resistance.wrapping_add(delta) & 0xffff;
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargeelmdefstate.cpp

// ============================================================================
// ============================================================================

// COMPONENT_VARIANT_END: GameServer
