//! Обход области ослабления и координация живых игровых владельцев.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/weakphalanx.cpp.
//!
//! Каждая клетка получает свой снимок фигур; допуск и наличие состояния
//! проверяются при обработке цели. Игрок-Master ищется глобально для допуска,
//! а User состояния — повторно в регионе области. Уже существующий Weak
//! не заменяется. Перекрытие вызывает полный End старой области до AddShape;
//! при истечении ему предшествует отдельный проход прямых End состояний.
//! Клетки разрешаются в своём регионе; поиск User требует живой объект
//! регионального реестра, но не координаты или таблицу геометрии монстра.

use super::*;
use crate::gameserver::appserver::skills::weakphalanx::{weak_cell_targets, CWeakPhalanx};
use crate::gameserver::appserver::skills::weakstate::{begin_primary_weak_state, WEAK_STATE_ID};
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, end_move_shape_state, resolve_region_move_shape, resolve_state_move_shape,
};

impl CGame {
    pub(crate) fn add_weak_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CWeakPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let owner = self.find_region(region_id)?;
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            tile_x, tile_y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        for shape in shapes {
            if shape.identity.object_type != SUMMON_SHAPE_TYPE { continue; }
            let Some(old) = self.weak_phalanx(region_id, shape.identity.id) else { continue; };
            if old.shape().get_tile_x() == Ok(tile_x) && old.shape().get_tile_y() == Ok(tile_y) {
                self.end_weak_phalanx(region_id, shape.identity.id);
            }
        }
        let Some(mut owner) = self.take_region_owner(region_id) else {
            let _ = self.publish_weak_phalanx_entry(&phalanx, runtime);
            return None;
        };
        let result = owner.base_mut().add_weak_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(match result {
            Ok(id) => Ok(id),
            Err((block, phalanx)) => {
                // Отказ AddShape не отменяет сериализацию нового объекта.
                let _ = self.publish_weak_phalanx_entry(&phalanx, runtime);
                Err(block)
            }
        })
    }

    pub(crate) fn send_weak_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx_id: i32, runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.weak_phalanx(region_id, phalanx_id)?;
        self.publish_weak_phalanx_entry(phalanx, runtime)
    }

    fn publish_weak_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &self, phalanx: &CWeakPhalanx, runtime: &mut Runtime,
    ) -> Option<()> {
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let identity = phalanx.shape().identity();
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        if !phalanx.shape().is_assigned_to_server_region() { return Some(()); }
        let region = self.find_region(phalanx.shape().get_region_id())?;
        let _ = self.send_game_shape_around(region.base(), phalanx.shape(), None, &message);
        Some(())
    }

    fn weak_phalanx(&self, region_id: i32, id: i32) -> Option<&CWeakPhalanx> {
        match self.find_region(region_id)?.base().find_skill_phalanx(id)? {
            SummonedSkillShape::Weak(phalanx) => Some(phalanx),
            _ => None,
        }
    }

    fn weak_caster(&self, region_id: i32, id: i32) -> Option<(i32, ShapeIdentity)> {
        let master = self.weak_phalanx(region_id, id)?.master();
        let source = resolve_region_move_shape(self, region_id, ShapeIdentity {
            object_type: master.master_type,
            id: master.master_id,
            ex_id: CGuid::GUID_INVALID,
        })?.shape();
        Some((source.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID,
            ..source.identity()
        }))
    }

    pub(super) fn apply_weak_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx_id: i32, runtime: &mut Runtime,
    ) -> usize {
        let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { return 0; };
        let cells = phalanx.active_cells();
        let mut applied = 0usize;
        for (x, y) in cells {
            for target in weak_cell_targets(self, region_id, x, y) {
                let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { return applied; };
                if target == phalanx.shape().identity() { continue; }
                let Some(shape) = resolve_state_move_shape(self, region_id, target) else { continue; };
                let target_region = shape.shape().get_region_id();
                let master = phalanx.master();
                if target.object_type == master.master_type && target.id == master.master_id {
                    continue;
                }
                if master.master_type == PLAYER_TYPE {
                    let Some(player) = self.find_player(master.master_id) else { continue; };
                    let source = player.shape().identity();
                    let attackable = match target.object_type {
                        PLAYER_TYPE | MONSTER_TYPE => self.live_skill_target_attackable(target_region, source, target),
                        1100 | 1200 => self.stationary_build_attackable_by_player(source.id, target_region, target),
                        _ => false,
                    };
                    if !attackable { continue; }
                }
                if resolve_state_move_shape(self, region_id, target)
                    .is_none_or(|shape| shape.has_state_by_skill_id(WEAK_STATE_ID))
                { continue; }
                let Some(caster) = self.weak_caster(region_id, phalanx_id) else { continue; };
                let Some(state) = self.weak_phalanx(region_id, phalanx_id).and_then(CWeakPhalanx::state)
                else { continue; };
                if begin_primary_weak_state(
                    self, region_id, target, Some(caster), Some((target_region, target)),
                    state, &mut || runtime.now_milliseconds(),
                ).is_some() {
                    let _ = self.update_move_shape_properties(region_id, target);
                    applied = applied.wrapping_add(1);
                }
            }
        }
        applied
    }

    fn finish_weak_states_in_cell(
        &mut self, region_id: i32, phalanx_id: i32, x: i32, y: i32, destroy_residual: bool,
    ) -> usize {
        let mut ended = 0usize;
        for target in weak_cell_targets(self, region_id, x, y) {
            let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { break; };
            if target == phalanx.shape().identity() { continue; }
            let master = phalanx.master();
            let same_type = target.object_type == master.master_type;
            let same_id = target.id == master.master_id;
            // End исключает совпадение любого компонента Master; предварительный
            // expiry-проход исключает лишь совпадение пары. Это разные условия.
            if (destroy_residual && (same_type || same_id)) || (same_type && same_id) {
                continue;
            }
            let Some((index, key)) = resolve_state_move_shape(self, region_id, target)
                .and_then(|shape| shape.find_state_position(|state| state.state_id() == WEAK_STATE_ID))
            else { continue; };
            if destroy_residual {
                let _ = end_and_destroy_state_at(self, region_id, target, index);
            } else {
                end_move_shape_state(self, region_id, target, key);
            }
            ended = ended.wrapping_add(1);
        }
        ended
    }

    pub(super) fn expire_weak_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx_id: i32, _runtime: &mut Runtime,
    ) -> usize {
        let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { return 0; };
        let cells = phalanx.active_cells();
        let mut ended = 0usize;
        for (x, y) in cells {
            ended = ended.wrapping_add(self.finish_weak_states_in_cell(region_id, phalanx_id, x, y, false));
        }
        ended.wrapping_add(self.end_weak_phalanx(region_id, phalanx_id))
    }

    pub(super) fn end_weak_phalanx(&mut self, region_id: i32, phalanx_id: i32) -> usize {
        let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { return 0; };
        let cells = phalanx.active_cells();
        let mut ended = 0usize;
        for _ in cells {
            let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id) else { return ended; };
            let (Ok(y), Ok(x)) = (phalanx.shape().get_tile_y(), phalanx.shape().get_tile_x())
            else { break; };
            // В отличие от AI, каждый шаг End заново обходит центральную клетку,
            // а не текущие координаты геометрического цикла.
            ended = ended.wrapping_add(self.finish_weak_states_in_cell(region_id, phalanx_id, x, y, true));
        }
        if let Some(SummonedSkillShape::Weak(phalanx)) = self.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_skill_phalanx_mut(phalanx_id))
        {
            phalanx.mark_ended();
        }
        if let Some(phalanx) = self.weak_phalanx(region_id, phalanx_id)
            && phalanx.shape().is_assigned_to_server_region()
            && let Some(region) = self.find_region(phalanx.shape().get_region_id())
        {
            let _ = self.send_shape_exit_around(region.base(), phalanx.shape());
        }
        ended
    }
}
