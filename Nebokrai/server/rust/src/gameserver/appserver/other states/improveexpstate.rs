//! Сценарное состояние `CImproveExpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/improveexpstate.cpp`. Достигнутый путь создаётся
//! `CMoveShape::AddState`; каждый живой экземпляр независимо добавляет
//! `coefficient * float(0.01)` к неокруглённому x87-подобному множителю опыта
//! в порядке канонического списка.
//! Время и коэффициент принадлежат единственному payload ScriptMoveState;
//! здесь остаётся только формула, без повторного config-объекта.
//! Default ctor 0x005D5E90 задаёт keep=0/coefficient=0; промежуточный объект
//! заменён прямой загрузкой полей общего payload, без наблюдаемых callbacks.
//! Exact vtable направляет клиентский срок на
//! `CAgilityState2::GetRemainedTime` по `0x005D5F30`.

pub(crate) const IMPROVE_EXP_STATE_ID: i32 = 100_009;

pub(crate) fn multiplier_delta(coefficient: u32) -> f64 {
    f64::from(coefficient) * f64::from(0.01_f32)
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\improveexpstate.cpp

// ============================================================================
// FUNCTION: CImproveExpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\improveexpstate.cpp:64
// RVA: 0x001D5F60
// ADDRESS: 005d5f60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CImproveExpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\improveexpstate.cpp:75
// RVA: 0x001D6000
// ADDRESS: 005d6000
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
