//! Общие сообщения, ForceMove, End и допуск региональных призванных форм.
//! Источник: gameserver.exe/GameServer.pdb, appserver/summonshape.cpp.
//! CSummonShape отправляет BF604 до базового SetTileXY, без ожидания AI
//! и перестройки spatial membership CMoveShape. Регион берётся из живой
//! формы, не из ключа её хранилища. Порядок пары в BF604 — ID, затем type;
//! он отличается от обычного ForceMove CMoveShape.
//! End сначала отмечает удаление, затем отправляет BF504(type,id,0) вокруг
//! фактической формы; повторный End не подавляется по флагу удаления.

use super::*;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;

impl CGame {
    pub(super) fn publish_summoned_shape_entry(&mut self, shape: &CShape, payload: &[u8]) -> Option<()> {
        let identity = shape.identity();
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(payload);
        message.base_mut().add_char(0);
        if shape.is_assigned_to_server_region() {
            let (x, y) = (shape.get_tile_x().ok()?, shape.get_tile_y().ok()?);
            let _ = self.send_shape_position_around(shape.get_region_id(), x, y, &message);
        }
        Some(())
    }

    pub(super) fn end_summoned_shape(&mut self, holder_region: i32, id: i32) {
        let Some(shape) = self.mark_damage_phalanx_deleted(holder_region, id) else { return; };
        if shape.is_assigned_to_server_region()
            && let Some(region) = self.find_region(shape.get_region_id()).map(ServerRegionOwner::base)
        {
            let _ = self.send_shape_exit_around(region, &shape);
        }
    }

    // Сканирование проверяет свежий CMoveShape после предыдущего попадания.
    // У non-player master нет IsAttackAble; HP проверяет отдельный Attack.
    pub(crate) fn summoned_skill_scan_target_allowed(
        &self, region_id: i32, master: MasterInfo, target: ShapeIdentity,
    ) -> bool {
        if (target.object_type == master.master_type && target.id == master.master_id)
            || resolve_state_move_shape(self, region_id, target).is_none()
        { return false; }
        if master.master_type != PLAYER_TYPE { return true; }
        let Some(source) = self.find_player(master.master_id).map(|player| player.shape().identity())
        else { return false; };
        self.live_skill_target_attackable(region_id, source, target)
    }

    pub(super) fn force_move_summoned_shape(
        &mut self, holder_region: i32, id: i32, x: i32, y: i32, duration_ms: u32,
    ) -> bool {
        let Some(shape) = self.find_region(holder_region)
            .and_then(|owner| owner.base().find_skill_phalanx(id)).map(SummonedSkillShape::shape)
        else { return false; };
        if !shape.is_assigned_to_server_region() { return false; }
        let actual_region = shape.get_region_id();
        let Some(region) = self.find_region(actual_region).map(ServerRegionOwner::base)
        else { return false; };
        let width = region.region.width;
        let height = region.region.height;
        let destination_x = if x < 0 { 0 } else if x >= width { width.wrapping_sub(1) } else { x };
        // Native сравнивает Y с высотой, но при выходе подставляет ширину.
        let destination_y = if y < 0 { 0 } else if y >= height { width.wrapping_sub(1) } else { y };
        let (Ok(old_x), Ok(old_y)) = (shape.get_tile_x(), shape.get_tile_y()) else { return false; };
        let identity = shape.identity();
        let mut message = CMessage::new(0x000b_f604);
        message.add_long(identity.id);
        message.add_long(identity.object_type);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_ulong(duration_ms);
        message.add_long(0);
        let _ = self.send_shape_position_around(actual_region, old_x, old_y, &message);
        self.find_region_mut(holder_region).and_then(|owner| {
            owner.base_mut().set_owned_skill_phalanx_tile_position(id, destination_x, destination_y)
        }).is_some()
    }
}
