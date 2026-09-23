//! Базовый CVisualEffect для состояний и навыков Zone.
//! GameServer/gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/states/visualeffect.h/.cpp. Проверены VA 0x005DC1B0..0x005DC234.
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E;
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.

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
