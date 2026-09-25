//! Session терминальный слой async confirm-подтверждений organizing-сообщений:
//! типы-конечники, их in-memory очередь и четыре endpoint-узла слиты в одного
//! владельца [`WorldOrganizingSessionRuntimeOwner`]; старый
//! `appworld/message/organsysmessage.rs` держит type-алиас прежнего имени
//! владельца для main-loop runtime и адаптеров.
//!
//! Здесь же чистые data-контракты диспетчера: opcode-константы и
//! dispatch/outcome/block/response-семейства, чьи поля цитируют только
//! Realm/Shared-типы, ветвь `0x6012D` region param update, которую открывает
//! только [`crate::app::world_game_view::WorldGameView`], все
//! membership/governance ветви (`0x60107`—`0x6013E` блоки заявок, выходов,
//! распусков, dub/purview, leave-word/pronounce, налоговые и городские
//! операции, списки и goods-war исход) и war-ветви (`0x60101` player died,
//! `0x6011F` declare faction war, village/city application+result
//! `0x60135`—`0x60138`, goods war `0x60139`/`0x6013A` и семейный
//! `reload_attack_city`). Ветви принимают организационный владелец как
//! [`WorldOrganizingDispatchView`] (статический, generic context подписей
//! контроллера); их `CGame`/war-зависимые context-адаптеры остаются у старого
//! пакета и передаются generic-параметрами, а goods-war member контекст
//! целиком видовой — поверх `WorldOrganizingDispatchView` и
//! [`WorldGameView`]. Диспетчерные ветви, открывающие узлы старого пакета
//! напрямую (session-handlers, creation `0x60103`, upgrade `0x60126`,
//! billboard, transfer), остаются в старом файле; он реэкспортирует
//! перенесённое и держит тонкую обвязку для ветвей с адаптерами — включая
//! двухфазную `0x6011F`, чей decode игрока из wire-хвоста выполняется
//! старым владельцем игры между realm-разбором и realm-завершением.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{RegionRoutePoint, RegionRouterChangeOutcome};
use nebokrai_shared::runtime::{CTimer, NetSessionCallbackOutcome, TimerId};
use nebokrai_shared::values::TagTime;
use parking_lot::Mutex;

use crate::activities::attackcitysys::{
    AttackCityApplicationContext, AttackCityApplicationReport, AttackCityCallbacks,
    AttackCityReloadBlock, AttackCityReloadReport, AttackCityWarEndContext, AttackCityWarResultBlock,
    AttackCityWarResultContext, AttackCityWarResultReport, CAttackCitySys,
};
use crate::activities::factionwarsys::{
    CFactionWarSys, FactionWarDeclarationBlock, FactionWarDeclarationContext,
    FactionWarDeclarationOutcome, FactionWarPlayerDiedBlock, FactionWarPlayerDiedContext,
    FactionWarPlayerDiedOutcome,
};
use crate::activities::villagewarsys::{
    CVillageWarSys, VillageWarApplicationContext, VillageWarApplicationReport, VillageWarCallbacks,
    VillageWarResultBlock, VillageWarResultContext, VillageWarResultReport,
};
use crate::app::world_game_view::{WorldGameView, WorldRegionParamUpdateOutcome};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::world_organizing_view::WorldOrganizingDispatchView;
use crate::characters::player::{PlayerCodecError, PlayerFactionInfoUpdateReport};
use crate::content::organizing::{ECityState, EOperator, TagTimeValue};
use crate::organizations::country::CountryGovernanceContextBlock;
use crate::organizations::faction::{
    FactionApplyForJoinEffects, FactionApplyForJoinOutcome, FactionBillboardStatBlock,
    FactionContributorContext, FactionDemiseBlock, FactionDemiseContext, FactionDemiseOutcome,
    FactionDisbandContext, FactionDoJoinEffects, FactionDubBlock, FactionDubContext,
    FactionDubOutcome, FactionExitBlock, FactionExitContext, FactionExitOutcome,
    FactionExperienceBlock, FactionExperienceUpdate, FactionFireOutBlock, FactionFireOutContext,
    FactionFireOutOutcome, FactionInitialPropertyBlock, FactionMemberInfoRequest,
    FactionOperationBlock, FactionOperationOutcome, FactionOperationRejection,
    FactionOrganizingInfoContext, FactionPermitBlock, FactionPermitUpdate, FactionPurviewChange,
    FactionPurviewChangeBlock, FactionPurviewChangeContext, FactionPurviewChangeOutcome,
    FactionSetParameterBlock, FactionSetParameterContext, FactionSetParameterOutcome,
    FactionUpgradeBlock, FactionUpgradeOutcome, FactionUploadIconBlock,
    FactionUploadIconContext, FactionUploadIconOutcome, OwnedCityMutationBuildError,
    current_local_member_time,
};
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;
use crate::organizations::goodswarmember::{
    CGoodsWarMember, GoodsWarAuditEnvironment, GoodsWarAuditPlayer, GoodsWarDeliveryContext,
    GoodsWarFactionSnapshot, GoodsWarFactionWinReport, GoodsWarFactionWinSnapshot,
    GoodsWarMemberBlock, GoodsWarMemberContext, GoodsWarMutationReport, GoodsWarRefreshReport,
};
use crate::organizations::organizingctrl::{
    AllFactionInfoClientBlock, ApplyFactionLookup, AttackCityEndBlock, AttackCityEndEffects,
    AttackCityEndReport, CityTransferSessionRuntime, CityTransferStartOutcome,
    ConfederationCreationSessionRuntime, DeclareWarFactionPage, DeclareWarFactionPageBlock,
    FactionClientSnapshotBlock, FactionCountryCountBlock, FactionCreationOutcome,
    FactionListPage, FactionListPageBlock, FactionMasterLookupBlock,
    FactionUnionMembershipLookupBlock, FreePlayerLookup, OrganizingConfederationDisbandBlock,
    OrganizingConfederationDisbandOutcome, OrganizingContributorBlock,
    OrganizingContributorOutcome, OrganizingDisbandBlock, OrganizingDisbandOutcome,
    OrganizingDisbandPlayer, OrganizingDisbandProgress, OrganizingDisbandRejection,
    OrganizingFactionApplicationBlock, OrganizingFactionDoJoinBlock,
    OrganizingFactionDoJoinOutcome, OrganizingFactionExperienceMutation,
    OrganizingFactionMemberStateOutcome, OrganizingFactionWarDeclarationBlock,
    OrganizingFactionWarPlayerDiedBlock, OrganizingInfoDelivery,
    OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock, OrganizingLeaveWordEditOutcome,
    OrganizingLeaveWordEnableBlock, OrganizingLeaveWordEnableOutcome,
    OrganizingLeaveWordOutcome, OrganizingNameCountryBlock, OrganizingNameKind,
    OrganizingNameLookupBlock, OrganizingNameMatch, OrganizingNamedUnionApplicationBlock,
    OrganizingPronounceBlock,
    OrganizingPronounceOutcome, OrganizingUnionApplyForJoinOutcome,
    OrganizingUnionByMasterBlock, OrganizingUnionDemiseBlock, OrganizingUnionDemiseOutcome,
    OrganizingUnionExitBlock, OrganizingUnionExitOutcome, OrganizingUnionFireOutBlock,
    OrganizingUnionFireOutOutcome, PlayerInviteFactionOutcome,
    RemovePersonFromApplyFactionListOutcome, UnionClientSnapshotByPlayerBlock,
    UnionClientSnapshotByPlayerOutcome,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::organizations::union::{
    CityTransferEndpointBlock, CityTransferTerminal, ConfederationCreationEndpointBlock,
    ConfederationCreationTerminal, UnionApplicationEndpointBlock,
    UnionApplicationSessionRuntime, UnionApplicationTerminal, UnionApplyForJoinEffects,
    UnionApplyForJoinOutcome, UnionFireOutEffects, UnionInvitationSessionRuntime,
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


/// Контекст country-lookup-а ветви `0x60107`: `None` — offline miss,
/// внутренний `None` — ещё не готовый country найденного player-owner-а.
/// Перенесён из старого диспетчера вместе с обработчиком списка фракций.
pub trait FactionApplicationListContext: FactionOrganizingInfoContext {
    fn online_player_country(&self, player_id: i32) -> Option<Option<u8>>;
}

/// Выполняет `0x6011A`: один `Long`, master-faction lookup и включение
/// leave-word функции без route/tail/wire ingress-а.
pub fn dispatch_leave_word_enable<Context>(
    message: &mut CMessage,
    organizing: &mut impl WorldOrganizingDispatchView,
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

/// Выполняет `0x6011B`: текст сообщения до ёмкости, один `Long` и один
/// local-time snapshot перед virtual `CFaction::LeaveWord`.
pub fn dispatch_leave_word(
    message: &mut CMessage,
    game: &mut dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
) -> Option<Result<OrganizingLeaveWordDispatch, OrganizingLeaveWordBlock>> {
    if message.message_type() != LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(LEAVE_WORD_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let time = current_local_member_time();
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

/// Выполняет `0x6011C`: два `Long` и virtual `CFaction::EditLeaveWord` с
/// фиксированным `EOperator::Delete`.
pub fn dispatch_leave_word_edit(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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

/// Выполняет `0x6011D`: текст сообщения до ёмкости, один `Long` и один
/// local-time snapshot перед virtual `CFaction::Pronounce`.
pub fn dispatch_pronounce(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
) -> Option<Result<OrganizingPronounceDispatch, OrganizingPronounceBlock>> {
    if message.message_type() != PRONOUNCE_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(PRONOUNCE_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let time = current_local_member_time();
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

/// Выполняет `0x60107`: `(request ID, cookie, player ID, page)`, пустой
/// notice и 11-элементная страница списка фракций страны игрока.
pub fn dispatch_faction_list<Context>(
    message: &mut CMessage,
    organizing: &impl WorldOrganizingDispatchView,
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

/// Выполняет `0x60108`: `(player ID, discarded Long, name[20])`,
/// online/country gate и virtual `ApplyForJoin(player ID, 0, 0)`.
#[allow(clippy::too_many_arguments, reason = "точная форма ветви диспетчера")]
pub fn dispatch_faction_application<FactionEffects, UnionEffects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    faction_effects: &mut FactionEffects,
    union_effects: &mut UnionEffects,
) -> Option<
    Result<
        OrganizingFactionApplicationDispatch<UnionEffects::SessionReport>,
        OrganizingFactionApplicationDispatchBlock<UnionEffects::SessionBlock>,
    >,
>
where
    FactionEffects: FactionApplyForJoinEffects,
    UnionEffects: UnionApplyForJoinEffects,
{
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
            let outcome = match organizing.apply_for_faction_join_by_map_key(
                game,
                parameters,
                matched.map_key,
                player_id,
                0,
                0,
                faction_effects,
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
                union_effects,
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

/// Выполняет `0x60109`: очищает все faction apply-list и уведомляет
/// только о подтверждённом переходе из положительной apply-faction в пустую.
pub fn dispatch_faction_application_cancel<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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

/// Выполняет `0x6010A`: три `Long`, manager-faction lookup, один local
/// time snapshot и virtual `CFaction::DoJoin` без route/tail/wire ingress-а.
pub fn dispatch_faction_application_decision<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    effects: &mut Effects,
) -> Option<
    Result<OrganizingFactionApplicationDecisionDispatch, OrganizingFactionDoJoinBlock>,
>
where
    Effects: FactionDoJoinEffects,
{
    if message.message_type() != FACTION_APPLICATION_DECISION_MESSAGE_TYPE {
        return None;
    }

    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let applicant_id = message.base_mut().get_long().unwrap_or(0);
    let approve_flag = message.base_mut().get_long().unwrap_or(0);
    let outcome = match organizing.do_faction_join_by_manager(
        game,
        parameters,
        manager_id,
        applicant_id,
        approve_flag,
        current_local_member_time,
        effects,
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

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}


/// Выполняет `0x6010B`: два `Long`, manager-faction lookup и virtual
/// `CFaction::FireOut` без route/tail/wire ingress-а.
pub fn dispatch_faction_fire_out<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionFireOutDispatch, OrganizingFactionFireOutBlock>>
where
    Context: FactionFireOutContext,
{
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
    let outcome = match faction.fire_out(game, parameters, manager_id, target_id, context) {
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

/// Выполняет `0x6010C`: два `Long`, nullable `GetUnion(manager)`, virtual
/// `CUnion::FireOut` и автоматический disband при member count `<= 1`.
pub fn dispatch_union_fire_out<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    effects: &mut Effects,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>>
where
    Effects: UnionFireOutEffects,
{
    if message.message_type() != UNION_FIRE_OUT_MESSAGE_TYPE {
        return None;
    }

    let manager_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = match organizing.fire_out_union_by_master(
        game,
        parameters,
        manager_id,
        target_faction_id,
        effects,
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

/// Выполняет `0x6010D`: один `Long`, faction lookup и virtual
/// `CFaction::Exit` без route/tail/wire ingress-а.
pub fn dispatch_faction_exit<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionExitDispatch, OrganizingFactionExitBlock>>
where
    Context: FactionExitContext,
{
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
    let outcome = match faction.exit(game, parameters, player_id, context) {
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

/// Выполняет `0x6010E`: один `Long`, player/faction/union lookup,
/// virtual `CUnion::Exit` и automatic disband через `GetPlayerHeader`.
pub fn dispatch_union_exit<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    effects: &mut Effects,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>>
where
    Effects: UnionFireOutEffects,
{
    if message.message_type() != UNION_EXIT_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = match organizing.exit_union_by_player(
        game,
        parameters,
        player_id,
        effects,
        update_player,
    ) {
        Ok(outcome) => outcome,
        Err(source) => return Some(Err(source)),
    };
    Some(Ok(OrganizingUnionExitDispatch { player_id, outcome }))
}

/// Выполняет `0x6010F`: два `Long`, faction lookup и virtual
/// `CFaction::Demise` без route/tail/wire ingress-а.
pub fn dispatch_faction_demise<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionDemiseDispatch, OrganizingFactionDemiseBlock>>
where
    Context: FactionDemiseContext,
{
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
    let outcome = match faction.demise(
        game,
        parameters,
        old_master_id,
        new_master_id,
        context,
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

/// Выполняет `0x60110`: два полных `Long`, nullable
/// `GetUnion(old master)` и virtual `CUnion::Demise(old, new faction)`.
pub fn dispatch_union_demise<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    effects: &mut Effects,
    get_tick: &mut dyn FnMut() -> u32,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>>
where
    Effects: UnionFireOutEffects,
{
    if message.message_type() != UNION_DEMISE_MESSAGE_TYPE {
        return None;
    }

    let old_master_player_id = message.base_mut().get_long().unwrap_or(0);
    let new_master_faction_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = match organizing.demise_union_by_master(
        game,
        old_master_player_id,
        new_master_faction_id,
        effects,
        get_tick,
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

/// Выполняет synchronous prefix `0x60111`: один `Long`, ordered
/// `IsFreePlayer` и безусловный `DisbandFaction(player, faction)`.
pub fn dispatch_faction_disband<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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
pub fn finalize_faction_disband_dispatch<ClearPlayer, WriteLog>(
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

/// Выполняет `0x60112`: один `Long`, nullable `GetUnion(player)`,
/// virtual `GetID` и `DisbandConferation(player, union ID)`.
pub fn dispatch_union_disband<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    effects: &mut Effects,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>>
where
    Effects: UnionFireOutEffects,
{
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
    let outcome = match organizing.disband_confederation(
        game,
        parameters,
        player_id,
        union_id,
        effects,
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

/// Выполняет `0x60113`: `(target, job-level, title[20], manager)`,
/// faction lookup по manager и virtual `CFaction::DubAndSetJobLvl`.
pub fn dispatch_faction_dub<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    context: &mut Context,
) -> Option<Result<OrganizingFactionDubDispatch, OrganizingFactionDubBlock>>
where
    Context: FactionDubContext,
{
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
    let outcome = match faction.dub_and_set_job_level(
        game,
        manager_id,
        target_id,
        &mut title,
        job_level,
        context,
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

/// Выполняет парные `0x60114/0x60115`: три `Long`, faction lookup по
/// manager и virtual grant/revoke owner без ingress gates.
pub fn dispatch_faction_purview<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    context: &mut Context,
) -> Option<Result<OrganizingFactionPurviewDispatch, OrganizingFactionPurviewBlock>>
where
    Context: FactionPurviewChangeContext,
{
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
    let changed = match change {
        FactionPurviewChange::Grant => faction.endue_right_to_member(
            game,
            manager_id,
            target_id,
            purview,
            context,
        ),
        FactionPurviewChange::Revoke => faction.abolish_right_to_member(
            game,
            manager_id,
            target_id,
            purview,
            context,
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

/// Выполняет `0x6011E`: master gate фракции игрока, notice-ветви и
/// 11-элементная страница целей объявления войны.
pub fn dispatch_declare_war_faction_list<Context>(
    message: &mut CMessage,
    organizing: &impl WorldOrganizingDispatchView,
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


/// Выполняет `0x60127`: два `Long`, existence gate фракции и один
/// local-time snapshot перед controller `UploadFactionIcon`.
pub fn dispatch_faction_upload_icon<Context>(
    message: &mut CMessage,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>>
where
    Context: FactionUploadIconContext,
{
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

    let time = current_local_member_time();
    let outcome = match organizing.upload_faction_icon(
        parameters,
        faction_id,
        player_id,
        &time,
        context,
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

/// Выполняет `0x60128`: три `Long` и controller `SetContributor принося`
/// без обязательства результата контекста.
pub fn dispatch_faction_contributor<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>>
where
    Context: FactionContributorContext,
{
    if message.message_type() != SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE {
        return None;
    }

    let target_player_id = message.base_mut().get_long().unwrap_or(0);
    let enabled_value = message.base_mut().get_long().unwrap_or(0);
    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .set_contributor_for_player(
                game,
                parameters,
                requester_player_id,
                target_player_id,
                enabled_value != 0,
                context,
            )
            .map(|outcome| OrganizingFactionContributorDispatch {
                target_player_id,
                enabled_value,
                requester_player_id,
                outcome,
            }),
    )
}

/// Выполняет `0x60129`: три `Long`, controller add-experience и отложенный
/// опциональный DB-log только при online авторе опыта.
pub fn dispatch_faction_experience(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    use_log_system: bool,
    faction_experience_log_enabled: bool,
    write_faction_experience_log: &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
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
    if use_log_system && faction_experience_log_enabled {
        if let Some(player) = game.online_player_by_id(player_id as u32) {
            write_faction_experience_log(
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

/// Выполняет `0x6012A`: три `Long` и продолжение чтения operation-specific
/// поля из wire с той же позиции.
pub fn dispatch_faction_member_state(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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
        &mut || message.base_mut().get_long().unwrap_or(0),
    );
    Some(OrganizingFactionMemberStateDispatch {
        faction_id,
        player_id,
        operation,
        outcome,
    })
}

/// Выполняет парные `0x6012B/0x6012C`: war-state gates региона,
/// operator-проверку и ответ тем же сообщением при авторизации.
pub fn dispatch_faction_tax<Context>(
    message: &mut CMessage,
    organizing: &impl WorldOrganizingDispatchView,
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

/// Выполняет `0x6012F`: operator-проверку city gate и ретрансляцию
/// сообщения game server-у региона при авторизации.
pub fn dispatch_city_gate(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &impl WorldOrganizingDispatchView,
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

/// Выполняет `0x60132`: faction lookup player-а и установку permit-флага
/// с union-master сверкой внутри контроллера.
pub fn dispatch_admission_permit(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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

/// Выполняет `0x60133`: четыре `Long` и controller-итог атаки города
/// без route/wire иных ветвей.
#[allow(clippy::too_many_arguments, reason = "точная форма ветви диспетчера")]
pub fn dispatch_attack_city_end<Effects>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
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

// Ветви `0x60139`/`0x6013A` перенесены ниже волной war-ветвей: заёмный
// блокер волны организационного view закрыт тем, что goods-war member
// контекст здесь целиком видовой (`WorldGoodsWarMemberDispatchContext`
// поверх `WorldOrganizingDispatchView`, чей `set_faction_goods_war_count`
// добавлен той волной, и `WorldGameView` с `login_server_id`), а не держит
// конкретных заёмов owner-ов старого пакета.

/// Выполняет `0x6013E`: `(player ID, parameter[0x32], value)`,
/// master-faction lookup и controller `SetFactionParameter`.
pub fn dispatch_faction_parameter<Context>(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    parameters: &COrganizingParam,
    context: &mut Context,
) -> Option<Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>>
where
    Context: FactionSetParameterContext,
{
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

    let outcome = match organizing.set_faction_parameter(
        game,
        parameters,
        faction_id,
        &parameter,
        value,
        context,
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

// ---------------------------------------------------------------------------
// War-ветви диспетчера: faction war (`0x60101`, `0x6011F`), village war
// (`0x60135`, `0x60136`), city war (`0x60137`, `0x60138` и семейный
// `reload_attack_city`), goods war (`0x60139`, `0x6013A`).
//
// War-системы (`CFactionWarSys`/`CVillageWarSys`/`CAttackCitySys`/
// `CGoodsWarMember`) уже принадлежат Realm и generic по своим context-ам,
// поэтому ветви держат только разбор wire-полей, вызов war-системы и сборку
// исходящих delivery в исходных позициях; context-адаптеры с живыми
// `CGame`/`COrganizingCtrl`/country-швами старого пакета приходят
// generic-параметрами от обвязки старого файла. Исключение — goods war: её
// контекст ниже целиком видовой.

/// Выполняет `0x60101`: два `Long` (побеждённый master-игрок и победитель) и
/// `CFactionWarSys::OnPlayerDied`.
pub fn dispatch_faction_war_player_died<Context>(
    message: &mut CMessage,
    faction_war: &mut CFactionWarSys,
    context: &mut Context,
) -> Option<OrganizingFactionWarPlayerDiedDispatch>
where
    Context: FactionWarPlayerDiedContext<Block = OrganizingFactionWarPlayerDiedBlock>,
{
    if message.message_type() != FACTION_WAR_PLAYER_DIED_MESSAGE_TYPE {
        return None;
    }
    let defeated_master_player_id = message.base_mut().get_long().unwrap_or(0);
    let victor_player_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = faction_war.on_player_died(defeated_master_player_id, victor_player_id, context);
    Some(OrganizingFactionWarPlayerDiedDispatch {
        defeated_master_player_id,
        victor_player_id,
        outcome,
    })
}

/// Разобранная шапка ветви `0x6011F` declare faction war. Вынесена в отдельный
/// тип, потому что decode игрока из wire-хвоста (`DecordOnLinePlayerByID`)
/// остаётся у старого владельца игры и выполняется обвязкой между realm-
/// разбором и realm-завершением: заранее построенный effects-адаптер ветви
/// держит общий заём той же игры, и два заёма одного owner-а не проходят одну
/// границу вызова (та же семья блокировки, что волна организационного view
/// зафиксировала у `0x60126`); разбивка сохраняет исходный порядок «разбор →
/// decode → declaration/response» буквально.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingDeclareFactionWarRequest {
    pub socket_id: i32,
    pub request_id: i64,
    pub cookie: i32,
    pub player_id: i32,
    pub target_faction_id: i32,
    pub war_type: i32,
}

/// Разбирает `0x6011F`: `(socket, request ID, cookie, player ID, target
/// faction, war type)`. `None` — чужой opcode, wire-курсор не сдвигается.
pub fn parse_declare_faction_war(
    message: &mut CMessage,
) -> Option<OrganizingDeclareFactionWarRequest> {
    if message.message_type() != DECLARE_FACTION_WAR_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let war_type = message.base_mut().get_long().unwrap_or(0);
    Some(OrganizingDeclareFactionWarRequest {
        socket_id,
        request_id,
        cookie,
        player_id,
        target_faction_id,
        war_type,
    })
}

/// Завершает `0x6011F` после decode игрока обвязкой: при online — snapshot
/// local time и `DigUpTheHatchet` (деньги только проверяются и отражаются в
/// ответе `0x7FE19`, как в исходной ветви), затем response на socket.
pub fn finish_declare_faction_war<Context>(
    request: OrganizingDeclareFactionWarRequest,
    player_online: bool,
    faction_wars: &mut CFactionWarSys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Result<OrganizingDeclareFactionWarDispatch, OrganizingDeclareFactionWarBlock>
where
    Context: FactionWarDeclarationContext<Block = OrganizingFactionWarDeclarationBlock>,
{
    let (outcome, result_money) = if player_online {
        let declaration_time = TagTime::local_now();
        let declaration = match faction_wars.dig_up_the_hatchet(
            request.player_id,
            request.target_faction_id,
            request.war_type,
            declaration_time,
            context,
        ) {
            Ok(declaration) => declaration,
            Err(source) => {
                return Err(OrganizingDeclareFactionWarBlock::Declaration(source));
            }
        };
        let result_money = if declaration.legacy_result() {
            faction_wars.get_dec_war_money_by_type(request.war_type)
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
    response.base_mut().add_long64(request.request_id);
    response.base_mut().add_long(request.cookie);
    response.base_mut().add_long(request.player_id);
    response.base_mut().add_long(result_money);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, request.socket_id);

    Ok(OrganizingDeclareFactionWarDispatch {
        request_id: request.request_id,
        cookie: request.cookie,
        player_id: request.player_id,
        target_faction_id: request.target_faction_id,
        war_type: request.war_type,
        outcome,
        response: OrganizingDeclareFactionWarResponse {
            socket_id: request.socket_id,
            result_money,
            wire,
            delivery,
        },
    })
}

/// Выполняет `0x60135`: `(player ID, war number, legacy third Long)` и
/// `CVillageWarSys::ApplyForVillageWar`; принятая заявка отвечает `0x7FE34`
/// на исходный map.
pub fn dispatch_village_war_application<Context>(
    message: &mut CMessage,
    village_war: &mut CVillageWarSys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingVillageWarApplicationDispatch, Context::Block>>
where
    Context: VillageWarApplicationContext + ?Sized,
{
    if message.message_type() != APPLY_FOR_VILLAGE_WAR_MESSAGE_TYPE {
        return None;
    }

    let source_map_id = message.map_id();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let war_number = message.base_mut().get_long().unwrap_or(0);
    let legacy_third_parameter = message.base_mut().get_long().unwrap_or(0);
    let outcome = village_war.apply_for_village_war(
        player_id,
        war_number,
        legacy_third_parameter,
        context,
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

/// Выполняет `0x60136`: четыре `Long` (war number, region, winner faction,
/// legacy fourth) и `CVillageWarSys::OnFactionWinVillage` с generic timer.
pub fn dispatch_village_war_result<Callback, Context>(
    message: &mut CMessage,
    village_war: &mut CVillageWarSys,
    timer: &mut CTimer<Callback>,
    callbacks: VillageWarCallbacks<Callback>,
    context: &mut Context,
) -> Option<OrganizingVillageWarResultDispatch>
where
    Callback: Copy,
    Context: VillageWarResultContext<Block = OrganizingVillageWarResultContextBlock> + ?Sized,
{
    if message.message_type() != VILLAGE_WAR_RESULT_MESSAGE_TYPE {
        return None;
    }

    let war_number = message.base_mut().get_long().unwrap_or(0);
    let war_region_id = message.base_mut().get_long().unwrap_or(0);
    let winner_faction_id = message.base_mut().get_long().unwrap_or(0);
    let legacy_fourth_parameter = message.base_mut().get_long().unwrap_or(0);
    let outcome = village_war.on_faction_win_village(
        war_number,
        war_region_id,
        winner_faction_id,
        legacy_fourth_parameter,
        timer,
        callbacks,
        context,
    );
    Some(OrganizingVillageWarResultDispatch {
        war_number,
        war_region_id,
        winner_faction_id,
        legacy_fourth_parameter,
        outcome,
    })
}

/// Выполняет `0x60137`: `(player ID, war number, legacy third Long)` и
/// `CAttackCitySys::OnPlayerDeclareWar`; принятая заявка отвечает `0x7FE37`
/// на исходный map.
pub fn dispatch_city_war_application<Context>(
    message: &mut CMessage,
    attack_city: &mut CAttackCitySys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingCityWarApplicationDispatch, Context::Block>>
where
    Context: AttackCityApplicationContext + ?Sized,
{
    if message.message_type() != APPLY_FOR_CITY_WAR_MESSAGE_TYPE {
        return None;
    }

    let source_map_id = message.map_id();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let war_number = message.base_mut().get_long().unwrap_or(0);
    let legacy_third_parameter = message.base_mut().get_long().unwrap_or(0);
    let outcome = attack_city.on_player_declare_war(
        player_id,
        war_number,
        legacy_third_parameter,
        context,
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

/// Выполняет `0x60138`: четыре `Long` (war number, region, winner faction,
/// reported union) и `CAttackCitySys::OnFactionWinCity` с generic timer.
/// Country-мутации (флаг войны и governance-переход короля) идут внутри
/// context-адаптера через [`crate::app::world_game_view::WorldCountryWarGate`].
pub fn dispatch_city_war_result<Callback, Context>(
    message: &mut CMessage,
    attack_city: &mut CAttackCitySys,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    context: &mut Context,
) -> Option<OrganizingCityWarResultDispatch>
where
    Callback: Copy,
    Context: AttackCityWarResultContext<Block = OrganizingCityWarResultContextBlock> + ?Sized,
{
    if message.message_type() != CITY_WAR_RESULT_MESSAGE_TYPE {
        return None;
    }

    let war_number = message.base_mut().get_long().unwrap_or(0);
    let war_region_id = message.base_mut().get_long().unwrap_or(0);
    let winner_faction_id = message.base_mut().get_long().unwrap_or(0);
    let reported_union_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = attack_city.on_faction_win_city(
        war_number,
        war_region_id,
        winner_faction_id,
        reported_union_id,
        timer,
        attack_callbacks,
        context,
    );
    Some(OrganizingCityWarResultDispatch {
        war_number,
        war_region_id,
        winner_faction_id,
        reported_union_id,
        outcome,
    })
}

/// Семейный `CAttackCitySys::Reload` вне диспетчера: снимает прежние timer
/// events и завершает активные войны прежде чем `Initialize` читает новые
/// расписания, а `send_all` сохраняет `0x7FE22` между end и новым Initialize.
/// Ошибка последующего `Initialize` (`Load`) не откатывает уже выполненные
/// завершения — тот же договор, что у результата-ingress `0x60138`, поэтому
/// контекст общий; snapshot sender-а и сборку контекста выполняет обвязка
/// старого файла.
#[allow(
    clippy::too_many_arguments,
    reason = "source/time/timer/callbacks/tax/context/send сохраняют исходные owners CAttackCitySys::Reload"
)]
pub fn reload_attack_city<Callback, Context, SendAll>(
    attack_city: &mut CAttackCitySys,
    source: Option<&[u8]>,
    now: TagTime,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    today_tax_event_id: Option<TimerId>,
    context: &mut Context,
    send_all: SendAll,
) -> Result<AttackCityReloadReport, AttackCityReloadBlock<Context::Block>>
where
    Callback: Copy,
    Context: AttackCityWarEndContext + ?Sized,
    SendAll: FnMut(&CMessage) -> i32,
{
    attack_city.reload(
        source,
        now,
        timer,
        attack_callbacks,
        today_tax_event_id,
        context,
        send_all,
    )
}

/// Goods-war member контекст ветвей `0x60139`/`0x6013A`, целиком поверх
/// видов: организационный доступ — [`WorldOrganizingDispatchView`]
/// (`faction_by_id` и `set_faction_goods_war_count` волны организационного
/// view), игровой — [`WorldGameView`] (`current_game_server_sender`,
/// `configured_world_number`, `login_server_id`, `map_player`). В отличие от
/// прежнего адаптера старого файла он не держит конкретных заёмов owner-ов,
/// поэтому winner-снимок ветви `0x6013A` может читаться тем же view до
/// построения контекста без второго заёма.
struct WorldGoodsWarMemberDispatchContext<'a, Organizing>
where
    Organizing: WorldOrganizingDispatchView + ?Sized,
{
    game: &'a (dyn WorldGameView + 'a),
    organizing: &'a mut Organizing,
}

impl<Organizing> GoodsWarDeliveryContext for WorldGoodsWarMemberDispatchContext<'_, Organizing>
where
    Organizing: WorldOrganizingDispatchView + ?Sized,
{
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl<Organizing> GoodsWarMemberContext for WorldGoodsWarMemberDispatchContext<'_, Organizing>
where
    Organizing: WorldOrganizingDispatchView + ?Sized,
{
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

/// Выполняет `0x60139`: `operation Long` и команду `CGoodsWarMember`
/// (`2` delete member, `4` refresh all, `0x11` insert faction, `0x12` append
/// faction to count, прочее — игнор) с продолжением чтения operand Long на
/// исходных позициях.
pub fn dispatch_goods_war_command(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    goods_war: &mut CGoodsWarMember,
) -> Option<
    Result<OrganizingGoodsWarCommandDispatch, GoodsWarMemberBlock<OrganizingGoodsWarContextBlock>>,
> {
    if message.message_type() != GOODS_WAR_COMMAND_MESSAGE_TYPE {
        return None;
    }

    let operation = message.base_mut().get_long().unwrap_or(0);
    let mut context = WorldGoodsWarMemberDispatchContext { game, organizing };
    let outcome = match operation {
        2 => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            OrganizingGoodsWarCommandOutcome::DeleteOneMember {
                player_id,
                report: goods_war.delete_one_member(player_id, &mut context),
            }
        }
        4 => OrganizingGoodsWarCommandOutcome::RefreshAll(goods_war.refresh_all(&mut context)),
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

/// Выполняет `0x6013A`: `faction ID Long`, winner-снимок фракции общим
/// заёмом организационного view (до построения member-контекста — оба заёма
/// одного владельца последовательны и не пересекают границу вызова) и
/// `CGoodsWarMember::FactionWin`.
pub fn dispatch_goods_war_faction_win(
    message: &mut CMessage,
    game: &dyn WorldGameView,
    organizing: &mut impl WorldOrganizingDispatchView,
    goods_war: &mut CGoodsWarMember,
) -> Option<
    Result<OrganizingGoodsWarFactionWinDispatch, OrganizingGoodsWarFactionWinBlock>,
> {
    if message.message_type() != GOODS_WAR_FACTION_WIN_MESSAGE_TYPE {
        return None;
    }

    let requested_faction_id = message.base_mut().get_long().unwrap_or(0);
    let winner = {
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
        GoodsWarFactionWinSnapshot {
            faction_id: actual_faction_id,
            name: faction.name().to_vec(),
            master_id,
            member_ids: faction.get_members().keys().copied().collect(),
        }
    };
    let mut context = WorldGoodsWarMemberDispatchContext { game, organizing };
    let report = goods_war.faction_win(&winner, &mut context);
    Some(Ok(OrganizingGoodsWarFactionWinDispatch {
        requested_faction_id,
        faction_found: true,
        report: Some(report),
    }))
}
