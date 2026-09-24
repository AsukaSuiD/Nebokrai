//! Сценарное состояние `CAutoProtectState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/autoprotectstate.cpp`. Object Begin запрещён GM,
//! создаёт loop=1 без Update и изменения флага. Post-append UpdateProperty
//! (+0x24) включает защиту; End снимает флаг после visual, до RemoveState.
//! Вход в бой либо удар завершает первый живой экземпляр. Payload и lifecycle
//! принадлежат ScriptMoveState, конструктор данных не выполняет GM-gate.
//! Default ctor 0x005D41A0 задаёт keep=0; промежуточный объект заменён
//! прямой загрузкой полей общего payload, без наблюдаемых callbacks.

pub(crate) use nebokrai_zone::effects::AUTO_PROTECT_STATE_ID;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\autoprotectstate.cpp

// ============================================================================
// FUNCTION: CAutoProtectState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\autoprotectstate.cpp:85
// RVA: 0x001D4340
// ADDRESS: 005d4340
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CAutoProtectState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\autoprotectstate.cpp:111
// RVA: 0x001D4410
// ADDRESS: 005d4410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
