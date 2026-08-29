//! Состояние предмета `CUseGoodsEnlargeElmDefState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/usegoodsenlargeelmdefstate.cpp`. Достигнутый путь
//! сохраняет `DWORD` времени и коэффициента; формула сопротивления выполняет
//! исходное float-округление, wrapping-сложение и маску младших 16 бит.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const USE_GOODS_ENLARGE_ELM_DEF_STATE_ID: i32 = 100_011;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UseGoodsEnlargeElmDefState {
    _time_to_keep: u32,
    coefficient: u32,
}

impl UseGoodsEnlargeElmDefState {
    pub(crate) const fn new(time_to_keep: u32, coefficient: u32) -> Self {
        Self { _time_to_keep: time_to_keep, coefficient }
    }

    pub(crate) const fn state_id(self) -> i32 { USE_GOODS_ENLARGE_ELM_DEF_STATE_ID }

    pub(crate) fn apply(self, properties: &mut PlayerCombatProperties) {
        let delta = ((self.coefficient as f32) * 0.01 * (properties.element_resistance as f32)).round() as u32;
        properties.element_resistance = properties.element_resistance.wrapping_add(delta) & 0xffff;
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargeelmdefstate.cpp

// ============================================================================
// FUNCTION: CUseGoodsEnlargeElmDefState::CUseGoodsEnlargeElmDefState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargeelmdefstate.cpp:25
// RVA: 0x001D4B30
// ADDRESS: 005d4b30
// PROTOTYPE: undefined __thiscall CUseGoodsEnlargeElmDefState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeElmDefState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargeelmdefstate.cpp:76
// RVA: 0x001D4BC0
// ADDRESS: 005d4bc0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CUseGoodsEnlargeElmDefState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\usegoodsenlargeelmdefstate.cpp:87
// RVA: 0x001D4C60
// ADDRESS: 005d4c60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
