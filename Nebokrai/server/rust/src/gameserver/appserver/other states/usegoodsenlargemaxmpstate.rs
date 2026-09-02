//! Состояние предмета `CUseGoodsEnlargeMaxMpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargemaxmpstate.cpp`. Достигнутый путь
//! создаётся фабрикой `CMoveShape::AddState`, хранит исходные `DWORD` времени
//! и коэффициента и при пересчёте увеличивает максимум MP с FISTP-усечением,
//! wrapping-сложением и верхней границей `i32::MAX`. Недостигнутые перегрузки
//! сохранены ниже.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const USE_GOODS_ENLARGE_MAX_MP_STATE_ID: i32 = 100_008;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UseGoodsEnlargeMaxMpState {
    _time_to_keep: u32,
    coefficient: u32,
}

impl UseGoodsEnlargeMaxMpState {
    pub(crate) const fn new(time_to_keep: u32, coefficient: u32) -> Self {
        Self { _time_to_keep: time_to_keep, coefficient }
    }

    pub(crate) const fn state_id(self) -> i32 { USE_GOODS_ENLARGE_MAX_MP_STATE_ID }

    pub(crate) fn apply(self, properties: &mut PlayerCombatProperties) {
        let delta = (f64::from(self.coefficient)
            * f64::from(0.01_f32)
            * f64::from(properties.maximum_mp))
        .trunc() as i32 as u32;
        properties.maximum_mp = properties
            .maximum_mp
            .wrapping_add(delta)
            .min(i32::MAX as u32);
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxmpstate.cpp

// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxMpState::CUseGoodsEnlargeMaxMpState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxmpstate.cpp:25
// RVA: 0x001D54F0
// ADDRESS: 005d54f0
// PROTOTYPE: undefined __thiscall CUseGoodsEnlargeMaxMpState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxmpstate.cpp:76
// RVA: 0x001D5580
// ADDRESS: 005d5580
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeMaxMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargemaxmpstate.cpp:87
// RVA: 0x001D5620
// ADDRESS: 005d5620
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
