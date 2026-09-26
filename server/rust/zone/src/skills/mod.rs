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
pub mod bossbluefury; // CBossBlueFury (0x1F7): Check/AI и продув состояний (продув каждого 0x1F7 → новый state → UpdateProperty → End(1)); FIX F3 — машинный полный sweep вместо первого ключа; visual с BYTE-формой отказов (кластер E3; hub-швы `statecast`+`fury`, монстр-вход — hub `monsterattack`).
pub mod bossbluefurystate; // живые Begin/End/Restart/AI и OnUpdateProperties CBossBlueFuryState с швом fightable (кластер E3; данные/кодек — `effects/bossbluefury`).
pub mod callosity; // Check/AI взаимно исключающих CCallosity/CCallosity2 (порция №6c; hub-швы `selfcast`).
pub mod callositystate; // живые replace/restart/AI/End CCallosityState/CCallosityState2 (порция №6c).
mod chaossphere; // движущаяся область CChaosSpherePhalanx, её живая форма и тело Summon (порция T5).
pub mod corpsecandleblasting; // CCorpseCandleBlasting (0x194): execute_owned буквально (маска x+3y с дырой в центре, X→Y, 600→600 внутри Attack, BF60B-кадр, stage-for-delete, death-скрипт); FIX F1 — MIN/MAX 20008/20009 и hit 20001 из Calc 0x582EA0 (кластер D Monster 0x19x; hub `monsterattack` для A2 + фасады скрипта/удаления).
pub mod corpseptomaine; // CCorpsePtomaine (0x19F): обе ветви буквально, полный 3×3, MP-fаза player, AddState-замена первого 0x191; FIX F2 — player-scan без allowlist типов (AI 0x53A230) (кластер D; hub `monsterattack` + общая арена `spiderpoison`).
pub mod cure; // Check/AI CCure, числовое правило и выбор снимаемых состояний CastCure (порция №6a; hub-швы `statecast`).
pub mod curestate; // живые Begin/restart/AI/End CCureState над hub-швами `statecast` (порция №6a).
pub mod daubpoison; // CDaubPoison (0xDF): правило срока и apply с заменой первого непустого 0xDF-слота (ctor keep только из Query(10002), Begin(U,U)); скелет Check/AI — hub `selfstatecast` (граница D), отклонение не-player в AI — подтверждённое сознательное (кластер D, порция D4).
pub mod daubpoisonstate; // живые Begin/restart/update/End CDaubPoisonState над hub `statecast`; данные/кодек — `effects/daubpoison` (кластер D, порция D4).
pub mod dash; // рывки: общая геометрия пути, единый visual и контакт Flash/LittleFlash + hub-швы DashSkillGame семейства.
mod directelement; // числовой расчёт прямых элементальных ударов.
pub mod directprojectile; // CChuckStone/CSkeletonArchery (0x19D/0x1A1): Check/AI прямого снаряда буквально + hub-швы делегата.
mod dispatch; // форма цели и снимок ожидающей команды навыка.
mod elementphalanx; // снимок и числовой расчёт элементального удара призванных областей; живое применение попадания и проход боевых духов (порция T5).
pub mod energyholding; // Check/AI накопления энергии CEnergyHolding (порция №6c; hub-швы `selfcast`).
pub mod energyholdingstate; // живые add/consume/restart/End CEnergyHoldingState над швами `selfcast` (порция №6c).
// Исполнение зарегистрированного навыка: typed payload player/BF и полная
// запись реестра; hub-monster payload подключается generic-сваркой (порция 5).
pub mod execution;

mod firewall; // правила призыва и маска области CFireWall.
mod fatalblow; // Check/AI и Summon CFatalBlow (0x21C) над hub-швами `battlefairyskill` (порция №6b).
mod fatalblowphalanx; // снаряд CFatalBlowPhalanx: форма, тики, клиентский снимок и формула (порция №6b).
pub mod flash; // CFlash (0x69): Check/AI рывка, visual, master_info/target_level семейства.
pub mod fury; // общая RP-подготовка CFury/CRageBreak (check/расход RP, свип девяти id) и RP-шов RageCastPlayer; сам CFury остаётся у прежнего владельца (хелперы размещены здесь порцией T4).
pub mod godbless; // Check/AI CGodBless/CGodBless2 и параметры создания их состояний (порция №6a; clock/install-шов `GodBlessCastRuntime`).
pub mod godblessstate; // живые callbacks CGodBlessState/CGodBlessState2 (порция №6a).
mod godthunder; // окна целей, клиентские поля областей GodThunder/GodThunder2, живая форма CGodThunderPhalanx и тела их Summon (порция T5).
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
pub mod monsterbasedispatch; // диспетчерский костяк CMonsterBaseAttack: select/change навыка и продолжение cast из OnFighting; реестр исполнителей остаётся hub-швом (кластер A1 Monster 0x19x).
pub mod monsterattack; // общая доставка удара боевых навыков монстров: допуск целей, клеточный resolver 400/500/600/1100/1200, снимок цели и применение попадания; hub-трейты `MonsterCombat*` (кластер A2).
pub mod monsterbaseattack; // CMonsterBaseAttack (0x2bd): player-путь Check/AI и машинная база семьи (500-skip, IsAttackAble-вирт, IncreaseRp, max(max-min,0)+1, записи 1/3/4, weapon-фактор vt+0x184, dyn-CPlayer crit); End без movement-restore; монстр-вход — hub прежнего dispatcher; FIX B1 — End(1)+reuse при мёртвой цели mid-cast (якорь 0x114820) (кластер A2).
pub mod monsterfastattack; // CMonsterFastAttack (0x2d1): фазы +0x50/+0x54/+0x58, кумулятивные сроки 15001/15002, MP только player, двойной Attack с End(1), calc max(max-min,0)+1 (кластер A2).
pub mod monsterrangeattack; // CMonsterRangeAttack (0x2ef): Check (player-only MP + молчаливый нулевой cost), AI без поворота, маска 7×7 (x+7y, центр −3, dedup после hit), calc trunc(unsigned(EM)×0.01f×EC) (кластер A2).
pub mod monsterthorn; // CMonsterThorn (0x197): Check/AI/Attack/Calc и shared End 0x146090; обязательный второй RNG crit-roll (vt+0x114 ≡ 0 у монстра); монстр-вход буквально (кластер A2).
pub mod monstertaming; // CMonsterTaming (0xd4): player-путь приручения; FIX T1 — нулевой MP-cost → молчаливый reject (jbe→ret0 0x57C18B), FIX T2 — терминальный кадр {0xBFE01,0,2} после каждого CheckCastCondition-отказа всех трёх Begin (кластер A2).
pub mod pathprojectile; // CEnergyBolt/CSnakeBolt/CZombieClaw (0x1A0/0x1A5/0x1A2): Check/AI путевого снаряда буквально + hub-швы делегата.
pub mod pillar; // Check/AI и параметры стойки CPillar (порция №6c; hub-швы `selfcast`).
pub mod pillarstate; // живые toggle/restart/AI/End CPillarState (порция №6c).
pub mod poisonarrow; // CPoisonArrow (0x21E): Check/AI и позднее наложение яда буквально; общая для стрел 0x21D/0x21E обёртка `execute_periodic_battle_fairy_arrow` (кластер D, порция D5; hub `battlefairyskill` + швы арены ядов/PK).
mod poisonfog; // данные и живая форма области CPoisonFogPhalanx; тело Summon (порция T5).
pub mod poisonfogstate; // живые Begin/update/restart/AI/End CPoisonFogState (порция T5).
pub mod poisonmoth; // CPoisonMoth (0xCF): Check/AI поклеточного выстрела буквально (квазнота MAX+1 против MAX в Check, двойной visual(3), одна клетка за тик, visual-target перед ударом, (0,0)-гейт); швы семей `rangedweaponcast`/`crossbowattack` (кластер D, порция D6).
pub mod promotion; // Check/AI CPromotion (порция №6a; hub-швы `statecast`).
pub mod promotionstate; // живой Begin/restart CPromotionState и wire-тип его пакета (порция №6a).
mod projectile; // Прицельные снаряды: общий полёт, элементный контакт, усилитель душами, физический контакт Archery, движение пути FireBall, общий серверный decoder и живые композиты FireBall и GodPunishment.
pub mod ragebreak; // CRageBreak (0x6E): Check/AI и порядок состояний (End+dtor прежнего 0x6E → новый state → свип девяти id → только-End первого 0x131 → CCureState → UpdateProperty → End(1)); visual с DWORD-формой mode 8 (порция T4; hub-швы `statecast`+`fury`).
pub mod ragebreakstate; // живые Begin×3/End/Restart/AI и пересчёт CRageBreakState, общие AttackGain-callbacks семьи и consume для ThunderSlash (порция T4; hub-шов `statecast` + AttackGain-фасады CGame).
pub mod roar; // Check/AI и границы обхода клеток CRoar (порция №6c; hub-швы `selfcast`).
pub mod roarstate; // живые replace/restart/AI/End и OnUpdateProperties CRoarState (порция №6c).
pub mod rush; // CRush/CRush2 (0x73/0x7C): Check/AI, AddRushState, visual и типы состояний RushState/Rush2State.
pub mod seal; // CSeal (0x138): Check-гейты и формула keep-time буквально; полёт TargetedProjectile у hub-владельца.
pub mod selfcast; // hub-швы self/zone-кастов семьи blind/energyholding/roar/pillar/callosity/soulmirror (порция №6c; реализация фасадов у прежнего владельца).
mod selfstate; // правила self-state семьи: ветка состояния, текст MP-отказа, создание Agility-состояний.
pub mod skillfactory; // фабричные владельцы и реестр runtime-свойств навыков.
mod snowstorm; // данные и живая форма области CSnowStormPhalanx; тело Summon и применение окна (порция T5).
pub mod soulmirror; // маска, параметры клетки и живой обход области CSoulMirror (порция №6c; hub-швы `selfcast`).
pub mod spiderpoison; // CSpiderPoison (0x191): Check/AI/Attack/Calc и позднее наложение яда буквально поверх hub `baseattackruntime`; общая арена ядовой линии `SpiderPoisonStateArena` (Cure-факт, замена первого 0x191) для кластера D; обвязка stateskill остаётся hub прежнего пакета (кластер D Monster 0x19x).
pub mod spidermist; // CSpiderMist (0x198): Check/AI/Summon буквально, область CSpiderMistPhalanx (маска, AI обхода, entry 0xBF502), hub-швы семьи summoncreatureskill (кластер C Monster 0x19x).
pub mod statecast; // общие hub-швы и wire-кадр visual state-кастов пятёрки и heal-квартета (порция №6a; реализация фасадов у прежнего владельца).
pub mod state; // клиентские контракты состояний: проекция живых записей и runtime-план visual.
pub mod statefactory; // декодирование последовательности состояний из GameSave.
pub mod summoncreatureskill; // семья CSummonSkill (0x19A/0x19B/0x19C/0x1F9): Begin/AI/Summon буквально + hub-швы SummonSkill семьи; поворот внешний (кластер C Monster 0x19x).
mod summonshape; // CSummonShape: общий тип/правило ID и wire-конверт снимков призванных фаланг.
mod thunder; // CThunder (0x21F): семейные Check/AI громовых облаков, круг BF918 (fix №2) и hub-трейт `SummonCloudGame` (порция T1).
mod thunder2phalanx; // живая область CLeimingPhalanx2: одна активная ячейка, expiry-attack, собственные Replace/AddTo/Decord (порция T2).
pub mod thunderblow; // CThunderBlow (0x13F) и его живая область CThunderBlowPhalanx: Begin/Check/AI/Summon и формула; FIX порции T3 — MP/поворот до повторной дальности (якорь 0x17A520) (порция T3).
pub mod thunderblow2; // CThunderBlow2 (0x14D): Check/AI с отбрасыванием/контактом по impactattack-швам и wire-visual 0xBFE01 modes 0/1/3 (порция T3; visual слит из thunderblow2visual — один исходный thunderblow2.cpp).
pub mod thunderfirephalanx; // форма CThunderFirePhalanx (0x322) предметного CItemSkill_2: путь с разовым ForceMove, calc с GetWeaponModifier; FIX #2 — idx>=count завершает форму только в due-ветке с регионом (якорь 0x5E1970) (порция T4).
mod thunderphalanx; // живая область CThunderPhalanx: 49-ячеечные окна, три часа, wire со счётчиком окон и calc с оружейным швом (порция T2).
pub mod thunderslash; // CThunderSlash (0x72) и слитый visual CThunderSlashEffect (один cpp): Check оружия/MP/RP, двухфазный AI с consume первого 0x6E без RTTI/ended-фильтра, Summon свежей таблицы (порция T4; hub-швы поверх `statecast`).
pub mod thunderslashphalanx; // форма CThunderSlashPhalanx: три часа со штампом freq до разрешения региона, цель собственной клетки, Attack с IncreaseRp(1,0); FIX #1 — Calc info[+0] := instance-id [this+8] (якорь 0x5F77B0) (порция T4).
mod tianhuo; // CTianhuo (0x21A): часы до reuse, equipment[10] даже при нулевой цене, точечная BF918, поворот U, свёртка старой области (порция T1).
mod tianhuophalanx; // живая область CTianhuoPhalanx: скан клетки каждый проход, End→BF504, x87-calc (порция T2).
mod visualeffect; // visual-ресурс зарегистрированного навыка.
mod wangsheng; // прямое восстановление HP навыком CWangsheng, без создания WangshengState.
mod weak; // правила и живая форма CWeakPhalanx; тело Summon CWeak (порция T5).
pub mod weakstate; // живые Begin/update/restart/AI/End и смена региона CWeakState (порция T5).
mod wuxing; // ID и подготовка 24 параметров пяти состояний У-син.
mod yinyang; // параметры и маски областей CYinYang и CYinYang2; тела их Summon (порция T5).
pub mod yunshenglightning; // CYunShengLightning (0x19E): кадры visual, формулы и dispatch-предикаты; hub-оркестрация у делегата.
pub mod zonalcast; // hub-швы и скелет Begin/Check/AI/visual/End областных призывов Weak/PoisonFog/SnowStorm/YinYang[2]/GodThunder[2]/FireWall/ChaosSphere/SoulMirror (порция T5).

pub use visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
pub use lifecycle::{SkillExecutionKernel, SkillLifecycle, SkillStage, SkillTermination};
pub use lifecycle::skill_is_restored;
pub use baseattackruntime::{BASE_ATTACK_SKILL_ID,
    BaseAttackExecutionOutcome, BaseAttackContact, BaseAttackGame, BaseAttackMoveShape,
    BaseAttackPkPermissions, BaseAttackPlayer, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    abort_player_base_attack_on_region_change, cancel_player_base_attack,
    execute_owned_monster_base_attack, execute_player_base_attack, publish_base_attack_visual};
pub use battlefairy::{BattleFairyResetItemChange, BattleFairyResetItemLookup,
    BattleFairyResetPreflight,
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
pub use weak::{WEAK_SKILL_ID, WeakPhalanx, WeakPhalanxTick};
pub use poisonfog::PoisonFogPhalanxTick;
pub use spidermist::{SPIDER_MIST_SKILL_ID, SKILL_USAGE_STATE_PERSIST_TIME,
    SKILL_USAGE_TARGET_AFFECT_FREQUENCY, SpiderMistPhalanx, SpiderMistPhalanxTick,
    apply_spider_mist_targets, cancel_player_spider_mist, execute_owned_spider_mist,
    execute_player_spider_mist, is_player_spider_mist_dispatch, spider_mist_cell_targets,
    spider_mist_entry_message};
pub use summoncreatureskill::{BOSS_FIEND_SUMMON_SKILL_ID, SKILL_USAGE_CAN_BE_BREAKED,
    SKILL_USAGE_CONST, SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME,
    SUMMON_CORPSE_CANDLE_SKILL_ID, SUMMON_SKELETON_SKILL_ID, SUMMON_SPORE_SKILL_ID,
    SummonMonsterCast, SummonMonsterFacts, SummonSkillContact, SummonSkillGame,
    SummonSkillOutcome, SummonSkillPlayer,
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
    CBattleFairyBaseMagicPhalanx, calculate_owned_battle_fairy_base_magic_attack};
pub use lifeshield::{LIFE_SHIELD_SKILL_ID, execute_battle_fairy_life_shield};
pub use thunder::{SummonCloudGame, THUNDER_SKILL_ID, THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY,
    ThunderSummon, execute_battle_fairy_thunder, thunder_base_damage};
pub use leiming2::{LEIMING2_SKILL_ID, LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY, Leiming2Summon,
    execute_battle_fairy_leiming2};
pub use tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY, TianhuoSummon,
    execute_battle_fairy_tianhuo};
pub use thunderphalanx::{CThunderPhalanx, ThunderPhalanxGame, ThunderPhalanxTick,
    calculate_owned_thunder_attack};
pub use thunder2phalanx::{CLeimingPhalanx2, Leiming2PhalanxTick,
    calculate_owned_leiming2_attack};
pub use tianhuophalanx::{CTianhuoPhalanx, TianhuoPhalanxTick, calculate_owned_tianhuo_attack};
pub use masked_area::{MaskedAreaPulse, MaskedElementPhalanx};
pub use yinyang::{YIN_YANG_SKILL_ID, YIN_YANG_2_SKILL_ID,
    YinYangSummonParameters, yin_yang_scope};
pub use elementphalanx::{ElementPhalanxAttack, ElementSummonLiveField};
pub use godthunder::{CGodThunderPhalanx, GOD_THUNDER_SKILL_ID, GOD_THUNDER_2_SKILL_ID,
    ROUNDED_THUNDER_SCOPE, ROUNDED_THUNDER_SCOPE_SIDE};
pub use chaossphere::{CChaosSpherePhalanx, CHAOS_SPHERE_SKILL_ID};
pub use soulmirror::SOUL_MIRROR_SKILL_ID;
pub use cure::is_cure_removable_state_id;
pub use daubpoison::{DAUB_POISON_SKILL_ID, daub_poison_keep_time_ms};
pub use fury::is_fury_conflicting_state_id;
pub use pillar::PILLAR_SKILL_ID;
pub use roar::{ROAR_SKILL_ID, roar_bounds};
pub use godbless::GodBlessGains;
pub use hearten::hearten_state;
pub use immediate::{ImmediateStatePayload, ImmediateStatePlacement,
    immediate_ai_sufferer_fallback, immediate_completion_end_argument,
    immediate_state_placement, is_immediate_state_skill};
pub use selfstate::{AGILITY_2_VISUAL_LOOP, PERSISTENT_AGILITY_FAMILY_VISUAL_LOOP,
    SelfStateBranch, agility_state_2, is_self_shield_skill,
    persistent_agility_family_state, self_state_branch, self_state_mana_failure_text};
pub use wuxing::{is_wuxing_skill, prepare_wuxing_parameters};
pub use wangsheng::{WANGSHENG_SKILL_ID, execute_battle_fairy_wangsheng};
pub use directelement::{DirectElementProfile, DirectElementLiveField};
pub use summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_prefix,
    encode_related_phalanx_snapshot, next_summon_shape_id};
pub use projectile::{ARCHERY_HIT_MODIFIER_PROPERTY, ArcheryProjectileAttack,
    ArcheryProjectileLiveField, BaseProjectileFlight, CFireBallPhalanx,
    CGodPunishmentPhalanx, ElementProjectileAttack,
    ElementProjectileLiveField, FIRE_BALL_SKILL_ID,
    GOD_PUNISHMENT_SKILL_ID, SoulProjectileAmplification};

// Порция T5 «zonalcast-хаб»: скелет и швы, тела Summon владельцев,
// композиты областей и применение элементных ударов.
pub use zonalcast::{ZonalCastAiOutcome, ZonalCastContact, ZonalCastGame,
    ZonalCastMoveShape, ZonalCastPathBlock, ZonalCastPlayer, ZonalCastPropertyTarget,
    check_zonal_cast, is_zonal_cast_skill, prepare_element_summon, publish_zonal_cast_visual,
    run_zonal_cast_ai, zonal_cast_resolved_user};
pub use weak::{CWeakPhalanx, summon_weak};
pub use poisonfog::{CPoisonFogPhalanx, summon_poison_fog};
pub use snowstorm::{CSnowStormPhalanx, apply_snow_storm_attack, summon_snow_storm};
pub use yinyang::summon_yin_yang;
pub use godthunder::summon_god_thunder;
pub use chaossphere::summon_chaos_sphere;
pub use elementphalanx::{apply_element_phalanx_attack, apply_element_phalanx_war_soul};

// Кластер A2 «семьи боевых навыков монстров»: hub-трейты доставки и
// перенесённые тела навыков 0x2bd/0x2d1/0x2ef/0x197/0xd4.
pub use monsterattack::{MonsterCombatCast, MonsterCombatContact, MonsterCombatFacts,
    MonsterCombatGame, MonsterCombatOutcome, MonsterCombatPlayer, MonsterShapeFacts,
    MonsterTamingTarget, OwnedMonsterAttackTarget,
    apply_owned_monster_attack_hit, end_owned_monster_skill_without_reuse,
    finish_owned_monster_attack_impact, monster_attack_cell_candidates,
    resolve_owned_monster_attack_target};
pub use monsterbaseattack::{MONSTER_BASE_ATTACK_SKILL_ID, execute_player_monster_base_attack,
    finish_player_monster_base_attack, is_player_monster_base_attack};
pub use monsterfastattack::{MONSTER_FAST_ATTACK_SKILL_ID, SKILL_USAGE_FIRST_TIME,
    SKILL_USAGE_SECOND_TIME, fast_attack_fire_message};
pub use monsterrangeattack::{MONSTER_RANGE_ATTACK_SKILL_ID, MonsterRangeAttackDispatch,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, begin_owned_monster_range_cast,
    calculate_monster_range_attack, execute_owned_monster_range_target,
    execute_player_monster_range_attack, finish_player_monster_range_attack,
    prepare_owned_monster_range_cast, range_attack_cell_candidates, range_attack_fire_message,
    range_attack_scope_cells};
pub use monsterthorn::{MONSTER_THORN_SKILL_ID, cancel_player_monster_thorn,
    complete_player_monster_thorn, execute_owned_monster_thorn, execute_player_monster_thorn,
    is_player_monster_thorn_dispatch};
pub use monstertaming::{MONSTER_TAMING_SKILL_ID, cancel_player_monster_taming,
    complete_player_monster_taming, execute_player_monster_taming};

// Кластер D «трупная/ядовая state-линия»: hub-фасады и перенесённые тела
// навыков 0x191 (stateskill-обвязка у прежнего hub), 0x194 и 0x19F (порции
// D1–D3) и продолжение 0xDF (+state), 0x21E, 0xCF (порции D4–D6; скелет
// 0xDF — hub `selfstatecast`, BF-координатор 0x21E — hub `battlefairyskill`,
// удар/формула 0xCF — швы `rangedweaponcast`/`crossbowattack`); FIX F1
// (MIN/MAX/hit ключи Calc 0x582EA0) и FIX F2 (player-scan без allowlist)
// зафиксированы в шапках владельцев.
pub use spiderpoison::{SPIDER_POISON_SKILL_ID, SpiderPoisonBeginTarget, SpiderPoisonGame,
    SpiderPoisonMoveShape, SpiderPoisonStateArena, check_spider_poison_cast,
    execute_spider_poison_ai, is_player_spider_poison_dispatch};
pub use corpsecandleblasting::{CORPSE_CANDLE_BLASTING_SKILL_ID, CorpseCandleContact,
    CorpseCandleGame, corpse_candle_death_message, corpse_candle_fire_message,
    corpse_candle_start_message, execute_owned_corpse_candle_blasting};
pub use corpseptomaine::{CORPSE_PTOMAINE_SKILL_ID, CorpsePtomaineContact, CorpsePtomaineGame,
    CorpsePtomaineOutcome, cancel_player_corpse_ptomaine, corpse_ptomaine_fire_message,
    corpse_ptomaine_start_message, execute_owned_corpse_ptomaine, execute_player_corpse_ptomaine,
    is_player_corpse_ptomaine_dispatch};
pub use daubpoison::apply_daub_poison;
pub use daubpoisonstate::{begin_primary_daub_poison_state, end_daub_poison_state,
    restart_daub_poison_state, update_daub_poison_state};
pub use poisonarrow::{ArrowEffect, POISON_ARROW_SKILL_ID, PoisonArrowContact, PoisonArrowState,
    PoisonArrowStateArena, execute_battle_fairy_poison_arrow, execute_periodic_battle_fairy_arrow};
pub use poisonmoth::{POISON_MOTH_SKILL_ID, PoisonMothAiOutcome, PoisonMothContact, PoisonMothGame,
    PoisonMothMoveShape, check_poison_moth_cast, execute_poison_moth_ai};
