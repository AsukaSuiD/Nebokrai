//! Блоки свойств, owned-city и билборда фракций, вынесенные сюда заранее:
//! сами `CFactionCtrl`/`COrganizingCtrl` остаются в старом пакете до шага
//! переноса organizing-области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use std::error::Error;
use std::fmt;

use crate::app::world_message::SendMessageError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionInitialPropertyBlock;

#[derive(Debug, Eq, PartialEq)]
pub struct FactionOwnedCityDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnedCitiesWireBuildError {
    pub region_id: i32,
    pub byte_len: usize,
    pub completed_cities: usize,
}

impl fmt::Display for OwnedCitiesWireBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "имя региона {} длиной {} байт не помещается в старый char[256]",
            self.region_id, self.byte_len
        )
    }
}

impl Error for OwnedCitiesWireBuildError {}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionOwnedCityUpdateBuildError {
    Preflight(OwnedCitiesWireBuildError),
    Recipient {
        source: OwnedCitiesWireBuildError,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<FactionOwnedCityDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OwnedCityMutationBuildError {
    pub state_changed: bool,
    pub source: FactionOwnedCityUpdateBuildError,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionPropertyDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionPropertyReinitialization {
    pub level_parameters_found: bool,
    pub deliveries: Vec<FactionPropertyDelivery>,
}

/// Доставка faction-разговора `0x7FA02` одному получателю. Сам цикл по
/// членам фракции вместе с wire-форматом остаётся у старого `CFaction::talk`
/// до шага переноса organizing-области.
#[derive(Debug, Eq, PartialEq)]
pub struct FactionTalkDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionBillboardKind {
    MemberCount,
    OffenseVictories,
    DefenceVictories,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionBillboardStatBlock {
    MissingEstablishedTime {
        map_key: i32,
        billboard: FactionBillboardKind,
    },
    MissingBaseProperty {
        map_key: i32,
        billboard: FactionBillboardKind,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionReinitializationEntry {
    pub map_key: i32,
    pub result: FactionPropertyReinitialization,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionReinitializationBlock {
    pub map_key: i32,
    pub source: FactionInitialPropertyBlock,
    pub completed: Vec<FactionReinitializationEntry>,
}
