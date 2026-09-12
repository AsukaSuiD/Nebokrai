//! Предварительный допуск навыков в CPlayerAI::OnSchedule.
//! Подключённый owner регистрирует вариант Begin один раз: inherited вызывает
//! общий OnBeginSkill здесь, owner сохраняет свой порядок в конкретной логике.
//! Перечень политик отличает подключённого владельца от неизвестного ID,
//! но не хранит состояние Begin/End зарегистрированного экземпляра.
//! Допуск CSkillFactory шире подключённых owners и не подменяет эту границу.
//! Self-запрос сообщения идёт в объектный Attack с самим CPlayer
//! (0x00488E20, WarSoul 0x0048953D..0x00489547). Оба расписания используют
//! общий object_target: self проходит IsDied/DoesTargetEffective как объект,
//! а не отклоняется по форме команды. Point сохраняет свой отдельный вход.
//! Зарегистрированный owner разрешается координатором до допуска: обычный
//! OnSchedule при null выбирает virtual default игрока и повторяет GetCurrentSkill
//! (0x00509A34..0x00509A60), а WarSoul при null GetSkill тихо возвращается
//! (0x005097FB..0x00509804). Ни один вход не создаёт исполнение вне реестра.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/ai/playerai.cpp,
//! VA 0x005098d0; CSkill::DoesTargetEffective — 0x004d82e0.
//! Только IsEnded допускает Begin. До объектного Begin проверяются IsDied
//! и virtual DoesTargetEffective; отказ вызывает EnterCombatState, один
//! RejectUseSkillRequest и OnLoseTarget. У ещё не начатого навыка OnLoseTarget
//! не вызывает End(1), но возвращает default attack после удаления команды.
//! Отказ самого Begin имеет тот же общий RejectUseSkillRequest/OnLoseTarget
//! после конкретной диагностики: объектный хвост 0x00509B0B, координатный
//! 0x00509B32. Достигнутый координатор пока различает отказ Begin и отказ AI
//! по наличию concrete-исполнения до и после вызова owner-а; это не native
//! IsEnded и не полная обработка изменённой базы при отказе без payload.
//! Luvinia PlayerAI использует CheckNextAct/stModuParam вместо этого расписания;
//! отказ старого Begin восстановлен по нашему EXE, без переноса нового AI.
//! Координатный вход не проходит эту проверку; отсутствие и объекта, и
//! координат отклоняется без EnterCombatState. Стадии активного навыка сюда
//! не входят: повторная проверка изменила бы RNG и момент отказа его AI.
//! Обычные материализованные атаки наследуют CSkill::DoesTargetEffective
//! (IsAttackAble цели): подтверждено слотом vtable +0x6c классов из
//! CSkillFactory::QuerySkill (0x00469870). Базовый lifecycle и concrete-данные
//! читаются из зарегистрированного экземпляра; один current_skill_id не
//! доказывает начало или завершение.
//! NonFun наследует тот же допуск, но его мгновенное исполнение хранится
//! отдельно от владельцев с внешним End.
//! Единственная SkillLifecycle хранится в Inactive до concrete-данных,
//! затем перемещается в их kernel. CState::Begin пишет source/target, время
//! и ended=false до OnBeginSkill; неудача не откатывает эту запись.
//! Установка kernel сохраняет ту же базу, без timestamp-маркеров CPlayerAI.
//! Поэтому отсутствие concrete-данных не означает IsEnded: перевод всех
//! materialized-проверок расписания и End на эту границу ещё не завершён.
//! Полный End должен охватывать и начатую базу без payload; один сброс
//! команды или возврат default ID его не заменяет.
//! Выбор TargetRule хранит только игровое различие целей, не второй перечень
//! полей CPlayerAI. Перечень ниже ограничивает подключённые owners, включая
//! NonFun, Swordship и немедленные состояния, но не выбирает поле или тип
//! исполнения. Поддержка concrete End проверяется отдельно его dispatcher-ом;
//! полный registered End и визуальные ресурсы ещё требуют подключения.
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
//! LifeShield, проходит отдельный вход OnScheduleAboutWarSoul (0x00509730).
//! Он проверяет IsDied и DoesTargetEffective до Begin, но не вызывает
//! EnterCombatState. Отказ отправляет байты 4, 2 через
//! RejectUseSkillRequestWarSoul (0x0042e720); OnLoseTargetWarSoul
//! (0x005091b0) у ещё не начатого навыка только возвращает ID 0x224.
//! Для Yujia/Yubing/Yumo/Yufa, LifeShield, Wangsheng, Huoxieshu и Lingzhishu
//! DoesTargetEffective постоянен true; прочие навыки духа наследуют
//! IsAttackAble. Очередь с ненулевыми координатами проходит дальше к Begin,
//! отсутствие и объекта, и координат отклоняется до исполнения.
//! При false из Begin отдельный хвост 0x0050988D также отправляет общий
//! war-soul отказ после конкретного, затем возвращает базовый навык феи.
//! Этот хвост не применяется как повторный отказ уже активного AI;
//! завершение базы после отказавшего Begin не выводится из наличия payload.
//! Даже с координатами war-soul расписание вызывает объектный Begin с null
//! (0x00509861), а не координатную перегрузку; конкретный owner сохраняет
//! собственный порядок отказа этого входа.
//! Во всех 19 Begin после общей базы создаётся собственный CVisualEffect
//! размера 0xC и вызывается BeginVisualEffect(1), до конкретных проверок.
//! Общий Rust-ресурс доступен и отказавшему Begin без payload. Установка
//! concrete-данных переносит уже начатую базу, не вызывает повторный Begin.
//! При отказе координатор завершает захваченный экземпляр вместе с ресурсом,
//! затем отправляет внешний 4,2; внутренние отказы остаются у владельца.
//! Однако после допуска Begin всех 19 навыков феи (0x212..0x224) безусловно
//! вызывает общий 0x00601A50, затем CSkill::Begin (0x004D83E0). Источник в
//! OnScheduleAboutWarSoul 0x00509861 — CPlayer, не визуальный объект type 700:
//! его OnBeginSkill (0x0042CC90) переводит в бой любой ID, кроме 0xA.
//! Поэтому перед конкретными проверками ресурсов время Begin записывается
//! в базу навыка и выполняется EnterCombatState, но при отказе допуска — нет.
//! Очередь остаётся у CPlayerAI, lifecycle и kernel — у зарегистрированного
//! экземпляра CMoveShape, боевые правила — у существующих владельцев целей.
//! Объектные и координатные Begin перечисленных ниже конкретных классов
//! проверены по символам PDB и первым вызовам EXE: без предварительной ветки
//! они проходят общий Begin Attack/State/SummonSkill (объединённые компилятором
//! 0x00601A50/0x005DFC00), затем CSkill::Begin и OnBeginSkill (0x0042CC90).
//! Это относится и к монстровым навыкам, когда их источник — игрок;
//! самостоятельное исполнение AI монстра этот вход не использует.
//! Переход в бой здесь
//! предшествует конкретным проверкам оружия, MP и reuse, но не допуску цели.
//! Владельцы с собственным вызовом EnterCombatState не включены повторно.
//! ItemSkill_2 имеет отдельный Begin, поэтому правило на него не переносится.
//! NonFun использует тот же базовый вызов (0x0050DF50/0x0050E010), хотя
//! его конкретный AI не наносит урон: общий переход в бой всё равно обязателен.
//! Luvinia Server/GameServer/Application/MoveShape.cpp использует одноимённый
//! OnBeginSkill для CNewSkill/пассивных модулей: этот новый контракт не перенесён.

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

#[derive(Clone, Copy, Eq, PartialEq)]
enum PlayerSkillBeginPolicy {
    Inherited,
    Owner,
}

impl CGame {
    pub(super) fn player_skill_begin_pending(&self, player_id: i32, skill_id: u32) -> bool {
        self.materialized_player_skill_active(player_id, skill_id) == Some(false)
    }

    pub(super) fn begin_player_skill_schedule<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
        runtime: &mut Runtime,
    ) {
        let skill_id = dispatch.skill_id();
        let inherited_begin = Self::player_skill_begin_policy(skill_id)
            == Some(PlayerSkillBeginPolicy::Inherited);
        let needs_begin = inherited_begin
            && self.player_skill_begin_pending(player_id, skill_id);
        if needs_begin {
            self.begin_player_skill_with_combat(player_id, dispatch, runtime.now_milliseconds());
        }
    }

    pub(super) fn begin_battle_fairy_skill_schedule<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
        ai: &mut CPlayerAI,
        runtime: &mut Runtime,
    ) {
        if (0x212..=0x224).contains(&dispatch.skill_id())
            && !self.battle_fairy_skill_execution_is_materialized(player_id, ai)
        {
            self.begin_battle_fairy_skill_lifecycle(player_id, dispatch, runtime.now_milliseconds());
            self.enter_player_combat_state(player_id);
            self.finish_player_skill_base_begin(player_id, dispatch.skill_id(), true);
            self.replace_player_skill_visual_effect(
                player_id, dispatch.skill_id(),
                crate::gameserver::appserver::states::visualeffect::SkillVisualEffect::new(
                    crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind::BattleFairy, 1,
                ),
            );
        }
    }

    pub(super) fn reject_battle_fairy_skill_schedule(
        &mut self,
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
        ai: &CPlayerAI,
    ) -> bool {
        if self.battle_fairy_skill_execution_is_materialized(player_id, ai)
            || !(0x212..=0x224).contains(&dispatch.skill_id())
        {
            return false;
        }
        let rejected = match dispatch.object_target() {
            None => matches!(dispatch, BattleFairySkillDispatch::Point { x, y, .. } if x == 0 || y == 0),
            Some(target) => {
                let Some(player) = self.find_player(player_id) else { return false };
                let Some(region_id) = player.server_region_id() else { return false };
                let master = crate::gameserver::appserver::skills::flash::master_info(player);
                let accepts_any_live_target = matches!(dispatch.skill_id(), 0x216..=0x219 | 0x220..=0x223);
                self.base_magic_target_view(region_id, target).is_none()
                    || self.base_magic_target_dead(region_id, target)
                    || (!accepts_any_live_target && !match target.object_type {
                        1100 | 1200 => self.stationary_build_attackable_by_player(player_id, region_id, target),
                        _ => self.owned_player_skill_target_attackable(master, target, region_id),
                    })
            }
        };
        if rejected {
            self.send_battle_fairy_skill_failure(player_id, 2);
        }
        rejected
    }

    fn player_skill_begin_policy(skill_id: u32) -> Option<PlayerSkillBeginPolicy> {
        match skill_id {
            BASE_ATTACK_SKILL_ID
            | ARCHERY_SKILL_ID
            | HEARTLESS_ARROW_SKILL_ID
            | HEARTLESS_ARROW_2_SKILL_ID
            | HEARTLESS_ARROW_3_SKILL_ID
            | LIGHTING_ARROW_SKILL_ID
            | RAIN_ARROW_SKILL_ID
            | BASE_MAGIC_SKILL_ID
            | FIRE_BOLT_SKILL_ID
            | FIRE_BALL_SKILL_ID
            | FIRE_WALL_SKILL_ID
            | SEVEN_SHOOTING_STAR_SKILL_ID
            | THUNDER_SLASH_SKILL_ID
            | CHAIN_LIGHTNING_SKILL_ID
            | THUNDER_BLOW_SKILL_ID
            | PILLAR_SKILL_ID
            | RUSH_SKILL_ID
            | RUSH_2_SKILL_ID
            | ROAR_SKILL_ID
            | ENERGY_HOLDING_SKILL_ID
            | INVERSE_CHOPPED_SKILL_ID
            | INFERNOL_SKILL_ID
            | THUNDER_BLOW_2_SKILL_ID
            | MOSOU_SKILL_ID
            | GHOST_CUT_SKILL_ID
            | GHOST_CUT_2_SKILL_ID
            | GHOST_CUT_3_SKILL_ID
            | ARMY_BREAK_SKILL_ID
            | ARMY_BREAK_2_SKILL_ID
            | RAGE_BREAK_SKILL_ID
            | FURY_SKILL_ID
            | FLASH_SKILL_ID
            | SWALLOW_SKILL_ID
            | LEAF_CUT_SKILL_ID
            | LEAF_CUT_2_SKILL_ID
            | LEAF_CUT_3_SKILL_ID
            | JU_CUT_SKILL_ID
            | LIGHTNING_SWORD_SKILL_ID
            | LIGHTNING_SWORD_2_SKILL_ID
            | LIGHTNING_SWORD_3_SKILL_ID
            | LIGHTNING_SWORD_4_SKILL_ID
            | LITTLE_FLASH_SKILL_ID
            | LITTLE_FLASH_2_SKILL_ID
            | LITTLE_STAR_SKILL_ID
            | ENERGY_BOLT_SKILL_ID
            | ZOMBIE_CLAW_SKILL_ID
            | SNAKE_BOLT_SKILL_ID
            | CHUCK_STONE_SKILL_ID
            | SKELETON_ARCHERY_SKILL_ID
            | YUNSHENG_LIGHTNING_SKILL_ID
            | CORPSE_PTOMAINE_SKILL_ID
            | MONSTER_THORN_SKILL_ID
            | SPIDER_MIST_SKILL_ID
            | SPIDER_WEB_SKILL_ID
            | KNOCK_OUT_SKILL_ID
            | SPIDER_POISON_SKILL_ID
            | PROMOTION_SKILL_ID
            | SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
            | BOSS_BLUE_FURY_SKILL_ID
            | BOSS_BLUE_QUAKE_SKILL_ID
            | BOSS_FIEND_PENETRATE_SKILL_ID
            | SPRITE_BURN_SKILL_ID
            | MACHINERY_STOMP_SKILL_ID
            | LORD_WIDERANGING_ATTACK_SKILL_ID
            | LORD_FAST_ATTACK_SKILL_ID
            | MONSTER_FAST_ATTACK_SKILL_ID
            | MONSTER_BASE_ATTACK_SKILL_ID
            | MONSTER_RANGE_ATTACK_SKILL_ID
            | CHAOS_SPHERE_SKILL_ID
            | LIGHTNING_SKILL_ID
            | SEAL_SKILL_ID
            | YIN_YANG_SKILL_ID
            | YIN_YANG_2_SKILL_ID
            | GOD_PUNISHMENT_SKILL_ID
            | GOD_THUNDER_SKILL_ID
            | GOD_THUNDER_2_SKILL_ID
            | SOUL_COLLECT_SKILL_ID
            | SOUL_MIRROR_SKILL_ID
            | LIGHTING_ARROW_2_SKILL_ID
            | METEOR_ARROW_MASS_SKILL_ID
            | METEOR_ARROW_SKILL_ID
            | POISON_MOTH_SKILL_ID
            | BLOOD_ROSE_SKILL_ID
            | SCORPION_SKILL_ID
            | BOA_LOCK_SKILL_ID
            | FALLING_STAR_SKILL_ID
            | EXPLOSIVE_ARROW_SKILL_ID
            | EXPLOSIVE_ARROW_2_SKILL_ID
            | EXPLOSIVE_ARROW_3_SKILL_ID
            | STRIKE_SKILL_ID
            | YAKSHA_SLASH_SKILL_ID
            | DAUB_POISON_SKILL_ID
            | IGNITION_SKILL_ID
            | KEROSENE_SKILL_ID
            | BLIND_SKILL_ID
            | POISON_FOG_SKILL_ID
            | SNOW_STORM_SKILL_ID
            | WEAK_SKILL_ID
            | GOD_BLESS_SKILL_ID
            | GOD_BLESS_2_SKILL_ID
            | GIBE_SKILL_ID => Some(PlayerSkillBeginPolicy::Inherited),
            HEARTEN_SKILL_ID
            | CURE_SKILL_ID
            | PETS_CONTROL_SKILL_ID
            | MONSTER_TAMING_SKILL_ID
            | ITEM_SKILL_2_ID
            | KNIGHT_CUT_SKILL_ID
            | RAGE_SKILL_ID
            | CALLOSITY_SKILL_ID
            | CALLOSITY_2_SKILL_ID
            | AGILITY_SKILL_ID
            | AGILITY_2_SKILL_ID
            | NATURAL_SKILL_ID
            | RAPTURE_SKILL_ID => Some(PlayerSkillBeginPolicy::Owner),
            _ if is_non_fun_skill(skill_id) => Some(PlayerSkillBeginPolicy::Inherited),
            _ if is_swordship_skill(skill_id) || is_immediate_state_skill(skill_id)
                || is_heal_skill(skill_id) || is_self_shield_skill(skill_id) =>
            {
                Some(PlayerSkillBeginPolicy::Owner)
            }
            _ => None,
        }
    }

    fn materialized_player_skill_active(&self, player_id: i32, skill_id: u32) -> Option<bool> {
        Self::player_skill_begin_policy(skill_id)
            .map(|_| self.player_skill_execution(player_id, skill_id).is_some())
    }

    pub(super) fn reject_player_skill_schedule(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
    ) -> bool {
        let skill_id = dispatch.skill_id();
        if self.materialized_player_skill_active(player_id, skill_id) != Some(false) {
            return false;
        }
        let rule = match skill_id {
            CURE_SKILL_ID => TargetRule::Cure,
            SEAL_SKILL_ID => TargetRule::Monster,
            PROMOTION_SKILL_ID | AGILITY_SKILL_ID | AGILITY_2_SKILL_ID
            | NATURAL_SKILL_ID | RAPTURE_SKILL_ID | CALLOSITY_SKILL_ID
            | CALLOSITY_2_SKILL_ID | BOSS_BLUE_FURY_SKILL_ID | DAUB_POISON_SKILL_ID
            | ENERGY_HOLDING_SKILL_ID | FURY_SKILL_ID | GOD_BLESS_SKILL_ID
            | GOD_BLESS_2_SKILL_ID | HEARTEN_SKILL_ID | METEOR_ARROW_MASS_SKILL_ID
            | PILLAR_SKILL_ID | RAGE_SKILL_ID | RAGE_BREAK_SKILL_ID | ROAR_SKILL_ID
            | SOUL_COLLECT_SKILL_ID | MACHINE_SHIELD_SKILL_ID | MANA_SHIELD_SKILL_ID => TargetRule::Any,
            id if is_heal_skill(id) => TargetRule::Any,
            id if is_swordship_skill(id) || is_immediate_state_skill(id) => TargetRule::Never,
            _ => TargetRule::Attackable,
        };
        let enter_combat = match dispatch.object_target() {
            None => return false,
            Some(target) => {
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
