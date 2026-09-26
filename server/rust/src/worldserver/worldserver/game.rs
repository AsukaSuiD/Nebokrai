//! Переходный shim старого пакета: главный `CGame` из `worldserver/game.cpp/.h`
//! перенесён в Realm волной C5-C. Тип живёт в `nebokrai_realm::app::world_game`,
//! `Init`/`Release` и net lifecycle — в `world_game_init`, `MainLoop` со
//! stage-функциями — в `world_main_loop`, `ReLoad` со стадиями — в
//! `world_reload`, диспетчер сообщений и country/organizing glue — в
//! `world_dispatch`, save-семья — в `nebokrai_realm::persistence::world_db_data_collect`.
//!
//! Ниже glob-реэкспорт владельца и точечные реэкспорты realm-данных для
//! остающихся потребителей пакета (обвязка `writelogmessage.rs` и соседние
//! модули). Волной C5-D process owners переехали в Realm `app/world_process*`,
//! а `process/worldserver.rs` пользуется realm-входом
//! `app::world_runtime::game_thread_func` напрямую.

// Тип `CGame`, его адаптер загрузки и pub-типы владельца приходят glob-ом;
// иначе потребовались бы ручные alias-строки на сотни имён.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_dispatch::*;
pub(crate) use nebokrai_realm::app::world_game::*;
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_init::*;
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_main_loop::*;
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_reload::*;
// Драйвер потока игры World (`GameThreadFunc` со связкой init/release
// data-типов) живёт в Realm `app::world_runtime`; волной C5-C туда же
// переехал сам `CGame`, волной C5-D — impl context-трейтов драйвера у
// process-owners `app/world_process*`. Ниже переходный реэкспорт для
// старого пакета.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_runtime::{
    PlayerRanksStatRunBlock, PlayerRanksStatRunReport, WorldClientInitialization, WorldClientInitializationError, WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitBlock, WorldGameInitBlockReason, WorldGameInitBooleanOwner, WorldGameInitEvent, WorldGameInitOperatorNotice, WorldGameInitReport, WorldGameInitResult, WorldGameInitVoidOwner, WorldGameInitWorkerKind, WorldGameReleaseBlock, WorldGameReleaseContext, WorldGameReleaseDatabaseOwner, WorldGameReleaseEvent, WorldGameReleaseLiveList, WorldGameReleaseOptionalOwner, WorldGameReleaseReport, WorldGameReleaseResult, WorldGameReleaseVoidOwner, WorldNetworkInitializationError, WorldRegionOwner, WorldSaveCityRegionBlock, WorldServerSetupLoadReport, WorldSetupLoadReport, WorldSetupOpenError, WorldSetupSource, WorldStringTableEncodingBlock,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::WorldGameServerLookupError;

// Hub-данные Init/MainLoop World (config/string-table отчёты, init-callbacks,
// effect-контексты faction-war/attack-city, маршрутизированные сообщения с
// union terminal-событиями и ProcessedWorldEvent, state-структуры и AI/
// session-factory stage-отчёты, route-order загрузки игрока) перенесены в
// Realm `app/world_hub_data` волной C5-A: игра входит у них generic-параметром
// через объявленные швы WorldGameView/WorldPlayerFactionInfoUpdateView, а
// старый пакет специализирует их своим CGame. Здесь реэкспорт для стадий
// хода, selector-а, диспетчеров сообщений и прочих потребителей пакета.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_hub_data::{
    ProcessedWorldEvent, RoutedWorldMessage, WorldCityTransferTerminalDispatch, WorldConfederationCreationTerminalDispatch, WorldGameInitAttackCityContext, WorldGameInitCallbacks, WorldGameInitEnemyMutationEffects, WorldLoadedPlayerRouteOrder, WorldMainLoopAiStageReport, WorldMainLoopClockState, WorldMainLoopFactionWarBlock, WorldMainLoopFactionWarEffects, WorldMainLoopInitializationState, WorldMainLoopLargessState, WorldMainLoopLoginReleaseState, WorldMainLoopProfileState, WorldMainLoopSessionFactoryStageReport, WorldMainLoopTailClockState, WorldMessageOwner, WorldMessageSource, WorldProcessMessageOutcome, WorldProcessMessageStageState, WorldStringTableLoadReport, WorldStringTableUpdateCompletion, WorldStringTableUpdateReport, WorldUnionApplicationRuntimeReport, WorldUnionApplicationTerminalDispatch, WorldUnionInvitationTerminalDispatch,
};

// Init-контракт хода перенесён в Realm `app/world_init_context` волной C5-B:
// CGame-типизированный параметр `load_region_parameters` заменён там готовым
// швом `&mut dyn RegionParameterLoadTarget` (`regions/rsregion.rs`), impl
// перенесён в Realm `app/world_process_init` волной C5-D. Здесь реэкспорт
// для generic-связок init-стадий пакета.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_init_context::WorldGameInitContext;

// Контракты reconnect-семейства LoginServer (итог попытки, snapshot endpoint,
// итог worker-а и restart-итоги) вместе с ошибкой попытки перенесены в Realm
// app к своему worker-у. Здесь реэкспорт для остающихся process-owner связей:
// setup snapshot, lifecycle thread-owner-а и отчёт диспетчера server-сообщений.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::loginreconnectworker::{
    WorldLoginReconnect, WorldLoginReconnectError, WorldLoginReconnectSpec,
    WorldLoginReconnectThreadRestart, WorldLoginReconnectThreadStart,
    WorldLoginReconnectWorkerOutcome,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldCdkeySnapshot, WorldOnlinePlayerAppendOutcome, WorldReconnectedPlayerDecode,
    WorldReconnectedPlayerOwner, WorldServerSnapshotPlayerDecode, WorldServerSnapshotPlayerOwner,
};

// Типы decode/снимков ветвей player_return и player_detail перевезены в
// Realm world_game_view вместе с ветвями; организационный исход online-
// снятия там свёрнут в число удалённых вхождений. Здесь реэкспорт для
// inherent decode-метода и dispatcher-адаптера.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::{
    WorldReturnedPlayerDecode, WorldReturnedPlayerDecodeOwner, WorldReturnedPlayerSnapshot,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldOnlinePlayerRemoveOutcome, WorldPlayerSaveResponseProgress,
};

// Отчёты терминала `on_game_server_lost` перевезены в Realm
// `app::worldserver` вместе со швом ветви `0x3FC02`; здесь реэкспорт для
// inherent-метода.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldGameServerLostReport, WorldLostGameServerPlayer,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldCdkeySnapshotError, WorldRegionChangePlayerTransition,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldothermessage::{
    WorldGoodsLink, WorldGoodsLinkPayload, WorldHonorEliminatorRegistration,
    WorldPlayerNameChangeDisposition, WorldPlayerNameChangeReport, WorldPlayerNameLookupError,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_message::WorldLocalMessageQueueBlock;

// Диагностические типы player-data маршрута и player-load FIFO перевезены в
// Realm world_game_view вместе с ветвью select и queue-стадией MainLoop; здесь
// реэкспорт для inherent маршрута и producer-ов присутствия.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::{
    WorldFriendPresenceUpdate, WorldPlayerDataQueueRejectReason, WorldPlayerLoadRequestBlock,
    WorldPlayerLoadRequestOutcome, WorldProcessPlayerDataQueueBlock,
    WorldProcessPlayerDataQueueError, WorldProcessPlayerDataQueueOutcome,
};

// Отчёт queue-стадии MainLoop перенесён в Realm app вместе со stage-handler-ом;
// здесь реэкспорт для сборки MainLoop report и block-ветки.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::playerdataqueue::WorldMainLoopPlayerDataQueueStageReport;

// Stage-отчёты timer-стадии MainLoop (player-ranks refresh, country/village/
// attack/four-nation war timers и block-семья callback-диспетча) перенесены
// в Realm `app/world_main_loop_data` волной C5-A вместе со связкой хода;
// здесь реэкспорт для handler-а timer-стадии и сборки MainLoop report.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_main_loop_data::{
    AttackCityTimerOutcome, AttackCityTimerReport, CountryWarTimerBlock, CountryWarTimerReport,
    FourNationWarTimerReport, PlayerRanksTimerRefreshBlock, PlayerRanksTimerRefreshReport,
    VillageWarTimerOutcome, VillageWarTimerReport, WorldMainLoopTimerStageBlock,
    WorldMainLoopTimerStageReport, WorldTimerCallbackBlock,
};

// Данные хода MainLoop (stage-отчёты, конфигурация, JJC/LeiTing worker-
// адаптеры и связка StateOwners/Owners/Callbacks/Block/Report) перенесены в
// Realm `app/world_main_loop_data` волной C5-A. rs_player/largess/Game входят
// теперь generic-параметрами: глубокие точки process_* держат прежнюю
// конкретную декларацию до следующей порции, поэтому старый владелец
// закрепляет TiberiusRsPlayer/TiberiusLargess/CGame при вызове хода (см.
// main_loop ниже). Здесь реэкспорт для runtime-сборки и стадий пакета.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_main_loop_data::{
    WorldDbMiscDeliveryContext, WorldLoginTimeoutEntryOutcome,
    WorldLoginTimeoutFriendOutcome, WorldLoginTimeoutReport,
    WorldMainLoopBaiTanJjcStageReport, WorldMainLoopBlock, WorldMainLoopCallbacks,
    WorldMainLoopConfiguration, WorldMainLoopDbMiscStageReport,
    WorldMainLoopFactionWarStageReport, WorldMainLoopMinuteStageBlock,
    WorldMainLoopMinuteStageReport, WorldMainLoopNetSessionStageReport,
    WorldMainLoopOwners, WorldMainLoopPacingReport, WorldMainLoopPingError,
    WorldMainLoopPingStageReport, WorldMainLoopReport, WorldMainLoopResult,
    WorldMainLoopSaveStageDisposition, WorldMainLoopSaveStageReport,
    WorldMainLoopStateOwners, WorldMainLoopTailStageReport, WorldRefreshExternalCounts,
};

// JJC/LeiTing runtime-швы, их worker-мосты и process-impl перенесены в Realm
// `app/world_main_loop_contexts` волной C5-B (там же живут сами
// process-контексты; прежняя посадка impl в `runtime.rs` дала бы
// orphan-нарушение). Здесь реэкспорт для main-loop стадий пакета; мосты
// конструируются через `new`.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_main_loop_contexts::{
    WorldJjcRuntimeContext, WorldJjcWorkerContext, WorldLeiTingRuntimeContext,
    WorldLeiTingWorkerContext,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldLoginTimeoutTeamExit;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::WorldRegionChangeTeamUpdate;

// Refresh/reload-контракт хода (snapshot-gate RefreshInfo, atomic reload-flags
// с таблицей профилей, resource snapshot стадии, maintenance player-ranks/
// honor, снимки профилирования и profile-init helpers, process-message
// stage-типы и count-gate snapshot-стадии) перенесены в Realm
// `app/world_reload_profiles` волной C5-A; reload-диспетчер остаётся у
// владельца хода и пользуется ими через здесь стоящий реэкспорт.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_reload_profiles::{
    WORLD_RELOAD_ACTIONS, WorldAuctionBangMaintenanceDisposition, WorldHonorRanksMaintenanceBlock, WorldHonorRanksMaintenanceDisposition, WorldMainLoopLargessGateReport, WorldMainLoopMaintenanceBlock, WorldMainLoopMaintenanceReport, WorldMainLoopProfileReport, WorldMainLoopProfileSnapshot, WorldMainLoopRefreshDisposition, WorldMainLoopRefreshStageReport, WorldMainLoopResourceContext, WorldMainLoopResourceSnapshot, WorldPlayerRanksMaintenanceDisposition, WorldPlayerRanksRequestState, WorldProcessMessageError, WorldProcessMessageStageReport, WorldRefreshSnapshotBlock, WorldRegionLoadSpec, WorldReloadActionKind, WorldReloadConfLogBlock, WorldReloadConfLogDisposition, WorldReloadOneScriptBlock, WorldReloadOneScriptResult, WorldReloadProfile, WorldReloadProfileEvent, WorldReloadProfileFlags, WorldReloadProfilesReport, WorldReloadRegionSetupBlock, WorldScriptLoadContext, initialize_main_loop_profile_if_needed, initialize_main_loop_refresh_if_needed, initialize_main_loop_save_if_needed, initialize_main_loop_tail_clocks, legacy_refresh_count, start_main_loop_profile_stage, update_main_loop_current_tick,
};

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::app::worldserver::WorldReloadContext;

// `tagSaveCountry`-снимок country-параметров (`current_country_save_limits`)
// перенесён в Realm `app/world_reload_profiles` вместе со stage-типом его
// ошибки (волна C5-A); здесь реэкспорт для process-message стадии.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_reload_profiles::current_country_save_limits;

// Конфигурация `tagSetup` WorldServer и позиционный парсинг plain/encoded
// setup перенесены в Realm `app/world_setup` волной C5-A; токен-обёртка чтения
// serverSetup.ini и разрешение runtime-файла живут там же, здесь реэкспорт
// для владельца игры и loader-стадий пакета.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_setup::{
    WorldServerSetupTokens, WorldSetup, resolve_world_runtime_file,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::WorldPingGameServerInfo;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::baitan::{
    WorldBaiTanCompletion, WorldBaiTanLists, WorldBaiTanRegistration, WorldBaiTanRemoval,
    WorldDoneBaiTanListReport,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldInitialRegionSnapshot, WorldInitialRegionSnapshotBlock, WorldInitialRegionSnapshotKind,
    WorldInitialRegionSnapshotSource,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldRegionListBlock, WorldRegionOwnerLoadBlock, WorldRegionOwnerSerializationBlock,
    WorldReloadBlock, WorldReloadRegionSnapshotBlock, WorldReloadResult,
};

// Записи таблиц состояния World (materialized-регион, системная рассылка и
// её AI-отчёт, x87 money-truncate, записи game/login серверов, origin-отчёты
// и organizing player-контексты) перенесены в Realm `app/world_hub_entries`
// волной C5-A; здесь реэкспорт для владельца таблиц и стадий хода.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_hub_entries::{
    WorldAuctionSellerMoney, WorldDetachedFactionInfoContext,
    WorldFactionPlayerOrganizingContext, WorldGameAiReport, WorldGameServerEntry,
    WorldLoginPlayerEntry, WorldOriginGoodsBlock, WorldOriginGoodsReport,
    WorldPlayerFactionInfoContext, WorldPlayerOrganizingContext, WorldRegionAssignment,
    WorldSystemBroadcast, WorldSystemBroadcastDisposition, WorldSystemBroadcastTarget,
    truncate_legacy_money, truncate_scaled_legacy_money,
};


#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::gmmessage::{WorldNamedRegionLookup, WorldNamedRegionMatch, WorldRegionIdRouteScan, WorldRegionIdRoute};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldRegionNameLookup;

// `WorldRegionParamUpdateOutcome` перевезён в `world_game_view` со швом ветки
// `0x6012D`; здесь реэкспорт для dispatcher-обвязки и outcome-события.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldRegionParamUpdateOutcome;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::WorldRegionParamDecodeOutcome;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldReceivedPlayerDataRead, WorldReceivedPlayerDataUpdate,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldGameServerConnectionState;

// Снимок disconnect-мутации реестра ветви `0x3FC02` перевезён в Realm
// world_game_view вместе с швом; здесь реэкспорт для inherent-метода.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldGameServerDisconnectionState;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{WorldGlobeVariables, WorldGlobeVariablesDelivery};

// Свободный monitoring-owner `SendErrLog` и его delivery-отчёт перенесены
// в Realm `app/worldserver` в monitoring-message волне; здесь реэкспорт
// для inherent facade `CGame::send_err_log` и save thread closure.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldErrorLogDelivery, send_err_log_to_login,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldLoginAccountPlayer;

// Снимок login-маршрута ветки player_detail перевезён в Realm
// world_game_view вместе с ветвью; здесь реэкспорт для inherent snapshot.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldLoginPlayerRouteSnapshot;

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_game_view::WorldOnlineAccountPlayerRoute;

// Data/handle-типы save-batch `tagDBData` (snapshot удалённого игрока,
// accumulator `WorldDbData` с session facade и отделённый batch owner)
// перенесены в Realm persistence вместе с impl session-а. Здесь реэкспорт
// для остающихся inherent точек, save-trigger-а и worker-сборки.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::persistence::savedata::{
    DeletionPlayerSnapshot, WorldDbData, WorldDbDataSaveSession, WorldSaveDataOwner,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{WorldGenerateDbDataBlock, WorldGenerateDbDataReport};

// Frozen-вход системного `SaveThreadFunc` перенесён в Realm
// persistence/savedb вместе с цитируемым им `SaveDataLifecycleState`;
// здесь реэкспорт для save-trigger-а, worker-сборки и worker-связи.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::savedb::WorldSaveThreadJob;

// Worker-вход SaveThreadFunc, его report-тип и динамическая граница между
// game-триггером и process save-owner-ом перенесены в Realm
// `persistence/saveworker` (волна 7 сохраняющего пайплайна); их потребители
// (save-trigger игры и process save-runtime) живут там же с волн C5-C/C5-D.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::saveworker::{
    WorldSaveRuntimeContext, WorldSaveThreadReport,
};

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::{
    WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest, prepare_save_thread_launch,
};

// Save-state и trigger-отчёты хода (collect player-data, ручной запрос,
// pre-gate, launch/notify и терминальный trigger-итог) перенесены в Realm
// `app/world_save_reports` волной C5-A. Локальные alias-формы отчётов потока
// игры сняты волнами C5-C/C5-D вместе с типом игры и process owners.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_save_reports::{
    WorldCollectPlayerDataBroadcast, WorldCollectPlayerDataRequestState,
    WorldManualSaveRequestReport, WorldRunImmediateSaveReport, WorldRunSaveLaunchReport,
    WorldRunSaveTriggerDisposition, WorldRunSaveTriggerState,
    WorldSaveAllOrganizationsLaunchReport, WorldSaveNotifyDelivery, WorldSaveNotifyReport,
};

// Первый локальный IPv4 процесса: форма перенесена в Realm `app::worldserver`
// к техническим helpers process-owner-а; старый пакет получает её реэкспортом.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldserver::resolve_first_local_ipv4;

// Единый владелец tick-формы — Realm `app::misc_game` волны Misc: тело
// идентично прежнему (Boottime-часы), мировой дубликат заменён реэкспортом.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::app::misc_game::legacy_tick_ms;

// Resolution LoginServer endpoint-а общая: initial client-owner и reconnect
// worker используют одну реализацию Realm loginreconnectworker; типовой итог
// ошибки остаётся у process-owner-а.
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::loginreconnectworker::resolve_login_endpoint;
