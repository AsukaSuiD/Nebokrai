//! Данные живых навыков Zone, которыми временно управляет прежний Game.

mod baseattackruntime; // исполнение CBaseAttack игроком и монстром: стадии, формула, visual, terminal; фасадные швы прежнего CGame.
mod battlefairy; // правила навыков боевого духа (сброс, стоимость, запись).
mod chaossphere; // движущаяся область CChaosSpherePhalanx и её живая форма.
mod cure; // числовое правило и выбор снимаемых состояний CCure.
mod daubpoison; // числовое правило смазки оружия ядом CDaubPoison.
pub mod dash; // рывки: общая геометрия пути, единый visual и контакт Flash/LittleFlash + hub-швы DashSkillGame семейства.
mod directelement; // числовой расчёт прямых элементальных ударов.
mod dispatch; // форма цели и снимок ожидающей команды навыка.
mod elementphalanx; // снимок и числовой расчёт элементального удара призванных областей.
// Исполнение зарегистрированного навыка: typed payload player/BF и полная
// запись реестра; hub-monster payload подключается generic-сваркой (порция 5).
pub mod execution;

mod firewall; // правила призыва и маска области CFireWall.
pub mod flash; // CFlash (0x69): Check/AI рывка, visual, master_info/target_level семейства.
mod fury; // ID состояний, которые Fury снимает перед созданием.
mod godbless; // параметры CGodBless/CGodBless2 при создании состояния.
mod godthunder; // окна целей, клиентские поля областей GodThunder/GodThunder2 и живая форма CGodThunderPhalanx.
mod hearten; // параметры нового состояния CHearten.
mod immediate; // правила цикла immediate-состояний: ID-карта, ветка установки, End-политика и payload.
mod lifecycle; // база и стадии живого навыка.
pub mod littleflash; // CLittleFlash/CLittleFlash2 (0x71/0x7F): Check/AI и visual малых рывков.
mod masked_area; // маска неподвижных областей FireWall и YinYang и живая форма MaskedElementPhalanx.
mod pillar; // параметры создаваемой стойки CPillar.
mod poisonfog; // данные живой области CPoisonFogPhalanx.
mod projectile; // Прицельные снаряды: общий полёт, элементный контакт, усилитель душами, физический контакт Archery, движение пути FireBall, общий серверный decoder и живые композиты FireBall и GodPunishment.
mod roar; // границы обхода клеток CRoar.
pub mod rush; // CRush/CRush2 (0x73/0x7C): Check/AI, AddRushState, visual и типы состояний RushState/Rush2State.
mod selfstate; // правила self-state семьи: ветка состояния, текст MP-отказа, создание Agility-состояний.
pub mod skillfactory; // фабричные владельцы и реестр runtime-свойств навыков.
mod snowstorm; // данные области CSnowStormPhalanx и окна выбранных клеток.
mod soulmirror; // маска и параметры клетки CSoulMirror.
mod spidermist; // данные области CSpiderMistPhalanx и её маска.
pub mod state; // клиентские контракты состояний: проекция живых записей и runtime-план visual.
pub mod statefactory; // декодирование последовательности состояний из GameSave.
mod summonshape; // CSummonShape: общий тип/правило ID и wire-конверт снимков призванных фаланг.
mod visualeffect; // visual-ресурс зарегистрированного навыка.
mod wangsheng; // прямое восстановление HP навыком CWangsheng, без создания WangshengState.
mod weak; // правила области ослабления CWeakPhalanx и срока призыва CWeak.
mod wuxing; // ID и подготовка 24 параметров пяти состояний У-син.
mod yinyang; // параметры и маски областей CYinYang и CYinYang2.

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
pub use dispatch::{BattleFairySkillDispatch, BattleFairySkillRequest,
    BattleFairySkillRequestFacts, PlayerSkillDispatch, PlayerSkillRequest,
    PlayerSkillRequestFacts, SkillTarget, SkillTargetForm};
pub use weak::{WEAK_SKILL_ID, WeakPhalanx, WeakPhalanxTick, weak_lifetime};
pub use poisonfog::{PoisonFogPhalanx, PoisonFogPhalanxTick};
pub use spidermist::{SPIDER_MIST_SKILL_ID, SpiderMistPhalanx, SpiderMistPhalanxTick};
pub use snowstorm::{SNOW_STORM_SKILL_ID, SNOW_STORM_SCOPE_AREA, SnowStormAttack,
    SnowStormParametersError, SnowStormPhalanx, SnowStormSummonParameters};
pub use firewall::{FIRE_WALL_SKILL_ID, FireWallSummonParameters, fire_wall_scope};
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
pub use wangsheng::{WANGSHENG_SKILL_ID, wangsheng_restored_health};
pub use directelement::{DirectElementProfile, DirectElementLiveField};
pub use summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_prefix,
    encode_related_phalanx_snapshot, next_summon_shape_id};
pub use projectile::{ARCHERY_HIT_MODIFIER_PROPERTY, ArcheryProjectileAttack,
    ArcheryProjectileLiveField, BaseProjectileFlight, CFireBallPhalanx,
    CGodPunishmentPhalanx, ElementProjectileAttack,
    ElementProjectileLiveField, FIRE_BALL_SKILL_ID, FireBallPath,
    GOD_PUNISHMENT_SKILL_ID,
    ProjectileServerSnapshotPrefix, SoulProjectileAmplification};
