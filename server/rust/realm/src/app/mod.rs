//! Композиция и сетевой край Realm: сообщения направлений, слушающие порты,
//! места диспетчеризации и действующая realm-оркестрация объединённого
//! Realm-процесса (CGame, MainLoop, драйвер процесса WorldServer) по
//! [карте владельцев]. Старого процессного `realmserver` ещё нет.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod auth_message; // wire-сообщение направления LoginServer->AuthServer.
pub mod auth_server; // принимающий сервер Auth-направления (CMyNetServer_Auth).
pub mod auth_server_client; // состояние принятого Login-соединения Auth-направления.
pub mod billing_message; // wire-сообщение направления GameServer<->BillingServer.
pub mod billing_server; // принимающий сервер Game-соединений Billing (CServerForGS).
pub mod billing_server_client; // состояние принятого Game-соединения у Billing (CClientForGS).
pub mod countrymessage; // country-сообщения 0x603xx: типы ветвей и хвостовой диспетчер World.
pub mod login_auth_client; // исходящий клиент Login->Auth (CMyNetClientAuth).
pub mod login_message; // wire-сообщение трёх направлений LoginServer.
pub mod login_server; // listener игровых клиентов Login-направления.
pub mod login_server_client; // состояние принятого игрового клиента Login-направления.
pub mod login_world_server; // listener World-соединений у Login-направления.
pub mod login_world_server_client; // состояние принятого World-соединения у Login.
pub mod loginreconnectworker; // reconnect worker World->Login: исходный 8-сек cadence, публикация replacement client в World FIFO
pub mod misc_client; // исходящий клиент Misc->World.
pub mod misc_game; // runtime-владелец роли MiscServer: World-соединение, auction room и FIFO.
pub mod misc_message; // wire-сообщение направления MiscServer.
pub mod miscservermessage; // auction-ветви 0x14EDxx MiscServer.
pub mod onbillserver; // служебные ветви Misc: client-close и очистка auction room.
pub mod othermessage; // ответные ветви MiscServer: нулевое 32-битное поле без приоритета.
pub mod setup; // позиционная конфигурация MiscServer из setup.ini.
pub mod auction; // auction-сообщения MSG_S2W_AUCTION у World.
pub mod baitan; // bai-tan реестр CGame: очередь заявок по ip, маршруты игроков и счётчики повторных ip
pub mod gmamessage; // GMA-ветви World: kick-player 0x4FD01 и transport 0x4FD04/0x604xx.
pub mod gmmessage; // GM-диспетчер World и маршруты именованных регионов.
pub mod jjcsysmessage; // входящий JJC-диспетчер и leaf-ветви World.
pub mod logmessage; // login lifecycle-ветви OnLogMessage World.
pub mod onmsg_m2w_auction; // auction-диспетчер MSG_M2W_AUCTION у World.
pub mod organsysmessage; // терминальный слой async confirm organizing-сообщений.
pub mod playermessage; // перенаправления игроков 0x5FC01..0x5FC04 -> 0x7FA08..0x7FA0B.
pub mod player_base; // ответ World->Login со списком персонажей 0x1FF02.
pub mod playerdataqueue; // loaded-queue стадия MainLoop: FIFO загруженных игроков через route-шов hub-владельца
pub mod servermessage; // диспетчер server-сообщений World.
pub mod teammessage; // входящий team-диспетчер 0x600xx World.
pub mod world_client; // исходящий клиент World->Login.
pub mod world_game_view; // узкий game-view обработчиков мировых сообщений.
pub mod world_game; // тип CGame старого WorldServer: объявление, new, hub-таблицы/accessors, timer/effect glue и impl-ы Realm-швов.
pub mod world_game_init; // Init/Release и net init/reconnect CGame: load_setup, ресурсные и DB-владельцы, workers.
pub mod world_dispatch; // process_world_message: диспетчер мировых сообщений, drain union runtime, country/organizing effect-glue и DeleteRole/CreateRole мосты.
pub mod world_hub_data; // hub-данные Init/MainLoop World: сетевая конфигурация, init-callbacks, события диспетча ProcessedWorldEvent с union terminal-семьёй, state-структуры и effect-контексты за view-швами игры.
pub mod world_hub_entries; // записи таблиц состояния World: materialized-регион, системная рассылка и её AI-отчёт, x87 money-truncate, записи game/login серверов, origin-отчёты и organizing player-контексты.
pub mod world_message; // wire-сообщение направлений Login/Game<->World.
pub mod world_init_context; // Init-context World: process DB owners/settings, dbmisc configuration, typed-доставка его событий и runtime-контракт Init с region-load швом dyn RegionParameterLoadTarget (impl у process-owner-а)
pub mod world_main_loop_contexts; // post-init контексты World: JJC/LeiTing platform-glue, runtime-швы с worker-мостами и process-impl, INI-замена, build error MainLoop DB-stage
pub mod world_main_loop_data; // данные хода MainLoop World: stage-отчёты, конфигурация, JJC/LeiTing worker-адаптеры и связка StateOwners/Owners/Callbacks/Block/Report; rs_player/largess/Game входят generic-параметрами.
pub mod world_main_loop; // MainLoop и stage-функции хода CGame: ai, process_message, timer/faction-war/lei-ting/db-misc/net-session/ping/minute/bai-tan/tail/refresh/maintenance стадии, largess/profile.
pub mod world_network; // net-thread прокладка хода World: accept/I/O worker, опрос Login
pub mod world_organizing_view; // узкие organizing-view мировых диспетчеров.
pub mod world_process; // process owners исторического WorldServer: domain owners + initialize_game, состояние MainLoop, network-обвязка хода, post-init DB-контексты и WorldProcessRuntime
pub mod world_process_init; // Init-контекст процесса World: 12 Option DB-owner-ов + Largess Arc + instance guard, impl WorldGameInitContext и player-load БД
pub mod world_process_release; // Release-контекст процесса World и impl-ы драйвера world_runtime для WorldProcessRuntime, включая Game = CGame
pub mod world_process_resources; // ресурсный держатель World (registries + WorldReloadContext); посадка переходная: целевое место content/, пока в app/
pub mod world_process_save; // save-worker процесса World: launch строит 12 Tiberius save-DB owner-ов и зовёт save_thread_func; save-runtime и trigger-guard
pub mod world_reload_profiles; // refresh/reload-контракт хода World: snapshot-gate, atomic reload-flags с таблицей профилей, resource snapshot, maintenance ranks и profile-init helpers.
pub mod world_reload; // ReLoad CGame и reload-стадии: war-семейство, string-table, initial configuration, reload profiles/reload_conf_log (PARTIAL).
pub mod world_runtime; // драйвер потока игры World: CreateGame→Init→turn→Release→DeleteGame + init/release data-bundle; CGame входит assoc-типом и фабрикой, context-impl у process-owner-а
pub mod world_save_reports; // save-state и trigger/notify отчёты сохранения хода World: collect player-data, pre-gate, launch и терминальный trigger-итог.
pub mod world_setup; // позиционная конфигурация tagSetup WorldServer: поля, plain/encoded парсинг и разрешение runtime-файла.
pub mod worldothermessage; // прочие сообщения World 0x5FDxx.
pub mod world_server; // принимающий сервер Game-соединений World.
pub mod world_server_client; // состояние принятого Game-соединения у World.
pub mod worldserver; // процессные helpers исторического WorldServer: operator-log, имя и lifecycle.
pub mod writelogmessage; // write-log диспетчер 0x60201..0x60218: decode → persistence FIFO + live increment/auction publish; game-контакты только map_player и push_write_log_command из WorldGameView
