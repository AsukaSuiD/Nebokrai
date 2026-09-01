//! PK-policy исторического GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/pksys.cpp`. Достигнутый `OnFirstSkill`
//! вызывается обработчиком `skillmessage 0x90001` перед постановкой навыка с
//! целью-объектом в `CPlayerAI`, а `OnFirstAttack` — владельцем контактного навыка после
//! подтверждённой цели. Оба пути сохраняют точные проверки безопасности и
//! войн фракций, различающееся правило страны, переход преступного состояния
//! и аудит World `0x6020A`. Проход смерти также использует перенесённые `GetDiedLostExp`,
//! `GetDiedLostGoods` и `OnKill`: все три читают один текущий снимок настроек,
//! игрока и региона, а вызывающая сторона сохраняет эффекты контейнера и сети.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp

use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::player::CPlayer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FirstSkillPkFacts {
    pub(crate) victim_is_badman: bool,
    pub(crate) security: RegionSecurity,
    pub(crate) city_war_enemies: bool,
    pub(crate) faction_war_enemies: bool,
    pub(crate) gods_battle_region: bool,
    pub(crate) same_gods_battle_faction: bool,
    pub(crate) same_country: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FirstAttackPkFacts {
    pub(crate) victim_is_badman: bool,
    pub(crate) security: RegionSecurity,
    pub(crate) city_war_enemies: bool,
    pub(crate) faction_war_enemies: bool,
    pub(crate) gods_battle_region: bool,
    pub(crate) same_gods_battle_faction: bool,
    pub(crate) same_country: bool,
    pub(crate) attacker_country_identity: u8,
    pub(crate) attacker_kill_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FirstContactPkDisposition {
    VictimAlreadyBadman,
    ProtectedSecurity,
    CityWarEnemies,
    FactionWarEnemies,
    AllowedCombat,
    EnterCriminalState,
}

/// Stateless singleton semantics исходного `CPKSys::OnFirstSkill`.
pub(crate) struct CPKSys;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct DiedLostGoods {
    pub(crate) equipment: [f32; 17],
    pub(crate) hand: f32,
    pub(crate) packet: f32,
    pub(crate) money: f32,
    pub(crate) money_percent: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiedLostGoodsDisposition {
    UnsupportedSecurity,
    MissingPlayerAttacker,
    FactionWarProtected,
    Enabled { table: usize, lost_class: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KillPkDisposition {
    VictimBadman,
    ProtectedSecurity,
    CityWarEnemies,
    FactionWarEnemies,
    AllowedCombat,
    ReportMurderer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KillPkFacts {
    pub(crate) victim_is_badman: bool,
    pub(crate) security: RegionSecurity,
    pub(crate) city_war_enemies: bool,
    pub(crate) faction_war_enemies: bool,
    pub(crate) gods_battle_region: bool,
    pub(crate) same_gods_battle_faction: bool,
    pub(crate) same_country: bool,
    pub(crate) attacker_country_identity: u8,
    pub(crate) attacker_kill_count: u32,
}

impl CPKSys {
    pub(crate) fn is_city_war_state(
        first: Option<&CPlayer>,
        second: Option<&CPlayer>,
    ) -> bool {
        let (Some(first), Some(second)) = (first, second) else {
            return false;
        };
        let first_faction = first.faction_id();
        let second_faction = second.faction_id();
        first_faction > 0
            && second_faction > 0
            && (first.is_city_war_enemy_faction_member(second_faction)
                || second.is_city_war_enemy_faction_member(first_faction))
    }

    pub(crate) fn is_faction_war_state(
        first: Option<&CPlayer>,
        second: Option<&CPlayer>,
    ) -> bool {
        let (Some(first), Some(second)) = (first, second) else {
            return false;
        };
        let first_faction = first.faction_id();
        let second_faction = second.faction_id();
        first_faction > 0
            && second_faction > 0
            && (first.is_enemy_faction_member(second_faction)
                || second.is_enemy_faction_member(first_faction))
    }

    pub(crate) fn on_first_attack(facts: FirstAttackPkFacts) -> FirstContactPkDisposition {
        if facts.victim_is_badman {
            FirstContactPkDisposition::VictimAlreadyBadman
        } else if facts.security != RegionSecurity::FREE {
            FirstContactPkDisposition::ProtectedSecurity
        } else if facts.city_war_enemies {
            FirstContactPkDisposition::CityWarEnemies
        } else if facts.faction_war_enemies {
            FirstContactPkDisposition::FactionWarEnemies
        } else if facts.gods_battle_region && facts.same_gods_battle_faction {
            FirstContactPkDisposition::EnterCriminalState
        } else if !facts.gods_battle_region
            && facts.same_country
            && (facts.attacker_country_identity != 7 || facts.attacker_kill_count > 10)
        {
            FirstContactPkDisposition::EnterCriminalState
        } else {
            FirstContactPkDisposition::AllowedCombat
        }
    }

    pub(crate) fn on_first_skill(facts: FirstSkillPkFacts) -> FirstContactPkDisposition {
        if facts.victim_is_badman {
            FirstContactPkDisposition::VictimAlreadyBadman
        } else if facts.security != RegionSecurity::FREE {
            FirstContactPkDisposition::ProtectedSecurity
        } else if facts.city_war_enemies {
            FirstContactPkDisposition::CityWarEnemies
        } else if facts.faction_war_enemies {
            FirstContactPkDisposition::FactionWarEnemies
        } else if (facts.gods_battle_region && facts.same_gods_battle_faction)
            || (!facts.gods_battle_region && facts.same_country)
        {
            FirstContactPkDisposition::EnterCriminalState
        } else {
            FirstContactPkDisposition::AllowedCombat
        }
    }

    pub(crate) fn died_lost_experience(
        setup: &crate::setup::globesetup::GlobeSetupSnapshot,
        security: RegionSecurity,
    ) -> i32 {
        setup.died_lost_experience(security.value())
    }

    pub(crate) fn died_lost_goods(
        setup: &crate::setup::globesetup::GlobeSetupSnapshot,
        security: RegionSecurity,
        lost_class: usize,
        player_attacker_found: bool,
        faction_war_enemies: bool,
    ) -> Result<DiedLostGoods, DiedLostGoodsDisposition> {
        let table = if security == RegionSecurity::FREE {
            if !player_attacker_found {
                return Err(DiedLostGoodsDisposition::MissingPlayerAttacker);
            }
            if faction_war_enemies && lost_class < 2 {
                return Err(DiedLostGoodsDisposition::FactionWarProtected);
            }
            0
        } else if security == RegionSecurity::CITY_WAR {
            1
        } else {
            return Err(DiedLostGoodsDisposition::UnsupportedSecurity);
        };
        if lost_class >= 4 {
            return Err(DiedLostGoodsDisposition::UnsupportedSecurity);
        }
        let probability = |field| {
            setup
                .died_drop_probability(field, table, lost_class)
                .unwrap_or_default()
        };
        // Equipment enum order differs from tagDiedLost ABI order.
        let equipment = [
            probability(1),
            probability(2),
            probability(0),
            probability(3),
            probability(4),
            probability(6),
            probability(6),
            probability(6),
            probability(14),
            probability(5),
            probability(11),
            probability(12),
            probability(13),
            probability(15),
            probability(16),
            probability(17),
            0.0,
        ];
        Ok(DiedLostGoods {
            equipment,
            hand: probability(7),
            packet: probability(8),
            money: probability(9),
            money_percent: probability(10),
        })
    }

    pub(crate) fn on_kill(facts: KillPkFacts) -> KillPkDisposition {
        if facts.victim_is_badman {
            KillPkDisposition::VictimBadman
        } else if facts.security != RegionSecurity::FREE {
            KillPkDisposition::ProtectedSecurity
        } else if facts.city_war_enemies {
            KillPkDisposition::CityWarEnemies
        } else if facts.faction_war_enemies {
            KillPkDisposition::FactionWarEnemies
        } else if (facts.gods_battle_region && facts.same_gods_battle_faction)
            || (!facts.gods_battle_region
                && facts.same_country
                && (facts.attacker_country_identity != 7 || facts.attacker_kill_count > 10))
        {
            KillPkDisposition::ReportMurderer
        } else {
            KillPkDisposition::AllowedCombat
        }
    }
}

// ============================================================================
// FUNCTION: CPKSys::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:33
// RVA: 0x0001E560
// ADDRESS: 0041e560
// PROTOTYPE: CPKSys * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:42
// RVA: 0x0001E590
// ADDRESS: 0041e590
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::GetDiedLostExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:335
// RVA: 0x0001E5B0
// ADDRESS: 0041e5b0
// PROTOTYPE: long __thiscall GetDiedLostExp(CPlayer * param_1, CServerRegion * param_2, eSecurity param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::IsCityWarState
// STATUS: IMPLEMENTED, VERIFIED_RAW
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:454
// RVA: 0x0001E640
// ADDRESS: 0041e640
// PROTOTYPE: bool __thiscall IsCityWarState(CPlayer * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::ISFactionWarState
// STATUS: IMPLEMENTED, VERIFIED_RAW
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:471
// RVA: 0x0001E6A0
// ADDRESS: 0041e6a0
// PROTOTYPE: bool __thiscall ISFactionWarState(CPlayer * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::ReportMurderer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:496
// RVA: 0x0001E700
// ADDRESS: 0041e700
// PROTOTYPE: void __thiscall ReportMurderer(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::OnFirstSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:162
// RVA: 0x0001E9F0
// ADDRESS: 0041e9f0
// PROTOTYPE: void __thiscall OnFirstSkill(CPlayer * param_1, CPlayer * param_2, CServerRegion * param_3, eSecurity param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::OnKill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:250
// RVA: 0x0001EC00
// ADDRESS: 0041ec00
// PROTOTYPE: void __thiscall OnKill(CPlayer * param_1, CPlayer * param_2, CServerRegion * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPKSys::GetDiedLostGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp:366
// RVA: 0x0001ECE0
// ADDRESS: 0041ece0
// PROTOTYPE: bool __thiscall GetDiedLostGoods(CPlayer * param_1, long param_2, long param_3, CServerRegion * param_4, eSecurity param_5, long param_6, tagDiedLost * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagBillInfo::tagGoods::tagGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0002EC00
// ADDRESS: 0042ec00
// PROTOTYPE: undefined __thiscall tagGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::stLogNode::stLogNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0002EDD0
// ADDRESS: 0042edd0
// PROTOTYPE: undefined __thiscall stLogNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00432321
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00032321
// ADDRESS: 00432321
// PROTOTYPE: undefined Catch@00432321()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagBaseProperty::~tagBaseProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00032570
// ADDRESS: 00432570
// PROTOTYPE: void __thiscall ~tagBaseProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::tagCarriageInfo::~tagCarriageInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x000325E0
// ADDRESS: 004325e0
// PROTOTYPE: void __thiscall ~tagCarriageInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043775b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0003775B
// ADDRESS: 0043775b
// PROTOTYPE: undefined Catch@0043775b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00438c18
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00038C18
// ADDRESS: 00438c18
// PROTOTYPE: undefined Catch@00438c18()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::stComposeNode::~stComposeNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00038F20
// ADDRESS: 00438f20
// PROTOTYPE: void __thiscall ~stComposeNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00439834
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00039834
// ADDRESS: 00439834
// PROTOTYPE: undefined Catch@00439834()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00439c85
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00039C85
// ADDRESS: 00439c85
// PROTOTYPE: undefined Catch@00439c85()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00439ef2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00039EF2
// ADDRESS: 00439ef2
// PROTOTYPE: undefined Catch@00439ef2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00439fc9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00039FC9
// ADDRESS: 00439fc9
// PROTOTYPE: undefined Catch@00439fc9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043b263
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0003B263
// ADDRESS: 0043b263
// PROTOTYPE: undefined Catch@0043b263()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043b4c6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0003B4C6
// ADDRESS: 0043b4c6
// PROTOTYPE: undefined Catch@0043b4c6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440281
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00040281
// ADDRESS: 00440281
// PROTOTYPE: undefined Catch@00440281()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044095d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x0004095D
// ADDRESS: 0044095d
// PROTOTYPE: undefined Catch@0044095d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440a10
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\pksys.cpp
// RVA: 0x00040A10
// ADDRESS: 00440a10
// PROTOTYPE: undefined Catch@00440a10()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
