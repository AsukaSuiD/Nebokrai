//! Базовый CVisualEffect для состояний и навыков Zone.
//! GameServer/gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/states/visualeffect.h/.cpp. Проверены VA 0x005DC1B0..0x005DC234.
//! Идентификаторы пары —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки.

#[derive(Debug, Eq, PartialEq)]
pub struct CVisualEffect {
    ended: bool,
    loop_value: i32,
}

impl CVisualEffect {
    pub const fn new() -> Self {
        Self {
            ended: false,
            loop_value: 0,
        }
    }

    pub const fn is_ended(&self) -> bool {
        self.ended
    }

    pub const fn loop_value(&self) -> i32 {
        self.loop_value
    }

    pub const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.ended = false;
        self.loop_value = loop_value;
    }

    pub const fn end_visual_effect(&mut self) {
        self.ended = true;
    }

    pub const fn update_visual_effect(&mut self) {
        if self.loop_value == 0 {
            self.end_visual_effect();
        }
    }
}

impl Default for CVisualEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CVisualEffect {
    fn drop(&mut self) {
        self.ended = true;
        self.loop_value = 0;
    }
}
