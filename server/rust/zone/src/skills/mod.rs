//! Данные живых навыков Zone, которыми временно управляет прежний Game.

mod baseattackruntime; // исполнение CBaseAttack игроком и монстром: стадии, формула, visual, terminal; фасадные швы прежнего CGame.
mod battlefairy; // правила навыков боевого духа (сброс, стоимость, запись).
mod battlefairyattribute; // Check/AI атрибутного октета Po/Yu (0x212..0x219) над hub-швами `battlefairyskill` (порция №6b).
mod battlefairybasemagic; // Check/AI и Summon BFBaseAttack (0x224) над hub-швами (порция №6b).
mod battlefairybasemagicphalanx; // снаряд CBFBaseAttackPhalanx: форма, тики, клиентский снимок и формула (порция №6b).
pub mod battlefairygear; // экипировка, потенциал (0x8FC2A), улучшение (аудит 0x60202/0x60203) и сброс ZHQLS01/ZHJNS01-02 боевой феи у CPlayer: BFPropertyAdd double-apply quirk и полные resolution-оркестрации (порция №7b; hub-трейт `BattleFairyGearHost` прежнего CPlayer).
pub mod battlefairyskill; // координатор BF-семейства: общий вход, End-контракт, visual-таблица 19 тел и hub-трейты (порция №6b).
mod battlefairytransfer; // Check/AI CHuoxieshu/CLingzhishu (порция №6b; точечная BF918 — решение C).
pub mod battlefairysummon; // призыв, следование и гибель/воскрешение боевого духа у CPlayer: SetWarSoulStaus, SummonBF±1, ComputeWarSoulXY, spatial tails и death/revive (порция №7a; view/closure-швы прежнего hub CPlayer).
pub mod blind; // CBlind (0x76): kernel-вход, visual и AddBlindState (порция №6c; hub-швы `selfcast`).
pub mod blindstate; // общий lifecycle 8-байт lock-состояний Blind/Rush/Rush2/BoaLock/KnockOut/KnightCut/SpiderWeb/Seal/Strike (порция №6c).
pub mod callosity; // Check/AI взаимно исключающих CCallosity/CCallosity2 (порция №6c; hub-швы `selfcast`).
pub mod callositystate; // живые replace/restart/AI/End CCallosityState/CCallosityState2 (порция №6c).
mod chaossphere; // движущаяся область CChaosSpherePhalanx и её живая форма.
pub mod cure; // Check/AI CCure, числовое правило и выбор снимаемых состояний CastCure (порция №6a; hub-швы `statecast`).
pub mod curestate; // живые Begin/restart/AI/End CCureState над hub-швами `statecast` (порция №6a).
mod daubpoison; // числовое правило смазки оружия ядом CDaubPoison.
pub mod dash; // рывки: общая геометрия пути, единый visual и контакт Flash/LittleFlash + hub-швы DashSkillGame семейства.
mod directelement; // числовой расчёт прямых элементальных ударов.
pub mod directprojectile; // CChuckStone/CSkeletonArchery (0x19D/0x1A1): Check/AI прямого снаряда буквально + hub-швы делегата.
mod dispatch; // форма цели и снимок ожидающей команды навыка.
mod elementphalanx; // снимок и числовой расчёт элементального удара призванных областей.
pub mod energyholding; // Check/AI накопления энергии CEnergyHolding (порция №6c; hub-швы `selfcast`).
pub mod energyholdingstate; // живые add/consume/restart/End CEnergyHoldingState над швами `selfcast` (порция №6c).
// Исполнение зарегистрированного навыка: typed payload player/BF и полная
// запись реестра; hub-monster payload подключается generic-сваркой (порция 5).
pub mod execution;

mod firewall; // правила призыва и маска области CFireWall.
mod fatalblow; // Check/AI и Summon CFatalBlow (0x21C) над hub-швами `battlefairyskill` (порция №6b).
mod fatalblowphalanx; // снаряд CFatalBlowPhalanx: форма, тики, клиентский снимок и формула (порция №6b).
pub mod flash; // CFlash (0x69): Check/AI рывка, visual, master_info/target_level семейства.
mod fury; // ID состояний, которые Fury снимает перед созданием.
pub mod godbless; // Check/AI CGodBless/CGodBless2 и параметры создания их состояний (порция №6a; clock/install-шов `GodBlessCastRuntime`).
pub mod godblessstate; // живые callbacks CGodBlessState/CGodBlessState2 (порция №6a).
mod godthunder; // окна целей, клиентские поля областей GodThunder/GodThunder2 и живая форма CGodThunderPhalanx.
pub mod heal; // Check/AI квартета CHeal/CHeal2/CSuperHeal/CSuperHeal2 с ID навыков (порция №6a; FREQ машинно 6001, якорь 0x581861).
pub mod healstate; // живые Begin/restart/AI/End состояний периодического лечения (порция №6a).
pub mod hearten; // Check/AI CHearten и параметры нового состояния (порция №6a).
pub mod heartenstate; // живые Begin/restart/AI/End CHeartenState (порция №6a).
mod immediate; // правила цикла immediate-состояний: ID-карта, ветка установки, End-политика и payload.
mod leiming2; // CLeiming2 (0x21B): семейные с CThunder Check/AI и Summon с AddElementAtk-слагаемым (порция T1).
mod lifecycle; // база и стадии живого навыка.
mod lifeshield; // Check/AI CLifeShield (0x220) над hub-швами `battlefairyskill` (порция №6b).
pub mod littleflash; // CLittleFlash/CLittleFlash2 (0x71/0x7F): Check/AI и visual малых рывков.
pub mod littlestar; // CLittleStar (0x1A4): кадры visual, формулы, геометрия пути и правила длительности; hub-оркестрация у делегата.
mod masked_area; // маска неподвижных областей FireWall и YinYang и живая форма MaskedElementPhalanx.
pub mod pathprojectile; // CEnergyBolt/CSnakeBolt/CZombieClaw (0x1A0/0x1A5/0x1A2): Check/AI путевого снаряда буквально + hub-швы делегата.
pub mod pillar; // Check/AI и параметры стойки CPillar (порция №6c; hub-швы `selfcast`).
pub mod pillarstate; // живые toggle/restart/AI/End CPillarState (порция №6c).
mod poisonfog; // данные живой области CPoisonFogPhalanx.
pub mod promotion; // Check/AI CPromotion (порция №6a; hub-швы `statecast`).
pub mod promotionstate; // живой Begin/restart CPromotionState и wire-тип его пакета (порция №6a).
mod projectile; // Прицельные снаряды: общий полёт, элементный контакт, усилитель душами, физический контакт Archery, движение пути FireBall, общий серверный decoder и живые композиты FireBall и GodPunishment.
pub mod roar; // Check/AI и границы обхода клеток CRoar (порция №6c; hub-швы `selfcast`).
pub mod roarstate; // живые replace/restart/AI/End и OnUpdateProperties CRoarState (порция №6c).
pub mod rush; // CRush/CRush2 (0x73/0x7C): Check/AI, AddRushState, visual и типы состояний RushState/Rush2State.
pub mod seal; // CSeal (0x138): Check-гейты и формула keep-time буквально; полёт TargetedProjectile у hub-владельца.
pub mod selfcast; // hub-швы self/zone-кастов семьи blind/energyholding/roar/pillar/callosity/soulmirror (порция №6c; реализация фасадов у прежнего владельца).
mod selfstate; // правила self-state семьи: ветка состояния, текст MP-отказа, создание Agility-состояний.
pub mod skillfactory; // фабричные владельцы и реестр runtime-свойств навыков.
mod snowstorm; // данные области CSnowStormPhalanx и окна выбранных клеток.
pub mod soulmirror; // маска, параметры клетки и живой обход области CSoulMirror (порция №6c; hub-швы `selfcast`).
pub mod spidermist; // CSpiderMist (0x198): Check/AI/Summon буквально, область CSpiderMistPhalanx (маска, AI обхода, entry 0xBF502), hub-швы семьи summoncreatureskill (кластер C Monster 0x19x).
pub mod statecast; // общие hub-швы и wire-кадр visual state-кастов пятёрки и heal-квартета (порция №6a; реализация фасадов у прежнего владельца).
pub mod state; // клиентские контракты состояний: проекция живых записей и runtime-план visual.
pub mod statefactory; // декодирование последовательности состояний из GameSave.
pub mod summoncreatureskill; // семья CSummonSkill (0x19A/0x19B/0x19C/0x1F9): Begin/AI/Summon буквально + hub-швы SummonSkill семьи; поворот внешний (кластер C Monster 0x19x).
mod summonshape; // CSummonShape: общий тип/правило ID и wire-конверт снимков призванных фаланг.
mod thunder; // CThunder (0x21F): семейные Check/AI громовых облаков, круг BF918 (fix №2) и hub-трейт `SummonCloudGame` (порция T1).
mod tianhuo; // CTianhuo (0x21A): часы до reuse, equipment[10] даже при нулевой цене, точечная BF918, поворот U, свёртка старой области (порция T1).
mod visualeffect; // visual-ресурс зарегистрированного навыка.
mod wangsheng; // прямое восстановление HP навыком CWangsheng, без создания WangshengState.
mod weak; // правила области ослабления CWeakPhalanx и срока призыва CWeak.
mod wuxing; // ID и подготовка 24 параметров пяти состояний У-син.
mod yinyang; // параметры и маски областей CYinYang и CYinYang2.
pub mod yunshenglightning; // CYunShengLightning (0x19E): кадры visual, формулы и dispatch-предикаты; hub-оркестрация у делегата.

pub use visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
pub use lifecycle::{SkillExecutionKernel, SkillLifecycle, SkillStage, SkillTermination};
pub use lifecycle::skill_is_restored;
pub use baseattackruntime::{BASE_ATTACK_SKILL_ID, BaseAttackExecutionState,
    BaseAttackExecutionOutcome, BaseAttackContact, BaseAttackGame, BaseAttackMoveShape,
    BaseAttackPkPermissions, BaseAttackPlayer, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    abort_player_base_attack_on_region_change, cancel_player_base_attack,
    execute_owned_monster_base_attack, execute_player_base_attack, publish_base_attack_visual};
pub use battlefairy::{BattleFairyResetItemChange, BattleFairyResetItemLookup,
    BattleFairyResetPreflight, BattleFairyResetSlot,
    BattleFairySkillProperty, battle_fairy_mana_text_cost,
    battle_fairy_reset_item, battle_fairy_reset_item_change,
    battle_fairy_reset_preflight,
    battle_fairy_reset_notice_cost, battle_fairy_skill_level, battle_fairy_skill_id,
    battle_fairy_skill_entry,
    battle_fairy_reset_slot, EQUIPPED_SKILL_PROPERTIES, select_battle_fairy_reset_skill,
    write_battle_fairy_reset_skill};
pub use battlefairyskill::{BattleFairyGame, BattleFairyMoveShape, BattleFairyPkPermissions,
    BattleFairyPlayer, BattleFairySkillOutcome, battle_fairy_master_info,
    cancel_active_battle_fairy_skill, check_battle_fairy_target_states,
    execute_registered_battle_fairy_skill, execute_registered_battle_fairy_state,
    finish_registered_battle_fairy_skill, publish_battle_fairy_visual,
    send_battle_fairy_goods_update, send_battle_fairy_skill_failure,
    summon_user_add_element, summon_user_cch, summon_user_region};
pub use battlefairyattribute::{BattleFairyAttributeSkill,
    definition as battle_fairy_attribute_definition, execute_battle_fairy_attribute};
pub use battlefairytransfer::{BattleFairyTransferKind, HUOXIESHU_SKILL_ID, LINGZHISHU_SKILL_ID,
    execute_battle_fairy_transfer};
pub use dispatch::{BattleFairySkillDispatch, BattleFairySkillRequest,
    BattleFairySkillRequestFacts, PlayerSkillDispatch, PlayerSkillRequest,
    PlayerSkillRequestFacts, SkillTarget, SkillTargetForm};
pub use weak::{WEAK_SKILL_ID, WeakPhalanx, WeakPhalanxTick, weak_lifetime};
pub use poisonfog::{PoisonFogPhalanx, PoisonFogPhalanxTick};
pub use spidermist::{SPIDER_MIST_SKILL_ID, SKILL_USAGE_STATE_PERSIST_TIME,
    SKILL_USAGE_TARGET_AFFECT_FREQUENCY, SpiderMistPhalanx, SpiderMistPhalanxTick,
    apply_spider_mist_targets, cancel_player_spider_mist, execute_owned_spider_mist,
    execute_player_spider_mist, is_player_spider_mist_dispatch, spider_mist_cell_targets,
    spider_mist_entry_message};
pub use summoncreatureskill::{BOSS_FIEND_SUMMON_SKILL_ID, SKILL_USAGE_CAN_BE_BREAKED,
    SKILL_USAGE_CONST, SKILL_USAGE_SUMMONED_CREATURE_ID, SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME,
    SUMMON_CORPSE_CANDLE_SKILL_ID, SUMMON_SKELETON_SKILL_ID, SUMMON_SPORE_SKILL_ID,
    SummonMonsterCast, SummonMonsterFacts, SummonSkillContact, SummonSkillGame,
    SummonSkillOutcome, SummonSkillPlayer, boss_fiend_summoned_creature_usage,
    cancel_player_summon_creature, execute_owned_summon_creature,
    execute_player_summon_creature, is_player_summon_creature_dispatch,
    summon_face_direction, summon_visual_fire_message, summon_visual_start_message};
pub use snowstorm::{SNOW_STORM_SKILL_ID, SNOW_STORM_SCOPE_AREA, SnowStormAttack,
    SnowStormParametersError, SnowStormPhalanx, SnowStormSummonParameters};
pub use firewall::{FIRE_WALL_SKILL_ID, FireWallSummonParameters, fire_wall_scope};
pub use fatalblow::{FatalBlowSummon, execute_battle_fairy_fatal_blow};
pub use fatalblowphalanx::{CFatalBlowPhalanx, FATAL_BLOW_SKILL_ID, FatalBlowPhalanxTick,
    calculate_owned_fatal_blow_attack};
pub use battlefairybasemagic::{BattleFairyBaseMagicSummon, execute_battle_fairy_base_magic};
pub use battlefairybasemagicphalanx::{BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, BattleFairyPhalanxTick,
    CBattleFairyBaseMagicPhalanx, calculate_battle_fairy_base_magic_attack,
    calculate_owned_battle_fairy_base_magic_attack};
pub use lifeshield::{LIFE_SHIELD_SKILL_ID, execute_battle_fairy_life_shield};
pub use thunder::{SummonCloudGame, THUNDER_SKILL_ID, THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY,
    ThunderSummon, execute_battle_fairy_thunder, thunder_base_damage};
pub use leiming2::{LEIMING2_SKILL_ID, LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY, Leiming2Summon,
    execute_battle_fairy_leiming2};
pub use tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY, TianhuoSummon,
    execute_battle_fairy_tianhuo};
pub use masked_area::{MaskedArea, MaskedAreaPulse, MaskedElementPhalanx};
pub use yinyang::{YIN_YANG_SKILL_ID, YIN_YANG_2_SKILL_ID,
    YinYangSummonParameters, yin_yang_scope};
pub use elementphalanx::{ElementPhalanxAttack, ElementSummonLiveField};
pub use godthunder::{CGodThunderPhalanx, GOD_THUNDER_SKILL_ID, GOD_THUNDER_2_SKILL_ID,
    ROUNDED_THUNDER_SCOPE, ROUNDED_THUNDER_SCOPE_SIDE,
    GodThunderParametersError, GodThunderPhalanx, GodThunderSummonParameters};
pub use chaossphere::{CChaosSpherePhalanx, CHAOS_SPHERE_SKILL_ID, ChaosSpherePhalanx,
    ChaosSphereSummonParameters, chaos_sphere_path_length};
pub use soulmirror::{SOUL_MIRROR_SKILL_ID, SoulMirrorArea, SoulMirrorSummonParameters,
    soul_mirror_scope_size, soul_mirror_scope_cell};
pub use cure::{cure_threshold, is_cure_removable_state_id};
pub use daubpoison::{DAUB_POISON_SKILL_ID, daub_poison_keep_time_ms};
pub use fury::is_fury_conflicting_state_id;
pub use pillar::{PILLAR_SKILL_ID, pillar_state_parameters};
pub use roar::{ROAR_SKILL_ID, RoarBounds, roar_bounds};
pub use godbless::GodBlessGains;
pub use hearten::hearten_state;
pub use immediate::{ImmediateStatePayload, ImmediateStatePlacement,
    immediate_ai_sufferer_fallback, immediate_completion_end_argument,
    immediate_state_placement, is_immediate_state_skill};
pub use selfstate::{AGILITY_2_VISUAL_LOOP, PERSISTENT_AGILITY_FAMILY_VISUAL_LOOP,
    SelfStateBranch, agility_state_2, is_self_shield_skill,
    persistent_agility_family_state, self_state_branch, self_state_mana_failure_text};
pub use wuxing::{is_wuxing_skill, prepare_wuxing_parameters};
pub use wangsheng::{WANGSHENG_SKILL_ID, execute_battle_fairy_wangsheng, wangsheng_restored_health};
pub use directelement::{DirectElementProfile, DirectElementLiveField};
pub use summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_prefix,
    encode_related_phalanx_snapshot, next_summon_shape_id};
pub use projectile::{ARCHERY_HIT_MODIFIER_PROPERTY, ArcheryProjectileAttack,
    ArcheryProjectileLiveField, BaseProjectileFlight, CFireBallPhalanx,
    CGodPunishmentPhalanx, ElementProjectileAttack,
    ElementProjectileLiveField, FIRE_BALL_SKILL_ID, FireBallPath,
    GOD_PUNISHMENT_SKILL_ID,
    ProjectileServerSnapshotPrefix, SoulProjectileAmplification};
