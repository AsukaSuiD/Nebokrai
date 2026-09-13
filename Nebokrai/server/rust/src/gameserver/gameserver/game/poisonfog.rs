//! Обход области ядовитого тумана и координация её игровых владельцев.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfogphalanx.cpp.
//!
//! Список фигур клетки фиксируется один раз, но источник, допуск и состояния
//! читаются при обработке каждой цели. PK предшествует End старого состояния;
//! повторный поиск источника после End не откатывает уже выполненное удаление.
//! Область остаётся в регионе во время callbacks и не заменяется копией.
//! Источник разрешается через региональный GetObject без геометрического
//! view; совпадающий ID из другого региона не подменяет объект клетки.

use super::*;
use crate::gameserver::appserver::skills::curestate::CURE_STATE_SKILL_ID;
use crate::gameserver::appserver::skills::poisonfogphalanx::{poison_fog_targets, CPoisonFogPhalanx};
use crate::gameserver::appserver::skills::poisonfogstate::{
    begin_primary_poison_fog_state, POISON_FOG_STATE_ID,
};
use crate::gameserver::appserver::states::state::{
    end_move_shape_state, resolve_region_move_shape, resolve_state_move_shape,
};

impl CGame {
    pub(crate) fn add_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CPoisonFogPhalanx,
        x: i32,
        y: i32,
        now_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_poison_fog_phalanx(
            phalanx, x, y, self.area_width, self.area_height, now_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(match result {
            Ok(id) => Ok(id),
            Err((block, phalanx)) => {
                // AddShape не отменяет сериализацию. Непринятый объект
                // остаётся жив до этого хвоста и затем безопасно освобождается.
                let _ = self.publish_poison_fog_phalanx_entry(&phalanx, runtime);
                Err(block)
            }
        })
    }

    pub(crate) fn send_poison_fog_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, id: i32, runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.poison_fog_phalanx(region_id, id)?;
        self.publish_poison_fog_phalanx_entry(phalanx, runtime)
    }

    fn publish_poison_fog_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &self, phalanx: &CPoisonFogPhalanx, runtime: &mut Runtime,
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

    fn poison_fog_phalanx(&self, region_id: i32, id: i32) -> Option<&CPoisonFogPhalanx> {
        match self.find_region(region_id)?.base().find_skill_phalanx(id)? {
            SummonedSkillShape::PoisonFog(phalanx) => Some(phalanx),
            _ => None,
        }
    }

    fn poison_fog_caster(&self, region_id: i32, id: i32) -> Option<(i32, ShapeIdentity)> {
        let master = self.poison_fog_phalanx(region_id, id)?.master();
        let identity = ShapeIdentity {
            object_type: master.master_type,
            id: master.master_id,
            ex_id: CGuid::GUID_INVALID,
        };
        let source = resolve_region_move_shape(self, region_id, identity)?.shape();
        Some((source.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID,
            ..source.identity()
        }))
    }

    fn poison_fog_player_target_allowed(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        let attackable = match target.object_type {
            400 | 600 => self.live_skill_target_attackable(region_id, source, target),
            1100 | 1200 => self.stationary_build_attackable_by_player(source.id, region_id, target),
            _ => false,
        };
        if !attackable { return false; }
        let Some(source) = self.find_player(source.id) else { return false; };
        let (Ok(source_y), Ok(source_x)) = (source.shape().get_tile_y(), source.shape().get_tile_x())
        else { return false; };
        let Some(region) = self.find_region(region_id) else { return false; };
        let Ok(security) = region.get_security(source_x, source_y) else { return false; };
        if security == RegionSecurity::SAFE { return false; }
        let Some(target) = resolve_state_move_shape(self, region_id, target) else { return false; };
        let (Ok(target_y), Ok(target_x)) = (target.shape().get_tile_y(), target.shape().get_tile_x())
        else { return false; };
        let Ok(security) = region.get_security(target_x, target_y) else { return false; };
        security != RegionSecurity::SAFE
    }

    pub(super) fn apply_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> usize {
        let Some(phalanx) = self.poison_fog_phalanx(region_id, phalanx_id) else { return 0; };
        let targets = poison_fog_targets(self, region_id, phalanx);
        let mut applied = 0usize;
        for target in targets {
            let Some(phalanx) = self.poison_fog_phalanx(region_id, phalanx_id) else { break; };
            if target == phalanx.shape().identity() { continue; }
            let Some(shape) = resolve_state_move_shape(self, region_id, target) else { continue; };
            let master = phalanx.master();
            if target.object_type == master.master_type && target.id == master.master_id {
                continue;
            }
            if self.base_magic_target_dead(region_id, target)
                || shape.has_state_by_skill_id(CURE_STATE_SKILL_ID)
            { continue; }

            // Отсутствующий игрок-источник не запрещает замену. Если источник
            // не игрок, исходная ветвь также пропускает player PK/security.
            if let Some((_, caster)) = self.poison_fog_caster(region_id, phalanx_id)
                && caster.object_type == PLAYER_TYPE
            {
                if !self.poison_fog_player_target_allowed(region_id, caster, target) { continue; }
                if target.object_type == PLAYER_TYPE {
                    let Some(player) = self.find_player(caster.id) else { continue; };
                    let (Ok(y), Ok(x)) = (player.shape().get_tile_y(), player.shape().get_tile_x())
                    else { continue; };
                    let _ = self.player_on_first_skill_at_position(
                        caster.id, target.id, region_id, x, y, runtime,
                    );
                }
            }
            if let Some((_, key)) = resolve_state_move_shape(self, region_id, target)
                .and_then(|shape| shape.find_state_position(|state| state.state_id() == POISON_FOG_STATE_ID))
            {
                // Это прямой End первого C9: дополнительного удаления остатка
                // слота здесь нет, в отличие от ClearAllStates или Ignition.
                end_move_shape_state(self, region_id, target, key);
            }
            let Some(caster) = self.poison_fog_caster(region_id, phalanx_id) else { continue; };
            let Some(state) = self.poison_fog_phalanx(region_id, phalanx_id).map(CPoisonFogPhalanx::state)
            else { break; };
            if begin_primary_poison_fog_state(
                self, region_id, target, Some(caster), Some((region_id, target)),
                state, &mut || runtime.now_milliseconds(),
            ).is_some() {
                let _ = self.update_move_shape_properties(region_id, target);
                applied = applied.wrapping_add(1);
            }
        }
        applied
    }
}
