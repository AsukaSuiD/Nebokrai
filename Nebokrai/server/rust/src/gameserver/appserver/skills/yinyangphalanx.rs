//! Однократные области инь-ян 0x139/0x146.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/yinyangphalanx{,2}.cpp.
//! Маски первого варианта 0x6A554C/0x6A5558/0x6A5564 — полные 3×3,
//! второго — 1×1. После строгого истечения срока живой обход X→Y делает
//! дедупликацию до допуска и завершает область только после всех попаданий.
//! Replace записывает транспонированную клетку, хотя AI читает обычный X/Y;
//! это подтверждённая особенность CScope::Set/Get, не замена геометрии.
//! Summon сам Replace не вызывает. Хранение маски заменено Vec; элементальное
//! попадание вынесено в общий адаптер четырёх одинаковых native-формул.
//! Снимок использует inherited CSummonShape::AddToByteArray 0x5E47A0.

use super::elementphalanxattack::ElementPhalanxAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CYinYangPhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    length: i32,
    height: i32,
    scope: Vec<bool>,
}

impl CYinYangPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new_for_skill(skill_id: u32, id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, minimum: i32, maximum: i32, element: i32, critical_chance: i32) -> Self {
        let (length, height, scope) = Self::level_scope(skill_id);
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape, attack: ElementPhalanxAttack { master, skill_id, skill_level, minimum, maximum, element, critical_chance },
            started_at_ms, lifetime_ms, length, height, scope: scope.to_vec(),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.attack.skill_level }
    pub(crate) const fn skill_id(&self) -> u32 { self.attack.skill_id }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub(crate) const fn dimensions(&self) -> (i32, i32) { (self.length, self.height) }

    fn level_scope(skill_id: u32) -> (i32, i32, &'static [bool]) {
        if skill_id == super::yinyang2::YIN_YANG_2_SKILL_ID {
            super::yinyangphalanx2::YIN_YANG_2_SCOPE
        } else {
            (3, 3, &[true; 9])
        }
    }

    pub(crate) fn origin(&self) -> (i32, i32) {
        let x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        (x.wrapping_sub(self.length >> 1), y.wrapping_sub(self.height >> 1))
    }

    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        let (current_left, current_top) = self.origin();
        let (new_length, new_height, new_scope) = Self::level_scope(self.skill_id());
        let new_left = tile_x.wrapping_sub(new_length >> 1);
        let new_top = tile_y.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let new_x = current_left.wrapping_add(x).wrapping_sub(new_left);
                let new_y = current_top.wrapping_add(y).wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height { continue; }
                let new_index = new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize;
                if new_scope.get(new_index).copied().unwrap_or(false) {
                    let current_index = x.wrapping_mul(self.length).wrapping_add(y) as usize;
                    if let Some(cell) = self.scope.get_mut(current_index) { *cell = false; }
                }
            }
        }
    }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) fn cell_active(&self, x: i32, y: i32) -> bool {
        self.scope.get(y.wrapping_mul(self.length).wrapping_add(x) as usize)
            .copied().unwrap_or(false)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, self.skill_id() as i32, self.skill_level(),
            self.master().master_type, self.master().master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}
