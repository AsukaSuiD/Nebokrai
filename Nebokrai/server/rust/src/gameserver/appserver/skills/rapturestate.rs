//! Каноническое состояние `CRaptureState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0xdb`, нулевые клиентские время и дополнительные данные,
//! сообщения начала/окончания `0xBFE03/0xBFE04` и сложение `blast_attack`
//! как `u16` с переполнением. Владение состоянием остаётся у
//! `CanonicalStateStorage`; сохранённый псевдокод ниже описывает остальной
//! непереведённый корпус.

use super::rapture::RAPTURE_SKILL_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RaptureState {
    blast_attack_gain: u16,
}

impl RaptureState {
    pub(crate) const fn new(blast_attack_gain: u16) -> Self {
        Self { blast_attack_gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        RAPTURE_SKILL_ID
    }
    pub(crate) const fn blast_attack_gain(self) -> u16 {
        self.blast_attack_gain
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp

// ============================================================================
// FUNCTION: CRaptureState::CRaptureState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:15
// RVA: 0x001F3BC0
// ADDRESS: 005f3bc0
// PROTOTYPE: undefined __thiscall CRaptureState(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::CRaptureState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:24
// RVA: 0x001F3C30
// ADDRESS: 005f3c30
// PROTOTYPE: undefined __thiscall CRaptureState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::~CRaptureState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:33
// RVA: 0x001F3CA0
// ADDRESS: 005f3ca0
// PROTOTYPE: void __thiscall ~CRaptureState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:91
// RVA: 0x001F3CB0
// ADDRESS: 005f3cb0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:109
// RVA: 0x001F3D70
// ADDRESS: 005f3d70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:38
// RVA: 0x001F3E90
// ADDRESS: 005f3e90
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:76
// RVA: 0x001F3EE0
// ADDRESS: 005f3ee0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRaptureStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rapturestate .cpp:176
// RVA: 0x001F3F90
// ADDRESS: 005f3f90
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
