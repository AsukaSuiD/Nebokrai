//! Состояние предмета `CUseGoodsEnlargeDefState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargedefstate.cpp`. Достигнутый путь
//! сохраняет `DWORD` времени и коэффициента; формула защиты выполняет исходное
//! FISTP-усечение, wrapping-сложение и сужение к младшим 16 битам.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const USE_GOODS_ENLARGE_DEF_STATE_ID: i32 = 100_010;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UseGoodsEnlargeDefState {
    _time_to_keep: u32,
    coefficient: u32,
}

impl UseGoodsEnlargeDefState {
    pub(crate) const fn new(time_to_keep: u32, coefficient: u32) -> Self {
        Self { _time_to_keep: time_to_keep, coefficient }
    }

    pub(crate) const fn state_id(self) -> i32 { USE_GOODS_ENLARGE_DEF_STATE_ID }

    pub(crate) fn apply(self, properties: &mut PlayerCombatProperties) {
        let delta = (f64::from(self.coefficient)
            * f64::from(0.01_f32)
            * f64::from(properties.defense))
        .trunc() as i32 as u32;
        properties.defense = properties.defense.wrapping_add(delta) & 0xffff;
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargedefstate.cpp

// ============================================================================
// FUNCTION: CUseGoodsEnlargeDefState::CUseGoodsEnlargeDefState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargedefstate.cpp:25
// RVA: 0x001D5030
// ADDRESS: 005d5030
// PROTOTYPE: undefined __thiscall CUseGoodsEnlargeDefState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeDefState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargedefstate.cpp:78
// RVA: 0x001D50C0
// ADDRESS: 005d50c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeDefState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargedefstate.cpp:89
// RVA: 0x001D5160
// ADDRESS: 005d5160
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
