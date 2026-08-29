//! Сценарное состояние `CAutoProtectState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/autoprotectstate.cpp`. Состояние запрещено GM,
//! немедленно включает защитный флаг и визуальный эффект, а вход в бой либо
//! полученный удар завершает первый живой экземпляр с точным end-пакетом.

pub(crate) const AUTO_PROTECT_STATE_ID: i32 = 110_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AutoProtectState {
    _time_to_keep: i32,
}

impl AutoProtectState {
    pub(crate) const fn new(time_to_keep: i32, sufferer_is_gm: bool) -> Option<Self> {
        if sufferer_is_gm { None } else { Some(Self { _time_to_keep: time_to_keep }) }
    }

    pub(crate) const fn state_id(self) -> i32 { AUTO_PROTECT_STATE_ID }

    pub(crate) const fn apply(self, auto_protected: &mut bool) {
        *auto_protected = true;
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\autoprotectstate.cpp

// ============================================================================
// FUNCTION: CAutoProtectState::CAutoProtectState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\autoprotectstate.cpp:24
// RVA: 0x001D41A0
// ADDRESS: 005d41a0
// PROTOTYPE: undefined __thiscall CAutoProtectState(void)
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
