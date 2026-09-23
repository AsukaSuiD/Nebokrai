//! Маска неподвижных областей FireWall и YinYang.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/firewallphalanx.cpp и yinyangphalanx{,2}.cpp.
//! ReplaceAffectRegion VA 0x005FFCD0, 0x005FE270, 0x005F2190;
//! CScope::SetInScope VA 0x005E9990. AI FireWall VA 0x006003E0,
//! YinYang VA 0x005FE9A0 и YinYang2 VA 0x005F28C0.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaskedAreaPulse {
    Once,
    Periodic { frequency_ms: u32, last_attack_ms: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaskedArea {
    started_at_ms: u32,
    lifetime_ms: u32,
    pulse: MaskedAreaPulse,
    length: i32,
    height: i32,
    cells: Vec<bool>,
}

impl MaskedArea {
    pub fn new(
        started_at_ms: u32, lifetime_ms: u32, pulse: MaskedAreaPulse,
        (length, height, mask): (i32, i32, &[bool]),
    ) -> Self {
        Self { started_at_ms, lifetime_ms, pulse, length, height, cells: mask.to_vec() }
    }

    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn dimensions(&self) -> (i32, i32) { (self.length, self.height) }
    pub const fn is_periodic(&self) -> bool {
        matches!(self.pulse, MaskedAreaPulse::Periodic { .. })
    }
    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        match self.pulse {
            MaskedAreaPulse::Once => self.expired_at(now),
            MaskedAreaPulse::Periodic { frequency_ms, last_attack_ms } =>
                frequency_ms.wrapping_add(last_attack_ms) < now,
        }
    }
    pub fn mark_attack_at(&mut self, now: u32) {
        if let MaskedAreaPulse::Periodic { last_attack_ms, .. } = &mut self.pulse {
            *last_attack_ms = now;
        }
    }

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
