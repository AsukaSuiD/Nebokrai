//! Замена и публикация области GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godpunishment.cpp.
//! Регион фиксируется Summon до проверки клетки и снимка свойств. После
//! SetTile берётся один региональный снимок прежних форм: каждый 13A получает
//! свежий уровень и ReplaceAffectRegion, вызывающий полный End при совпадении
//! живых координат. Замена предшествует Add; отказ Add не отменяет encoder.

use super::*;
use crate::gameserver::appserver::skills::godpunishmentphalanx::CGodPunishmentPhalanx;

impl CGame {
    fn god_punishment_phalanx(&self, region: i32, id: i32) -> Option<&CGodPunishmentPhalanx> {
        let SummonedSkillShape::GodPunishment(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(crate) fn spawn_god_punishment_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, instance: RegisteredSkill, region: i32, mut phalanx: CGodPunishmentPhalanx,
        x: i32, y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let owner = self.find_region(region)?;
        phalanx.shape_mut().set_pos_xy_base(x as f32 + 0.5, y as f32 + 0.5);
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            x, y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        for shape in shapes {
            if shape.identity.object_type != SUMMON_SHAPE_TYPE
                || self.god_punishment_phalanx(region, shape.identity.id).is_none()
            { continue; }
            let level = self.registered_skill(instance)?.level();
            if self.god_punishment_phalanx(region, shape.identity.id)
                .is_some_and(|existing| existing.replacement_matches(i32::from(level), x, y))
            {
                self.end_summoned_shape(region, shape.identity.id);
            }
        }
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_god_punishment_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.god_punishment_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }
}
