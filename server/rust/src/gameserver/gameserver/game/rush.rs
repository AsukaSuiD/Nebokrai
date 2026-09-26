//! Общие runtime-границы контактного удара и принудительного перемещения.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rush.cpp,
//! rush2.cpp, strike.cpp, boalock.cpp и appserver/moveshape.cpp.
//! Создание состояний и порядок эффектов принадлежат навыкам; навыковые
//! вызывающие перенесены в Zone `skills/{dash,rush}.rs` и достигают этих
//! границ через швы `DashSkillGame`/`DashSkillContact` (порция №5 «player
//! melee»), сами координационные фасады остаются здесь без изменений. Пустая
//! атака проходит тот же OnBeenAttacked, что обычный урон. ForceMove игрока
//! публикует BF604 перед виртуальным SetTileXY с отменой захвата, затем
//! обращается к свежему AI. Ошибка пространственной записи сохраняется в
//! результате, но не отменяет следующий native Stand.

use super::*;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;

impl CGame {
    /// Прямой virtual OnBeenAttacked(..., false), без IsAttackAble.
    /// MasterInfo сохраняет четыре флага PK до Calculate; отдельные
    /// guard-проверки Normal/Country читают живого игрока.
    pub(crate) fn apply_owned_skill_contact<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        match target.object_type {
            PLAYER_TYPE => self.receive_player_skill_attack(
                master, target.id, region_id, attack, false, runtime,
            ),
            MONSTER_TYPE => self.receive_monster_skill_attack(
                master, target.id, region_id, attack, runtime,
            ),
            1100 | 1200 => self.receive_stationary_build_skill_attack(
                region_id, target, attack, runtime,
            ),
            _ => {}
        }
    }

    pub(crate) fn force_move_skill_target(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        x: i32,
        y: i32,
        duration_ms: u32,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        if target.object_type == PLAYER_TYPE {
            let player = self.find_player(target.id)?;
            if !player.shape().is_assigned_to_server_region() { return Some(Ok(false)); }
            let actual_region = player.shape().get_region_id();
            let owner = self.find_region(actual_region)?;
            let (destination, message) = match player.move_shape().force_move_message(
                owner.base(), x, y, duration_ms,
            ) {
                Ok(plan) => plan,
                Err(error) => return Some(Err(error)),
            };
            if let Err(error) = self.send_move_shape_around(actual_region, target, &message)? {
                return Some(Err(MoveShapeCommandBlock::Coordinate(error)));
            }
            let position = self.set_player_tile_position(target.id, destination.x, destination.y);
            if let Some(player) = self.find_player_mut(target.id) {
                player.player_ai_mut().begin_forced_stand(duration_ms, game_tick_milliseconds());
            }
            return position.map(|result| result.map(|()| true).map_err(MoveShapeCommandBlock::Position));
        }
        let actual_region = resolve_state_move_shape(self, region_id, target)?.shape().get_region_id();
        let mut owner = self.take_region_owner(actual_region)?;
        let result = if matches!(target.object_type, 1100 | 1200) {
            self.force_move_stationary_build(&mut owner, target, x, y, duration_ms)
        } else if target.object_type == MONSTER_TYPE {
            self.force_move_owned_monster(owner.base_mut(), target.id, x, y, duration_ms)
        } else {
            None
        };
        self.restore_region_owner(owner);
        result
    }

    fn force_move_stationary_build(
        &mut self, owner: &mut ServerRegionOwner, target: ShapeIdentity,
        x: i32, y: i32, duration_ms: u32,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        let (build, region) = owner.stationary_build_and_region_mut(target)?;
        let (area_width, area_height) = self.area_dimensions();
        let facts = crate::gameserver::appserver::moveshape::MoveShapePositionFacts {
            current_hit_points: build.hp(),
            figure: build.shape_view().figure,
            current_area: None,
            area_width,
            area_height,
        };
        let around = GameServerAroundRuntime::new(self, &self.session_factory, area_width, area_height)?;
        // У постройки нет CBaseAI: общий ForceMove не добавляет ожидание AI.
        Some(build.move_shape_mut().force_move(Some(region), x, y, duration_ms, facts, &around))
    }
}
