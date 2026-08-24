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

use crate::gameserver::appserver::player::{CPlayer, PlayerCombatProperties};
use crate::gameserver::appserver::skills::skillfactory::{CSkillFactory, UNKNOWN_SKILL_ID};

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
    player: &mut CPlayer,
    appellation_id: u32,
    factory: &CSkillFactory,
) -> RealmBonusMutation {
    let previous_identity = player.realm_appellation_bonus_identity();
    let previous_properties = player.combat_properties();

    for skill_id in BONUS_SKILLS {
        let _ = player.delete_realm_appellation_skill(skill_id, factory);
    }
    player.set_realm_appellation_bonus_identity(UNKNOWN_SKILL_ID, 0);

    let mut current_identity = None;
    let succeeded = if appellation_id == 0 {
        true
    } else if let Some(identity) = resolve_bonus(appellation_id) {
        if player.realm_appellation_entitled(appellation_id)
            && player.add_realm_appellation_skill(identity.skill_id, identity.level, factory)
        {
            player.set_realm_appellation_bonus_identity(identity.skill_id, identity.level);
            current_identity = Some(identity);
            true
        } else {
            false
        }
    } else {
        false
    };

    RealmBonusMutation {
        appellation_id,
        previous_identity,
        current_identity,
        previous_properties,
        succeeded,
    }
}
