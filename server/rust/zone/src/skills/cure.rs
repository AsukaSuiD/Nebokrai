//! Числовое правило и выбор снимаемых состояний CCure.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/cure.cpp/.h, CCure::CastCure VA 0x005ADB10–0x005ADC91.

use crate::combat::truncate_original;

pub const fn is_cure_removable_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        0x138 | 0xd2 | 0xc9 | 0x67 | 0x192 | 0x191 | 0x198 | 0x199 | 0x1a6 | 0x73 | 0x7c | 0x1f8
    )
}

/// `FILD` unsigned-модификатора с поправкой 2^32, затем `FMUL 0.01`,
/// `FIMUL` свойства игрока и `FISTP DWORD` перед DWORD-арифметикой.
pub fn cure_threshold(element_modify: i32, probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    (scaled as u32).wrapping_mul(constant).wrapping_add(probability) as i32
}
