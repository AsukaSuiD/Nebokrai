//! Попадания семейства GhostCut по фигурам клетки и вынесенной боевой фее.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/ghostcut*.cpp.
//!
//! Только первый вариант сначала обходит упорядоченную карту боевых фей.
//! Их владельцы разрешаются глобально, исключаются self, action6 и IsDied;
//! после живого IsAttackAble удар идёт с warSoul=true, без списка целей и RP.
//! Обычный GetShapes выполняется после этой ветви. Все CMoveShape проходят
//! живой допуск без фильтра смерти; затем локальная дедупликация клетки и
//! постоянный список канонического экземпляра. Постоянная отметка ставится
//! до Calculate, локальная — после Attack. End очищает настоящий список,
//! поэтому через callbacks он не извлекается и не восстанавливается копией.
//! Контакт использует общий оружейный расчёт с сырой шириной RNG; отсутствие
//! таблицы Calculate сохраняет default UNKNOWN/1 и не отменяет доставку/RP.

use super::flash::cell_views;
use super::ghostcut::GhostCutExecutionState;
use super::skillfactory::SkillOwner;
use super::weaponattack::{PlayerWeaponRoll, calculate_player_weapon_attack};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), war_soul: bool, runtime: &mut Runtime,
) {
    if !war_soul && !game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<GhostCutExecutionState>())
        .is_some_and(|state| state.mark_target_attacked(target.1))
    {
        return;
    }
    let Some((master, attack)) = calculate_player_weapon_attack(
        game, instance, source, target, TARGET_DAMAGE_FACTOR, PlayerWeaponRoll::RawRange,
    ) else { return; };
    if war_soul {
        game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
    } else {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
        game.increase_owned_player_rp(source.1.id, true, 0);
    }
}

pub(super) fn attack_ghost_cut_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    if x == 0 && y == 0 { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let has_war_soul_attack = skill.owner() == SkillOwner::CGhostCut;
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let shape = user.shape();
    if !shape.is_assigned_to_server_region() { return; }
    let region_id = shape.get_region_id();
    let user_identity = shape.identity();
    let Some(owner) = game.find_region(region_id) else { return; };
    if has_war_soul_attack {
        let war_souls = owner.base().war_souls_at(x, y);
        for (player_id, _) in war_souls {
            let Some(target) = game.find_player(player_id as i32) else { continue; };
            if target.player_id() == user_identity.id || target.shape().get_action() == 6
                || target.is_dead()
            {
                continue;
            }
            let identity = target.shape().identity();
            let target_region = target.shape().get_region_id();
            if game.live_skill_target_attackable(target_region, user_identity, identity) {
                attack_target(game, instance, source, (target_region, identity), true, runtime);
            }
        }
    }
    let mut seen_in_cell = Vec::new();
    for view in cell_views(game, region_id, x, y) {
        let Some(target) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
        let identity = target.shape().identity();
        if identity == user_identity { continue; }
        let target_region = target.shape().get_region_id();
        if !game.live_skill_target_attackable(target_region, user_identity, identity)
            || seen_in_cell.contains(&identity)
        {
            continue;
        }
        attack_target(game, instance, source, (target_region, identity), false, runtime);
        seen_in_cell.push(identity);
    }
}
