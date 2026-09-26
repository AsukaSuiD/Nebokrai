//! ИИ неподвижного лучника `CFixedPositionArcher` (AI5) и наследующая его
//! стационарная семья: строгая очередь `OnIdle`, хвост `OnChangeSkill` с
//! ожиданием восстановления навыка и необычное правило `OnSearchEnemy` с
//! минимальной дистанцией текущего навыка.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp`.
//! Тела дочитаны машинно волной Z-AI (4 из 4 методов класса, кроме ctor
//! `0x0060F9D0`):
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `OnChangeSkill`: виртуальный `SelectAttackSkill` (vt `+0x8C`); навык не разрешился → `SetCurrentSkill(GetDefaultAttackSkillID())` (owner vt `+0xC0`/`+0xC4`) и возврат 1; не восстановленный навык (`IsRestored` vt `+0x80` == 0) ставит `AddAIEvent(Stand, GetRestoreTime(vt +0x84), 0)` с полным сроком | VA `0x0060F9F0` | [`queue_fixed_archer_skill_delay`], [`inherits_fixed_archer_change_skill`] (AI5 и наследующий AI23) | `MATCH` |
//! | `OnFighting`: базовый `CBaseAI::OnFighting` (`0x004C9320`, ChangeSkill-часть) и при его успехе `AddAIEvent(5)` | VA `0x0060FA70` | [`attack_completion_actions`] | `MATCH` (варианты `0x0060D930` живого владельца и `0x0060F6F0` StupidArcher — по прежней таблице владельца) |
//! | `OnIdle`: при живом владельце строгая очередь `ChangeSkill(6) → Stand(stop_frame, [monster+0x210]+0x94) → SearchEnemy(5)`; иначе девять областей `::_area` непустым plug-list проверяются до вызова `Hibernate` (vt `+0x64`) | VA `0x0060FAA0` | [`queue_stationary_guard_idle`], потребитель гейта — `ai/monsterai.rs::hibernates_without_nearby_players` | `MATCH` |
//! | `OnSearchEnemy`: живые игроки, затем питомцы внутри `GetGuardRange` (vt `+0x138`); ближайшая цель не ближе минимальной дистанции навыка (`vt +0x70`), при равной/меньшей дистанции selected слишком близкая заменяется следующей допустимой записью | VA `0x0060FBC0` | [`consider_fixed_archer_target`], [`select_fixed_archer_enemy`] | `MATCH` |
//!
//! Стационарное расписание `0x0020B890` и предикат по типам — ранее
//! зафиксированный факт `ai/monsterai.rs`; входной state-вопрос `0x0047B150`
//! — `RET1`-эквивалент. Собственный виртуальный конструктор
//! `CVilCouGuardWithBow` строит `CMonsterAI` напрямую: стационарный idle сам
//! по себе не наследует этот хвост `OnChangeSkill` — расширение ограничено
//! точным набором {5, 23}.
//!
//! Граница порции Z-AI (не расхождения): реальный путь `monsterbaseattack`
//! (выбор навыка и назначение цели) и общий monster tick hub остаются
//! hub-владением своих порций; реестр навыков `CMoveShape` и его
//! `GetRestoreTime` — hub-швы [`MonsterDispatcherMoveShape`].

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
