//! Общие runtime-границы контактного удара и принудительного перемещения.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rush.cpp,
//! rush2.cpp, strike.cpp и boalock.cpp. Создание состояний и порядок эффектов принадлежат
//! навыкам; пустая атака проходит тот же OnBeenAttacked, что обычный урон.

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
            1100 | 1200 => self.apply_direct_player_skill_attack_to_stationary_build(
                master.master_id, region_id, target, attack, runtime,
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
        let actual_region = resolve_state_move_shape(self, region_id, target)?.shape().get_region_id();
        let mut owner = self.take_region_owner(actual_region)?;
        let result = self.force_move_owned_shape(owner.base_mut(), target, x, y, duration_ms);
        self.restore_region_owner(owner);
        result
    }
}
