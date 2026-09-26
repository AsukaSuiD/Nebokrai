//! Save-семья `CGame` (`GenerateDBData` и materialize/take/append/clear):
//! владелец типа — [`crate::app::world_game`], методы семьи живут здесь.
//!
//! БД-поля остаются обычными полями `CGame` (`db_data: Mutex<WorldDbData>` и
//! соседи), поэтому семья не параметризуется. Порядок снапшотов, отдельные
//! соединения, частичный успех и исходное сопоставление ошибок сохраняются.
//!
//! Контракт подтверждён точной парой `Nworldserver.exe` + `WorldServer.pdb`
//! (канонические идентификаторы сборки —
//! `server/rust/src/manifest/_worldserver_export_manifest.toml`).
//!
//! Общие нормализации save-семейства описаны у [`crate::app::world_game`].

use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::rsgodsbattle::{GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot};
use crate::app::servermessage::WorldCompletedSaveResponseLaunchReport;
use crate::app::world_client::CMyNetClient;
use crate::app::world_game::CGame;
use crate::app::world_message::CMessage;
use crate::app::world_runtime::WorldRegionOwner;
use crate::app::world_save_reports::{WorldCollectPlayerDataBroadcast, WorldCollectPlayerDataRequestState, WorldManualSaveRequestReport, WorldRunImmediateSaveReport, WorldRunSaveLaunchReport, WorldRunSaveTriggerDisposition, WorldRunSaveTriggerState, WorldSaveAllOrganizationsLaunchReport, WorldSaveNotifyDelivery, WorldSaveNotifyReport};
use crate::app::worldserver::{WorldGenerateDbDataBlock, WorldGenerateDbDataReport, WorldLogLocalTime, WorldLogTextOwner, WorldSaveThreadHandleState, prepare_save_thread_launch};
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerPropertyCoefficients};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::variablelist::CVariableList;
use crate::organizations::country::CountryKingSaveLimits;
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::dbcountry::CountrySaveSnapshot;
use crate::organizations::faction::CFaction;
use crate::organizations::organizingctrl::{COrganizingCtrl, OrganizingSaveDataBlock};
use crate::organizations::rsenemyfactions::EnemyFactionSaveSnapshot;
use crate::organizations::union::CUnion;
use crate::persistence::savedata::{WorldDbData, WorldDbDataSaveSession, WorldSaveDataOwner};
use crate::persistence::savedb::{SaveDataLifecycleState, WorldSaveThreadJob};
use crate::persistence::saveworker::WorldSaveRuntimeContext;
use crate::regions::rsregion::RegionSaveSnapshot;
use nebokrai_shared::resources::CGodsBattleConf;
use parking_lot::Mutex;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

pub type WorldRunSaveGuard<'game> =
    crate::persistence::saveworker::WorldRunSaveGuard<'game, CGame>;

pub type WorldRunSavePreGateReport<'game> =
    crate::app::world_save_reports::WorldRunSavePreGateReport<'game, CGame>;

pub type WorldRunSaveTriggerReport<'game> =
    crate::app::world_save_reports::WorldRunSaveTriggerReport<'game, CGame>;

impl CGame {
    pub fn db_data_save_session(&mut self) -> WorldDbDataSaveSession<'_> {
        WorldDbDataSaveSession {
            data: self.db_data.get_mut(),
        }
    }

 /// Передаёт сформированный DB batch фоновому worker-у и публикует пустой
 /// accumulator для событий, пришедших уже после save-trigger-а.
    pub fn take_save_data_owner(&mut self) -> WorldSaveDataOwner {
        WorldSaveDataOwner {
            data: std::mem::replace(self.db_data.get_mut(), WorldDbData::new()),
            login_sender: self
                .current_login_client()
                .map(CMyNetClient::send_queue_handle),
        }
    }

    pub fn take_save_thread_job(
        &mut self,
        variables: &CVariableList,
        registry: &GoodsBasePropertiesRegistry,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
    ) -> WorldSaveThreadJob {
        let (a_faction_xyd, b_faction_xyd) = gods_battle.faction_xyd();
        let gods_battle_npc_factions = gods_battle
            .npc_names()
            .iter()
            .map(|npc| GodsBattleNpcFactionSnapshot {
                faction: npc.faction,
                name: npc.name.clone(),
            })
            .collect();
        let server_id = self
            .net_server
            .as_ref()
            .expect("save-trigger достигается только после World network init")
            .local_ipv4_word() as i32;
        let world_number_bits = self
            .setup
            .world_number
            .expect("save-trigger достигается только после обязательного setup dwNumber");

        WorldSaveThreadJob {
            save: self.take_save_data_owner(),
            variables: variables.clone(),
            registry: registry.clone(),
            honor_ranks: honor_ranks.take_save_owner(),
            gods_battle_faction_xyd: GodsBattleFactionXydSnapshot {
                a_faction_xyd: a_faction_xyd as i32,
                b_faction_xyd,
            },
            gods_battle_npc_factions,
            use_old_save_largess_way: self.setup.use_old_save_largess_way,
            save_info_time_ms: self.setup.save_info_time_ms,
            lifecycle,
            server_name: self.setup.name.clone(),
            server_id,
            world_number_bits,
            write_log_queue: self.write_log_queue.clone(),
        }
    }

 /// Создаёт player-prefix `GenerateDBData` до доменных generators.
 ///
 /// Restore-копирование выполняется через эксклюзивный `&mut self` без lock;
 /// deletion и обе player-очереди используют исходную save-блокировку.
    pub fn generate_db_data_player_prefix(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<(), WorldGenerateDbDataBlock> {
        let leave_word_id = self.leave_word_id;
        let player_id = self.player_id;

        let db_data = self.db_data.get_mut();
        db_data.player_id = player_id;
        db_data.leave_word_id = leave_word_id;

        let creation_ids = self
            .creation_players
            .iter()
            .map(|player_id| *player_id as u32)
            .collect::<Vec<_>>();
        for player_id in creation_ids {
            if let Some(player) =
                self.clone_creation_player(player_id, registry, organizing_ctrl, coefficients)?
            {
                self.append_db_creation_player(player);
            }
        }

        self.db_data
            .get_mut()
            .restore_players
            .extend(self.restore_players.iter().copied());

        for entry in self.deletion_players.iter().copied() {
            self.db_data.lock().deletion_players.push_back(entry);
        }

        let player_ids = self.players.keys().copied().collect::<Vec<_>>();
        for player_id in player_ids {
            if let Some(player) =
                self.clone_map_player(player_id, registry, organizing_ctrl, coefficients)?
            {
                self.append_db_player(player);
            }
        }

        Ok(())
    }

 /// Создаёт полный DB snapshot в исходном порядке владельцев.
 ///
 /// Функция ничего не очищает в live player-list после snapshot: это
 /// отдельные операции caller-а, следующие за `GenerateDBData`.
    #[allow(
        clippy::too_many_arguments,
        reason = "семь прежних singleton/static зависимостей передаются явно без нового общего owner-а"
    )]

    pub fn generate_db_data(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        honor_ranks: &mut CHonorRanks,
    ) -> Result<WorldGenerateDbDataReport, WorldGenerateDbDataBlock> {
        self.generate_db_data_player_prefix(registry, organizing_ctrl, coefficients)?;
        let organizing = organizing_ctrl.generate_save_data(self, false)?;
        faction_war_sys.generate_save_data(self);
        self.geterate_region_db_data();
        country_handler.generate_save_data(self, country_limits);
        honor_ranks.generate_save_data();

        Ok(WorldGenerateDbDataReport { organizing })
    }

 /// Создаёт отдельную `g_bSaveAllOrg` ветвь `CGame::Run`.
 ///
 /// Она не вызывает player-prefix и Country generator: исходник выполнял
 /// только organizing, faction-war, region и HonorRanks перед тем же launch.
    pub fn materialize_save_all_organizations_snapshot(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        faction_war_sys: &CFactionWarSys,
        variables: &CVariableList,
        registry: &GoodsBasePropertiesRegistry,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
    ) -> Result<WorldSaveAllOrganizationsLaunchReport, OrganizingSaveDataBlock> {
        let organizing = organizing_ctrl.generate_save_data(self, false)?;
        faction_war_sys.generate_save_data(self);
        self.geterate_region_db_data();
        honor_ranks.generate_save_data();
        let launch = prepare_save_thread_launch(save_thread_handle);
        let job = self.take_save_thread_job(
            variables,
            registry,
            honor_ranks,
            gods_battle,
            lifecycle,
        );
        let resulting_handle = save_runtime.launch(&launch, job);
        *save_thread_handle = resulting_handle;
        Ok(WorldSaveAllOrganizationsLaunchReport {
            organizing,
            launch,
            resulting_handle,
        })
    }

 /// Выполняет snapshot и live-cleanup save-trigger ветви `CGame::Run`.
 ///
 /// Только после snapshot/cleanup закрывает прежний handle-state, передаёт
 /// job process save-owner-у и сохраняет возвращённое `Open/Empty` состояние.
    #[allow(
        clippy::too_many_arguments,
        reason = "исходный Run обращался к тем же семи singleton/static зависимостям"
    )]

    pub fn materialize_run_save_snapshot(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
    ) -> Result<WorldRunSaveLaunchReport, WorldGenerateDbDataBlock> {
        let snapshot = self.generate_db_data(
            registry,
            organizing_ctrl,
            coefficients,
            faction_war_sys,
            country_handler,
            country_limits,
            honor_ranks,
        )?;
        self.clear_map_player_for_offline();
        self.clear_restore_player();
        self.clear_creation_player();
        self.clear_deletion_player();
        self.clear_offline_player();
        let launch = prepare_save_thread_launch(save_thread_handle);
        let job = self.take_save_thread_job(
            variables,
            registry,
            honor_ranks,
            gods_battle,
            lifecycle,
        );
        let resulting_handle = save_runtime.launch(&launch, job);
        *save_thread_handle = resulting_handle;
        Ok(WorldRunSaveLaunchReport {
            snapshot,
            launch,
            resulting_handle,
        })
    }

 /// Выполняет snapshot/cleanup хвост завершённой ветви `0x5FA03`.
 ///
 /// Счётчик DB-ответов уже сброшен caller-ом. Handle replacement и передача
 /// job process save-owner-у достигаются только после успешных
 /// snapshot/cleanup. Тело переехало из бывшего
 /// `appworld/message/servermessage.rs` вместе с тем же save-хвостом
 /// `materialize_run_save_snapshot`; различие — только обязательный
 /// player-prefix в генераторе.
    #[allow(
        clippy::too_many_arguments,
        reason = "исходный handler повторно обращался к тем же singleton/static владельцам"
    )]

    pub fn materialize_completed_save_response_snapshot(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
    ) -> Result<WorldCompletedSaveResponseLaunchReport, WorldGenerateDbDataBlock> {
        let snapshot = self.generate_db_data(
            registry,
            organizing_ctrl,
            coefficients,
            faction_war_sys,
            country_handler,
            country_limits,
            honor_ranks,
        )?;
        self.clear_map_player_for_offline();
        self.clear_restore_player();
        self.clear_creation_player();
        self.clear_deletion_player();
        self.clear_offline_player();
        let launch = prepare_save_thread_launch(save_thread_handle);
        let job = self.take_save_thread_job(variables, registry, honor_ranks, gods_battle, lifecycle);
        let resulting_handle = save_runtime.launch(&launch, job);
        *save_thread_handle = resulting_handle;
        Ok(WorldCompletedSaveResponseLaunchReport {
            snapshot,
            launch,
            resulting_handle,
        })
    }

 /// Исполняет ручную collect-player-data ветвь `CGame::Run`.
 ///
 /// Флаг очищается до создания пустого broadcast; исходно игнорировавшийся
 /// результат `SendAll` сохраняется только как наблюдаемый отчёт.
    pub fn materialize_collect_player_data_request(
        &self,
        state: &mut WorldCollectPlayerDataRequestState,
    ) -> Option<WorldCollectPlayerDataBroadcast> {
        if !state.send_now {
            return None;
        }

        state.send_now = false;
        let message_type = 0x0007_F808;
        let message = CMessage::new(message_type);
        let sender = self.current_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        Some(WorldCollectPlayerDataBroadcast {
            message_type,
            delivery,
        })
    }

 /// Выполняет точный pre-gate save-участка `CGame::Run`.
 ///
 /// `try_enter` вызывается только после строгого прохождения wrapping-
 /// интервала. Его `false` соответствует занятому critical section и
 /// сдвигает прежний save tick на исходные `1000` миллисекунд.
    #[allow(
        clippy::too_many_arguments,
        reason = "Run обращался к тем же process-global и singleton владельцам"
    )]

    pub fn materialize_run_save_pre_gate<
        'game,
        GetSavePointTime,
        GetTick,
        GetLocalTime,
        PutLogInfo,
    >(
        &'game mut self,
        state: &mut WorldRunSaveTriggerState,
        now_ms: u32,
        get_save_point_time: &mut GetSavePointTime,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> WorldRunSavePreGateReport<'game>
    where
        GetSavePointTime: FnMut() -> u32,
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let manual_request = if state.send_save_message_now {
            let log = log.add_log_text(
                b"Manual Send Save Request!",
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            let save_point_time_ms = get_save_point_time();
            state.last_save_point_time_ms = now_ms.wrapping_sub(save_point_time_ms);
            state.send_save_message_now = false;
            Some(WorldManualSaveRequestReport {
                log,
                save_point_time_ms,
            })
        } else {
            None
        };

        let profile_started_at_ms = get_tick();
        let elapsed_ms = now_ms.wrapping_sub(state.last_save_point_time_ms);
        let save_point_time_ms = get_save_point_time();
        if elapsed_ms <= save_point_time_ms {
            return WorldRunSavePreGateReport::IntervalNotElapsed {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
            };
        }

        if !save_runtime.try_enter_trigger() {
            state.last_save_point_time_ms = state.last_save_point_time_ms.wrapping_add(1_000);
            return WorldRunSavePreGateReport::SaveLockBusy {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                adjusted_last_save_point_time_ms: state.last_save_point_time_ms,
            };
        }

        let trigger = self.materialize_run_save_trigger_after_lock(
            state,
            now_ms,
            registry,
            organizing_ctrl,
            coefficients,
            faction_war_sys,
            country_handler,
            country_limits,
            variables,
            honor_ranks,
            gods_battle,
            lifecycle,
            save_thread_handle,
            log,
            get_tick,
            get_local_time,
            put_log_info,
            save_runtime,
        );
        WorldRunSavePreGateReport::AfterLock {
            manual_request,
            profile_started_at_ms,
            elapsed_ms,
            save_point_time_ms,
            trigger,
        }
    }

 /// Выполняет действующее save-решение `CGame::Run` после успешного try-lock.
 ///
 /// При manual-save и живых GameServer исходник сначала строит локальный
 /// snapshot/launch, затем повторно считает подключения и рассылает notify.
 /// Blocked generator сохраняет guard: исходный невозвратившийся путь не
 /// достигает ни второй проверки, ни `LeaveCriticalSection`.
    #[allow(
        clippy::too_many_arguments,
        reason = "Run обращался к тем же process-global и singleton владельцам"
    )]

    pub fn materialize_run_save_trigger_after_lock<
        'game,
        GetTick,
        GetLocalTime,
        PutLogInfo,
    >(
        &'game mut self,
        state: &mut WorldRunSaveTriggerState,
        now_ms: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
        lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
        save_runtime: &mut dyn WorldSaveRuntimeContext,
    ) -> WorldRunSaveTriggerReport<'game>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let save_info_time_ms = self.setup.save_info_time_ms;
        let guard = WorldRunSaveGuard { game: self };

        if state.save_all_organizations {
            state.save_all_organizations = false;
            state.last_save_point_time_ms = now_ms;
            let save = match guard.game.materialize_save_all_organizations_snapshot(
                organizing_ctrl,
                faction_war_sys,
                variables,
                registry,
                honor_ranks,
                gods_battle,
                Arc::clone(&lifecycle),
                save_thread_handle,
                save_runtime,
            ) {
                Ok(save) => save,
                Err(block) => {
                    return WorldRunSaveTriggerReport::BlockedSaveAllOrganizations { guard, block };
                }
            };
            save_runtime.leave_trigger();
            guard.release();
            return WorldRunSaveTriggerReport::Complete(
                WorldRunSaveTriggerDisposition::SaveAllOrganizations(save),
            );
        }

        let immediate = if state.save_now_data || guard.game.connected_game_server_count() < 1 {
            let log = log.add_log_text(
                b"Manual Saveing Player Data Now...",
                save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            state.last_save_point_time_ms = now_ms;
            state.save_now_data = false;
            let save = match guard.game.materialize_run_save_snapshot(
                registry,
                organizing_ctrl,
                coefficients,
                faction_war_sys,
                country_handler,
                country_limits,
                variables,
                honor_ranks,
                gods_battle,
                Arc::clone(&lifecycle),
                save_thread_handle,
                save_runtime,
            ) {
                Ok(save) => save,
                Err(block) => {
                    return WorldRunSaveTriggerReport::BlockedImmediateSave { guard, log, block };
                }
            };
            Some(WorldRunImmediateSaveReport { log, save })
        } else {
            None
        };

        let notify = if guard.game.connected_game_server_count() > 0 {
            let log = log.add_log_text(
                b"Send SaveNotify to all GameServer...",
                save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            state.last_save_point_time_ms = now_ms;
            let previous_db_responses = std::mem::replace(&mut guard.game.db_responses, 0);
            let message = CMessage::new(0x0007_F803);
            let sender = guard.game.current_game_server_sender();
            let deliveries = guard
                .game
                .game_servers
                .values()
                .filter(|game_server| game_server.connected)
                .map(|game_server| WorldSaveNotifyDelivery {
                    game_server_index: game_server.index,
                    delivery: message.send_to_map_id(sender.as_ref(), game_server.index as i32),
                })
                .collect();
            Some(WorldSaveNotifyReport {
                log,
                previous_db_responses,
                message_type: message.message_type(),
                deliveries,
            })
        } else {
            None
        };

        save_runtime.leave_trigger();
        guard.release();
        WorldRunSaveTriggerReport::Complete(WorldRunSaveTriggerDisposition::PlayerData {
            immediate,
            notify,
        })
    }

    pub fn geterate_region_db_data(&self) {
        for assignment in self.regions.values() {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
                continue;
            };
            self.append_region_param(region.generate_save_data());
        }
    }

    pub(crate) fn player_organizing_region_types(&self) -> BTreeMap<i32, Option<u16>> {
        self.regions
            .iter()
            .filter_map(|(&region_id, assignment)| {
                assignment.region.as_ref()?;
                Some((
                    region_id,
                    assignment.region_type.map(|region_type| region_type as u16),
                ))
            })
            .collect()
    }

 /// Добавляет save-копию в `m_stDBData.liDBCreationPlayer`.
 ///
 /// При первом совпадении inherited ID старая копия уничтожается и
 /// удаляется, после чего новая всегда дописывается в хвост под тем же lock.
    pub fn append_db_creation_player(&self, player: Box<CPlayer>) {
        let player_id = player.get_id();
        let mut db_data = self.db_data.lock();

        if let Some(index) = db_data
            .creation_players
            .iter()
            .position(|existing| existing.get_id() == player_id)
        {
            drop(db_data.creation_players.remove(index));
        }
        db_data.creation_players.push_back(player);
    }

 /// Вставляет save-копию в `m_stDBData.mDBPlayer` по unsigned ID.
 ///
 /// Существующая копия уничтожается до вставки новой и всё изменение
 /// остаётся внутри исходной critical-section границы.
    pub fn append_db_player(&self, player: Box<CPlayer>) {
        let player_id = player.get_id() as u32;
        let mut db_data = self.db_data.lock();

        if let Some(previous) = db_data.players.remove(&player_id) {
            drop(previous);
        }
        db_data.players.insert(player_id, player);
    }

    pub fn append_save_faction(&self, faction: Box<CFaction>, goods_war_count: i32) {
        let faction_id = faction.faction_id();
        let mut db_data = self.db_data.lock();
        db_data.save_factions.push_back(faction);
        db_data
            .faction_goods_war_counts
            .insert(faction_id, goods_war_count);
    }

    pub fn append_save_union(&self, union: Box<CUnion>) {
        self.db_data.lock().save_unions.push_back(union);
    }

    pub fn append_delete_faction(&self, faction_id: i32) {
        self.db_data.lock().delete_factions.push_back(faction_id);
    }

    pub fn append_delete_union(&self, union_id: i32) {
        self.db_data.lock().delete_unions.push_back(union_id);
    }

    pub fn append_region_param(&self, region: RegionSaveSnapshot) {
        self.db_data.lock().regions.push_back(Some(region));
    }

    pub fn append_db_country(&self, country: CountrySaveSnapshot) {
        self.db_data.lock().countries.push_back(Some(country));
    }

 /// Заменяет весь `m_stDBData.listEnemyFactions` под save-lock.
 ///
 /// Вход уже владеет отдельными копиями. `Option` сохраняет допустимый
 /// null pointer-list элемент, хотя готовый generator создаёт только
 /// non-null записи.
    pub fn set_enemy_factions(
        &self,
        enemy_factions: VecDeque<Option<EnemyFactionSaveSnapshot>>,
    ) {
        let mut db_data = self.db_data.lock();
        db_data.enemy_factions = enemy_factions;
    }

 /// Выполняет полный действующий `CGame::ClearDBData` под одним lock.
 ///
 /// Scalar ID исходная функция не сбрасывала. Region-часть удаляет только
 /// nodes: их non-null save-копии обязан уничтожить предшествующий save.
    pub fn clear_db_data(&self) {
        let mut db_data = self.db_data.lock();

        while let Some(player) = db_data.creation_players.pop_front() {
            drop(player);
        }
        db_data.restore_players.clear();
        db_data.deletion_players.clear();
        while let Some((_player_id, player)) = db_data.players.pop_first() {
            drop(player);
        }
        while let Some(faction) = db_data.save_factions.pop_front() {
            drop(faction);
        }
        while let Some(union) = db_data.save_unions.pop_front() {
            drop(union);
        }
        db_data.delete_factions.clear();
        db_data.delete_unions.clear();
        // WorldServer не вызывает virtual destructor здесь:
        // save-фаза уничтожает value, сохраняя node до этой общей очистки.
        db_data.regions.clear();
    }

 /// Освобождает snapshot-очереди, которые точный `ClearDBData` не трогал.
 ///
 /// Вызывается только после `join_save_worker` в normal `Release`: к этой
 /// точке штатный `GameThreadFunc` уже не оставляет DB-потребителя и сразу
 /// уничтожает `CGame`. У самих snapshot-значений нет callback/DB side
 /// effects, поэтому это исправляет лишь внутреннее удержание owner-ов.
    pub(crate) fn clear_release_only_db_snapshots(&self) {
        let mut db_data = self.db_data.lock();
        db_data.enemy_factions.clear();
        db_data.countries.clear();
    }

}
