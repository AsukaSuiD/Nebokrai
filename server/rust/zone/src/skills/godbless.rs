//! Параметры CGodBless/CGodBless2 при создании состояния.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/godbless{,2}.cpp/.h;
//! CGodBless::AI VA 0x005B0626–0x005B0703, 0x005B0859–0x005B08D7;
//! CGodBless2::AI VA 0x00550BA8–0x00550C81, 0x00550DAB–0x00550E29.

use crate::combat::truncate_original;
use crate::effects::GodBlessState;

const TARGET_ELEMENT_GAIN: u32 = 115;
const TARGET_MINIMUM_GAIN: u32 = 116;
const TARGET_MAXIMUM_GAIN: u32 = 117;
const TARGET_ELEMENT_COEFFICIENT: u32 = 120;
const TARGET_MINIMUM_COEFFICIENT: u32 = 121;
const TARGET_MAXIMUM_COEFFICIENT: u32 = 122;
const STATE_PERSIST_TIME: u32 = 10_002;

#[derive(Clone, Copy, Debug)]
pub struct GodBlessGains {
    minimum: f32,
    maximum: f32,
    element: f32,
}

impl GodBlessGains {
    /// Снимок трёх прибавок до удаления прежнего состояния.
    pub fn read(weapon: u32, mut query_property: impl FnMut(u32) -> u32) -> Self {
        let mut gain = |coefficient, constant| {
            let coefficient = query_property(coefficient);
            let scaled = (f64::from(coefficient.wrapping_mul(weapon)) * f64::from(0.01_f32)) as f32;
            let constant = query_property(constant);
            (f64::from(constant) + f64::from(scaled)) as f32
        };
        Self {
            minimum: gain(TARGET_MINIMUM_COEFFICIENT, TARGET_MINIMUM_GAIN),
            maximum: gain(TARGET_MAXIMUM_COEFFICIENT, TARGET_MAXIMUM_GAIN),
            element: gain(TARGET_ELEMENT_COEFFICIENT, TARGET_ELEMENT_GAIN),
        }
    }

    /// Вызывать после удаления прежнего состояния: срок запрашивается именно здесь.
    pub fn create_state(
        self, skill_id: u32, mut query_property: impl FnMut(u32) -> u32,
    ) -> GodBlessState {
        let keep_time_ms = query_property(STATE_PERSIST_TIME);
        let element = truncate_original(f64::from(self.element)) as u32;
        let maximum = truncate_original(f64::from(self.maximum)) as u32;
        let minimum = truncate_original(f64::from(self.minimum)) as u32;
        GodBlessState::new(skill_id, keep_time_ms, minimum, maximum, element)
    }
}
