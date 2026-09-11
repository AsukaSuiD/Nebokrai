//! Межвладельческая координация ядовитого тумана.
//! Источник: gameserver.exe + GameServer.pdb, исходный owner
//! appserver/skills/poisonfogphalanx.cpp, AI0x005FC040. Player OnFirstSkill
//! (0x005FC2BB/0x005FC2C3) предшествует выбору первого nonnull состояния C9.
//! Direct End0x005FC2FF выполняется без принудительного destructor; затем
//! региональный GetShape/RTTI заново разрешает caster (0x005FC327/0x005FC32B).
//! При его отсутствии продолжается следующая цель: прежний End не отменяется.
//!
//! Применение навыка, формулы состояния и жизненный цикл области принадлежат
//! владельцам навыка. Здесь остаются проверки PK и региона, каноническая
//! замена состояния, пересчёт свойств и фактическая доставка вокруг объекта.
//! CPoisonFogPhalanx::AI: Begin 0x005FC3C0 → push_back 0x005FC3D6 →
//! UpdateProperty 0x005FC3DF; пересчёт выполняется и для живого монстра.
//! Object Begin0x00608420 только создаёт loop1 visual. Его первый BFE03
//! отправляет OnUpdateProperties после append, без дополнительного Begin-пакета.
//! Конструктор0x00607C40 не читает часы; непустой caster даёт одно чтение
//! в base Begin0x005DBD7F после всех guards. Rust заново разрешает target
//! перед этими часами вместо сохранения native указателя через End callback.
//! Успешный Begin представлен payload и существующей metadata/visual арены;
//! cache-record сохраняет полный keepTime без вызова игрового Serialize.

use super::*;
use crate::gameserver::appserver::skills::curestate::CURE_STATE_SKILL_ID;
use crate::gameserver::appserver::skills::poisonfogphalanx::{poison_fog_targets, CPoisonFogPhalanx};
use crate::gameserver::appserver::skills::poisonfogstate::POISON_FOG_STATE_ID;
use crate::gameserver::appserver::states::state::{
    end_move_shape_state, resolve_state_move_shape, resolve_state_move_shape_mut,
};

impl CGame {
    pub(crate) fn add_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: CPoisonFogPhalanx, x: i32, y: i32, now_ms: u32, runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> { let mut owner = self.take_region_owner(region_id)?; let result = owner.base_mut().add_poison_fog_phalanx(phalanx, x, y, self.area_width, self.area_height, now_ms, runtime); self.restore_region_owner(owner); Some(result) }
    pub(crate) fn send_poison_fog_phalanx_entry(&mut self, region_id: i32, id: i32) -> Option<()> { let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(id)?; let SummonedSkillShape::PoisonFog(phalanx) = phalanx else { return None }; let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?; let y = phalanx.shape().get_tile_y().ok()?; let payload = phalanx.encode_client_snapshot()?; let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id); message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?); message.base_mut().add(&payload); message.base_mut().add_char(0); let _ = self.send_shape_position_around(region_id, x, y, &message); Some(()) }
    fn poison_fog_target_attackable(&self, region_id: i32, phalanx: &CPoisonFogPhalanx, target: ShapeIdentity) -> bool {
        let master = phalanx.master();
        let Some(master_player) = self.find_player(master.master_id) else { return false };
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return false };
        let (Ok(mx), Ok(my)) = (master_player.shape().get_tile_x(), master_player.shape().get_tile_y()) else { return false };
        if region.block_at(mx, my) == Some(2) { return false }
        match target.object_type {
            PLAYER_TYPE => self.find_player(target.id).is_some_and(|player| !player.is_dead() && !player.has_state_by_skill_id(CURE_STATE_SKILL_ID) && player.shape().get_tile_x().ok().zip(player.shape().get_tile_y().ok()).is_some_and(|(x, y)| region.block_at(x, y) != Some(2)) && self.player_base_attackable(master.master_id, target.id)),
            MONSTER_TYPE => {
                let Some(monster) = region.find_monster_by_id(target.id) else { return false };
                if monster.hit_points() == 0 || monster.move_shape().has_state_by_skill_id(CURE_STATE_SKILL_ID) { return false }
                let (Ok(x), Ok(y)) = (monster.move_shape().shape().get_tile_x(), monster.move_shape().shape().get_tile_y()) else { return false };
                if region.block_at(x, y) == Some(2) { return false }
                let Some(property) = monster.base_property_key().and_then(|key| self.find_monster_property_by_origin_name(key)) else { return false };
                if !self.monster_attackable_by_player(master.master_id, region_id, property) { return false }
                let target_master = monster.master_info();
                if (monster.is_tamed() || monster.is_carriage(property)) && target_master.master_type == PLAYER_TYPE && target_master.master_id != 0 {
                    return if target_master.master_id == master.master_id { master.permitted_to_kill_criminal != 0 } else { self.player_base_attackable(master.master_id, target_master.master_id) };
                }
                true
            }
            _ => false,
        }
    }
    pub(super) fn apply_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: &CPoisonFogPhalanx,
        runtime: &mut Runtime,
    ) -> usize {
        let mut applied = 0usize;
        for target in poison_fog_targets(self, region_id, phalanx) {
            if !self.poison_fog_target_attackable(region_id, phalanx, target) {
                continue;
            }
            if target.object_type == PLAYER_TYPE {
                let _ = self.player_on_first_skill(
                    phalanx.master().master_id, target.id, Some(region_id), runtime,
                );
            }
            let previous = resolve_state_move_shape(self, region_id, target)
                .and_then(|shape| shape.find_state_position(|state| {
                    state.state_id() == POISON_FOG_STATE_ID
                }));
            if let Some((_, key)) = previous {
                end_move_shape_state(self, region_id, target, key);
            }

            let master = phalanx.master();
            let caster = ShapeIdentity {
                object_type: master.master_type,
                id: master.master_id,
                ex_id: CGuid::GUID_INVALID,
            };
            let Some((caster_region, caster_identity)) = self.find_shape_in_region(region_id, caster)
                .and_then(|shape| resolve_state_move_shape(self, region_id, shape.identity))
                .map(|shape| (shape.shape().get_region_id(), ShapeIdentity {
                    ex_id: CGuid::GUID_INVALID,
                    ..shape.shape().identity()
                }))
            else {
                continue;
            };
            let Some(shape) = resolve_state_move_shape_mut(self, region_id, target)
            else { continue };

            // Непустые caster/sufferer прошли Begin-guards. Единственные часы
            // базы стоят после прежнего End; конструктор и cache их не читают.
            let state = phalanx.state(runtime.now_milliseconds());
            let record = state.encoded_for_install();
            let key = shape.append_applied_state_record(state, &record);
            shape.mark_applied_state_begun(key);
            shape.set_applied_state_user(key, Some((caster_region, caster_identity)));
            let _ = self.update_move_shape_properties(region_id, target);
            applied = applied.wrapping_add(1);
        }
        applied
    }
}
