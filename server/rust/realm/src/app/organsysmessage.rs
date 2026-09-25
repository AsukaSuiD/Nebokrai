//! Session терминальный слой async confirm-подтверждений organizing-сообщений:
//! типы-конечники, их in-memory очередь и четыре endpoint-узла слиты в одного
//! владельца [`WorldOrganizingSessionRuntimeOwner`]; старый
//! `appworld/message/organsysmessage.rs` держит type-алиас прежнего имени
//! владельца для main-loop runtime и адаптеров. Диспетчеры `organsysmessage`
//! остаются в старом файле до шага переноса области; их callback-структуры
//! уже обслуживаются этим владельцем.
//!
//! Сюда же перенесены чистые data-контракты диспетчера: opcode-константы и
//! dispatch/outcome/block/response-семейства, чьи поля цитируют только
//! Realm/Shared-типы, а также ветвь `0x6012D` region param update, которую
//! открывает только [`crate::app::world_game_view::WorldGameView`]. Типы, чьи
//! поля цитируют узлы старого пакета (`CGame`-эффекты адаптеров,
//! `FactionCreationBlock` из старого `organizingctrl.rs`), остаются у старого
//! владельца до переноса швов; старый файл реэкспортирует перенесённое для
//! переходных потребителей.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{RegionRoutePoint, RegionRouterChangeOutcome};
use nebokrai_shared::runtime::NetSessionCallbackOutcome;
use parking_lot::Mutex;

use crate::activities::attackcitysys::{
    AttackCityApplicationReport, AttackCityWarResultBlock, AttackCityWarResultReport,
};
use crate::activities::factionwarsys::{
    FactionWarDeclarationBlock, FactionWarDeclarationOutcome, FactionWarPlayerDiedBlock,
    FactionWarPlayerDiedOutcome,
};
use crate::activities::villagewarsys::{
    VillageWarApplicationReport, VillageWarResultBlock, VillageWarResultReport,
};
use crate::app::world_game_view::{WorldGameView, WorldRegionParamUpdateOutcome};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::player::{PlayerCodecError, PlayerFactionInfoUpdateReport};
use crate::content::organizing::TagTimeValue;
use crate::organizations::country::CountryGovernanceContextBlock;
use crate::organizations::faction::{
    FactionApplyForJoinOutcome, FactionBillboardStatBlock, FactionDemiseBlock,
    FactionDemiseOutcome, FactionDubBlock, FactionDubOutcome, FactionExitBlock,
    FactionExitOutcome, FactionExperienceUpdate, FactionFireOutBlock, FactionFireOutOutcome,
    FactionInitialPropertyBlock, FactionOperationBlock, FactionOperationRejection,
    FactionPermitBlock, FactionPermitUpdate, FactionPurviewChange, FactionPurviewChangeBlock,
    FactionPurviewChangeOutcome, FactionSetParameterBlock, FactionSetParameterOutcome,
    FactionUpgradeBlock, FactionUpgradeOutcome, FactionUploadIconOutcome,
    OwnedCityMutationBuildError,
};
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;
use crate::organizations::goodswarmember::{
    GoodsWarFactionWinReport, GoodsWarMutationReport, GoodsWarRefreshReport,
};
use crate::organizations::organizingctrl::{
    AllFactionInfoClientBlock, AttackCityEndReport, CityTransferSessionRuntime,
    CityTransferStartOutcome, ConfederationCreationSessionRuntime,
    DeclareWarFactionPage, DeclareWarFactionPageBlock, FactionClientSnapshotBlock,
    FactionCountryCountBlock, FactionCreationOutcome, FactionListPage, FactionListPageBlock,
    FactionMasterLookupBlock, FactionUnionMembershipLookupBlock,
    OrganizingConfederationDisbandBlock, OrganizingConfederationDisbandOutcome,
    OrganizingContributorOutcome, OrganizingDisbandBlock, OrganizingDisbandOutcome,
    OrganizingDisbandProgress, OrganizingDisbandRejection, OrganizingFactionApplicationBlock,
    OrganizingFactionDoJoinOutcome, OrganizingFactionMemberStateOutcome,
    OrganizingFactionWarDeclarationBlock, OrganizingFactionWarPlayerDiedBlock,
    OrganizingInfoDelivery, OrganizingLeaveWordEditOutcome, OrganizingLeaveWordEnableOutcome,
    OrganizingLeaveWordOutcome, OrganizingNameCountryBlock, OrganizingNameLookupBlock,
    OrganizingNameMatch, OrganizingNamedUnionApplicationBlock, OrganizingPronounceOutcome,
    OrganizingUnionApplyForJoinOutcome, OrganizingUnionByMasterBlock,
    OrganizingUnionDemiseOutcome, OrganizingUnionExitOutcome, OrganizingUnionFireOutOutcome,
    PlayerInviteFactionOutcome, RemovePersonFromApplyFactionListOutcome,
    UnionClientSnapshotByPlayerBlock, UnionClientSnapshotByPlayerOutcome,
};
use crate::organizations::union::{
    CityTransferEndpointBlock, CityTransferTerminal, ConfederationCreationEndpointBlock,
    ConfederationCreationTerminal, UnionApplicationEndpointBlock,
    UnionApplicationSessionRuntime, UnionApplicationTerminal, UnionApplyForJoinOutcome,
    UnionInvitationSessionRuntime,
};

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

/// Терминалы, публикуемые async session callback-ом в FIFO владельца игры.
///
/// Исходный порядок — это сначала main-loop FIFO, а callback лишь кладёт
/// подтверждение; терминал публикует точную выявленную форму outcome из
/// `.exe/worldserver.exe`, и его изменение игры происходит только при
/// чтении владельца игры, как и раньше.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedUnionApplicationTerminal {
    pub union_id: i32,
    pub applicant_faction_id: i32,
    pub terminal: UnionApplicationTerminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedUnionInvitationTerminal {
    pub union_id: i32,
    pub inviter_faction_id: i32,
    pub invited_faction_id: i32,
    pub terminal: UnionApplicationTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedCityTransferTerminal {
    pub source_faction_id: i32,
    pub target_faction_id: i32,
    pub region_id: i32,
    pub region_name: Vec<u8>,
    pub terminal: CityTransferTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedConfederationCreationTerminal {
    pub first_player_id: i32,
    pub second_player_id: i32,
    pub first_faction_id: i32,
    pub second_faction_id: i32,
    pub union_name: Vec<u8>,
    pub terminal: ConfederationCreationTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueuedOrganizingSessionTerminal {
    Union(QueuedUnionApplicationTerminal),
    UnionInvitation(QueuedUnionInvitationTerminal),
    ConfederationCreation(QueuedConfederationCreationTerminal),
    CityTransfer(QueuedCityTransferTerminal),
}

/// Выгружает конкретное подтверждение Session confirmation с `SendMessageError`-
/// результатом; нулевое пuegosето SendMessageError не имеет смысла, ей выдаётся
/// исходный сообщение-маркер без инференцев.
#[derive(Debug, Eq, PartialEq)]
pub struct UnionApplicationConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CityTransferConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfederationCreationConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

/// Блокировка session endpoint-а, собранная в единый терминальный владелец.
///
/// Внутренний владелец очередей использует `parking_lot::Mutex`, а callback не
/// обновляет игру вне основного loop-a — зафиксировано в заявлении выше и в
/// подтверждающем заголовке старого диспетчера.
#[derive(Debug, Default)]
struct WorldUnionApplicationRuntimeState {
    terminals: Mutex<VecDeque<QueuedOrganizingSessionTerminal>>,
    confirmations: Mutex<VecDeque<UnionApplicationConfirmationDelivery>>,
    blocks: Mutex<VecDeque<crate::organizations::union::UnionApplicationEndpointBlock>>,
    city_confirmations: Mutex<VecDeque<CityTransferConfirmationDelivery>>,
    city_blocks: Mutex<VecDeque<crate::organizations::union::CityTransferEndpointBlock>>,
    confederation_creation_confirmations:
        Mutex<VecDeque<ConfederationCreationConfirmationDelivery>>,
    confederation_creation_blocks:
        Mutex<VecDeque<crate::organizations::union::ConfederationCreationEndpointBlock>>,
}

/// Идемпотентный владелец скопления endpoint-терминалов для одного World-а.
///
/// Это слитый владелец: открытые `Arc`-фабрики четырёх endpoint-узлов и есть
/// исходные точки подключения подтверждений; объектное поле `state` по
/// объявлению берёт собственную форму shared state из orginal-and-rec host
/// `worldserver.exe + worldserver.pdb`. Расхождения очередей с прежним
/// `WorldUnionApplicationRuntimeOwner` нет — старый файл ссылается на этот же
/// `Arc`-state через type-алиас.
#[derive(Clone, Default)]
pub struct WorldOrganizingSessionRuntimeOwner {
    state: Arc<WorldUnionApplicationRuntimeState>,
}

impl WorldOrganizingSessionRuntimeOwner {
    /// Endpoint union-application сессии: confirmation уходит указанному
    /// GameServer, терминал встаёт в общий FIFO.
    pub fn endpoint(
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

    pub fn invitation_endpoint(
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

    pub fn city_endpoint(
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

    pub fn confederation_creation_endpoint(
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

    pub fn pop_terminal(&self) -> Option<QueuedOrganizingSessionTerminal> {
        self.state.terminals.lock().pop_front()
    }

    pub fn take_confirmations(&self) -> Vec<UnionApplicationConfirmationDelivery> {
        self.state.confirmations.lock().drain(..).collect()
    }

    pub fn take_city_confirmations(&self) -> Vec<CityTransferConfirmationDelivery> {
        self.state.city_confirmations.lock().drain(..).collect()
    }

    pub fn take_confederation_creation_confirmations(
        &self,
    ) -> Vec<ConfederationCreationConfirmationDelivery> {
        self.state
            .confederation_creation_confirmations
            .lock()
            .drain(..)
            .collect()
    }

    pub fn take_blocks(&self) -> Vec<crate::organizations::union::UnionApplicationEndpointBlock> {
        self.state.blocks.lock().drain(..).collect()
    }

    pub fn take_city_blocks(&self) -> Vec<crate::organizations::union::CityTransferEndpointBlock> {
        self.state.city_blocks.lock().drain(..).collect()
    }

    pub fn take_confederation_creation_blocks(
        &self,
    ) -> Vec<crate::organizations::union::ConfederationCreationEndpointBlock> {
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

pub const SESSION_RESULT_MESSAGE_TYPES: [i32; 6] =
    [0x60117, 0x60119, 0x60120, 0x60122, 0x60124, 0x60131];
pub const FACTION_WAR_PLAYER_DIED_MESSAGE_TYPE: i32 = 0x60101;
pub const CREATE_FACTION_MESSAGE_TYPE: i32 = 0x60103;
pub const CREATE_FACTION_RESPONSE_TYPE: i32 = 0x7FE01;
pub const INITIAL_ORGANIZING_DATA_MESSAGE_TYPE: i32 = 0x60104;
pub const FACTION_LIST_MESSAGE_TYPE: i32 = 0x60107;
pub const FACTION_LIST_RESPONSE_TYPE: i32 = 0x7FE07;
pub const FACTION_APPLICATION_MESSAGE_TYPE: i32 = 0x60108;
pub const CANCEL_FACTION_APPLICATION_MESSAGE_TYPE: i32 = 0x60109;
pub const FACTION_APPLICATION_DECISION_MESSAGE_TYPE: i32 = 0x6010A;
pub const FACTION_FIRE_OUT_MESSAGE_TYPE: i32 = 0x6010B;
pub const UNION_FIRE_OUT_MESSAGE_TYPE: i32 = 0x6010C;
pub const FACTION_EXIT_MESSAGE_TYPE: i32 = 0x6010D;
pub const UNION_EXIT_MESSAGE_TYPE: i32 = 0x6010E;
pub const FACTION_DEMISE_MESSAGE_TYPE: i32 = 0x6010F;
pub const UNION_DEMISE_MESSAGE_TYPE: i32 = 0x60110;
pub const FACTION_DISBAND_MESSAGE_TYPE: i32 = 0x60111;
pub const UNION_DISBAND_MESSAGE_TYPE: i32 = 0x60112;
pub const FACTION_DUB_MESSAGE_TYPE: i32 = 0x60113;
pub const GRANT_FACTION_PURVIEW_MESSAGE_TYPE: i32 = 0x60114;
pub const REVOKE_FACTION_PURVIEW_MESSAGE_TYPE: i32 = 0x60115;
pub const PLAYER_INVITE_FACTION_MESSAGE_TYPE: i32 = 0x60116;
pub const UNION_APPLICATION_MESSAGE_TYPE: i32 = 0x60118;
pub const ENABLE_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011A;
pub const LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011B;
pub const EDIT_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011C;
pub const PRONOUNCE_MESSAGE_TYPE: i32 = 0x6011D;
pub const DECLARE_WAR_FACTION_LIST_MESSAGE_TYPE: i32 = 0x6011E;
pub const DECLARE_WAR_FACTION_LIST_RESPONSE_TYPE: i32 = 0x7FE18;
pub const DECLARE_FACTION_WAR_MESSAGE_TYPE: i32 = 0x6011F;
pub const DECLARE_FACTION_WAR_RESPONSE_TYPE: i32 = 0x7FE19;
pub const CONSUMED_LONG_MESSAGE_TYPES: [i32; 2] = [0x60121, 0x60123];
pub const FACTION_BILLBOARD_MESSAGE_TYPE: i32 = 0x60125;
pub const FACTION_BILLBOARD_RESPONSE_TYPE: i32 = 0x7FE1D;
pub const UPGRADE_FACTION_MESSAGE_TYPE: i32 = 0x60126;
pub const UPLOAD_FACTION_ICON_MESSAGE_TYPE: i32 = 0x60127;
pub const SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE: i32 = 0x60128;
pub const ADD_FACTION_EXPERIENCE_MESSAGE_TYPE: i32 = 0x60129;
pub const CHANGE_FACTION_MEMBER_STATE_MESSAGE_TYPE: i32 = 0x6012A;
pub const OPERATE_FACTION_TAX_MESSAGE_TYPE: i32 = 0x6012B;
pub const OPERATE_FACTION_TAX_RESPONSE_TYPE: i32 = 0x7FE28;
pub const ADJUST_FACTION_TAX_MESSAGE_TYPE: i32 = 0x6012C;
pub const ADJUST_FACTION_TAX_RESPONSE_TYPE: i32 = 0x7FE29;
pub const UPDATE_REGION_PARAM_MESSAGE_TYPE: i32 = 0x6012D;
pub const UPDATE_REGION_PARAM_RESPONSE_TYPE: i32 = 0x7FE2E;
pub const ROUTE_REGION_MESSAGE_TYPE: i32 = 0x6012E;
pub const ROUTE_REGION_RESPONSE_TYPE: i32 = 0x7FE2D;
pub const OPERATE_CITY_GATE_MESSAGE_TYPE: i32 = 0x6012F;
pub const OPERATE_CITY_GATE_RESPONSE_TYPE: i32 = 0x7FE2A;
pub const TRANSFER_CITY_OWNER_MESSAGE_TYPE: i32 = 0x60130;
pub const SET_FACTION_ADMISSION_PERMIT_MESSAGE_TYPE: i32 = 0x60132;
pub const ATTACK_CITY_END_MESSAGE_TYPE: i32 = 0x60133;
pub const APPLY_FOR_VILLAGE_WAR_MESSAGE_TYPE: i32 = 0x60135;
pub const APPLY_FOR_VILLAGE_WAR_RESPONSE_TYPE: i32 = 0x7FE34;
pub const VILLAGE_WAR_RESULT_MESSAGE_TYPE: i32 = 0x60136;
pub const APPLY_FOR_CITY_WAR_MESSAGE_TYPE: i32 = 0x60137;
pub const APPLY_FOR_CITY_WAR_RESPONSE_TYPE: i32 = 0x7FE37;
pub const CITY_WAR_RESULT_MESSAGE_TYPE: i32 = 0x60138;
pub const GOODS_WAR_COMMAND_MESSAGE_TYPE: i32 = 0x60139;
pub const GOODS_WAR_FACTION_WIN_MESSAGE_TYPE: i32 = 0x6013A;
pub const PLAYER_ADD_QUEST_MESSAGE_TYPE: i32 = 0x6013B;
pub const PLAYER_REMOVE_QUEST_MESSAGE_TYPE: i32 = 0x6013C;
pub const GAME_ADD_QUEST_MESSAGE_TYPE: i32 = 0x7FE38;
pub const GAME_REMOVE_QUEST_MESSAGE_TYPE: i32 = 0x7FE39;
pub const PLAYER_RUN_SCRIPT_MESSAGE_TYPE: i32 = 0x6013D;
pub const GAME_RUN_SCRIPT_MESSAGE_TYPE: i32 = 0x7FE3A;
pub const PLAYER_SCRIPT_CAPACITY: usize = 0x100;
pub const SET_FACTION_PARAMETER_MESSAGE_TYPE: i32 = 0x6013E;
pub const FACTION_PARAMETER_NAME_CAPACITY: usize = 0x32;
pub const CHANGE_REGION_ROUTER_MESSAGE_TYPE: i32 = 0x60144;
pub const CHANGE_REGION_ROUTER_RESPONSE_TYPE: i32 = 0x7FE4A;
pub const LEAVE_WORD_INPUT_CAPACITY: usize = 0xD2;
pub const PRONOUNCE_INPUT_CAPACITY: usize = 0x5000;

pub static FACTION_BILLBOARD_TITLES: OnceLock<[Vec<u8>; 3]> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingSessionResultDispatch {
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

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingPlayerInviteFactionDispatch<
    CreationReport,
    ApplicationReport,
    InvitationReport,
> {
    pub player_id: i32,
    pub invited_faction_id: i32,
    pub outcome:
        PlayerInviteFactionOutcome<CreationReport, ApplicationReport, InvitationReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionApplicationDispatch<SessionReport> {
    pub master_player_id: i32,
    pub applicant_faction_id: i32,
    pub outcome: OrganizingUnionApplyForJoinOutcome<SessionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingLeaveWordEnableDispatch {
    pub player_id: i32,
    pub outcome: OrganizingLeaveWordEnableOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingLeaveWordDispatch {
    pub player_id: i32,
    pub content: Vec<u8>,
    pub time: TagTimeValue,
    pub outcome: OrganizingLeaveWordOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingLeaveWordEditDispatch {
    pub leave_word_id: i32,
    pub player_id: i32,
    pub outcome: OrganizingLeaveWordEditOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingPronounceDispatch {
    pub player_id: i32,
    pub content: Vec<u8>,
    pub time: TagTimeValue,
    pub outcome: OrganizingPronounceOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizingFactionWarPlayerDiedDispatch {
    pub defeated_master_player_id: i32,
    pub victor_player_id: i32,
    pub outcome: Result<
        FactionWarPlayerDiedOutcome,
        FactionWarPlayerDiedBlock<OrganizingFactionWarPlayerDiedBlock>,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingCreateFactionGate {
    CountryMissing,
    PlayerLevel,
    RequiredGoods,
    Money,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingCreateFactionResponse {
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub result: i32,
    pub map_id: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingCreateFactionOutcome {
    EmptyName,
    PlayerOffline,
    GateRejected {
        gate: OrganizingCreateFactionGate,
        response: OrganizingCreateFactionResponse,
    },
    Creation {
        outcome: FactionCreationOutcome,
        response: OrganizingCreateFactionResponse,
        notice: OrganizingInfoDelivery,
        player_refresh: Option<PlayerFactionInfoUpdateReport>,
        faction_snapshot: Option<Result<bool, FactionClientSnapshotBlock>>,
        all_factions_snapshot: Option<Result<bool, AllFactionInfoClientBlock>>,
    },
}

#[derive(Debug)]
pub struct OrganizingCreateFactionDispatch {
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub country: u8,
    pub faction_name: Vec<u8>,
    pub outcome: OrganizingCreateFactionOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingInitialDataOutcome {
    PlayerOffline,
    AlreadyReceived,
    Sent {
        faction_snapshot: bool,
        union_snapshot: UnionClientSnapshotByPlayerOutcome,
        all_factions_snapshot: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingInitialDataBlock {
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
pub struct OrganizingInitialDataDispatch {
    pub player_id: i32,
    pub outcome: Result<OrganizingInitialDataOutcome, OrganizingInitialDataBlock>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionListResponse {
    pub socket_id: i32,
    pub total_factions: i32,
    pub included_page: Option<i32>,
    pub applied_faction_id: Option<i32>,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionListOutcome {
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
pub enum OrganizingFactionListBlock {
    PlayerCountry { player_id: i32 },
    Count(FactionCountryCountBlock),
    ApplyList { map_key: i32 },
    Page(FactionListPageBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionListDispatch {
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub page: i32,
    pub outcome: OrganizingFactionListOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionApplicationOutcome<SessionReport> {
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
pub enum OrganizingFactionApplicationDispatchBlock<SessionBlock> {
    PlayerCountry {
        player_id: i32,
    },
    NameLookup(OrganizingNameLookupBlock),
    OrganizingCountry(OrganizingNameCountryBlock),
    Faction(OrganizingFactionApplicationBlock),
    Union(OrganizingNamedUnionApplicationBlock<SessionBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionApplicationDispatch<SessionReport> {
    pub player_id: i32,
    pub discarded_value: i32,
    pub organizing_name: Vec<u8>,
    pub outcome: OrganizingFactionApplicationOutcome<SessionReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionApplicationCancelLookupPhase {
    BeforeRemoval,
    AfterRemoval,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionApplicationCancelBlock {
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
pub struct OrganizingFactionApplicationCancelDispatch {
    pub player_id: i32,
    pub previous_faction_id: i32,
    pub remaining_faction_id: Option<i32>,
    pub removal: RemovePersonFromApplyFactionListOutcome,
    pub notice_sent: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionApplicationDecisionDispatch {
    pub manager_id: i32,
    pub applicant_id: i32,
    pub approve_flag: i32,
    pub outcome: OrganizingFactionDoJoinOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionFireOutOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionFireOutOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionFireOutBlock {
    ManagerMembership { map_key: i32 },
    FireOut {
        faction_id: i32,
        source: FactionFireOutBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionFireOutDispatch {
    pub manager_id: i32,
    pub target_id: i32,
    pub outcome: OrganizingFactionFireOutOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionFireOutDispatch {
    pub manager_id: i32,
    pub target_faction_id: i32,
    pub outcome: OrganizingUnionFireOutOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionExitOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionExitOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionExitBlock {
    Membership { map_key: i32 },
    Exit {
        faction_id: i32,
        source: FactionExitBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionExitDispatch {
    pub player_id: i32,
    pub outcome: OrganizingFactionExitOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionExitDispatch {
    pub player_id: i32,
    pub outcome: OrganizingUnionExitOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDemiseOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionDemiseOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDemiseBlock {
    Membership { map_key: i32 },
    Demise {
        faction_id: i32,
        source: FactionDemiseBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionDemiseDispatch {
    pub old_master_id: i32,
    pub new_master_id: i32,
    pub outcome: OrganizingFactionDemiseOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionDemiseDispatch {
    pub old_master_player_id: i32,
    pub new_master_faction_id: i32,
    pub outcome: OrganizingUnionDemiseOutcome,
}

pub struct PendingOrganizingFactionDisbandDispatch {
    pub player_id: i32,
    pub faction_id: i32,
    pub outcome: OrganizingDisbandOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDisbandOutcome {
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
pub struct OrganizingFactionDisbandDispatch {
    pub player_id: i32,
    pub faction_id: i32,
    pub outcome: OrganizingFactionDisbandOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDisbandBlock {
    Membership { map_key: i32 },
    Disband {
        faction_id: i32,
        source: OrganizingDisbandBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionDisbandOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: OrganizingConfederationDisbandOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingUnionDisbandBlock {
    Lookup(OrganizingUnionByMasterBlock),
    Disband {
        union_id: i32,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingUnionDisbandDispatch {
    pub player_id: i32,
    pub outcome: OrganizingUnionDisbandOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDubOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionDubOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionDubBlock {
    Membership { map_key: i32 },
    Dub {
        faction_id: i32,
        source: FactionDubBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionDubDispatch {
    pub target_id: i32,
    pub job_level: i32,
    pub title: Vec<u8>,
    pub manager_id: i32,
    pub outcome: OrganizingFactionDubOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionPurviewOutcome {
    FactionNotFound { faction_id: i32 },
    Applied {
        faction_id: i32,
        outcome: FactionPurviewChangeOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionPurviewBlock {
    Membership { map_key: i32 },
    Change {
        faction_id: i32,
        source: FactionPurviewChangeBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionPurviewDispatch {
    pub change: FactionPurviewChange,
    pub target_id: i32,
    pub purview: i32,
    pub manager_id: i32,
    pub outcome: OrganizingFactionPurviewOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingConsumedLongDispatch {
    pub message_type: i32,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingDeclareWarFactionListNotice {
    MissingFaction,
    MasterRequired,
    NoFactions,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingDeclareWarFactionListResponse {
    pub socket_id: i32,
    pub total_factions: i32,
    pub included_page: Option<i32>,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDeclareWarFactionListOutcome {
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
pub enum OrganizingDeclareWarFactionListBlock {
    Membership { map_key: i32 },
    Page(DeclareWarFactionPageBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingDeclareWarFactionListDispatch {
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub page: i32,
    pub outcome: OrganizingDeclareWarFactionListOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingDeclareFactionWarResponse {
    pub socket_id: i32,
    pub result_money: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDeclareFactionWarOutcome {
    PlayerOffline,
    Declaration(FactionWarDeclarationOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDeclareFactionWarBlock {
    PlayerDecode(PlayerCodecError),
    Declaration(FactionWarDeclarationBlock<OrganizingFactionWarDeclarationBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingDeclareFactionWarDispatch {
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub target_faction_id: i32,
    pub war_type: i32,
    pub outcome: OrganizingDeclareFactionWarOutcome,
    pub response: OrganizingDeclareFactionWarResponse,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionBillboardResponse {
    pub socket_id: i32,
    pub title: Vec<u8>,
    pub payload: Vec<u8>,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionBillboardOutcome {
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
pub struct OrganizingFactionBillboardBlock {
    pub request_id: i32,
    pub billboard_type: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionUpgradeOutcome {
    PlayerOffline,
    FactionMissing,
    Upgrade(FactionUpgradeOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionUpgradeBlock {
    PlayerDecode(PlayerCodecError),
    Upgrade(FactionUpgradeBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionUpgradeDispatch {
    pub faction_id: i32,
    pub player_id: i32,
    pub outcome: OrganizingFactionUpgradeOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionUploadIconOutcome {
    FactionMissing,
    UploadIcon {
        time: TagTimeValue,
        outcome: FactionUploadIconOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionUploadIconDispatch {
    pub faction_id: i32,
    pub player_id: i32,
    pub outcome: OrganizingFactionUploadIconOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionContributorDispatch {
    pub target_player_id: i32,
    pub enabled_value: i32,
    pub requester_player_id: i32,
    pub outcome: OrganizingContributorOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionExperienceOutcome {
    FactionMissing,
    PlayerNotContributor,
    Applied {
        before_experience: i32,
        update: FactionExperienceUpdate,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionExperienceDispatch {
    pub faction_id: i32,
    pub player_id: i32,
    pub experience_delta: i32,
    pub outcome: OrganizingFactionExperienceOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionMemberStateDispatch {
    pub faction_id: i32,
    pub player_id: i32,
    pub operation: i32,
    pub outcome: OrganizingFactionMemberStateOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionTaxResponse {
    pub socket_id: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionTaxOutcome {
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
pub enum OrganizingFactionTaxBlock {
    Membership { map_key: i32 },
    Operation {
        faction_id: i32,
        source: FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionTaxDispatch {
    pub request_type: i32,
    pub player_id: i32,
    pub region_id: i32,
    pub outcome: OrganizingFactionTaxOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingRegionParamBroadcast {
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingRegionParamDispatch {
    pub region_id: i32,
    pub today_total_tax: u32,
    pub total_tax: u32,
    pub current_tax_rate: i32,
    pub region: WorldRegionParamUpdateOutcome,
    pub broadcast: Option<OrganizingRegionParamBroadcast>,
}

/// Ветвь `0x6012D` без organizing-шва: применение налоговых параметров региона
/// через [`WorldGameView`], ответ `0x7FE2E` рассылается всем GameServer ровно
/// при `Applied`, как в исходном диспетчере.
pub fn dispatch_region_param_update(
    message: &mut CMessage,
    game: &mut dyn WorldGameView,
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
pub struct OrganizingRegionRouteDispatch {
    pub region_id: i32,
    pub game_server_number: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingCityGateResponse {
    pub game_server_number: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingCityGateOutcome {
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
pub enum OrganizingCityGateBlock {
    Membership { map_key: i32 },
    Operation {
        faction_id: i32,
        source: FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingCityGateDispatch {
    pub player_id: i32,
    pub region_id: i32,
    pub outcome: OrganizingCityGateOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingCityTransferDispatch<SessionReport> {
    pub requester_player_id: i32,
    pub target_faction_id: i32,
    pub region_id: i32,
    pub outcome: CityTransferStartOutcome<SessionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingAdmissionPermitDispatch {
    pub requested_value: i32,
    pub player_id: i32,
    pub faction_id: Option<i32>,
    pub outcome: Option<FactionPermitUpdate>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingAdmissionPermitBlock {
    Membership { map_key: i32 },
    Permit {
        faction_id: i32,
        source: FactionPermitBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingAttackCityEndDispatch {
    pub result: i32,
    pub region_id: i32,
    pub attacker_player_id: i32,
    pub defender_faction_id: i32,
    pub outcome: AttackCityEndReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingVillageWarApplicationBlock {
    FactionMaster(FactionMasterLookupBlock),
    MissingFaction { faction_id: i32 },
    MissingFactionLevel { faction_id: i32 },
    NullUnion { map_key: i32 },
    MissingRegionOwner { region_id: i32 },
    MissingRegionName { region_id: i32 },
    NoticeWouldOverflow { visible_len: usize },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingVillageWarApplicationDispatch {
    pub player_id: i32,
    pub war_number: i32,
    pub legacy_third_parameter: i32,
    pub outcome: VillageWarApplicationReport,
    pub response: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingCityWarApplicationBlock {
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
pub struct OrganizingCityWarApplicationDispatch {
    pub player_id: i32,
    pub war_number: i32,
    pub legacy_third_parameter: i32,
    pub outcome: AttackCityApplicationReport,
    pub response: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingCityWarResultDispatch {
    pub war_number: i32,
    pub war_region_id: i32,
    pub winner_faction_id: i32,
    pub reported_union_id: i32,
    pub outcome: Result<
        AttackCityWarResultReport,
        AttackCityWarResultBlock<OrganizingCityWarResultContextBlock>,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingGoodsWarContextBlock {
    pub faction_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingGoodsWarCommandOutcome {
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
pub struct OrganizingGoodsWarCommandDispatch {
    pub operation: i32,
    pub outcome: OrganizingGoodsWarCommandOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingGoodsWarFactionWinBlock {
    MissingMasterId {
        requested_faction_id: i32,
        actual_faction_id: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingGoodsWarFactionWinDispatch {
    pub requested_faction_id: i32,
    pub faction_found: bool,
    pub report: Option<GoodsWarFactionWinReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingPlayerQuestCommandKind {
    Add,
    Remove,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingPlayerQuestCommandDispatch {
    pub kind: OrganizingPlayerQuestCommandKind,
    pub player_id: i32,
    pub quest_id: i16,
    pub game_server_id: i32,
    pub delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingPlayerRunScriptDispatch {
    pub player_id: i32,
    pub script: Vec<u8>,
    pub game_server_id: i32,
    pub delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionParameterOutcome {
    FactionMissing,
    Applied {
        faction_id: i32,
        outcome: FactionSetParameterOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingFactionParameterBlock {
    FactionMaster(FactionMasterLookupBlock),
    SetParameter {
        faction_id: i32,
        source: FactionSetParameterBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingFactionParameterDispatch {
    pub player_id: i32,
    pub parameter: Vec<u8>,
    pub value: i32,
    pub outcome: OrganizingFactionParameterOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizingChangeRegionRouterDispatch {
    pub request_id: i32,
    pub from_region: i32,
    pub to_region: i32,
    pub target: RegionRoutePoint,
    pub outcome: RegionRouterChangeOutcome,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingVillageWarResultContextBlock {
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
pub struct OrganizingVillageWarResultDispatch {
    pub war_number: i32,
    pub war_region_id: i32,
    pub winner_faction_id: i32,
    pub legacy_fourth_parameter: i32,
    pub outcome: Result<
        VillageWarResultReport,
        VillageWarResultBlock<OrganizingVillageWarResultContextBlock>,
    >,
}
