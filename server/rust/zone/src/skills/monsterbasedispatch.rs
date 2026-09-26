//! Диспетчерская часть `CMonsterBaseAttack`: взвешенный выбор боевого навыка
//! монстра, `OnChangeSkill` с проверкой `CSkill::IsRestored` и продолжение
//! уже начатого cast-а из `OnFighting` через реестр зарегистрированных
//! исполнителей.
//!
//! Реестр исполнителей `owned_registered_cast_executor` НЕ переносится и
//! остаётся владением старого `appserver/skills/monsterbaseattack.rs`; сюда
//! перенесён только диспетчерский костяк (continue-решение `OnFighting`,
//! выбор и смена навыка). Специфические selector-ы боссов, владыки и
//! стационарных лучников — швы своих групп (`bossblue`, `bossfiend`, `lord`,
//! `fixedpositionarcher`).
//!
//! UNKNOWN: состав и порядок обхода массивов default-ID по категориям
//! `GetDefaultAttackSkillID` — у владельца реестра `moveshape`.
//!
//! Исходный владелец PDB: `appserver/skills/monsterbaseattack.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#monsterbasedispatch--диспетчер-cmonsterai

use nebokrai_shared::resources::MonsterProperties;

use crate::ai::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherRegion, select_attack_skill,
};
use crate::regions::ShapeIdentity;
use crate::skills::skill_is_restored;

use super::baseattackruntime::SKILL_USAGE_REUSE_DELAY_TIME;

/// Продолжение уже зарегистрированного исполнения монстра: первый вход
/// `OnFighting` dispatcher-а. Реестр исполнителей и делегат общей
/// `CBaseAttack` остаются старым пакетом через швы ниже.
pub trait MonsterBaseDispatchGame: MonsterDispatcherGame {
    // Selector-группы владельцев производных AI (реализация у старого пакета).
    /// Пороговый выбор BossBlue (AI103) по здоровью и зарегистрированному Fury.
    fn choose_boss_blue_attack_skill(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        property: &MonsterProperties,
        hit_points: u32,
        roll: i32,
        default_skill_id: u16,
    ) -> Option<u16>;

    /// Фазовый выбор владыки (AI100) по доле здоровья от максимума.
    fn select_lord_attack_skill(
        hit_points: u32,
        maximum_hit_points: u32,
        skills: &[nebokrai_shared::resources::MonsterSkill],
        roll: i32,
        default_skill_id: u16,
    ) -> u16;

    /// Стационарная семья (AI5 и наследующий AI23) вместо отката ждёт полный
    /// restore delay в собственном FIFO.
    fn fixed_archer_change_skill_inherited(ai_type: u32) -> bool;
}

/// Маршруты диспетчера, связанные runtime-ом главного цикла (реестр
/// исполнителей, делегат общей базовой атаки и соседние selector-владельцы с
/// их прежними generic-границами). Реализация у старого пакета; generic-шов
/// соответствует ADR-0013, как у `skills/baseattackruntime.rs`.
pub trait MonsterBaseDispatchRuntime<Runtime>: MonsterBaseDispatchGame {
    /// Executor-реестр старого пакета: `Some(result)` при подключённом
    /// владельце навыка, `None` — если ID не зарегистрирован (dispatcher
    /// продолжает общий путь расписания). Сохранённый результат `bool`
    /// соответствует прежнему возврату executor-а.
    fn execute_registered_cast(
        &mut self,
        owner: &mut Option<Self::RegionOwner>,
        monster_id: i32,
        skill_id: u32,
        target: ShapeIdentity,
        skill_level: u16,
        runtime: &mut Runtime,
    ) -> Option<bool>;

    /// Продолжение `CBaseAttack` (`1`) общего монстра из `OnFighting`.
    fn continue_common_base_attack(
        &mut self,
        owner: &mut Option<Self::RegionOwner>,
        monster_id: i32,
        runtime: &mut Runtime,
    ) -> bool;

    /// Пороговый выбор BossFiend (AI104) поверх общего RNG ритма региона.
    fn choose_boss_fiend_attack_skill(
        &self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        property: &MonsterProperties,
        hit_points: u32,
        roll: i32,
        runtime: &mut Runtime,
    ) -> Option<u16>;

    /// Постановка restore-delay в хвост FIFO стационарного владельца.
    fn queue_fixed_archer_skill_delay(
        &self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        property: &MonsterProperties,
        selected_skill_id: u16,
        runtime: &mut Runtime,
    ) -> bool;
}

/// Продолжение активного cast-а из `OnFighting`: общая `CBaseAttack`, затем
/// реестр по точному ID. `None` означает «нет живого cast либо зарегистрированного
/// executor-а» — dispatcher продолжает общий путь OnSchedule ниже, как и
/// раньше (ранний `return` в исходном теле возникал только в этих ветвях).
pub fn continue_active_attack_cast<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
) -> Option<bool>
where
    Game: MonsterBaseDispatchRuntime<Runtime>,
{
    let cast = owner
        .as_ref()?
        .base()
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.current_active_attack_cast(game.skill_factory()))?;
    let dispatch = cast.dispatch();
    if dispatch.skill_id == super::baseattackruntime::BASE_ATTACK_SKILL_ID {
        return Some(game.continue_common_base_attack(owner, monster_id, runtime));
    }
    game.execute_registered_cast(
        owner,
        monster_id,
        dispatch.skill_id,
        dispatch.target,
        dispatch.skill_level,
        runtime,
    )
}

/// Взвешенный выбор `SelectAttackSkill` (RVA `0x1DD0B0`) с boss/lord
/// ответвлениями и записью выбранного ID каноническим полем формы.
/// `SelectAttackSkill` исполняется один раз за вызов: один RNG на весь список.
pub fn select_and_store_monster_attack_skill<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    monster_health: u32,
    runtime: &mut Runtime,
) -> Option<u16>
where
    Game: MonsterBaseDispatchRuntime<Runtime>,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let monster = region.find_monster_by_id(monster_id)?;
    let default_skill_id = monster.move_shape().default_attack_skill_id() as u16;
    let primary_ai = monster.active_primary_ai_type();
    let roll = game.skill_random_below(10_000);
    let selected = if primary_ai == Some(21) {
        game.choose_boss_blue_attack_skill(
            region,
            monster_id,
            property,
            monster_health,
            roll,
            default_skill_id,
        )
    } else if primary_ai == Some(23) {
        game.choose_boss_fiend_attack_skill(
            region,
            monster_id,
            property,
            monster_health,
            roll,
            runtime,
        )
    } else if primary_ai == Some(19) {
        Some(Game::select_lord_attack_skill(
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

/// Выполняет `CMonsterAI::OnChangeSkill` из FIFO либо непосредственно OnSchedule.
/// После единственного weighted RNG выбранный concrete skill проверяется через
/// `CSkill::IsRestored`; отсутствующий или ещё не восстановленный навык общего
/// monster AI заменяется `GetDefaultAttackSkillID`. AI5 и наследующий его
/// AI23 сохраняют существующий навык на cooldown и ставят полный restore
/// delay в хвост FIFO. Boss-specific пороги остаются в своих selector-owner-ах.
pub fn change_owned_monster_attack_skill<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterBaseDispatchRuntime<Runtime>,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((property, monster_health, primary_ai)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.hit_points(),
                monster.active_primary_ai_type(),
            ))
        })
    else {
        return false;
    };
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
    if primary_ai.is_some_and(Game::fixed_archer_change_skill_inherited) {
        if game.queue_fixed_archer_skill_delay(
            region,
            monster_id,
            &property,
            selected_skill_id,
            runtime,
        )
        {
            return true;
        }
    } else if region.find_monster_by_id(monster_id)
        .and_then(|monster| monster.move_shape().current_skill(game.skill_factory()))
        .and_then(|skill| {
            let properties = game.skill_base_properties(
                skill.id(),
                skill.level(),
            )?;
            let last_used_ms = region
                .find_monster_by_id(monster_id)?
                .skill_last_used_ms(u32::from(selected_skill_id), game.skill_factory());
            Some(skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_milliseconds(),
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
