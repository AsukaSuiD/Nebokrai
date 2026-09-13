//! Общие сообщения, ForceMove, End и допуск региональных призванных форм.
//! Источник: gameserver.exe/GameServer.pdb, appserver/summonshape.cpp
//! и ThunderBlowPhalanx::Begin из appserver/skills/thunderblowphalanx.cpp.
//! CSummonShape отправляет BF604 до базового SetTileXY, без ожидания AI
//! и перестройки spatial membership CMoveShape. Регион берётся из живой
//! формы, не из ключа её хранилища. Порядок пары в BF604 — ID, затем type;
//! он отличается от обычного ForceMove CMoveShape.
//! End сначала отмечает удаление, затем отправляет BF504(type,id,0) вокруг
//! фактической формы; повторный End не подавляется по флагу удаления.
//! Archery, BaseMagic и FireBolt вызывают общий End: только отметка удаления,
//! без немедленного BF504.
//! Одноклеточные HeartLessArrow2/3 и GodPunishment используют один живой AI:
//! абсолютный срок, снимок фактической клетки, свежий допуск и Attack→End
//! для каждого target. End не прерывает снимок; формулы остаются у owners.

use super::*;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::skills::heartlessarrowphalanx2::{
    CHeartlessArrowPhalanx, apply_heartless_arrow_attack,
};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;

impl CGame {
    pub(super) fn heartless_arrow_phalanx(&self, region: i32, id: i32) -> Option<&CHeartlessArrowPhalanx> {
        let SummonedSkillShape::HeartlessArrow(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_single_cell_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.find_region(holder_region)
            .and_then(|owner| owner.base().find_skill_phalanx(id))
        else { return false; };
        let expired = match phalanx {
            SummonedSkillShape::HeartlessArrow(phalanx) => phalanx.expired_at(now),
            SummonedSkillShape::GodPunishment(phalanx) => phalanx.expired_at(now),
            _ => return false,
        };
        if expired {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        if !phalanx.shape().is_assigned_to_server_region() { return true; }
        let region = phalanx.shape().get_region_id();
        let Some(owner) = self.find_region(region) else { return true; };
        let y = phalanx.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = phalanx.shape().get_tile_x().unwrap_or(i32::MIN);
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            x, y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        for target in shapes.into_iter().map(|view| view.identity) {
            let Some(phalanx) = self.find_region(holder_region)
                .and_then(|owner| owner.base().find_skill_phalanx(id))
            else { return false; };
            let identity = phalanx.shape().identity();
            if (target.object_type == identity.object_type && target.id == identity.id)
                || !self.summoned_skill_scan_target_allowed(region, phalanx.master(), target)
            { continue; }
            match phalanx {
                SummonedSkillShape::HeartlessArrow(phalanx) => {
                    let snapshot = phalanx.attack_snapshot();
                    apply_heartless_arrow_attack(self, snapshot, region, target, runtime);
                }
                SummonedSkillShape::GodPunishment(phalanx) => {
                    let snapshot = phalanx.attack_snapshot();
                    snapshot.apply(self, (region, target), false, runtime);
                }
                _ => return false,
            }
            self.end_summoned_shape(holder_region, id);
        }
        true
    }

    /// Summon световой стрелы и дождя стрел сначала вызывает Begin прежних
    /// ThunderBlow из снимка лицевой клетки. Уровень в нём не используется:
    /// совпадение живых координат вызывает полный End, не просто delete-флаг.
    pub(super) fn end_overlapping_thunder_blow(&mut self, region: i32, x: i32, y: i32) {
        let mut shapes = Vec::new();
        if let Some(owner) = self.find_region(region) {
            let _ = owner.base().get_shapes(
                x, y, self.area_width, self.area_height,
                &RegionShapeResolver { game: self, owner }, &mut shapes,
            );
        }
        for shape in shapes {
            if shape.identity.object_type != SUMMON_SHAPE_TYPE { continue; }
            let matched = self.find_region(region)
                .and_then(|owner| owner.base().find_skill_phalanx(shape.identity.id))
                .is_some_and(|existing| matches!(existing, SummonedSkillShape::ThunderBlow(existing)
                    if existing.shape().get_tile_x() == Ok(x)
                        && existing.shape().get_tile_y() == Ok(y)));
            if matched { self.end_summoned_shape(region, shape.identity.id); }
        }
    }

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
        if self.base_projectile_flight(holder_region, id).is_some() {
            self.end_base_projectile(holder_region, id);
            return;
        }
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
