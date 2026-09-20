//! Летающий рубящий удар CYakshaSlash (0x196) для игрока и монстра.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/yakshaslash.cpp.
//!
//! Check требует исходные U/S, но допускает self до первого AI. Reuse
//! проверяется по абсолютному unsigned-сроку. Ненулевой MAX читается дважды:
//! между запросами RealDistance использует исходные объекты, а сравнение
//! трактует MAX как signed DWORD. Затем свежий GetTargetPath проверяется
//! только на BLOCK_UNFLY. Move0 применяется ко всем CMoveShape; MP, оружие
//! и системные текстовые сообщения этот навык не проверяет.
//!
//! Общий targetedprojectile хранит единственные condition/attacking/flight:
//! первый AI задаёт CAN и направление, выпуск после start+delay возвращает
//! движение, заново проверяет преграды и публикует flight до prepared.
//! Контакт ждёт unsigned start+delay+flight и не вызывает IsAttackAble.
//! S и U захвачены в начале AI; путь и visual разрешают участников заново.
//!
//! PK-поля сохраняются до свежего Calculate. NULL таблица оставляет
//! UNKNOWN/1, но не отменяет сырой OnBeenAttacked. Коэффициент — unsigned
//! DAMAGE_FACTOR × 0.01_f32 с единственной записью float, без weapon modifier.
//! Общий расчёт компонентов сохраняет MAX→MIN→RNGabs→fresh MIN, живые
//! ELEMENT/SOUL/CCH и критическое усечение. Здесь нет DEX, RP и состояний.
//!
//! Player и monster используют один AI и общий зарегистрированный End:
//! phase/attacking/flight → свежий U Move1 → AttackEnd с фактическим аргументом.
//! Monster-адаптер публикует настоящий регион на время callbacks; выбор цели,
//! расписание и очередь остаются у его AI. Отдельного monster-снаряда нет.
//! Monster/Pet OnFighting ждёт IsEnded: prepared-полёт остаётся активной атакой.

use super::basemagic::{SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::kernel::skill_is_restored;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::targetedprojectile::{TargetedProjectileProgress, run_targeted_projectile_ai};
use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage, source_master};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};

pub(crate) const YAKSHA_SLASH_SKILL_ID: u32 = 0x196;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const USER_HIT_MODIFIER: u32 = 20_001;
const PLAYER_TYPE: i32 = 400;

fn original_shape_view(game: &CGame, object: (i32, ShapeIdentity)) -> Option<ShapeView> {
    if object.1.object_type == PLAYER_TYPE {
        game.find_player(object.1.id)?.shape_view()
    } else {
        game.shape_view_in_owner(game.find_region(object.0)?, object.1)
    }
}

pub(super) fn check_yaksha_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let (Some(user), Some(target)) = (original_user, original_target) else { return false; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return false; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return false; };
    let user = (source.shape().get_region_id(), source.shape().identity());
    let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        return false;
    }
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
        let Some(source) = original_shape_view(game, user) else { return false; };
        let Some(target) = original_shape_view(game, target) else { return false; };
        let distance = source.real_distance(Some(target));
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as i32;
        if maximum < distance {
            game.update_registered_skill_visual(instance, 11);
            return false;
        }
    }
    let path = game.skill_target_path(skill.lifecycle());
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_registered_skill_visual(instance, 15);
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) else { return false; };
    source.set_moveable(false);
    true
}

pub(super) fn apply_yaksha_slash_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let mut attack = AttackInformation::for_master(master);
    if let Some(skill) = game.registered_skill(instance)
        && let Some(properties) = game.skill_base_properties(skill.id(), skill.level())
    {
        attack.skill_id = skill.id();
        attack.skill_level = skill.level() as u8;
        attack.damage_modifier = 0;
        let factor = properties.query_property(TARGET_DAMAGE_FACTOR);
        attack.damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
        attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
        fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::AbsoluteRange, &mut attack);
    }
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

struct YakshaSlashSkill;

impl RegisteredStateSkill for YakshaSlashSkill {
    const ID: u32 = YAKSHA_SLASH_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::TargetedProjectile;

    fn prepare_monster(skill: &mut MoveShapeSkill) {
        skill.set_monster_progress(TargetedProjectileProgress::default());
    }

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill,
        begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        // Monster Begin до Check только записывает базу и visual: исходный U
        // ещё не мог измениться через callback. Player передаёт аргумент отдельно.
        let (region, identity) = skill.lifecycle().user();
        let original_user = resolve_state_move_shape(game, region, identity)
            .map(|source| (source.shape().get_region_id(), source.shape().identity()));
        let original_target = begin_target.resolve(game, skill, false);
        check_yaksha_slash(game, instance, original_user, original_target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_targeted_projectile_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_yaksha_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<YakshaSlashSkill, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
