//! Предварительный допуск атак в CPlayerAI::OnSchedule.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/ai/playerai.cpp,
//! VA 0x005098d0; CSkill::DoesTargetEffective — 0x004d82e0.
//! Только IsEnded допускает Begin. До объектного Begin проверяются IsDied
//! и virtual IsAttackAble цели; отказ вызывает EnterCombatState, один
//! RejectUseSkillRequest и OnLoseTarget. У ещё не начатого навыка OnLoseTarget
//! не вызывает End(1), но возвращает default attack после удаления команды.
//! Координатный вход не проходит эту проверку; отсутствие и объекта, и
//! координат отклоняется без EnterCombatState. Стадии активного навыка сюда
//! не входят: повторная проверка изменила бы RNG и момент отказа его AI.
//! Здесь подключены MonsterBase/Range/Fast, LordFast/Wideranging и MachineryStomp:
//! их vtable +0x6c указывает на CSkill::DoesTargetEffective. Особые реализации
//! остальных навыков этим адаптером не подменяются. Хранение очереди и kernel
//! остаётся у CPlayerAI, боевые правила — у существующих владельцев целей.

use super::*;

impl CGame {
    pub(super) fn reject_inherited_attack_schedule(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
        ai: &CPlayerAI,
    ) -> bool {
        let started = match dispatch.skill_id() {
            MONSTER_BASE_ATTACK_SKILL_ID => ai.monster_base_attack().is_some(),
            MONSTER_RANGE_ATTACK_SKILL_ID => ai.monster_range_attack().is_some(),
            MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID => ai.lord_fast_attack().is_some(),
            MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID => ai.wide_arc_attack().is_some(),
            _ => return false,
        };
        if started { return false; }
        let enter_combat = match dispatch {
            PlayerSkillDispatch::Point { .. } => return false,
            PlayerSkillDispatch::SelfTarget { .. } => false,
            PlayerSkillDispatch::Object { target, .. } => {
                let Some(player) = self.find_player(player_id) else { return false };
                let Some(region_id) = player.server_region_id() else { return false };
                let master = crate::gameserver::appserver::skills::flash::master_info(player);
                if self.base_magic_target_view(region_id, target).is_none() {
                    false
                } else {
                    let effective = !self.base_magic_target_dead(region_id, target)
                        && match target.object_type {
                            1100 | 1200 => self.stationary_build_attackable_by_player(player_id, region_id, target),
                            _ => self.owned_player_skill_target_attackable(master, target, region_id),
                        };
                    if effective { return false; }
                    true
                }
            }
        };
        if enter_combat { self.enter_player_combat_state(player_id); }
        let _ = self.send_base_attack_failure(player_id, 2);
        true
    }
}
