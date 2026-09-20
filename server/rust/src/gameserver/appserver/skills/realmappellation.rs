//! Realm-appellation bonus owner позднего GameServer.
//!
//! Источник: `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/skills/realmappellation.cpp`. Достигнутый
//! `AddJingJieBuff` сначала снимает все шесть скрытых skills, очищает identity,
//! затем для title `1001..1054` проверяет соответствующий progression skill и
//! ставит hidden skill `1010..1060` уровня `1..4`. Ноль только снимает bonus;
//! повреждённый либо невыданный title также снимает прежний bonus, но возвращает
//! false. Полный virtual `CPlayer::UpdateProperty` остаётся runtime-входом:
//! owner меняет skills/identity до recompute, но не подменяет equipment,
//! progression, state и GlobeSetup части приблизительной delta-формулой.
//! Каждый DelSkill (0x004CF320) проходит общую границу CGame: разрешённый
//! current при !IsEnded получает End(0) до удаления экземпляра. Заимствование
//! игрока не удерживается через этот вызов; identity очищается только после
//! всех шести удалений, а проверка entitlement остаётся после этой очистки.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::skills::skillfactory::UNKNOWN_SKILL_ID;
use crate::gameserver::gameserver::game::CGame;

const BONUS_SKILLS: [u32; 6] = [1010, 1020, 1030, 1040, 1050, 1060];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RealmBonusIdentity {
    pub(crate) skill_id: u32,
    pub(crate) level: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RealmBonusMutation {
    pub(crate) appellation_id: u32,
    pub(crate) previous_identity: Option<RealmBonusIdentity>,
    pub(crate) current_identity: Option<RealmBonusIdentity>,
    pub(crate) previous_properties: PlayerCombatProperties,
    pub(crate) succeeded: bool,
}

pub(crate) const fn is_title(appellation_id: u32) -> bool {
    if appellation_id < 1001 || appellation_id > 1054 {
        return false;
    }
    let family = (appellation_id - 1001) / 10;
    let level = appellation_id % 10;
    family < BONUS_SKILLS.len() as u32 && level >= 1 && level <= 4
}

pub(crate) fn is_bonus_skill(skill_id: u32) -> bool {
    BONUS_SKILLS.contains(&skill_id)
}

pub(crate) fn is_internal_skill(skill_id: u32) -> bool {
    is_title(skill_id) || is_bonus_skill(skill_id)
}

fn resolve_bonus(appellation_id: u32) -> Option<RealmBonusIdentity> {
    is_title(appellation_id).then(|| RealmBonusIdentity {
        skill_id: BONUS_SKILLS[((appellation_id - 1001) / 10) as usize],
        level: (appellation_id % 10) as i32,
    })
}

pub(crate) fn set_bonus(
    game: &mut CGame,
    player_id: i32,
    appellation_id: u32,
) -> Option<RealmBonusMutation> {
    let player = game.find_player(player_id)?;
    let previous_identity = player.realm_appellation_bonus_identity();
    let previous_properties = player.combat_properties();
    let region_id = player.shape().get_region_id();
    let holder = player.shape().identity();

    for skill_id in BONUS_SKILLS {
        let _ = game.delete_move_shape_skill(region_id, holder, skill_id);
    }
    game.find_player_mut(player_id)?
        .set_realm_appellation_bonus_identity(UNKNOWN_SKILL_ID, 0);

    let mut current_identity = None;
    let succeeded = if appellation_id == 0 {
        true
    } else if let Some(identity) = resolve_bonus(appellation_id) {
        if game.find_player(player_id)?
            .realm_appellation_entitled(appellation_id, game.skill_factory())
            && game.add_move_shape_skill(region_id, holder, identity.skill_id, identity.level)
        {
            game.find_player_mut(player_id)?
                .set_realm_appellation_bonus_identity(identity.skill_id, identity.level);
            current_identity = Some(identity);
            true
        } else {
            false
        }
    } else {
        false
    };

    Some(RealmBonusMutation {
        appellation_id,
        previous_identity,
        current_identity,
        previous_properties,
        succeeded,
    })
}
