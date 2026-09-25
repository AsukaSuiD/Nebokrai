//! Блоки свойств, owned-city, билборда, member-info и enemy-проекций
//! фракций плюс общий time-помощник, вынесенные сюда заранее: сами
//! `CFactionCtrl`/`COrganizingCtrl` остаются в старом пакете до шага
//! переноса organizing-области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use std::error::Error;
use std::fmt;

use chrono::{Datelike, Local, Timelike};

use crate::app::world_message::SendMessageError;
use crate::content::organizing::TagTimeValue;

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
pub struct OwnedCityMutationReport {
    pub state_changed: bool,
    pub deliveries: Vec<FactionOwnedCityDelivery>,
    pub refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OwnedCityAddOutcome {
    AlreadyOwned,
    Added(OwnedCityMutationReport),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionSuperiorOrganizingBlock {
    MissingBaseProperty,
    DeleteRemainTimeAbsent,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionEnemyDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionMemberInfoRequest<'a> {
    pub recipient_player_id: i32,
    pub first_text: &'a [u8],
    pub second_text: &'a [u8],
    pub information_type: i32,
    pub color: u32,
    pub trailing_value: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionMemberInfoReport {
    pub recipient_player_ids: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionOwnedCityRefreshBlock {
    MissingBaseProperty,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionOwnedCityRefreshReport {
    pub refreshed_region_ids: Vec<i32>,
}

/// Текущее локальное время в полях члена фракции. Одинаковая формула для
/// faction/union проекций живёт здесь с владельцем.
pub fn current_local_member_time() -> TagTimeValue {
    let now = Local::now();
    TagTimeValue {
        year: now.year() as u16,
        month: now.month() as u16,
        day_of_week: now.weekday().num_days_from_sunday() as u16,
        day: now.day() as u16,
        hour: now.hour() as u16,
        minute: now.minute() as u16,
        second: now.second() as u16,
        milliseconds: now.timestamp_subsec_millis() as u16,
    }
}
