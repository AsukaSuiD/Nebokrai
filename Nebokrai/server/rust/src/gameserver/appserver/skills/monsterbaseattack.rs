//! Базовая атака монстра и приручённого питомца (`CMonsterBaseAttack`).
//! Default в выборе и OnChangeSkill берётся из зарегистрированных навыков
//! CMoveShape (GetDefaultAttackSkillID, 0x004CE240), как при Stiffen.
//! Таблица MonsterProperties задаёт взвешенный выбор, но не заменяет реестр
//! владельца: неуспешно загруженный навык не участвует в выборе default.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/monsterbaseattack.cpp`. Модуль навыка хранит выбор цели,
//! стадии атаки, преследование и исходный физический разброс. Встроенный
//! `GetAddElementAtk` возвращает ноль; диапазон monster element здесь не
//! разыгрывается. Общие защита,
//! применение повреждений и точные пакеты принадлежат узкому
//! `monsterattack`; `CGame` оставляет возврат владельца региона и
//! межвладельческие последствия смерти. RAW вариантов `Begin` сохранён для
//! ещё не подключённого координатного входа монстра; координатный player-вход
//! разрешает цель через существующий `CState::GetSufferer` на каждом такте.
//! Назначенный питомцу NPC остаётся допустимым `CMoveShape` на входе команды,
//! но `CMonster::IsAttackAble` отвергает любой тип кроме игрока и монстра;
//! расписание поэтому выполняет обычный `OnLoseTarget` и ставит поиск заново.
//! Конструктор и ветвь `SKILL_MONSTER_BASE_ATTACK` фабрики подтверждают ID
//! `0x2bd`; навык игрока `1` принадлежит другому модулю и не подменяет этот ID.
//! Проверка reuse в расписании делегируется общему exact `CSkill::IsRestored`:
//! его wrapped DWORD deadline намеренно отличается от длительностей стадий.
//! Активный выбор принимает также пять немедленных состояний, четыре Swordship
//! и пять WuXing: они не блокируют весь список навыков монстра. Concrete owner
//! задаёт эффект, а `OnFighting` завершает активный ход даже после `End(0)`:
//! Swordship устанавливает состояние без reuse, WuXing отвергает type `600`
//! без эффекта и reuse. Это завершение не добавляется фоновой очереди.
//! Объектный `CBaseAttack` (`1`) использует тот же monster runtime, но сохраняет
//! свой ID в состоянии и wire. `AI` по VA `0x005B39B0/0x00514820` имеют
//! одинаковую последовательность; различия расчёта не стираются:
//! `CBaseAttack` (`0x005B3600`) берёт `max(max-min,0)`, MonsterBase/Fast
//! (`0x00514460/0x00513490`) прибавляют единицу, LordFast (`0x00530D60`)
//! использует `abs(max-min)+1` с DWORD-переполнением. MonsterBase/Fast перед
//! critical-roll требуют успешный cast в CPlayer, поэтому на монстре этого
//! RNG-вызова нет; BaseAttack/LordFast выполняют его даже при `GetCCH == 0`.
//! `BaseAttack/MonsterBaseAttack::AI` сравнивают задержку с абсолютным
//! wrapping DWORD deadline (`0x005B3B0E/0x0051497E`), а не с elapsed-time.
//! Player-путь `0x2bd` хранит отдельные kernel и reuse в `CPlayerAI`.
//! `CheckCastCondition` (VA `0x00514340`) требует источник и свойства,
//! проверяет reuse с failure 13 и `GS1143`, но не цель/MP/дальность/путь.
//! Первая AI-фаза проверяет дальность беззнаковым сравнением и поворачивает
//! источник; через delay посылается fire с identity и координатами цели,
//! затем один удар. Пустая цель не блокирует анимацию; self/NPC/недопустимая
//! цель не получают урон и не расходуют RNG. Мёртвая цель даёт failure 2
//! и `End(1)`, превышенная дальность — failure 11 и `End(0)`.
//! `End` (VA `0x005b3010`) не меняет движение и не пересчитывает свойства;
//! только успех изнашивает оружие и фиксирует reuse. Формула player-урона
//! общая с MonsterFastAttack, включая личный критический множитель; защита,
//! RP, смерть и сообщения используют существующий владелец применения атаки.
//! Постройки и ворота проходят существующие war/camp-проверки, защиту,
//! изменение HP, death-script и wire стационарного owner-а; NPC не атакуются.
//! У NPC нулевой combat HP: общая IsDied-проверка даёт failure 2 и End(1)
//! до начала анимации, а не пустую атаку по истечении delay.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:111
// RVA: 0x00113B80
// ADDRESS: 00513b80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:127
// RVA: 0x00113C50
// ADDRESS: 00513c50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{
    BASE_ATTACK_SKILL_ID as COMMON_BASE_ATTACK_SKILL_ID,
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::monsterfastattack::{
    MONSTER_FAST_ATTACK_SKILL_ID, SKILL_USAGE_FIRST_TIME, SKILL_USAGE_SECOND_TIME,
    fast_attack_fire_message,
};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attackable_by_monster, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::monsterrangeattack::{
    MONSTER_RANGE_ATTACK_SKILL_ID, MonsterRangeAttackDispatch,
    prepare_owned_monster_range_cast,
};
use super::chuckstone::CHUCK_STONE_SKILL_ID;
use super::archery::{
    ARCHERY_SKILL_ID, MonsterBaseProjectileKind, execute_owned_monster_base_projectile,
};
use super::basemagic::BASE_MAGIC_SKILL_ID as BASE_MAGIC_PROJECTILE_SKILL_ID;
use super::bossbluefury::{BOSS_BLUE_FURY_SKILL_ID, execute_owned_boss_blue_fury};
use super::bossbluequake::{BOSS_BLUE_QUAKE_SKILL_ID, execute_owned_boss_blue_quake};
use super::bossfiendsummon::BOSS_FIEND_SUMMON_SKILL_ID;
use super::bossfiendpenetrate::{
    BOSS_FIEND_PENETRATE_SKILL_ID, execute_owned_boss_fiend_penetrate,
};
use super::corpsecandleblasting::{
    CORPSE_CANDLE_BLASTING_SKILL_ID, execute_owned_corpse_candle_blasting,
};
use super::corpseptomaine::{CORPSE_PTOMAINE_SKILL_ID, execute_owned_corpse_ptomaine};
use super::energybolt::{ENERGY_BOLT_SKILL_ID, execute_owned_energy_bolt};
use super::fury::{FURY_SKILL_ID, execute_owned_fury};
use super::immediatestate::{execute_monster_immediate_state, is_immediate_state_skill};
use super::swordship::{execute_monster_auto_start_swordship, is_swordship_skill};
use super::wuxing::is_wuxing_skill;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillTermination};
use super::littlestar::{LITTLE_STAR_SKILL_ID, execute_owned_little_star};
use super::lordfastattack::LORD_FAST_ATTACK_SKILL_ID;
use super::lordwiderangingattack::{
    LORD_WIDERANGING_ATTACK_SKILL_ID, prepare_owned_lord_wideranging_attack,
};
use super::machinerystomp::{
    MACHINERY_STOMP_SKILL_ID, WideArcAttackDispatch, prepare_owned_machinery_stomp,
};
use super::monsterprojectile::{MonsterProjectileDispatch, prepare_owned_monster_projectile};
use super::monsterthorn::{MONSTER_THORN_SKILL_ID, execute_owned_monster_thorn};
use super::knockoutruntime::{KNOCK_OUT_SKILL_ID, execute_owned_monster_knock_out};
use super::promotion::{PROMOTION_SKILL_ID, execute_owned_monster_promotion};
use super::skeletonarchery::SKELETON_ARCHERY_SKILL_ID;
use super::snakebolt::{SNAKE_BOLT_SKILL_ID, execute_owned_snake_bolt};
use super::snowstorm::{SNOW_STORM_SKILL_ID, execute_owned_monster_snow_storm};
use super::spiderpoison::{SPIDER_POISON_SKILL_ID, execute_owned_spider_poison};
use super::spidermist::{SPIDER_MIST_SKILL_ID, execute_owned_spider_mist};
use super::spiderweb::{SPIDER_WEB_SKILL_ID, execute_owned_spider_web};
use super::sporeblasting::{SPORE_BLASTING_SKILL_ID, execute_owned_spore_blasting};
use super::spriteburn::{SPRITE_BURN_SKILL_ID, execute_owned_sprite_burn};
use super::summoncorpsecandle::SUMMON_CORPSE_CANDLE_SKILL_ID;
use super::summoncreatureskill::execute_owned_summon_creature;
use super::summonskeleton::SUMMON_SKELETON_SKILL_ID;
use super::summonspore::SUMMON_SPORE_SKILL_ID;
use super::yunshenglightning::{YUNSHENG_LIGHTNING_SKILL_ID, execute_owned_yunsheng_lightning};
use super::yakshaslash::{YAKSHA_SLASH_SKILL_ID, execute_owned_monster_yaksha_slash};
use super::zombieclaw::{ZOMBIE_CLAW_SKILL_ID, execute_owned_zombie_claw};
use crate::gameserver::appserver::ai::aifactory::MonsterAiKind;
use crate::gameserver::appserver::ai::archer::select_archer_enemy;
use crate::gameserver::appserver::ai::bossblue::{
    choose_boss_blue_attack_skill, select_boss_blue_enemy,
};
use crate::gameserver::appserver::ai::bossfiend::{
    choose_boss_fiend_attack_skill, select_boss_fiend_enemy,
};
use crate::gameserver::appserver::ai::bossidle::queue_boss_idle;
use crate::gameserver::appserver::ai::cityguardwithsword::{
    CitySwordTraceOutcome, lose_guard_sword_target, release_guard_sword_target,
    select_city_guard_enemy, trace_city_sword_target,
};
use crate::gameserver::appserver::ai::cityguardwithbow::stationary_bow_target_ready;
use crate::gameserver::appserver::ai::fixedpositionarcher::select_fixed_archer_enemy;
use crate::gameserver::appserver::ai::fixedpositionarcher::{
    inherits_fixed_archer_change_skill, queue_fixed_archer_skill_delay,
    queue_stationary_guard_idle,
};
use crate::gameserver::appserver::ai::gladiator::select_gladiator_enemy;
use crate::gameserver::appserver::ai::godsbattlemonster::select_gods_battle_enemy;
use crate::gameserver::appserver::ai::godsbattleguardwithsword::select_gods_battle_guard_enemy;
use crate::gameserver::appserver::ai::guardwithbow::select_guard_with_bow_target;
use crate::gameserver::appserver::ai::guardcountry::select_country_guard_target;
use crate::gameserver::appserver::ai::jiumai::{
    assign_jiumai_target, ensure_jiumai_twin, maintain_jiumai_twin, select_jiumai_enemy,
};
use crate::gameserver::appserver::ai::lord::{select_lord_attack_skill, select_lord_enemy};
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, has_owned_search_enemy,
    hibernates_without_nearby_players,
    queue_monster_idle, schedule_attack_interval, select_attack_skill,
};
use crate::gameserver::appserver::ai::baseai::one_step_move_delay_ms;
use crate::gameserver::appserver::ai::puninesscreature::search_puniness_enemy;
use crate::gameserver::appserver::ai::pet::{
    PetMasterRef, lose_pet_target_and_search, pet_master_ref, queue_pet_idle,
};
use crate::gameserver::appserver::ai::nationgladiator::select_nation_gladiator_enemy;
use crate::gameserver::appserver::ai::nationcouguardwithsword::select_nation_country_guard_enemy;
use crate::gameserver::appserver::ai::smartgladiator::select_smart_gladiator_enemy;
use crate::gameserver::appserver::ai::stupidarcher::search_stupid_archer_enemy;
use crate::gameserver::appserver::ai::warattackmonster::select_country_war_enemy;
use crate::gameserver::appserver::ai::vilcouguardwithsword::select_village_country_guard_enemy;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::state::{resolve_coordinate_sufferer, resolve_identity_sufferer};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::{MonsterProperties, MonsterSkill};

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
pub(crate) const MONSTER_BASE_ATTACK_SKILL_ID: u32 = 0x2bd;

pub(crate) const fn is_player_monster_base_attack(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: MONSTER_BASE_ATTACK_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE | 500 | 1100 | 1200, .. } })
}

fn player_base_attack_outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn end_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime, success: bool,
) {
    if success {
        game.damage_player_weapon(player_id, runtime);
    }
    if success {
        ai.mark_skill_used(MONSTER_BASE_ATTACK_SKILL_ID, runtime.now_milliseconds());
    }
}

pub(crate) fn finish_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime, success: bool,
) -> bool {
    let Some(dispatch) = ai.player_skill_execution(MONSTER_BASE_ATTACK_SKILL_ID).map(|kernel| kernel.dispatch()) else { return false };
    end_player_monster_base_attack(game, player_id, ai, runtime, success);
    ai.finish_player_skill(dispatch, if success { SkillTermination::Completed } else { SkillTermination::Cancelled })
}

pub(crate) fn execute_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    use super::lordfastattack::{calculate_attack, master_info, send_start};
    let rejected = || player_base_attack_outcome(QueuedSkillExecutionState::Rejected);
    if !is_player_monster_base_attack(dispatch) { return rejected(); }
    let Some((region_id, level, source)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.learned_skill_level(MONSTER_BASE_ATTACK_SKILL_ID), player.shape_view()?))
    }) else { return rejected() };
    let Some(properties) = game.skill_base_properties(MONSTER_BASE_ATTACK_SKILL_ID, level) else {
        end_player_monster_base_attack(game, player_id, ai, runtime, false);
        return rejected();
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(super::basemagic::SKILL_USAGE_CAN_BE_BREAKED);
    if ai.player_skill_execution(MONSTER_BASE_ATTACK_SKILL_ID).is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(MONSTER_BASE_ATTACK_SKILL_ID), reuse, now) {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS1143");
            end_player_monster_base_attack(game, player_id, ai, runtime, false);
            return rejected();
        }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, now));
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(MONSTER_BASE_ATTACK_SKILL_ID));
        }
        return player_base_attack_outcome(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_execution(MONSTER_BASE_ATTACK_SKILL_ID).is_none_or(|kernel| kernel.dispatch() != dispatch) {
        return rejected();
    }
    let requested = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => resolve_identity_sufferer(game, region_id, target),
        PlayerSkillDispatch::Point { x, y, .. } => resolve_coordinate_sufferer(game, region_id, x, y),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    };
    let target = requested.and_then(|identity| game.base_magic_target_view(region_id, identity).map(|view| (identity, view)));
    if target.is_some_and(|(identity, _)| game.base_magic_target_dead(region_id, identity)) {
        game.send_self_state_skill_failure(0x000b_fe01, player_id, 2);
        end_player_monster_base_attack(game, player_id, ai, runtime, true);
        return player_base_attack_outcome(QueuedSkillExecutionState::Completed);
    }
    let (fallback_x, fallback_y) = match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => (x, y),
        _ => (0, 0),
    };
    let (target_x, target_y) = target.map_or((fallback_x, fallback_y), |(_, view)| (view.tile_x, view.tile_y));
    if ai.player_skill_execution(MONSTER_BASE_ATTACK_SKILL_ID).is_some_and(|kernel| kernel.stage() == SkillStage::Begin) {
        let distance = target.map_or_else(
            || super::baseattack::real_distance(source.tile_x, source.tile_y, target_x, target_y),
            |(_, view)| source.real_distance(Some(view)),
        );
        if maximum_distance != 0 && maximum_distance < distance as u32 {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0b);
            end_player_monster_base_attack(game, player_id, ai, runtime, false);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, target_x, target_y));
        }
        send_start(game, player_id, MONSTER_BASE_ATTACK_SKILL_ID, level);
        if let Some(kernel) = ai.player_skill_execution_mut(MONSTER_BASE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = ai.player_skill_execution(MONSTER_BASE_ATTACK_SKILL_ID).map(|kernel| kernel.started_at_ms()).expect("базовая атака хранит начало");
    if !skill_is_restored(started, delay, runtime.now_milliseconds()) {
        return player_base_attack_outcome(QueuedSkillExecutionState::Pending);
    }
    let mut fire = CMessage::new(0x000b_fe01);
    fire.add_byte(2);
    fire.add_long(MONSTER_BASE_ATTACK_SKILL_ID as i32);
    fire.add_short(level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target.map_or(0, |(identity, _)| identity.object_type));
    fire.add_long(target.map_or(0, |(identity, _)| identity.id));
    fire.add_long(target_x);
    fire.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &fire);
    if let Some(kernel) = ai.player_skill_execution_mut(MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    if let Some((identity, _)) = target
        && !(identity.object_type == PLAYER_TYPE && identity.id == player_id)
        && let Some(master) = game.find_player(player_id).map(master_info)
        && (if matches!(identity.object_type, 1100 | 1200) {
            game.stationary_build_attackable_by_player(player_id, region_id, identity)
        } else {
            game.owned_player_skill_target_attackable(master, identity, region_id)
        })
        && let Some((master, attack)) = calculate_attack(game, player_id, MONSTER_BASE_ATTACK_SKILL_ID, level, hit_modifier)
    {
        match identity.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, identity.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, identity.id, region_id, attack, runtime),
            1100 | 1200 => game.apply_owned_skill_attack_to_stationary_build(player_id, region_id, identity, attack, runtime),
            _ => {}
        }
    }
    if let Some(kernel) = ai.player_skill_execution_mut(MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    end_player_monster_base_attack(game, player_id, ai, runtime, true);
    player_base_attack_outcome(QueuedSkillExecutionState::Completed)
}

const BASE_ATTACK_SKILL_ID: u16 = 1;
const BASE_ARCHERY_SKILL_ID: u16 = 2;
const BASE_MAGIC_SKILL_ID: u16 = 3;

fn is_owned_monster_attack_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        COMMON_BASE_ATTACK_SKILL_ID
            | ARCHERY_SKILL_ID
            | BASE_MAGIC_PROJECTILE_SKILL_ID
            | MONSTER_BASE_ATTACK_SKILL_ID
            | MONSTER_FAST_ATTACK_SKILL_ID
            | LORD_FAST_ATTACK_SKILL_ID
            | MONSTER_RANGE_ATTACK_SKILL_ID
            | MONSTER_THORN_SKILL_ID
            | SKELETON_ARCHERY_SKILL_ID
            | CHUCK_STONE_SKILL_ID
            | YUNSHENG_LIGHTNING_SKILL_ID
            | CORPSE_PTOMAINE_SKILL_ID
            | CORPSE_CANDLE_BLASTING_SKILL_ID
            | SPORE_BLASTING_SKILL_ID
            | ENERGY_BOLT_SKILL_ID
            | ZOMBIE_CLAW_SKILL_ID
            | FURY_SKILL_ID
            | LITTLE_STAR_SKILL_ID
            | SNAKE_BOLT_SKILL_ID
            | SPIDER_POISON_SKILL_ID
            | SPIDER_MIST_SKILL_ID
            | SPIDER_WEB_SKILL_ID
            | SPRITE_BURN_SKILL_ID
            | MACHINERY_STOMP_SKILL_ID
            | LORD_WIDERANGING_ATTACK_SKILL_ID
            | BOSS_BLUE_FURY_SKILL_ID
            | BOSS_BLUE_QUAKE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
            | BOSS_FIEND_PENETRATE_SKILL_ID
            | SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | PROMOTION_SKILL_ID
            | KNOCK_OUT_SKILL_ID
            | YAKSHA_SLASH_SKILL_ID
            | SNOW_STORM_SKILL_ID
    ) || is_immediate_state_skill(skill_id) || is_swordship_skill(skill_id)
}

/// Rust-владелец выбирает навык только когда любой явно установленный результат
/// броска уже имеет реального владельца исполнения. Иначе весь ход остаётся
/// внешней виртуальной ветви, чтобы она не получила второй вызов исходного
/// генератора случайных чисел после частичной диспетчеризации. Сумма `odds`
/// намеренно не обязана покрывать `0..9999`: подтверждённый
/// `SelectAttackSkill` возвращает `GetDefaultAttackSkillID` для оставшегося
/// диапазона. Исключённые записи `2` синего босса и `1/2` демона-босса и
/// владыки разрешены: их `odds` участвуют в накоплении, но сами ID специальные
/// селекторы не возвращают до запасной ветви.
fn owns_complete_skill_selection(skills: &[MonsterSkill], ai_type: u32) -> bool {
    !skills.is_empty()
        && skills.iter().all(|skill| {
            is_owned_monster_attack_skill(u32::from(skill.id))
                || (ai_type == 21 && skill.id == BASE_ARCHERY_SKILL_ID)
                || (ai_type == 23
                    && matches!(
                        skill.id,
                        BASE_ATTACK_SKILL_ID | BASE_ARCHERY_SKILL_ID
                    ))
                || (ai_type == 19
                    && matches!(
                        skill.id,
                        BASE_ATTACK_SKILL_ID | BASE_ARCHERY_SKILL_ID
                    ))
        })
}

fn installed_monster_skill(skills: &[MonsterSkill], skill_id: u16) -> Option<MonsterSkill> {
    skills
        .iter()
        .copied()
        .filter(|skill| skill.id == skill_id)
        .max_by_key(|skill| skill.level)
}

/// Точная встречная ветвь `CPet::OnStayingSchedule` и
/// `CPet::OnAttackingSchedule`. `GetAI` цели возвращает AI-owner, чьи поля
/// target type/id сравниваются с самим питомцем; только совпавший AI получает
/// виртуальный `OnLoseTarget`. Derived-переходы питомца и мечевого охранника
/// сохраняются, но внешний `SearchEnemy` текущего питомца сюда не переносится.
fn release_reciprocal_monster_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    target_id: i32,
    pet_identity: ShapeIdentity,
    runtime: &mut Runtime,
) {
    let Some((reciprocal, tamed, pet_action, stop_frame, ai_type)) = region
        .find_monster_by_id(target_id)
        .and_then(|target| {
            let property = game
                .find_monster_property_by_origin_name(target.base_property_key()?)?;
            Some((
                target.ai_target() == Some(pet_identity),
                target.is_tamed(),
                target.pet_action(),
                target.stop_frame(property),
                property.ai,
            ))
        })
    else {
        return;
    };
    if !reciprocal {
        return;
    }
    if tamed {
        if let Some(target) = region.find_monster_by_id_mut(target_id) {
            target.clear_ai_target();
        }
        if pet_action == 0 {
            let _ = queue_pet_idle(region, target_id, stop_frame, runtime);
        }
    } else if matches!(ai_type, 10 | 12 | 16) {
        release_guard_sword_target(game, region, target_id, runtime);
    } else if let Some(target) = region.find_monster_by_id_mut(target_id) {
        target.clear_ai_target();
    }
}

/// `CPet::GetPetMaster` сначала разрешает игрока глобальной таблицей, а для
/// остальных типов — ровно зарегистрированный `CMoveShape` текущего региона.
/// Боевой schedule использует найденную форму только как центр ограничения
/// преследования; отсутствие master-а оставляет прежний центр на питомце.
fn pet_combat_master_anchor(
    game: &CGame,
    region: &CServerRegion,
    master: MasterInfo,
) -> Option<(i32, i32)> {
    let master = match pet_master_ref(master)? {
        PetMasterRef::Player(player_id) => game.find_player(player_id)?.shape_view()?,
        PetMasterRef::Region(identity) => game.find_shape_in_region(region.id, identity)?,
    };
    Some((master.tile_x, master.tile_y))
}

fn select_and_store_monster_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    monster_health: u32,
    runtime: &mut Runtime,
) -> Option<u16> {
    let default_skill_id = region.find_monster_by_id(monster_id)?
        .move_shape().default_attack_skill_id() as u16;
    let roll = game.skill_random_below(10_000);
    let selected = if property.ai == 21 {
        choose_boss_blue_attack_skill(
            region,
            monster_id,
            property,
            monster_health,
            roll,
            default_skill_id,
        )
    } else if property.ai == 23 {
        choose_boss_fiend_attack_skill(
            game,
            region,
            monster_id,
            property,
            monster_health,
            roll,
            runtime,
        )
    } else if property.ai == 19 {
        Some(select_lord_attack_skill(
            monster_health,
            property.maximum_hp,
            &property.skills,
            roll,
            default_skill_id,
        ))
    } else {
        Some(select_attack_skill(
            &property.skills,
            roll,
            default_skill_id,
        ))
    }
    .unwrap_or(default_skill_id);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .set_current_skill_id(Some(u32::from(selected)));
    }
    Some(selected)
}

/// Выполняет точный `CMonsterAI::OnChangeSkill` отдельным FIFO-тактом. После
/// единственного weighted RNG выбранный concrete skill проверяется через
/// `CSkill::IsRestored`; отсутствующий или ещё не восстановленный навык общего
/// monster AI заменяется `GetDefaultAttackSkillID`. AI5 и наследующий его
/// AI103 сохраняют существующий навык на cooldown и ставят полный restore
/// delay в хвост FIFO. Boss-specific пороги остаются в своих selector-owner-ах.
pub(crate) fn change_owned_monster_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some((property, monster_health)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.hit_points(),
            ))
        })
    else {
        return false;
    };
    if !owns_complete_skill_selection(&property.skills, property.ai) {
        return false;
    }
    let selected = select_and_store_monster_attack_skill(
        game,
        region,
        monster_id,
        &property,
        monster_health,
        runtime,
    );
    let Some(selected_skill_id) = selected else {
        return false;
    };
    if inherits_fixed_archer_change_skill(property.ai) {
        if queue_fixed_archer_skill_delay(
            game,
            region,
            monster_id,
            &property,
            selected_skill_id,
            runtime,
        )
        {
            return true;
        }
    } else if installed_monster_skill(&property.skills, selected_skill_id)
        .and_then(|skill| {
            let properties = game.skill_base_properties(
                u32::from(selected_skill_id),
                i32::from(skill.level),
            )?;
            let last_used_ms = region
                .find_monster_by_id(monster_id)?
                .skill_last_used_ms(u32::from(selected_skill_id));
            Some(skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                runtime.now_milliseconds(),
            ))
        })
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let default_skill_id = monster.move_shape().default_attack_skill_id();
        monster
            .move_shape_mut()
            .set_current_skill_id(Some(default_skill_id));
    }
    true
}

/// Выполняет только подтверждённый `OnSearchEnemy` обычного агрессивного
/// монстра, умного и пассивного гладиаторов, слабого существа, двух лучников,
/// военного монстра, участника битвы богов, городского охранника, владыки,
/// близнецов JiuMai и двух боссов. Фабричный fallback `CMonsterAI` выполняет
/// унаследованный пустой `OnSearchEnemy`, но всё равно завершает FIFO-событие.
/// Предшествующее событие уже обработано владельцем FIFO, поэтому здесь не
/// начинается атака в том же такте.
pub(crate) fn search_owned_monster_enemy<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    if region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
        })
        .is_some_and(|property| property.ai == 7)
    {
        return search_puniness_enemy(game, region, monster_id);
    }
    if region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
        })
        .is_some_and(|property| property.ai == 1)
    {
        let Some((property, owner)) = region
            .find_monster_by_id(monster_id)
            .and_then(|monster| {
                let property = game
                    .find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone();
                Some((property.clone(), monster.shape_view(&property)?))
            })
        else {
            return false;
        };
        let Some(mut state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(CMonster::take_passive_gladiator_ai)
        else {
            return false;
        };
        let selected = state.select_target(owner, property.chase_range as i32, |player_id| {
            game.find_player(player_id).and_then(|player| {
                (player.server_region_id() == Some(region.id) && !player.is_dead())
                    .then(|| player.shape_view())
                    .flatten()
            })
        });
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.restore_passive_gladiator_ai(state);
            if let Some(selected) = selected {
                monster.set_ai_target(selected);
            }
        }
        return true;
    }
    let Some((property, owner, area_index, skill_id, skill_level, speed, master)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let owner = monster.shape_view(&property)?;
            let area_index = monster.move_shape().shape().area_index()?;
            let skill_id = monster.move_shape().current_skill_id()?;
            let skill = installed_monster_skill(&property.skills, skill_id as u16)?;
            Some((
                property,
                owner,
                area_index,
                skill_id,
                skill.level,
                monster.move_shape().shape().get_speed(),
                monster.master_info(),
            ))
        })
    else {
        return false;
    };
    if property.ai == 20 {
        if let Some(selected) = select_jiumai_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ) {
            let _ = assign_jiumai_target(region, monster_id, selected);
        }
        return true;
    }
    if property.ai == 2 {
        let selection = select_smart_gladiator_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        );
        if let Some(selected) = selection.vulnerable_target() {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.set_ai_target(selected);
            }
        } else if let Some(destination) = selection.retreat_step(owner)
            && let Some(state) = region
                .find_monster_by_id_mut(monster_id)
                .and_then(CMonster::smart_gladiator_ai_mut)
        {
            state.queue_step(destination);
        }
        return true;
    }
    let selected = match property.ai {
        0 | 3 => select_gladiator_enemy(
            game,
            region,
            owner,
            area_index,
            &property,
            false,
            master,
        ),
        4 => select_archer_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        5 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_fixed_archer_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
        }
        6 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            let _ = search_stupid_archer_enemy(
                game,
                region,
                monster_id,
                owner,
                area_index,
                &property,
                minimum_skill_distance,
                speed,
                runtime,
            );
            return true;
        }
        8 | 9 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_guard_with_bow_target(
                game,
                region,
                monster_id,
                &property,
                minimum_skill_distance,
            )
        }
        10 | 11 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_city_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
            .map(|selected| selected.identity)
        }
        17 | 100 | 101 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_country_guard_target(
                game,
                region,
                monster_id,
                &property,
                minimum_skill_distance,
            )
        }
        14 | 15 => select_country_war_enemy(
            game,
            region,
            owner,
            area_index,
            property.ai,
            property.guard_range as i32,
        ),
        12 | 13 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_village_country_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
            .map(|selected| selected.identity)
        }
        16 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_nation_country_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
                property.race,
            )
            .map(|selected| selected.identity)
        }
        18 => select_nation_gladiator_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
            property.race,
        ),
        103 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_gods_battle_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
                property.race,
            )
        }
        104 => select_gods_battle_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
            property.race,
        ),
        19 => select_lord_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        21 => select_boss_blue_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        23 => {
            let minimum_skill_distance = game
                .skill_base_properties(skill_id, i32::from(skill_level))
                .map_or(0, |properties| properties.query_property(5_004) as i32);
            select_boss_fiend_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
        }
        _ if MonsterAiKind::is_generic_ai_type(property.ai) => None,
        _ => return false,
    };
    if let Some(selected) = selected
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        monster.set_ai_target(selected);
    }
    true
}

pub(crate) fn execute_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
    range_dispatch: &mut Option<MonsterRangeAttackDispatch>,
    wide_arc_dispatch: &mut Option<WideArcAttackDispatch>,
    projectile_dispatch: &mut Option<MonsterProjectileDispatch>,
    snow_storm_entry: &mut Option<i32>,
) -> bool {
    let Some((
        property,
        monster_shape,
        monster_view,
        monster_health,
        target,
        cast,
        tamed,
        attacker_master,
        pet_attack_properties,
        stop_frame,
        area_index,
        pet_action,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let monster_view = monster.shape_view(&property)?;
        let pet_attack_properties = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property));
        let stop_frame = monster.stop_frame(&property);
        Some((
            property,
            monster.move_shape().shape().clone(),
            monster_view,
            monster.hit_points(),
            monster.ai_target(),
            monster.base_attack_cast(),
            monster.is_tamed(),
            monster.master_info(),
            pet_attack_properties,
            stop_frame,
            monster.move_shape().shape().area_index(),
            monster.pet_action(),
        ))
    })
    else {
        return false;
    };
    if !tamed && property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
        // Повозкой управляет отдельный производный ИИ; общий поиск цели и
        // расписание атаки обычного монстра для неё не выполняются.
        return false;
    }
    if CMoveShape::is_died(monster_health) {
        return false;
    }
    if property.ai == 20
        && !ensure_jiumai_twin(game, region, monster_id, &property)
    {
        return false;
    }
    if property.ai == 20
        && !maintain_jiumai_twin(game, region, monster_id, &property, runtime)
    {
        return false;
    }
    if !owns_complete_skill_selection(&property.skills, property.ai) {
        return false;
    }
    if matches!(property.ai, 10 | 12 | 16) {
        let left_chase_range = region
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| {
                let state = monster.guard_station_ai_mut()?;
                state.record_station(monster_view);
                Some(
                    target.is_some()
                        && state.left_chase_range(monster_view, property.chase_range as i32),
                )
            })
            .unwrap_or(false);
        if left_chase_range {
            lose_guard_sword_target(game, region, monster_id, runtime);
            return true;
        }
    }
    if target.is_none()
        && cast.is_none()
        && !tamed
        && hibernates_without_nearby_players(
            property.ai,
            region
                .find_monster_by_id(monster_id)
                .and_then(CMonster::smart_gladiator_ai)
                .is_some_and(|state| !state.has_queued_steps()),
        )
        && let Some(area_index) = area_index
        && region.player_ids_around_area(area_index).is_empty()
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.hibernate_ai(runtime.now_milliseconds());
            return true;
        }
    }
    if target.is_none() && cast.is_none() && !tamed && property.ai == 2 {
        let destination = region
            .find_monster_by_id_mut(monster_id)
            .and_then(CMonster::smart_gladiator_ai_mut)
            .and_then(|state| state.take_step());
        if let Some(destination) = destination {
            let moved = game.move_owned_monster_step(
                region,
                monster_id,
                destination.x,
                destination.y,
                CMonster::figure(&property),
            );
            if moved {
                let direction = get_line_direction(
                    monster_view.tile_x,
                    monster_view.tile_y,
                    destination.x,
                    destination.y,
                );
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    monster.begin_active_ai_move(
                        one_step_move_delay_ms(
                            direction,
                            monster_shape.get_speed(),
                            stop_frame,
                        ),
                        runtime.now_milliseconds(),
                    );
                }
            }
            return true;
        }
    }
    if target.is_none()
        && cast.is_none()
        && !tamed
        && matches!(property.ai, 5 | 8 | 11 | 13 | 17 | 100 | 101 | 103)
    {
        return queue_stationary_guard_idle(
            region,
            monster_id,
            stop_frame,
            runtime,
        );
    }
    if target.is_none()
        && cast.is_none()
        && !tamed
        && (matches!(property.ai, 0 | 1 | 2 | 3 | 4 | 6 | 9 | 10 | 12 | 14 | 15 | 16 | 18 | 19 | 20 | 104)
            || MonsterAiKind::is_generic_ai_type(property.ai))
    {
        return queue_monster_idle(game, region, monster_id, &property, runtime);
    }
    if target.is_none() && cast.is_none() && tamed {
        lose_pet_target_and_search(
            region,
            monster_id,
            stop_frame,
            runtime,
        );
        return true;
    }
    let selected_skill_id = if let Some(cast) = cast {
        cast.dispatch().skill_id as u16
    } else if let Some(skill_id) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.move_shape().current_skill_id())
    {
        skill_id as u16
    } else {
        let Some(selected) = select_and_store_monster_attack_skill(
            game,
            region,
            monster_id,
            &property,
            monster_health,
            runtime,
        ) else {
            return true;
        };
        selected
    };
    let Some(skill) = installed_monster_skill(&property.skills, selected_skill_id) else {
        return false;
    };
    let skill_id = u32::from(skill.id);
    if !is_owned_monster_attack_skill(skill_id) {
        return false;
    }
    if target.is_none()
        && cast.is_none()
        && !tamed
        && matches!(property.ai, 21 | 23)
        && queue_boss_idle(game, region, monster_id, &property, runtime)
    {
        return true;
    }
    let fast_attack = matches!(skill_id, MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID);
    let target = cast.map(|cast| cast.dispatch().target).or(target);
    let Some(target) = target else {
        return false;
    };
    if cast.is_none() {
        let Some(schedule_target) =
            resolve_owned_monster_attack_target(game, region, target)
        else {
            if tamed {
                lose_pet_target_and_search(
                    region,
                    monster_id,
                    stop_frame,
                    runtime,
                );
            } else if matches!(property.ai, 10 | 12 | 16) {
                lose_guard_sword_target(game, region, monster_id, runtime);
            } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                if has_owned_search_enemy(property.ai, tamed) {
                    monster.lose_ai_target_and_search(runtime.now_milliseconds());
                } else {
                    monster.clear_ai_target();
                }
            }
            return true;
        };
        let attackable = owned_monster_attackable(
            game,
            region.id,
            &property,
            tamed,
            attacker_master,
            target,
            &schedule_target,
        );
        if schedule_target.dead
            || (property.ai != 13
                && (schedule_target.god || schedule_target.city_dead || !attackable))
        {
            if !attackable && tamed {
                let pet_identity = ShapeIdentity {
                    object_type: MONSTER_TYPE,
                    id: monster_id,
                    ex_id: CGuid::GUID_INVALID,
                };
                if target.object_type == PLAYER_TYPE {
                    game.release_reciprocal_player_target(target.id, pet_identity, runtime);
                } else if target.object_type == MONSTER_TYPE {
                    release_reciprocal_monster_target(
                        game,
                        region,
                        target.id,
                        pet_identity,
                        runtime,
                    );
                }
            }
            if tamed {
                lose_pet_target_and_search(
                    region,
                    monster_id,
                    stop_frame,
                    runtime,
                );
            } else if matches!(property.ai, 10 | 12 | 16) {
                lose_guard_sword_target(game, region, monster_id, runtime);
            } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                if has_owned_search_enemy(property.ai, tamed) {
                    monster.lose_ai_target_and_search(runtime.now_milliseconds());
                } else {
                    monster.clear_ai_target();
                }
            }
            return true;
        }
    }
    let Some(skill_properties) = game
        .skill_base_properties(skill_id, i32::from(skill.level))
        .cloned()
    else {
        return false;
    };
    let now_ms = runtime.now_milliseconds();
    if is_immediate_state_skill(skill_id) || is_swordship_skill(skill_id) {
        let attack_interval = schedule_attack_interval(
            property.ai,
            pet_attack_properties.map_or(property.attack_speed, |pet| pet.attack_interval),
        );
        if attack_interval.is_some_and(|interval| {
            region
                .find_monster_by_id_mut(monster_id)
                .is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))
        }) {
            return true;
        }
        let reuse_delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let last_used_ms = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.skill_last_used_ms(skill_id))
            .unwrap_or_default();
        if !skill_is_restored(last_used_ms, reuse_delay_ms, now_ms) {
            return true;
        }
        let executed = if is_swordship_skill(skill_id) {
            execute_monster_auto_start_swordship(
                game, region, monster_id, skill_id, i32::from(skill.level),
            )
        } else if is_wuxing_skill(skill_id) {
            // Exact player-only отказ завершает навык, но не создаёт состояние.
            true
        } else {
            execute_monster_immediate_state(
                game, region, monster_id, skill_id, i32::from(skill.level), now_ms,
            )
        };
        if executed {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.finish_active_immediate_skill(now_ms);
            }
        }
        return executed;
    }
    if skill_id == FURY_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_fury(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
        );
    }
    if skill_id == PROMOTION_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_promotion(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
        );
    }
    if skill_id == KNOCK_OUT_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_knock_out(game, region, monster_id, target, skill.level, &skill_properties, &property, now_ms, runtime, deaths);
    }
    if skill_id == YAKSHA_SLASH_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_yaksha_slash(game, region, monster_id, target, skill.level, &skill_properties, &property, now_ms, runtime, deaths);
    }
    if skill_id == SNOW_STORM_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_snow_storm(game, region, monster_id, target, skill.level, &skill_properties, &property, now_ms, runtime, snow_storm_entry);
    }
    if matches!(skill_id, ARCHERY_SKILL_ID | BASE_MAGIC_PROJECTILE_SKILL_ID) {
        return execute_owned_monster_base_projectile(
            game,
            region,
            monster_id,
            target,
            skill.level,
            if skill_id == ARCHERY_SKILL_ID {
                MonsterBaseProjectileKind::Archery
            } else {
                MonsterBaseProjectileKind::Magic
            },
            runtime,
        );
    }
    if matches!(skill_id, SKELETON_ARCHERY_SKILL_ID | CHUCK_STONE_SKILL_ID) {
        let skill_properties = skill_properties.clone();
        return prepare_owned_monster_projectile(
            game,
            region,
            monster_id,
            target,
            skill_id,
            skill.level,
            &skill_properties,
            now_ms,
            projectile_dispatch,
        );
    }
    if skill_id == YUNSHENG_LIGHTNING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_yunsheng_lightning(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == CORPSE_PTOMAINE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_corpse_ptomaine(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == CORPSE_CANDLE_BLASTING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_corpse_candle_blasting(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == SPORE_BLASTING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spore_blasting(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == ENERGY_BOLT_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_energy_bolt(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == ZOMBIE_CLAW_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_zombie_claw(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == LITTLE_STAR_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_little_star(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == SNAKE_BOLT_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_snake_bolt(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == SPRITE_BURN_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_sprite_burn(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == MACHINERY_STOMP_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return prepare_owned_machinery_stomp(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            wide_arc_dispatch,
        );
    }
    if skill_id == LORD_WIDERANGING_ATTACK_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return prepare_owned_lord_wideranging_attack(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            wide_arc_dispatch,
        );
    }
    if skill_id == BOSS_BLUE_FURY_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_blue_fury(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
        );
    }
    if skill_id == BOSS_BLUE_QUAKE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_blue_quake(
            game, region, monster_id, target, skill.level, &skill_properties, now_ms, runtime, deaths,
        );
    }
    if skill_id == BOSS_FIEND_PENETRATE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_fiend_penetrate(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == MONSTER_THORN_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_thorn(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == SPIDER_POISON_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spider_poison(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
            deaths,
        );
    }
    if skill_id == SPIDER_MIST_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spider_mist(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == SPIDER_WEB_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spider_web(
            game,
            region,
            monster_id,
            target,
            skill.level,
            &skill_properties,
            now_ms,
        );
    }
    if matches!(
        skill_id,
        SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
    ) {
        let skill_properties = skill_properties.clone();
        return execute_owned_summon_creature(
            game,
            region,
            monster_id,
            target,
            skill_id,
            skill.level,
            &skill_properties,
            now_ms,
        );
    }
    if skill_id == MONSTER_RANGE_ATTACK_SKILL_ID && cast.is_some() {
        let skill_properties = skill_properties.clone();
        return prepare_owned_monster_range_cast(
            game,
            region,
            monster_id,
            &skill_properties,
            now_ms,
            range_dispatch,
        );
    }
    let delay_ms = skill_properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = skill_properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let hit_modifier = skill_properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let target_snapshot = match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).and_then(|player| {
            let target_view = player.shape_view()?;
            (player.server_region_id() == Some(region.id)).then(|| {
                (
                    player.shape().clone(),
                    target_view,
                    player.health(),
                    player.mana(),
                    player.war_soul_mana(game.goods_factory()),
                    Some(player.combat_properties()),
                    None,
                    player.is_dead(),
                    player.is_god_mode(),
                    player.city_war_died_state(),
                    None,
                    None,
                    false,
                    false,
                )
            })
        }),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .and_then(|target_monster| {
                let target_property = game.find_monster_property_by_origin_name(
                    target_monster.base_property_key()?,
                )?;
                let carriage = target_monster.is_carriage(target_property);
                let target_master = target_monster.master_info();
                let target_view = target_monster.shape_view(target_property)?;
                let target_attackable = if carriage {
                    game.carriage_attackable_by_monster(
                        &property,
                        tamed,
                        attacker_master,
                        target_master,
                        region.id,
                    )
                } else {
                    monster_attackable_by_monster(
                        game,
                        &property,
                        tamed,
                        attacker_master,
                        target_property,
                        target_monster.is_tamed(),
                        target_master,
                        region.id,
                    )
                };
                target_attackable.then(|| {
                    (
                        target_monster.move_shape().shape().clone(),
                        target_view,
                        target_monster.hit_points(),
                        0,
                        None,
                        None,
                        Some(target_monster.combat_properties(target_property)),
                        CMoveShape::is_died(target_monster.hit_points()),
                        target_monster.move_shape().is_god(),
                        false,
                        Some(target_master),
                        Some(target_property.clone()),
                        target_monster.is_tamed(),
                        carriage,
                    )
                })
            }),
        _ => None,
    };
    let Some((
        target_shape,
        target_view,
        mut target_health,
        mut target_mana,
        target_war_soul_mana,
        target_player_properties,
        target_monster_properties,
        target_dead,
        target_god,
        target_city_dead,
        target_master,
        target_monster_property,
        target_tamed,
        target_carriage,
    )) = target_snapshot
    else {
        if tamed {
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
        } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    if target_dead
        || ((property.ai != 13 || cast.is_some())
            && (target_god
                || target_city_dead
                || (!tamed
                    && property.kind == 5
                    && target.object_type == PLAYER_TYPE
                    && !game.player_attackable_by_monster(
                        target.id,
                        region.id,
                        &property,
                        tamed,
                        attacker_master,
                    ))))
    {
        if tamed {
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
        } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(monster_x), Ok(monster_y), Ok(target_x), Ok(target_y)) = (
        monster_shape.get_tile_x(),
        monster_shape.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else {
        return true;
    };

    if tamed && cast.is_none() && pet_action == 0 {
        let (anchor_x, anchor_y) = pet_combat_master_anchor(game, region, attacker_master)
            .unwrap_or((monster_x, monster_y));
        let anchor_distance = target_shape.distance_to_point(anchor_x, anchor_y);
        if game.globe_setup().maximum_pet_tracing_distance() as i32 <= anchor_distance {
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
            return true;
        }
    }
    if tamed
        && target.object_type == PLAYER_TYPE
        && attacker_master.master_type == PLAYER_TYPE
        && attacker_master.master_id != 0
        && game
            .find_player(attacker_master.master_id)
            .is_some_and(|master| master.server_region_id() == Some(region.id))
    {
        let pet_identity = ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: monster_id,
            ex_id: CGuid::GUID_INVALID,
        };
        if let Some((string_id, limit)) =
            game.player_base_attack_level_block(attacker_master.master_id, target.id)
        {
            game.send_base_attack_level_block(attacker_master.master_id, string_id, limit);
            game.release_reciprocal_player_target(target.id, pet_identity, runtime);
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
            return true;
        }
        if !game.player_base_attackable(attacker_master.master_id, target.id) {
            game.release_reciprocal_player_target(target.id, pet_identity, runtime);
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
            return true;
        }
    }

    if matches!(property.ai, 11 | 13)
        && cast.is_none()
        && !stationary_bow_target_ready(
            region,
            monster_view,
            target_view,
            skill_properties.query_property(5_004) as i32,
            maximum_distance as i32,
        )
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
            monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
        }
        return true;
    }

    if matches!(property.ai, 10 | 12 | 16)
        && cast.is_none()
        && trace_city_sword_target(
            game,
            region,
            monster_id,
            monster_view,
            target_view,
            skill_properties.query_property(5_004) as i32,
            maximum_distance as i32,
            property.chase_range as i32,
            runtime,
        ) == CitySwordTraceOutcome::Handled
    {
        return true;
    }

    if tamed && pet_action == 2 && cast.is_none() {
        let distance = monster_view.real_distance(Some(target_view));
        let minimum_distance = skill_properties.query_property(5_004) as i32;
        if distance < minimum_distance || distance > maximum_distance as i32 {
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
            return true;
        }
    }

    if let Some(cast) = cast {
        let delay_reached = if matches!(
            cast.dispatch().skill_id,
            COMMON_BASE_ATTACK_SKILL_ID | MONSTER_BASE_ATTACK_SKILL_ID,
        ) {
            cast.started_at_ms().wrapping_add(delay_ms) <= now_ms
        } else {
            time_reached(now_ms, cast.started_at_ms(), delay_ms)
        };
        if !delay_reached {
            return true;
        }
        let dispatch = cast.dispatch();
        let (hit_count, finish_cast) = if matches!(
            dispatch.skill_id,
            MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID
        ) {
            let first_time = skill_properties.query_property(SKILL_USAGE_FIRST_TIME);
            let second_time = skill_properties.query_property(SKILL_USAGE_SECOND_TIME);
            let Some(mut progress) = region
                .find_monster_by_id(monster_id)
                .and_then(CMonster::fast_attack_progress)
            else {
                return true;
            };
            if !progress.visual_started() {
                let fire = fast_attack_fire_message(
                    dispatch.skill_id,
                    dispatch.skill_level,
                    monster_id,
                    target_x,
                    target_y,
                );
                let _ = game.send_game_shape_around(region, &monster_shape, None, &fire);
                progress.mark_visual_started();
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    *monster
                        .fast_attack_progress_mut()
                        .expect("состояние быстрой атаки принадлежит текущему cast") = progress;
                    let _ = monster
                        .advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
                }
            }
            let first_due = time_reached(
                now_ms,
                cast.started_at_ms(),
                delay_ms.wrapping_add(first_time),
            );
            let second_due = time_reached(
                now_ms,
                cast.started_at_ms(),
                delay_ms.wrapping_add(first_time).wrapping_add(second_time),
            );
            let mut hits = 0;
            if !progress.first_attack_done() && first_due {
                progress.mark_first_attack_done();
                hits += 1;
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    *monster
                        .fast_attack_progress_mut()
                        .expect("состояние быстрой атаки принадлежит текущему cast") = progress;
                }
            }
            if progress.first_attack_done() && second_due {
                hits += 1;
            }
            if hits == 0 {
                return true;
            }
            (hits, second_due)
        } else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster
                    .advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
            }
            let mut fire = CMessage::new(0x000b_fe01);
            fire.add_byte(2);
            fire.add_long(dispatch.skill_id as i32);
            fire.add_short(dispatch.skill_level as i16);
            fire.add_long(MONSTER_TYPE);
            fire.add_long(monster_id);
            fire.add_long(target.object_type);
            fire.add_long(target.id);
            fire.add_long(target_x);
            fire.add_long(target_y);
            let _ = game.send_game_shape_around(region, &monster_shape, None, &fire);
            (1, true)
        };

        for hit_index in 0..hit_count {
            if hit_index != 0 {
                if target.object_type == PLAYER_TYPE {
                    let Some(player) = game.find_player(target.id) else { break; };
                    target_health = player.health();
                    target_mana = player.mana();
                } else {
                    let Some(monster) = region.find_monster_by_id(target.id) else { break; };
                    target_health = monster.hit_points();
                    target_mana = 0;
                }
                if target_health == 0 {
                    break;
                }
            }
            let ordinary_attack = region
                .find_monster_by_id(monster_id)
                .map(|monster| {
                    monster.state_attack_bounds(
                        property.minimum_attack,
                        property.maximum_attack,
                    )
                })
                .unwrap_or((property.minimum_attack, property.maximum_attack));
            let physical_minimum = pet_attack_properties
                .map_or(ordinary_attack.0, |pet| pet.minimum_attack) as i32;
            let physical_maximum = pet_attack_properties
                .map_or(ordinary_attack.1, |pet| pet.maximum_attack) as i32;
            let difference = physical_maximum.wrapping_sub(physical_minimum);
            let physical_span = match dispatch.skill_id {
                COMMON_BASE_ATTACK_SKILL_ID => difference.max(0),
                LORD_FAST_ATTACK_SKILL_ID => difference.wrapping_abs().wrapping_add(1),
                _ => difference.max(0).wrapping_add(1),
            };
            let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
            // `CMonster::GetAddElementAtk` остаётся нулевым даже для pet-owner.
            let element = 0;
            if matches!(dispatch.skill_id, COMMON_BASE_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID) {
                let _critical_roll = game.skill_random_below(100);
            }
            let attack = AttackInformation {
                skill_id: dispatch.skill_id,
                skill_level: dispatch.skill_level as u8,
                attacker_type: MONSTER_TYPE,
                attacker_id: monster_id,
                attacker_team_id: 0,
                attacker_faction_id: 0,
                attacker_union_id: 0,
                hit_modifier,
                damage_factor: 1.0,
                damage_modifier: 0,
                critical: false,
                blast_attack: false,
                full_miss: 0,
                damages: vec![
                    AttackPower {
                        kind: AttackPowerType::Physical,
                        hp_damage: physical.max(0),
                        mp_damage: 0,
                    },
                    AttackPower {
                        kind: AttackPowerType::Element,
                        hp_damage: element.max(0),
                        mp_damage: 0,
                    },
                    AttackPower {
                        kind: AttackPowerType::Soul,
                        hp_damage: i32::from(CMonster::resource_soul_attack(&property)),
                        mp_damage: 0,
                    },
                ],
            };
            let attack = defend_owned_monster_attack(
                game,
                target,
                target_mana,
                target_war_soul_mana,
                target_player_properties,
                target_monster_properties,
                attack,
            );
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                if matches!(
                    dispatch.skill_id,
                    MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID
                ) {
                    let _ = monster
                        .advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
                    if finish_cast && hit_index + 1 == hit_count {
                        let _ = monster
                            .advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
                    }
                } else {
                    let _ = monster
                        .advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
                    let _ = monster
                        .advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
                }
            }
            apply_owned_monster_attack_hit(
                game,
                region,
                runtime,
                now_ms,
                monster_id,
                attacker_master,
                target,
                &target_shape,
                target_health,
                target_mana,
                target_master,
                target_monster_property.clone(),
                target_tamed,
                target_carriage,
                attack,
                deaths,
            );
        }
        if finish_cast
            && let Some(monster) = region.find_monster_by_id_mut(monster_id)
        {
            monster.move_shape_mut().shape_mut().set_action(1);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    if !approach_attack_range(
        game,
        region,
        monster_id,
        MonsterTraceTarget::Shape(target_view),
        maximum_distance,
        now_ms,
    ) {
        if tamed && pet_action == 2 {
            lose_pet_target_and_search(region, monster_id, stop_frame, runtime);
        }
        return true;
    }
    let attack_interval = schedule_attack_interval(
        property.ai,
        pet_attack_properties.map_or(property.attack_speed, |pet| pet.attack_interval),
    );
    if let Some(attack_interval) = attack_interval {
        let attack_started = region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, attack_interval));
        if !attack_started {
            return true;
        }
    }
    let last_used_ms = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.skill_last_used_ms(skill_id))
        .unwrap_or_default();
    if !skill_is_restored(last_used_ms, reuse_delay_ms, now_ms) {
        return true;
    }
    let direction = get_line_direction(monster_x, monster_y, target_x, target_y);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .shape_mut()
            .set_direction(direction);
        monster.begin_base_attack_cast(target, skill_id, skill.level, now_ms);
        if fast_attack {
            monster.begin_fast_attack_progress();
        }
    }
    let mut start = CMessage::new(0x000b_fe01);
    start.add_byte(1);
    start.add_long(skill_id as i32);
    start.add_short(skill.level as i16);
    start.add_long(MONSTER_TYPE);
    start.add_long(monster_id);
    start.add_long(direction);
    let _ = game.send_game_shape_around(region, &monster_shape, None, &start);
    true
}
