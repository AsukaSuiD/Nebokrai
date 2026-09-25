//! Wire-константы, lookup/receipt/session-контракты и отчёты `COrganizingCtrl`
//! из `organizingctrl.cpp/.h`, вынесенные из старого
//! `appworld/organizingsystem/organizingctrl.rs`. Сам контроллер, его session
//! endpoints (`CNetSessionManager`), free-player scan и DB-загрузка остаются в
//! старом пакете до следующих волн переноса organizing-области и связываются с
//! этими типами через переэкспорт старого файла.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use nebokrai_shared::runtime::TimerId;
use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock};

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::content::organizing::TagTimeValue;
use crate::organizations::faction::{
    CFaction, FactionApplyForJoinBlock, FactionBillboardStatBlock, FactionCloneSaveBlock,
    FactionContributorBlock, FactionContributorOutcome, FactionDelMemberBlock,
    FactionDelMemberReport, FactionDeleteOrganizingBuildError, FactionDeleteOrganizingOutcome,
    FactionDisbandBlock, FactionDisbandProgress, FactionDisbandRejection, FactionDoJoinBlock,
    FactionDoJoinOutcome, FactionEditLeaveWordOutcome, FactionEnemyMutationBlock,
    FactionExperienceUpdate, FactionFeatureFunctionUpdate, FactionFullSnapshotBlock,
    FactionInitialPropertyBlock, FactionLeaveWordBlock,
    FactionLeaveWordOutcome, FactionMemberInfoReport, FactionMemberInfoRequest,
    FactionOtherInfoBuildError, FactionOtherInfoDelivery, FactionOwnedCityRefreshBlock,
    FactionOwnedCityRefreshReport, FactionPlayerHeaderBlock, FactionPronounceBlock,
    FactionPronounceOutcome, FactionPropertyDelivery, FactionRemoveApplyMemberOutcome,
    FactionSuperiorOrganizingBlock, MemberEnterOutcome, MemberExitOutcome,
    MemberLevelChangeOutcome, MemberPositionChangeOutcome, OwnedCityAddOutcome,
    OwnedCityBooleanMutationReport, OwnedCityMutationBuildError,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::organizations::union::{
    CityTransferEndpointBlock, CityTransferTerminal, ConfederationCreationEndpointBlock,
    ConfederationCreationTerminal, UnionAddFactionBlock, UnionAddFactionOutcome,
    UnionApplyForJoinBlock, UnionApplyForJoinEffects, UnionApplyForJoinOutcome, UnionDemiseBlock,
    UnionDemiseOutcome, UnionDisbandBlock, UnionDisbandOutcome, UnionDoJoinBlock,
    UnionDoJoinOutcome, UnionExitBlock, UnionExitOutcome, UnionFactionFanoutReport,
    UnionFireOutBlock, UnionFireOutOutcome, UnionFormatArgument, UnionInitialBlock,
    UnionInitialReport, UnionInviteBlock, UnionInviteEffects, UnionInviteOutcome,
    UnionMemberSnapshotBlock, UnionOwnedCityBooleanMutationReport, UnionOwnedCityFanoutReport,
    UnionOwnedCityMutationBlock, UnionPlayerRefreshReport, UnionVictorFanoutReport,
    UnionVictorMutationBlock,
};

/// Результат `COrganizingCtrl::is_free_player`: свободный игрок, член
/// фракции либо ячейка с null-указателем фракции.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreePlayerLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionMemberDetachOutcome {
    NonPositiveFactionId,
    FactionEntryMissing,
    NullFactionPointer,
    Detached {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionMemberDetachBlockSource {
    SuperiorOrganizing(FactionSuperiorOrganizingBlock),
    Property(FactionInitialPropertyBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionMemberDetachBlock {
    pub faction_id: i32,
    pub source: UnionMemberDetachBlockSource,
}

/// Наблюдаемый результат `COrganizingCtrl::OnDeleteRole` до wire-ответа
/// LoginServer. Числа совпадают с switch `0..=4`.
#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDeleteRoleOutcome {
    AllowedNoFaction,
    CountryJob,
    MemberRemoved {
        faction_id: i32,
        removal: Option<FactionDelMemberReport>,
    },
    FactionMaster {
        faction_id: i32,
    },
    UnionMissing {
        faction_id: i32,
        union_id: i32,
    },
    UnionDetached {
        faction_id: i32,
        union_id: i32,
        detach: UnionMemberDetachOutcome,
    },
}

impl OrganizingDeleteRoleOutcome {
    pub const fn legacy_code(&self) -> i32 {
        match self {
            Self::AllowedNoFaction | Self::MemberRemoved { .. } | Self::UnionMissing { .. } => 0,
            Self::FactionMaster { .. } => 1,
            Self::UnionDetached { .. } => 3,
            Self::CountryJob => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingDeleteRoleBlock {
    NullFactionDuringMembershipScan { map_key: i32 },
    MemberRemoval {
        faction_id: i32,
        source: FactionDelMemberBlock,
    },
    MissingFactionProperty { faction_id: i32 },
    UnionDetach(UnionMemberDetachBlock),
}

/// Узкий dyn-шов ветви удаления роли к organizing-контроллеру. Inherent
/// `COrganizingCtrl::on_delete_role` дополнительно получает игру как
/// `&dyn WorldGameView` (его единственный игровой путь — refresh свойств
/// фракции при отвязке от союза), поэтому gate реализуется на контроллере
/// у владельца организаций и делегирует inherent-методу; игру в шов
/// передаёт сам обработчик коротким перезаймом по форме handler-волны.
pub trait WorldDeleteRoleOrganizingGate {
    fn apply_delete_role(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        country_has_job: bool,
    ) -> Result<OrganizingDeleteRoleOutcome, OrganizingDeleteRoleBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct TopInfoDelivery {
    pub top_info_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TopInfoDeliveryReport {
    pub game_server_id: Option<i32>,
    pub skipped_expired: usize,
    pub deliveries: Vec<TopInfoDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingInfoDelivery {
    RouteRejected {
        recipient_player_id: i32,
    },
    Sent {
        recipient_player_id: i32,
        game_server_id: i32,
        result: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrganizingFactionWarDeclarationBlock {
    MasterLookup(FactionMasterLookupBlock),
    MissingFactionProperty { faction_id: i32 },
    UnionMembership { map_key: i32 },
    MissingFactionInUnion { union_id: i32, faction_id: i32 },
    MissingFactionForMutation { faction_id: i32 },
    EnemyMutation {
        faction_id: i32,
        enemy_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfederationMasterLookupBlock {
    pub map_key: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrganizingFactionWarPlayerDiedBlock {
    MasterLookup(FactionMasterLookupBlock),
    UnionMembership { map_key: i32 },
    UnionMaster(ConfederationMasterLookupBlock),
    PlayerMembership { map_key: i32 },
    MissingFactionInUnion { union_id: i32, faction_id: i32 },
    MissingFactionForMutation { faction_id: i32 },
    EnemyMutation {
        faction_id: i32,
        enemy_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreeFactionLookup {
    NoUnion,
    Union(i32),
    BlockedNullConfederation { map_key: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddUnionToFactionRejection {
    UnionIdNotResolved { requested_union_id: i32 },
    UnionEntryMissing { union_id: i32 },
    NullUnionPointer { union_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct AddUnionToFactionDelivery {
    pub member_map_key: i32,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AddUnionToFactionOutcome {
    Rejected(AddUnionToFactionRejection),
    Sent {
        union_id: i32,
        faction_id: i32,
        deliveries: Vec<AddUnionToFactionDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum AddUnionToFactionBlock {
    FreeFactionScan { map_key: i32 },
    FactionUnavailable {
        faction_id: i32,
        entry_present: bool,
    },
    Snapshot {
        union_id: i32,
        faction_id: i32,
        member_map_key: i32,
        recipient_player_id: i32,
        game_server_id: i32,
        source: UnionMemberSnapshotBlock,
        completed_deliveries: Vec<AddUnionToFactionDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionClientSnapshotByPlayerOutcome {
    NoFaction,
    NoUnion { faction_id: i32 },
    MissingUnion { faction_id: i32, union_id: i32 },
    PlayerOffline { faction_id: i32, union_id: i32 },
    Sent {
        faction_id: i32,
        union_id: i32,
        game_server_id: i32,
        result: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionClientSnapshotByPlayerBlock {
    PlayerMembership { map_key: i32 },
    UnionMembership { map_key: i32 },
    Snapshot {
        faction_id: i32,
        union_id: i32,
        source: UnionMemberSnapshotBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionUnionMembershipLookupBlock {
    pub map_key: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionPlayerHeaderLookupBlock {
    pub master_faction_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionMasterLookupBlock {
    NullFaction { map_key: i32 },
    MissingMasterId { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordEnableOutcome {
    FactionNotFound,
    Updated {
        faction_id: i32,
        update: FactionFeatureFunctionUpdate,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordEnableBlock {
    MasterLookup(FactionMasterLookupBlock),
    Property {
        faction_id: i32,
        source: FactionInitialPropertyBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionLeaveWordOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordBlock {
    Membership { map_key: i32 },
    Faction {
        faction_id: i32,
        source: FactionLeaveWordBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordEditOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionEditLeaveWordOutcome,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingLeaveWordEditBlock {
    Membership { map_key: i32 },
    Property {
        faction_id: i32,
        source: FactionInitialPropertyBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingPronounceOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionPronounceOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingContributorOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionContributorOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingContributorBlock {
    Membership { map_key: i32 },
    Contributor(FactionContributorBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionExperienceMutation {
    FactionNotFound,
    PlayerNotContributor,
    Applied {
        faction_id: i32,
        faction_name: Vec<u8>,
        before_experience: i32,
        update: FactionExperienceUpdate,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionMemberStateOutcome {
    FactionNotFound,
    Level(MemberLevelChangeOutcome),
    Position(MemberPositionChangeOutcome),
    UnknownOperation,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingPronounceBlock {
    Membership { map_key: i32 },
    Faction {
        faction_id: i32,
        source: FactionPronounceBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum DeclareWarFactionRelation {
    None = 0,
    SelfFaction = 1,
    SameUnion = 2,
    Enemy = 3,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeclareWarFactionEntry {
    pub faction_id: i32,
    pub country: u8,
    pub name: Vec<u8>,
    pub relation: DeclareWarFactionRelation,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeclareWarFactionPage {
    pub requested_page: i32,
    pub normalized_page: i32,
    pub total_factions: i32,
    pub entries: Vec<DeclareWarFactionEntry>,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclareWarFactionPageBlock {
    FactionCountOutsideLegacyRange { count: usize },
    MissingSourceFaction { faction_id: i32 },
    NullFaction { map_key: i32 },
    MissingCountry { faction_id: i32 },
    UnionMembership { faction_id: i32, map_key: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionCountryCountBlock {
    pub map_key: i32,
    pub matched_before_block: i32,
    pub source: FactionInitialPropertyBlock,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionListEntry {
    pub faction_id: i32,
    pub name: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionListPage {
    pub requested_page: i32,
    pub normalized_page: i32,
    pub country: u8,
    pub start_index: i32,
    pub total_factions: i32,
    pub entry_count: i32,
    pub entries: Vec<FactionListEntry>,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionListCountPhase {
    Initial,
    LastPageRecount,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionListPageBlock {
    Count {
        requested_page: i32,
        normalized_page: i32,
        country: u8,
        phase: FactionListCountPhase,
        source: FactionCountryCountBlock,
    },
    Country {
        page: FactionListPage,
        map_key: i32,
        source: FactionInitialPropertyBlock,
    },
    NameWouldOverflow {
        page: FactionListPage,
        map_key: i32,
        required_bytes_with_nul: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingNameKind {
    Faction,
    Union,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingNameMatch {
    pub kind: OrganizingNameKind,
    pub map_key: i32,
    pub organizing_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingNameLookupBlock {
    RequestedNameWouldOverflow {
        visible_len: usize,
    },
    NullOwner {
        kind: OrganizingNameKind,
        map_key: i32,
    },
    OwnerNameWouldOverflow {
        kind: OrganizingNameKind,
        map_key: i32,
        visible_len: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingNameCountryBlock {
    TargetMissing {
        kind: OrganizingNameKind,
        map_key: i32,
    },
    MissingFactionProperty {
        map_key: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingNamedUnionApplicationBlock<SessionBlock> {
    TargetMissing {
        map_key: i32,
    },
    Apply {
        map_key: i32,
        union_id: i32,
        source: UnionApplyForJoinBlock<FactionUnionMembershipLookupBlock, SessionBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum DetachedFactionApplicationContextBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    RemovalNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionApplicationBlock {
    TargetMissing {
        map_key: i32,
    },
    Apply {
        map_key: i32,
        source: FactionApplyForJoinBlock<DetachedFactionApplicationContextBlock>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionClientSnapshotBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    FullSnapshot {
        faction_id: i32,
        source: FactionFullSnapshotBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllFactionInfoClientBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    NullFaction {
        map_key: i32,
        completed_factions: usize,
    },
    MissingCountry {
        map_key: i32,
        faction_id: i32,
        completed_factions: usize,
    },
    NameWouldOverflow {
        map_key: i32,
        faction_id: i32,
        visible_len: usize,
        completed_factions: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum DetachedFactionDoJoinContextBlock {
    RemovalNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
    MembershipNullFaction {
        map_key: i32,
    },
    FactionSnapshot(FactionClientSnapshotBlock),
    AllFactionInfo(AllFactionInfoClientBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDoJoinOutcome {
    FactionNotFound {
        faction_id: i32,
    },
    Applied {
        faction_id: i32,
        join_time: TagTimeValue,
        outcome: FactionDoJoinOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDoJoinBlock {
    ManagerMembership {
        map_key: i32,
    },
    DoJoin {
        faction_id: i32,
        source: FactionDoJoinBlock<DetachedFactionDoJoinContextBlock>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingUnionByMasterBlock {
    FactionMaster(FactionMasterLookupBlock),
    UnionMembership(FactionUnionMembershipLookupBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingUnionByPlayerBlock {
    PlayerMembership { map_key: i32 },
    UnionMembership { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionFireOutOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionFireOutOutcome<UnionMemberDetachOutcome>,
        automatic_disband: Option<OrganizingConfederationDisbandOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionFireOutBlock {
    Lookup(OrganizingUnionByMasterBlock),
    FireOut {
        union_id: i32,
        source: UnionFireOutBlock<FactionMasterLookupBlock, UnionMemberDetachBlock>,
    },
    AutomaticDisband {
        union_id: i32,
        fire_out: UnionFireOutOutcome<UnionMemberDetachOutcome>,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionDemiseOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionDemiseOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionDemiseBlock {
    Lookup(OrganizingUnionByMasterBlock),
    Demise {
        union_id: i32,
        source: UnionDemiseBlock<FactionMasterLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionExitOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionExitOutcome<UnionMemberDetachOutcome>,
        automatic_disband: Option<OrganizingConfederationDisbandOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionExitBlock {
    Lookup(OrganizingUnionByPlayerBlock),
    Exit {
        union_id: i32,
        source: UnionExitBlock<FactionMasterLookupBlock, UnionMemberDetachBlock>,
    },
    AutomaticPlayerHeader {
        union_id: i32,
        exit: UnionExitOutcome<UnionMemberDetachOutcome>,
        source: UnionPlayerHeaderLookupBlock,
    },
    AutomaticDisband {
        union_id: i32,
        exit: UnionExitOutcome<UnionMemberDetachOutcome>,
        player_header: i32,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingConfederationDisbandRejection {
    StandardWar,
    CityWar,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingConfederationDisbandReport {
    pub union_id: i32,
    pub standard_enemy_clear: UnionFactionFanoutReport,
    pub city_enemy_clear: UnionFactionFanoutReport,
    pub union_outcome: UnionDisbandOutcome<UnionMemberDetachOutcome>,
    pub delete_queued: bool,
    pub owned_city_refreshes: Vec<(i32, i32, i32)>,
    pub player_refresh: Option<UnionPlayerRefreshReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingConfederationDisbandOutcome {
    UnionNotFound,
    Rejected(OrganizingConfederationDisbandRejection),
    Applied(OrganizingConfederationDisbandReport),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingConfederationDisbandBlock {
    Disband {
        union_id: i32,
        source: UnionDisbandBlock<
            FactionMasterLookupBlock,
            UnionMemberDetachBlock,
            UnionMemberDetachOutcome,
        >,
    },
}

pub type OrganizingUnionApplyForJoinBlock<SessionBlock> =
    UnionApplyForJoinBlock<FactionUnionMembershipLookupBlock, SessionBlock>;

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionApplyForJoinOutcome<SessionReport> {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionApplyForJoinOutcome<SessionReport>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionApplyForJoinDispatchBlock<SessionBlock> {
    Lookup(OrganizingUnionByMasterBlock),
    Apply {
        union_id: i32,
        source: OrganizingUnionApplyForJoinBlock<SessionBlock>,
    },
}

pub type OrganizingUnionApplicationJoinBlock = UnionDoJoinBlock<
    FactionUnionMembershipLookupBlock,
    FactionMasterLookupBlock,
    AddUnionToFactionBlock,
>;

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionApplicationCallbackReport {
    pub union_id: i32,
    pub applicant_faction_id: i32,
    pub union_found: bool,
    pub rejection_notice_sent: bool,
    pub join: Option<UnionDoJoinOutcome>,
    pub application_cleared: bool,
    pub establishment_reservation_removed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionApplicationCallbackBlock {
    PlayerHeader {
        union_id: i32,
        applicant_faction_id: i32,
        source: UnionPlayerHeaderLookupBlock,
        rejection_notice_sent: bool,
    },
    DoJoin {
        union_id: i32,
        applicant_faction_id: i32,
        source: OrganizingUnionApplicationJoinBlock,
        rejection_notice_sent: bool,
    },
}

pub type OrganizingUnionInviteBlock<SessionBlock> = UnionInviteBlock<
    FactionMasterLookupBlock,
    FactionUnionMembershipLookupBlock,
    SessionBlock,
>;

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionInviteOutcome<SessionReport> {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionInviteOutcome<SessionReport>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionInviteDispatchBlock<SessionBlock> {
    TargetMissing { union_id: i32 },
    Invite {
        union_id: i32,
        source: OrganizingUnionInviteBlock<SessionBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionInvitationCallbackReport {
    pub union_id: i32,
    pub inviter_faction_id: i32,
    pub invited_faction_id: i32,
    pub union_found: bool,
    pub rejection_notice_sent: bool,
    pub join: Option<UnionDoJoinOutcome>,
    pub application_cleared: bool,
    pub establishment_reservation_removed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionInvitationCallbackBlock {
    InviterPlayerHeader {
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        source: FactionPlayerHeaderBlock<UnionPlayerHeaderLookupBlock>,
        rejection_notice_sent: bool,
    },
    DoJoin {
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        source: OrganizingUnionApplicationJoinBlock,
        rejection_notice_sent: bool,
    },
}

pub trait PlayerInviteFactionEffects:
    ConfederationCreationEffects + UnionApplyForJoinEffects + UnionInviteEffects
{
}

impl<T> PlayerInviteFactionEffects for T where
    T: ConfederationCreationEffects + UnionApplyForJoinEffects + UnionInviteEffects
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerInviteFactionRejection {
    MasterFactionNotFound,
    InviterFactionMissing,
    InvitedFactionMissing,
    InviterStandardWar { notice_sent: bool },
    InviterCityWar { notice_sent: bool },
    InvitedStandardWar { notice_sent: bool },
    InvitedCityWar { notice_sent: bool },
    BothAlreadyInUnion { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub enum PlayerInviteFactionAction<CreationReport, ApplicationReport, InvitationReport> {
    Create(ConfederationCreationStartOutcome<CreationReport>),
    ApplyToInvitedUnion(UnionApplyForJoinOutcome<ApplicationReport>),
    InviteToInviterUnion(OrganizingUnionInviteOutcome<InvitationReport>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum PlayerInviteFactionOutcome<CreationReport, ApplicationReport, InvitationReport> {
    Rejected(PlayerInviteFactionRejection),
    Dispatched {
        inviter_faction_id: i32,
        invited_faction_id: i32,
        action: PlayerInviteFactionAction<CreationReport, ApplicationReport, InvitationReport>,
    },
}

#[derive(Debug)]
pub enum PlayerInviteFactionBlock<CreationBlock, ApplicationBlock, InvitationBlock> {
    Master(FactionMasterLookupBlock),
    InviterMembership { map_key: i32 },
    InvitedMembership { map_key: i32 },
    Creation(ConfederationCreationStartBlock<CreationBlock>),
    Application(OrganizingNamedUnionApplicationBlock<ApplicationBlock>),
    Invitation(OrganizingUnionInviteDispatchBlock<InvitationBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplyFactionRemoval {
    pub map_key: i32,
    pub outcome: FactionRemoveApplyMemberOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RemovePersonFromApplyFactionListOutcome {
    Completed {
        removals: Vec<ApplyFactionRemoval>,
    },
    BlockedNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyFactionLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingSaveDataBlock {
    FactionClone {
        map_key: i32,
        reason: FactionCloneSaveBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingSaveDataReport {
    pub saved_factions: usize,
    pub saved_unions: usize,
    pub deleted_factions: usize,
    pub deleted_unions: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingRunBlock {
    DeleteRemainTimeAbsent { map_key: i32 },
    MasterIdAbsent { map_key: i32, faction_id: i32 },
    Disband {
        faction_id: i32,
        master_id: i32,
        source: OrganizingDisbandBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingRunDisband {
    pub faction_id: i32,
    pub master_id: i32,
    pub result: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingRunReport {
    pub decremented_factions: usize,
    pub disbands: Vec<OrganizingRunDisband>,
    pub expired_top_infos: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionEnterDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum PlayerEnterGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
    Completed {
        faction: FactionEnterDispatch,
        top_info: TopInfoDeliveryReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionExitDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum PlayerExitGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
    Dispatched(FactionExitDispatch),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingOtherFactionUpdate {
    pub map_key: i32,
    pub deliveries: Vec<FactionOtherInfoDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingOtherFactionUpdateBlock {
    pub map_key: i32,
    pub source: FactionOtherInfoBuildError,
    pub completed: Vec<OrganizingOtherFactionUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizingDisbandPlayer {
    pub player_id: i32,
    pub player_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingDisbandRejection {
    FactionEntryMissing,
    StandardWar,
    CityWar,
    Faction(FactionDisbandRejection),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingDisbandProgress {
    pub cleared_city_war_enemies: usize,
    pub faction: FactionDisbandProgress,
    pub map_removed: bool,
    pub second_delete_organizing: Option<FactionDeleteOrganizingOutcome>,
    pub delete_faction_queued: bool,
    pub other_faction_updates: Option<Vec<OrganizingOtherFactionUpdate>>,
    pub player: Option<OrganizingDisbandPlayer>,
    pub log_written: bool,
}

pub enum OrganizingDisbandOutcome {
    Rejected {
        reason: OrganizingDisbandRejection,
        notice_sent: bool,
        cleared_city_war_enemies: usize,
    },
    Disbanded {
        progress: OrganizingDisbandProgress,
        retired_faction: Box<CFaction>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDisbandBlock {
    NullFaction {
        faction_id: i32,
    },
    Faction {
        source: FactionDisbandBlock,
        cleared_city_war_enemies: usize,
    },
    SecondDeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        progress: OrganizingDisbandProgress,
    },
    OtherFactionUpdate {
        source: OrganizingOtherFactionUpdateBlock,
        progress: OrganizingDisbandProgress,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfederationCreationSessionRequest {
    pub first_player_id: i32,
    pub second_player_id: i32,
    pub first_faction_id: i32,
    pub second_faction_id: i32,
    pub requested_session_id: i32,
    pub timeout_ticks: u32,
    pub union_name: Vec<u8>,
}

pub trait ConfederationCreationSessionRuntime: Send + Sync {
    fn send_confederation_creation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    fn finish_confederation_creation(
        &self,
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        union_name: &[u8],
        terminal: ConfederationCreationTerminal,
    );

    fn block_confederation_creation_endpoint(&self, block: ConfederationCreationEndpointBlock);
}

pub trait FactionCreationEffects {
    fn check_invalid_organizing_string(&mut self, name: &mut Vec<u8>, strict: bool) -> bool;
    fn persistent_player_name_exists(&mut self, name: &[u8]) -> bool;
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn faction_create_log_enabled(&self) -> bool;
    fn write_faction_create_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionCreationRejection {
    PlayerAlreadyInFaction,
    InvalidName { notice_sent: bool },
    NameExists,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FactionCreationReport {
    pub faction_id: i32,
    pub application_removals: Vec<ApplyFactionRemoval>,
    pub other_faction_updates: Vec<OrganizingOtherFactionUpdate>,
    pub log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FactionCreationOutcome {
    Rejected(FactionCreationRejection),
    Created(FactionCreationReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionCreationPreparation {
    Rejected(FactionCreationRejection),
    ReadyForPersistentLookup,
}

pub trait ConfederationCreationEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_confederation_creation_session(
        &mut self,
        request: ConfederationCreationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfederationCreationRejection {
    FirstFactionReserved,
    SecondFactionReserved,
    ZeroFactionId,
    FirstFactionMissing,
    SecondFactionMissing,
    CreationFunctionDisabled { notice_sent: bool },
    SecondMasterOffline { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConfederationCreationStartOutcome<SessionReport> {
    Rejected(ConfederationCreationRejection),
    Started {
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug)]
pub enum ConfederationCreationStartBlock<SessionBlock> {
    MissingFirstMaster { faction_id: i32 },
    MissingSecondMaster { faction_id: i32 },
    MissingFirstProperty { faction_id: i32 },
    Session {
        first_faction_id: i32,
        second_faction_id: i32,
        first_reservation_removed: bool,
        second_reservation_removed: bool,
        source: SessionBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfederationCreationSnapshotDelivery {
    pub player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfederationCreationCallbackReport {
    pub terminal: ConfederationCreationTerminal,
    pub rejection_notice_sent: bool,
    pub first_faction_found: bool,
    pub second_faction_found: bool,
    pub union_id: Option<i32>,
    pub initial: Option<UnionInitialReport>,
    pub first_faction_notice: Option<FactionMemberInfoReport>,
    pub second_faction_addition: Option<UnionAddFactionOutcome>,
    pub first_superior_assigned: Option<bool>,
    pub second_superior_assigned: Option<bool>,
    pub first_owned_city_refresh: Option<FactionOwnedCityRefreshReport>,
    pub second_owned_city_refresh: Option<FactionOwnedCityRefreshReport>,
    pub first_player_refresh: Option<Vec<i32>>,
    pub second_player_refresh: Option<Vec<i32>>,
    pub first_player_snapshot: Option<ConfederationCreationSnapshotDelivery>,
    pub second_player_snapshot: Option<ConfederationCreationSnapshotDelivery>,
    pub repeated_first_city_refreshes: Vec<i32>,
    pub first_reservation_removed: bool,
    pub second_reservation_removed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConfederationCreationCallbackBlock {
    Initial {
        report: ConfederationCreationCallbackReport,
        source: UnionInitialBlock,
    },
    AddSecondFaction {
        report: ConfederationCreationCallbackReport,
        source: UnionAddFactionBlock,
    },
    FirstSuperior {
        report: ConfederationCreationCallbackReport,
        source: FactionSuperiorOrganizingBlock,
    },
    SecondSuperior {
        report: ConfederationCreationCallbackReport,
        source: FactionSuperiorOrganizingBlock,
    },
    FirstOwnedCityRefresh {
        report: ConfederationCreationCallbackReport,
        source: FactionOwnedCityRefreshBlock,
    },
    SecondOwnedCityRefresh {
        report: ConfederationCreationCallbackReport,
        source: FactionOwnedCityRefreshBlock,
    },
    FirstSnapshot {
        report: ConfederationCreationCallbackReport,
        source: UnionMemberSnapshotBlock,
    },
    SecondSnapshot {
        report: ConfederationCreationCallbackReport,
        source: UnionMemberSnapshotBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CityTransferSessionRequest {
    pub requester_player_id: i32,
    pub source_faction_id: i32,
    pub target_master_player_id: i32,
    pub target_faction_id: i32,
    pub region_id: i32,
    pub requested_session_id: i32,
    pub timeout_ticks: u32,
    pub source_faction_name: Vec<u8>,
    pub region_name: Vec<u8>,
}

pub trait CityTransferSessionRuntime: Send + Sync {
    fn send_city_transfer_confirmation(&self, recipient_player_id: i32, message: &CMessage);

    fn finish_city_transfer(
        &self,
        source_faction_id: i32,
        target_faction_id: i32,
        region_id: i32,
        region_name: &[u8],
        terminal: CityTransferTerminal,
    );

    fn block_city_transfer_endpoint(&self, block: CityTransferEndpointBlock);
}

pub trait CityTransferEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_city_transfer_session(
        &mut self,
        request: CityTransferSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;
    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );
    fn broadcast_city_transfer(&mut self, text: &[u8]) -> Result<i32, SendMessageError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityTransferRejection {
    RegionNotFound,
    NullRegionPointer,
    NonPositiveRegion,
    CityWarActive { notice_sent: bool },
    MasterFactionNotFound,
    ZeroTargetFactionId,
    SourceFactionMissing,
    TargetFactionMissing,
    CountriesDiffer { notice_sent: bool },
    SourceMasterIsCountryKing { notice_sent: bool },
    SourceDoesNotOwnRegion,
    TargetAlreadyOwnsCity { notice_sent: bool },
    TargetDeclaredAttackWar { notice_sent: bool },
    TargetDeclaredVillageWar { notice_sent: bool },
    SourceFactionReserved { notice_sent: bool },
    TargetFactionReserved { notice_sent: bool },
    TargetMasterOffline { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityTransferStartOutcome<SessionReport> {
    Rejected(CityTransferRejection),
    Started {
        source_faction_id: i32,
        target_master_player_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug)]
pub enum CityTransferStartBlock<SessionBlock> {
    FactionMaster(FactionMasterLookupBlock),
    MissingSourceCountry { faction_id: i32 },
    MissingTargetCountry { faction_id: i32 },
    MissingSourceMaster { faction_id: i32 },
    MissingTargetMaster { faction_id: i32 },
    Session {
        source_faction_id: i32,
        target_faction_id: i32,
        source_reserved: bool,
        target_reserved: bool,
        source: SessionBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CityTransferFinishReport {
    pub source_reservation_removed: bool,
    pub target_reservation_removed: bool,
    pub source_faction_found: bool,
    pub target_faction_found: bool,
    pub source_cities: Option<OwnedCityBooleanMutationReport>,
    pub target_city: Option<OwnedCityAddOutcome>,
    pub target_union_id: Option<i32>,
    pub broadcast: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityTransferFinishBlock {
    SourceCities {
        report: CityTransferFinishReport,
        source: OwnedCityMutationBuildError,
    },
    TargetCity {
        report: CityTransferFinishReport,
        source: OwnedCityMutationBuildError,
    },
    TargetUnion {
        report: CityTransferFinishReport,
        map_key: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityWarOrganizingOwner {
    Faction(i32),
    Union(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarVictorMutation {
    Faction(Vec<FactionPropertyDelivery>),
    Union(UnionVictorFanoutReport),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarOwnedCityRemoval {
    Faction(OwnedCityBooleanMutationReport),
    Union(UnionOwnedCityBooleanMutationReport),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarOwnedCityAddition {
    Faction(OwnedCityAddOutcome),
    Union(UnionOwnedCityFanoutReport),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttackCityEndReport {
    pub attacker_faction_id: Option<i32>,
    pub attacker_union_id: Option<i32>,
    pub attacker_owner: Option<CityWarOrganizingOwner>,
    pub defender_union_id: Option<i32>,
    pub defender_owner: Option<CityWarOrganizingOwner>,
    pub defender_owns_city: Option<bool>,
    pub offense_victors: Option<CityWarVictorMutation>,
    pub defender_city_removal: Option<CityWarOwnedCityRemoval>,
    pub attacker_city_addition: Option<CityWarOwnedCityAddition>,
    pub defence_victors: Option<CityWarVictorMutation>,
    pub refreshed_owner: Option<(i32, i32, i32)>,
    pub broadcast: Option<Result<i32, SendMessageError>>,
}

impl AttackCityEndReport {
    pub const fn pending() -> Self {
        Self {
            attacker_faction_id: None,
            attacker_union_id: None,
            attacker_owner: None,
            defender_union_id: None,
            defender_owner: None,
            defender_owns_city: None,
            offense_victors: None,
            defender_city_removal: None,
            attacker_city_addition: None,
            defence_victors: None,
            refreshed_owner: None,
            broadcast: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarVictorMutationBlock {
    Faction(FactionInitialPropertyBlock),
    Union(UnionVictorMutationBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CityWarOwnedCityMutationBlock {
    Faction(OwnedCityMutationBuildError),
    Union(UnionOwnedCityMutationBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum AttackCityEndBlock {
    RegionNameWouldOverflow {
        report: AttackCityEndReport,
        visible_len: usize,
    },
    AttackerMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    AttackerUnionMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    DefenderUnionMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    OffenseVictors {
        report: AttackCityEndReport,
        source: CityWarVictorMutationBlock,
    },
    DefenderCityRemoval {
        report: AttackCityEndReport,
        source: CityWarOwnedCityMutationBlock,
    },
    AttackerCityAddition {
        report: AttackCityEndReport,
        source: CityWarOwnedCityMutationBlock,
    },
    DefenceVictors {
        report: AttackCityEndReport,
        source: CityWarVictorMutationBlock,
    },
    MissingAttackerOwnerForDefenceNotice {
        report: AttackCityEndReport,
    },
    NoticeWouldOverflow {
        report: AttackCityEndReport,
        string_id: &'static [u8],
        formatted_len: usize,
    },
}

pub trait AttackCityEndEffects {
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );

    fn broadcast_city_war_result(&mut self, text: &[u8]) -> Result<i32, SendMessageError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingDatabasePublishReport {
    pub published_unions: usize,
    pub published_factions: usize,
}

#[derive(Debug)]
pub struct OrganizingDatabasePublishBlock {
    pub union_id: i32,
    pub source: UnionInitialBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingDatabaseLoadDisposition {
    PublishedAll,
    FactionLoadFailed,
    FactionLoadBlocked,
}

#[derive(Debug)]
pub struct OrganizingDatabaseLoadReport {
    pub published_unions: usize,
    pub published_factions: usize,
 /// Точный `int` из `CRsUnion::LoadAllConfederation`, который `Initialize`
 /// передаёт в `%d` log независимо от дальнейшей publication-семантики.
    pub union_reported_count: i32,
    pub faction_reported_count: i32,
    pub union_load_returned_true: bool,
    pub disposition: OrganizingDatabaseLoadDisposition,
}

#[derive(Clone, Copy, Debug)]
pub struct OrganizingNewDayScheduleReport {
    pub scheduled_time: TagTime,
    pub event_id: TimerId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingCtrlReleaseReport {
    pub previous_new_day_event_id: Option<TimerId>,
    pub timer_event_removed: Option<bool>,
}

#[derive(Debug)]
pub struct OrganizingNewDayReport {
    pub reset_faction_ids: Vec<i32>,
    pub country_day: u16,
    pub scheduled_time: TagTime,
    pub event_id: TimerId,
}

#[derive(Debug)]
pub enum OrganizingNewDayBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingNewDayScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug)]
pub struct OrganizingInitializeSuffixReport {
    pub new_day_schedule: OrganizingNewDayScheduleReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingInitializeSuffixBlock {
    NewDaySchedule(TagTimeArithmeticBlock),
    Billboard(FactionBillboardStatBlock),
}

#[derive(Debug)]
pub struct OrganizingInitializeReport {
    pub database: OrganizingDatabaseLoadReport,
    pub suffix: OrganizingInitializeSuffixReport,
}

/// Safe-граница полного `Initialize` не откатывает ранее опубликованный prefix
/// либо уже поставленный timer event.
#[derive(Debug)]
pub enum OrganizingInitializeBlock {
    Database(OrganizingDatabasePublishBlock),
    Suffix(OrganizingInitializeSuffixBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyFactionRelationOutcome {
    FirstFactionMissing,
    SecondFactionMissing,
    Applied,
}
