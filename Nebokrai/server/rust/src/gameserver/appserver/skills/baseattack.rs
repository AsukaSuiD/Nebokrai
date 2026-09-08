//! Базовая атака GameServer (`SKILL_BASE_ATTACK == 1`).
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/baseattack.cpp`. Первый такт AI проверяет дальность,
//! поворачивает игрока и публикует действие 0; после
//! `SKILL_USAGE_DELAY_TIME` действие 1 предшествует расчёту атаки. Мёртвая
//! цель завершает навык кодом 2, удалённая цель — кодом `0x0b`.
//! `SkillExecutionKernel` хранится в `CPlayerAI` и переживает задержку между
//! тактами. Формулы PvP, RNG и построение пакетов находятся в соседнем
//! модуле исполнения навыка; `CGame` разрешает владельцев и применяет урон.
//! Общий хвост подтверждённых `CAttackSkill::End` сохраняет восстановление
//! движения, `AfterUseSkill`, время восстановления и очистку исполнения;
//! конкретный владелец явно выбирает задержанный или немедленный вариант.
//! CSkill::End (0x004D84C0) вызывает virtual +0x158, у CPlayer это пустой
//! 0x00485540, а не UpdateProperty. Дополнительного пересчёта свойств нет;
//! изменения от износа оружия обслуживает сам OnWeaponDamaged (0x00441D50).
//! Luvinia EndSkill освобождает CBaseModule нового движка: этот lifecycle
//! не является соответствием нашим CSkill/AfterUseSkill.
//! При входе в другой регион исходный `OnChangeRegion` выполняет `End(false)`:
//! движение возвращается и исполнение освобождается без износа оружия и фиксации
//! времени восстановления. Координатный `Begin` разрешает первый `CMoveShape`
//! клетки через точный `CState::GetSufferer` без fallback к заклинателю.
//! Объектное исполнение навыка монстром проходит `monsterbaseattack`: ID `1`
//! сохраняется, физический разброс исключает верхнюю границу, а critical-roll
//! выполняется и при нулевом monster `GetCCH`. Это не подмена навыком `0x2bd`.
//! CheckCastCondition (0x005B2E40) требует только источник и свойства,
//! не проверяя reuse. Monster Begin сохраняет Begin-фазу без поворота/пакета.
//! Первый AI (0x005B39B0) проверяет unsigned RealDistance, при отказе
//! выполняет End(0), иначе поворачивает и публикует старт до delay.
//! Расписание сохраняет отдельный GetAttackSpeed; пропуск reuse навыка
//! не пропускает этот таймер. AI (0x005B39B0) завершает мёртвую цель
//! через End(1); failure 2 message-owner-а адресован только player-источнику.
//! При исчезновении объекта использует нулевой fallback CState::Begin:
//! первый AI проверяет его дальность, после delay отправляет fire с нулевой
//! identity/координатами (CBaseAttackEffect, 0x005B3100) и выполняет End(1)
//! без RNG/Attack. Эти ветви не отменяют AI-цель или движение. Для живой
//! цели Attack проверяет self/IsAttackAble после fire, перед RNG; отказ
//! оставляет AI-цель и движение, но не отменяет последующий End(1).
//! Разрешение цели и политика отношений переиспользуют общий monsterattack;
//! NPC не входят в его боевой снимок и у исходного Attack исключены отдельно.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::real_distance_between_points;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillTermination};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const BASE_ATTACK_SKILL_ID: u32 = 1;

#[allow(clippy::too_many_arguments, reason = "контекст исходного Attack и его IsAttackAble")]
pub(crate) fn owned_monster_base_attack_allowed(
    game: &CGame,
    region: &crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    property: &crate::setup::monsterlist::MonsterProperties,
    tamed: bool,
    master: crate::gameserver::appserver::masterinfo::MasterInfo,
    target: crate::gameserver::appserver::shape::ShapeIdentity,
) -> bool {
    if target.object_type == 600 && target.id == monster_id {
        return false;
    }
    super::monsterattack::resolve_owned_monster_attack_target(game, region, target)
        .is_some_and(|snapshot| {
            !snapshot.god && !snapshot.city_dead
                && super::monsterattack::owned_monster_attackable(
                    game, region.id, property, tamed, master, target, &snapshot,
                )
        })
}

pub(crate) fn begin_owned_monster_base_attack(
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    target: crate::gameserver::appserver::shape::ShapeIdentity,
    skill_level: u16,
    started_at_ms: u32,
) {
    use crate::gameserver::appserver::monster::{MonsterBaseAttackCast, MonsterBaseAttackDispatch};
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.install_base_attack_cast(MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
            target, skill_id: BASE_ATTACK_SKILL_ID, skill_level,
        }, started_at_ms));
    }
}

pub(crate) fn start_owned_monster_base_attack_ai(
    game: &CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    source: crate::gameserver::appserver::shape::ShapeView,
    target: Option<crate::gameserver::appserver::shape::ShapeView>,
    maximum_distance: u32,
) -> bool {
    use super::kernel::SkillStage;
    let distance = if let Some(target) = target {
        source.real_distance(Some(target))
    } else {
        let Some(monster) = region.find_monster_by_id(monster_id) else { return false };
        monster.move_shape().shape().real_distance_to_point(0, 0)
    };
    if maximum_distance != 0 && distance as u32 > maximum_distance {
        let _ = super::monsterattack::end_owned_monster_skill_without_reuse(region, monster_id, BASE_ATTACK_SKILL_ID);
        return false;
    }
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false };
    let Some(cast) = monster.current_active_attack_cast() else { return false };
    let (target_x, target_y) = target.map_or((0, 0), |target| (target.tile_x, target.tile_y));
    monster.move_shape_mut().shape_mut().set_direction(crate::public::tools::get_line_direction(
        source.tile_x, source.tile_y, target_x, target_y,
    ));
    let shape = monster.move_shape().shape().clone();
    let mut start = crate::nets::netserver::message::CMessage::new(0x000b_fe01);
    start.add_byte(1);
    start.add_long(BASE_ATTACK_SKILL_ID as i32);
    start.add_short(cast.dispatch().skill_level as i16);
    start.add_long(600);
    start.add_long(monster_id);
    start.add_long(shape.get_direction());
    let _ = game.send_game_shape_around(region, &shape, None, &start);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Begin, SkillStage::Check);
    }
    true
}

pub(crate) fn handle_owned_monster_base_target_loss<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    source: crate::gameserver::appserver::shape::ShapeView,
    properties: &super::skillbaseproperties::CSkillBaseProperties,
    runtime: &mut Runtime,
) -> bool {
    use super::kernel::SkillStage;
    let Some(cast) = region.find_monster_by_id(monster_id).and_then(|monster| monster.current_active_attack_cast()) else { return false };
    if cast.dispatch().skill_id != BASE_ATTACK_SKILL_ID { return false }
    let target = super::monsterattack::resolve_owned_monster_attack_target(game, region, cast.dispatch().target);
    if let Some(target) = target {
        if !target.dead { return false }
    } else {
        if cast.stage() == SkillStage::Begin && !start_owned_monster_base_attack_ai(
            game, region, monster_id, source, None,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
        ) {
            return true;
        }
        if !super::kernel::skill_is_restored(
            cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME), runtime.now_milliseconds(),
        ) {
            return true;
        }
        let Some(monster) = region.find_monster_by_id(monster_id) else { return true };
        let mut fire = crate::nets::netserver::message::CMessage::new(0x000b_fe01);
        fire.add_byte(2);
        fire.add_long(BASE_ATTACK_SKILL_ID as i32);
        fire.add_short(cast.dispatch().skill_level as i16);
        fire.add_long(600);
        fire.add_long(monster_id);
        for _ in 0..4 { fire.add_long(0); }
        let _ = game.send_game_shape_around(region, monster.move_shape().shape(), None, &fire);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
    }
    true
}
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5003;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_USER_HIT_MODIFIER: u32 = 20_001;

pub(crate) type BaseAttackExecutionState = SkillExecutionKernel<PlayerSkillDispatch>;

pub(crate) const fn time_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    now_ms.wrapping_sub(started_at_ms) >= delay_ms
}

pub(crate) fn real_distance(source_x: i32, source_y: i32, target_x: i32, target_y: i32) -> i32 {
    real_distance_between_points(source_x, source_y, target_x, target_y)
}

/// End(0) не вызывает AfterUseSkill. Projectile-варианты (0x005AE7A0)
/// возвращают движение до CSummonSkill::End; CBaseAttack (0x005B3010) — нет.
/// Освобождение kernel выполняет общий хвост очереди после возврата результата.
pub(crate) fn finish_failed_base_attack(game: &mut CGame, player_id: i32, restore_movement: bool) {
    if let Some(player) = game.find_player_mut(player_id) {
        if restore_movement {
            player.set_skill_moveable(true);
        }
    }
}

/// Общий достигнутый хвост `CBaseAttack::End`, `CBaseMagic::End` и
/// `CArchery::End`: износ оружия
/// предшествует фиксации времени восстановления; CSkill::End не сбрасывает
/// выбранный навык игрока (его +0x158 — пустой ret 0x00485540);
/// задержанные варианты сначала возвращают движение.
fn finish_base_attack_owner<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    restore_movement: bool,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    if restore_movement
        && let Some(player) = game.find_player_mut(player_id)
    {
        player.set_skill_moveable(true);
    }
    game.damage_player_weapon(player_id, runtime);
    mark_used(player_ai, runtime.now_milliseconds());
}

pub(crate) fn finish_delayed_base_attack<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    finish_base_attack_owner(game, player_id, player_ai, runtime, true, mark_used);
}

pub(crate) fn finish_immediate_base_attack<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    finish_base_attack_owner(game, player_id, player_ai, runtime, false, mark_used);
}

pub(crate) fn finish_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_immediate_base_attack(
        game,
        player_id,
        player_ai,
        runtime,
        |player_ai, now_ms| player_ai.mark_skill_used(BASE_ATTACK_SKILL_ID, now_ms),
    );
}

pub(crate) fn cancel_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_base_attack(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn abort_player_base_attack_on_region_change(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp

// ============================================================================
// FUNCTION: CBaseAttack::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:110
// RVA: 0x00113E00
// ADDRESS: 00513e00
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:18
// RVA: 0x001B2DB0
// ADDRESS: 005b2db0
// PROTOTYPE: undefined __thiscall CBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::~CBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:26
// RVA: 0x001B2E20
// ADDRESS: 005b2e20
// PROTOTYPE: void __thiscall ~CBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:33
// RVA: 0x001B2E40
// ADDRESS: 005b2e40
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:81
// RVA: 0x001B2F40
// ADDRESS: 005b2f40
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:48
// RVA: 0x001B3040
// ADDRESS: 005b3040
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttackEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:346
// RVA: 0x001B3100
// ADDRESS: 005b3100
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:279
// RVA: 0x001B3600
// ADDRESS: 005b3600
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:248
// RVA: 0x001B3860
// ADDRESS: 005b3860
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:123
// RVA: 0x001B39B0
// ADDRESS: 005b39b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
