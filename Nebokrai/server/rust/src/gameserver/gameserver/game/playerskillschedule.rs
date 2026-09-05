//! Предварительный допуск навыков в CPlayerAI::OnSchedule.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/ai/playerai.cpp,
//! VA 0x005098d0; CSkill::DoesTargetEffective — 0x004d82e0.
//! Только IsEnded допускает Begin. До объектного Begin проверяются IsDied
//! и virtual DoesTargetEffective; отказ вызывает EnterCombatState, один
//! RejectUseSkillRequest и OnLoseTarget. У ещё не начатого навыка OnLoseTarget
//! не вызывает End(1), но возвращает default attack после удаления команды.
//! Координатный вход не проходит эту проверку; отсутствие и объекта, и
//! координат отклоняется без EnterCombatState. Стадии активного навыка сюда
//! не входят: повторная проверка изменила бы RNG и момент отказа его AI.
//! Здесь подключены MonsterBase/Range/Fast, LordFast/Wideranging и MachineryStomp:
//! их vtable +0x6c указывает на CSkill::DoesTargetEffective (IsAttackAble цели).
//! Cure (0x005ad590) допускает цель того же типа, что источник, либо монстра-
//! повозку: вызов 0x004e6d30 — CMonster::IsCarriage, не IsTamed. Promotion
//! (0x00568680) требует только ненулевые источник/цель; Seal (0x005a9b00)
//! дополнительно требует type 600. Общая проверка смерти предшествует всем
//! этим правилам. Особые реализации остальных навыков не подменяются.
//! Swordship, WuXing, TaiJi, Origin и три Enlarge используют постоянный false
//! (0x005af9e0). Agility/Natural/Rapture, Callosity и Heal/SuperHeal используют
//! постоянный true (0x005afce0); это не отменяет предварительный IsDied.
//! Тот же true используют BossBlueFury, DaubPoison, EnergyHolding, Fury,
//! GodBless, Hearten, MeteorArrowMass, Pillar, Rage/RageBreak, Roar,
//! SoulCollect и два обычных щита. Независимая очередь боевой феи, включая
//! LifeShield, этим владельцем не обрабатывается.
//! Хранение очереди и kernel
//! остаётся у CPlayerAI, боевые правила — у существующих владельцев целей.

use super::*;
use crate::gameserver::appserver::skills::machineshield::MACHINE_SHIELD_SKILL_ID;
use crate::gameserver::appserver::skills::manashield::MANA_SHIELD_SKILL_ID;

#[derive(Clone, Copy)]
enum TargetRule {
    Attackable,
    Cure,
    Any,
    Never,
    Monster,
}

impl CGame {
    pub(super) fn reject_player_skill_schedule(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
        ai: &CPlayerAI,
    ) -> bool {
        let (started, rule) = match dispatch.skill_id() {
            MONSTER_BASE_ATTACK_SKILL_ID => (ai.monster_base_attack().is_some(), TargetRule::Attackable),
            MONSTER_RANGE_ATTACK_SKILL_ID => (ai.monster_range_attack().is_some(), TargetRule::Attackable),
            MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID => (ai.lord_fast_attack().is_some(), TargetRule::Attackable),
            MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID => (ai.wide_arc_attack().is_some(), TargetRule::Attackable),
            CURE_SKILL_ID => (ai.cure().is_some(), TargetRule::Cure),
            PROMOTION_SKILL_ID => (ai.promotion().is_some(), TargetRule::Any),
            SEAL_SKILL_ID => (ai.seal().is_some(), TargetRule::Monster),
            AGILITY_SKILL_ID | AGILITY_2_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID => (ai.agility_family().is_some(), TargetRule::Any),
            CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => (ai.callosity().is_some(), TargetRule::Any),
            BOSS_BLUE_FURY_SKILL_ID => (ai.boss_blue_fury().is_some(), TargetRule::Any),
            DAUB_POISON_SKILL_ID => (ai.daub_poison().is_some(), TargetRule::Any),
            ENERGY_HOLDING_SKILL_ID => (ai.energy_holding().is_some(), TargetRule::Any),
            FURY_SKILL_ID => (ai.fury().is_some(), TargetRule::Any),
            GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID => (ai.god_bless().is_some(), TargetRule::Any),
            HEARTEN_SKILL_ID => (ai.hearten().is_some(), TargetRule::Any),
            METEOR_ARROW_MASS_SKILL_ID => (ai.meteor_arrow_mass().is_some(), TargetRule::Any),
            PILLAR_SKILL_ID => (ai.pillar().is_some(), TargetRule::Any),
            RAGE_SKILL_ID => (ai.rage().is_some(), TargetRule::Any),
            RAGE_BREAK_SKILL_ID => (ai.rage_break().is_some(), TargetRule::Any),
            ROAR_SKILL_ID => (ai.roar().is_some(), TargetRule::Any),
            SOUL_COLLECT_SKILL_ID => (ai.soul_collect().is_some(), TargetRule::Any),
            MACHINE_SHIELD_SKILL_ID => (ai.machine_shield().is_some(), TargetRule::Any),
            MANA_SHIELD_SKILL_ID => (ai.mana_shield().is_some(), TargetRule::Any),
            id if is_heal_skill(id) => ((0..4).any(|index| ai.heal_family(index).is_some()), TargetRule::Any),
            id if is_swordship_skill(id) => (ai.swordship().is_some(), TargetRule::Never),
            id if is_immediate_state_skill(id) => (ai.immediate_state().is_some(), TargetRule::Never),
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
                        && match rule {
                            TargetRule::Any => true,
                            TargetRule::Never => false,
                            TargetRule::Monster => target.object_type == MONSTER_TYPE,
                            TargetRule::Cure => target.object_type == PLAYER_TYPE
                                || (target.object_type == MONSTER_TYPE && self.find_region(region_id).is_some_and(|region| {
                                    region.base().find_monster_by_id(target.id).is_some_and(|monster| {
                                        monster.base_property_key()
                                            .and_then(|key| self.find_monster_property_by_origin_name(key))
                                            .is_some_and(|properties| monster.is_carriage(properties))
                                    })
                                })),
                            TargetRule::Attackable => match target.object_type {
                                1100 | 1200 => self.stationary_build_attackable_by_player(player_id, region_id, target),
                                _ => self.owned_player_skill_target_attackable(master, target, region_id),
                            },
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
