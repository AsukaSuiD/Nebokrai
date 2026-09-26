//! Достигнутая часть городского охранника с луком (AI11).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/cityguardwithbow.cpp подтверждают общий городской поиск игроков
//! и питомцев по владельцу фракции и союза, преимущество игрока при равной
//! дистанции и повторный поиск после урона только вне боя. `OnIdle` ставит
//! строгую очередь `ChangeSkill → Stand → SearchEnemy`, а завершённая атака
//! ставит базовый ChangeSkill, затем SearchEnemy для живого владельца
//! (OnFighting 0x0060D930). `OnSchedule` не преследует цель:
//! только диапазон текущего навыка допускает атаку, а miss сбрасывает цель и
//! повторяет поиск. `OnSearch` завершён проходом вражеских повозок `603` с
//! теми же faction/union-фильтрами владельца города.
//! Диапазон OnSchedule 0x0060B890 теперь проверяется общим dispatcher-ом
//! стационарной семьи до Begin; проверка прямого пути принадлежит навыку.

use super::cityguardwithsword::select_city_guard_enemy;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

/// `WhenBeenHurted` AI11 не принимает атакующего напрямую: вне боя он заново
/// выполняет общий городской поиск игроков и питомцев текущим навыком.
pub(crate) fn retarget_city_bow_guard_after_hurt(
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
    let selected = select_city_guard_enemy(
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
