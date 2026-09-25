//! Блоки результата городской войны organizing-сообщений, вынесенные сюда
//! заранее: диспетчеры `organsysmessage` остаются в старом
//! `appworld/message/organsysmessage.rs` до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::organizations::country::CountryGovernanceContextBlock;
use crate::organizations::faction::{
    FactionBillboardStatBlock, FactionInitialPropertyBlock, OwnedCityMutationBuildError,
};
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingCityWarResultContextBlock {
    MissingRegionOwner { region_id: i32 },
    MissingFactionForMutation {
        faction_id: i32,
        operation: &'static str,
    },
    MissingFactionMaster { faction_id: i32 },
    MissingFactionCountry { faction_id: i32 },
    NullUnion { map_key: i32 },
    MissingEnemyOrganizing { organizing_id: i32 },
    EnemyMutation {
        organizing_id: i32,
        enemy_organizing_id: i32,
        source: FactionEnemyMutationBlock,
    },
    OwnedCity {
        faction_id: i32,
        operation: &'static str,
        source: OwnedCityMutationBuildError,
    },
    VictorCount {
        faction_id: i32,
        operation: &'static str,
        source: FactionInitialPropertyBlock,
    },
    Billboard(FactionBillboardStatBlock),
    NoticeWouldOverflow {
        string_id: &'static [u8],
        visible_len: usize,
    },
    MissingCountryOwner { country_id: u8 },
    CountryGovernance {
        country_id: u8,
        source: CountryGovernanceContextBlock,
    },
}
