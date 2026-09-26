//! Диспетчер мировых сообщений `CGame` (`process_world_message`) и glue
//! между process-owner-ом и Realm-обработчиками ветвей.
//!
//! Async `process_world_message` снимает вход FIFO через `WorldOwnerSelector`
//! и передаёт ветви realm-диспетчерам; country/organizing effect-glue и
//! адаптеры `WorldUnionApplicationEffect*`/`World*Effects`/`Bridge` собирают
//! те же исходные владельцы. DeleteRole и CreateRole мосты замыкают ветви,
//! чья середина остаётся inherent-логикой игры.
//!
//! Drain-каркас тела `process_message` (`1:00a30`) — VERIFIED полным
//! машинным разбором точной пары: снятие обеих FIFO, virtual execute/delete
//! записей и учёт `g_l*MessageTime` (корпус `.local/verify-c5c/`). Ветви
//! обработчиков World-направления живут у своих модулей и остаются PARTIAL
//! (см. `crate::app::world_main_loop`); сам диспетчер разбирает
//! опубликованные исходы без новых gate-ев.
//!
//! Имена и проекции типов здесь — Realm/Shared формы (см.
//! `crate::app::world_game`); impl-ы швов для `CGame` допустимы только в
//! crate типа (orphan-правило).

use crate::activities::attackcitysys::{AttackCityApplicationContext, AttackCityCallbacks, AttackCityEnemyRelationContext, AttackCityReloadBlock, AttackCityReloadReport, AttackCityWarEndContext, AttackCityWarResultContext, AttackCityWarResultFaction, AttackCityWarResultFormatArgument, AttackCityWarResultRegion, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarDeclarationAuthority, CountryWarDeclarationContext, CountryWarDeclarationPlayer, CountryWarPhaseContext, CountryWarSys, CountryWarTopInfoContext, CountryWarVictoryContext, CountryWarVictoryRegion};
use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationCountryFailContext, FourNationWarResultContext};
use crate::activities::jjcsystem::{CJJcSystem, JjcRunContext};
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarApplicationContext, VillageWarCallbacks, VillageWarResultContext, VillageWarResultFaction, VillageWarResultRegion};
use crate::app::auction::{WorldServerAuctionMessageDispatch, on_msg_s2w_auction};
use crate::app::countrymessage::{WorldCountryAbsolveRequestSync, WorldCountryAppointMinisterSync, WorldCountryDemiseSync, WorldCountryDeposeMinisterSync, WorldCountryDirectAppointmentSync, WorldCountryExileRequestSync, WorldCountryExileResultSync, WorldCountryGovernanceGate, WorldCountryInfoSync, WorldCountryMessageDispatch, WorldCountryMessageOutcome, WorldCountryNewDaySync, WorldCountryPlayerChangeSync, WorldCountryPlayerChangeView, WorldCountryPlayersListSync, WorldCountrySilenceRequestSync, dispatch_country_war_declaration_message, dispatch_country_war_victory_message, dispatch_four_nation_country_fail_message, dispatch_four_nation_exploit_message, dispatch_four_nation_war_result_message, dispatch_four_nation_war_time_message};
use crate::app::gmamessage::{WorldGmaMessageDispatch, on_gma_message};
use crate::app::gmmessage::{WorldGmMessageDispatch, on_gm_message};
use crate::app::jjcsysmessage::on_jjc_system_message;
use crate::app::logmessage::{WorldLogMessageDispatch, on_log_message};
use crate::app::misc_game::legacy_tick_ms;
use crate::app::onmsg_m2w_auction::{WorldMiscAuctionMessageDispatch, on_msg_m2w_auction};
use crate::app::organsysmessage::{FactionApplicationListContext, OrganizingChangeRegionRouterDispatch, OrganizingCityTransferDispatch, OrganizingCityWarApplicationBlock, OrganizingCityWarApplicationDispatch, OrganizingCityWarResultContextBlock, OrganizingCityWarResultDispatch, OrganizingConsumedLongDispatch, OrganizingCreateFactionDispatch, OrganizingDeclareFactionWarBlock, OrganizingDeclareFactionWarDispatch, OrganizingFactionApplicationDecisionDispatch, OrganizingFactionApplicationDispatch, OrganizingFactionApplicationDispatchBlock, OrganizingFactionBillboardBlock, OrganizingFactionBillboardOutcome, OrganizingFactionContributorDispatch, OrganizingFactionDemiseBlock, OrganizingFactionDemiseDispatch, OrganizingFactionDubBlock, OrganizingFactionDubDispatch, OrganizingFactionExitBlock, OrganizingFactionExitDispatch, OrganizingFactionExperienceDispatch, OrganizingFactionFireOutBlock, OrganizingFactionFireOutDispatch, OrganizingFactionParameterBlock, OrganizingFactionParameterDispatch, OrganizingFactionPurviewBlock, OrganizingFactionPurviewDispatch, OrganizingFactionUpgradeBlock, OrganizingFactionUpgradeDispatch, OrganizingFactionUploadIconDispatch, OrganizingFactionWarPlayerDiedDispatch, OrganizingInitialDataDispatch, OrganizingPlayerInviteFactionDispatch, OrganizingPlayerQuestCommandDispatch, OrganizingPlayerRunScriptDispatch, OrganizingRegionRouteDispatch, OrganizingSessionResultDispatch, OrganizingUnionApplicationDispatch, OrganizingUnionDemiseDispatch, OrganizingUnionDisbandBlock, OrganizingUnionDisbandDispatch, OrganizingUnionExitDispatch, OrganizingUnionFireOutDispatch, OrganizingVillageWarApplicationBlock, OrganizingVillageWarApplicationDispatch, OrganizingVillageWarResultContextBlock, OrganizingVillageWarResultDispatch, QueuedOrganizingSessionTerminal, dispatch_admission_permit, dispatch_attack_city_end, dispatch_city_gate, dispatch_declare_war_faction_list, dispatch_faction_application_cancel, dispatch_faction_disband, dispatch_faction_list, dispatch_faction_member_state, dispatch_faction_tax, dispatch_goods_war_command, dispatch_goods_war_faction_win, dispatch_leave_word, dispatch_leave_word_edit, dispatch_leave_word_enable, dispatch_pronounce, dispatch_region_param_update, finalize_faction_disband_dispatch};
use crate::app::playermessage::{WorldPlayerMessageDispatch, on_player_message};
use crate::app::servermessage::{WorldInitialConfigurationRunCompletion, WorldServerMessageDispatch, WorldServerMessageOutcome, on_server_message};
use crate::app::teammessage::on_team_message;
use crate::app::world_game::{CGame, WorldCompletedSaveResponseMaterializationAdapter};
use crate::app::world_game_view::{WorldCountryKingGateBlock, WorldCountryMutGate, WorldCountryView, WorldCountryWarGate, WorldDeleteRoleCountryGate, WorldRegionNameLookup};
use crate::app::world_hub_data::{ProcessedWorldEvent, RoutedWorldMessage, WorldCityTransferTerminalDispatch, WorldConfederationCreationTerminalDispatch, WorldMessageOwner, WorldMessageSource, WorldUnionApplicationRuntimeReport, WorldUnionApplicationTerminalDispatch, WorldUnionInvitationTerminalDispatch};
use crate::app::world_message::{CMessage, SendMessageError, WorldMessageHandlers};
use crate::app::world_reload_profiles::WorldMainLoopResourceContext;
use crate::app::worldothermessage::{WorldOtherMessageDispatch, on_other_message};
use crate::app::worldserver::{AddLogTextDisposition, WorldSaveThreadHandleState};
use crate::app::writelogmessage::{WorldWriteLogMessageDispatch, on_write_log_message};
use crate::auction::auctionlog::CAuctionLog;
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerCodecError, PlayerCountryChangeReport, PlayerFactionInfoUpdateBlock, PlayerFactionInfoUpdateReport, PlayerPropertyCoefficients};
use crate::characters::playerranks::CPlayerRanks;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::cgoodsfactory::{GoodsOriginalNameIndex, query_goods_name};
use crate::content::countryparam::{CCountryParam, CountryParameterUnavailable};
use crate::content::organizing::TagTimeValue;
use crate::content::skillfactory::CSkillFactory;
use crate::content::variablelist::CVariableList;
use crate::organizations::country::{CountryAbsolveCounterReset, CountryAbsolveReport, CountryAppointMinisterReport, CountryBaseInfoDisposition, CountryCanAbsolveDisposition, CountryCanAppointMinisterDisposition, CountryCanDemiseDisposition, CountryCanDeposeMinisterDisposition, CountryCanExileDisposition, CountryCanSilenceDisposition, CountryDemiseReport, CountryDeposeMinisterReport, CountryExileMessageDelivery, CountryExileRequestDisposition, CountryExileResultContext, CountryExileTarget, CountryExileTextArgument, CountryExileTimeLookup, CountryFactionSnapshot, CountryGovernanceContextBlock, CountryHasJobContext, CountryInitialKingReport, CountryKingSaveLimits, CountryNewTermContext, CountryOnlinePlayer, CountryPlayersListContext, CountryPlayersListContextBlock, CountryPlayersListReport, CountryQuestSwitchUpdate, CountryScalarUpdate, CountrySetKingReport, CountrySetNewDayContext, CountrySilenceReport, CountrySuccessExiledReport, CountryVillageTaxContext, CountryVillageTaxContextBlock, CountryVillageTaxRegion, KingPointUpdate};
use crate::organizations::countryhandler::{CCountryHandler, CountryHandlerNewDayReport, CountryInfoDeliveryContext};
use crate::organizations::faction::{CFaction, FactionApplyForJoinEffects, FactionContributorContext, FactionDemiseContext, FactionDemiseOutcome, FactionDisbandContext, FactionDoJoinEffects, FactionDubContext, FactionDubFormatArgument, FactionEnemyMutationContext, FactionEnemyWarLogArgument, FactionExitContext, FactionExperienceBlock, FactionFireOutContext, FactionLevelContext, FactionMemberInfoRequest, FactionOrganizingInfoContext, FactionPurviewChange, FactionPurviewChangeContext, FactionSetParameterContext, FactionUpgradeBlock, FactionUpgradeContext, FactionUpgradeFormatArgument, FactionUpgradeOutcome, FactionUploadIconBlock, FactionUploadIconContext, goods_war_check_for_faction_id};
use crate::organizations::goodswarmember::{CGoodsWarMember, GoodsWarDeliveryContext};
use crate::organizations::king::set_control_point;
use crate::organizations::organizingctrl::{AllFactionInfoClientBlock, AttackCityEndEffects, COrganizingCtrl, CityTransferEffects, CityTransferSessionBlock, CityTransferSessionReport, CityTransferSessionRequest, CityTransferStartBlock, CityTransferStartOutcome, ConfederationCreationEffects, ConfederationCreationSessionBlock, ConfederationCreationSessionReport, ConfederationCreationSessionRequest, FactionClientSnapshotBlock, FactionCreationBlock, FactionCreationEffects, FactionCreationOutcome, FactionCreationPreparation, FactionUnionMembershipLookupBlock, FreeFactionLookup, FreePlayerLookup, OrganizingContributorBlock, OrganizingFactionDoJoinBlock, OrganizingInfoDelivery, OrganizingUnionApplyForJoinDispatchBlock, OrganizingUnionApplyForJoinOutcome, OrganizingUnionDemiseBlock, OrganizingUnionExitBlock, OrganizingUnionFireOutBlock, PlayerInviteFactionBlock, PlayerInviteFactionEffects, PlayerInviteFactionOutcome, UnionClientSnapshotByPlayerBlock, UnionClientSnapshotByPlayerOutcome, UnionOrganizingBridge, WorldFactionWarDeclarationEffects, begin_city_transfer_session, begin_confederation_creation_session};
use crate::organizations::organizingparam::COrganizingParam;
use crate::organizations::union::{UnionAddFactionEffects, UnionApplicationSessionBlock, UnionApplicationSessionReport, UnionApplicationSessionRequest, UnionApplyForJoinEffects, UnionFactionStateMutationContext, UnionFireOutEffects, UnionFormatArgument, UnionInvitationSessionRequest, UnionInviteEffects, UnionOwnedCityMutationContext, begin_union_application_session, begin_union_invitation_session};
use crate::persistence::dbmisc::{CDbMisc, DbMiscContext};
use crate::persistence::rsplayer::TiberiusRsPlayer;
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::savedb::SaveDataLifecycleState;
use crate::persistence::saveworker::WorldSaveRuntimeContext;
use crate::sessions::csessionfactory::CSessionFactory;
use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{CGodsBattleConf, GlobeSetupSnapshot, RegionRouter};
use nebokrai_shared::runtime::{CNetSessionManager, CTimer, TimerId, put_string_to_file};
use nebokrai_shared::values::TagTime;
use parking_lot::Mutex;
use std::convert::Infallible;
use std::ffi::CString;
use std::sync::Arc;
use crate::app::organsysmessage::WorldOrganizingSessionRuntimeOwner as WorldUnionApplicationRuntimeOwner;
use crate::app::servermessage as servermessage;
use crate::activities::jjcsystem::GlobeSetupJjcWorldConfig;
use crate::persistence::rsplayer::RsPlayerOwner;

pub(crate) struct WorldCountryInfoDelivery<'a> {
    pub(crate) game: &'a CGame,
}

impl CountryInfoDeliveryContext for WorldCountryInfoDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

pub(crate) struct WorldCountryWarEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) country_handler: &'a mut CCountryHandler,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
}

pub(crate) struct WorldCountryExileResultEffects<'a> {
    pub(crate) game: &'a mut CGame,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
}

impl WorldCountryExileResultEffects<'_> {
    pub(crate) fn format_world_string(
        &mut self,
        string_id: &[u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        self.game.format_world_string(string_id, arguments)
    }
}

pub(crate) struct WorldCountryPlayersListEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) organizing: &'a COrganizingCtrl,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
}

pub(crate) struct WorldCountryDemiseEffects<'a> {
    pub(crate) base: WorldCountryExileResultEffects<'a>,
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) organizing_parameters: &'a COrganizingParam,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) goods_war: &'a CGoodsWarMember,
    pub(crate) refresh_owned_city: &'a mut dyn FnMut(&CGame, i32, i32, i32, Option<u8>),
    pub(crate) update_player: &'a mut dyn FnMut(i32),
    pub(crate) faction_master_log_enabled: bool,
    pub(crate) write_faction_master_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

pub(crate) struct WorldCountryFactionDemiseEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) goods_war: &'a CGoodsWarMember,
    pub(crate) update_player: &'a mut dyn FnMut(i32),
    pub(crate) country_id: u8,
    pub(crate) king_id: i32,
    pub(crate) demise_faction: bool,
    pub(crate) faction_master_log_enabled: bool,
    pub(crate) write_faction_master_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

pub(crate) struct WorldOrganizingDisbandEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) village_war: &'a CVillageWarSys,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) country_handler: &'a CCountryHandler,
    pub(crate) goods_war: &'a mut CGoodsWarMember,
}

/// Отдельный immutable transport-view устраняет искусственную зависимость
/// Goods War delete-публикаций от mutable organizing lookup-а.
pub(crate) struct WorldGoodsWarDelivery<'a> {
    pub(crate) game: &'a CGame,
}

/// Bridge-адаптер [`WorldCountryWarGate`] ветви `0x60138` city war result и
/// семейного `reload_attack_city`: лёгкие country-контакты делегирует
/// inherent-вызовам живого `CCountryHandler`, тяжёлый governance-переход
/// выполняет связно `take_country_owner → SetKing → (success) city_id →
/// restore_country_owner` в исходном порядке с параметрами страны владельца.
pub(crate) struct CityWarCountryGateBridge<'a> {
    pub(crate) country_handler: &'a mut CCountryHandler,
    pub(crate) country_parameters: &'a CCountryParam,
}

impl WorldCountryWarGate for CityWarCountryGateBridge<'_> {
    fn country_exists(&self, country: u8) -> bool {
        self.country_handler.get_country(country).is_some()
    }

    fn set_country_warring(&mut self, country: u8, warring: bool) -> bool {
        let Some(country_state) = self.country_handler.get_country_mut(country) else {
            return false;
        };
        country_state.is_warring = warring;
        true
    }

    fn set_country_king_and_city(
        &mut self,
        country: u8,
        master_id: i32,
        city_region_id: i32,
        context: &mut dyn CountryExileResultContext,
    ) -> Result<(), WorldCountryKingGateBlock> {
        let Some(mut country_state) = self.country_handler.take_country_owner(country) else {
            return Err(WorldCountryKingGateBlock::MissingCountryOwner);
        };
        let set_king = country_state.set_king(master_id, self.country_parameters, context);
        if set_king.is_ok() {
            country_state.city_id = city_region_id;
        }
        self.country_handler
            .restore_country_owner(country, country_state);
        set_king
            .map(|_| ())
            .map_err(WorldCountryKingGateBlock::Governance)
    }
}

pub(crate) struct WorldFourNationWarResultEffects<'a> {
    pub(crate) game: &'a CGame,
}

pub(crate) struct WorldFourNationCountryFailEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) organizing: &'a COrganizingCtrl,
}

impl CountryNewTermContext for WorldCountryExileResultEffects<'_> {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }
}

impl CountryVillageTaxContext for WorldCountryExileResultEffects<'_> {
    fn village_regions(
        &mut self,
        country_id: u8,
    ) -> Result<Vec<CountryVillageTaxRegion>, CountryVillageTaxContextBlock> {
        let mut regions = Vec::new();
        for (&map_key, assignment) in &self.game.region_registry.regions {
            let Some(region_type) = assignment.region_type else {
                return Err(CountryVillageTaxContextBlock::UninitializedRegionType { map_key });
            };
            if region_type != 1 {
                continue;
            }
            let Some(owner) = assignment.region.as_ref() else {
                continue;
            };
            let Some(region_country) = owner.base().region_base().country() else {
                return Err(CountryVillageTaxContextBlock::UninitializedRegionCountry { map_key });
            };
            if region_country == country_id {
                regions.push(CountryVillageTaxRegion {
                    map_key,
                    name: legacy_c_string_prefix(owner.base().get_name()).to_vec(),
                });
            }
        }
        Ok(regions)
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
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
        self.format_world_string(string_id, &arguments)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl CountryNewTermContext for WorldCountryDemiseEffects<'_> {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        CountryNewTermContext::send_all(&mut self.base, message)
    }
}

impl CountryVillageTaxContext for WorldCountryDemiseEffects<'_> {
    fn village_regions(
        &mut self,
        country_id: u8,
    ) -> Result<Vec<CountryVillageTaxRegion>, CountryVillageTaxContextBlock> {
        CountryVillageTaxContext::village_regions(&mut self.base, country_id)
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        CountryVillageTaxContext::country_name(&mut self.base, country_id)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        CountryVillageTaxContext::format_world_string(
            &mut self.base,
            string_id,
            arguments,
        )
    }

    fn put_king_log(&mut self, text: &[u8]) {
        CountryVillageTaxContext::put_king_log(&mut self.base, text);
    }
}

impl FourNationWarResultContext for WorldFourNationWarResultEffects<'_> {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> i32 {
        self.game.game_server_number_by_region_id(region_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }
}

impl CountryExileResultContext for WorldCountryExileResultEffects<'_> {
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
        self.format_world_string(string_id, &arguments)
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

impl CountryPlayersListContext for WorldCountryPlayersListEffects<'_> {
    fn online_players(&mut self) -> Vec<CountryOnlinePlayer> {
        self.game
            .player_registry
            .online_players
            .iter()
            .filter_map(|&player_id| self.game.map_player(player_id))
            .map(|player| CountryOnlinePlayer {
                id: player.get_id(),
                name: legacy_c_string_prefix(player.get_name()).to_vec(),
                country: player.country(),
                occupation: player.get_occupation(),
                level: player.get_level(),
                is_god: player.is_god(),
            })
            .collect()
    }

    fn player_faction(
        &mut self,
        player_id: i32,
    ) -> Result<(Vec<u8>, bool), CountryPlayersListContextBlock> {
        let faction_id = match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => 0,
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { .. } => {
                return Err(CountryPlayersListContextBlock::PlayerFactionLookup);
            }
        };
        let faction_name = if faction_id > 0 {
            self.organizing
                .faction_by_id(faction_id)
                .map(|faction| faction.name().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let is_faction_master = self
            .organizing
            .faction_id_by_master_player(player_id)
            .map_err(|_| CountryPlayersListContextBlock::FactionMasterLookup)?
            != 0;
        Ok((faction_name, is_faction_master))
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
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

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl FactionOrganizingInfoContext for WorldCountryFactionDemiseEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl GoodsWarDeliveryContext for WorldGoodsWarDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl FactionOrganizingInfoContext for WorldOrganizingDisbandEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDisbandContext for WorldOrganizingDisbandEffects<'_> {
    fn village_war_declared(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn city_war_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_disband(&self, faction_id: i32, _player_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_king_id(&self, country: u8) -> Option<i32> {
        self.country_handler
            .get_country(country)
            .map(|country| country.king.id)
    }

    fn delete_goods_war_members_by_faction_id(&mut self, faction_id: i32) {
        let mut delivery = WorldGoodsWarDelivery { game: self.game };
        let _ = self
            .goods_war
            .delete_members_by_faction_id(faction_id, &mut delivery);
    }

    fn decrement_goods_war_faction_count(&mut self, _faction_id: i32, faction_name: &[u8]) {
        let mut delivery = WorldGoodsWarDelivery { game: self.game };
        let _ = self
            .goods_war
            .delete_one_faction_count_by_name(faction_name, &mut delivery);
    }
}

impl FactionDemiseContext for WorldCountryFactionDemiseEffects<'_> {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_demise(&self, faction_id: i32, _old_master_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool {
        country == self.country_id
            && old_master_id == self.king_id
            && !self.demise_faction
    }

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        self.game
            .format_world_string(string_id, &[UnionFormatArgument::Signed(value)])
    }

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8> {
        self.game.format_world_string(
            string_id,
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

impl CountryExileResultContext for WorldCountryDemiseEffects<'_> {
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>> {
        self.base.map_player_name(player_id)
    }

    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget> {
        self.base.online_player(player_id)
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset> {
        self.base.reset_online_player_murder_counters(player_id)
    }

    fn faction_id_by_master_player(
        &mut self,
        player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(|_| CountryGovernanceContextBlock::FactionMasterLookup)
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
        self.organizing.faction_by_id(faction_id).map(|faction| CountryFactionSnapshot {
            faction_id: faction.faction_id(),
            name: faction.name().to_vec(),
            owned_cities: faction.owned_cities().iter().copied().collect(),
        })
    }

    fn union_id_for_faction(
        &mut self,
        faction_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { .. } => {
                Err(CountryGovernanceContextBlock::UnionLookup)
            }
        }
    }

    fn clear_faction_owned_cities(
        &mut self,
        faction_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        self.organizing
            .faction_by_id_mut(faction_id)
            .ok_or(CountryGovernanceContextBlock::OwnedCityMutation)?
            .clear_owned_cities(
                &*self.base.game,
                |_view, faction, player_id| {
                    let _ = self
                        .base
                        .game
                        .update_player_faction_info_from_faction(faction, player_id);
                    (self.update_player)(player_id);
                },
            )
            .map(|_| ())
            .map_err(|_| CountryGovernanceContextBlock::OwnedCityMutation)
    }

    fn add_faction_owned_city(
        &mut self,
        faction_id: i32,
        city_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        self.organizing
            .faction_by_id_mut(faction_id)
            .ok_or(CountryGovernanceContextBlock::OwnedCityMutation)?
            .add_owned_city(
                &*self.base.game,
                city_id,
                |_view, faction, player_id| {
                    let _ = self
                        .base
                        .game
                        .update_player_faction_info_from_faction(faction, player_id);
                    (self.update_player)(player_id);
                },
            )
            .map(|_| ())
            .map_err(|_| CountryGovernanceContextBlock::OwnedCityMutation)
    }

    fn refresh_owned_city(
        &mut self,
        city_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.refresh_owned_city)(
            self.base.game,
            city_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn demise_faction(
        &mut self,
        faction_id: i32,
        old_master_id: i32,
        new_master_id: i32,
        country_id: u8,
        king_id: i32,
        demise_faction: bool,
    ) -> Result<bool, CountryGovernanceContextBlock> {
        let Some(faction) = self.organizing.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        let game = &*self.base.game;
        let mut effects = WorldCountryFactionDemiseEffects {
            game,
            attack_city: self.attack_city,
            goods_war: self.goods_war,
            update_player: &mut *self.update_player,
            country_id,
            king_id,
            demise_faction,
            faction_master_log_enabled: self.faction_master_log_enabled,
            write_faction_master_log: &mut *self.write_faction_master_log,
        };
        faction
            .demise(
                game,
                self.organizing_parameters,
                old_master_id,
                new_master_id,
                &mut effects,
            )
            .map(|outcome| matches!(outcome, FactionDemiseOutcome::Transferred(_)))
            .map_err(|_| CountryGovernanceContextBlock::FactionDemise)
    }

    fn current_tick_ms(&mut self) -> u32 {
        legacy_tick_ms()
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        CountryExileResultContext::country_name(&mut self.base, country_id)
    }

    fn country_identity_name(&mut self, identity: u8) -> Vec<u8> {
        self.base.country_identity_name(identity)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        CountryExileResultContext::format_world_string(
            &mut self.base,
            string_id,
            arguments,
        )
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.base.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        self.base.send_to_map_id(message, map_id)
    }

    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery> {
        self.base.send_to_connected_game_servers(message)
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        CountryExileResultContext::send_all(&mut self.base, message)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        CountryExileResultContext::put_king_log(&mut self.base, text);
    }
}

impl FourNationCountryFailContext for WorldFourNationCountryFailEffects<'_> {
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(argument))
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn send_top_info(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        self.organizing
            .send_top_info_to_client(self.game, -1, 1, 2, text)
    }
}

impl CountryWarDeclarationContext for WorldCountryWarEffects<'_> {
    fn online_player_country(&mut self, player_id: i32) -> CountryWarDeclarationPlayer {
        let Some(player) = self.game.online_player_by_id(player_id as u32) else {
            return CountryWarDeclarationPlayer::Missing;
        };
        match player.country() {
            Some(country) => CountryWarDeclarationPlayer::Country(country),
            None => CountryWarDeclarationPlayer::CountryUnavailable,
        }
    }

    fn declaration_authority(
        &mut self,
        country: u8,
        player_id: i32,
    ) -> CountryWarDeclarationAuthority {
        let Some(owner) = self.country_handler.get_country(country) else {
            return CountryWarDeclarationAuthority::CountryMissing;
        };
        let is_king = owner.has_king_id(player_id);
        let is_minister = owner.has_minister_id(player_id);
        let country_name = self
            .globe_setup
            .country_name(country)
            .unwrap_or_default()
            .to_vec();

        if is_king {
            return CountryWarDeclarationAuthority::Authorized;
        }
        let king_log = format_union_world_string(
            self.game.get_string_by_id(b"WS0034"),
            &[UnionFormatArgument::Text(&country_name)],
        );
        let king_log = legacy_c_string_prefix(&king_log);
        put_string_to_file("king", &king_log[..king_log.len().min(0x103)]);

        if is_minister {
            return CountryWarDeclarationAuthority::Authorized;
        }
        let identity_name = self
            .globe_setup
            .country_identity_name(5)
            .unwrap_or_default();
        let minister_log = format_union_world_string(
            self.game.get_string_by_id(b"WS0037"),
            &[
                UnionFormatArgument::Text(&country_name),
                UnionFormatArgument::Text(identity_name),
            ],
        );
        let minister_log = legacy_c_string_prefix(&minister_log);
        put_string_to_file(
            "king",
            &minister_log[..minister_log.len().min(0x103)],
        );
        CountryWarDeclarationAuthority::Rejected
    }

    fn region(&mut self, region_id: i32) -> Option<CountryWarVictoryRegion> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(CountryWarVictoryRegion {
                name: name.to_vec(),
            }),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
        }
    }

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn format_declaration_notice(
        &mut self,
        attack_country: u8,
        defend_country: i32,
        region_name: &[u8],
    ) -> Vec<u8> {
        let attack_name = self
            .globe_setup
            .country_name(attack_country)
            .unwrap_or_default();
        let defend_name = u8::try_from(defend_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let notice = format_union_world_string(
            self.game.get_string_by_id(b"WS0104"),
            &[
                UnionFormatArgument::Text(attack_name),
                UnionFormatArgument::Text(defend_name),
                UnionFormatArgument::Text(region_name),
            ],
        );
        let notice = legacy_c_string_prefix(&notice);
        notice[..notice.len().min(0x1ff)].to_vec()
    }

    fn send_private_to_country_king(
        &mut self,
        country: u8,
        text: &[u8],
    ) -> Option<Result<i32, SendMessageError>> {
        let text = legacy_c_string_prefix(text);
        if text.is_empty() {
            return None;
        }
        let king_id = self.country_handler.get_country(country)?.king.id;
        let map_id = self.game.game_server_number_by_player_id(king_id);
        if map_id == 0 {
            return None;
        }
        let text = CString::new(text).expect("legacy C-string prefix не содержит NUL");
        let mut message = CMessage::new(0x7ff13);
        message.base_mut().add_long(king_id);
        message.base_mut().add_str(Some(&text));
        Some(message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id))
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn send_country_info(&mut self, text: &[u8], title: u32, color: u32) -> i32 {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        self.country_handler
            .send_info_to_client(&text, title, color, &mut delivery)
    }
}

impl CountryWarVictoryContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<CountryWarVictoryRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(CountryWarVictoryRegion {
                name: name.to_vec(),
            }),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
        })
    }

    fn country_exists(&mut self, country: u8) -> Result<bool, Self::Block> {
        Ok(self.country_handler.get_country(country).is_some())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }

    fn set_country_war_result(
        &mut self,
        country: u8,
        result: i32,
    ) -> Result<(), Self::Block> {
        let _ = self
            .country_handler
            .set_country_war_result(country, result);
        Ok(())
    }

    fn format_victory_notice(
        &mut self,
        string_id: &'static [u8],
        attack_country: i32,
        defend_country: i32,
        region_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let attack_name = u8::try_from(attack_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let defend_name = u8::try_from(defend_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let formatted = format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[
                UnionFormatArgument::Text(attack_name),
                UnionFormatArgument::Text(defend_name),
                UnionFormatArgument::Text(region_name),
            ],
        );
        let visible = legacy_c_string_prefix(&formatted);
        // Нормальный output сохраняется byte-; переполнение старого
        // 256-byte `_sprintf` было внутренним UB, поэтому safe adapter
        // оставляет место под C-string NUL вместо чтения за stack-buffer.
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block> {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        Ok(self
            .country_handler
            .send_info_to_client(&text, title, color, &mut delivery))
    }
}

impl CountryWarPhaseContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn reset_country_war_result_if_present(
        &mut self,
        country: u8,
    ) -> Result<bool, Self::Block> {
        Ok(self.country_handler.set_country_war_result(country, 0))
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn format_phase_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let formatted = format_union_world_string(self.game.get_string_by_id(string_id), &[]);
        let visible = legacy_c_string_prefix(&formatted);
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block> {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        Ok(self
            .country_handler
            .send_info_to_client(&text, title, color, &mut delivery))
    }
}

impl CountryWarTopInfoContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn format_top_info_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let formatted = format_union_world_string(self.game.get_string_by_id(string_id), &[]);
        let visible = legacy_c_string_prefix(&formatted);
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn add_top_info(
        &mut self,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
        get_tick: &mut dyn FnMut() -> u32,
    ) -> i32 {
        self.country_handler
            .add_one_top_info(timer_flag, duration_ms, text, get_tick)
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
    ) -> i32 {
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        self.country_handler.send_top_info_to_client(
            top_info_id,
            timer_flag,
            duration_ms,
            text,
            &mut delivery,
        )
    }
}

/// Контекст форматирования строк удаляемой должности ветви delete-role:
/// связка `globe_setup` + game-view прежнего `DeleteRoleCountryEffects` из
/// dispatcher-адаптера, перенесена вместе с gate-мостом к единственному
/// callsite-у Log-диспетчера.
pub(crate) struct DeleteRoleCountryEffects<'a> {
    pub(crate) game: &'a (dyn crate::app::world_game_view::WorldGameView + 'a),
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
}

impl CountryHasJobContext for DeleteRoleCountryEffects<'_> {
    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
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
                CountryExileTextArgument::Text(value) => UnionFormatArgument::Text(value),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

/// Адаптер странового gate ветви delete-role: повторяет исходную цепочку
/// `country_handler.get_country → country_state.has_job` буквально, игра
/// приходит через короткий перезайм view от обработчика.
pub(crate) struct DeleteRoleCountryGateBridge<'a> {
    pub(crate) country_handler: &'a CCountryHandler,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
}

impl WorldDeleteRoleCountryGate for DeleteRoleCountryGateBridge<'_> {
    fn country_has_job(
        &mut self,
        game: &dyn crate::app::world_game_view::WorldGameView,
        country: u8,
        player_id: i32,
    ) -> bool {
        let Some(country_state) = self.country_handler.get_country(country) else {
            return false;
        };
        let mut effects = DeleteRoleCountryEffects {
            game,
            globe_setup: self.globe_setup,
        };
        country_state.has_job(player_id, &mut effects) != 0
    }
}

/// Адаптер страновой таблицы ветви create-role: ровно
/// `get_country(...).is_some()` прежнего обработчика без переноса самого
/// `CCountryHandler` в сигнатуру диспетчера. Прямой impl трейта для
/// `CCountryHandler` здесь невозможен (и трейт, и тип уже в Realm —
/// orphan-правило), поэтому seam остаётся у единого владельца — этого
/// callsite-а.
pub(crate) struct CreateRoleCountryViewAdapter<'a> {
    pub(crate) handler: &'a CCountryHandler,
}

impl WorldCountryView for CreateRoleCountryViewAdapter<'_> {
    fn country_exists(&self, country: u8) -> bool {
        self.handler.get_country(country).is_some()
    }
}

pub(crate) async fn process_world_message<TimerCallback, DbMiscContextOwner, JjcContext>(
    game: &mut CGame,
    honor_ranks: &mut CHonorRanks,
    player_ranks: &CPlayerRanks,
    increment_log: &mut CIncrementLog,
    auction_log: &mut CAuctionLog,
    db_misc: &CDbMisc,
    db_misc_context: &mut DbMiscContextOwner,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    country_handler: &mut CCountryHandler,
    country_parameters: &mut CCountryParam,
    country_war: &mut CountryWarSys,
    four_nation_war: &mut CFourNationWarSys,
    country_limits: CountryKingSaveLimits,
    faction_war_sys: &mut CFactionWarSys,
    attack_city: &mut CAttackCitySys,
    attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
    village_war: &mut CVillageWarSys,
    goods_war: &mut CGoodsWarMember,
    timer: &mut CTimer<TimerCallback>,
    village_war_callbacks: VillageWarCallbacks<TimerCallback>,
    resources: &mut dyn WorldMainLoopResourceContext,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    net_sessions: &CNetSessionManager,
    jjc: &mut CJJcSystem,
    jjc_context: &mut JjcContext,
    application_runtime: &WorldUnionApplicationRuntimeOwner,
    application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    write_faction_title_log:
        &mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
    write_faction_purview_log:
        &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
    write_faction_apply_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    write_faction_join_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    write_faction_quit_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    write_faction_fire_out_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    write_faction_master_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
    write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    save_lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
    save_thread_handle: &mut WorldSaveThreadHandleState,
    save_runtime: &mut dyn WorldSaveRuntimeContext,
    session_factory: &mut CSessionFactory,
    mut general_variables: Option<&mut CVariableList>,
    gods_battle: &mut CGodsBattleConf,
    skills: &mut CSkillFactory,
    mut rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
    mut gods_battle_database: Option<&mut WorldTdsClient>,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    update_player: &mut dyn FnMut(i32),
    source: WorldMessageSource,
    mut message: CMessage,
) -> ProcessedWorldEvent
where
    TimerCallback: Copy,
    DbMiscContextOwner: DbMiscContext,
    JjcContext: JjcRunContext + ?Sized,
{
    let mut resource_snapshot = resources.main_loop_resource_snapshot();
    let registry = &resource_snapshot.registry;
    let original_name_index = &resource_snapshot.original_name_index;
    let coefficients = &resource_snapshot.coefficients;
    let globe_setup = &resource_snapshot.globe_setup;
    let jjc_config = globe_setup.jjc_run_config_world();
    let region_router = &resource_snapshot.region_router;
    let player_list = &mut resource_snapshot.player_list;
    let log_system = &resource_snapshot.log_system;
    let faction_chat_log_enabled = log_system.faction_chat_enabled();
    let private_chat_log_enabled = log_system.private_chat_enabled();
    let delete_log_enabled = log_system.delete_log_enabled();
    let faction_create_log_enabled = log_system.faction_create_enabled();
    let faction_title_log_enabled = log_system.faction_title_enabled();
    let faction_purview_add_log_enabled = log_system.faction_purview_add_enabled();
    let faction_purview_revoke_log_enabled = log_system.faction_purview_revoke_enabled();
    let faction_apply_log_enabled = log_system.faction_apply_enabled();
    let faction_join_log_enabled = log_system.faction_join_enabled();
    let faction_quit_log_enabled = log_system.faction_quit_enabled();
    let faction_fire_out_log_enabled = log_system.faction_fire_out_enabled();
    let faction_master_log_enabled = log_system.faction_master_changed_enabled();
    let faction_disband_log_enabled = log_system.faction_disband_enabled();
    application_callbacks.faction_level_log_enabled = log_system.faction_level_enabled();
    application_callbacks.faction_experience_log_enabled =
        log_system.faction_experience_enabled();
    let message_type = message.message_type();
    let mut selector = WorldOwnerSelector {
        write_log_enabled: game.setup.use_log_system,
        owner: None,
    };
    let legacy_run_result = message.run(&mut selector);

    if selector.owner == Some(WorldMessageOwner::Server) {
        let mut save_materialization = WorldCompletedSaveResponseMaterializationAdapter {
            faction_war_sys,
            country_handler,
            country_limits,
            lifecycle: Arc::clone(&save_lifecycle),
            save_thread_handle,
            save_runtime,
        };
        match on_server_message(
            game,
            message,
            registry,
            coefficients,
            organizing,
            honor_ranks,
            &mut save_materialization,
            add_log_text,
            session_factory,
            general_variables.as_deref_mut(),
            globe_setup,
            gods_battle,
            rs_gods_battle.as_deref_mut(),
            gods_battle_database.as_deref_mut(),
        )
        .await
        {
            WorldServerMessageDispatch::Handled(mut outcome) => {
                if let WorldServerMessageOutcome::GameServerConnection(report) = &mut outcome {
                    if let servermessage::WorldGameServerConnectionContinuation::InitialConfigurationPending {
                        socket_id,
                        game_server_index,
                    } = report.continuation.clone()
                    {
                        let configuration = game.send_initial_game_server_configuration(
                            &mut *resources,
                            socket_id,
                            game_server_index,
                            registry,
                            player_list,
                            skills,
                            globe_setup,
                            region_router,
                            country_parameters,
                            country_handler,
                            gods_battle,
                            four_nation_war,
                            honor_ranks,
                            player_ranks,
                            general_variables.as_deref(),
                            attack_city,
                            village_war,
                            country_war,
                        );
                        report.continuation = match configuration.completion {
                            WorldInitialConfigurationRunCompletion::Complete => {
                                servermessage::WorldGameServerConnectionContinuation::InitialConfigurationComplete {
                                    socket_id,
                                    game_server_index,
                                }
                            }
                            WorldInitialConfigurationRunCompletion::Blocked { owner } => {
                                servermessage::WorldGameServerConnectionContinuation::InitialConfigurationBlocked {
                                    socket_id,
                                    game_server_index,
                                    owner,
                                }
                            }
                        };
                        tracing::debug!(
                            socket_id,
                            messages = configuration.deliveries.len(),
                            payload_bytes = configuration.deliveries.iter().map(|delivery| delivery.payload_length).sum::<usize>(),
                            "World поставил начальную конфигурацию в очередь отправки"
                        );
                        report.initial_configuration = Some(configuration);
                    }
                    tracing::debug!(
                        socket_id = report.socket_id,
                        ip = %String::from_utf8_lossy(&report.ip),
                        port = report.port,
                        game_server_index = ?report.game_server_index,
                        continuation = ?report.continuation,
                        "World обработал регистрацию GameServer"
                    );
                }
                return ProcessedWorldEvent::ServerMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldServerMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Log) {
        let delete_log_enabled = game.setup.use_log_system && delete_log_enabled;
        // Gate-мост и обвязки журнала/tick собираются до диспетчера: побочных
        // эффектов у них нет, а порядок ветвей и точки снятия `_time`/
        // errno-log сохранены внутри Realm-диспетчера.
        let mut country_gate = DeleteRoleCountryGateBridge {
            country_handler,
            globe_setup,
        };
        let country_view = CreateRoleCountryViewAdapter {
            handler: country_handler,
        };
        let mut log_wrapper = |bytes: &[u8]| {
            let _ = add_log_text(bytes);
        };
        let mut get_tick = legacy_tick_ms;
        match on_log_message(
            game,
            organizing,
            organizing_parameters,
            &mut country_gate,
            &country_view,
            country_parameters,
            player_list,
            session_factory,
            registry,
            original_name_index,
            coefficients,
            load_player_largess,
            globe_setup,
            rs_player,
            player_database.as_deref_mut(),
            delete_log_enabled,
            &mut log_wrapper,
            &mut get_tick,
            &mut *application_callbacks.random,
            message,
        )
        .await
        {
            WorldLogMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::LogMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldLogMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Player) {
        match on_player_message(game, message) {
            WorldPlayerMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::PlayerMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldPlayerMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Gma) {
        match on_gma_message(game, message, add_log_text) {
            WorldGmaMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::GmaMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldGmaMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Other) {
        let faction_chat_log_enabled =
            game.setup.use_log_system && faction_chat_log_enabled;
        let private_chat_log_enabled =
            game.setup.use_log_system && private_chat_log_enabled;
        match on_other_message(
            game,
            organizing,
            honor_ranks,
            increment_log,
            globe_setup,
            registry,
            &mut *application_callbacks.random,
            rs_player,
            player_database.as_deref_mut(),
            faction_chat_log_enabled,
            private_chat_log_enabled,
            add_log_text,
            message,
        )
        .await
        {
            WorldOtherMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::OtherMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldOtherMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Gm) {
        match on_gm_message(
            game,
            jjc,
            gods_battle,
            skills,
            rs_gods_battle.as_deref_mut(),
            rs_player,
            player_database.as_deref_mut(),
            &mut *resources,
            message,
        )
        .await
        {
            WorldGmMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::GmMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldGmMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Team) {
        let outcome = on_team_message(game, session_factory, &mut message);
        return ProcessedWorldEvent::TeamMessage {
            source,
            legacy_run_result,
            outcome,
        };
    }

    if selector.owner == Some(WorldMessageOwner::JjcSystem) {
        let outcome = on_jjc_system_message(game, jjc, jjc_config, jjc_context, &mut message);
        return ProcessedWorldEvent::JjcMessage {
            source,
            legacy_run_result,
            outcome,
        };
    }

    if selector.owner == Some(WorldMessageOwner::MiscAuction) {
        match on_msg_m2w_auction(game, db_misc, db_misc_context, message) {
            WorldMiscAuctionMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::MiscAuctionMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldMiscAuctionMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::ServerAuction) {
        match on_msg_s2w_auction(
            game,
            auction_log,
            db_misc,
            db_misc_context,
            globe_setup,
            registry,
            coefficients,
            message,
        ) {
            WorldServerAuctionMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::ServerAuctionMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldServerAuctionMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::WriteLog) {
        match on_write_log_message(game, increment_log, auction_log, add_log_text, message) {
            WorldWriteLogMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::WriteLogMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldWriteLogMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Country) {
        if let Some(sync) = dispatch_four_nation_exploit_message(
            &mut message,
            game,
            four_nation_war,
            rs_player,
            player_database.as_deref_mut(),
            add_log_text,
        )
        .await
        {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationExploit(sync),
            };
        }
        if let Some(sync) = dispatch_country_player_change_message(
            &mut message,
            game,
            &*country_handler,
        ) {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::PlayerCountryChanged(sync),
            };
        }
        let new_day = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_new_day_message(
                &message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = new_day {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::NewDaySet(sync),
            };
        }
        let direct_appointment = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_direct_appointment_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = direct_appointment {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryAppointedDirectly(sync),
            };
        }
        let country_info = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_info_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = country_info {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryInfoSent(sync),
            };
        }
        let players_list = {
            let mut effects = WorldCountryPlayersListEffects {
                game: &*game,
                organizing: &*organizing,
                globe_setup,
            };
            dispatch_country_players_list_message(
                &mut message,
                country_handler,
                &mut effects,
            )
        };
        if let Some(sync) = players_list {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryPlayersListed(sync),
            };
        }
        let exile_result = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_exile_result_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = exile_result {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::ExileResultSynchronized(sync),
            };
        }
        let silence_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_silence_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = silence_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::SilenceRequested(sync),
            };
        }
        let absolve_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_absolve_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = absolve_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::AbsolveRequested(sync),
            };
        }
        let demise = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_demise_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = demise {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::KingDemised(sync),
            };
        }
        let depose_minister = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_depose_minister_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = depose_minister {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::MinisterDeposed(sync),
            };
        }
        let appoint_minister = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_appoint_minister_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = appoint_minister {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::MinisterAppointed(sync),
            };
        }
        let exile_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
            };
            dispatch_country_exile_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = exile_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::ExileRequested(sync),
            };
        }
        let four_nation_country_fail = {
            let mut effects = WorldFourNationCountryFailEffects {
                game,
                organizing: &*organizing,
            };
            dispatch_four_nation_country_fail_message(
                &mut message,
                &*four_nation_war,
                &mut effects,
            )
        };
        if let Some(sync) = four_nation_country_fail {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationCountryFail(sync),
            };
        }
        let four_nation_war_time = {
            let mut effects = WorldFourNationWarResultEffects { game };
            dispatch_four_nation_war_time_message(
                &mut message,
                four_nation_war,
                &mut effects,
            )
        };
        if let Some(sync) = four_nation_war_time {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationWarTime(sync),
            };
        }
        let four_nation_result = {
            let mut effects = WorldFourNationWarResultEffects { game };
            dispatch_four_nation_war_result_message(
                &mut message,
                four_nation_war,
                &mut effects,
            )
        };
        if let Some(sync) = four_nation_result {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationWarResult(sync),
            };
        }
        let declaration = {
            let mut effects = WorldCountryWarEffects {
                game,
                country_handler,
                globe_setup,
            };
            dispatch_country_war_declaration_message(&mut message, country_war, &mut effects)
        };
        if let Some(sync) = declaration {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryWarDeclared(sync),
            };
        }
        let victory = {
            let mut effects = WorldCountryWarEffects {
                game,
                country_handler,
                globe_setup,
            };
            dispatch_country_war_victory_message(&mut message, country_war, &mut effects)
        };
        match victory {
            Ok(Some(sync)) => {
                return ProcessedWorldEvent::CountryMessage {
                    source,
                    legacy_run_result,
                    outcome: WorldCountryMessageOutcome::CountryWarVictory(sync),
                };
            }
            Ok(None) => {}
            Err(block) => match block.source {},
        }
        match on_country_message(
            game,
            country_handler,
            country_parameters,
            globe_setup,
            message,
        ) {
            WorldCountryMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::CountryMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldCountryMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::OrganizingSystem) {
        if let Some(outcome) = dispatch_faction_war_player_died(
            &mut message,
            game,
            organizing,
            faction_war_sys,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionWarPlayerDied {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let faction_create = dispatch_create_faction(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            country_handler,
            registry,
            original_name_index,
            coefficients,
            rs_player,
            player_database.as_deref_mut(),
            application_callbacks,
            game.setup.use_log_system && faction_create_log_enabled,
            write_faction_create_log,
        )
        .await;
        if let Some(outcome) = faction_create {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCreateFaction {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_initial_organizing_data(&mut message, game, organizing)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingInitialData {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_leave_word(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWord {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_leave_word_edit(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWordEdit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_pronounce(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPronounce {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        if let Some(outcome) = dispatch_declare_faction_war(
            &mut message,
            game,
            organizing,
            faction_war_sys,
            registry,
            coefficients,
            update_player,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingDeclareFactionWar {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        let mut world_string = |string_id: &[u8]| game.get_string_by_id(string_id).to_vec();
        if let Some(outcome) = dispatch_faction_billboard(
            &mut message,
            organizing,
            &mut world_string,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionBillboard {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let use_log_system = game.setup.use_log_system;
        if let Some(outcome) = dispatch_faction_upgrade(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            registry,
            original_name_index,
            coefficients,
            use_log_system,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionUpgrade {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_upload_icon(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionUploadIcon {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_contributor(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionContributor {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_experience(
            &mut message,
            game,
            organizing,
            use_log_system,
            application_callbacks,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionExperience {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_member_state(
            &mut message,
            game,
            organizing,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionMemberState {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        if let Some(outcome) = dispatch_region_param_update(
            &mut message,
            game,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingRegionParamUpdate {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_goods_war_command(&mut message, game, organizing, goods_war)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingGoodsWarCommand {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_goods_war_faction_win(&mut message, game, organizing, goods_war)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingGoodsWarFactionWin {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_quest_command(
            &mut message,
            game,
            game.current_game_server_sender().as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerQuestCommand {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_run_script(
            &mut message,
            game,
            game.current_game_server_sender().as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerRunScript {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_parameter(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionParameter {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_change_region_router(
            &mut message,
            region_router,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingChangeRegionRouter {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_village_war_application(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            attack_city,
            village_war,
            application_callbacks,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingVillageWarApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_village_war_result(
            &mut message,
            game,
            organizing,
            village_war,
            timer,
            village_war_callbacks,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingVillageWarResult {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_war_application(
            &mut message,
            game,
            globe_setup,
            organizing,
            organizing_parameters,
            attack_city,
            village_war,
            application_callbacks,
            update_player,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityWarApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let country_gate = &mut CityWarCountryGateBridge {
            country_handler,
            country_parameters: &*country_parameters,
        };
        if let Some(outcome) = dispatch_city_war_result(
            &mut message,
            game,
            organizing,
            country_gate,
            attack_city,
            timer,
            attack_city_callbacks,
            application_callbacks,
            update_player,
            globe_setup,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityWarResult {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application_decision(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_join_log_enabled,
            write_faction_join_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplicationDecision {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_fire_out(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_fire_out_log_enabled,
            write_faction_fire_out_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionFireOut {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_exit(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_quit_log_enabled,
            write_faction_quit_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionExit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_exit(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionExit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_demise(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            &*country_handler,
            &*attack_city,
            &*goods_war,
            game.setup.use_log_system && faction_master_log_enabled,
            write_faction_master_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDemise {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_demise(
            &mut message,
            game,
            organizing,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionDemise {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let faction_disband = {
            let mut effects = WorldOrganizingDisbandEffects {
                game: &*game,
                village_war: &*village_war,
                attack_city: &*attack_city,
                country_handler: &*country_handler,
                goods_war: &mut *goods_war,
            };
            dispatch_faction_disband(&mut message, &*game, organizing, &mut effects)
        };
        if let Some(outcome) = faction_disband {
            let faction_disband_log_enabled =
                game.setup.use_log_system && faction_disband_log_enabled;
            let outcome = outcome.map(|pending| {
                finalize_faction_disband_dispatch(
                    pending,
                    faction_disband_log_enabled,
                    |player_id| game.clear_disbanded_player_faction_data(player_id),
                    |faction_id, faction_name, player_id, player_name| {
                        write_faction_disband_log(
                            faction_id,
                            faction_name,
                            player_id,
                            player_name,
                        );
                    },
                )
            });
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDisband {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_disband(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionDisband {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_dub(
            &mut message,
            game,
            organizing,
            application_callbacks,
            game.setup.use_log_system,
            faction_title_log_enabled,
            write_faction_title_log,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDub {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_purview(
            &mut message,
            game,
            organizing,
            application_callbacks,
            game.setup.use_log_system,
            faction_purview_add_log_enabled,
            faction_purview_revoke_log_enabled,
            write_faction_purview_log,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionPurview {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_fire_out(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionFireOut {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *application_callbacks.random,
            refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
            faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
            faction_experience_log_enabled:
                application_callbacks.faction_experience_log_enabled,
            write_faction_experience_log:
                &mut *application_callbacks.write_faction_experience_log,
        };
        let mut effects = WorldUnionApplicationEffects::new(
            game,
            net_sessions,
            application_runtime,
            callbacks,
        );
        if let Some(outcome) = dispatch_region_route(&mut message, game) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingRegionRoute {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_gate(&mut message, game, organizing) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityGate {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_transfer(
            &mut message,
            game,
            country_handler,
            organizing,
            attack_city,
            village_war,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityTransfer {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_admission_permit(&mut message, game, organizing) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingAdmissionPermit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_attack_city_end(
            &mut message,
            game,
            organizing,
            &mut effects,
            update_player,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingAttackCityEnd {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_tax(
            &mut message,
            organizing,
            attack_city,
            village_war,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionTax {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_list(
            &mut message,
            organizing,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionList {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            game.setup.use_log_system,
            faction_apply_log_enabled,
            write_faction_apply_log,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application_cancel(
            &mut message,
            game,
            organizing,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplicationCancel {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_declare_war_faction_list(
            &mut message,
            organizing,
            faction_war_sys,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingDeclareWarFactionList {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_consumed_long(&mut message) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingConsumedLong {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_invite_faction(
            &mut message,
            game,
            organizing,
            village_war,
            attack_city,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerInviteFaction {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        match dispatch_organizing_session_result(&mut message, net_sessions) {
            OrganizingSessionResultDispatch::NotHandled => {}
            outcome => {
                let runtime = drain_union_application_runtime(
                    game,
                    organizing,
                    organizing_parameters,
                    application_runtime,
                    &mut effects,
                    update_player,
                );
                return ProcessedWorldEvent::OrganizingSessionResult {
                    source,
                    legacy_run_result,
                    outcome,
                    runtime,
                };
            }
        }
        if let Some(outcome) =
            dispatch_union_application(&mut message, game, organizing, &mut effects)
        {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_leave_word_enable(&mut message, organizing, &mut effects)
        {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWordEnable {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }

        let callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *application_callbacks.random,
            refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
            faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
            faction_experience_log_enabled:
                application_callbacks.faction_experience_log_enabled,
            write_faction_experience_log:
                &mut *application_callbacks.write_faction_experience_log,
        };
        let mut effects = WorldUnionApplicationEffects::new(
            game,
            net_sessions,
            application_runtime,
            callbacks,
        );
        let runtime = drain_union_application_runtime(
            game,
            organizing,
            organizing_parameters,
            application_runtime,
            &mut effects,
            update_player,
        );
        return ProcessedWorldEvent::OrganizingNoOp {
            source,
            legacy_run_result,
            request_type: message_type,
            runtime,
        };
    }

    ProcessedWorldEvent::Message(RoutedWorldMessage {
        source,
        message_type,
        owner: selector.owner,
        legacy_run_result,
        message,
    })
}

pub(crate) fn drain_union_application_runtime(
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    runtime: &WorldUnionApplicationRuntimeOwner,
    effects: &mut WorldUnionApplicationEffects<'_>,
    update_player: &mut dyn FnMut(i32),
) -> WorldUnionApplicationRuntimeReport {
    let mut terminals = Vec::new();
    let mut invitation_terminals = Vec::new();
    let mut city_terminals = Vec::new();
    let mut confederation_creation_terminals = Vec::new();
    while let Some(request) = runtime.pop_terminal() {
        match request {
            QueuedOrganizingSessionTerminal::Union(request) => {
                let outcome = organizing.finish_union_application(
                    game,
                    organizing_parameters,
                    request.union_id,
                    request.applicant_faction_id,
                    request.terminal,
                    effects,
                    update_player,
                );
                terminals.push(WorldUnionApplicationTerminalDispatch { request, outcome });
            }
            QueuedOrganizingSessionTerminal::UnionInvitation(request) => {
                let outcome = organizing.finish_union_invitation(
                    game,
                    organizing_parameters,
                    request.union_id,
                    request.inviter_faction_id,
                    request.invited_faction_id,
                    request.terminal,
                    effects,
                    update_player,
                );
                invitation_terminals.push(WorldUnionInvitationTerminalDispatch {
                    request,
                    outcome,
                });
            }
            QueuedOrganizingSessionTerminal::ConfederationCreation(request) => {
                let outcome = organizing.finish_confederation_creation(
                    game,
                    organizing_parameters,
                    request.first_player_id,
                    request.second_player_id,
                    request.first_faction_id,
                    request.second_faction_id,
                    &request.union_name,
                    request.terminal,
                    effects,
                    update_player,
                );
                confederation_creation_terminals.push(
                    WorldConfederationCreationTerminalDispatch { request, outcome },
                );
            }
            QueuedOrganizingSessionTerminal::CityTransfer(request) => {
                let outcome = organizing.finish_city_transfer(
                    game,
                    request.source_faction_id,
                    request.target_faction_id,
                    request.region_id,
                    &request.region_name,
                    request.terminal,
                    effects,
                    update_player,
                );
                city_terminals.push(WorldCityTransferTerminalDispatch { request, outcome });
            }
        }
    }
    WorldUnionApplicationRuntimeReport {
        terminals,
        invitation_terminals,
        confirmations: runtime.take_confirmations(),
        endpoint_blocks: runtime.take_blocks(),
        city_terminals,
        city_confirmations: runtime.take_city_confirmations(),
        city_endpoint_blocks: runtime.take_city_blocks(),
        confederation_creation_terminals,
        confederation_creation_confirmations:
            runtime.take_confederation_creation_confirmations(),
        confederation_creation_endpoint_blocks:
            runtime.take_confederation_creation_blocks(),
    }
}

pub(crate) struct WorldOwnerSelector {
    pub(crate) write_log_enabled: bool,
    pub(crate) owner: Option<WorldMessageOwner>,
}

impl WorldOwnerSelector {
    pub(crate) fn select(&mut self, owner: WorldMessageOwner) {
        self.owner = Some(owner);
    }
}

impl WorldMessageHandlers for WorldOwnerSelector {
    fn write_log_enabled(&self) -> bool {
        self.write_log_enabled
    }

    fn on_server(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Server);
    }

    fn on_log(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Log);
    }

    fn on_gma(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Gma);
    }

    fn on_player(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Player);
    }

    fn on_other(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Other);
    }

    fn on_gm(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Gm);
    }

    fn on_team(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Team);
    }

    fn on_orgasys(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::OrganizingSystem);
    }

    fn on_write_log(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::WriteLog);
    }

    fn on_country(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Country);
    }

    fn on_server_auction(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::ServerAuction);
    }

    fn on_jjc_system(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::JjcSystem);
    }

    fn on_misc_auction(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::MiscAuction);
    }
}

pub(crate) fn add_legacy_c_string(message: &mut nebokrai_shared::network::CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

// Gate `ShowSaveInfo` живёт в Realm persistence/savedb вместе со своей
// save-публикующей семьёй.
pub(crate) fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

#[allow(dead_code, reason = "исторически неиспользуемый format-helper семьи world-строк: перенесён буквально; вызовов нет ни в старом пакете, ни здесь")]
pub(crate) fn format_faction_enemy_world_string(
    template: &[u8],
    arguments: &[FactionEnemyWarLogArgument<'_>],
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() {
        if template[offset] != b'%' || offset + 1 == template.len() {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let specifier = template[offset + 1];
        if specifier == b'%' {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index) else {
            output.extend_from_slice(&template[offset..offset + 2]);
            offset += 2;
            continue;
        };
        match (specifier, argument) {
            (b's', FactionEnemyWarLogArgument::Text(text)) => {
                output.extend_from_slice(legacy_c_string_prefix(text));
            }
            (b'd' | b'i' | b'u', FactionEnemyWarLogArgument::Unsigned(value)) => {
                output.extend_from_slice(value.to_string().as_bytes());
            }
            _ => {
                output.extend_from_slice(&template[offset..offset + 2]);
                offset += 2;
                continue;
            }
        }
        argument_index += 1;
        offset += 2;
    }
    output
}

/// Safe MSVCRT-compatible subset, используемый исходными World string ID.
/// Поддерживаются только реально передаваемые `%s`, `%d`, `%i`, `%u` и `%%`;
/// неизвестный либо не согласованный с аргументом specifier сохраняется как
/// текст вместо чтения отсутствующего vararg и внутреннего UB оригинала.
pub(crate) fn format_union_world_string(
    template: &[u8],
    arguments: &[UnionFormatArgument<'_>],
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() {
        if template[offset] != b'%' || offset + 1 == template.len() {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let specifier = template[offset + 1];
        if specifier == b'%' {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index) else {
            output.extend_from_slice(&template[offset..offset + 2]);
            offset += 2;
            continue;
        };
        match (specifier, argument) {
            (b's', UnionFormatArgument::Text(text)) => {
                output.extend_from_slice(legacy_c_string_prefix(text));
            }
            (b'd' | b'i', UnionFormatArgument::Signed(value)) => {
                output.extend_from_slice(value.to_string().as_bytes());
            }
            (b'u', UnionFormatArgument::Signed(value)) => {
                output.extend_from_slice((*value as u32).to_string().as_bytes());
            }
            _ => {
                output.extend_from_slice(&template[offset..offset + 2]);
                offset += 2;
                continue;
            }
        }
        argument_index += 1;
        offset += 2;
    }
    output
}

pub(crate) fn format_legacy_percent_s(template: &[u8], argument: &[u8]) -> Vec<u8> {
    let Some(position) = template.windows(2).position(|window| window == b"%s") else {
        return template.to_vec();
    };
    let mut formatted = Vec::with_capacity(template.len() + argument.len());
    formatted.extend_from_slice(&template[..position]);
    formatted.extend_from_slice(argument);
    formatted.extend_from_slice(&template[position + 2..]);
    formatted
}

pub(crate) fn copy_name_for_legacy_lowercase(value: &[u8]) -> Vec<u8> {
    let value = legacy_c_string_prefix(value);
    let mut copy = value.to_vec();
    CGame::to_strlwr(&mut copy);
    copy
}

pub struct WorldUnionApplicationEffectCallbacks<'a> {
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

pub struct WorldUnionApplicationEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) manager: &'a CNetSessionManager,
    pub(crate) runtime: &'a WorldUnionApplicationRuntimeOwner,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: WorldUnionApplicationEffectCallbacks<'a>,
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

pub(crate) struct WorldFactionCreationEffects<'game, 'callbacks, 'effects, 'log> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) persistent_name_exists: bool,
    pub(crate) faction_create_log_enabled: bool,
    pub(crate) write_faction_create_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8]),
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

pub(crate) struct WorldUnionFireOutEffects<'game, 'callbacks, 'effects> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
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

pub(crate) struct WorldFactionDubEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
    pub(crate) use_log_system: bool,
    pub(crate) faction_title_log_enabled: bool,
    pub(crate) write_faction_title_log:
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

pub(crate) struct WorldFactionPurviewEffects<'game, 'callbacks, 'effects, 'log> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) use_log_system: bool,
    pub(crate) add_log_enabled: bool,
    pub(crate) revoke_log_enabled: bool,
    pub(crate) write_log: &'log mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
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

/// Узкий war/string/log adapter faction-ветви `0x60108`. Держит игру прямо,
/// а не через общий session/transport owner: realm-обработчик ветви получает
/// `&mut`-ссылки и на этот адаптер, и на owner-а соседней union-ветви
/// одновременно, поэтому owner-borrow здесь не нужен и не допустим; игровой
/// объект тот же, что и в owner-е, во всех call-site-ах диспетчера.
pub(crate) struct WorldFactionApplicationEffects<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) village_war: &'a CVillageWarSys,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) use_log_system: bool,
    pub(crate) faction_apply_log_enabled: bool,
    pub(crate) write_faction_apply_log: &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionApplyForJoinEffects for WorldFactionApplicationEffects<'_> {
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
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
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

pub(crate) struct WorldFactionDoJoinEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    pub(crate) game: &'game CGame,
    pub(crate) village_war: &'game CVillageWarSys,
    pub(crate) attack_city: &'game CAttackCitySys,
    pub(crate) goods_war: &'game CGoodsWarMember,
    pub(crate) use_log_system: bool,
    pub(crate) faction_join_log_enabled: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
    pub(crate) write_faction_join_log:
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

pub(crate) struct WorldFactionDemiseEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    pub(crate) game: &'game CGame,
    pub(crate) attack_city: &'game CAttackCitySys,
    pub(crate) goods_war: &'game CGoodsWarMember,
    pub(crate) country_handler: &'game CCountryHandler,
    pub(crate) faction_master_log_enabled: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
    pub(crate) write_faction_master_log:
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

pub(crate) struct WorldFactionExitEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    pub(crate) game: &'game CGame,
    pub(crate) village_war: &'game CVillageWarSys,
    pub(crate) attack_city: &'game CAttackCitySys,
    pub(crate) goods_war: &'game mut CGoodsWarMember,
    pub(crate) use_log_system: bool,
    pub(crate) faction_quit_log_enabled: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
    pub(crate) write_faction_quit_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
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

pub(crate) struct WorldFactionFireOutEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    pub(crate) game: &'game CGame,
    pub(crate) village_war: &'game CVillageWarSys,
    pub(crate) attack_city: &'game CAttackCitySys,
    pub(crate) goods_war: &'game mut CGoodsWarMember,
    pub(crate) use_log_system: bool,
    pub(crate) faction_fire_out_log_enabled: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
    pub(crate) write_faction_fire_out_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
}

pub(crate) struct WorldFactionFireOutGoodsWarDelivery<'game> {
    pub(crate) game: &'game CGame,
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

pub(crate) struct WorldFactionSetParameterEffects<'game, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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

pub(crate) struct WorldFactionUpgradeEffects<'game, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game CGame,
    pub(crate) registry: &'game GoodsBasePropertiesRegistry,
    pub(crate) original_name_index: &'game GoodsOriginalNameIndex,
    pub(crate) use_log_system: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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

pub(crate) struct WorldFactionUploadIconEffects<'game, 'callbacks, 'effects> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
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

pub(crate) struct WorldFactionContributorEffects<'game, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game CGame,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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

/// Session-result семейство (`0x60117/0x60119/0x60120/0x60122/0x60124/0x60131`)
/// обслуживает realm-модуль (`crate::app::organsysmessage`) вместе с общим
/// runtime-менеджером; обвязка только сохраняет прежнюю точку входа main-loop.
pub(crate) fn dispatch_organizing_session_result(
    message: &mut CMessage,
    manager: &CNetSessionManager,
) -> OrganizingSessionResultDispatch {
    crate::app::organsysmessage::dispatch_organizing_session_result(message, manager)
}

/// Bridge-адаптер ветви `0x60116`: связывает game/war/effects владельцев
/// старого пакета для realm-обработчика; сам inherent-вход контроллера
/// остаётся единственным вызовом шва.
pub(crate) struct WorldPlayerInviteFactionBridge<'a, Effects> {
    pub(crate) game: &'a CGame,
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) village_war: &'a CVillageWarSys,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) effects: &'a mut Effects,
}

impl<Effects> crate::app::organsysmessage::OrganizingPlayerInviteFactionBridge
    for WorldPlayerInviteFactionBridge<'_, Effects>
where
    Effects: PlayerInviteFactionEffects,
{
    type CreationReport = <Effects as ConfederationCreationEffects>::SessionReport;
    type ApplicationReport = <Effects as UnionApplyForJoinEffects>::SessionReport;
    type InvitationReport = <Effects as UnionInviteEffects>::SessionReport;
    type CreationBlock = <Effects as ConfederationCreationEffects>::SessionBlock;
    type ApplicationBlock = <Effects as UnionApplyForJoinEffects>::SessionBlock;
    type InvitationBlock = <Effects as UnionInviteEffects>::SessionBlock;

    fn on_player_invite_faction(
        &mut self,
        player_id: i32,
        invited_faction_id: i32,
    ) -> Result<
        PlayerInviteFactionOutcome<
            Self::CreationReport,
            Self::ApplicationReport,
            Self::InvitationReport,
        >,
        PlayerInviteFactionBlock<
            Self::CreationBlock,
            Self::ApplicationBlock,
            Self::InvitationBlock,
        >,
    > {
        self.organizing.on_player_invite_faction(
            self.game,
            self.village_war,
            self.attack_city,
            player_id,
            invited_faction_id,
            self.effects,
        )
    }
}

/// Ветвь `0x60116` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом invite-входа контроллера; обвязка лишь собирает адаптер и
/// передаёт ход realm-обработчику в исходных типах.
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
    let mut bridge = WorldPlayerInviteFactionBridge {
        game,
        organizing,
        village_war,
        attack_city,
        effects,
    };
    crate::app::organsysmessage::dispatch_player_invite_faction(message, &mut bridge)
}

/// Bridge-адаптер ветви `0x60118` union application: связывает game/effects
/// владельцев старого пакета для realm-обработчика.
pub(crate) struct WorldUnionApplicationBridge<'a, Effects> {
    pub(crate) game: &'a CGame,
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) effects: &'a mut Effects,
}

impl<Effects> crate::app::organsysmessage::OrganizingUnionApplicationBridge
    for WorldUnionApplicationBridge<'_, Effects>
where
    Effects: UnionApplyForJoinEffects,
{
    type SessionReport = Effects::SessionReport;
    type SessionBlock = Effects::SessionBlock;

    fn apply_for_union_join(
        &mut self,
        master_player_id: i32,
        applicant_faction_id: i32,
        second_parameter: i32,
        third_parameter: i32,
    ) -> Result<
        OrganizingUnionApplyForJoinOutcome<Self::SessionReport>,
        OrganizingUnionApplyForJoinDispatchBlock<Self::SessionBlock>,
    > {
        self.organizing.apply_for_union_join(
            self.game,
            master_player_id,
            applicant_faction_id,
            second_parameter,
            third_parameter,
            self.effects,
        )
    }
}

/// Ветвь `0x60118` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом union-входа контроллера; обвязка лишь собирает адаптер и
/// передаёт ход realm-обработчику в исходных типах.
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
    let mut bridge = WorldUnionApplicationBridge {
        game,
        organizing,
        effects,
    };
    crate::app::organsysmessage::dispatch_union_application(message, &mut bridge)
}

/// Ветвь `0x60101` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает faction-war effects адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_war_player_died(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    faction_war: &mut CFactionWarSys,
    update_player: &mut dyn FnMut(i32),
) -> Option<OrganizingFactionWarPlayerDiedDispatch> {
    let mut effects = WorldFactionWarDeclarationEffects::new(
        game,
        organizing,
        update_player,
    );
    crate::app::organsysmessage::dispatch_faction_war_player_died(
        message,
        faction_war,
        &mut effects,
    )
}

/// Причина остановки ветви `0x60103` — type-алиас generic realm-перечислителя
/// `OrganizingCreateFactionDispatchBlock` на конкретном [`FactionCreationBlock`]
/// старого `COrganizingCtrl`; форма и порядок вариантов прежнего локального
/// enum сохраняются один в один.
pub type OrganizingCreateFactionBlock =
    crate::app::organsysmessage::OrganizingCreateFactionDispatchBlock<FactionCreationBlock>;

/// Bridge-адаптер ветви `0x60103`: связывает game/country/DB/log владельцев
/// старого пакета для realm-обработчика. Середина `prepare → DB → finish` и
/// игровые снапшоты держат concrete `&CGame`, а decode игрока из wire-хвоста —
/// `&mut`-заём той же игры, поэтому вызовы разводятся отдельными методами шва
/// внутри одного адаптера; DB-контакт (`is_name_exist`) покрыт прежним
/// `RsPlayerOwner::is_name_exist` по ADR-0013, отдельный DB-view не создаётся.
pub(crate) struct WorldCreateFactionBridge<'game, 'view, 'db, 'callbacks, 'effects, 'log> {
    pub(crate) game: &'game mut CGame,
    pub(crate) organizing: &'game mut COrganizingCtrl,
    pub(crate) parameters: &'view COrganizingParam,
    pub(crate) countries: &'view CCountryHandler,
    pub(crate) registry: &'view GoodsBasePropertiesRegistry,
    pub(crate) coefficients: &'view PlayerPropertyCoefficients,
    pub(crate) rs_player: &'db mut TiberiusRsPlayer,
    pub(crate) player_database: Option<&'db mut WorldTdsClient>,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) persistent_name_exists: bool,
    pub(crate) faction_create_log_enabled: bool,
    pub(crate) write_faction_create_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8]),
}

impl crate::app::organsysmessage::OrganizingCreateFactionBridge
    for WorldCreateFactionBridge<'_, '_, '_, '_, '_, '_>
{
    type CreationBlock = FactionCreationBlock;

    fn world_game(&self) -> &dyn crate::app::world_game_view::WorldGameView {
        self.game
    }

    fn country_exists(&self, country: u8) -> bool {
        self.countries.get_country(country).is_some()
    }

    fn decode_online_player(
        &mut self,
        player_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            self.registry,
            self.coefficients,
        )
    }

    async fn persistent_name_exists(&mut self, name: &[u8]) -> bool {
        let exists = self
            .rs_player
            .is_name_exist(
                legacy_c_string_prefix(name),
                self.player_database.as_deref_mut(),
            )
            .await;
        self.persistent_name_exists = exists;
        exists
    }

    fn prepare_faction_creation(
        &mut self,
        player_id: i32,
        faction_name: &mut Vec<u8>,
    ) -> Result<FactionCreationPreparation, Self::CreationBlock> {
        let game: &CGame = &*self.game;
        let mut effects = WorldFactionCreationEffects {
            game,
            callbacks: &mut *self.callbacks,
            persistent_name_exists: self.persistent_name_exists,
            faction_create_log_enabled: self.faction_create_log_enabled,
            write_faction_create_log: &mut *self.write_faction_create_log,
        };
        self.organizing
            .prepare_faction_creation(game, player_id, faction_name, &mut effects)
    }

    fn finish_faction_creation(
        &mut self,
        player_id: i32,
        established_time: TagTimeValue,
        faction_name: &[u8],
        country: u8,
        persistent_name_exists: bool,
    ) -> Result<FactionCreationOutcome, Self::CreationBlock> {
        let game: &CGame = &*self.game;
        let mut effects = WorldFactionCreationEffects {
            game,
            callbacks: &mut *self.callbacks,
            persistent_name_exists: self.persistent_name_exists,
            faction_create_log_enabled: self.faction_create_log_enabled,
            write_faction_create_log: &mut *self.write_faction_create_log,
        };
        self.organizing.finish_faction_creation(
            game,
            self.parameters,
            player_id,
            0,
            established_time,
            faction_name,
            country,
            persistent_name_exists,
            &mut effects,
        )
    }

    fn update_player_faction_info(
        &mut self,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        self.game
            .update_player_faction_info(self.organizing, player_id)
    }

    fn add_faction_to_client_by_player_id(
        &mut self,
        player_id: i32,
    ) -> Result<bool, FactionClientSnapshotBlock> {
        self.organizing
            .add_faction_to_client_by_player_id(&*self.game, player_id)
    }

    fn add_all_faction_info_to_client_by_player_id(
        &mut self,
        player_id: i32,
    ) -> Result<bool, AllFactionInfoClientBlock> {
        self.organizing
            .add_all_faction_info_to_client_by_player_id(&*self.game, player_id)
    }

    fn send_organizing_info_to_client(
        &mut self,
        request: FactionMemberInfoRequest<'_>,
    ) -> OrganizingInfoDelivery {
        COrganizingCtrl::send_organizing_info_to_client(&*self.game, request)
    }
}

/// Ветвь `0x60103` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом creation-середины; обвязка лишь собирает адаптер и передаёт
/// ход realm-обработчику в исходных типах.
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
    let mut bridge = WorldCreateFactionBridge {
        game,
        organizing,
        parameters,
        countries,
        registry,
        coefficients,
        rs_player,
        player_database,
        callbacks,
        persistent_name_exists: false,
        faction_create_log_enabled,
        write_faction_create_log,
    };
    crate::app::organsysmessage::dispatch_create_faction(
        message,
        parameters,
        original_name_index,
        &mut bridge,
    )
    .await
}

/// Bridge-адаптер ветви `0x60104`: связывает game/organizing владельцев
/// старого пакета для realm-обработчика; три client-snapshot вызова шва
/// делегируют одноимённым inherent-методам контроллера.
pub(crate) struct WorldInitialDataBridge<'a> {
    pub(crate) game: &'a CGame,
    pub(crate) organizing: &'a mut COrganizingCtrl,
}

impl crate::app::organsysmessage::OrganizingInitialDataBridge
    for WorldInitialDataBridge<'_>
{
    fn add_faction_to_client_by_player_id(
        &mut self,
        player_id: i32,
    ) -> Result<bool, FactionClientSnapshotBlock> {
        self.organizing
            .add_faction_to_client_by_player_id(self.game, player_id)
    }

    fn add_union_to_client_by_player_id(
        &mut self,
        player_id: i32,
    ) -> Result<UnionClientSnapshotByPlayerOutcome, UnionClientSnapshotByPlayerBlock> {
        self.organizing
            .add_union_to_client_by_player_id(self.game, player_id)
    }

    fn add_all_faction_info_to_client_by_player_id(
        &mut self,
        player_id: i32,
    ) -> Result<bool, AllFactionInfoClientBlock> {
        self.organizing
            .add_all_faction_info_to_client_by_player_id(self.game, player_id)
    }
}

/// Ветвь `0x60104` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом client-snapshot вызовов контроллера; обвязка лишь собирает
/// адаптер и передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_initial_organizing_data(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<OrganizingInitialDataDispatch> {
    let mut bridge = WorldInitialDataBridge { game, organizing };
    crate::app::organsysmessage::dispatch_initial_organizing_data(
        message,
        game,
        &mut bridge,
    )
}

/// Выполняет `0x60108`: `(player ID, discarded Long, name[20])`,
/// online/country gate и virtual `ApplyForJoin(player ID, 0, 0)`.
/// Ветвь `0x60108` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает war/string/log адаптер faction-пути и передаёт оба
/// effects-владельца обработчику в исходных типах.
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
    let mut faction_effects = WorldFactionApplicationEffects {
        game,
        village_war,
        attack_city,
        use_log_system,
        faction_apply_log_enabled,
        write_faction_apply_log,
    };
    crate::app::organsysmessage::dispatch_faction_application(
        message,
        game,
        organizing,
        parameters,
        &mut faction_effects,
        effects,
    )
}

/// Ветвь `0x6010A` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает join-effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
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
    crate::app::organsysmessage::dispatch_faction_application_decision(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010B` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает fire-out effects адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
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
    crate::app::organsysmessage::dispatch_faction_fire_out(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010C` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_fire_out(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    crate::app::organsysmessage::dispatch_union_fire_out(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x6010D` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает exit effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
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
    crate::app::organsysmessage::dispatch_faction_exit(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010E` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_exit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    crate::app::organsysmessage::dispatch_union_exit(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x6010F` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает demise effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
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
    crate::app::organsysmessage::dispatch_faction_demise(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60110` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и исходный
/// tick-callback, передавая ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_demise(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let mut get_tick = legacy_tick_ms;
    crate::app::organsysmessage::dispatch_union_demise(
        message,
        game,
        organizing,
        &mut effects,
        &mut get_tick,
        update_player,
    )
}

/// Ветвь `0x60112` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_disband(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    crate::app::organsysmessage::dispatch_union_disband(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x60113` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает dub effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
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
    let mut effects = WorldFactionDubEffects {
        game,
        callbacks,
        update_player,
        use_log_system,
        faction_title_log_enabled,
        write_faction_title_log,
    };
    crate::app::organsysmessage::dispatch_faction_dub(
        message,
        game,
        organizing,
        &mut effects,
    )
}

/// Ветви `0x60114/0x60115` обслуживает realm-модуль
/// (`crate::app::organsysmessage`); обвязка лишь собирает purview
/// effects адаптер старого пакета и передаёт ход realm-обработчику.
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
    let mut effects = WorldFactionPurviewEffects {
        game,
        callbacks,
        use_log_system,
        add_log_enabled,
        revoke_log_enabled,
        write_log,
    };
    crate::app::organsysmessage::dispatch_faction_purview(
        message,
        game,
        organizing,
        &mut effects,
    )
}

/// Decode-only ветви `0x60121/0x60123` обслуживает realm-модуль
/// (`crate::app::organsysmessage`); обвязка сохраняет прежнюю точку
/// входа main-loop.
pub(crate) fn dispatch_consumed_long(
    message: &mut CMessage,
) -> Option<OrganizingConsumedLongDispatch> {
    crate::app::organsysmessage::dispatch_consumed_long(message)
}

/// Ветвь `0x6011F` перенесена в realm (`crate::app::organsysmessage`)
/// двухфазно: realm разбирает шапку, обвязка выполняет decode игрока из
/// wire-хвоста прежним владельцем игры (общий заём effects-адаптера и
/// `&mut`-decode одной игры не проходят одну границу вызова), затем realm
/// завершает declaration и ответ в исходном порядке.
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
    let request = crate::app::organsysmessage::parse_declare_faction_war(message)?;
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            request.player_id as u32,
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
    let mut effects = WorldFactionWarDeclarationEffects::new(
        game,
        organizing,
        update_player,
    );
    Some(crate::app::organsysmessage::finish_declare_faction_war(
        request,
        player_online,
        faction_wars,
        &mut effects,
        sender,
    ))
}

/// Ветвь `0x60125` перенесена в realm (`crate::app::organsysmessage`)
/// вместе с OnceLock-инициализацией заголовков — singleton тот же
/// (`FACTION_BILLBOARD_TITLES` живёт у realm-контракта); обвязка лишь
/// связывает serialize-проход контроллера closure-швом.
pub(crate) fn dispatch_faction_billboard(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    world_string: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>> {
    let mut add_billboard_payload = |output: &mut Vec<u8>, billboard_type: i32| {
        organizing.add_faction_billboard_to_byte_array(output, billboard_type);
    };
    crate::app::organsysmessage::dispatch_faction_billboard(
        message,
        &mut add_billboard_payload,
        world_string,
        sender,
    )
}

/// Bridge-адаптер ветви `0x60126`: связывает game/goods/log владельцев
/// старого пакета для realm-обработчика. Inherent `upgrade_faction` держит
/// concrete `&CGame` одновременно с effects-адаптером, а decode игрока из
/// wire-хвоста — `&mut`-заём той же игры, поэтому вызовы разводятся
/// отдельными методами шва внутри одного адаптера.
pub(crate) struct WorldFactionUpgradeBridge<'game, 'view, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game mut CGame,
    pub(crate) organizing: &'game mut COrganizingCtrl,
    pub(crate) parameters: &'view COrganizingParam,
    pub(crate) registry: &'view GoodsBasePropertiesRegistry,
    pub(crate) original_name_index: &'view GoodsOriginalNameIndex,
    pub(crate) coefficients: &'view PlayerPropertyCoefficients,
    pub(crate) use_log_system: bool,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
}

impl crate::app::organsysmessage::OrganizingFactionUpgradeBridge
    for WorldFactionUpgradeBridge<'_, '_, '_, '_, '_>
{
    fn decode_online_player(
        &mut self,
        player_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            self.registry,
            self.coefficients,
        )
    }

    fn upgrade_faction(
        &mut self,
        faction_id: i32,
        player_id: i32,
    ) -> Result<Option<FactionUpgradeOutcome>, FactionUpgradeBlock> {
        let game_ref: &CGame = &*self.game;
        let mut effects = WorldFactionUpgradeEffects {
            game: game_ref,
            registry: self.registry,
            original_name_index: self.original_name_index,
            use_log_system: self.use_log_system,
            callbacks: &mut *self.callbacks,
            update_player: &mut *self.update_player,
        };
        self.organizing
            .upgrade_faction(game_ref, self.parameters, faction_id, player_id, &mut effects)
    }
}

/// Ветвь `0x60126` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом upgrade-входа контроллера; обвязка лишь собирает адаптер и
/// передаёт ход realm-обработчику в исходных типах.
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
    let mut bridge = WorldFactionUpgradeBridge {
        game,
        organizing,
        parameters,
        registry,
        original_name_index,
        coefficients,
        use_log_system,
        callbacks,
        update_player,
    };
    crate::app::organsysmessage::dispatch_faction_upgrade(message, &mut bridge)
}

/// Ветвь `0x60127` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает upload-icon effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_upload_icon(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>> {
    let mut effects = WorldFactionUploadIconEffects { game, callbacks };
    crate::app::organsysmessage::dispatch_faction_upload_icon(
        message,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60128` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает contributor effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_contributor(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>> {
    let mut effects = WorldFactionContributorEffects {
        game,
        callbacks,
        update_player,
    };
    crate::app::organsysmessage::dispatch_faction_contributor(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60129` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь раскрывает скаляры log-callback старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_experience(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>> {
    crate::app::organsysmessage::dispatch_faction_experience(
        message,
        game,
        organizing,
        use_log_system,
        callbacks.faction_experience_log_enabled,
        &mut *callbacks.write_faction_experience_log,
    )
}

// `OrganizingRegionParamDispatch` и `dispatch_region_param_update` живут в
// realm (`crate::app::organsysmessage`); имена доступны здесь через glob
// re-export выше.
/// Ветвь `0x6012E` region route перенесена в realm
/// (`crate::app::organsysmessage`): её game-контакты покрыты
/// `WorldGameView`; обвязка сохраняет прежнюю точку входа main-loop.
pub(crate) fn dispatch_region_route(
    message: &mut CMessage,
    game: &CGame,
) -> Option<OrganizingRegionRouteDispatch> {
    crate::app::organsysmessage::dispatch_region_route(message, game)
}

/// Bridge-адаптер ветви `0x60130`: связывает game/country/war/effects
/// владельцев старого пакета для realm-обработчика; inherent
/// `transfer_city_owner` остаётся единственным вызовом шва.
pub(crate) struct WorldCityTransferBridge<'a, Effects> {
    pub(crate) game: &'a CGame,
    pub(crate) countries: &'a CCountryHandler,
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) attack_city: &'a CAttackCitySys,
    pub(crate) village_war: &'a CVillageWarSys,
    pub(crate) effects: &'a mut Effects,
}

impl<Effects> crate::app::organsysmessage::OrganizingCityTransferBridge
    for WorldCityTransferBridge<'_, Effects>
where
    Effects: CityTransferEffects,
{
    type SessionReport = Effects::SessionReport;
    type SessionBlock = Effects::SessionBlock;

    fn transfer_city_owner(
        &mut self,
        requester_player_id: i32,
        target_faction_id: i32,
        region_id: i32,
    ) -> Result<
        CityTransferStartOutcome<Self::SessionReport>,
        CityTransferStartBlock<Self::SessionBlock>,
    > {
        self.organizing.transfer_city_owner(
            self.game,
            self.countries,
            self.attack_city,
            self.village_war,
            requester_player_id,
            target_faction_id,
            region_id,
            self.effects,
        )
    }
}

/// Ветвь `0x60130` перенесена в realm (`crate::app::organsysmessage`)
/// с bridge-швом transfer-входа контроллера; обвязка лишь собирает адаптер и
/// передаёт ход realm-обработчику в исходных типах.
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
    let mut bridge = WorldCityTransferBridge {
        game,
        countries,
        organizing,
        attack_city,
        village_war,
        effects,
    };
    crate::app::organsysmessage::dispatch_city_transfer(message, &mut bridge)
}

pub(crate) struct WorldVillageWarApplicationContext<'game, 'callbacks, 'effects> {
    pub(crate) game: &'game CGame,
    pub(crate) organizing: &'game COrganizingCtrl,
    pub(crate) organizing_parameters: &'game COrganizingParam,
    pub(crate) attack_city: &'game CAttackCitySys,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
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

/// Ветвь `0x60135` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает application-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
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
    let mut context = WorldVillageWarApplicationContext {
        game,
        organizing,
        organizing_parameters,
        attack_city,
        callbacks,
    };
    crate::app::organsysmessage::dispatch_village_war_application(
        message,
        village_war,
        &mut context,
        sender,
    )
}

pub(crate) struct CityWarEnemyMutationEffects<'game> {
    pub(crate) game: &'game CGame,
    pub(crate) enemy_id: i32,
    pub(crate) enemy_name: Vec<u8>,
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

pub(crate) struct WorldAttackCityApplicationContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game CGame,
    pub(crate) globe_setup: &'game GlobeSetupSnapshot,
    pub(crate) organizing: &'organizing mut COrganizingCtrl,
    pub(crate) organizing_parameters: &'game COrganizingParam,
    pub(crate) village_war: &'game CVillageWarSys,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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

pub(crate) fn bounded_city_war_notice(
    _string_id: &'static [u8],
    notice: Vec<u8>,
) -> Result<Vec<u8>, OrganizingCityWarApplicationBlock> {
    let notice = legacy_c_string_prefix(&notice);
    Ok(notice.to_vec())
}

/// Ветвь `0x60137` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает application-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
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
    let mut context = WorldAttackCityApplicationContext {
        game,
        globe_setup,
        organizing,
        organizing_parameters,
        village_war,
        callbacks,
        update_player,
    };
    crate::app::organsysmessage::dispatch_city_war_application(
        message,
        attack_city,
        &mut context,
        sender,
    )
}

/// Результат-контекст ветви `0x60138` и семейного reload: game/organizing
/// доступы идут напрямую в прежних owner-ах, а все country-контакты
/// (присутствие, флаг войны, governance-переход короля) — через узкий
/// [`WorldCountryWarGate`], чья реализация собирается у dispatcher-а в
/// `worldserver/game.rs` над живым `CCountryHandler`.
/// Результат-контекст ветви `0x60138` и семейного reload: game/organizing
/// доступы идут напрямую в прежних owner-ах, а все country-контакты
/// (присутствие, флаг войны, governance-переход короля) — через узкий
/// [`WorldCountryWarGate`], чья реализация собирается у dispatcher-а в
/// `worldserver/game.rs` над живым `CCountryHandler`. Gate хранится в
/// `Option` и изымается (`take`) ровно на один SetKing-переход, потому что
/// governance context-сеанс `CCountry::SetKing` обслуживает сам
/// результат-контекст (`self`), а `&mut dyn` gate-а и `&mut self` одного
/// адаптера одновременно заиметь нельзя; до и после перехода slot всегда
/// полон (см. лёгкие методы ниже).
pub(crate) struct WorldAttackCityResultContext<
    'game,
    'organizing,
    'country,
    'callbacks,
    'effects,
    'update,
    'configuration,
> {
    pub(crate) game: &'game mut CGame,
    pub(crate) organizing: &'organizing mut COrganizingCtrl,
    pub(crate) country_gate: Option<&'country mut dyn WorldCountryWarGate>,
    pub(crate) globe_setup: &'configuration GlobeSetupSnapshot,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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
        let gate = self
            .country_gate
            .as_deref_mut()
            .expect("country gate изымается только на SetKing-переход");
        let _ = gate.set_country_warring(country_id, false);
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
        Ok(self
            .country_gate
            .as_deref()
            .expect("country gate изымается только на SetKing-переход")
            .country_exists(country_id))
    }

 /// Тяжёлый governance-шов выполняет bridge-адаптер [`WorldCountryWarGate`] у
 /// dispatcher-а: связность `take/SetKing/city_id/restore` сохраняется одним
 /// вызовом, а governance context-сеанс `SetKing` по-прежнему обслуживает сам
 /// результат-контекст (`self`). Gate изымается из адаптера на один переход,
 /// чтобы `&mut dyn` gate-а и `&mut self` прошли раздельно, и сразу
 /// возвращается — прежний полный контекст SetKing буквально тот же `self`.
    fn set_country_king_and_city(
        &mut self,
        country_id: u8,
        master_id: i32,
        city_region_id: i32,
    ) -> Result<(), Self::Block> {
        let gate = self
            .country_gate
            .take()
            .expect("country gate изымается только на SetKing-переход");
        let outcome =
            gate.set_country_king_and_city(country_id, master_id, city_region_id, &mut *self);
        self.country_gate = Some(gate);
        outcome.map_err(|block| match block {
            WorldCountryKingGateBlock::MissingCountryOwner => {
                OrganizingCityWarResultContextBlock::MissingCountryOwner { country_id }
            }
            WorldCountryKingGateBlock::Governance(source) => {
                OrganizingCityWarResultContextBlock::CountryGovernance {
                    country_id,
                    source,
                }
            }
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

/// Ветвь `0x60138` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает результат-контекст адаптер старого пакета с country-
/// gate от dispatcher-а и передаёт ход realm-обработчику в исходных типах.
#[allow(
    clippy::too_many_arguments,
    reason = "явные параметры сохраняют границы исходных singleton-owner-ов"
)]

pub(crate) fn dispatch_city_war_result<Callback: Copy>(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    country_gate: &mut dyn WorldCountryWarGate,
    attack_city: &mut CAttackCitySys,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    globe_setup: &GlobeSetupSnapshot,
) -> Option<OrganizingCityWarResultDispatch> {
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_gate: Some(country_gate),
        globe_setup,
        callbacks: effects,
        update_player,
    };
    crate::app::organsysmessage::dispatch_city_war_result(
        message,
        attack_city,
        timer,
        attack_callbacks,
        &mut context,
    )
}

/// Соединяет concrete `CAttackCitySys::Reload` с World runtime owners; сам
/// reload-контракт живёт в realm (`crate::app::organsysmessage`).
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
    country_gate: &mut dyn WorldCountryWarGate,
    globe_setup: &GlobeSetupSnapshot,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Result<AttackCityReloadReport, AttackCityReloadBlock<OrganizingCityWarResultContextBlock>> {
    let sender = game.current_game_server_sender();
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_gate: Some(country_gate),
        globe_setup,
        callbacks: effects,
        update_player,
    };
    crate::app::organsysmessage::reload_attack_city(
        attack_city,
        source,
        now,
        timer,
        attack_callbacks,
        today_tax_event_id,
        &mut context,
        |message| message.send_all(sender.as_ref()).unwrap_or(0),
    )
}

/// Ветви `0x6013B/0x6013C` quest-команд обслуживает realm-модуль
/// (`crate::app::organsysmessage`): их game-контакты покрыты
/// `WorldGameView`; обвязка сохраняет прежнюю точку входа main-loop.
pub(crate) fn dispatch_player_quest_command(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerQuestCommandDispatch> {
    crate::app::organsysmessage::dispatch_player_quest_command(message, game, sender)
}

/// Ветвь `0x6013D` run-script перенесена в realm
/// (`crate::app::organsysmessage`); обвязка сохраняет прежнюю точку
/// входа main-loop.
pub(crate) fn dispatch_player_run_script(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerRunScriptDispatch> {
    crate::app::organsysmessage::dispatch_player_run_script(message, game, sender)
}

/// Ветвь `0x6013E` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает set-parameter effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_parameter(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>> {
    let mut effects = WorldFactionSetParameterEffects {
        game,
        callbacks,
        update_player,
    };
    crate::app::organsysmessage::dispatch_faction_parameter(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60144` change region router перенесена в realm
/// (`crate::app::organsysmessage`): её владелец маршрутов — тот же
/// Shared `RegionRouter`; обвязка сохраняет прежнюю точку входа main-loop.
pub(crate) fn dispatch_change_region_router(
    message: &mut CMessage,
    router: &RegionRouter,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingChangeRegionRouterDispatch> {
    crate::app::organsysmessage::dispatch_change_region_router(message, router, sender)
}

pub(crate) struct WorldVillageWarResultContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    pub(crate) game: &'game CGame,
    pub(crate) organizing: &'organizing mut COrganizingCtrl,
    #[allow(dead_code, reason = "несущее поле исходного организационного адаптера: конструируется для исходной связки обработчиков, чтения в Rust-рёбрах нет (в старом пакете предупреждение глушил модульный allow)")]
    pub(crate) callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    pub(crate) update_player: &'update mut dyn FnMut(i32),
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

/// Ветвь `0x60136` перенесена в realm (`crate::app::organsysmessage`);
/// обвязка лишь собирает результат-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
#[allow(
    clippy::too_many_arguments,
    reason = "timer/callbacks/effects/update грань подтверждена CVillageWarSys::OnFactionWinVillage"
)]

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
    let mut context = WorldVillageWarResultContext {
        game,
        organizing,
        callbacks: effects,
        update_player,
    };
    crate::app::organsysmessage::dispatch_village_war_result(
        message,
        village_war,
        timer,
        callbacks,
        &mut context,
    )
}



/// Адаптер [`WorldCountryMutGate`] поверх живого `CCountryHandler`: ровно те
/// же `get_country_mut`/`get_country` цепочки, что выполнял прежний
/// диспетчер. Тик `legacy_tick_ms` для exile-lookup снимается в исходной
/// позиции — внутри ветки найденной страны, до вызова
/// `CCountry::exile_remaining_time`; отсутствующая страна тик не тратит.
pub(crate) struct CountryHandlerMutGate<'a> {
    pub(crate) handler: &'a mut CCountryHandler,
}

impl WorldCountryMutGate for CountryHandlerMutGate<'_> {
    fn apply_server_scalar(
        &mut self,
        country: u8,
        selector: i8,
        value: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<Option<CountryScalarUpdate>, CountryParameterUnavailable>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.apply_server_scalar(selector, value, country_parameters))
    }

    fn set_quest_switch(
        &mut self,
        country: u8,
        job: u8,
        enabled: bool,
    ) -> Option<Option<CountryQuestSwitchUpdate>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.set_quest_switch(job, enabled))
    }

    fn exile_remaining_time(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<CountryExileTimeLookup, CountryParameterUnavailable>> {
        self.handler.get_country(country).map(|country| {
            country.exile_remaining_time(player_id, legacy_tick_ms(), country_parameters)
        })
    }
}

/// Хвостовые ветви `OnCountryMessage` (`0x6030F`, `0x6031B`,
/// `0x60314..0x60316`, relay `0x60310`/`0x60311`, no-op) обслуживает realm
/// `crate::app::countrymessage::on_country_message`, который уже
/// возвращает общий `WorldCountryMessageOutcome`; здесь только подача
/// адаптера `CCountryHandler` в шов [`WorldCountryMutGate`].
pub(crate) fn on_country_message(
    game: &CGame,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    message: CMessage,
) -> WorldCountryMessageDispatch {
    let mut gate = CountryHandlerMutGate {
        handler: country_handler,
    };
    WorldCountryMessageDispatch::Handled(
        crate::app::countrymessage::on_country_message(
            game,
            &mut gate,
            country_parameters,
            globe_setup,
            message,
        ),
    )
}

/// Адаптер [`WorldCountryGovernanceGate`] поверх живого `CCountryHandler`:
/// ровно те цепочки `get_country`/`get_country_mut → метод CCountry`, что
/// выполнял прежний диспетчер governance-ветвей. Receiver gate-метода
/// повторяет receiver метода `CCountry`: `&self`-шаги идут через
/// `get_country`, `&mut self`-шаги через `get_country_mut`; единая
/// `get_country_mut`-разрешение ветви машинного оригинала поведения не
/// меняет — за время ветви таблица недоступна никому (заимствование).
pub(crate) struct CountryHandlerGovernanceGate<'a> {
    pub(crate) handler: &'a mut CCountryHandler,
}

#[allow(clippy::type_complexity, reason = "вложенные формы повторяют исходные цепочки get_country_mut/get_country один к одному")]
impl WorldCountryGovernanceGate for CountryHandlerGovernanceGate<'_> {
    fn authorize_king(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_king(candidate, context))
    }

    fn authorize_minister(
        &self,
        country: u8,
        player_id: i32,
        job: u8,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_minister(player_id, job, context))
    }

    fn authorize_king_for_players(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_king_for_players(candidate, context))
    }

    fn can_exile(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanExileDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_exile(country_parameters, context))
    }

    fn exile(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryExileRequestDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.exile(player_id, country_parameters, context))
    }

    fn can_silence(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanSilenceDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_silence(country_parameters, context))
    }

    fn silence(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySilenceReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.silence(player_id, country_parameters, context))
    }

    fn can_absolve(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAbsolveDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_absolve(country_parameters, context))
    }

    fn absolve(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAbsolveReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.absolve(player_id, country_parameters, context))
    }

    fn can_depose_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDeposeMinisterDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_depose_minister(country_parameters, context))
    }

    fn depose_minister(
        &mut self,
        country: u8,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDeposeMinisterReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.depose_minister(job, mode, country_parameters, context))
    }

    fn can_appoint_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAppointMinisterDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_appoint_minister(country_parameters, context))
    }

    fn appoint_minister(
        &mut self,
        country: u8,
        player_id: i32,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAppointMinisterReport> {
        self.handler.get_country_mut(country).map(|country| {
            country.appoint_minister(player_id, job, mode, country_parameters, context)
        })
    }

    fn can_demise(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDemiseDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_demise(country_parameters, context))
    }

    fn demise(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDemiseReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.demise(player_id, country_parameters, context))
    }

    fn get_info(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryBaseInfoDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.get_info(country_parameters, context))
    }

    fn get_players_list(
        &self,
        country: u8,
        page: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<Result<CountryPlayersListReport, CountryPlayersListContextBlock>> {
        self.handler
            .get_country(country)
            .map(|country| country.get_players_list(page, context))
    }

    fn set_control_point(
        &mut self,
        country: u8,
        requested: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<KingPointUpdate, CountryParameterUnavailable>> {
        self.handler
            .get_country_mut(country)
            .map(|country| set_control_point(&mut country.king, requested, country_parameters))
    }

    fn set_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<Result<CountrySetKingReport, CountryGovernanceContextBlock>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.set_king(player_id, country_parameters, context))
    }

    fn register_initial_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryInitialKingReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.register_initial_king(player_id, country_parameters, context))
    }

    fn success_exiled(
        &mut self,
        country: u8,
        player_id: i32,
        success: bool,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySuccessExiledReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.success_exiled(player_id, success, country_parameters, context))
    }

    fn set_new_day(
        &mut self,
        requested_day: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountrySetNewDayContext,
    ) -> CountryHandlerNewDayReport {
        self.handler
            .set_new_day(requested_day, country_parameters, context)
    }
}

/// Адаптер [`WorldCountryView`] поверх `CCountryHandler` ветви `0x60301`:
/// ровно `get_country(...).is_some()` прежнего замыкания `country_exists`,
/// по прецеденту `CreateRoleCountryViewAdapter` в `logmessage.rs`.
pub(crate) struct CountryHandlerExistsView<'a> {
    pub(crate) handler: &'a CCountryHandler,
}

impl WorldCountryView for CountryHandlerExistsView<'_> {
    fn country_exists(&self, country: u8) -> bool {
        self.handler.get_country(country).is_some()
    }
}

/// Bridge [`WorldCountryPlayerChangeView`] владельца игры: делегирует
/// одноимённый inherent-метод `CGame`; closure-предикат передаётся без
/// обёрток — `&mut dyn FnMut(u8) -> bool` удовлетворяет исходному
/// `impl FnOnce(u8) -> bool` через std-blanket для `&mut F`.
impl WorldCountryPlayerChangeView for CGame {
    fn change_online_player_country(
        &mut self,
        player_id: u32,
        requested_country: u8,
        country_exists: &mut dyn FnMut(u8) -> bool,
    ) -> Option<PlayerCountryChangeReport> {
        CGame::change_online_player_country(self, player_id, requested_country, country_exists)
    }
}

/// Ветвь `0x6030E` перенесена в realm; обёртка сохраняет прежнюю сигнатуру и
/// отображение исхода один к одному.
pub(crate) fn dispatch_country_exile_result_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryExileResultSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_exile_result_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030D` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_exile_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryExileRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_exile_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030C` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_silence_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountrySilenceRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_silence_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030B` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_absolve_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryAbsolveRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_absolve_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030A` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_depose_minister_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDeposeMinisterSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_depose_minister_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60309` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_appoint_minister_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryAppointMinisterSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_appoint_minister_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60307` перенесена в realm; обёртка сохраняет прежнюю сигнатуру,
/// только shared handler заменён на mutable для адаптера шва.
pub(crate) fn dispatch_country_players_list_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    context: &mut dyn CountryPlayersListContext,
) -> Option<WorldCountryPlayersListSync> {
    let countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_players_list_message(
        message,
        &countries,
        context,
    )
}

/// Ветвь `0x60301` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_player_change_message(
    message: &mut CMessage,
    game: &mut CGame,
    country_handler: &CCountryHandler,
) -> Option<WorldCountryPlayerChangeSync> {
    let countries = CountryHandlerExistsView {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_player_change_message(
        message,
        game,
        &countries,
    )
}

/// Ветвь `0x60313` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_new_day_message(
    message: &CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountrySetNewDayContext,
) -> Option<WorldCountryNewDaySync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_new_day_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60304` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_direct_appointment_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDirectAppointmentSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_direct_appointment_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60306` перенесена в realm; обёртка сохраняет прежнюю сигнатуру,
/// только shared handler заменён на mutable для адаптера шва.
pub(crate) fn dispatch_country_info_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryInfoSync> {
    let countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_info_message(
        message,
        &countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60308` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_demise_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDemiseSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    crate::app::countrymessage::dispatch_country_demise_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}
