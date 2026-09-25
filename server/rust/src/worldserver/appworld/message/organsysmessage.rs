//! Организационные сообщения `OnOrgasysMessage` из `organsysmessage.cpp`,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Dispatcher разрешает player/faction/union/country и передаёт governance,
//! membership, city/war, application, transfer и billboard операции их
//! владельцам в исходном порядке. Дополнительные socket/ownership/tail gates
//! не добавляются; повторный lookup остаётся повторным.
//!
//! Async session callback публикует terminal action в main-loop FIFO и лишь
//! там изменяет `CGame`; confirmation остаётся на исходной позиции callback.
//! Короткий payload останавливает ветку после уже выполненного префикса.

use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

use crate::dbaccess::worlddb::rsplayer::{RsPlayerOwner, TiberiusRsPlayer};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::regionrouter::{
    RegionRoutePoint, RegionRouter, RegionRouterChangeOutcome,
};
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveCounterReset, CountryExileMessageDelivery, CountryExileResultContext,
    CountryExileTarget, CountryExileTextArgument, CountryFactionSnapshot,
    CountryGovernanceContextBlock,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionCallbackOutcome};
use crate::public::date::TagTime;
use crate::public::timer::{CTimer, TimerId};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex, query_goods_name,
};
use crate::worldserver::appworld::goodswarmember::{
    CGoodsWarMember, GoodsWarAuditEnvironment, GoodsWarAuditPlayer,
    GoodsWarFactionSnapshot, GoodsWarFactionWinReport, GoodsWarFactionWinSnapshot,
    GoodsWarDeliveryContext, GoodsWarMemberBlock, GoodsWarMemberContext,
    GoodsWarMutationReport,
    GoodsWarRefreshReport,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    CFaction, FactionApplyForJoinEffects, FactionApplyForJoinOutcome, FactionContributorContext,
    FactionDemiseBlock, FactionDemiseContext, FactionDemiseOutcome, FactionDisbandContext,
    FactionDoJoinEffects,
    FactionDubBlock, FactionDubContext, FactionDubFormatArgument, FactionDubOutcome,
    FactionExitBlock, FactionExitContext, FactionExitOutcome,
    FactionFireOutBlock, FactionFireOutContext, FactionFireOutOutcome,
    current_local_member_time, goods_war_check_for_faction_id,
    FactionEnemyMutationBlock, FactionEnemyMutationContext,
    FactionEnemyWarLogArgument, FactionExperienceBlock, FactionExperienceUpdate,
    FactionLevelContext, FactionMemberInfoRequest, FactionOrganizingInfoContext,
    FactionPurviewChange, FactionPurviewChangeBlock, FactionPurviewChangeContext,
    FactionPurviewChangeOutcome,
    FactionInitialPropertyBlock, FactionOperationBlock, FactionOperationOutcome,
    FactionOperationRejection, OwnedCityMutationBuildError,
    FactionPermitBlock, FactionPermitUpdate,
    FactionSetParameterBlock, FactionSetParameterContext, FactionSetParameterOutcome,
    FactionUpgradeBlock, FactionUpgradeContext, FactionUpgradeFormatArgument,
    FactionUpgradeOutcome, FactionUploadIconBlock, FactionUploadIconContext,
    FactionUploadIconOutcome,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::{
    CFactionWarSys, FactionWarDeclarationBlock, FactionWarDeclarationOutcome,
    FactionWarPlayerDiedBlock, FactionWarPlayerDiedOutcome,
};
use crate::worldserver::appworld::organizingsystem::attackcitysys::{
    AttackCityApplicationContext, AttackCityApplicationReport, AttackCityCallbacks,
    AttackCityEnemyRelationContext, AttackCityReloadBlock, AttackCityReloadReport,
    AttackCityWarEndContext, AttackCityWarResultBlock,
    AttackCityWarResultContext, AttackCityWarResultFaction, AttackCityWarResultFormatArgument,
    AttackCityWarResultRegion, AttackCityWarResultReport, CAttackCitySys,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    AttackCityEndBlock, AttackCityEndEffects, AttackCityEndReport, COrganizingCtrl,
    ConfederationCreationEffects, ConfederationCreationEndpointBlock,
    ConfederationCreationSessionBlock, ConfederationCreationSessionReport,
    ConfederationCreationSessionRequest, ConfederationCreationSessionRuntime,
    ConfederationCreationTerminal,
    CityTransferEffects, CityTransferEndpointBlock,
    CityTransferSessionBlock, CityTransferSessionReport, CityTransferSessionRequest,
    CityTransferSessionRuntime, CityTransferStartBlock, CityTransferStartOutcome,
    CityTransferTerminal, DeclareWarFactionPage, DeclareWarFactionPageBlock,
    ApplyFactionLookup, FactionCountryCountBlock, FactionListPage, FactionListPageBlock,
    RemovePersonFromApplyFactionListOutcome,
    FactionMasterLookupBlock, FreeFactionLookup, FreePlayerLookup,

    OrganizingContributorBlock, OrganizingContributorOutcome,
    OrganizingDisbandBlock, OrganizingDisbandOutcome, OrganizingDisbandPlayer,
    OrganizingDisbandProgress, OrganizingDisbandRejection,
    OrganizingConfederationDisbandBlock, OrganizingConfederationDisbandOutcome,
    FactionUnionMembershipLookupBlock, OrganizingFactionExperienceMutation,
    OrganizingFactionMemberStateOutcome,
    OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock,
    OrganizingLeaveWordEditOutcome, OrganizingLeaveWordEnableBlock,
    OrganizingFactionWarDeclarationBlock, OrganizingFactionWarPlayerDiedBlock,
    WorldFactionWarDeclarationEffects,
    OrganizingLeaveWordEnableOutcome, OrganizingLeaveWordOutcome, OrganizingPronounceBlock,
    OrganizingPronounceOutcome, OrganizingUnionApplyForJoinDispatchBlock,
    OrganizingUnionApplyForJoinOutcome, OrganizingFactionApplicationBlock,
    FactionCreationBlock, FactionCreationEffects, FactionCreationOutcome,
    FactionCreationPreparation, FactionClientSnapshotBlock, AllFactionInfoClientBlock,
    UnionClientSnapshotByPlayerBlock, UnionClientSnapshotByPlayerOutcome, UnionOrganizingBridge,
    PlayerInviteFactionBlock, PlayerInviteFactionEffects, PlayerInviteFactionOutcome,
    OrganizingNameCountryBlock, OrganizingNameKind, OrganizingNameLookupBlock,
    OrganizingNameMatch, OrganizingNamedUnionApplicationBlock,
    OrganizingFactionDoJoinBlock, OrganizingFactionDoJoinOutcome,
    begin_city_transfer_session, begin_confederation_creation_session,
    OrganizingUnionDemiseBlock, OrganizingUnionDemiseOutcome,
    OrganizingUnionExitBlock, OrganizingUnionExitOutcome,
    OrganizingUnionFireOutBlock, OrganizingUnionFireOutOutcome,
    OrganizingUnionByMasterBlock,
};
use crate::worldserver::appworld::organizingsystem::organizing::{
    ECityState, EOperator, TagTimeValue,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::union::{
    UnionAddFactionEffects, UnionApplicationEndpointBlock, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionApplicationSessionRequest, UnionApplicationSessionRuntime,
    UnionApplicationTerminal, UnionApplyForJoinEffects, UnionApplyForJoinOutcome,
    UnionInvitationSessionRequest, UnionInvitationSessionRuntime, UnionInviteEffects,
    UnionFactionStateMutationContext, UnionFireOutEffects,
    UnionFormatArgument, UnionOwnedCityMutationContext,
    begin_union_application_session, begin_union_invitation_session,
};
use crate::worldserver::appworld::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarApplicationContext, VillageWarApplicationReport,
    VillageWarCallbacks, VillageWarResultBlock, VillageWarResultContext,
    VillageWarResultFaction, VillageWarResultRegion, VillageWarResultReport,
};
use crate::worldserver::appworld::player::{
    PlayerCodecError, PlayerFactionInfoUpdateBlock, PlayerFactionInfoUpdateReport,
    PlayerPropertyCoefficients,
};
use crate::worldserver::worldserver::game::{
    CGame, WorldRegionNameLookup, WorldRegionParamUpdateOutcome, format_union_world_string,
    legacy_tick_ms,
};

const SESSION_RESULT_MESSAGE_TYPES: [i32; 6] =
    [0x60117, 0x60119, 0x60120, 0x60122, 0x60124, 0x60131];
const FACTION_WAR_PLAYER_DIED_MESSAGE_TYPE: i32 = 0x60101;
const CREATE_FACTION_MESSAGE_TYPE: i32 = 0x60103;
const CREATE_FACTION_RESPONSE_TYPE: i32 = 0x7FE01;
const INITIAL_ORGANIZING_DATA_MESSAGE_TYPE: i32 = 0x60104;
const FACTION_LIST_MESSAGE_TYPE: i32 = 0x60107;
const FACTION_LIST_RESPONSE_TYPE: i32 = 0x7FE07;
const FACTION_APPLICATION_MESSAGE_TYPE: i32 = 0x60108;
const CANCEL_FACTION_APPLICATION_MESSAGE_TYPE: i32 = 0x60109;
const FACTION_APPLICATION_DECISION_MESSAGE_TYPE: i32 = 0x6010A;
const FACTION_FIRE_OUT_MESSAGE_TYPE: i32 = 0x6010B;
const UNION_FIRE_OUT_MESSAGE_TYPE: i32 = 0x6010C;
const FACTION_EXIT_MESSAGE_TYPE: i32 = 0x6010D;
const UNION_EXIT_MESSAGE_TYPE: i32 = 0x6010E;
const FACTION_DEMISE_MESSAGE_TYPE: i32 = 0x6010F;
const UNION_DEMISE_MESSAGE_TYPE: i32 = 0x60110;
const FACTION_DISBAND_MESSAGE_TYPE: i32 = 0x60111;
const UNION_DISBAND_MESSAGE_TYPE: i32 = 0x60112;
const FACTION_DUB_MESSAGE_TYPE: i32 = 0x60113;
const GRANT_FACTION_PURVIEW_MESSAGE_TYPE: i32 = 0x60114;
const REVOKE_FACTION_PURVIEW_MESSAGE_TYPE: i32 = 0x60115;
const PLAYER_INVITE_FACTION_MESSAGE_TYPE: i32 = 0x60116;
const UNION_APPLICATION_MESSAGE_TYPE: i32 = 0x60118;
const ENABLE_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011A;
const LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011B;
const EDIT_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011C;
const PRONOUNCE_MESSAGE_TYPE: i32 = 0x6011D;
const DECLARE_WAR_FACTION_LIST_MESSAGE_TYPE: i32 = 0x6011E;
const DECLARE_WAR_FACTION_LIST_RESPONSE_TYPE: i32 = 0x7FE18;
const DECLARE_FACTION_WAR_MESSAGE_TYPE: i32 = 0x6011F;
const DECLARE_FACTION_WAR_RESPONSE_TYPE: i32 = 0x7FE19;
const CONSUMED_LONG_MESSAGE_TYPES: [i32; 2] = [0x60121, 0x60123];
const FACTION_BILLBOARD_MESSAGE_TYPE: i32 = 0x60125;
const FACTION_BILLBOARD_RESPONSE_TYPE: i32 = 0x7FE1D;
const UPGRADE_FACTION_MESSAGE_TYPE: i32 = 0x60126;
const UPLOAD_FACTION_ICON_MESSAGE_TYPE: i32 = 0x60127;
const SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE: i32 = 0x60128;
const ADD_FACTION_EXPERIENCE_MESSAGE_TYPE: i32 = 0x60129;
const CHANGE_FACTION_MEMBER_STATE_MESSAGE_TYPE: i32 = 0x6012A;
const OPERATE_FACTION_TAX_MESSAGE_TYPE: i32 = 0x6012B;
const OPERATE_FACTION_TAX_RESPONSE_TYPE: i32 = 0x7FE28;
const ADJUST_FACTION_TAX_MESSAGE_TYPE: i32 = 0x6012C;
const ADJUST_FACTION_TAX_RESPONSE_TYPE: i32 = 0x7FE29;
const UPDATE_REGION_PARAM_MESSAGE_TYPE: i32 = 0x6012D;
const UPDATE_REGION_PARAM_RESPONSE_TYPE: i32 = 0x7FE2E;
const ROUTE_REGION_MESSAGE_TYPE: i32 = 0x6012E;
const ROUTE_REGION_RESPONSE_TYPE: i32 = 0x7FE2D;
const OPERATE_CITY_GATE_MESSAGE_TYPE: i32 = 0x6012F;
const OPERATE_CITY_GATE_RESPONSE_TYPE: i32 = 0x7FE2A;
const TRANSFER_CITY_OWNER_MESSAGE_TYPE: i32 = 0x60130;
const SET_FACTION_ADMISSION_PERMIT_MESSAGE_TYPE: i32 = 0x60132;
const ATTACK_CITY_END_MESSAGE_TYPE: i32 = 0x60133;
const APPLY_FOR_VILLAGE_WAR_MESSAGE_TYPE: i32 = 0x60135;
const APPLY_FOR_VILLAGE_WAR_RESPONSE_TYPE: i32 = 0x7FE34;
const VILLAGE_WAR_RESULT_MESSAGE_TYPE: i32 = 0x60136;
const APPLY_FOR_CITY_WAR_MESSAGE_TYPE: i32 = 0x60137;
const APPLY_FOR_CITY_WAR_RESPONSE_TYPE: i32 = 0x7FE37;
const CITY_WAR_RESULT_MESSAGE_TYPE: i32 = 0x60138;
const GOODS_WAR_COMMAND_MESSAGE_TYPE: i32 = 0x60139;
const GOODS_WAR_FACTION_WIN_MESSAGE_TYPE: i32 = 0x6013A;
const PLAYER_ADD_QUEST_MESSAGE_TYPE: i32 = 0x6013B;
const PLAYER_REMOVE_QUEST_MESSAGE_TYPE: i32 = 0x6013C;
const GAME_ADD_QUEST_MESSAGE_TYPE: i32 = 0x7FE38;
const GAME_REMOVE_QUEST_MESSAGE_TYPE: i32 = 0x7FE39;
const PLAYER_RUN_SCRIPT_MESSAGE_TYPE: i32 = 0x6013D;
const GAME_RUN_SCRIPT_MESSAGE_TYPE: i32 = 0x7FE3A;
const PLAYER_SCRIPT_CAPACITY: usize = 0x100;
const SET_FACTION_PARAMETER_MESSAGE_TYPE: i32 = 0x6013E;
const FACTION_PARAMETER_NAME_CAPACITY: usize = 0x32;
const CHANGE_REGION_ROUTER_MESSAGE_TYPE: i32 = 0x60144;
const CHANGE_REGION_ROUTER_RESPONSE_TYPE: i32 = 0x7FE4A;
const LEAVE_WORD_INPUT_CAPACITY: usize = 0xD2;
const PRONOUNCE_INPUT_CAPACITY: usize = 0x5000;

static FACTION_BILLBOARD_TITLES: OnceLock<[Vec<u8>; 3]> = OnceLock::new();

pub(crate) use nebokrai_realm::app::organsysmessage::{
    CityTransferConfirmationDelivery, ConfederationCreationConfirmationDelivery,
    QueuedCityTransferTerminal, QueuedConfederationCreationTerminal,
    QueuedOrganizingSessionTerminal, QueuedUnionApplicationTerminal,
    QueuedUnionInvitationTerminal, UnionApplicationConfirmationDelivery,
};

#[derive(Default)]
struct WorldUnionApplicationRuntimeState {
    terminals: Mutex<VecDeque<QueuedOrganizingSessionTerminal>>,
    confirmations: Mutex<VecDeque<UnionApplicationConfirmationDelivery>>,
    blocks: Mutex<VecDeque<UnionApplicationEndpointBlock>>,
    city_confirmations: Mutex<VecDeque<CityTransferConfirmationDelivery>>,
    city_blocks: Mutex<VecDeque<CityTransferEndpointBlock>>,
    confederation_creation_confirmations:
        Mutex<VecDeque<ConfederationCreationConfirmationDelivery>>,
    confederation_creation_blocks: Mutex<VecDeque<ConfederationCreationEndpointBlock>>,
}

#[derive(Clone, Default)]
pub(crate) struct WorldUnionApplicationRuntimeOwner {
    state: Arc<WorldUnionApplicationRuntimeState>,
}

impl WorldUnionApplicationRuntimeOwner {
    fn endpoint(
        &self,
        sender: Option<ServerCommandHandle>,
        game_server_id: i32,
    ) -> Arc<dyn UnionApplicationSessionRuntime> {
        Arc::new(WorldUnionApplicationEndpointRuntime {
            state: Arc::clone(&self.state),
            sender,
            game_server_id,
        })
    }

    fn invitation_endpoint(
        &self,
        sender: Option<ServerCommandHandle>,
        game_server_id: i32,
    ) -> Arc<dyn UnionInvitationSessionRuntime> {
        Arc::new(WorldUnionInvitationEndpointRuntime {
            state: Arc::clone(&self.state),
            sender,
            game_server_id,
        })
    }

    fn city_endpoint(
        &self,
        sender: Option<ServerCommandHandle>,
        game_server_id: i32,
    ) -> Arc<dyn CityTransferSessionRuntime> {
        Arc::new(WorldCityTransferEndpointRuntime {
            state: Arc::clone(&self.state),
            sender,
            game_server_id,
        })
    }

    fn confederation_creation_endpoint(
        &self,
        sender: Option<ServerCommandHandle>,
        game_server_id: i32,
    ) -> Arc<dyn ConfederationCreationSessionRuntime> {
        Arc::new(WorldConfederationCreationEndpointRuntime {
            state: Arc::clone(&self.state),
            sender,
            game_server_id,
        })
    }

    pub(crate) fn pop_terminal(&self) -> Option<QueuedOrganizingSessionTerminal> {
        self.state.terminals.lock().pop_front()
    }

    pub(crate) fn take_confirmations(&self) -> Vec<UnionApplicationConfirmationDelivery> {
        self.state.confirmations.lock().drain(..).collect()
    }

    pub(crate) fn take_blocks(&self) -> Vec<UnionApplicationEndpointBlock> {
        self.state.blocks.lock().drain(..).collect()
    }

    pub(crate) fn take_city_confirmations(&self) -> Vec<CityTransferConfirmationDelivery> {
        self.state.city_confirmations.lock().drain(..).collect()
    }

    pub(crate) fn take_city_blocks(&self) -> Vec<CityTransferEndpointBlock> {
        self.state.city_blocks.lock().drain(..).collect()
    }

    pub(crate) fn take_confederation_creation_confirmations(
        &self,
    ) -> Vec<ConfederationCreationConfirmationDelivery> {
        self.state
            .confederation_creation_confirmations
            .lock()
            .drain(..)
            .collect()
    }

    pub(crate) fn take_confederation_creation_blocks(
        &self,
    ) -> Vec<ConfederationCreationEndpointBlock> {
        self.state
            .confederation_creation_blocks
            .lock()
            .drain(..)
            .collect()
    }
}

struct WorldUnionApplicationEndpointRuntime {
    state: Arc<WorldUnionApplicationRuntimeState>,
    sender: Option<ServerCommandHandle>,
    game_server_id: i32,
}

impl UnionApplicationSessionRuntime for WorldUnionApplicationEndpointRuntime {
    fn send_union_application_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    ) {
        let result = message.send_to_map_id(self.sender.as_ref(), self.game_server_id);
        self.state
            .confirmations
            .lock()
            .push_back(UnionApplicationConfirmationDelivery {
                recipient_player_id,
                game_server_id: self.game_server_id,
                result,
            });
    }

    fn finish_union_application(
        &self,
        union_id: i32,
        applicant_faction_id: i32,
        terminal: UnionApplicationTerminal,
    ) {
        self.state
            .terminals
            .lock()
            .push_back(QueuedOrganizingSessionTerminal::Union(
                QueuedUnionApplicationTerminal {
                    union_id,
                    applicant_faction_id,
                    terminal,
                },
            ));
    }

    fn block_union_application_endpoint(&self, block: UnionApplicationEndpointBlock) {
        self.state.blocks.lock().push_back(block);
    }
}

struct WorldUnionInvitationEndpointRuntime {
    state: Arc<WorldUnionApplicationRuntimeState>,
    sender: Option<ServerCommandHandle>,
    game_server_id: i32,
}

impl UnionInvitationSessionRuntime for WorldUnionInvitationEndpointRuntime {
    fn send_union_invitation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    ) {
        let result = message.send_to_map_id(self.sender.as_ref(), self.game_server_id);
        self.state
            .confirmations
            .lock()
            .push_back(UnionApplicationConfirmationDelivery {
                recipient_player_id,
                game_server_id: self.game_server_id,
                result,
            });
    }

    fn finish_union_invitation(
        &self,
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        terminal: UnionApplicationTerminal,
    ) {
        self.state.terminals.lock().push_back(
            QueuedOrganizingSessionTerminal::UnionInvitation(
                QueuedUnionInvitationTerminal {
                    union_id,
                    inviter_faction_id,
                    invited_faction_id,
                    terminal,
                },
            ),
        );
    }

    fn block_union_invitation_endpoint(&self, block: UnionApplicationEndpointBlock) {
        self.state.blocks.lock().push_back(block);
    }
}

struct WorldConfederationCreationEndpointRuntime {
    state: Arc<WorldUnionApplicationRuntimeState>,
    sender: Option<ServerCommandHandle>,
    game_server_id: i32,
}

impl ConfederationCreationSessionRuntime for WorldConfederationCreationEndpointRuntime {
    fn send_confederation_creation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    ) {
        let result = message.send_to_map_id(self.sender.as_ref(), self.game_server_id);
        self.state
            .confederation_creation_confirmations
            .lock()
            .push_back(ConfederationCreationConfirmationDelivery {
                recipient_player_id,
                game_server_id: self.game_server_id,
                result,
            });
    }

    fn finish_confederation_creation(
        &self,
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        union_name: &[u8],
        terminal: ConfederationCreationTerminal,
    ) {
        self.state.terminals.lock().push_back(
            QueuedOrganizingSessionTerminal::ConfederationCreation(
                QueuedConfederationCreationTerminal {
                    first_player_id,
                    second_player_id,
                    first_faction_id,
                    second_faction_id,
                    union_name: union_name.to_vec(),
                    terminal,
                },
            ),
        );
    }

    fn block_confederation_creation_endpoint(
        &self,
        block: ConfederationCreationEndpointBlock,
    ) {
        self.state
            .confederation_creation_blocks
            .lock()
            .push_back(block);
    }
}

struct WorldCityTransferEndpointRuntime {
    state: Arc<WorldUnionApplicationRuntimeState>,
    sender: Option<ServerCommandHandle>,
    game_server_id: i32,
}

impl CityTransferSessionRuntime for WorldCityTransferEndpointRuntime {
    fn send_city_transfer_confirmation(&self, recipient_player_id: i32, message: &CMessage) {
        let result = message.send_to_map_id(self.sender.as_ref(), self.game_server_id);
        self.state
            .city_confirmations
            .lock()
            .push_back(CityTransferConfirmationDelivery {
                recipient_player_id,
                game_server_id: self.game_server_id,
                result,
            });
    }

    fn finish_city_transfer(
        &self,
        source_faction_id: i32,
        target_faction_id: i32,
        region_id: i32,
        region_name: &[u8],
        terminal: CityTransferTerminal,
    ) {
        self.state
            .terminals
            .lock()
            .push_back(QueuedOrganizingSessionTerminal::CityTransfer(
                QueuedCityTransferTerminal {
                    source_faction_id,
                    target_faction_id,
                    region_id,
                    region_name: region_name.to_vec(),
                    terminal,
                },
            ));
    }

    fn block_city_transfer_endpoint(&self, block: CityTransferEndpointBlock) {
        self.state.city_blocks.lock().push_back(block);
    }
}

pub(crate) struct WorldUnionApplicationEffectCallbacks<'a> {
    pub(crate) random: &'a mut dyn FnMut(i32) -> i32,
    pub(crate) refresh_owned_city:
        &'a mut dyn FnMut(&CGame, i32, i32, i32, Option<u8>),
    pub(crate) faction_level_log_enabled: bool,
    pub(crate) write_faction_level_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8]),
    pub(crate) faction_experience_log_enabled: bool,
    pub(crate) write_faction_experience_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
}

pub(crate) struct WorldUnionApplicationEffects<'a> {
    game: &'a CGame,
    manager: &'a CNetSessionManager,
    runtime: &'a WorldUnionApplicationRuntimeOwner,
    callbacks: WorldUnionApplicationEffectCallbacks<'a>,
}

impl<'a> WorldUnionApplicationEffects<'a> {
    pub(crate) fn new(
        game: &'a CGame,
        manager: &'a CNetSessionManager,
        runtime: &'a WorldUnionApplicationRuntimeOwner,
        callbacks: WorldUnionApplicationEffectCallbacks<'a>,
    ) -> Self {
        Self {
            game,
            manager,
            runtime,
            callbacks,
        }
    }
}

impl UnionApplyForJoinEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = UnionApplicationSessionReport;
    type SessionBlock = UnionApplicationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_union_application_session(
        &mut self,
        request: UnionApplicationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
 // `Beging` вызывает `DoAsyncCall` синхронно; route и клонируемый
 // transport handle снимаются непосредственно перед session creation.
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.recipient_player_id);
        let endpoint = self
            .runtime
            .endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_union_application_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }
}

impl UnionInviteEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = UnionApplicationSessionReport;
    type SessionBlock = UnionApplicationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_union_invitation_session(
        &mut self,
        request: UnionInvitationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.recipient_player_id);
        let endpoint = self
            .runtime
            .invitation_endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_union_invitation_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }
}

impl UnionAddFactionEffects for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl ConfederationCreationEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = ConfederationCreationSessionReport;
    type SessionBlock = ConfederationCreationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_confederation_creation_session(
        &mut self,
        request: ConfederationCreationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.second_player_id);
        let endpoint = self.runtime.confederation_creation_endpoint(
            self.game.current_game_server_sender(),
            game_server_id,
        );
        begin_confederation_creation_session(
            self.manager,
            request,
            endpoint,
            |upper_bound| (self.callbacks.random)(upper_bound),
        )
    }
}

struct WorldFactionCreationEffects<'game, 'callbacks, 'effects, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    persistent_name_exists: bool,
    faction_create_log_enabled: bool,
    write_faction_create_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8]),
}

impl FactionCreationEffects for WorldFactionCreationEffects<'_, '_, '_, '_> {
    fn check_invalid_organizing_string(&mut self, name: &mut Vec<u8>, strict: bool) -> bool {
        self.game.check_invalid_string(name, strict)
    }

    fn persistent_player_name_exists(&mut self, _name: &[u8]) -> bool {
        self.persistent_name_exists
    }

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn faction_create_log_enabled(&self) -> bool {
        self.faction_create_log_enabled
    }

    fn write_faction_create_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    ) {
        (self.write_faction_create_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
        );
    }
}

struct WorldUnionFireOutEffects<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl UnionFireOutEffects for WorldUnionFireOutEffects<'_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }
}

struct WorldFactionDubEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    use_log_system: bool,
    faction_title_log_enabled: bool,
    write_faction_title_log:
        &'log mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
}

impl FactionOrganizingInfoContext for WorldFactionDubEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDubContext for WorldFactionDubEffects<'_, '_, '_, '_, '_> {
    fn check_invalid_string(&mut self, value: &mut Vec<u8>, mode: bool) -> bool {
        self.game.check_invalid_string(value, mode)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionDubFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionDubFormatArgument::Text(value) => UnionFormatArgument::Text(value),
                FactionDubFormatArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_title_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_title_log_enabled
    }

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
    ) {
        (self.write_faction_title_log)(
            member_id,
            member_name,
            old_title,
            new_title,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
        );
    }
}

struct WorldFactionPurviewEffects<'game, 'callbacks, 'effects, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    use_log_system: bool,
    add_log_enabled: bool,
    revoke_log_enabled: bool,
    write_log: &'log mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionPurviewEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionPurviewChangeContext for WorldFactionPurviewEffects<'_, '_, '_, '_> {
    fn format_world_string(&mut self, string_id: &'static [u8], member_name: &[u8]) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Text(member_name)],
        )
    }

    fn faction_purview_log_enabled(&self, change: FactionPurviewChange) -> bool {
        self.use_log_system
            && match change {
                FactionPurviewChange::Grant => self.add_log_enabled,
                FactionPurviewChange::Revoke => self.revoke_log_enabled,
            }
    }

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
    ) {
        (self.write_log)(
            member_id,
            member_name,
            purview,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }
}

impl CityTransferEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = CityTransferSessionReport;
    type SessionBlock = CityTransferSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_city_transfer_session(
        &mut self,
        request: CityTransferSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.target_master_player_id);
        let endpoint = self
            .runtime
            .city_endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_city_transfer_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn broadcast_city_transfer(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFDA_EDFE,
            0x328F_93FC,
        )
    }
}

impl AttackCityEndEffects for WorldUnionApplicationEffects<'_> {
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn broadcast_city_war_result(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFDA_EDFE,
            0x328F_93FC,
        )
    }
}

impl FactionOrganizingInfoContext for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionApplicationListContext for WorldUnionApplicationEffects<'_> {
    fn online_player_country(&self, player_id: i32) -> Option<Option<u8>> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.country())
    }
}

/// Узкий war/string/log adapter faction-ветви `0x60108` поверх общего
/// session/transport owner-а, который нужен соседней union-ветви.
struct WorldFactionApplicationEffects<'owner, 'effects> {
    village_war: &'owner CVillageWarSys,
    attack_city: &'owner CAttackCitySys,
    use_log_system: bool,
    faction_apply_log_enabled: bool,
    write_faction_apply_log: &'owner mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    owner: &'owner mut WorldUnionApplicationEffects<'effects>,
}

impl FactionOrganizingInfoContext for WorldFactionApplicationEffects<'_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.owner.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.owner.game, request);
    }
}

impl FactionApplyForJoinEffects for WorldFactionApplicationEffects<'_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.owner.game.get_string_by_id(string_id), &arguments)
    }

    fn faction_apply_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_apply_log_enabled
    }

    fn write_faction_apply_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_apply_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
            log_type,
        );
    }
}

struct WorldFactionDoJoinEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game CGoodsWarMember,
    use_log_system: bool,
    faction_join_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_join_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionDoJoinEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDoJoinEffects for WorldFactionDoJoinEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_join(&self, faction_id: i32, _manager_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_join_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_join_log_enabled
    }

    fn write_faction_join_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_join_log)(
            member_id,
            member_name,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }
}

struct WorldFactionDemiseEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game CGoodsWarMember,
    country_handler: &'game CCountryHandler,
    faction_master_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_master_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

impl FactionOrganizingInfoContext for WorldFactionDemiseEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDemiseContext for WorldFactionDemiseEffects<'_, '_, '_, '_, '_> {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_demise(&self, faction_id: i32, _old_master_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool {
        self.country_handler.get_country(country).is_some_and(|state| {
            state.king.id == old_master_id && !state.demise_faction
        })
    }

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[
                UnionFormatArgument::Text(old_master_name),
                UnionFormatArgument::Text(new_master_name),
            ],
        )
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_master_log_enabled(&self) -> bool {
        self.faction_master_log_enabled
    }

    fn write_faction_master_log(
        &mut self,
        old_master_id: i32,
        old_master_name: &[u8],
        new_master_id: i32,
        new_master_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    ) {
        (self.write_faction_master_log)(
            old_master_id,
            old_master_name,
            new_master_id,
            new_master_name,
            faction_id,
            faction_name,
        );
    }
}

struct WorldFactionExitEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game mut CGoodsWarMember,
    use_log_system: bool,
    faction_quit_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_quit_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionExitEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionExitContext for WorldFactionExitEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_exit(&self, faction_id: i32, _player_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_quit_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_quit_log_enabled
    }

    fn write_faction_quit_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_quit_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
            log_type,
        );
    }

    fn delete_goods_war_member(&mut self, player_id: i32) {
        let mut delivery = WorldFactionFireOutGoodsWarDelivery { game: self.game };
        let _ = self.goods_war.delete_one_member(player_id, &mut delivery);
    }
}

struct WorldFactionFireOutEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game mut CGoodsWarMember,
    use_log_system: bool,
    faction_fire_out_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_fire_out_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
}

struct WorldFactionFireOutGoodsWarDelivery<'game> {
    game: &'game CGame,
}

impl GoodsWarDeliveryContext for WorldFactionFireOutGoodsWarDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl FactionOrganizingInfoContext for WorldFactionFireOutEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionFireOutContext for WorldFactionFireOutEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_fire_out(&self, faction_id: i32, _manager_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_fire_out_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_fire_out_log_enabled
    }

    fn write_faction_fire_out_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_fire_out_log)(
            member_id,
            member_name,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }

    fn delete_goods_war_member(&mut self, player_id: i32) {
        let mut delivery = WorldFactionFireOutGoodsWarDelivery { game: self.game };
        let _ = self.goods_war.delete_one_member(player_id, &mut delivery);
    }
}

struct WorldFactionSetParameterEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionLevelContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn format_world_string_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }
}

impl FactionSetParameterContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }
}

struct WorldFactionUpgradeEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    registry: &'game GoodsBasePropertiesRegistry,
    original_name_index: &'game GoodsOriginalNameIndex,
    use_log_system: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionLevelContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn format_world_string_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }
}

impl FactionUpgradeContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn player_money(&self, player_id: i32) -> Option<u32> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.money())
    }

    fn goods_in_packet(&self, player_id: i32, original_name: &[u8]) -> i32 {
        let Ok(original_name) = CString::new(original_name) else {
            return 0;
        };
        self.game
            .online_player_by_id(player_id as u32)
            .map_or(0, |player| {
                player.check_goods_in_packet(
                    Some(original_name.as_c_str()),
                    self.original_name_index,
                )
            })
    }

    fn goods_display_name(&self, original_name: &[u8]) -> Option<Vec<u8>> {
        let goods_id = self.original_name_index.get(original_name).copied()?;
        query_goods_name(self.registry, goods_id).map(ToOwned::to_owned)
    }

    fn format_upgrade_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionUpgradeFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionUpgradeFormatArgument::Text(value) => UnionFormatArgument::Text(value),
                FactionUpgradeFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_level_log_enabled(&self) -> bool {
        self.use_log_system && self.callbacks.faction_level_log_enabled
    }

    fn write_faction_level_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        level: i32,
        master_id: i32,
        master_name: &[u8],
    ) {
        (self.callbacks.write_faction_level_log)(
            faction_id,
            faction_name,
            level,
            master_id,
            master_name,
        );
    }
}

struct WorldFactionUploadIconEffects<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl FactionOrganizingInfoContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionUploadIconContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn format_upload_icon_interval(
        &mut self,
        string_id: &'static [u8],
        interval_minutes: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(interval_minutes)],
        )
    }
}

struct WorldFactionContributorEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionContributorContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn format_contributor_string(
        &mut self,
        string_id: &'static [u8],
        member_name: &[u8],
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Text(member_name)],
        )
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingSessionResultDispatch {
    NotHandled,
    Delivered {
        message_type: i32,
        session_id: i64,
        cookie_first: i32,
        cookie_second: i32,
        result: i32,
        outcome: NetSessionCallbackOutcome,
    },
}

/// Декодирует общий session-result branch `OnOrgasysMessage` в исходном
/// порядке полей и передаёт callback менеджеру без дополнительных ответов.
pub(crate) fn dispatch_organizing_session_result(
    message: &mut CMessage,
    manager: &CNetSessionManager,
) -> OrganizingSessionResultDispatch {
    let message_type = message.message_type();
    if !SESSION_RESULT_MESSAGE_TYPES.contains(&message_type) {
        return OrganizingSessionResultDispatch::NotHandled;
    }

    let session_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie_second = message.base_mut().get_long().unwrap_or(0);
    let result = message
        .base_mut()
        .get_char()
        .map_or(0, |result| i32::from(result as u8));
    let cookie_first = message.base_mut().get_long().unwrap_or(0);
    let outcome =
        manager.on_sync_callback_result(session_id, cookie_first, cookie_second, result);
    OrganizingSessionResultDispatch::Delivered {
        message_type,
        session_id,
        cookie_first,
        cookie_second,
        result,
        outcome,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPlayerInviteFactionDispatch<
    CreationReport,
    ApplicationReport,
    InvitationReport,
> {
    pub(crate) player_id: i32,
    pub(crate) invited_faction_id: i32,
    pub(crate) outcome:
        PlayerInviteFactionOutcome<CreationReport, ApplicationReport, InvitationReport>,
}

pub(crate) fn dispatch_player_invite_faction<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingPlayerInviteFactionDispatch<
            <Effects as ConfederationCreationEffects>::SessionReport,
            <Effects as UnionApplyForJoinEffects>::SessionReport,
            <Effects as UnionInviteEffects>::SessionReport,
        >,
        PlayerInviteFactionBlock<
            <Effects as ConfederationCreationEffects>::SessionBlock,
            <Effects as UnionApplyForJoinEffects>::SessionBlock,
            <Effects as UnionInviteEffects>::SessionBlock,
        >,
    >,
>
where
    Effects: PlayerInviteFactionEffects,
{
    if message.message_type() != PLAYER_INVITE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let invited_faction_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .on_player_invite_faction(
                game,
                village_war,
                attack_city,
                player_id,
                invited_faction_id,
                effects,
            )
            .map(|outcome| OrganizingPlayerInviteFactionDispatch {
                player_id,
                invited_faction_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionApplicationDispatch<SessionReport> {
    pub(crate) master_player_id: i32,
    pub(crate) applicant_faction_id: i32,
    pub(crate) outcome: OrganizingUnionApplyForJoinOutcome<SessionReport>,
}

pub(crate) fn dispatch_union_application<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingUnionApplicationDispatch<Effects::SessionReport>,
        OrganizingUnionApplyForJoinDispatchBlock<Effects::SessionBlock>,
    >,
>
where
    Effects: UnionApplyForJoinEffects,
{
    if message.message_type() != UNION_APPLICATION_MESSAGE_TYPE {
        return None;
    }

    let master_player_id = message.base_mut().get_long().unwrap_or(0);
    let applicant_faction_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .apply_for_union_join(
                game,
                master_player_id,
                applicant_faction_id,
                0,
                master_player_id,
                effects,
            )
            .map(|outcome| OrganizingUnionApplicationDispatch {
                master_player_id,
                applicant_faction_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordEnableDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingLeaveWordEnableOutcome,
}

pub(crate) fn dispatch_leave_word_enable<Context>(
    message: &mut CMessage,
    organizing: &mut COrganizingCtrl,
    context: &mut Context,
) -> Option<Result<OrganizingLeaveWordEnableDispatch, OrganizingLeaveWordEnableBlock>>
where
    Context: FactionOrganizingInfoContext,
{
    if message.message_type() != ENABLE_LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }
    let player_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .enable_leave_word_for_master(player_id, context)
            .map(|outcome| OrganizingLeaveWordEnableDispatch { player_id, outcome }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordDispatch {
    pub(crate) player_id: i32,
    pub(crate) content: Vec<u8>,
    pub(crate) time: TagTimeValue,
    pub(crate) outcome: OrganizingLeaveWordOutcome,
}

fn capture_local_tag_time() -> TagTimeValue {
    let local_time = TagTime::local_now();
    TagTimeValue {
        year: local_time.year,
        month: local_time.month,
        day_of_week: local_time.day_of_week,
        day: local_time.day,
        hour: local_time.hour,
        minute: local_time.minute,
        second: local_time.second,
        milliseconds: local_time.milliseconds,
    }
}

pub(crate) fn dispatch_leave_word(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingLeaveWordDispatch, OrganizingLeaveWordBlock>> {
    if message.message_type() != LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(LEAVE_WORD_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let time = capture_local_tag_time();
    Some(
        organizing
            .leave_word_for_player(game, player_id, &mut content, time)
            .map(|outcome| OrganizingLeaveWordDispatch {
                player_id,
                content,
                time,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordEditDispatch {
    pub(crate) leave_word_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingLeaveWordEditOutcome,
}

pub(crate) fn dispatch_leave_word_edit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEditBlock>> {
    if message.message_type() != EDIT_LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }

    let leave_word_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .edit_leave_word_for_player(game, player_id, leave_word_id, EOperator::Delete)
            .map(|outcome| OrganizingLeaveWordEditDispatch {
                leave_word_id,
                player_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPronounceDispatch {
    pub(crate) player_id: i32,
    pub(crate) content: Vec<u8>,
    pub(crate) time: TagTimeValue,
    pub(crate) outcome: OrganizingPronounceOutcome,
}

pub(crate) fn dispatch_pronounce(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingPronounceDispatch, OrganizingPronounceBlock>> {
    if message.message_type() != PRONOUNCE_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(PRONOUNCE_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let time = capture_local_tag_time();
    Some(
        organizing
            .pronounce_for_player(game, player_id, &mut content, time)
            .map(|outcome| OrganizingPronounceDispatch {
                player_id,
                content,
                time,
                outcome,
            }),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionWarPlayerDiedDispatch {
    pub(crate) defeated_master_player_id: i32,
    pub(crate) victor_player_id: i32,
    pub(crate) outcome: Result<
        FactionWarPlayerDiedOutcome,
        FactionWarPlayerDiedBlock<OrganizingFactionWarPlayerDiedBlock>,
    >,
}

pub(crate) fn dispatch_faction_war_player_died(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    faction_war: &mut CFactionWarSys,
    update_player: &mut dyn FnMut(i32),
) -> Option<OrganizingFactionWarPlayerDiedDispatch> {
    if message.message_type() != FACTION_WAR_PLAYER_DIED_MESSAGE_TYPE {
        return None;
    }
    let defeated_master_player_id = message.base_mut().get_long().unwrap_or(0);
    let victor_player_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldFactionWarDeclarationEffects::new(
        game,
        organizing,
        update_player,
    );
    let outcome = faction_war.on_player_died(
        defeated_master_player_id,
        victor_player_id,
        &mut effects,
    );
    Some(OrganizingFactionWarPlayerDiedDispatch {
        defeated_master_player_id,
        victor_player_id,
        outcome,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingCreateFactionGate {
    CountryMissing,
    PlayerLevel,
    RequiredGoods,
    Money,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCreateFactionResponse {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) result: i32,
    pub(crate) map_id: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingCreateFactionOutcome {
    EmptyName,
    PlayerOffline,
    GateRejected {
        gate: OrganizingCreateFactionGate,
        response: OrganizingCreateFactionResponse,
    },
    Creation {
        outcome: FactionCreationOutcome,
        response: OrganizingCreateFactionResponse,
        notice: crate::worldserver::appworld::organizingsystem::organizingctrl::OrganizingInfoDelivery,
        player_refresh: Option<PlayerFactionInfoUpdateReport>,
        faction_snapshot: Option<Result<bool, FactionClientSnapshotBlock>>,
        all_factions_snapshot: Option<Result<bool, AllFactionInfoClientBlock>>,
    },
}

#[derive(Debug)]
pub(crate) enum OrganizingCreateFactionBlock {
    PlayerDecode(PlayerCodecError),
    Creation(FactionCreationBlock),
    PlayerRefresh(PlayerFactionInfoUpdateBlock),
    PlayerRefreshOwnerMissing,
}

#[derive(Debug)]
pub(crate) struct OrganizingCreateFactionDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) country: u8,
    pub(crate) faction_name: Vec<u8>,
    pub(crate) outcome: OrganizingCreateFactionOutcome,
}

#[allow(
    clippy::too_many_arguments,
    reason = "границы один к одному соответствуют exact World owner-ам"
)]
pub(crate) async fn dispatch_create_faction(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    countries: &CCountryHandler,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    rs_player: &mut TiberiusRsPlayer,
    player_database: Option<&mut WorldTdsClient>,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    faction_create_log_enabled: bool,
    write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
) -> Option<Result<OrganizingCreateFactionDispatch, OrganizingCreateFactionBlock>> {
    if message.message_type() != CREATE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let country = message.base_mut().get_byte().unwrap_or(0);
    let mut faction_name = message.base_mut().get_str_bytes(20).unwrap_or_default();
    if faction_name.is_empty() {
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::EmptyName,
        }));
    }

    let established_time = capture_local_tag_time();

    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingCreateFactionBlock::PlayerDecode(source)));
            }
        }
    };
    if !player_online {
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::PlayerOffline,
        }));
    }

    let gate = if countries.get_country(country).is_none() {
        Some(OrganizingCreateFactionGate::CountryMissing)
    } else {
        let player = game
            .online_player_by_id(player_id as u32)
            .expect("успешный Decord сохранил тот же online owner");
        if i32::from(player.get_level()) < parameters.create_faction_player_level() {
            Some(OrganizingCreateFactionGate::PlayerLevel)
        } else {
            let required_goods = legacy_c_string_prefix(parameters.create_faction_goods());
            let has_required_goods = if required_goods == b"0" {
                true
            } else {
                CString::new(required_goods).is_ok_and(|name| {
                    player.check_goods_in_packet(
                        Some(name.as_c_str()),
                        original_name_index,
                    ) > 0
                })
            };
            if !has_required_goods {
                Some(OrganizingCreateFactionGate::RequiredGoods)
            } else if player.money() < parameters.create_faction_money() as u32 {
                Some(OrganizingCreateFactionGate::Money)
            } else {
                None
            }
        }
    };
    let map_id = message.map_id();
    if let Some(gate) = gate {
        let response = send_create_faction_response(
            game, map_id, request_id, cookie, player_id, 0,
        );
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::GateRejected { gate, response },
        }));
    }

    let mut effects = WorldFactionCreationEffects {
        game,
        callbacks,
        persistent_name_exists: false,
        faction_create_log_enabled,
        write_faction_create_log,
    };
    let preparation = match organizing.prepare_faction_creation(
        game,
        player_id,
        &mut faction_name,
        &mut effects,
    ) {
        Ok(preparation) => preparation,
        Err(source) => return Some(Err(OrganizingCreateFactionBlock::Creation(source))),
    };
    let creation = match preparation {
        FactionCreationPreparation::Rejected(reason) => {
            FactionCreationOutcome::Rejected(reason)
        }
        FactionCreationPreparation::ReadyForPersistentLookup => {
            let persistent_name_exists = rs_player
                .is_name_exist(legacy_c_string_prefix(&faction_name), player_database)
                .await;
            effects.persistent_name_exists = persistent_name_exists;
            match organizing.finish_faction_creation(
                game,
                parameters,
                player_id,
                0,
                established_time,
                &faction_name,
                country,
                persistent_name_exists,
                &mut effects,
            ) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Some(Err(OrganizingCreateFactionBlock::Creation(source)));
                }
            }
        }
    };
    let (result, notice_id, created) = match &creation {
        FactionCreationOutcome::Rejected(
            crate::worldserver::appworld::organizingsystem::organizingctrl::FactionCreationRejection::NameExists,
        ) => (0, b"WS0116".as_slice(), false),
        FactionCreationOutcome::Rejected(_) => (0, b"WS0115".as_slice(), false),
        FactionCreationOutcome::Created(_) => (1, b"WS0117".as_slice(), true),
    };
    drop(effects);
    let player_refresh = if created {
        match game.update_player_faction_info(organizing, player_id) {
            Ok(Some(report)) => Some(report),
            Ok(None) => {
                return Some(Err(
                    OrganizingCreateFactionBlock::PlayerRefreshOwnerMissing,
                ));
            }
            Err(source) => {
                return Some(Err(OrganizingCreateFactionBlock::PlayerRefresh(source)));
            }
        }
    } else {
        None
    };
    let faction_snapshot = created.then(|| {
        organizing.add_faction_to_client_by_player_id(game, player_id)
    });
    let all_factions_snapshot = created.then(|| {
        organizing.add_all_faction_info_to_client_by_player_id(game, player_id)
    });
    let response = send_create_faction_response(
        game, map_id, request_id, cookie, player_id, result,
    );
    let first_text = game.get_string_by_id(notice_id).to_vec();
    let second_text = game.get_string_by_id(b"WS0118").to_vec();
    let notice = COrganizingCtrl::send_organizing_info_to_client(
        game,
        FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: legacy_c_string_prefix(&first_text),
            second_text: legacy_c_string_prefix(&second_text),
            information_type: map_id,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        },
    );
    Some(Ok(OrganizingCreateFactionDispatch {
        request_id,
        cookie,
        player_id,
        country,
        faction_name,
        outcome: OrganizingCreateFactionOutcome::Creation {
            outcome: creation,
            response,
            notice,
            player_refresh,
            faction_snapshot,
            all_factions_snapshot,
        },
    }))
}

fn send_create_faction_response(
    game: &CGame,
    map_id: i32,
    request_id: i64,
    cookie: i32,
    player_id: i32,
    result: i32,
) -> OrganizingCreateFactionResponse {
    let mut response = CMessage::new(CREATE_FACTION_RESPONSE_TYPE);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(result);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = game.send_msg_to_game_server(map_id, &response);
    OrganizingCreateFactionResponse {
        request_id,
        cookie,
        player_id,
        result,
        map_id,
        wire,
        delivery,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingInitialDataOutcome {
    PlayerOffline,
    AlreadyReceived,
    Sent {
        faction_snapshot: bool,
        union_snapshot: UnionClientSnapshotByPlayerOutcome,
        all_factions_snapshot: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingInitialDataBlock {
    Faction(FactionClientSnapshotBlock),
    Union {
        faction_snapshot: bool,
        source: UnionClientSnapshotByPlayerBlock,
    },
    AllFactions {
        faction_snapshot: bool,
        union_snapshot: UnionClientSnapshotByPlayerOutcome,
        source: AllFactionInfoClientBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingInitialDataDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: Result<OrganizingInitialDataOutcome, OrganizingInitialDataBlock>,
}

pub(crate) fn dispatch_initial_organizing_data(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<OrganizingInitialDataDispatch> {
    if message.message_type() != INITIAL_ORGANIZING_DATA_MESSAGE_TYPE {
        return None;
    }
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let Some(player) = game.online_player_by_id(player_id as u32) else {
        return Some(OrganizingInitialDataDispatch {
            player_id,
            outcome: Ok(OrganizingInitialDataOutcome::PlayerOffline),
        });
    };
    if player.faction_data_received() {
        return Some(OrganizingInitialDataDispatch {
            player_id,
            outcome: Ok(OrganizingInitialDataOutcome::AlreadyReceived),
        });
    }

    let faction_snapshot = match organizing.add_faction_to_client_by_player_id(game, player_id) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Some(OrganizingInitialDataDispatch {
                player_id,
                outcome: Err(OrganizingInitialDataBlock::Faction(source)),
            });
        }
    };
    let union_snapshot = match organizing.add_union_to_client_by_player_id(game, player_id) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Some(OrganizingInitialDataDispatch {
                player_id,
                outcome: Err(OrganizingInitialDataBlock::Union {
                    faction_snapshot,
                    source,
                }),
            });
        }
    };
    let all_factions_snapshot =
        match organizing.add_all_faction_info_to_client_by_player_id(game, player_id) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Some(OrganizingInitialDataDispatch {
                    player_id,
                    outcome: Err(OrganizingInitialDataBlock::AllFactions {
                        faction_snapshot,
                        union_snapshot,
                        source,
                    }),
                });
            }
        };
    Some(OrganizingInitialDataDispatch {
        player_id,
        outcome: Ok(OrganizingInitialDataOutcome::Sent {
            faction_snapshot,
            union_snapshot,
            all_factions_snapshot,
        }),
    })
}

pub(crate) trait FactionApplicationListContext: FactionOrganizingInfoContext {
 /// `None` означает offline miss, внутренний `None` — ещё не
 /// Готовый country найденного player-owner-а.
    fn online_player_country(&self, player_id: i32) -> Option<Option<u8>>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionListResponse {
    pub(crate) socket_id: i32,
    pub(crate) total_factions: i32,
    pub(crate) included_page: Option<i32>,
    pub(crate) applied_faction_id: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionListOutcome {
    PlayerOffline,
    Empty {
        country: u8,
        response: OrganizingFactionListResponse,
    },
    PageOutsideRange {
        country: u8,
        total_factions: i32,
        page: i32,
    },
    Page {
        country: u8,
        page: FactionListPage,
        applied_faction_id: i32,
        response: OrganizingFactionListResponse,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionListBlock {
    PlayerCountry { player_id: i32 },
    Count(FactionCountryCountBlock),
    ApplyList { map_key: i32 },
    Page(FactionListPageBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionListDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) page: i32,
    pub(crate) outcome: OrganizingFactionListOutcome,
}

pub(crate) fn dispatch_faction_list<Context>(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionListDispatch, OrganizingFactionListBlock>>
where
    Context: FactionApplicationListContext,
{
    if message.message_type() != FACTION_LIST_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let page = message.base_mut().get_long().unwrap_or(0);
    let outcome = match context.online_player_country(player_id) {
        None => OrganizingFactionListOutcome::PlayerOffline,
        Some(None) => {
            return Some(Err(OrganizingFactionListBlock::PlayerCountry {
                player_id,
            }));
        }
        Some(Some(country)) => {
            let total_factions = match organizing.faction_count_by_country(country) {
                Ok(total_factions) => total_factions,
                Err(block) => return Some(Err(OrganizingFactionListBlock::Count(block))),
            };
            if total_factions == 0 {
                send_faction_list_empty_notice(context, player_id);
                OrganizingFactionListOutcome::Empty {
                    country,
                    response: send_faction_list_response(
                        sender, socket_id, player_id, 0, request_id, cookie, None,
                    ),
                }
            } else {
                let start = page.wrapping_mul(11).wrapping_sub(11);
                if start >= total_factions {
                    OrganizingFactionListOutcome::PageOutsideRange {
                        country,
                        total_factions,
                        page,
                    }
                } else {
                    let applied_faction_id = match organizing
                        .faction_by_player_in_apply_list(player_id)
                    {
                        ApplyFactionLookup::NoFaction => 0,
                        ApplyFactionLookup::Faction(faction_id) => faction_id,
                        ApplyFactionLookup::BlockedNullFaction { map_key } => {
                            return Some(Err(OrganizingFactionListBlock::ApplyList {
                                map_key,
                            }));
                        }
                    };
                    let page_data = match organizing.faction_list_page(page, country) {
                        Ok(page_data) => page_data,
                        Err(block) => {
                            return Some(Err(OrganizingFactionListBlock::Page(block)));
                        }
                    };
                    let response = send_faction_list_response(
                        sender,
                        socket_id,
                        player_id,
                        total_factions,
                        request_id,
                        cookie,
                        Some((page, applied_faction_id, &page_data)),
                    );
                    OrganizingFactionListOutcome::Page {
                        country,
                        page: page_data,
                        applied_faction_id,
                        response,
                    }
                }
            }
        }
    };

    Some(Ok(OrganizingFactionListDispatch {
        request_id,
        cookie,
        player_id,
        page,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionApplicationOutcome<SessionReport> {
    PlayerOffline,
    OrganizingNotFound,
    CountryMismatch {
        matched: OrganizingNameMatch,
        player_country: u8,
        organizing_country: u8,
    },
    Faction {
        matched: OrganizingNameMatch,
        outcome: FactionApplyForJoinOutcome,
    },
    Union {
        matched: OrganizingNameMatch,
        outcome: UnionApplyForJoinOutcome<SessionReport>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionApplicationDispatchBlock<SessionBlock> {
    PlayerCountry {
        player_id: i32,
    },
    NameLookup(OrganizingNameLookupBlock),
    OrganizingCountry(OrganizingNameCountryBlock),
    Faction(OrganizingFactionApplicationBlock),
    Union(OrganizingNamedUnionApplicationBlock<SessionBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionApplicationDispatch<SessionReport> {
    pub(crate) player_id: i32,
    pub(crate) discarded_value: i32,
    pub(crate) organizing_name: Vec<u8>,
    pub(crate) outcome: OrganizingFactionApplicationOutcome<SessionReport>,
}

/// Выполняет `0x60108`: `(player ID, discarded Long, name[20])`,
/// online/country gate и virtual `ApplyForJoin(player ID, 0, 0)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_application(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    use_log_system: bool,
    faction_apply_log_enabled: bool,
    write_faction_apply_log: &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    effects: &mut WorldUnionApplicationEffects<'_>,
) -> Option<
    Result<
        OrganizingFactionApplicationDispatch<UnionApplicationSessionReport>,
        OrganizingFactionApplicationDispatchBlock<UnionApplicationSessionBlock>,
    >,
> {
    if message.message_type() != FACTION_APPLICATION_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let discarded_value = message.base_mut().get_long().unwrap_or(0);
    let organizing_name = message
        .base_mut()
        .get_str_bytes(20)
        .unwrap_or_default();
    let Some(player) = game.online_player_by_id(player_id as u32) else {
        return Some(Ok(OrganizingFactionApplicationDispatch {
            player_id,
            discarded_value,
            organizing_name,
            outcome: OrganizingFactionApplicationOutcome::PlayerOffline,
        }));
    };
    let Some(player_country) = player.country() else {
        return Some(Err(
            OrganizingFactionApplicationDispatchBlock::PlayerCountry { player_id },
        ));
    };
    let matched = match organizing.organizing_by_name(&organizing_name) {
        Ok(Some(matched)) => matched,
        Ok(None) => {
            return Some(Ok(OrganizingFactionApplicationDispatch {
                player_id,
                discarded_value,
                organizing_name,
                outcome: OrganizingFactionApplicationOutcome::OrganizingNotFound,
            }));
        }
        Err(source) => {
            return Some(Err(
                OrganizingFactionApplicationDispatchBlock::NameLookup(source),
            ));
        }
    };
    let organizing_country = match organizing.country_by_name_match(matched) {
        Ok(country) => country,
        Err(source) => {
            return Some(Err(
                OrganizingFactionApplicationDispatchBlock::OrganizingCountry(source),
            ));
        }
    };
    if player_country != organizing_country {
        return Some(Ok(OrganizingFactionApplicationDispatch {
            player_id,
            discarded_value,
            organizing_name,
            outcome: OrganizingFactionApplicationOutcome::CountryMismatch {
                matched,
                player_country,
                organizing_country,
            },
        }));
    }

    let outcome = match matched.kind {
        OrganizingNameKind::Faction => {
            let mut faction_effects = WorldFactionApplicationEffects {
                village_war,
                attack_city,
                use_log_system,
                faction_apply_log_enabled,
                write_faction_apply_log,
                owner: effects,
            };
            let outcome = match organizing.apply_for_faction_join_by_map_key(
                game,
                parameters,
                matched.map_key,
                player_id,
                0,
                0,
                &mut faction_effects,
            ) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Some(Err(
                        OrganizingFactionApplicationDispatchBlock::Faction(source),
                    ));
                }
            };
            OrganizingFactionApplicationOutcome::Faction { matched, outcome }
        }
        OrganizingNameKind::Union => {
            let outcome = match organizing.apply_for_named_union_join(
                game,
                matched.map_key,
                player_id,
                0,
                0,
                effects,
            ) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Some(Err(
                        OrganizingFactionApplicationDispatchBlock::Union(source),
                    ));
                }
            };
            OrganizingFactionApplicationOutcome::Union { matched, outcome }
        }
    };
    Some(Ok(OrganizingFactionApplicationDispatch {
        player_id,
        discarded_value,
        organizing_name,
        outcome,
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionApplicationCancelLookupPhase {
    BeforeRemoval,
    AfterRemoval,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionApplicationCancelBlock {
    Lookup {
        phase: FactionApplicationCancelLookupPhase,
        map_key: i32,
    },
    Removal {
        previous_faction_id: i32,
        outcome: RemovePersonFromApplyFactionListOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionApplicationCancelDispatch {
    pub(crate) player_id: i32,
    pub(crate) previous_faction_id: i32,
    pub(crate) remaining_faction_id: Option<i32>,
    pub(crate) removal: RemovePersonFromApplyFactionListOutcome,
    pub(crate) notice_sent: bool,
}

/// Выполняет `0x60109`: очищает все faction apply-list и уведомляет
/// только о подтверждённом переходе из положительной apply-faction в пустую.
pub(crate) fn dispatch_faction_application_cancel<Context>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    context: &mut Context,
) -> Option<
    Result<OrganizingFactionApplicationCancelDispatch, OrganizingFactionApplicationCancelBlock>,
>
where
    Context: FactionOrganizingInfoContext,
{
    if message.message_type() != CANCEL_FACTION_APPLICATION_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let previous_faction_id = match organizing.faction_by_player_in_apply_list(player_id) {
        ApplyFactionLookup::NoFaction => 0,
        ApplyFactionLookup::Faction(faction_id) => faction_id,
        ApplyFactionLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionApplicationCancelBlock::Lookup {
                phase: FactionApplicationCancelLookupPhase::BeforeRemoval,
                map_key,
            }));
        }
    };
    let removal = organizing.remove_person_from_apply_faction_list(game, player_id);
    if matches!(
        &removal,
        RemovePersonFromApplyFactionListOutcome::BlockedNullFaction { .. }
    ) {
        return Some(Err(OrganizingFactionApplicationCancelBlock::Removal {
            previous_faction_id,
            outcome: removal,
        }));
    }

    let mut remaining_faction_id = None;
    let mut notice_sent = false;
    if previous_faction_id > 0 {
        let remaining = match organizing.faction_by_player_in_apply_list(player_id) {
            ApplyFactionLookup::NoFaction => 0,
            ApplyFactionLookup::Faction(faction_id) => faction_id,
            ApplyFactionLookup::BlockedNullFaction { map_key } => {
                return Some(Err(OrganizingFactionApplicationCancelBlock::Lookup {
                    phase: FactionApplicationCancelLookupPhase::AfterRemoval,
                    map_key,
                }));
            }
        };
        remaining_faction_id = Some(remaining);
        if remaining <= 0 {
            send_faction_application_cancel_notice(context, player_id);
            notice_sent = true;
        }
    }

    Some(Ok(OrganizingFactionApplicationCancelDispatch {
        player_id,
        previous_faction_id,
        remaining_faction_id,
        removal,
        notice_sent,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionApplicationDecisionDispatch {
    pub(crate) manager_id: i32,
    pub(crate) applicant_id: i32,
    pub(crate) approve_flag: i32,
    pub(crate) outcome: OrganizingFactionDoJoinOutcome,
}

/// Выполняет `0x6010A`: три `Long`, manager-faction lookup, один local
/// time snapshot и virtual `CFaction::DoJoin` без route/tail/wire ingress-а.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_application_decision(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &CGoodsWarMember,
    use_log_system: bool,
    faction_join_log_enabled: bool,
    write_faction_join_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<
    Result<OrganizingFactionApplicationDecisionDispatch, OrganizingFactionDoJoinBlock>,
> {
    if message.message_type() != FACTION_APPLICATION_DECISION_MESSAGE_TYPE {
        return None;
    }

    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let applicant_id = message.base_mut().get_long().unwrap_or(0);
    let approve_flag = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldFactionDoJoinEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_join_log_enabled,
        callbacks,
        update_player,
        write_faction_join_log,
    };
    let outcome = match organizing.do_faction_join_by_manager(
        game,
        parameters,
        manager_id,
        applicant_id,
        approve_flag,
        current_local_member_time,
        &mut effects,
    ) {
        Ok(outcome) => outcome,
        Err(source) => return Some(Err(source)),
    };
    Some(Ok(OrganizingFactionApplicationDecisionDispatch {
        manager_id,
        applicant_id,
        approve_flag,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionFireOutOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionFireOutOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionFireOutBlock {
    ManagerMembership { map_key: i32 },
    FireOut {
        faction_id: i32,
        source: FactionFireOutBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionFireOutDispatch {
    pub(crate) manager_id: i32,
    pub(crate) target_id: i32,
    pub(crate) outcome: OrganizingFactionFireOutOutcome,
}

/// Выполняет `0x6010B`: два `Long`, manager-faction lookup и virtual
/// `CFaction::FireOut` без route/tail/wire ingress-а.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_fire_out(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &mut CGoodsWarMember,
    use_log_system: bool,
    faction_fire_out_log_enabled: bool,
    write_faction_fire_out_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionFireOutDispatch, OrganizingFactionFireOutBlock>> {
    if message.message_type() != FACTION_FIRE_OUT_MESSAGE_TYPE {
        return None;
    }

    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let target_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(manager_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionFireOutBlock::ManagerMembership {
                map_key,
            }));
        }
    };
    let Some(faction) = organizing.faction_by_id_mut(faction_id) else {
        return Some(Ok(OrganizingFactionFireOutDispatch {
            manager_id,
            target_id,
            outcome: OrganizingFactionFireOutOutcome::FactionNotFound { faction_id },
        }));
    };
    let mut effects = WorldFactionFireOutEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_fire_out_log_enabled,
        callbacks,
        update_player,
        write_faction_fire_out_log,
    };
    let outcome = match faction.fire_out(game, parameters, manager_id, target_id, &mut effects) {
        Ok(outcome) => OrganizingFactionFireOutOutcome::Applied {
            faction_id,
            outcome,
        },
        Err(source) => {
            return Some(Err(OrganizingFactionFireOutBlock::FireOut {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionFireOutDispatch {
        manager_id,
        target_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionFireOutDispatch {
    pub(crate) manager_id: i32,
    pub(crate) target_faction_id: i32,
    pub(crate) outcome: OrganizingUnionFireOutOutcome,
}

/// Выполняет `0x6010C`: два `Long`, nullable `GetUnion(manager)`, virtual
/// `CUnion::FireOut` и автоматический disband при member count `<= 1`.
pub(crate) fn dispatch_union_fire_out(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>> {
    if message.message_type() != UNION_FIRE_OUT_MESSAGE_TYPE {
        return None;
    }

    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let outcome = match organizing.fire_out_union_by_master(
        game,
        parameters,
        manager_id,
        target_faction_id,
        &mut effects,
        update_player,
    ) {
        Ok(outcome) => outcome,
        Err(source) => return Some(Err(source)),
    };
    Some(Ok(OrganizingUnionFireOutDispatch {
        manager_id,
        target_faction_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionExitOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionExitOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionExitBlock {
    Membership { map_key: i32 },
    Exit {
        faction_id: i32,
        source: FactionExitBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionExitDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingFactionExitOutcome,
}

/// Выполняет `0x6010D`: один `Long`, faction lookup и virtual
/// `CFaction::Exit` без route/tail/wire ingress-а.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_exit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &mut CGoodsWarMember,
    use_log_system: bool,
    faction_quit_log_enabled: bool,
    write_faction_quit_log: &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionExitDispatch, OrganizingFactionExitBlock>> {
    if message.message_type() != FACTION_EXIT_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionExitBlock::Membership { map_key }));
        }
    };
    let Some(faction) = organizing.faction_by_id_mut(faction_id) else {
        return Some(Ok(OrganizingFactionExitDispatch {
            player_id,
            outcome: OrganizingFactionExitOutcome::FactionNotFound { faction_id },
        }));
    };
    let mut effects = WorldFactionExitEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_quit_log_enabled,
        callbacks,
        update_player,
        write_faction_quit_log,
    };
    let outcome = match faction.exit(game, parameters, player_id, &mut effects) {
        Ok(outcome) => OrganizingFactionExitOutcome::Applied {
            faction_id,
            outcome,
        },
        Err(source) => {
            return Some(Err(OrganizingFactionExitBlock::Exit {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionExitDispatch { player_id, outcome }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionExitDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingUnionExitOutcome,
}

/// Выполняет `0x6010E`: один `Long`, player/faction/union lookup,
/// virtual `CUnion::Exit` и automatic disband через `GetPlayerHeader`.
pub(crate) fn dispatch_union_exit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>> {
    if message.message_type() != UNION_EXIT_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let outcome = match organizing.exit_union_by_player(
        game,
        parameters,
        player_id,
        &mut effects,
        update_player,
    ) {
        Ok(outcome) => outcome,
        Err(source) => return Some(Err(source)),
    };
    Some(Ok(OrganizingUnionExitDispatch { player_id, outcome }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDemiseOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionDemiseOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDemiseBlock {
    Membership { map_key: i32 },
    Demise {
        faction_id: i32,
        source: FactionDemiseBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionDemiseDispatch {
    pub(crate) old_master_id: i32,
    pub(crate) new_master_id: i32,
    pub(crate) outcome: OrganizingFactionDemiseOutcome,
}

/// Выполняет `0x6010F`: два `Long`, faction lookup и virtual
/// `CFaction::Demise` без route/tail/wire ingress-а.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_demise(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    country_handler: &CCountryHandler,
    attack_city: &CAttackCitySys,
    goods_war: &CGoodsWarMember,
    faction_master_log_enabled: bool,
    write_faction_master_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionDemiseDispatch, OrganizingFactionDemiseBlock>> {
    if message.message_type() != FACTION_DEMISE_MESSAGE_TYPE {
        return None;
    }

    let old_master_id = message.base_mut().get_long().unwrap_or(0);
    let new_master_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(old_master_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionDemiseBlock::Membership { map_key }));
        }
    };
    let Some(faction) = organizing.faction_by_id_mut(faction_id) else {
        return Some(Ok(OrganizingFactionDemiseDispatch {
            old_master_id,
            new_master_id,
            outcome: OrganizingFactionDemiseOutcome::FactionNotFound { faction_id },
        }));
    };
    let mut effects = WorldFactionDemiseEffects {
        game,
        attack_city,
        goods_war,
        country_handler,
        faction_master_log_enabled,
        callbacks,
        update_player,
        write_faction_master_log,
    };
    let outcome = match faction.demise(
        game,
        parameters,
        old_master_id,
        new_master_id,
        &mut effects,
    ) {
        Ok(outcome) => OrganizingFactionDemiseOutcome::Applied {
            faction_id,
            outcome,
        },
        Err(source) => {
            return Some(Err(OrganizingFactionDemiseBlock::Demise {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionDemiseDispatch {
        old_master_id,
        new_master_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionDemiseDispatch {
    pub(crate) old_master_player_id: i32,
    pub(crate) new_master_faction_id: i32,
    pub(crate) outcome: OrganizingUnionDemiseOutcome,
}

/// Выполняет `0x60110`: два полных `Long`, nullable
/// `GetUnion(old master)` и virtual `CUnion::Demise(old, new faction)`.
pub(crate) fn dispatch_union_demise(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>> {
    if message.message_type() != UNION_DEMISE_MESSAGE_TYPE {
        return None;
    }

    let old_master_player_id = message.base_mut().get_long().unwrap_or(0);
    let new_master_faction_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let mut get_tick = legacy_tick_ms;
    let outcome = match organizing.demise_union_by_master(
        game,
        old_master_player_id,
        new_master_faction_id,
        &mut effects,
        &mut get_tick,
        update_player,
    ) {
        Ok(outcome) => outcome,
        Err(source) => return Some(Err(source)),
    };
    Some(Ok(OrganizingUnionDemiseDispatch {
        old_master_player_id,
        new_master_faction_id,
        outcome,
    }))
}

pub(crate) struct PendingOrganizingFactionDisbandDispatch {
    player_id: i32,
    faction_id: i32,
    outcome: OrganizingDisbandOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDisbandOutcome {
    Rejected {
        reason: OrganizingDisbandRejection,
        notice_sent: bool,
        cleared_city_war_enemies: usize,
    },
    Disbanded {
        progress: OrganizingDisbandProgress,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionDisbandDispatch {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) outcome: OrganizingFactionDisbandOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDisbandBlock {
    Membership { map_key: i32 },
    Disband {
        faction_id: i32,
        source: OrganizingDisbandBlock,
    },
}

/// Выполняет synchronous prefix `0x60111`: один `Long`, ordered
/// `IsFreePlayer` и безусловный `DisbandFaction(player, faction)`.
pub(crate) fn dispatch_faction_disband<Context>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    context: &mut Context,
) -> Option<
    Result<PendingOrganizingFactionDisbandDispatch, OrganizingFactionDisbandBlock>,
>
where
    Context: FactionDisbandContext,
{
    if message.message_type() != FACTION_DISBAND_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionDisbandBlock::Membership { map_key }));
        }
    };
    let outcome = match organizing.disband_faction(game, player_id, faction_id, context) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Some(Err(OrganizingFactionDisbandBlock::Disband {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(PendingOrganizingFactionDisbandDispatch {
        player_id,
        faction_id,
        outcome,
    }))
}

/// Завершает controller continuation: player flag, optional DB-log и
/// немедленный Drop удалённого faction-owner-а.
pub(crate) fn finalize_faction_disband_dispatch<ClearPlayer, WriteLog>(
    pending: PendingOrganizingFactionDisbandDispatch,
    faction_disband_log_enabled: bool,
    mut clear_player: ClearPlayer,
    mut write_log: WriteLog,
) -> OrganizingFactionDisbandDispatch
where
    ClearPlayer: FnMut(i32) -> Option<OrganizingDisbandPlayer>,
    WriteLog: FnMut(i32, &[u8], i32, &[u8]),
{
    let PendingOrganizingFactionDisbandDispatch {
        player_id,
        faction_id,
        outcome,
    } = pending;
    let outcome = match outcome {
        OrganizingDisbandOutcome::Rejected {
            reason,
            notice_sent,
            cleared_city_war_enemies,
        } => OrganizingFactionDisbandOutcome::Rejected {
            reason,
            notice_sent,
            cleared_city_war_enemies,
        },
        OrganizingDisbandOutcome::Disbanded {
            mut progress,
            retired_faction,
        } => {
            progress.player = clear_player(player_id);
            if faction_disband_log_enabled
                && let Some(player) = progress.player.as_ref()
            {
                write_log(
                    faction_id,
                    legacy_c_string_prefix(retired_faction.name()),
                    player.player_id,
                    legacy_c_string_prefix(&player.player_name),
                );
                progress.log_written = true;
            }
            drop(retired_faction);
            OrganizingFactionDisbandOutcome::Disbanded { progress }
        }
    };
    OrganizingFactionDisbandDispatch {
        player_id,
        faction_id,
        outcome,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionDisbandOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: OrganizingConfederationDisbandOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionDisbandBlock {
    Lookup(OrganizingUnionByMasterBlock),
    Disband {
        union_id: i32,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionDisbandDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingUnionDisbandOutcome,
}

/// Выполняет `0x60112`: один `Long`, nullable `GetUnion(player)`,
/// virtual `GetID` и `DisbandConferation(player, union ID)`.
pub(crate) fn dispatch_union_disband(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>> {
    if message.message_type() != UNION_DISBAND_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let union_id = match organizing.union_id_by_master_player(player_id) {
        Ok(Some(union_id)) => union_id,
        Ok(None) => {
            return Some(Ok(OrganizingUnionDisbandDispatch {
                player_id,
                outcome: OrganizingUnionDisbandOutcome::UnionNotFound,
            }));
        }
        Err(source) => return Some(Err(OrganizingUnionDisbandBlock::Lookup(source))),
    };
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let outcome = match organizing.disband_confederation(
        game,
        parameters,
        player_id,
        union_id,
        &mut effects,
        update_player,
    ) {
        Ok(outcome) => OrganizingUnionDisbandOutcome::Applied { union_id, outcome },
        Err(source) => {
            return Some(Err(OrganizingUnionDisbandBlock::Disband {
                union_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingUnionDisbandDispatch { player_id, outcome }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDubOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionDubOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDubBlock {
    Membership { map_key: i32 },
    Dub {
        faction_id: i32,
        source: FactionDubBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionDubDispatch {
    pub(crate) target_id: i32,
    pub(crate) job_level: i32,
    pub(crate) title: Vec<u8>,
    pub(crate) manager_id: i32,
    pub(crate) outcome: OrganizingFactionDubOutcome,
}

/// Выполняет `0x60113`: `(target, job-level, title[20], manager)`,
/// faction lookup по manager и virtual `CFaction::DubAndSetJobLvl`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_dub(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    use_log_system: bool,
    faction_title_log_enabled: bool,
    write_faction_title_log: &mut dyn FnMut(
        i32,
        &[u8],
        &[u8],
        &[u8],
        i32,
        &[u8],
        i32,
        &[u8],
    ),
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionDubDispatch, OrganizingFactionDubBlock>> {
    if message.message_type() != FACTION_DUB_MESSAGE_TYPE {
        return None;
    }

    let target_id = message.base_mut().get_long().unwrap_or(0);
    let job_level = message.base_mut().get_long().unwrap_or(0);
    let mut title = message.base_mut().get_str_bytes(20).unwrap_or_default();
    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(manager_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionDubBlock::Membership { map_key }));
        }
    };
    let Some(faction) = organizing.faction_by_id_mut(faction_id) else {
        return Some(Ok(OrganizingFactionDubDispatch {
            target_id,
            job_level,
            title,
            manager_id,
            outcome: OrganizingFactionDubOutcome::FactionNotFound { faction_id },
        }));
    };
    let mut effects = WorldFactionDubEffects {
        game,
        callbacks,
        update_player,
        use_log_system,
        faction_title_log_enabled,
        write_faction_title_log,
    };
    let outcome = match faction.dub_and_set_job_level(
        game,
        manager_id,
        target_id,
        &mut title,
        job_level,
        &mut effects,
    ) {
        Ok(outcome) => OrganizingFactionDubOutcome::Applied {
            faction_id,
            outcome,
        },
        Err(source) => {
            return Some(Err(OrganizingFactionDubBlock::Dub {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionDubDispatch {
        target_id,
        job_level,
        title,
        manager_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionPurviewOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionPurviewChangeOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionPurviewBlock {
    Membership { map_key: i32 },
    Change {
        faction_id: i32,
        source: FactionPurviewChangeBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionPurviewDispatch {
    pub(crate) change: FactionPurviewChange,
    pub(crate) target_id: i32,
    pub(crate) purview: i32,
    pub(crate) manager_id: i32,
    pub(crate) outcome: OrganizingFactionPurviewOutcome,
}

/// Выполняет парные `0x60114/0x60115`: три `Long`, faction lookup по
/// manager и virtual grant/revoke owner без ingress-gates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_purview(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    use_log_system: bool,
    add_log_enabled: bool,
    revoke_log_enabled: bool,
    write_log: &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
) -> Option<Result<OrganizingFactionPurviewDispatch, OrganizingFactionPurviewBlock>> {
    let change = match message.message_type() {
        GRANT_FACTION_PURVIEW_MESSAGE_TYPE => FactionPurviewChange::Grant,
        REVOKE_FACTION_PURVIEW_MESSAGE_TYPE => FactionPurviewChange::Revoke,
        _ => return None,
    };
    let target_id = message.base_mut().get_long().unwrap_or(0);
    let purview = message.base_mut().get_long().unwrap_or(0);
    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(manager_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionPurviewBlock::Membership { map_key }));
        }
    };
    let Some(faction) = organizing.faction_by_id_mut(faction_id) else {
        return Some(Ok(OrganizingFactionPurviewDispatch {
            change,
            target_id,
            purview,
            manager_id,
            outcome: OrganizingFactionPurviewOutcome::FactionNotFound { faction_id },
        }));
    };
    let mut effects = WorldFactionPurviewEffects {
        game,
        callbacks,
        use_log_system,
        add_log_enabled,
        revoke_log_enabled,
        write_log,
    };
    let changed = match change {
        FactionPurviewChange::Grant => faction.endue_right_to_member(
            game,
            manager_id,
            target_id,
            purview,
            &mut effects,
        ),
        FactionPurviewChange::Revoke => faction.abolish_right_to_member(
            game,
            manager_id,
            target_id,
            purview,
            &mut effects,
        ),
    };
    let outcome = match changed {
        Ok(outcome) => OrganizingFactionPurviewOutcome::Applied {
            faction_id,
            outcome,
        },
        Err(source) => {
            return Some(Err(OrganizingFactionPurviewBlock::Change {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionPurviewDispatch {
        change,
        target_id,
        purview,
        manager_id,
        outcome,
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingConsumedLongDispatch {
    pub(crate) message_type: i32,
    pub(crate) value: i32,
}

pub(crate) fn dispatch_consumed_long(
    message: &mut CMessage,
) -> Option<OrganizingConsumedLongDispatch> {
    let message_type = message.message_type();
    if !CONSUMED_LONG_MESSAGE_TYPES.contains(&message_type) {
        return None;
    }
    let value = message.base_mut().get_long().unwrap_or(0);
    Some(OrganizingConsumedLongDispatch {
        message_type,
        value,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListNotice {
    MissingFaction,
    MasterRequired,
    NoFactions,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareWarFactionListResponse {
    pub(crate) socket_id: i32,
    pub(crate) total_factions: i32,
    pub(crate) included_page: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListOutcome {
    Empty {
        faction_id: i32,
        notice: OrganizingDeclareWarFactionListNotice,
        response: OrganizingDeclareWarFactionListResponse,
    },
    PageOutsideRange {
        faction_id: i32,
        total_factions: i32,
        page: i32,
    },
    Page {
        faction_id: i32,
        page: DeclareWarFactionPage,
        response: OrganizingDeclareWarFactionListResponse,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListBlock {
    Membership { map_key: i32 },
    Page(DeclareWarFactionPageBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareWarFactionListDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) page: i32,
    pub(crate) outcome: OrganizingDeclareWarFactionListOutcome,
}

pub(crate) fn dispatch_declare_war_faction_list<Context>(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    faction_wars: &CFactionWarSys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<
    Result<OrganizingDeclareWarFactionListDispatch, OrganizingDeclareWarFactionListBlock>,
>
where
    Context: FactionOrganizingInfoContext,
{
    if message.message_type() != DECLARE_WAR_FACTION_LIST_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let page = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingDeclareWarFactionListBlock::Membership { map_key }));
        }
    };

    let outcome = if organizing.faction_by_id(faction_id).is_none() {
        send_declare_war_faction_list_notice(
            context,
            player_id,
            OrganizingDeclareWarFactionListNotice::MissingFaction,
        );
        OrganizingDeclareWarFactionListOutcome::Empty {
            faction_id,
            notice: OrganizingDeclareWarFactionListNotice::MissingFaction,
            response: send_declare_war_faction_list_response(
                sender, socket_id, player_id, 0, request_id, cookie, None,
            ),
        }
    } else if organizing
        .faction_by_id(faction_id)
        .is_some_and(|faction| faction.is_master(player_id) == 0)
    {
        send_declare_war_faction_list_notice(
            context,
            player_id,
            OrganizingDeclareWarFactionListNotice::MasterRequired,
        );
        OrganizingDeclareWarFactionListOutcome::Empty {
            faction_id,
            notice: OrganizingDeclareWarFactionListNotice::MasterRequired,
            response: send_declare_war_faction_list_response(
                sender, socket_id, player_id, 0, request_id, cookie, None,
            ),
        }
    } else {
        let total_factions = match organizing.declare_war_faction_count() {
            Ok(total_factions) => total_factions,
            Err(block) => {
                return Some(Err(OrganizingDeclareWarFactionListBlock::Page(block)));
            }
        };
        if total_factions == 0 {
            send_declare_war_faction_list_notice(
                context,
                player_id,
                OrganizingDeclareWarFactionListNotice::NoFactions,
            );
            OrganizingDeclareWarFactionListOutcome::Empty {
                faction_id,
                notice: OrganizingDeclareWarFactionListNotice::NoFactions,
                response: send_declare_war_faction_list_response(
                    sender, socket_id, player_id, 0, request_id, cookie, None,
                ),
            }
        } else {
            let start = page.wrapping_mul(11).wrapping_sub(11);
            if start >= total_factions {
                OrganizingDeclareWarFactionListOutcome::PageOutsideRange {
                    faction_id,
                    total_factions,
                    page,
                }
            } else {
                let page_data = match organizing.declare_war_faction_page(
                    faction_id,
                    page,
                    faction_wars,
                ) {
                    Ok(page_data) => page_data,
                    Err(block) => {
                        return Some(Err(OrganizingDeclareWarFactionListBlock::Page(block)));
                    }
                };
                let response = send_declare_war_faction_list_response(
                    sender,
                    socket_id,
                    player_id,
                    total_factions,
                    request_id,
                    cookie,
                    Some(&page_data),
                );
                OrganizingDeclareWarFactionListOutcome::Page {
                    faction_id,
                    page: page_data,
                    response,
                }
            }
        }
    };

    Some(Ok(OrganizingDeclareWarFactionListDispatch {
        request_id,
        cookie,
        player_id,
        page,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareFactionWarResponse {
    pub(crate) socket_id: i32,
    pub(crate) result_money: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareFactionWarOutcome {
    PlayerOffline,
    Declaration(FactionWarDeclarationOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareFactionWarBlock {
    PlayerDecode(PlayerCodecError),
    Declaration(FactionWarDeclarationBlock<OrganizingFactionWarDeclarationBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareFactionWarDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) target_faction_id: i32,
    pub(crate) war_type: i32,
    pub(crate) outcome: OrganizingDeclareFactionWarOutcome,
    pub(crate) response: OrganizingDeclareFactionWarResponse,
}

pub(crate) fn dispatch_declare_faction_war(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    faction_wars: &mut CFactionWarSys,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    update_player: &mut dyn FnMut(i32),
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingDeclareFactionWarDispatch, OrganizingDeclareFactionWarBlock>> {
    if message.message_type() != DECLARE_FACTION_WAR_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let war_type = message.base_mut().get_long().unwrap_or(0);
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingDeclareFactionWarBlock::PlayerDecode(source)));
            }
        }
    };

    let (outcome, result_money) = if player_online {
        let declaration_time = TagTime::local_now();
        let mut effects = WorldFactionWarDeclarationEffects::new(
            game,
            organizing,
            update_player,
        );
        let declaration = match faction_wars.dig_up_the_hatchet(
            player_id,
            target_faction_id,
            war_type,
            declaration_time,
            &mut effects,
        ) {
            Ok(declaration) => declaration,
            Err(source) => {
                return Some(Err(OrganizingDeclareFactionWarBlock::Declaration(source)));
            }
        };
        let result_money = if declaration.legacy_result() {
            faction_wars.get_dec_war_money_by_type(war_type)
        } else {
            0
        };
        (
            OrganizingDeclareFactionWarOutcome::Declaration(declaration),
            result_money,
        )
    } else {
        (OrganizingDeclareFactionWarOutcome::PlayerOffline, 0)
    };

    let mut response = CMessage::new(DECLARE_FACTION_WAR_RESPONSE_TYPE);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(result_money);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);

    Some(Ok(OrganizingDeclareFactionWarDispatch {
        request_id,
        cookie,
        player_id,
        target_faction_id,
        war_type,
        outcome,
        response: OrganizingDeclareFactionWarResponse {
            socket_id,
            result_money,
            wire,
            delivery,
        },
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionBillboardResponse {
    pub(crate) socket_id: i32,
    pub(crate) title: Vec<u8>,
    pub(crate) payload: Vec<u8>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionBillboardOutcome {
    TypeAboveRange {
        request_id: i32,
        billboard_type: i32,
    },
    Sent {
        request_id: i32,
        billboard_type: i32,
        response: OrganizingFactionBillboardResponse,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionBillboardBlock {
    pub(crate) request_id: i32,
    pub(crate) billboard_type: i32,
}

pub(crate) fn dispatch_faction_billboard(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    world_string: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>> {
    if message.message_type() != FACTION_BILLBOARD_MESSAGE_TYPE {
        return None;
    }

 // Три `GetStringByID` исходник выполнял при первом входе в case, ещё до
 // чтения request и проверки типа, после чего process-static строки уже не
 // реагировали на reload string table.
    let titles = FACTION_BILLBOARD_TITLES.get_or_init(|| {
        [
            legacy_c_string_prefix(&world_string(b"WS0134")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0133")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0132")).to_vec(),
        ]
    });
    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long().unwrap_or(0);
    let billboard_type = message.base_mut().get_long().unwrap_or(0);
    if billboard_type > 2 {
        return Some(Ok(OrganizingFactionBillboardOutcome::TypeAboveRange {
            request_id,
            billboard_type,
        }));
    }
    let Ok(title_index) = usize::try_from(billboard_type) else {
 // Оригинал проверяет только `2 < type`, поэтому отрицательный selector
 // индексирует статический `std::string[3]` до начала массива. Rust не
 // воспроизводит результат такого чтения за границами.
        return Some(Err(OrganizingFactionBillboardBlock {
            request_id,
            billboard_type,
        }));
    };

    let title = titles[title_index].clone();
    let mut payload = Vec::new();
    organizing.add_faction_billboard_to_byte_array(&mut payload, billboard_type);

    let mut response = CMessage::new(FACTION_BILLBOARD_RESPONSE_TYPE);
    response.base_mut().add_long(request_id);
    response.base_mut().add(&title);
    response.base_mut().add_byte(0);
    response.base_mut().add(&payload);
    response.base_mut().update();
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);

    Some(Ok(OrganizingFactionBillboardOutcome::Sent {
        request_id,
        billboard_type,
        response: OrganizingFactionBillboardResponse {
            socket_id,
            title,
            payload,
            wire,
            delivery,
        },
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUpgradeOutcome {
    PlayerOffline,
    FactionMissing,
    Upgrade(FactionUpgradeOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUpgradeBlock {
    PlayerDecode(PlayerCodecError),
    Upgrade(FactionUpgradeBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionUpgradeDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingFactionUpgradeOutcome,
}

#[allow(
    clippy::too_many_arguments,
    reason = "opcode использует прежние game/faction/goods/string/log singleton-ы"
)]
pub(crate) fn dispatch_faction_upgrade(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionUpgradeDispatch, OrganizingFactionUpgradeBlock>> {
    if message.message_type() != UPGRADE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingFactionUpgradeBlock::PlayerDecode(source)));
            }
        }
    };
    if !player_online {
        return Some(Ok(OrganizingFactionUpgradeDispatch {
            faction_id,
            player_id,
            outcome: OrganizingFactionUpgradeOutcome::PlayerOffline,
        }));
    }

    let game_ref: &CGame = game;
    let mut effects = WorldFactionUpgradeEffects {
        game: game_ref,
        registry,
        original_name_index,
        use_log_system,
        callbacks,
        update_player,
    };
    let outcome = match organizing.upgrade_faction(
        game_ref,
        parameters,
        faction_id,
        player_id,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionUpgradeOutcome::Upgrade(outcome),
        Ok(None) => OrganizingFactionUpgradeOutcome::FactionMissing,
        Err(source) => return Some(Err(OrganizingFactionUpgradeBlock::Upgrade(source))),
    };

    Some(Ok(OrganizingFactionUpgradeDispatch {
        faction_id,
        player_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUploadIconOutcome {
    FactionMissing,
    UploadIcon {
        time: TagTimeValue,
        outcome: FactionUploadIconOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionUploadIconDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingFactionUploadIconOutcome,
}

pub(crate) fn dispatch_faction_upload_icon(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>> {
    if message.message_type() != UPLOAD_FACTION_ICON_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    if organizing.faction_by_id(faction_id).is_none() {
        return Some(Ok(OrganizingFactionUploadIconDispatch {
            faction_id,
            player_id,
            outcome: OrganizingFactionUploadIconOutcome::FactionMissing,
        }));
    }

    let time = capture_local_tag_time();
    let mut effects = WorldFactionUploadIconEffects { game, callbacks };
    let outcome = match organizing.upload_faction_icon(
        parameters,
        faction_id,
        player_id,
        &time,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionUploadIconOutcome::UploadIcon { time, outcome },
        Ok(None) => OrganizingFactionUploadIconOutcome::FactionMissing,
        Err(source) => return Some(Err(source)),
    };

    Some(Ok(OrganizingFactionUploadIconDispatch {
        faction_id,
        player_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionContributorDispatch {
    pub(crate) target_player_id: i32,
    pub(crate) enabled_value: i32,
    pub(crate) requester_player_id: i32,
    pub(crate) outcome: OrganizingContributorOutcome,
}

pub(crate) fn dispatch_faction_contributor(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>> {
    if message.message_type() != SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE {
        return None;
    }

    let target_player_id = message.base_mut().get_long().unwrap_or(0);
    let enabled_value = message.base_mut().get_long().unwrap_or(0);
    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldFactionContributorEffects {
        game,
        callbacks,
        update_player,
    };
    Some(
        organizing
            .set_contributor_for_player(
                game,
                parameters,
                requester_player_id,
                target_player_id,
                enabled_value != 0,
                &mut effects,
            )
            .map(|outcome| OrganizingFactionContributorDispatch {
                target_player_id,
                enabled_value,
                requester_player_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionExperienceOutcome {
    FactionMissing,
    PlayerNotContributor,
    Applied {
        before_experience: i32,
        update: FactionExperienceUpdate,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionExperienceDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) experience_delta: i32,
    pub(crate) outcome: OrganizingFactionExperienceOutcome,
}

pub(crate) fn dispatch_faction_experience(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>> {
    if message.message_type() != ADD_FACTION_EXPERIENCE_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let experience_delta = message.base_mut().get_long().unwrap_or(0);
    let mutation = match organizing.add_contributor_experience(
        game,
        faction_id,
        player_id,
        experience_delta,
    ) {
        Ok(mutation) => mutation,
        Err(source) => return Some(Err(source)),
    };
    let (actual_faction_id, faction_name, before_experience, update) = match mutation {
        OrganizingFactionExperienceMutation::FactionNotFound => {
            return Some(Ok(OrganizingFactionExperienceDispatch {
                faction_id,
                player_id,
                experience_delta,
                outcome: OrganizingFactionExperienceOutcome::FactionMissing,
            }));
        }
        OrganizingFactionExperienceMutation::PlayerNotContributor => {
            return Some(Ok(OrganizingFactionExperienceDispatch {
                faction_id,
                player_id,
                experience_delta,
                outcome: OrganizingFactionExperienceOutcome::PlayerNotContributor,
            }));
        }
        OrganizingFactionExperienceMutation::Applied {
            faction_id,
            faction_name,
            before_experience,
            update,
        } => (faction_id, faction_name, before_experience, update),
    };

    let mut log_written = false;
    if use_log_system && callbacks.faction_experience_log_enabled {
        if let Some(player) = game.online_player_by_id(player_id as u32) {
            (callbacks.write_faction_experience_log)(
                actual_faction_id,
                &faction_name,
                player.get_id(),
                player.get_name(),
                before_experience,
                experience_delta,
            );
            log_written = true;
        }
    }

    Some(Ok(OrganizingFactionExperienceDispatch {
        faction_id,
        player_id,
        experience_delta,
        outcome: OrganizingFactionExperienceOutcome::Applied {
            before_experience,
            update,
            log_written,
        },
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionMemberStateDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) operation: i32,
    pub(crate) outcome: OrganizingFactionMemberStateOutcome,
}

pub(crate) fn dispatch_faction_member_state(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<OrganizingFactionMemberStateDispatch> {
    if message.message_type() != CHANGE_FACTION_MEMBER_STATE_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let operation = message.base_mut().get_long().unwrap_or(0);
    let outcome = organizing.change_faction_member_state(
        game,
        faction_id,
        player_id,
        operation,
        || message.base_mut().get_long().unwrap_or(0),
    );
    Some(OrganizingFactionMemberStateDispatch {
        faction_id,
        player_id,
        operation,
        outcome,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionTaxResponse {
    pub(crate) socket_id: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionTaxOutcome {
    FactionNotFound,
    AttackCityFight,
    VillageWarFight,
    Rejected {
        faction_id: i32,
        reason: FactionOperationRejection,
    },
    Authorized {
        faction_id: i32,
        response: OrganizingFactionTaxResponse,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionTaxBlock {
    Membership { map_key: i32 },
    Operation {
        faction_id: i32,
        source: FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionTaxDispatch {
    pub(crate) request_type: i32,
    pub(crate) player_id: i32,
    pub(crate) region_id: i32,
    pub(crate) outcome: OrganizingFactionTaxOutcome,
}

pub(crate) fn dispatch_faction_tax<Context>(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    attack_city: &CAttackCitySys,
    village_war: &CVillageWarSys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionTaxDispatch, OrganizingFactionTaxBlock>>
where
    Context: FactionOrganizingInfoContext,
{
    let request_type = message.message_type();
    let response_type = match request_type {
        OPERATE_FACTION_TAX_MESSAGE_TYPE => OPERATE_FACTION_TAX_RESPONSE_TYPE,
        ADJUST_FACTION_TAX_MESSAGE_TYPE => ADJUST_FACTION_TAX_RESPONSE_TYPE,
        _ => return None,
    };

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let region_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => {
            return Some(Ok(OrganizingFactionTaxDispatch {
                request_type,
                player_id,
                region_id,
                outcome: OrganizingFactionTaxOutcome::FactionNotFound,
            }));
        }
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingFactionTaxBlock::Membership { map_key }));
        }
    };
    if organizing.faction_by_id(faction_id).is_none() {
        return Some(Ok(OrganizingFactionTaxDispatch {
            request_type,
            player_id,
            region_id,
            outcome: OrganizingFactionTaxOutcome::FactionNotFound,
        }));
    }

    if attack_city.get_city_state(region_id) == ECityState::Fight {
        send_faction_tax_notice(context, player_id, b"WS0126");
        return Some(Ok(OrganizingFactionTaxDispatch {
            request_type,
            player_id,
            region_id,
            outcome: OrganizingFactionTaxOutcome::AttackCityFight,
        }));
    }
    if village_war.get_region_state(region_id) == ECityState::Fight {
        send_faction_tax_notice(context, player_id, b"WS0127");
        return Some(Ok(OrganizingFactionTaxDispatch {
            request_type,
            player_id,
            region_id,
            outcome: OrganizingFactionTaxOutcome::VillageWarFight,
        }));
    }

    let operation = match organizing.operate_faction_tax(faction_id, player_id, region_id) {
        Ok(Some(operation)) => operation,
        Ok(None) => {
            return Some(Ok(OrganizingFactionTaxDispatch {
                request_type,
                player_id,
                region_id,
                outcome: OrganizingFactionTaxOutcome::FactionNotFound,
            }));
        }
        Err(source) => {
            return Some(Err(OrganizingFactionTaxBlock::Operation {
                faction_id,
                source,
            }));
        }
    };
    let outcome = match operation {
        FactionOperationOutcome::Rejected(reason) => OrganizingFactionTaxOutcome::Rejected {
            faction_id,
            reason,
        },
        FactionOperationOutcome::Authorized => {
            message.set_message_type(response_type);
            let socket_id = message.socket_id();
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_to_socket(sender, socket_id);
            OrganizingFactionTaxOutcome::Authorized {
                faction_id,
                response: OrganizingFactionTaxResponse {
                    socket_id,
                    wire,
                    delivery,
                },
            }
        }
    };
    Some(Ok(OrganizingFactionTaxDispatch {
        request_type,
        player_id,
        region_id,
        outcome,
    }))
}

fn send_faction_tax_notice<Context>(
    context: &mut Context,
    player_id: i32,
    first_string_id: &'static [u8],
) where
    Context: FactionOrganizingInfoContext,
{
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    let second_text = context.world_string(b"WS0121").unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRegionParamBroadcast {
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRegionParamDispatch {
    pub(crate) region_id: i32,
    pub(crate) today_total_tax: u32,
    pub(crate) total_tax: u32,
    pub(crate) current_tax_rate: i32,
    pub(crate) region: WorldRegionParamUpdateOutcome,
    pub(crate) broadcast: Option<OrganizingRegionParamBroadcast>,
}

pub(crate) fn dispatch_region_param_update(
    message: &mut CMessage,
    game: &mut CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingRegionParamDispatch> {
    if message.message_type() != UPDATE_REGION_PARAM_MESSAGE_TYPE {
        return None;
    }

    let region_id = message.base_mut().get_long().unwrap_or(0);
    let today_total_tax = message.base_mut().get_long().unwrap_or(0) as u32;
    let total_tax = message.base_mut().get_long().unwrap_or(0) as u32;
    let current_tax_rate = message.base_mut().get_long().unwrap_or(0);
    let region = game.set_region_param_from_game_server(
        region_id,
        current_tax_rate,
        today_total_tax,
        total_tax,
    );
    let broadcast = if region == WorldRegionParamUpdateOutcome::Applied {
        message.set_message_type(UPDATE_REGION_PARAM_RESPONSE_TYPE);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = message.send_all(sender);
        Some(OrganizingRegionParamBroadcast { wire, delivery })
    } else {
        None
    };
    Some(OrganizingRegionParamDispatch {
        region_id,
        today_total_tax,
        total_tax,
        current_tax_rate,
        region,
        broadcast,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRegionRouteDispatch {
    pub(crate) region_id: i32,
    pub(crate) game_server_number: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

pub(crate) fn dispatch_region_route(
    message: &mut CMessage,
    game: &CGame,
) -> Option<OrganizingRegionRouteDispatch> {
    if message.message_type() != ROUTE_REGION_MESSAGE_TYPE {
        return None;
    }

    let region_id = message.base_mut().get_long().unwrap_or(0);
    let game_server_number = game.game_server_number_by_region_id(region_id);
    message.set_message_type(ROUTE_REGION_RESPONSE_TYPE);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = game.send_msg_to_game_server(game_server_number, message);
    Some(OrganizingRegionRouteDispatch {
        region_id,
        game_server_number,
        wire,
        delivery,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCityGateResponse {
    pub(crate) game_server_number: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingCityGateOutcome {
    FactionNotFound,
    Rejected {
        faction_id: i32,
        reason: FactionOperationRejection,
    },
    Authorized {
        faction_id: i32,
        response: OrganizingCityGateResponse,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingCityGateBlock {
    Membership { map_key: i32 },
    Operation {
        faction_id: i32,
        source: FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCityGateDispatch {
    pub(crate) player_id: i32,
    pub(crate) region_id: i32,
    pub(crate) outcome: OrganizingCityGateOutcome,
}

pub(crate) fn dispatch_city_gate(
    message: &mut CMessage,
    game: &CGame,
    organizing: &COrganizingCtrl,
) -> Option<Result<OrganizingCityGateDispatch, OrganizingCityGateBlock>> {
    if message.message_type() != OPERATE_CITY_GATE_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let region_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => {
            return Some(Ok(OrganizingCityGateDispatch {
                player_id,
                region_id,
                outcome: OrganizingCityGateOutcome::FactionNotFound,
            }));
        }
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingCityGateBlock::Membership { map_key }));
        }
    };
    let operation = match organizing.operate_faction_city_gate(faction_id, player_id, region_id) {
        Ok(Some(operation)) => operation,
        Ok(None) => {
            return Some(Ok(OrganizingCityGateDispatch {
                player_id,
                region_id,
                outcome: OrganizingCityGateOutcome::FactionNotFound,
            }));
        }
        Err(source) => {
            return Some(Err(OrganizingCityGateBlock::Operation {
                faction_id,
                source,
            }));
        }
    };
    let outcome = match operation {
        FactionOperationOutcome::Rejected(reason) => OrganizingCityGateOutcome::Rejected {
            faction_id,
            reason,
        },
        FactionOperationOutcome::Authorized => {
            message.set_message_type(OPERATE_CITY_GATE_RESPONSE_TYPE);
            let game_server_number = game.game_server_number_by_region_id(region_id);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(game_server_number, message);
            OrganizingCityGateOutcome::Authorized {
                faction_id,
                response: OrganizingCityGateResponse {
                    game_server_number,
                    wire,
                    delivery,
                },
            }
        }
    };
    Some(Ok(OrganizingCityGateDispatch {
        player_id,
        region_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCityTransferDispatch<SessionReport> {
    pub(crate) requester_player_id: i32,
    pub(crate) target_faction_id: i32,
    pub(crate) region_id: i32,
    pub(crate) outcome: CityTransferStartOutcome<SessionReport>,
}

pub(crate) fn dispatch_city_transfer<Effects>(
    message: &mut CMessage,
    game: &CGame,
    countries: &CCountryHandler,
    organizing: &mut COrganizingCtrl,
    attack_city: &CAttackCitySys,
    village_war: &CVillageWarSys,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingCityTransferDispatch<Effects::SessionReport>,
        CityTransferStartBlock<Effects::SessionBlock>,
    >,
>
where
    Effects: CityTransferEffects,
{
    if message.message_type() != TRANSFER_CITY_OWNER_MESSAGE_TYPE {
        return None;
    }

    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let region_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = organizing.transfer_city_owner(
        game,
        countries,
        attack_city,
        village_war,
        requester_player_id,
        target_faction_id,
        region_id,
        effects,
    );
    Some(outcome.map(|outcome| OrganizingCityTransferDispatch {
        requester_player_id,
        target_faction_id,
        region_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingAdmissionPermitDispatch {
    pub(crate) requested_value: i32,
    pub(crate) player_id: i32,
    pub(crate) faction_id: Option<i32>,
    pub(crate) outcome: Option<FactionPermitUpdate>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingAdmissionPermitBlock {
    Membership { map_key: i32 },
    Permit {
        faction_id: i32,
        source: FactionPermitBlock,
    },
}

pub(crate) fn dispatch_admission_permit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingAdmissionPermitDispatch, OrganizingAdmissionPermitBlock>> {
    if message.message_type() != SET_FACTION_ADMISSION_PERMIT_MESSAGE_TYPE {
        return None;
    }

    let requested_value = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => {
            return Some(Ok(OrganizingAdmissionPermitDispatch {
                requested_value,
                player_id,
                faction_id: None,
                outcome: None,
            }));
        }
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingAdmissionPermitBlock::Membership { map_key }));
        }
    };
    let outcome = organizing
        .set_faction_admission_permit(game, faction_id, player_id, requested_value != 0)
        .map_err(|source| OrganizingAdmissionPermitBlock::Permit { faction_id, source });
    Some(outcome.map(|outcome| OrganizingAdmissionPermitDispatch {
        requested_value,
        player_id,
        faction_id: Some(faction_id),
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingAttackCityEndDispatch {
    pub(crate) result: i32,
    pub(crate) region_id: i32,
    pub(crate) attacker_player_id: i32,
    pub(crate) defender_faction_id: i32,
    pub(crate) outcome: AttackCityEndReport,
}

pub(crate) fn dispatch_attack_city_end<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    effects: &mut Effects,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingAttackCityEndDispatch, AttackCityEndBlock>>
where
    Effects: AttackCityEndEffects,
{
    if message.message_type() != ATTACK_CITY_END_MESSAGE_TYPE {
        return None;
    }

    let result = message.base_mut().get_long().unwrap_or(0);
    let region_id = message.base_mut().get_long().unwrap_or(0);
    let attacker_player_id = message.base_mut().get_long().unwrap_or(0);
    let defender_faction_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = organizing.on_attack_city_end(
        game,
        result,
        region_id,
        attacker_player_id,
        defender_faction_id,
        effects,
        update_player,
    );
    Some(outcome.map(|outcome| OrganizingAttackCityEndDispatch {
        result,
        region_id,
        attacker_player_id,
        defender_faction_id,
        outcome,
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingVillageWarApplicationBlock {
    FactionMaster(FactionMasterLookupBlock),
    MissingFaction { faction_id: i32 },
    MissingFactionLevel { faction_id: i32 },
    NullUnion { map_key: i32 },
    MissingRegionOwner { region_id: i32 },
    MissingRegionName { region_id: i32 },
    NoticeWouldOverflow { visible_len: usize },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingVillageWarApplicationDispatch {
    pub(crate) player_id: i32,
    pub(crate) war_number: i32,
    pub(crate) legacy_third_parameter: i32,
    pub(crate) outcome: VillageWarApplicationReport,
    pub(crate) response: Option<Result<i32, SendMessageError>>,
}

struct WorldVillageWarApplicationContext<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    organizing: &'game COrganizingCtrl,
    organizing_parameters: &'game COrganizingParam,
    attack_city: &'game CAttackCitySys,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl VillageWarApplicationContext for WorldVillageWarApplicationContext<'_, '_, '_> {
    type Block = OrganizingVillageWarApplicationBlock;

    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingVillageWarApplicationBlock::FactionMaster)
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn attack_village_min_level(&mut self) -> Result<i32, Self::Block> {
        Ok(self.organizing_parameters.attack_village_minimum_level())
    }

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })?
            .level()
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFactionLevel { faction_id })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingVillageWarApplicationBlock::MissingRegionOwner { region_id },
        )
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingVillageWarApplicationBlock::NullUnion { map_key })
            }
        }
    }

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.owned_cities().len())
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })
    }

    fn already_declared_for_city_war(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.attack_city.is_already_declared_for_war(faction_id))
    }

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })
    }

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Ok(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => Err(
                OrganizingVillageWarApplicationBlock::MissingRegionName { region_id },
            ),
        }
    }

    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block> {
        let title = self.game.get_string_by_id(title_string_id).to_vec();
        let text = self.game.get_string_by_id(text_string_id).to_vec();
        let _ = COrganizingCtrl::send_organizing_info_to_client(
            self.game,
            FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: legacy_c_string_prefix(&text),
                second_text: legacy_c_string_prefix(&title),
                information_type: -1,
                color: 0xFFFF_0000,
                trailing_value: u32::MAX,
            },
        );
        Ok(())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(legacy_c_string_prefix(argument)))
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

pub(crate) fn dispatch_village_war_application(
    message: &mut CMessage,
    game: &CGame,
    organizing: &COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    attack_city: &CAttackCitySys,
    village_war: &mut CVillageWarSys,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    sender: Option<&ServerCommandHandle>,
) -> Option<
    Result<OrganizingVillageWarApplicationDispatch, OrganizingVillageWarApplicationBlock>,
> {
    if message.message_type() != APPLY_FOR_VILLAGE_WAR_MESSAGE_TYPE {
        return None;
    }

    let source_map_id = message.map_id();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let war_number = message.base_mut().get_long().unwrap_or(0);
    let legacy_third_parameter = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldVillageWarApplicationContext {
        game,
        organizing,
        organizing_parameters,
        attack_city,
        callbacks,
    };
    let outcome = village_war.apply_for_village_war(
        player_id,
        war_number,
        legacy_third_parameter,
        &mut context,
    );
    Some(outcome.map(|outcome| {
        let response = outcome.accepted.then(|| {
            let mut response = CMessage::new(APPLY_FOR_VILLAGE_WAR_RESPONSE_TYPE);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(legacy_third_parameter);
            response.send_to_map_id(sender, source_map_id)
        });
        OrganizingVillageWarApplicationDispatch {
            player_id,
            war_number,
            legacy_third_parameter,
            outcome,
            response,
        }
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingCityWarApplicationBlock {
    FactionMaster(FactionMasterLookupBlock),
    MissingFaction { faction_id: i32 },
    MissingFactionLevel { faction_id: i32 },
    MissingRegionOwner { region_id: i32 },
    MissingRegionName { region_id: i32 },
    MissingRegionCountry { region_id: i32 },
    NullUnion { map_key: i32 },
    MissingEnemyOrganizing { organizing_id: i32 },
    EnemyMutation {
        organizing_id: i32,
        enemy_organizing_id: i32,
        source: FactionEnemyMutationBlock,
    },
    NoticeWouldOverflow {
        string_id: &'static [u8],
        visible_len: usize,
        capacity: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCityWarApplicationDispatch {
    pub(crate) player_id: i32,
    pub(crate) war_number: i32,
    pub(crate) legacy_third_parameter: i32,
    pub(crate) outcome: AttackCityApplicationReport,
    pub(crate) response: Option<Result<i32, SendMessageError>>,
}

struct CityWarEnemyMutationEffects<'game> {
    game: &'game CGame,
    enemy_id: i32,
    enemy_name: Vec<u8>,
}

impl FactionEnemyMutationContext for CityWarEnemyMutationEffects<'_> {
    fn organizing_name(&self, organizing_id: i32) -> Option<Vec<u8>> {
        (organizing_id == self.enemy_id).then(|| self.enemy_name.clone())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionEnemyWarLogArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionEnemyWarLogArgument::Text(text) => UnionFormatArgument::Text(text),
                FactionEnemyWarLogArgument::Unsigned(value) => {
                    UnionFormatArgument::Signed(*value as i32)
                }
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }
}

struct WorldAttackCityApplicationContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    globe_setup: &'game GlobeSetupSnapshot,
    organizing: &'organizing mut COrganizingCtrl,
    organizing_parameters: &'game COrganizingParam,
    village_war: &'game CVillageWarSys,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl AttackCityEnemyRelationContext
    for WorldAttackCityApplicationContext<'_, '_, '_, '_, '_>
{
    type Block = OrganizingCityWarApplicationBlock;

    fn clear_all_city_faction_relations(&mut self) -> Result<(), Self::Block> {
        self.organizing.clear_all_city_faction_relations();
        Ok(())
    }

    fn city_owner_faction_id(
        &mut self,
        city_region_id: i32,
    ) -> Result<Option<i32>, Self::Block> {
        Ok(self.game.region_owned_faction_id(city_region_id))
    }

    fn expand_faction_organizings(
        &mut self,
        faction_id: i32,
    ) -> Result<Vec<i32>, Self::Block> {
        self.organizing
            .expand_city_war_faction_organizings(faction_id)
            .map_err(|FactionUnionMembershipLookupBlock { map_key }| {
                OrganizingCityWarApplicationBlock::NullUnion { map_key }
            })
    }

    fn add_city_war_enemy_organizing(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_organizing_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarApplicationBlock::MissingEnemyOrganizing {
                organizing_id: enemy_organizing_id,
            })?;
        let mut effects = CityWarEnemyMutationEffects {
            game: self.game,
            enemy_id: enemy_organizing_id,
            enemy_name,
        };
        let found = self
            .organizing
            .add_city_war_enemy_organizing(
                organizing_id,
                enemy_organizing_id,
                &mut effects,
            )
            .map_err(|source| OrganizingCityWarApplicationBlock::EnemyMutation {
                organizing_id,
                enemy_organizing_id,
                source,
            })?;
        if !found {
            return Err(
                OrganizingCityWarApplicationBlock::MissingEnemyOrganizing { organizing_id },
            );
        }
        Ok(())
    }

    fn set_all_city_faction_enemy_changed(
        &mut self,
        changed: bool,
    ) -> Result<(), Self::Block> {
        self.organizing
            .set_all_city_faction_enemy_changed(changed);
        Ok(())
    }

    fn update_all_city_enemy_faction_relations(&mut self) -> Result<(), Self::Block> {
        let _ = self
            .organizing
            .update_all_city_enemy_faction_relations(self.game, self.update_player);
        Ok(())
    }
}

impl AttackCityApplicationContext
    for WorldAttackCityApplicationContext<'_, '_, '_, '_, '_>
{
    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingCityWarApplicationBlock::FactionMaster)
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn attack_city_min_level(&mut self) -> Result<i32, Self::Block> {
        Ok(self.organizing_parameters.attack_city_minimum_level())
    }

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })?
            .level()
            .ok_or(OrganizingCityWarApplicationBlock::MissingFactionLevel { faction_id })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingCityWarApplicationBlock::MissingRegionOwner { region_id },
        )
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingCityWarApplicationBlock::NullUnion { map_key })
            }
        }
    }

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.owned_cities().len())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn already_declared_for_village_war(
        &mut self,
        faction_id: i32,
    ) -> Result<bool, Self::Block> {
        Ok(self.village_war.is_already_declared_for_war(faction_id))
    }

    fn faction_organizing_id(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.faction_id())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Ok(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => {
                Err(OrganizingCityWarApplicationBlock::MissingRegionName { region_id })
            }
        }
    }

    fn region_country(&mut self, region_id: i32) -> Result<u8, Self::Block> {
        self.game.region_country_id(region_id).ok_or(
            OrganizingCityWarApplicationBlock::MissingRegionCountry { region_id },
        )
    }

    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block> {
        let title = self.game.get_string_by_id(title_string_id).to_vec();
        let text = self.game.get_string_by_id(text_string_id).to_vec();
        let _ = COrganizingCtrl::send_organizing_info_to_client(
            self.game,
            FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: legacy_c_string_prefix(&text),
                second_text: legacy_c_string_prefix(&title),
                information_type: -1,
                color: 0xFFFF_0000,
                trailing_value: u32::MAX,
            },
        );
        Ok(())
    }

    fn format_declaration_notice(
        &mut self,
        country_id: u8,
        faction_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let fallback = self.game.get_string_by_id(b"WS0103").to_vec();
        let (string_id, notice) = match self.globe_setup.country_name(country_id) {
            Some(country_name) => (
                b"WS0145" as &'static [u8],
                self.game.format_world_string(
                    b"WS0145",
                    &[
                        UnionFormatArgument::Text(country_name),
                        UnionFormatArgument::Text(legacy_c_string_prefix(faction_name)),
                        UnionFormatArgument::Text(country_name),
                    ],
                ),
            ),
            None => (b"WS0103" as &'static [u8], fallback),
        };
        bounded_city_war_notice(string_id, notice)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                AttackCityWarResultFormatArgument::Text(text) => {
                    UnionFormatArgument::Text(legacy_c_string_prefix(text))
                }
                AttackCityWarResultFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        bounded_city_war_notice(
            string_id,
            self.game.format_world_string(string_id, &arguments),
        )
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

fn bounded_city_war_notice(
    _string_id: &'static [u8],
    notice: Vec<u8>,
) -> Result<Vec<u8>, OrganizingCityWarApplicationBlock> {
    let notice = legacy_c_string_prefix(&notice);
    Ok(notice.to_vec())
}

#[allow(
    clippy::too_many_arguments,
    reason = "аргументы явно связывают исходные singleton-owner-ы без глобального состояния"
)]
pub(crate) fn dispatch_city_war_application(
    message: &mut CMessage,
    game: &CGame,
    globe_setup: &GlobeSetupSnapshot,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    attack_city: &mut CAttackCitySys,
    village_war: &CVillageWarSys,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingCityWarApplicationDispatch, OrganizingCityWarApplicationBlock>> {
    if message.message_type() != APPLY_FOR_CITY_WAR_MESSAGE_TYPE {
        return None;
    }

    let source_map_id = message.map_id();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let war_number = message.base_mut().get_long().unwrap_or(0);
    let legacy_third_parameter = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldAttackCityApplicationContext {
        game,
        globe_setup,
        organizing,
        organizing_parameters,
        village_war,
        callbacks,
        update_player,
    };
    let outcome = attack_city.on_player_declare_war(
        player_id,
        war_number,
        legacy_third_parameter,
        &mut context,
    );
    Some(outcome.map(|outcome| {
        let response = outcome.accepted.then(|| {
            let mut response = CMessage::new(APPLY_FOR_CITY_WAR_RESPONSE_TYPE);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(legacy_third_parameter);
            response.send_to_map_id(sender, source_map_id)
        });
        OrganizingCityWarApplicationDispatch {
            player_id,
            war_number,
            legacy_third_parameter,
            outcome,
            response,
        }
    }))
}

pub(crate) use nebokrai_realm::app::organsysmessage::OrganizingCityWarResultContextBlock;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingCityWarResultDispatch {
    pub(crate) war_number: i32,
    pub(crate) war_region_id: i32,
    pub(crate) winner_faction_id: i32,
    pub(crate) reported_union_id: i32,
    pub(crate) outcome: Result<
        AttackCityWarResultReport,
        AttackCityWarResultBlock<OrganizingCityWarResultContextBlock>,
    >,
}

struct WorldAttackCityResultContext<
    'game,
    'organizing,
    'country,
    'callbacks,
    'effects,
    'update,
    'configuration,
> {
    game: &'game mut CGame,
    organizing: &'organizing mut COrganizingCtrl,
    country_handler: &'country mut CCountryHandler,
    country_parameters: &'configuration CCountryParam,
    globe_setup: &'configuration GlobeSetupSnapshot,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl AttackCityEnemyRelationContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    type Block = OrganizingCityWarResultContextBlock;

    fn clear_all_city_faction_relations(&mut self) -> Result<(), Self::Block> {
        self.organizing.clear_all_city_faction_relations();
        Ok(())
    }

    fn city_owner_faction_id(
        &mut self,
        city_region_id: i32,
    ) -> Result<Option<i32>, Self::Block> {
        Ok(self.game.region_owned_faction_id(city_region_id))
    }

    fn expand_faction_organizings(
        &mut self,
        faction_id: i32,
    ) -> Result<Vec<i32>, Self::Block> {
        self.organizing
            .expand_city_war_faction_organizings(faction_id)
            .map_err(|FactionUnionMembershipLookupBlock { map_key }| {
                OrganizingCityWarResultContextBlock::NullUnion { map_key }
            })
    }

    fn add_city_war_enemy_organizing(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_organizing_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarResultContextBlock::MissingEnemyOrganizing {
                organizing_id: enemy_organizing_id,
            })?;
        let mut effects = CityWarEnemyMutationEffects {
            game: self.game,
            enemy_id: enemy_organizing_id,
            enemy_name,
        };
        let found = self
            .organizing
            .add_city_war_enemy_organizing(
                organizing_id,
                enemy_organizing_id,
                &mut effects,
            )
            .map_err(|source| OrganizingCityWarResultContextBlock::EnemyMutation {
                organizing_id,
                enemy_organizing_id,
                source,
            })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingEnemyOrganizing { organizing_id },
            );
        }
        Ok(())
    }

    fn set_all_city_faction_enemy_changed(
        &mut self,
        changed: bool,
    ) -> Result<(), Self::Block> {
        self.organizing
            .set_all_city_faction_enemy_changed(changed);
        Ok(())
    }

    fn update_all_city_enemy_faction_relations(&mut self) -> Result<(), Self::Block> {
        let _ = self
            .organizing
            .update_all_city_enemy_faction_relations(self.game, self.update_player);
        Ok(())
    }
}

impl AttackCityWarEndContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn clear_region_country_warring_if_present(
        &mut self,
        city_region_id: i32,
    ) -> Result<(), Self::Block> {
        let Some(country_id) = self.game.region_country_id(city_region_id) else {
            return Ok(());
        };
        if let Some(country) = self.country_handler.get_country_mut(country_id) {
            country.is_warring = false;
        }
        Ok(())
    }
}

impl CountryExileResultContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>> {
        self.game
            .map_player(player_id as u32)
            .map(|player| legacy_c_string_prefix(player.get_name()).to_vec())
    }

    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| CountryExileTarget {
                name: legacy_c_string_prefix(player.get_name()).to_vec(),
                country: player.country(),
                level: player.get_level(),
                credit: player.credit(),
                pk_count: player.pk_count(),
                is_god: player.is_god(),
            })
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset> {
        self.game
            .reset_online_player_murder_counters(player_id as u32)
            .map(|reset| CountryAbsolveCounterReset {
                previous_kill_count: reset.previous_kill_count,
                previous_pk_count: reset.previous_pk_count,
            })
    }

    fn faction_id_by_player(
        &mut self,
        player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Ok(0),
            FreePlayerLookup::Faction(faction_id) => Ok(faction_id),
            FreePlayerLookup::BlockedNullFaction { .. } => {
                Err(CountryGovernanceContextBlock::PlayerFactionLookup)
            }
        }
    }

    fn faction_snapshot(&mut self, faction_id: i32) -> Option<CountryFactionSnapshot> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| CountryFactionSnapshot {
                faction_id: faction.faction_id(),
                name: faction.name().to_vec(),
                owned_cities: faction.owned_cities().iter().copied().collect(),
            })
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn country_identity_name(&mut self, identity: u8) -> Vec<u8> {
        self.globe_setup
            .country_identity_name(identity)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(text) => UnionFormatArgument::Text(text),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.game.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery> {
        let sender = self.game.current_game_server_sender();
        self.game
            .connected_game_server_indices()
            .map(|map_id| CountryExileMessageDelivery {
                map_id,
                delivery: message.send_to_map_id(sender.as_ref(), map_id),
            })
            .collect()
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl AttackCityWarResultContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<AttackCityWarResultRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
            WorldRegionNameLookup::Name(name) => Some(AttackCityWarResultRegion {
                name: legacy_c_string_prefix(name).to_vec(),
            }),
        })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingCityWarResultContextBlock::MissingRegionOwner { region_id },
        )
    }

    fn faction(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<AttackCityWarResultFaction>, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).map(|faction| {
            AttackCityWarResultFaction {
                name: legacy_c_string_prefix(faction.name()).to_vec(),
            }
        }))
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingCityWarResultContextBlock::NullUnion { map_key })
            }
        }
    }

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block> {
        let country_id = self
            .organizing
            .faction_by_id(faction_id)
            .and_then(CFaction::country);
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
        Ok(())
    }

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_add_owned_city(
            &mut bridge,
            faction_id,
            region_id,
            self.update_player,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::OwnedCity {
            faction_id,
            operation: "AddOwnedCity",
            source,
        })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_clear_owned_cities(
            &mut bridge,
            faction_id,
            self.update_player,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::OwnedCity {
            faction_id,
            operation: "ClearOwnedCity",
            source,
        })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "ClearOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn faction_master_id(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "GetMasterID",
                },
            )?
            .master_id()
            .ok_or(OrganizingCityWarResultContextBlock::MissingFactionMaster { faction_id })
    }

    fn add_defence_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_defence_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::VictorCount {
                faction_id,
                operation: "AddDefenceVictorCounts",
                source,
            })?;
        if found.is_none() {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddDefenceVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn add_offense_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_offense_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::VictorCount {
                faction_id,
                operation: "AddOffenseVictorCounts",
                source,
            })?;
        if found.is_none() {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOffenseVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn stat_billboard(&mut self) -> Result<(), Self::Block> {
        self.organizing
            .stat_billboard()
            .map_err(OrganizingCityWarResultContextBlock::Billboard)
    }

    fn faction_country(&mut self, faction_id: i32) -> Result<u8, Self::Block> {
        let country = self
            .organizing
            .country_by_faction(faction_id)
            .map_err(|_| OrganizingCityWarResultContextBlock::MissingFactionCountry {
                faction_id,
            })?;
        country.ok_or(
            OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                faction_id,
                operation: "GetCountry",
            },
        )
    }

    fn country_exists(&mut self, country_id: u8) -> Result<bool, Self::Block> {
        Ok(self.country_handler.get_country(country_id).is_some())
    }

    fn set_country_king_and_city(
        &mut self,
        country_id: u8,
        master_id: i32,
        city_region_id: i32,
    ) -> Result<(), Self::Block> {
        let Some(mut country) = self.country_handler.take_country_owner(country_id) else {
            return Err(OrganizingCityWarResultContextBlock::MissingCountryOwner {
                country_id,
            });
        };
        let set_king = country.set_king(master_id, self.country_parameters, self);
        if set_king.is_ok() {
            country.city_id = city_region_id;
        }
        self.country_handler
            .restore_country_owner(country_id, country);
        set_king
            .map(|_| ())
            .map_err(|source| OrganizingCityWarResultContextBlock::CountryGovernance {
                country_id,
                source,
            })
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                AttackCityWarResultFormatArgument::Text(text) => {
                    UnionFormatArgument::Text(legacy_c_string_prefix(text))
                }
                AttackCityWarResultFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        parameter: i32,
        text: &[u8],
    ) -> Result<(), Self::Block> {
        let _ = self.organizing.send_top_info_to_client(
            self.game,
            top_info_id,
            timer_flag,
            parameter,
            text,
        );
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "явные параметры сохраняют границы исходных singleton-owner-ов"
)]
pub(crate) fn dispatch_city_war_result<Callback: Copy>(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    attack_city: &mut CAttackCitySys,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    globe_setup: &GlobeSetupSnapshot,
) -> Option<OrganizingCityWarResultDispatch> {
    if message.message_type() != CITY_WAR_RESULT_MESSAGE_TYPE {
        return None;
    }

    let war_number = message.base_mut().get_long().unwrap_or(0);
    let war_region_id = message.base_mut().get_long().unwrap_or(0);
    let winner_faction_id = message.base_mut().get_long().unwrap_or(0);
    let reported_union_id = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_handler,
        country_parameters,
        globe_setup,
        callbacks: effects,
        update_player,
    };
    let outcome = attack_city.on_faction_win_city(
        war_number,
        war_region_id,
        winner_faction_id,
        reported_union_id,
        timer,
        attack_callbacks,
        &mut context,
    );
    Some(OrganizingCityWarResultDispatch {
        war_number,
        war_region_id,
        winner_faction_id,
        reported_union_id,
        outcome,
    })
}

/// Соединяет concrete `CAttackCitySys::Reload` с World runtime owners.
///
/// `Reload` сначала снимает прежние timer events и завершает активные войны;
/// поэтому вызывающий не имеет права превращать ошибку последующего `Initialize`
/// в транзакционный rollback. Контекст буквально совпадает с result-ingress
/// `0x60138`, а `send_all` сохраняет `0x7FE22` между end и новым Initialize.
#[allow(
    clippy::too_many_arguments,
    reason = "все singleton-владельцы и callback-грань подтверждены CAttackCitySys::Reload"
)]
pub(crate) fn reload_attack_city<Callback: Copy>(
    game: &mut CGame,
    attack_city: &mut CAttackCitySys,
    source: Option<&[u8]>,
    now: TagTime,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    today_tax_event_id: Option<TimerId>,
    organizing: &mut COrganizingCtrl,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Result<AttackCityReloadReport, AttackCityReloadBlock<OrganizingCityWarResultContextBlock>> {
    let sender = game.current_game_server_sender();
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_handler,
        country_parameters,
        globe_setup,
        callbacks: effects,
        update_player,
    };
    attack_city.reload(
        source,
        now,
        timer,
        attack_callbacks,
        today_tax_event_id,
        &mut context,
        |message| message.send_all(sender.as_ref()).unwrap_or(0),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingGoodsWarContextBlock {
    pub(crate) faction_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingGoodsWarCommandOutcome {
    DeleteOneMember {
        player_id: i32,
        report: GoodsWarMutationReport,
    },
    RefreshAll(GoodsWarRefreshReport),
    InsertOneFaction {
        faction_id: i32,
        report: GoodsWarMutationReport,
    },
    AppendOneFactionToCount {
        faction_id: i32,
        report: GoodsWarMutationReport,
    },
    Ignored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingGoodsWarCommandDispatch {
    pub(crate) operation: i32,
    pub(crate) outcome: OrganizingGoodsWarCommandOutcome,
}

struct WorldGoodsWarMemberContext<'game, 'organizing> {
    game: &'game CGame,
    organizing: &'organizing mut COrganizingCtrl,
}

impl GoodsWarDeliveryContext for WorldGoodsWarMemberContext<'_, '_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl GoodsWarMemberContext for WorldGoodsWarMemberContext<'_, '_> {
    type Block = OrganizingGoodsWarContextBlock;

    fn faction_snapshot(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<GoodsWarFactionSnapshot>, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).map(|faction| {
            GoodsWarFactionSnapshot {
                name: legacy_c_string_prefix(faction.name()).to_vec(),
                goods_war_count: faction.goods_war_count(),
            }
        }))
    }

    fn set_faction_goods_war_count(
        &mut self,
        faction_id: i32,
        count: i32,
    ) -> Result<i32, Self::Block> {
        self.organizing
            .set_faction_goods_war_count(faction_id, count)
            .ok_or(OrganizingGoodsWarContextBlock { faction_id })
    }

    fn faction_win_audit_environment(&mut self) -> Option<GoodsWarAuditEnvironment> {
        let world_number = self.game.configured_world_number()?;
        Some(GoodsWarAuditEnvironment {
            login_server_id: self.game.login_server_id(),
            world_number,
        })
    }

    fn faction_win_audit_player(&mut self, player_id: i32) -> Option<GoodsWarAuditPlayer> {
        self.game
            .map_player(player_id as u32)
            .map(|player| GoodsWarAuditPlayer {
                account: player.get_account().to_vec(),
                player_id: player.get_id(),
                name: player.get_name().to_vec(),
                level: player.get_level(),
            })
    }
}

pub(crate) fn dispatch_goods_war_command(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    goods_war: &mut CGoodsWarMember,
) -> Option<
    Result<
        OrganizingGoodsWarCommandDispatch,
        GoodsWarMemberBlock<OrganizingGoodsWarContextBlock>,
    >,
> {
    if message.message_type() != GOODS_WAR_COMMAND_MESSAGE_TYPE {
        return None;
    }

    let operation = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldGoodsWarMemberContext { game, organizing };
    let outcome = match operation {
        2 => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            OrganizingGoodsWarCommandOutcome::DeleteOneMember {
                player_id,
                report: goods_war.delete_one_member(player_id, &mut context),
            }
        }
        4 => OrganizingGoodsWarCommandOutcome::RefreshAll(
            goods_war.refresh_all(&mut context),
        ),
        0x11 => {
            let faction_id = message.base_mut().get_long().unwrap_or(0);
            let report = match goods_war.insert_one_faction(faction_id, &mut context) {
                Ok(report) => report,
                Err(block) => return Some(Err(block)),
            };
            OrganizingGoodsWarCommandOutcome::InsertOneFaction { faction_id, report }
        }
        0x12 => {
            let faction_id = message.base_mut().get_long().unwrap_or(0);
            let report = match goods_war.append_one_faction_to_count(faction_id, &mut context) {
                Ok(report) => report,
                Err(block) => return Some(Err(block)),
            };
            OrganizingGoodsWarCommandOutcome::AppendOneFactionToCount { faction_id, report }
        }
        _ => OrganizingGoodsWarCommandOutcome::Ignored,
    };
    Some(Ok(OrganizingGoodsWarCommandDispatch { operation, outcome }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingGoodsWarFactionWinBlock {
    MissingMasterId {
        requested_faction_id: i32,
        actual_faction_id: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingGoodsWarFactionWinDispatch {
    pub(crate) requested_faction_id: i32,
    pub(crate) faction_found: bool,
    pub(crate) report: Option<GoodsWarFactionWinReport>,
}

pub(crate) fn dispatch_goods_war_faction_win(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    goods_war: &mut CGoodsWarMember,
) -> Option<
    Result<OrganizingGoodsWarFactionWinDispatch, OrganizingGoodsWarFactionWinBlock>,
> {
    if message.message_type() != GOODS_WAR_FACTION_WIN_MESSAGE_TYPE {
        return None;
    }

    let requested_faction_id = message.base_mut().get_long().unwrap_or(0);
    let Some(faction) = organizing.faction_by_id(requested_faction_id) else {
        return Some(Ok(OrganizingGoodsWarFactionWinDispatch {
            requested_faction_id,
            faction_found: false,
            report: None,
        }));
    };
    let actual_faction_id = faction.faction_id();
    let Some(master_id) = faction.master_id() else {
        return Some(Err(OrganizingGoodsWarFactionWinBlock::MissingMasterId {
            requested_faction_id,
            actual_faction_id,
        }));
    };
    let winner = GoodsWarFactionWinSnapshot {
        faction_id: actual_faction_id,
        name: faction.name().to_vec(),
        master_id,
        member_ids: faction.get_members().keys().copied().collect(),
    };
    let mut context = WorldGoodsWarMemberContext { game, organizing };
    let report = goods_war.faction_win(&winner, &mut context);
    Some(Ok(OrganizingGoodsWarFactionWinDispatch {
        requested_faction_id,
        faction_found: true,
        report: Some(report),
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingPlayerQuestCommandKind {
    Add,
    Remove,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPlayerQuestCommandDispatch {
    pub(crate) kind: OrganizingPlayerQuestCommandKind,
    pub(crate) player_id: i32,
    pub(crate) quest_id: i16,
    pub(crate) game_server_id: i32,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

pub(crate) fn dispatch_player_quest_command(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerQuestCommandDispatch> {
    let (kind, response_type) = match message.message_type() {
        PLAYER_ADD_QUEST_MESSAGE_TYPE => {
            (OrganizingPlayerQuestCommandKind::Add, GAME_ADD_QUEST_MESSAGE_TYPE)
        }
        PLAYER_REMOVE_QUEST_MESSAGE_TYPE => (
            OrganizingPlayerQuestCommandKind::Remove,
            GAME_REMOVE_QUEST_MESSAGE_TYPE,
        ),
        _ => return None,
    };

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let quest_id = message.base_mut().get_short().unwrap_or(0);
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let delivery = (game_server_id != 0).then(|| {
        let mut response = CMessage::new(response_type);
        response.base_mut().add_long(player_id);
        response.base_mut().add_short(quest_id);
        response.send_to_map_id(sender, game_server_id)
    });
    Some(OrganizingPlayerQuestCommandDispatch {
        kind,
        player_id,
        quest_id,
        game_server_id,
        delivery,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPlayerRunScriptDispatch {
    pub(crate) player_id: i32,
    pub(crate) script: Vec<u8>,
    pub(crate) game_server_id: i32,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

pub(crate) fn dispatch_player_run_script(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerRunScriptDispatch> {
    if message.message_type() != PLAYER_RUN_SCRIPT_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let script = message
        .base_mut()
        .get_str_bytes(PLAYER_SCRIPT_CAPACITY)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let delivery = (game_server_id != 0).then(|| {
        let script = CString::new(script.as_slice())
            .expect("bounded GetStr возвращает bytes до первого NUL");
        let mut response = CMessage::new(GAME_RUN_SCRIPT_MESSAGE_TYPE);
        response.base_mut().add_long(player_id);
        response.base_mut().add_str(Some(&script));
        response.send_to_map_id(sender, game_server_id)
    });
    Some(OrganizingPlayerRunScriptDispatch {
        player_id,
        script,
        game_server_id,
        delivery,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionParameterOutcome {
    FactionMissing,
    Applied {
        faction_id: i32,
        outcome: FactionSetParameterOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionParameterBlock {
    FactionMaster(FactionMasterLookupBlock),
    SetParameter {
        faction_id: i32,
        source: FactionSetParameterBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionParameterDispatch {
    pub(crate) player_id: i32,
    pub(crate) parameter: Vec<u8>,
    pub(crate) value: i32,
    pub(crate) outcome: OrganizingFactionParameterOutcome,
}

pub(crate) fn dispatch_faction_parameter(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>> {
    if message.message_type() != SET_FACTION_PARAMETER_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let parameter = message
        .base_mut()
        .get_str_bytes(FACTION_PARAMETER_NAME_CAPACITY)
        .expect("literal 0x32 исключает zero-capacity GetStr");
    let value = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.faction_id_by_master_player(player_id) {
        Ok(faction_id) => faction_id,
        Err(source) => {
            return Some(Err(OrganizingFactionParameterBlock::FactionMaster(source)));
        }
    };
    if faction_id < 1 {
        return Some(Ok(OrganizingFactionParameterDispatch {
            player_id,
            parameter,
            value,
            outcome: OrganizingFactionParameterOutcome::FactionMissing,
        }));
    }

    let mut effects = WorldFactionSetParameterEffects {
        game,
        callbacks,
        update_player,
    };
    let outcome = match organizing.set_faction_parameter(
        game,
        parameters,
        faction_id,
        &parameter,
        value,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionParameterOutcome::Applied {
            faction_id,
            outcome,
        },
        Ok(None) => OrganizingFactionParameterOutcome::FactionMissing,
        Err(source) => {
            return Some(Err(OrganizingFactionParameterBlock::SetParameter {
                faction_id,
                source,
            }));
        }
    };
    Some(Ok(OrganizingFactionParameterDispatch {
        player_id,
        parameter,
        value,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingChangeRegionRouterDispatch {
    pub(crate) request_id: i32,
    pub(crate) from_region: i32,
    pub(crate) to_region: i32,
    pub(crate) target: RegionRoutePoint,
    pub(crate) outcome: RegionRouterChangeOutcome,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

pub(crate) fn dispatch_change_region_router(
    message: &mut CMessage,
    router: &RegionRouter,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingChangeRegionRouterDispatch> {
    if message.message_type() != CHANGE_REGION_ROUTER_MESSAGE_TYPE {
        return None;
    }

    let request_id = message.base_mut().get_long().unwrap_or(0);
    let from_region = message.base_mut().get_long().unwrap_or(0);
    let to_region = message.base_mut().get_long().unwrap_or(0);
    let target = RegionRoutePoint {
        x: message.base_mut().get_long().unwrap_or(0),
        y: message.base_mut().get_long().unwrap_or(0),
    };
    let outcome = router.change_region_router(from_region, to_region, target);

    let route = match &outcome {
        RegionRouterChangeOutcome::RegionNotFound { .. } => &[][..],
        RegionRouterChangeOutcome::Complete(route) => route.as_slice(),
    };
    let mut response = CMessage::new(CHANGE_REGION_ROUTER_RESPONSE_TYPE);
    response.base_mut().add_long(request_id);
 // `vector::size()` попадал в 32-битный `Add` низшими битами без range gate.
    response.base_mut().add_long(route.len() as i32);
    for step in route {
        response.base_mut().add_long(step.region_id);
        response.base_mut().add_long(step.point.x);
        response.base_mut().add_long(step.point.y);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_all(sender);
    Some(OrganizingChangeRegionRouterDispatch {
        request_id,
        from_region,
        to_region,
        target,
        outcome,
        wire,
        delivery,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingVillageWarResultContextBlock {
    MissingRegionOwner { region_id: i32 },
    NullUnion { map_key: i32 },
    MissingFactionForMutation {
        faction_id: i32,
        operation: &'static str,
    },
    AddOwnedCity {
        faction_id: i32,
        source: OwnedCityMutationBuildError,
    },
    ClearOwnedCity {
        faction_id: i32,
        source: OwnedCityMutationBuildError,
    },
    AddVictorCount {
        faction_id: i32,
        source: FactionInitialPropertyBlock,
    },
    NoticeWouldOverflow { visible_len: usize },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingVillageWarResultDispatch {
    pub(crate) war_number: i32,
    pub(crate) war_region_id: i32,
    pub(crate) winner_faction_id: i32,
    pub(crate) legacy_fourth_parameter: i32,
    pub(crate) outcome: Result<
        VillageWarResultReport,
        VillageWarResultBlock<OrganizingVillageWarResultContextBlock>,
    >,
}

struct WorldVillageWarResultContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    organizing: &'organizing mut COrganizingCtrl,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl VillageWarResultContext for WorldVillageWarResultContext<'_, '_, '_, '_, '_> {
    type Block = OrganizingVillageWarResultContextBlock;

    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<VillageWarResultRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
            WorldRegionNameLookup::Name(name) => Some(VillageWarResultRegion {
                name: legacy_c_string_prefix(name).to_vec(),
            }),
        })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingVillageWarResultContextBlock::MissingRegionOwner { region_id },
        )
    }

    fn faction(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<VillageWarResultFaction>, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).map(|faction| {
            VillageWarResultFaction {
                organizing_id: faction.faction_id(),
                name: legacy_c_string_prefix(faction.name()).to_vec(),
            }
        }))
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingVillageWarResultContextBlock::NullUnion { map_key })
            }
        }
    }

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block> {
        let country_id = self
            .organizing
            .faction_by_id(faction_id)
            .and_then(CFaction::country);
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
        Ok(())
    }

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_add_owned_city(
            &mut bridge,
            faction_id,
            region_id,
            self.update_player,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::AddOwnedCity {
            faction_id,
            source,
        })?;
        if !found {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_clear_owned_cities(
            &mut bridge,
            faction_id,
            self.update_player,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::ClearOwnedCity {
            faction_id,
            source,
        })?;
        if !found {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "ClearOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn add_village_war_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_village_war_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::AddVictorCount {
            faction_id,
            source,
        })?;
        if found.is_none() {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddVillageWarVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(legacy_c_string_prefix(argument)))
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        parameter: i32,
        text: &[u8],
    ) -> Result<(), Self::Block> {
        let _ = self.organizing.send_top_info_to_client(
            self.game,
            top_info_id,
            timer_flag,
            parameter,
            text,
        );
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

pub(crate) fn dispatch_village_war_result<Callback: Copy>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    village_war: &mut CVillageWarSys,
    timer: &mut CTimer<Callback>,
    callbacks: VillageWarCallbacks<Callback>,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<OrganizingVillageWarResultDispatch> {
    if message.message_type() != VILLAGE_WAR_RESULT_MESSAGE_TYPE {
        return None;
    }

    let war_number = message.base_mut().get_long().unwrap_or(0);
    let war_region_id = message.base_mut().get_long().unwrap_or(0);
    let winner_faction_id = message.base_mut().get_long().unwrap_or(0);
    let legacy_fourth_parameter = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldVillageWarResultContext {
        game,
        organizing,
        callbacks: effects,
        update_player,
    };
    let outcome = village_war.on_faction_win_village(
        war_number,
        war_region_id,
        winner_faction_id,
        legacy_fourth_parameter,
        timer,
        callbacks,
        &mut context,
    );
    Some(OrganizingVillageWarResultDispatch {
        war_number,
        war_region_id,
        winner_faction_id,
        legacy_fourth_parameter,
        outcome,
    })
}

fn send_declare_war_faction_list_notice<Context>(
    context: &mut Context,
    player_id: i32,
    notice: OrganizingDeclareWarFactionListNotice,
) where
    Context: FactionOrganizingInfoContext,
{
    let first_string_id = match notice {
        OrganizingDeclareWarFactionListNotice::MissingFaction => b"WS0122".as_slice(),
        OrganizingDeclareWarFactionListNotice::MasterRequired => b"WS0123".as_slice(),
        OrganizingDeclareWarFactionListNotice::NoFactions => b"WS0124".as_slice(),
    };
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    let second_text = context.world_string(b"WS0121").unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_faction_list_empty_notice<Context>(context: &mut Context, player_id: i32)
where
    Context: FactionOrganizingInfoContext,
{
    let first_text = context.world_string(b"WS0120").unwrap_or_default();
    let second_text = context.world_string(b"WS0119").unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_faction_application_cancel_notice<Context>(context: &mut Context, player_id: i32)
where
    Context: FactionOrganizingInfoContext,
{
    let first_text = context.world_string(b"WS0125").unwrap_or_default();
    let second_text = context.world_string(b"WS0119").unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_faction_list_response(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    player_id: i32,
    total_factions: i32,
    request_id: i64,
    cookie: i32,
    page: Option<(i32, i32, &FactionListPage)>,
) -> OrganizingFactionListResponse {
    let mut response = CMessage::new(FACTION_LIST_RESPONSE_TYPE);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(total_factions);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    if let Some((requested_page, applied_faction_id, page_data)) = page {
        response.base_mut().add_long(requested_page);
        response.base_mut().add_long(applied_faction_id);
        response.base_mut().add(&page_data.payload);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);
    OrganizingFactionListResponse {
        socket_id,
        total_factions,
        included_page: page.map(|(requested_page, _, _)| requested_page),
        applied_faction_id: page.map(|(_, applied_faction_id, _)| applied_faction_id),
        wire,
        delivery,
    }
}

fn send_declare_war_faction_list_response(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    player_id: i32,
    total_factions: i32,
    request_id: i64,
    cookie: i32,
    page: Option<&DeclareWarFactionPage>,
) -> OrganizingDeclareWarFactionListResponse {
    let mut response = CMessage::new(DECLARE_WAR_FACTION_LIST_RESPONSE_TYPE);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(total_factions);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    if let Some(page) = page {
        response.base_mut().add_long(page.requested_page);
        response.base_mut().add(&page.payload);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);
    OrganizingDeclareWarFactionListResponse {
        socket_id,
        total_factions,
        included_page: page.map(|page| page.requested_page),
        wire,
        delivery,
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
