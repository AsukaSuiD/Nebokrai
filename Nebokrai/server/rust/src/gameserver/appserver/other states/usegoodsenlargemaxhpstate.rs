//! Состояние предмета `CUseGoodsEnlargeMaxHpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargemaxhpstate.cpp`. Достигнутый путь
//! создаётся фабрикой `CMoveShape::AddState`, хранит исходные `DWORD` времени
//! и коэффициента и при пересчёте увеличивает максимум HP с FISTP-усечением,
//! wrapping-сложением и верхней границей `i32::MAX`. Недостигнутые перегрузки
//! сохранены ниже.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const USE_GOODS_ENLARGE_MAX_HP_STATE_ID: i32 = 100_007;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UseGoodsEnlargeMaxHpState {
    _time_to_keep: u32,
    coefficient: u32,
}

impl UseGoodsEnlargeMaxHpState {
    pub(crate) const fn new(time_to_keep: u32, coefficient: u32) -> Self {
        Self { _time_to_keep: time_to_keep, coefficient }
    }

    pub(crate) const fn state_id(self) -> i32 { USE_GOODS_ENLARGE_MAX_HP_STATE_ID }

    pub(crate) fn apply(self, properties: &mut PlayerCombatProperties) {
        let delta = (f64::from(self.coefficient)
            * f64::from(0.01_f32)
            * f64::from(properties.maximum_hp))
        .trunc() as i32 as u32;
        properties.maximum_hp = properties
            .maximum_hp
            .wrapping_add(delta)
            .min(i32::MAX as u32);
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxhpstate.cpp

// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxHpState::CUseGoodsEnlargeMaxHpState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxhpstate.cpp:25
// RVA: 0x001D59A0
// ADDRESS: 005d59a0
// PROTOTYPE: undefined __thiscall CUseGoodsEnlargeMaxHpState(void)
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
