//! Достигнутая часть окружного охранника с луком (AI16).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/vilcouguardwithbow.cpp подтверждают общий со стражем с мечом
//! поиск игроков и питомцев, преимущество игрока при равной дистанции и новый
//! поиск после урона только вне боя. `OnIdle` ставит строгую очередь
//! `ChangeSkill → Stand → SearchEnemy`; общий окружной selector теперь также
//! исполняет сохранённый ниже проход вражеских повозок. Собственный
//! `OnSchedule` не преследует цель и не ждёт `attack_speed`: вне дальности или
//! при отказе `CheckCast` он отпускает цель и повторяет поиск.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\vilcouguardwithbow.cpp
// `OnSchedule` сопоставлен с RVA 0x0020B890.

// COMPONENT_VARIANT_END: GameServer

use super::vilcouguardwithsword::select_village_country_guard_enemy;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

/// Повторяет `WhenBeenHurted` AI16: базовое событие защиты ставится всегда,
/// но новый противник выбирается общим country-search только если охранник до
/// удара ещё не вёл бой и у него установлен текущий навык.
pub(crate) fn retarget_village_bow_guard_after_hurt(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    now_ms: u32,
) {
    let Some((owner, area_index, current_skill_id, was_fighting)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.shape_view(property)?,
                monster.move_shape().shape().area_index(),
                monster.move_shape().current_skill_id(),
                monster.ai_target().is_some(),
            ))
        })
    else {
        return;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.when_been_hurted(now_ms);
    }
    if was_fighting {
        return;
    }
    let Some((area_index, current_skill_id)) = area_index.zip(current_skill_id) else {
        return;
    };
    let Some(skill) = property
        .skills
        .iter()
        .filter(|skill| u32::from(skill.id) == current_skill_id)
        .max_by_key(|skill| skill.level)
    else {
        return;
    };
    let Some(skill_properties) =
        game.skill_base_properties(current_skill_id, i32::from(skill.level))
    else {
        return;
    };
    let selected = select_village_country_guard_enemy(
        game,
        region,
        owner,
        area_index,
        property.guard_range as i32,
        skill_properties.query_property(5_004) as i32,
    );
    if let (Some(selected), Some(monster)) =
        (selected, region.find_monster_by_id_mut(monster_id))
    {
        monster.set_ai_target(selected.identity);
    }
}
