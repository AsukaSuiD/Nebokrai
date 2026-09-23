//! Маска неподвижных областей FireWall и YinYang.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/firewallphalanx.cpp и yinyangphalanx{,2}.cpp.
//! ReplaceAffectRegion VA 0x005FFCD0, 0x005FE270, 0x005F2190;
//! CScope::SetInScope VA 0x005E9990.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaskedArea {
    length: i32,
    height: i32,
    cells: Vec<bool>,
}

impl MaskedArea {
    pub fn new((length, height, mask): (i32, i32, &[bool])) -> Self {
        Self { length, height, cells: mask.to_vec() }
    }

    pub const fn dimensions(&self) -> (i32, i32) { (self.length, self.height) }

    pub fn origin(&self, (center_x, center_y): (i32, i32)) -> (i32, i32) {
        (center_x.wrapping_sub(self.length >> 1), center_y.wrapping_sub(self.height >> 1))
    }

    pub fn replace_affect_region(
        &mut self, center: (i32, i32), incoming_center: (i32, i32),
        (new_length, new_height, incoming_mask): (i32, i32, &[bool]),
    ) {
        let (left, top) = self.origin(center);
        let new_left = incoming_center.0.wrapping_sub(new_length >> 1);
        let new_top = incoming_center.1.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let new_x = left.wrapping_add(x).wrapping_sub(new_left);
                let new_y = top.wrapping_add(y).wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height { continue; }
                if incoming_mask.get(new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize)
                    .copied().unwrap_or(false)
                {
                    // Native передаёт смещения в SetInScope в переставленном порядке.
                    let index = x.wrapping_mul(self.length).wrapping_add(y) as usize;
                    if let Some(cell) = self.cells.get_mut(index) { *cell = false; }
                }
            }
        }
    }

    pub fn cell_active(&self, x: i32, y: i32) -> bool {
        self.cells.get(y.wrapping_mul(self.length).wrapping_add(x) as usize)
            .copied().unwrap_or(false)
    }
}
