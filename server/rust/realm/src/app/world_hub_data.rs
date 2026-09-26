//! Данные hub-уровня `CGame` WorldServer (Init/MainLoop) без самой игры:
//! сетевая конфигурация и отчёты string-table, init-callbacks,
//! маршрутизированное сообщение и события диспетча (`ProcessedWorldEvent` с
//! union terminal-семьёй), state-структуры и stage-отчёты MainLoop,
//! effect-контексты faction-war/attack-city за объявленными view-швами.
//! Источник контракта — та же точная пара, что у
//! [`crate::app::world_runtime`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`, RSDS совпадает).
//!
//! Effect-контексты не тянут `CGame`: игра входит generic-параметром `Game`
//! через объявленные швы [`WorldGameView`] и [`WorldPlayerFactionInfoUpdateView`]
//! (прецедент — `WorldFactionWarDeclarationEffects` в
//! [`crate::organizations::organizingctrl`]); старый пакет специализирует их
//! своим `CGame`. `WorldGameInitContext`/`WorldGameThreadRuntime` живут в
//! [`crate::app::world_init_context`]/[`crate::app::world_runtime`]
//! (объявленные швы вместо CGame-typed подписей), а route-контракты загрузки
//! игроков остаются у process-owner-а. Ветвь `0x60103` пишется в
//! `ProcessedWorldEvent` напрямую generic-формой realm
//! `OrganizingCreateFactionDispatchBlock<FactionCreationBlock>`.

use std::fmt;

use nebokrai_shared::runtime::put_string_to_file;
use nebokrai_shared::values::TagTime;

use crate::activities::attackcitysys::AttackCityEnemyRelationContext;
use crate::activities::factionwarsys::FactionWarStopContext;
use crate::app::auction::WorldServerAuctionMessageOutcome;
use crate::app::countrymessage::WorldCountryMessageOutcome;
use crate::app::gmamessage::WorldGmaMessageOutcome;
use crate::app::gmmessage::WorldGmMessageOutcome;
use crate::app::jjcsysmessage::JjcSystemMessageOutcome;
use crate::app::logmessage::WorldLogMessageOutcome;
use crate::app::onmsg_m2w_auction::WorldMiscAuctionMessageOutcome;
use crate::app::organsysmessage::{
    CityTransferConfirmationDelivery, ConfederationCreationConfirmationDelivery,
    OrganizingAdmissionPermitBlock, OrganizingAdmissionPermitDispatch,
    OrganizingAttackCityEndDispatch, OrganizingChangeRegionRouterDispatch,
    OrganizingCityGateBlock, OrganizingCityGateDispatch, OrganizingCityTransferDispatch,
    OrganizingCityWarApplicationBlock, OrganizingCityWarApplicationDispatch,
    OrganizingCityWarResultDispatch, OrganizingConsumedLongDispatch,
    OrganizingCreateFactionDispatch, OrganizingCreateFactionDispatchBlock,
    OrganizingDeclareFactionWarBlock, OrganizingDeclareFactionWarDispatch,
    OrganizingDeclareWarFactionListBlock, OrganizingDeclareWarFactionListDispatch,
    OrganizingFactionApplicationCancelBlock, OrganizingFactionApplicationCancelDispatch,
    OrganizingFactionApplicationDecisionDispatch, OrganizingFactionApplicationDispatch,
    OrganizingFactionApplicationDispatchBlock, OrganizingFactionBillboardBlock,
    OrganizingFactionBillboardOutcome, OrganizingFactionContributorDispatch,
    OrganizingFactionDemiseBlock, OrganizingFactionDemiseDispatch,
    OrganizingFactionDisbandBlock, OrganizingFactionDisbandDispatch,
    OrganizingFactionDubBlock, OrganizingFactionDubDispatch,
    OrganizingFactionExitBlock, OrganizingFactionExitDispatch,
    OrganizingFactionExperienceDispatch, OrganizingFactionFireOutBlock,
    OrganizingFactionFireOutDispatch, OrganizingFactionListBlock,
    OrganizingFactionListDispatch, OrganizingFactionMemberStateDispatch,
    OrganizingFactionParameterBlock, OrganizingFactionParameterDispatch,
    OrganizingFactionPurviewBlock, OrganizingFactionPurviewDispatch,
    OrganizingFactionTaxBlock, OrganizingFactionTaxDispatch,
    OrganizingFactionUpgradeBlock, OrganizingFactionUpgradeDispatch,
    OrganizingFactionUploadIconDispatch, OrganizingFactionWarPlayerDiedDispatch,
    OrganizingGoodsWarCommandDispatch, OrganizingGoodsWarContextBlock,
    OrganizingGoodsWarFactionWinBlock, OrganizingGoodsWarFactionWinDispatch,
    OrganizingInitialDataDispatch, OrganizingLeaveWordDispatch,
    OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEnableDispatch,
    OrganizingPlayerInviteFactionDispatch, OrganizingPlayerQuestCommandDispatch,
    OrganizingPlayerRunScriptDispatch, OrganizingPronounceDispatch,
    OrganizingRegionParamDispatch, OrganizingRegionRouteDispatch,
    OrganizingSessionResultDispatch, OrganizingUnionApplicationDispatch,
    OrganizingUnionDemiseDispatch, OrganizingUnionDisbandBlock,
    OrganizingUnionDisbandDispatch, OrganizingUnionExitDispatch,
    OrganizingUnionFireOutDispatch, OrganizingVillageWarApplicationBlock,
    OrganizingVillageWarApplicationDispatch, OrganizingVillageWarResultDispatch,
    QueuedCityTransferTerminal, QueuedConfederationCreationTerminal,
    QueuedUnionApplicationTerminal, QueuedUnionInvitationTerminal,
    UnionApplicationConfirmationDelivery,
};
use crate::app::playermessage::WorldPlayerMessageOutcome;
use crate::app::servermessage::{WorldLoginClientReplacement, WorldServerMessageOutcome};
use crate::app::teammessage::WorldTeamMessageOutcome;
use crate::app::world_game_view::{WorldGameView, WorldPlayerFactionInfoUpdateView};
use crate::app::world_hub_entries::WorldGameAiReport;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::world_runtime::{WorldGameInitAttackCityRelationBlock, WorldStringTableEncodingBlock};
use crate::app::worldserver::WorldLogLocalTime;
use crate::app::worldothermessage::WorldOtherMessageOutcome;
use crate::app::writelogmessage::WorldWriteLogMessageOutcome;
use crate::organizations::faction::{
    FactionEnemyMutationContext, FactionEnemyWarLogArgument, FactionExperienceBlock,
    FactionUploadIconBlock,
};
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;
use crate::organizations::goodswarmember::GoodsWarMemberBlock;
use crate::organizations::organizingctrl::{
    AttackCityEndBlock, COrganizingCtrl, CityTransferFinishBlock, CityTransferFinishReport,
    CityTransferSessionBlock, CityTransferSessionReport, CityTransferStartBlock,
    ConfederationCreationCallbackBlock, ConfederationCreationCallbackReport,
    ConfederationCreationSessionBlock, ConfederationCreationSessionReport, FactionCreationBlock,
    FactionUnionMembershipLookupBlock, FreeFactionLookup, OrganizingContributorBlock,
    OrganizingFactionDoJoinBlock, OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock,
    OrganizingLeaveWordEnableBlock, OrganizingPronounceBlock,
    OrganizingUnionApplicationCallbackBlock, OrganizingUnionApplicationCallbackReport,
    OrganizingUnionApplyForJoinDispatchBlock, OrganizingUnionDemiseBlock,
    OrganizingUnionExitBlock, OrganizingUnionFireOutBlock,
    OrganizingUnionInvitationCallbackBlock, OrganizingUnionInvitationCallbackReport,
    PlayerInviteFactionBlock,
};
use crate::organizations::union::{
    CityTransferEndpointBlock, ConfederationCreationEndpointBlock, UnionApplicationEndpointBlock,
    UnionApplicationSessionBlock, UnionApplicationSessionReport, UnionFormatArgument,
};
use crate::sessions::csessionfactory::WorldSessionFactoryAiReport;

#[derive(Clone, Copy)]
pub struct WorldNetworkConfig {
    pub ban_ip_time_ms: u32,
    pub maximum_client_send_buffer: i32,
    pub maximum_message_length: u32,
    pub maximum_byte_count: u32,
    pub check_message_content: bool,
    pub maximum_connections: i32,
    pub maximum_io_sends: i32,
    pub check_net: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldStringTableLoadReport {
    pub package: Vec<u8>,
    pub succeeded: bool,
    pub log_payload: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldStringTableUpdateCompletion {
    DefaultLanguageFailed,
    ConfiguredLanguageFailed,
    EncodingBlocked(WorldStringTableEncodingBlock),
    Empty,
    Broadcast {
        message_type: i32,
        payload_length: usize,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Полный результат reload-а; `requested_package` фиксирует игнорируемый
/// исходной функцией аргумент вместо того, чтобы молча приписать ему смысл.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldStringTableUpdateReport {
    pub requested_package: Vec<u8>,
    pub completion: WorldStringTableUpdateCompletion,
}

pub struct WorldGameInitCallbacks<'a> {
    pub get_tick: &'a mut dyn FnMut() -> u32,
    pub get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub put_log_info: &'a mut dyn FnMut(&[u8]),
}

/// Enemy-mutation формат и war-log проходят объявленным швом игры; текстовые
/// и id-аргументы исходно шли `%s`/`%u`-спецификаторами `UnionFormatArgument`.
pub struct WorldGameInitEnemyMutationEffects<'a, Game: ?Sized> {
    pub game: &'a Game,
    pub enemy_id: i32,
    pub enemy_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMainLoopFactionWarBlock {
    NullUnion { map_key: i32 },
    MissingFaction { faction_id: i32 },
    EnemyMutation {
        faction_id: i32,
        enemy_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

pub struct WorldMainLoopFactionWarEffects<'a, Game: ?Sized> {
    pub game: &'a Game,
    pub organizing: &'a mut COrganizingCtrl,
    pub players_to_update: Vec<i32>,
}

impl<Game: WorldGameView> FactionWarStopContext
    for WorldMainLoopFactionWarEffects<'_, Game>
{
    type Block = WorldMainLoopFactionWarBlock;

    fn faction_exists(&self, faction_id: i32) -> bool {
        self.organizing.faction_by_id(faction_id).is_some()
    }

    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block> {
        match self.organizing.is_free_faction(root_faction_id) {
            FreeFactionLookup::NoUnion => Ok(vec![root_faction_id]),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(WorldMainLoopFactionWarBlock::NullUnion { map_key })
            }
            FreeFactionLookup::Union(union_id) => Ok(self
                .organizing
                .confederation_by_id(union_id)
                .map(|union| union.member_ids_snapshot())
                .unwrap_or_default()),
        }
    }

    fn del_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(WorldMainLoopFactionWarBlock::MissingFaction {
                faction_id: enemy_id,
            })?;
        let faction = self
            .organizing
            .faction_by_id_mut(faction_id)
            .ok_or(WorldMainLoopFactionWarBlock::MissingFaction { faction_id })?;
        let mut effects = WorldGameInitEnemyMutationEffects {
            game: self.game,
            enemy_id,
            enemy_name,
        };
        faction
            .del_enemy_organizing(enemy_id, &mut effects)
            .map(|_| ())
            .map_err(|source| WorldMainLoopFactionWarBlock::EnemyMutation {
                faction_id,
                enemy_id,
                source,
            })
    }

    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let faction = self
            .organizing
            .faction_by_id_mut(faction_id)
            .ok_or(WorldMainLoopFactionWarBlock::MissingFaction { faction_id })?;
        let _ = faction.update_enemy_faction(self.game, |_view, faction, player_id| {
            let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
            self.players_to_update.push(player_id)
        });
        Ok(())
    }

    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(WorldMainLoopFactionWarBlock::MissingFaction { faction_id })
    }

    fn format_world_string(&mut self, string_id: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(argument))
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn send_orga_info_to_all(&mut self, info: &[u8], kind: u32, color: u32) {
        let _ = COrganizingCtrl::send_organizing_info_to_all(self.game, info, kind, color);
    }

    fn put_war_log(&mut self, info: &[u8]) {
        put_string_to_file("war", info);
    }
}

impl<Game: WorldGameView> FactionEnemyMutationContext
    for WorldGameInitEnemyMutationEffects<'_, Game>
{
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

pub struct WorldGameInitAttackCityContext<'a, Game: ?Sized> {
    pub game: &'a mut Game,
    pub organizing: &'a mut COrganizingCtrl,
}

impl<Game> AttackCityEnemyRelationContext for WorldGameInitAttackCityContext<'_, Game>
where
    Game: WorldPlayerFactionInfoUpdateView<OrganizingContext = COrganizingCtrl>,
{
    type Block = WorldGameInitAttackCityRelationBlock;

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
                WorldGameInitAttackCityRelationBlock::NullUnion { map_key }
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
            .ok_or(WorldGameInitAttackCityRelationBlock::MissingOrganizing {
                organizing_id: enemy_organizing_id,
            })?;
        let mut effects = WorldGameInitEnemyMutationEffects {
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
            .map_err(|source| WorldGameInitAttackCityRelationBlock::EnemyMutation {
                organizing_id,
                enemy_organizing_id,
                source,
            })?;
        if !found {
            return Err(WorldGameInitAttackCityRelationBlock::MissingOrganizing {
                organizing_id,
            });
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
        let mut players_to_refresh = Vec::new();
        let _ = self
            .organizing
            .update_all_city_enemy_faction_relations(self.game, &mut |player_id| {
                players_to_refresh.push(player_id);
            });
        for player_id in players_to_refresh {
            let _ = self
                .game
                .update_player_faction_info(self.organizing, player_id);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMessageSource {
    GameServer,
    LoginServer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMessageOwner {
    Server,
    Log,
    Gma,
    Player,
    Other,
    Gm,
    Team,
    OrganizingSystem,
    WriteLog,
    Country,
    ServerAuction,
    JjcSystem,
    MiscAuction,
}

pub struct RoutedWorldMessage {
    pub source: WorldMessageSource,
    pub message_type: i32,
    pub owner: Option<WorldMessageOwner>,
    pub legacy_run_result: i32,
    pub message: CMessage,
}

impl fmt::Debug for RoutedWorldMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoutedWorldMessage")
            .field("source", &self.source)
            .field("message_type", &self.message_type)
            .field("owner", &self.owner)
            .field("legacy_run_result", &self.legacy_run_result)
            .field("wire_length", &self.message.as_wire_bytes().len())
            .finish()
    }
}

pub type WorldUnionApplicationStartBlock =
    OrganizingUnionApplyForJoinDispatchBlock<UnionApplicationSessionBlock>;
pub type WorldPlayerInviteFactionDispatch = OrganizingPlayerInviteFactionDispatch<
    ConfederationCreationSessionReport,
    UnionApplicationSessionReport,
    UnionApplicationSessionReport,
>;
pub type WorldPlayerInviteFactionStartBlock = PlayerInviteFactionBlock<
    ConfederationCreationSessionBlock,
    UnionApplicationSessionBlock,
    UnionApplicationSessionBlock,
>;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldUnionApplicationTerminalDispatch {
    pub request: QueuedUnionApplicationTerminal,
    pub outcome: Result<
        OrganizingUnionApplicationCallbackReport,
        OrganizingUnionApplicationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldUnionInvitationTerminalDispatch {
    pub request: QueuedUnionInvitationTerminal,
    pub outcome: Result<
        OrganizingUnionInvitationCallbackReport,
        OrganizingUnionInvitationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCityTransferTerminalDispatch {
    pub request: QueuedCityTransferTerminal,
    pub outcome: Result<CityTransferFinishReport, CityTransferFinishBlock>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldConfederationCreationTerminalDispatch {
    pub request: QueuedConfederationCreationTerminal,
    pub outcome: Result<
        ConfederationCreationCallbackReport,
        ConfederationCreationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldUnionApplicationRuntimeReport {
    pub terminals: Vec<WorldUnionApplicationTerminalDispatch>,
    pub invitation_terminals: Vec<WorldUnionInvitationTerminalDispatch>,
    pub confirmations: Vec<UnionApplicationConfirmationDelivery>,
    pub endpoint_blocks: Vec<UnionApplicationEndpointBlock>,
    pub city_terminals: Vec<WorldCityTransferTerminalDispatch>,
    pub city_confirmations: Vec<CityTransferConfirmationDelivery>,
    pub city_endpoint_blocks: Vec<CityTransferEndpointBlock>,
    pub confederation_creation_terminals:
        Vec<WorldConfederationCreationTerminalDispatch>,
    pub confederation_creation_confirmations:
        Vec<ConfederationCreationConfirmationDelivery>,
    pub confederation_creation_endpoint_blocks:
        Vec<ConfederationCreationEndpointBlock>,
}

#[derive(Debug)]
pub enum ProcessedWorldEvent {
    Message(RoutedWorldMessage),
    ServerMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldServerMessageOutcome,
    },
    LogMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldLogMessageOutcome,
    },
    OtherMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldOtherMessageOutcome,
    },
    WriteLogMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldWriteLogMessageOutcome,
    },
    PlayerMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldPlayerMessageOutcome,
    },
    CountryMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldCountryMessageOutcome,
    },
    GmaMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldGmaMessageOutcome,
    },
    GmMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldGmMessageOutcome,
    },
    JjcMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: JjcSystemMessageOutcome,
    },
    MiscAuctionMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldMiscAuctionMessageOutcome,
    },
    ServerAuctionMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldServerAuctionMessageOutcome,
    },
    TeamMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldTeamMessageOutcome,
    },
    OrganizingSessionResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingSessionResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingConsumedLong {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingConsumedLongDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCreateFaction {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingCreateFactionDispatch,
            OrganizingCreateFactionDispatchBlock<FactionCreationBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionWarPlayerDied {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingFactionWarPlayerDiedDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingInitialData {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingInitialDataDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingDeclareWarFactionList {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingDeclareWarFactionListDispatch,
            OrganizingDeclareWarFactionListBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionList {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionListDispatch, OrganizingFactionListBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplicationCancel {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationCancelDispatch,
            OrganizingFactionApplicationCancelBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationDispatch<UnionApplicationSessionReport>,
            OrganizingFactionApplicationDispatchBlock<UnionApplicationSessionBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplicationDecision {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationDecisionDispatch,
            OrganizingFactionDoJoinBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionFireOut {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionFireOutDispatch, OrganizingFactionFireOutBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionExit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionExitDispatch, OrganizingFactionExitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionExit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDemise {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDemiseDispatch, OrganizingFactionDemiseBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionDemise {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDisband {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDisbandDispatch, OrganizingFactionDisbandBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionDisband {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDub {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDubDispatch, OrganizingFactionDubBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionPurview {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionPurviewDispatch, OrganizingFactionPurviewBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionFireOut {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingDeclareFactionWar {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingDeclareFactionWarDispatch,
            OrganizingDeclareFactionWarBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionBillboard {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionUpgrade {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionUpgradeDispatch, OrganizingFactionUpgradeBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionUploadIcon {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionContributor {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionExperience {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionMemberState {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingFactionMemberStateDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionTax {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionTaxDispatch, OrganizingFactionTaxBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingRegionParamUpdate {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingRegionParamDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingRegionRoute {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingRegionRouteDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityGate {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingCityGateDispatch, OrganizingCityGateBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityTransfer {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingCityTransferDispatch<CityTransferSessionReport>,
            CityTransferStartBlock<CityTransferSessionBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingAdmissionPermit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingAdmissionPermitDispatch, OrganizingAdmissionPermitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingAttackCityEnd {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingAttackCityEndDispatch, AttackCityEndBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingVillageWarApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingVillageWarApplicationDispatch,
            OrganizingVillageWarApplicationBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityWarApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingCityWarApplicationDispatch,
            OrganizingCityWarApplicationBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityWarResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingCityWarResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingGoodsWarCommand {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingGoodsWarCommandDispatch,
            GoodsWarMemberBlock<OrganizingGoodsWarContextBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingGoodsWarFactionWin {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingGoodsWarFactionWinDispatch,
            OrganizingGoodsWarFactionWinBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerQuestCommand {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingPlayerQuestCommandDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerRunScript {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingPlayerRunScriptDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionParameter {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingChangeRegionRouter {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingChangeRegionRouterDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingVillageWarResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingVillageWarResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingUnionApplicationDispatch<UnionApplicationSessionReport>,
            WorldUnionApplicationStartBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerInviteFaction {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            WorldPlayerInviteFactionDispatch,
            WorldPlayerInviteFactionStartBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWordEnable {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordEnableDispatch, OrganizingLeaveWordEnableBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWord {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordDispatch, OrganizingLeaveWordBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWordEdit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEditBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPronounce {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingPronounceDispatch, OrganizingPronounceBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
 /// default `OnOrgasysMessage`: неизвестный organizing opcode не имеет
 /// side effects, но уже накопленные terminal callbacks всё равно исполняются.
    OrganizingNoOp {
        source: WorldMessageSource,
        legacy_run_result: i32,
        request_type: i32,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    LoginClientReconnected(WorldLoginClientReplacement),
}

#[derive(Debug)]
pub struct WorldProcessMessageOutcome {
    pub legacy_result: i32,
    pub initial_server_events: i32,
    pub initial_login_messages: Option<i32>,
    pub server_slots_visited: i32,
    pub login_slots_visited: i32,
    pub events: Vec<ProcessedWorldEvent>,
    pub game_server_message_time_ms: u32,
    pub login_server_message_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldProcessMessageStageState {
    pub accumulated_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldMainLoopInitializationState {
    pub mask: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldMainLoopClockState {
    pub current_tick_ms: u32,
    pub last_refresh_tick_ms: u32,
    pub stage_started_at_ms: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldMainLoopTailClockState {
    pub current_tick_ms: u32,
    pub pacing_deadline_ms: u32,
    pub minute_started_at_ms: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldMainLoopLoginReleaseState {
    pub last_checked_at_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopTailClockInitialization {
    pub previous_mask: u32,
    pub initialized_mask: u32,
    pub initial_current_tick_ms: Option<u32>,
    pub initial_pacing_deadline_ms: Option<u32>,
    pub initial_minute_started_at_ms: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldMainLoopLargessState {
    pub pass_count: u32,
    pub last_start_request_tick_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopProfileState {
    pub last_published_at_ms: u32,
    pub ai_calls: u32,
    pub ai_time_ms: u32,
    pub refresh_text_time_ms: u32,
    pub net_session_time_ms: u32,
    pub faction_war_time_ms: u32,
    pub timer_time_ms: u32,
    pub process_player_data_queue_time_ms: u32,
    pub session_factory_time_ms: u32,
    pub save_point_time_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopAiStageReport {
    pub previous_stage_finished_at_ms: u32,
    pub save_point_elapsed_ms: u32,
    pub accumulated_save_point_time_ms: u32,
    pub ai_calls: u32,
    pub ai_started_at_ms: u32,
    pub ai: WorldGameAiReport,
    pub ai_finished_at_ms: u32,
    pub ai_elapsed_ms: u32,
    pub accumulated_ai_time_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopSessionFactoryStageReport {
    pub ai: WorldSessionFactoryAiReport,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub accumulated_time_ms: u32,
    pub next_stage_started_at_ms: u32,
}

/// Два подтверждённых порядка одного route: direct `GetPlayerData` публикует
/// player до friend-loop, а `ProcessPlayerDataQueue` — после него.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldLoadedPlayerRouteOrder {
    Direct,
    LoadedQueue,
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
