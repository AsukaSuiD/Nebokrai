//! Узкий game-view для обработчиков мировых сообщений Realm.
//!
//! Server-волна добавила reconnect/ping/route делегации диспетчера
//! [`on_server_message`](crate::app::servermessage::on_server_message) и шов
//! [`WorldServerMessageGameView`] с ассоциированными типами владельцев, которые
//! пока остаются в старом пакете (организации, страна, save-пайплайн).

use std::future::Future;
use std::pin::Pin;

use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{CGodsBattleConf, GlobeSetupSnapshot};

use crate::activities::jjcsystem::CJJcSystem;
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::app::loginreconnectworker::WorldLoginReconnectThreadRestart;
use crate::app::servermessage::WorldCompletedSaveResponseLaunchReport;
use crate::app::baitan::WorldBaiTanRemoval;
use crate::app::gmmessage::{WorldNamedRegionLookup, WorldRegionIdRouteScan};
use crate::app::world_client::CMyNetClient;
use crate::app::world_message::{CMessage, SendMessageError, WorldLocalMessageQueueBlock};
use crate::app::worldserver::{
    WorldCdkeySnapshot, WorldCdkeySnapshotError, WorldGameServerLookupError,
    WorldGenerateDbDataBlock, WorldGlobeVariablesDelivery, WorldOnlinePlayerAppendOutcome,
    WorldOnlinePlayerRemoveOutcome, WorldPingGameServerInfo, WorldReceivedPlayerDataRead,
    WorldReceivedPlayerDataUpdate,
    WorldReconnectedPlayerDecode, WorldRegionChangePlayerTransition, WorldRegionChangeTeamUpdate,
    WorldRegionParamDecodeOutcome, WorldPlayerSaveResponseProgress, WorldReloadContext,
    WorldReloadResult, WorldServerSnapshotPlayerDecode,
};
use crate::app::worldothermessage::{
    WorldGoodsLink, WorldHonorEliminatorRegistration, WorldPlayerNameChangeReport,
    WorldPlayerNameLookupError,
};
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::writelog::WorldWriteLogCommand;
use crate::app::player_base::WorldPlayerBaseGameView;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{
    CPlayer, PlayerBaseWireSnapshot, PlayerCodecError, PlayerDbProjectionBlock,
    PlayerDefaultPropertyBlock, PlayerDefaultPropertyReport, PlayerFactionInfoUpdateBlock,
    PlayerFactionInfoUpdateReport, PlayerMurderCounterReset, PlayerMurderCounterUpdate,
    PlayerOrganizingUpdateError, PlayerOriginEquipmentBlock, PlayerOriginEquipmentOutcome,
    PlayerPropertyCoefficients,
};
use crate::characters::playerexploit::PlayerExploitUpdate;
use crate::content::countryparam::{CCountryParam, CountryParameterUnavailable};
use crate::content::cgoodsfactory::GoodsOriginalNameIndex;
use crate::content::variablelist::CVariableList;
use nebokrai_shared::resources::CPlayerList;
use crate::organizations::country::{
    CountryExileResultContext, CountryExileTimeLookup, CountryGovernanceContextBlock,
    CountryQuestSwitchUpdate, CountryScalarUpdate,
};
use crate::organizations::faction::CFaction;
use crate::organizations::union::UnionFormatArgument;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::skillfactory::CSkillFactory;
use crate::sessions::csessionfactory::CSessionFactory;

/// Трёхсторонний поиск имени региона: отсутствующий ключ карты, null-ячейка
/// owner-а и найденное имя. Свёртка в `Option` потеряла бы отличие
/// `NullRegionPointer` (Blocked ветки) от `RegionNotFound` (пустое имя).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRegionNameLookup<'a> {
    RegionNotFound,
    NullRegionPointer,
    Name(&'a [u8]),
}

/// Snapshot полей login-строки игрока: `_strcmpi` совпадение долгожителям ещё до
/// поставки `worldserver.exe` строки, legacy-порядок map-итерации сохранён.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldLoginAccountPlayer {
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
}

/// Онлайн account маршрут с optional team-form fields; map-owner исходящие
/// states добавляются записями той же backend формы.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldOnlineAccountPlayerRoute {
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
    pub game_server_index: u32,
}

/// Снимок состояния game server-а: наличие коннекта и числовой индекс маршрута.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldGameServerSnapshot {
    pub connected: bool,
    pub index: u32,
}

/// Итог адресной регистрации GameServer ветви `0x5FA01`. Тип перевезён из
/// `game.rs` вместе с швом connect; старый пакет реэкспортирует его для
/// оставшегося адресного поиска.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldGameServerConnectionState {
    pub index: u32,
    pub previous_connected: bool,
}

/// Owned-снимок маршрута региона ветви `0x5FA02`: фильтр `connected` и
/// выбор ip/port остаются у обработчика, как у исходной замыкательной цепочки
/// `get_region_game_server → filter → map`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRegionGameServerRoute {
    pub connected: bool,
    pub index: u32,
    pub ip: Vec<u8>,
    pub port: Option<u32>,
}

/// Результат `CGame::exit_team_player`: три исхода проверки внешней
/// session/team-реквизитация plug identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldLoginTimeoutTeamExit {
    SessionMissingOrNotTeam,
    PlugMissing,
    Exited,
}

/// Исход применения регионального параметра из GameServer-пакета: ячейка
/// карты отсутствует, владелец null-ячейки либо значение применено.
/// Тип перевезён из `game.rs` вместе со швом ветки `0x6012D`; старый пакет
/// реэкспортирует его для оставшегося dispatcher-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRegionParamUpdateOutcome {
    RegionNotFound,
    NullRegionPointer,
    Applied,
}

/// Точки обратного вызова handler-ветвей мира в владельца игры. Реализация
/// живёт у владельца игры (старый `CGame`) и делегирует его inherent-
/// методам; имена намеренно совпадают — inherent priority исключает рекурсию.
pub trait WorldGameView {
    fn current_game_server_sender(&self) -> Option<ServerCommandHandle>;

    fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError>;

    fn publish_team_session(&mut self, team_id: u32, session_id: i32);

    fn remove_team_session(&mut self, team_id: u32);

    fn get_team_session_id(&self, team_id: u32) -> i32;

    fn player_game_server(&self, player_id: i32) -> Option<WorldGameServerSnapshot>;

    /// Индекс маршрута game server-а региона; `None` — региона нет в карте.
    /// `None` отличим от legacy `0`: ветка private-chat выбирает по нему
    /// `PrivateUnavailable`.
    fn region_game_server_index(&self, region_id: i32) -> Option<u32>;

    fn map_player(&self, player_id: u32) -> Option<&CPlayer>;

    fn online_player_id_by_name(&self, name: &[u8]) -> u32;

    fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer>;

    fn configured_world_number(&self) -> Option<u32>;

    fn current_login_client(&self) -> Option<&CMyNetClient>;

    fn game_server_number_by_player_id(&self, player_id: i32) -> i32;

    /// Числовой индекс маршрута региона ветки `player_detail`; `0` при
    /// отсутствии региона или game server-а. Реализация делегирует
    /// одноимённому inherent-методу; сравнение с source map остаётся у
    /// обработчика.
    fn game_server_number_by_region_id(&self, region_id: i32) -> i32;

    fn legacy_tick_ms(&self) -> u32;

    fn set_map_player_jjc_identity(&mut self, player_id: u32, level: u8, jjc_level: u32) -> bool;

    fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool;

    fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer>;

    fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError>;

    fn decode_online_player_lei_ting(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError>;

    fn reset_honor_eliminate_info(&mut self, rank_mask: u32) -> bool;

    fn register_honor_eliminator(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration;

    fn add_goods_link(&mut self, link: WorldGoodsLink) -> u32;

    fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink>;

    fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool;

    fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval;

    fn game_server(&self, index: u32) -> Option<WorldGameServerSnapshot>;

    fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize;

    fn get_string_by_id(&self, string_id: &[u8]) -> &[u8];

    fn map_player_id_by_name(&self, name: &[u8]) -> u32;

    fn named_region_lookup(&self, name: &[u8]) -> WorldNamedRegionLookup;

    fn region_routes_by_owner_id(&self, region_id: i32) -> WorldRegionIdRouteScan;

    fn region_name(&self, region_id: i32) -> WorldRegionNameLookup<'_>;

    fn queue_local_world_message(&self, message: CMessage) -> Result<(), WorldLocalMessageQueueBlock>;

    fn allocate_leave_word_id(&mut self) -> i32;

    /// Обновляет проекцию faction-информации игрока по переданной фракции.
    fn update_player_faction_info_from_faction(
        &self,
        faction: &CFaction,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock>;

    /// Сброс faction-data флага mapped-игрока ветки `player_detail` после
    /// перевода в offline: `true` — owner найден в карте, флаг снят внутри
    /// владельца игры блочным set_c-вызовом.
    fn reset_map_player_faction_data(&self, map_key: u32) -> bool;

    fn online_player_count(&self) -> usize;

    /// In-memory списки `restore_players`/`deletion_players` при restore-role
    /// ходе: заявленные взаимные семантики delete/append сохраняют прежний
    /// порядок и повторный gate; id хранится как u32 на время заявки.
    fn is_restore_player_exist(&self, player_id: u32) -> bool;

    fn delete_restore_player(&mut self, player_id: u32);

    fn deletion_player_time(&self, player_id: u32) -> i32;

    fn delete_deletion_player(&mut self, player_id: u32);

    fn append_restore_player(&mut self, player_id: u32);

    fn append_deletion_player(&mut self, player_id: u32, deletion_time: i32);

    /// Списки login/online/offline при cleanup/disconnect: login-строка
    /// маршрута сохранён как legacy-lookup фиксну БД адресной очередью;
    /// список login и player_load-очередь у старого владельца, взамен login
    /// или SB account offline записей;
    /// team session exit — те же session-factory что и owner world queue.
    fn login_player_by_account(&self, account: &[u8]) -> Option<WorldLoginAccountPlayer>;

    fn remove_login_player(&mut self, player_id: u32) -> bool;

    fn remove_player_load_data(&self, player_id: i32) -> bool;

    fn append_offline_player_id(&mut self, player_id: u32) -> bool;

    /// Снятие offline-записей игрока ветки `player_detail` при повторной
    /// публикации online: ровно retain всех дубликатов id без возврата,
    /// как в исходном владельце.
    fn remove_offline_player(&mut self, player_id: u32);

    fn online_player_route_by_account(&self, account: &[u8]) -> Option<WorldOnlineAccountPlayerRoute>;

    /// Проверки связки player↔cdkey ветки `player_select`: live-карта и
    /// frozen save-map владельца игры. Реализации делегируют одноимённым
    /// inherent-предикатам (`_strcmpi` над c-string префиксами); выбор слоя,
    /// DB-fallback и отказ `0x1C` остаются у обработчика.
    fn validate_player_id_in_cdkey(&self, account: &[u8], player_id: u32) -> bool;

    fn validate_db_player_id_in_cdkey(&self, account: &[u8], player_id: u32) -> bool;

    /// Постановка player-load FIFO ветки `player_select` при полном miss.
    /// Реализация делегирует одноимённый inherent-метод с той же проверкой
    /// cdkey-capacity; `Duplicate` остаётся диагностикой без перепостановки.
    fn push_player_load_request(
        &self,
        account: &[u8],
        player_id: u32,
        client_ip: u32,
    ) -> Result<WorldPlayerLoadRequestOutcome, WorldPlayerLoadRequestBlock>;

    /// Проверки обработчика `create_role`: занятость имени в live-карте и в
    /// двух слоях загруженных DB-данных. Реализации делегируют одноимённым
    /// inherent-методам владельца игры; порядок вызовов остаётся у обработчика.
    fn is_name_exist_in_map_player(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    fn is_name_exist_in_db_creation(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    fn is_name_exist_in_db_data(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    /// Форматирование строки live StringTable владельца игры: реализация
    /// делегирует inherent `CGame::format_world_string` с тем списком
    /// аргументов, который формирует вызывающая сторона шва.
    fn format_world_string(&self, string_id: &[u8], arguments: &[UnionFormatArgument<'_>]) -> Vec<u8>;

    fn check_create_role_name(
        &self,
        name: &mut Vec<u8>,
        allow_short: bool,
        apply_filter: bool,
    ) -> bool;

    fn allocate_player_id(&mut self) -> i32;

    fn creation_player_count_in_cdkey(&mut self, cdkey: &[u8]) -> u8;

    /// Занятость имени игроком в стадии создания: свёртка `Option<&CPlayer>`
    /// inherent-результата в предикат, как делает сам обработчик.
    fn creation_player_by_name(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError>;

    fn exit_team_player(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit;

    fn replace_online_player_silience_time(&mut self, player_id: u32, silience_time: i32) -> Option<i32>;

    /// Единственный владелец перезагрузки мира: реализация делегирует
    /// inherent `CGame::reload` старого пакета. Boxed future, а не `async fn`,
    /// сохраняет объектную безопасность: view потребляется как
    /// `&mut dyn WorldGameView` сессиями и GM-диспетчером.
    #[allow(clippy::type_complexity, reason = "точный вызов inherent reload с теми же аргументами")]
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
    ) -> Pin<Box<dyn Future<Output = WorldReloadResult> + 'a>>;

    /// Переименование игрока с DB-нагрузкой через owner: boxed future по
    /// ADR-0013 поверх inherent `CGame::change_map_player_name`, async-
    /// контракт и порядок DB-эффектов не меняются.
    #[allow(clippy::type_complexity, reason = "точный вызов inherent change_map_player_name с теми же аргументами")]
    fn change_map_player_name<'a>(
        &'a mut self,
        player_id: u32,
        requested_name: Option<&'a [u8]>,
        globe_setup: &'a GlobeSetupSnapshot,
        rs_player: &'a mut (dyn WorldRenameDbView + 'a),
        player_database: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError>> + 'a>>;

    /// Применение регионального параметра ветки `0x6012D`; реализация
    /// делегирует одноимённый inherent-метод с той же трёхисходной свёрткой
    /// карты регионов.
    fn set_region_param_from_game_server(
        &mut self,
        region_id: i32,
        current_tax_rate: i32,
        today_total_tax: u32,
        total_tax: u32,
    ) -> WorldRegionParamUpdateOutcome;

    /// Назначенный LoginServer id; `0` до `assign_login_server_id`.
    fn login_server_id(&self) -> i32;

    /// Legacy words-filter `Check` над c-string префиксом; реализация
    /// делегирует одноимённый inherent-метод владельца игры.
    fn check_invalid_string(&self, value: &mut Vec<u8>, replace: bool) -> bool;

    /// Owned-city фракция региона; `None` — регион не найден либо null-owner.
    fn region_owned_faction_id(&self, region_id: i32) -> Option<i32>;

    /// Страна региона; `None` — регион не найден, null-owner либо страна не
    /// назначена в region_base.
    fn region_country_id(&self, region_id: i32) -> Option<u8>;

    /// Материализованный регион: entry карты есть, и владелец не null.
    fn has_materialized_region(&self, region_id: i32) -> bool;

    /// Индексы подключённых GameServer; inherent-iterator свёрнут в `Vec`
    /// на границе dyn-view.
    fn connected_game_server_indices(&self) -> Vec<i32>;

    /// Сброс murder-counters mapped-игрока; `None` — игрок не в online-list.
    fn reset_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterReset>;

    /// Wrapping-начисление exploit mapped-игроку ветви four-nation `0x6031A`;
    /// `None` — игрок исчез из карты до локального обновления. Реализация
    /// делегирует одноимённый inherent-метод владельца игры.
    fn add_map_player_exploit_wrapping(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate>;

    /// Перезапуск reconnect-worker-а ветви `0x3FC01`: прежний worker
    /// останавливается и забывается до старта нового, исход регистрируется
    /// restart-итогом. Реализация делегирует одноимённый inherent-метод;
    /// tokio-дескриптор текущего runtime снимается обработчиком, как делал
    /// исходный диспетчер.
    fn create_connect_login_thread(
        &mut self,
        runtime: tokio::runtime::Handle,
    ) -> WorldLoginReconnectThreadRestart;

    /// Начало ping-волны ветви `0x4FC01`: флаг in-progress, сброс накопленных
    /// ответов и отметка времени одной мутацией; возвращает число сброшенных
    /// ответов и tick старта.
    fn begin_game_server_ping(&mut self) -> (usize, u32);

    /// Назначение LoginServer id ветви `0x4FC03`; возвращает прежний id.
    fn assign_login_server_id(&mut self, login_server_id: i32) -> i32;

    /// Адресная регистрация GameServer ветви `0x5FA01`: entry с совпавшим
    /// byte-IP помечается connected, неизвестный port блокирует только
    /// совпавший по IP узел. Реализация делегирует одноимённый inherent-метод.
    fn connect_game_server_by_address(
        &mut self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<WorldGameServerConnectionState>, WorldGameServerLookupError>;

    /// Выброс globe-переменных на один socket ветви `0x5FA01`; порядок
    /// значений snapshot-а сохранён у владельца.
    fn send_globe_variables_to_game_server(&self, socket_id: i32) -> WorldGlobeVariablesDelivery;

    /// Наличие entry карты регионов (без различения null-owner) ветви
    /// `0x5FA02`; `has_materialized_region` намеренно не подменяет эту
    /// проверку — исходник спрашивал только присутствие записи.
    fn region_assignment_exists(&self, region_id: i32) -> bool;

    /// Маршрут game server-а региона ветви `0x5FA02` owned-снимком; `None` —
    /// региона нет в карте либо game server-а нет в реестре.
    fn region_game_server_route(&self, region_id: i32) -> Option<WorldRegionGameServerRoute>;

    /// Обновление региона team owner-а после перехода ветви `0x5FA02`; свёртка
    /// session/plug исходов остаётся той же трёхисходной.
    fn set_team_player_owner_region(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        region_id: i32,
    ) -> WorldRegionChangeTeamUpdate;

    /// Decode снимка игрока save-пакетов `0x5FA03`/sync-пакетов `0x5FA09`;
    /// владение созданным owner-ом и сдвиг курсора не меняются.
    fn decord_server_snapshot_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldServerSnapshotPlayerDecode, PlayerCodecError>;

    /// Decode возвращающегося игрока reconnect-цепочки `0x5FA01`; созданный
    /// owner публикуется в карту и offline-list в исходном порядке.
    fn decord_reconnected_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReconnectedPlayerDecode, PlayerCodecError>;

    /// Учёт save-ответа GameServer ветви `0x5FA03`: счётчик ответов растёт
    /// wrapping-арифметикой и сбрасывается в ноль при covered-совпадении с
    /// числом подключённых не-аукционных game server-ов.
    fn record_player_save_response(&mut self, completion_counted: bool) -> WorldPlayerSaveResponseProgress;

    /// Murder-counters online-игрока ветви `0x5FA06`; `None` — онлайн-записи
    /// нет. Реализация делегирует одноимённый inherent-метод.
    fn increment_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterUpdate>;

    /// Decode регионального параметра из GameServer-пакета ветви `0x5FA07`;
    /// трёхисходная свёртка карты сохранена.
    fn decode_region_param_from_game_server(
        &mut self,
        region_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> WorldRegionParamDecodeOutcome;

    /// Счётчики принятых player-data sync-пакетов ветви `0x5FA09`.
    fn reset_received_player_data(&mut self, game_server_index: i32) -> WorldReceivedPlayerDataUpdate;

    fn increment_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate;

    fn received_player_data(&self, game_server_index: i32) -> WorldReceivedPlayerDataRead;

    /// Регистрация ping-ответа ветви `0x5FA0A`; возвращает накопленный размер.
    fn record_game_server_ping(&mut self, response: WorldPingGameServerInfo) -> usize;

    /// Полный snapshot аккаунтов для LoginServer; `Ok(None)` — Login client
    /// не опубликован. Реализация делегирует одноимённый inherent-метод.
    fn send_cdkey_to_login_server(
        &self,
    ) -> Result<Option<WorldCdkeySnapshot>, WorldCdkeySnapshotError>;

    /// Внутрипроцессная замена LoginServer client: прежний закрывается и
    /// уничтожается до публикации нового; возвращает, был ли закрыт прежний.
    fn replace_login_client(&mut self, client: CMyNetClient) -> bool;

    /// dwNumber после успешного CD-key snapshot (panic-контракт исходного
    /// `expect` сохранён у владельца).
    fn world_number_after_cdkey_snapshot(&self) -> u32;

    /// Имя мира из setup для registration-пакета `0x1FE01`.
    fn world_name(&self) -> &[u8];

    /// Изменяемая ссылка на опубликованный LoginServer client (control-send
    /// включение после registration).
    fn current_login_client_mut(&mut self) -> Option<&mut CMyNetClient>;
}

/// Шов server-диспетчера [`crate::app::servermessage::on_server_message`] для
/// цепочек, чьи владельцы пока живут в старом пакете. Organizing-контекст
/// наследуется от [`WorldPlayerBaseGameView`] — единственная реализация
/// `CGame` делегирует одноимённые inherent-методы; порядок внутри цепочек не
/// меняется. Хвост save-волны проходит отдельным швом
/// [`WorldCompletedSaveResponseMaterialization`]: его owners в старом пакете
/// не выразимы методом игры без `&CGame`-контекста.
pub trait WorldServerMessageGameView: WorldPlayerBaseGameView {
    /// Переход online-игрока между GameServer регионами ветви `0x5FA02`:
    /// decode в mapped-owner-а, снятие offline/online записей, organizing exit
    /// и повторная login-постановка остаются одной связной мутацией владельца
    /// игры; `Ok(None)` — online-запись исчезла до decode.
    #[allow(clippy::too_many_arguments, reason = "точная форма inherent-делегации перехода игрока")]
    fn transition_online_player_region(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        requested_player_id: u32,
        target_region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<WorldRegionChangePlayerTransition>, PlayerCodecError>;

    /// Публикация online-записи декодированного reconnect-игрока ветви
    /// `0x5FA01`: push уникального id и organizing enter одним вызовом
    /// владельца игры, как в исходной цепочке `append_online_player_id`.
    fn append_online_player_id(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        player_id: i32,
    ) -> WorldOnlinePlayerAppendOutcome;
}

/// Шов хвоста завершённой save-волны ветви `0x5FA03`: snapshot БД-данных,
/// cleanup live-карт игроков и launch save-потока остаются одной связной
/// операцией владельца игры; уже выполненный snapshot/cleanup не откатывается
/// при отказе launch, handle-state переходит только после успешного request.
/// Реализация живёт в адаптере у dispatcher-а старого пакета и связывает
/// owners, ещё не перенесённые в Realm (faction war, страновая таблица,
/// lifecycle и runtime save-потока); игра, organizing и realm-владельцы
/// приходят параметрами вызова (форма [`WorldDeleteRoleCountryGate`]) —
/// поэтому шов generic по игре, а не dyn.
pub trait WorldCompletedSaveResponseMaterialization<Game: WorldServerMessageGameView + ?Sized> {
    #[allow(clippy::too_many_arguments, reason = "исходный handler повторно обращался к тем же singleton/static владельцам")]
    fn materialize_completed_save_response_snapshot(
        &mut self,
        game: &mut Game,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &mut Game::OrganizingContext,
        coefficients: &PlayerPropertyCoefficients,
        variables: &CVariableList,
        honor_ranks: &mut CHonorRanks,
        gods_battle: &CGodsBattleConf,
    ) -> Result<WorldCompletedSaveResponseLaunchReport, WorldGenerateDbDataBlock>;
}

/// Узкий dyn-заменитель одного DB-запроса переименования. `RsPlayerOwner`
/// с RPITIT-возвратами и generic-методами dyn-несовместим (vtable),
/// поэтому запрос `is_name_exist` публикуется как boxed future по той же
/// форме ADR-0013; реализация живёт у владельца игрока в старом пакете и
/// делегирует `RsPlayerOwner::<CPlayer>::is_name_exist`.
pub trait WorldRenameDbView {
    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>>;
}

/// Узкий dyn-заменитель одного DB-запроса ветви four-nation exploit
/// `0x6031A`: offline-начисление `CSL_PLAYER_ABILITY.Exploit` для игрока вне
/// карты. По той же причине dyn-несовместимости `RsPlayerOwner`, что и у
/// [`WorldRenameDbView`], запрос публикуется boxed future по ADR-0013;
/// реализация живёт у владельца игрока в старом пакете и повторяет исходные
/// текст запроса и порядок bind. В отличие от rename-семейства, активная
/// транзакция обязательна: отсутствие подключения к БД — отдельная ветвь
/// обработчика (`ConnectionUnavailable` с журналом и повторным локальным
/// convert), seam в этом случае не вызывается. Ошибка выполнения свёрнута в
/// строку, как делал исходный диспетчер для `ExecutionFailed`.
pub trait WorldExploitDbView {
    fn add_player_exploit<'a>(
        &'a mut self,
        increment: i32,
        player_id: i32,
        active_transaction: &'a mut WorldTdsClient,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>>;
}

/// Узкий dyn-заменитель трёх DB-запросов ветви удаления роли: страна
/// игрока, дата постановки на удаление и имя для журнала удаления. По той
/// же причине dyn-несовместимости `RsPlayerOwner`, что и у
/// [`WorldRenameDbView`], каждый запрос публикуется boxed future по
/// ADR-0013; реализация живёт у владельца игрока в старом пакете и
/// делегирует `RsPlayerOwner::<CPlayer>` одноимённым методам.
#[allow(clippy::type_complexity, reason = "boxed-формы повторяют параметры owner-методов один к одному")]
pub trait WorldDeleteRoleDbView {
    fn get_player_country_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = u8> + 'a>>;

    fn get_player_deletion_date<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = i32> + 'a>>;

    fn get_player_name_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Vec<u8>> + 'a>>;
}

/// Узкий dyn-заменитель двух DB-запросов ветви создания роли: счётчик
/// персонажей аккаунта и проверка занятости имени в базе. По той же причине
/// dyn-несовместимости `RsPlayerOwner`, что и у [`WorldRenameDbView`], оба
/// запроса публикуются boxed future по ADR-0013; реализация живёт у
/// владельца игрока в старом пакете и делегирует одноимённым методам
/// `RsPlayerOwner::<CPlayer>`.
#[allow(clippy::type_complexity, reason = "boxed-формы повторяют параметры owner-методов один к одному")]
pub trait WorldCreateRoleDbView {
    fn get_player_count_in_cdkey<'a>(
        &'a mut self,
        account: &'a [u8],
        creation_count: u8,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Option<u8>> + 'a>>;

    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>>;
}

#[derive(Debug)]
pub struct WorldOriginGoodsReport {
    pub entries: Vec<PlayerOriginEquipmentOutcome>,
}

#[derive(Debug)]
pub struct WorldOriginGoodsBlock {
    pub origin_index: usize,
    pub source: PlayerOriginEquipmentBlock,
}

/// Отчёт launch-а нового игрока ветки create_role. Все этапы верхнего уровня
/// обработчика переносятся как связная мутация: список creation-ID, вставка
/// в карту, выдача origin goods и снятие wire-снимка выполняет владелец игры,
/// а порядок и решения записываются через этот отчёт.
#[derive(Debug)]
pub struct WorldCreateRoleLaunchSuccess {
    pub player_id: u32,
    pub defaults: PlayerDefaultPropertyReport,
    pub origin_goods: WorldOriginGoodsReport,
    pub snapshot: PlayerBaseWireSnapshot,
}

/// Точная причина отклонения launch-а. Структурные коллизии карты/очереди
/// остаются typed без типа источника: обработчик повторяет их ветви
/// отказа `create_role_blocked` буквально.
#[derive(Debug)]
pub enum WorldCreateRoleLaunchFailure {
    PlayerName(WorldPlayerNameLookupError),
    DefaultProperty(PlayerDefaultPropertyBlock),
    OriginGoods(WorldOriginGoodsBlock),
    AppendDuplicateCreationId,
    AppendExistingMapOwner,
    PublishedPlayerMissing { player_id: u32 },
    Snapshot(PlayerDbProjectionBlock),
    OrganizingName(WorldCreateRoleOrganizingLookupBlock),
}

/// Упрощённый seam-блок организационного lookup-а имени. Realm не содержит
/// `OrganizingNameLookupBlock` старого пакета — вместо переноса всей таблицы
/// контроллера шов отражает только три возможные причины отказа.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCreateRoleOrganizingLookupBlock {
    RequestedNameWouldOverflow,
    NullOwner,
    OwnerNameWouldOverflow,
}

/// Узкий dyn-view живой таблицы организационных имён (фракции/союзы стран).
/// Реализация делегирует `COrganizingCtrl::organizing_by_name(...).map(|m| m.is_some())`
/// адаптером у dispatcher-а старого пакета; сам контроллер не переносится.
pub trait WorldCreateRoleOrganizingView {
    fn name_exists(&self, name: &[u8]) -> Result<bool, WorldCreateRoleOrganizingLookupBlock>;
}

/// Узкий dyn-шов launch-а нового игрока: вся мутация состояния карты и
/// очереди создаваемых игроков остаётся у владельца игры. Реализация живёт
/// на `CGame` старого пакета и повторяет именно исходную последовательность
/// `load_default_property → set_creation_identity → set_creation_service_defaults
/// → allocate_player_id → set_id → add_origin_goods_to_player →
/// append_creation_player → map_player → player_base_wire_snapshot`, с тем
/// же `log`-callback в форме прежнего обработчика. Sequence счётчик ID
/// потребляется в исходной позиции — после применения service defaults и
/// до публикации в карту, поэтому пути отказа до этого шага не тратят ID.
#[allow(clippy::too_many_arguments, reason = "точная форма launch-цепочки обработчика")]
pub trait WorldCreateRoleLaunchGate {
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
    ) -> Result<WorldCreateRoleLaunchSuccess, WorldCreateRoleLaunchFailure>;

    /// Занятость имени организацией: живой список фракций (`COrganizingCtrl`)
    /// пока остаётся в старом пакете, dyn-заменитель `is_name_exit_in_faction`
    /// сохраняет ту же линейку вызова без переноса самой таблицы.
    fn is_name_exit_in_faction(
        &mut self,
        organizing: &dyn WorldCreateRoleOrganizingView,
        name: &[u8],
    ) -> Result<bool, WorldCreateRoleLaunchFailure>;
}

/// Узкий dyn-view страновой таблицы: ровно тот же `get_country(...).is_some()`
/// который делает исходный `CCountryHandler` без переноса самой таблицы.
pub trait WorldCountryView {
    fn country_exists(&self, country: u8) -> bool;
}

/// Узкий dyn-шов мутаций страны ветвей `0x60314`/`0x60315`/`0x60316`
/// диспетчера `OnCountryMessage`: живые объекты `CCountry` и сама таблица
/// `CCountryHandler` остаются в старом пакете, а обработчики в Realm
/// повторяют исходные цепочки `get_country_mut → apply_server_scalar`,
/// `get_country_mut → set_quest_switch` и
/// `get_country → exile_remaining_time` один вызов на цепочку. Реализация
/// живёт в адаптере `CCountryHandler` у dispatcher-а старого пакета по
/// прецеденту `CreateRoleCountryViewAdapter`; свёртка
/// `CountryMissing`/`SelectorIgnored`/`OfficerMissing` и журнал king-лога
/// остаются у обработчика. Съём `legacy_tick_ms` для exile-lookup
/// выполняет сам адаптер в исходной позиции — внутри ветки найденной
/// страны, поэтому момент времени не тратится на отсутствующую страну.
/// Каждый метод возвращает несвёрнутый результат соответствующей цепочки:
/// внешний `Option` — страна отсутствует.
#[allow(clippy::type_complexity, reason = "вложенные формы повторяют исходные цепочки get_country_mut/get_country один к одному")]
pub trait WorldCountryMutGate {
    fn apply_server_scalar(
        &mut self,
        country: u8,
        selector: i8,
        value: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<Option<CountryScalarUpdate>, CountryParameterUnavailable>>;

    fn set_quest_switch(
        &mut self,
        country: u8,
        job: u8,
        enabled: bool,
    ) -> Option<Option<CountryQuestSwitchUpdate>>;

    fn exile_remaining_time(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<CountryExileTimeLookup, CountryParameterUnavailable>>;
}

/// Узкий dyn-шов проверки должности игрока в стране к владельцу страновых
/// состояний: `CCountryHandler` держит live-таблицу, а контекст текста
/// короля получает игру извне — поэтому метод принимает `&dyn WorldGameView`
/// коротким перезаймом от обработчика. Реализация живёт в адаптере у
/// dispatcher-а старого пакета и повторяет исходную цепочку
/// `get_country → has_job` с effects-структурой кода delete-role.
pub trait WorldDeleteRoleCountryGate {
    fn country_has_job(
        &mut self,
        game: &dyn WorldGameView,
        country: u8,
        player_id: i32,
    ) -> bool;
}

/// Причина отказа governance-перехода короля и столицы ветви `0x60138`:
/// slot страны пуст на момент временного изъятия владельца либо сам
/// `CCountry::SetKing` вернул блочный исход своего context-сеанса.
/// Различие сохранено отдельными вариантами, потому что диспетчер отражает
/// их в разные ветви своего context-блока (`MissingCountryOwner` /
/// `CountryGovernance`).
#[derive(Debug)]
pub enum WorldCountryKingGateBlock {
    MissingCountryOwner,
    Governance(CountryGovernanceContextBlock),
}

/// Узкий dyn-шов war-контактов ветви `0x60138` city war result и семейного
/// `reload_attack_city` с живой страновой таблицей: лёгкие проверка присутствия
/// и запись флага войны плюс тяжёлый governance-переход назначения короля и
/// города. Реализация живёт в bridge-адаптере у dispatcher-а старого пакета
/// (`worldserver/game.rs`) и делегирует inherent-вызовам `CCountryHandler`;
/// живые `CCountry` и сама таблица не покидают старый пакет.
///
/// Лёгкие методы свёрнуты ровно до цепочек, которые выполнял прежний
/// результат-контекст: `get_country(...).is_some()` и
/// `get_country_mut → is_warring = value`; lookup региона в страну
/// (`clear_region_country_warring_if_present`) остаётся у war-системы, а
/// slot вне таблицы воспринимается как отсутствующая страна — записи не
/// происходит.
///
/// Тяжёлый `set_country_king_and_city` намеренно НЕ разложен на примитивы:
/// временное изъятие owner-а из таблицы, вызов `CCountry::SetKing` с
/// governance-контекстом самой ветви, запись `city_id` только при успехе и
/// возврат owner-а в slot остаются одной связной операцией в исходном
/// порядке (то же separation of concerns, что у
/// [`WorldDeleteRoleCountryGate`]). Контекст приходит от вызывающей ветви как
/// `&mut dyn CountryExileResultContext`: результат-контекст диспетчера сам
/// реализует этот трейт, и dyn-форма не меняет наблюдаемое поведение
/// (`?Sized`-граница `CCountry::SetKing` допускает trait object без обхода).
pub trait WorldCountryWarGate {
    fn country_exists(&self, country: u8) -> bool;

    /// Записывает `is_warring` стране, если slot занят; возвращает,
    /// была ли страна найдена (запись применена). Прежний контекст игнорировал
    /// отсутствующую страну молча — тот же отказ доступен ветви без `?`.
    fn set_country_warring(&mut self, country: u8, warring: bool) -> bool;

    /// Связный переход «новый король + город-столица» ветви `0x60138`:
    /// `take_country_owner → SetKing → (success) city_id = city →
    /// restore_country_owner`, результат `SetKing` (и только он) определяет
    /// исход, как в исходном результат-контексте.
    fn set_country_king_and_city(
        &mut self,
        country: u8,
        master_id: i32,
        city_region_id: i32,
        context: &mut dyn CountryExileResultContext,
    ) -> Result<(), WorldCountryKingGateBlock>;
}

/// Причина отказа player-data маршрута владельца игры. Тип перевезён из
/// `game.rs` вместе с ветвью select, чтобы typed seam маршрута жил в Realm;
/// старый пакет реэкспортирует его для оставшейся queue-обработки.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerDataQueueRejectReason {
    NullPlayer,
    MissingRegion {
        region_id: i32,
    },
    MissingOrDisconnectedGameServer {
        region_id: i32,
        game_server_index: u32,
    },
}

/// Единичное уведомление о присутствии друга при маршруте загруженного
/// игрока. Тип перевезён из `game.rs` вместе с ветвью select; старый пакет
/// реэкспортирует его для оставшихся producer-ов присутствия.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldFriendPresenceUpdate {
    pub friend_index: usize,
    pub player_id: u32,
    pub online: bool,
    pub target_game_server_index: Option<u32>,
    pub delivery: Option<Result<i32, SendMessageError>>,
}

/// Отклонение постановки player-load FIFO: c-string префикс cdkey не
/// помещается в фиксированную ёмкость записи. Тип перевезён из `game.rs`
/// вместе с ветвью select; старый пакет реэкспортирует.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadRequestBlock {
    pub account_length: usize,
}

/// Итог постановки player-load FIFO: новая запись или уже стоящий дубликат
/// без перепостановки. Тип перевезён из `game.rs` вместе с ветвью select;
/// старый пакет реэкспортирует.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldPlayerLoadRequestOutcome {
    Queued,
    Duplicate,
}

/// Итог одного queue-прохода FIFO загруженных игроков. `initial_size`
/// фиксирует snapshot размера очереди на вход, `null_pops` — повторные pop
/// пустых записей до первой извлечённой либо до исчерпания snapshot;
/// `NoRecord` ставится только при исчерпанном snapshot, `Rejected`/`Accepted` —
/// ровно один раз на вызов. Тип перевезён из `game.rs` вместе с queue-стадией
/// MainLoop; старый пакет реэкспортирует. От формы select
/// [`WorldPlayerSelectRouteOutcome`] тип сознательно отделён: direct-маршрут
/// фиксирует `initial_size`/`null_pops` нулями и не несёт метаданных snapshot.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldProcessPlayerDataQueueOutcome {
    NoRecord {
        initial_size: u32,
        null_pops: u32,
    },
    Rejected {
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        reason: WorldPlayerDataQueueRejectReason,
        login_delivery: Result<i32, SendMessageError>,
    },
    Accepted {
        initial_size: u32,
        null_pops: u32,
        player_id: u32,
        client_ip: u32,
        game_server_index: u32,
        login_delivery: Result<i32, SendMessageError>,
        friend_updates: Vec<WorldFriendPresenceUpdate>,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        replaced_existing_player: bool,
        login_time_ms: u32,
    },
}

/// Блокирующий дефект queue-прохода: незавершённый cdkey в записи очереди,
/// отказ organizing set либо неинициализированный порт game server-а. В отличие
/// от сокращённой формы select ([`WorldPlayerSelectRouteBlock`]) содержит ветку
/// `UnterminatedCdkey` — она ставится только при разборе записи очереди и не
/// достижима чтением самого маршрута. Тип перевезён из `game.rs` вместе с
/// queue-стадией; старый пакет реэкспортирует.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldProcessPlayerDataQueueBlock {
    UnterminatedCdkey,
    Organizing(PlayerOrganizingUpdateError),
    UninitializedGameServerPort { game_server_index: u32 },
}

/// Отказ queue-прохода с метаданными snapshot, как их фиксировал исходный
/// producer: `player_id` — id записи, чтение которой блокировало проход.
/// Тип перевезён из `game.rs` вместе с queue-стадией; старый пакет
/// реэкспортирует.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldProcessPlayerDataQueueError {
    pub initial_size: u32,
    pub null_pops: u32,
    pub player_id: u32,
    pub block: WorldProcessPlayerDataQueueBlock,
}

/// Технический дефект direct-маршрута ветки select: organizing set или
/// неинициализированный порт game server-а. Форма повторяет
/// `WorldProcessPlayerDataQueueBlock` исходного `route_loaded_player` без
/// ветки `UnterminatedCdkey` — она ставится только при разборе записи очереди
/// в `process_player_data_queue`, чтение самого маршрута её не производит.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerSelectRouteBlock {
    Organizing(PlayerOrganizingUpdateError),
    UninitializedGameServerPort { game_server_index: u32 },
}

/// Отказ direct-маршрута ветки select до любого ответа: исходные
/// `initial_size`/`null_pops` фиксированы нулевыми на этом call-site и не
/// дублируются, `player_id` — id клонированного игрока.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerSelectRouteError {
    pub player_id: u32,
    pub block: WorldPlayerSelectRouteBlock,
}

/// Решение direct-маршрута ветки select. Поля повторяют ветви Rejected и
/// Accepted `WorldProcessPlayerDataQueueOutcome` исходного вызова
/// `route_loaded_player` с фиксированными `initial_size = 0`, `null_pops = 0`
/// и Direct-порядком; входные `queue_player_id`/`client_ip` обработчик хранит
/// в своём `request`, поэтому здесь они не дублируются. Организационный исход
/// снятия online-записей (`PlayerExitGameOutcome` у владельца организаций
/// старого пакета) seam сворачивает в число удалённых вхождений: владелец
/// организаций пока не переносится ради одной ветки.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldPlayerSelectRouteOutcome {
    Rejected {
        reason: WorldPlayerDataQueueRejectReason,
        login_delivery: Result<i32, SendMessageError>,
    },
    Accepted {
        game_server_index: u32,
        login_delivery: Result<i32, SendMessageError>,
        friend_updates: Vec<WorldFriendPresenceUpdate>,
        online_removed_occurrences: usize,
        replaced_existing_player: bool,
        login_time_ms: u32,
    },
}

/// Шов direct-маршрута ветки `player_select`: вся связная мутация маршрута
/// (organizing set, отказ либо login-ответ `0x1FF01` + Largess-вызов,
/// publish map/login, оповещение друзей и сброс login-flags) остаётся одним
/// вызовом inherent `route_loaded_player` у владельца игры. Реализация живёт
/// на `CGame` старого пакета, фиксирует исходную форму вызова select-ветки
/// (`initial_size = 0`, `null_pops = 0`, `Direct`) и делегирует без
/// перестановки. Семантика клонирования и organizing-context наследуется от
/// спискового view без второй реализации.
pub trait WorldPlayerSelectGameView: WorldPlayerBaseGameView {
    #[allow(clippy::too_many_arguments, reason = "точная форма route-вызова ветки select")]
    fn route_select_player(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        get_tick: &mut dyn FnMut() -> u32,
    ) -> Result<WorldPlayerSelectRouteOutcome, WorldPlayerSelectRouteError>;
}

/// Шов loaded-queue маршрута стадии MainLoop: та же связная мутация inherent
/// `route_loaded_player` у владельца игры, что у direct-шва select выше, но
/// с исходными `initial_size`/`null_pops` snapshot-а очереди и порядком
/// `LoadedQueue` (player публикуется после friend-loop, а не до него).
/// Реализация живёт на `CGame` старого пакета и делегирует без перестановки;
/// имя совпадает с inherent-методом специально (inherent priority исключает
/// рекурсию). В отличие от select-формы исход сохраняет полные метаданные
/// snapshot [`WorldProcessPlayerDataQueueOutcome`].
pub trait WorldPlayerQueueGameView: WorldPlayerBaseGameView {
    #[allow(clippy::too_many_arguments, reason = "точная форма route-вызова queue-стадии")]
    fn route_loaded_player(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        get_tick: &mut dyn FnMut() -> u32,
    ) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError>;
}

/// Узкий dyn-заменитель двух DB-запросов ветки выбора роли: проверка связки
/// ID↔cdkey в базе и дата постановки на удаление. По той же причине dyn-
/// несовместимости `RsPlayerOwner`, что и у [`WorldRenameDbView`], оба запроса
/// публикуются boxed future по ADR-0013; реализация живёт у владельца игрока
/// в старом пакете и делегирует одноимённым методам `RsPlayerOwner::<CPlayer>`.
#[allow(clippy::type_complexity, reason = "boxed-формы повторяют параметры owner-методов один к одному")]
pub trait WorldPlayerSelectDbView {
    fn validate_player_id_in_cdkey<'a>(
        &'a mut self,
        account: &'a [u8],
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>>;

    fn get_player_deletion_date<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = i32> + 'a>>;
}

/// Исход владения декодированным возвращающимся игроком subtype-`1` ветки
/// player_return. Тип перевезён из `game.rs` вместе с ветвью; организационный
/// исход раннего offline-перехода (`PlayerExitGameOutcome` у владельца
/// организаций старого пакета) seam сворачивает в число удалённых online-
/// вхождений — по форме свёртки маршрута select, владелец организаций
/// ради одной ветки не переносится. Старый пакет реэкспортирует.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldReturnedPlayerDecodeOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
        login_removed: bool,
        online_removed_occurrences: usize,
        offline_inserted: bool,
    },
}

/// Итог subtype-`1` decode ветки player_return. Тип перевезён из `game.rs`
/// вместе с ветвью; старый пакет реэкспортирует.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldReturnedPlayerDecode {
    pub requested_player_id: u32,
    pub decoded_player_id: i32,
    pub owner: WorldReturnedPlayerDecodeOwner,
}

/// Снимок возвращающегося игрока для ветки player_return: account и name
/// хранятся c-string префиксами, friend names собраны в legacy-порядке.
/// Тип перевезён из `game.rs` вместе с ветвью; старый пакет реэкспортирует.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldReturnedPlayerSnapshot {
    pub account: Vec<u8>,
    pub name: Vec<u8>,
    pub level: u8,
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
    pub friend_names: Vec<Vec<u8>>,
}

/// Снимок login-маршрута игрока для ветки player_detail: ключ карты, id
/// owner-а и регион маршрута. Тип перевезён из `game.rs` вместе с ветвью;
/// старый пакет реэкспортирует.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLoginPlayerRouteSnapshot {
    pub map_key: u32,
    pub owner_id: i32,
    pub region_id: i32,
}

/// Общий шов снятия online-записи ветвей player_return и player_detail:
/// реализация делегирует inherent `CGame::remove_online_player`, а
/// организационный исход (`PlayerExitGameOutcome` у владельца организаций
/// старого пакета) seam сворачивает в число удалённых вхождений — по форме
/// свёртки маршрута select.
pub trait WorldOnlinePlayerRemovalView: WorldPlayerBaseGameView {
    fn remove_online_player(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        player_id: u32,
    ) -> usize;
}

/// Шов ветки `player_return` `0x5FB02`: subtype-`1` decode и снимок
/// возвращающегося игрока остаются одним вызовом у владельца игры без
/// перестановки; реализация на `CGame` делегирует одноимённым inherent-
/// методам (pet vector cleanup и ранний offline-переход внутри decode).
pub trait WorldPlayerReturnGameView: WorldOnlinePlayerRemovalView {
    #[allow(clippy::too_many_arguments, reason = "точная форма decode-вызова ветки return")]
    fn decord_returned_player(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReturnedPlayerDecode, PlayerCodecError>;

    fn returned_player_snapshot(&self, player_id: u32) -> Option<WorldReturnedPlayerSnapshot>;
}

/// Шов ветки `player_detail` `0x5FB01`: снимок login-маршрута, serialize
/// полного mapped-игрока и повторная публикация online остаются у владельца
/// игры; исход append свёрнут в флаг вставки (`PlayerEnterGameOutcome`
/// у владельца организаций старого пакета), как свёртка remove выше.
pub trait WorldPlayerDetailGameView: WorldOnlinePlayerRemovalView {
    fn login_player_route_snapshot(&self, player_id: u32) -> Option<WorldLoginPlayerRouteSnapshot>;

    fn encode_map_player_full_snapshot(
        &mut self,
        organizing: &Self::OrganizingContext,
        map_key: u32,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Vec<u8>>, PlayerCodecError>;

    fn append_online_player_id(
        &mut self,
        organizing: &mut Self::OrganizingContext,
        player_id: i32,
    ) -> bool;
}

/// Общий шов organizing-вызова обновления faction-информации игрока ветвей
/// диспетчера: реализация делегирует inherent `CGame::update_player_faction_info`,
/// а organizing-контекст принадлежит владельцу игры — по форме шва
/// [`WorldOnlinePlayerRemovalView`].
pub trait WorldPlayerFactionInfoUpdateView: WorldPlayerBaseGameView {
    fn update_player_faction_info(
        &self,
        organizing: &Self::OrganizingContext,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock>;
}

