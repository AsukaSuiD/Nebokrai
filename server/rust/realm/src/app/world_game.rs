//! Тип `CGame` старого WorldServer из `worldserver/game.cpp/.h` (сборка
//! `worldserver.exe`/`worldserver.pdb`), перенесённый в Realm волной C5-C.
//! До волны тип жил в старом пакете `worldserver/worldserver/game.rs`; старый
//! файл теперь glob-shim над `nebokrai_realm::app::{world_game, world_game_init,
//! world_main_loop, world_reload, world_dispatch}`.
//!
//! Здесь — объявление структуры, конструктор `new`, hub-таблицы и accessors,
//! timer/effect glue (WorldTimerHandler, war/city timer-контексты) и impl-ы
//! Realm-швов (`WorldGameView` и соседи), включая бывшие
//! `EnemyFactionSink`/`CountrySaveSink`/`OrganizingSaveSink`/`HonorRanksGameView`
//! и `WorldGameThreadGame`. Inherent-тела загрузки/хода/reload/save/dispatch
//! лежат в соседних файлах `world_game_init`, `world_main_loop`, `world_reload`,
//! `world_dispatch` и `persistence::world_db_data_collect`.
//!
//! Нормализации волны (тела методов совпадают с переносимым состоянием):
//! пути старого пакета заменены на Realm/Shared эквиваленты
//! (`crate::worldserver::appworld::*` -> `crate::{organizations, activities,
//! characters, content, regions, sessions, billing, auction}::*`,
//! `crate::dbaccess::worlddb::*` -> `crate::{persistence, organizations,
//! activities, characters, regions}::*`, `crate::nets::*` -> `crate::app::world_*`
//! и `nebokrai_shared::network`, `crate::public::*`/`crate::setup::*` ->
//! `nebokrai_shared::{resources, runtime, values}`); ссылки `nebokrai_realm::`
//! записаны через `crate::`; полям `CGame` добавлен `pub(crate)` (impl-блоки
//! разнесены по соседям), inherent-методам — `pub` (переходная нормализация:
//! старый process-owner `runtime.rs` и shim-пакет обращаются к ним до волны
//! C5-D), glue-типам швов — `pub` как участникам pub-сигнатур; desugared-формы
//! (`&dyn` view-швы) соответствуют ADR-0013.
//!
//! Генератор системных broadcast-maintenance полагается на CRT `rand` след:
//! `0x453560` (закреплён прежними волнами, см. `world_reload`).

use crate::activities::attackcitysys::{AttackCityCallbackKind, AttackCityCallbacks, AttackCityCountdownContext, AttackCityCountdownRequest, AttackCityPhaseContext, AttackCityPhaseEffect, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarCallbackKind, CountryWarCallbacks, CountryWarPhase, CountryWarSys, CountryWarTopInfoKind};
use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarCalendarBlock, FourNationWarCallbackContext, FourNationWarCallbackKind, FourNationWarCallbacks, FourNationWarRegionIndexBlock};
use crate::activities::jjcsystem::CJJcSystem;
use crate::activities::leiting::LeiTingLocalTime;
use crate::activities::misc::{CopyNumberResetReport, CopyNumberTimerState};
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarAnnouncement, VillageWarCallbackKind, VillageWarCallbacks, VillageWarCountdownContext, VillageWarCountdownRequest, VillageWarPhaseContext};
use crate::app::baitan::{WorldBaiTanCompletion, WorldBaiTanLists, WorldBaiTanRegistration, WorldBaiTanRemoval, WorldDoneBaiTanListReport};
use crate::app::gmmessage::{WorldNamedRegionLookup, WorldNamedRegionMatch, WorldRegionIdRoute, WorldRegionIdRouteScan};
use crate::app::loginreconnectworker::{WorldLoginReconnectThreadRestart, WorldLoginReconnectWorker};
use crate::app::misc_game::legacy_tick_ms;
use crate::app::servermessage::WorldCompletedSaveResponseLaunchReport;
use crate::app::world_client::CMyNetClient;
use crate::app::world_dispatch::{WorldCountryInfoDelivery, WorldCountryWarEffects, add_legacy_c_string, copy_name_for_legacy_lowercase, legacy_c_string_prefix};
use crate::app::world_game_view::{WorldCreateRoleLaunchFailure, WorldCreateRoleLaunchSuccess, WorldGameServerConnectionState, WorldGameServerDisconnectionState, WorldLoginAccountPlayer, WorldLoginPlayerRouteSnapshot, WorldLoginTimeoutTeamExit, WorldOnlineAccountPlayerRoute, WorldPlayerLoadRequestBlock, WorldPlayerLoadRequestOutcome, WorldPlayerSelectRouteBlock, WorldPlayerSelectRouteError, WorldPlayerSelectRouteOutcome, WorldProcessPlayerDataQueueBlock, WorldProcessPlayerDataQueueError, WorldProcessPlayerDataQueueOutcome, WorldRegionNameLookup, WorldRegionParamUpdateOutcome, WorldReturnedPlayerDecode, WorldReturnedPlayerDecodeOwner, WorldReturnedPlayerSnapshot};
use crate::app::world_hub_data::WorldLoadedPlayerRouteOrder;
use crate::app::world_hub_entries::{WorldAuctionSellerMoney, WorldDetachedFactionInfoContext, WorldFactionPlayerOrganizingContext, WorldGameServerEntry, WorldLoginPlayerEntry, WorldOriginGoodsBlock, WorldOriginGoodsReport, WorldPlayerFactionInfoContext, WorldPlayerOrganizingContext, WorldRegionAssignment, WorldSystemBroadcast, truncate_legacy_money, truncate_scaled_legacy_money};
use crate::app::world_main_loop_data::{AttackCityTimerOutcome, AttackCityTimerReport, CountryWarTimerBlock, CountryWarTimerReport, FourNationWarTimerReport, PlayerRanksTimerRefreshBlock, PlayerRanksTimerRefreshReport, VillageWarTimerOutcome, VillageWarTimerReport, WorldTimerCallbackBlock};
use crate::app::world_message::{CMessage, SendMessageError, WorldLocalMessageQueueBlock};
use crate::app::world_runtime::{WorldGameReleaseContext, WorldGameReleaseResult, WorldRegionOwner};
use crate::app::world_server::CMyNetServer;
use crate::app::world_setup::WorldSetup;
use crate::app::worldothermessage::{WorldGoodsLink, WorldGoodsLinkPayload, WorldHonorEliminatorRegistration, WorldPlayerNameChangeDisposition, WorldPlayerNameChangeReport, WorldPlayerNameLookupError};
use crate::app::worldserver::{WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldErrorLogDelivery, WorldGameServerLookupError, WorldGameServerLostReport, WorldGenerateDbDataBlock, WorldGlobeVariables, WorldGlobeVariablesDelivery, WorldInitialRegionSnapshot, WorldInitialRegionSnapshotBlock, WorldInitialRegionSnapshotKind, WorldInitialRegionSnapshotSource, WorldLogLocalTime, WorldLogTextOwner, WorldLostGameServerPlayer, WorldOnlinePlayerAppendOutcome, WorldOnlinePlayerRemoveOutcome, WorldPingGameServerInfo, WorldPlayerSaveResponseProgress, WorldReceivedPlayerDataRead, WorldReceivedPlayerDataUpdate, WorldReconnectedPlayerDecode, WorldReconnectedPlayerOwner, WorldRegionChangePlayerTransition, WorldRegionChangeTeamUpdate, WorldRegionParamDecodeOutcome, WorldReloadContext, WorldReloadResult, WorldSaveThreadHandleState, WorldServerSnapshotPlayerDecode, WorldServerSnapshotPlayerOwner, send_err_log_to_login};
use crate::persistence::writelog::WorldWriteLogCommand;
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerCodecError, PlayerCountryChangeReport, PlayerDbProjectionBlock, PlayerEquipmentWireSnapshot, PlayerExploitUpdate, PlayerFactionInfoUpdateBlock, PlayerFactionInfoUpdateReport, PlayerLeiTingClock, PlayerLeiTingUpdateBlock, PlayerLeiTingUpdateReport, PlayerLoadDataOutcome, PlayerLoadDataOwner, PlayerMurderCounterReset, PlayerMurderCounterUpdate, PlayerPropertyCoefficients};
use crate::characters::playerdataqueue::CPlayerDataQueue;
use crate::characters::playerloadqueue::{CPlayerLoadQueue, PLAYER_LOAD_CDKEY_CAPACITY, PlayerLoadPushOutcome, PlayerLoadQueueEntry};
use crate::characters::playerloadworker::{WorldPlayerDataLoadOwner, WorldPlayerLoadWorkerPool};
use crate::characters::playerranks::CPlayerRanks;
use crate::content::{QuestCatalog, ScriptResources};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::cgoodsfactory::{GoodsOriginalNameIndex};
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::CSkillFactory;
use crate::content::{TimeToReturn, TimeToReturnCallbacks, TimeToReturnContext, TimeToReturnFireReport};
use crate::content::variablelist::CVariableList;
use crate::organizations::country::CountryKingSaveLimits;
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::dbcountry::CountrySaveSnapshot;
use crate::organizations::faction::{CFaction, FactionInitialPropertyBlock};
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::{COrganizingCtrl, OrganizingDisbandPlayer, OrganizingNameLookupBlock};
use crate::organizations::organizingparam::{COrganizingParam, OrganizingTodayTaxRefreshReport, PreparedTodayTaxRefresh};
use crate::organizations::rsenemyfactions::EnemyFactionSaveSnapshot;
use crate::organizations::union::{CUnion, UnionFormatArgument};
use crate::persistence::rsplayer::RsPlayerOwner;
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::savedata::{DeletionPlayerSnapshot, WorldDbData};
use crate::persistence::savedb::SaveDataLifecycleState;
use crate::persistence::saveworker::WorldSaveRuntimeContext;
use crate::persistence::writelogqueue::WorldWriteLogQueue;
use crate::persistence::writelogworker::WorldWriteLogWorker;
use crate::regions::region::CRegion;
use crate::regions::rsregion::{RegionDatabaseParameters, RegionParameterLoadTarget};
use crate::regions::worldregion::CWorldRegion;
use crate::sessions::csessionfactory::CSessionFactory;
use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{CCiQingSetup, CContributeSetup, CDupliRegionSetup, CEmotion, CGodsBattleConf, CHitLevelSetup, CIncrementShopList, CPlayerList, CQuestSystem, CTaoZhuangSetup, CThingSetup, CTradeList, CWordsFilter, EquipmentComposeList, GlobeSetupSnapshot, MyStringTable, PrisonConf};
use nebokrai_shared::runtime::{AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, CTimer, CalendarTimerRegistration, TimerCallbackInvocation, TimerCallbackSource, TimerId, put_string_to_file};
use nebokrai_shared::values::TagTime;
use parking_lot::Mutex;
use std::fmt;
use std::collections::{BTreeMap, VecDeque};
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::app::world_dispatch::format_union_world_string;

/// Связывает полный `CPlayer::LoadData` с bool-контрактом фонового World worker-а.
///
/// DB-owner, player-list и setup snapshots остаются явно принадлежащими
/// вызывающему коду. Это заменяет только process-global singleton lookup-и;
/// порядок `CRsPlayer::LoadPlayer` и последующей post-load стадии не меняется.
pub struct WorldPlayerLoadDataAdapter<'owner, Loader> {
    pub(crate) loader: &'owner mut Loader,
    pub(crate) player_list: &'owner mut CPlayerList,
    pub(crate) globe_setup: &'owner GlobeSetupSnapshot,
    pub(crate) coefficients: &'owner PlayerPropertyCoefficients,
}

impl<'owner, Loader> WorldPlayerLoadDataAdapter<'owner, Loader> {
    pub fn new(
        loader: &'owner mut Loader,
        player_list: &'owner mut CPlayerList,
        globe_setup: &'owner GlobeSetupSnapshot,
        coefficients: &'owner PlayerPropertyCoefficients,
    ) -> Self {
        Self {
            loader,
            player_list,
            globe_setup,
            coefficients,
        }
    }
}

impl<Loader> WorldPlayerDataLoadOwner<CPlayer> for WorldPlayerLoadDataAdapter<'_, Loader>
where
    Loader: PlayerLoadDataOwner,
{
    fn load_player_data<'a>(
        &'a mut self,
        player: &'a mut CPlayer,
    ) -> impl Future<Output = bool> + 'a {
        async move {
            let player_id = player.get_id();
            let outcome = player
                .load_data(
                    self.loader,
                    self.player_list,
                    self.globe_setup,
                    self.coefficients,
                )
                .await;
            match outcome {
                PlayerLoadDataOutcome::Loaded(_) => {
                    tracing::debug!(
                        player_id,
                        region_id = player.get_region_id(),
                        "World завершил загрузку персонажа"
                    );
                    true
                }
                PlayerLoadDataOutcome::ReturnedFalse => {
                    tracing::error!(
                        player_id,
                        stage = "database",
                        outcome = "returned_false",
                        "World не загрузил персонажа"
                    );
                    false
                }
                PlayerLoadDataOutcome::BlockedDatabase(_) => {
                    tracing::error!(
                        player_id,
                        stage = "database",
                        outcome = "blocked",
                        "World не загрузил персонажа"
                    );
                    false
                }
                PlayerLoadDataOutcome::BlockedProperty(source) => {
                    tracing::error!(
                        player_id,
                        stage = "post_load_property",
                        error = %source,
                        "World не рассчитал характеристики загруженного персонажа"
                    );
                    false
                }
            }
        }
    }
}

pub(crate) struct WorldFourNationWarTimerEffects<'a, GetTick> {
    pub(crate) game: &'a CGame,
    pub(crate) country_handler: &'a mut CCountryHandler,
    pub(crate) log: &'a mut WorldLogTextOwner,
    pub(crate) get_tick: &'a mut GetTick,
    pub(crate) get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub(crate) put_log_info: &'a mut dyn FnMut(&[u8]),
}

pub(crate) struct WorldTimeToReturnEffects<'a> {
    pub(crate) game: &'a CGame,
}

pub(crate) struct WorldTerritoryWarTimerEffects<'a, GetTick> {
    pub(crate) game: &'a CGame,
    pub(crate) country_handler: &'a mut CCountryHandler,
    pub(crate) get_tick: &'a mut GetTick,
}

impl<GetTick: FnMut() -> u32> WorldTerritoryWarTimerEffects<'_, GetTick> {
    pub(crate) fn region_name(&self, region_id: i32) -> Option<Vec<u8>> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound | WorldRegionNameLookup::NullRegionPointer => {
                None
            }
        }
    }

    fn format(&self, string_id: &[u8], arguments: &[UnionFormatArgument<'_>]) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn send_organizing_info(&self, text: &[u8]) {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            (-366_i32) as u32,
            0xFFFF_0000,
        );
    }

    fn publish_countdown(&mut self, duration_ms: i32, text: &[u8]) {
        let info_id = self
            .country_handler
            .add_one_top_info(2, duration_ms, text, &mut *self.get_tick);
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        let _ = self.country_handler.send_top_info_to_client(
            info_id,
            2,
            duration_ms,
            text,
            &mut delivery,
        );
    }
}

impl<GetTick: FnMut() -> u32> AttackCityPhaseContext
    for WorldTerritoryWarTimerEffects<'_, GetTick>
{
    type Block = Infallible;

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }

    fn apply_effect(&mut self, effect: AttackCityPhaseEffect) -> Result<(), Self::Block> {
        match effect {
            AttackCityPhaseEffect::RegionAnnouncement {
                war_number,
                city_region_id,
                notice_string_id,
                log_string_id,
                set_region_country_warring,
            } => {
                let Some(region_name) = self.region_name(city_region_id) else {
                    return Ok(());
                };
                let notice = self.format(
                    notice_string_id,
                    &[UnionFormatArgument::Text(&region_name)],
                );
                self.send_organizing_info(&notice);
                if set_region_country_warring
                    && let Some(country_id) = self.game.region_country_id(city_region_id)
                    && let Some(country) = self.country_handler.get_country_mut(country_id)
                {
                    country.is_warring = true;
                }
                let log = self.format(
                    log_string_id,
                    &[
                        UnionFormatArgument::Signed(war_number),
                        UnionFormatArgument::Text(&region_name),
                    ],
                );
                put_string_to_file("war", &log);
            }
            AttackCityPhaseEffect::EndLog {
                war_number,
                log_string_id,
            } => {
                let log = self.format(
                    log_string_id,
                    &[UnionFormatArgument::Signed(war_number)],
                );
                put_string_to_file("war", &log);
            }
        }
        Ok(())
    }
}

impl<GetTick: FnMut() -> u32> AttackCityCountdownContext
    for WorldTerritoryWarTimerEffects<'_, GetTick>
{
    type Block = Infallible;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn publish_countdown(
        &mut self,
        request: AttackCityCountdownRequest,
    ) -> Result<(), Self::Block> {
        if let Some(region_name) = self.region_name(request.city_region_id) {
            let text = self.format(
                request.world_string_id,
                &[UnionFormatArgument::Text(&region_name)],
            );
            self.publish_countdown(request.duration_ms, &text);
        }
        Ok(())
    }
}

impl<GetTick: FnMut() -> u32> VillageWarPhaseContext
    for WorldTerritoryWarTimerEffects<'_, GetTick>
{
    type Block = Infallible;

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }

    fn announce(&mut self, request: VillageWarAnnouncement) -> Result<(), Self::Block> {
        let Some(region_name) = self.region_name(request.war_region_id) else {
            return Ok(());
        };
        let text = self.format(
            request.world_string_id,
            &[UnionFormatArgument::Text(&region_name)],
        );
        self.send_organizing_info(&text);
        if request.set_all_countries_warring {
            for country_id in 1..=4 {
                if let Some(country) = self.country_handler.get_country_mut(country_id) {
                    country.is_warring = true;
                }
            }
        }
        put_string_to_file("war", &text);
        Ok(())
    }
}

impl<GetTick: FnMut() -> u32> VillageWarCountdownContext
    for WorldTerritoryWarTimerEffects<'_, GetTick>
{
    type Block = Infallible;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn publish_countdown(
        &mut self,
        request: VillageWarCountdownRequest,
    ) -> Result<(), Self::Block> {
        if let Some(region_name) = self.region_name(request.war_region_id) {
            let text = self.format(
                request.world_string_id,
                &[UnionFormatArgument::Text(&region_name)],
            );
            self.publish_countdown(request.duration_ms, &text);
        }
        Ok(())
    }
}

impl TimeToReturnContext for WorldTimeToReturnEffects<'_> {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> Option<i32> {
        let map_id = self.game.game_server_number_by_region_id(region_id);
        (map_id != 0).then_some(map_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }
}

impl<GetTick: FnMut() -> u32> WorldFourNationWarTimerEffects<'_, GetTick> {
    pub(crate) fn format(&self, string_id: &[u8], arguments: &[UnionFormatArgument<'_>]) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }
}

impl<GetTick: FnMut() -> u32> FourNationWarCallbackContext
    for WorldFourNationWarTimerEffects<'_, GetTick>
{
    fn send_all(&mut self, message: &CMessage) {
        let _ = message.send_all(self.game.current_game_server_sender().as_ref());
    }

    fn region_name(&mut self, region_id: i32) -> Option<Vec<u8>> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound | WorldRegionNameLookup::NullRegionPointer => {
                None
            }
        }
    }

    fn war_start_notice(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0022", &[UnionFormatArgument::Text(region_name)])
    }

    fn war_start_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8> {
        self.format(
            b"XBWS0023",
            &[
                UnionFormatArgument::Signed(index),
                UnionFormatArgument::Text(region_name),
            ],
        )
    }

    fn send_organizing_info(&mut self, text: &[u8], color: u32, trailing: u32) {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            color,
            trailing,
        );
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }

    fn enter_start_notice(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0027", &[UnionFormatArgument::Text(region_name)])
    }

    fn enter_start_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8> {
        self.format(
            b"XBWS0028",
            &[
                UnionFormatArgument::Signed(index),
                UnionFormatArgument::Text(region_name),
            ],
        )
    }

    fn enter_end_notice(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0029", &[UnionFormatArgument::Text(region_name)])
    }

    fn enter_end_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8> {
        self.format(
            b"XBWS0030",
            &[
                UnionFormatArgument::Signed(index),
                UnionFormatArgument::Text(region_name),
            ],
        )
    }

    fn sign_up_end_log(&mut self, _index: i32, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0026", &[UnionFormatArgument::Text(region_name)])
    }

    fn sign_up_start_notice(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0024", &[UnionFormatArgument::Text(region_name)])
    }

    fn sign_up_start_timed_text(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0025", &[UnionFormatArgument::Text(region_name)])
    }

    fn sign_up_start_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8> {
        format_union_world_string(
            b"(Num:%d)[%s]Four Nation War System start!!.",
            &[
                UnionFormatArgument::Signed(index),
                UnionFormatArgument::Text(region_name),
            ],
        )
    }

    fn war_end_started_log(&mut self) -> Vec<u8> {
        self.game.get_string_by_id(b"XBWS0032").to_vec()
    }

    fn add_log_text(&mut self, text: &[u8]) {
        let _ = self.log.add_log_text_no_arguments(
            text,
            self.game.setup.save_info_time_ms,
            &mut *self.get_tick,
            &mut *self.get_log_local_time,
            &mut *self.put_log_info,
        );
    }

    fn war_end_notice(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0033", &[UnionFormatArgument::Text(region_name)])
    }

    fn war_end_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8> {
        self.format(
            b"XBWS0034",
            &[
                UnionFormatArgument::Signed(index),
                UnionFormatArgument::Text(region_name),
            ],
        )
    }

    fn current_time(&mut self) -> TagTime {
        TagTime::local_now()
    }

    fn war_end_info_text(&mut self, region_name: &[u8]) -> Vec<u8> {
        self.format(b"XBWS0031", &[UnionFormatArgument::Text(region_name)])
    }

    fn add_timed_top_info(&mut self, timer_flag: i32, milliseconds: i32, text: &[u8]) -> i32 {
        self.country_handler
            .add_one_top_info(timer_flag, milliseconds, text, &mut *self.get_tick)
    }

    fn send_timed_top_info(
        &mut self,
        info_id: i32,
        timer_flag: i32,
        milliseconds: i32,
        text: &[u8],
    ) {
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        let _ = self.country_handler.send_top_info_to_client(
            info_id,
            timer_flag,
            milliseconds,
            text,
            &mut delivery,
        );
    }
}

/// Adapter-путь timer-стадии: DB-owner статистики входит generic-параметром
/// `RsPlayer` (шов `RsPlayerOwner<CPlayer>`), как в перенесённой связке хода;
/// глубокие точки `process_*` сохраняют прежнюю конкретную декларацию.
pub(crate) struct WorldTimerHandler<'a, Callback, RsPlayer> {
    pub(crate) game: &'a CGame,
    pub(crate) attack_city: &'a mut CAttackCitySys,
    pub(crate) attack_city_callbacks: AttackCityCallbacks<Callback>,
    pub(crate) village_war: &'a mut CVillageWarSys,
    pub(crate) village_war_callbacks: VillageWarCallbacks<Callback>,
    pub(crate) country_war: &'a mut CountryWarSys,
    pub(crate) country_handler: &'a mut CCountryHandler,
    pub(crate) country_war_callbacks: CountryWarCallbacks<Callback>,
    pub(crate) four_nation_war: &'a mut CFourNationWarSys,
    pub(crate) four_nation_war_callbacks: FourNationWarCallbacks<Callback>,
    pub(crate) time_to_return: &'a mut TimeToReturn,
    pub(crate) time_to_return_callbacks: TimeToReturnCallbacks<Callback>,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
    pub(crate) organizing_parameters: &'a mut COrganizingParam,
    pub(crate) player_ranks: &'a mut CPlayerRanks,
    pub(crate) rs_player: &'a mut RsPlayer,
    pub(crate) player_database: Option<&'a mut WorldTdsClient>,
    pub(crate) organizing: &'a COrganizingCtrl,
    pub(crate) copy_number_timer: &'a mut CopyNumberTimerState,
    pub(crate) log: &'a mut WorldLogTextOwner,
    pub(crate) get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub(crate) put_log_info: &'a mut dyn FnMut(&[u8]),
    pub(crate) copy_number_resets: Vec<CopyNumberResetReport>,
    pub(crate) refreshes: Vec<PlayerRanksTimerRefreshReport>,
    pub(crate) tax_refreshes: Vec<OrganizingTodayTaxRefreshReport>,
    pub(crate) time_to_returns: Vec<TimeToReturnFireReport>,
    pub(crate) attack_city_wars: Vec<AttackCityTimerReport>,
    pub(crate) village_wars: Vec<VillageWarTimerReport>,
    pub(crate) country_wars: Vec<CountryWarTimerReport>,
    pub(crate) four_nation_wars: Vec<FourNationWarTimerReport>,
    pub(crate) pending_copy_number_registration: Option<usize>,
    pub(crate) pending_player_ranks_registration: Option<usize>,
    pub(crate) pending_tax_registration: Option<PreparedTodayTaxRefresh>,
}

impl<Callback, GetTick, GetTimerLocalTime, RsPlayer>
    AsyncTimerCallbackHandler<Callback, GetTick, GetTimerLocalTime>
    for WorldTimerHandler<'_, Callback, RsPlayer>
where
    Callback: Copy + PartialEq,
    GetTick: FnMut() -> u32,
    GetTimerLocalTime: FnMut() -> TagTime,
    RsPlayer: RsPlayerOwner<CPlayer>,
{
    type Block = WorldTimerCallbackBlock;

    async fn dispatch(
        &mut self,
        timer: &mut CTimer<Callback>,
        invocation: TimerCallbackInvocation<Callback>,
        get_tick: &mut GetTick,
        get_timer_local_time: &mut GetTimerLocalTime,
    ) -> Result<AsyncTimerCallbackDisposition<Callback>, Self::Block> {
        let copy_number_event = matches!(
            invocation.source,
            TimerCallbackSource::Calendar(event_id)
                if self.copy_number_timer.is_event(event_id)
        );
        if copy_number_event {
            let current_time = get_timer_local_time();
            let report = self
                .copy_number_timer
                .prepare_reset(current_time)
                .map_err(WorldTimerCallbackBlock::CopyNumber)?;
            let next_time = report.scheduled_time;
            let report_index = self.copy_number_resets.len();
            self.copy_number_resets.push(report);
            self.pending_copy_number_registration = Some(report_index);
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        let tax_event_id = match invocation.source {
            TimerCallbackSource::Calendar(event_id)
                if self.organizing_parameters.is_tax_event(event_id) => Some(event_id),
            _ => None,
        };
        if let Some(event_id) = tax_event_id {
            let current_time = get_timer_local_time();
            let prepared = self
                .organizing_parameters
                .prepare_today_tax_refresh(
                    event_id,
                    current_time,
                    self.game.current_game_server_sender().as_ref(),
                )
                .map_err(WorldTimerCallbackBlock::OrganizingTax)?;
            let next_time = prepared.next_time;
            self.pending_tax_registration = Some(prepared);
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        let is_player_ranks_event = matches!(
            invocation.source,
            TimerCallbackSource::Calendar(event_id)
                if self.player_ranks.stat_event_id() == Some(event_id)
        );
        if is_player_ranks_event {
            let stat = self
                .game
                .stat_player_ranks(
                    self.player_ranks,
                    self.rs_player,
                    self.player_database.as_deref_mut(),
                    self.organizing,
                    self.log,
                    get_tick,
                    &mut *self.get_log_local_time,
                    &mut *self.put_log_info,
                )
                .await
                .map_err(PlayerRanksTimerRefreshBlock::Stat)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let sender = self.game.current_game_server_sender();
            let publication = self
                .player_ranks
                .update_ranks_to_game_server(sender.as_ref())
                .map_err(PlayerRanksTimerRefreshBlock::Serialization)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let current_time = get_timer_local_time();
            let next_time = self
                .player_ranks
                .next_stat_time(current_time)
                .map_err(PlayerRanksTimerRefreshBlock::Schedule)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let refresh_index = self.refreshes.len();
            self.refreshes.push(PlayerRanksTimerRefreshReport {
                stat,
                publication,
                next_time,
                next_event_id: None,
            });
            self.pending_player_ranks_registration = Some(refresh_index);

            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        if invocation.callback == self.time_to_return_callbacks.on_time {
            let mut effects = WorldTimeToReturnEffects { game: self.game };
            let report = self.time_to_return.on_time(
                invocation.parameter,
                timer,
                self.time_to_return_callbacks,
                &mut effects,
            );
            self.time_to_returns.push(report);
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: None,
            });
        }

        if let Some(callback) = self
            .four_nation_war_callbacks
            .kind(&invocation.callback)
        {
            let index = invocation.parameter;
            let mut effects = WorldFourNationWarTimerEffects {
                game: self.game,
                country_handler: self.country_handler,
                log: self.log,
                get_tick,
                get_log_local_time: self.get_log_local_time,
                put_log_info: self.put_log_info,
            };
            let region_block = |source: FourNationWarRegionIndexBlock| {
                WorldTimerCallbackBlock::FourNationWar(
                    FourNationWarCalendarBlock::RegionIndex(source),
                )
            };
            match callback {
                FourNationWarCallbackKind::SignUpStart => self
                    .four_nation_war
                    .on_sign_up_war_start(index, &mut effects)
                    .map_err(WorldTimerCallbackBlock::FourNationWar)?,
                FourNationWarCallbackKind::SignUpEnd => self
                    .four_nation_war
                    .on_sign_up_war_end(index, &mut effects)
                    .map_err(region_block)?,
                FourNationWarCallbackKind::WarStart => self
                    .four_nation_war
                    .on_war_start(index, &mut effects)
                    .map_err(region_block)?,
                FourNationWarCallbackKind::WarEnd => self
                    .four_nation_war
                    .on_war_end(
                        index,
                        timer,
                        self.four_nation_war_callbacks,
                        &mut effects,
                    )
                    .map_err(WorldTimerCallbackBlock::FourNationWar)?,
                FourNationWarCallbackKind::WarEndInfo => self
                    .four_nation_war
                    .on_war_end_info(index, &mut effects)
                    .map_err(region_block)?,
                FourNationWarCallbackKind::EnterStart => self
                    .four_nation_war
                    .on_enter_start(index, &mut effects)
                    .map_err(region_block)?,
                FourNationWarCallbackKind::EnterEnd => self
                    .four_nation_war
                    .on_enter_end(index, &mut effects)
                    .map_err(region_block)?,
                FourNationWarCallbackKind::RefreshRegion => {
                    CFourNationWarSys::on_refresh_region(index, &mut |message| {
                        effects.send_all(message)
                    });
                }
                FourNationWarCallbackKind::ClearWar => {
                    CFourNationWarSys::on_clear_war(index, &mut |message| {
                        effects.send_all(message)
                    });
                }
            }
            self.four_nation_wars
                .push(FourNationWarTimerReport { callback, index });
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: None,
            });
        }

        if let Some(callback) = self.attack_city_callbacks.kind(&invocation.callback) {
            let war_number = invocation.parameter;
            let mut effects = WorldTerritoryWarTimerEffects {
                game: self.game,
                country_handler: self.country_handler,
                get_tick,
            };
            let outcome = match callback {
                AttackCityCallbackKind::Declare => AttackCityTimerOutcome::Phase(
                    self.attack_city
                        .on_declare_war(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                AttackCityCallbackKind::StartInfo => {
                    let now = get_timer_local_time();
                    AttackCityTimerOutcome::Countdown(
                        self.attack_city
                            .on_attack_city_start_info(war_number, now, &mut effects)
                            .map_err(WorldTimerCallbackBlock::AttackCity)?,
                    )
                }
                AttackCityCallbackKind::Start => AttackCityTimerOutcome::Phase(
                    self.attack_city
                        .on_attack_city_start(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                AttackCityCallbackKind::EndInfo => {
                    let now = get_timer_local_time();
                    AttackCityTimerOutcome::Countdown(
                        self.attack_city
                            .on_attack_city_end_info(war_number, now, &mut effects)
                            .map_err(WorldTimerCallbackBlock::AttackCity)?,
                    )
                }
                AttackCityCallbackKind::End => AttackCityTimerOutcome::Phase(
                    self.attack_city
                        .on_attack_city_end(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                AttackCityCallbackKind::Mass => AttackCityTimerOutcome::Phase(
                    self.attack_city
                        .on_mass(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                AttackCityCallbackKind::ClearOtherPlayer => AttackCityTimerOutcome::Phase(
                    self.attack_city
                        .on_clear_other_player(war_number, &mut effects),
                ),
                AttackCityCallbackKind::RefreshRegion => AttackCityTimerOutcome::Phase(
                    self.attack_city.on_refresh_region(war_number, &mut effects),
                ),
            };
            self.attack_city_wars.push(AttackCityTimerReport {
                callback,
                war_number,
                outcome,
            });
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: None,
            });
        }

        if let Some(callback) = self.village_war_callbacks.kind(&invocation.callback) {
            let war_number = invocation.parameter;
            let mut effects = WorldTerritoryWarTimerEffects {
                game: self.game,
                country_handler: self.country_handler,
                get_tick,
            };
            let outcome = match callback {
                VillageWarCallbackKind::Declare => VillageWarTimerOutcome::Phase(
                    self.village_war
                        .on_declare_war(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                VillageWarCallbackKind::StartInfo => {
                    let now = get_timer_local_time();
                    VillageWarTimerOutcome::Countdown(
                        self.village_war
                            .on_attack_village_start_info(war_number, now, &mut effects)
                            .map_err(WorldTimerCallbackBlock::VillageWar)?,
                    )
                }
                VillageWarCallbackKind::Start => VillageWarTimerOutcome::Phase(
                    self.village_war
                        .on_attack_village_start(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                VillageWarCallbackKind::EndInfo => {
                    let now = get_timer_local_time();
                    VillageWarTimerOutcome::Countdown(
                        self.village_war
                            .on_attack_village_end_info(war_number, now, &mut effects)
                            .map_err(WorldTimerCallbackBlock::VillageWar)?,
                    )
                }
                VillageWarCallbackKind::End => VillageWarTimerOutcome::Phase(
                    self.village_war
                        .on_attack_village_end(war_number, &mut effects)
                        .unwrap_or_else(|never| match never {}),
                ),
                VillageWarCallbackKind::ClearPlayer => VillageWarTimerOutcome::Phase(
                    self.village_war.on_clear_player(war_number, &mut effects),
                ),
            };
            self.village_wars.push(VillageWarTimerReport {
                callback,
                war_number,
                outcome,
            });
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: None,
            });
        }

        let Some(callback) = self.country_war_callbacks.kind(invocation.callback) else {
            return Err(WorldTimerCallbackBlock::UnexpectedCallback {
                source: invocation.source,
                parameter: invocation.parameter,
            });
        };
        let mut effects = WorldCountryWarEffects {
            game: self.game,
            country_handler: &mut *self.country_handler,
            globe_setup: self.globe_setup,
        };
        let report = match callback {
            CountryWarCallbackKind::Clear
            | CountryWarCallbackKind::DeclareBegin
            | CountryWarCallbackKind::DeclareEnd
            | CountryWarCallbackKind::PrepareBegin
            | CountryWarCallbackKind::PrepareEnd => {
                let phase = match callback {
                    CountryWarCallbackKind::Clear => CountryWarPhase::Clear,
                    CountryWarCallbackKind::DeclareBegin => CountryWarPhase::DeclareBegin,
                    CountryWarCallbackKind::DeclareEnd => CountryWarPhase::DeclareEnd,
                    CountryWarCallbackKind::PrepareBegin => CountryWarPhase::PrepareBegin,
                    CountryWarCallbackKind::PrepareEnd => CountryWarPhase::PrepareEnd,
                    _ => unreachable!("ветка ограничена phase callbacks"),
                };
                let report = self
                    .country_war
                    .run_phase(phase, invocation.parameter, &mut effects)
                    .map_err(CountryWarTimerBlock::Phase)
                    .map_err(WorldTimerCallbackBlock::CountryWar)?;
                CountryWarTimerReport::Phase {
                    callback,
                    war_id: invocation.parameter,
                    report,
                }
            }
            CountryWarCallbackKind::Start => CountryWarTimerReport::Start {
                war_id: invocation.parameter,
                report: self
                    .country_war
                    .run_war_start(invocation.parameter, &mut effects)
                    .map_err(CountryWarTimerBlock::Start)
                    .map_err(WorldTimerCallbackBlock::CountryWar)?,
            },
            CountryWarCallbackKind::End => {
                let now = get_timer_local_time();
                CountryWarTimerReport::End {
                    war_id: invocation.parameter,
                    report: self
                        .country_war
                        .run_war_end(invocation.parameter, now, get_tick, &mut effects)
                        .map_err(CountryWarTimerBlock::End)
                        .map_err(WorldTimerCallbackBlock::CountryWar)?,
                }
            }
            CountryWarCallbackKind::StartInfo | CountryWarCallbackKind::EndInfo => {
                let kind = match callback {
                    CountryWarCallbackKind::StartInfo => CountryWarTopInfoKind::Start,
                    CountryWarCallbackKind::EndInfo => CountryWarTopInfoKind::End,
                    _ => unreachable!("ветка ограничена top-info callbacks"),
                };
                let now = get_timer_local_time();
                CountryWarTimerReport::TopInfo {
                    callback,
                    report: self
                        .country_war
                        .run_top_info(kind, invocation.parameter, now, get_tick, &mut effects)
                        .map_err(CountryWarTimerBlock::TopInfo)
                        .map_err(WorldTimerCallbackBlock::CountryWar)?,
                }
            }
        };
        self.country_wars.push(report);
        Ok(AsyncTimerCallbackDisposition::Handled {
            next_calendar_event: None,
        })
    }

    fn calendar_event_registered(
        &mut self,
        _invocation: TimerCallbackInvocation<Callback>,
        event_id: TimerId,
    ) {
        if let Some(report_index) = self.pending_copy_number_registration.take() {
            self.copy_number_timer.finish_reset(
                &mut self.copy_number_resets[report_index],
                event_id,
            );
            return;
        }

        if let Some(prepared) = self.pending_tax_registration.take() {
            let mut world_string = |string_id: &[u8]| {
                self.game.get_string_by_id(string_id).to_vec()
            };
            let report = self.organizing_parameters.finish_today_tax_refresh(
                prepared,
                event_id,
                &mut world_string,
            );
            self.tax_refreshes.push(report);
            return;
        }

        self.player_ranks.finish_stat_schedule(event_id);
        let refresh_index = self
            .pending_player_ranks_registration
            .take()
            .expect("handled PlayerRanks callback всегда просит следующее событие");
        self.refreshes[refresh_index].next_event_id = Some(event_id);
    }
}

const INITIAL_GOODS_LINK_PLACEHOLDERS: usize = 500;

const LEGACY_GOODS_LINK_MAX_SIZE: usize = 0x0CCC_CCCC;

static NEXT_GOODS_LINK_INDEX: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Eq, PartialEq)]
pub struct WorldOwnedCityRefreshReport {
    pub(crate) region_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) union_id: i32,
    pub(crate) country_id: u8,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldOwnedCityRefreshOutcome {
    RegionNotFound,
    NullRegionPointer,
    Refreshed(WorldOwnedCityRefreshReport),
}

pub enum WorldCreationPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    DuplicateReleased {
        player_id: u32,
    },
    ExistingMapOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

pub enum WorldMapPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    ExistingOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCreationPlayerAppendLog {
    Duplicate { player_id: u32 },
    ExistingMapOwner,
}

impl fmt::Display for WorldCreationPlayerAppendLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { player_id } => {
                write!(formatter, "{player_id} Player Is In CreationPlayerList.")
            }
            Self::ExistingMapOwner => formatter.write_str("MapPlayer Not Found or NULL."),
        }
    }
}

pub struct CGame {
    pub(crate) setup: WorldSetup,
    pub(crate) thing_setup: CThingSetup,
    pub(crate) emotion: CEmotion,
    pub(crate) globe_variables: WorldGlobeVariables,
    pub(crate) string_table: MyStringTable,
    pub(crate) string_table_array: Vec<u8>,
    pub(crate) words_filter: CWordsFilter,
    pub(crate) dupli_region_setup: Option<CDupliRegionSetup>,
    pub(crate) equipment_compose_list: EquipmentComposeList,
    pub(crate) ci_qing_setup: CCiQingSetup,
    pub(crate) tao_zhuang_setup: CTaoZhuangSetup,
    pub(crate) hit_level_setup: CHitLevelSetup,
    pub(crate) trade_list: CTradeList,
    pub(crate) increment_shop_list: CIncrementShopList,
    pub(crate) prison_conf: PrisonConf,
    pub(crate) contribute_setup: CContributeSetup,
    pub(crate) quest_system: QuestCatalog,
    pub(crate) connect_login_worker: Option<WorldLoginReconnectWorker>,
    pub(crate) write_log_worker: Option<WorldWriteLogWorker>,
    pub(crate) player_load_workers: WorldPlayerLoadWorkerPool,
    pub(crate) net_client: Option<CMyNetClient>,
    pub(crate) net_server: Option<CMyNetServer>,
    pub(crate) regions: BTreeMap<i32, WorldRegionAssignment>,
    pub(crate) script_resources: ScriptResources,
    pub(crate) game_servers: BTreeMap<u32, WorldGameServerEntry>,
    pub(crate) system_broadcasts: VecDeque<WorldSystemBroadcast>,
    pub(crate) goods_links: VecDeque<WorldGoodsLink>,
    pub(crate) write_log_queue: WorldWriteLogQueue,
    pub(crate) player_data_queue: CPlayerDataQueue<CPlayer>,
    pub(crate) player_load_queue: CPlayerLoadQueue,
    pub(crate) players: BTreeMap<u32, Box<CPlayer>>,
    pub(crate) team_session_ids: BTreeMap<u32, i32>,
    pub(crate) creation_players: VecDeque<i32>,
    pub(crate) restore_players: VecDeque<u32>,
    pub(crate) deletion_players: VecDeque<DeletionPlayerSnapshot>,
    pub(crate) player_id: u32,
    pub(crate) leave_word_id: i32,
    pub(crate) online_players: VecDeque<u32>,
    pub(crate) offline_players: VecDeque<u32>,
    pub(crate) login_players: VecDeque<WorldLoginPlayerEntry>,
    pub(crate) db_responses: i32,
    pub(crate) db_data: Mutex<WorldDbData>,
    pub(crate) ping_game_servers: Vec<WorldPingGameServerInfo>,
    pub(crate) bai_tan: WorldBaiTanLists,
    pub(crate) honor_eliminate_list: BTreeMap<u32, VecDeque<u32>>,
    pub(crate) login_server_id: i32,
    pub(crate) ping_in_progress: bool,
    pub(crate) last_ping_game_server_time_ms: u32,
    pub(crate) game_server_message_time_ms: u32,
    pub(crate) login_server_message_time_ms: u32,
}

impl crate::activities::factionwarsys::EnemyFactionSink for CGame {
    fn set_enemy_factions(&self, enemy_factions: VecDeque<Option<EnemyFactionSaveSnapshot>>) {
        CGame::set_enemy_factions(self, enemy_factions);
    }
}

impl crate::organizations::countryhandler::CountrySaveSink for CGame {
    fn append_db_country(&self, country: CountrySaveSnapshot) {
        CGame::append_db_country(self, country);
    }
}

/// Прямая делегация одноимённым inherent-методам: Realm `COrganizingCtrl`
/// постановляет organizing save/delete очереди `m_stDBData` только через
/// готовый sink владельца сохранения (прецедент `CountrySaveSink`).
impl crate::organizations::organizingctrl::OrganizingSaveSink for CGame {
    fn append_save_faction(&self, faction: Box<CFaction>, goods_war_count: i32) {
        CGame::append_save_faction(self, faction, goods_war_count);
    }

    fn append_save_union(&self, union: Box<CUnion>) {
        CGame::append_save_union(self, union);
    }

    fn append_delete_faction(&self, faction_id: i32) {
        CGame::append_delete_faction(self, faction_id);
    }

    fn append_delete_union(&self, union_id: i32) {
        CGame::append_delete_union(self, union_id);
    }
}

impl crate::characters::honorranks::HonorRanksGameView for CGame {
    fn queue_honor_ranks_world_message(
        &self,
        message: CMessage,
    ) -> Result<(), crate::characters::honorranks::HonorRanksLocalQueueBlock> {
        self.queue_local_world_message(message).map_err(|block| {
            crate::characters::honorranks::HonorRanksLocalQueueBlock {
                message_type: block.message_type,
            }
        })
    }

    fn honor_ranks_game_server_sender(&self) -> Option<ServerCommandHandle> {
        self.current_game_server_sender()
    }
}

impl crate::app::world_game_view::WorldGameView for CGame {
    fn current_game_server_sender(&self) -> Option<ServerCommandHandle> {
        CGame::current_game_server_sender(self)
    }

    fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError> {
        CGame::send_msg_to_game_server(self, map_id, message)
    }

    fn publish_team_session(&mut self, team_id: u32, session_id: i32) {
        CGame::publish_team_session(self, team_id, session_id);
    }

    fn remove_team_session(&mut self, team_id: u32) {
        CGame::remove_team_session(self, team_id);
    }

    fn get_team_session_id(&self, team_id: u32) -> i32 {
        CGame::get_team_session_id(self, team_id)
    }

    fn player_game_server(
        &self,
        player_id: i32,
    ) -> Option<crate::app::world_game_view::WorldGameServerSnapshot> {
        CGame::player_game_server(self, player_id).map(|entry| {
            crate::app::world_game_view::WorldGameServerSnapshot {
                connected: entry.connected,
                index: entry.index,
            }
        })
    }

    fn map_player(&self, player_id: u32) -> Option<&CPlayer> {
        CGame::map_player(self, player_id)
    }

    fn online_player_id_by_name(&self, name: &[u8]) -> u32 {
        CGame::online_player_id_by_name(self, name)
    }

    fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer> {
        CGame::online_player_by_cdkey(self, cdkey)
    }

    fn configured_world_number(&self) -> Option<u32> {
        CGame::configured_world_number(self)
    }

    fn current_login_client(&self) -> Option<&CMyNetClient> {
        CGame::current_login_client(self)
    }

    fn game_server_number_by_player_id(&self, player_id: i32) -> i32 {
        CGame::game_server_number_by_player_id(self, player_id)
    }

    fn legacy_tick_ms(&self) -> u32 {
        legacy_tick_ms()
    }

    fn set_map_player_jjc_identity(&mut self, player_id: u32, level: u8, jjc_level: u32) -> bool {
        CGame::set_map_player_jjc_identity(self, player_id, level, jjc_level)
    }

    fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool {
        CGame::set_map_player_jjc_snapshot(self, player_id, level, jjc_level, jjc_score, counters)
    }

    fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer> {
        CGame::online_player_by_id(self, player_id)
    }

    fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        CGame::decord_online_player_by_id(self, player_id, source, cursor, registry, coefficients)
    }

    fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool {
        CGame::add_item_to_bai_tan_request_list(self, ip, player_id)
    }

    fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval {
        CGame::del_item_from_bai_tan_list(self, player_id)
    }

    fn game_server(
        &self,
        index: u32,
    ) -> Option<crate::app::world_game_view::WorldGameServerSnapshot> {
        CGame::game_server(self, index).map(|entry| {
            crate::app::world_game_view::WorldGameServerSnapshot {
                connected: entry.connected,
                index: entry.index,
            }
        })
    }

    fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize {
        CGame::push_write_log_command(self, command)
    }

    fn get_string_by_id(&self, string_id: &[u8]) -> &[u8] {
        CGame::get_string_by_id(self, string_id)
    }

    fn map_player_id_by_name(&self, name: &[u8]) -> u32 {
        CGame::map_player_id_by_name(self, name)
    }

    fn named_region_lookup(&self, name: &[u8]) -> WorldNamedRegionLookup {
        CGame::named_region_lookup(self, name)
    }

    fn region_routes_by_owner_id(&self, region_id: i32) -> WorldRegionIdRouteScan {
        CGame::region_routes_by_owner_id(self, region_id)
    }

    fn region_name(&self, region_id: i32) -> WorldRegionNameLookup<'_> {
        CGame::region_name(self, region_id)
    }

    fn queue_local_world_message(
        &self,
        message: CMessage,
    ) -> Result<(), WorldLocalMessageQueueBlock> {
        CGame::queue_local_world_message(self, message)
    }

    fn allocate_leave_word_id(&mut self) -> i32 {
        CGame::allocate_leave_word_id(self)
    }

    fn update_player_faction_info_from_faction(
        &self,
        faction: &CFaction,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        CGame::update_player_faction_info_from_faction(self, faction, player_id)
    }

    fn online_player_count(&self) -> usize {
        CGame::online_player_count(self)
    }

    fn is_restore_player_exist(&self, player_id: u32) -> bool {
        CGame::is_restore_player_exist(self, player_id)
    }

    fn delete_restore_player(&mut self, player_id: u32) {
        CGame::delete_restore_player(self, player_id)
    }

    fn deletion_player_time(&self, player_id: u32) -> i32 {
        CGame::deletion_player_time(self, player_id)
    }

    fn delete_deletion_player(&mut self, player_id: u32) {
        CGame::delete_deletion_player(self, player_id)
    }

    fn append_restore_player(&mut self, player_id: u32) {
        CGame::append_restore_player(self, player_id)
    }

    fn append_deletion_player(&mut self, player_id: u32, deletion_time: i32) {
        CGame::append_deletion_player(self, player_id, deletion_time)
    }

    fn login_player_by_account(&self, account: &[u8]) -> Option<WorldLoginAccountPlayer> {
        CGame::login_player_by_account(self, account)
    }

    fn remove_login_player(&mut self, player_id: u32) -> bool {
        CGame::remove_login_player(self, player_id)
    }

    fn remove_player_load_data(&self, player_id: i32) -> bool {
        CGame::remove_player_load_data(self, player_id)
    }

    fn append_offline_player_id(&mut self, player_id: u32) -> bool {
        CGame::append_offline_player_id(self, player_id)
    }

    fn online_player_route_by_account(&self, account: &[u8]) -> Option<WorldOnlineAccountPlayerRoute> {
        CGame::online_player_route_by_account(self, account)
    }

    fn validate_player_id_in_cdkey(&self, account: &[u8], player_id: u32) -> bool {
        CGame::validate_player_id_in_cdkey(self, account, player_id)
    }

    fn validate_db_player_id_in_cdkey(&self, account: &[u8], player_id: u32) -> bool {
        CGame::validate_db_player_id_in_cdkey(self, account, player_id)
    }

    fn push_player_load_request(
        &self,
        account: &[u8],
        player_id: u32,
        client_ip: u32,
    ) -> Result<WorldPlayerLoadRequestOutcome, WorldPlayerLoadRequestBlock> {
        CGame::push_player_load_request(self, account, player_id, client_ip)
    }

    fn exit_team_player(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit {
        CGame::exit_team_player(self, factory, session_id, owner_type, owner_id)
    }

    fn is_name_exist_in_map_player(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError> {
        CGame::is_name_exist_in_map_player(self, name)
    }

    fn is_name_exist_in_db_creation(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError> {
        CGame::is_name_exist_in_db_creation(self, name)
    }

    fn is_name_exist_in_db_data(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError> {
        CGame::is_name_exist_in_db_data(self, name)
    }

    fn check_create_role_name(
        &self,
        name: &mut Vec<u8>,
        allow_short: bool,
        apply_filter: bool,
    ) -> bool {
        CGame::check_create_role_name(self, name, allow_short, apply_filter)
    }

    fn allocate_player_id(&mut self) -> i32 {
        CGame::allocate_player_id(self)
    }

    fn creation_player_by_name(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError> {
        CGame::creation_player_by_name(self, name).map(|found| found.is_some())
    }

    fn creation_player_count_in_cdkey(&mut self, cdkey: &[u8]) -> u8 {
        CGame::creation_player_count_in_cdkey(self, cdkey)
    }

    fn format_world_string(
        &self,
        string_id: &[u8],
        arguments: &[crate::organizations::union::UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        CGame::format_world_string(self, string_id, arguments)
    }


    fn replace_online_player_silience_time(
        &mut self,
        player_id: u32,
        silience_time: i32,
    ) -> Option<i32> {
        CGame::replace_online_player_silience_time(self, player_id, silience_time)
    }

    fn reload<'a>(
        &'a mut self,
        context: &'a mut dyn WorldReloadContext,
        jjc: &'a mut CJJcSystem,
        gods_battle: &'a mut CGodsBattleConf,
        skills: &'a mut CSkillFactory,
        rs_gods_battle: Option<&'a mut TiberiusRsGodsBattle>,
        profile: &'a [u8],
        send_to_game_servers: bool,
        reload_server_resources: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorldReloadResult> + 'a>> {
        Box::pin(CGame::reload(
            self,
            context,
            jjc,
            gods_battle,
            skills,
            rs_gods_battle,
            profile,
            send_to_game_servers,
            reload_server_resources,
        ))
    }

    fn region_game_server_index(&self, region_id: i32) -> Option<u32> {
        self.get_region_game_server(region_id)
            .map(|entry| entry.index)
    }

    fn game_server_number_by_region_id(&self, region_id: i32) -> i32 {
        CGame::game_server_number_by_region_id(self, region_id)
    }

    fn reset_map_player_faction_data(&self, map_key: u32) -> bool {
        CGame::reset_map_player_faction_data(self, map_key)
    }

    fn remove_offline_player(&mut self, player_id: u32) {
        CGame::remove_offline_player(self, player_id)
    }

    fn decode_online_player_lei_ting(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        CGame::decode_online_player_lei_ting(self, player_id, source, cursor)
    }

    fn reset_honor_eliminate_info(&mut self, rank_mask: u32) -> bool {
        CGame::reset_honor_eliminate_info(self, rank_mask)
    }

    fn register_honor_eliminator(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration {
        CGame::register_honor_eliminator(self, player_id, eliminator_id)
    }

    fn add_goods_link(&mut self, link: WorldGoodsLink) -> u32 {
        CGame::add_goods_link(self, link)
    }

    fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink> {
        CGame::find_goods_link(self, index)
    }

    fn change_map_player_name<'a>(
        &'a mut self,
        player_id: u32,
        requested_name: Option<&'a [u8]>,
        globe_setup: &'a GlobeSetupSnapshot,
        rs_player: &'a mut (dyn crate::app::world_game_view::WorldRenameDbView + 'a),
        player_database: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError>> + 'a>> {
        Box::pin(CGame::change_map_player_name(
            self,
            player_id,
            requested_name,
            globe_setup,
            rs_player,
            player_database,
        ))
    }

    fn set_region_param_from_game_server(
        &mut self,
        region_id: i32,
        current_tax_rate: i32,
        today_total_tax: u32,
        total_tax: u32,
    ) -> WorldRegionParamUpdateOutcome {
        CGame::set_region_param_from_game_server(
            self,
            region_id,
            current_tax_rate,
            today_total_tax,
            total_tax,
        )
    }

    fn login_server_id(&self) -> i32 {
        CGame::login_server_id(self)
    }

    fn check_invalid_string(&self, value: &mut Vec<u8>, replace: bool) -> bool {
        CGame::check_invalid_string(self, value, replace)
    }

    fn region_owned_faction_id(&self, region_id: i32) -> Option<i32> {
        CGame::region_owned_faction_id(self, region_id)
    }

    fn region_country_id(&self, region_id: i32) -> Option<u8> {
        CGame::region_country_id(self, region_id)
    }

    fn has_materialized_region(&self, region_id: i32) -> bool {
        CGame::has_materialized_region(self, region_id)
    }

    fn connected_game_server_indices(&self) -> Vec<i32> {
        CGame::connected_game_server_indices(self).collect()
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterReset> {
        CGame::reset_online_player_murder_counters(self, player_id)
    }

    fn add_map_player_exploit_wrapping(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate> {
        CGame::add_map_player_exploit_wrapping(self, player_id, increment)
    }

    fn create_connect_login_thread(
        &mut self,
        runtime: tokio::runtime::Handle,
    ) -> WorldLoginReconnectThreadRestart {
        CGame::create_connect_login_thread(self, runtime)
    }

    fn begin_game_server_ping(&mut self) -> (usize, u32) {
        CGame::begin_game_server_ping(self)
    }

    fn assign_login_server_id(&mut self, login_server_id: i32) -> i32 {
        CGame::assign_login_server_id(self, login_server_id)
    }

    fn connect_game_server_by_address(
        &mut self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<WorldGameServerConnectionState>, WorldGameServerLookupError> {
        CGame::connect_game_server_by_address(self, ip, port)
    }

    fn disconnect_game_server(
        &mut self,
        game_server_index: u32,
    ) -> Option<WorldGameServerDisconnectionState> {
        CGame::disconnect_game_server(self, game_server_index)
    }

    fn send_globe_variables_to_game_server(
        &self,
        socket_id: i32,
    ) -> WorldGlobeVariablesDelivery {
        CGame::send_globe_variables_to_game_server(self, socket_id)
    }

    fn region_assignment_exists(&self, region_id: i32) -> bool {
        self.region(region_id).is_some()
    }

    fn region_game_server_route(
        &self,
        region_id: i32,
    ) -> Option<crate::app::world_game_view::WorldRegionGameServerRoute> {
        CGame::get_region_game_server(self, region_id).map(|entry| {
            crate::app::world_game_view::WorldRegionGameServerRoute {
                connected: entry.connected,
                index: entry.index,
                ip: entry.ip.clone(),
                port: entry.port,
            }
        })
    }

    fn set_team_player_owner_region(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        region_id: i32,
    ) -> WorldRegionChangeTeamUpdate {
        CGame::set_team_player_owner_region(self, factory, session_id, owner_type, owner_id, region_id)
    }

    fn decord_server_snapshot_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldServerSnapshotPlayerDecode, PlayerCodecError> {
        CGame::decord_server_snapshot_player(
            self,
            requested_player_id,
            source,
            cursor,
            registry,
            coefficients,
        )
    }

    fn decord_reconnected_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReconnectedPlayerDecode, PlayerCodecError> {
        CGame::decord_reconnected_player(
            self,
            requested_player_id,
            source,
            cursor,
            registry,
            coefficients,
        )
    }

    fn record_player_save_response(
        &mut self,
        completion_counted: bool,
    ) -> WorldPlayerSaveResponseProgress {
        CGame::record_player_save_response(self, completion_counted)
    }

    fn increment_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterUpdate> {
        CGame::increment_online_player_murder_counters(self, player_id)
    }

    fn decode_region_param_from_game_server(
        &mut self,
        region_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> WorldRegionParamDecodeOutcome {
        CGame::decode_region_param_from_game_server(self, region_id, source, cursor)
    }

    fn reset_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        CGame::reset_received_player_data(self, game_server_index)
    }

    fn increment_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        CGame::increment_received_player_data(self, game_server_index)
    }

    fn received_player_data(&self, game_server_index: i32) -> WorldReceivedPlayerDataRead {
        CGame::received_player_data(self, game_server_index)
    }

    fn record_game_server_ping(&mut self, response: WorldPingGameServerInfo) -> usize {
        CGame::record_game_server_ping(self, response)
    }

    fn send_cdkey_to_login_server(
        &self,
    ) -> Result<Option<WorldCdkeySnapshot>, WorldCdkeySnapshotError> {
        CGame::send_cdkey_to_login_server(self)
    }

    fn replace_login_client(&mut self, client: CMyNetClient) -> bool {
        CGame::replace_login_client(self, client)
    }

    fn world_number_after_cdkey_snapshot(&self) -> u32 {
        CGame::world_number_after_cdkey_snapshot(self)
    }

    fn world_name(&self) -> &[u8] {
        CGame::world_name(self)
    }

    fn current_login_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        CGame::current_login_client_mut(self)
    }
}

impl crate::app::world_game_view::WorldServerMessageGameView for CGame {
    fn transition_online_player_region(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        target_region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<WorldRegionChangePlayerTransition>, PlayerCodecError> {
        CGame::transition_online_player_region(
            self,
            organizing,
            requested_player_id,
            target_region_id,
            tile_x,
            tile_y,
            direction,
            source,
            cursor,
            registry,
            coefficients,
        )
    }

    fn append_online_player_id(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player_id: i32,
    ) -> WorldOnlinePlayerAppendOutcome {
        CGame::append_online_player_id(self, organizing, player_id)
    }

    fn on_game_server_lost(
        &mut self,
        organizing: &mut COrganizingCtrl,
        game_server_index: u32,
        add_player_list: &mut dyn FnMut(&[u8]),
    ) -> WorldGameServerLostReport {
        CGame::on_game_server_lost(self, organizing, game_server_index, add_player_list)
    }
}

/// Gate-адаптер хвоста завершённой save-волны ветви `0x5FA03`: owners,
/// остающиеся в старом пакете (faction war, страновая таблица, lifecycle и
/// runtime save-потока), связываются один раз у вызова dispatcher-а; игра,
/// organizing и realm-владельцы приходят параметрами. Порядок цепочки
/// сохранён inherent `CGame::materialize_completed_save_response_snapshot`.
pub struct WorldCompletedSaveResponseMaterializationAdapter<'a> {
    pub(crate) faction_war_sys: &'a CFactionWarSys,
    pub(crate) country_handler: &'a CCountryHandler,
    pub(crate) country_limits: CountryKingSaveLimits,
    pub(crate) lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
    pub(crate) save_thread_handle: &'a mut WorldSaveThreadHandleState,
    pub(crate) save_runtime: &'a mut dyn WorldSaveRuntimeContext,
}

impl crate::app::world_game_view::WorldCompletedSaveResponseMaterialization<CGame>
    for WorldCompletedSaveResponseMaterializationAdapter<'_>
{
    fn materialize_completed_save_response_snapshot(
        &mut self,
        game: &mut CGame,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
    ) -> Result<WorldCompletedSaveResponseLaunchReport, WorldGenerateDbDataBlock> {
        game.materialize_completed_save_response_snapshot(
            registry,
            organizing,
            coefficients,
            self.faction_war_sys,
            self.country_handler,
            self.country_limits,
            variables,
            honor_ranks,
            gods_battle,
            Arc::clone(&self.lifecycle),
            self.save_thread_handle,
            self.save_runtime,
        )
    }
}

impl crate::app::world_game_view::WorldPlayerFactionInfoUpdateView for CGame {
    fn update_player_faction_info(
        &self,
        organizing: &COrganizingCtrl,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        CGame::update_player_faction_info(self, organizing, player_id)
    }
}

impl crate::app::world_game_view::WorldCreateRoleLaunchGate for CGame {
    #[allow(clippy::too_many_arguments)]
    fn launch_creation_player(
        &mut self,
        request: &crate::app::logmessage::WorldCreateRoleRequest,
        player_list: &mut CPlayerList,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        country_parameters: &mut CCountryParam,
        coefficients: &PlayerPropertyCoefficients,
        globe_setup: &GlobeSetupSnapshot,
        random: &mut dyn FnMut(i32) -> i32,
        add_log_text: &mut dyn FnMut(&[u8]),
    ) -> Result<
        WorldCreateRoleLaunchSuccess,
        WorldCreateRoleLaunchFailure,
    > {
        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let defaults = player
            .load_default_property(
                request.sex,
                request.occupation,
                request.country,
                country_parameters,
                self.dupli_region_setup(),
                player_list,
                globe_setup,
                self.thing_setup(),
                coefficients,
                |region_id| self.creation_region_base(region_id),
                random,
                || TagTime::local_now().day_of_week,
                || {
                    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                        Ok(duration) => duration.as_secs() as u32,
                        Err(error) => 0_u32.wrapping_sub(error.duration().as_secs() as u32),
                    }
                },
            )
            .map_err(WorldCreateRoleLaunchFailure::DefaultProperty)?;
        player.set_creation_identity(
            &request.name,
            &request.account,
            request.head_picture,
            request.face_picture,
        );
        player.set_creation_service_defaults();
        // Исходная точка потребления sequence счётчика: после применения
        // service defaults, до публикации игрока — как у прежнего обработчика.
        let player_id = self.allocate_player_id();
        player.set_id(player_id);
        let origin_goods = self
            .add_origin_goods_to_player(
                &mut player,
                player_list,
                registry,
                original_name_index,
                random,
            )
            .map_err(|block| WorldCreateRoleLaunchFailure::OriginGoods(
                crate::app::world_game_view::WorldOriginGoodsBlock {
                    origin_index: block.origin_index,
                    source: block.source,
                },
            ))?;

        match self.append_creation_player(player, |entry| {
            let text = entry.to_string();
            add_log_text(text.as_bytes());
        }) {
            WorldCreationPlayerAppendOutcome::Inserted { .. } => {}
            WorldCreationPlayerAppendOutcome::DuplicateReleased { .. } => {
                return Err(WorldCreateRoleLaunchFailure::AppendDuplicateCreationId);
            }
            WorldCreationPlayerAppendOutcome::ExistingMapOwnerKept {
                incoming,
                ..
            } => {
                drop(incoming);
                return Err(WorldCreateRoleLaunchFailure::AppendExistingMapOwner);
            }
        }

        let Some(created_player) = self.map_player(player_id as u32) else {
            return Err(WorldCreateRoleLaunchFailure::PublishedPlayerMissing {
                player_id: player_id as u32,
            });
        };
        let snapshot = created_player
            .player_base_wire_snapshot()
            .map_err(WorldCreateRoleLaunchFailure::Snapshot)?;

        Ok(WorldCreateRoleLaunchSuccess {
            player_id: player_id as u32,
            defaults,
            origin_goods: crate::app::world_game_view::WorldOriginGoodsReport {
                entries: origin_goods.entries,
            },
            snapshot,
        })
    }

    fn is_name_exit_in_faction(
        &mut self,
        organizing: &dyn crate::app::world_game_view::WorldCreateRoleOrganizingView,
        name: &[u8],
    ) -> Result<bool, WorldCreateRoleLaunchFailure> {
        match organizing.name_exists(name) {
            Ok(found) => Ok(found),
            Err(source) => Err(WorldCreateRoleLaunchFailure::OrganizingName(source)),
        }
    }
}

impl crate::app::player_base::WorldPlayerBaseGameView for CGame {
    type OrganizingContext = COrganizingCtrl;

    fn creation_player_count_in_cdkey(&self, account: &[u8]) -> u8 {
        CGame::creation_player_count_in_cdkey(self, account)
    }

    fn creation_player_ids_by_cdkey(&self, account: &[u8]) -> Vec<u32> {
        CGame::creation_player_ids_by_cdkey(self, account)
    }

    fn clone_map_player_for_base(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        CGame::clone_map_player(self, player_id, registry, organizing, coefficients)
    }

    fn clone_saving_player_for_base(
        &self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        CGame::clone_saving_player(self, player_id, registry, organizing, coefficients)
    }
}

impl crate::app::world_game_view::WorldPlayerSelectGameView for CGame {
    #[allow(clippy::too_many_arguments, reason = "точная форма route-вызова ветки select")]
    fn route_select_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        get_tick: &mut dyn FnMut() -> u32,
    ) -> Result<WorldPlayerSelectRouteOutcome, WorldPlayerSelectRouteError> {
        match CGame::route_loaded_player(
            self,
            organizing,
            0,
            0,
            queue_player_id,
            client_ip,
            cdkey,
            player,
            WorldLoadedPlayerRouteOrder::Direct,
            after_login_send,
            get_tick,
        ) {
            Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                reason,
                login_delivery,
                ..
            }) => Ok(WorldPlayerSelectRouteOutcome::Rejected {
                reason,
                login_delivery,
            }),
            Ok(WorldProcessPlayerDataQueueOutcome::Accepted {
                game_server_index,
                login_delivery,
                friend_updates,
                online_removal,
                replaced_existing_player,
                login_time_ms,
                ..
            }) => Ok(WorldPlayerSelectRouteOutcome::Accepted {
                game_server_index,
                login_delivery,
                friend_updates,
                online_removed_occurrences: online_removal.removed_occurrences,
                replaced_existing_player,
                login_time_ms,
            }),
            // `route_loaded_player` возвращает только Rejected/Accepted;
            // NoRecord ставится единолично `process_player_data_queue`.
            Ok(WorldProcessPlayerDataQueueOutcome::NoRecord { .. }) => {
                unreachable!("route_loaded_player не возвращает NoRecord")
            }
            Err(error) => Err(WorldPlayerSelectRouteError {
                player_id: error.player_id,
                block: match error.block {
                    WorldProcessPlayerDataQueueBlock::Organizing(source) => {
                        WorldPlayerSelectRouteBlock::Organizing(source)
                    }
                    WorldProcessPlayerDataQueueBlock::UninitializedGameServerPort {
                        game_server_index,
                    } => WorldPlayerSelectRouteBlock::UninitializedGameServerPort {
                        game_server_index,
                    },
                    // `UnterminatedCdkey` ставится только при разборе записи
                    // очереди в `process_player_data_queue`; сам маршрут его
                    // не производит.
                    WorldProcessPlayerDataQueueBlock::UnterminatedCdkey => {
                        unreachable!("route_loaded_player не возвращает UnterminatedCdkey")
                    }
                },
            }),
        }
    }
}

impl crate::app::world_game_view::WorldPlayerQueueGameView for CGame {
    #[allow(clippy::too_many_arguments, reason = "точная форма route-вызова queue-стадии")]
    fn route_loaded_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        get_tick: &mut dyn FnMut() -> u32,
    ) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError> {
        CGame::route_loaded_player(
            self,
            organizing,
            initial_size,
            null_pops,
            queue_player_id,
            client_ip,
            cdkey,
            player,
            WorldLoadedPlayerRouteOrder::LoadedQueue,
            after_login_send,
            get_tick,
        )
    }
}

impl crate::app::world_game_view::WorldOnlinePlayerRemovalView for CGame {
    fn remove_online_player(&mut self, organizing: &mut COrganizingCtrl, player_id: u32) -> usize {
        CGame::remove_online_player(self, organizing, player_id).removed_occurrences
    }
}

impl crate::app::world_game_view::WorldPlayerReturnGameView for CGame {
    #[allow(clippy::too_many_arguments, reason = "точная форма decode-вызова ветки return")]
    fn decord_returned_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReturnedPlayerDecode, PlayerCodecError> {
        CGame::decord_returned_player(
            self,
            organizing,
            requested_player_id,
            source,
            cursor,
            registry,
            coefficients,
        )
    }

    fn returned_player_snapshot(&self, player_id: u32) -> Option<WorldReturnedPlayerSnapshot> {
        CGame::returned_player_snapshot(self, player_id)
    }
}

impl crate::app::world_game_view::WorldPlayerDetailGameView for CGame {
    fn login_player_route_snapshot(&self, player_id: u32) -> Option<WorldLoginPlayerRouteSnapshot> {
        CGame::login_player_route_snapshot(self, player_id)
    }

    fn encode_map_player_full_snapshot(
        &mut self,
        organizing: &COrganizingCtrl,
        map_key: u32,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Vec<u8>>, PlayerCodecError> {
        CGame::encode_map_player_full_snapshot(self, organizing, map_key, registry, coefficients)
    }

    fn append_online_player_id(&mut self, organizing: &mut COrganizingCtrl, player_id: i32) -> bool {
        CGame::append_online_player_id(self, organizing, player_id).inserted
    }
}

impl crate::activities::leiting::LeiTingGameView for CGame {
    type Player = CPlayer;

    fn lei_ting_player_map_keys(&self) -> Vec<u32> {
        self.player_map_keys()
    }

    fn lei_ting_update_map_player<Clock: PlayerLeiTingClock>(
        &mut self,
        map_key: u32,
        update_kind: u32,
        stamp: &mut LeiTingLocalTime,
        globe_setup: &GlobeSetupSnapshot,
        clock: &mut Clock,
    ) -> Result<Option<PlayerLeiTingUpdateReport>, PlayerLeiTingUpdateBlock<Clock::Block>> {
        CGame::update_map_player_lei_ting(self, map_key, update_kind, stamp, globe_setup, clock)
    }

    fn lei_ting_map_player(&self, map_key: u32) -> Option<&Self::Player> {
        self.map_player(map_key)
    }
}

impl crate::activities::jjcsystem::JjcGameView for CGame {
    fn jjc_online_player(
        &self,
        player_id: u32,
    ) -> Option<&dyn crate::activities::jjcsystem::JjcPlayerView> {
        self.online_player_by_id(player_id)
            .map(|player| player as &dyn crate::activities::jjcsystem::JjcPlayerView)
    }

    fn jjc_map_player(
        &self,
        map_key: u32,
    ) -> Option<&dyn crate::activities::jjcsystem::JjcPlayerView> {
        self.map_player(map_key)
            .map(|player| player as &dyn crate::activities::jjcsystem::JjcPlayerView)
    }

    fn jjc_region_exists(&self, region_id: i32) -> bool {
        self.region(region_id).is_some()
    }

    fn jjc_region_game_server(
        &self,
        region_id: i32,
    ) -> Option<crate::activities::jjcsystem::JjcGameServerSnapshot> {
        self.get_region_game_server(region_id).map(|entry| {
            crate::activities::jjcsystem::JjcGameServerSnapshot {
                connected: entry.connected,
                index: entry.index,
            }
        })
    }

    fn jjc_player_game_server(
        &self,
        player_id: i32,
    ) -> Option<crate::activities::jjcsystem::JjcGameServerSnapshot> {
        self.player_game_server(player_id).map(|entry| {
            crate::activities::jjcsystem::JjcGameServerSnapshot {
                connected: entry.connected,
                index: entry.index,
            }
        })
    }

    fn jjc_game_server_sender(&self) -> Option<ServerCommandHandle> {
        self.current_game_server_sender()
    }
}

impl RegionParameterLoadTarget for CGame {
    fn has_region_parameter_target(&self, region_id: i32) -> bool {
        self.regions
            .get(&region_id)
            .is_some_and(|assignment| assignment.region.is_some())
    }

    fn apply_region_database_parameters(&mut self, parameters: RegionDatabaseParameters) -> bool {
        let Some(region) = self
            .regions
            .get_mut(&parameters.region_id)
            .and_then(|assignment| assignment.region.as_mut())
        else {
            return false;
        };
        region.base_mut().set_param_from_db(
            parameters.owned_faction_id,
            parameters.owned_union_id,
            parameters.current_tax_rate,
            parameters.today_total_tax,
            parameters.total_tax,
        );
        true
    }
}

// Связка старого пакета с драйвером Realm `app::world_runtime`, пока `CGame`
// живёт у этого owner-а: драйверу передаётся фабричный owner и его `Release`
// через shim-трейт. После волны самого `CGame` переходный блок снимается.
impl crate::app::world_runtime::WorldGameThreadGame for CGame {
    fn release<Context: WorldGameReleaseContext>(
        &mut self,
        context: &mut Context,
        goods_war: &mut CGoodsWarMember,
        increment_log: &mut CIncrementLog,
        skills: &mut CSkillFactory,
    ) -> WorldGameReleaseResult {
        CGame::release(self, context, goods_war, increment_log, skills)
    }
}

impl CGame {
    pub fn get_faction_by_id(
        organizing: &COrganizingCtrl,
        faction_id: i32,
    ) -> Option<&CFaction> {
        if faction_id == 0 {
            None
        } else {
            organizing.faction_by_id(faction_id)
        }
    }

    pub fn check_invalid_string(&self, value: &mut Vec<u8>, replace: bool) -> bool {
        self.words_filter.check(value, replace)
    }

    pub fn check_create_role_name(
        &self,
        value: &mut Vec<u8>,
        replace: bool,
        reject_all_numbers: bool,
    ) -> bool {
        self.words_filter
            .check_with_numeric_gate(value, replace, reject_all_numbers)
    }

    pub fn words_filter(&self) -> &CWordsFilter {
        &self.words_filter
    }

    pub fn emotion(&self) -> &CEmotion {
        &self.emotion
    }

    pub fn equipment_compose_list(&self) -> &EquipmentComposeList {
        &self.equipment_compose_list
    }

    pub fn ci_qing_setup(&self) -> &CCiQingSetup {
        &self.ci_qing_setup
    }

    pub fn tao_zhuang_setup(&self) -> &CTaoZhuangSetup {
        &self.tao_zhuang_setup
    }

    pub fn hit_level_setup(&self) -> &CHitLevelSetup {
        &self.hit_level_setup
    }

    pub fn trade_list(&self) -> &CTradeList {
        &self.trade_list
    }

    pub fn increment_shop_list(&self) -> &CIncrementShopList {
        &self.increment_shop_list
    }

    pub fn prison_conf(&self) -> &PrisonConf {
        &self.prison_conf
    }

    pub fn contribute_setup(&self) -> &CContributeSetup {
        &self.contribute_setup
    }

    pub fn quest_system(&self) -> &CQuestSystem {
        self.quest_system.system()
    }

    pub fn dupli_region_setup(&self) -> &CDupliRegionSetup {
        self.dupli_region_setup
            .as_ref()
            .expect("CDupliRegionSetup доступен только после успешного CGame::Init")
    }

    pub fn new() -> Self {
        Self {
            setup: WorldSetup::for_game(),
            thing_setup: CThingSetup::new(),
            emotion: CEmotion::default(),
            globe_variables: WorldGlobeVariables::default(),
            string_table: MyStringTable::new(),
            string_table_array: Vec::new(),
            words_filter: CWordsFilter::new(),
            dupli_region_setup: None,
            equipment_compose_list: EquipmentComposeList::default(),
            ci_qing_setup: CCiQingSetup::default(),
            tao_zhuang_setup: CTaoZhuangSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            trade_list: CTradeList::default(),
            increment_shop_list: CIncrementShopList::default(),
            prison_conf: PrisonConf::default(),
            contribute_setup: CContributeSetup::default(),
            quest_system: QuestCatalog::default(),
            connect_login_worker: None,
            write_log_worker: None,
            player_load_workers: WorldPlayerLoadWorkerPool::new(),
            net_client: None,
            net_server: None,
            regions: BTreeMap::new(),
            script_resources: ScriptResources::default(),
            game_servers: BTreeMap::new(),
            system_broadcasts: VecDeque::new(),
            goods_links: std::iter::repeat_with(WorldGoodsLink::placeholder)
                .take(INITIAL_GOODS_LINK_PLACEHOLDERS)
                .collect(),
            write_log_queue: WorldWriteLogQueue::default(),
            player_data_queue: CPlayerDataQueue::new(),
            player_load_queue: CPlayerLoadQueue::new(),
            players: BTreeMap::new(),
            team_session_ids: BTreeMap::new(),
            creation_players: VecDeque::new(),
            restore_players: VecDeque::new(),
            deletion_players: VecDeque::new(),
            player_id: 0,
            leave_word_id: 0,
            online_players: VecDeque::new(),
            offline_players: VecDeque::new(),
            login_players: VecDeque::new(),
            db_responses: 0,
            db_data: Mutex::new(WorldDbData::new()),
            ping_game_servers: Vec::new(),
            bai_tan: WorldBaiTanLists::new(),
            honor_eliminate_list: BTreeMap::new(),
            login_server_id: 0,
            ping_in_progress: false,
            last_ping_game_server_time_ms: legacy_tick_ms(),
            game_server_message_time_ms: 0,
            login_server_message_time_ms: 0,
        }
    }

    pub fn get_string_by_id(&self, string_id: &[u8]) -> &[u8] {
        self.string_table
            .table()
            .get_string_by_id(legacy_c_string_prefix(string_id))
            .unwrap_or_default()
    }

 /// Форматирует строку того же live `StringTable`, не создавая внешний
 /// callback, способный разойтись с reload-состоянием `CGame`.
    pub fn format_world_string(
        &self,
        string_id: &[u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.get_string_by_id(string_id), arguments)
    }

 /// Добавляет точную POD-запись в хвост `m_listGoodsLink`.
 ///
 /// Constructor уже создал 500 нулевых placeholder-ов, а process-global
 /// индекс начинается с `1`. Changed-запись сохраняет ID декодированного
 /// товара и global не двигает. Редкая `list::max_size` ветвь удаляет голову;
 /// Rust одновременно освобождает её owned товар, исправляя только утечку.
    pub fn add_goods_link(&mut self, mut link: WorldGoodsLink) -> u32 {
        if self.goods_links.len() == LEGACY_GOODS_LINK_MAX_SIZE {
            let _ = self.goods_links.pop_front();
        }
        if matches!(&link.payload, WorldGoodsLinkPayload::Original { .. }) {
            link.index = NEXT_GOODS_LINK_INDEX.fetch_add(1, Ordering::Relaxed);
        }
        let index = link.index;
        self.goods_links.push_back(link);
        index
    }

    pub fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize {
        self.write_log_queue.push(command)
    }

 /// Передаёт process-callback-ам producer того же FIFO, не открывая им
 /// mutable доступ к `CGame` во время одного MainLoop-прохода.
    pub fn write_log_queue(&self) -> WorldWriteLogQueue {
        self.write_log_queue.clone()
    }

 /// Возвращает первое совпадение в list-order, включая constructor-ный
 /// placeholder для индекса `0`.
    pub fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink> {
        self.goods_links.iter().find(|link| link.index == index)
    }

 /// Возвращает appearance snapshot экипировки игрока.
 ///
 /// Rust-ссылка исключает неопределённый null-вызов; EXE без проверок проходит
 /// slots `0,1,3,4,2,9,10,12,13,14,15`, оставляет нули для пустых slots и
 /// сужает signed `GAP_WEAPON_LEVEL` до младшего байта.
    pub fn get_player_equip_id(
        &self,
        player: &CPlayer,
    ) -> Result<PlayerEquipmentWireSnapshot, PlayerDbProjectionBlock> {
        player.equipment_wire_snapshot()
    }

 /// Вычисляет комиссию и остаток продавца по World auction-контракту.
 ///
 /// `None` заменяет единственную исходную проверку nullable `CGoodsNode*`.
 /// `dwMoneySeller` сначала читается как signed Windows `long`; затем EXE
 /// умножает его на `fAuctionFactorC`, отбрасывает дробную часть и поочерёдно
 /// ограничивает `fSxfJinMin/fSxfJinMax`. Целочисленное разложение factor-а
 /// сохраняет x87-произведение без лишнего `f32`-округления. Невалидные и
 /// out-of-range setup-значения определённо насыщаются вместо UB старого cast.
    pub fn get_opt_money_jin(
        globe_setup: &GlobeSetupSnapshot,
        seller_money: Option<u32>,
    ) -> Option<WorldAuctionSellerMoney> {
        let seller_money = seller_money? as i32;
        let mut fee =
            truncate_scaled_legacy_money(seller_money, globe_setup.auction_factor_c());
        if f64::from(fee) < f64::from(globe_setup.auction_fee_minimum()) {
            fee = truncate_legacy_money(f64::from(globe_setup.auction_fee_minimum()));
        }
        if f64::from(globe_setup.auction_fee_maximum()) < f64::from(fee) {
            fee = truncate_legacy_money(f64::from(globe_setup.auction_fee_maximum()));
        }

        let seller_money_after_fee = if fee <= seller_money {
            seller_money.wrapping_sub(fee)
        } else {
            fee = seller_money;
            0
        };
        Some(WorldAuctionSellerMoney {
            fee,
            seller_money_after_fee,
        })
    }

    pub fn get_script_file_data(&self, path: &[u8]) -> Option<&[u8]> {
        self.script_resources.get(path)
    }

    pub const fn thing_setup(&self) -> &CThingSetup {
        &self.thing_setup
    }

    pub const fn save_info_time_ms(&self) -> u32 {
        self.setup.save_info_time_ms
    }

    pub fn function_list_file_data(&self) -> Option<&[u8]> {
        self.script_resources.functions()
    }

    pub fn variable_list_file_data(&self) -> Option<&[u8]> {
        self.script_resources.variables()
    }

    pub fn initial_script_files(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.script_resources.iter().map(|(path, data)| {
            (
                legacy_c_string_prefix(path),
                legacy_c_string_prefix(data),
            )
        })
    }

    pub fn allocate_leave_word_id(&mut self) -> i32 {
        self.leave_word_id = self.leave_word_id.wrapping_add(1);
        self.leave_word_id
    }

    pub fn allocate_player_id(&mut self) -> i32 {
        self.player_id = self.player_id.wrapping_add(1);
        self.player_id as i32
    }

    pub fn clear_restore_player(&mut self) {
        self.restore_players.clear();
    }

    pub fn clear_deletion_player(&mut self) {
        self.deletion_players.clear();
    }

    pub fn clear_map_player_for_offline(&mut self) {
        let online_players = &self.online_players;
        let login_players = &self.login_players;
        self.players.retain(|player_id, _| {
            online_players.contains(player_id)
                || login_players
                    .iter()
                    .any(|entry| entry.player_id == *player_id)
        });
    }

    pub fn delete_restore_player(&mut self, player_id: u32) {
        if let Some(index) = self
            .restore_players
            .iter()
            .position(|existing| *existing == player_id)
        {
            self.restore_players.remove(index);
        }
    }

    pub fn is_restore_player_exist(&self, player_id: u32) -> bool {
        self.restore_players.contains(&player_id)
    }

    pub fn deletion_player_time(&self, player_id: u32) -> i32 {
        self.deletion_players
            .iter()
            .find(|entry| entry.player_id == player_id)
            .map_or(0, |entry| entry.deletion_time)
    }

    pub fn delete_deletion_player(&mut self, player_id: u32) {
        if let Some(index) = self
            .deletion_players
            .iter()
            .position(|entry| entry.player_id == player_id)
        {
            self.deletion_players.remove(index);
        }
    }

    pub fn append_restore_player(&mut self, player_id: u32) {
        if !self.restore_players.contains(&player_id) {
            self.restore_players.push_back(player_id);
        }
    }

    pub fn append_deletion_player(&mut self, player_id: u32, deletion_time: i32) {
        if self
            .deletion_players
            .iter()
            .any(|entry| entry.player_id == player_id)
        {
            return;
        }
        self.deletion_players.push_back(DeletionPlayerSnapshot {
            player_id,
            deletion_time,
        });
    }

 /// Создаёт byte-array копию player-map owner-а либо возвращает `None` при miss.
 ///
 /// Encoder mutates исходный player в исходных `SetPlayerOrganizing` и
 /// `UpdateProperty`; decoder начинает с нулевого cursor и `include_child=true`.
 /// Его `false` уничтожает новую копию, как virtual deleting destructor старого
 /// owner-а. Typed codec-error останавливает только неопределённую safe-границу.
    pub fn clone_map_player(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        let region_types = self.player_organizing_region_types();
        let Some(source) = self.players.get_mut(&player_id) else {
            return Ok(None);
        };

        let mut cloned = CPlayer::with_clone_decode_constructor_state();
        let mut wire = Vec::new();
        let mut updater = organizing_ctrl.player_updater(&region_types);
        let _ = source.add_to_byte_array(&mut wire, true, registry, &mut updater, coefficients)?;
        let mut cursor = 0;
        if !cloned.decord_from_byte_array(&wire, &mut cursor, true, registry, coefficients)? {
            return Ok(None);
        }
        Ok(Some(Box::new(cloned)))
    }

 /// Клонирует map-owner только если ID ещё состоит в creation-list.
 ///
 /// caller сначала линейно проходил весь `m_lCreationPlayer`, а при
 /// первом совпадении без дополнительной мутации вызывал `CloneMapPlayer`.
    pub fn clone_creation_player(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        if !self
            .creation_players
            .iter()
            .any(|creation_id| *creation_id as u32 == player_id)
        {
            return Ok(None);
        }
        self.clone_map_player(player_id, registry, organizing_ctrl, coefficients)
    }

 /// Уничтожает единственного player-owner-а по unsigned map-key.
 ///
 /// `Box`/`BTreeMap::remove` заменяют virtual deleting destructor и erase;
 /// bool сообщает caller-у только наблюдаемый факт наличия, которого старый
 /// void API наружу не выдавал.
    pub fn delete_map_player(&mut self, player_id: u32) -> bool {
        self.players.remove(&player_id).is_some()
    }

    pub fn clone_saving_player(
        &self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        let region_types = self.player_organizing_region_types();
        let mut db_data = self.db_data.lock();
        let Some(source) = db_data.players.get_mut(&player_id) else {
            return Ok(None);
        };

        let mut cloned = CPlayer::with_clone_decode_constructor_state();
        let mut wire = Vec::new();
        let mut updater = organizing_ctrl.player_updater(&region_types);
        let _ = source.add_to_byte_array(&mut wire, true, registry, &mut updater, coefficients)?;
        let mut cursor = 0;
        if !cloned.decord_from_byte_array(&wire, &mut cursor, true, registry, coefficients)? {
            return Ok(None);
        }
        Ok(Some(Box::new(cloned)))
    }

    pub fn get_team_session_id(&self, team_id: u32) -> i32 {
        self.team_session_ids.get(&team_id).copied().unwrap_or(0)
    }

    pub fn team_session_count(&self) -> usize {
        self.team_session_ids.len()
    }

    pub fn publish_team_session(&mut self, team_id: u32, session_id: i32) {
        self.team_session_ids.insert(team_id, session_id);
    }

    pub fn remove_team_session(&mut self, team_id: u32) {
        self.team_session_ids.remove(&team_id);
    }

    pub fn exit_team_player(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit {
        let Some(plug_id) = factory
            .with_team(self, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            })
            .flatten()
        else {
            return if factory.is_team(session_id) {
                WorldLoginTimeoutTeamExit::PlugMissing
            } else {
                WorldLoginTimeoutTeamExit::SessionMissingOrNotTeam
            };
        };
        if factory
            .with_teamate(self, plug_id, |teamate| teamate.exit())
            .is_some()
        {
            WorldLoginTimeoutTeamExit::Exited
        } else {
            WorldLoginTimeoutTeamExit::PlugMissing
        }
    }

    pub fn set_team_player_owner_region(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        region_id: i32,
    ) -> WorldRegionChangeTeamUpdate {
        let Some(plug_id) = factory
            .with_team(self, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            })
            .flatten()
        else {
            return if factory.is_team(session_id) {
                WorldRegionChangeTeamUpdate::PlugMissing
            } else {
                WorldRegionChangeTeamUpdate::SessionMissingOrNotTeam
            };
        };
        if factory
            .with_teamate(self, plug_id, |teamate| {
                teamate.set_owner_region_id(region_id)
            })
            .is_some()
        {
            WorldRegionChangeTeamUpdate::Updated
        } else {
            WorldRegionChangeTeamUpdate::PlugMissing
        }
    }

    pub fn replace_login_client(&mut self, client: CMyNetClient) -> bool {
        let mut previous = self.net_client.take();
        if let Some(previous) = previous.as_mut() {
            let _legacy_result = previous.close();
        }
        let previous_client_closed = previous.is_some();
        drop(previous);
        self.net_client = Some(client);
        previous_client_closed
    }

    pub fn world_number_after_cdkey_snapshot(&self) -> u32 {
        self.setup
            .world_number
            .expect("успешный CD-key snapshot проверил dwNumber")
    }

    pub const fn configured_world_number(&self) -> Option<u32> {
        self.setup.world_number
    }

    pub fn world_name(&self) -> &[u8] {
        &self.setup.name
    }

    pub fn current_login_client(&self) -> Option<&CMyNetClient> {
        self.net_client.as_ref()
    }

    pub fn current_login_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        self.net_client.as_mut()
    }

    pub fn process_login_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        self.net_client.as_mut()
    }

    pub fn process_game_server_mut(&mut self) -> Option<&mut CMyNetServer> {
        self.net_server.as_mut()
    }

 /// Воспроизводит свободный `SendErrLog`: `0x1FE08 + char + long + long + C-string`.
 ///
 /// Nullable text сохраняет исходный ранний return. Внутренние bytes после
 /// первого NUL не принадлежат старой C-строке и не входят в wire; отсутствие
 /// Login owner сохраняет обычный результат `CMessage::Send == 0`.
    pub fn send_err_log(
        &self,
        message_type: i8,
        server_ip: i32,
        world_id: i32,
        text: Option<&[u8]>,
    ) -> WorldErrorLogDelivery {
        send_err_log_to_login(
            self.current_login_client().map(CMyNetClient::send_queue),
            message_type,
            server_ip,
            world_id,
            text,
        )
    }

    pub fn current_game_server_sender(&self) -> Option<ServerCommandHandle> {
        self.net_server.as_ref().map(CMyNetServer::command_handle)
    }

    pub fn send_globe_variables_to_game_server(
        &self,
        socket_id: i32,
    ) -> WorldGlobeVariablesDelivery {
        let mut message = CMessage::new(0x0007_F80D);
        for value in self.globe_variables.values() {
            message.base_mut().add_long(value);
        }
        let sender = self.current_game_server_sender();
        let delivery = message.send_to_socket(sender.as_ref(), socket_id);
        WorldGlobeVariablesDelivery {
            socket_id,
            variables: self.globe_variables,
            delivery,
        }
    }

    pub fn connected_game_server_count(&self) -> i32 {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected)
            .fold(0_i32, |count, _| count.wrapping_add(1))
    }

    pub fn connected_game_server_indices(&self) -> impl Iterator<Item = i32> + '_ {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected)
            .map(|game_server| game_server.index as i32)
    }

    pub fn connected_game_server_count_ex(&self) -> i32 {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected && game_server.index != 5)
            .fold(0_i32, |count, _| count.wrapping_add(1))
    }

 /// Возвращает первую запись с полным byte- IP и тем же port.
 ///
 /// Входной slice соответствует байтам старой C-строки до первого NUL.
 /// Неизвестный port блокирует только сравнение уже совпавшего IP.
    pub fn game_server_by_address(
        &self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<&WorldGameServerEntry>, WorldGameServerLookupError> {
        for game_server in self.game_servers.values() {
            if game_server.ip != ip {
                continue;
            }
            match game_server.port {
                Some(candidate) if candidate == port => return Ok(Some(game_server)),
                Some(_) => {}
                None => {
                    return Err(WorldGameServerLookupError::PortUnavailable {
                        index: game_server.index,
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn connect_game_server_by_address(
        &mut self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<WorldGameServerConnectionState>, WorldGameServerLookupError> {
        let index = self
            .game_server_by_address(ip, port)?
            .map(|game_server| game_server.index);
        let Some(index) = index else {
            return Ok(None);
        };
        let game_server = self
            .game_servers
            .get_mut(&index)
            .expect("адресный поиск вернул живой ключ того же реестра");
        let previous_connected = game_server.connected;
        game_server.connected = true;
        Ok(Some(WorldGameServerConnectionState {
            index,
            previous_connected,
        }))
    }

 /// Помечает запись GameServer disconnected ветви `0x3FC02`.
 ///
 /// Найденная запись получает `connected = false`, как исходное
 /// `mov byte ptr [esi], 0`; owned-снимок несёт поля операторского
 /// лога исходного layout `tagGameServer`. `None` — записи с таким
 /// identity нет (Unknown-ветвь диспетчера).
    pub fn disconnect_game_server(
        &mut self,
        game_server_index: u32,
    ) -> Option<WorldGameServerDisconnectionState> {
        let game_server = self.game_servers.get_mut(&game_server_index)?;
        let previous_connected = game_server.connected;
        game_server.connected = false;
        Some(WorldGameServerDisconnectionState {
            index: game_server.index,
            previous_connected,
            ip: game_server.ip.clone(),
            port: game_server.port,
        })
    }

    pub fn game_server(&self, index: u32) -> Option<&WorldGameServerEntry> {
        self.game_servers.get(&index)
    }

    pub fn reset_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        let Some(game_server) = self.game_servers.get_mut(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataUpdate::GameServerNotFound;
        };
        let previous = game_server.received_player_data.replace(0);
        WorldReceivedPlayerDataUpdate::Updated {
            previous,
            current: 0,
        }
    }

    pub fn increment_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        let Some(game_server) = self.game_servers.get_mut(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataUpdate::GameServerNotFound;
        };
        let Some(previous) = game_server.received_player_data else {
            return WorldReceivedPlayerDataUpdate::Uninitialized;
        };
        let current = previous.wrapping_add(1);
        game_server.received_player_data = Some(current);
        WorldReceivedPlayerDataUpdate::Updated {
            previous: Some(previous),
            current,
        }
    }

    pub fn received_player_data(
        &self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataRead {
        let Some(game_server) = self.game_servers.get(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataRead::GameServerNotFound { legacy_value: 0 };
        };
        game_server.received_player_data.map_or(
            WorldReceivedPlayerDataRead::Uninitialized,
            WorldReceivedPlayerDataRead::Value,
        )
    }

    pub fn is_game_server_connected(&self, server_number: i32) -> bool {
        self.game_server(server_number as u32)
            .is_some_and(|game_server| game_server.connected)
    }

    pub fn to_strlwr(value: &mut [u8]) -> &mut [u8] {
        let end = value
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(value.len());
        for byte in &mut value[..end] {
            *byte = match *byte {
                0x54 => 0xAC,
                0xC0..=0xD7 | 0xDA..=0xDF => byte.wrapping_add(0x20),
                0xD8 | 0xD9 => 0xF9,
                unchanged => unchanged,
            };
        }
        value
    }

    pub fn map_player(&self, player_id: u32) -> Option<&CPlayer> {
        self.players.get(&player_id).map(Box::as_ref)
    }

 /// Повторяет `ValidatePlayerIDinCdkey`: lookup идёт только по live map,
 /// а account сравнивается старым `_strcmpi` до первого NUL.
    pub fn validate_player_id_in_cdkey(
        &self,
        account: &[u8],
        player_id: u32,
    ) -> bool {
        let Some(player) = self.map_player(player_id) else {
            return false;
        };
        legacy_c_string_prefix(player.get_account())
            .eq_ignore_ascii_case(legacy_c_string_prefix(account))
    }

    pub fn validate_db_player_id_in_cdkey(
        &self,
        account: &[u8],
        player_id: u32,
    ) -> bool {
        let db_data = self.db_data.lock();
        let Some(player) = db_data.players.get(&player_id) else {
            return false;
        };
        legacy_c_string_prefix(player.get_account())
            .eq_ignore_ascii_case(legacy_c_string_prefix(account))
    }

    pub fn set_map_player_jjc_identity(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
    ) -> bool {
        let Some(player) = self.players.get_mut(&player_id) else {
            return false;
        };
        player.set_jjc_identity(level, jjc_level);
        true
    }

    pub fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool {
        let Some(player) = self.players.get_mut(&player_id) else {
            return false;
        };
        player.set_jjc_snapshot(level, jjc_level, jjc_score, counters);
        true
    }

    pub fn player_map_keys(&self) -> Vec<u32> {
        self.players.keys().copied().collect()
    }

    pub fn update_map_player_lei_ting<Clock: PlayerLeiTingClock>(
        &mut self,
        map_key: u32,
        update_kind: u32,
        stamp: &mut LeiTingLocalTime,
        globe_setup: &GlobeSetupSnapshot,
        clock: &mut Clock,
    ) -> Result<
        Option<PlayerLeiTingUpdateReport>,
        PlayerLeiTingUpdateBlock<Clock::Block>,
    > {
        let Some(player) = self.players.get_mut(&map_key) else {
            return Ok(None);
        };
        player
            .update_lei_ting(
                update_kind,
                stamp,
                globe_setup.total_jing_li_dan_count(),
                &self.thing_setup,
                clock,
            )
            .map(Some)
    }

 /// Повторяет continuation `DisbandFaction`: snapshot имени берётся
 /// до прямой записи `m_bGetFactionData=false` тому же online map-owner-у.
    pub(crate) fn clear_disbanded_player_faction_data(
        &mut self,
        player_id: i32,
    ) -> Option<OrganizingDisbandPlayer> {
        let player_id = player_id as u32;
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players.get_mut(&player_id).map(|player| {
            let player = player.as_mut();
            let snapshot = OrganizingDisbandPlayer {
                player_id: player.get_id(),
                player_name: legacy_c_string_prefix(player.get_name()).to_vec(),
            };
            player.set_faction_data_received(false);
            snapshot
        })
    }

    pub fn add_map_player_exploit_wrapping(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate> {
        self.players
            .get_mut(&player_id)
            .map(|player| player.add_exploit_wrapping(increment))
    }

    pub fn map_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        self.players
            .iter()
            .find_map(|(&player_id, player)| {
                legacy_c_string_prefix(player.get_name())
                    .eq_ignore_ascii_case(name)
                    .then_some(player_id)
            })
            .unwrap_or(0)
    }

    pub fn append_map_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log_text: impl FnMut(&'static str),
    ) -> WorldMapPlayerAppendOutcome {
        let player_id = incoming.get_id() as u32;
        if self.players.contains_key(&player_id) {
            add_log_text("MapPlayer Not Found or NULL.");
            return WorldMapPlayerAppendOutcome::ExistingOwnerKept {
                player_id,
                incoming,
            };
        }
        self.players.insert(player_id, incoming);
        WorldMapPlayerAppendOutcome::Inserted { player_id }
    }

    pub fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer> {
        let is_online = self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id);
        if !is_online {
            return None;
        }
        self.map_player(player_id)
    }

 /// Повторяет online-list scan `0x4FB07`: account-match без назначенного
 /// GameServer не завершает поиск, а переходит к следующему list-node.
    pub fn online_player_route_by_account(
        &self,
        account: &[u8],
    ) -> Option<WorldOnlineAccountPlayerRoute> {
        let account = legacy_c_string_prefix(account);
        for &online_id in &self.online_players {
            let Some(player) = self.map_player(online_id) else {
                continue;
            };
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(account) {
                continue;
            }
            let Some(game_server) = self.get_region_game_server(player.get_region_id()) else {
                continue;
            };
            return Some(WorldOnlineAccountPlayerRoute {
                team_id: player.get_team_id(),
                owner_type: player.get_type(),
                owner_id: player.get_id(),
                game_server_index: game_server.index,
            });
        }
        None
    }

 /// Выполняет concrete `CPlayer::UpdateFactionInfo` для map-owner-а.
 ///
 /// Изменяемая organizing-проекция находится внутри player-owner-а: это
 /// позволяет синхронному доменному callback-у обновить игрока через shared
 /// game-view без второго mutable alias всего `CGame`.
    pub fn update_player_faction_info(
        &self,
        organizing: &COrganizingCtrl,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        let region_types = self.player_organizing_region_types();
        let game_server_id = self.game_server_number_by_player_id(player_id);
        let sender = self.current_game_server_sender();
        let Some(player) = self.players.get(&(player_id as u32)) else {
            return Ok(None);
        };
        let outcome = {
            let mut context = WorldPlayerFactionInfoContext {
                organizing: WorldPlayerOrganizingContext {
                    organizing,
                    region_types: &region_types,
                },
                game_server_id,
                sender,
            };
            player.update_faction_info(&mut context)
        };
        outcome.map(Some)
    }

 /// Выполняет тот же owner по faction-проекции, переданной непосредственно
 /// из точки доменной мутации.
    pub fn update_player_faction_info_from_faction(
        &self,
        faction: &CFaction,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        let region_types = self.player_organizing_region_types();
        let game_server_id = self.game_server_number_by_player_id(player_id);
        let sender = self.current_game_server_sender();
        let Some(player) = self.players.get(&(player_id as u32)) else {
            return Ok(None);
        };
        let mut context = WorldDetachedFactionInfoContext {
            organizing: WorldFactionPlayerOrganizingContext {
                faction,
                region_types: &region_types,
            },
            game_server_id,
            sender,
        };
        player.update_faction_info(&mut context).map(Some)
    }

    pub fn change_online_player_country(
        &mut self,
        player_id: u32,
        requested_country: u8,
        country_exists: impl FnOnce(u8) -> bool,
    ) -> Option<PlayerCountryChangeReport> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.change_country(requested_country, country_exists))
    }

    pub fn replace_online_player_silience_time(
        &mut self,
        player_id: u32,
        silience_time: i32,
    ) -> Option<i32> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.replace_silience_time(silience_time))
    }

    pub fn increment_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterUpdate> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.increment_murder_counters())
    }

    pub fn reset_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterReset> {
        if !self.online_players.iter().any(|&online_id| online_id == player_id) {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.reset_murder_counters())
    }

    pub fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return Ok(false);
        }
        let Some(player) = self.players.get_mut(&player_id) else {
            return Ok(false);
        };
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        Ok(true)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "positional поля wire остаются видимыми у точного call-site"
    )]

    pub fn transition_online_player_region(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        target_region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<WorldRegionChangePlayerTransition>, PlayerCodecError> {
        if !self.online_players.contains(&requested_player_id) {
            return Ok(None);
        }
        let Some(player) = self.players.get_mut(&requested_player_id) else {
            return Ok(None);
        };

        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        player.set_region_id(target_region_id);
        player.set_tile_xy(tile_x, tile_y);
        let direction_applied = player.set_direction(direction);
        let decoded_player_id = player.get_id() as u32;
        let team_id = player.get_team_id();
        let owner_type = player.get_type();
        let owner_id = player.get_id();

        self.remove_offline_player(decoded_player_id);
        let online_removal = self.remove_online_player(organizing, decoded_player_id);
        let login_time_ms = legacy_tick_ms();
        self.append_login_player(decoded_player_id, login_time_ms);

        Ok(Some(WorldRegionChangePlayerTransition {
            requested_player_id,
            decoded_player_id,
            target_region_id,
            tile_x,
            tile_y,
            direction,
            direction_applied,
            team_id,
            owner_type,
            owner_id,
            offline_removal_completed: true,
            online_removal,
            login_time_ms,
        }))
    }

    pub fn decode_online_player_lei_ting(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return Ok(false);
        }
        let Some(player) = self.players.get_mut(&player_id) else {
            return Ok(false);
        };
        player.decode_byte_array_lei_ting(source, cursor)?;
        Ok(true)
    }

    pub fn online_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_name()).eq_ignore_ascii_case(name) {
                continue;
            }
            if self
                .online_players
                .iter()
                .any(|&online_id| online_id == player_id)
            {
                return player_id;
            }
        }
        0
    }

    pub fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer> {
        let cdkey = legacy_c_string_prefix(cdkey);
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            if self
                .online_players
                .iter()
                .any(|&online_id| online_id == player_id)
            {
                return Some(player);
            }
        }
        None
    }

    pub fn online_player_count(&self) -> usize {
        self.online_players.len()
    }

    pub fn append_online_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player: &CPlayer,
    ) -> WorldOnlinePlayerAppendOutcome {
        self.append_online_player_id(organizing, player.get_id())
    }

    pub fn append_online_player_id(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player_id: i32,
    ) -> WorldOnlinePlayerAppendOutcome {
        let online_id = player_id as u32;
        let inserted = if self.online_players.contains(&online_id) {
            false
        } else {
            self.online_players.push_back(online_id);
            true
        };
        let organizing = organizing.on_player_enter_game(self, player_id);
        WorldOnlinePlayerAppendOutcome {
            inserted,
            organizing,
        }
    }

    pub fn decord_reconnected_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReconnectedPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            return Ok(WorldReconnectedPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldReconnectedPlayerOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.insert(decoded_key, player).is_some();
        let offline_inserted = self.append_offline_player_id(decoded_key);
        Ok(WorldReconnectedPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldReconnectedPlayerOwner::Created {
                replaced_existing_decoded_id,
                offline_inserted,
            },
        })
    }

    pub fn decord_server_snapshot_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldServerSnapshotPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            return Ok(WorldServerSnapshotPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldServerSnapshotPlayerOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.remove(&decoded_key).is_some();
        self.players.insert(decoded_key, player);
        Ok(WorldServerSnapshotPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldServerSnapshotPlayerOwner::Created {
                replaced_existing_decoded_id,
            },
        })
    }

 /// Принимает subtype `1` из `0x5FB02`, очищает transient pet vector и
 /// выполняет ранний offline-переход только для вновь созданного owner-а.
    pub fn decord_returned_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReturnedPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            player.clear_uncreated_pets();
            player.set_faction_data_received(false);
            return Ok(WorldReturnedPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldReturnedPlayerDecodeOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        player.clear_uncreated_pets();
        player.set_faction_data_received(false);
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.remove(&decoded_key).is_some();
        self.players.insert(decoded_key, player);
        let login_removed = self.remove_login_player(decoded_key);
        let online_removal = self.remove_online_player(organizing, decoded_key);
        let offline_inserted = self.append_offline_player_id(decoded_key);
        Ok(WorldReturnedPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldReturnedPlayerDecodeOwner::Created {
                replaced_existing_decoded_id,
                login_removed,
                online_removed_occurrences: online_removal.removed_occurrences,
                offline_inserted,
            },
        })
    }

    pub fn returned_player_snapshot(
        &self,
        player_id: u32,
    ) -> Option<WorldReturnedPlayerSnapshot> {
        let player = self.map_player(player_id)?;
        Some(WorldReturnedPlayerSnapshot {
            account: legacy_c_string_prefix(player.get_account()).to_vec(),
            name: legacy_c_string_prefix(player.get_name()).to_vec(),
            level: player.get_level(),
            team_id: player.get_team_id(),
            owner_type: player.get_type(),
            owner_id: player.get_id(),
            friend_names: (0..player.friend_count())
                .filter_map(|index| player.friend_name(index))
                .map(legacy_c_string_prefix)
                .map(<[u8]>::to_vec)
                .collect(),
        })
    }

 /// Повторяет wrapping increment и точное equality-решение `0x5FA03`.
 ///
 /// Проверка выполняется после каждого batch, даже не terminal. При равенстве
 /// счётчик сбрасывается до `GenerateDBData`, как в EXE.
    pub fn record_player_save_response(
        &mut self,
        completion_counted: bool,
    ) -> WorldPlayerSaveResponseProgress {
        let previous_responses = self.db_responses;
        if completion_counted {
            self.db_responses = self.db_responses.wrapping_add(1);
        }
        let responses_before_reset = self.db_responses;
        let connected_game_servers = self.connected_game_server_count_ex();
        let save_triggered = responses_before_reset == connected_game_servers;
        if save_triggered {
            self.db_responses = 0;
        }
        WorldPlayerSaveResponseProgress {
            previous_responses,
            completion_counted,
            responses_before_reset,
            connected_game_servers,
            save_triggered,
        }
    }

    pub fn remove_online_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player_id: u32,
    ) -> WorldOnlinePlayerRemoveOutcome {
        let old_len = self.online_players.len();
        self.online_players
            .retain(|online_id| *online_id != player_id);
        let removed_occurrences = old_len - self.online_players.len();
        let organizing = organizing.on_player_exit_game(self, player_id as i32);
        WorldOnlinePlayerRemoveOutcome {
            removed_occurrences,
            organizing,
        }
    }

 /// Переводит игроков потерянного GameServer в offline и уведомляет Login.
 ///
 /// Сначала собираются фактические `pRegion->ID` всех assignments указанного
 /// GS в signed map-order. Затем online-list и login-list в таком порядке
 /// дают уникальные player ID. Для каждого выполняются side effects:
 /// удаление всех online-дубликатов, organizing exit, `AddPlayerList`, удаление
 /// первой login-записи и unique offline append. В конце Login получает
 /// `0x1FE03`, signed count и C-string имена в том же player-list order.
 ///
 /// Вызывающая цепочка — ветвь `0x3FC02` Realm-диспетчера через шов
 /// `WorldServerMessageGameView::on_game_server_lost` (обе концовки
 /// машинной ветви завершают `OnGameServerLost(K)`).
    pub fn on_game_server_lost<AddPlayerList>(
        &mut self,
        organizing: &mut COrganizingCtrl,
        game_server_index: u32,
        mut add_player_list: AddPlayerList,
    ) -> WorldGameServerLostReport
    where
        AddPlayerList: FnMut(&[u8]),
    {
        let mut skipped_null_region_owners = 0;
        let affected_region_ids = self
            .regions
            .values()
            .filter(|assignment| assignment.game_server_index == game_server_index)
            .filter_map(|assignment| match assignment.region.as_ref() {
                Some(region) => Some(region.base().get_id()),
                None => {
 // Старый код разыменовывал повреждённый null pRegion. Такой
 // внутренний UB не является compatibility-поведением.
                    skipped_null_region_owners += 1;
                    None
                }
            })
            .collect::<Vec<_>>();

        let candidate_ids = self
            .online_players
            .iter()
            .copied()
            .chain(self.login_players.iter().map(|entry| entry.player_id))
            .collect::<Vec<_>>();
        let mut affected_players = Vec::<(u32, Vec<u8>)>::new();
        for player_id in candidate_ids {
            let Some(player) = self.players.get(&player_id) else {
                continue;
            };
            if !affected_region_ids.contains(&player.get_region_id())
                || affected_players
                    .iter()
                    .any(|(affected_id, _)| *affected_id == player_id)
            {
                continue;
            }
            affected_players.push((
                player_id,
                legacy_c_string_prefix(player.get_name()).to_vec(),
            ));
        }

        let mut players = Vec::with_capacity(affected_players.len());
        for (player_id, player_name) in &affected_players {
            let online_removal = self.remove_online_player(organizing, *player_id);
            add_player_list(player_name);
            let login_removed = self.remove_login_player(*player_id);
            let offline_inserted = self.append_offline_player_id(*player_id);
            players.push(WorldLostGameServerPlayer {
                player_id: *player_id,
                player_name: player_name.clone(),
                online_removal,
                login_removed,
                offline_inserted,
            });
        }

        let mut notice = CMessage::new(0x0001_FE03);
        notice
            .base_mut()
            .add_long(affected_players.len() as i32);
        for (_, player_name) in &affected_players {
            add_legacy_c_string(notice.base_mut(), player_name);
        }
        let login_notice_delivery = notice.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );

        WorldGameServerLostReport {
            game_server_index,
            affected_region_ids,
            skipped_null_region_owners,
            players,
            login_notice_type: 0x0001_FE03,
            login_notice_delivery,
        }
    }

    pub fn login_player_by_id(&self, player_id: u32) -> Option<&CPlayer> {
        let is_login = self
            .login_players
            .iter()
            .any(|login_player| login_player.player_id == player_id);
        if !is_login {
            return None;
        }
        self.map_player(player_id)
    }

    pub fn login_player_route_snapshot(
        &self,
        player_id: u32,
    ) -> Option<WorldLoginPlayerRouteSnapshot> {
        let player = self.login_player_by_id(player_id)?;
        Some(WorldLoginPlayerRouteSnapshot {
            map_key: player_id,
            owner_id: player.get_id(),
            region_id: player.get_region_id(),
        })
    }

 /// Сериализует полный mapped `CPlayer` с тем же concrete organizing
 /// adapter-ом, не меняя login/online/offline списки при safe-block-е.
    pub fn encode_map_player_full_snapshot(
        &mut self,
        organizing: &COrganizingCtrl,
        map_key: u32,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Vec<u8>>, PlayerCodecError> {
        let Some(mut player) = self.players.remove(&map_key) else {
            return Ok(None);
        };
        let region_types = self.player_organizing_region_types();
        let mut payload = Vec::new();
        let encoded = {
            let mut updater = WorldPlayerOrganizingContext {
                organizing,
                region_types: &region_types,
            };
            player.add_to_byte_array(
                &mut payload,
                true,
                registry,
                &mut updater,
                coefficients,
            )
        };
        self.players.insert(map_key, player);
        encoded.map(|_| Some(payload))
    }

    pub fn reset_map_player_faction_data(&self, map_key: u32) -> bool {
        let Some(player) = self.map_player(map_key) else {
            return false;
        };
        player.set_faction_data_received(false);
        true
    }

    pub fn login_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        for login_player in &self.login_players {
            let Some(player) = self.map_player(login_player.player_id) else {
                continue;
            };
            if legacy_c_string_prefix(player.get_name()) == name {
                return player.get_id() as u32;
            }
        }
        0
    }

    pub fn is_name_exist_in_map_player(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        for player in self.players.values() {
 // lower-case-ит player-buffer раньше requested-buffer.
            let player_name = copy_name_for_legacy_lowercase(player.get_name());
            let requested_name = copy_name_for_legacy_lowercase(name);
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn creation_player_by_name(
        &self,
        name: &[u8],
    ) -> Result<Option<&CPlayer>, WorldPlayerNameLookupError> {
        for (&player_id, player) in &self.players {
 // сохраняет обратный порядок двух ToStrlwr-вызовов.
            let requested_name = copy_name_for_legacy_lowercase(name);
            let player_name = copy_name_for_legacy_lowercase(player.get_name());
            if player_name == requested_name && self.creation_players.contains(&(player_id as i32))
            {
                return Ok(Some(player.as_ref()));
            }
        }
        Ok(None)
    }

    pub fn is_name_exist_in_db_creation(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        let requested_name = copy_name_for_legacy_lowercase(name);
        let db_data = self.db_data.lock();
        for player in &db_data.creation_players {
            let player_name = copy_name_for_legacy_lowercase(player.get_name());
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_name_exist_in_db_data(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        let requested_name = copy_name_for_legacy_lowercase(name);
        let db_data = self.db_data.lock();
        for player in db_data.players.values() {
            let player_name = copy_name_for_legacy_lowercase(player.get_name());
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

 /// Повторяет `IsNameExitInFaction`: общий organizing lookup ищет
 /// сначала faction, затем union и сворачивает любой match в `true`.
    pub fn is_name_exit_in_faction(
        &self,
        organizing: &COrganizingCtrl,
        name: &[u8],
    ) -> Result<bool, OrganizingNameLookupBlock> {
        organizing
            .organizing_by_name(name)
            .map(|matched| matched.is_some())
    }

 /// Выполняет полный `CPlayer::ChangeName` без global singleton-ов.
 /// Filter получает отдельную mutable копию, а последующие проверки и
 /// финальное присваивание используют исходные bytes, как owner.
    pub async fn change_map_player_name(
        &mut self,
        player_id: u32,
        requested_name: Option<&[u8]>,
        globe_setup: &GlobeSetupSnapshot,
        database: &mut dyn crate::app::world_game_view::WorldRenameDbView,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError> {
        let report = |requested_name: &[u8], legacy_result, disposition| {
            WorldPlayerNameChangeReport {
                player_id,
                requested_name: requested_name.to_vec(),
                legacy_result,
                disposition,
            }
        };

        let Some(player) = self.players.get(&player_id) else {
            return Ok(report(
                requested_name.unwrap_or_default(),
                1,
                WorldPlayerNameChangeDisposition::PlayerMissing,
            ));
        };
        let Some(requested_name) = requested_name else {
            return Ok(report(
                &[],
                1,
                WorldPlayerNameChangeDisposition::NullName,
            ));
        };
        let requested_name = legacy_c_string_prefix(requested_name);
        if requested_name.len() > 0x10 {
            return Ok(report(
                requested_name,
                8,
                WorldPlayerNameChangeDisposition::NameTooLong {
                    length: requested_name.len(),
                },
            ));
        }

        let current_name = legacy_c_string_prefix(player.get_name()).to_vec();
        let special_string = globe_setup.special_string();
        let contains_special_string = special_string.is_empty()
            || current_name
                .windows(special_string.len())
                .any(|window| window == special_string);
        if !contains_special_string {
            return Ok(report(
                requested_name,
                2,
                WorldPlayerNameChangeDisposition::CurrentNameMissingSpecialString,
            ));
        }

        let mut checked_name = requested_name.to_vec();
        if !self.check_invalid_string(&mut checked_name, false) {
            return Ok(report(
                requested_name,
                3,
                WorldPlayerNameChangeDisposition::InvalidString,
            ));
        }
        if self.is_name_exist_in_map_player(requested_name)? {
            return Ok(report(
                requested_name,
                4,
                WorldPlayerNameChangeDisposition::MapPlayerNameExists,
            ));
        }
        if self.is_name_exist_in_db_data(requested_name)? {
            return Ok(report(
                requested_name,
                5,
                WorldPlayerNameChangeDisposition::DbDataNameExists,
            ));
        }
        if self.is_name_exist_in_db_creation(requested_name)? {
            return Ok(report(
                requested_name,
                6,
                WorldPlayerNameChangeDisposition::DbCreationNameExists,
            ));
        }
        if database
            .is_name_exist(requested_name, active_transaction)
            .await
        {
            return Ok(report(
                requested_name,
                7,
                WorldPlayerNameChangeDisposition::PersistentNameExists,
            ));
        }

        self.players
            .get_mut(&player_id)
            .expect("эксклюзивный CGame borrow сохраняет map-owner через DB await")
            .set_validated_name(requested_name);
        Ok(report(
            requested_name,
            0,
            WorldPlayerNameChangeDisposition::Changed {
                previous_name: current_name,
            },
        ))
    }

    pub fn clear_creation_player(&mut self) {
        self.creation_players.clear();
    }

    pub fn creation_player_count_in_cdkey(&self, cdkey: &[u8]) -> u8 {
        let cdkey = legacy_c_string_prefix(cdkey);
        let mut count = 0_u8;
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            for &creation_id in &self.creation_players {
                if creation_id as u32 == player_id {
                    count = count.wrapping_add(1);
                }
            }
        }
        count
    }

    pub fn creation_player_ids_by_cdkey(&self, cdkey: &[u8]) -> Vec<u32> {
        let cdkey = legacy_c_string_prefix(cdkey);
        let mut player_ids = Vec::new();
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            for &creation_id in &self.creation_players {
                if creation_id as u32 == player_id {
                    player_ids.push(player_id);
                }
            }
        }
        player_ids
    }

 /// Передаёт уникального creation-игрока владеющему map после list-вставки.
 ///
 /// На обеих collision-ветвях синхронно передаёт точный payload исходного
 /// `AddLogText`; duplicate уничтожается только после возврата callback-а.
    pub fn append_creation_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log_text: impl FnMut(WorldCreationPlayerAppendLog),
    ) -> WorldCreationPlayerAppendOutcome {
        let signed_player_id = incoming.get_id();
        let player_id = signed_player_id as u32;
        if self.creation_players.contains(&signed_player_id) {
            add_log_text(WorldCreationPlayerAppendLog::Duplicate { player_id });
 // удаляет incoming до исходного UAF.
 // Box::drop сохраняет destruction; typed outcome запрещает caller-у
 // продолжить с уже уничтоженным non-owning alias.
            drop(incoming);
            return WorldCreationPlayerAppendOutcome::DuplicateReleased { player_id };
        }

        self.creation_players.push_back(signed_player_id);
        if self.players.contains_key(&player_id) {
            add_log_text(WorldCreationPlayerAppendLog::ExistingMapOwner);
 // Original уже добавил list-ID, оставил старый map-owner и вернул
 // incoming pointer caller-у. Box выражает именно это непринятое
 // владение; дальнейшая судьба объекта принадлежит OnLogMessage.
            return WorldCreationPlayerAppendOutcome::ExistingMapOwnerKept {
                player_id,
                incoming,
            };
        }

        self.players.insert(player_id, incoming);
        WorldCreationPlayerAppendOutcome::Inserted { player_id }
    }

 /// Выполняет list-order `AddOrginGoodsToPlayer`; reject одного slot-а
 /// не останавливает дальнейший обход, как исходный debug-only failure.
    pub fn add_origin_goods_to_player<Random>(
        &self,
        player: &mut CPlayer,
        player_list: &CPlayerList,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        random: &mut Random,
    ) -> Result<WorldOriginGoodsReport, WorldOriginGoodsBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
    {
        let mut entries = Vec::with_capacity(player_list.origin_equipment().len());
        for (origin_index, origin) in player_list.origin_equipment().iter().enumerate() {
            let outcome = player
                .add_origin_equipment(origin, registry, original_name_index, random)
                .map_err(|source| WorldOriginGoodsBlock {
                    origin_index,
                    source,
                })?;
            entries.push(outcome);
        }
        Ok(WorldOriginGoodsReport { entries })
    }

    pub fn append_offline_player(&mut self, player: &CPlayer) {
        let _ = self.append_offline_player_id(player.get_id() as u32);
    }

    pub fn append_offline_player_id(&mut self, player_id: u32) -> bool {
        if self.offline_players.contains(&player_id) {
            return false;
        }
        self.offline_players.push_back(player_id);
        true
    }

    pub fn clear_offline_player(&mut self) {
        self.offline_players.clear();
    }

    pub fn remove_offline_player(&mut self, player_id: u32) {
        self.offline_players
            .retain(|offline_id| *offline_id != player_id);
    }

    pub fn append_login_player(&mut self, player_id: u32, login_time_ms: u32) {
        if self
            .login_players
            .iter()
            .any(|login_player| login_player.player_id == player_id)
        {
            return;
        }
        self.login_players.push_back(WorldLoginPlayerEntry {
            player_id,
            login_time_ms,
        });
    }

 /// Возвращает первый mapped player в порядке login-list с `_strcmpi`
 /// совпавшим account, не переставляя и не очищая отсутствующие map-owner-ы.
    pub fn login_player_by_account(
        &self,
        account: &[u8],
    ) -> Option<WorldLoginAccountPlayer> {
        let account = legacy_c_string_prefix(account);
        for login in &self.login_players {
            let Some(player) = self.map_player(login.player_id) else {
                continue;
            };
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(account) {
                continue;
            }
            return Some(WorldLoginAccountPlayer {
                team_id: player.get_team_id(),
                owner_type: player.get_type(),
                owner_id: player.get_id(),
            });
        }
        None
    }

    pub fn remove_player_load_data(&self, player_id: i32) -> bool {
        self.player_load_queue
            .remove_player_load_data(player_id)
            .is_some()
    }

    pub fn push_player_load_request(
        &self,
        account: &[u8],
        player_id: u32,
        client_ip: u32,
    ) -> Result<WorldPlayerLoadRequestOutcome, WorldPlayerLoadRequestBlock> {
        let account = legacy_c_string_prefix(account);
        if account.len() >= PLAYER_LOAD_CDKEY_CAPACITY {
            return Err(WorldPlayerLoadRequestBlock {
                account_length: account.len(),
            });
        }

        let mut fixed_account = [0_u8; PLAYER_LOAD_CDKEY_CAPACITY];
        fixed_account[..account.len()].copy_from_slice(account);
        let entry = PlayerLoadQueueEntry::new(
            fixed_account,
            player_id as i32,
            client_ip,
        );
        Ok(match self.player_load_queue.push_player_load_data(entry) {
            PlayerLoadPushOutcome::Queued => WorldPlayerLoadRequestOutcome::Queued,
            PlayerLoadPushOutcome::Duplicate(_) => WorldPlayerLoadRequestOutcome::Duplicate,
        })
    }

    pub fn remove_login_player(&mut self, player_id: u32) -> bool {
        let Some(index) = self
            .login_players
            .iter()
            .position(|login_player| login_player.player_id == player_id)
        else {
            return false;
        };
        let _ = self.login_players.remove(index);
        true
    }

    pub fn region(&self, region_id: i32) -> Option<&WorldRegionAssignment> {
        self.regions.get(&region_id)
    }

    pub fn refresh_owned_city_org(
        &self,
        organizing: &COrganizingCtrl,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<WorldOwnedCityRefreshOutcome, FactionInitialPropertyBlock> {
        let country_id = organizing.country_by_faction(faction_id)?;
        Ok(self.refresh_owned_city_org_with_country(
            region_id,
            faction_id,
            union_id,
            country_id,
        ))
    }

 /// Применяет уже разрешённую organizing-проекцию без повторного заимствования
 /// controller-а из синхронного доменного callback-а.
    pub fn refresh_owned_city_org_with_country(
        &self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) -> WorldOwnedCityRefreshOutcome {
        let Some(assignment) = self.regions.get(&region_id) else {
            return WorldOwnedCityRefreshOutcome::RegionNotFound;
        };
        let Some(region) = assignment.region.as_ref() else {
            return WorldOwnedCityRefreshOutcome::NullRegionPointer;
        };
        let country_id = country_id.unwrap_or(0);
        region.base().set_owned_city_org(faction_id, union_id);
        region
            .base()
            .region_base()
            .set_country(country_id);

        let mut message = CMessage::new(0x0007_FE27);
        message.base_mut().add_long(region_id);
        message.base_mut().add_long(faction_id);
        message.base_mut().add_long(union_id);
        message.base_mut().add_byte(country_id);
        let delivery = message.send_all(self.current_game_server_sender().as_ref());
        WorldOwnedCityRefreshOutcome::Refreshed(
            WorldOwnedCityRefreshReport {
                region_id,
                faction_id,
                union_id,
                country_id,
                delivery,
            },
        )
    }

    pub fn creation_region_base(&self, region_id: i32) -> Option<&CRegion> {
        self.regions
            .get(&region_id)?
            .region
            .as_ref()
            .map(WorldRegionOwner::base)
            .map(CWorldRegion::creation_region_base)
    }

    pub fn set_region_param_from_game_server(
        &mut self,
        region_id: i32,
        current_tax_rate: i32,
        today_total_tax: u32,
        total_tax: u32,
    ) -> WorldRegionParamUpdateOutcome {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return WorldRegionParamUpdateOutcome::RegionNotFound;
        };
        let Some(region) = assignment.region.as_mut() else {
            return WorldRegionParamUpdateOutcome::NullRegionPointer;
        };
        region.base_mut().set_param_from_gs(
            current_tax_rate,
            today_total_tax,
            total_tax,
        );
        WorldRegionParamUpdateOutcome::Applied
    }

    pub fn decode_region_param_from_game_server(
        &mut self,
        region_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> WorldRegionParamDecodeOutcome {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return WorldRegionParamDecodeOutcome::RegionNotFound;
        };
        let Some(region) = assignment.region.as_mut() else {
            return WorldRegionParamDecodeOutcome::NullRegionPointer;
        };
        WorldRegionParamDecodeOutcome::Decoded(
            region
                .base_mut()
                .decord_region_param_from_byte_array(source, cursor, true),
        )
    }

 /// Сериализует initial-config регионы в signed map-order и передаёт каждый
 /// элемент visitor-у до перехода к следующему узлу.
    pub fn visit_initial_region_snapshots<Visit>(
        &self,
        target_game_server_index: u32,
        mut visit: Visit,
    ) -> Result<(), WorldInitialRegionSnapshotBlock>
    where
        Visit: FnMut(WorldInitialRegionSnapshot),
    {
        for (&map_key, assignment) in &self.regions {
            let region = assignment.region.as_ref().ok_or(
                WorldInitialRegionSnapshotBlock {
                    map_key,
                    source: WorldInitialRegionSnapshotSource::MissingRegionOwner,
                },
            )?;
            let region_id = region.base().get_id();
            let mut payload = Vec::new();
            let kind = if assignment.game_server_index == target_game_server_index {
                let region_type = assignment.region_type.ok_or(
                    WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::UninitializedRegionType,
                    },
                )?;
                region
                    .add_full_initial_snapshot(&mut payload)
                    .map_err(|source| WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::Full(source),
                    })?;
                WorldInitialRegionSnapshotKind::Assigned { region_type }
            } else {
                let _ = region
                    .base()
                    .add_to_byte_array_for_proxy(&mut payload, true)
                    .map_err(|source| WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::Proxy(source),
                    })?;
                WorldInitialRegionSnapshotKind::Proxy
            };
            visit(WorldInitialRegionSnapshot {
                map_key,
                region_id,
                kind,
                payload,
            });
        }
        Ok(())
    }

    pub fn region_name(&self, region_id: i32) -> WorldRegionNameLookup<'_> {
        let Some(assignment) = self.region(region_id) else {
            return WorldRegionNameLookup::RegionNotFound;
        };
        let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
            return WorldRegionNameLookup::NullRegionPointer;
        };
        WorldRegionNameLookup::Name(region.get_name())
    }

 /// Повторяет ordered `GetRegion(char const*)` с case-sensitive `strcmp`.
 ///
 /// Route-поля являются typed snapshot найденного `tagRegion`, а не новой
 /// ступенью поиска; null owner безопасно учитывается вместо старого UB.
    pub fn named_region_lookup(&self, name: &[u8]) -> WorldNamedRegionLookup {
        let name = legacy_c_string_prefix(name);
        let mut skipped_null_owners = 0;
        for assignment in self.regions.values() {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
 // В EXE `GetRegion(name)` разыменовывал null `pRegion`. Это
 // внутренний UB повреждённого состояния, а не wire-контракт.
                skipped_null_owners += 1;
                continue;
            };
            if legacy_c_string_prefix(region.get_name()) != name {
                continue;
            }
            let game_server = self.game_server(assignment.game_server_index);
            return WorldNamedRegionLookup {
                skipped_null_owners,
                matched: Some(WorldNamedRegionMatch {
                    region_id: region.get_id(),
                    game_server_index: assignment.game_server_index,
                    game_server_entry_found: game_server.is_some(),
                    game_server_connected: game_server.is_some_and(|server| server.connected),
                }),
            };
        }
        WorldNamedRegionLookup {
            skipped_null_owners,
            matched: None,
        }
    }

    pub fn region_routes_by_owner_id(&self, region_id: i32) -> WorldRegionIdRouteScan {
        let mut skipped_null_owners = 0;
        let mut matching_region_keys = 0;
        let mut routes = Vec::new();
        for (&map_key, assignment) in &self.regions {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
                skipped_null_owners += 1;
                continue;
            };
            if region.get_id() != region_id {
                continue;
            }
            matching_region_keys += 1;
            let game_server_id = self.game_server_number_by_region_id(map_key);
            if game_server_id != 0 {
                routes.push(WorldRegionIdRoute {
                    map_key,
                    game_server_id,
                });
            }
        }
        WorldRegionIdRouteScan {
            skipped_null_owners,
            matching_region_keys,
            routes,
        }
    }

    pub fn has_materialized_region(&self, region_id: i32) -> bool {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .is_some()
    }

    pub fn region_owned_faction_id(&self, region_id: i32) -> Option<i32> {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .map(|region| region.base().get_owned_city_faction())
    }

    pub fn region_country_id(&self, region_id: i32) -> Option<u8> {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .and_then(|region| region.base().region_base().country())
    }

    pub fn get_region_game_server(&self, region_id: i32) -> Option<&WorldGameServerEntry> {
        let region = self.region(region_id)?;
        self.game_server(region.game_server_index)
    }

    pub fn game_server_number_by_region_id(&self, region_id: i32) -> i32 {
        self.get_region_game_server(region_id)
            .map_or(0, |game_server| game_server.index as i32)
    }

    pub fn player_game_server(&self, player_id: i32) -> Option<&WorldGameServerEntry> {
        let player = self.online_player_by_id(player_id as u32)?;
        self.get_region_game_server(player.get_region_id())
    }

    pub fn game_server_number_by_player_id(&self, player_id: i32) -> i32 {
        let Some(player) = self.online_player_by_id(player_id as u32) else {
            return 0;
        };
        let Some(game_server) = self.get_region_game_server(player.get_region_id()) else {
            return 0;
        };
        game_server.index as i32
    }

    pub fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError> {
        let sender = self.current_game_server_sender();
        message.send_to_map_id(sender.as_ref(), map_id)
    }

    pub fn begin_game_server_ping(&mut self) -> (usize, u32) {
        self.ping_in_progress = true;
        let cleared_responses = self.ping_game_servers.len();
        self.ping_game_servers.clear();
        let started_at_ms = legacy_tick_ms();
        self.last_ping_game_server_time_ms = started_at_ms;
        (cleared_responses, started_at_ms)
    }

    pub fn record_game_server_ping(&mut self, response: WorldPingGameServerInfo) -> usize {
        self.ping_game_servers.push(response);
        self.ping_game_servers.len()
    }

    pub fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool {
        self.bai_tan.add_item_to_bai_tan_request_list(ip, player_id)
    }

    pub fn add_item_to_bai_tan_list(
        &mut self,
        player_id: i32,
        ip: u32,
    ) -> WorldBaiTanRegistration {
        let game_server_index = self.game_server_number_by_player_id(player_id);
        self.bai_tan
            .add_item_to_bai_tan_list(player_id, ip, game_server_index)
    }

    pub fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval {
        self.bai_tan.del_item_from_bai_tan_list(player_id)
    }

    pub fn done_bai_tan_list(&mut self) -> WorldDoneBaiTanListReport {
        let requests = self.bai_tan.request_entries();
        let mut completions = Vec::with_capacity(requests.len());
        for (requested_ip, player_id) in requests {
            let registration = self.add_item_to_bai_tan_list(player_id, requested_ip);

            let mut response = CMessage::new(0x0008_040D);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(1);
            let route_game_server_index = self.game_server_number_by_player_id(player_id);
            let delivery = self.send_msg_to_game_server(route_game_server_index, &response);
            completions.push(WorldBaiTanCompletion {
                requested_ip,
                player_id,
                registration,
                route_game_server_index,
                delivery,
            });
        }
        let cleared_requests = self.bai_tan.clear_requests();
        WorldDoneBaiTanListReport {
            completions,
            cleared_requests,
        }
    }

    pub fn assign_login_server_id(&mut self, login_server_id: i32) -> i32 {
        std::mem::replace(&mut self.login_server_id, login_server_id)
    }

    pub const fn login_server_id(&self) -> i32 {
        self.login_server_id
    }

    pub fn reset_honor_eliminate_info(&mut self, rank_mask: u32) -> bool {
        for player in self.players.values_mut() {
            player.reset_honor_eliminate_info(rank_mask);
        }
        self.honor_eliminate_list.clear();
        self.player_data_queue
            .reset_honor_eliminate_info(rank_mask);
        true
    }

    pub fn register_honor_eliminator(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration {
        if self.online_player_by_id(player_id).is_none() {
            return WorldHonorEliminatorRegistration::MissingOnlinePlayer;
        }

        let eliminators = self.honor_eliminate_list.entry(player_id).or_default();
        if eliminators
            .iter()
            .any(|tracked_id| *tracked_id == eliminator_id)
        {
            return WorldHonorEliminatorRegistration::Duplicate;
        }
        eliminators.push_back(eliminator_id);
        WorldHonorEliminatorRegistration::Accepted
    }

    pub fn queue_local_world_message(
        &self,
        message: CMessage,
    ) -> Result<(), WorldLocalMessageQueueBlock> {
        let message_type = message.message_type();
        let Some(net_server) = self.net_server.as_ref() else {
 // Исходный владелец безусловно разыменовывал
 // обязательный s_pNetServer. Safe Rust не подменяет этот путь
 // прямым вызовом handler-а и сохраняет границу FIFO.
            return Err(WorldLocalMessageQueueBlock { message_type });
        };
        net_server.publish_local_message(message);
        Ok(())
    }

}
