//! Каноническая достигнутая player-часть `COriginState`.
//!
//! Игрок получает знаково расширенные младшие 16 бит параметра через
//! wrapping-сложение с `element_modify`. Состояние `304` не имеет собственного
//! таймера или визуального сообщения. Monster-ветвь передаёт полный signed gain
//! в `SetElementModify`, поэтому заменяет значение, а не складывает его.
//! Constructor RVA `0x00201210` задаёт ID `0x130` и нулевой gain.

use super::origin::ORIGIN_SKILL_ID;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OriginState {
    element_modify_gain: i32,
}

impl OriginState {
    pub(crate) const fn new(element_modify_gain: i32) -> Self {
        Self {
            element_modify_gain,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ORIGIN_SKILL_ID
    }

    pub(crate) const fn apply_to_player(self, value: i32) -> i32 {
        value.wrapping_add((self.element_modify_gain as i16) as i32)
    }

    pub(crate) const fn apply_to_monster(self) -> i32 {
        self.element_modify_gain
    }
}
