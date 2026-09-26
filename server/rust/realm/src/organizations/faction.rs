//! Фракция `CFaction` из `faction.cpp/.h`, подтверждённая
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Владелец хранит ordered members/applications, properties, enemy и Goods-War
//! state, регионы, журналы, DB snapshots и wire publications. Membership и
//! governance сохраняют порядок lookup, permissions, mutations, callbacks и
//! broadcasts; ранний отказ не откатывает уже опубликованный префикс.
//!
//! Live state и DB projection разделены. `BTreeMap`, `VecDeque`, owned values
//! и traits заменяют MSVC containers, RTTI и raw pointers. Fixed C-string
//! границы останавливают только прежний выход за буфер; локальные quirks
//! описаны возле соответствующих операций.
//!
//! Владелец игры достижим через `&dyn WorldGameView`; контекст-трейты
//! membership/governance больше не получают game-параметр — реализация
//! держит собственную ссылку на игру
//! (прецедент моста `WorldRegionOwnerOrganizingView`, см. `union.rs`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

use chrono::{Datelike, Local, Timelike};

use crate::app::world_game_view::{WorldGameView, WorldRegionNameLookup};
use crate::app::world_message::{CMessage, SendMessageError, WorldLocalMessageQueueBlock};
use crate::characters::player::{PlayerOrganizingState, PlayerOrganizingUpdateError};
use crate::content::organizing::{
    EOperator, EPurview, EPurviewOwnState, MemberPurviewMutation, TagMemInfo, TagTimeValue,
    UnterminatedMemberField,
};
use crate::organizations::organizingparam::COrganizingParam;

pub use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;

const MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0D;
const FACTION_MEMBER_REMOVED_LOCAL_MESSAGE_TYPE: i32 = 0x60508;
const FACTION_UPGRADE_CHARGE_MESSAGE_TYPE: i32 = 0x7FE1E;
const APPLY_MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0A;
const MAX_APPLY_PERSON_COUNT: u32 = 40;
const PRONOUNCE_UPDATE_MESSAGE_TYPE: i32 = 0x7FE10;
const LEAVE_WORD_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0F;
const OTHER_FACTION_UPDATE_MESSAGE_TYPE: i32 = 0x7FE15;
const FACTION_TALK_MESSAGE_TYPE: i32 = 0x7FA02;
const FACTION_TALK_CHANNEL: i32 = 400;
const DELETE_ORGANIZING_MESSAGE_TYPE: i32 = 0x7FE03;
const OWNED_CITY_UPDATE_MESSAGE_TYPE: i32 = 0x7FE13;
const LEAVE_WORD_NAME_CAPACITY: usize = 20;
const LEAVE_WORD_CONTENT_CAPACITY: usize = 212;
const LEAVE_WORD_CONTENT_LIMIT: usize = 210;
const LEAVE_WORD_LIMIT: usize = 60;
const APPLY_PERSON_NAME_CAPACITY: usize = 20;
const FACTION_MEMBER_NAME_CAPACITY: usize = 32;
const FACTION_MEMBER_TEXT_CAPACITY: usize = 64;
const PRONOUNCE_NAME_CAPACITY: usize = 20;
const PRONOUNCE_CONTENT_CAPACITY: usize = 2048;
const PRONOUNCE_DATA_SIZE: usize = 0x828;
const FACTION_BASE_PROPERTY_SIZE: usize = 0x38;

const ZERO_TIME: TagTimeValue = TagTimeValue {
    year: 0,
    month: 0,
    day_of_week: 0,
    day: 0,
    hour: 0,
    minute: 0,
    second: 0,
    milliseconds: 0,
};

/// Полный byte- блок исходного `CFaction::tagFacBaseProperty`.
///
/// Три байта выравнивания после `btCountry` сохраняются вместе с полями:
/// `CloneSaveData` копировал структуру четырнадцатью 32-битными словами.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FactionBaseProperty {
    bytes: [u8; FACTION_BASE_PROPERTY_SIZE],
}

impl FactionBaseProperty {
    pub const fn from_complete_bytes(bytes: [u8; FACTION_BASE_PROPERTY_SIZE]) -> Self {
        Self { bytes }
    }

    const fn signed_at(&self, offset: usize) -> i32 {
        i32::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
            self.bytes[offset + 2],
            self.bytes[offset + 3],
        ])
    }

    pub const fn level(&self) -> i32 {
        self.signed_at(0x00)
    }

    pub const fn experience(&self) -> i32 {
        self.signed_at(0x04)
    }

    pub const fn upgrade_experience(&self) -> i32 {
        self.signed_at(0x20)
    }

    pub const fn offense_victor_counts(&self) -> i32 {
        self.signed_at(0x08)
    }

    pub const fn defence_victor_counts(&self) -> i32 {
        self.signed_at(0x0C)
    }

    pub const fn village_war_victor_counts(&self) -> i32 {
        self.signed_at(0x10)
    }

    pub const fn member_count(&self) -> i32 {
        self.signed_at(0x14)
    }

    pub const fn union_id(&self) -> i32 {
        self.signed_at(0x18)
    }

    pub const fn permit(&self) -> bool {
        self.bytes[0x2B] != 0
    }

    pub const fn country(&self) -> u8 {
        self.bytes[0x2C]
    }

    pub const fn property_1(&self) -> i32 {
        self.signed_at(0x30)
    }

    pub const fn property_2(&self) -> i32 {
        self.signed_at(0x34)
    }

    pub const fn leave_word_function(&self) -> bool {
        self.bytes[0x25] != 0
    }

    pub const fn pronounce_function(&self) -> bool {
        self.bytes[0x24] != 0
    }

    pub const fn create_union_function(&self) -> bool {
        self.bytes[0x2A] != 0
    }

    pub const fn upload_icon_function(&self) -> bool {
        self.bytes[0x27] != 0
    }

    const fn wire_bytes(&self) -> &[u8; FACTION_BASE_PROPERTY_SIZE] {
        &self.bytes
    }

    fn write_signed(&mut self, offset: usize, value: i32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_country(&mut self, country: u8) {
        self.bytes[0x2C] = country;
    }

    fn set_permit(&mut self, permit: bool) {
        self.bytes[0x2B] = u8::from(permit);
    }

    fn feature_function(&self, feature: FactionFeatureFunction) -> bool {
        self.bytes[feature.property_offset()] != 0
    }

    fn set_feature_function(&mut self, feature: FactionFeatureFunction, enabled: bool) {
        self.bytes[feature.property_offset()] = u8::from(enabled);
    }

    fn set_initial_level_permissions(&mut self, parameters: &COrganizingParam) {
        let level = self.level();
        self.bytes[0x24] = u8::from(parameters.pronounce_minimum_level() <= level);
        self.bytes[0x25] = u8::from(parameters.leave_word_minimum_level() <= level);
        self.bytes[0x26] = u8::from(parameters.endue_right_minimum_level() <= level);
        self.bytes[0x2A] = u8::from(parameters.create_union_minimum_level() <= level);
        self.bytes[0x28] = u8::from(parameters.attack_village_minimum_level() <= level);
        self.bytes[0x29] = u8::from(parameters.attack_city_minimum_level() <= level);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionCloneSaveBlock {
    MasterIdMissing,
    MissingBaseProperty,
    EstablishedTimeUnknown,
    DeleteRemainTimeAbsent,
}

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

/// Доставка faction-разговора `0x7FA02` одному получателю. Цикл по членам
/// фракции вместе с wire-форматом принадлежит `CFaction::talk` ниже.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionInitialStringField {
    MasterName,
    MasterTitle,
    MasterRegion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionInitialBlock {
    pub field: FactionInitialStringField,
    pub visible_len: usize,
    pub capacity: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDelMemberBlock {
    MasterIdMissing,
    DeleteRemainTimeAbsent,
    MissingBaseProperty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionDelMemberReport {
    pub removed: bool,
    pub disband_countdown_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionOperatorValidationBlock;

#[derive(Debug, Eq, PartialEq)]
pub struct FactionEnemyRefreshReport {
    pub deliveries: Vec<FactionEnemyDelivery>,
    pub refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarEnemyRefreshOutcome {
    ChangeFlagUnknown,
    Unchanged,
    Published(FactionEnemyRefreshReport),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionPronounceDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionLeaveWordDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionApplyMemberDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionApplyMemberUpdateBuildError {
    pub candidate_player_id: i32,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub completed_deliveries: Vec<FactionApplyMemberDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionRemoveApplyMemberOutcome {
    NotFound,
    Removed {
        faction_id: i32,
        deliveries: Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionApplyForJoinRejection {
    StandardOrVillageWar,
    CityWar,
    MemberLimit,
    ApplyListLimit,
    PlayerAlreadyInFaction,
    PlayerOffline,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionApplyForJoinOutcome {
    Rejected(FactionApplyForJoinRejection),
    Applied {
        deliveries: Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>,
        log_written: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionApplyForJoinContextOperation {
    PlayerMembershipLookup,
    RemovePreviousApplications,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionApplyForJoinBlock<ContextBlock> {
    MissingBaseProperty,
    Context {
        operation: FactionApplyForJoinContextOperation,
        source: ContextBlock,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        visible_len: usize,
    },
    SuccessNoticeWouldOverflow {
        player_id: i32,
        formatted_len: usize,
        application_inserted: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDoJoinRejection {
    StandardOrVillageWar,
    CityWar,
    GoodsWar,
    PermissionDenied,
    ApplicationNotFound,
    MemberLimit,
    ApplicantAlreadyInFaction,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDoJoinOutcome {
    Rejected {
        reason: FactionDoJoinRejection,
        application_removal: Option<FactionRemoveApplyMemberOutcome>,
    },
    ApplicationDenied {
        application_removal: FactionRemoveApplyMemberOutcome,
    },
    Joined {
        application_removal: FactionRemoveApplyMemberOutcome,
        member_information: FactionMemberInfoReport,
        refreshed_player_ids: Vec<i32>,
        add_faction_to_client_result: bool,
        add_all_faction_info_result: bool,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        disband_countdown_cancelled: bool,
        log_written: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDoJoinContextOperation {
    RemovePreviousApplications,
    ApplicantMembershipLookup,
    AddFactionToClient,
    AddAllFactionInfoToClient,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDoJoinStringField {
    DenialNotice,
    MemberLimitNotice,
    MemberTitle,
    OnlinePlayerName,
    OnlineRegionName,
    MemberJoinedNotice,
    ApplicantJoinedNotice,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDoJoinBlock<ContextBlock> {
    Context {
        operation: FactionDoJoinContextOperation,
        source: ContextBlock,
        application_removed: bool,
        member_inserted: bool,
    },
    MissingBaseProperty {
        application_removed: bool,
    },
    StringWouldOverflow {
        field: FactionDoJoinStringField,
        visible_len: usize,
        capacity: usize,
        application_removed: bool,
        member_inserted: bool,
    },
    UnterminatedManagerName {
        manager_id: i32,
        member_inserted: bool,
    },
    ManagerMemberMissing {
        manager_id: i32,
        member_inserted: bool,
    },
    MissingDeleteRemainTime {
        member_inserted: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionExitRejection {
    StandardOrVillageWar,
    CityWar,
    PermissionDenied,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionExitOutcome {
    Rejected {
        reason: FactionExitRejection,
        goods_war_notice_sent: bool,
    },
    Exited {
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        delete_organizing: FactionDeleteOrganizingOutcome,
        refreshed_player_ids: Vec<i32>,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionExitBlock {
    UnterminatedMemberName {
        player_id: i32,
        goods_war_notice_sent: bool,
    },
    NoticeWouldOverflow {
        player_id: i32,
        formatted_len: usize,
        goods_war_notice_sent: bool,
    },
    DelMember {
        source: FactionDelMemberBlock,
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removed: bool,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        refreshed_player_ids: Vec<i32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionFireOutRejection {
    StandardOrVillageWar,
    CityWar,
    GoodsWar,
    OperatorValidationFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionFireOutOutcome {
    Rejected(FactionFireOutRejection),
    Fired {
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        delete_organizing: FactionDeleteOrganizingOutcome,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        refreshed_player_ids: Vec<i32>,
        log_written: bool,
        local_message: Result<(), WorldLocalMessageQueueBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionFireOutBlock {
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedTargetName {
        target_id: i32,
    },
    NoticeWouldOverflow {
        target_id: i32,
        formatted_len: usize,
    },
    DelMember {
        source: FactionDelMemberBlock,
        member_information: FactionMemberInfoReport,
        member_removed: bool,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
    },
    UnterminatedManagerName {
        manager_id: i32,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        refreshed_player_ids: Vec<i32>,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        delete_organizing: FactionDeleteOrganizingOutcome,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDubRejection {
    InvalidTitle,
    OperatorValidationFailed,
    InvalidJobLevel,
    TargetNotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDubNotice {
    Title,
    JobLevel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDubFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionDubProgress {
    pub input_title_truncated: bool,
    pub title_changed: bool,
    pub job_level_changed: bool,
    pub title_information: Option<FactionMemberInfoReport>,
    pub title_refreshed_player_ids: Vec<i32>,
    pub job_level_information: Option<FactionMemberInfoReport>,
    pub member_update: Option<Result<MemberUpdateReport, MemberUpdateBuildError>>,
    pub dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDubOutcome {
    Rejected {
        reason: FactionDubRejection,
        input_title_truncated: bool,
    },
    Updated {
        progress: FactionDubProgress,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDubBlock {
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedMemberField {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionDubProgress,
    },
    OldTitleWouldOverflow {
        target_id: i32,
        visible_len: usize,
        progress: FactionDubProgress,
    },
    NoticeWouldOverflow {
        notice: FactionDubNotice,
        formatted_len: usize,
        progress: FactionDubProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionPurviewChange {
    Grant,
    Revoke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionPurviewChangeRejection {
    FunctionDisabled,
    OperatorValidationFailed,
    InvalidPurview,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionPurviewChangeProgress {
    pub mutation: MemberPurviewMutation,
    pub member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
    pub dirty_set: bool,
    pub apply_snapshot:
        Option<Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>>,
    pub member_information: Option<FactionMemberInfoReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionPurviewChangeOutcome {
    Rejected(FactionPurviewChangeRejection),
    Changed {
        progress: FactionPurviewChangeProgress,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionPurviewChangeBlock {
    MissingBaseProperty,
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionPurviewChangeProgress,
    },
    NoticeWouldOverflow {
        formatted_len: usize,
        progress: FactionPurviewChangeProgress,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionMaximumMembersUpdate {
    Unchanged,
    Updated(FactionMemberInfoReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionMaximumMembersBlock {
    MissingBaseProperty,
    NoticeWouldOverflow {
        maximum_members: i32,
        formatted_len: usize,
        property_changed: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionLevelUpdate {
    Unchanged,
    Updated {
        pronounce: FactionFeatureFunctionUpdate,
        leave_word: FactionFeatureFunctionUpdate,
        endue_right: FactionFeatureFunctionUpdate,
        create_union: FactionFeatureFunctionUpdate,
        join_village_war: FactionFeatureFunctionUpdate,
        join_city_war: FactionFeatureFunctionUpdate,
        maximum_members: FactionMaximumMembersUpdate,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionLevelBlock {
    MissingBaseProperty {
        level_changed: bool,
    },
    MaximumMembers {
        source: FactionMaximumMembersBlock,
        level_changed: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionSetParameterKind {
    Level,
    Experience,
    Unknown,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionSetParameterProgress {
    pub parameter: FactionSetParameterKind,
    pub level_update: Option<FactionLevelUpdate>,
    pub experience_update: Option<FactionExperienceUpdate>,
    pub assigned_experience: Option<i32>,
    pub assigned_upgrade_experience: Option<i32>,
    pub property_deliveries: Option<Vec<FactionPropertyDelivery>>,
    pub refreshed_player_ids: Option<Vec<i32>>,
    pub dirty_bit_requested: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionSetParameterOutcome {
    LevelAboveMaximum {
        requested_level: i32,
    },
    LevelParametersMissing {
        level: i32,
        progress: FactionSetParameterProgress,
    },
    Applied(FactionSetParameterProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionSetParameterBlock {
    MissingBaseProperty {
        progress: FactionSetParameterProgress,
    },
    Level {
        source: FactionLevelBlock,
        progress: FactionSetParameterProgress,
    },
    Experience {
        source: FactionExperienceBlock,
        progress: FactionSetParameterProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUpgradeRejection {
    PlayerNotMaster,
    MaximumLevel,
    CurrentLevelParametersMissing,
    InsufficientExperience,
    PlayerOffline,
    InsufficientMoney,
    MasterLevelTooLow,
    RequiredGoodsMissing,
    NextLevelParametersMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUpgradeNotice {
    Experience,
    Money,
    MasterLevel,
    Goods,
    Success,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUpgradeFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionUpgradeChargeDelivery {
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionUpgradeProgress {
    pub level_update: Option<FactionLevelUpdate>,
    pub experience_update: Option<FactionExperienceUpdate>,
    pub charge_delivery: Option<FactionUpgradeChargeDelivery>,
    pub next_upgrade_experience: Option<i32>,
    pub member_information: Option<FactionMemberInfoReport>,
    pub property_deliveries: Option<Vec<FactionPropertyDelivery>>,
    pub dirty_set: bool,
    pub refreshed_player_ids: Vec<i32>,
    pub log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionUpgradeOutcome {
    Rejected {
        reason: FactionUpgradeRejection,
        notice_sent: bool,
        progress: FactionUpgradeProgress,
    },
    Upgraded(FactionUpgradeProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionUpgradeBlock {
    MissingBaseProperty {
        progress: FactionUpgradeProgress,
    },
    MissingPlayerMoney {
        player_id: i32,
        progress: FactionUpgradeProgress,
    },
    NoticeWouldOverflow {
        notice: FactionUpgradeNotice,
        formatted_len: usize,
        progress: FactionUpgradeProgress,
    },
    Level {
        source: FactionLevelBlock,
        progress: FactionUpgradeProgress,
    },
    Experience {
        source: FactionExperienceBlock,
        progress: FactionUpgradeProgress,
    },
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionUpgradeProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionContributorRejection {
    RequesterNotMaster,
    MaximumContributors,
    TargetNotMember,
    Unchanged,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionContributorProgress {
    pub contributor_changed: bool,
    pub member_update: Option<MemberUpdateReport>,
    pub refreshed_player_ids: Vec<i32>,
    pub member_information: Option<FactionMemberInfoReport>,
    pub dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionContributorOutcome {
    Rejected(FactionContributorRejection),
    Updated(FactionContributorProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionContributorBlock {
    MemberUpdate {
        source: MemberUpdateBuildError,
        progress: FactionContributorProgress,
    },
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionContributorProgress,
    },
    NoticeWouldOverflow {
        formatted_len: usize,
        progress: FactionContributorProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUploadIconRejection {
    PlayerNotMaster,
    FunctionDisabled,
    IntervalActive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUploadIconOutcome {
    Rejected {
        reason: FactionUploadIconRejection,
        notice_sent: bool,
    },
    Accepted {
        dirty_set: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionUploadIconBlock {
    MissingBaseProperty,
    NoticeWouldOverflow {
        formatted_len: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDisbandRejection {
    OperatorNotPermitted,
    HasSuperiorOrganizing,
    StandardWar,
    CityWar,
    GoodsWar,
    CountryKing,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionDisbandProgress {
    pub goods_war_members_deleted: bool,
    pub goods_war_faction_count_decremented: bool,
    pub cleared_apply_persons: usize,
    pub cleared_leave_words: usize,
    pub delete_organizing: Option<FactionDeleteOrganizingOutcome>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDisbandOutcome {
    Rejected {
        reason: FactionDisbandRejection,
        notice_sent: bool,
    },
    Disbanded(FactionDisbandProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDisbandBlock {
    MissingBaseProperty,
    CountryMissing {
        country: u8,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        progress: FactionDisbandProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDemiseRejection {
    SamePlayer,
    OldPlayerNotMaster,
    NewPlayerNotMember,
    StandardWar,
    CityWar,
    GoodsWar,
    OldPlayerOffline,
    NewPlayerOffline,
    NewPlayerOperatingFactionWar,
    OldPlayerOperatingFactionWar,
    NewMasterLevelTooLow,
    DemiseForbidden,
    CountryKingBlocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDemiseMember {
    OldMaster,
    NewMaster,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionDemiseNotice {
    RequiredLevel,
    DemiseForbidden,
    CountryKingBlocked,
    Success,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionDemiseProgress {
    pub permit_demise_disabled: bool,
    pub master_changed: bool,
    pub new_member_changed: bool,
    pub old_member_changed: bool,
    pub old_member_update: Option<MemberUpdateReport>,
    pub new_member_update: Option<MemberUpdateReport>,
    pub base_dirty_set: bool,
    pub members_dirty_set: bool,
    pub refreshed_player_ids: Vec<i32>,
    pub member_information: Option<FactionMemberInfoReport>,
    pub log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDemiseOutcome {
    Rejected {
        reason: FactionDemiseRejection,
        notice_sent: bool,
    },
    Transferred(FactionDemiseProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDemiseBlock {
    MissingBaseProperty,
    PermitDemiseUnknown,
    OldMasterMemberMissing {
        player_id: i32,
    },
    TitleWouldOverflow {
        member: FactionDemiseMember,
        string_id: &'static [u8],
        visible_len: usize,
        progress: FactionDemiseProgress,
    },
    MemberUpdate {
        member: FactionDemiseMember,
        source: MemberUpdateBuildError,
        progress: FactionDemiseProgress,
    },
    UnterminatedMemberName {
        member: FactionDemiseMember,
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionDemiseProgress,
    },
    NoticeWouldOverflow {
        notice: FactionDemiseNotice,
        formatted_len: usize,
        progress: FactionDemiseProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionOperationRejection {
    UnionMasterMismatch {
        union_id: i32,
        master_faction_id: i32,
    },
    RegionNotOwned {
        region_id: i32,
    },
    PlayerNotPermitted {
        player_id: i32,
        purview: EPurview,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionOperationOutcome {
    Rejected(FactionOperationRejection),
    Authorized,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionOperationBlock<ContextBlock> {
    MissingBaseProperty,
    UnionMembershipLookup(ContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionLeaveWordUpdateBuildError {
    MissingLastLeaveWord,
    Recipient {
        source: UnterminatedLeaveWordField,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<FactionLeaveWordDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionEditLeaveWordOutcome {
    FunctionDisabled,
    PermissionDenied,
    LeaveWordNotFound,
    Deleted {
        deliveries: Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionLeaveWordBlock {
    MissingBaseProperty,
    OfflineAuthorNameUnknown {
        player_id: i32,
        allocated_leave_word_id: i32,
        input_truncated: bool,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        allocated_leave_word_id: i32,
        visible_len: usize,
        input_truncated: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionLeaveWordOutcome {
    FunctionDisabled,
    PermissionDenied,
    Published {
        leave_word_id: i32,
        input_truncated: bool,
        evicted_count: usize,
        deliveries: Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionLoadLeaveWordReport {
    pub evicted_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionPronounceUpdateBuildError {
    pub source: UnterminatedPronounceField,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub completed_deliveries: Vec<FactionPronounceDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionPronounceBlock {
    MissingBaseProperty,
    ContentWouldOverflow {
        visible_len: usize,
        input_truncated: bool,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        visible_len: usize,
        input_truncated: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionPronounceOutcome {
    FunctionDisabled,
    PermissionDenied,
    Published {
        input_truncated: bool,
        deliveries: Result<Vec<FactionPronounceDelivery>, FactionPronounceUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FactionFeatureFunction {
    LeaveWord,
    Pronounce,
    EndueRight,
    JoinVillageWar,
    JoinCityWar,
    CreateUnion,
}

impl FactionFeatureFunction {
    const fn property_offset(self) -> usize {
        match self {
            Self::Pronounce => 0x24,
            Self::LeaveWord => 0x25,
            Self::EndueRight => 0x26,
            Self::JoinVillageWar => 0x28,
            Self::JoinCityWar => 0x29,
            Self::CreateUnion => 0x2A,
        }
    }

    const fn notification_string_id(self, enabled: bool) -> &'static [u8] {
        match (self, enabled) {
            (Self::LeaveWord, true) => b"WS0172",
            (Self::LeaveWord, false) => b"WS0173",
            (Self::Pronounce, true) => b"WS0174",
            (Self::Pronounce, false) => b"WS0175",
            (Self::EndueRight, true) => b"WS0176",
            (Self::EndueRight, false) => b"WS0177",
            (Self::JoinVillageWar, true) => b"WS0180",
            (Self::JoinVillageWar, false) => b"WS0181",
            (Self::JoinCityWar, true) => b"WS0182",
            (Self::JoinCityWar, false) => b"WS0183",
            (Self::CreateUnion, true) => b"WS0184",
            (Self::CreateUnion, false) => b"WS0185",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionFeatureFunctionUpdate {
    Unchanged,
    Updated(FactionMemberInfoReport),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionOtherInfoDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionOtherInfoBuildError {
    pub visible_name_len: usize,
}

pub trait FactionOrganizingInfoContext {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>>;

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
}

pub trait FactionApplyForJoinEffects: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn faction_apply_log_enabled(&self) -> bool;

    fn write_faction_apply_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    );
}

pub trait FactionApplyForJoinContext: FactionApplyForJoinEffects {
    type Block;

    fn player_already_in_faction(
        &self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block>;

    fn remove_previous_faction_applications(
        &mut self,
        current_faction: &mut CFaction,
        player_id: i32,
    ) -> Result<(), Self::Block>;
}

pub trait FactionDoJoinEffects: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_join(&self, faction_id: i32, manager_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_join_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_join_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );
}

pub trait FactionDoJoinContext: FactionDoJoinEffects {
    type Block;

    fn remove_previous_faction_applications(
        &mut self,
        current_faction: &mut CFaction,
        player_id: i32,
    ) -> Result<(), Self::Block>;

    fn applicant_already_in_faction(
        &self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block>;

    fn add_faction_to_client_by_player_id(
        &mut self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block>;

    fn add_all_faction_info_to_client_by_player_id(
        &mut self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block>;

}

pub trait FactionExitContext: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_exit(&self, faction_id: i32, player_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_quit_log_enabled(&self) -> bool;

    fn write_faction_quit_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    );

    fn delete_goods_war_member(&mut self, player_id: i32);
}

pub trait FactionFireOutContext: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_fire_out(&self, faction_id: i32, manager_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_fire_out_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_fire_out_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );

    fn delete_goods_war_member(&mut self, player_id: i32);
}

pub trait FactionDubContext: FactionOrganizingInfoContext {
    fn check_invalid_string(&mut self, value: &mut Vec<u8>, mode: bool) -> bool;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionDubFormatArgument<'_>],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_title_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_title_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        old_title: &[u8],
        new_title: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    );
}

pub trait FactionPurviewChangeContext: FactionOrganizingInfoContext {
    fn format_world_string(&mut self, string_id: &'static [u8], member_name: &[u8]) -> Vec<u8>;

    fn faction_purview_log_enabled(&self, change: FactionPurviewChange) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_purview_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        purview: i32,
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );
}

pub trait FactionLevelContext: FactionOrganizingInfoContext {
    fn format_world_string_signed(&mut self, string_id: &'static [u8], value: i32) -> Vec<u8>;
}

pub trait FactionSetParameterContext: FactionLevelContext {
    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);
}

pub trait FactionUpgradeContext: FactionLevelContext {
    fn player_money(&self, player_id: i32) -> Option<u32>;

    fn goods_in_packet(&self, player_id: i32, original_name: &[u8]) -> i32;

    fn goods_display_name(&self, original_name: &[u8]) -> Option<Vec<u8>>;

    fn format_upgrade_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionUpgradeFormatArgument<'_>],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_level_log_enabled(&self) -> bool;

    fn write_faction_level_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        level: i32,
        master_id: i32,
        master_name: &[u8],
    );
}

pub trait FactionContributorContext: FactionOrganizingInfoContext {
    fn format_contributor_string(
        &mut self,
        string_id: &'static [u8],
        member_name: &[u8],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);
}

pub trait FactionUploadIconContext: FactionOrganizingInfoContext {
    fn format_upload_icon_interval(
        &mut self,
        string_id: &'static [u8],
        interval_minutes: i32,
    ) -> Vec<u8>;
}

pub trait FactionDisbandContext: FactionOrganizingInfoContext {
    fn village_war_declared(&self, faction_id: i32) -> bool;

    fn city_war_declared(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_disband(&self, faction_id: i32, player_id: i32) -> bool;

    fn country_king_id(&self, country: u8) -> Option<i32>;

    fn delete_goods_war_members_by_faction_id(&mut self, faction_id: i32);

    fn decrement_goods_war_faction_count(&mut self, faction_id: i32, faction_name: &[u8]);
}

pub trait FactionDemiseContext: FactionOrganizingInfoContext {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_demise(&self, faction_id: i32, old_master_id: i32) -> bool;

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool;

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8>;

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32);

    fn faction_master_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_master_log(
        &mut self,
        old_master_id: i32,
        old_master_name: &[u8],
        new_master_id: i32,
        new_master_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    );
}

pub trait FactionOperationAuthorityContext {
    type Block;

    fn union_id_for_faction(&self, faction_id: i32) -> Result<i32, Self::Block>;

    fn union_master_faction_id(&self, union_id: i32) -> Option<i32>;
}

pub trait FactionPlayerHeaderContext {
    type Block;

 /// `None` означает miss/null самого union; `Some(0)` — найденный союз,
 /// который штатно не смог разрешить свою master-faction.
    fn union_player_header(&self, union_id: i32) -> Result<Option<i32>, Self::Block>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionPlayerHeaderBlock<ContextBlock> {
    MissingBaseProperty,
    MissingMasterId,
    Context(ContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionDeleteOrganizingDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionDeleteOrganizingOutcome {
    SingleTarget {
        delivery: Option<FactionDeleteOrganizingDelivery>,
    },
    Broadcast {
        deliveries: Vec<FactionDeleteOrganizingDelivery>,
        information_recipient_ids: Vec<i32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionDeleteOrganizingBuildError {
    pub localized_info_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionEnemyWarLogArgument<'a> {
    Text(&'a [u8]),
    Unsigned(u32),
}

pub trait FactionEnemyMutationContext {
    fn organizing_name(&self, organizing_id: i32) -> Option<Vec<u8>>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionEnemyWarLogArgument<'_>],
    ) -> Vec<u8>;

    fn put_war_log(&mut self, text: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionEnemyMutationReport {
    pub state_changed: bool,
    pub changed_flag_set: bool,
    pub war_log_written: bool,
}


#[derive(Debug, Eq, PartialEq)]
pub struct OwnedCityBooleanMutationReport {
    pub legacy_result: bool,
    pub mutation: OwnedCityMutationReport,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionExperienceDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionExperienceBlock {
    MissingBaseProperty,
    MasterIdMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionExperienceUpdate {
    MaximumLevel,
    Unchanged { experience: i32 },
    Updated {
        experience: i32,
        deliveries: Vec<FactionExperienceDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionPermitBlock {
    MasterIdMissing,
    MissingBaseProperty,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionPermitUpdate {
    RequesterIsNotMaster,
    SuperiorMasterMismatch,
    Unchanged,
    Updated {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy)]
enum EnemyFactionSetKind {
    Standard,
    CityWar,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MemberUpdateDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MemberUpdateReport {
    pub target_found: Option<bool>,
    pub deliveries: Vec<MemberUpdateDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MemberUpdateBuildError {
    pub field: UnterminatedMemberField,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub completed_deliveries: Vec<MemberUpdateDelivery>,
}

impl fmt::Display for MemberUpdateBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "member-update для игрока {} не построен: {}",
            self.recipient_player_id, self.field
        )
    }
}

impl Error for MemberUpdateBuildError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemberEnterBlockedReason {
    NullRegionPointer { region_id: i32 },
    UnterminatedMemberRegion(UnterminatedMemberField),
    RegionNameExceedsMemberField { region_id: i32, byte_len: usize },
}

impl fmt::Display for MemberEnterBlockedReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullRegionPointer { region_id } => write!(
                formatter,
                "у tagRegion {} отсутствует исходный pRegion",
                region_id
            ),
            Self::UnterminatedMemberRegion(field) => field.fmt(formatter),
            Self::RegionNameExceedsMemberField {
                region_id,
                byte_len,
            } => write!(
                formatter,
                "имя региона {} длиной {} байт не помещается в старый strRegion[64]",
                region_id, byte_len
            ),
        }
    }
}

impl Error for MemberEnterBlockedReason {}

#[derive(Debug, Eq, PartialEq)]
pub enum MemberEnterOutcome {
    PlayerNotOnline,
    MemberNotFound,
    RegionUnchanged,
    Blocked(MemberEnterBlockedReason),
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum MemberExitOutcome {
    MemberNotFound,
    RegionAlreadyEmpty,
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum MemberLevelChangeOutcome {
    MemberNotFound,
    Unchanged,
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemberPositionChangeBlocked {
    pub region_id: i32,
    pub byte_len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MemberPositionChangeOutcome {
    MemberNotFound,
    RegionNotFound,
    NullRegionPointer,
    Blocked(MemberPositionChangeBlocked),
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnterminatedApplyPersonName {
    pub player_id: i32,
    pub completed_persons: usize,
}

impl fmt::Display for UnterminatedApplyPersonName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "у кандидата {} отсутствует NUL в tagApplyPerson::strName",
            self.player_id
        )
    }
}

impl Error for UnterminatedApplyPersonName {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionFullSnapshotBlock {
    MasterIdMissing,
    MasterTitle(UnterminatedMemberField),
    EstablishedTimeUnknown,
    MissingBaseProperty,
    Pronounce(UnterminatedPronounceField),
    LeaveWords(UnterminatedLeaveWordField),
    Members(UnterminatedMemberField),
    ApplyPersons(UnterminatedApplyPersonName),
    OwnedCities(OwnedCitiesWireBuildError),
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TagApplyPerson {
    pub id: i32,
    pub name: [u8; APPLY_PERSON_NAME_CAPACITY],
    pub occupation: i32,
    pub level: i32,
}

impl TagApplyPerson {
    pub const fn from_complete_fields(
        id: i32,
        name: [u8; APPLY_PERSON_NAME_CAPACITY],
        occupation: i32,
        level: i32,
    ) -> Self {
        Self {
            id,
            name,
            occupation,
            level,
        }
    }

    const fn empty_with_id(id: i32) -> Self {
        Self {
            id,
            name: [0; APPLY_PERSON_NAME_CAPACITY],
            occupation: 0,
            level: 0,
        }
    }

    fn name_wire_bytes(&self) -> Option<&[u8]> {
        let terminator = self.name.iter().position(|byte| *byte == 0)?;
        Some(&self.name[..=terminator])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnterminatedPronounceField {
    pub field: &'static str,
}

impl fmt::Display for UnterminatedPronounceField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedPronounceField {}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TagPronounceWord {
    pub player_id: i32,
    pub name: [u8; PRONOUNCE_NAME_CAPACITY],
    pub time: TagTimeValue,
    pub content: [u8; PRONOUNCE_CONTENT_CAPACITY],
}

impl TagPronounceWord {
    const ZERO: Self = Self {
        player_id: 0,
        name: [0; PRONOUNCE_NAME_CAPACITY],
        time: ZERO_TIME,
        content: [0; PRONOUNCE_CONTENT_CAPACITY],
    };

    pub const fn from_complete_fields(
        player_id: i32,
        name: [u8; PRONOUNCE_NAME_CAPACITY],
        time: TagTimeValue,
        content: [u8; PRONOUNCE_CONTENT_CAPACITY],
    ) -> Self {
        Self {
            player_id,
            name,
            time,
            content,
        }
    }

 /// Копирует fixed `tagPronounceWord` из binary DB chunk безопасным
 /// field-wise разбором вместо старого `SafeArrayAccessData` + `memcpy`.
    pub fn from_database_blob(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < PRONOUNCE_DATA_SIZE {
            return None;
        }
        let bytes = &bytes[..PRONOUNCE_DATA_SIZE];
        let player_id = i32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let mut name = [0; PRONOUNCE_NAME_CAPACITY];
        name.copy_from_slice(&bytes[4..0x18]);
        let time_values: [u16; 8] = bytes[0x18..0x28]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;
        let mut content = [0; PRONOUNCE_CONTENT_CAPACITY];
        content.copy_from_slice(&bytes[0x28..PRONOUNCE_DATA_SIZE]);
        Some(Self::from_complete_fields(
            player_id,
            name,
            TagTimeValue {
                year: time_values[0],
                month: time_values[1],
                day_of_week: time_values[2],
                day: time_values[3],
                hour: time_values[4],
                minute: time_values[5],
                second: time_values[6],
                milliseconds: time_values[7],
            },
            content,
        ))
    }

    fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedPronounceField> {
        let Some(terminator) = self.name.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedPronounceField {
                field: "tagPronounceWord::strName",
            });
        };
        Ok(&self.name[..=terminator])
    }

    fn content_wire_bytes(&self) -> Result<&[u8], UnterminatedPronounceField> {
        let Some(terminator) = self.content.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedPronounceField {
                field: "tagPronounceWord::strContent",
            });
        };
        Ok(&self.content[..=terminator])
    }

    fn append_wire_bytes(&self, output: &mut Vec<u8>) {
        append_i32(output, self.player_id);
        output.extend_from_slice(&self.name);
        output.extend_from_slice(&self.time.wire_bytes());
        output.extend_from_slice(&self.content);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnterminatedLeaveWordField {
    pub field: &'static str,
}

impl fmt::Display for UnterminatedLeaveWordField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedLeaveWordField {}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TagLeaveWord {
    pub id: i32,
    pub player_id: i32,
    pub name: [u8; LEAVE_WORD_NAME_CAPACITY],
    pub time: TagTimeValue,
    pub content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
}

impl TagLeaveWord {
    pub const fn from_complete_fields(
        id: i32,
        player_id: i32,
        name: [u8; LEAVE_WORD_NAME_CAPACITY],
        time: TagTimeValue,
        content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
    ) -> Self {
        Self {
            id,
            player_id,
            name,
            time,
            content,
        }
    }

    pub fn content_wire_bytes(&self) -> Result<&[u8], UnterminatedLeaveWordField> {
        let Some(terminator) = self.content.iter().position(|byte| *byte == 0) else {
 // WorldServer `SaveLeaveWords` передавал
 // `node + 0x34` в `CheckPoint` без размера; тот читал бы за
 // `char[212]`, а достижимость и результат этого пути не определены.
            return Err(UnterminatedLeaveWordField {
                field: "tagLeaveWord::strContent",
            });
        };
        Ok(&self.content[..=terminator])
    }

    pub fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedLeaveWordField> {
        let Some(terminator) = self.name.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedLeaveWordField {
                field: "tagLeaveWord::strName",
            });
        };
        Ok(&self.name[..=terminator])
    }
}

const _: () = {
    assert!(size_of::<FactionBaseProperty>() == FACTION_BASE_PROPERTY_SIZE);
    assert!(size_of::<TagApplyPerson>() == 0x20);
    assert!(offset_of!(TagApplyPerson, id) == 0x00);
    assert!(offset_of!(TagApplyPerson, name) == 0x04);
    assert!(offset_of!(TagApplyPerson, occupation) == 0x18);
    assert!(offset_of!(TagApplyPerson, level) == 0x1C);
    assert!(size_of::<TagPronounceWord>() == PRONOUNCE_DATA_SIZE);
    assert!(offset_of!(TagPronounceWord, player_id) == 0x00);
    assert!(offset_of!(TagPronounceWord, name) == 0x04);
    assert!(offset_of!(TagPronounceWord, time) == 0x18);
    assert!(offset_of!(TagPronounceWord, content) == 0x28);
    assert!(size_of::<TagLeaveWord>() == 0x100);
    assert!(offset_of!(TagLeaveWord, id) == 0x00);
    assert!(offset_of!(TagLeaveWord, player_id) == 0x04);
    assert!(offset_of!(TagLeaveWord, name) == 0x08);
    assert!(offset_of!(TagLeaveWord, time) == 0x1C);
    assert!(offset_of!(TagLeaveWord, content) == 0x2C);
};

/// Поля одной строки `CSL_FACTION_BaseProperty`, которые
/// `CRsFaction::LoadFactionProperty` присваивает после public-конструктора
/// `CFaction`.
pub struct FactionDatabaseBaseState {
    pub faction_id: i32,
    pub name: Vec<u8>,
    pub master_id: i32,
    pub established_time: TagTimeValue,
    pub level: i32,
    pub experience: i32,
    pub offense_victor_counts: i32,
    pub defence_victor_counts: i32,
    pub village_war_victor_counts: i32,
    pub member_count: i32,
    pub union_id: i32,
    pub permit: bool,
    pub property_1: i32,
    pub property_2: i32,
    pub delete_remain_time: i32,
    pub country: u8,
    pub goods_war_count: i32,
    pub goods_war_last_win_time: String,
}

pub struct CFaction {
    faction_id: i32,
    name: Vec<u8>,
    master_id: Option<i32>,
    members: BTreeMap<i32, TagMemInfo>,
    base_property: Option<FactionBaseProperty>,
    established_time: Option<TagTimeValue>,
    delete_remain_time: Option<i32>,
    owned_cities: VecDeque<i32>,
    enemy_factions: BTreeSet<i32>,
    city_war_enemy_factions: BTreeSet<i32>,
    permit_demise: Option<bool>,
    enemy_factions_changed: Option<bool>,
    city_war_enemy_factions_changed: Option<bool>,
    apply_persons: BTreeMap<i32, TagApplyPerson>,
    pronounce: TagPronounceWord,
    leave_words: VecDeque<TagLeaveWord>,
    last_upload_icon_time: TagTimeValue,
    icon_data: Vec<u8>,
    change_data_type: i32,
    goods_war_count: i32,
    goods_war_last_win_time: String,
 /// Rust-проекция master faction текущего union для синхронного
 /// `CPlayer::UpdateFactionInfo` без повторного заимствования controller-а.
    union_master_id: i32,
}

impl CFaction {
    pub const fn with_reached_member_state(faction_id: i32) -> Self {
        Self {
            faction_id,
            name: Vec::new(),
            master_id: None,
            members: BTreeMap::new(),
            base_property: None,
            established_time: None,
            delete_remain_time: None,
            owned_cities: VecDeque::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            permit_demise: None,
            enemy_factions_changed: None,
            city_war_enemy_factions_changed: None,
            apply_persons: BTreeMap::new(),
            pronounce: TagPronounceWord::ZERO,
            leave_words: VecDeque::new(),
            last_upload_icon_time: ZERO_TIME,
            icon_data: Vec::new(),
            change_data_type: 0,
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
            union_master_id: 0,
        }
    }

 /// Строит live-state нового `CFaction` из constructor + `Initial`.
 ///
 /// MSVC `strcpy` для трёх fixed-полей заменён явной границей: допустимые
 /// C-string дают те же байты, а переполнение не превращается в memory UB.
    pub fn for_creation(
        faction_id: i32,
        master_id: i32,
        established_time: TagTimeValue,
        faction_name: &[u8],
        master_title: &[u8],
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
    ) -> Result<Self, FactionInitialBlock> {
        let (master_name, master_level, master_occupation, master_region) =
            if let Some(player) = game.online_player_by_id(master_id as u32) {
                let master_name = fixed_initial_string::<FACTION_MEMBER_NAME_CAPACITY>(
                    FactionInitialStringField::MasterName,
                    player.get_name(),
                )?;
                let region_name = match game.region_name(player.get_region_id()) {
                    WorldRegionNameLookup::RegionNotFound
                    | WorldRegionNameLookup::NullRegionPointer => &[][..],
                    WorldRegionNameLookup::Name(name) => name,
                };
                let master_region = fixed_initial_string::<FACTION_MEMBER_TEXT_CAPACITY>(
                    FactionInitialStringField::MasterRegion,
                    region_name,
                )?;
                (
                    master_name,
                    i32::from(player.get_level()),
                    i32::from(player.get_occupation()),
                    master_region,
                )
            } else {
                (
                    [0; FACTION_MEMBER_NAME_CAPACITY],
                    1,
                    1,
                    [0; FACTION_MEMBER_TEXT_CAPACITY],
                )
            };
        let master_title = fixed_initial_string::<FACTION_MEMBER_TEXT_CAPACITY>(
            FactionInitialStringField::MasterTitle,
            master_title,
        )?;

        let mut purview = [EPurviewOwnState::Permit; 11];
        purview[EPurview::DubJobLevel.index()] = EPurviewOwnState::No;
        let master = TagMemInfo::from_complete_fields(
            master_id,
            master_name,
            master_level,
            master_occupation,
            1,
            master_title,
            purview,
            master_region,
            established_time,
            false,
        );

        let mut property = FactionBaseProperty::from_complete_bytes([0; FACTION_BASE_PROPERTY_SIZE]);
        property.write_signed(0x00, 1);
        property.write_signed(0x14, 1);
        property.set_permit(true);

        let mut faction = Self {
            faction_id,
            name: faction_name.to_vec(),
            master_id: Some(master_id),
            members: BTreeMap::from([(master_id, master)]),
            base_property: Some(property),
            established_time: Some(established_time),
            delete_remain_time: Some(parameters.disband_faction_minutes()),
            owned_cities: VecDeque::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            permit_demise: Some(true),
            enemy_factions_changed: None,
            city_war_enemy_factions_changed: None,
            apply_persons: BTreeMap::new(),
            pronounce: TagPronounceWord::ZERO,
            leave_words: VecDeque::new(),
            last_upload_icon_time: established_time,
            icon_data: Vec::new(),
            change_data_type: 0,
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
            union_master_id: 0,
        };
        let _ = faction
            .initial_property_by_level(parameters)
            .expect("for_creation всегда материализует полный base property");
        Ok(faction)
    }

 /// Создаёт base-row в точном порядке `LoadFactionProperty`.
 ///
 /// Public constructor уже выполняет `Initial` до DB assignments. Поэтому
 /// этот owner начинает с `for_creation`, сохраняя fallback master-member и
 /// constructor-derived property bytes, и заменяет только поля row. Полные
 /// member/leave/ability/apply данные принадлежат следующим DB owner-ам.
    pub fn from_database_base_state(
        state: FactionDatabaseBaseState,
        master_title: &[u8],
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
    ) -> Result<Self, FactionInitialBlock> {
        let mut faction = Self::for_creation(
            state.faction_id,
            state.master_id,
            state.established_time,
            &state.name,
            master_title,
            game,
            parameters,
        )?;
        let property = faction
            .base_property
            .as_mut()
            .expect("for_creation всегда назначает полный base property");
        property.write_signed(0x00, state.level);
        property.write_signed(0x04, state.experience);
        property.write_signed(0x08, state.offense_victor_counts);
        property.write_signed(0x0C, state.defence_victor_counts);
        property.write_signed(0x10, state.village_war_victor_counts);
        property.write_signed(0x14, state.member_count);
        property.write_signed(0x18, state.union_id);
        property.set_permit(state.permit);
        property.write_signed(0x30, state.property_1);
        property.write_signed(0x34, state.property_2);
        faction.delete_remain_time = Some(state.delete_remain_time);
        faction.goods_war_count = state.goods_war_count;
        faction.goods_war_last_win_time = state.goods_war_last_win_time;
        faction.change_data_type = 0;
        Ok(faction)
    }

 /// Вставляет одну строку private DB owner-а `LoadFactionMembers`.
 ///
 /// `std::map::operator[]` заменял duplicate player ID последней строкой;
 /// Rust сохраняет именно опубликованное значение, но освобождает прежнее
 /// value обычным ownership вместо старой внутренней lifetime-механики.
 /// Загрузка не ставит dirty-bit и не публикует сетевых сообщений.
    pub fn load_database_member(&mut self, member: TagMemInfo) {
        self.members.insert(member.id, member);
    }

 /// Вставляет одну DB-строку `CSL_Faction_Apply` после join с player base.
 /// Duplicate key сохраняет последнее значение старого `map::operator[]`.
    pub fn load_database_apply_person(&mut self, person: TagApplyPerson) {
        self.apply_persons.insert(person.id, person);
    }

 /// Применяет ровно `0x828` байт поля `Pronounce` из `LoadFactionPronounce`.
 /// Загрузка не меняет dirty-bit и не рассылает обновление клиентам.
    pub fn load_database_pronounce(&mut self, pronounce: TagPronounceWord) {
        self.pronounce = pronounce;
    }

 /// `LoadIconData` в читал blob, но не копировал его в `m_IconData`;
 /// наблюдаемое присваивание относится только к этому времени.
    pub fn load_database_icon_upload_time(&mut self, time: TagTimeValue) {
        self.last_upload_icon_time = time;
    }

 /// Завершает DB-prefix `LoadAllFaction` перед публикацией в controller map.
 ///
 /// Старый owner перезаписывает MemberNums фактическим размером map, затем
 /// вызывает `InitialPropertyByLvl`, игнорируя его bool, и очищает dirty
 /// mask. Никаких client/event side effect этот этап не имеет.
    pub fn complete_database_load(
        &mut self,
        parameters: &COrganizingParam,
    ) -> Result<(), FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        property.write_signed(0x14, self.members.len() as i32);
        let _ = self.initial_property_by_level(parameters)?;
        self.set_change_data(0);
        Ok(())
    }

    pub const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn set_name(&mut self, name: &[u8]) -> bool {
        self.name.clear();
        self.name.extend_from_slice(name);
        true
    }

    pub const fn master_id(&self) -> Option<i32> {
        self.master_id
    }

    pub const fn level(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.level()),
            None => None,
        }
    }

    pub const fn experience(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.experience()),
            None => None,
        }
    }

    pub const fn base_property(&self) -> Option<FactionBaseProperty> {
        self.base_property
    }

    pub const fn country(&self) -> Option<u8> {
        match self.base_property {
            Some(property) => Some(property.country()),
            None => None,
        }
    }

    pub fn set_country(
        &mut self,
        country: u8,
    ) -> Result<(), FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        property.set_country(country);
        Ok(())
    }

    pub const fn permit_demise(&self) -> Option<bool> {
        self.permit_demise
    }

    pub fn set_permit_demise(&mut self, permit: bool) {
        self.permit_demise = Some(permit);
    }

    pub const fn enemy_factions_changed(&self) -> Option<bool> {
        self.enemy_factions_changed
    }

    pub fn set_enemy_factions_changed(&mut self, changed: bool) {
        self.enemy_factions_changed = Some(changed);
    }

    pub const fn city_war_enemy_factions_changed(&self) -> Option<bool> {
        self.city_war_enemy_factions_changed
    }

    pub fn set_city_war_enemy_factions_changed(&mut self, changed: bool) {
        self.city_war_enemy_factions_changed = Some(changed);
    }

    pub const fn is_permitted(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.permit()),
            None => None,
        }
    }

 /// Меняет faction-permit в точном порядке `SetIsPermit`.
 ///
 /// Lookup заменяет исходный singleton и получает ровно текущий union ID;
 /// его результат — master-faction ID, а не master-player ID. `None`
 /// сохраняет общий null/missing результат старого controller-а.
    pub fn set_is_permitted<F>(
        &mut self,
        game: &dyn WorldGameView,
        requester_id: i32,
        permit: bool,
        superior_master_faction_by_id: F,
    ) -> Result<FactionPermitUpdate, FactionPermitBlock>
    where
        F: FnOnce(i32) -> Option<i32>,
    {
        let master_player_id = self
            .master_id
            .ok_or(FactionPermitBlock::MasterIdMissing)?;
        if requester_id != master_player_id {
            return Ok(FactionPermitUpdate::RequesterIsNotMaster);
        }

        let property = self
            .base_property
            .ok_or(FactionPermitBlock::MissingBaseProperty)?;
        if superior_master_faction_by_id(property.union_id())
            .is_some_and(|master_faction_id| master_faction_id != self.faction_id)
        {
            return Ok(FactionPermitUpdate::SuperiorMasterMismatch);
        }
        if property.permit() == permit {
            return Ok(FactionPermitUpdate::Unchanged);
        }

        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionPermitBlock::MissingBaseProperty)?;
        property.set_permit(permit);
        let deliveries = self
            .update_property_to_client(game)
            .map_err(|_| FactionPermitBlock::MissingBaseProperty)?;
        self.set_change_data(1);
        Ok(FactionPermitUpdate::Updated { deliveries })
    }

    pub const fn is_leave_word_function(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.leave_word_function()),
            None => None,
        }
    }

    pub const fn is_create_union_function(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.create_union_function()),
            None => None,
        }
    }

    pub const fn defence_victor_counts(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.defence_victor_counts()),
            None => None,
        }
    }

    pub const fn offense_victor_counts(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.offense_victor_counts()),
            None => None,
        }
    }

    pub fn initial_property_by_level(
        &mut self,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let level = property.level();
        property.set_initial_level_permissions(parameters);

        let maximum_members = parameters.get_max_number_by_level(level);
        if property.signed_at(0x1C) != maximum_members {
            property.write_signed(0x1C, maximum_members);
        }

        let Some(level_parameters) = parameters.get_level_param(level) else {
            return Ok(false);
        };
        property.write_signed(0x20, level_parameters.experience);
        Ok(true)
    }

    pub fn update_property_to_client(
        &self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE0B);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add(property.wire_bytes());
            deliveries.push(FactionPropertyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn reinitialize_property_by_level(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
    ) -> Result<FactionPropertyReinitialization, FactionInitialPropertyBlock> {
        let level_parameters_found = self.initial_property_by_level(parameters)?;
        let deliveries = self.update_property_to_client(game)?;
        Ok(FactionPropertyReinitialization {
            level_parameters_found,
            deliveries,
        })
    }

    fn add_victor_count(
        &mut self,
        game: &dyn WorldGameView,
        property_offset: usize,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let next = property.signed_at(property_offset).wrapping_add(1);
        property.write_signed(property_offset, next);
        let deliveries = self.update_property_to_client(game)?;
        self.set_change_data(1);
        Ok(deliveries)
    }

    pub fn add_defence_victor_count(
        &mut self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x0C)
    }

    pub fn add_offense_victor_count(
        &mut self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x08)
    }

    pub fn add_village_war_victor_count(
        &mut self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x10)
    }

    pub const fn established_time(&self) -> Option<TagTimeValue> {
        self.established_time
    }

    pub const fn delete_remain_time(&self) -> Option<i32> {
        self.delete_remain_time
    }

    pub fn set_delete_remain_time(&mut self, value: i32) {
        if self.delete_remain_time != Some(value) {
            self.delete_remain_time = Some(value);
        }
    }

    pub const fn owned_cities(&self) -> &VecDeque<i32> {
        &self.owned_cities
    }

    pub fn refresh_owned_city_info<F>(
        &self,
        mut refresh_owned_city: F,
    ) -> Result<FactionOwnedCityRefreshReport, FactionOwnedCityRefreshBlock>
    where
        F: FnMut(i32, i32, i32, Option<u8>),
    {
        if self.owned_cities.is_empty() {
            return Ok(FactionOwnedCityRefreshReport {
                refreshed_region_ids: Vec::new(),
            });
        }
        let union_id = self
            .base_property
            .as_ref()
            .ok_or(FactionOwnedCityRefreshBlock::MissingBaseProperty)?
            .union_id();
        let country_id = self
            .base_property
            .as_ref()
            .expect("base property проверен перед country lookup")
            .country();
        let mut refreshed_region_ids = Vec::with_capacity(self.owned_cities.len());
        for &region_id in &self.owned_cities {
            refresh_owned_city(region_id, self.faction_id, union_id, Some(country_id));
            refreshed_region_ids.push(region_id);
        }
        Ok(FactionOwnedCityRefreshReport {
            refreshed_region_ids,
        })
    }

    pub fn add_owned_cities_to_byte_array(
        &self,
        game: &dyn WorldGameView,
        output: &mut Vec<u8>,
    ) -> Result<bool, OwnedCitiesWireBuildError> {
        output.extend_from_slice(&(self.owned_cities.len() as u32).to_le_bytes());
        for &region_id in &self.owned_cities {
            append_i32(output, region_id);
            let region_name = match game.region_name(region_id) {
                WorldRegionNameLookup::RegionNotFound
                | WorldRegionNameLookup::NullRegionPointer => &[][..],
                WorldRegionNameLookup::Name(name) => name,
            };
            output.extend_from_slice(region_name);
            output.push(0);
        }
        Ok(true)
    }

    pub fn update_owned_cities_to_client(
        &self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionOwnedCityDelivery>, FactionOwnedCityUpdateBuildError> {
        let mut preflight = Vec::new();
        self.add_owned_cities_to_byte_array(game, &mut preflight)
            .map_err(FactionOwnedCityUpdateBuildError::Preflight)?;

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut serialized_cities = Vec::new();
            if let Err(source) =
                self.add_owned_cities_to_byte_array(game, &mut serialized_cities)
            {
                return Err(FactionOwnedCityUpdateBuildError::Recipient {
                    source,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            }
            let mut message = CMessage::new(OWNED_CITY_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add(&serialized_cities);
            deliveries.push(FactionOwnedCityDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn set_player_organizing_projection(
        &self,
        player_id: i32,
        region_types: &BTreeMap<i32, Option<u16>>,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        if !self.members.contains_key(&player_id) {
            organizing.faction_id = 0;
            return Ok(());
        }

        organizing.faction_id = self.faction_id;
        organizing.faction_level = self
            .level()
            .ok_or(PlayerOrganizingUpdateError::UninitializedFactionField {
                faction_id: self.faction_id,
                field: "m_Property.lLvl",
            })? as u16;
        organizing.faction_experience = self.experience().ok_or(
            PlayerOrganizingUpdateError::UninitializedFactionField {
                faction_id: self.faction_id,
                field: "m_Property.lExp",
            },
        )?;
        organizing.faction_contribute = self.is_contribute(player_id);
        organizing.faction_name = self.name.clone();
        organizing.faction_title = self.member_title(player_id).map_err(|_| {
            PlayerOrganizingUpdateError::UnterminatedFactionMemberTitle {
                faction_id: self.faction_id,
                player_id,
            }
        })?;
        organizing.faction_master_id = self.master_id.ok_or(
            PlayerOrganizingUpdateError::UninitializedFactionField {
                faction_id: self.faction_id,
                field: "m_lMastterID",
            },
        )?;
        organizing.enemy_factions = self.enemy_factions.clone();
        organizing.city_war_enemy_factions = self.city_war_enemy_factions.clone();
        organizing.clear_owned_regions();
        for &region_id in &self.owned_cities {
            let Some(region_type) = region_types.get(&region_id) else {
                continue;
            };
            let Some(region_type) = *region_type else {
                return Err(PlayerOrganizingUpdateError::UninitializedRegionType { region_id });
            };
            organizing.add_owned_region(region_id, region_type);
        }
        organizing.union_id = self.superior_organizing().unwrap_or(0);
        organizing.union_master_id = if organizing.union_id > 0 {
            self.union_master_id
        } else {
            0
        };
        Ok(())
    }

    pub fn update_player_faction_info<F>(
        &self,
        game: &dyn WorldGameView,
        player_id: i32,
        mut update_player: F,
    ) -> Vec<i32>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        let mut updated_player_ids = Vec::new();
        if player_id == 0 {
            for &member_id in self.members.keys() {
                if game.online_player_by_id(member_id as u32).is_none() {
                    continue;
                }
                update_player(game, self, member_id);
                updated_player_ids.push(member_id);
            }
        } else if game.online_player_by_id(player_id as u32).is_some() {
            update_player(game, self, player_id);
            updated_player_ids.push(player_id);
        }
        updated_player_ids
    }

    fn finish_owned_city_mutation<F>(
        &self,
        game: &dyn WorldGameView,
        state_changed: bool,
        update_player: F,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        let deliveries = self.update_owned_cities_to_client(game).map_err(|source| {
            OwnedCityMutationBuildError {
                state_changed,
                source,
            }
        })?;
        let refreshed_player_ids = self.update_player_faction_info(game, 0, update_player);
        Ok(OwnedCityMutationReport {
            state_changed,
            deliveries,
            refreshed_player_ids,
        })
    }

    pub fn clear_owned_cities<F>(
        &mut self,
        game: &dyn WorldGameView,
        update_player: F,
    ) -> Result<OwnedCityBooleanMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        let state_changed = !self.owned_cities.is_empty();
        self.owned_cities.clear();
        let mutation = self.finish_owned_city_mutation(game, state_changed, update_player)?;
        Ok(OwnedCityBooleanMutationReport {
            legacy_result: true,
            mutation,
        })
    }

    pub fn add_owned_city<F>(
        &mut self,
        game: &dyn WorldGameView,
        region_id: i32,
        update_player: F,
    ) -> Result<OwnedCityAddOutcome, OwnedCityMutationBuildError>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        if self.owned_cities.contains(&region_id) {
            return Ok(OwnedCityAddOutcome::AlreadyOwned);
        }
        self.owned_cities.push_back(region_id);
        let report = self.finish_owned_city_mutation(game, true, update_player)?;
        Ok(OwnedCityAddOutcome::Added(report))
    }

    pub fn add_owned_city_list<F>(
        &mut self,
        game: &dyn WorldGameView,
        region_ids: &VecDeque<i32>,
        update_player: F,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        let previous_state = self.owned_cities.clone();
        self.owned_cities.extend(region_ids.iter().copied());
        let mut previous = None;
        self.owned_cities.retain(|region_id| {
            let keep = previous != Some(*region_id);
            previous = Some(*region_id);
            keep
        });
        let state_changed = self.owned_cities != previous_state;
        self.finish_owned_city_mutation(game, state_changed, update_player)
    }

    pub fn delete_owned_city<F>(
        &mut self,
        game: &dyn WorldGameView,
        region_id: i32,
        update_player: F,
    ) -> Result<OwnedCityBooleanMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        let position = self
            .owned_cities
            .iter()
            .position(|owned_region_id| *owned_region_id == region_id);
        let state_changed = position.is_some();
        if let Some(position) = position {
            self.owned_cities.remove(position);
        }
        let mutation = self.finish_owned_city_mutation(game, state_changed, update_player)?;
        Ok(OwnedCityBooleanMutationReport {
            legacy_result: true,
            mutation,
        })
    }

    pub fn set_owned_cities(
        &mut self,
        game: &dyn WorldGameView,
        region_ids: &VecDeque<i32>,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError> {
        let state_changed = self.owned_cities != *region_ids;
        self.owned_cities.clone_from(region_ids);
        let deliveries = self.update_owned_cities_to_client(game).map_err(|source| {
            OwnedCityMutationBuildError {
                state_changed,
                source,
            }
        })?;
        Ok(OwnedCityMutationReport {
            state_changed,
            deliveries,
            refreshed_player_ids: Vec::new(),
        })
    }

    pub fn is_owned_city(&self, region_id: i32) -> i32 {
        if self.owned_cities.contains(&region_id) {
            self.faction_id
        } else {
            0
        }
    }

    pub const fn is_master(&self, player_id: i32) -> i32 {
        match self.master_id {
            Some(master_id) if master_id == player_id => self.faction_id,
            Some(_) | None => 0,
        }
    }

    pub const fn enemy_factions(&self) -> &BTreeSet<i32> {
        &self.enemy_factions
    }

    pub fn enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        self.enemy_factions.clone()
    }

    pub fn clear_enemy_factions(&mut self) {
        if !self.enemy_factions.is_empty() {
            self.enemy_factions_changed = Some(true);
        }
        self.enemy_factions.clear();
    }

    pub fn has_enemy_faction(&self) -> bool {
        !self.enemy_factions.is_empty()
    }

    pub const fn city_war_enemy_factions(&self) -> &BTreeSet<i32> {
        &self.city_war_enemy_factions
    }

    pub fn city_war_enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        self.city_war_enemy_factions.clone()
    }

    pub fn clear_city_war_enemy_factions(&mut self) {
        if !self.city_war_enemy_factions.is_empty() {
            self.city_war_enemy_factions_changed = Some(true);
        }
        self.city_war_enemy_factions.clear();
    }

    pub fn has_city_war_enemy_faction(&self) -> bool {
        !self.city_war_enemy_factions.is_empty()
    }

    pub const fn is_enemy_faction(&self, _faction_id: i32) -> i32 {
        0
    }

    pub const fn enemy_leader_organizing_id(&self) -> i32 {
        0
    }

    fn write_enemy_war_log<Context>(
        &self,
        context: &mut Context,
        enemy_id: i32,
        string_id: &'static [u8],
        remaining_enemy_count: Option<u32>,
        _state_changed: bool,
        _changed_flag_set: bool,
    ) -> Result<bool, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        if enemy_id <= 0 {
            return Ok(false);
        }
        let Some(enemy_name) = context.organizing_name(enemy_id) else {
            return Ok(false);
        };

        let mut arguments = vec![
            FactionEnemyWarLogArgument::Text(&self.name),
            FactionEnemyWarLogArgument::Text(&enemy_name),
        ];
        if let Some(remaining_enemy_count) = remaining_enemy_count {
            arguments.push(FactionEnemyWarLogArgument::Unsigned(
                remaining_enemy_count,
            ));
        }
        let text = context.format_world_string(string_id, &arguments);
        context.put_war_log(&text);
        Ok(true)
    }

    pub fn add_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.enemy_factions.insert(enemy_id);
        if !state_changed {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        let war_log_written =
            self.write_enemy_war_log(context, enemy_id, b"WS0158", None, true, false)?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    pub fn del_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.enemy_factions.remove(&enemy_id);
        if !state_changed {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        let remaining_enemy_count = self.enemy_factions.len() as u32;
        let war_log_written = self.write_enemy_war_log(
            context,
            enemy_id,
            b"WS0159",
            Some(remaining_enemy_count),
            true,
            false,
        )?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    pub fn add_city_war_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        if self.city_war_enemy_factions.contains(&enemy_id) {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        self.city_war_enemy_factions_changed = Some(true);
        self.city_war_enemy_factions.insert(enemy_id);
        let war_log_written =
            self.write_enemy_war_log(context, enemy_id, b"WS0160", None, true, true)?;
        Ok(FactionEnemyMutationReport {
            state_changed: true,
            changed_flag_set: true,
            war_log_written,
        })
    }

    pub fn del_city_war_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.city_war_enemy_factions.remove(&enemy_id);
        let remaining_enemy_count = self.city_war_enemy_factions.len() as u32;
        let war_log_written = self.write_enemy_war_log(
            context,
            enemy_id,
            b"WS0161",
            Some(remaining_enemy_count),
            state_changed,
            false,
        )?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    pub fn add_enemy_factions_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        append_signed_set(output, &self.enemy_factions);
        true
    }

    pub fn add_city_war_enemy_factions_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> bool {
        append_signed_set(output, &self.city_war_enemy_factions);
        true
    }

    fn update_enemy_set_to_client(
        &self,
        game: &dyn WorldGameView,
        kind: EnemyFactionSetKind,
    ) -> Vec<FactionEnemyDelivery> {
        let (message_type, enemy_factions) = match kind {
            EnemyFactionSetKind::Standard => (0x7FE11, &self.enemy_factions),
            EnemyFactionSetKind::CityWar => (0x7FE12, &self.city_war_enemy_factions),
        };
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(message_type);
            message.base_mut().add_long(recipient_player_id);
            let mut serialized_set = Vec::new();
            append_signed_set(&mut serialized_set, enemy_factions);
            message.base_mut().add(&serialized_set);
            deliveries.push(FactionEnemyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        deliveries
    }

    pub fn update_enemy_factions_to_client(
        &self,
        game: &dyn WorldGameView,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::Standard)
    }

    pub fn update_city_war_enemy_factions_to_client(
        &self,
        game: &dyn WorldGameView,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::CityWar)
    }

    pub fn update_enemy_faction<F>(
        &mut self,
        game: &dyn WorldGameView,
        update_player: F,
    ) -> FactionEnemyRefreshReport
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        self.enemy_factions_changed = Some(true);
        let deliveries = self.update_enemy_factions_to_client(game);
        let refreshed_player_ids = self.update_player_faction_info(game, 0, update_player);
        FactionEnemyRefreshReport {
            deliveries,
            refreshed_player_ids,
        }
    }

    pub fn update_city_war_enemy_faction<F>(
        &self,
        game: &dyn WorldGameView,
        update_player: F,
    ) -> CityWarEnemyRefreshOutcome
    where
        F: FnMut(&dyn WorldGameView, &CFaction, i32),
    {
        match self.city_war_enemy_factions_changed {
            None => CityWarEnemyRefreshOutcome::ChangeFlagUnknown,
            Some(false) => CityWarEnemyRefreshOutcome::Unchanged,
            Some(true) => {
                let deliveries = self.update_city_war_enemy_factions_to_client(game);
                let refreshed_player_ids =
                    self.update_player_faction_info(game, 0, update_player);
                CityWarEnemyRefreshOutcome::Published(FactionEnemyRefreshReport {
                    deliveries,
                    refreshed_player_ids,
                })
            }
        }
    }

    pub fn update_experience_to_client(
        &self,
        game: &dyn WorldGameView,
    ) -> Result<Vec<FactionExperienceDelivery>, FactionExperienceBlock> {
        let property = self
            .base_property
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        let master_id = self
            .master_id
            .ok_or(FactionExperienceBlock::MasterIdMissing)?;
        let mut deliveries = Vec::new();
        for (&recipient_player_id, member) in &self.members {
            if !member.contribute && recipient_player_id != master_id {
                continue;
            }
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE14);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(property.experience());
            message.base_mut().add_long(property.upgrade_experience());
            deliveries.push(FactionExperienceDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn update_pronounce_to_client(
        &self,
        game: &dyn WorldGameView,
        operator: EOperator,
    ) -> Result<Vec<FactionPronounceDelivery>, FactionPronounceUpdateBuildError> {
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(PRONOUNCE_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(self.pronounce.player_id);
            let name = match self.pronounce.name_wire_bytes() {
                Ok(name) => name,
                Err(source) => {
                    return Err(FactionPronounceUpdateBuildError {
                        source,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
            };
            message.base_mut().add(name);
            let content = match self.pronounce.content_wire_bytes() {
                Ok(content) => content,
                Err(source) => {
                    return Err(FactionPronounceUpdateBuildError {
                        source,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
            };
            message.base_mut().add(content);
            message.base_mut().add(&self.pronounce.time.wire_bytes());
            deliveries.push(FactionPronounceDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn update_all_apply_members_to_client(
        &self,
        game: &dyn WorldGameView,
        recipient_player_id: i32,
    ) -> Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError> {
        let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
        let player = game.online_player_by_id(recipient_player_id as u32);
        if game_server_id == 0 || player.is_none_or(|player| !player.faction_data_received()) {
            return Ok(Vec::new());
        }

        let mut deliveries = Vec::new();
        for person in self.apply_persons.values() {
            let Some(name) = person.name_wire_bytes() else {
                return Err(FactionApplyMemberUpdateBuildError {
                    candidate_player_id: person.id,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            };

            let mut message = CMessage::new(APPLY_MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(EOperator::Add.wire_value());
            message.base_mut().add_long(person.id);
            message.base_mut().add(name);
            message.base_mut().add_long(person.occupation);
            message.base_mut().add_long(person.level);
            deliveries.push(FactionApplyMemberDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn update_apply_member_to_client(
        &self,
        game: &dyn WorldGameView,
        candidate_player_id: i32,
        operator: EOperator,
    ) -> Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError> {
        let person = self
            .apply_persons
            .get(&candidate_player_id)
            .copied()
            .unwrap_or_else(|| TagApplyPerson::empty_with_id(candidate_player_id));
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received())
                || game_server_id == 0
                || !self.is_using_purview(recipient_player_id, EPurview::ConMem as i32)
            {
                continue;
            }

            let mut message = CMessage::new(APPLY_MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(person.id);
            let Some(name) = person.name_wire_bytes() else {
                return Err(FactionApplyMemberUpdateBuildError {
                    candidate_player_id: person.id,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            };
            message.base_mut().add(name);
            message.base_mut().add_long(person.occupation);
            message.base_mut().add_long(person.level);
            deliveries.push(FactionApplyMemberDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn remove_apply_member(
        &mut self,
        game: &dyn WorldGameView,
        candidate_player_id: i32,
    ) -> FactionRemoveApplyMemberOutcome {
        if self.apply_persons.remove(&candidate_player_id).is_none() {
            return FactionRemoveApplyMemberOutcome::NotFound;
        }

        let deliveries =
            self.update_apply_member_to_client(game, candidate_player_id, EOperator::Delete);
        self.set_change_data(8);
        FactionRemoveApplyMemberOutcome::Removed {
            faction_id: self.faction_id,
            deliveries,
        }
    }

    pub fn apply_for_join<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        _legacy_second_parameter: i32,
        _legacy_third_parameter: i32,
        context: &mut Context,
    ) -> Result<FactionApplyForJoinOutcome, FactionApplyForJoinBlock<Context::Block>>
    where
        Context: FactionApplyForJoinContext + ?Sized,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::StandardOrVillageWar,
            ));
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, player_id, b"WS0163", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::CityWar,
            ));
        }

        let level = self
            .level()
            .ok_or(FactionApplyForJoinBlock::MissingBaseProperty)?;
        let maximum_members = parameters.get_max_number_by_level(level);
        if self.members.len() as u32 as i32 >= maximum_members {
            send_apply_join_information(context, player_id, b"WS0164", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::MemberLimit,
            ));
        }
        if self.apply_persons.len() as u32 >= MAX_APPLY_PERSON_COUNT {
            send_apply_join_information(context, player_id, b"WS0165", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::ApplyListLimit,
            ));
        }

        if context
            .player_already_in_faction(self, player_id)
            .map_err(|source| FactionApplyForJoinBlock::Context {
                operation: FactionApplyForJoinContextOperation::PlayerMembershipLookup,
                source,
            })?
        {
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::PlayerAlreadyInFaction,
            ));
        }
        context
            .remove_previous_faction_applications(self, player_id)
            .map_err(|source| FactionApplyForJoinBlock::Context {
                operation: FactionApplyForJoinContextOperation::RemovePreviousApplications,
                source,
            })?;

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::PlayerOffline,
            ));
        };
        let player_name = legacy_c_string_visible_bytes(player.get_name());
        if player_name.len() >= APPLY_PERSON_NAME_CAPACITY {
            return Err(FactionApplyForJoinBlock::PlayerNameWouldOverflow {
                player_id,
                visible_len: player_name.len(),
            });
        }
        let mut name = [0; APPLY_PERSON_NAME_CAPACITY];
        name[..player_name.len()].copy_from_slice(player_name);
        let person = TagApplyPerson::from_complete_fields(
            player_id,
            name,
            i32::from(player.get_occupation()),
            i32::from(player.get_level()),
        );
        self.apply_persons.insert(player_id, person);

        let formatted =
            context.format_world_string(b"WS0166", &[legacy_c_string_visible_bytes(&self.name)]);
        let formatted = legacy_c_string_visible_bytes(&formatted);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: formatted,
            second_text: legacy_c_string_visible_bytes(&second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });

        let deliveries = self.update_apply_member_to_client(game, player_id, EOperator::Add);
        self.set_change_data(8);
        let log_written = context.faction_apply_log_enabled();
        if log_written {
            context.write_faction_apply_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                player_id,
                player_name,
                2,
            );
        }
        Ok(FactionApplyForJoinOutcome::Applied {
            deliveries,
            log_written,
        })
    }

    pub fn exit<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionExitOutcome, FactionExitBlock>
    where
        Context: FactionExitContext,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::StandardOrVillageWar,
                goods_war_notice_sent: false,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, player_id, b"WS0163", b"WS0121");
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::CityWar,
                goods_war_notice_sent: false,
            });
        }

        let goods_war_notice_sent = context.goods_war_blocks_exit(self.faction_id, player_id);
        if goods_war_notice_sent {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
        }

        if !self.is_using_purview(player_id, EPurview::Exit as i32) {
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::PermissionDenied,
                goods_war_notice_sent,
            });
        }

        let member = self
            .members
            .get(&player_id)
            .expect("PV_Exit может принадлежать только существующему member");
        let player_name_wire = member.name_wire_bytes().map_err(|_| {
            FactionExitBlock::UnterminatedMemberName {
                player_id,
                goods_war_notice_sent,
            }
        })?;
        let player_name = player_name_wire[..player_name_wire.len() - 1].to_vec();
        let notice = context.format_world_string(b"WS0187", &[&player_name]);
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0188").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let member_removal = match self.del_member(player_id, parameters) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionExitBlock::DelMember {
                    source,
                    goods_war_notice_sent,
                    member_information,
                    member_removed: !self.members.contains_key(&player_id),
                });
            }
        };
        let refreshed_player_ids =
            self.update_player_faction_info(game, player_id, |_view, faction, player_id| {
                context.update_player_faction_info(faction, player_id);
            });
        let delete_organizing = match self.delete_organizing_to_client(game, player_id, context) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(FactionExitBlock::DeleteOrganizing {
                    source,
                    goods_war_notice_sent,
                    member_information,
                    member_removal,
                    refreshed_player_ids,
                });
            }
        };
        let member_update = self.update_member_info_to_client(game, player_id, EOperator::Delete);
        self.set_change_data(2);

        let log_written = context.faction_quit_log_enabled();
        if log_written {
            context.write_faction_quit_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                player_id,
                &player_name,
                3,
            );
        }
        context.delete_goods_war_member(player_id);

        Ok(FactionExitOutcome::Exited {
            goods_war_notice_sent,
            member_information,
            member_removal,
            delete_organizing,
            refreshed_player_ids,
            member_update,
            log_written,
        })
    }

    pub fn fire_out<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_id: i32,
        context: &mut Context,
    ) -> Result<FactionFireOutOutcome, FactionFireOutBlock>
    where
        Context: FactionFireOutContext,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::StandardOrVillageWar,
            ));
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, manager_id, b"WS0163", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::CityWar,
            ));
        }
        if context.goods_war_blocks_fire_out(self.faction_id, manager_id) {
            send_apply_join_information(context, manager_id, b"WS0363 ", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::GoodsWar,
            ));
        }

        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::FireOut as i32)
            .map_err(FactionFireOutBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::OperatorValidationFailed,
            ));
        }

        let target = self
            .members
            .get(&target_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = target
            .name_wire_bytes()
            .map_err(|_| FactionFireOutBlock::UnterminatedTargetName { target_id })?;
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let notice = context.format_world_string(b"WS0192", &[&target_name]);
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let member_removal = match self.del_member(target_id, parameters) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionFireOutBlock::DelMember {
                    source,
                    member_information,
                    member_removed: !self.members.contains_key(&target_id),
                });
            }
        };
        let delete_organizing = match self.delete_organizing_to_client(game, target_id, context) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(FactionFireOutBlock::DeleteOrganizing {
                    source,
                    member_information,
                    member_removal,
                });
            }
        };
        let member_update = self.update_member_info_to_client(game, target_id, EOperator::Delete);
        let refreshed_player_ids =
            self.update_player_faction_info(game, target_id, |_view, faction, player_id| {
                context.update_player_faction_info(faction, player_id);
            });
        self.set_change_data(2);

        let log_written = context.faction_fire_out_log_enabled();
        if log_written {
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate сохраняет manager member");
            let manager_name = match manager.name_wire_bytes() {
                Ok(name) => &name[..name.len() - 1],
                Err(_) => {
                    return Err(FactionFireOutBlock::UnterminatedManagerName {
                        manager_id,
                        member_information,
                        member_removal,
                        refreshed_player_ids,
                        member_update,
                        delete_organizing,
                    });
                }
            };
            context.write_faction_fire_out_log(
                target_id,
                &target_name,
                manager.id,
                manager_name,
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                1,
            );
        }
        context.delete_goods_war_member(target_id);

        let mut message = CMessage::new(FACTION_MEMBER_REMOVED_LOCAL_MESSAGE_TYPE);
        message.base_mut().add_long(target_id);
        let mut fixed_faction_name = [0; FACTION_MEMBER_NAME_CAPACITY];
        let copied_name_len = self.name.len().min(FACTION_MEMBER_NAME_CAPACITY);
        fixed_faction_name[..copied_name_len].copy_from_slice(&self.name[..copied_name_len]);
        message.base_mut().add(&fixed_faction_name);
        let local_message = game.queue_local_world_message(message);

        Ok(FactionFireOutOutcome::Fired {
            member_information,
            member_removal,
            delete_organizing,
            member_update,
            refreshed_player_ids,
            log_written,
            local_message,
        })
    }

    pub fn dub_and_set_job_level<Context>(
        &mut self,
        game: &dyn WorldGameView,
        manager_id: i32,
        target_id: i32,
        title: &mut Vec<u8>,
        job_level: i32,
        context: &mut Context,
    ) -> Result<FactionDubOutcome, FactionDubBlock>
    where
        Context: FactionDubContext,
    {
        let mut progress = FactionDubProgress {
            input_title_truncated: false,
            title_changed: false,
            job_level_changed: false,
            title_information: None,
            title_refreshed_player_ids: Vec::new(),
            job_level_information: None,
            member_update: None,
            dirty_set: false,
        };

        if !context.check_invalid_string(title, false) {
            send_apply_join_information(context, manager_id, b"WS0194", b"WS0193");
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::InvalidTitle,
                input_title_truncated: false,
            });
        }
        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::DubJobLevel as i32)
            .map_err(FactionDubBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::OperatorValidationFailed,
                input_title_truncated: false,
            });
        }
        if !(1..=99).contains(&job_level) {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::InvalidJobLevel,
                input_title_truncated: false,
            });
        }

        if 20 < title.len() {
            title.truncate(20);
            progress.input_title_truncated = true;
        }
        let Some(target) = self.members.get(&target_id) else {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::TargetNotFound,
                input_title_truncated: progress.input_title_truncated,
            });
        };
        let target_title_wire = match target.title_wire_bytes() {
            Ok(value) => value,
            Err(source) => {
                return Err(FactionDubBlock::UnterminatedMemberField {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let target_title = &target_title_wire[..target_title_wire.len() - 1];
        let new_title = legacy_c_string_visible_bytes(title).to_vec();
        let mut old_title = Vec::new();

        if target_title != new_title {
            old_title.extend_from_slice(target_title);
            let target = self
                .members
                .get_mut(&target_id)
                .expect("target найден до title-ветки");
            target.title[..new_title.len()].copy_from_slice(&new_title);
            target.title[new_title.len()] = 0;
            progress.title_changed = true;

            let target = self
                .members
                .get(&target_id)
                .expect("title-ветка не удаляет target");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let target_name = &target_name_wire[..target_name_wire.len() - 1];
            let notice = context.format_world_string(
                b"WS0195",
                &[
                    FactionDubFormatArgument::Text(target_name),
                    FactionDubFormatArgument::Text(&new_title),
                ],
            );
            let notice = legacy_c_string_visible_bytes(&notice);
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            progress.title_information = Some(self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ));
            progress.title_refreshed_player_ids =
                self.update_player_faction_info(game, target_id, |_view, faction, player_id| {
                    context.update_player_faction_info(faction, player_id);
                });
        }

        let current_job_level = self
            .members
            .get(&target_id)
            .expect("target найден до job-level ветки")
            .job_level;
        if current_job_level != job_level {
            self.members
                .get_mut(&target_id)
                .expect("target найден до job-level записи")
                .job_level = job_level;
            progress.job_level_changed = true;

            let target = self
                .members
                .get(&target_id)
                .expect("job-level ветка не удаляет target");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let target_name = &target_name_wire[..target_name_wire.len() - 1];
            let notice = context.format_world_string(
                b"WS0196",
                &[
                    FactionDubFormatArgument::Text(target_name),
                    FactionDubFormatArgument::Signed(job_level),
                ],
            );
            let notice = legacy_c_string_visible_bytes(&notice);
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            progress.job_level_information = Some(self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ));
        }

        progress.member_update = Some(self.update_member_info_to_client(
            game,
            target_id,
            EOperator::Update,
        ));
        self.set_change_data(2);
        progress.dirty_set = true;

        let log_written = context.faction_title_log_enabled();
        if log_written {
            let target = self
                .members
                .get(&target_id)
                .expect("успешный owner сохраняет target member");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate сохраняет manager member");
            let manager_name_wire = match manager.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: manager_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_title_log(
                target.id,
                &target_name_wire[..target_name_wire.len() - 1],
                &old_title,
                &new_title,
                manager.id,
                &manager_name_wire[..manager_name_wire.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
            );
        }

        Ok(FactionDubOutcome::Updated {
            progress,
            log_written,
        })
    }

    pub fn endue_right_to_member<Context>(
        &mut self,
        game: &dyn WorldGameView,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        self.change_member_purview(
            game,
            manager_id,
            target_id,
            purview,
            FactionPurviewChange::Grant,
            context,
        )
    }

    pub fn abolish_right_to_member<Context>(
        &mut self,
        game: &dyn WorldGameView,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        self.change_member_purview(
            game,
            manager_id,
            target_id,
            purview,
            FactionPurviewChange::Revoke,
            context,
        )
    }

    pub fn set_maximum_members<Context>(
        &mut self,
        maximum_members: i32,
        context: &mut Context,
    ) -> Result<FactionMaximumMembersUpdate, FactionMaximumMembersBlock>
    where
        Context: FactionLevelContext,
    {
        let property = self
            .base_property
            .ok_or(FactionMaximumMembersBlock::MissingBaseProperty)?;
        if property.signed_at(0x1C) == maximum_members {
            return Ok(FactionMaximumMembersUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionMaximumMembersBlock::MissingBaseProperty)?
            .write_signed(0x1C, maximum_members);

        let notice = context.format_world_string_signed(b"WS0186", maximum_members);
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        Ok(FactionMaximumMembersUpdate::Updated(
            self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ),
        ))
    }

    pub const fn inc_maximum_number(&self, _amount: i32) -> bool {
        true
    }

    pub fn set_level<Context>(
        &mut self,
        level: i32,
        parameters: &COrganizingParam,
        context: &mut Context,
    ) -> Result<FactionLevelUpdate, FactionLevelBlock>
    where
        Context: FactionLevelContext,
    {
        let property = self
            .base_property
            .ok_or(FactionLevelBlock::MissingBaseProperty {
                level_changed: false,
            })?;
        if !(1..=12).contains(&level) || property.level() == level {
            return Ok(FactionLevelUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionLevelBlock::MissingBaseProperty {
                level_changed: false,
            })?
            .write_signed(0x00, level);

        let feature_block = |_| FactionLevelBlock::MissingBaseProperty {
            level_changed: true,
        };
        let pronounce = self
            .set_feature_function(
                FactionFeatureFunction::Pronounce,
                parameters.pronounce_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let leave_word = self
            .set_feature_function(
                FactionFeatureFunction::LeaveWord,
                parameters.leave_word_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let endue_right = self
            .set_feature_function(
                FactionFeatureFunction::EndueRight,
                parameters.endue_right_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let create_union = self
            .set_feature_function(
                FactionFeatureFunction::CreateUnion,
                parameters.create_union_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let join_village_war = self
            .set_feature_function(
                FactionFeatureFunction::JoinVillageWar,
                parameters.attack_village_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let join_city_war = self
            .set_feature_function(
                FactionFeatureFunction::JoinCityWar,
                parameters.attack_city_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let maximum_members = self
            .set_maximum_members(parameters.get_max_number_by_level(level), context)
            .map_err(|source| FactionLevelBlock::MaximumMembers {
                source,
                level_changed: true,
            })?;

        Ok(FactionLevelUpdate::Updated {
            pronounce,
            leave_word,
            endue_right,
            create_union,
            join_village_war,
            join_city_war,
            maximum_members,
        })
    }

    pub fn set_parameter<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        parameter: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Result<FactionSetParameterOutcome, FactionSetParameterBlock>
    where
        Context: FactionSetParameterContext,
    {
        let parameter = match legacy_c_string_visible_bytes(parameter) {
            b"Level" => FactionSetParameterKind::Level,
            b"Experience" => FactionSetParameterKind::Experience,
            _ => FactionSetParameterKind::Unknown,
        };
        if parameter == FactionSetParameterKind::Level && value > 12 {
            return Ok(FactionSetParameterOutcome::LevelAboveMaximum {
                requested_level: value,
            });
        }

        let mut progress = empty_faction_set_parameter_progress(parameter);
        match parameter {
            FactionSetParameterKind::Level => {
                progress.level_update = match self.set_level(value, parameters, context) {
                    Ok(update) => Some(update),
                    Err(source) => {
                        return Err(FactionSetParameterBlock::Level { source, progress });
                    }
                };

                let level = match self.base_property {
                    Some(property) => property.level(),
                    None => {
                        return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
                    }
                };
                let Some(level_parameters) = parameters.get_level_param(level) else {
                    return Ok(FactionSetParameterOutcome::LevelParametersMissing {
                        level,
                        progress,
                    });
                };
                let Some(property) = self.base_property.as_mut() else {
                    return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
                };
                property.write_signed(0x20, level_parameters.experience);
                progress.assigned_upgrade_experience = Some(level_parameters.experience);
            }
            FactionSetParameterKind::Experience => {
                let experience_before = self.base_property.map(|property| property.experience());
                progress.experience_update = match self.set_experience(game, value) {
                    Ok(update) => {
                        if let FactionExperienceUpdate::Updated { experience, .. } = &update {
                            progress.assigned_experience = Some(*experience);
                            progress.dirty_bit_requested = true;
                        }
                        Some(update)
                    }
                    Err(source) => {
                        let experience_after =
                            self.base_property.map(|property| property.experience());
                        if experience_after != experience_before {
                            progress.assigned_experience = experience_after;
                            progress.dirty_bit_requested = true;
                        }
                        return Err(FactionSetParameterBlock::Experience { source, progress });
                    }
                };
            }
            FactionSetParameterKind::Unknown => {}
        }

        progress.property_deliveries = match self.update_property_to_client(game) {
            Ok(deliveries) => Some(deliveries),
            Err(_) => {
                return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
            }
        };
        progress.refreshed_player_ids = Some(self.update_player_faction_info(
            game,
            0,
            |_view, faction, player_id| {
                context.update_player_faction_info(faction, player_id)
            },
        ));
        self.set_change_data(1);
        progress.dirty_bit_requested = true;
        Ok(FactionSetParameterOutcome::Applied(progress))
    }

    pub fn upgrade<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionUpgradeOutcome, FactionUpgradeBlock>
    where
        Context: FactionUpgradeContext,
    {
        let mut progress = empty_faction_upgrade_progress();
        if self.is_master(player_id) == 0 {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::PlayerNotMaster,
                notice_sent: false,
                progress,
            });
        }
        let property = match self.base_property {
            Some(property) => property,
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        };
        let current_level = property.level();
        if current_level >= 12 {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::MaximumLevel,
                notice_sent: false,
                progress,
            });
        }
        let Some(level_parameters) = parameters.get_level_param(current_level).cloned() else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::CurrentLevelParametersMissing,
                notice_sent: false,
                progress,
            });
        };
        let current_experience = property.experience();
        if current_experience < level_parameters.experience {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Experience,
                &[FactionUpgradeFormatArgument::Signed(
                    level_parameters.experience,
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Experience,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::InsufficientExperience,
                notice_sent: true,
                progress,
            });
        }

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::PlayerOffline,
                notice_sent: false,
                progress,
            });
        };
        let player_money = match context.player_money(player_id) {
            Some(money) => money,
            None => {
                return Err(FactionUpgradeBlock::MissingPlayerMoney {
                    player_id,
                    progress,
                });
            }
        };
        if player_money < level_parameters.money as u32 {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Money,
                &[FactionUpgradeFormatArgument::Signed(level_parameters.money)],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Money,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::InsufficientMoney,
                notice_sent: true,
                progress,
            });
        }
        if i32::from(player.get_level()) < level_parameters.master_level {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::MasterLevel,
                &[FactionUpgradeFormatArgument::Signed(
                    level_parameters.master_level,
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::MasterLevel,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::MasterLevelTooLow,
                notice_sent: true,
                progress,
            });
        }

        let goods_original = legacy_c_string_visible_bytes(&level_parameters.goods);
        if level_parameters.goods.as_slice() != b"0"
            && context.goods_in_packet(player_id, goods_original) < 1
        {
            let display_name = context
                .goods_display_name(goods_original)
                .unwrap_or_else(|| goods_original.to_vec());
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Goods,
                &[FactionUpgradeFormatArgument::Text(
                    legacy_c_string_visible_bytes(&display_name),
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Goods,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::RequiredGoodsMissing,
                notice_sent: true,
                progress,
            });
        }

        let level_update = match self.set_level(current_level.wrapping_add(1), parameters, context) {
            Ok(update) => update,
            Err(source) => {
                return Err(FactionUpgradeBlock::Level { source, progress });
            }
        };
        progress.level_update = Some(level_update);
        let experience_update = match self.set_experience(
            game,
            current_experience.wrapping_sub(level_parameters.experience),
        ) {
            Ok(update) => update,
            Err(source) => {
                return Err(FactionUpgradeBlock::Experience { source, progress });
            }
        };
        progress.experience_update = Some(experience_update);

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let mut charge = CMessage::new(FACTION_UPGRADE_CHARGE_MESSAGE_TYPE);
        charge.base_mut().add_long(player_id);
        charge.base_mut().add_long(level_parameters.money);
        charge
            .base_mut()
            .add(&legacy_c_string_wire_bytes(&level_parameters.goods));
        progress.charge_delivery = Some(FactionUpgradeChargeDelivery {
            game_server_id,
            result: game.send_msg_to_game_server(game_server_id, &charge),
        });

        let upgraded_level = match self.level() {
            Some(level) => level,
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        };
        let Some(next_level_parameters) = parameters.get_level_param(upgraded_level) else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::NextLevelParametersMissing,
                notice_sent: false,
                progress,
            });
        };
        let next_upgrade_experience = next_level_parameters.experience;
        match self.base_property.as_mut() {
            Some(property) => property.write_signed(0x20, next_upgrade_experience),
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        }
        progress.next_upgrade_experience = Some(next_upgrade_experience);

        let notice = match format_faction_upgrade_notice(
            context,
            FactionUpgradeNotice::Success,
            &[FactionUpgradeFormatArgument::Signed(upgraded_level)],
        ) {
            Ok(notice) => notice,
            Err(formatted_len) => {
                return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                    notice: FactionUpgradeNotice::Success,
                    formatted_len,
                    progress,
                });
            }
        };
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members(
            &notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        ));
        progress.property_deliveries = Some(match self.update_property_to_client(game) {
            Ok(deliveries) => deliveries,
            Err(_) => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        });
        self.set_change_data(1);
        progress.dirty_set = true;
        progress.refreshed_player_ids = self.update_player_faction_info(game, 0, |_view, faction, player_id| {
            context.update_player_faction_info(faction, player_id);
        });

        if context.faction_level_log_enabled() {
            let member = self
                .members
                .get(&player_id)
                .expect("master invariant и Upgrade не удаляют member");
            let member_name_wire = match member.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionUpgradeBlock::UnterminatedMemberName {
                        player_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_level_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                upgraded_level,
                member.id,
                &member_name_wire[..member_name_wire.len() - 1],
            );
            progress.log_written = true;
        }

        Ok(FactionUpgradeOutcome::Upgraded(progress))
    }

    #[allow(clippy::too_many_arguments)]
    fn change_member_purview<Context>(
        &mut self,
        game: &dyn WorldGameView,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        change: FactionPurviewChange,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        let property = self
            .base_property
            .ok_or(FactionPurviewChangeBlock::MissingBaseProperty)?;
        if !property.feature_function(FactionFeatureFunction::EndueRight) {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::FunctionDisabled,
            ));
        }
        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::EndueRor as i32)
            .map_err(FactionPurviewChangeBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::OperatorValidationFailed,
            ));
        }
        if !(EPurview::DubJobLevel as i32..=EPurview::OperCityGate as i32).contains(&purview) {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::InvalidPurview,
            ));
        }

        let mutation = match change {
            FactionPurviewChange::Grant => self.set_member_purview(target_id, purview),
            FactionPurviewChange::Revoke => self.abolish_member_purview(target_id, purview),
        };
        let member_update =
            self.update_member_info_to_client(game, target_id, EOperator::Update);
        self.set_change_data(2);
        let apply_snapshot = if change == FactionPurviewChange::Grant
            && purview == EPurview::ConMem as i32
        {
            Some(self.update_all_apply_members_to_client(game, target_id))
        } else {
            None
        };
        let mut progress = FactionPurviewChangeProgress {
            mutation,
            member_update,
            dirty_set: true,
            apply_snapshot,
            member_information: None,
        };

        let target = self
            .members
            .get(&target_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = match target.name_wire_bytes() {
            Ok(value) => value,
            Err(source) => {
                return Err(FactionPurviewChangeBlock::UnterminatedMemberName {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let string_id = faction_purview_notice_string_id(change, purview);
        let notice = context.format_world_string(string_id, &target_name);
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        ));

        let log_written = context.faction_purview_log_enabled(change);
        if log_written {
            let target = self
                .members
                .get(&target_id)
                .expect("permission owner не удаляет target");
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate гарантирует manager member");
            let manager_name_wire = match manager.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionPurviewChangeBlock::UnterminatedMemberName {
                        player_id: manager_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_purview_log(
                target.id,
                &target_name,
                purview,
                manager.id,
                &manager_name_wire[..manager_name_wire.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                match change {
                    FactionPurviewChange::Grant => 0,
                    FactionPurviewChange::Revoke => 1,
                },
            );
        }

        Ok(FactionPurviewChangeOutcome::Changed {
            progress,
            log_written,
        })
    }

    pub fn do_join<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_id: i32,
        approve_flag: i32,
        join_time: TagTimeValue,
        context: &mut Context,
    ) -> Result<FactionDoJoinOutcome, FactionDoJoinBlock<Context::Block>>
    where
        Context: FactionDoJoinContext + ?Sized,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::StandardOrVillageWar,
                application_removal: None,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, manager_id, b"WS0163", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::CityWar,
                application_removal: None,
            });
        }
        if context.goods_war_blocks_join(self.faction_id, manager_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::GoodsWar,
                application_removal: None,
            });
        }
        if !self.is_using_purview(manager_id, EPurview::ConMem as i32) {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::PermissionDenied,
                application_removal: None,
            });
        }

        let Some(apply_person) = self.apply_persons.get(&applicant_id).copied() else {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::ApplicationNotFound,
                application_removal: None,
            });
        };
        let application_removal = self.remove_apply_member(game, applicant_id);

        if approve_flag == 0 {
            let notice = context
                .format_world_string(b"WS0167", &[legacy_c_string_visible_bytes(&self.name)]);
            let notice = legacy_c_string_visible_bytes(&notice);
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: applicant_id,
                first_text: notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDoJoinOutcome::ApplicationDenied {
                application_removal,
            });
        }

        let level = self
            .level()
            .ok_or(FactionDoJoinBlock::MissingBaseProperty {
                application_removed: true,
            })?;
        let maximum_members = parameters.get_max_number_by_level(level);
        if self.members.len() as u32 as i32 >= maximum_members {
            let first_text = context.world_string(b"WS0168").unwrap_or_default();
            let first_text = legacy_c_string_visible_bytes(&first_text);
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::MemberLimit,
                application_removal: Some(application_removal),
            });
        }

        context
            .remove_previous_faction_applications(self, applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::RemovePreviousApplications,
                source,
                application_removed: true,
                member_inserted: false,
            })?;
        if context
            .applicant_already_in_faction(self, applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::ApplicantMembershipLookup,
                source,
                application_removed: true,
                member_inserted: false,
            })?
        {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::ApplicantAlreadyInFaction,
                application_removal: Some(application_removal),
            });
        }

        let title = context.world_string(b"WS0169").unwrap_or_default();
        let title = fixed_do_join_string::<FACTION_MEMBER_TEXT_CAPACITY, Context::Block>(
            FactionDoJoinStringField::MemberTitle,
            &title,
            true,
            false,
        )?;
        // Досверка 2026-09-26 (Nworldserver.exe; CFaction::DoJoin=0x4BEE40 —
        // PDB public 1:000bde40 занижен на 0x1000, как у соседних CFaction-методов,
        // истинные адреса через vftable 0x547A8C):
        // purviews нового member пишутся машинно: слоту Exit 2 по 0x4BF3A1
        // (локальный pair-blob +0x74) и слоту LeaveWord 2 по 0x4BF3A8 (+0x88),
        // затем девять нулей в остальные слоты 0x4BF3D2..0x4BF40A; каждый слот
        // пишется один раз — бывшие гипотезы «dead double-store» и «слоты
        // idx1/idx6 не пишутся» были следом ошибки учёта esp через push между
        // записями. job_level=99 пишется явно: 0x4BF263 (blob +0x2C), strcpy
        // title идёт в +0x30 и его не перекрывает. Копия join tagTime полная
        // (бывший F2 опровергнут): t+0→+0 0x4BF231, t+4→+4 0x4BF23B,
        // t+0xC→+0xC 0x4BF245, t+8→+8 0x4BF255 — store идёт после push 0x4BF24C
        // и попадает в свой слот +8, не поверх +0xC; insert (rep movsd 0x3C
        // @0x4BF601 от blob) уносит все 16 байт в map до рассылки.
        let mut purview = [EPurviewOwnState::No; 11];
        purview[EPurview::Exit as usize] = EPurviewOwnState::Permit;
        purview[EPurview::LeaveWord as usize] = EPurviewOwnState::Permit;

        let (name, member_level, occupation, region) =
            if let Some(player) = game.online_player_by_id(applicant_id as u32) {
                let name = fixed_do_join_string::<FACTION_MEMBER_NAME_CAPACITY, Context::Block>(
                    FactionDoJoinStringField::OnlinePlayerName,
                    player.get_name(),
                    true,
                    false,
                )?;
                let region_source = match game.region_name(player.get_region_id()) {
                    WorldRegionNameLookup::RegionNotFound
                    | WorldRegionNameLookup::NullRegionPointer => &[][..],
                    WorldRegionNameLookup::Name(name) => name,
                };
                let region = fixed_do_join_string::<FACTION_MEMBER_TEXT_CAPACITY, Context::Block>(
                    FactionDoJoinStringField::OnlineRegionName,
                    region_source,
                    true,
                    false,
                )?;
                (
                    name,
                    i32::from(player.get_level()),
                    i32::from(player.get_occupation()),
                    region,
                )
            } else {
                let mut name = [0; FACTION_MEMBER_NAME_CAPACITY];
                name[..APPLY_PERSON_NAME_CAPACITY].copy_from_slice(&apply_person.name);
                (
                    name,
                    apply_person.level,
                    apply_person.occupation,
                    [0; FACTION_MEMBER_TEXT_CAPACITY],
                )
            };

        let member = TagMemInfo::from_complete_fields(
            applicant_id,
            name,
            member_level,
            occupation,
            99,
            title,
            purview,
            region,
            join_time,
            false,
        );
        let member_name = member
            .name_wire_bytes()
            .expect("online и offline join-name ограничены до создания member");
        let member_name = member_name[..member_name.len() - 1].to_vec();

        let joined_notice = context.format_world_string(b"WS0170", &[&member_name]);
        let joined_notice = legacy_c_string_visible_bytes(&joined_notice);
        let joined_second_text = context.world_string(b"WS0119").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            joined_notice,
            legacy_c_string_visible_bytes(&joined_second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let applicant_notice =
            context.format_world_string(b"WS0171", &[legacy_c_string_visible_bytes(&self.name)]);
        let applicant_notice = legacy_c_string_visible_bytes(&applicant_notice);
        let applicant_second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: applicant_id,
            first_text: applicant_notice,
            second_text: legacy_c_string_visible_bytes(&applicant_second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });

        self.members.insert(applicant_id, member);
        let refreshed_player_ids =
            self.update_player_faction_info(game, applicant_id, |_view, faction, player_id| {
                context.update_player_faction_info(faction, player_id);
            });
        let add_faction_to_client_result = context
            .add_faction_to_client_by_player_id(self, applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::AddFactionToClient,
                source,
                application_removed: true,
                member_inserted: true,
            })?;
        let add_all_faction_info_result = context
            .add_all_faction_info_to_client_by_player_id(self, applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::AddAllFactionInfoToClient,
                source,
                application_removed: true,
                member_inserted: true,
            })?;
        let member_update = self.update_member_info_to_client(game, applicant_id, EOperator::Add);
        self.set_change_data(2);

        let delete_remain_time =
            self.delete_remain_time
                .ok_or(FactionDoJoinBlock::MissingDeleteRemainTime {
                    member_inserted: true,
                })?;
        let disband_countdown_cancelled = parameters.disband_faction_minimum_members() as u32
            <= self.members.len() as u32
            && delete_remain_time >= 0;
        if disband_countdown_cancelled {
            self.delete_remain_time = Some(-1);
        }

        let log_written = context.faction_join_log_enabled();
        if log_written {
            let manager =
                self.members
                    .get(&manager_id)
                    .ok_or(FactionDoJoinBlock::ManagerMemberMissing {
                        manager_id,
                        member_inserted: true,
                    })?;
            let manager_name = manager.name_wire_bytes().map_err(|_| {
                FactionDoJoinBlock::UnterminatedManagerName {
                    manager_id,
                    member_inserted: true,
                }
            })?;
            context.write_faction_join_log(
                applicant_id,
                &member_name,
                manager_id,
                &manager_name[..manager_name.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                0,
            );
        }

        Ok(FactionDoJoinOutcome::Joined {
            application_removal,
            member_information,
            refreshed_player_ids,
            add_faction_to_client_result,
            add_all_faction_info_result,
            member_update,
            disband_countdown_cancelled,
            log_written,
        })
    }

    pub fn update_leave_word_to_client(
        &self,
        game: &dyn WorldGameView,
        leave_word_id: i32,
        operator: EOperator,
    ) -> Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError> {
        let leave_word = if operator == EOperator::Delete {
            None
        } else {
            self.leave_words.back()
        };

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(LEAVE_WORD_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            if operator != EOperator::Delete {
                let Some(leave_word) = leave_word else {
                    return Err(FactionLeaveWordUpdateBuildError::MissingLastLeaveWord);
                };
                message.base_mut().add_long(operator.wire_value());
                message.base_mut().add_long(leave_word.id);
                message.base_mut().add_long(leave_word.player_id);
                let name = match leave_word.name_wire_bytes() {
                    Ok(name) => name,
                    Err(source) => {
                        return Err(FactionLeaveWordUpdateBuildError::Recipient {
                            source,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                };
                message.base_mut().add(name);
                let content = match leave_word.content_wire_bytes() {
                    Ok(content) => content,
                    Err(source) => {
                        return Err(FactionLeaveWordUpdateBuildError::Recipient {
                            source,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                };
                message.base_mut().add(content);
                message.base_mut().add(&leave_word.time.wire_bytes());
            } else {
                message.base_mut().add_long(EOperator::Delete.wire_value());
                message.base_mut().add_long(leave_word_id);
            }
            deliveries.push(FactionLeaveWordDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn edit_leave_word(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        leave_word_id: i32,
        _operator: EOperator,
    ) -> Result<FactionEditLeaveWordOutcome, FactionInitialPropertyBlock> {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        if !property.leave_word_function() {
            return Ok(FactionEditLeaveWordOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::EditLeaveWord as i32) {
            return Ok(FactionEditLeaveWordOutcome::PermissionDenied);
        }
        let Some(position) = self
            .leave_words
            .iter()
            .position(|leave_word| leave_word.id == leave_word_id)
        else {
            return Ok(FactionEditLeaveWordOutcome::LeaveWordNotFound);
        };

        self.leave_words.remove(position);
        let deliveries =
            self.update_leave_word_to_client(game, leave_word_id, EOperator::Delete);
        self.set_change_data(4);
        Ok(FactionEditLeaveWordOutcome::Deleted { deliveries })
    }

    pub fn leave_word(
        &mut self,
        game: &mut dyn WorldGameView,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<FactionLeaveWordOutcome, FactionLeaveWordBlock> {
        let property = self
            .base_property
            .ok_or(FactionLeaveWordBlock::MissingBaseProperty)?;
        if !property.leave_word_function() {
            return Ok(FactionLeaveWordOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::LeaveWord as i32) {
            return Ok(FactionLeaveWordOutcome::PermissionDenied);
        }

        let input_truncated = content.len() > LEAVE_WORD_CONTENT_LIMIT;
        content.truncate(LEAVE_WORD_CONTENT_LIMIT);
        let visible_content = legacy_c_string_visible_bytes(content);
        let leave_word_id = game.allocate_leave_word_id();

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Err(FactionLeaveWordBlock::OfflineAuthorNameUnknown {
                player_id,
                allocated_leave_word_id: leave_word_id,
                input_truncated,
            });
        };
        let visible_name = legacy_c_string_visible_bytes(player.get_name());
        if visible_name.len() >= LEAVE_WORD_NAME_CAPACITY {
            return Err(FactionLeaveWordBlock::PlayerNameWouldOverflow {
                player_id,
                allocated_leave_word_id: leave_word_id,
                visible_len: visible_name.len(),
                input_truncated,
            });
        }

        let mut name = [0; LEAVE_WORD_NAME_CAPACITY];
        name[..visible_name.len()].copy_from_slice(visible_name);
        let mut stored_content = [0; LEAVE_WORD_CONTENT_CAPACITY];
        stored_content[..visible_content.len()].copy_from_slice(visible_content);
        let leave_word = TagLeaveWord::from_complete_fields(
            leave_word_id,
            player_id,
            name,
            time,
            stored_content,
        );

        let mut evicted_count = 0;
        while self.leave_words.len() >= LEAVE_WORD_LIMIT {
            let _ = self.leave_words.pop_front();
            evicted_count += 1;
        }
        self.leave_words.push_back(leave_word);
        let deliveries =
            self.update_leave_word_to_client(game, leave_word_id, EOperator::Add);
        self.set_change_data(4);
        Ok(FactionLeaveWordOutcome::Published {
            leave_word_id,
            input_truncated,
            evicted_count,
            deliveries,
        })
    }

    pub fn load_leave_word(
        &mut self,
        leave_word: TagLeaveWord,
    ) -> FactionLoadLeaveWordReport {
        let mut evicted_count = 0;
        while self.leave_words.len() >= LEAVE_WORD_LIMIT {
            let _ = self.leave_words.pop_front();
            evicted_count += 1;
        }
        self.leave_words.push_back(leave_word);
        FactionLoadLeaveWordReport { evicted_count }
    }

    pub fn pronounce(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<FactionPronounceOutcome, FactionPronounceBlock> {
        let property = self
            .base_property
            .ok_or(FactionPronounceBlock::MissingBaseProperty)?;
        if !property.pronounce_function() {
            return Ok(FactionPronounceOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::Pronounce as i32) {
            return Ok(FactionPronounceOutcome::PermissionDenied);
        }

        let input_truncated = content.len() > PRONOUNCE_CONTENT_CAPACITY;
        content.truncate(PRONOUNCE_CONTENT_CAPACITY);
        let visible_content = legacy_c_string_visible_bytes(content);
        if visible_content.len() >= PRONOUNCE_CONTENT_CAPACITY {
            return Err(FactionPronounceBlock::ContentWouldOverflow {
                visible_len: visible_content.len(),
                input_truncated,
            });
        }

        let online_player = game.online_player_by_id(player_id as u32);
        let visible_name =
            online_player.map(|player| legacy_c_string_visible_bytes(player.get_name()));
        if let Some(visible_name) = visible_name {
            if visible_name.len() >= PRONOUNCE_NAME_CAPACITY {
                return Err(FactionPronounceBlock::PlayerNameWouldOverflow {
                    player_id,
                    visible_len: visible_name.len(),
                    input_truncated,
                });
            }
        }

        self.pronounce.player_id = player_id;
        self.pronounce.time = time;
        self.pronounce.content[..visible_content.len()].copy_from_slice(visible_content);
        self.pronounce.content[visible_content.len()] = 0;
        if let Some(visible_name) = visible_name {
            self.pronounce.name[..visible_name.len()].copy_from_slice(visible_name);
            self.pronounce.name[visible_name.len()] = 0;
        }

        let deliveries = self.update_pronounce_to_client(game, EOperator::Update);
        self.set_change_data(8);
        Ok(FactionPronounceOutcome::Published {
            input_truncated,
            deliveries,
        })
    }

    pub fn upload_icon<Context>(
        &mut self,
        parameters: &COrganizingParam,
        player_id: i32,
        _time: &TagTimeValue,
        context: &mut Context,
    ) -> Result<FactionUploadIconOutcome, FactionUploadIconBlock>
    where
        Context: FactionUploadIconContext,
    {
        if self.is_master(player_id) == 0 {
            return Ok(FactionUploadIconOutcome::Rejected {
                reason: FactionUploadIconRejection::PlayerNotMaster,
                notice_sent: false,
            });
        }
        let property = self
            .base_property
            .ok_or(FactionUploadIconBlock::MissingBaseProperty)?;
        if !property.upload_icon_function() {
            send_apply_join_information(context, player_id, b"WS0225", b"WS0119");
            return Ok(FactionUploadIconOutcome::Rejected {
                reason: FactionUploadIconRejection::FunctionDisabled,
                notice_sent: true,
            });
        }

        let interval_minutes = parameters.upload_icon_interval_minutes();
        if interval_minutes < 1 {
            self.set_change_data(8);
            return Ok(FactionUploadIconOutcome::Accepted { dirty_set: true });
        }

        let notice = context.format_upload_icon_interval(b"WS0226", interval_minutes);
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: notice,
            second_text: legacy_c_string_visible_bytes(&second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });
        Ok(FactionUploadIconOutcome::Rejected {
            reason: FactionUploadIconRejection::IntervalActive,
            notice_sent: true,
        })
    }

    pub fn disband<Context>(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionDisbandOutcome, FactionDisbandBlock>
    where
        Context: FactionDisbandContext,
    {
        if !self.check_operator_validate(player_id, EPurview::Disband as i32) {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::OperatorNotPermitted,
                notice_sent: false,
            });
        }
        let superior_organizing = self
            .superior_organizing()
            .ok_or(FactionDisbandBlock::MissingBaseProperty)?;
        if superior_organizing > 0 {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::HasSuperiorOrganizing,
                notice_sent: false,
            });
        }
        if self.has_enemy_faction() || context.village_war_declared(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0189", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::StandardWar,
                notice_sent: true,
            });
        }
        if self.has_city_war_enemy_faction() || context.city_war_declared(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0190", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::CityWar,
                notice_sent: true,
            });
        }
        if context.goods_war_blocks_disband(self.faction_id, player_id) {
            send_apply_join_information(context, player_id, b"ws0362", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::GoodsWar,
                notice_sent: true,
            });
        }

        let country = self
            .country()
            .ok_or(FactionDisbandBlock::MissingBaseProperty)?;
        let king_id = context
            .country_king_id(country)
            .ok_or(FactionDisbandBlock::CountryMissing { country })?;
        if king_id == player_id {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::CountryKing,
                notice_sent: false,
            });
        }

        let mut progress = FactionDisbandProgress {
            goods_war_members_deleted: false,
            goods_war_faction_count_decremented: false,
            cleared_apply_persons: 0,
            cleared_leave_words: 0,
            delete_organizing: None,
        };
        context.delete_goods_war_members_by_faction_id(self.faction_id);
        progress.goods_war_members_deleted = true;
        context.decrement_goods_war_faction_count(self.faction_id, self.name());
        progress.goods_war_faction_count_decremented = true;

        progress.cleared_apply_persons = self.apply_persons.len();
        self.apply_persons.clear();
        progress.cleared_leave_words = self.leave_words.len();
        self.leave_words.clear();
        progress.delete_organizing =
            Some(match self.delete_organizing_to_client(game, 0, context) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(FactionDisbandBlock::DeleteOrganizing { source, progress });
                }
            });
        Ok(FactionDisbandOutcome::Disbanded(progress))
    }

    pub fn demise<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        old_master_id: i32,
        new_master_id: i32,
        context: &mut Context,
    ) -> Result<FactionDemiseOutcome, FactionDemiseBlock>
    where
        Context: FactionDemiseContext,
    {
        if old_master_id == new_master_id {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::SamePlayer,
                notice_sent: false,
            });
        }
        if self.is_master(old_master_id) == 0 {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerNotMaster,
                notice_sent: false,
            });
        }
        if self.is_member(new_master_id) == 0 {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerNotMember,
                notice_sent: false,
            });
        }
        if self.has_enemy_faction() || context.attack_city_system_declared(self.faction_id) {
            send_apply_join_information(context, old_master_id, b"WS0213", b"WS0119");
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::StandardWar,
                notice_sent: true,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.attack_city_system_declared(self.faction_id)
        {
            send_apply_join_information(context, old_master_id, b"WS0214", b"WS0119");
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::CityWar,
                notice_sent: true,
            });
        }
        if context.goods_war_blocks_demise(self.faction_id, old_master_id) {
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: b"???",
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::GoodsWar,
                notice_sent: true,
            });
        }

        let old_player = game.online_player_by_id(old_master_id as u32);
        let new_player = game.online_player_by_id(new_master_id as u32);
        let Some(old_player) = old_player else {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerOffline,
                notice_sent: false,
            });
        };
        let Some(new_player) = new_player else {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerOffline,
                notice_sent: false,
            });
        };
        if new_player.faction_war_operator() {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerOperatingFactionWar,
                notice_sent: false,
            });
        }
        if old_player.faction_war_operator() {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerOperatingFactionWar,
                notice_sent: false,
            });
        }

        let required_level = parameters.create_faction_player_level();
        if i32::from(new_player.get_level()) < required_level {
            let notice = context.format_demise_signed(b"WS0215", required_level);
            let notice = bounded_demise_notice(&notice)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::RequiredLevel,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewMasterLevelTooLow,
                notice_sent: true,
            });
        }

        let permit_demise = self
            .permit_demise
            .ok_or(FactionDemiseBlock::PermitDemiseUnknown)?;
        if !permit_demise {
            let notice = context.world_string(b"WS0216").unwrap_or_default();
            let notice = bounded_demise_notice(&notice)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::DemiseForbidden,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::DemiseForbidden,
                notice_sent: true,
            });
        }

        let country = self
            .country()
            .ok_or(FactionDemiseBlock::MissingBaseProperty)?;
        if context.country_blocks_demise(country, old_master_id) {
            let notice = context.format_demise_signed(b"WS0217", required_level);
            let notice = bounded_demise_notice(&notice)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::CountryKingBlocked,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::CountryKingBlocked,
                notice_sent: true,
            });
        }

        if !self.members.contains_key(&old_master_id) {
            return Err(FactionDemiseBlock::OldMasterMemberMissing {
                player_id: old_master_id,
            });
        }

        let mut progress = empty_faction_demise_progress();
        self.permit_demise = Some(false);
        progress.permit_demise_disabled = true;
        self.master_id = Some(new_master_id);
        progress.master_changed = true;

        let new_title_source = context.world_string(b"WS0157").unwrap_or_default();
        let new_title = match fixed_demise_title(&new_title_source) {
            Ok(title) => title,
            Err(visible_len) => {
                return Err(FactionDemiseBlock::TitleWouldOverflow {
                    member: FactionDemiseMember::NewMaster,
                    string_id: b"WS0157",
                    visible_len,
                    progress,
                });
            }
        };
        let new_member = self
            .members
            .get_mut(&new_master_id)
            .expect("IsMember(new) и Demise не удаляют member");
        new_member.title = new_title;
        new_member.job_level = 1;
        new_member.id = new_master_id;
        new_member.purview = demise_new_master_purview();
        progress.new_member_changed = true;

        let old_title_source = context.world_string(b"WS0218").unwrap_or_default();
        let old_title = match fixed_demise_title(&old_title_source) {
            Ok(title) => title,
            Err(visible_len) => {
                return Err(FactionDemiseBlock::TitleWouldOverflow {
                    member: FactionDemiseMember::OldMaster,
                    string_id: b"WS0218",
                    visible_len,
                    progress,
                });
            }
        };
        let old_member = self
            .members
            .get_mut(&old_master_id)
            .expect("old-master invariant проверен до leadership mutation");
        old_member.id = old_master_id;
        old_member.job_level = 99;
        old_member.title = old_title;
        old_member.purview = demise_old_master_purview();
        progress.old_member_changed = true;

        progress.old_member_update = Some(match self.update_member_info_to_client(
            game,
            old_master_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionDemiseBlock::MemberUpdate {
                    member: FactionDemiseMember::OldMaster,
                    source,
                    progress,
                });
            }
        });
        progress.new_member_update = Some(match self.update_member_info_to_client(
            game,
            new_master_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionDemiseBlock::MemberUpdate {
                    member: FactionDemiseMember::NewMaster,
                    source,
                    progress,
                });
            }
        });
        self.set_change_data(1);
        progress.base_dirty_set = true;
        self.set_change_data(2);
        progress.members_dirty_set = true;
        progress.refreshed_player_ids = self.update_player_faction_info(game, 0, |_view, faction, player_id| {
            context.update_player_faction_info(faction, player_id);
        });

        let old_master_name = match self
            .members
            .get(&old_master_id)
            .expect("old-master member сохранён")
            .name_wire_bytes()
        {
            Ok(name) => name[..name.len() - 1].to_vec(),
            Err(source) => {
                return Err(FactionDemiseBlock::UnterminatedMemberName {
                    member: FactionDemiseMember::OldMaster,
                    player_id: old_master_id,
                    source,
                    progress,
                });
            }
        };
        let new_master_name = match self
            .members
            .get(&new_master_id)
            .expect("new-master member сохранён")
            .name_wire_bytes()
        {
            Ok(name) => name[..name.len() - 1].to_vec(),
            Err(source) => {
                return Err(FactionDemiseBlock::UnterminatedMemberName {
                    member: FactionDemiseMember::NewMaster,
                    player_id: new_master_id,
                    source,
                    progress,
                });
            }
        };
        let notice = context.format_demise_change(
            b"WS0219",
            &old_master_name,
            &new_master_name,
        );
        let notice = match bounded_demise_notice(&notice) {
            Ok(notice) => notice,
            Err(formatted_len) => {
                return Err(FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::Success,
                    formatted_len,
                    progress,
                });
            }
        };
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        // Декомпилятор показывает у этого вызова `K=0x87a238`, но в теле
        // хелпера `0x4B4890` (`SendInfoToAllMember(…, -1, K)`) аргумент
        // цвета мёртв — жёсткий push константы `0xFFDAEDFE`; в кадр
        // `0x7F804` всегда уходит `0xFFDAEDFE`. Машинный факт: тело
        // `0x4B4890` в `Nworldserver.exe` + `WorldServer.pdb`.
        // См. `send_info_to_all_members_with_color`.
        progress.member_information = Some(self.send_info_to_all_members_with_color(
            &notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            0xFFDA_EDFE,
            |request| context.send_organizing_info(request),
        ));

        if context.faction_master_log_enabled() {
            context.write_faction_master_log(
                old_master_id,
                &old_master_name,
                new_master_id,
                &new_master_name,
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
            );
            progress.log_written = true;
        }

        Ok(FactionDemiseOutcome::Transferred(progress))
    }

    pub fn operator_tax<Context>(
        &self,
        player_id: i32,
        region_id: i32,
        context: &Context,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<Context::Block>>
    where
        Context: FactionOperationAuthorityContext,
    {
        let union_id = context
            .union_id_for_faction(self.faction_id)
            .map_err(FactionOperationBlock::UnionMembershipLookup)?;
        if union_id > 0
            && let Some(master_faction_id) = context.union_master_faction_id(union_id)
            && master_faction_id != self.faction_id
        {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::UnionMasterMismatch {
                    union_id,
                    master_faction_id,
                },
            ));
        }
        self.authorize_owned_city_operation(player_id, region_id, EPurview::ObtainTax)
    }

    pub fn operator_city_gate<Context>(
        &self,
        player_id: i32,
        region_id: i32,
        context: &Context,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<Context::Block>>
    where
        Context: FactionOperationAuthorityContext,
    {
        let union_id = self
            .superior_organizing()
            .ok_or(FactionOperationBlock::MissingBaseProperty)?;
        if union_id > 0
            && let Some(master_faction_id) = context.union_master_faction_id(union_id)
            && master_faction_id != self.faction_id
        {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::UnionMasterMismatch {
                    union_id,
                    master_faction_id,
                },
            ));
        }
        self.authorize_owned_city_operation(player_id, region_id, EPurview::OperCityGate)
    }

    fn authorize_owned_city_operation<ContextBlock>(
        &self,
        player_id: i32,
        region_id: i32,
        purview: EPurview,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<ContextBlock>> {
        if self.is_owned_city(region_id) == 0 {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::RegionNotOwned { region_id },
            ));
        }
        if !self.check_operator_validate(player_id, purview as i32) {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::PlayerNotPermitted { player_id, purview },
            ));
        }
        Ok(FactionOperationOutcome::Authorized)
    }

    pub fn send_info_to_all_members<'a, F>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        send_organizing_info: F,
    ) -> FactionMemberInfoReport
    where
        F: FnMut(FactionMemberInfoRequest<'a>),
    {
        self.send_info_to_all_members_with_color(
            first_text,
            second_text,
            information_type,
            0xFFDA_EDFE,
            send_organizing_info,
        )
    }

    /// Рассылка info-notice всем членам фракции.
    ///
    /// Wire-цвет всех таких рассылок — `0xFFDAEDFE`. Аргумент цвета у
    /// оригинального хелпера `SendInfoToAllMember` (`0x4B4890`) мёртв: тело
    /// жёстко пушит константу `0xFFDAEDFE`, а передаваемый у части вызовов
    /// `K=0x87a238` на провод не попадает. Машинный факт по телу `0x4B4890`
    /// в `Nworldserver.exe` + `WorldServer.pdb`.
    pub fn send_info_to_all_members_with_color<'a, F>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        mut send_organizing_info: F,
    ) -> FactionMemberInfoReport
    where
        F: FnMut(FactionMemberInfoRequest<'a>),
    {
        let mut recipient_player_ids = Vec::with_capacity(self.members.len());
        for &recipient_player_id in self.members.keys() {
            send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id,
                first_text,
                second_text,
                information_type,
                color,
                trailing_value: 0,
            });
            recipient_player_ids.push(recipient_player_id);
        }
        FactionMemberInfoReport {
            recipient_player_ids,
        }
    }

    pub fn set_leave_word_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::LeaveWord, enabled, context)
    }

    pub fn set_pronounce_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::Pronounce, enabled, context)
    }

    pub fn set_endue_right_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::EndueRight, enabled, context)
    }

    pub fn set_join_village_war_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::JoinVillageWar, enabled, context)
    }

    pub fn set_join_city_war_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::JoinCityWar, enabled, context)
    }

    pub fn set_create_union_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::CreateUnion, enabled, context)
    }

    fn set_feature_function<Context>(
        &mut self,
        feature: FactionFeatureFunction,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        if property.feature_function(feature) == enabled {
            return Ok(FactionFeatureFunctionUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?
            .set_feature_function(feature, enabled);

        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        let first_text = context
            .world_string(feature.notification_string_id(enabled))
            .unwrap_or_default();
        let report = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&first_text),
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );
        Ok(FactionFeatureFunctionUpdate::Updated(report))
    }

    pub fn update_other_faction_info_to_client(
        &self,
        game: &dyn WorldGameView,
        other_faction_id: i32,
        other_faction_name: &[u8],
        operator: EOperator,
    ) -> Result<Vec<FactionOtherInfoDelivery>, FactionOtherInfoBuildError> {
        let name_wire = legacy_c_string_wire_bytes(other_faction_name);
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(OTHER_FACTION_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(other_faction_id);
            message.base_mut().add(&name_wire);
            deliveries.push(FactionOtherInfoDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    pub fn talk(
        &self,
        game: &dyn WorldGameView,
        speaker_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    ) -> Vec<FactionTalkDelivery> {
        let first_text = legacy_c_string_wire_bytes(first_text);
        let second_text = legacy_c_string_wire_bytes(second_text);
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(FACTION_TALK_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(FACTION_TALK_CHANNEL);
            message.base_mut().add_long(speaker_id);
            message.base_mut().add(&first_text);
            message.base_mut().add(&second_text);
            deliveries.push(FactionTalkDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        deliveries
    }

    pub fn delete_organizing_to_client<Context>(
        &self,
        game: &dyn WorldGameView,
        target_player_id: i32,
        context: &mut Context,
    ) -> Result<FactionDeleteOrganizingOutcome, FactionDeleteOrganizingBuildError>
    where
        Context: FactionOrganizingInfoContext,
    {
        if target_player_id > 0 {
            let player = game.online_player_by_id(target_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(target_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                return Ok(FactionDeleteOrganizingOutcome::SingleTarget { delivery: None });
            }

            let mut message = CMessage::new(DELETE_ORGANIZING_MESSAGE_TYPE);
            message.base_mut().add_long(target_player_id);
            message.base_mut().add_long(self.faction_id);
            return Ok(FactionDeleteOrganizingOutcome::SingleTarget {
                delivery: Some(FactionDeleteOrganizingDelivery {
                    recipient_player_id: target_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                }),
            });
        }

        let first_text = context.world_string(b"WS0191").unwrap_or_default();
        let first_text = legacy_c_string_visible_bytes(&first_text);
        let first_text = first_text.to_vec();

        let mut deliveries = Vec::new();
        let mut information_recipient_ids = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none() || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(DELETE_ORGANIZING_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(self.faction_id);
            deliveries.push(FactionDeleteOrganizingDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });

            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id,
                first_text: &first_text,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: game_server_id,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            information_recipient_ids.push(recipient_player_id);
        }

        Ok(FactionDeleteOrganizingOutcome::Broadcast {
            deliveries,
            information_recipient_ids,
        })
    }

    pub fn set_experience(
        &mut self,
        game: &dyn WorldGameView,
        experience: i32,
    ) -> Result<FactionExperienceUpdate, FactionExperienceBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        if property.level() >= 12 {
            return Ok(FactionExperienceUpdate::MaximumLevel);
        }
        let experience = experience.clamp(0, 100_000_000);
        if property.experience() == experience {
            return Ok(FactionExperienceUpdate::Unchanged { experience });
        }
        property.write_signed(0x04, experience);
        self.set_change_data(1);
        let deliveries = self.update_experience_to_client(game)?;
        Ok(FactionExperienceUpdate::Updated {
            experience,
            deliveries,
        })
    }

    pub const fn superior_organizing(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.union_id()),
            None => None,
        }
    }

    pub fn set_union_master_projection(&mut self, union_master_id: i32) {
        self.union_master_id = union_master_id;
    }

    pub fn player_header<Context>(
        &self,
        context: &Context,
    ) -> Result<i32, FactionPlayerHeaderBlock<Context::Block>>
    where
        Context: FactionPlayerHeaderContext,
    {
        let property = self
            .base_property
            .ok_or(FactionPlayerHeaderBlock::MissingBaseProperty)?;
        if property.union_id() > 0 {
            if let Some(player_header) = context
                .union_player_header(property.union_id())
                .map_err(FactionPlayerHeaderBlock::Context)?
            {
                return Ok(player_header);
            }
        }
        self.master_id
            .ok_or(FactionPlayerHeaderBlock::MissingMasterId)
    }

    pub fn set_superior_organizing(
        &mut self,
        organizing_id: i32,
        union_master_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<(), FactionSuperiorOrganizingBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionSuperiorOrganizingBlock::MissingBaseProperty)?;
        if property.union_id() != organizing_id {
            property.write_signed(0x18, organizing_id);
        }
        self.union_master_id = if organizing_id > 0 {
            union_master_id
        } else {
            0
        };

        if organizing_id < 1 {
            let member_count = self.members.len() as u32;
            let minimum_members = parameters.disband_faction_minimum_members() as u32;
            if member_count < minimum_members {
                let delete_remain_time = self
                    .delete_remain_time
                    .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
                if delete_remain_time < 0 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                }
            }
        } else {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time > 0 {
                self.delete_remain_time = Some(-1);
            }
        }
        Ok(())
    }

    pub fn del_member(
        &mut self,
        player_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<Option<FactionDelMemberReport>, FactionDelMemberBlock> {
        let master_id = self.master_id.ok_or(FactionDelMemberBlock::MasterIdMissing)?;
        if player_id == master_id {
            return Ok(None);
        }

        let removed = self.members.remove(&player_id).is_some();
        let member_count = self.members.len() as u32;
        let minimum_members = parameters.disband_faction_minimum_members() as u32;
        let mut disband_countdown_started = false;
        if member_count < minimum_members {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionDelMemberBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time < 0 {
                let union_id = self
                    .base_property
                    .ok_or(FactionDelMemberBlock::MissingBaseProperty)?
                    .union_id();
                if union_id < 1 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                    disband_countdown_started = true;
                }
            }
        }

        Ok(Some(FactionDelMemberReport {
            removed,
            disband_countdown_started,
        }))
    }

    pub fn member_title(&self, player_id: i32) -> Result<Vec<u8>, UnterminatedMemberField> {
        let Some(member) = self.members.get(&player_id) else {
            return Ok(Vec::new());
        };
        let wire = member.title_wire_bytes()?;
        Ok(wire[..wire.len() - 1].to_vec())
    }

    pub fn is_contribute(&self, player_id: i32) -> bool {
        self.members
            .get(&player_id)
            .is_some_and(|member| member.contribute)
    }

    pub fn contributor_count(&self) -> i32 {
        self.members.values().fold(0i32, |count, member| {
            if member.contribute {
                count.wrapping_add(1)
            } else {
                count
            }
        })
    }

    pub fn set_contributor<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        requester_id: i32,
        target_id: i32,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionContributorOutcome, FactionContributorBlock>
    where
        Context: FactionContributorContext,
    {
        if self.is_master(requester_id) == 0 {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::RequesterNotMaster,
            ));
        }
        if enabled && parameters.maximum_contributors() <= self.contributor_count() {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::MaximumContributors,
            ));
        }
        let Some(target) = self.members.get_mut(&target_id) else {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::TargetNotMember,
            ));
        };
        if target.contribute == enabled {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::Unchanged,
            ));
        }

        target.contribute = enabled;
        let mut progress = FactionContributorProgress {
            contributor_changed: true,
            member_update: None,
            refreshed_player_ids: Vec::new(),
            member_information: None,
            dirty_set: false,
        };
        progress.member_update = Some(match self.update_member_info_to_client(
            game,
            target_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionContributorBlock::MemberUpdate { source, progress });
            }
        });
        progress.refreshed_player_ids =
            self.update_player_faction_info(game, target_id, |_view, faction, player_id| {
                context.update_player_faction_info(faction, player_id);
            });

        let target = self
            .members
            .get(&target_id)
            .expect("успешный target lookup и callbacks не удаляют member");
        let target_name_wire = match target.name_wire_bytes() {
            Ok(name) => name,
            Err(source) => {
                return Err(FactionContributorBlock::UnterminatedMemberName {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let notice = context.format_contributor_string(
            if enabled { b"WS0227" } else { b"WS0228" },
            &target_name_wire[..target_name_wire.len() - 1],
        );
        let notice = legacy_c_string_visible_bytes(&notice);
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        // Цвет тот же `0xFFDAEDFE`, что и у `Demise`: декомпиляторский
        // `K=0x87a238` мёртв в теле хелпера `0x4B4890`, см. машинный факт
        // у `send_info_to_all_members_with_color`.
        progress.member_information = Some(self.send_info_to_all_members_with_color(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            0xFFDA_EDFE,
            |request| context.send_organizing_info(request),
        ));
        self.set_change_data(2);
        progress.dirty_set = true;

        Ok(FactionContributorOutcome::Updated(progress))
    }

    pub fn member_job_level(&self, player_id: i32) -> u16 {
        self.members
            .get(&player_id)
            .map_or(0, |member| member.job_level as u16)
    }

    pub fn is_using_purview(&self, player_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        self.members
            .get(&player_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    pub fn set_member_purview(
        &mut self,
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state != EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::Permit;
        MemberPurviewMutation::Changed
    }

    pub fn abolish_member_purview(
        &mut self,
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    pub fn check_operator_validate(&self, requester_id: i32, purview: i32) -> bool {
        self.is_member(requester_id) != 0 && self.is_using_purview(requester_id, purview)
    }

    pub fn check_operator_validate_target(
        &self,
        requester_id: i32,
        target_id: i32,
        purview: i32,
    ) -> Result<bool, FactionOperatorValidationBlock> {
        let master_id = self.master_id.ok_or(FactionOperatorValidationBlock)?;
        if requester_id == target_id
            || target_id == master_id
            || self.is_member(requester_id) == 0
            || self.is_member(target_id) == 0
            || !self.is_using_purview(requester_id, purview)
        {
            return Ok(false);
        }
        if self.is_using_purview(target_id, purview) && requester_id != master_id {
            return Ok(false);
        }
        Ok(true)
    }

    pub const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }

    pub fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    pub fn clone_save_data(&self) -> Result<Option<Self>, FactionCloneSaveBlock> {
        let change_data_type = self.change_data_type;
        if change_data_type == 0 {
            return Ok(None);
        }

        let copy_property = change_data_type & 1 != 0;
        let master_id = if copy_property {
            Some(
                self.master_id
                    .ok_or(FactionCloneSaveBlock::MasterIdMissing)?,
            )
        } else {
            None
        };
        let base_property = if copy_property {
            Some(
                self.base_property
                    .ok_or(FactionCloneSaveBlock::MissingBaseProperty)?,
            )
        } else {
            None
        };
        let established_time = if copy_property {
            Some(
                self.established_time
                    .ok_or(FactionCloneSaveBlock::EstablishedTimeUnknown)?,
            )
        } else {
            None
        };
        let delete_remain_time = if copy_property {
            Some(
                self.delete_remain_time
                    .ok_or(FactionCloneSaveBlock::DeleteRemainTimeAbsent)?,
            )
        } else {
            None
        };

        Ok(Some(Self {
            faction_id: self.faction_id,
            name: if copy_property {
                self.name.clone()
            } else {
                Vec::new()
            },
            master_id,
            members: if change_data_type & 2 != 0 {
                self.members.clone()
            } else {
                BTreeMap::new()
            },
            base_property,
            established_time,
            delete_remain_time,
            owned_cities: if change_data_type & 8 != 0 {
                self.owned_cities.clone()
            } else {
                VecDeque::new()
            },
 // CloneSaveData не копирует оба runtime enemy-set.
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
 // Private clone-constructor не назначает transient bool.
            permit_demise: None,
            enemy_factions_changed: None,
            city_war_enemy_factions_changed: None,
 // DB ability-owner читает ordered keys; wire-owner подтверждает
 // и сохраняемый вместе с ними полный tagApplyPerson value.
            apply_persons: if change_data_type & 8 != 0 {
                self.apply_persons.clone()
            } else {
                BTreeMap::new()
            },
            pronounce: if change_data_type & 8 != 0 {
                self.pronounce
            } else {
                TagPronounceWord::ZERO
            },
            leave_words: if change_data_type & 4 != 0 {
                self.leave_words.clone()
            } else {
                VecDeque::new()
            },
            last_upload_icon_time: if change_data_type & 8 != 0 {
                self.last_upload_icon_time
            } else {
                ZERO_TIME
            },
            icon_data: if change_data_type & 8 != 0 {
                self.icon_data.clone()
            } else {
                Vec::new()
            },
            change_data_type,
 // Private clone-constructor не назначает эти поля; текущие
 // безопасные значения остаются только до связанный canonical
 // нормализации SaveFactionProperty.
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
            union_master_id: if copy_property {
                self.union_master_id
            } else {
                0
            },
        }))
    }

    pub fn set_goods_war_count(&mut self, goods_war_count: i32) -> i32 {
        let now = Local::now();
        self.goods_war_last_win_time = format!(
            "{}-{}-{} {}:{}:{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        self.set_change_data(1);
        self.goods_war_count = if goods_war_count <= 0 {
            0
        } else {
            goods_war_count
        };
        self.goods_war_count
    }

    pub const fn goods_war_count(&self) -> i32 {
        self.goods_war_count
    }

    pub fn goods_war_last_win_time(&self) -> &str {
        &self.goods_war_last_win_time
    }

    pub fn set_goods_war_last_win_time(&mut self, value: String) {
        self.goods_war_last_win_time = value;
    }

    pub const fn get_members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    pub fn get_organizing_member_list<T>(&self, output: &mut VecDeque<T>) {
        output.clear();
    }

    pub fn get_member_id_list(&self, output: &mut VecDeque<i32>) {
        output.clear();
        output.extend(self.members.keys().copied());
    }

    pub const fn get_apply_persons(&self) -> &BTreeMap<i32, TagApplyPerson> {
        &self.apply_persons
    }

    pub fn get_apply_person_ids(&self) -> impl Iterator<Item = &i32> {
        self.apply_persons.keys()
    }

    pub fn is_in_apply_members(&self, player_id: i32) -> i32 {
        if self.apply_persons.contains_key(&player_id) {
            self.faction_id
        } else {
            0
        }
    }

    pub fn clear_apply_list(&mut self, operator_id: i32) -> bool {
        if !self.is_using_purview(operator_id, EPurview::ConMem as i32) {
            return false;
        }
        self.apply_persons.clear();
        true
    }

    pub fn get_pronounce_data(&self, output: &mut Vec<u8>) -> bool {
        self.pronounce.append_wire_bytes(output);
        true
    }

    pub const fn get_leave_words(&self) -> &VecDeque<TagLeaveWord> {
        &self.leave_words
    }

    pub const fn last_upload_icon_time(&self) -> TagTimeValue {
        self.last_upload_icon_time
    }

    pub fn icon_data(&self) -> &[u8] {
        &self.icon_data
    }

    pub fn get_member_num(&self) -> i32 {
        self.members.len() as u32 as i32
    }

    pub fn is_member(&self, player_id: i32) -> i32 {
        if self.members.contains_key(&player_id) {
            self.faction_id
        } else {
            0
        }
    }

 /// Дописывает точный полный faction snapshot virtual slot `+0x0C`.
 ///
 /// Title master-а исходник копировал только при visible длине не более
 /// двадцати байт; более длинный title оставлял уже обнулённый `char[256]`.
    pub fn add_full_snapshot_to_byte_array(
        &self,
        game: &dyn WorldGameView,
        output: &mut Vec<u8>,
    ) -> Result<bool, FactionFullSnapshotBlock> {
        append_i32(output, self.faction_id);
        output.extend_from_slice(&legacy_c_string_wire_bytes(&self.name));

        let master_id = self
            .master_id
            .ok_or(FactionFullSnapshotBlock::MasterIdMissing)?;
        append_i32(output, master_id);
        let master_title = if let Some(master) = self.members.get(&master_id) {
            let wire = master
                .title_wire_bytes()
                .map_err(FactionFullSnapshotBlock::MasterTitle)?;
            (wire.len() - 1 <= 20).then_some(wire)
        } else {
            None
        };
        output.extend_from_slice(master_title.unwrap_or(&[0]));

        let established_time = self
            .established_time
            .ok_or(FactionFullSnapshotBlock::EstablishedTimeUnknown)?;
        output.extend_from_slice(&established_time.wire_bytes());
        let property = self
            .base_property
            .as_ref()
            .ok_or(FactionFullSnapshotBlock::MissingBaseProperty)?;
        output.extend_from_slice(property.wire_bytes());

        append_i32(output, self.pronounce.player_id);
        output.extend_from_slice(&self.pronounce.time.wire_bytes());
        output.extend_from_slice(
            self.pronounce
                .content_wire_bytes()
                .map_err(FactionFullSnapshotBlock::Pronounce)?,
        );
        output.extend_from_slice(
            self.pronounce
                .name_wire_bytes()
                .map_err(FactionFullSnapshotBlock::Pronounce)?,
        );
        self.add_leave_words_to_byte_array(output)
            .map_err(FactionFullSnapshotBlock::LeaveWords)?;
        self.add_members_to_byte_array(output)
            .map_err(FactionFullSnapshotBlock::Members)?;
        self.add_apply_persons_to_byte_array(output)
            .map_err(FactionFullSnapshotBlock::ApplyPersons)?;
        self.add_owned_cities_to_byte_array(game, output)
            .map_err(FactionFullSnapshotBlock::OwnedCities)?;
        self.add_enemy_factions_to_byte_array(output);
        self.add_city_war_enemy_factions_to_byte_array(output);
        Ok(true)
    }

    pub fn add_members_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedMemberField> {
        output.extend_from_slice(&(self.members.len() as u32).to_le_bytes());
        for member in self.members.values() {
            append_i32(output, member.id);
            append_i32(output, member.job_level);
            output.extend_from_slice(member.title_wire_bytes()?);
            output.extend_from_slice(&member.purview_wire_bytes());
            append_i32(output, member.level);
            append_i32(output, member.occupation);
            output.extend_from_slice(member.name_wire_bytes()?);
            output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
            output.extend_from_slice(member.region_wire_bytes()?);
            output.extend_from_slice(&member.last_online_wire_bytes());
        }
        Ok(true)
    }

    pub fn add_apply_persons_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedApplyPersonName> {
        output.extend_from_slice(&(self.apply_persons.len() as u32).to_le_bytes());
        for (completed_persons, person) in self.apply_persons.values().enumerate() {
            append_i32(output, person.id);
            let Some(name) = person.name_wire_bytes() else {
                return Err(UnterminatedApplyPersonName {
                    player_id: person.id,
                    completed_persons,
                });
            };
            output.extend_from_slice(name);
            append_i32(output, person.occupation);
            append_i32(output, person.level);
        }
        Ok(true)
    }

    pub fn add_leave_words_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedLeaveWordField> {
        output.extend_from_slice(&(self.leave_words.len() as u32).to_le_bytes());
        for leave_word in &self.leave_words {
            append_i32(output, leave_word.id);
            append_i32(output, leave_word.player_id);
            output.extend_from_slice(&leave_word.time.wire_bytes());
            output.extend_from_slice(leave_word.content_wire_bytes()?);
            output.extend_from_slice(leave_word.name_wire_bytes()?);
        }
        Ok(true)
    }

 /// Публикует delete либо полный non-delete member-update всем готовым
 /// получателям.
    pub fn update_member_info_to_client(
        &mut self,
        game: &dyn WorldGameView,
        target_player_id: i32,
        operator: EOperator,
    ) -> Result<MemberUpdateReport, MemberUpdateBuildError> {
        let target_found = if operator == EOperator::Delete {
            None
        } else {
            let Some(target) = self.members.get_mut(&target_player_id) else {
                return Ok(MemberUpdateReport {
                    target_found: Some(false),
                    deliveries: Vec::new(),
                });
            };
            target.last_online_time = current_local_member_time();
            Some(true)
        };

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(target_player_id);

            if operator != EOperator::Delete {
                let target = self
                    .members
                    .get(&target_player_id)
                    .expect("non-delete target найден до recipient-прохода");
                let mut fields = Vec::new();
                if let Err(field) = append_member_update_fields(&mut fields, target) {
                    return Err(MemberUpdateBuildError {
                        field,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
                message.base_mut().add(&fields);
            }

            deliveries.push(MemberUpdateDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }

        Ok(MemberUpdateReport {
            target_found,
            deliveries,
        })
    }

    pub fn on_member_level_change(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        level: i32,
    ) -> MemberLevelChangeOutcome {
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberLevelChangeOutcome::MemberNotFound;
        };
        if member.level == level {
            return MemberLevelChangeOutcome::Unchanged;
        }
        member.level = level;
        MemberLevelChangeOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    pub fn on_member_position_change(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        region_id: i32,
    ) -> MemberPositionChangeOutcome {
        if !self.members.contains_key(&player_id) {
            return MemberPositionChangeOutcome::MemberNotFound;
        }

        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => {
                return MemberPositionChangeOutcome::RegionNotFound;
            }
            WorldRegionNameLookup::NullRegionPointer => {
                return MemberPositionChangeOutcome::NullRegionPointer;
            }
            WorldRegionNameLookup::Name(name) => name,
        };
        let member = self
            .members
            .get_mut(&player_id)
            .expect("member проверен до region lookup");
        if region_name.len() >= member.region.len() {
            return MemberPositionChangeOutcome::Blocked(MemberPositionChangeBlocked {
                region_id,
                byte_len: region_name.len(),
            });
        }

        member.region[..region_name.len()].copy_from_slice(region_name);
        member.region[region_name.len()] = 0;
        MemberPositionChangeOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    pub fn on_member_enter_game(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
    ) -> MemberEnterOutcome {
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return MemberEnterOutcome::PlayerNotOnline;
        };
        if !self.members.contains_key(&player_id) {
            return MemberEnterOutcome::MemberNotFound;
        }

        let region_id = player.get_region_id();
        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => &[][..],
            WorldRegionNameLookup::NullRegionPointer => {
 // разыменовывает найденный
 // `tagRegion::pRegion` без проверки на null.
 // Достижимость и наблюдаемая реакция null не определены.
                return MemberEnterOutcome::Blocked(MemberEnterBlockedReason::NullRegionPointer {
                    region_id,
                });
            }
            WorldRegionNameLookup::Name(name) => name,
        };

        let member_region_matches = {
            let member = self
                .members
                .get(&player_id)
                .expect("member проверен до region lookup");
            let member_region = match member.region_wire_bytes() {
                Ok(bytes) => &bytes[..bytes.len() - 1],
                Err(field) => {
 // strcmp читал бы за strRegion[64].
                    return MemberEnterOutcome::Blocked(
                        MemberEnterBlockedReason::UnterminatedMemberRegion(field),
                    );
                }
            };
            member_region == region_name
        };
        if member_region_matches {
            return MemberEnterOutcome::RegionUnchanged;
        }

        let member_region_capacity = self
            .members
            .get(&player_id)
            .expect("member проверен до region lookup")
            .region
            .len();
        if region_name.len() >= member_region_capacity {
 // Второй `strcpy` по переполнял бы
 // `strRegion[64]` уже после исходного неравенства.
            return MemberEnterOutcome::Blocked(
                MemberEnterBlockedReason::RegionNameExceedsMemberField {
                    region_id,
                    byte_len: region_name.len(),
                },
            );
        }

        let member = self
            .members
            .get_mut(&player_id)
            .expect("member проверен до region lookup");
        member.region[..region_name.len()].copy_from_slice(region_name);
        member.region[region_name.len()] = 0;

        MemberEnterOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    pub fn on_member_exit_game(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
    ) -> MemberExitOutcome {
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberExitOutcome::MemberNotFound;
        };

        let exit_time = current_local_member_time();
        if member.region[0] == 0 {
            return MemberExitOutcome::RegionAlreadyEmpty;
        }

        member.region[0] = 0;
        member.last_online_time = exit_time;
        MemberExitOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }
}

pub fn goods_war_check_for_faction_id<F>(
    faction_id: i32,
    is_in_faction_id_list: F,
) -> bool
where
    F: FnOnce(i32) -> bool,
{
    let now = Local::now();
    if now.weekday().num_days_from_sunday() != 6 {
        return false;
    }
    let inside_time_window = match now.hour() {
        19 => now.minute() >= 30,
        20 => true,
        21 => now.minute() <= 10,
        _ => false,
    };
    inside_time_window && is_in_faction_id_list(faction_id)
}

fn append_member_update_fields(
    output: &mut Vec<u8>,
    member: &TagMemInfo,
) -> Result<(), UnterminatedMemberField> {
    output.extend_from_slice(member.name_wire_bytes()?);
    append_i32(output, member.level);
    append_i32(output, member.occupation);
    append_i32(output, member.job_level);
    output.extend_from_slice(member.title_wire_bytes()?);
    output.extend_from_slice(&member.purview_wire_bytes());
    output.extend_from_slice(member.region_wire_bytes()?);
    output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
    output.extend_from_slice(&member.last_online_wire_bytes());
    Ok(())
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn legacy_c_string_visible_bytes(value: &[u8]) -> &[u8] {
    match value.iter().position(|byte| *byte == 0) {
        Some(terminator) => &value[..terminator],
        None => value,
    }
}

fn legacy_c_string_wire_bytes(value: &[u8]) -> Vec<u8> {
    let visible = legacy_c_string_visible_bytes(value);
    let mut wire = Vec::with_capacity(visible.len() + 1);
    wire.extend_from_slice(visible);
    wire.push(0);
    wire
}

fn empty_faction_set_parameter_progress(
    parameter: FactionSetParameterKind,
) -> FactionSetParameterProgress {
    FactionSetParameterProgress {
        parameter,
        level_update: None,
        experience_update: None,
        assigned_experience: None,
        assigned_upgrade_experience: None,
        property_deliveries: None,
        refreshed_player_ids: None,
        dirty_bit_requested: false,
    }
}

fn send_apply_join_information<Context>(
    context: &mut Context,
    recipient_player_id: i32,
    first_string_id: &'static [u8],
    second_string_id: &'static [u8],
) where
    Context: FactionOrganizingInfoContext + ?Sized,
{
    let second_text = context.world_string(second_string_id).unwrap_or_default();
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn ensure_do_join_string_fits<ContextBlock>(
    field: FactionDoJoinStringField,
    visible_len: usize,
    capacity: usize,
    application_removed: bool,
    member_inserted: bool,
) -> Result<(), FactionDoJoinBlock<ContextBlock>> {
    if visible_len >= capacity {
        return Err(FactionDoJoinBlock::StringWouldOverflow {
            field,
            visible_len,
            capacity,
            application_removed,
            member_inserted,
        });
    }
    Ok(())
}

fn faction_purview_notice_string_id(
    change: FactionPurviewChange,
    purview: i32,
) -> &'static [u8] {
    match (change, purview) {
        (FactionPurviewChange::Grant, 2) => b"WS0197",
        (FactionPurviewChange::Grant, 3) => b"WS0198",
        (FactionPurviewChange::Grant, 4) => b"WS0199",
        (FactionPurviewChange::Grant, 5) => b"WS0200",
        (FactionPurviewChange::Grant, 6) => b"WS0201",
        (FactionPurviewChange::Grant, 7) => b"WS0204",
        (FactionPurviewChange::Grant, 8) => b"WS0203",
        (FactionPurviewChange::Grant, 9) => b"WS0202",
        (FactionPurviewChange::Revoke, 2) => b"WS0205",
        (FactionPurviewChange::Revoke, 3) => b"WS0206",
        (FactionPurviewChange::Revoke, 4) => b"WS0207",
        (FactionPurviewChange::Revoke, 5) => b"WS0208",
        (FactionPurviewChange::Revoke, 6) => b"WS0209",
        (FactionPurviewChange::Revoke, 7) => b"WS0212",
        (FactionPurviewChange::Revoke, 8) => b"WS0211",
        (FactionPurviewChange::Revoke, 9) => b"WS0210",
        _ => unreachable!("purview диапазон проверен до выбора notice ID"),
    }
}

fn empty_faction_upgrade_progress() -> FactionUpgradeProgress {
    FactionUpgradeProgress {
        level_update: None,
        experience_update: None,
        charge_delivery: None,
        next_upgrade_experience: None,
        member_information: None,
        property_deliveries: None,
        dirty_set: false,
        refreshed_player_ids: Vec::new(),
        log_written: false,
    }
}

fn empty_faction_demise_progress() -> FactionDemiseProgress {
    FactionDemiseProgress {
        permit_demise_disabled: false,
        master_changed: false,
        new_member_changed: false,
        old_member_changed: false,
        old_member_update: None,
        new_member_update: None,
        base_dirty_set: false,
        members_dirty_set: false,
        refreshed_player_ids: Vec::new(),
        member_information: None,
        log_written: false,
    }
}

fn bounded_demise_notice(value: &[u8]) -> Result<Vec<u8>, usize> {
    let visible = legacy_c_string_visible_bytes(value);
    Ok(visible.to_vec())
}

fn fixed_demise_title(
    value: &[u8],
) -> Result<[u8; FACTION_MEMBER_TEXT_CAPACITY], usize> {
    let visible = legacy_c_string_visible_bytes(value);
    if visible.len() >= FACTION_MEMBER_TEXT_CAPACITY {
        return Err(visible.len());
    }
    let mut title = [0; FACTION_MEMBER_TEXT_CAPACITY];
    title[..visible.len()].copy_from_slice(visible);
    Ok(title)
}

fn demise_new_master_purview() -> [EPurviewOwnState; 11] {
    [
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
    ]
}

fn demise_old_master_purview() -> [EPurviewOwnState; 11] {
    [
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
    ]
}

fn faction_upgrade_notice_string_id(notice: FactionUpgradeNotice) -> &'static [u8] {
    match notice {
        FactionUpgradeNotice::Experience => b"WS0220",
        FactionUpgradeNotice::Money => b"WS0221",
        FactionUpgradeNotice::MasterLevel => b"WS0222",
        FactionUpgradeNotice::Goods => b"WS0223",
        FactionUpgradeNotice::Success => b"WS0224",
    }
}

fn format_faction_upgrade_notice<Context>(
    context: &mut Context,
    notice: FactionUpgradeNotice,
    arguments: &[FactionUpgradeFormatArgument<'_>],
) -> Result<Vec<u8>, usize>
where
    Context: FactionUpgradeContext,
{
    let formatted = context.format_upgrade_string(
        faction_upgrade_notice_string_id(notice),
        arguments,
    );
    let formatted = legacy_c_string_visible_bytes(&formatted);
    Ok(formatted.to_vec())
}

fn fixed_do_join_string<const CAPACITY: usize, ContextBlock>(
    field: FactionDoJoinStringField,
    value: &[u8],
    application_removed: bool,
    member_inserted: bool,
) -> Result<[u8; CAPACITY], FactionDoJoinBlock<ContextBlock>> {
    let visible = legacy_c_string_visible_bytes(value);
    ensure_do_join_string_fits::<ContextBlock>(
        field,
        visible.len(),
        CAPACITY,
        application_removed,
        member_inserted,
    )?;
    let mut output = [0; CAPACITY];
    output[..visible.len()].copy_from_slice(visible);
    Ok(output)
}

fn fixed_initial_string<const CAPACITY: usize>(
    field: FactionInitialStringField,
    value: &[u8],
) -> Result<[u8; CAPACITY], FactionInitialBlock> {
    let visible = legacy_c_string_visible_bytes(value);
    if visible.len() >= CAPACITY {
        return Err(FactionInitialBlock {
            field,
            visible_len: visible.len(),
            capacity: CAPACITY,
        });
    }
    let mut output = [0; CAPACITY];
    output[..visible.len()].copy_from_slice(visible);
    Ok(output)
}

fn append_signed_set(output: &mut Vec<u8>, values: &BTreeSet<i32>) {
    output.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for &value in values {
        append_i32(output, value);
    }
}
