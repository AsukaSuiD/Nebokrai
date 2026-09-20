//! Межвладельческая координация громового рассечения.
//!
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderslash.cpp
//! и thunderslashphalanx.cpp. Форма остаётся в едином региональном хранилище
//! во время AI и callbacks. Снимок неизменяемых параметров атаки не заменяет
//! живые часы, регион и состояние удаления. Add не определяет отправку BF502:
//! при отказе объект сериализуется и затем безопасно освобождается.

use super::*;
use crate::gameserver::appserver::skills::thunderslashphalanx::{
    CThunderSlashPhalanx, apply_thunder_slash_attack, thunder_slash_target,
};

impl CGame {
    pub(crate) fn spawn_thunder_slash_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CThunderSlashPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_thunder_slash_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => {
                let SummonedSkillShape::ThunderSlash(phalanx) =
                    self.find_region(region_id)?.base().find_skill_phalanx(id)?
                else { return None; };
                phalanx
            }
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let shape = phalanx.shape();
        let identity = shape.identity();
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        if shape.is_assigned_to_server_region() {
            let (x, y) = (shape.get_tile_x().ok()?, shape.get_tile_y().ok()?);
            let _ = self.send_shape_position_around(shape.get_region_id(), x, y, &message);
        }
        Some(())
    }

    fn thunder_slash_phalanx(&self, region_id: i32, id: i32) -> Option<&CThunderSlashPhalanx> {
        let SummonedSkillShape::ThunderSlash(phalanx) =
            self.find_region(region_id)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_thunder_slash_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.thunder_slash_phalanx(region_id, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(region_id, id);
            return true;
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.thunder_slash_phalanx(region_id, id) else { return false; };
        if !phalanx.attack_due_at(now) { return true; }
        let sampled_at_ms = runtime.now_milliseconds();
        let snapshot = {
            let Some(SummonedSkillShape::ThunderSlash(phalanx)) = self.find_region_mut(region_id)
                .and_then(|owner| owner.base_mut().find_skill_phalanx_mut(id))
            else { return false; };
            phalanx.mark_attack_at(sampled_at_ms);
            phalanx.clone()
        };
        if !snapshot.shape().is_assigned_to_server_region() { return true; }
        let actual_region = snapshot.shape().get_region_id();
        if let Some(target) = thunder_slash_target(self, actual_region, &snapshot) {
            apply_thunder_slash_attack(self, &snapshot, actual_region, target, runtime);
        }
        true
    }
}
