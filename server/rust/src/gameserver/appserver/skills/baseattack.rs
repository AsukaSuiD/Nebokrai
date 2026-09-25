//! Базовая атака CBaseAttack (1): оставшийся приёмник после переноса runtime.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/baseattack.cpp.
//! Исполнение игрока и монстра (порядок стадий AI, формула, visual, abort-формы
//! и общий terminal End) перенесено буквально в
//! `nebokrai_zone::skills::baseattackruntime` (основание и статусы MATCH см.
//! там); публичные входы ниже остаются тонкими делегациями с прежними
//! сигнатурами, потребители не меняются.
//! Здесь остаются caller Begin монстра (hub `CMonster`), общие
//! delayed/immediate helpers не переведённых на полный End caller-ов и
//! технические helpers расстояния/времени.
//! End(1) изнашивает оружие только у игрока и фиксирует reuse; End(0) не делает ни того, ни другого.
//! Выбранный навык меняет AI, а не пустой callback OnEndSkill.
//! Monster-origin получает Begin здесь, а собственный AI исполнения — у zone-владельца.
//! Достижимость отдельного Restart (0x00113E00) по-прежнему UNKNOWN;
//! метаданные его исследования перенесены в zone-владельца.

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::shape::real_distance_between_points;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner, game_tick_milliseconds};

pub(crate) use nebokrai_zone::skills::{
    BASE_ATTACK_SKILL_ID, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
};

pub(crate) fn begin_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) {
    let target_object = resolve_owned_skill_begin_object(game, region, target);
    let started = runtime.now_milliseconds();
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return; };
    if !monster.prepare_base_attack_cast(
        target, BASE_ATTACK_SKILL_ID, skill_level, started, target_object, game.skill_factory(),
    ) { return; }
    monster.move_shape_mut().replace_skill_visual_effect(
        BASE_ATTACK_SKILL_ID, game.skill_factory(), SkillVisualEffect::new(SkillVisualEffectKind::BaseAttack, 1),
    );
    let valid = monster.move_shape().skill(BASE_ATTACK_SKILL_ID, game.skill_factory())
        .is_some_and(|skill| game.skill_base_properties(BASE_ATTACK_SKILL_ID, skill.level()).is_some());
    if valid {
        monster.enqueue_base_attack_cast(started);
    } else {
        // CMonster::OnBeginSkill — RET1, AfterUse для него пуст. Отказ
        // CBaseAttack::Begin вызывает только End(0), без собственного visual 2.
        let _ = monster.finish_base_attack_cast_without_reuse(BASE_ATTACK_SKILL_ID, game.skill_factory());
    }
}

pub(crate) const fn time_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    now_ms.wrapping_sub(started_at_ms) >= delay_ms
}

pub(crate) fn real_distance(source_x: i32, source_y: i32, target_x: i32, target_y: i32) -> i32 {
    real_distance_between_points(source_x, source_y, target_x, target_y)
}

/// Отказ ещё не переведённых projectile-caller-ов восстанавливает движение
/// без AfterUse. Сам kernel освобождает общий хвост очереди.
pub(crate) fn finish_failed_base_attack(game: &mut CGame, player_id: i32, restore_movement: bool) {
    if let Some(player) = game.find_player_mut(player_id) {
        if restore_movement {
            player.set_skill_moveable(true);
        }
    }
}

/// Частичный хвост ещё не переведённых caller-ов: восстановление движения
/// при необходимости, затем износ оружия и reuse. Это не полный End.
fn finish_base_attack_owner<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
    restore_movement: bool,
) {
    if restore_movement
        && let Some(player) = game.find_player_mut(player_id)
    {
        player.set_skill_moveable(true);
    }
    game.after_use_player_skill(player_id, skill_id, runtime);
}

pub(crate) fn finish_delayed_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    finish_base_attack_owner(game, player_id, skill_id, runtime, true);
}

pub(crate) fn finish_immediate_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    finish_base_attack_owner(game, player_id, skill_id, runtime, false);
}

pub(crate) fn cancel_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    nonzero_end: bool,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::skills::cancel_player_base_attack(
        game, player_id, player_ai, nonzero_end, runtime,
    )
}

pub(crate) fn abort_player_base_attack_on_region_change(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
) -> bool {
    nebokrai_zone::skills::abort_player_base_attack_on_region_change(
        game, player_id, player_ai,
    )
}

/// Публикуется исходный регион целиком: visual, приём удара и вложенный End
/// работают с тем же экземпляром. FIFO и выбор следующего навыка остаются у AI.
pub(crate) fn execute_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::skills::execute_owned_monster_base_attack(
        game, owner, monster_id, runtime, game_tick_milliseconds,
    )
}
