//! `Init`/`Release` и net init/reconnect `CGame` из `worldserver/game.cpp/.h`
//! (см. [`crate::app::world_game`]).
//!
//! `Init` (`1:00017ee0`) сохраняет порядок загрузки ресурсов, подключения БД,
//! создания игровых registry, сетевых владельцев и workers. Ошибка не
//! откатывает уже созданное; вызывающий всегда выполняет `Release` над
//! частичным состоянием. `Release` (`1:0000d7f0`) останавливает producers,
//! закрывает сеть, проводит save barrier, дожидается БД/workers и освобождает
//! владельцев в исходном порядке.
//!
//! Имена и проекции типов — Realm/Shared формы (см.
//! `crate::app::world_game`); inherent-методы `pub` для process-owner-а и
//! старого пакета.

use crate::activities::attackcitysys::{AttackCityCallbacks, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarCallbacks, CountryWarSys};
use crate::activities::factionwarsys::{CFactionWarSys, FactionWarIniLoadCompletion};
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarCallbacks};
use crate::activities::jjcsystem::{CJJcSystem, JJC_CONFIG_PATH, JJC_LEVEL_LIST_PATH, JJC_REGION_LIST_PATH, JjcConfigurationLoadReport};
use crate::activities::misc::CopyNumberTimerState;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarCallbacks};
use crate::app::loginreconnectworker::{WorldLoginReconnect, WorldLoginReconnectError, WorldLoginReconnectSpec, WorldLoginReconnectThreadRestart, WorldLoginReconnectThreadStart, WorldLoginReconnectWorker, WorldLoginReconnectWorkerCompletion, WorldLoginReconnectWorkerOutcome, resolve_login_endpoint};
use crate::app::misc_game::legacy_tick_ms;
use crate::app::world_client::CMyNetClient;
use crate::app::world_dispatch::{WorldCountryExileResultEffects, add_legacy_c_string, legacy_c_string_prefix};
use crate::app::world_game::CGame;
use crate::app::world_hub_data::{WorldGameInitAttackCityContext, WorldGameInitCallbacks, WorldGameInitEnemyMutationEffects};
use crate::app::world_hub_entries::WorldGameServerEntry;
use crate::app::world_init_context::WorldGameInitContext;
use crate::app::world_message::CMessage;
use crate::app::world_runtime::{WorldClientInitialization, WorldClientInitializationError, WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitBlock, WorldGameInitBlockReason, WorldGameInitBooleanOwner, WorldGameInitEvent, WorldGameInitOperatorNotice, WorldGameInitReport, WorldGameInitResult, WorldGameInitVoidOwner, WorldGameInitWorkerKind, WorldGameReleaseBlock, WorldGameReleaseContext, WorldGameReleaseDatabaseOwner, WorldGameReleaseEvent, WorldGameReleaseLiveList, WorldGameReleaseOptionalOwner, WorldGameReleaseReport, WorldGameReleaseResult, WorldGameReleaseVoidOwner, WorldNetworkInitializationError, WorldRegionOwner, WorldSaveCityRegionBlock, WorldServerSetupLoadReport, WorldSetupLoadReport, WorldSetupOpenError, WorldSetupSource};
use crate::app::world_server::CMyNetServer;
use crate::app::world_setup::{WorldServerSetupTokens, resolve_world_runtime_file};
use crate::app::worldserver::{WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldLogTextOwner, WorldReloadContext, resolve_first_local_ipv4};
use crate::auction::auctionlog::{AuctionLogLoadOutcome, CAuctionLog};
use crate::billing::incrementlog::CIncrementLog;
use crate::rankings::honorranks::CHonorRanks;
use crate::characters::player::CPlayer;
use crate::characters::playerloadworker::{WorldGameInitWorkerHandleState, WorldPlayerDataLoadOwner, WorldPlayerLoadBatchBlock, WorldPlayerLoadBatchReport, WorldPlayerLoadWorkerBlock, WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerSpec};
use crate::rankings::playerranks::{CPlayerRanks, PlayerRanksInitializationConfig};
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::CSkillFactory;
use crate::content::{TimeToReturn, TimeToReturnCallbacks, TimeToReturnLoadError, TimeToReturnLoadReport};
use crate::content::variablelist::CVariableList;
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::faction::{FactionEnemyMutationBlock, FactionInitialPropertyBlock};
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::COrganizingCtrl;
use crate::organizations::organizingparam::{COrganizingParam, OrganizingParamLoadError};
use crate::persistence::largess::{CostDatabaseSettings, CostDatabaseSettingsParts};
use crate::persistence::rssetup::{LoadedSetupIds, WorldDatabaseSettings, WorldDatabaseSettingsParts};
use crate::persistence::writelogworker::{WorldWriteLogWorker, WorldWriteLogWorkerSpec};
use nebokrai_shared::network::{DEFAULT_SOCKET_TYPE, bind_tcp_ipv4, legacy_ipv4_word};
use nebokrai_shared::resources::{CDupliRegionSetup, CGodsBattleConf};
use nebokrai_shared::runtime::{CTimer, ini_decode};
use std::{fs, io};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use crate::activities::rsgodsbattle::RsGodsBattleOwner;
use crate::organizations::rsenemyfactions::RsEnemyFactionsOwner;

/// Адаптер-мост `init_owner_relation`: связывает `&mut COrganizingCtrl` с
/// `&CGame` на единственном init-call-site, потому что inherent
/// `add_owned_city_to_faction` требует владельца игры для owned-city wire.
pub(crate) struct RegionOwnerOrganizingBridge<'a> {
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) game: &'a CGame,
}

impl crate::app::world_organizing_view::WorldRegionOwnerOrganizingView
    for RegionOwnerOrganizingBridge<'_>
{
    fn has_faction(&self, faction_id: i32) -> bool {
        self.organizing.faction_by_id(faction_id).is_some()
    }

    fn has_confederation(&self, union_id: i32) -> bool {
        self.organizing.confederation_by_id(union_id).is_some()
    }

    fn add_owned_city_to_faction(
        &mut self,
        faction_id: i32,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        Option<crate::organizations::faction::OwnedCityAddOutcome>,
        crate::organizations::faction::OwnedCityMutationBuildError,
    > {
        self.organizing.add_owned_city_to_faction(
            self.game,
            faction_id,
            region_id,
            update_player,
        )
    }

    fn country_by_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<u8>, FactionInitialPropertyBlock> {
        self.organizing.country_by_faction(faction_id)
    }
}

impl CGame {
    pub(crate) fn write_log_worker_spec(&self) -> WorldWriteLogWorkerSpec {
        let settings = WorldDatabaseSettings::from_parts(WorldDatabaseSettingsParts {
            host: self.setup.log_system_server.clone(),
            database: self.setup.log_system_database.clone(),
            user: self.setup.log_system_user.clone(),
            password: self.setup.log_system_password.clone(),
        });
        WorldWriteLogWorkerSpec::new(
            self.setup.use_log_system,
            settings,
            self.write_log_queue.clone(),
        )
    }

    pub(crate) fn player_load_worker_spec(&self) -> WorldPlayerLoadWorkerSpec<CPlayer> {
        WorldPlayerLoadWorkerSpec::new(
            self.player_load_queue.clone(),
            self.player_data_queue.clone(),
        )
    }

    pub const fn apply_loaded_setup_ids(&mut self, loaded: LoadedSetupIds) {
        self.player_registry.player_id = loaded.player_id;
        self.leave_words.next = loaded.leave_world_id;
    }

    /// Позиционно читает `setup.ini`, а при ошибке открытия — `setup.dat`.
    ///
    /// Успешное открытие остаётся успешной загрузкой даже после stream
    /// fail-state. Метод не запускает сервисы и не публикует значения файла.
    pub fn load_setup<ClaimSingleInstance>(
        &mut self,
        runtime_directory: &Path,
        mut claim_single_instance: ClaimSingleInstance,
    ) -> Result<WorldSetupLoadReport, WorldSetupOpenError>
    where
        ClaimSingleInstance: FnMut(&[u8]) -> bool,
    {
        let plain_path = runtime_directory.join("setup.ini");
        let encoded_path = runtime_directory.join("setup.dat");

        let (source, parsed_pairs, stopped_at_pair) = match fs::read(&plain_path) {
            Ok(bytes) => {
                let (parsed, stopped) = self.setup.parse_plain(&bytes);
                (WorldSetupSource::Plain, parsed, stopped)
            }
            Err(plain) => match fs::read(&encoded_path) {
                Ok(bytes) => {
                    let decoded = ini_decode(&bytes);
                    let c_string_len = decoded
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(decoded.len());
                    let (parsed, stopped) = self.setup.parse_encoded(&decoded[..c_string_len]);
                    (WorldSetupSource::Encoded, parsed, stopped)
                }
                Err(encoded) => {
                    return Err(WorldSetupOpenError::new(plain_path, plain, encoded_path, encoded));
                }
            },
        };

        let mut instance_title = b"WorldServer[".to_vec();
        instance_title.extend_from_slice(&self.setup.name);
        instance_title.push(b']');
        if source == WorldSetupSource::Plain {
            instance_title.extend_from_slice(b"-Saga3D2");
        }
        let instance_claimed = claim_single_instance(&instance_title);

        Ok(WorldSetupLoadReport {
            source,
            parsed_pairs,
            stopped_at_pair,
            instance_title,
            instance_claimed,
        })
    }

    /// Читает `serverSetup.ini`, не очищая прежний GameServer registry.
    ///
    /// Ошибка открытия не меняет map и соответствует старому `false`.
    /// Успешное открытие сохраняет legacy-успех даже после stream fail-state;
    /// безопасная граница останавливает только запись с неизвестным первым ID.
    pub fn load_server_setup(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<WorldServerSetupLoadReport, io::Error> {
        let path = resolve_world_runtime_file(runtime_directory, "serverSetup.ini")?;
        let bytes = fs::read(path)?;
        let mut tokens = WorldServerSetupTokens::new(&bytes);

        let _header = tokens.next_bytes();
        let declared_records = tokens.next_ascii::<i32>().unwrap_or(0);
        let mut current_index = None;
        let mut current_ip = Vec::new();
        let mut current_port = None;
        let mut applied_records = 0;
        let mut blocked_at_record = None;

        for record_index in 0..declared_records {
            let _marker_found = tokens.seek_to(b"#");
            if let Some(index) = tokens.next_ascii::<u32>() {
                current_index = Some(index);
            }
            if let Some(ip) = tokens.next_bytes() {
                current_ip = ip.to_vec();
            }
            if let Some(port) = tokens.next_ascii::<u32>() {
                current_port = Some(port);
            }

            let Some(index) = current_index else {
                // При неуспехе первого числового чтения всё равно
                // использовал неизвестный
                // stack-key в map::operator[]. Safe Rust не выбирает ключ.
                blocked_at_record = Some(record_index as usize + 1);
                break;
            };
            self.region_registry.game_servers.insert(
                index,
                WorldGameServerEntry {
                    connected: false,
                    index,
                    ip: current_ip.clone(),
                    port: current_port,
                    // ->
                    // переносит неинициализированный stack DWORD.
                    received_player_data: None,
                },
            );
            applied_records += 1;
        }

        Ok(WorldServerSetupLoadReport {
            declared_records,
            applied_records,
            unique_game_servers: self.region_registry.game_servers.len(),
            stream_complete: !tokens.failed(),
            blocked_at_record,
            read_end_notice: blocked_at_record.is_none(),
        })
    }

    pub(crate) fn record_game_init_log(
        &self,
        events: &mut Vec<WorldGameInitEvent>,
        log: &mut WorldLogTextOwner,
        callbacks: &mut WorldGameInitCallbacks<'_>,
        payload: &[u8],
    ) {
        let disposition = log.add_log_text(
            payload,
            self.setup.save_info_time_ms,
            &mut *callbacks.get_tick,
            &mut *callbacks.get_log_local_time,
            &mut *callbacks.put_log_info,
        );
        events.push(WorldGameInitEvent::Log {
            payload: payload.to_vec(),
            disposition,
        });
    }

    pub(crate) fn record_game_init_notice<Context: WorldGameInitContext>(
        events: &mut Vec<WorldGameInitEvent>,
        context: &mut Context,
        title: &[u8],
        message: &[u8],
    ) {
        let notice = WorldGameInitOperatorNotice {
            title: title.to_vec(),
            message: message.to_vec(),
        };
        context.notify_operator(&notice);
        events.push(WorldGameInitEvent::OperatorNotice(notice));
    }

    pub(crate) fn load_jjc_configuration_from_resources<Context: WorldReloadContext + ?Sized>(
        jjc: &mut CJJcSystem,
        context: &mut Context,
    ) -> JjcConfigurationLoadReport {
        let Some(region_source) = context.read_resource(JJC_REGION_LIST_PATH) else {
            return jjc.load_configuration(None, None, None);
        };
        let Some(level_source) = context.read_resource(JJC_LEVEL_LIST_PATH) else {
            return jjc.load_configuration(Some(&region_source), None, None);
        };
        let config_source = context.read_resource(JJC_CONFIG_PATH);
        jjc.load_configuration(
            Some(&region_source),
            Some(&level_source),
            config_source.as_deref(),
        )
    }

    pub(crate) fn database_initialization_snapshot(&self) -> WorldGameDatabaseInitialization {
        WorldGameDatabaseInitialization {
            settings: WorldDatabaseSettings::from_parts(WorldDatabaseSettingsParts {
                host: self.setup.sql_server_ip.clone(),
                database: self.setup.database_name.clone(),
                user: self.setup.sql_user_name.clone(),
                password: self.setup.sql_password.clone(),
            }),
            log_settings: WorldDatabaseSettings::from_parts(WorldDatabaseSettingsParts {
                host: self.setup.log_system_server.clone(),
                database: self.setup.log_system_database.clone(),
                user: self.setup.log_system_user.clone(),
                password: self.setup.log_system_password.clone(),
            }),
            cost_settings: CostDatabaseSettings::from_parts(CostDatabaseSettingsParts {
                provider: self.setup.cost_database_provider.clone(),
                host: self.setup.cost_database_ip.clone(),
                database: self.setup.cost_database_name.clone(),
                user: self.setup.cost_database_user.clone(),
                password: self.setup.cost_database_password.clone(),
            }),
            incoming_cost_settings: CostDatabaseSettings::from_parts(
                CostDatabaseSettingsParts {
                    provider: self.setup.login_cost_database_provider.clone(),
                    host: self.setup.login_cost_database_ip.clone(),
                    database: self.setup.login_cost_database_name.clone(),
                    user: self.setup.login_cost_database_user.clone(),
                    password: self.setup.login_cost_database_password.clone(),
                },
            ),
            load_largess_time_ms: self.setup.load_largess_time_ms.unwrap_or(0),
            use_old_save_largess_way: self.setup.use_old_save_largess_way,
            connection_type: self.setup.sql_connection_type.clone(),
            legacy_zero: b"0",
            integrated_security: b"SSPI",
        }
    }

    /// Применяет одну DB-пару enemy factions через живые organizing owners.
    ///
    /// Контекст форматирования принадлежит текущему `CGame`: process-оболочка
    /// не может корректно держать вторую копию StringTable или захватывать
    /// `CGame` внешней closure на время его же `Init`.
    pub(crate) fn apply_loaded_enemy_faction_relation(
        &self,
        organizing: &mut COrganizingCtrl,
        first_faction_id: i32,
        second_faction_id: i32,
    ) -> Result<(), FactionEnemyMutationBlock> {
        if first_faction_id <= 0 || second_faction_id <= 0 {
            return Ok(());
        }
        let Some(first_name) = organizing
            .faction_by_id(first_faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
        else {
            return Ok(());
        };
        let Some(second_name) = organizing
            .faction_by_id(second_faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
        else {
            return Ok(());
        };

        let mut first_effects = WorldGameInitEnemyMutationEffects {
            game: self,
            enemy_id: second_faction_id,
            enemy_name: second_name,
        };
        let _ = organizing.add_city_war_enemy_organizing(
            first_faction_id,
            second_faction_id,
            &mut first_effects,
        )?;

        let mut second_effects = WorldGameInitEnemyMutationEffects {
            game: self,
            enemy_id: first_faction_id,
            enemy_name: first_name,
        };
        let _ = organizing.add_city_war_enemy_organizing(
            second_faction_id,
            first_faction_id,
            &mut second_effects,
        )?;
        Ok(())
    }

    /// Выполняет `CGame::Init` до запуска write/player-load workers.
    ///
    /// Resources, setup, DB, registries, timers и network owners создаются в
    /// исходном fail-fast порядке; уже выполненные стадии при ошибке не откатываются.
    /// Country initialization идёт после параметров и optional honor ranks, затем
    /// запускается country war. Goods War DB reload выполняется между Country и
    /// DbMisc, но его ошибка не блокирует Init. Increment log загружается после
    /// general variables и также сохраняет свой старый нефатальный результат.
    /// Tiberius, resource callbacks и явные timer/clock owners заменяют globals,
    /// ADO и Windows API без перестановки стадий.
    #[allow(
        clippy::too_many_arguments,
        reason = "прямые PlayerRanks/country/timer/increment owners заменяют прежние opaque callbacks"
    )]

    pub async fn init<
        Context,
        ReloadContext,
        TimerCallback,
    >(
        &mut self,
        runtime_directory: &Path,
        context: &mut Context,
        reload_context: &mut ReloadContext,
        jjc: &mut CJJcSystem,
        gods_battle: &mut CGodsBattleConf,
        skills: &mut CSkillFactory,
        time_to_return: &mut TimeToReturn,
        time_to_return_callbacks: TimeToReturnCallbacks<TimerCallback>,
        general_variables: &mut Option<CVariableList>,
        organizing_parameters: &mut COrganizingParam,
        attack_city: &mut CAttackCitySys,
        attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
        four_nation_war: &mut CFourNationWarSys,
        four_nation_war_callbacks: FourNationWarCallbacks<TimerCallback>,
        village_war: &mut CVillageWarSys,
        village_war_callbacks: VillageWarCallbacks<TimerCallback>,
        faction_war: &mut CFactionWarSys,
        player_ranks: &mut CPlayerRanks,
        timer: &mut CTimer<TimerCallback>,
        copy_number_timer: &mut CopyNumberTimerState,
        copy_number_callback: TimerCallback,
        organizing_tax_callback: TimerCallback,
        player_ranks_callback: TimerCallback,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        goods_war: &mut CGoodsWarMember,
        country_war_system: &mut CountryWarSys,
        country_war_callbacks: CountryWarCallbacks<TimerCallback>,
        honor_ranks: &mut CHonorRanks,
        increment_log: &mut CIncrementLog,
        auction_log: &mut CAuctionLog,
        log: &mut WorldLogTextOwner,
        callbacks: &mut WorldGameInitCallbacks<'_>,
    ) -> WorldGameInitResult<<Context as WorldGameInitContext>::Block>
    where
        Context: WorldGameInitContext,
        ReloadContext: WorldReloadContext,
        TimerCallback: Copy,
    {
        let mut events = Vec::new();
        macro_rules! stop {
            ($reason:expr) => {
                return Err(Box::new(WorldGameInitBlock {
                    events,
                    reason: $reason,
                }))
            };
        }

        context.install_crash_reporter();
        events.push(WorldGameInitEvent::CrashReporterInstalled);

        let seed = context.current_time_seconds() as u32;
        context.seed_random(seed);
        let discarded_roll = context.random(100);
        events.push(WorldGameInitEvent::RandomInitialized {
            seed,
            discarded_roll,
        });

        // Оба critical section уже являются Rust owners: `db_data` Mutex и
        // эксклюзивная save-thread guard-граница. До worker-start их не видно.
        events.push(WorldGameInitEvent::RustLocksReady);
        context.put_debug_string(b"WorldServer start!");
        events.push(WorldGameInitEvent::DebugStartPublished);
        let _ = self.load_server_resource_from_context(reload_context);
        events.push(WorldGameInitEvent::ServerResourcesLoaded);

        let setup = match self.load_setup(runtime_directory, |title| {
            context.claim_single_instance(title)
        }) {
            Ok(setup) => setup,
            Err(error) => stop!(WorldGameInitBlockReason::SetupOpen(error)),
        };
        let instance_claimed = setup.instance_claimed;
        let instance_title = setup.instance_title.clone();
        events.push(WorldGameInitEvent::SetupLoaded(setup));
        if !instance_claimed {
            let mut message = instance_title.clone();
            message.extend_from_slice(b" App Is Running!");
            Self::record_game_init_notice(&mut events, context, b"ERROR", &message);
            stop!(WorldGameInitBlockReason::ExistingInstance {
                title: instance_title,
            });
        }

        let server_setup = match self.load_server_setup(runtime_directory) {
            Ok(setup) => setup,
            Err(error) => stop!(WorldGameInitBlockReason::ServerSetup(error)),
        };
        events.push(WorldGameInitEvent::ServerSetupLoaded(server_setup));

        let Some(player_load_thread_count) = self.setup.player_load_thread_count else {
            stop!(WorldGameInitBlockReason::MissingPlayerLoadThreadCount);
        };
        if player_load_thread_count == 0 || 8 < player_load_thread_count {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"message",
                b"Player I/O Threads Must between 1 And 8",
            );
            stop!(WorldGameInitBlockReason::InvalidPlayerLoadThreadCount {
                count: player_load_thread_count,
                legacy_exit_code: 1,
            });
        }
        events.push(WorldGameInitEvent::PlayerLoadThreadCountValidated(
            player_load_thread_count,
        ));

        self.clear_string_table();
        events.push(WorldGameInitEvent::StringTablesCleared);
        const DEFAULT_LANGUAGE: &[u8] = b"data/Language.lag";
        let default_language_source = reload_context.read_resource(DEFAULT_LANGUAGE);
        let default_language = self.load_string_table_resource(
            DEFAULT_LANGUAGE,
            default_language_source.as_deref(),
        );
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            &default_language.log_payload,
        );
        if !default_language.succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"read language (data/Language.lag).....failed!",
            );
            stop!(WorldGameInitBlockReason::DefaultLanguageTable);
        }
        events.push(WorldGameInitEvent::StringTableLoaded {
            package: DEFAULT_LANGUAGE.to_vec(),
        });
        let configured_language = self.setup.language_package.clone();
        let configured_language_source = reload_context.read_resource(&configured_language);
        let configured_language_load = self.load_string_table_resource(
            &configured_language,
            configured_language_source.as_deref(),
        );
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            &configured_language_load.log_payload,
        );
        if !configured_language_load.succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"Load language packet [data/Language.lag]...FAILED! ",
            );
            stop!(WorldGameInitBlockReason::ConfiguredLanguageTable);
        }
        events.push(WorldGameInitEvent::StringTableLoaded {
            package: configured_language,
        });
        if let Err(block) = self.code_string_table() {
            stop!(WorldGameInitBlockReason::StringTableEncoding(block));
        }
        events.push(WorldGameInitEvent::StringTablesCoded);

        const DUPLI_REGION_SETUP_PATH: &[u8] = b"setup/DupliRegionsSetup.ini";
        self.content_catalogs.dupli_region_setup = Some(CDupliRegionSetup::default());
        let dupli_region_source = reload_context.read_resource(DUPLI_REGION_SETUP_PATH);
        let dupli_region_loaded = self
            .content_catalogs
            .dupli_region_setup
            .as_mut()
            .expect("owner опубликован перед Load")
            .load(dupli_region_source.as_deref());
        if !dupli_region_loaded {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"message",
                b"Can't find file setup/DupliRegionsSetup.ini",
            );
            stop!(WorldGameInitBlockReason::DupliRegionSetup);
        }
        events.push(WorldGameInitEvent::DupliRegionSetupLoaded);

        if let Err(block) =
            context.initialize_database_layer(self.database_initialization_snapshot())
        {
            stop!(WorldGameInitBlockReason::Context(block));
        }
        events.push(WorldGameInitEvent::DatabaseLayerInitialized);
        let jjc_configuration =
            Self::load_jjc_configuration_from_resources(jjc, reload_context);
        if let Some(path) = jjc_configuration.missing_path() {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"file not found",
                path,
            );
            stop!(WorldGameInitBlockReason::JjcConfiguration(
                jjc_configuration,
            ));
        }
        events.push(WorldGameInitEvent::JjcConfigurationLoaded(jjc_configuration));

        if let Err(block) = context
            .create_database_owner(WorldGameDatabaseOwner::RsPlayer)
            .await
        {
            stop!(WorldGameInitBlockReason::Context(block));
        }
        events.push(WorldGameInitEvent::DatabaseOwnerCreated(
            WorldGameDatabaseOwner::RsPlayer,
        ));
        let loaded_setup_ids = match context.create_rs_setup_owner().await {
            Ok(ids) => ids,
            Err(block) => stop!(WorldGameInitBlockReason::Context(block)),
        };
        self.apply_loaded_setup_ids(loaded_setup_ids);
        events.push(WorldGameInitEvent::RsSetupOwnerCreated(loaded_setup_ids));

        const DATABASE_OWNERS_BEFORE_GOODS_WAR: &[WorldGameDatabaseOwner] = &[
            WorldGameDatabaseOwner::RsGenVar,
            WorldGameDatabaseOwner::RsFaction,
            WorldGameDatabaseOwner::RsUnion,
            WorldGameDatabaseOwner::RsEnemyFactions,
            WorldGameDatabaseOwner::RsVillageWar,
            WorldGameDatabaseOwner::RsCityWar,
            WorldGameDatabaseOwner::RsRegion,
            WorldGameDatabaseOwner::DbCountry,
        ];
        for &owner in DATABASE_OWNERS_BEFORE_GOODS_WAR {
            if let Err(block) = context.create_database_owner(owner).await {
                stop!(WorldGameInitBlockReason::Context(block));
            }
            events.push(WorldGameInitEvent::DatabaseOwnerCreated(owner));
        }

        // constructor ловил DB/COM error внутри `reInitDB`: owner
        // оставался опубликованным, а CGame::Init продолжал следующий шаг.
        // Замена прежнего Rust owner-а повторяет `new`; старый owner штатно
        // освобождается Drop вместо исходной утечки при повторном Init.
        if let Err(block) = context
            .create_database_owner(WorldGameDatabaseOwner::GoodsWarMember)
            .await
        {
            stop!(WorldGameInitBlockReason::Context(block));
        }
        *goods_war = CGoodsWarMember::with_reached_empty_state();
        goods_war.begin_lifecycle();
        let goods_war_report = goods_war
            .reinitialize_database(context.goods_war_database_connection())
            .await;
        events.push(WorldGameInitEvent::DatabaseOwnerCreated(
            WorldGameDatabaseOwner::GoodsWarMember,
        ));
        events.push(WorldGameInitEvent::GoodsWarMemberLoaded(goods_war_report));

        const DATABASE_OWNERS_AFTER_GOODS_WAR: &[WorldGameDatabaseOwner] = &[
            WorldGameDatabaseOwner::DbMisc,
            WorldGameDatabaseOwner::RsGodsBattle,
        ];
        for &owner in DATABASE_OWNERS_AFTER_GOODS_WAR {
            if let Err(block) = context.create_database_owner(owner).await {
                stop!(WorldGameInitBlockReason::Context(block));
            }
            events.push(WorldGameInitEvent::DatabaseOwnerCreated(owner));
        }

        const INITIAL_RELOADS: &[&[u8]] = &[
            b"Allthing",
            b"PlayerList",
            b"GoodsList",
            b"MonsterList",
            b"PreciousBoxConf",
            b"FairyExpConf",
            b"TradeList",
            b"IncrementShopList",
            b"ContributeSetup",
            b"SkillList",
            b"GlobeSetup",
            b"LogSystem",
            b"ScriptFile",
            b"GMList",
            b"RegionList",
            b"NewSkillMonsterList",
            b"SynthesisList",
            b"EquipmentCompose",
            b"DaKongXiangQian",
            b"GoodsDestroyConf",
            b"HonorElimilate",
        ];
        for &profile in INITIAL_RELOADS {
            let legacy_result = match self
                .reload(
                    reload_context,
                    jjc,
                    gods_battle,
                    skills,
                    context.gods_battle_database(),
                    profile,
                    false,
                    false,
                )
                .await
            {
                Ok(result) => result,
                Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let region_parameters_loaded = context.load_region_parameters(self).await;
        events.push(WorldGameInitEvent::RegionParametersLoaded {
            succeeded: region_parameters_loaded,
        });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if region_parameters_loaded {
                b"load region tax rate from db...OK!"
            } else {
                b"load region tax rate from db...FAILED!"
            },
        );
        const SECONDARY_RELOADS: &[&[u8]] = &[
            b"RegionLevelSetup",
            b"HitLevelSetup",
            b"Help",
            b"Broadcast",
            b"PrisonConf",
            b"ciqing",
            b"taozhuang",
        ];
        for &profile in SECONDARY_RELOADS {
            let legacy_result = if profile == b"Broadcast" {
                let source = reload_context.read_resource(b"setup/sysboardcast.ini");
                let mut random = |upper_bound| context.random(upper_bound);
                let loaded = self.reload_system_broadcasts(
                    source.as_deref(),
                    &mut random,
                    &mut *callbacks.get_tick,
                );
                reload_context.add_log_text(if loaded {
                    b"Load sysboardcast.ini...OK!"
                } else {
                    b"Load sysboardcast.ini...FAILED!"
                });
                i32::from(loaded)
            } else {
                match self
                    .reload(
                        reload_context,
                        jjc,
                        gods_battle,
                        skills,
                        context.gods_battle_database(),
                        profile,
                        false,
                        false,
                    )
                    .await
                {
                    Ok(result) => result,
                    Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
                }
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let time_to_return_source = reload_context.read_resource(b"setup/TimeToReturn.ini");
        let time_to_return_initialization = match time_to_return.initialize(
            time_to_return_source.as_deref(),
            (callbacks.get_timer_local_time)(),
            timer,
            time_to_return_callbacks,
        ) {
            Ok(report) => report,
            Err(TimeToReturnLoadError::ResourceMissing) => {
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"setup/TimeToReturn.ini can't found!",
                );
                TimeToReturnLoadReport::default()
            }
            Err(source) => {
                let owner = WorldGameInitBooleanOwner::InitializeTimeToReturn;
                events.push(WorldGameInitEvent::BooleanOwner {
                    owner,
                    succeeded: false,
                });
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"Load CityRetern Timing FAILED...",
                );
                stop!(WorldGameInitBlockReason::TimeToReturnLoad(source));
            }
        };
        events.push(WorldGameInitEvent::BooleanOwner {
            owner: WorldGameInitBooleanOwner::InitializeTimeToReturn,
            succeeded: true,
        });
        events.push(WorldGameInitEvent::TimeToReturnInitialized(
            time_to_return_initialization,
        ));

        const INVALID_STRINGS: &[u8] = b"setup/InvalidStr.ini";
        const CHAR_CODES: &[u8] = b"setup/charcode.ini";
        let invalid_strings = reload_context.read_resource(INVALID_STRINGS);
        let char_codes = invalid_strings
            .as_ref()
            .and_then(|_| reload_context.read_resource(CHAR_CODES));
        let _ = self.content_catalogs.words_filter.initial(
            INVALID_STRINGS,
            CHAR_CODES,
            invalid_strings.as_deref(),
            char_codes.as_deref(),
        );
        events.push(WorldGameInitEvent::WordsFilterInitialized);
        for &profile in &[
            b"BattleFairyExpConfig".as_slice(),
            b"BattleFairyCombineConfig",
        ] {
            let legacy_result = match self
                .reload(
                    reload_context,
                    jjc,
                    gods_battle,
                    skills,
                    context.gods_battle_database(),
                    profile,
                    false,
                    false,
                )
                .await
            {
                Ok(result) => result,
                Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let organizing_parameters_now = (callbacks.get_timer_local_time)();
        let organizing_parameters_load = match organizing_parameters.initialize(
            runtime_directory,
            organizing_parameters_now,
            timer,
            organizing_tax_callback,
        ) {
            Ok(report) => report,
            Err(source) => {
                if matches!(&source, OrganizingParamLoadError::Open { .. }) {
                    Self::record_game_init_notice(
                        &mut events,
                        context,
                        b"ERROR",
                        b"file 'data/FactionParam.ini' can't found!",
                    );
                }
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"Load OrganizingParam FAILED...",
                );
                stop!(WorldGameInitBlockReason::OrganizingParameters(source));
            }
        };
        events.push(WorldGameInitEvent::OrganizingParametersLoaded(
            organizing_parameters_load,
        ));

        let legacy_result = match self
            .reload(
                reload_context,
                jjc,
                gods_battle,
                skills,
                context.gods_battle_database(),
                b"godsBattle",
                false,
                false,
            )
            .await
        {
            Ok(result) => result,
            Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
        };
        events.push(WorldGameInitEvent::Reload {
            profile: b"godsBattle",
            legacy_result,
        });
        let rs_gods_battle = context
            .gods_battle_database()
            .expect("успешный godsBattle reload проверил DB-owner");
        let succeeded = rs_gods_battle.load_faction_xyd(gods_battle).await;
        events.push(WorldGameInitEvent::GodsBattleFactionXydLoaded { succeeded });

        let attack_city_now = (callbacks.get_timer_local_time)();
        let attack_city_source = reload_context.read_resource(b"setup/CityWarSys.ini");
        let attack_city_initialization = match attack_city.initialize(
            attack_city_source.as_deref(),
            attack_city_now,
            timer,
            attack_city_callbacks,
        ) {
            Ok(report) => report,
            Err(source) => {
                let owner = WorldGameInitBooleanOwner::InitializeAttackCity;
                events.push(WorldGameInitEvent::BooleanOwner {
                    owner,
                    succeeded: false,
                });
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"Load setup/CityWarSys.ini FAILED...",
                );
                stop!(WorldGameInitBlockReason::AttackCityLoad(source));
            }
        };
        events.push(WorldGameInitEvent::BooleanOwner {
            owner: WorldGameInitBooleanOwner::InitializeAttackCity,
            succeeded: true,
        });
        events.push(WorldGameInitEvent::AttackCityInitialized(
            attack_city_initialization,
        ));
        let mut attack_city_context = WorldGameInitAttackCityContext {
            game: self,
            organizing,
        };
        let attack_city_relations = match attack_city
            .initial_city_all_faction_enemy_relation(&mut attack_city_context)
        {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::AttackCityEnemyRelation(source)),
        };
        events.push(WorldGameInitEvent::AttackCityEnemyRelationsInitialized(
            attack_city_relations,
        ));

        const FOUR_NATION_WAR_PATH: &[u8] = b"setup/FourNationWarSys.ini";
        let four_nation_country_names = reload_context.four_nation_country_names();
        let four_nation_source = reload_context.read_resource(FOUR_NATION_WAR_PATH);
        let four_nation_initialization = match four_nation_war.initialize(
            four_nation_country_names,
            four_nation_source.as_deref(),
            (callbacks.get_timer_local_time)(),
            timer,
            four_nation_war_callbacks,
            |region_id| {
                let path = format!("regions/{region_id}.nation");
                reload_context.read_resource(path.as_bytes())
            },
            |payload| self.record_game_init_log(&mut events, log, callbacks, payload),
        ) {
            Ok(report) => report,
            Err(source) => {
                let owner = WorldGameInitBooleanOwner::InitializeFourNationWar;
                events.push(WorldGameInitEvent::BooleanOwner {
                    owner,
                    succeeded: false,
                });
                let localized = self.get_string_by_id(b"XBWS0021").to_vec();
                self.record_game_init_log(&mut events, log, callbacks, &localized);
                stop!(WorldGameInitBlockReason::FourNationWarLoad(source));
            }
        };
        events.push(WorldGameInitEvent::BooleanOwner {
            owner: WorldGameInitBooleanOwner::InitializeFourNationWar,
            succeeded: true,
        });
        events.push(WorldGameInitEvent::FourNationWarInitialized(
            four_nation_initialization,
        ));

        let village_war_source = reload_context.read_resource(b"setup/villageWarSys.ini");
        let village_war_initialization = match village_war.initialize(
            village_war_source.as_deref(),
            (callbacks.get_timer_local_time)(),
            timer,
            village_war_callbacks,
        ) {
            Ok(report) => report,
            Err(source) => {
                let owner = WorldGameInitBooleanOwner::InitializeVillageWar;
                events.push(WorldGameInitEvent::BooleanOwner {
                    owner,
                    succeeded: false,
                });
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"Load setup/villageWarSys.ini failed!",
                );
                stop!(WorldGameInitBlockReason::VillageWarLoad(source));
            }
        };
        events.push(WorldGameInitEvent::BooleanOwner {
            owner: WorldGameInitBooleanOwner::InitializeVillageWar,
            succeeded: true,
        });
        events.push(WorldGameInitEvent::VillageWarInitialized(
            village_war_initialization,
        ));

        let union_master_title = self.get_string_by_id(b"WS0154").to_vec();
        let faction_master_title = self.get_string_by_id(b"WS0157").to_vec();
        let organizing_now = (callbacks.get_timer_local_time)();
        let organizing_initialize = {
            let (union_database, faction_database) = context.organizing_databases();
            let mut add_log_text = |payload: &[u8]| {
                self.record_game_init_log(&mut events, log, callbacks, payload);
            };
            organizing
                .initialize_from_database_owners(
                    union_database,
                    faction_database,
                    &union_master_title,
                    &faction_master_title,
                    self,
                    organizing_parameters,
                    organizing_now,
                    timer,
                    organizing_tax_callback,
                    &mut add_log_text,
                )
                .await
        };
        match organizing_initialize {
            Ok(report) => events.push(WorldGameInitEvent::OrganizingControllerInitialized(report)),
            Err(source) => stop!(WorldGameInitBlockReason::OrganizingController(source)),
        }
        let region_ids = self.region_registry.regions.keys().copied().collect::<Vec<_>>();
        let mut players_to_refresh = Vec::new();
        for region_id in region_ids {
            let Some(mut region_owner) = self
                .region_registry.regions
                .get_mut(&region_id)
                .and_then(|assignment| assignment.region.take())
            else {
                continue;
            };
            let relation = {
                let mut bridge = RegionOwnerOrganizingBridge {
                    organizing,
                    game: &*self,
                };
                region_owner.base_mut().init_owner_relation(
                    &mut bridge,
                    &mut |player_id| players_to_refresh.push(player_id),
                )
            };
            self.region_registry.regions
                .get_mut(&region_id)
                .expect("region-map key не удаляется во время owner relation")
                .region = Some(region_owner);
            let report = match relation {
                Ok(report) => report,
                Err(source) => stop!(WorldGameInitBlockReason::RegionOwnerRelation {
                    region_id,
                    source,
                }),
            };
            events.push(WorldGameInitEvent::RegionOwnerRelationInitialized {
                region_id,
                report,
            });
        }
        for player_id in players_to_refresh {
            let _ = self.update_player_faction_info(organizing, player_id);
        }

        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            b"Load FactionWarSys:EnemyFactions...",
        );
        faction_war.clear_enemy_factions_for_load();
        let faction_war_load = context.enemy_factions_database().load_all_enemy_factions().await;
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            b"Load FactionWarSys:EnemyFactions SUCCESS...",
        );
        let faction_war_ini_source = reload_context.read_resource(b"data/FactionWarSys.ini");
        let faction_war_initialization = faction_war.initialize_from_loaded_relations(
            faction_war_load,
            faction_war_ini_source.as_deref(),
            |first_faction_id, second_faction_id| {
                self.apply_loaded_enemy_faction_relation(
                    organizing,
                    first_faction_id,
                    second_faction_id,
                )
            },
        );
        let faction_war_initialization = match faction_war_initialization {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::FactionWar(source)),
        };
        if matches!(
            faction_war_initialization.ini.completion,
            FactionWarIniLoadCompletion::FileMissing
        ) {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"data/FactionWarSys.txt can't found!",
            );
        }
        events.push(WorldGameInitEvent::FactionWarInitialized(
            faction_war_initialization,
        ));
        let quest_system = self.initialize_quest_system(reload_context);
        events.push(WorldGameInitEvent::QuestSystemInitialized(quest_system));

        let player_ranks_configuration = PlayerRanksInitializationConfig {
            stat_time: organizing_parameters.stat_player_ranks_time(),
            maximum_count: organizing_parameters.player_ranks_count(),
        };
        let player_ranks_now = (callbacks.get_timer_local_time)();
        let player_ranks_initialization = match player_ranks.initialize(
            player_ranks_configuration,
            player_ranks_now,
            timer,
            player_ranks_callback,
        ) {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::PlayerRanksSchedule(source)),
        };
        events.push(WorldGameInitEvent::PlayerRanksInitialized(
            player_ranks_initialization,
        ));
        let player_ranks_stat = {
            let (database, active_transaction) = context.player_database();
            self.stat_player_ranks(
                player_ranks,
                database,
                active_transaction,
                organizing,
                log,
                &mut *callbacks.get_tick,
                &mut *callbacks.get_log_local_time,
                &mut *callbacks.put_log_info,
            )
            .await
        };
        let player_ranks_stat = match player_ranks_stat {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::PlayerRanksStat(source)),
        };
        events.push(WorldGameInitEvent::PlayerRanksLoaded(player_ranks_stat));

        let country_parameter_source = reload_context.read_resource(b"data/CountryParam.ini");
        let country_parameter_report = match country_parameters
            .initialize(country_parameter_source.as_deref())
        {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::CountryParameters(source)),
        };
        events.push(WorldGameInitEvent::CountryParametersLoaded(
            country_parameter_report,
        ));

        if reload_context.globe_setup().use_appellation_function() {
            let _unused_system_time = (callbacks.get_log_local_time)();
            let started_at_ms = (callbacks.get_tick)();
            self.record_game_init_log(&mut events, log, callbacks, b"Start total HonorRankks!");
            let outcome = {
                let (database, active_transaction) = context.player_database();
                honor_ranks
                    .load_honor_ranks(database, active_transaction)
                    .await
            };
            let finished_at_ms = (callbacks.get_tick)();
            let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
            let complete = format!(
                "Total today HonorRankks complete,consume time {} millisecond!",
                elapsed_ms as i32,
            )
            .into_bytes();
            self.record_game_init_log(&mut events, log, callbacks, &complete);
            events.push(WorldGameInitEvent::HonorRanksLoaded {
                started_at_ms,
                finished_at_ms,
                elapsed_ms,
                outcome,
            });
        }

        let country_local_time = (callbacks.get_timer_local_time)();
        let country_initialization = {
            let (globe_setup, _) = reload_context.globe_setup_and_router();
            let mut country_context = WorldCountryExileResultEffects {
                game: self,
                globe_setup,
            };
            let (country_database, country_database_connection) = context.country_database();
            country_handler
                .initialize(
                    i32::from(country_local_time.day),
                    country_database,
                    country_database_connection,
                    country_parameters,
                    &mut country_context,
                )
                .await
        };
        let succeeded = country_initialization.legacy_result;
        events.push(WorldGameInitEvent::CountryHandlerInitialized(
            country_initialization,
        ));
        if !succeeded {
            stop!(WorldGameInitBlockReason::CountryHandler);
        }
        self.record_game_init_log(&mut events, log, callbacks, b"Load Country SUCCESS...");

        let country_war_source = reload_context.read_resource(b"setup/CountryWarSys.ini");
        let country_war_now = (callbacks.get_timer_local_time)();
        let country_war_initialization = country_war_system.initialize(
            country_war_source.as_deref(),
            country_war_now,
            timer,
            country_war_callbacks,
            |payload| {
                let disposition = log.add_log_text(
                    payload,
                    self.setup.save_info_time_ms,
                    &mut *callbacks.get_tick,
                    &mut *callbacks.get_log_local_time,
                    &mut *callbacks.put_log_info,
                );
                events.push(WorldGameInitEvent::Log {
                    payload: payload.to_vec(),
                    disposition,
                });
            },
        );
        let country_war_initialization = match country_war_initialization {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::CountryWarLoad(source)),
        };
        let succeeded = country_war_initialization.legacy_result;
        events.push(WorldGameInitEvent::CountryWarInitialized(
            country_war_initialization,
        ));
        if !succeeded {
            stop!(WorldGameInitBlockReason::CountryWar);
        }

        // Init публикует новый owner до config/DB loading. Drop заменяет
        // предварительный delete и не сохраняет его dangling-lifetime риск.
        *general_variables = Some(CVariableList::default());
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::CreateGeneralVariableList,
        ));
        let general_variable_source = reload_context.read_resource(b"data/general_variable.ini");
        let general_variable_load = general_variables
            .as_mut()
            .expect("owner опубликован перед LoadVarList")
            .load_var_list(general_variable_source.as_deref());
        events.push(WorldGameInitEvent::GeneralVariableListLoaded(
            general_variable_load,
        ));
        let general_variable_data_load = general_variables
            .as_mut()
            .expect("owner опубликован перед LoadVarData")
            .load_var_data(context.general_variable_database())
            .await;
        // `LoadVarData` был void: исходный Init не ветвился по bool Load.
        events.push(WorldGameInitEvent::GeneralVariableDataLoaded(
            general_variable_data_load,
        ));

        let increment_log_days = reload_context.globe_setup().increment_log_days();
        let outcome = increment_log
            .load(context.increment_log_database(), increment_log_days)
            .await;
        let succeeded = outcome.succeeded();
        events.push(WorldGameInitEvent::IncrementLogLoaded { outcome });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if succeeded {
                b"Load IncShop Log SUCCESS..."
            } else {
                b"Load IncShop Log FAILED..."
            },
        );

        let outcome = auction_log
            .load_item(context.auction_log_database(), increment_log_days)
            .await;
        let succeeded = matches!(&outcome, AuctionLogLoadOutcome::ReturnedTrue);
        events.push(WorldGameInitEvent::AuctionLogLoaded { outcome });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if succeeded {
                b"load Auction Log SUCCESS!"
            } else {
                b"load auctionLog FAILED!"
            },
        );

        for owner in [
            WorldGameInitVoidOwner::InitializeBaseMessage,
            WorldGameInitVoidOwner::InitializeSocket,
        ] {
            // Rust message/socket owners не требуют отдельного глобального
            // initialize-вызова; их живые transport-owner-ы создаются ниже.
            events.push(WorldGameInitEvent::VoidOwner(owner));
        }

        match self.init_net_client().await {
            Ok(initialized) => {
                events.push(WorldGameInitEvent::NetworkClientInitialized(initialized))
            }
            Err(error) => {
                Self::record_game_init_notice(
                    &mut events,
                    context,
                    b"Message",
                    b"Can't connect to LoginServer, please run LoginServer first!",
                );
                stop!(WorldGameInitBlockReason::NetworkClient(error));
            }
        }
        match self.init_net_server() {
            Ok(()) => events.push(WorldGameInitEvent::NetworkServerInitialized),
            Err(error) => {
                Self::record_game_init_notice(
                    &mut events,
                    context,
                    b"Message",
                    b"Can't init NetServer!",
                );
                stop!(WorldGameInitBlockReason::NetworkServer(error));
            }
        }

        self.player_data_queue.clear();
        events.push(WorldGameInitEvent::PlayerDataQueueCleared);
        let copy_number_schedule = match copy_number_timer.register(
            (callbacks.get_timer_local_time)(),
            timer,
            copy_number_callback,
        ) {
            Ok(schedule) => schedule,
            Err(error) => stop!(WorldGameInitBlockReason::CopyNumberSchedule(error)),
        };
        events.push(WorldGameInitEvent::CopyNumberResetScheduled(
            copy_number_schedule,
        ));

        let kind = WorldGameInitWorkerKind::WriteLog;
        let handle = match WorldWriteLogWorker::start(
            self.write_log_worker_spec(),
            context.write_log_worker_runtime(),
        ) {
            Ok(worker) => {
                self.write_log_worker = Some(worker);
                WorldGameInitWorkerHandleState::Open
            }
            Err(error) => {
                context.report_worker_spawn_error(kind, &error);
                self.write_log_worker = None;
                WorldGameInitWorkerHandleState::Empty
            }
        };
        events.push(WorldGameInitEvent::WorkerStarted { kind, handle });
        for worker_index in 0..player_load_thread_count {
            let kind = WorldGameInitWorkerKind::LoadPlayerData { worker_index };
            let (runtime, database, load_largess, get_tick) =
                context.player_load_worker_runtime(worker_index);
            let worker_spec = self.player_load_worker_spec();
            let (handle, error) = self.player_load_workers.start(
                worker_spec,
                worker_index,
                runtime,
                database,
                load_largess,
                get_tick,
            );
            if let Some(error) = error.as_ref() {
                context.report_worker_spawn_error(kind, error);
            }
            events.push(WorldGameInitEvent::WorkerStarted { kind, handle });
        }

        Ok(WorldGameInitReport {
            events,
            legacy_result: 1,
        })
    }

    pub(crate) fn save_city_region<SaveRegion>(
        &mut self,
        selector: i32,
        mut save_region: SaveRegion,
    ) -> Result<Vec<i32>, (Vec<i32>, WorldSaveCityRegionBlock)>
    where
        SaveRegion: FnMut(i32, &mut WorldRegionOwner),
    {
        if selector != 0 {
            return Ok(Vec::new());
        }

        let region_ids = self.region_registry.regions.keys().copied().collect::<Vec<_>>();
        let mut saved = Vec::new();
        for region_id in region_ids {
            let assignment = self
                .region_registry.regions
                .get_mut(&region_id)
                .expect("region ID взят из текущего map");
            let Some(region_type) = assignment.region_type else {
                return Err((
                    saved,
                    WorldSaveCityRegionBlock::UninitializedRegionType { region_id },
                ));
            };
            if region_type != 2 {
                continue;
            }
            let Some(region) = assignment.region.as_mut() else {
                // WorldServer разыменовывал `pRegion` без null-check.
                return Err((
                    saved,
                    WorldSaveCityRegionBlock::NullCityRegion { region_id },
                ));
            };
            save_region(region_id, region);
            saved.push(region_id);
        }
        Ok(saved)
    }

    pub(crate) fn clear_map_player(&mut self) -> usize {
        let entries = self.player_registry.players.len();
        while let Some((_player_id, player)) = self.player_registry.players.pop_first() {
            drop(player);
        }
        entries
    }

    /// Выполняет полный `CGame::Release` до legacy result `1`.
    /// Concrete Goods War owner освобождается между CityWar и RsRegion, как
    /// pointer-owner, но его Rust collections использует обычный Drop.
    pub fn release<Context: WorldGameReleaseContext>(
        &mut self,
        context: &mut Context,
        goods_war: &mut CGoodsWarMember,
        increment_log: &mut CIncrementLog,
        skills: &mut CSkillFactory,
    ) -> WorldGameReleaseResult {
        let mut events = Vec::new();

        context.put_debug_string(b"WorldServer Exiting...");
        events.push(WorldGameReleaseEvent::DebugPublished(
            b"WorldServer Exiting...",
        ));
        self.player_data_queue.clear();
        events.push(WorldGameReleaseEvent::PlayerDataQueueCleared);

        let saved_city_regions = match self.save_city_region(0, |region_id, region| {
            context.save_city_region(region_id, region);
        }) {
            Ok(saved) => saved,
            Err((saved, block)) => {
                for region_id in saved {
                    events.push(WorldGameReleaseEvent::CityRegionSaved { region_id });
                }
                return Err(Box::new(WorldGameReleaseBlock { events, block }));
            }
        };
        for region_id in saved_city_regions {
            events.push(WorldGameReleaseEvent::CityRegionSaved { region_id });
        }

        if let Some(server) = self.net_server.as_mut() {
            context.exit_network_server_worker(server);
            events.push(WorldGameReleaseEvent::NetworkServerWorkerExited);
        }
        if let Some(client) = self.net_client.as_mut() {
            context.exit_network_client_worker(client);
            events.push(WorldGameReleaseEvent::NetworkClientWorkerExited);
        }

        macro_rules! clear_live_list {
            ($list:expr, $owner:expr) => {{
                let entries = $list.len();
                $list.clear();
                events.push(WorldGameReleaseEvent::LiveListCleared {
                    owner: $owner,
                    entries,
                });
            }};
        }
        clear_live_list!(self.player_registry.creation_players, WorldGameReleaseLiveList::Creation);
        clear_live_list!(self.player_registry.restore_players, WorldGameReleaseLiveList::Restore);
        clear_live_list!(self.player_registry.deletion_players, WorldGameReleaseLiveList::Deletion);
        clear_live_list!(self.player_registry.online_players, WorldGameReleaseLiveList::Online);
        clear_live_list!(self.player_registry.offline_players, WorldGameReleaseLiveList::Offline);
        clear_live_list!(self.player_registry.login_players, WorldGameReleaseLiveList::Login);

        let entries = self.clear_map_player();
        events.push(WorldGameReleaseEvent::PlayerMapCleared { entries });
        self.clear_db_data();
        events.push(WorldGameReleaseEvent::DbDataCleared);

        let previous_handle = context.join_save_worker();
        events.push(WorldGameReleaseEvent::SaveWorkerJoined { previous_handle });
        self.clear_release_only_db_snapshots();

        context.release_void_owner(WorldGameReleaseVoidOwner::ReleaseGoodsLinks);
        self.goods_links.clear();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseGoodsLinks,
        ));

        let region_ids = self.region_registry.regions.keys().copied().collect::<Vec<_>>();
        for region_id in region_ids {
            let Some(region) = self
                .region_registry.regions
                .get_mut(&region_id)
                .and_then(|assignment| assignment.region.take())
            else {
                continue;
            };
            drop(region);
            events.push(WorldGameReleaseEvent::RegionOwnerReleased { region_id });
        }
        // `ClearRegionList` удаляет сначала каждый `pRegion`, затем освобождает
        // узлы самой map. После этого места normal Release больше не читает
        // region registry, поэтому clear устраняет только внутреннее удержание
        // пустых Rust map-node до немедленного `DeleteGame`.
        self.region_registry.regions.clear();

        let scripts_released = self.content_catalogs.script_resources.clear();
        for (owner, released) in [
            (
                WorldGameReleaseOptionalOwner::FunctionListFileData,
                scripts_released.functions,
            ),
            (
                WorldGameReleaseOptionalOwner::VariableListFileData,
                scripts_released.variables,
            ),
            (
                WorldGameReleaseOptionalOwner::ScriptFileData,
                scripts_released.scripts,
            ),
        ] {
            events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });
        }
        // После этого места нет ни одного team lookup до немедленного
        // `DeleteGame`, поэтому Rust освобождает только пустые map-node, не
        // меняя session ID, routing либо внешний порядок.
        self.team_sessions.clear();
        let owner = WorldGameReleaseOptionalOwner::GeneralVariableList;
        let released = context.release_optional_owner(owner);
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });

        self.content_catalogs.increment_shop_list.release();
        for owner in [WorldGameReleaseVoidOwner::UninitializeTimeToReturn] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        increment_log.uninitialize();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::UninitializeIncrementLog,
        ));
        context.release_void_owner(WorldGameReleaseVoidOwner::ReleaseCountryHandler);
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseCountryHandler,
        ));
        self.content_catalogs.words_filter.clear();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseWordsFilter,
        ));
        for owner in [
            WorldGameReleaseVoidOwner::ReleaseOrganizingController,
            WorldGameReleaseVoidOwner::ReleaseAttackCity,
            WorldGameReleaseVoidOwner::ReleaseVillageWar,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        let organizing_parameters = context.release_organizing_parameters();
        events.push(WorldGameReleaseEvent::OrganizingParametersReleased(
            organizing_parameters,
        ));
        self.content_catalogs.quest_system.clear();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseQuestSystem,
        ));
        context.release_void_owner(WorldGameReleaseVoidOwner::ReleaseFactionWar);
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseFactionWar,
        ));
        let player_ranks = context.release_player_ranks();
        events.push(WorldGameReleaseEvent::PlayerRanksReleased(player_ranks));
        let owner = WorldGameReleaseVoidOwner::ReleaseTimer;
        context.release_void_owner(owner);
        events.push(WorldGameReleaseEvent::VoidOwner(owner));

        // Background reconnect может владеть только producer FIFO, но перед
        // разрушением transport-owner-а он обязан завершиться. Это устраняет
        // внутренний dangling-lifetime старого process-global worker-а; sleep
        // не прерывается, как у его обычного stop/join owner-а.
        let _connect_login_completion = self.stop_connect_login_thread();

        if let Some(client) = self.net_client.take() {
            drop(client);
            events.push(WorldGameReleaseEvent::NetworkClientReleased);
        }
        if let Some(server) = self.net_server.take() {
            drop(server);
            events.push(WorldGameReleaseEvent::NetworkServerReleased);
        }

        for owner in [
            WorldGameReleaseDatabaseOwner::RsPlayer,
            WorldGameReleaseDatabaseOwner::RsSetup,
            WorldGameReleaseDatabaseOwner::RsGenVar,
            WorldGameReleaseDatabaseOwner::RsFaction,
            WorldGameReleaseDatabaseOwner::RsUnion,
            WorldGameReleaseDatabaseOwner::RsEnemyFactions,
            WorldGameReleaseDatabaseOwner::RsVillageWar,
            WorldGameReleaseDatabaseOwner::RsCityWar,
        ] {
            let released = context.release_database_owner(owner);
            events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        }
        let owner = WorldGameReleaseDatabaseOwner::GoodsWarMember;
        let released = goods_war.release_lifecycle();
        events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        for owner in [
            WorldGameReleaseDatabaseOwner::RsRegion,
            WorldGameReleaseDatabaseOwner::DbCountry,
            WorldGameReleaseDatabaseOwner::RsGodsBattle,
        ] {
            let released = context.release_database_owner(owner);
            events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        }
        // WorldServer создаёт `m_pRsMisc` в Init, но не удаляет
        // и не обнуляет его ни в одном Release call-site до skill-cache cleanup.
        events.push(WorldGameReleaseEvent::DatabaseMiscRetained);

        skills.clear_skill_cache();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ClearSkillCache,
        ));
        skills.clear_usage_cache();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ClearSkillUsageCache,
        ));
        let owner = WorldGameReleaseVoidOwner::ReleaseGoodsFactory;
        context.release_void_owner(owner);
        events.push(WorldGameReleaseEvent::VoidOwner(owner));

        // `db_data` и save serialization являются Rust owners; после этой
        // позиции Release к ним больше не обращается, фактический Drop — DeleteGame.
        events.push(WorldGameReleaseEvent::RustLocksRetired);
        for owner in [
            WorldGameReleaseVoidOwner::CleanupSocket,
            WorldGameReleaseVoidOwner::ReleaseBaseMessage,
            WorldGameReleaseVoidOwner::ReleaseNetSessionManager,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        if let Some(worker) = self.write_log_worker.as_ref() {
            worker.request_exit();
        }
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::RequestWriteLogWorkerExit,
        ));

        let previous_handle = match self.write_log_worker.take() {
            Some(mut worker) => {
                let _completion = worker.join();
                WorldGameInitWorkerHandleState::Open
            }
            None => WorldGameInitWorkerHandleState::Empty,
        };
        events.push(WorldGameReleaseEvent::WriteLogWorkerJoined { previous_handle });
        let workers = self.player_load_workers.stop().len() as u32;
        events.push(WorldGameReleaseEvent::PlayerLoadWorkersStopped { workers });

        for owner in [
            WorldGameReleaseVoidOwner::UninitializeLargess,
            WorldGameReleaseVoidOwner::UninitializeDatabaseLayer,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        let owner = WorldGameReleaseOptionalOwner::DefaultClientResource;
        let released = context.release_optional_owner(owner);
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });
        let owner = WorldGameReleaseOptionalOwner::DupliRegionSetup;
        let released = self.content_catalogs.dupli_region_setup.take().is_some();
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });

        context.put_debug_string(b"WorldServer Exited!");
        events.push(WorldGameReleaseEvent::DebugPublished(
            b"WorldServer Exited!",
        ));
        Ok(WorldGameReleaseReport {
            events,
            legacy_result: 1,
        })
    }

    /// Создаёт и публикует World listener-owner, затем применяет setup.
    ///
    /// Ошибка `Host` оставляет новый owner опубликованным. Отсутствующее позднее
    /// поле возвращается только после успешного listen и hostname-resolution,
    /// то есть уже выполненные исходные побочные эффекты не откатываются.
    pub fn init_net_server(&mut self) -> Result<(), WorldNetworkInitializationError> {
        let server = CMyNetServer::new(legacy_tick_ms());
        let listen_port =
            self.setup
                .listen_port
                .ok_or(WorldNetworkInitializationError::MissingSetupField(
                    "dwListenPort",
                ))?;

        // публиковал s_pNetServer до проверки результата Host.
        self.net_server = Some(server);
        let server = self
            .net_server
            .as_mut()
            .expect("World network owner только что опубликован");
        server
            .host(listen_port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(WorldNetworkInitializationError::Host)?;

        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }

        let config = self.setup.network_config_after_host()?;
        server.configure_after_host(
            config.check_net,
            config.maximum_io_sends,
            config.maximum_byte_count,
            config.maximum_connections,
            config.check_message_content,
            config.ban_ip_time_ms,
            config.maximum_message_length,
            config.maximum_client_send_buffer,
        );
        Ok(())
    }

    /// Пересоздаёт initial World-to-Login client и ставит регистрацию мира.
    ///
    /// Bind, resolution либо connect error выполняют исходный close/delete и
    /// оставляют `net_client = None`. Если безопасная граница `dwNumber`
    /// достигается уже после connect, опубликованный подключённый owner и
    /// включённый control-send не откатываются.
    pub async fn init_net_client(
        &mut self,
    ) -> Result<WorldClientInitialization, WorldClientInitializationError> {
        self.net_client.take();

        // записывал s_pNetClient до Create(0, 0) и Connect.
        self.net_client = Some(CMyNetClient::new());
        let socket = bind_tcp_ipv4(None, 0);
        let login_port =
            self.setup
                .login_port
                .ok_or(WorldClientInitializationError::MissingSetupField(
                    "dwLoginPort",
                ))?;
        let endpoint = resolve_login_endpoint(&self.setup.login_ip, login_port)
            .map_err(WorldClientInitializationError::from);

        let socket = match socket {
            Ok(socket) => socket,
            Err(error) => {
                self.close_and_remove_net_client();
                return Err(WorldClientInitializationError::Bind(error));
            }
        };
        let endpoint = match endpoint {
            Ok(endpoint) => endpoint,
            Err(error) => {
                self.close_and_remove_net_client();
                return Err(error);
            }
        };

        let connect_result = self
            .net_client
            .as_mut()
            .expect("World client owner только что опубликован")
            .connect(socket, endpoint)
            .await;
        if let Err(error) = connect_result {
            self.close_and_remove_net_client();
            return Err(WorldClientInitializationError::Connect(error));
        }

        self.net_client
            .as_mut()
            .expect("успешно подключённый World client остаётся опубликованным")
            .enable_control_send();

        // При успешно открытом, но оборванном до первой
        // пары setup исходный `dwNumber` не инициализирован. Уже выполненные
        // connect/control-send не откатываем и неизвестный DWORD не выбираем.
        let world_number =
            self.setup
                .world_number
                .ok_or(WorldClientInitializationError::MissingSetupField(
                    "dwNumber",
                ))?;
        let mut registration = CMessage::new(0x0001_FE01);
        registration.base_mut().add_ulong(world_number);
        add_legacy_c_string(registration.base_mut(), &self.setup.name);
        let registration = registration.send(
            self.net_client.as_ref().map(CMyNetClient::send_queue),
            false,
        );

        Ok(WorldClientInitialization {
            endpoint,
            registration,
        })
    }

    /// Подключает новый LoginServer client и передаёт его World FIFO.
    ///
    /// Текущий `net_client` остаётся неизменным. Новый owner не получает
    /// control-send: его включит только связанный обработка typed handoff в
    /// исходной позиции `0x3FC03` после замены и постановки регистрации.
    pub async fn reconnect_login_server(
        &self,
    ) -> Result<WorldLoginReconnect, WorldLoginReconnectError> {
        self.login_reconnect_spec().reconnect_once().await
    }

    /// Отделяет ровно те данные, которые свободный reconnect-worker читал из
    /// process-global `g_pGame`: login endpoint и producer World FIFO.
    pub(crate) fn login_reconnect_spec(&self) -> WorldLoginReconnectSpec {
        WorldLoginReconnectSpec {
            login_ip: self.setup.login_ip.clone(),
            login_port: self.setup.login_port,
            event_sender: self.net_server.as_ref().map(CMyNetServer::event_sender),
        }
    }

    /// Выполняет stop/wait/start owner вместо Win32 thread handle.
    ///
    /// Если старый worker спит, stop не будит его: `WaitForSingleObject` также
    /// ждал завершения полного `Sleep(8000)`. Ошибка создания не оставляет
    /// выдуманный handle и сообщается caller-у отдельным typed итогом.
    pub fn create_connect_login_thread(
        &mut self,
        runtime: tokio::runtime::Handle,
    ) -> WorldLoginReconnectThreadRestart {
        let previous_completion = self
            .connect_login_worker
            .as_mut()
            .and_then(WorldLoginReconnectWorker::stop);
        self.connect_login_worker = None;

        let started = match WorldLoginReconnectWorker::start(
            self.login_reconnect_spec(),
            runtime,
        ) {
            Ok(worker) => {
                self.connect_login_worker = Some(worker);
                WorldLoginReconnectThreadStart::Started
            }
            Err(error) => WorldLoginReconnectThreadStart::SpawnFailed(error),
        };

        WorldLoginReconnectThreadRestart {
            previous_completion,
            started,
        }
    }

    pub fn stop_connect_login_thread(
        &mut self,
    ) -> Option<WorldLoginReconnectWorkerCompletion> {
        let completion = self
            .connect_login_worker
            .as_mut()
            .and_then(WorldLoginReconnectWorker::stop);
        self.connect_login_worker = None;
        completion
    }

    /// Выполняет точный retry-loop свободного `ConnectLoginServerFunc`.
    ///
    /// После начального stop-check каждая попытка всегда следует за полной
    /// восьмисекундной паузой. Stop, пришедший во время паузы, намеренно не
    /// отменяет следующую попытку: EXE проверял флаг только после её failure.
    /// `ReConnectLoginServer`-ошибки остаются внутренней причиной следующего
    /// retry и не получают нового observable error mapping.
    pub async fn connect_login_server_func(
        &mut self,
        connect_thread_exit: &AtomicBool,
    ) -> WorldLoginReconnectWorkerOutcome {
        if connect_thread_exit.load(Ordering::Relaxed) {
            return WorldLoginReconnectWorkerOutcome::StoppedBeforeRetry;
        }

        let mut attempts = 0_u32;
        loop {
            tokio::time::sleep(Duration::from_secs(8)).await;
            attempts = attempts.wrapping_add(1);
            if let Ok(reconnect) = self.reconnect_login_server().await {
                return WorldLoginReconnectWorkerOutcome::Reconnected {
                    attempts,
                    reconnect,
                };
            }
            if connect_thread_exit.load(Ordering::Relaxed) {
                return WorldLoginReconnectWorkerOutcome::StoppedAfterFailedRetry { attempts };
            }
        }
    }

    /// Ставит LoginServer полный snapshot аккаунтов в порядке online-list.
    ///
    /// `Ok(None)` буквально соответствует nullable `s_pNetClient` и не создаёт
    /// сообщения. Ошибки безопасной границы возникают до единственного send;
    /// уже собранный локальный payload при этом, как и старый stack-owner, не
    /// становится наблюдаемым соседним процессом.
    pub fn send_cdkey_to_login_server(
        &self,
    ) -> Result<Option<WorldCdkeySnapshot>, WorldCdkeySnapshotError> {
        let Some(client) = self.net_client.as_ref() else {
            return Ok(None);
        };
        let world_number = self
            .setup
            .world_number
            .ok_or(WorldCdkeySnapshotError::MissingWorldNumber)?;
        let declared_online_players = u32::try_from(self.player_registry.online_players.len()).map_err(|_| {
            WorldCdkeySnapshotError::OnlinePlayerCountOutsideLegacyRange {
                count: self.player_registry.online_players.len(),
            }
        })?;

        let mut snapshot = CMessage::new(0x0001_FE02);
        snapshot.base_mut().add_ulong(world_number);
        snapshot.base_mut().add_ulong(declared_online_players);
        for &player_id in &self.player_registry.online_players {
            // World owner выполнял
            // чтение поля по смещению +0x744 через найденный объект; при отсутствии
            // записи указатель оставался нулевым.
            // Достижимость/реакция null-dereference не определена; safe Rust не
            // отправляет частичный snapshot и не выдаёт эту ошибку за legacy.
            let player = self
                .player_registry.players
                .get(&player_id)
                .ok_or(WorldCdkeySnapshotError::MissingPlayerOwner { player_id })?;
            add_legacy_c_string(snapshot.base_mut(), player.get_account());
        }
        let delivery = snapshot.send(Some(client.send_queue()), true);

        Ok(Some(WorldCdkeySnapshot {
            declared_online_players,
            delivery,
        }))
    }

    /// Выполняет один точный drain/process batch фонового DB-load worker-а.
    ///
    /// Несколько worker-ов конкурируют только за атомарный drain очереди; один
    /// победитель последовательно обрабатывает весь полученный FIFO-list.
    pub async fn process_player_load_batch<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        get_tick: GetTick,
    ) -> Result<WorldPlayerLoadBatchReport, WorldPlayerLoadBatchBlock>
    where
        Loader: WorldPlayerDataLoadOwner<CPlayer> + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        self.player_load_worker_spec()
            .process_batch(worker_index, loader, load_largess, get_tick)
            .await
    }

    pub async fn run_player_load_worker<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        game_thread_exit: &AtomicBool,
        player_load_threads_exit: &AtomicBool,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        get_tick: GetTick,
    ) -> Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>
    where
        Loader: WorldPlayerDataLoadOwner<CPlayer> + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        self.player_load_worker_spec()
            .run(
                worker_index,
                game_thread_exit,
                player_load_threads_exit,
                loader,
                load_largess,
                get_tick,
            )
            .await
    }

    pub(crate) fn close_and_remove_net_client(&mut self) {
        if let Some(client) = self.net_client.as_mut() {
            let _legacy_result = client.close();
        }
        self.net_client = None;
    }

}
