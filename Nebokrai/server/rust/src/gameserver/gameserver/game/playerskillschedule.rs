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
//! Обычные материализованные атаки наследуют CSkill::DoesTargetEffective
//! (IsAttackAble цели): подтверждено слотом vtable +0x6c классов из
//! CSkillFactory::QuerySkill (0x00469870). Проверка наличия исполнения общая
//! с маршрутом End; неизвестный этому маршруту навык не объявляется начатым
//! или завершённым по одному current_skill_id.
//! NonFun наследует тот же допуск, но его мгновенное исполнение хранится
//! отдельно от владельцев с внешним End.
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
    pub(super) fn materialized_player_skill_active(player_ai: &CPlayerAI, skill_id: u32) -> Option<bool> {
        Some(match skill_id {
            BASE_ATTACK_SKILL_ID => player_ai.base_attack().is_some(),
            BASE_MAGIC_SKILL_ID => player_ai.base_magic().is_some(),
            FIRE_BOLT_SKILL_ID => player_ai.fire_bolt().is_some(),
            FIRE_BALL_SKILL_ID => player_ai.fire_ball().is_some(),
            FIRE_WALL_SKILL_ID => player_ai.fire_wall().is_some(),
            SEVEN_SHOOTING_STAR_SKILL_ID => player_ai.seven_shooting_star().is_some(),
            THUNDER_SLASH_SKILL_ID => player_ai.thunder_slash().is_some(),
            CHAIN_LIGHTNING_SKILL_ID => player_ai.chain_lightning().is_some(),
            THUNDER_BLOW_SKILL_ID => player_ai.thunder_blow().is_some(),
            ITEM_SKILL_2_ID => player_ai.item_skill_2().is_some(),
            PILLAR_SKILL_ID => player_ai.pillar().is_some(),
            RUSH_SKILL_ID => player_ai.rush().is_some(),
            RUSH_2_SKILL_ID => player_ai.rush_2().is_some(),
            ROAR_SKILL_ID => player_ai.roar().is_some(),
            ENERGY_HOLDING_SKILL_ID => player_ai.energy_holding().is_some(),
            INVERSE_CHOPPED_SKILL_ID => player_ai.inverse_chopped().is_some(),
            INFERNOL_SKILL_ID => player_ai.infernol().is_some(),
            THUNDER_BLOW_2_SKILL_ID => player_ai.thunder_blow_2().is_some(),
            MOSOU_SKILL_ID => player_ai.mosou().is_some(),
            GHOST_CUT_SKILL_ID | GHOST_CUT_2_SKILL_ID | GHOST_CUT_3_SKILL_ID => {
                player_ai.ghost_cut().is_some()
            }
            KNIGHT_CUT_SKILL_ID => player_ai.knight_cut().is_some(),
            ARMY_BREAK_SKILL_ID | ARMY_BREAK_2_SKILL_ID => player_ai.army_break().is_some(),
            RAGE_SKILL_ID => player_ai.rage().is_some(),
            RAGE_BREAK_SKILL_ID => player_ai.rage_break().is_some(),
            FURY_SKILL_ID => player_ai.fury().is_some(),
            FLASH_SKILL_ID => player_ai.flash().is_some(),
            SWALLOW_SKILL_ID => player_ai.swallow().is_some(),
            LEAF_CUT_SKILL_ID => player_ai.leaf_cut().is_some(),
            LEAF_CUT_2_SKILL_ID => player_ai.leaf_cut_2().is_some(),
            LEAF_CUT_3_SKILL_ID => player_ai.leaf_cut_3().is_some(),
            JU_CUT_SKILL_ID => player_ai.ju_cut().is_some(),
            LIGHTNING_SWORD_SKILL_ID
            | LIGHTNING_SWORD_2_SKILL_ID
            | LIGHTNING_SWORD_3_SKILL_ID
            | LIGHTNING_SWORD_4_SKILL_ID => player_ai.lightning_sword().is_some(),
            LITTLE_FLASH_SKILL_ID | LITTLE_FLASH_2_SKILL_ID => {
                player_ai.little_flash().is_some()
            }
            LITTLE_STAR_SKILL_ID => player_ai.little_star().is_some(),
            ENERGY_BOLT_SKILL_ID | ZOMBIE_CLAW_SKILL_ID | SNAKE_BOLT_SKILL_ID => {
                player_ai.path_projectile().is_some()
            }
            CHUCK_STONE_SKILL_ID | SKELETON_ARCHERY_SKILL_ID => {
                player_ai.direct_projectile().is_some()
            }
            YUNSHENG_LIGHTNING_SKILL_ID => player_ai.yunsheng_lightning().is_some(),
            CORPSE_PTOMAINE_SKILL_ID => player_ai.corpse_ptomaine().is_some(),
            MONSTER_THORN_SKILL_ID => player_ai.monster_thorn().is_some(),
            SPIDER_MIST_SKILL_ID => player_ai.spider_mist().is_some(),
            SPIDER_WEB_SKILL_ID => player_ai.spider_web().is_some(),
            SPIDER_POISON_SKILL_ID => player_ai.spider_poison().is_some(),
            SUMMON_CORPSE_CANDLE_SKILL_ID | SUMMON_SKELETON_SKILL_ID | SUMMON_SPORE_SKILL_ID | BOSS_FIEND_SUMMON_SKILL_ID => player_ai.summon_creature().is_some(),
            BOSS_BLUE_FURY_SKILL_ID => player_ai.boss_blue_fury().is_some(),
            BOSS_BLUE_QUAKE_SKILL_ID => player_ai.boss_blue_quake().is_some(),
            BOSS_FIEND_PENETRATE_SKILL_ID => player_ai.boss_fiend_penetrate().is_some(),
            SPRITE_BURN_SKILL_ID => player_ai.sprite_burn().is_some(),
            MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID => {
                player_ai.wide_arc_attack().is_some()
            }
            LORD_FAST_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID => player_ai.lord_fast_attack().is_some(),
            MONSTER_BASE_ATTACK_SKILL_ID => player_ai.monster_base_attack().is_some(),
            MONSTER_RANGE_ATTACK_SKILL_ID => player_ai.monster_range_attack().is_some(),
            CHAOS_SPHERE_SKILL_ID => player_ai.chaos_sphere().is_some(),
            LIGHTNING_SKILL_ID => player_ai.lightning().is_some(),
            SEAL_SKILL_ID => player_ai.seal().is_some(),
            YIN_YANG_SKILL_ID => player_ai.yin_yang().is_some(),
            YIN_YANG_2_SKILL_ID => player_ai.yin_yang_2().is_some(),
            GOD_PUNISHMENT_SKILL_ID => player_ai.god_punishment().is_some(),
            GOD_THUNDER_SKILL_ID => player_ai.god_thunder().is_some(),
            GOD_THUNDER_2_SKILL_ID => player_ai.god_thunder_2().is_some(),
            SOUL_COLLECT_SKILL_ID => player_ai.soul_collect().is_some(),
            SOUL_MIRROR_SKILL_ID => player_ai.soul_mirror().is_some(),
            ARCHERY_SKILL_ID => player_ai.archery().is_some(),
            HEARTLESS_ARROW_SKILL_ID => player_ai.heartless_arrow().is_some(),
            HEARTLESS_ARROW_2_SKILL_ID | HEARTLESS_ARROW_3_SKILL_ID => {
                player_ai.heartless_arrow_area().is_some()
            }
            LIGHTING_ARROW_SKILL_ID => player_ai.lighting_arrow().is_some(),
            LIGHTING_ARROW_2_SKILL_ID => player_ai.lighting_arrow_2().is_some(),
            METEOR_ARROW_MASS_SKILL_ID => player_ai.meteor_arrow_mass().is_some(),
            METEOR_ARROW_SKILL_ID => player_ai.meteor_arrow().is_some(),
            RAIN_ARROW_SKILL_ID => player_ai.rain_arrow().is_some(),
            POISON_MOTH_SKILL_ID => player_ai.poison_moth().is_some(),
            BLOOD_ROSE_SKILL_ID => player_ai.blood_rose().is_some(),
            SCORPION_SKILL_ID => player_ai.scorpion().is_some(),
            BOA_LOCK_SKILL_ID => player_ai.boa_lock().is_some(),
            FALLING_STAR_SKILL_ID => player_ai.falling_star().is_some(),
            EXPLOSIVE_ARROW_SKILL_ID | EXPLOSIVE_ARROW_2_SKILL_ID | EXPLOSIVE_ARROW_3_SKILL_ID => {
                player_ai.explosive_arrow().is_some()
            }
            STRIKE_SKILL_ID => player_ai.strike().is_some(),
            YAKSHA_SLASH_SKILL_ID => player_ai.yaksha_slash().is_some(),
            DAUB_POISON_SKILL_ID => player_ai.daub_poison().is_some(),
            IGNITION_SKILL_ID => player_ai.ignition().is_some(),
            KEROSENE_SKILL_ID => player_ai.kerosene().is_some(),
            BLIND_SKILL_ID => player_ai.blind().is_some(),
            CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => player_ai.callosity().is_some(),
            AGILITY_SKILL_ID | AGILITY_2_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID => {
                player_ai.agility_family().is_some()
            }
            HEARTEN_SKILL_ID => player_ai.hearten().is_some(),
            POISON_FOG_SKILL_ID => player_ai.poison_fog().is_some(),
            SNOW_STORM_SKILL_ID => player_ai.snow_storm().is_some(),
            WEAK_SKILL_ID => player_ai.weak().is_some(),
            GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID => player_ai.god_bless().is_some(),
            CURE_SKILL_ID => player_ai.cure().is_some(),
            PROMOTION_SKILL_ID => player_ai.promotion().is_some(),
            PETS_CONTROL_SKILL_ID => player_ai.pets_control().is_some(),
            MONSTER_TAMING_SKILL_ID => player_ai.monster_taming().is_some(),
            KNOCK_OUT_SKILL_ID => player_ai.knock_out().is_some(),
            GIBE_SKILL_ID => player_ai.gibe().is_some(),
            _ if is_heal_skill(skill_id) => (0..4).any(|index| player_ai.heal_family(index).is_some()),
            _ if is_self_shield_skill(skill_id) => {
                materialized_self_shield_active(player_ai, skill_id)
            }
            _ => return None,
        })
    }

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
            id if is_non_fun_skill(id) => (ai.non_fun().is_some(), TargetRule::Attackable),
            id => match Self::materialized_player_skill_active(ai, id) {
                Some(started) => (started, TargetRule::Attackable),
                None => return false,
            },
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
