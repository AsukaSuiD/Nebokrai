//! ИИ неподвижного лучника `CFixedPositionArcher` (AI5) и наследующая его
//! стационарная семья: строгая очередь `OnIdle`, хвост `OnChangeSkill` с
//! ожиданием восстановления навыка и правило `OnSearchEnemy` с минимальной
//! дистанцией текущего навыка. Исходный владелец PDB:
//! `appserver/ai/fixedpositionarcher.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb` (разобраны все четыре метода класса,
//! кроме ctor).
//!
//! Наследование хвоста `OnChangeSkill` ограничено точным набором {5, 23}:
//! собственный виртуальный конструктор `CVilCouGuardWithBow` строит
//! `CMonsterAI` напрямую, и стационарный idle сам по себе хвост не наследует.
//! Варианты `OnFighting` живого владельца и StupidArcher — по прежней таблице
//! владельца. Остаются hub-владением: реальный путь `monsterbaseattack` и общий
//! monster tick; реестр навыков и `GetRestoreTime` — hub-шов
//! [`MonsterDispatcherMoveShape`].
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use nebokrai_shared::resources::MonsterProperties;

use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::shape::ShapeView;

use super::events::AiShapeAction;
use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherPlayer, MonsterDispatcherRegion, uses_stationary_attack_schedule,
};
use crate::skills::SKILL_USAGE_REUSE_DELAY_TIME;
use crate::skills::execution::RegisteredSkillRecord;
use crate::skills::skill_is_restored;
use crate::skills::skillfactory::CSkillFactory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixedArcherTarget {
    pub identity: ShapeIdentity,
    pub distance: i32,
}

/// OnFighting `0x0060FA70` сначала вызывает `CBaseAI` `0x004C9320`
/// (ChangeSkill), затем добавляет SearchEnemy. Вариант `0x0060D930`
/// добавляет поиск только живому владельцу; StupidArcher `0x0060F6F0` не
/// вызывает базовый обработчик. Без текущего навыка базовое событие
/// отсутствует; производный поиск стационарных владельцев сохраняется, но
/// StupidArcher ничего не ставит.
pub const fn attack_completion_actions(
    ai_type: u32,
    alive: bool,
    skill_ended: bool,
) -> &'static [AiShapeAction] {
    if ai_type == 6 {
        if skill_ended { &[AiShapeAction::SearchEnemy] } else { &[] }
    } else if matches!(ai_type, 5 | 23)
        || (alive && uses_stationary_attack_schedule(ai_type))
    {
        if skill_ended { &[AiShapeAction::ChangeSkill, AiShapeAction::SearchEnemy] }
        else { &[AiShapeAction::SearchEnemy] }
    } else if skill_ended {
        &[AiShapeAction::ChangeSkill]
    } else {
        &[]
    }
}

pub const fn inherits_fixed_archer_change_skill(ai_type: u32) -> bool {
    matches!(ai_type, 5 | 23)
}

/// Подвижная форма стационарной семьи: переходный фасад прежнего
/// `CMoveShape`, открывающий запись выбранного навыка по ID. Оставшаяся
/// setup-запись после отказа `AddSkill` не создаёт фиктивного навыка.
pub trait FixedArcherDispatcherMoveShape: MonsterDispatcherMoveShape {
    fn skill(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&RegisteredSkillRecord<Self::Execution>>;
}

/// Ставит общую точную очередь стационарного `OnIdle`. Каждый исходный
/// `AddAIEvent` получает отдельный замер часов.
pub fn queue_stationary_guard_idle<Region>(
    region: &mut Region,
    monster_id: i32,
    stop_frame: u32,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Region: MonsterDispatcherRegion,
{
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    monster.begin_active_ai_change_skill(now_milliseconds());
    monster.begin_active_ai_stand(stop_frame, now_milliseconds());
    monster.begin_active_ai_search_enemy(now_milliseconds());
    true
}

/// Выполняет производный хвост `OnChangeSkill` AI5 и наследующего его AI23.
/// `false` означает, что выбранный concrete skill не разрешился и общий
/// owner обязан назначить default skill. Разрешённый выбор сохраняется; для
/// ещё не восстановленного навыка полный `GetRestoreTime` дописывается в
/// FIFO. Отдельный вызов часов повторяет точный `CSkill::IsRestored`.
pub fn queue_fixed_archer_skill_delay<Game, Region>(
    game: &Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    selected_skill_id: u16,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: MonsterDispatcherMonster<MoveShape: FixedArcherDispatcherMoveShape>,
{
    if !inherits_fixed_archer_change_skill(property.ai) {
        return false;
    }
    let Some(skill) = region.find_monster_by_id(monster_id).and_then(|monster| {
        monster
            .move_shape()
            .skill(u32::from(selected_skill_id), game.skill_factory())
    })
    else {
        return false;
    };
    let Some(skill_properties) = game.skill_base_properties(skill.id(), skill.level()) else {
        return false;
    };
    let delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let last_used_ms = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.skill_last_used_ms(skill.id(), game.skill_factory()))
        .unwrap_or_default();
    let restored_at_ms = now_milliseconds();
    if skill_is_restored(last_used_ms, delay_ms, restored_at_ms) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_stand(delay_ms, now_milliseconds());
    }
    true
}

/// Повторяет необычное правило `OnSearchEnemy`: выбирается ближайшая цель
/// не ближе минимальной дистанции навыка; если уже выбранная цель слишком
/// близка, следующая допустимая по дальности охраны запись заменяет её даже
/// при большей дистанции. Поэтому порядок игроков, затем питомцев и порядок
/// внутри индексов являются частью результата.
pub fn consider_fixed_archer_target(
    selected: Option<FixedArcherTarget>,
    candidate: FixedArcherTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<FixedArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    let Some(current) = selected else {
        return Some(candidate);
    };
    if current.distance <= candidate.distance {
        if current.distance < minimum_skill_distance {
            Some(candidate)
        } else {
            Some(current)
        }
    } else if candidate.distance < minimum_skill_distance {
        Some(current)
    } else {
        Some(candidate)
    }
}

/// Выполняет достигнутый `OnSearchEnemy` AI5, сохраняя отдельные проходы
/// игроков и питомцев по упорядоченным индексам региона.
pub fn select_fixed_archer_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id()) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    selected.map(|selected| selected.identity)
}
