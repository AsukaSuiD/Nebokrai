//! Owner `CUnion` WorldServer из `appworld/organizingsystem/union.cpp/.h`.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`. Union хранит master
//! faction, ordered member factions, applications, properties, owned cities,
//! purview state и DB/wire projections. ID/name/job fields, list/map order,
//! capacities и dirty bits сохраняются.
//!
//! Add/remove/disband/demise/governance operations применяют permission gates,
//! mutations, faction callbacks, logs и publications в исходной очередности.
//! Некоторые helpers намеренно не читают receiver либо используют asymmetric
//! limits; эти quirks остаются возле соответствующего метода и не исправляются
//! общим рефакторингом.
//!
//! Load/clone/save разделяют live state и DB snapshot; partial prefix и
//! nullable owner mapping сохраняют исходные outcomes. Standard collections,
//! owned values и traits заменяют MSVC nodes, raw pointers и vtable plumbing,
//! но не меняют wire/DB/gameplay semantics. Malformed fixed strings и null map
//! entries останавливаются typed-границей до UB.
//!
//! Перенесён в Realm `organizations/`. Game-параметры контекст-трейтов
//! заменены bridge-адаптером при единственном исходном владельце (прецедент
//! `WorldRegionOwnerOrganizingView`): контексты organizing больше не читают
//! `&CGame`, реализация с game-доступом живёт в bridge-структуре старого
//! пакета. Собственные отправки `CUnion` идут через `&dyn WorldGameView`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use nebokrai_shared::runtime::{
    NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionEndpoint,
};
use nebokrai_shared::runtime::{
    CNetSessionManager, CreatedNetSession, NetSessionCreateBlock, NetSessionManagerBeginBlock,
    NetSessionSetCallbackBlock,
};

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::content::organizing::{
    EOperator, EPurview, EPurviewOwnState, MemberPurviewMutation, TagMemInfo, TagTimeValue,
    UnterminatedMemberField,
};
use crate::organizations::faction::{
    current_local_member_time, FactionEnemyDelivery, FactionInitialPropertyBlock,
    FactionMemberInfoReport, FactionMemberInfoRequest, FactionOwnedCityDelivery,
    FactionOwnedCityRefreshBlock, FactionOwnedCityRefreshReport,
    FactionOwnedCityUpdateBuildError, FactionPropertyDelivery,
    FactionSuperiorOrganizingBlock, OwnedCityMutationBuildError,
};
use crate::organizations::organizingparam::COrganizingParam;

const DELETE_UNION_ORGANIZING_MESSAGE_TYPE: i32 = 0x7FE05;
const UNION_MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0E;
const UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE: i32 = 0x7FE17;
const MAX_UNION_MEMBER_COUNT: i32 = 50;
const UNION_DEMISE_MEMBER_LIMIT: usize = 6;
const UNION_DEMISE_COOLDOWN_MS: u32 = 10_800_000;

pub trait UnionOperatorValidationContext {
    type Block;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block>;
}

pub trait UnionMasterFactionQueryContext {
    fn faction_is_owned_city(&self, faction_id: i32, region_id: i32) -> Option<i32>;

    fn faction_is_enemy_faction(&self, faction_id: i32, enemy_id: i32) -> Option<i32>;

    fn faction_owned_cities(&self, faction_id: i32) -> Option<VecDeque<i32>>;

    fn faction_has_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_has_city_war_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_enemy_leader_organizing_id(&self, faction_id: i32) -> Option<i32>;
}

pub trait UnionOwnedCityMutationContext {
    fn faction_add_owned_city(
        &mut self,
        faction_id: i32,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_add_owned_cities(
        &mut self,
        faction_id: i32,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_clear_owned_cities(
        &mut self,
        faction_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_set_owned_cities(
        &mut self,
        faction_id: i32,
        region_ids: &VecDeque<i32>,
    ) -> Result<bool, OwnedCityMutationBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCityFanoutReport {
    pub invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCityMutationBlock {
    pub faction_id: i32,
    pub completed_faction_ids: Vec<i32>,
    pub source: OwnedCityMutationBuildError,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCityBooleanMutationReport {
    pub legacy_result: bool,
    pub invoked_faction_ids: Vec<i32>,
}

pub trait UnionFactionStateMutationContext {
    fn faction_clear_enemy_factions(&mut self, faction_id: i32) -> bool;

    fn faction_clear_city_war_enemy_factions(&mut self, faction_id: i32) -> bool;

    fn faction_add_defence_victor_count(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_offense_victor_count(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_village_war_victor_count(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionFactionFanoutReport {
    pub invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionVictorFactionReport {
    pub faction_id: i32,
    pub deliveries: Vec<FactionPropertyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionVictorFanoutReport {
    pub factions: Vec<UnionVictorFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionVictorMutationBlock {
    pub faction_id: i32,
    pub completed_factions: Vec<UnionVictorFactionReport>,
    pub source: FactionInitialPropertyBlock,
}

pub trait UnionPlayerRefreshContext {
    fn faction_update_player_info(
        &self,
        faction_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Option<Vec<i32>>;
}

pub trait UnionInitialMutationContext {
    fn faction_set_superior_organizing(
        &mut self,
        faction_id: i32,
        union_id: i32,
        union_master_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionSuperiorOrganizingBlock>;
}

pub trait UnionFactionJoinContext:
    UnionFactionMemberContext
    + UnionInitialMutationContext
    + UnionPlayerRefreshContext
    + UnionSendInfoContext
{
    fn faction_refresh_owned_city_info(
        &self,
        faction_id: i32,
        refresh_owned_city: &mut dyn FnMut(i32, i32, i32, Option<u8>),
    ) -> Result<Option<FactionOwnedCityRefreshReport>, FactionOwnedCityRefreshBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

pub trait UnionAddFactionEffects {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn put_war_log(&mut self, text: &[u8]);

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
}

pub trait UnionDoJoinContext {
    type FreeFactionBlock;
    type InitialSnapshotBlock;

    fn union_id_for_joining_faction(
        &self,
        faction_id: i32,
    ) -> Result<i32, Self::FreeFactionBlock>;

    fn add_current_union_to_client_by_faction_id(
        &mut self,
        union: &mut CUnion,
        faction_id: i32,
    ) -> Result<bool, Self::InitialSnapshotBlock>;
}

pub trait UnionFireOutContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
{
    type DetachBlock;
    type DetachOutcome;

    fn detach_union_member_for_fire_out(
        &mut self,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock>;
}

pub trait UnionDemiseContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
    + UnionMasterProjectionMutationContext
{
}

impl<T> UnionDemiseContext for T where
    T: UnionOperatorValidationContext
        + UnionMasterFactionQueryContext
        + UnionSendInfoContext
        + UnionFactionMemberContext
        + UnionPlayerRefreshContext
        + UnionMasterProjectionMutationContext
{
}

pub trait UnionMasterProjectionMutationContext {
    fn set_union_master_projection(
        &mut self,
        faction_ids: &[i32],
        union_master_id: i32,
    );
}

pub trait UnionExitContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
{
    type DetachBlock;
    type DetachOutcome;

    fn detach_union_member_for_exit(
        &mut self,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock>;
}

pub trait UnionFireOutEffects {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);

    fn put_war_log(&mut self, text: &[u8]);

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionApplicationFactionSnapshot {
    pub faction_id: i32,
    pub name: Vec<u8>,
    pub player_header: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionApplicationFactionBlock {
    pub faction_id: i32,
}

pub trait UnionApplyForJoinContext {
    fn union_application_is_reserved(&self, faction_id: i32) -> bool;

    fn union_application_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<UnionApplicationFactionSnapshot>, UnionApplicationFactionBlock>;

    fn reserve_union_application(&mut self, faction_id: i32);
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionApplicationSessionRequest {
    pub union_id: i32,
    pub applicant_faction_id: i32,
    pub recipient_player_id: i32,
    pub requested_session_id: i32,
    pub timeout_ticks: u32,
    pub confirmation_kind: i32,
    pub applicant_faction_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionApplicationTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfederationCreationEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityTransferEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

/// Терминалы ConfederationCreation `/ CityTransfer сессий, разделённых от
/// organizingctrl и нужные диспетчеру очередей в Realm app.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfederationCreationTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityTransferTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionApplicationEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

pub trait UnionApplicationSessionRuntime: Send + Sync {
    fn send_union_application_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    fn finish_union_application(
        &self,
        union_id: i32,
        applicant_faction_id: i32,
        terminal: UnionApplicationTerminal,
    );

    fn block_union_application_endpoint(&self, block: UnionApplicationEndpointBlock);
}

pub struct PlayerApplyForJoinConfeder {
    request: UnionApplicationSessionRequest,
    runtime: Arc<dyn UnionApplicationSessionRuntime>,
}

impl PlayerApplyForJoinConfeder {
    pub fn new(
        request: UnionApplicationSessionRequest,
        runtime: Arc<dyn UnionApplicationSessionRuntime>,
    ) -> Self {
        Self { request, runtime }
    }
}

impl NetSessionEndpoint for PlayerApplyForJoinConfeder {
    fn do_async_call(&self, session_id: i64, cookie_second: i32) {
        let request = &self.request;
        let mut message = CMessage::new(UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.recipient_player_id);
        message.base_mut().add_long(request.confirmation_kind);
        message
            .base_mut()
            .add(legacy_c_string_visible_bytes(&request.applicant_faction_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime
            .send_union_application_confirmation(request.recipient_player_id, &message);
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result.value else {
                return;
            };
            if decision == 1 {
                UnionApplicationTerminal::Approved
            } else {
                UnionApplicationTerminal::Denied
            }
        } else {
            UnionApplicationTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_union_application(
            self.request.union_id,
            self.request.applicant_faction_id,
            terminal,
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionApplicationSessionReport {
    pub session: CreatedNetSession,
}

pub enum UnionApplicationSessionBlock {
    Create(NetSessionCreateBlock),
    SetCallback {
        session: CreatedNetSession,
        source: NetSessionSetCallbackBlock,
    },
    Begin {
        session: CreatedNetSession,
        source: NetSessionManagerBeginBlock,
    },
}

impl std::fmt::Debug for UnionApplicationSessionBlock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(source) => formatter.debug_tuple("Create").field(source).finish(),
            Self::SetCallback { session, source } => {
                let source = match source {
                    NetSessionSetCallbackBlock::SessionNotFound { .. } => "SessionNotFound",
                    NetSessionSetCallbackBlock::AlreadyAssigned(_) => "AlreadyAssigned",
                };
                formatter
                    .debug_struct("SetCallback")
                    .field("session", session)
                    .field("source", &source)
                    .finish()
            }
            Self::Begin { session, source } => formatter
                .debug_struct("Begin")
                .field("session", session)
                .field("source", source)
                .finish(),
        }
    }
}

pub fn begin_union_application_session(
    manager: &CNetSessionManager,
    request: UnionApplicationSessionRequest,
    runtime: Arc<dyn UnionApplicationSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<UnionApplicationSessionReport, UnionApplicationSessionBlock> {
    let session = manager
        .create_session(
            request.recipient_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(UnionApplicationSessionBlock::Create)?;
    let timeout_ticks = request.timeout_ticks;
    let endpoint = Box::new(PlayerApplyForJoinConfeder::new(request, runtime));
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| UnionApplicationSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, timeout_ticks)
        .map_err(|source| UnionApplicationSessionBlock::Begin { session, source })?;
    Ok(UnionApplicationSessionReport { session })
}

pub trait UnionApplyForJoinEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);

    fn begin_union_application_session(
        &mut self,
        request: UnionApplicationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionInvitationSessionRequest {
    pub union_id: i32,
    pub inviter_faction_id: i32,
    pub invited_faction_id: i32,
    pub recipient_player_id: i32,
    pub requested_session_id: i32,
    pub timeout_ticks: u32,
    pub inviter_faction_name: Vec<u8>,
}

pub trait UnionInvitationSessionRuntime: Send + Sync {
    fn send_union_invitation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    fn finish_union_invitation(
        &self,
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        terminal: UnionApplicationTerminal,
    );

    fn block_union_invitation_endpoint(&self, block: UnionApplicationEndpointBlock);
}

pub struct InviteJoinConfeder {
    request: UnionInvitationSessionRequest,
    runtime: Arc<dyn UnionInvitationSessionRuntime>,
}

impl NetSessionEndpoint for InviteJoinConfeder {
    fn do_async_call(&self, session_id: i64, cookie_second: i32) {
        let request = &self.request;
        let mut message = CMessage::new(UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.recipient_player_id);
        message.base_mut().add_long(1);
        message
            .base_mut()
            .add(legacy_c_string_visible_bytes(&request.inviter_faction_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime
            .send_union_invitation_confirmation(request.recipient_player_id, &message);
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result.value else {
                return;
            };
            if decision == 1 {
                UnionApplicationTerminal::Approved
            } else {
                UnionApplicationTerminal::Denied
            }
        } else {
            UnionApplicationTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_union_invitation(
            self.request.union_id,
            self.request.inviter_faction_id,
            self.request.invited_faction_id,
            terminal,
        );
    }
}

pub fn begin_union_invitation_session(
    manager: &CNetSessionManager,
    request: UnionInvitationSessionRequest,
    runtime: Arc<dyn UnionInvitationSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<UnionApplicationSessionReport, UnionApplicationSessionBlock> {
    let session = manager
        .create_session(
            request.recipient_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(UnionApplicationSessionBlock::Create)?;
    let timeout_ticks = request.timeout_ticks;
    let endpoint = Box::new(InviteJoinConfeder { request, runtime });
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| UnionApplicationSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, timeout_ticks)
        .map_err(|source| UnionApplicationSessionBlock::Begin { session, source })?;
    Ok(UnionApplicationSessionReport { session })
}

pub trait UnionInviteEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_union_invitation_session(
        &mut self,
        request: UnionInvitationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionInviteRejection {
    PendingApplication,
    ZeroFactionId,
    InvitedFactionReserved,
    InviterNotPermitted,
    InvitedAlreadyInUnion { union_id: i32 },
    InviterFactionMissing,
    InvitedFactionMissing,
    InvitedMasterOffline { notice_sent: bool },
    MemberLimit {
        member_count: i32,
        maximum: i32,
        allocated_net_exchange_id: i32,
        notice_sent: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionInviteOutcome<SessionReport> {
    Rejected(UnionInviteRejection),
    Started {
        invited_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionInviteBlock<OperatorBlock, MembershipBlock, SessionBlock> {
    PendingApplicationUninitialized,
    Operator(OperatorBlock),
    Membership(MembershipBlock),
    FactionSnapshot(UnionApplicationFactionBlock),
    Session {
        source: SessionBlock,
        invited_faction_id: i32,
        net_exchange_id: i32,
        application_assigned: bool,
        establishment_reserved: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionApplyForJoinRejection {
    PendingApplication,
    ZeroApplicantFactionId,
    ApplicantAlreadyReserved,
    ApplicantAlreadyInUnion { union_id: i32 },
    ApplicantFactionMissing,
    MasterFactionMissing,
    MasterPlayerOffline { notice_sent: bool },
    MemberLimit {
        member_count: i32,
        maximum: i32,
        allocated_net_exchange_id: i32,
        notice_sent: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionApplyForJoinOutcome<SessionReport> {
    Rejected(UnionApplyForJoinRejection),
    Started {
        applicant_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionApplyForJoinBlock<MembershipBlock, SessionBlock> {
    PendingApplicationUninitialized,
    MembershipScan(MembershipBlock),
    FactionSnapshot(UnionApplicationFactionBlock),
    Session {
        source: SessionBlock,
        applicant_faction_id: i32,
        net_exchange_id: i32,
        application_assigned: bool,
        establishment_reserved: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionFactionPlayerRefreshReport {
    pub faction_id: i32,
    pub refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionPlayerRefreshReport {
    pub factions: Vec<UnionFactionPlayerRefreshReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionInitialReport {
    pub master_faction_found: bool,
    pub superior_assigned: bool,
    pub player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionInitialBlock {
    MasterTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
    },
    MissingFactionLevel(UnionFactionLevelBlock),
    SuperiorOrganizing {
        faction_id: i32,
        source: FactionSuperiorOrganizingBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionAddFactionReport {
    pub faction_id: i32,
    pub replaced_existing_member: bool,
    pub superior_assigned: bool,
    pub owned_city_refresh: FactionOwnedCityRefreshReport,
    pub member_information: UnionInfoFanoutReport,
    pub player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionAddFactionOutcome {
    Added(UnionAddFactionReport),
    FactionUnavailable {
        faction_id: i32,
        war_log: Vec<u8>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionAddFactionBlock {
    MemberTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
    },
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_length: usize,
        capacity: usize,
    },
    MissingFactionLevel(UnionFactionLevelBlock),
    SuperiorOrganizing {
        faction_id: i32,
        member_inserted: bool,
        source: FactionSuperiorOrganizingBlock,
    },
    OwnedCityRefresh {
        faction_id: i32,
        member_inserted: bool,
        superior_assigned: bool,
        source: FactionOwnedCityRefreshBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionDoJoinRejection {
    MemberLimit {
        member_count: i32,
        maximum: i32,
    },
    InvalidApplication {
        manager_id: i32,
        applicant_faction_id: i32,
        pending_applicant_faction_id: Option<i32>,
    },
    ApplicantAlreadyInUnion {
        applicant_faction_id: i32,
        union_id: i32,
    },
    OperatorNotAuthorized,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionDoJoinReport {
    pub add_faction: UnionAddFactionOutcome,
    pub initial_snapshot_legacy_result: bool,
    pub member_update: UnionMemberUpdateReport,
    pub dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDoJoinOutcome {
    Rejected {
        reason: UnionDoJoinRejection,
        application_cleared: bool,
    },
    Joined(UnionDoJoinReport),
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDoJoinBlock<FreeFactionBlock, OperatorBlock, InitialSnapshotBlock> {
    FreeFactionScan {
        source: FreeFactionBlock,
        application_cleared: bool,
    },
    OperatorValidation {
        source: OperatorBlock,
        application_cleared: bool,
    },
    AddFaction {
        source: UnionAddFactionBlock,
        application_cleared: bool,
    },
    InitialSnapshot {
        source: InitialSnapshotBlock,
        application_cleared: bool,
        add_faction: UnionAddFactionOutcome,
    },
    MemberUpdate {
        source: UnionMemberUpdateBlock,
        application_cleared: bool,
        add_faction: UnionAddFactionOutcome,
        initial_snapshot_legacy_result: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionFireOutRejection {
    StandardWar,
    CityWar,
    OperatorValidationFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionFireOutReport<DetachOutcome> {
    pub member_information: UnionInfoFanoutReport,
    pub detach: DetachOutcome,
    pub member_removed: bool,
    pub delete_organizing: UnionDeleteOrganizingReport,
    pub member_update: UnionMemberUpdateReport,
    pub player_refresh: UnionPlayerRefreshReport,
    pub war_log: Option<Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionFireOutOutcome<DetachOutcome> {
    Rejected(UnionFireOutRejection),
    Fired(UnionFireOutReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionFireOutBlock<OperatorBlock, DetachBlock> {
    OperatorValidation(OperatorBlock),
    UnterminatedTargetName {
        target_faction_id: i32,
        source: UnterminatedMemberField,
    },
    Detach {
        target_faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
    },
    MemberUpdate {
        target_faction_id: i32,
        source: UnionMemberUpdateBlock,
        member_information: UnionInfoFanoutReport,
        member_removed: bool,
        delete_organizing: UnionDeleteOrganizingReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionDemiseRejection {
    MemberCountLimit {
        member_count: usize,
        limit: usize,
    },
    LegacyIdentifiersEqual,
    OldMasterFactionNotFound,
    ZeroNewMasterFaction,
    Cooldown {
        elapsed_ms: u32,
        minimum_ms: u32,
        notice_sent: bool,
    },
    CityWar {
        notice_sent: bool,
    },
    StandardWar {
        notice_sent: bool,
    },
    OldFactionIsNotCurrentMaster {
        old_faction_id: i32,
        current_master_faction_id: i32,
    },
    OldMemberNotFound {
        old_faction_id: i32,
    },
    NewMemberNotFound {
        new_faction_id: i32,
    },
    OldMasterOffline,
    OldMasterOperatingFactionWar,
    CurrentMasterFactionMissing {
        faction_id: i32,
    },
    NewMasterFactionMissing {
        faction_id: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionDemiseReport {
    pub old_faction_id: i32,
    pub new_faction_id: i32,
    pub old_member_update: UnionMemberUpdateReport,
    pub new_member_update: UnionMemberUpdateReport,
    pub member_information: UnionInfoFanoutReport,
    pub player_refresh: UnionPlayerRefreshReport,
    pub war_log: Option<Vec<u8>>,
    pub completed_at_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDemiseOutcome {
    Rejected(UnionDemiseRejection),
    Transferred(UnionDemiseReport),
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDemiseBlock<OperatorBlock> {
    MasterLookup(OperatorBlock),
    UnterminatedMemberField {
        member_id: i32,
        source: UnterminatedMemberField,
    },
    DemotedTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
        master_changed: bool,
    },
    MemberUpdate {
        target_faction_id: i32,
        source: UnionMemberUpdateBlock,
        old_member_update: Option<UnionMemberUpdateReport>,
    },
    InformationWouldOverflow {
        visible_length: usize,
        capacity: usize,
        old_member_update: UnionMemberUpdateReport,
        new_member_update: UnionMemberUpdateReport,
    },
    WarLogWouldOverflow {
        visible_length: usize,
        capacity: usize,
        old_member_update: UnionMemberUpdateReport,
        new_member_update: UnionMemberUpdateReport,
        member_information: UnionInfoFanoutReport,
        player_refresh: UnionPlayerRefreshReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionExitRejection {
    StandardWar,
    CityWar,
    OperatorValidationFailed,
    FactionMasterNotFound,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionExitReport<DetachOutcome> {
    pub leaving_faction_id: i32,
    pub member_information: UnionInfoFanoutReport,
    pub war_log: Option<Vec<u8>>,
    pub detach: DetachOutcome,
    pub member_removed: bool,
    pub delete_organizing: UnionDeleteOrganizingReport,
    pub member_update: UnionMemberUpdateReport,
    pub player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionExitOutcome<DetachOutcome> {
    Rejected(UnionExitRejection),
    Exited(UnionExitReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionExitBlock<OperatorBlock, DetachBlock> {
    OperatorValidation(OperatorBlock),
    MasterLookup(OperatorBlock),
    MissingLeavingMember { leaving_faction_id: i32 },
    UnterminatedLeavingName {
        leaving_faction_id: i32,
        source: UnterminatedMemberField,
    },
    Detach {
        leaving_faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Option<Vec<u8>>,
    },
    MemberUpdate {
        leaving_faction_id: i32,
        source: UnionMemberUpdateBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Option<Vec<u8>>,
        member_removed: bool,
        delete_organizing: UnionDeleteOrganizingReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionDisbandRejection {
    OperatorValidationFailed,
    War,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionDisbandReport<DetachOutcome> {
    pub member_information: UnionInfoFanoutReport,
    pub war_log: Vec<u8>,
    pub detached_members: Vec<(i32, DetachOutcome)>,
    pub application_cleared: bool,
    pub delete_organizing: UnionDeleteOrganizingReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDisbandOutcome<DetachOutcome> {
    Rejected(UnionDisbandRejection),
    Disbanded(UnionDisbandReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionDisbandBlock<OperatorBlock, DetachBlock, DetachOutcome> {
    OperatorValidation(OperatorBlock),
    Detach {
        faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Vec<u8>,
        completed: Vec<(i32, DetachOutcome)>,
    },
}

pub trait UnionClientSnapshotContext {
    fn faction_update_enemy_snapshot(
        &self,
        faction_id: i32,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_city_war_enemy_snapshot(
        &self,
        faction_id: i32,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_owned_city_snapshot(
        &self,
        faction_id: i32,
    ) -> Result<Option<Vec<FactionOwnedCityDelivery>>, FactionOwnedCityUpdateBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionEnemySnapshotFactionReport {
    pub faction_id: i32,
    pub deliveries: Vec<FactionEnemyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionEnemySnapshotReport {
    pub factions: Vec<UnionEnemySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCitySnapshotFactionReport {
    pub faction_id: i32,
    pub deliveries: Vec<FactionOwnedCityDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCitySnapshotReport {
    pub factions: Vec<UnionOwnedCitySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionOwnedCitySnapshotBlock {
    pub faction_id: i32,
    pub completed_factions: Vec<UnionOwnedCitySnapshotFactionReport>,
    pub source: FactionOwnedCityUpdateBuildError,
}

pub trait UnionSendInfoContext {
    fn faction_send_info_to_members<'a>(
        &self,
        faction_id: i32,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        send_organizing_info: &mut dyn FnMut(FactionMemberInfoRequest<'a>),
    ) -> Option<FactionMemberInfoReport>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionFactionInfoFanout {
    pub faction_id: i32,
    pub recipient_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionInfoFanoutReport {
    pub factions: Vec<UnionFactionInfoFanout>,
}

pub trait UnionFactionMemberContext {
    fn faction_member_player_ids(&self, faction_id: i32) -> Option<Vec<i32>>;

    fn faction_name(&self, faction_id: i32) -> Option<Vec<u8>>;

    fn faction_level(
        &self,
        faction_id: i32,
    ) -> Result<Option<i32>, UnionFactionLevelBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionFactionLevelBlock {
    pub faction_id: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionMemberSnapshotBlock {
    UnterminatedMemberField {
        member_id: i32,
        field: UnterminatedMemberField,
        completed_members: usize,
    },
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_length: usize,
        capacity: usize,
        completed_members: usize,
    },
    MissingFactionLevel {
        source: UnionFactionLevelBlock,
        completed_members: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionDeleteOrganizingDelivery {
    pub faction_id: i32,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionDeleteOrganizingReport {
    pub visited_faction_ids: Vec<i32>,
    pub deliveries: Vec<UnionDeleteOrganizingDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionMemberUpdateDelivery {
    pub recipient_faction_id: i32,
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnionMemberUpdateReport {
    pub target_found: Option<bool>,
    pub target_level: Option<i32>,
    pub deliveries: Vec<UnionMemberUpdateDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionMemberUpdateBlock {
    MissingFactionLevel {
        source: UnionFactionLevelBlock,
        target_level: i32,
        completed_deliveries: Vec<UnionMemberUpdateDelivery>,
    },
    UnterminatedField {
        field: UnterminatedMemberField,
        target_level: i32,
        recipient_faction_id: i32,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<UnionMemberUpdateDelivery>,
    },
}

pub struct CUnion {
    union_id: i32,
    name: Vec<u8>,
    master_id: i32,
    members: BTreeMap<i32, TagMemInfo>,
    established_time: TagTimeValue,
    apply_person: Option<i32>,
    change_data_type: i32,
    last_demise_time_ms: u32,
}

impl CUnion {
 /// Создаёт действующую инфраструктурную часть private `CUnion::CUnion`.
 ///
 /// Старая функция конструирует пустые string/map и `tagTime`, но не пишет
 /// scalar-поля. Последующее чтение таких полей было внутренним UB, поэтому
 /// safe baseline задаёт нейтральные значения. Время приходит от concrete
 /// фабрики, сохраняя её исходный момент снятия local clock.
    fn with_private_constructor_defaults(established_time: TagTimeValue) -> Self {
        Self {
            union_id: 0,
            name: Vec::new(),
            master_id: 0,
            members: BTreeMap::new(),
            established_time,
            apply_person: None,
            change_data_type: 0,
            last_demise_time_ms: 0,
        }
    }

 /// Строит concrete union на DB-пути `LoadAllConfederation`.
 ///
 /// Public constructor оригинала вызывает `Initial` ещё до
 /// `LoadConfeMembers`: поэтому `m_ApplyPerson` становится `0`, а в map
 /// появляется fallback master-member с `WS0154` и current local time.
 /// Переданные DB members затем заменяют соответствующие ключи, как старый
 /// `map::operator[]`. Faction-map на этой стадии ещё пуста, поэтому
 /// constructor не меняет имя/union ID живой faction и не делает player
 /// refresh; эти side effect старого null lookup отсутствуют.
    pub fn from_database_load_state(
        union_id: i32,
        name: Vec<u8>,
        master_id: i32,
        master_title: &[u8],
        members: BTreeMap<i32, TagMemInfo>,
    ) -> Result<Self, UnionInitialBlock> {
        let visible_title = legacy_c_string_visible_bytes(master_title);
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionInitialBlock::MasterTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);
        let established_time = current_local_member_time();
        let fallback_master = TagMemInfo::from_complete_fields(
            master_id,
            [0; 32],
            0,
            0,
            1,
            title,
            [
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            established_time,
            false,
        );
        let mut union = Self::with_private_constructor_defaults(established_time);
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        union.members.insert(master_id, fallback_master);
        union.members.extend(members);
        union.apply_person = Some(0);
        Ok(union)
    }

 /// Строит live-union и выполняет исходный `Initial` с явными callbacks.
 ///
 /// При safe-блокировке возвращает сам частично инициализированный owner,
 /// чтобы caller не терял уже выполненные внешние эффекты.
    pub fn from_live_state<Context>(
        union_id: i32,
        master_id: i32,
        name: Vec<u8>,
        master_title: Option<&[u8]>,
        context: &mut Context,
        parameters: &COrganizingParam,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<(Self, UnionInitialReport), (Self, UnionInitialBlock)>
    where
        Context:
            UnionFactionMemberContext + UnionInitialMutationContext + UnionPlayerRefreshContext,
    {
        let mut union = Self::with_private_constructor_defaults(current_local_member_time());
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        match union.initial_live(master_title, context, parameters, update_player) {
            Ok(report) => Ok((union, report)),
            Err(source) => Err((union, source)),
        }
    }

    pub fn from_reached_save_state(
        union_id: i32,
        name: Vec<u8>,
        master_id: i32,
        members: BTreeMap<i32, TagMemInfo>,
        established_time: TagTimeValue,
        change_data_type: i32,
    ) -> Self {
        let mut union = Self::with_private_constructor_defaults(established_time);
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        union.members = members;
        union.change_data_type = change_data_type;
        union
    }

    pub fn initial_live<Context>(
        &mut self,
        master_title: Option<&[u8]>,
        context: &mut Context,
        parameters: &COrganizingParam,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionInitialReport, UnionInitialBlock>
    where
        Context:
            UnionFactionMemberContext + UnionInitialMutationContext + UnionPlayerRefreshContext,
    {
        self.apply_person = Some(0);
        let member_time = current_local_member_time();

        let visible_title = legacy_c_string_visible_bytes(master_title.unwrap_or_default());
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionInitialBlock::MasterTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);

        let master_name = if self.master_id > 0 {
            context.faction_name(self.master_id)
        } else {
            None
        };
        let master_faction_found = master_name.is_some();
        let level = if master_faction_found {
            match context.faction_level(self.master_id) {
                Ok(Some(level)) => level,
                Ok(None) => 0,
                Err(source) => return Err(UnionInitialBlock::MissingFactionLevel(source)),
            }
        } else {
            0
        };
        if let Some(master_name) = master_name {
            self.name = master_name;
        }

        let member = TagMemInfo::from_complete_fields(
            self.master_id,
            [0; 32],
            level,
            0,
            1,
            title,
            [
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            member_time,
            false,
        );
        self.members.insert(self.master_id, member);

        let superior_assigned = if master_faction_found {
            context
                .faction_set_superior_organizing(
                    self.master_id,
                    self.union_id,
                    self.master_id,
                    parameters,
                )
                .map_err(|source| UnionInitialBlock::SuperiorOrganizing {
                    faction_id: self.master_id,
                    source,
                })?
        } else {
            false
        };
        self.change_data_type = 0;
        let player_refresh = self.update_player_faction_info(
            self.master_id,
            context,
            update_player,
        );
        Ok(UnionInitialReport {
            master_faction_found,
            superior_assigned,
            player_refresh,
        })
    }

    pub fn clone_save_data(&self) -> Option<Self> {
        (self.change_data_type != 0).then(|| Self {
            union_id: self.union_id,
            name: self.name.clone(),
            master_id: self.master_id,
            members: self.members.clone(),
            established_time: self.established_time,
            apply_person: None,
            change_data_type: self.change_data_type,
            last_demise_time_ms: 0,
        })
    }

 /// Вызывает DB-owner с live union и игнорирует его результат.
 ///
 /// Оригинал разрешал `CGame::m_pRsConfederation`, один раз вызывал
 /// `SaveConfederation(this)` и всегда возвращал `true`. Rust передаёт
 /// внешний DB-owner явным one-shot callback-ом; snapshot-фаза `savedb`
 /// остаётся отдельным асинхронным owner-ом.
    pub fn save(&self, save_confederation: impl FnOnce(&Self)) -> bool {
        save_confederation(self);
        true
    }

    pub fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    pub const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub const fn master_id(&self) -> i32 {
        self.master_id
    }

    pub const fn established_time(&self) -> TagTimeValue {
        self.established_time
    }

    pub const fn apply_person(&self) -> Option<i32> {
        self.apply_person
    }

    pub fn finish_union_application_callback(&mut self) {
        self.apply_person = Some(0);
    }

    pub fn apply_for_join<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        applicant_faction_id: i32,
        _second_parameter: i32,
        _third_parameter: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionApplyForJoinOutcome<Effects::SessionReport>,
        UnionApplyForJoinBlock<
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            Effects::SessionBlock,
        >,
    >
    where
        Context: UnionApplyForJoinContext + UnionDoJoinContext,
        Effects: UnionApplyForJoinEffects,
    {
        let Some(pending_application) = self.apply_person else {
            return Err(UnionApplyForJoinBlock::PendingApplicationUninitialized);
        };
        if pending_application >= 1 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::PendingApplication,
            ));
        }
        if applicant_faction_id == 0 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ZeroApplicantFactionId,
            ));
        }
        if context.union_application_is_reserved(applicant_faction_id) {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantAlreadyReserved,
            ));
        }

        let existing_union_id = context
            .union_id_for_joining_faction(applicant_faction_id)
            .map_err(UnionApplyForJoinBlock::MembershipScan)?;
        if existing_union_id > 0 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantAlreadyInUnion {
                    union_id: existing_union_id,
                },
            ));
        }

        let applicant = context
            .union_application_faction(applicant_faction_id)
            .map_err(UnionApplyForJoinBlock::FactionSnapshot)?;
        let Some(applicant) = applicant else {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantFactionMissing,
            ));
        };
        let master = context
            .union_application_faction(self.master_id)
            .map_err(UnionApplyForJoinBlock::FactionSnapshot)?;
        let Some(master) = master else {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MasterFactionMissing,
            ));
        };

        let Some(master_player) = game.online_player_by_id(master.player_header as u32) else {
            send_union_application_notice(
                effects,
                applicant.player_header,
                b"WS0264",
            );
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MasterPlayerOffline { notice_sent: true },
            ));
        };

        let net_exchange_id = master_player.get_net_exchange_id();
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            send_union_application_notice(
                effects,
                applicant.player_header,
                b"WS0265",
            );
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                    allocated_net_exchange_id: net_exchange_id,
                    notice_sent: true,
                },
            ));
        }

        self.apply_person = Some(applicant_faction_id);
        context.reserve_union_application(applicant_faction_id);
        let request = UnionApplicationSessionRequest {
            union_id: self.union_id,
            applicant_faction_id,
            recipient_player_id: master.player_header,
            requested_session_id: net_exchange_id,
            timeout_ticks: 1_000,
            confirmation_kind: 2,
            applicant_faction_name: applicant.name,
        };
        let session = effects
            .begin_union_application_session(request)
            .map_err(|source| UnionApplyForJoinBlock::Session {
                source,
                applicant_faction_id,
                net_exchange_id,
                application_assigned: true,
                establishment_reserved: true,
            })?;

        Ok(UnionApplyForJoinOutcome::Started {
            applicant_faction_id,
            net_exchange_id,
            session,
        })
    }

    pub fn invite<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionInviteOutcome<Effects::SessionReport>,
        UnionInviteBlock<
            <Context as UnionOperatorValidationContext>::Block,
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            Effects::SessionBlock,
        >,
    >
    where
        Context: UnionOperatorValidationContext
            + UnionApplyForJoinContext
            + UnionDoJoinContext,
        Effects: UnionInviteEffects,
    {
        let Some(pending_application) = self.apply_person else {
            return Err(UnionInviteBlock::PendingApplicationUninitialized);
        };
        if pending_application > 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::PendingApplication,
            ));
        }
        if inviter_faction_id == 0 || invited_faction_id == 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::ZeroFactionId,
            ));
        }
        if context.union_application_is_reserved(invited_faction_id) {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedFactionReserved,
            ));
        }

 // caller передаёт сюда faction ID, хотя validation owner
 // интерпретирует аргумент как master-player ID. Наблюдаемый gate
 // сохраняется без подмены исходным player ID ingress-а.
        let permitted = self
            .check_operator_validate(inviter_faction_id, EPurview::ConMem as i32, context)
            .map_err(UnionInviteBlock::Operator)?;
        if !permitted {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InviterNotPermitted,
            ));
        }

        let existing_union_id = context
            .union_id_for_joining_faction(invited_faction_id)
            .map_err(UnionInviteBlock::Membership)?;
        if existing_union_id > 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedAlreadyInUnion {
                    union_id: existing_union_id,
                },
            ));
        }

        let inviter = context
            .union_application_faction(inviter_faction_id)
            .map_err(UnionInviteBlock::FactionSnapshot)?;
        let Some(inviter) = inviter else {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InviterFactionMissing,
            ));
        };
        let invited = context
            .union_application_faction(invited_faction_id)
            .map_err(UnionInviteBlock::FactionSnapshot)?;
        let Some(invited) = invited else {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedFactionMissing,
            ));
        };

        let Some(invited_master) = game.online_player_by_id(invited.player_header as u32) else {
            send_union_invitation_notice(effects, inviter.player_header, b"WS0264");
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedMasterOffline { notice_sent: true },
            ));
        };
        let net_exchange_id = invited_master.get_net_exchange_id();
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            send_union_invitation_notice(effects, inviter.player_header, b"WS0265");
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                    allocated_net_exchange_id: net_exchange_id,
                    notice_sent: true,
                },
            ));
        }

        self.apply_person = Some(invited_faction_id);
        context.reserve_union_application(invited_faction_id);
        let session = effects
            .begin_union_invitation_session(UnionInvitationSessionRequest {
                union_id: self.union_id,
                inviter_faction_id,
                invited_faction_id,
                recipient_player_id: invited.player_header,
                requested_session_id: net_exchange_id,
                timeout_ticks: 1_000,
                inviter_faction_name: inviter.name,
            })
            .map_err(|source| UnionInviteBlock::Session {
                source,
                invited_faction_id,
                net_exchange_id,
                application_assigned: true,
                establishment_reserved: true,
            })?;

        Ok(UnionInviteOutcome::Started {
            invited_faction_id,
            net_exchange_id,
            session,
        })
    }

    pub fn clear_apply_list<Context>(
        &mut self,
        manager_player_id: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        if !self.check_operator_validate(
            manager_player_id,
            EPurview::ConMem as i32,
            context,
        )? {
            return Ok(false);
        }
        self.apply_person = Some(0);
        Ok(true)
    }

    pub fn add_faction<Context, Effects>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        parameters: &COrganizingParam,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionAddFactionOutcome, UnionAddFactionBlock>
    where
        Context: UnionFactionJoinContext,
        Effects: UnionAddFactionEffects,
    {
        let member_time = current_local_member_time();
        let member_title = effects.world_string(b"WS0268");
        let visible_title = legacy_c_string_visible_bytes(&member_title);
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionAddFactionBlock::MemberTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);

        let faction_name = if faction_id > 0 {
            context.faction_name(faction_id)
        } else {
            None
        };
        let Some(faction_name) = faction_name else {
            let war_log = effects.format_world_string(
                b"WS0269",
                &[
                    UnionFormatArgument::Signed(faction_id),
                    UnionFormatArgument::Signed(self.union_id),
                ],
            );
            let war_log = legacy_c_string_visible_bytes(&war_log).to_vec();
            effects.put_war_log(&war_log);
            return Ok(UnionAddFactionOutcome::FactionUnavailable {
                faction_id,
                war_log,
            });
        };

        let visible_faction_name = legacy_c_string_visible_bytes(&faction_name);
        let mut fixed_name = [0; 32];
        if visible_faction_name.len() >= fixed_name.len() {
            return Err(UnionAddFactionBlock::FactionNameWouldOverflow {
                faction_id,
                visible_length: visible_faction_name.len(),
                capacity: fixed_name.len(),
            });
        }
        fixed_name[..visible_faction_name.len()].copy_from_slice(visible_faction_name);
        let level = match context.faction_level(faction_id) {
            Ok(Some(level)) => level,
            Ok(None) => {
                return Err(UnionAddFactionBlock::MissingFactionLevel(
                    UnionFactionLevelBlock { faction_id },
                ));
            }
            Err(source) => return Err(UnionAddFactionBlock::MissingFactionLevel(source)),
        };

        let member = TagMemInfo::from_complete_fields(
            faction_id,
            fixed_name,
            level,
            0,
            99,
            title,
            [
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            member_time,
            false,
        );
        let replaced_existing_member = self.members.insert(faction_id, member).is_some();

        let superior_assigned = context
            .faction_set_superior_organizing(
                faction_id,
                self.union_id,
                self.master_id,
                parameters,
            )
            .map_err(|source| UnionAddFactionBlock::SuperiorOrganizing {
                faction_id,
                member_inserted: true,
                source,
            })?;
        let mut refresh_owned_city = |region_id, owner_faction_id, owner_union_id, country_id| {
            effects.refresh_owned_city(
                region_id,
                owner_faction_id,
                owner_union_id,
                country_id,
            );
        };
        let owned_city_refresh = context
            .faction_refresh_owned_city_info(faction_id, &mut refresh_owned_city)
            .map_err(|source| UnionAddFactionBlock::OwnedCityRefresh {
                faction_id,
                member_inserted: true,
                superior_assigned,
                source,
            })?
            .unwrap_or(FactionOwnedCityRefreshReport {
                refreshed_region_ids: Vec::new(),
            });

        let information = effects.format_world_string(
            b"WS0270",
            &[
                UnionFormatArgument::Text(visible_faction_name),
                UnionFormatArgument::Text(legacy_c_string_visible_bytes(&self.name)),
            ],
        );
        let information = legacy_c_string_visible_bytes(&information);
        let information_title = effects.world_string(b"WS0188");
        let information_title = legacy_c_string_visible_bytes(&information_title);
 // Машинный факт общего кадра 0x7F804: цвет рассылки всегда 0xFFDAEDFE.
 // `CUnion::SendInfoToAllMember` (RVA 0x4C6290) лишь пробрасывает K через
 // два уровня до `CFaction::SendInfoToAllMember` (RVA 0x4B5890), где
 // литерал 0xFFDAEDFE его убивает — K мёртв в обоих классах.
        let member_information = self.send_info_to_all_members(
            information,
            information_title,
            -1,
            0xFFDA_EDFE,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        let player_refresh = self.update_player_faction_info(
            faction_id,
            context,
            update_player,
        );
        Ok(UnionAddFactionOutcome::Added(UnionAddFactionReport {
            faction_id,
            replaced_existing_member,
            superior_assigned,
            owned_city_refresh,
            member_information,
            player_refresh,
        }))
    }

 /// Принимает pending faction-заявку в точном порядке `CUnion::DoJoin`.
 ///
 /// сборке не относится. После совпадения заявки она сбрасывается до
 /// membership/authority-проверок. `approve_flag` и `join_time` исходная
 /// функция не читала. Результат initial snapshot игнорируется, но
 /// сохраняется в отчёте; safe-блокировки malformed state не продолжают
 /// путь после места старого UB.
    pub fn do_join<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_faction_id: i32,
        _approve_flag: i32,
        _join_time: TagTimeValue,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionDoJoinOutcome,
        UnionDoJoinBlock<
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            <Context as UnionOperatorValidationContext>::Block,
            <Context as UnionDoJoinContext>::InitialSnapshotBlock,
        >,
    >
    where
        Context: UnionFactionJoinContext
            + UnionOperatorValidationContext
            + UnionDoJoinContext,
        Effects: UnionAddFactionEffects,
    {
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                },
                application_cleared: false,
            });
        }
        if manager_id == 0
            || applicant_faction_id == 0
            || self.apply_person != Some(applicant_faction_id)
        {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::InvalidApplication {
                    manager_id,
                    applicant_faction_id,
                    pending_applicant_faction_id: self.apply_person,
                },
                application_cleared: false,
            });
        }

        self.apply_person = Some(0);
        let existing_union_id = context
            .union_id_for_joining_faction(applicant_faction_id)
            .map_err(|source| UnionDoJoinBlock::FreeFactionScan {
                source,
                application_cleared: true,
            })?;
        if existing_union_id > 0 {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::ApplicantAlreadyInUnion {
                    applicant_faction_id,
                    union_id: existing_union_id,
                },
                application_cleared: true,
            });
        }

        let operator_authorized = self
            .check_operator_validate(manager_id, EPurview::ConMem as i32, context)
            .map_err(|source| UnionDoJoinBlock::OperatorValidation {
                source,
                application_cleared: true,
            })?;
        if !operator_authorized {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::OperatorNotAuthorized,
                application_cleared: true,
            });
        }

        let add_faction = self
            .add_faction(
                applicant_faction_id,
                context,
                effects,
                parameters,
                update_player,
            )
            .map_err(|source| UnionDoJoinBlock::AddFaction {
                source,
                application_cleared: true,
            })?;
        let initial_snapshot_legacy_result =
            match context.add_current_union_to_client_by_faction_id(self, applicant_faction_id) {
                Ok(result) => result,
                Err(source) => {
                    return Err(UnionDoJoinBlock::InitialSnapshot {
                        source,
                        application_cleared: true,
                        add_faction,
                    });
                }
            };
        let member_update = match self.update_member_info_to_client(
            game,
            applicant_faction_id,
            EOperator::Add,
            context,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(UnionDoJoinBlock::MemberUpdate {
                    source,
                    application_cleared: true,
                    add_faction,
                    initial_snapshot_legacy_result,
                });
            }
        };
        self.set_change_data(2);

        Ok(UnionDoJoinOutcome::Joined(UnionDoJoinReport {
            add_faction,
            initial_snapshot_legacy_result,
            member_update,
            dirty_set: true,
        }))
    }

    pub fn demise<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        old_master_player_id: i32,
        new_master_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        get_tick: &mut dyn FnMut() -> u32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionDemiseOutcome, UnionDemiseBlock<Context::Block>>
    where
        Context: UnionDemiseContext,
        Effects: UnionFireOutEffects,
    {
        if self.members.len() >= UNION_DEMISE_MEMBER_LIMIT {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::MemberCountLimit {
                    member_count: self.members.len(),
                    limit: UNION_DEMISE_MEMBER_LIMIT,
                },
            ));
        }
 // сравнивает player ID с faction ID. Разные домены здесь
 // намеренно не "исправляются": результат наблюдаем клиентом.
        if old_master_player_id == new_master_faction_id {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::LegacyIdentifiersEqual,
            ));
        }

        let old_faction_id = context
            .faction_id_by_master_player(old_master_player_id)
            .map_err(UnionDemiseBlock::MasterLookup)?;
        if old_faction_id == 0 {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterFactionNotFound,
            ));
        }
        if new_master_faction_id == 0 {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::ZeroNewMasterFaction,
            ));
        }

        let now_ms = get_tick();
        let elapsed_ms = now_ms.wrapping_sub(self.last_demise_time_ms);
        if elapsed_ms < UNION_DEMISE_COOLDOWN_MS {
            send_union_demise_notice(effects, old_master_player_id, b"WS0353");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::Cooldown {
                    elapsed_ms,
                    minimum_ms: UNION_DEMISE_COOLDOWN_MS,
                    notice_sent: true,
                },
            ));
        }
        if self.has_city_war_enemy_faction(context) {
            send_union_demise_notice(effects, old_master_player_id, b"WS0279");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::CityWar { notice_sent: true },
            ));
        }
        if self.has_enemy_faction(context) {
            send_union_demise_notice(effects, old_master_player_id, b"WS0280");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::StandardWar { notice_sent: true },
            ));
        }
        if old_faction_id != self.master_id {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldFactionIsNotCurrentMaster {
                    old_faction_id,
                    current_master_faction_id: self.master_id,
                },
            ));
        }
        if !self.members.contains_key(&new_master_faction_id) {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::NewMemberNotFound {
                    new_faction_id: new_master_faction_id,
                },
            ));
        }

        let Some(old_master_player) = game.online_player_by_id(old_master_player_id as u32) else {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterOffline,
            ));
        };
        if old_master_player.faction_war_operator() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterOperatingFactionWar,
            ));
        }
        if context.faction_name(self.master_id).is_none() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::CurrentMasterFactionMissing {
                    faction_id: self.master_id,
                },
            ));
        }
        if context.faction_name(new_master_faction_id).is_none() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::NewMasterFactionMissing {
                    faction_id: new_master_faction_id,
                },
            ));
        }

 // `IsMaster(old)` не гарантирует наличие повреждённого master-key в
 // map. Старый operator[] тогда создавал частично неинициализированный
 // tagMemInfo; safe owner останавливает этот внутренний UB.
        let Some(old_member) = self.members.get(&old_faction_id) else {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMemberNotFound { old_faction_id },
            ));
        };
        let old_title = old_member
            .title_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: old_faction_id,
                source,
            })?
            .to_vec();
        let old_name = old_member
            .name_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: old_faction_id,
                source,
            })?;
        let old_name = old_name[..old_name.len() - 1].to_vec();
        let new_member = self
            .members
            .get(&new_master_faction_id)
            .expect("IsMember(new faction) проверен выше");
        let new_name = new_member
            .name_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: new_master_faction_id,
                source,
            })?;
        let new_name = new_name[..new_name.len() - 1].to_vec();
        let old_purview = old_member.purview;
        let old_job_level = old_member.job_level;

        self.master_id = new_master_faction_id;
        let member_faction_ids = self.members.keys().copied().collect::<Vec<_>>();
        context.set_union_master_projection(&member_faction_ids, new_master_faction_id);
        {
            let new_member = self
                .members
                .get_mut(&new_master_faction_id)
                .expect("new member существует до мутации");
            new_member.title[..old_title.len()].copy_from_slice(&old_title);
            new_member.purview = old_purview;
            new_member.job_level = old_job_level;
        }
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member проверен до мутации");
            old_member.job_level = 99;
            old_member.id = old_faction_id;
        }

        let demoted_title = effects.world_string(b"WS0268");
        let demoted_title = legacy_c_string_visible_bytes(&demoted_title);
        let title_capacity = self
            .members
            .get(&old_faction_id)
            .expect("old member существует при title assignment")
            .title
            .len();
        if demoted_title.len() >= title_capacity {
            return Err(UnionDemiseBlock::DemotedTitleWouldOverflow {
                visible_length: demoted_title.len(),
                capacity: title_capacity,
                master_changed: true,
            });
        }
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member существует при demotion");
            old_member.title[..demoted_title.len()].copy_from_slice(demoted_title);
            old_member.title[demoted_title.len()] = 0;
        }
        self.name = new_name.clone();
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member существует при purview demotion");
            old_member.purview = [
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ];
        }

        let old_member_update = self
            .update_member_info_to_client(game, old_faction_id, EOperator::Update, context)
            .map_err(|source| UnionDemiseBlock::MemberUpdate {
                target_faction_id: old_faction_id,
                source,
                old_member_update: None,
            })?;
        let new_member_update = match self.update_member_info_to_client(
            game,
            new_master_faction_id,
            EOperator::Update,
            context,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(UnionDemiseBlock::MemberUpdate {
                    target_faction_id: new_master_faction_id,
                    source,
                    old_member_update: Some(old_member_update),
                });
            }
        };
        self.set_change_data(3);

        let information = effects.format_world_string(
            b"WS0281",
            &[
                UnionFormatArgument::Text(&old_name),
                UnionFormatArgument::Text(&new_name),
            ],
        );
        let information = legacy_c_string_visible_bytes(&information);
        let title = effects.world_string(b"WS0188");
 // Машинный факт общего кадра 0x7F804: цвет рассылки всегда 0xFFDAEDFE.
 // `CUnion::SendInfoToAllMember` (RVA 0x4C6290) лишь пробрасывает K через
 // два уровня до `CFaction::SendInfoToAllMember` (RVA 0x4B5890), где
 // литерал 0xFFDAEDFE его убивает — K мёртв в обоих классах.
        let member_information = self.send_info_to_all_members(
            information,
            legacy_c_string_visible_bytes(&title),
            -1,
            0xFFDA_EDFE,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        let player_refresh = self.update_player_faction_info(0, context, update_player);

 // EXE после player refresh дважды повторяет nullable faction lookup.
 // После назначения master оба lookup относятся к одной new faction.
        let war_log = if context.faction_name(new_master_faction_id).is_some()
            && context.faction_name(self.master_id).is_some()
        {
            let war_log = effects.format_world_string(
                b"WS0282",
                &[
                    UnionFormatArgument::Text(&old_name),
                    UnionFormatArgument::Signed(old_faction_id),
                    UnionFormatArgument::Text(&new_name),
                    UnionFormatArgument::Signed(new_master_faction_id),
                ],
            );
            let war_log = legacy_c_string_visible_bytes(&war_log).to_vec();
            effects.put_war_log(&war_log);
            Some(war_log)
        } else {
            None
        };
        let completed_at_ms = get_tick();
        self.last_demise_time_ms = completed_at_ms;

        Ok(UnionDemiseOutcome::Transferred(UnionDemiseReport {
            old_faction_id,
            new_faction_id: new_master_faction_id,
            old_member_update,
            new_member_update,
            member_information,
            player_refresh,
            war_log,
            completed_at_ms,
        }))
    }

    pub fn fire_out<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionFireOutOutcome<Context::DetachOutcome>,
        UnionFireOutBlock<Context::Block, Context::DetachBlock>,
    >
    where
        Context: UnionFireOutContext,
        Effects: UnionFireOutEffects,
    {
        if self.has_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0271");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionFireOutOutcome::Rejected(
                UnionFireOutRejection::StandardWar,
            ));
        }
        if self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0272");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionFireOutOutcome::Rejected(UnionFireOutRejection::CityWar));
        }

        let operation_valid = self
            .check_operator_validate_target(
                manager_id,
                target_faction_id,
                EPurview::FireOut as i32,
                context,
            )
            .map_err(UnionFireOutBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionFireOutOutcome::Rejected(
                UnionFireOutRejection::OperatorValidationFailed,
            ));
        }

        let target = self
            .members
            .get(&target_faction_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = target.name_wire_bytes().map_err(|source| {
            UnionFireOutBlock::UnterminatedTargetName {
                target_faction_id,
                source,
            }
        })?;
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let information = effects.format_world_string(
            b"WS0277",
            &[
                UnionFormatArgument::Text(&target_name),
                UnionFormatArgument::Text(legacy_c_string_visible_bytes(&self.name)),
            ],
        );
        let title = effects.world_string(b"WS0188");
 // Машинный факт общего кадра 0x7F804: цвет рассылки всегда 0xFFDAEDFE.
 // `CUnion::SendInfoToAllMember` (RVA 0x4C6290) лишь пробрасывает K через
 // два уровня до `CFaction::SendInfoToAllMember` (RVA 0x4B5890), где
 // литерал 0xFFDAEDFE его убивает — K мёртв в обоих классах.
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0xFFDA_EDFE,
            context,
            &mut |request| effects.send_organizing_info(request),
        );

        let detach = match context.detach_union_member_for_fire_out(parameters, target_faction_id)
        {
            Ok(detach) => detach,
            Err(source) => {
                return Err(UnionFireOutBlock::Detach {
                    target_faction_id,
                    source,
                    member_information,
                });
            }
        };
        let member_removed = self.members.remove(&target_faction_id).is_some();
        let delete_organizing =
            self.delete_organizing_to_client(target_faction_id, context, game);
        let member_update = match self.update_member_info_to_client(
            game,
            target_faction_id,
            EOperator::Delete,
            context,
        ) {
            Ok(member_update) => member_update,
            Err(source) => {
                return Err(UnionFireOutBlock::MemberUpdate {
                    target_faction_id,
                    source,
                    member_information,
                    member_removed,
                    delete_organizing,
                });
            }
        };
        self.set_change_data(2);
        let player_refresh =
            self.update_player_faction_info(target_faction_id, context, update_player);

        let war_log = match (
            context.faction_name(target_faction_id),
            context.faction_name(self.master_id),
        ) {
            (Some(target_faction_name), Some(master_faction_name)) => {
                let text = effects.format_world_string(
                    b"WS0278",
                    &[
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &target_faction_name,
                        )),
                        UnionFormatArgument::Signed(target_faction_id),
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &master_faction_name,
                        )),
                        UnionFormatArgument::Signed(self.union_id),
                    ],
                );
                effects.put_war_log(&text);
                Some(text)
            }
            _ => None,
        };

        Ok(UnionFireOutOutcome::Fired(UnionFireOutReport {
            member_information,
            detach,
            member_removed,
            delete_organizing,
            member_update,
            player_refresh,
            war_log,
        }))
    }

    pub fn exit<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionExitOutcome<Context::DetachOutcome>,
        UnionExitBlock<Context::Block, Context::DetachBlock>,
    >
    where
        Context: UnionExitContext,
        Effects: UnionFireOutEffects,
    {
        if self.has_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0246");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionExitOutcome::Rejected(UnionExitRejection::StandardWar));
        }
        if self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0247");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionExitOutcome::Rejected(UnionExitRejection::CityWar));
        }

        let operation_valid = self
            .check_operator_validate(player_id, EPurview::Exit as i32, context)
            .map_err(UnionExitBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionExitOutcome::Rejected(
                UnionExitRejection::OperatorValidationFailed,
            ));
        }

 // Оригинал повторно вызывал IsFactionMaster после validation.
        let leaving_faction_id = context
            .faction_id_by_master_player(player_id)
            .map_err(UnionExitBlock::MasterLookup)?;
        if leaving_faction_id == 0 {
            return Ok(UnionExitOutcome::Rejected(
                UnionExitRejection::FactionMasterNotFound,
            ));
        }
        let leaving = self.members.get(&leaving_faction_id).ok_or(
            UnionExitBlock::MissingLeavingMember { leaving_faction_id },
        )?;
        let leaving_name_wire = leaving.name_wire_bytes().map_err(|source| {
            UnionExitBlock::UnterminatedLeavingName {
                leaving_faction_id,
                source,
            }
        })?;
        let leaving_name = leaving_name_wire[..leaving_name_wire.len() - 1].to_vec();
        let information = effects.format_world_string(
            b"WS0273",
            &[UnionFormatArgument::Text(&leaving_name)],
        );
        let title = effects.world_string(b"WS0188");
 // Машинный факт общего кадра 0x7F804: цвет рассылки всегда 0xFFDAEDFE.
 // `CUnion::SendInfoToAllMember` (RVA 0x4C6290) лишь пробрасывает K через
 // два уровня до `CFaction::SendInfoToAllMember` (RVA 0x4B5890), где
 // литерал 0xFFDAEDFE его убивает — K мёртв в обоих классах.
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0xFFDA_EDFE,
            context,
            &mut |request| effects.send_organizing_info(request),
        );

        let war_log = match (
            context.faction_name(leaving_faction_id),
            context.faction_name(self.master_id),
        ) {
            (Some(leaving_faction_name), Some(master_faction_name)) => {
                let text = effects.format_world_string(
                    b"WS0274",
                    &[
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &leaving_faction_name,
                        )),
                        UnionFormatArgument::Signed(leaving_faction_id),
                        UnionFormatArgument::Signed(self.union_id),
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &master_faction_name,
                        )),
                    ],
                );
                effects.put_war_log(&text);
                Some(text)
            }
            _ => None,
        };

        let detach = match context.detach_union_member_for_exit(parameters, leaving_faction_id) {
            Ok(detach) => detach,
            Err(source) => {
                return Err(UnionExitBlock::Detach {
                    leaving_faction_id,
                    source,
                    member_information,
                    war_log,
                });
            }
        };
        let member_removed = self.members.remove(&leaving_faction_id).is_some();
        let delete_organizing =
            self.delete_organizing_to_client(leaving_faction_id, context, game);
        let member_update = match self.update_member_info_to_client(
            game,
            leaving_faction_id,
            EOperator::Delete,
            context,
        ) {
            Ok(member_update) => member_update,
            Err(source) => {
                return Err(UnionExitBlock::MemberUpdate {
                    leaving_faction_id,
                    source,
                    member_information,
                    war_log,
                    member_removed,
                    delete_organizing,
                });
            }
        };
        self.set_change_data(2);
        let player_refresh =
            self.update_player_faction_info(leaving_faction_id, context, update_player);

        Ok(UnionExitOutcome::Exited(UnionExitReport {
            leaving_faction_id,
            member_information,
            war_log,
            detach,
            member_removed,
            delete_organizing,
            member_update,
            player_refresh,
        }))
    }

    pub fn disband<Context, Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionDisbandOutcome<Context::DetachOutcome>,
        UnionDisbandBlock<Context::Block, Context::DetachBlock, Context::DetachOutcome>,
    >
    where
        Context: UnionFireOutContext,
        Effects: UnionFireOutEffects,
    {
        let operation_valid = self
            .check_operator_validate(manager_id, EPurview::Disband as i32, context)
            .map_err(UnionDisbandBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionDisbandOutcome::Rejected(
                UnionDisbandRejection::OperatorValidationFailed,
            ));
        }
        if self.has_enemy_faction(context) || self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0275");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionDisbandOutcome::Rejected(UnionDisbandRejection::War));
        }

        let information = effects.format_world_string(
            b"WS0276",
            &[UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                &self.name,
            ))],
        );
        let title = effects.world_string(b"WS0188");
 // Машинный факт общего кадра 0x7F804: цвет рассылки всегда 0xFFDAEDFE.
 // `CUnion::SendInfoToAllMember` (RVA 0x4C6290) лишь пробрасывает K через
 // два уровня до `CFaction::SendInfoToAllMember` (RVA 0x4B5890), где
 // литерал 0xFFDAEDFE его убивает — K мёртв в обоих классах.
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0xFFDA_EDFE,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        effects.put_war_log(&information);

        let member_ids: Vec<i32> = self.members.keys().copied().collect();
        let mut detached_members = Vec::with_capacity(member_ids.len());
        for faction_id in member_ids {
            match context.detach_union_member_for_fire_out(parameters, faction_id) {
                Ok(outcome) => detached_members.push((faction_id, outcome)),
                Err(source) => {
                    return Err(UnionDisbandBlock::Detach {
                        faction_id,
                        source,
                        member_information,
                        war_log: information,
                        completed: detached_members,
                    });
                }
            }
        }
        self.apply_person = Some(0);
        let delete_organizing = self.delete_organizing_to_client(0, context, game);
        Ok(UnionDisbandOutcome::Disbanded(UnionDisbandReport {
            member_information,
            war_log: information,
            detached_members,
            application_cleared: true,
            delete_organizing,
        }))
    }

    pub const fn dub_and_set_job_level(
        &self,
        _manager_id: i32,
        _target_id: i32,
        _title: &[u8],
        _job_level: i32,
    ) -> bool {
        true
    }

    pub const fn edit_leave_word(
        &self,
        _player_id: i32,
        _word_id: i32,
        _operation: EOperator,
    ) -> bool {
        false
    }

    pub const fn operator_tax(&self, _player_id: i32, _operation: i32) -> bool {
        false
    }

    pub const fn set_contributor(
        &self,
        _requester_id: i32,
        _target_id: i32,
        _enabled: bool,
    ) {}

    pub const fn upgrade(&self, _player_id: i32) -> bool {
        false
    }

    pub fn player_header<Lookup, Block>(
        &self,
        master_player_id_by_faction: Lookup,
    ) -> Result<i32, Block>
    where
        Lookup: FnOnce(i32) -> Result<Option<i32>, Block>,
    {
        if self.master_id <= 0 {
            return Ok(0);
        }
        Ok(master_player_id_by_faction(self.master_id)?.unwrap_or(0))
    }

    pub const fn members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    pub fn member_ids_snapshot(&self) -> Vec<i32> {
        self.members.keys().copied().collect()
    }

    pub fn add_members_to_byte_array<Context>(
        &mut self,
        output: &mut Vec<u8>,
        context: &Context,
    ) -> Result<bool, UnionMemberSnapshotBlock>
    where
        Context: UnionFactionMemberContext,
    {
        output.extend_from_slice(&(self.members.len() as u32).to_le_bytes());
        for (completed_members, member) in self.members.values_mut().enumerate() {
            append_i32(output, member.id);
            append_i32(output, member.job_level);
            let title = member.title_wire_bytes().map_err(|field| {
                UnionMemberSnapshotBlock::UnterminatedMemberField {
                    member_id: member.id,
                    field,
                    completed_members,
                }
            })?;
            output.extend_from_slice(title);
            output.extend_from_slice(&member.purview_wire_bytes());

            if member.id > 0 {
                if let Some(faction_name) = context.faction_name(member.id) {
                    let visible_name = legacy_c_string_visible_bytes(&faction_name);
                    if visible_name.len() >= member.name.len() {
                        return Err(UnionMemberSnapshotBlock::FactionNameWouldOverflow {
                            faction_id: member.id,
                            visible_length: visible_name.len(),
                            capacity: member.name.len(),
                            completed_members,
                        });
                    }
                    member.name[..visible_name.len()].copy_from_slice(visible_name);
                    member.name[visible_name.len()] = 0;
                    match context.faction_level(member.id) {
                        Ok(Some(level)) => member.level = level,
                        Ok(None) => {}
                        Err(source) => {
                            return Err(UnionMemberSnapshotBlock::MissingFactionLevel {
                                source,
                                completed_members,
                            });
                        }
                    }
                }
            }

            append_i32(output, member.level);
            append_i32(output, member.occupation);
            let name = member.name_wire_bytes().map_err(|field| {
                UnionMemberSnapshotBlock::UnterminatedMemberField {
                    member_id: member.id,
                    field,
                    completed_members,
                }
            })?;
            output.extend_from_slice(name);
            output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
            output.push(0);
            output.extend_from_slice(&member.last_online_wire_bytes());
        }
        Ok(true)
    }

    pub fn add_to_byte_array<Context>(
        &mut self,
        output: &mut Vec<u8>,
        context: &Context,
    ) -> Result<bool, UnionMemberSnapshotBlock>
    where
        Context: UnionFactionMemberContext,
    {
        append_i32(output, self.union_id);
        append_legacy_c_string(output, &self.name);
        append_i32(output, self.master_id);
        if self.master_id > 0 {
            if let Some(master_name) = context.faction_name(self.master_id) {
                append_legacy_c_string(output, &master_name);
            } else {
                output.push(0);
            }
        } else {
            output.push(0);
        }
        self.add_members_to_byte_array(output, context)?;
        Ok(true)
    }

    pub fn enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    pub fn city_war_enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    pub fn member_organizings<'a, T, Lookup>(&self, mut lookup: Lookup) -> Vec<&'a T>
    where
        T: ?Sized + 'a,
        Lookup: FnMut(i32) -> Option<&'a T>,
    {
        let mut output: Vec<&'a T> = Vec::new();
        for &member_id in self.members.keys() {
            if member_id < 1 {
                continue;
            }
            let Some(organizing) = lookup(member_id) else {
                continue;
            };
            if output
                .iter()
                .any(|current| std::ptr::eq(*current, organizing))
            {
                continue;
            }
            output.push(organizing);
        }
        output
    }

    pub fn is_member(&self, faction_id: i32) -> i32 {
        if self.members.contains_key(&faction_id) {
            self.union_id
        } else {
            0
        }
    }

    pub fn is_using_purview(&self, faction_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        if faction_id == 0 {
            return false;
        }
        self.members
            .get(&faction_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    pub fn set_member_purview(
        &mut self,
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
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
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    pub fn check_operator_validate<Context>(
        &self,
        manager_player_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0 || self.is_member(manager_faction_id) == 0 {
            return Ok(false);
        }
        Ok(self.is_using_purview(manager_faction_id, purview))
    }

 /// Проверяет manager-а относительно другой member-faction.
 ///
 /// Это полный owner `CUnion::CheckOperValidate(manager, target, purview)`
 ///: выделенный оригинал хвост не является
 /// самостоятельной функцией и продолжает те же проверки membership и
 /// противоположности purview.
    pub fn check_operator_validate_target<Context>(
        &self,
        manager_player_id: i32,
        target_faction_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        if manager_player_id == target_faction_id {
            return Ok(false);
        }
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0
            || target_faction_id == 0
            || target_faction_id == self.master_id
            || self.is_member(manager_faction_id) == 0
            || self.is_member(target_faction_id) == 0
        {
            return Ok(false);
        }
        let manager_permitted = self.is_using_purview(manager_faction_id, purview);
        if !manager_permitted {
            return Ok(false);
        }
        Ok(manager_permitted != self.is_using_purview(target_faction_id, purview))
    }

    pub fn is_owned_city<Context>(&self, region_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_owned_city(self.master_id, region_id)
            .unwrap_or(0)
    }

    pub fn is_enemy_faction<Context>(&self, enemy_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_enemy_faction(self.master_id, enemy_id)
            .unwrap_or(0)
    }

    pub fn owned_cities_snapshot<Context>(&self, context: &Context) -> VecDeque<i32>
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return VecDeque::new();
        }
        context
            .faction_owned_cities(self.master_id)
            .unwrap_or_default()
    }

    pub fn has_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_enemy(self.master_id)
                .unwrap_or(false)
    }

    pub fn has_city_war_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_city_war_enemy(self.master_id)
                .unwrap_or(false)
    }

    pub fn enemy_leader_organizing_id<Context>(&self, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_enemy_leader_organizing_id(self.master_id)
            .unwrap_or(0)
    }

    fn fan_out_owned_city_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        mut dispatch: Dispatch,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
        ) -> Result<bool, OwnedCityMutationBuildError>,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id) {
                Ok(true) => invoked_faction_ids.push(faction_id),
                Ok(false) => {}
                Err(source) => {
                    return Err(UnionOwnedCityMutationBlock {
                        faction_id,
                        completed_faction_ids: invoked_faction_ids,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCityFanoutReport {
            invoked_faction_ids,
        })
    }

    pub fn add_owned_city<Context>(
        &self,
        context: &mut Context,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_city(faction_id, region_id, update_player)
        })
    }

    pub fn add_owned_cities<Context>(
        &self,
        context: &mut Context,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_cities(faction_id, region_ids, update_player)
        })
    }

    pub fn clear_owned_cities<Context>(
        &self,
        context: &mut Context,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        let fanout = self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_clear_owned_cities(faction_id, update_player)
        })?;
        Ok(UnionOwnedCityBooleanMutationReport {
            legacy_result: true,
            invoked_faction_ids: fanout.invoked_faction_ids,
        })
    }

    pub fn set_owned_cities<Context>(
        &self,
        context: &mut Context,
        region_ids: &VecDeque<i32>,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_set_owned_cities(self.master_id, region_ids) {
            Ok(true) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: vec![self.master_id],
            }),
            Ok(false) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    pub fn delete_owned_city<Context>(
        &self,
        context: &mut Context,
        _region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: false,
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_clear_owned_cities(self.master_id, update_player) {
            Ok(found) => Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: found,
                invoked_faction_ids: found.then_some(self.master_id).into_iter().collect(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    pub fn clear_enemy_factions<Context>(
        &self,
        context: &mut Context,
    ) -> UnionFactionFanoutReport
    where
        Context: UnionFactionStateMutationContext,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id > 0 && context.faction_clear_enemy_factions(faction_id) {
                invoked_faction_ids.push(faction_id);
            }
        }
        UnionFactionFanoutReport {
            invoked_faction_ids,
        }
    }

    pub fn clear_city_war_enemy_factions<Context>(
        &self,
        context: &mut Context,
    ) -> UnionFactionFanoutReport
    where
        Context: UnionFactionStateMutationContext,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id > 0 && context.faction_clear_city_war_enemy_factions(faction_id) {
                invoked_faction_ids.push(faction_id);
            }
        }
        UnionFactionFanoutReport {
            invoked_faction_ids,
        }
    }

    fn fan_out_victor_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        mut dispatch: Dispatch,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
        ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>,
    {
        let mut factions = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id) {
                Ok(Some(deliveries)) => factions.push(UnionVictorFactionReport {
                    faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionVictorMutationBlock {
                        faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionVictorFanoutReport { factions })
    }

    pub fn add_defence_victor_counts<Context>(
        &self,
        context: &mut Context,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, |context, faction_id| {
            context.faction_add_defence_victor_count(faction_id)
        })
    }

    pub fn add_offense_victor_counts<Context>(
        &self,
        context: &mut Context,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, |context, faction_id| {
            context.faction_add_offense_victor_count(faction_id)
        })
    }

    pub fn add_village_war_victor_counts<Context>(
        &self,
        context: &mut Context,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, |context, faction_id| {
            context.faction_add_village_war_victor_count(faction_id)
        })
    }

    fn target_faction_ids(&self, faction_id: i32) -> Vec<i32> {
        if faction_id == 0 {
            self.members.keys().copied().collect()
        } else {
            vec![faction_id]
        }
    }

    pub fn update_player_faction_info<Context>(
        &self,
        faction_id: i32,
        context: &Context,
        update_player: &mut dyn FnMut(i32),
    ) -> UnionPlayerRefreshReport
    where
        Context: UnionPlayerRefreshContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(refreshed_player_ids) = context.faction_update_player_info(
                target_faction_id,
                update_player,
            ) else {
                continue;
            };
            factions.push(UnionFactionPlayerRefreshReport {
                faction_id: target_faction_id,
                refreshed_player_ids,
            });
        }
        UnionPlayerRefreshReport { factions }
    }

    pub fn update_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_enemy_snapshot(target_faction_id)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    pub fn update_city_war_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_city_war_enemy_snapshot(target_faction_id)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    pub fn update_owned_cities_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
    ) -> Result<UnionOwnedCitySnapshotReport, UnionOwnedCitySnapshotBlock>
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            match context.faction_update_owned_city_snapshot(target_faction_id) {
                Ok(Some(deliveries)) => factions.push(UnionOwnedCitySnapshotFactionReport {
                    faction_id: target_faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionOwnedCitySnapshotBlock {
                        faction_id: target_faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCitySnapshotReport { factions })
    }

    pub fn send_info_to_all_members<'a, Context>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        context: &Context,
        send_organizing_info: &mut dyn FnMut(FactionMemberInfoRequest<'a>),
    ) -> UnionInfoFanoutReport
    where
        Context: UnionSendInfoContext,
    {
        let mut factions = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            let Some(report) = context.faction_send_info_to_members(
                faction_id,
                first_text,
                second_text,
                information_type,
                color,
                send_organizing_info,
            ) else {
                continue;
            };
            factions.push(UnionFactionInfoFanout {
                faction_id,
                recipient_player_ids: report.recipient_player_ids,
            });
        }
        UnionInfoFanoutReport { factions }
    }

    pub fn delete_organizing_to_client<Context>(
        &self,
        faction_id: i32,
        context: &Context,
        game: &dyn WorldGameView,
    ) -> UnionDeleteOrganizingReport
    where
        Context: UnionFactionMemberContext,
    {
        let target_faction_ids: Vec<i32> = if faction_id > 0 {
            vec![faction_id]
        } else {
            self.members.keys().copied().collect()
        };
        let mut visited_faction_ids = Vec::new();
        let mut deliveries = Vec::new();
        for target_faction_id in target_faction_ids {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(recipient_player_ids) =
                context.faction_member_player_ids(target_faction_id)
            else {
                continue;
            };
            visited_faction_ids.push(target_faction_id);
            for recipient_player_id in recipient_player_ids {
                let player = game.online_player_by_id(recipient_player_id as u32);
                let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
                if player.is_none_or(|player| !player.faction_data_received())
                    || game_server_id == 0
                {
                    continue;
                }
                let mut message = CMessage::new(DELETE_UNION_ORGANIZING_MESSAGE_TYPE);
                message.base_mut().add_long(recipient_player_id);
                message.base_mut().add_long(self.union_id);
                deliveries.push(UnionDeleteOrganizingDelivery {
                    faction_id: target_faction_id,
                    recipient_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                });
            }
        }
        UnionDeleteOrganizingReport {
            visited_faction_ids,
            deliveries,
        }
    }

    pub fn update_member_info_to_client<Context>(
        &mut self,
        game: &dyn WorldGameView,
        target_faction_id: i32,
        operator: EOperator,
        context: &Context,
    ) -> Result<UnionMemberUpdateReport, UnionMemberUpdateBlock>
    where
        Context: UnionFactionMemberContext,
    {
        let update_time = if operator == EOperator::Delete {
            None
        } else {
            let Some(current_level) = self.members.get(&target_faction_id).map(|member| member.level)
            else {
                return Ok(UnionMemberUpdateReport {
                    target_found: Some(false),
                    target_level: None,
                    deliveries: Vec::new(),
                });
            };
            match context.faction_level(target_faction_id) {
                Ok(Some(level)) => {
                    self.members
                        .get_mut(&target_faction_id)
                        .expect("target key проверен до faction lookup")
                        .level = level;
                }
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionMemberUpdateBlock::MissingFactionLevel {
                        source,
                        target_level: current_level,
                        completed_deliveries: Vec::new(),
                    });
                }
            }
            Some(current_local_member_time())
        };

        let recipient_faction_ids: Vec<i32> = self.members.keys().copied().collect();
        let mut deliveries = Vec::new();
        for recipient_faction_id in recipient_faction_ids {
            if recipient_faction_id <= 0 {
                continue;
            }
            let Some(recipient_player_ids) =
                context.faction_member_player_ids(recipient_faction_id)
            else {
                continue;
            };
            for recipient_player_id in recipient_player_ids {
                let player = game.online_player_by_id(recipient_player_id as u32);
                let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
                if player.is_none_or(|player| !player.faction_data_received())
                    || game_server_id == 0
                {
                    continue;
                }

                let mut message = CMessage::new(UNION_MEMBER_UPDATE_MESSAGE_TYPE);
                message.base_mut().add_long(recipient_player_id);
                message.base_mut().add_long(operator.wire_value());
                message.base_mut().add_long(target_faction_id);

                if let Some(update_time) = update_time {
                    let member_faction_id = self
                        .members
                        .get(&target_faction_id)
                        .expect("non-delete target существует до recipient-прохода")
                        .id;
                    if member_faction_id > 0 {
                        let current_level = self
                            .members
                            .get(&target_faction_id)
                            .expect("target существует до level refresh")
                            .level;
                        match context.faction_level(member_faction_id) {
                            Ok(Some(level)) => {
                                self.members
                                    .get_mut(&target_faction_id)
                                    .expect("target существует до level assignment")
                                    .level = level;
                            }
                            Ok(None) => {}
                            Err(source) => {
                                return Err(UnionMemberUpdateBlock::MissingFactionLevel {
                                    source,
                                    target_level: current_level,
                                    completed_deliveries: deliveries,
                                });
                            }
                        }
                    }

                    let target = self
                        .members
                        .get(&target_faction_id)
                        .expect("non-delete target существует до wire build");
                    let mut fields = Vec::new();
                    if let Err(field) =
                        append_union_member_update_fields(&mut fields, target, update_time)
                    {
                        return Err(UnionMemberUpdateBlock::UnterminatedField {
                            field,
                            target_level: target.level,
                            recipient_faction_id,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                    message.base_mut().add(&fields);
                }

                deliveries.push(UnionMemberUpdateDelivery {
                    recipient_faction_id,
                    recipient_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                });
            }
        }

        Ok(UnionMemberUpdateReport {
            target_found: (operator != EOperator::Delete).then_some(true),
            target_level: (operator != EOperator::Delete).then(|| {
                self.members
                    .get(&target_faction_id)
                    .expect("non-delete target существует после recipient-прохода")
                    .level
            }),
            deliveries,
        })
    }

    pub const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

fn append_union_member_update_fields(
    output: &mut Vec<u8>,
    member: &TagMemInfo,
    update_time: TagTimeValue,
) -> Result<(), UnterminatedMemberField> {
    output.extend_from_slice(member.name_wire_bytes()?);
    output.extend_from_slice(&member.level.to_le_bytes());
    output.extend_from_slice(&member.occupation.to_le_bytes());
    output.extend_from_slice(&member.job_level.to_le_bytes());
    output.extend_from_slice(member.title_wire_bytes()?);
    output.extend_from_slice(&member.purview_wire_bytes());
    output.push(0);
    output.extend_from_slice(&update_time.wire_bytes());
    Ok(())
}

fn send_union_application_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionApplyForJoinEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_union_invitation_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionInviteEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_union_demise_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionFireOutEffects,
{
    let second_text = effects.world_string(b"WS0119");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
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

fn append_legacy_c_string(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(legacy_c_string_visible_bytes(value));
    output.push(0);
}
