//! `MainLoop` и stage-функции хода `CGame` из `worldserver/game.cpp/.h`
//! (см. [`crate::app::world_game`]).
//!
//! `MainLoop` (`1:00018a00`): 32-битные wrapping ticks и строгие интервалы;
//! каждый сетевой/message turn снимает FIFO один раз, опубликованное
//! callback-ом остаётся следующему проходу; порядок AI, сообщений, reconnect,
//! обслуживания, сохранения и рассылок не распараллеливается.
//!
//! Статус: тела `process_message` (`1:00a30`) и `ai` (`1:138a0`) сверены
//! полным машинным разбором точной пары — VERIFIED; единственная правка по
//! ней — statement-order DIFF-A1 в `ai` (см. тело). Остальные stage-функции
//! полагаются на данные и отчёты
//! `app::world_main_loop_data`/`app::world_hub_data`.
//!
//! Имена и проекции типов — Realm/Shared формы (см. `crate::app::world_game`).

use crate::activities::attackcitysys::{AttackCityCallbacks, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarCallbacks, CountryWarSys};
use crate::activities::factionwarsys::{CFactionWarSys, FactionWarStopBlock};
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarCallbacks};
use crate::activities::jjcmaintenanceworker::WorldJjcWeekClearWorker;
use crate::activities::jjcsystem::{CJJcSystem, JjcRunBlock, JjcRunConfig, JjcRunContext};
use crate::activities::leiting::{CLeiTing, LeiTingBlock, LeiTingLocalTime, LeiTingRunReport};
use crate::activities::leitingresetworker::WorldLeiTingResetWorker;
use crate::activities::misc::CopyNumberTimerState;
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarCallbacks};
use crate::app::misc_game::legacy_tick_ms;
use crate::app::playerdataqueue::WorldMainLoopPlayerDataQueueStageReport;
use crate::app::servermessage::on_login_client_reconnected;
use crate::app::world_client::CMyNetClient;
use crate::app::world_dispatch::{WorldCountryDemiseEffects, WorldCountryExileResultEffects, WorldOrganizingDisbandEffects, WorldUnionApplicationEffectCallbacks, WorldUnionApplicationEffects, add_legacy_c_string, drain_union_application_runtime, legacy_c_string_prefix, process_world_message};
use crate::app::world_game::{CGame, WorldMapPlayerAppendOutcome, WorldTimerHandler};
use crate::app::world_game_view::{WorldFriendPresenceUpdate, WorldPlayerDataQueueRejectReason, WorldProcessPlayerDataQueueBlock, WorldProcessPlayerDataQueueError, WorldProcessPlayerDataQueueOutcome};
use crate::app::world_hub_data::{ProcessedWorldEvent, WorldLoadedPlayerRouteOrder, WorldMainLoopAiStageReport, WorldMainLoopClockState, WorldMainLoopFactionWarBlock, WorldMainLoopFactionWarEffects, WorldMainLoopInitializationState, WorldMainLoopLargessState, WorldMainLoopLoginReleaseState, WorldMainLoopProfileState, WorldMainLoopSessionFactoryStageReport, WorldMainLoopTailClockState, WorldMessageSource, WorldProcessMessageOutcome, WorldProcessMessageStageState};
use crate::app::world_hub_entries::{WorldGameAiReport, WorldSystemBroadcastDisposition, WorldSystemBroadcastTarget};
use crate::app::world_main_loop_contexts::{WorldJjcRuntimeContext, WorldJjcWorkerContext, WorldLeiTingRuntimeContext, WorldLeiTingWorkerContext};
use crate::app::world_main_loop_data::{WorldDbMiscDeliveryContext, WorldLoginTimeoutEntryOutcome, WorldLoginTimeoutFriendOutcome, WorldLoginTimeoutReport, WorldMainLoopBaiTanJjcStageReport, WorldMainLoopBlock, WorldMainLoopCallbacks, WorldMainLoopConfiguration, WorldMainLoopDbMiscStageReport, WorldMainLoopFactionWarStageReport, WorldMainLoopMinuteStageBlock, WorldMainLoopMinuteStageReport, WorldMainLoopNetSessionStageReport, WorldMainLoopOwners, WorldMainLoopPacingReport, WorldMainLoopPingError, WorldMainLoopPingStageReport, WorldMainLoopReport, WorldMainLoopResult, WorldMainLoopSaveStageDisposition, WorldMainLoopSaveStageReport, WorldMainLoopStateOwners, WorldMainLoopTailStageReport, WorldMainLoopTimerStageBlock, WorldMainLoopTimerStageReport, WorldRefreshExternalCounts};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::world_reload::reload_profiles;
use crate::app::world_reload_profiles::{WorldAuctionBangMaintenanceDisposition, WorldHonorRanksMaintenanceBlock, WorldHonorRanksMaintenanceDisposition, WorldMainLoopLargessGateReport, WorldMainLoopMaintenanceBlock, WorldMainLoopMaintenanceReport, WorldMainLoopProfileReport, WorldMainLoopProfileSnapshot, WorldMainLoopRefreshDisposition, WorldMainLoopRefreshStageReport, WorldMainLoopResourceContext, WorldPlayerRanksMaintenanceDisposition, WorldPlayerRanksRequestState, WorldProcessMessageError, WorldProcessMessageStageReport, WorldRefreshSnapshotBlock, WorldReloadProfilesReport, current_country_save_limits, initialize_main_loop_profile_if_needed, initialize_main_loop_refresh_if_needed, initialize_main_loop_save_if_needed, initialize_main_loop_tail_clocks, legacy_refresh_count, start_main_loop_profile_stage, update_main_loop_current_tick};
use crate::app::world_runtime::{PlayerRanksStatRunBlock, PlayerRanksStatRunReport};
use crate::app::world_server::WorldServerEvent;
use crate::app::worldserver::{AddLogTextDisposition, WorldLogLocalTime, WorldLogTextOwner, WorldOnlinePlayerRemoveOutcome, WorldRefreshInfoCurrent, WorldRefreshInfoHighWater, WorldRefreshSaveState, WorldSaveThreadHandleState, refresh_info_text};
use crate::persistence::writelog::{WorldFactionLogWrite, WorldWriteLogCommand};
use crate::auction::auctionlog::{AuctionBangUpdateOutcome, CAuctionLog};
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerCodecError};
use crate::characters::playerranks::CPlayerRanks;
use crate::content::cgoods::CGoods;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::CSkillFactory;
use crate::content::{TimeToReturn, TimeToReturnCallbacks};
use crate::content::variablelist::CVariableList;
use crate::organizations::country::CountryKingSaveLimits;
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::{COrganizingCtrl, OrganizingDisbandOutcome};
use crate::organizations::organizingparam::COrganizingParam;
use crate::persistence::dbmisc::{CDbMisc, DbMiscContext, DbMiscDeliveryContext, DbMiscDoneOutBlock};
use crate::persistence::largess::{LoadLargessBlock, LoadLargessReport, TiberiusLargess};
use crate::persistence::rsplayer::{PlayerRanksStatOutcome, RsPlayerOwner, TiberiusRsPlayer};
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::savedb::SaveDataLifecycleState;
use crate::persistence::saveworker::WorldSaveRuntimeContext;
use crate::persistence::world_db_data_collect::{WorldRunSavePreGateReport, WorldRunSaveTriggerReport};
use crate::sessions::csessionfactory::CSessionFactory;
use nebokrai_shared::resources::{CGodsBattleConf, GlobeSetupSnapshot};
use nebokrai_shared::runtime::{AsyncTimerRunBlock, CNetSessionManager, CTimer};
use nebokrai_shared::values::TagTime;
use parking_lot::Mutex;
use std::sync::Arc;
use crate::app::organsysmessage::WorldOrganizingSessionRuntimeOwner as WorldUnionApplicationRuntimeOwner;
use crate::activities::jjcsystem::GlobeSetupJjcWorldConfig;

#[derive(Debug)]
pub struct WorldPlayerLargessLoadReport {
    #[allow(dead_code, reason = "поле отчёта конструируется для совместимости формы исходного отчёта; читателей нет")]
    pub(crate) load: LoadLargessReport,
    #[allow(dead_code, reason = "поле отчёта конструируется для совместимости формы исходного отчёта; читателей нет")]
    pub(crate) write_log_queue_length: Option<usize>,
}

impl CGame {
    pub fn publish_largess_load_log(
        &self,
        report: &mut LoadLargessReport,
    ) -> Option<usize> {
        report.write_log.take().map(|record| {
            self.push_write_log_command(WorldWriteLogCommand::LargessLog(record))
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_player_largess<Random, Upgrade>(
        &self,
        largess: &TiberiusLargess,
        player: &mut CPlayer,
        registry: &GoodsBasePropertiesRegistry,
        gold_coin_index: u32,
        gold_coin_limit: u32,
        use_log_system: bool,
        random: &mut Random,
        upgrade_equipment: &mut Upgrade,
    ) -> Result<WorldPlayerLargessLoadReport, LoadLargessBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
        Upgrade: FnMut(&mut CGoods, i32) + ?Sized,
    {
        let mut load = largess.load_largess(
            player,
            registry,
            gold_coin_index,
            gold_coin_limit,
            use_log_system,
            random,
            upgrade_equipment,
        )?;
        let write_log_queue_length = self.publish_largess_load_log(&mut load);
        Ok(WorldPlayerLargessLoadReport {
            load,
            write_log_queue_length,
        })
    }

 /// Выполняет полный `CGame::AI` в исходном порядке.
 ///
 /// Ordered region map вызывает отдельного virtual owner-а только для
 /// ненулевого `pRegion`. После всего обхода снимается один общий broadcast
 /// tick; list cadence, оба random-вызова, wire-поля, send и последующие
 /// мутации сохраняют исходный порядок и wrapping 32-битную арифметику.
    pub fn ai<GetTick, Random>(
        &mut self,
        get_tick: &mut GetTick,
        random: &mut Random,
    ) -> WorldGameAiReport
    where
        GetTick: FnMut() -> u32,
        Random: FnMut(i32) -> i32,
    {
        let mut region_ids_run = Vec::new();
        for (&region_id, assignment) in &mut self.regions {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            region.ai();
            region_ids_run.push(region_id);
        }

        let broadcast_tick_ms = get_tick();
        let now_seconds = broadcast_tick_ms / 1000;
        let mut broadcasts = Vec::with_capacity(self.system_broadcasts.len());
        for index in 0..self.system_broadcasts.len() {
            let broadcast = &self.system_broadcasts[index];
            let elapsed_seconds = now_seconds.wrapping_sub(broadcast.last_notify_time_seconds);
            if broadcast.interval_seconds >= elapsed_seconds {
                broadcasts.push(WorldSystemBroadcastDisposition::Waiting {
                    elapsed_seconds,
                    interval_seconds: broadcast.interval_seconds,
                });
                continue;
            }

            let roll = random(100);
            if (roll as u32) >= broadcast.odds {
                broadcasts.push(WorldSystemBroadcastDisposition::OddsMissed {
                    roll,
                    odds: broadcast.odds,
                });
                continue;
            }

            let region_id = broadcast.region_id;
            let import_level = broadcast.import_level;
            let text_color = broadcast.text_color;
            let back_color = broadcast.back_color;
            let message_text = broadcast.message.clone();
            let min_time_seconds = broadcast.min_time_seconds;
            let max_time_seconds = broadcast.max_time_seconds;

            let mut message = CMessage::new(0x0007_FA03);
            message.base_mut().add_long(region_id);
            message.base_mut().add_long(import_level);
            message.base_mut().add_ulong(text_color);
            message.base_mut().add_ulong(back_color);
            add_legacy_c_string(message.base_mut(), &message_text);

            let target = if region_id == 0 {
                let sender = self.current_game_server_sender();
                WorldSystemBroadcastTarget::All {
                    delivery: message.send_all(sender.as_ref()),
                }
            } else {
                let game_server_index = self
                    .get_region_game_server(region_id)
                    .map(|game_server| game_server.index);
                let delivery = game_server_index.map(|game_server_index| {
                    let sender = self.current_game_server_sender();
                    message.send_to_map_id(sender.as_ref(), game_server_index as i32)
                });
                WorldSystemBroadcastTarget::Region {
                    region_id,
                    game_server_index,
                    delivery,
                }
            };

            let broadcast = &mut self.system_broadcasts[index];
 // DIFF-A1 (машинная досверка): оригинал записывает
 // `last_notify_time = now` до вызова `random(max-min)`; порядок —
 // контракт, сам по себе эффекта не даёт.
            broadcast.last_notify_time_seconds = now_seconds;
            let random_range = (max_time_seconds as i32).wrapping_sub(min_time_seconds as i32);
            let assigned_interval_seconds =
                random(random_range).wrapping_add(min_time_seconds as i32) as u32;
            broadcast.interval_seconds = assigned_interval_seconds;
            broadcasts.push(WorldSystemBroadcastDisposition::Broadcast {
                roll,
                target,
                assigned_last_notify_time_seconds: now_seconds,
                assigned_interval_seconds,
            });
        }

        WorldGameAiReport {
            region_ids_run,
            broadcast_tick_ms,
            broadcasts,
            legacy_result: 1,
        }
    }

 /// Выполняет непосредственно окружающий `CGame::AI` profiling-порядок.
 ///
 /// Первый tick закрывает SavePoint-стадию, второй становится общим началом
 /// AI, внутренний tick принадлежит самому `CGame::AI`, четвёртый закрывает
 /// AI. Следующим исходным owner-ом остаётся готовый
 /// `process_message_main_loop_stage`.
    pub fn run_main_loop_ai_stage<GetTick, Random>(
        &mut self,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
        mut random: Random,
    ) -> WorldMainLoopAiStageReport
    where
        GetTick: FnMut() -> u32,
        Random: FnMut(i32) -> i32,
    {
        let previous_stage_finished_at_ms = get_tick();
        let save_point_elapsed_ms =
            previous_stage_finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.ai_calls = profile_state.ai_calls.wrapping_add(1);
        profile_state.save_point_time_ms = profile_state
            .save_point_time_ms
            .wrapping_add(save_point_elapsed_ms);

        let ai_started_at_ms = get_tick();
        clocks.stage_started_at_ms = ai_started_at_ms;
        let ai = self.ai(&mut get_tick, &mut random);
        let ai_finished_at_ms = get_tick();
        let ai_elapsed_ms = ai_finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.ai_time_ms = profile_state.ai_time_ms.wrapping_add(ai_elapsed_ms);

        WorldMainLoopAiStageReport {
            previous_stage_finished_at_ms,
            save_point_elapsed_ms,
            accumulated_save_point_time_ms: profile_state.save_point_time_ms,
            ai_calls: profile_state.ai_calls,
            ai_started_at_ms,
            ai,
            ai_finished_at_ms,
            ai_elapsed_ms,
            accumulated_ai_time_ms: profile_state.ai_time_ms,
        }
    }

 /// Обрабатывает server FIFO, затем FIFO текущего Login client.
 ///
 /// Размер server FIFO фиксируется первым. Reconnect-событие из него может
 /// заменить Login owner до снятия второго snapshot; новые сообщения остаются
 /// следующему проходу. Каждый opcode выбирает ровно одного component owner,
 /// а неизвестный opcode сохраняет no-op соответствующего dispatcher-а.
 /// Terminal session actions применяются FIFO до следующего сообщения.
 /// Синхронные в оригинале DB-ветви завершаются до перехода к следующему slot;
 /// конкретные opcode и их payload-контракты описаны в message-модулях.
    pub async fn process_message<TimerCallback, DbMiscContextOwner, JjcContext>(
        &mut self,
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
    ) -> Result<WorldProcessMessageOutcome, WorldProcessMessageError>
    where
        TimerCallback: Copy,
        DbMiscContextOwner: DbMiscContext,
        JjcContext: JjcRunContext + ?Sized,
    {
        let server_started_at = legacy_tick_ms();
        let mut server_remaining = self
            .net_server
            .as_ref()
            .ok_or(WorldProcessMessageError::MissingNetworkServerOwner)?
            .pending_events();
        let initial_server_events = server_remaining;
        let mut server_slots_visited = 0_i32;
        let mut events = Vec::new();

        while server_remaining > 0 {
            let event = self
                .net_server
                .as_ref()
                .expect("server-owner проверен до snapshot")
                .pop_received_event();
            if let Some(event) = event {
                match event {
                    WorldServerEvent::Message(message) => {
                        events.push(process_world_message(
                            self,
                            honor_ranks,
                            player_ranks,
                            increment_log,
                            auction_log,
                            db_misc,
                            &mut *db_misc_context,
                            organizing,
                            organizing_parameters,
                            country_handler,
                            country_parameters,
                            country_war,
                            four_nation_war,
                            current_country_save_limits(country_parameters)?,
                            faction_war_sys,
                            attack_city,
                            attack_city_callbacks,
                            village_war,
                            goods_war,
                            timer,
                            village_war_callbacks,
                            &mut *resources,
                            &mut *load_player_largess,
                            net_sessions,
                            jjc,
                            jjc_context,
                            application_runtime,
                            application_callbacks,
                            &mut *write_faction_create_log,
                            &mut *write_faction_title_log,
                            &mut *write_faction_purview_log,
                            &mut *write_faction_apply_log,
                            &mut *write_faction_join_log,
                            &mut *write_faction_quit_log,
                            &mut *write_faction_fire_out_log,
                            &mut *write_faction_master_log,
                            &mut *write_faction_disband_log,
                            &mut *rs_player,
                            player_database.as_deref_mut(),
                            Arc::clone(&save_lifecycle),
                            &mut *save_thread_handle,
                            &mut *save_runtime,
                            &mut *session_factory,
                            general_variables.as_deref_mut(),
                            &mut *gods_battle,
                            &mut *skills,
                            rs_gods_battle.as_deref_mut(),
                            gods_battle_database.as_deref_mut(),
                            &mut *add_log_text,
                            update_player,
                            WorldMessageSource::GameServer,
                            message,
                        )
                        .await);
                    }
                    WorldServerEvent::LoginClientReconnected(client) => {
                        let replacement = on_login_client_reconnected(self, client)
                            .map_err(WorldProcessMessageError::ServerMessage)?;
                        events.push(ProcessedWorldEvent::LoginClientReconnected(replacement));
                    }
                }
            }
            server_remaining -= 1;
            server_slots_visited += 1;
        }
        self.game_server_message_time_ms = self
            .game_server_message_time_ms
            .wrapping_add(legacy_tick_ms().wrapping_sub(server_started_at));

        let login_started_at = legacy_tick_ms();
        let initial_login_messages = self.net_client.as_ref().map(CMyNetClient::pending_messages);
        let mut login_remaining = initial_login_messages.unwrap_or(0);
        let mut login_slots_visited = 0_i32;
        while login_remaining > 0 {
            let message = self
                .net_client
                .as_ref()
                .and_then(CMyNetClient::pop_received_message);
            if let Some(message) = message {
                events.push(process_world_message(
                    self,
                    honor_ranks,
                    player_ranks,
                    increment_log,
                    auction_log,
                    db_misc,
                    &mut *db_misc_context,
                    organizing,
                    organizing_parameters,
                    country_handler,
                    country_parameters,
                    country_war,
                    four_nation_war,
                    current_country_save_limits(country_parameters)?,
                    faction_war_sys,
                    attack_city,
                    attack_city_callbacks,
                    village_war,
                    goods_war,
                    timer,
                    village_war_callbacks,
                    &mut *resources,
                    &mut *load_player_largess,
                    net_sessions,
                    jjc,
                    jjc_context,
                    application_runtime,
                    application_callbacks,
                    &mut *write_faction_create_log,
                    &mut *write_faction_title_log,
                    &mut *write_faction_purview_log,
                    &mut *write_faction_apply_log,
                    &mut *write_faction_join_log,
                    &mut *write_faction_quit_log,
                    &mut *write_faction_fire_out_log,
                    &mut *write_faction_master_log,
                    &mut *write_faction_disband_log,
                    &mut *rs_player,
                    player_database.as_deref_mut(),
                    Arc::clone(&save_lifecycle),
                    &mut *save_thread_handle,
                    &mut *save_runtime,
                    &mut *session_factory,
                    general_variables.as_deref_mut(),
                    &mut *gods_battle,
                    &mut *skills,
                    rs_gods_battle.as_deref_mut(),
                    gods_battle_database.as_deref_mut(),
                    &mut *add_log_text,
                    update_player,
                    WorldMessageSource::LoginServer,
                    message,
                )
                .await);
            }
            login_remaining -= 1;
            login_slots_visited += 1;
        }
        self.login_server_message_time_ms = self
            .login_server_message_time_ms
            .wrapping_add(legacy_tick_ms().wrapping_sub(login_started_at));

        Ok(WorldProcessMessageOutcome {
            legacy_result: 1,
            initial_server_events,
            initial_login_messages,
            server_slots_visited,
            login_slots_visited,
            events,
            game_server_message_time_ms: self.game_server_message_time_ms,
            login_server_message_time_ms: self.login_server_message_time_ms,
        })
    }

 /// Выполняет внешний MainLoop profiling call-site `ProcessMessage`.
 ///
 /// Safe block не получает придуманных end/next-stage ticks и не меняет
 /// накопитель: исходный невозвратившийся путь их не достигал.
    pub async fn process_message_main_loop_stage<
        TimerCallback,
        DbMiscContextOwner,
        JjcContext,
        GetTick,
    >(
        &mut self,
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
        player_database: Option<&mut WorldTdsClient>,
        save_lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
        session_factory: &mut CSessionFactory,
        general_variables: Option<&mut CVariableList>,
        gods_battle: &mut CGodsBattleConf,
        skills: &mut CSkillFactory,
        rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
        gods_battle_database: Option<&mut WorldTdsClient>,
        log: &mut WorldLogTextOwner,
        get_log_local_time: &mut dyn FnMut() -> WorldLogLocalTime,
        put_log_info: &mut dyn FnMut(&[u8]),
        update_player: &mut dyn FnMut(i32),
        clocks: &mut WorldMainLoopClockState,
        state: &mut WorldProcessMessageStageState,
        mut get_tick: GetTick,
    ) -> WorldProcessMessageStageReport
    where
        TimerCallback: Copy,
        DbMiscContextOwner: DbMiscContext,
        JjcContext: JjcRunContext + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let started_at_ms = get_tick();
        let save_info_time_ms = self.setup.save_info_time_ms;
        let mut add_log_text = |message: &[u8]| {
            log.add_log_text(
                message,
                save_info_time_ms,
                &mut get_tick,
                &mut *get_log_local_time,
                &mut *put_log_info,
            )
        };
        let outcome = match self.process_message(
            honor_ranks,
            player_ranks,
            increment_log,
            auction_log,
            db_misc,
            db_misc_context,
            organizing,
            organizing_parameters,
            country_handler,
            country_parameters,
            country_war,
            four_nation_war,
            faction_war_sys,
            attack_city,
            attack_city_callbacks,
            village_war,
            goods_war,
            timer,
            village_war_callbacks,
            resources,
            load_player_largess,
            net_sessions,
            jjc,
            jjc_context,
            application_runtime,
            application_callbacks,
            write_faction_create_log,
            write_faction_title_log,
            write_faction_purview_log,
            write_faction_apply_log,
            write_faction_join_log,
            write_faction_quit_log,
            write_faction_fire_out_log,
            write_faction_master_log,
            write_faction_disband_log,
            rs_player,
            player_database,
            save_lifecycle,
            save_thread_handle,
            save_runtime,
            session_factory,
            general_variables,
            gods_battle,
            skills,
            rs_gods_battle,
            gods_battle_database,
            &mut add_log_text,
            update_player,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                return WorldProcessMessageStageReport::Blocked {
                    started_at_ms,
                    error,
                };
            }
        };
        drop(add_log_text);
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
        state.accumulated_time_ms = state.accumulated_time_ms.wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        WorldProcessMessageStageReport::Complete {
            started_at_ms,
            outcome,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: state.accumulated_time_ms,
            next_stage_started_at_ms,
        }
    }

 /// Выполняет следующий MainLoop owner после `ProcessMessage`.
 ///
 /// Входной shared tick уже назначен успешным
 /// `process_message_main_loop_stage`; первый новый tick закрывает
 /// `CSessionFactory::AI` в `DAT_0056e528`, второй начинает соседний
 /// готовый `ProcessPlayerDataQueue`.
    pub fn run_main_loop_session_factory_stage<GetTick>(
        &mut self,
        factory: &mut CSessionFactory,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> WorldMainLoopSessionFactoryStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let ai = factory.ai(self);
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.session_factory_time_ms = profile_state
            .session_factory_time_ms
            .wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        WorldMainLoopSessionFactoryStageReport {
            ai,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.session_factory_time_ms,
            next_stage_started_at_ms,
        }
    }

    pub(crate) fn update_detached_player_friends(
        &self,
        player: &mut CPlayer,
    ) -> Vec<WorldFriendPresenceUpdate> {
        let mut updates = Vec::with_capacity(player.friend_count());
        for friend_index in 0..player.friend_count() {
            let friend_name = player
                .friend_name(friend_index)
                .expect("friend index получен из текущего len")
                .to_vec();
            let mut friend_player_id = self.online_player_id_by_name(&friend_name);
            if friend_player_id == 0 {
                friend_player_id = self.login_player_id_by_name(&friend_name);
            }
            let online = friend_player_id != 0;
            let updated = player.set_friend_online(friend_index, online);
            debug_assert!(updated, "friend index не менялся между read и write");
            updates.push(self.send_friend_presence_update(
                friend_index,
                friend_player_id,
                online,
                player.get_name(),
            ));
        }
        updates
    }

    pub(crate) fn update_published_player_friends(
        &mut self,
        player_id: u32,
    ) -> Vec<WorldFriendPresenceUpdate> {
        let friend_count = self
            .map_player(player_id)
            .map_or(0, CPlayer::friend_count);
        let mut updates = Vec::with_capacity(friend_count);
        for friend_index in 0..friend_count {
            let (friend_name, player_name) = {
                let player = self
                    .map_player(player_id)
                    .expect("direct route уже опубликовал selected player");
                (
                    player
                        .friend_name(friend_index)
                        .expect("friend index получен из текущего len")
                        .to_vec(),
                    player.get_name().to_vec(),
                )
            };
            let mut friend_player_id = self.online_player_id_by_name(&friend_name);
            if friend_player_id == 0 {
                friend_player_id = self.login_player_id_by_name(&friend_name);
            }
            let online = friend_player_id != 0;
            let updated = self
                .players
                .get_mut(&player_id)
                .expect("selected player остаётся опубликованным")
                .set_friend_online(friend_index, online);
            debug_assert!(updated, "friend index не менялся между read и write");
            updates.push(self.send_friend_presence_update(
                friend_index,
                friend_player_id,
                online,
                &player_name,
            ));
        }
        updates
    }

    pub(crate) fn send_friend_presence_update(
        &self,
        friend_index: usize,
        friend_player_id: u32,
        online: bool,
        player_name: &[u8],
    ) -> WorldFriendPresenceUpdate {
        if !online {
            return WorldFriendPresenceUpdate {
                friend_index,
                player_id: 0,
                online: false,
                target_game_server_index: None,
                delivery: None,
            };
        }
        let target_game_server_index = self
            .online_player_by_id(friend_player_id)
            .and_then(|friend| self.get_region_game_server(friend.get_region_id()))
            .map(|game_server| game_server.index);
        let mut presence = CMessage::new(0x0007_F904);
        presence.base_mut().add_ulong(friend_player_id);
        add_legacy_c_string(presence.base_mut(), player_name);
        let sender = self.current_game_server_sender();
        let delivery = presence.send_to_map_id(
            sender.as_ref(),
            target_game_server_index.unwrap_or(0) as i32,
        );
        WorldFriendPresenceUpdate {
            friend_index,
            player_id: friend_player_id,
            online: true,
            target_game_server_index,
            delivery: Some(delivery),
        }
    }

    pub(crate) fn publish_loaded_player<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        player_id: u32,
        player: Box<CPlayer>,
        mut get_tick: GetTick,
    ) -> (WorldOnlinePlayerRemoveOutcome, bool, u32)
    where
        GetTick: FnMut() -> u32,
    {
        let online_removal = self.remove_online_player(organizing_ctrl, player_id);
        self.remove_offline_player(player_id);
        let login_time_ms = get_tick();
        self.append_login_player(player_id, login_time_ms);
        let replaced_existing_player = self.delete_map_player(player_id);
        let append_outcome = self.append_map_player(player, |_| {});
        let WorldMapPlayerAppendOutcome::Inserted {
            player_id: inserted_player_id,
        } = append_outcome
        else {
            unreachable!("map key удалён непосредственно перед AppendMapPlayer")
        };
        debug_assert_eq!(inserted_player_id, player_id);
        (
            online_removal,
            replaced_existing_player,
            login_time_ms,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "queue metadata сохраняет exact diagnostic outcome producer-а"
    )]

    pub fn route_loaded_player<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        route_order: WorldLoadedPlayerRouteOrder,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        mut get_tick: GetTick,
    ) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError>
    where
        GetTick: FnMut() -> u32,
    {
        let Some(mut player) = player else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::NullPlayer,
                login_delivery,
            });
        };

        let player_id = player.get_id() as u32;
        let region_types = self.player_organizing_region_types();
        let organizing_result = {
            let mut updater = organizing_ctrl.player_updater(&region_types);
            player.set_player_organizing(&mut updater)
        };
        if let Err(error) = organizing_result {
            return Err(WorldProcessPlayerDataQueueError {
                initial_size,
                null_pops,
                player_id,
                block: WorldProcessPlayerDataQueueBlock::Organizing(error),
            });
        }

        let region_id = player.get_region_id();
        let Some(game_server_index) = self
            .region(region_id)
            .map(|region| region.game_server_index)
        else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            drop(player);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::MissingRegion { region_id },
                login_delivery,
            });
        };

 // CPlayer vtable +0x84 = `CShape::SetState(0)`.
        player.set_state(0);
        let Some(game_server) = self
            .game_server(game_server_index)
            .filter(|game_server| game_server.connected)
        else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            drop(player);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::MissingOrDisconnectedGameServer {
                    region_id,
                    game_server_index,
                },
                login_delivery,
            });
        };

        let game_server_ip = game_server.ip.clone();
        let game_server_port = game_server.port.ok_or(WorldProcessPlayerDataQueueError {
            initial_size,
            null_pops,
            player_id,
            block: WorldProcessPlayerDataQueueBlock::UninitializedGameServerPort {
                game_server_index,
            },
        })?;

        let mut login_reply = CMessage::new(0x0001_FF01);
        login_reply.base_mut().add_byte(0x1D);
        add_legacy_c_string(login_reply.base_mut(), cdkey);
        add_legacy_c_string(login_reply.base_mut(), &game_server_ip);
        login_reply.base_mut().add_ulong(game_server_port);
        add_legacy_c_string(login_reply.base_mut(), player.get_name());
        login_reply.base_mut().add_byte(player.get_level());
        let login_delivery = login_reply.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );
        after_login_send(&mut player);

        let (friend_updates, online_removal, replaced_existing_player, login_time_ms) =
            match route_order {
                WorldLoadedPlayerRouteOrder::LoadedQueue => {
                    let friend_updates = self.update_detached_player_friends(&mut player);
                    let (online_removal, replaced_existing_player, login_time_ms) = self
                        .publish_loaded_player(
                            organizing_ctrl,
                            player_id,
                            player,
                            &mut get_tick,
                        );
                    (
                        friend_updates,
                        online_removal,
                        replaced_existing_player,
                        login_time_ms,
                    )
                }
                WorldLoadedPlayerRouteOrder::Direct => {
                    let (online_removal, replaced_existing_player, login_time_ms) = self
                        .publish_loaded_player(
                            organizing_ctrl,
                            player_id,
                            player,
                            &mut get_tick,
                        );
                    let friend_updates = self.update_published_player_friends(player_id);
                    self.players
                        .get_mut(&player_id)
                        .expect("direct route сохраняет опубликованного player-owner-а")
                        .reset_selected_login_flags();
                    (
                        friend_updates,
                        online_removal,
                        replaced_existing_player,
                        login_time_ms,
                    )
                }
            };

        Ok(WorldProcessPlayerDataQueueOutcome::Accepted {
            initial_size,
            null_pops,
            player_id,
            client_ip,
            game_server_index,
            login_delivery,
            friend_updates,
            online_removal,
            replaced_existing_player,
            login_time_ms,
        })
    }

 /// Выполняет `CTimer::Run`, закрывает `DAT_0056e520` и начинает faction-war стадию.
 ///
 /// `profile_state.ai_calls` является текущим `CGame::s_lAITick`. Timer
 /// получает тот же tick-provider, которым затем MainLoop закрывает стадию
 /// и отдельно назначает shared start для готового `CFactionWarSys::Run`.
    #[allow(
        clippy::too_many_arguments,
        reason = "timer callback сохраняет явные DB, ranking, clock и log owners"
    )]

    pub async fn run_main_loop_timer_stage<Callback, GetTick, GetTimerLocalTime, RsPlayer>(
        &self,
        timer: &mut CTimer<Callback>,
        time_to_return: &mut TimeToReturn,
        time_to_return_callbacks: TimeToReturnCallbacks<Callback>,
        attack_city: &mut CAttackCitySys,
        attack_city_callbacks: AttackCityCallbacks<Callback>,
        village_war: &mut CVillageWarSys,
        village_war_callbacks: VillageWarCallbacks<Callback>,
        country_war: &mut CountryWarSys,
        country_handler: &mut CCountryHandler,
        country_war_callbacks: CountryWarCallbacks<Callback>,
        four_nation_war: &mut CFourNationWarSys,
        four_nation_war_callbacks: FourNationWarCallbacks<Callback>,
        globe_setup: &GlobeSetupSnapshot,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        copy_number_timer: &mut CopyNumberTimerState,
        organizing_parameters: &mut COrganizingParam,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut RsPlayer,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_timer_local_time: &mut GetTimerLocalTime,
        get_log_local_time: &mut dyn FnMut() -> WorldLogLocalTime,
        put_log_info: &mut dyn FnMut(&[u8]),
    ) -> Result<WorldMainLoopTimerStageReport, WorldMainLoopTimerStageBlock>
    where
        Callback: Copy + PartialEq,
        GetTick: FnMut() -> u32 + ?Sized,
        GetTimerLocalTime: FnMut() -> TagTime + ?Sized,
        RsPlayer: RsPlayerOwner<CPlayer>,
    {
        let mut handler = WorldTimerHandler {
            game: self,
            attack_city,
            attack_city_callbacks,
            village_war,
            village_war_callbacks,
            country_war,
            country_handler,
            country_war_callbacks,
            four_nation_war,
            four_nation_war_callbacks,
            time_to_return,
            time_to_return_callbacks,
            globe_setup,
            organizing_parameters,
            player_ranks,
            rs_player,
            player_database,
            organizing,
            copy_number_timer,
            log,
            get_log_local_time,
            put_log_info,
            copy_number_resets: Vec::new(),
            refreshes: Vec::new(),
            tax_refreshes: Vec::new(),
            time_to_returns: Vec::new(),
            attack_city_wars: Vec::new(),
            village_wars: Vec::new(),
            country_wars: Vec::new(),
            four_nation_wars: Vec::new(),
            pending_copy_number_registration: None,
            pending_player_ranks_registration: None,
            pending_tax_registration: None,
        };
        let timer_report = timer
            .run_with_async_handler(
                profile_state.ai_calls,
                &mut *get_tick,
                &mut *get_timer_local_time,
                &mut handler,
                |_, _| unreachable!("WorldServer timer-owner обработал каждый callback"),
            )
            .await;
        let timer_report = match timer_report {
            Ok(timer_report) => timer_report,
            Err(AsyncTimerRunBlock { timer, source }) => {
                return Err(WorldMainLoopTimerStageBlock { timer, source });
            }
        };
        let copy_number_resets = handler.copy_number_resets;
        let player_ranks = handler.refreshes;
        let organizing_taxes = handler.tax_refreshes;
        let time_to_returns = handler.time_to_returns;
        let attack_city_wars = handler.attack_city_wars;
        let village_wars = handler.village_wars;
        let country_wars = handler.country_wars;
        let four_nation_wars = handler.four_nation_wars;
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.timer_time_ms = profile_state.timer_time_ms.wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        Ok(WorldMainLoopTimerStageReport {
            timer: timer_report,
            copy_number_resets,
            player_ranks,
            organizing_taxes,
            time_to_returns,
            attack_city_wars,
            village_wars,
            country_wars,
            four_nation_wars,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.timer_time_ms,
            next_stage_started_at_ms,
        })
    }

 /// Выполняет `CFactionWarSys::Run` и закрывает `DAT_0056e51c`.
 ///
 /// оригинал MainLoop не назначает новый shared start перед следующим
 /// `CLeiTing::Run`, поэтому этот call-site делает только один end tick и
 /// оставляет `clocks.stage_started_at_ms` без изменения.
    pub fn run_main_loop_faction_war_stage<GetTick>(
        &mut self,
        faction_war_sys: &mut CFactionWarSys,
        organizing: &mut COrganizingCtrl,
        clocks: &WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> Result<
        WorldMainLoopFactionWarStageReport,
        FactionWarStopBlock<WorldMainLoopFactionWarBlock>,
    >
    where
        GetTick: FnMut() -> u32,
    {
        let (faction_war, players_to_update) = {
            let mut context = WorldMainLoopFactionWarEffects {
                game: self,
                organizing,
                players_to_update: Vec::new(),
            };
            let report = faction_war_sys.run(&mut context, &mut get_tick)?;
            (report, context.players_to_update)
        };
        for player_id in players_to_update {
            let _ = self.update_player_faction_info(organizing, player_id);
        }
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.faction_war_time_ms =
            profile_state.faction_war_time_ms.wrapping_add(elapsed_ms);
        Ok(WorldMainLoopFactionWarStageReport {
            faction_war,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.faction_war_time_ms,
        })
    }

    pub fn run_main_loop_lei_ting_stage<Context, GetLocalTime>(
        &mut self,
        lei_ting: &mut CLeiTing,
        globe_setup: &GlobeSetupSnapshot,
        context: &mut Context,
        reset_worker: &WorldLeiTingResetWorker,
        runtime: tokio::runtime::Handle,
        mut get_local_time: GetLocalTime,
    ) -> Result<LeiTingRunReport, LeiTingBlock<Context::Block, PlayerCodecError>>
    where
        Context: WorldLeiTingRuntimeContext,
        GetLocalTime: FnMut() -> LeiTingLocalTime,
    {
        while let Some(event) = reset_worker.try_next_event() {
            context.on_database_reset_worker_event(event);
        }
        let current = get_local_time();
        let result = {
            let mut worker_context = WorldLeiTingWorkerContext::new(context, reset_worker, runtime);
            lei_ting.run(current, self, globe_setup, &mut worker_context)
        };
        while let Some(event) = reset_worker.try_next_event() {
            context.on_database_reset_worker_event(event);
        }
        result
    }

 /// Выполняет соседний `DoneOutList -> Pop/DoneListIn -> LoadAuction` batch.
 ///
 /// Между `CLeiTing::Run` и этими тремя owners исходный MainLoop не снимал
 /// tick. Единственный clock-call после `LoadAuction` назначает shared start
 /// следующей готовой стадии `CNetSessionManager::Run`.
    pub fn run_main_loop_db_misc_stage<Context, Delivery, GetTick>(
        db_misc: &mut CDbMisc,
        context: &mut Context,
        delivery: &mut Delivery,
        clocks: &mut WorldMainLoopClockState,
        mut get_tick: GetTick,
    ) -> Result<WorldMainLoopDbMiscStageReport, DbMiscDoneOutBlock>
    where
        Context: DbMiscContext,
        Delivery: DbMiscDeliveryContext,
        GetTick: FnMut() -> u32,
    {
        let output = db_misc.done_out_list(delivery)?;
        let input_notes = db_misc.pop_item_from_list_in(context, 0);
        let input = db_misc.done_list_in(context, input_notes);
        let auction = db_misc.load_auction(context);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        Ok(WorldMainLoopDbMiscStageReport {
            output,
            input,
            auction,
            next_stage_started_at_ms,
        })
    }

 /// Выполняет полный session timeout pass, применяет его terminal actions и
 /// закрывает `DAT_0056e518`.
 ///
 /// Следующий участок MainLoop начинает проверку ping без нового shared
 /// start tick, поэтому `clocks.stage_started_at_ms` здесь не меняется.
    pub fn run_main_loop_net_session_stage<GetTick>(
        &self,
        manager: &CNetSessionManager,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
        clocks: &WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> WorldMainLoopNetSessionStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let sessions = manager.run();
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
        let mut effects =
            WorldUnionApplicationEffects::new(self, manager, application_runtime, callbacks);
        let union_applications = drain_union_application_runtime(
            self,
            organizing,
            organizing_parameters,
            application_runtime,
            &mut effects,
            update_player,
        );
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.net_session_time_ms =
            profile_state.net_session_time_ms.wrapping_add(elapsed_ms);
        WorldMainLoopNetSessionStageReport {
            sessions,
            union_applications,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.net_session_time_ms,
        }
    }

 /// Завершает либо продолжает один GameServer ping без нового clock-call.
 ///
 /// При публикации `m_bInPing` очищается до сборки и неприоритетной отправки
 /// `0x1FE04`; порядок ответов совпадает с порядком исходного vector.
    pub fn run_main_loop_ping_stage(
        &mut self,
        clocks: &WorldMainLoopClockState,
    ) -> Result<WorldMainLoopPingStageReport, WorldMainLoopPingError> {
        if !self.ping_in_progress {
            return Ok(WorldMainLoopPingStageReport::Idle);
        }

        let received_responses = u32::try_from(self.ping_game_servers.len()).map_err(|_| {
            WorldMainLoopPingError::ResponseCountOutsideLegacyRange {
                count: self.ping_game_servers.len(),
            }
        })?;
        let connected_game_servers = self.connected_game_server_count();
        let elapsed_ms = clocks
            .current_tick_ms
            .wrapping_sub(self.last_ping_game_server_time_ms);
        let all_connected_responded = connected_game_servers as u32 <= received_responses;
        let timed_out = 5_000 < elapsed_ms;
        if !all_connected_responded && !timed_out {
            return Ok(WorldMainLoopPingStageReport::Waiting {
                connected_game_servers,
                received_responses,
                elapsed_ms,
            });
        }

        let online_players = u32::try_from(self.online_players.len()).map_err(|_| {
            WorldMainLoopPingError::OnlinePlayerCountOutsideLegacyRange {
                count: self.online_players.len(),
            }
        })?;
        self.ping_in_progress = false;

        let declared_responses = received_responses as i32;
        let mut snapshot = CMessage::new(0x0001_FE04);
        snapshot.base_mut().add_ulong(online_players);
        snapshot.base_mut().add_long(declared_responses);
        if 0 < declared_responses {
            for response in &self.ping_game_servers {
                add_legacy_c_string(snapshot.base_mut(), &response.ip);
                snapshot.base_mut().add_long(response.map_id);
                snapshot.base_mut().add_long(response.player_count);
            }
        }
        let delivery = snapshot.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );

        Ok(WorldMainLoopPingStageReport::Published {
            connected_game_servers,
            received_responses,
            elapsed_ms,
            all_connected_responded,
            timed_out,
            online_players,
            delivery,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "два ещё отдельных downstream owner-а и clock передаются явно"
    )]

    pub fn run_main_loop_minute_stage<GetTick>(
        &mut self,
        initialization: &mut WorldMainLoopInitializationState,
        clocks: &mut WorldMainLoopTailClockState,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &CCountryParam,
        organizing_parameters: &COrganizingParam,
        attack_city: &CAttackCitySys,
        village_war: &CVillageWarSys,
        goods_war: &mut CGoodsWarMember,
        globe_setup: &GlobeSetupSnapshot,
        mut get_tick: GetTick,
        refresh_owned_city: &mut dyn FnMut(&CGame, i32, i32, i32, Option<u8>),
        update_player: &mut dyn FnMut(i32),
        faction_master_log_enabled: bool,
        write_faction_master_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
        faction_disband_log_enabled: bool,
        write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    ) -> Result<WorldMainLoopMinuteStageReport, WorldMainLoopMinuteStageBlock>
    where
        GetTick: FnMut() -> u32,
    {
        let initialized = initialize_main_loop_tail_clocks(initialization, clocks, &mut get_tick);
        let current_tick_ms = get_tick();
        clocks.current_tick_ms = current_tick_ms;
        let minute_delta = current_tick_ms
            .wrapping_sub(clocks.minute_started_at_ms)
            .wrapping_div(60_000) as i32;

        let faction_disband_log_enabled =
            self.setup.use_log_system && faction_disband_log_enabled;
        let organizing_report = organizing
            .run(minute_delta, |organizing, player_id, faction_id| {
                let outcome = {
                    let mut effects = WorldOrganizingDisbandEffects {
                        game: &*self,
                        village_war,
                        attack_city,
                        country_handler: &*country_handler,
                        goods_war: &mut *goods_war,
                    };
                    organizing.disband_faction(
                        &*self,
                        player_id,
                        faction_id,
                        &mut effects,
                    )?
                };
                let OrganizingDisbandOutcome::Disbanded {
                    mut progress,
                    retired_faction,
                } = outcome
                else {
                    return Ok(false);
                };
                progress.player = self.clear_disbanded_player_faction_data(player_id);
                if faction_disband_log_enabled
                    && let Some(player) = progress.player.as_ref()
                {
                    write_faction_disband_log(
                        faction_id,
                        legacy_c_string_prefix(retired_faction.name()),
                        player.player_id,
                        legacy_c_string_prefix(&player.player_name),
                    );
                    progress.log_written = true;
                }
                drop(retired_faction);
                Ok(true)
            })
            .map_err(WorldMainLoopMinuteStageBlock::Organizing)?;
        let faction_master_log_enabled = self.setup.use_log_system && faction_master_log_enabled;
        let base = WorldCountryExileResultEffects {
            game: self,
            globe_setup,
        };
        let mut effects = WorldCountryDemiseEffects {
            base,
            organizing,
            organizing_parameters,
            attack_city,
            goods_war,
            refresh_owned_city,
            update_player,
            faction_master_log_enabled,
            write_faction_master_log,
        };
        let country = country_handler
            .run(
                minute_delta,
                &mut get_tick,
                country_parameters,
                &mut effects,
            )
            .map_err(WorldMainLoopMinuteStageBlock::Country)?;
        clocks.minute_started_at_ms = current_tick_ms;

        Ok(WorldMainLoopMinuteStageReport {
            initialization: initialized,
            current_tick_ms,
            minute_delta,
            organizing: organizing_report,
            country,
        })
    }

    pub fn run_main_loop_bai_tan_jjc_stage<Context: WorldJjcRuntimeContext>(
        &mut self,
        jjc_system: &mut CJJcSystem,
        jjc_config: JjcRunConfig,
        context: &mut Context,
        week_clear_worker: &WorldJjcWeekClearWorker,
        runtime: tokio::runtime::Handle,
    ) -> Result<WorldMainLoopBaiTanJjcStageReport, JjcRunBlock> {
        while let Some(event) = week_clear_worker.try_next_event() {
            context.on_week_clear_worker_event(event);
        }
        let bai_tan = self.done_bai_tan_list();
        let jjc = {
            let mut worker_context = WorldJjcWorkerContext::new(context, week_clear_worker, runtime);
            jjc_system.run(self, jjc_config, &mut worker_context)?
        };
        while let Some(event) = week_clear_worker.try_next_event() {
            context.on_week_clear_worker_event(event);
        }
        Ok(WorldMainLoopBaiTanJjcStageReport { bai_tan, jjc })
    }

 /// Обходит весь login-list по одному общему tick snapshot и освобождает
 /// только просроченные записи, у которых ещё существует player-owner.
    pub fn process_time_out_login_player<GetTick>(
        &mut self,
        release_interval_ms: u32,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        session_factory: &mut CSessionFactory,
    ) -> WorldLoginTimeoutReport
    where
        GetTick: FnMut() -> u32,
    {
        let snapshot_tick_ms = get_tick();
        let mut entries = Vec::with_capacity(self.login_players.len());
        let mut login_index = 0;

        while login_index < self.login_players.len() {
            let login = self.login_players[login_index];
            let elapsed_ms = snapshot_tick_ms.wrapping_sub(login.login_time_ms);
            if release_interval_ms >= elapsed_ms {
                entries.push(WorldLoginTimeoutEntryOutcome::Waiting {
                    player_id: login.player_id,
                    elapsed_ms,
                });
                login_index += 1;
                continue;
            }

            let Some(player) = self.map_player(login.player_id) else {
 // оставляет такой просроченный list-node на месте.
                entries.push(WorldLoginTimeoutEntryOutcome::ExpiredMissingPlayer {
                    player_id: login.player_id,
                    elapsed_ms,
                });
                login_index += 1;
                continue;
            };

            let account = player.get_account().to_vec();
            let player_name = player.get_name().to_vec();
            let player_level = player.get_level();
            let owner_type = player.get_type();
            let owner_id = player.get_id();
            let team_id = player.get_team_id();
            let friend_names = (0..player.friend_count())
                .map(|friend_index| {
                    player
                        .friend_name(friend_index)
                        .expect("friend index получен из текущего len")
                        .to_vec()
                })
                .collect::<Vec<_>>();

            let mut release = CMessage::new(0x0001_FF06);
            add_legacy_c_string(release.base_mut(), &account);
            add_legacy_c_string(release.base_mut(), &player_name);
            release.base_mut().add_byte(player_level);
            let login_delivery = release.send(
                self.current_login_client().map(CMyNetClient::send_queue),
                false,
            );

            let team_session_id = self.get_team_session_id(team_id as u32);
            let team_exit = self.exit_team_player(
                session_factory,
                team_session_id,
                owner_type,
                owner_id,
            );

            let removed = self.login_players.remove(login_index);
            debug_assert_eq!(removed.map(|entry| entry.player_id), Some(login.player_id));

            let online_removal = self.remove_online_player(organizing, login.player_id);
            let offline_inserted = self.append_offline_player_id(login.player_id);

            let mut friend_outcomes = Vec::with_capacity(friend_names.len());
            for (friend_index, friend_name) in friend_names.into_iter().enumerate() {
                let friend_player_id = self.online_player_id_by_name(&friend_name);
                if friend_player_id == 0 {
                    friend_outcomes.push(WorldLoginTimeoutFriendOutcome::Offline { friend_index });
                    continue;
                }

                let target_game_server_index = self
                    .online_player_by_id(friend_player_id)
                    .and_then(|friend| self.get_region_game_server(friend.get_region_id()))
                    .map(|game_server| game_server.index);
                let mut presence = CMessage::new(0x0007_F905);
                presence.base_mut().add_ulong(friend_player_id);
                add_legacy_c_string(presence.base_mut(), &player_name);
                let sender = self.current_game_server_sender();
                let delivery = presence.send_to_map_id(
                    sender.as_ref(),
                    target_game_server_index.unwrap_or(0) as i32,
                );
                friend_outcomes.push(WorldLoginTimeoutFriendOutcome::Notified {
                    friend_index,
                    friend_player_id,
                    target_game_server_index,
                    delivery,
                });
            }

            entries.push(WorldLoginTimeoutEntryOutcome::Released {
                player_id: login.player_id,
                elapsed_ms,
                login_delivery,
                team_id,
                team_session_id,
                team_exit,
                online_removal,
                offline_inserted,
                friend_outcomes,
            });
 // После erase сохранённый next node занимает тот же VecDeque index.
        }

        WorldLoginTimeoutReport {
            snapshot_tick_ms,
            entries,
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "clock, wait/debug adapters и session factory являются разными границами"
    )]

    pub fn run_main_loop_tail_stage<GetTick, Wait, DebugOutput>(
        &mut self,
        clocks: &mut WorldMainLoopTailClockState,
        release_state: &mut WorldMainLoopLoginReleaseState,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        mut wait: Wait,
        mut output_debug: DebugOutput,
        session_factory: &mut CSessionFactory,
    ) -> WorldMainLoopTailStageReport
    where
        GetTick: FnMut() -> u32,
        Wait: FnMut(u32),
        DebugOutput: FnMut(&'static str),
    {
        let sampled_tick_ms = get_tick();
        clocks.current_tick_ms = sampled_tick_ms;
        let wait_duration_ms = if sampled_tick_ms.wrapping_sub(clocks.pacing_deadline_ms) < 40 {
            let duration = clocks
                .pacing_deadline_ms
                .wrapping_sub(sampled_tick_ms)
                .wrapping_add(40);
            wait(duration);
            Some(duration)
        } else {
            None
        };

        clocks.pacing_deadline_ms = clocks.pacing_deadline_ms.wrapping_add(40);
        let signed_lag_ms = sampled_tick_ms.wrapping_sub(clocks.pacing_deadline_ms) as i32;
        let warning_resync_tick_ms = if 1_000 < signed_lag_ms {
            output_debug("warning!!! 1 second not call AI()\n");
            let resync_tick_ms = get_tick();
            clocks.pacing_deadline_ms = resync_tick_ms;
            Some(resync_tick_ms)
        } else {
            None
        };

        let release_gate_tick_ms = get_tick();
        clocks.current_tick_ms = release_gate_tick_ms;
        let release_gate_elapsed_ms =
            release_gate_tick_ms.wrapping_sub(release_state.last_checked_at_ms);
        let pacing = WorldMainLoopPacingReport {
            sampled_tick_ms,
            wait_duration_ms,
            next_deadline_ms: clocks.pacing_deadline_ms,
            signed_lag_ms,
            warning_resync_tick_ms,
            release_gate_tick_ms,
            release_gate_elapsed_ms,
        };

        let Some(release_interval_ms) = self.setup.release_login_player_time_ms else {
 // Конструктор не задавал это поле, а safe Rust
 // не выбирает значение для исходного чтения неинициализированного DWORD.
            return WorldMainLoopTailStageReport::BlockedMissingReleaseInterval { pacing };
        };

        let login_timeout = if release_interval_ms < release_gate_elapsed_ms {
            release_state.last_checked_at_ms = release_gate_tick_ms;
            Some(self.process_time_out_login_player(
                release_interval_ms,
                organizing,
                &mut get_tick,
                session_factory,
            ))
        } else {
            None
        };

        WorldMainLoopTailStageReport::Complete {
            pacing,
            release_interval_ms,
            login_timeout,
        }
    }

 /// Выполняет `CGame::MainLoop` в исходном порядке владельцев.
 ///
 /// Ошибка безопасной границы прекращает только оставшийся хвост прохода;
 /// уже выполненные мутации, отправки, clock reads и callbacks не откатываются.
    #[allow(
        clippy::too_many_arguments,
        reason = "явные state/domain/platform owners сохраняют исходные границы процесса"
    )]

    pub async fn main_loop<
        TimerCallback,
        LeiTingContextOwner,
        DbMiscContextOwner,
        JjcContext,
    >(
        &mut self,
        configuration: WorldMainLoopConfiguration,
        state: &mut WorldMainLoopStateOwners<'_>,
 // Generic-связка Realm `app/world_main_loop_data` закрепляется конкретными
 // DB/game владельцами на этой границе: глубокие точки process_message/
 // process_world_message/route_loaded_player держат конкретную декларацию
 // на самой границе процесса.
        owners: &mut WorldMainLoopOwners<
            '_,
            TimerCallback,
            LeiTingContextOwner,
            DbMiscContextOwner,
            JjcContext,
            TiberiusRsPlayer,
        >,
        callbacks: &mut WorldMainLoopCallbacks<'_, CGame, Arc<TiberiusLargess>>,
    ) -> WorldMainLoopResult<LeiTingContextOwner::Block>
    where
        TimerCallback: Copy + PartialEq,
        LeiTingContextOwner: WorldLeiTingRuntimeContext,
        DbMiscContextOwner: DbMiscContext,
        JjcContext: WorldJjcRuntimeContext,
    {
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_create_log =
            move |faction_id: i32, faction_name: &[u8], player_id: i32, player_name: &[u8]| {
                let _ = queue.push(WorldWriteLogCommand::FactionLog(
                    WorldFactionLogWrite::Faction {
                        faction_id,
                        faction_name: faction_name.to_vec(),
                        player_id,
                        player_name: player_name.to_vec(),
                        log_type: 0,
                    },
                ));
            };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_title_log = move |
            member_id: i32,
            member_name: &[u8],
            old_title: &[u8],
            new_title: &[u8],
            manager_id: i32,
            manager_name: &[u8],
            faction_id: i32,
            faction_name: &[u8],
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Title {
                    member_id,
                    member_name: member_name.to_vec(),
                    old_title: old_title.to_vec(),
                    new_title: new_title.to_vec(),
                    manager_id,
                    manager_name: manager_name.to_vec(),
                    faction_id,
                    faction_name: faction_name.to_vec(),
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_purview_log = move |
            member_id: i32,
            member_name: &[u8],
            purview: i32,
            manager_id: i32,
            manager_name: &[u8],
            faction_id: i32,
            faction_name: &[u8],
            log_type: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Purview {
                    member_id,
                    member_name: member_name.to_vec(),
                    purview,
                    manager_id,
                    manager_name: manager_name.to_vec(),
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    log_type,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_level_log = move |
            faction_id: i32,
            faction_name: &[u8],
            level: i32,
            master_id: i32,
            master_name: &[u8],
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Level {
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    level,
                    master_id,
                    master_name: master_name.to_vec(),
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_experience_log = move |
            faction_id: i32,
            faction_name: &[u8],
            member_id: i32,
            member_name: &[u8],
            before_experience: i32,
            experience: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Experience {
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    member_id,
                    member_name: member_name.to_vec(),
                    before_experience,
                    experience,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_apply_log = move |
            faction_id: i32,
            faction_name: &[u8],
            player_id: i32,
            player_name: &[u8],
            log_type: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Faction {
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    player_id,
                    player_name: player_name.to_vec(),
                    log_type,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_join_log = move |
            member_id: i32,
            member_name: &[u8],
            manager_id: i32,
            manager_name: &[u8],
            faction_id: i32,
            faction_name: &[u8],
            log_type: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Member {
                    member_id,
                    member_name: member_name.to_vec(),
                    manager_id,
                    manager_name: manager_name.to_vec(),
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    log_type,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_quit_log = move |
            faction_id: i32,
            faction_name: &[u8],
            player_id: i32,
            player_name: &[u8],
            log_type: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Faction {
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    player_id,
                    player_name: player_name.to_vec(),
                    log_type,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_fire_out_log = move |
            member_id: i32,
            member_name: &[u8],
            manager_id: i32,
            manager_name: &[u8],
            faction_id: i32,
            faction_name: &[u8],
            log_type: i32,
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Member {
                    member_id,
                    member_name: member_name.to_vec(),
                    manager_id,
                    manager_name: manager_name.to_vec(),
                    faction_id,
                    faction_name: faction_name.to_vec(),
                    log_type,
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_master_log = move |
            old_master_id: i32,
            old_master_name: &[u8],
            new_master_id: i32,
            new_master_name: &[u8],
            faction_id: i32,
            faction_name: &[u8],
        | {
            let _ = queue.push(WorldWriteLogCommand::FactionLog(
                WorldFactionLogWrite::Master {
                    old_master_id,
                    old_master_name: old_master_name.to_vec(),
                    new_master_id,
                    new_master_name: new_master_name.to_vec(),
                    faction_id,
                    faction_name: faction_name.to_vec(),
                },
            ));
        };
        let queue = callbacks.write_log_queue.clone();
        let mut write_faction_disband_log =
            move |faction_id: i32, faction_name: &[u8], player_id: i32, player_name: &[u8]| {
                let _ = queue.push(WorldWriteLogCommand::FactionLog(
                    WorldFactionLogWrite::Faction {
                        faction_id,
                        faction_name: faction_name.to_vec(),
                        player_id,
                        player_name: player_name.to_vec(),
                        log_type: 1,
                    },
                ));
            };

        let profile_initialization = initialize_main_loop_profile_if_needed(
            state.initialization,
            state.profile,
            &mut *callbacks.get_tick,
        );
        let save_initialization = initialize_main_loop_save_if_needed(
            state.initialization,
            state.save_trigger,
            &mut *callbacks.get_tick,
        );
        let refresh_initialization =
            initialize_main_loop_refresh_if_needed(state.initialization, state.clocks);
        let current_tick_ms = update_main_loop_current_tick(state.clocks, &mut *callbacks.get_tick);

        let largess = self.evaluate_main_loop_largess_gate(state.clocks, state.largess);
        match largess {
            WorldMainLoopLargessGateReport::BlockedMissingFact { .. } => {
                return Err(Box::new(WorldMainLoopBlock::Largess(largess)));
            }
            WorldMainLoopLargessGateReport::StartWorkerRequested { .. } => {
                let world_number = self
                    .setup
                    .world_number
                    .expect("Largess gate проверил dwNumber до запроса worker-а");
                let _ = callbacks.largess.start_worker(world_number);
            }
            WorldMainLoopLargessGateReport::Disabled { .. }
            | WorldMainLoopLargessGateReport::Waiting { .. } => {}
        }

        let refresh_profile_started_at_ms =
            start_main_loop_profile_stage(state.clocks, &mut *callbacks.get_tick);
        let save_lifecycle_snapshot = *state.save_lifecycle.lock();
        let refresh = self.run_main_loop_refresh_stage(
            state.clocks,
            state.profile,
            state.process_message,
            state.refresh_high_water,
            configuration.refresh_external_counts,
            &save_lifecycle_snapshot,
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_save_point_time,
            &mut callbacks.get_log_local_time,
            &mut callbacks.put_log_info,
        );
        let refresh = match refresh {
            complete @ WorldMainLoopRefreshStageReport::Complete { .. } => complete,
            blocked @ WorldMainLoopRefreshStageReport::BlockedMissingFact { .. } => {
                return Err(Box::new(WorldMainLoopBlock::Refresh(blocked)));
            }
        };

        let reload_input_resources = owners.resources.main_loop_resource_snapshot();
        let mut reload_union_application_callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *callbacks.random,
            refresh_owned_city: &mut *callbacks.refresh_union_owned_city,
            faction_level_log_enabled: reload_input_resources.log_system.faction_level_enabled(),
            write_faction_level_log: &mut write_faction_level_log,
            faction_experience_log_enabled: reload_input_resources
                .log_system
                .faction_experience_enabled(),
            write_faction_experience_log: &mut write_faction_experience_log,
        };
        let reload = reload_profiles(
            self,
            state.reload_flags,
            &mut *owners.resources,
            owners.jjc,
            owners.gods_battle,
            owners.skills,
            owners.rs_gods_battle.as_deref_mut(),
            &mut *callbacks.get_log_local_time,
            owners.country_war,
            owners.timer,
            owners.country_war_callbacks,
            owners.four_nation_war,
            owners.four_nation_war_callbacks,
            owners.time_to_return,
            owners.time_to_return_callbacks,
            owners.village_war,
            owners.village_war_callbacks,
            owners.attack_city,
            owners.attack_city_callbacks,
            owners.organizing,
            owners.faction_war,
            owners.country,
            owners.country_parameters,
            owners.organizing_parameters,
            owners.organizing_tax_callback,
            &reload_input_resources.globe_setup,
            &mut *callbacks.get_tick,
            &mut reload_union_application_callbacks,
            &mut *callbacks.update_union_player,
            &mut *callbacks.get_timer_local_time,
        )
        .await;
        let reload = match reload {
            complete @ WorldReloadProfilesReport::Complete { .. } => complete,
            blocked @ (WorldReloadProfilesReport::BlockedMissingFact { .. }
            | WorldReloadProfilesReport::BlockedRegionSetup { .. }
            | WorldReloadProfilesReport::BlockedReloadOwner { .. }) => {
                return Err(Box::new(WorldMainLoopBlock::Reload(blocked)));
            }
        };
        let resources = owners.resources.main_loop_resource_snapshot();
        let country_limits = CountryKingSaveLimits {
            control_point: owners
                .country_parameters
                .max_king_control_point()
                .ok_or_else(|| {
                    Box::new(WorldMainLoopBlock::MissingCountryLimit {
                        parameter: "_max_king_control_point",
                    })
                })?,
            material_point: owners
                .country_parameters
                .max_king_material_point()
                .ok_or_else(|| {
                    Box::new(WorldMainLoopBlock::MissingCountryLimit {
                        parameter: "_max_king_material_point",
                    })
                })?,
            war_point: owners
                .country_parameters
                .max_king_war_point()
                .ok_or_else(|| {
                    Box::new(WorldMainLoopBlock::MissingCountryLimit {
                        parameter: "_max_king_war_point",
                    })
                })?,
        };

        let maintenance = self.run_main_loop_maintenance_stage(
            state.player_ranks_request,
            resources.globe_setup.use_appellation_function(),
            owners.player_ranks,
            owners.rs_player,
            owners.player_database.as_deref_mut(),
            owners.organizing,
            owners.honor_ranks,
            owners.auction_log,
            owners.auction_log_database.as_deref_mut(),
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_log_local_time,
            &mut callbacks.get_auction_month_day,
            &mut callbacks.put_log_info,
        )
        .await;
        let maintenance = match maintenance {
            Ok(maintenance) => maintenance,
            Err(block) => return Err(Box::new(WorldMainLoopBlock::Maintenance(block))),
        };
        let collect_player_data =
            self.materialize_collect_player_data_request(state.collect_player_data);

        let save = self.materialize_run_save_pre_gate(
            state.save_trigger,
            current_tick_ms,
            &mut callbacks.get_save_point_time,
            &mut *callbacks.save_runtime,
            &resources.registry,
            owners.organizing,
            &resources.coefficients,
            owners.faction_war,
            owners.country,
            country_limits,
            owners
                .general_variables
                .as_deref()
                .expect("World MainLoop запускается после загрузки general variables"),
            owners.honor_ranks,
            owners.gods_battle,
            Arc::clone(state.save_lifecycle),
            state.save_thread_handle,
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_log_local_time,
            &mut callbacks.put_log_info,
        );
        let save = match save {
            WorldRunSavePreGateReport::IntervalNotElapsed {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::IntervalNotElapsed,
            },
            WorldRunSavePreGateReport::SaveLockBusy {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                adjusted_last_save_point_time_ms,
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::SaveLockBusy {
                    adjusted_last_save_point_time_ms,
                },
            },
            WorldRunSavePreGateReport::AfterLock {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                trigger: WorldRunSaveTriggerReport::Complete(trigger),
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::Triggered(trigger),
            },
            WorldRunSavePreGateReport::AfterLock {
                trigger: WorldRunSaveTriggerReport::BlockedSaveAllOrganizations { guard, block },
                ..
            } => {
                guard.stop_outer_owner();
                return Err(Box::new(WorldMainLoopBlock::SaveAllOrganizations { block }));
            }
            WorldRunSavePreGateReport::AfterLock {
                trigger: WorldRunSaveTriggerReport::BlockedImmediateSave { guard, log, block },
                ..
            } => {
                guard.stop_outer_owner();
                return Err(Box::new(WorldMainLoopBlock::ImmediateSave { log, block }));
            }
        };
        state.clocks.stage_started_at_ms = save.profile_started_at_ms;

        let ai = self.run_main_loop_ai_stage(
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
            &mut *callbacks.random,
        );
        let mut union_application_callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *callbacks.random,
            refresh_owned_city: &mut *callbacks.refresh_union_owned_city,
            faction_level_log_enabled: resources.log_system.faction_level_enabled(),
            write_faction_level_log: &mut write_faction_level_log,
            faction_experience_log_enabled: resources.log_system.faction_experience_enabled(),
            write_faction_experience_log: &mut write_faction_experience_log,
        };
        let process_message = match self.process_message_main_loop_stage(
            owners.honor_ranks,
            owners.player_ranks,
            owners.increment_log,
            owners.auction_log,
            &*owners.db_misc,
            owners.db_misc_context,
            owners.organizing,
            owners.organizing_parameters,
            owners.country,
            owners.country_parameters,
            owners.country_war,
            owners.four_nation_war,
            owners.faction_war,
            owners.attack_city,
            owners.attack_city_callbacks,
            owners.village_war,
            owners.goods_war,
            owners.timer,
            owners.village_war_callbacks,
            &mut *owners.resources,
            &mut *owners.load_player_largess,
            owners.net_sessions,
            owners.jjc,
            owners.jjc_context,
            owners.union_application_runtime,
            &mut union_application_callbacks,
            &mut write_faction_create_log,
            &mut write_faction_title_log,
            &mut write_faction_purview_log,
            &mut write_faction_apply_log,
            &mut write_faction_join_log,
            &mut write_faction_quit_log,
            &mut write_faction_fire_out_log,
            &mut write_faction_master_log,
            &mut write_faction_disband_log,
            owners.rs_player,
            owners.player_database.as_deref_mut(),
            Arc::clone(state.save_lifecycle),
            state.save_thread_handle,
            &mut *callbacks.save_runtime,
            owners.session_factory,
            owners.general_variables.as_deref_mut(),
            owners.gods_battle,
            owners.skills,
            owners.rs_gods_battle.as_deref_mut(),
            owners.gods_battle_database.as_deref_mut(),
            owners.log,
            &mut *callbacks.get_log_local_time,
            &mut *callbacks.put_log_info,
            &mut *callbacks.update_union_player,
            state.clocks,
            state.process_message,
            &mut *callbacks.get_tick,
        )
        .await
        {
            complete @ WorldProcessMessageStageReport::Complete { .. } => complete,
            blocked @ WorldProcessMessageStageReport::Blocked { .. } => {
                return Err(Box::new(WorldMainLoopBlock::ProcessMessage(blocked)));
            }
        };
        let resources = owners.resources.main_loop_resource_snapshot();
        let session_factory = self.run_main_loop_session_factory_stage(
            owners.session_factory,
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
        );
        // Arc-handle той же очереди вместо поля self: G-аргумент игры и queue
        // уходят раздельными заёмными ссылками, поведение FIFO не меняется.
        let player_data_queue_handle = self.player_data_queue.clone();
        let player_data_queue =
            match crate::app::playerdataqueue::run_main_loop_player_data_queue_stage(
                self,
                owners.organizing,
                &player_data_queue_handle,
                &mut state.clocks.stage_started_at_ms,
                &mut state.profile.process_player_data_queue_time_ms,
                &mut *callbacks.get_tick,
            ) {
                complete @ WorldMainLoopPlayerDataQueueStageReport::Complete { .. } => complete,
                blocked @ WorldMainLoopPlayerDataQueueStageReport::Blocked { .. } => {
                    return Err(Box::new(WorldMainLoopBlock::PlayerDataQueue(blocked)));
                }
            };
        let timer = self
            .run_main_loop_timer_stage(
                owners.timer,
                owners.time_to_return,
                owners.time_to_return_callbacks,
                owners.attack_city,
                owners.attack_city_callbacks,
                owners.village_war,
                owners.village_war_callbacks,
                owners.country_war,
                owners.country,
                owners.country_war_callbacks,
                owners.four_nation_war,
                owners.four_nation_war_callbacks,
                &resources.globe_setup,
                state.clocks,
                state.profile,
                state.copy_number_timer,
                owners.organizing_parameters,
                owners.player_ranks,
                owners.rs_player,
                owners.player_database.as_deref_mut(),
                owners.organizing,
                owners.log,
                &mut *callbacks.get_tick,
                &mut *callbacks.get_timer_local_time,
                &mut *callbacks.get_log_local_time,
                &mut *callbacks.put_log_info,
            )
            .await
            .map_err(|block| Box::new(WorldMainLoopBlock::Timer(block)))?;
        let faction_war = self
            .run_main_loop_faction_war_stage(
                owners.faction_war,
                owners.organizing,
                state.clocks,
                state.profile,
                &mut *callbacks.get_tick,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::FactionWar(block)))?;
        let lei_ting = self
            .run_main_loop_lei_ting_stage(
                owners.lei_ting,
                &resources.globe_setup,
                owners.lei_ting_context,
                owners.lei_ting_reset_worker,
                owners.tokio_runtime.clone(),
                &mut *callbacks.get_lei_ting_local_time,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::LeiTing(block)))?;
        let db_misc = {
            let save_info_time_ms = self.setup.save_info_time_ms;
            let mut delivery = WorldDbMiscDeliveryContext {
                game: self,
                gold_coin_index: resources.gold_coin_index,
                offline_drops: 0,
            };
            let result = Self::run_main_loop_db_misc_stage(
                owners.db_misc,
                owners.db_misc_context,
                &mut delivery,
                state.clocks,
                &mut *callbacks.get_tick,
            );
            let offline_drops = delivery.offline_drops;
            drop(delivery);
            for _ in 0..offline_drops {
                let _ = owners.log.add_log_text(
                    b"player not online, drop goods",
                    save_info_time_ms,
                    &mut *callbacks.get_tick,
                    &mut *callbacks.get_log_local_time,
                    &mut *callbacks.put_log_info,
                );
            }
            result.map_err(|block| Box::new(WorldMainLoopBlock::DbMisc(block)))?
        };
        let mut union_application_callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *callbacks.random,
            refresh_owned_city: &mut *callbacks.refresh_union_owned_city,
            faction_level_log_enabled: resources.log_system.faction_level_enabled(),
            write_faction_level_log: &mut write_faction_level_log,
            faction_experience_log_enabled: resources.log_system.faction_experience_enabled(),
            write_faction_experience_log: &mut write_faction_experience_log,
        };
        let net_sessions = self.run_main_loop_net_session_stage(
            owners.net_sessions,
            owners.organizing,
            owners.organizing_parameters,
            owners.union_application_runtime,
            &mut union_application_callbacks,
            &mut *callbacks.update_union_player,
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
        );
        let ping = self
            .run_main_loop_ping_stage(state.clocks)
            .map_err(|block| Box::new(WorldMainLoopBlock::Ping(block)))?;
        let minute = self
            .run_main_loop_minute_stage(
                state.initialization,
                state.tail_clocks,
                owners.organizing,
                owners.country,
                owners.country_parameters,
                owners.organizing_parameters,
                owners.attack_city,
                owners.village_war,
                owners.goods_war,
                &resources.globe_setup,
                &mut *callbacks.get_tick,
                &mut *callbacks.refresh_union_owned_city,
                &mut *callbacks.update_union_player,
                resources.log_system.faction_master_changed_enabled(),
                &mut write_faction_master_log,
                resources.log_system.faction_disband_enabled(),
                &mut write_faction_disband_log,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::Minute(block)))?;
        let bai_tan_jjc = self
            .run_main_loop_bai_tan_jjc_stage(
                owners.jjc,
                resources.globe_setup.jjc_run_config_world(),
                owners.jjc_context,
                owners.jjc_week_clear_worker,
                owners.tokio_runtime.clone(),
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::Jjc(block)))?;
        let tail = self.run_main_loop_tail_stage(
            state.tail_clocks,
            state.login_release,
            owners.organizing,
            &mut *callbacks.get_tick,
            &mut *callbacks.wait,
            &mut *callbacks.output_debug,
            owners.session_factory,
        );
        let tail = match tail {
            complete @ WorldMainLoopTailStageReport::Complete { .. } => complete,
            WorldMainLoopTailStageReport::BlockedMissingReleaseInterval { pacing } => {
                return Err(Box::new(WorldMainLoopBlock::Tail(pacing)));
            }
        };

        Ok(WorldMainLoopReport {
            profile_initialization,
            save_initialization,
            refresh_initialization,
            current_tick_ms,
            largess,
            refresh_profile_started_at_ms,
            refresh,
            reload,
            maintenance,
            collect_player_data,
            save,
            ai,
            process_message,
            session_factory,
            player_data_queue,
            timer,
            faction_war,
            lei_ting,
            db_misc,
            net_sessions,
            ping,
            minute,
            bai_tan_jjc,
            tail,
            legacy_result: 1,
        })
    }

    pub(crate) fn send_player_data_queue_rejection(&self, cdkey: &[u8]) -> Result<i32, SendMessageError> {
        let mut rejection = CMessage::new(0x0001_FF01);
        rejection.base_mut().add_byte(0x1C);
        add_legacy_c_string(rejection.base_mut(), cdkey);
        rejection.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        )
    }

    pub fn evaluate_main_loop_largess_gate(
        &self,
        clocks: &WorldMainLoopClockState,
        state: &mut WorldMainLoopLargessState,
    ) -> WorldMainLoopLargessGateReport {
        let Some(load_interval_ms) = self.setup.load_largess_time_ms else {
 // Конструктор не задавал `dwLoadLargessTime`;
 // неизвестное C++-чтение не позволяет назначить последующий counter.
            return WorldMainLoopLargessGateReport::BlockedMissingFact {
                field: "dwLoadLargessTime",
            };
        };

        state.pass_count = state.pass_count.wrapping_add(1);
        if load_interval_ms == 0 {
            return WorldMainLoopLargessGateReport::Disabled {
                load_interval_ms,
                pass_count: state.pass_count,
            };
        }
        if self.setup.world_number.is_none() {
 // TransferLargessThread форматировал `%d`
 // непосредственно из исходно неинициализированного dwNumber.
            return WorldMainLoopLargessGateReport::BlockedMissingFact {
                field: "dwNumber",
            };
        }

        let elapsed_ms = clocks
            .current_tick_ms
            .wrapping_sub(state.last_start_request_tick_ms);
        if load_interval_ms < elapsed_ms {
            state.last_start_request_tick_ms = clocks.current_tick_ms;
            WorldMainLoopLargessGateReport::StartWorkerRequested {
                load_interval_ms,
                pass_count: state.pass_count,
                elapsed_ms,
                requested_at_ms: clocks.current_tick_ms,
            }
        } else {
            WorldMainLoopLargessGateReport::Waiting {
                load_interval_ms,
                pass_count: state.pass_count,
                elapsed_ms,
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "caller связывает CGame, пять process-global state owners и три точных callbacks"
    )]

    pub fn run_main_loop_refresh_stage<GetTick, GetSavePointTime, GetLocalTime, PutLogInfo>(
        &mut self,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        process_message_state: &mut WorldProcessMessageStageState,
        high_water: &mut WorldRefreshInfoHighWater,
        external_counts: WorldRefreshExternalCounts,
        save_state: &SaveDataLifecycleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_save_point_time: &mut GetSavePointTime,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> WorldMainLoopRefreshStageReport
    where
        GetTick: FnMut() -> u32,
        GetSavePointTime: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let elapsed_since_refresh_ms = clocks
            .current_tick_ms
            .wrapping_sub(clocks.last_refresh_tick_ms);
        let mut refresh = WorldMainLoopRefreshDisposition::NotDue;
        if self.setup.refresh_info_time_ms < elapsed_since_refresh_ms {
            clocks.last_refresh_tick_ms = clocks.current_tick_ms;
            let current = match self.capture_refresh_info_current(external_counts) {
                Ok(current) => current,
                Err(block) => {
                    return WorldMainLoopRefreshStageReport::BlockedMissingFact {
                        elapsed_since_refresh_ms,
                        assigned_last_refresh_tick_ms: clocks.last_refresh_tick_ms,
                        block,
                    };
                }
            };
            refresh = match current {
                None => WorldMainLoopRefreshDisposition::MissingNetworkOwner,
                Some(current) => {
                    let save_point_time_ms = get_save_point_time();
                    let last_save_time = save_state.last_save_time;
                    let save = WorldRefreshSaveState {
                        last_save_time: WorldLogLocalTime {
                            year: last_save_time.year,
                            month: last_save_time.month,
                            day: last_save_time.day,
                            hour: last_save_time.hour,
                            minute: last_save_time.minute,
                            second: last_save_time.second,
                        },
                        save_point_time_ms,
                        last_save_tick_ms: save_state.last_save_tick_ms,
                        this_save_start_tick_ms: save_state.this_save_start_tick_ms,
                    };
                    WorldMainLoopRefreshDisposition::Refreshed(refresh_info_text(
                        log,
                        current,
                        high_water,
                        save,
                        &mut *get_tick,
                    ))
                }
            };
        }

        let finished_at_ms = get_tick();
        let elapsed_stage_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.refresh_text_time_ms = profile_state
            .refresh_text_time_ms
            .wrapping_add(elapsed_stage_ms);
        let profile = self.publish_main_loop_profile_if_due(
            clocks.current_tick_ms,
            profile_state,
            process_message_state,
            log,
            get_tick,
            get_local_time,
            put_log_info,
        );

        WorldMainLoopRefreshStageReport::Complete {
            elapsed_since_refresh_ms,
            refresh,
            finished_at_ms,
            elapsed_stage_ms,
            accumulated_refresh_time_ms: profile_state.refresh_text_time_ms,
            profile,
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "явные DB, organizing, clock и log owners сохраняют исходный порядок"
    )]

    pub(crate) async fn stat_player_ranks<PlayerDatabase, GetTick, GetLocalTime, PutLogInfo>(
        &self,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut PlayerDatabase,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> Result<PlayerRanksStatRunReport, PlayerRanksStatRunBlock>
    where
        PlayerDatabase: RsPlayerOwner<CPlayer>,
        GetTick: FnMut() -> u32 + ?Sized,
        GetLocalTime: FnMut() -> WorldLogLocalTime + ?Sized,
        PutLogInfo: FnMut(&[u8]) + ?Sized,
    {
        player_ranks.clear();
        let start_log = log.add_log_text(
            b"PlayerRanks Stat. START...",
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );
        let started_at_ms = get_tick();
        let outcome = rs_player
            .stat_ranks(player_ranks, organizing, player_database)
            .await;
        if let PlayerRanksStatOutcome::BlockedMissingFact(source) = &outcome {
            return Err(PlayerRanksStatRunBlock {
                started_at_ms,
                start_log,
                source: *source,
            });
        }
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
        let complete_text = format!(
            "PlayerRanks Stat. END(USED TIME:{}MS)",
            elapsed_ms as i32,
        )
        .into_bytes();
        let complete_log = log.add_log_text(
            &complete_text,
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );
        Ok(PlayerRanksStatRunReport {
            started_at_ms,
            finished_at_ms,
            elapsed_ms,
            outcome,
            start_log,
            complete_log,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "caller сохраняет clock/log callbacks и явные границы доменных owners"
    )]

    pub async fn run_main_loop_maintenance_stage<
        GetTick,
        GetLocalTime,
        GetAuctionMonthDay,
        PutLogInfo,
        RsPlayer,
    >(
        &self,
        player_ranks_request: &WorldPlayerRanksRequestState,
        use_appellation_function: bool,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut RsPlayer,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        honor_ranks_owner: &mut CHonorRanks,
        auction_log: &mut CAuctionLog,
        auction_log_database: Option<&mut WorldTdsClient>,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        get_auction_month_day: &mut GetAuctionMonthDay,
        put_log_info: &mut PutLogInfo,
    ) -> Result<WorldMainLoopMaintenanceReport, WorldMainLoopMaintenanceBlock>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        GetAuctionMonthDay: FnMut() -> i32,
        PutLogInfo: FnMut(&[u8]),
        RsPlayer: RsPlayerOwner<CPlayer>,
    {
        let player_ranks = if player_ranks_request.take_if_requested() {
            let stat = self
                .stat_player_ranks(
                    player_ranks,
                    rs_player,
                    player_database,
                    organizing,
                    log,
                    get_tick,
                    get_local_time,
                    put_log_info,
                )
                .await
                .map_err(WorldMainLoopMaintenanceBlock::PlayerRanksStat)?;
            let sender = self.current_game_server_sender();
            let publication = player_ranks
                .update_ranks_to_game_server(sender.as_ref())
                .map_err(WorldMainLoopMaintenanceBlock::PlayerRanksSerialization)?;
            WorldPlayerRanksMaintenanceDisposition::Updated {
                stat,
                publication,
            }
        } else {
            WorldPlayerRanksMaintenanceDisposition::NotRequested
        };

        let honor_ranks = if !use_appellation_function {
            WorldHonorRanksMaintenanceDisposition::Disabled
        } else {
            let current_day = u32::from(get_local_time().day);
            match honor_ranks_owner.sort_day() {
                sort_day if sort_day == current_day => {
                    WorldHonorRanksMaintenanceDisposition::AlreadyCurrent {
                        current_day,
                        sort_day,
                    }
                }
                previous_sort_day => {
                    let started_at_ms = get_tick();
                    let start_log = log.add_log_text(
                        b"Start total HonorRankks!",
                        self.setup.save_info_time_ms,
                        &mut *get_tick,
                        &mut *get_local_time,
                        &mut *put_log_info,
                    );
                    let rollover = honor_ranks_owner.on_new_day(self, false).map_err(|source| {
                        WorldMainLoopMaintenanceBlock::HonorRanks(
                            WorldHonorRanksMaintenanceBlock {
                                current_day,
                                previous_sort_day,
                                started_at_ms,
                                start_log: start_log.clone(),
                                source,
                            },
                        )
                    })?;
                    let finished_at_ms = get_tick();
                    let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
                    let complete_text = format!(
                        "Total today HonorRankks complete,consume time {} millisecond!",
                        elapsed_ms as i32,
                    )
                    .into_bytes();
                    let complete_log = log.add_log_text(
                        &complete_text,
                        self.setup.save_info_time_ms,
                        &mut *get_tick,
                        &mut *get_local_time,
                        &mut *put_log_info,
                    );
                    WorldHonorRanksMaintenanceDisposition::Updated {
                        current_day,
                        previous_sort_day,
                        started_at_ms,
                        finished_at_ms,
                        elapsed_ms,
                        start_log,
                        complete_log,
                        rollover,
                    }
                }
            }
        };

        let current_month_day = get_auction_month_day();
        let old_month_day = auction_log.old_auction_day();
        let auction_bang = if current_month_day == old_month_day {
            WorldAuctionBangMaintenanceDisposition::AlreadyCurrent {
                current_month_day,
                old_month_day,
            }
        } else {
            let start_log = log.add_log_text(
                b"Start AuctionBang Update",
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            auction_log.set_old_auction_day(current_month_day);
            let outcome = auction_log
                .update_auction_bang_db(auction_log_database)
                .await;
            let update_succeeded = matches!(&outcome, AuctionBangUpdateOutcome::ReturnedTrue);
            let result_text = if update_succeeded {
                b"AuctionBang Update Success!".as_slice()
            } else {
                b"AuctionBang Update Fail!".as_slice()
            };
            let result_log = log.add_log_text(
                result_text,
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            WorldAuctionBangMaintenanceDisposition::Updated {
                current_month_day,
                previous_old_month_day: old_month_day,
                update_succeeded,
                outcome,
                start_log,
                result_log,
            }
        };

        Ok(WorldMainLoopMaintenanceReport {
            player_ranks,
            honor_ranks,
            auction_bang,
        })
    }

    pub(crate) fn capture_refresh_info_current(
        &self,
        external: WorldRefreshExternalCounts,
    ) -> Result<Option<WorldRefreshInfoCurrent>, WorldRefreshSnapshotBlock> {
        let Some(net_server) = self.net_server.as_ref() else {
            return Ok(None);
        };
        let map_players = legacy_refresh_count("m_mPlayer", self.players.len())? as i32;
        let online_players = legacy_refresh_count("m_lOnlinePlayer", self.online_players.len())?;
        let offline_players = legacy_refresh_count("m_lOfflinePlayer", self.offline_players.len())?;
        let login_players = legacy_refresh_count("m_lLoginPlayer", self.login_players.len())?;
        let creation_players =
            legacy_refresh_count("m_lCreationPlayer", self.creation_players.len())?;
        let deletion_players =
            legacy_refresh_count("m_lDeletionPlayer", self.deletion_players.len())?;
        let restore_players = legacy_refresh_count("m_lRestorePlayer", self.restore_players.len())?;
        let saving_players = {
            let db_data = self.db_data.lock();
            legacy_refresh_count("m_stDBData.mDBPlayer", db_data.players.len())? as i32
        };
        let write_log_queue =
            legacy_refresh_count("m_qWriteLogData", self.write_log_queue.len())?;

        Ok(Some(WorldRefreshInfoCurrent {
            connections: net_server.client_count(),
            map_players,
            online_players,
            offline_players,
            login_players,
            creation_players,
            deletion_players,
            restore_players,
            saving_players,
            team_sessions: external.team_sessions,
            largess_entries: external.largess_entries,
            write_log_queue,
            player_load_queue: self.player_load_queue.get_size(),
            reback_messages: external.reback_messages,
        }))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "gate связывает два точных accumulator-owner-а и готовый logger"
    )]

    pub fn publish_main_loop_profile_if_due<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        now_ms: u32,
        state: &mut WorldMainLoopProfileState,
        process_message_state: &mut WorldProcessMessageStageState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> Option<WorldMainLoopProfileReport>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let elapsed_since_last_publish_ms = now_ms.wrapping_sub(state.last_published_at_ms);
        if elapsed_since_last_publish_ms <= 600_000 {
            return None;
        }

        let snapshot = WorldMainLoopProfileSnapshot {
            ai_calls: state.ai_calls,
            ai_time_ms: state.ai_time_ms,
            refresh_text_time_ms: state.refresh_text_time_ms,
            process_message_time_ms: process_message_state.accumulated_time_ms,
            login_server_message_time_ms: self.login_server_message_time_ms,
            game_server_message_time_ms: self.game_server_message_time_ms,
            net_session_time_ms: state.net_session_time_ms,
            faction_war_time_ms: state.faction_war_time_ms,
            timer_time_ms: state.timer_time_ms,
            process_player_data_queue_time_ms: state.process_player_data_queue_time_ms,
            session_factory_time_ms: state.session_factory_time_ms,
            save_point_time_ms: state.save_point_time_ms,
        };
        let message = format!(
            "In 600 S Called {} AI.\r\nAI:{}\r\nRefeashText:{}\r\nMessage:{} (LS:{}  GS:{})\r\nNetSession:{}\r\nFactionWarSys:{}\r\nTimer:{}\r\nProcessPlayerDataQueue:{}\r\nSessionFactory:{}\r\nSavePoint:{}\r\n",
            snapshot.ai_calls as i32,
            snapshot.ai_time_ms as i32,
            snapshot.refresh_text_time_ms as i32,
            snapshot.process_message_time_ms as i32,
            snapshot.login_server_message_time_ms as i32,
            snapshot.game_server_message_time_ms as i32,
            snapshot.net_session_time_ms as i32,
            snapshot.faction_war_time_ms as i32,
            snapshot.timer_time_ms as i32,
            snapshot.process_player_data_queue_time_ms as i32,
            snapshot.session_factory_time_ms as i32,
            snapshot.save_point_time_ms as i32,
        );
        let log = log.add_log_text(
            message.as_bytes(),
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );

        state.ai_time_ms = 0;
        state.refresh_text_time_ms = 0;
        process_message_state.accumulated_time_ms = 0;
        self.login_server_message_time_ms = 0;
        self.game_server_message_time_ms = 0;
        state.net_session_time_ms = 0;
        state.faction_war_time_ms = 0;
        state.timer_time_ms = 0;
        state.process_player_data_queue_time_ms = 0;
        state.session_factory_time_ms = 0;
        state.save_point_time_ms = 0;
        state.ai_calls = 0;
        state.last_published_at_ms = now_ms;

        Some(WorldMainLoopProfileReport {
            elapsed_since_last_publish_ms,
            snapshot,
            log,
        })
    }

}
